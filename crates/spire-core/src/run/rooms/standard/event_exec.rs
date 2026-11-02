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
/// potion appearance, plus the event's listed gold and relic rows. `gold_range` (inclusive) is drawn
/// when the fight starts and added to `gold`: the source merges every gold addition into one row
/// (`addGoldToRewards`). Reward relics are ordinary (not screenless) offers.
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct RewardSpec { pub gold:u32, pub gold_range:Option<(u32,u32)>, pub relics:Vec<RelicChoice> }

#[allow(dead_code)]
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) enum Outcome {
    GainGold(u32), LoseGold(u32), LoseAllGold,
    Heal(u32), LoseHp(u32), Damage(u32),
    /// NORMAL damage whose owner is the player (Wing Statue, World of Goop, Shining Light pass the
    /// player; event-neow research §6), so Torii's owned-damage rule can apply.
    PlayerDamage(u32),
    GainMaxHp(u32), LoseMaxHp(u32),
    ObtainCard{id:CardId,upgrades:u32}, ObtainCurse(CardId), ObtainRandomCurse,
    /// Note For Yourself's direct obtain of the stored card (`obtain_stored_card`: no Omamori,
    /// obtain hooks apply; event-neow research :289).
    ObtainStoredNote,
    /// Note For Yourself: the deck card `instance` becomes the stored card (identity and upgrades).
    StoreNote(u64),
    /// A nested card selection (design §5.6) returning to this list.
    Select{op:EventSelectOp,count:u8,skippable:bool},
    TransformRandom(u8), UpgradeRandom(u8), UpgradeAll, RemoveSpecific(u64),
    ObtainRelic(RelicChoice), LoseRelic(RelicId),
    /// Select a relic, pay `gold`, then grant it (The Addict's Pay; PM ruling Q2).
    BuyRelic{choice:RelicChoice,gold:u32},
    ObtainPotion(PotionChoice),
    /// Write `value` to the frame's captured `values[slot]` (per-event progress, e.g. Match and Keep).
    SetValue{slot:u8,value:u32},
    /// A draw `n` in `0..bound` at this point of the list: the first arm with `n < upper` continues
    /// the list (Scrap Ooze, Dead Adventurer, Wheel of Change). Frame control: last in its list, and
    /// every arm is a complete list.
    Roll{bound:u32,arms:Vec<(u32,Vec<Outcome>)>},
    /// An event combat (design §5.7): `rewards` opens the reward screen after a victory; without
    /// it the frame continues at `next_stage`. `elite` is the source's `eliteTrigger` (elite-only
    /// combat effects; the room stays an EventRoom).
    Combat{encounter:EncounterId,rewards:Option<RewardSpec>,next_stage:u8,elite:bool},
    /// A non-combat reward screen (Lab, Wheel of Change): `potions` flat class potions and the relic
    /// rows, no card offer; Proceed finishes the room.
    Rewards{potions:u8,relics:Vec<RelicChoice>},
    Key(Key),
    NextStage(u8), Leave, EnterBoss,
}

/// The typed interim for Secret Portal's boss entry (chunk plan §3).
pub(in crate::run) const ENTER_BOSS_INTERIM:&str="M28 Secret Portal boss entry lands in M28f";

/// The candidates of an event selection, one filter per operation (design §5.6): removable,
/// transformable and chosen (`Choose`: Bonfire, Note) cards are the purgeable cards; upgradeable
/// ones pass `can_upgrade`; any card can be duplicated.
pub(in crate::run) fn select_candidates(state:&RunState,op:EventSelectOp)->Vec<&CardInstance>{
    candidates_in(state,&state.deck,op)
}
fn candidates_in<'a>(state:&RunState,deck:&'a [CardInstance],op:EventSelectOp)->Vec<&'a CardInstance>{
    deck.iter().filter(|card|match op{
        EventSelectOp::Remove|EventSelectOp::Transform|EventSelectOp::Choose=>crate::run::rooms::selection::purgeable(state,card),
        EventSelectOp::Upgrade=>mutations::can_upgrade(state,card),
        EventSelectOp::Duplicate=>true,
    }).collect()
}

