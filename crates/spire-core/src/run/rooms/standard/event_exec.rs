//! The event executor (design §5.4). Event bodies never mutate `RunState`: they return a closed
//! list of outcomes, which `check` validates before the first mutation and `apply` runs in order
//! through the shared owners. A selection or a combat suspends the list on the frame; the rest
//! runs when it returns (`resume`).
use super::*;
use crate::content::{EncounterId, PotionId, RelicTier};
use crate::items::{self, relic_pools, lifecycle::{RelicObtained, RelicSource}};
use super::event_frame::{self, EventCombat};

/// A relic an outcome obtains: a fixed relic, a fixed tier, or a tier rolled on M27's ordinary
/// table (50/33/17). Event relics are screenless (`RelicSource::Event`, M27).
#[allow(dead_code)] // design §5.4's full vocabulary; later chunks' bodies use the rest.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum RelicChoice { Specific(RelicId), Tier(RelicTier), RandomTier }
#[allow(dead_code)]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum PotionChoice { Specific(PotionId), Random }

/// An event combat's reward screen (design §5.7): the standard `RewardRoom::Event` card offer and
/// potion appearance, plus the event's listed gold and relic rows.
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct RewardSpec { pub gold:u32, pub relic_tiers:Vec<RelicTier> }

#[allow(dead_code)]
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum Outcome {
    GainGold(u32), LoseGold(u32), LoseAllGold,
    Heal(u32), LoseHp(u32), Damage(u32),
    GainMaxHp(u32), LoseMaxHp(u32),
    ObtainCard{id:CardId,upgrades:u32}, ObtainCurse(CardId), ObtainRandomCurse,
    /// A nested card selection (design §5.6) returning to this list.
    Select{op:EventSelectOp,count:u8,skippable:bool},
    TransformRandom(u8), UpgradeRandom(u8), UpgradeAll, RemoveSpecific(u64),
    ObtainRelic(RelicChoice), LoseRelic(RelicId),
    /// Select a relic, pay `gold`, then grant it (The Addict's Pay; PM ruling Q2).
    BuyRelic{choice:RelicChoice,gold:u32},
    ObtainPotion(PotionChoice),
    /// An event combat (design §5.7): `rewards` opens the reward screen after a victory; without
    /// it the frame continues at `next_stage`.
    Combat{encounter:EncounterId,rewards:Option<RewardSpec>,next_stage:u8},
    Key(Key),
    NextStage(u8), Leave, EnterBoss,
}

/// The typed interim for Secret Portal's boss entry (chunk plan §3).
pub(in crate::run) const ENTER_BOSS_INTERIM:&str="M28 Secret Portal boss entry lands in M28f";

/// The candidates of an event selection, one filter per operation (design §5.6): removable and
/// transformable cards are the purgeable cards; upgradeable ones pass `can_upgrade`; any card can
/// be duplicated.
pub(in crate::run) fn select_candidates(state:&RunState,op:EventSelectOp)->Vec<&CardInstance>{
    state.deck.iter().filter(|card|match op{
        EventSelectOp::Remove|EventSelectOp::Transform=>crate::run::rooms::selection::purgeable(state,card),
        EventSelectOp::Upgrade=>mutations::can_upgrade(state,card),
        EventSelectOp::Duplicate=>true,
    }).collect()
}