/// The simulation `check` runs: the values earlier outcomes change, in order.
#[derive(Clone)]
struct Sim { gold:i64, free:usize, owned:Vec<RelicId>, deck:Option<RunState>, chosen:Vec<(EventSelectOp,usize,Vec<u64>)>, random:Vec<(EventSelectOp,usize)> }

/// Every precondition of `outcomes`, before any mutation (design §5.4; review focus 1), simulated in
/// order (M28c review M3, made exact in M28d for N2): gold to pay after earlier gains (Ectoplasm
/// blocks gains); a card to select after earlier outcomes changed the deck: deterministic card
/// outcomes (`ObtainCard`, `ObtainCurse` with Omamori, `ObtainStoredNote`, `RemoveSpecific`) are
/// applied to a staged copy, and an earlier selection takes from a later one only its forced overlap
/// `max(0, count − |earlier ∖ later|)` (the player chooses the rest elsewhere; an upgrade takes only
/// from a later upgrade); an owned relic to lose, an unowned relic to obtain or buy, a free potion
/// slot (Sozu obtains nothing and needs none); frame control last, and every `Roll` arm complete.
/// Random deck outcomes before a selection keep the worst case; no landed body has that shape
/// (`m28_event_body_shapes_check_exactly`).
pub(in crate::run) fn check(state:&RunState,outcomes:&[Outcome])->Result<(),RunError>{
    let sim=Sim{gold:i64::from(state.gold),free:state.potions.iter().filter(|slot|slot.is_none()).count(),owned:state.relics.clone(),deck:None,chosen:vec![],random:vec![]};
    check_from(state,sim,outcomes)
}

fn takes(earlier:EventSelectOp,later:EventSelectOp)->bool{
    match earlier{
        // A removed or transformed card leaves every later candidate set.
        EventSelectOp::Remove|EventSelectOp::Transform|EventSelectOp::Choose=>true,
        EventSelectOp::Upgrade=>later==EventSelectOp::Upgrade,
        EventSelectOp::Duplicate=>false,
    }
}

fn check_from(state:&RunState,mut sim:Sim,outcomes:&[Outcome])->Result<(),RunError>{
    let gains=!state.relics.contains(&RelicId::Ectoplasm);
    let sozu=state.relics.contains(&RelicId::Sozu);
    let later_select=|index:usize|outcomes[index+1..].iter().any(|o|matches!(o,Outcome::Select{..}|Outcome::Roll{..}));
    for (index,outcome) in outcomes.iter().enumerate(){
        let last=index+1==outcomes.len();
        match outcome{
            Outcome::GainGold(n)=>if gains{sim.gold+=i64::from(*n)},
            Outcome::LoseGold(n)=>{if sim.gold<i64::from(*n){return Err(RunError::InvalidAction)}sim.gold-=i64::from(*n)}
            // N1 (M28c review v2): a purchase of a specific relic needs it unowned, as an obtain does.
            Outcome::BuyRelic{choice,gold:n}=>{
                if sim.gold<i64::from(*n){return Err(RunError::InvalidAction)}
                sim.gold-=i64::from(*n);
                if let RelicChoice::Specific(id)=choice{if *id!=RelicId::Circlet&&sim.owned.contains(id){return Err(RunError::InvalidAction)}sim.owned.push(*id)}
            }
            Outcome::LoseAllGold=>sim.gold=0,
            Outcome::Heal(_)=>if state.hp>state.max_hp{return Err(RunError::InvalidAction)},
            Outcome::Select{op,count,..}=>{
                let deck=sim.deck.as_ref().map_or(&state.deck,|staged|&staged.deck);
                let candidates:Vec<u64>=candidates_in(state,deck,*op).iter().map(|c|c.instance).collect();
                let mut available=candidates.len();
                for (earlier,n,from) in &sim.chosen{
                    if !takes(*earlier,*op){continue}
                    let elsewhere=from.iter().filter(|i|!candidates.contains(i)).count();
                    available=available.saturating_sub(n.saturating_sub(elsewhere));
                }
                for (earlier,n) in &sim.random{if takes(*earlier,*op){available=available.saturating_sub(*n)}}
                if *count==0||available<usize::from(*count){return Err(RunError::InvalidAction)}
                sim.chosen.push((*op,usize::from(*count),candidates));
            }
            Outcome::ObtainCard{..}|Outcome::ObtainCurse(_)|Outcome::ObtainStoredNote|Outcome::RemoveSpecific(_)=>{
                if let Outcome::RemoveSpecific(instance)=outcome{
                    let deck=sim.deck.as_ref().map_or(&state.deck,|staged|&staged.deck);
                    if !deck.iter().any(|card|card.instance==*instance){return Err(RunError::InvalidAction)}
                }
                if later_select(index){
                    let staged=sim.deck.get_or_insert_with(||state.clone());
                    match outcome{
                        Outcome::ObtainCard{id,upgrades}=>{mutations::obtain_card_or_negated(staged,*id,*upgrades)?;}
                        Outcome::ObtainCurse(id)=>{mutations::obtain_card_or_negated(staged,*id,0)?;}
                        Outcome::ObtainStoredNote=>{let (id,upgrades)=staged.stored_note;mutations::obtain_stored_card(staged,id,upgrades)?;}
                        Outcome::RemoveSpecific(instance)=>{mutations::remove_card(staged,*instance)?;}
                        _=>unreachable!("deterministic card outcome"),
                    }
                }
            }
            Outcome::TransformRandom(n)=>sim.random.push((EventSelectOp::Transform,usize::from(*n))),
            Outcome::UpgradeRandom(n)=>sim.random.push((EventSelectOp::Upgrade,usize::from(*n))),
            Outcome::UpgradeAll=>sim.random.push((EventSelectOp::Upgrade,usize::MAX)),
            Outcome::ObtainRelic(RelicChoice::Specific(id))=>{
                if *id!=RelicId::Circlet&&sim.owned.contains(id){return Err(RunError::InvalidAction)}
                sim.owned.push(*id)
            }
            Outcome::LoseRelic(id)=>{let Some(i)=sim.owned.iter().position(|r|r==id) else {return Err(RunError::InvalidAction)};sim.owned.remove(i);}
            Outcome::ObtainPotion(_)=>if !sozu{if sim.free==0{return Err(RunError::InvalidAction)}sim.free-=1},
            Outcome::SetValue{..}=>{}
            Outcome::StoreNote(instance)=>{let deck=sim.deck.as_ref().map_or(&state.deck,|staged|&staged.deck);if !deck.iter().any(|c|c.instance==*instance){return Err(RunError::InvalidAction)}}
            Outcome::Roll{bound,arms}=>{
                if !last||*bound==0||arms.is_empty()||arms.last().map(|(upper,_)|*upper)!=Some(*bound)||arms.windows(2).any(|w|w[0].0>w[1].0){return Err(RunError::InvalidAction)}
                for (_,arm) in arms{if arm.is_empty(){return Err(RunError::InvalidAction)}check_from(state,sim.clone(),arm)?;}
            }
            Outcome::Combat{..}|Outcome::Rewards{..}|Outcome::NextStage(_)|Outcome::Leave=>if !last{return Err(RunError::InvalidAction)},
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
            // `Damage` has no owner (Golden Idol, Scrap Ooze, Face Trader), so Torii never applies to it.
            Outcome::LoseHp(n)=>if !mutations::lose_hp(state,n,mutations::OutOfCombatLoss::HpLoss){return run.commit_terminal(RunTerminal::Death)},
            Outcome::Damage(n)=>if !mutations::lose_hp(state,n,mutations::OutOfCombatLoss::Damage{owned:false}){return run.commit_terminal(RunTerminal::Death)},
            Outcome::PlayerDamage(n)=>if !mutations::lose_hp(state,n,mutations::OutOfCombatLoss::Damage{owned:true}){return run.commit_terminal(RunTerminal::Death)},
            Outcome::GainMaxHp(n)=>mutations::increase_max_hp(state,n)?,
            Outcome::LoseMaxHp(n)=>mutations::decrease_max_hp(state,n),
            Outcome::ObtainCard{id,upgrades}=>{mutations::obtain_card_or_negated(state,id,upgrades)?;}
            Outcome::ObtainCurse(id)=>{mutations::obtain_card_or_negated(state,id,0)?;}
            Outcome::ObtainStoredNote=>{let (id,upgrades)=state.stored_note;mutations::obtain_stored_card(state,id,upgrades)?;}
            Outcome::StoreNote(instance)=>{let card=state.deck.iter().find(|c|c.instance==instance).ok_or(RunError::InvalidAction)?;state.stored_note=(card.id,card.upgrades);}
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
            Outcome::RemoveSpecific(instance)=>{
                mutations::remove_card(state,instance)?;
                if state.deck.is_empty(){return run.commit_terminal(RunTerminal::EmptyDeck)}
            }
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
            Outcome::SetValue{slot,value}=>event_frame::set_value(run,slot,value)?,
            Outcome::Roll{bound,arms}=>{
                let n=draw(state,&key,bound as usize)? as u32;
                let (_,arm)=arms.into_iter().find(|(upper,_)|n<*upper).ok_or(RunError::InvalidAction)?;
                for outcome in arm.into_iter().rev(){queue.push_front(outcome)}
            }
            Outcome::Combat{encounter,mut rewards,next_stage,elite}=>{
                // The fight's gold range is drawn when it starts (D3) and joins the listed gold.
                if let Some(spec)=rewards.as_mut(){
                    if let Some((low,high))=spec.gold_range.take(){
                        let span=high.checked_sub(low).and_then(|d|d.checked_add(1)).ok_or(RunError::InvalidAction)?;
                        spec.gold=spec.gold.checked_add(low+draw(state,&key,span as usize)? as u32).ok_or(RunError::InvalidAction)?;
                    }
                }
                event_frame::suspend(run,queue.into(),Some(EventCombat{encounter,rewards,next_stage,elite}),false)?;
                return event_frame::enter_combat(run,encounter)
            }
            Outcome::Rewards{potions,relics}=>{
                if obtained.is_some(){return Err(RunError::Unsupported("event reward screen before its relic's equip screens"))}
                let mut rows=vec![];
                for (slot,choice) in relics.into_iter().enumerate(){
                    let key=[&key[..],&[0xE0,slot as u64]].concat();
                    let relic=match choice{
                        RelicChoice::Specific(id)=>id,
                        RelicChoice::Tier(tier)=>relic_pools::take(state,tier,relic_pools::Selector::Front,RelicSource::Event,&key)?,
                        RelicChoice::RandomTier=>{
                            let tier=relic_pools::roll_tier(state,relic_pools::TierTable::Ordinary,&key);
                            relic_pools::take(state,tier,relic_pools::Selector::Front,RelicSource::Event,&key)?
                        }
                    };
                    rows.push((RoomChoiceKind::Relic(relic),None));
                }
                for slot in 0..potions{
                    let key=[&key[..],&[0xB0,u64::from(slot)]].concat();
                    let potion=crate::items::potion_generation::flat(state.character,|bound|{
                        state.rng.bounded_scoped_checked(RunRngScope::M28EventBody,&key,bound)
                    })?;
                    rows.push((RoomChoiceKind::Potion(potion),None));
                }
                rows.push((RoomChoiceKind::Proceed,None));
                return event_frame::open_rewards(run,rows)
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