/// Every precondition of `outcomes`, before any mutation (design §5.4; review focus 1), simulated in
/// order (M28c review M3): gold to pay after earlier gains (Ectoplasm blocks gains), a card to
/// select or remove after the cards earlier outcomes may have taken (worst case: every earlier
/// removal, transform or random upgrade took a candidate of the later selection), an owned relic to
/// lose, an unowned relic to obtain, a free potion slot (Sozu obtains nothing and needs none), and
/// frame control last.
pub(in crate::run) fn check(state:&RunState,outcomes:&[Outcome])->Result<(),RunError>{
    let mut gold=i64::from(state.gold);
    let gains=!state.relics.contains(&RelicId::Ectoplasm);
    let mut taken=0usize;
    let mut free=state.potions.iter().filter(|slot|slot.is_none()).count();
    let sozu=state.relics.contains(&RelicId::Sozu);
    let mut owned=state.relics.clone();
    for (index,outcome) in outcomes.iter().enumerate(){
        let last=index+1==outcomes.len();
        match outcome{
            Outcome::GainGold(n)=>if gains{gold+=i64::from(*n)},
            Outcome::LoseGold(n)|Outcome::BuyRelic{gold:n,..}=>{if gold<i64::from(*n){return Err(RunError::InvalidAction)}gold-=i64::from(*n)}
            Outcome::LoseAllGold=>gold=0,
            Outcome::Heal(_)=>if state.hp>state.max_hp{return Err(RunError::InvalidAction)},
            Outcome::Select{op,count,..}=>{
                if *count==0||select_candidates(state,*op).len().saturating_sub(taken)<usize::from(*count){return Err(RunError::InvalidAction)}
                if *op!=EventSelectOp::Duplicate{taken+=usize::from(*count);}
            }
            Outcome::RemoveSpecific(instance)=>{if !state.deck.iter().any(|card|card.instance==*instance){return Err(RunError::InvalidAction)}taken+=1}
            Outcome::TransformRandom(n)|Outcome::UpgradeRandom(n)=>taken+=usize::from(*n),
            Outcome::ObtainRelic(RelicChoice::Specific(id))=>{if *id!=RelicId::Circlet&&owned.contains(id){return Err(RunError::InvalidAction)}owned.push(*id)}
            Outcome::LoseRelic(id)=>{let Some(i)=owned.iter().position(|r|r==id) else {return Err(RunError::InvalidAction)};owned.remove(i);}
            Outcome::ObtainPotion(_)=>if !sozu{if free==0{return Err(RunError::InvalidAction)}free-=1},
            Outcome::Combat{..}|Outcome::NextStage(_)|Outcome::Leave=>if !last{return Err(RunError::InvalidAction)},
            Outcome::EnterBoss=>return Err(RunError::Unsupported(ENTER_BOSS_INTERIM)),
            _=>{}
        }
    }
    Ok(())
}

fn draw(state:&mut RunState,key:&[u64],bound:usize)->Result<usize,RunError>{
    state.rng.bounded_scoped_checked(RunRngScope::M28EventBody,key,bound as u64)
}

/// `count` distinct cards from `pool`, uniformly, in deck order of the remaining candidates.
fn random_cards(state:&mut RunState,mut pool:Vec<u64>,count:u8,key:&[u64])->Result<Vec<u64>,RunError>{
    let mut chosen=vec![];
    for slot in 0..count{
        if pool.is_empty(){break}
        let key=[key,&[u64::from(slot)]].concat();
        chosen.push(pool.remove(draw(state,&key,pool.len())?));
    }
    Ok(chosen)
}

/// Apply `outcomes` in order. The caller has checked them and stages the Run.
pub(in crate::run) fn apply(run:&mut Run,outcomes:Vec<Outcome>)->Result<RunTransition,RunError>{
    let mut queue:std::collections::VecDeque<_>=outcomes.into();
    let mut obtained:Option<(RelicId,RelicObtained)>=None;
    while let Some(outcome)=queue.pop_front(){
        let key=event_frame::next_key(run)?;
        let state=&mut run.state;
        match outcome{
            Outcome::GainGold(n)=>mutations::gain_gold(state,n)?,
            Outcome::LoseGold(n)=>state.gold=state.gold.checked_sub(n).ok_or(RunError::InvalidAction)?,
            Outcome::LoseAllGold=>state.gold=0,
            Outcome::Heal(n)=>mutations::heal(state,n)?,
            // F2 (PM source check Q4): the shared out-of-combat damage law, rescue included.
            // Event damage has no owner, so Torii never applies to it (Golden Idol, World of Goop).
            Outcome::LoseHp(n)=>if !mutations::lose_hp(state,n,mutations::OutOfCombatLoss::HpLoss){return run.commit_terminal(RunTerminal::Death)},
            Outcome::Damage(n)=>if !mutations::lose_hp(state,n,mutations::OutOfCombatLoss::Damage{owned:false}){return run.commit_terminal(RunTerminal::Death)},
            Outcome::GainMaxHp(n)=>mutations::increase_max_hp(state,n)?,
            Outcome::LoseMaxHp(n)=>mutations::decrease_max_hp(state,n),
            Outcome::ObtainCard{id,upgrades}=>{mutations::obtain_card_or_negated(state,id,upgrades)?;}
            Outcome::ObtainCurse(id)=>{mutations::obtain_card_or_negated(state,id,0)?;}
            Outcome::ObtainRandomCurse=>{
                let id=mutations::draw_card(state,crate::content::pools::CardPool::Curse,&key)?;
                mutations::obtain_card_or_negated(state,id,0)?;
            }
            Outcome::Select{op,count,skippable}=>{
                event_frame::suspend(run,queue.into(),None,skippable)?;
                return event_frame::open_select(run,op,count)
            }
            Outcome::TransformRandom(n)=>{
                let pool=select_candidates(state,EventSelectOp::Transform).iter().map(|c|c.instance).collect();
                for (slot,instance) in random_cards(state,pool,n,&key)?.into_iter().enumerate(){
                    mutations::transform_card(state,instance,false,&[&key[..],&[0xF0,slot as u64]].concat())?;
                }
            }
            Outcome::UpgradeRandom(n)=>{
                let pool=select_candidates(state,EventSelectOp::Upgrade).iter().map(|c|c.instance).collect();
                for instance in random_cards(state,pool,n,&key)?{mutations::upgrade_card(state,instance)?;}
            }
            Outcome::UpgradeAll=>{
                let all:Vec<u64>=select_candidates(state,EventSelectOp::Upgrade).iter().map(|c|c.instance).collect();
                for instance in all{mutations::upgrade_card(state,instance)?;}
            }
            Outcome::RemoveSpecific(instance)=>{mutations::remove_card(state,instance)?;}
            Outcome::ObtainRelic(_)|Outcome::BuyRelic{..}=>{
                let (choice,price)=match outcome{Outcome::BuyRelic{choice,gold}=>(choice,gold),Outcome::ObtainRelic(choice)=>(choice,0),_=>unreachable!()};
                let relic=match choice{
                    RelicChoice::Specific(id)=>id,
                    RelicChoice::Tier(tier)=>relic_pools::take(state,tier,relic_pools::Selector::Front,RelicSource::Event,&key)?,
                    RelicChoice::RandomTier=>{
                        let tier=relic_pools::roll_tier(state,relic_pools::TierTable::Ordinary,&key);
                        relic_pools::take(state,tier,relic_pools::Selector::Front,RelicSource::Event,&key)?
                    }
                };
                // A purchase pays after the relic is selected and before it is granted.
                state.gold=state.gold.checked_sub(price).ok_or(RunError::InvalidAction)?;
                let result=items::obtain_relic(state,relic,RelicSource::Event)?;
                if result.selection.is_some()||!result.offers.is_empty(){
                    // One relic per list may continue into M27's equip screens, after the frame's screen.
                    if obtained.is_some(){return Err(RunError::Unsupported("event obtained two relics with equip screens"))}
                    obtained=Some((relic,result));
                }
            }
            Outcome::LoseRelic(id)=>items::lose_relic(state,id)?,
            Outcome::ObtainPotion(choice)=>{
                let potion=match choice{
                    PotionChoice::Specific(id)=>id,
                    PotionChoice::Random=>crate::items::potion_generation::rarity_first(state.character,|bound|{
                        state.rng.bounded_scoped_checked(RunRngScope::M28EventBody,&key,bound)
                    })?,
                };
                items::obtain_run_potion(state,potion)?;
            }
            Outcome::Combat{encounter,rewards,next_stage}=>{
                event_frame::suspend(run,queue.into(),Some(EventCombat{encounter,rewards,next_stage}),false)?;
                return event_frame::enter_combat(run,encounter)
            }
            Outcome::Key(k)=>{
                let event=mutations::acquire_key(state,k)?;
                reward::commit(state,std::slice::from_ref(&event))?;
            }
            Outcome::NextStage(stage)=>event_frame::set_stage(run,stage)?,
            Outcome::Leave=>{
                if obtained.is_some(){return Err(RunError::Unsupported("event left before its relic's equip screens"))}
                return event_frame::leave(run)
            }
            Outcome::EnterBoss=>return Err(RunError::Unsupported(ENTER_BOSS_INTERIM)),
        }
    }
    event_frame::publish(run)?;
    match obtained{
        Some((relic,result))=>{let choice=event_frame::accepted_choice(run)?;crate::run::rooms::after_obtain(run,choice,relic,result)}
        None=>Ok(empty_transition()),
    }
}
