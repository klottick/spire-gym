//! Finite Watcher producers on the shared resolver and card owner.
use super::*;
use super::super::model::OwnedCard;
mod m25;
mod m25_direct;
mod m25_state;
mod m25_powers;
mod m25_generation;
mod m25_foreign;
pub(in crate::combat) use m25_foreign::{output_template as foreign_output_template,
    proposals_with_support as foreign_influence_proposals, rarity_tier_for_test as foreign_influence_rarity_tier,
    validate_payload_with_support as validate_foreign_influence_payload};
pub(in crate::combat) use m25::initialize_m25_profile as initialize_m25_base_profile;

/// Extend the complete-Watcher constructor only for transient Foreign Influence
/// outputs. Permanent off-colour deck admission remains owned by Run validation.
pub(in crate::combat) fn initialize_m25_profile(card: &mut LiveCard) -> Result<(), InputError> {
    if crate::content::watcher::foreign_influence_support().contains(&card.id) {
        let color = crate::content::Catalog::standard().card(card.id)
            .map(|row| row.color)
            .ok_or_else(|| InputError::Invariant("Foreign Influence output lacks its catalog colour".into()))?;
        if color != crate::content::CardColor::Purple {
            if matches!(card.origin, CombatCardRef::Permanent(_)) {
                return Err(InputError::Unsupported("off-colour permanent outside complete Watcher admission".into()));
            }
            // Blue metadata owns the existing Defect construction fields. Red, Green
            // and Colorless use their established shared constructor defaults and
            // payload owners; no Watcher initialization is applied to them.
            if color == crate::content::CardColor::Blue { super::defect::initialize(card); }
            return Ok(());
        }
    }
    initialize_m25_base_profile(card)
}

pub(in crate::combat) fn initialize(card:&mut LiveCard) {
    use crate::content::CardId::*;let up=card.upgrades>0;
    card.local.self_retain=matches!(card.id,WindmillStrike|Perseverance|SandsOfTime|Miracle|Smite|Safety)||card.id==Worship&&up;
    if matches!(card.id,Smite|Safety){card.local.exhaust_on_finish=true;}
    if matches!(card.id,Establishment|BattleHymn){card.local.innate=up;}
    match card.id {
        FlurryOfBlows|Weave=>card.local.base_damage=Some(if up{6}else{4}),
        Brilliance=>card.local.base_damage=Some(if up{16}else{12}),
        WindmillStrike=>card.local.base_damage=Some(if up{10}else{7}),
        SandsOfTime=>card.local.base_damage=Some(if up{26}else{20}),
        CarveReality=>card.local.base_damage=Some(if up{10}else{6}),
        CutThroughFate=>card.local.base_damage=Some(if up{9}else{7}),
        JustLucky=>{card.local.base_damage=Some(if up{4}else{3});card.local.base_block=Some(if up{3}else{2});},
        Smite=>card.local.base_damage=Some(if up{16}else{12}),
        Perseverance=>card.local.base_block=Some(if up{7}else{5}),
        DeceiveReality=>card.local.base_block=Some(if up{7}else{4}),
        ThirdEye=>card.local.base_block=Some(if up{9}else{7}),
        Safety=>card.local.base_block=Some(if up{16}else{12}),
        Prostrate=>card.local.base_block=Some(4),
        _=>(),
    }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum WatcherEffect { OpenStanceChoice, PotionEnergy{slot:usize,amount:i32}, CreateRequest{id:crate::content::CardId,count:u32}, Create{template:LiveCard,count:u32,next:u32}, CreateShuffled{template:LiveCard,count:u32,next:u32}, WishOption{option:crate::content::CardId}, Meditate{count:usize}, RequestEndTurn, Scry{amount:i32}, ScryFromRelic{relic:crate::content::RelicId,amount:i32}, CommitSelection{selection:super::super::PendingSelection,next:usize}, RetainRestore { cards:Vec<CardHandle>,next:usize }, ReduceCost{card:CardHandle,delta:i32}, EstablishmentDiscount{amount:i32}, GainMantra { amount:i32 }, StanceEnergy { reason:StanceEnergy }, ReturnDiscard { card:CardHandle },
    PressurePoints, GrantExtraTurn, ExhaustDrawnDeus{card:CardHandle},
    OpenOmniscience, OpenForeignInfluence, JudgmentKill{threshold:i32},
    /// Omniscience's queue items are random-target: each picks its live target when it is
    /// dequeued (GameActionManager randomTarget; source-check-dead-target-v1).
    OmnisciencePlay{child:CardHandle,energy_on_use:i32,cancelled:bool},
    OmniscienceRepeat{child:CardHandle,energy_on_use:i32,cancelled:bool} }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum StanceEnergy { CalmExit, DivinityEntry, VioletLotus }
pub(in crate::combat) fn active(state:&State)->bool {
    state.silent.source_character==Some(crate::content::Character::Watcher)
}
/// M26 §6.3 class L: Watcher card laws run for the Watcher or under the complete capability.
pub(in crate::combat) fn cards_active(state:&State)->bool {
    active(state)||complete(state)
}
/// M26 complete card capability of this combat.
pub(in crate::combat) fn complete(state:&State)->bool {
    state.item_context.as_ref().is_some_and(|items|items.complete_cards())
}
fn bottom(state:&mut State,kind:EffectKind) {
    super::super::enemies::act2::bottom(state,Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind});
}
pub(in crate::combat) fn stance_transition(state:&mut State,next:Stance)->Result<bool,EffectError> {
    if !cards_active(state){return Ok(false)}
    let previous=state.character.stance;
    if previous==next||state.player_full_powers.iter().any(|p|p.id.0=="cannot_change_stance"){return Ok(true)}
    for power in state.player_full_powers.clone() {
        match power.id.0 {
            "mental_fortress"=>bottom(state,EffectKind::GainBlock(power.state.amount)),
            "rushdown" if next==Stance::Wrath=>bottom(state,EffectKind::Draw(power.state.amount)),
            _=>(),
        }
    }
    if previous==Stance::Calm {
        if state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::VioletLotus)) {
            bottom(state,EffectKind::Watcher(WatcherEffect::StanceEnergy{reason:StanceEnergy::VioletLotus}));
        }
        bottom(state,EffectKind::Watcher(WatcherEffect::StanceEnergy{reason:StanceEnergy::CalmExit}));
    }
    state.character.stance=next;
    if next==Stance::Divinity {bottom(state,EffectKind::Watcher(WatcherEffect::StanceEnergy{reason:StanceEnergy::DivinityEntry}));}
    let returns=state.piles.discard.iter().filter(|card|card.full().is_some_and(|card|card.id==crate::content::CardId::FlurryOfBlows)).map(|card|card.handle()).collect::<Vec<_>>();
    for card in returns{bottom(state,EffectKind::Watcher(WatcherEffect::ReturnDiscard{card}));}
    super::super::effects::refresh_enemy_values(state)?;Ok(true)
}
pub(in crate::combat) fn apply(state:&mut State,effect:&Effect,work:&WatcherEffect)->Result<(),EffectError> {
    validate(state,effect,work)?;
    match work {
        WatcherEffect::OpenStanceChoice=>{
            let Source::Potion{slot,id}=effect.source else{return Err(InputError::Invariant("stance choice lacks potion parent".into()).into())};
            if id!=crate::content::PotionId::StancePotion||state.pending_selection.is_some()||state.silent.card_offer.is_some()||state.watcher.stance_choice.is_some()||state.in_play.is_some()
                ||!state.item_context.as_ref().is_some_and(|i|i.consumed_potion_potency(slot,id)==Some(0)) {return Err(InputError::Invariant("stance potion must be consumed once before choice".into()).into())}
            state.watcher.stance_choice=Some(PendingStanceChoice{parent:SelectionParent::Potion{slot,id},options:vec![Stance::Wrath,Stance::Calm],resume_phase:state.phase});state.phase=Phase::Selection;
        },
        WatcherEffect::PotionEnergy{amount,..}=>super::super::effects::gain_energy(state,*amount,true)?,
        WatcherEffect::CreateRequest{id,count}=>{
            let kind=create_action(state,*id,*count)?;state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind});
        },
        WatcherEffect::Create{template,count,next}=>deliver(state,effect,template,*count,*next)?,
        WatcherEffect::CreateShuffled{template,count,next}=>deliver_shuffled(state,effect,template,*count,*next)?,
        WatcherEffect::WishOption{option}=>m25_generation::apply_wish_option(state,effect,*option)?,
        WatcherEffect::Meditate{count}=>meditate(state,effect,*count)?,
        WatcherEffect::RequestEndTurn=>{
            super::ironclad::autoplay::cancel_pending_card_queue(state);
            // Keep the shared Decision checkpoint alive until this physical
            // parent's FinishCard (including Time Warp) has completed. Reuse
            // the same queued request; a forced Time Warp owns the single end.
            if let Some(finish)=state.pending.iter().position(|e|e.kind==EffectKind::FinishCard&&e.source==effect.source){
                state.pending.insert(finish+1,effect.clone());
            }else if state.pending_time_warp.is_none(){
                if let Some(marker)=state.pending.iter_mut().find(|e|e.kind==EffectKind::Decision){
                    marker.kind=EffectKind::EndPlayerTurn;
                }else if state.phase==Phase::Decision&&state.in_play.is_none()&&state.play_frames.is_empty(){
                    // No FinishCard or Decision marker remains, nothing is in play and no play frame
                    // is open. Only a potion action (Distilled Chaos) reaches this state today; it
                    // queues no Decision marker, so its boundary is the queue bottom.
                    state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::EndPlayerTurn});
                }else{
                    return Err(InputError::Invariant("Meditate lost its parent finish/decision checkpoint".into()).into());
                }
            }
        },
        WatcherEffect::Scry{amount}=>scry(state,effect,*amount,None)?,
        WatcherEffect::ScryFromRelic{relic,amount}=>scry(state,effect,*amount,Some(*relic))?,
        WatcherEffect::CommitSelection{selection,next}=>commit_selection(state,effect,selection,*next)?,
        WatcherEffect::RetainRestore{cards,next}=>restore(state,effect,cards,*next)?,
        WatcherEffect::ReduceCost{card,delta}=>{
            let base=super::super::piles::owned_full_cards(state).find(|live|live.instance.0.to_string()==card.as_str()).and_then(|live|super::base_cost(state,live));
            let mut result=Ok(());super::super::piles::for_each_owned_card_mut(state,|live|{if live.instance.0.to_string()==card.as_str(){result=super::super::card_state::modify_cost_for_combat(live,*delta,base);}});result?;
        },
        WatcherEffect::EstablishmentDiscount{amount}=>{
            // EstablishmentPowerAction discounts cards marked retain or selfRetain when it runs:
            // Equilibrium's mark and resolved Well-Laid Plans picks count; Runic Pyramid's hold
            // marks nothing (source-check-endturn-v1). The engine holds those marks in
            // temporary_retain, which Pyramid alone also fills.
            let pyramid_only=state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::RunicPyramid))
                &&!state.player_full_powers.iter().any(|power|power.id.0=="equilibrium");
            let marked=state.temporary_retain.as_ref().filter(|retain|retain.turn==state.turn&&!pyramid_only)
                .map(|retain|retain.cards.clone()).unwrap_or_default();
            let bases=state.piles.hand.iter().map(|owned|owned.full().and_then(|card|super::base_cost(state,card))).collect::<Vec<_>>();
            for (owned,base) in state.piles.hand.iter_mut().zip(bases){
                let held=marked.contains(&owned.handle());
                if let Some(card)=owned.full_mut().filter(|c|c.local.retain||c.local.self_retain||held){super::super::card_state::modify_cost_for_combat(card,amount.wrapping_neg(),base)?;}
            }
        },
        WatcherEffect::GainMantra{amount}=>{
            state.character.mantra_gained=state.character.mantra_gained.checked_add(*amount).ok_or_else(||InputError::Unsupported("Watcher Mantra history int32 exhausted".into()))?;
            bottom(state,EffectKind::ApplyFullPower{id:FullPowerId("mantra"),amount:*amount});
        },
        WatcherEffect::StanceEnergy{reason}=>{
            let amount=match reason{StanceEnergy::CalmExit=>2,StanceEnergy::DivinityEntry=>3,StanceEnergy::VioletLotus=>1};
            super::super::effects::gain_energy(state,amount,true)?;
        },
        WatcherEffect::ReturnDiscard{card}=>{
            super::super::piles::retrieve_ordinary_from_discard(state,card)?;
            super::super::effects::refresh_enemy_values(state)?;
        },
        WatcherEffect::PressurePoints=>{
            let mut damage=state.enemies.iter().filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0)
                .filter_map(|enemy|enemy.full_powers.iter().find(|power|power.id.0=="mark")
                    .filter(|power|power.state.amount>0).map(|power|Effect{source:effect.source.clone(),
                        target:Some(Actor::Enemy(enemy.instance_id.clone())),
                        kind:EffectKind::Damage(super::super::DamagePayload::HpLoss(power.state.amount))}))
                .collect::<Vec<_>>();
            for child in damage.drain(..).rev(){state.pending.push_front(child);}
        },
        WatcherEffect::GrantExtraTurn=>state.extra_turn_pending=true,
        WatcherEffect::ExhaustDrawnDeus{card}=>{
            let index=state.piles.hand.iter().position(|owned|owned.handle()==*card)
                .ok_or_else(||InputError::Invariant("drawn Deus Ex Machina left hand before exhaust".into()))?;
            let owned=state.piles.hand.remove(index);
            super::super::piles::exhaust_owned_card(state,owned)?;
        },
        WatcherEffect::OpenOmniscience=>open_omniscience(state,effect)?,
        WatcherEffect::OpenForeignInfluence=>m25_foreign::open(state,effect)?,
        WatcherEffect::JudgmentKill{threshold}=>if state.enemies.iter().any(|enemy|matches!(&effect.target,Some(Actor::Enemy(owner)) if &enemy.instance_id==owner&&enemy.hp<=*threshold)){
            super::super::effects::instant_kill(state,effect)?
        },
        WatcherEffect::OmnisciencePlay{child,energy_on_use,cancelled}=>{
            let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("Omniscience autoplay lost card parent".into()).into())};
            if *cancelled{
                let card=state.queued_autoplay.remove(child).ok_or_else(||InputError::Invariant("queued Omniscience original absent".into()))?;
                super::super::piles::finish_owned_card(state,card,super::super::model::FinishDisposition::Exhaust,true)?;
                super::ironclad::autoplay::resume_parent(state,child)?;
            }else{
                let actor=omniscience_random_target(state,child)?;
                let playable=super::ironclad::autoplay::queued_child_is_playable(state,child,actor.clone(),*energy_on_use,false)?;
                let repeat=if playable{
                    let instance=super::super::piles::next_generated_instance(state)?;
                    let mut copy=super::super::piles::make_stat_equivalent_copy_from_owner(state,child,instance)?;
                    copy.local.free_play=true;copy.local.purge=true;
                    let copy=OwnedCard::Full(copy);let handle=copy.handle();
                    if state.card_ids_issued.contains(&handle)||state.queued_autoplay.insert(handle.clone(),copy).is_some(){
                        return Err(InputError::Invariant("Omniscience repeat copy identity collision".into()).into())
                    }
                    state.card_ids_issued.insert(handle.clone());Some(handle)
                }else{None};
                super::ironclad::autoplay::start_queued_child(state,child,actor,false,true,*energy_on_use,
                    super::super::model::PlayFrameParentOwner::Card(parent.clone()))?;
                if let Some(copy)=repeat{
                    super::ironclad::autoplay::queue_after_finish(state,&Source::Card(child.clone()),Effect{source:Source::Card(child.clone()),target:Some(Actor::Player),
                        kind:EffectKind::Watcher(WatcherEffect::OmniscienceRepeat{child:copy,energy_on_use:-1,cancelled:false})})?;
                }
            }
        },
        WatcherEffect::OmniscienceRepeat{child,energy_on_use,cancelled}=>{
            let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("Omniscience repeat lost selected-card parent".into()).into())};
            if *cancelled{
                let copy=state.queued_autoplay.remove(child).ok_or_else(||InputError::Invariant("queued Omniscience repeat copy absent".into()))?;
                super::super::piles::finish_owned_card(state,copy,super::super::model::FinishDisposition::Purge,true)?;
                super::ironclad::autoplay::resume_parent(state,child)?;
            }else{
                let target=omniscience_random_target(state,child)?;
                super::ironclad::autoplay::start_queued_child(state,child,target,true,false,*energy_on_use,
                    super::super::model::PlayFrameParentOwner::Card(parent.clone()))?;
            }
        },
    }Ok(())
}
/// A random-target queue item's dequeue-time target: a card that needs an enemy target
/// picks a fresh live one from the card-random stream (the shared Havoc selector); the
/// original Omniscience card and its repeat each pick their own (source-check-dead-target-v1).
fn omniscience_random_target(state:&mut State,child:&CardHandle)->Result<Option<Actor>,EffectError>{
    let live=state.queued_autoplay.get(child).and_then(OwnedCard::full)
        .ok_or_else(||InputError::Invariant("queued Omniscience card absent".into()))?;
    let targeted=crate::content::Catalog::standard().card(live.id).is_some_and(|definition|
        !matches!(definition.card_type,crate::content::CardType::Status|crate::content::CardType::Curse))
        &&matches!(super::payload_with_wrist_blade(state,live,None),Err(crate::run::RunError::InvalidAction));
    if !targeted{return Ok(None)}
    Ok(super::ironclad::autoplay::capture_random_enemy_target(state)?.map(Actor::Enemy))
}
pub(in crate::combat) fn validate(state:&State,effect:&Effect,work:&WatcherEffect)->Result<(),InputError> {
    if !cards_active(state)||(!matches!(work,WatcherEffect::JudgmentKill{..})&&effect.target!=Some(Actor::Player)){
        return Err(InputError::Invariant("Watcher effect lacks its character/target owner".into()))
    }
    let invalid=||InputError::Invariant("Watcher payload lost its owned source/cursor/template".into());
    let potion_capture=|slot:usize,id:crate::content::PotionId,amount:i32|state.item_context.as_ref().is_some_and(|items|(items.m21_profile()||items.complete_items())&&
        (items.potions().get(slot)==Some(&Some(id))&&items.potion_potency(slot)==Some(amount)||items.consumed_potion_potency(slot,id)==Some(amount)));
    match work{
        WatcherEffect::OpenStanceChoice=>if !matches!(effect.source,Source::Potion{slot,id:crate::content::PotionId::StancePotion} if potion_capture(slot,crate::content::PotionId::StancePotion,0)){return Err(invalid())},
        WatcherEffect::CreateRequest{id,count}=>if *id!=crate::content::CardId::Miracle||!matches!(effect.source,Source::Potion{slot,id:crate::content::PotionId::BottledMiracle} if i32::try_from(*count).is_ok_and(|n|potion_capture(slot,crate::content::PotionId::BottledMiracle,n))){return Err(invalid())},
        WatcherEffect::Create{template,count,next}=>{
            let m25=state.item_context.as_ref().is_some_and(|items|items.m25_cards())
                &&crate::content::watcher::supports_m25_generated(template.id);
            if next>count||*count>i32::MAX as u32||!(crate::content::watcher::M21_WATCHER_GENERATED.contains(&template.id)||m25)
                ||template.instance!=CardInstanceId(0)||template.origin!=CombatCardRef::CombatLocal(0)||template.upgrades>1||!template.local.m21{return Err(invalid())}
        },
        WatcherEffect::CreateShuffled{template,count,next}=>{
            let m25=state.item_context.as_ref().is_some_and(|items|items.m25_cards())
                &&crate::content::watcher::supports_m25_generated(template.id);
            let card_parent=matches!((&effect.source,&state.in_play),
                (Source::Card(owner),Some(card)) if owner==&card.handle()&&card.full().is_some_and(|live|matches!(live.id,
                    crate::content::CardId::Evaluate|crate::content::CardId::Pray|crate::content::CardId::ReachHeaven
                    |crate::content::CardId::Alpha|crate::content::CardId::Beta|crate::content::CardId::ConjureBlade))
                    &&state.pending.iter().any(|pending|pending.kind==EffectKind::FinishCard&&pending.source==effect.source));
            let study_hook=effect.source==Source::Actor(Actor::Player)
                &&state.player_full_powers.iter().any(|power|power.id.0=="study"&&power.state.amount>=i32::try_from(*count).unwrap_or(i32::MAX));
            if !m25||next>count||*count==0||*count>i32::MAX as u32||(!card_parent&&!study_hook)
                ||template.instance!=CardInstanceId(0)||template.origin!=CombatCardRef::CombatLocal(0)
                ||template.upgrades>1||!template.local.m21||template.misc<0
                ||template.id!=crate::content::CardId::Expunger&&template.misc!=0{return Err(invalid())}
        },
        WatcherEffect::WishOption{option}=>{
            let parent=matches!((&effect.source,&state.in_play),(Source::Card(owner),Some(card)) if owner==&card.handle()
                &&card.full().is_some_and(|live|live.id==crate::content::CardId::Wish));
            let finish=state.pending.iter().any(|pending|pending.kind==EffectKind::FinishCard&&pending.source==effect.source);
            if !crate::content::watcher::M25_WATCHER_CHOICES.contains(option)
                ||!state.item_context.as_ref().is_some_and(|items|items.m25_cards())||!parent||!finish{return Err(invalid())}
        },
        WatcherEffect::CommitSelection{selection,next}=>{
            let unique=selection.selected.iter().collect::<std::collections::BTreeSet<_>>();
            if !modal(selection.operation)||*next>selection.selected.len()||unique.len()!=selection.selected.len()||selection.selected.iter().any(|id|!selection.candidates.contains(id)||!state.card_ids_issued.contains(id)){return Err(invalid())}
        },
        WatcherEffect::RetainRestore{cards,next}=>if *next>cards.len()||cards.iter().collect::<std::collections::BTreeSet<_>>().len()!=cards.len()||cards.iter().any(|id|!state.card_ids_issued.contains(id)){return Err(invalid())},
        WatcherEffect::StanceEnergy{..}=>if effect.source!=Source::Actor(Actor::Player){return Err(invalid())},
        WatcherEffect::EstablishmentDiscount{amount}=>if *amount<=0||effect.source!=Source::Actor(Actor::Player){return Err(invalid())},
        WatcherEffect::ReduceCost{card,delta}=>if *delta!= -1||!state.card_ids_issued.contains(card){return Err(invalid())},
        WatcherEffect::ReturnDiscard{card}=>if !state.card_ids_issued.contains(card){return Err(invalid())},
        WatcherEffect::OpenOmniscience=>if !matches!((&effect.source,&state.in_play),
            (Source::Card(owner),Some(card)) if owner==&card.handle()&&card.full().is_some_and(|live|live.id==crate::content::CardId::Omniscience)
                &&state.item_context.as_ref().is_some_and(|items|items.m25_cards())){return Err(invalid())},
        WatcherEffect::PressurePoints=>if !matches!((&effect.source,&state.in_play),
            (Source::Card(owner),Some(OwnedCard::Full(card))) if owner==&super::super::card_state::handle(card.instance)
                &&card.id==crate::content::CardId::PressurePoints&&card.local.magic_number.is_some()
                &&state.item_context.as_ref().is_some_and(|items|items.m25_cards())){return Err(invalid())},
        WatcherEffect::GrantExtraTurn=>if !matches!((&effect.source,&state.in_play),
            (Source::Card(owner),Some(OwnedCard::Full(card))) if owner==&super::super::card_state::handle(card.instance)
                &&card.id==crate::content::CardId::Vault&&state.item_context.as_ref().is_some_and(|items|items.m25_cards())){return Err(invalid())},
        WatcherEffect::ExhaustDrawnDeus{card}=>if !matches!(&effect.source,Source::Card(owner) if owner==card)
            ||!state.piles.hand.iter().any(|owned|owned.handle()==*card&&owned.full().is_some_and(|live|live.id==crate::content::CardId::DeusExMachina))
            ||!state.item_context.as_ref().is_some_and(|items|items.m25_cards()){return Err(invalid())},
        WatcherEffect::OpenForeignInfluence=>if !matches!((&effect.source,&state.in_play),
            (Source::Card(owner),Some(card)) if owner==&card.handle()&&card.full().is_some_and(|live|live.id==crate::content::CardId::ForeignInfluence)
                &&state.item_context.as_ref().is_some_and(|items|items.m25_cards())
                &&crate::content::watcher::foreign_influence_support().len()==106){return Err(invalid())},
        WatcherEffect::JudgmentKill{threshold}=>{
            let valid=matches!((&effect.source,&effect.target,&state.in_play,&state.play_context),
                (Source::Card(owner),Some(Actor::Enemy(target)),Some(OwnedCard::Full(card)),Some(context))
                    if owner==&super::super::card_state::handle(card.instance)&&context.card==*owner
                        &&context.target==Some(Actor::Enemy(target.clone()))&&card.id==crate::content::CardId::Judgment
                        &&card.local.magic_number==Some(*threshold));
            if !state.full_runtime||!state.item_context.as_ref().is_some_and(|items|items.m25_cards())
                ||!(30..=40).contains(threshold)||!valid
                ||!state.enemies.iter().any(|enemy|matches!(&effect.target,Some(Actor::Enemy(owner)) if &enemy.instance_id==owner&&enemy.life==ActorLife::Active&&enemy.hp>0)){
                return Err(invalid())
            }
        },
        WatcherEffect::OmnisciencePlay{child,energy_on_use,..}=>{
            let Source::Card(parent)=&effect.source else{return Err(invalid())};
            let queued=state.queued_autoplay.get(child).and_then(OwnedCard::full).is_some();
            let frame=state.play_frames.last().is_some_and(|frame|frame.child==*child&&frame.parent_finish_checkpoint_done
                &&matches!(&frame.parent_owner,super::super::model::PlayFrameParentOwner::Card(owner) if owner==parent)
                &&frame.parent.as_ref().is_some_and(|owner|owner.handle()==*parent&&owner.full().is_some_and(|live|live.id==crate::content::CardId::Omniscience)));
            if !queued||!frame||*energy_on_use!= -1{return Err(invalid())}
        },
        WatcherEffect::OmniscienceRepeat{child,energy_on_use,..}=>{
            let Source::Card(parent)=&effect.source else{return Err(invalid())};
            let queued=state.queued_autoplay.get(child).and_then(OwnedCard::full).is_some_and(|live|live.local.purge&&live.local.free_play);
            let frame=state.play_frames.last().is_some_and(|frame|frame.child==*child&&frame.parent_finish_checkpoint_done
                &&matches!(&frame.parent_owner,super::super::model::PlayFrameParentOwner::Card(owner) if owner==parent)
                &&frame.parent.as_ref().is_some_and(|owner|owner.handle()==*parent))
                // A repeat still queued behind its source (a nested Omniscience choice is open) is owned while that
                // source is in play or suspended as a play-frame parent, as for Burst's repeat.
                ||state.in_play.as_ref().is_some_and(|card|card.handle()==*parent)
                ||state.play_frames.iter().any(|frame|matches!(&frame.parent_owner,super::super::model::PlayFrameParentOwner::Card(owner) if owner==parent));
            if !queued||!frame||*energy_on_use!= -1{return Err(invalid())}
        },
        WatcherEffect::Scry{amount}|WatcherEffect::ScryFromRelic{amount,..}|WatcherEffect::GainMantra{amount}=>if *amount<0{return Err(invalid())},
        WatcherEffect::Meditate{count}=>if !(1..=2).contains(count){return Err(invalid())},
        _=>(),
    }
    if let WatcherEffect::PotionEnergy{slot,amount}=work{
        let valid=matches!(effect.source,Source::Potion{slot:source_slot,id:crate::content::PotionId::EnergyPotion} if source_slot==*slot)
            &&state.item_context.as_ref().is_some_and(|items|(items.m21_profile()||items.complete_items())&&
                (items.potions().get(*slot)==Some(&Some(crate::content::PotionId::EnergyPotion))&&items.potion_potency(*slot)==Some(*amount)
                ||items.consumed_potion_potency(*slot,crate::content::PotionId::EnergyPotion)==Some(*amount)));
        if !valid{return Err(InputError::Invariant("Watcher potion energy requires its actual captured slot/potency".into()))}
    }
    if let WatcherEffect::ScryFromRelic{relic,..}=work {
        if *relic!=crate::content::RelicId::Melange||effect.source!=Source::Actor(Actor::Player)
            ||!state.item_context.as_ref().is_some_and(|items|items.owns_relic(*relic)) {return Err(invalid())}
    }
    if matches!(work,WatcherEffect::StanceEnergy{reason:StanceEnergy::VioletLotus})&&!state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::VioletLotus)){
        return Err(InputError::Invariant("Violet Lotus energy lacks equipped relic".into()))
    }Ok(())
}

pub(in crate::combat) fn starting_cost(id:crate::content::CardId,upgrades:u32)->Option<i32>{
    use crate::content::CardId::*;Some(match id{
        FlurryOfBlows|Prostrate|JustLucky|Weave=>0,
        Worship|WindmillStrike=>2,SandsOfTime=>4,
        MasterReality|Rushdown=>if upgrades>0{0}else{1},
        Brilliance|Devotion|ThirdEye|CutThroughFate|Foresight|Nirvana|Perseverance|Establishment|Meditate|CarveReality|DeceiveReality|BattleHymn|MentalFortress|LikeWater|Smite|Safety=>1,
        _=>return None,
    })
}
pub(in crate::combat) fn payload(state:&State,card:&LiveCard,target:Option<u64>)->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use crate::content::CardId::*;
    // Complete-profile (M25) additions; the finite M21 kinds keep the arms below.
    if card.local.m21&&!crate::content::watcher::supports_generated(card.id){
        if let Some(result)=m25_direct::payload(state,card,target)?{return Ok(Some(result))}
        if let Some(result)=m25_state::payload(state,card,target)?{return Ok(Some(result))}
        if let Some(result)=m25_powers::payload(state,card,target)?{return Ok(Some(result))}
        if let Some(result)=m25_generation::payload(state,card,target)?{return Ok(Some(result))}
        if let Some(result)=m25_foreign::payload(state,card,target)?{return Ok(Some(result))}
    }
    let Some(cost)=starting_cost(card.id,card.upgrades)else{return Ok(None)};
    let up=card.upgrades>0;
    let power=match card.id{
        Devotion=>Some(("devotion",if up{3}else{2})),Foresight=>Some(("foresight",if up{4}else{3})),
        Nirvana=>Some(("nirvana",if up{4}else{3})),Establishment=>Some(("establishment",1)),
        MasterReality=>Some(("master_reality",-1)),BattleHymn=>Some(("battle_hymn",1)),
        MentalFortress=>Some(("mental_fortress",if up{6}else{4})),Rushdown=>Some(("rushdown",2)),LikeWater=>Some(("like_water",if up{7}else{5})),_=>None,
    };
    if let Some((id,amount))=power{
        if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
        return Ok(Some((cost,vec![(Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId(id),amount})])))
    }
    let effects=match card.id{
        FlurryOfBlows|Weave|Brilliance|WindmillStrike|SandsOfTime|Smite=>{
            let target=target.ok_or(crate::run::RunError::InvalidAction)?;
            let mut base=card.local.base_damage.ok_or(crate::run::RunError::InvalidAction)?;
            if card.id==Brilliance{base=base.wrapping_add(state.character.mantra_gained);}
            vec![(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::modified_mutable_damage(state,target,base)?)))]
        },
        Meditate=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            vec![(Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::Meditate{count:if up{2}else{1}})),
                (Some(PayloadTarget::Player),EffectKind::Character(CharacterEffect::Stance(Stance::Calm))),
                (Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::RequestEndTurn))]
        },
        CarveReality|DeceiveReality=>{
            let mut effects=if card.id==CarveReality{let target=target.ok_or(crate::run::RunError::InvalidAction)?;vec![(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::modified_mutable_damage(state,target,card.local.base_damage.unwrap())?)))]}
                else{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,card.local.base_block.unwrap())?))]};
            effects.push((Some(PayloadTarget::Player),create_action(state,if card.id==CarveReality{Smite}else{Safety},1).map_err(|_|crate::run::RunError::InvalidAction)?));effects
        },
        ThirdEye=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            let bonus=if state.item_context.as_ref().is_some_and(|i|i.owns_relic(crate::content::RelicId::GoldenEye)){2}else{0};
            vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,if up{9}else{7})?)),
                (Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::Scry{amount:if up{5+bonus}else{3+bonus}}))]
        },
        CutThroughFate|JustLucky=>{
            let selected=target.ok_or(crate::run::RunError::InvalidAction)?;
            let bonus=if state.item_context.as_ref().is_some_and(|i|i.owns_relic(crate::content::RelicId::GoldenEye)){2}else{0};
            let hit=(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::modified_mutable_damage(state,selected,card.local.base_damage.unwrap())?)));
            let scry=(Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::Scry{amount:bonus+if card.id==CutThroughFate{if up{3}else{2}}else if up{2}else{1}}));
            if card.id==CutThroughFate{vec![hit,scry,(Some(PayloadTarget::Player),EffectKind::Draw(1))]}
            else{vec![scry,(Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,if up{3}else{2})?)),hit]}
        },
        Prostrate|Worship=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            let mut effects=vec![(Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::GainMantra{amount:if card.id==Worship{5}else if up{3}else{2}}))];
            if card.id==Prostrate{effects.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,4)?)));}effects
        },
        Perseverance|Safety=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(super::modified_mutable_block(state,card.local.base_block.ok_or(crate::run::RunError::InvalidAction)?)?))]
        },
        _=>return Ok(None),
    };Ok(Some((cost,effects)))
}

pub(in crate::combat) fn discard_hand(state:&mut State,effect:&Effect)->Result<(),EffectError>{
    if let Some(temporary)=state.temporary_retain.take(){
        for card in &mut state.piles.hand{if temporary.cards.contains(&card.handle()){if let Some(card)=card.full_mut(){card.local.retain=true;}}}
    }
    let mut retained=Vec::new();let mut i=0;
    while i<state.piles.hand.len(){
        if state.piles.hand[i].full().is_some_and(|c|c.local.retain||c.local.self_retain){
            let card=state.piles.hand.remove(i);retained.push(card.handle());state.inactive_cards.push(card);
        }else{i+=1;}
    }
    state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::RetainRestore{cards:retained,next:0})});
    super::super::piles::end_turn_cleanup(state,&[])?;
    Ok(())
}
pub(in crate::combat) fn restore(state:&mut State,effect:&Effect,cards:&[CardHandle],next:usize)->Result<(),EffectError>{
    if next==cards.len(){return Ok(())}
    let handle=&cards[next];let index=state.inactive_cards.iter().position(|c|c.handle()==*handle).ok_or_else(||InputError::Invariant("retained physical card lost its custody".into()))?;
    let card=state.inactive_cards[index].full_mut().ok_or_else(||InputError::Invariant("retained legacy card".into()))?;
    let retained=card.local.self_retain||card.local.retain;
    let sands=retained&&card.id==crate::content::CardId::SandsOfTime;
    if retained{
        match card.id{
            crate::content::CardId::WindmillStrike=>card.local.base_damage=Some(card.local.base_damage.unwrap().checked_add(if card.upgrades>0{5}else{4}).ok_or_else(||InputError::Unsupported("Watcher retained damage int32 exhausted".into()))?),
            crate::content::CardId::Perseverance=>card.local.base_block=Some(card.local.base_block.unwrap().checked_add(if card.upgrades>0{3}else{2}).ok_or_else(||InputError::Unsupported("Watcher retained Block int32 exhausted".into()))?),
            _=>(),
        }
    }
    if sands{bottom(state,EffectKind::Watcher(WatcherEffect::ReduceCost{card:handle.clone(),delta:-1}));}
    let owned=state.inactive_cards.remove(index);state.piles.hand.push(owned);
    if retained{state.piles.hand.last_mut().unwrap().full_mut().unwrap().local.retain=false;}
    if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Watcher(WatcherEffect::RetainRestore{cards:cards.to_vec(),next:next+1});}
    state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::RetainRestore{cards:cards.to_vec(),next:next+1})});
    super::super::piles::refresh_hand_values(state);super::super::effects::refresh_enemy_values(state)?;Ok(())
}

#[derive(Clone,Default,PartialEq,Eq)]
pub struct WatcherState {pub(in crate::combat) stance_choice:Option<PendingStanceChoice>,pub(in crate::combat) resolving_stance_choice:Option<PendingStanceChoice>,
    /// Combat-to-Run permanent upgrade journal; ordinals are contiguous from 1.
    pub(in crate::combat) permanent_upgrades:Vec<crate::combat::PermanentCardUpgrade>,
    /// Energy granted at the next Deva recharge; the `deva_form` power stores growth step.
    pub(in crate::combat) deva_energy_gain:i32}
impl std::fmt::Debug for WatcherState {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        let mut debug=f.debug_struct("WatcherState");
        debug.field("stance_choice",&self.stance_choice).field("resolving_stance_choice",&self.resolving_stance_choice);
        if !self.permanent_upgrades.is_empty(){debug.field("permanent_upgrades",&self.permanent_upgrades);}
        if self.deva_energy_gain>0{debug.field("deva_energy_gain",&self.deva_energy_gain);}
        debug.finish()
    }
}
/// Each entry names the permanent source of a live card of that kind; one source may
/// appear more than once only for a kind that can be upgraded repeatedly.
pub(in crate::combat) fn validate_permanent_upgrades(state:&State)->Result<(),InputError>{
    for (index,upgrade) in state.watcher.permanent_upgrades.iter().enumerate(){
        if upgrade.ordinal!=index as u64+1{return Err(InputError::Invariant("permanent upgrade ordinal".into()))}
        let live=super::super::piles::owned_full_cards(state).any(|card|card.origin==CombatCardRef::Permanent(upgrade.instance)&&card.instance==upgrade.instance&&card.id==upgrade.id);
        if !live{return Err(InputError::Invariant("permanent upgrade live owner mismatch".into()))}
    }
    Ok(())
}
pub(in crate::combat) fn validate_deva_state(state:&State)->Result<(),InputError>{
    let growth=state.player_full_powers.iter().find(|power|power.id.0=="deva_form").map_or(0,|power|power.state.amount);
    if !state.item_context.as_ref().is_some_and(|items|items.m25_cards()){
        if growth!=0||state.watcher.deva_energy_gain!=0{return Err(InputError::Invariant("Deva state requires M25 profile".into()))}
    }else if (growth==0)!=(state.watcher.deva_energy_gain==0)||growth<0||state.watcher.deva_energy_gain<growth{
        return Err(InputError::Invariant("Deva growth step/current energy gain mismatch".into()))
    }
    Ok(())
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct PendingStanceChoice{pub parent:SelectionParent,pub options:Vec<Stance>,pub resume_phase:Phase}
pub(in crate::combat) fn modal(operation:SelectionOperation)->bool{matches!(operation,SelectionOperation::Scry|SelectionOperation::MeditateReturn)}
/// Melange's Scry is the relic's own current payload; like every selection it
/// may suspend any live continuation through the shared selection record.
pub(in crate::combat) fn melange_scry_source(state:&State)->bool{
    state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::Melange))
        &&state.current.as_ref().is_some_and(|effect|effect.source==Source::Actor(Actor::Player)
            &&effect.target==Some(Actor::Player)
            &&matches!(effect.kind,EffectKind::Watcher(WatcherEffect::ScryFromRelic{relic:crate::content::RelicId::Melange,..})))
}
pub(in crate::combat) fn notify_scry_discard(state:&mut State){
    let cards=state.piles.discard.iter().filter(|c|c.full().is_some_and(|c|c.id==crate::content::CardId::Weave)).map(|c|c.handle()).collect::<Vec<_>>();
    for card in cards{bottom(state,EffectKind::Watcher(WatcherEffect::ReturnDiscard{card}));}
}
pub(in crate::combat) fn scry(state:&mut State,effect:&Effect,amount:i32,relic_parent:Option<crate::content::RelicId>)->Result<(),EffectError>{
    if super::super::enemies::act2::combat_victory_ready(state){return Ok(())}
    for power in state.player_full_powers.clone(){if power.id.0=="nirvana"{bottom(state,EffectKind::GainBlock(power.state.amount));}}
    let candidates=state.piles.draw.iter().rev().take(amount.max(0) as usize).map(|c|c.handle()).collect::<Vec<_>>();
    if candidates.is_empty(){return Ok(())}
    let parent=match &effect.source{Source::Card(card) if relic_parent.is_none()=>SelectionParent::Card(card.clone()),
        Source::Actor(Actor::Player) if relic_parent==Some(crate::content::RelicId::Melange)
            &&state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::Melange))=>
            SelectionParent::Power{turn:state.turn,id:FullPowerId("melange")},
        Source::Actor(Actor::Player) if relic_parent.is_none()&&state.phase==Phase::PlayerStart=>SelectionParent::Power{turn:state.turn,id:FullPowerId("foresight")},
        _=>return Err(InputError::Invariant("Scry requires a live card or Foresight parent".into()).into())};
    if candidates.len()>state.public.known_draw.len(){state.public.known_draw=candidates.clone();}
    super::super::effects::begin_selection_range(state,parent,SelectionSourceZone::Draw,candidates.clone(),0,candidates.len(),SelectionDisposition::Discard,EmptySelection::Continue)?;
    state.pending_selection.as_mut().unwrap().operation=SelectionOperation::Scry;
    notify_scry_discard(state);Ok(())
}
fn open_omniscience(state:&mut State,effect:&Effect)->Result<(),EffectError>{
    let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("Omniscience selection lacks its card parent".into()).into())};
    if state.piles.draw.is_empty(){return Ok(())}
    let candidates=canonical_omniscience_candidates(state);
    super::super::effects::begin_selection_range(state,SelectionParent::Card(parent.clone()),SelectionSourceZone::Draw,candidates,1,1,
        SelectionDisposition::Exhaust,EmptySelection::Continue)?;
    state.pending_selection.as_mut().unwrap().operation=SelectionOperation::Omniscience;
    Ok(())
}
fn canonical_omniscience_candidates(state:&State)->Vec<CardHandle>{
    let mut candidates=state.piles.draw.iter().map(OwnedCard::handle).collect::<Vec<_>>();
    candidates.sort_by_key(|handle|{let card=state.piles.draw.iter().find(|card|card.handle()==*handle).unwrap().full().unwrap();(card.id.as_str(),card.instance)});
    candidates
}
pub(in crate::combat) fn validate_selection(state:&State,s:&PendingSelection)->Result<(),InputError>{
    if s.operation==SelectionOperation::Omniscience{
        let expected=canonical_omniscience_candidates(state);
        let valid_parent=matches!((&s.parent,&state.in_play),
            (SelectionParent::Card(parent),Some(owner)) if parent==&owner.handle()&&owner.full().is_some_and(|card|card.id==crate::content::CardId::Omniscience));
        if !cards_active(state)||!state.full_runtime||!state.item_context.as_ref().is_some_and(|items|items.m25_cards())
            ||!valid_parent||s.source_zone!=SelectionSourceZone::Draw||s.disposition!=SelectionDisposition::Exhaust
            ||s.minimum!=1||s.maximum!=1||s.empty!=EmptySelection::Continue||s.candidates.is_empty()
            ||s.candidates!=expected||s.selected.len()>1{
            return Err(InputError::Invariant("Omniscience selection lost its exact draw candidates or parent".into()))
        }
        return Ok(())
    }
    if !modal(s.operation){return Ok(())}
    let valid_parent=match &s.parent {
        SelectionParent::Card(owner)=>state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full()).is_some_and(|c|match s.operation{
            SelectionOperation::Scry=>matches!(c.id,crate::content::CardId::ThirdEye|crate::content::CardId::CutThroughFate|crate::content::CardId::JustLucky),
            SelectionOperation::MeditateReturn=>c.id==crate::content::CardId::Meditate,_=>false}),
        SelectionParent::Power{turn,id}=>s.operation==SelectionOperation::Scry&&*turn==state.turn&&(
            id.0=="foresight"&&state.in_play.is_none()&&s.resume.phase==Phase::PlayerStart
                &&state.player_full_powers.iter().any(|p|p.id==*id&&p.state.amount>0)
            ||id.0=="melange"&&state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::Melange))),
        _=>false,
    };
    if !cards_active(state)||!valid_parent||super::super::selection::validate_resume(state,&s.resume).is_err()
        ||s.source_zone!=if s.operation==SelectionOperation::Scry{SelectionSourceZone::Draw}else{SelectionSourceZone::Discard}
        ||s.disposition!=if s.operation==SelectionOperation::Scry{SelectionDisposition::Discard}else{SelectionDisposition::Hand}{
        return Err(InputError::Invariant("Watcher selection lost exact operation/parent/resume ownership".into()))
    }Ok(())
}
pub(in crate::combat) fn commit_selection(state:&mut State,effect:&Effect,s:&PendingSelection,next:usize)->Result<(),EffectError>{
    if next>s.selected.len(){return Err(InputError::Invariant("Watcher selected movement cursor".into()).into())}
    if next==s.selected.len(){
        if s.operation==SelectionOperation::Scry{notify_scry_discard(state);}
        return Ok(())
    }
    let handle=&s.selected[next];
    if s.operation==SelectionOperation::Scry{
        let index=state.piles.draw.iter().position(|card|card.handle()==*handle).ok_or_else(||InputError::Invariant("Scry selected physical card left draw".into()))?;
        let card=state.piles.draw.remove(index);state.piles.discard.push(card);
        state.public.known_draw.retain(|id|id!=handle);state.public.known_draw_bottom.retain(|id|id!=handle);
    }else{
        if state.piles.hand.len()<10{
            let index=state.piles.discard.iter().position(|card|card.handle()==*handle).ok_or_else(||InputError::Invariant("Meditate selected physical card left discard".into()))?;
            let mut card=state.piles.discard.remove(index);card.full_mut().unwrap().local.retain=true;state.piles.hand.push(card);
        }
    }
    // Record committed movement before any subsequent derived-value callback.
    if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Watcher(WatcherEffect::CommitSelection{selection:s.clone(),next:next+1});}
    super::super::piles::refresh_hand_values(state);super::super::effects::refresh_enemy_values(state)?;
    state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::CommitSelection{selection:s.clone(),next:next+1})});Ok(())
}
pub(in crate::combat) fn upgrade_local(card:&mut LiveCard)->Result<(),crate::run::RunError>{
    if !card.local.m21&&starting_cost(card.id,0).is_none(){return Ok(())}
    let mut base=super::super::card_state::permanent(card.id,CardInstanceId(0),0);
    let mut upgraded=super::super::card_state::permanent(card.id,CardInstanceId(0),1);
    // M21 kinds re-initialize exactly as before; complete-profile kinds were admitted by M25.
    if card.local.m21{initialize_m25_profile(&mut base).map_err(|_|crate::run::RunError::InvalidAction)?;initialize_m25_profile(&mut upgraded).map_err(|_|crate::run::RunError::InvalidAction)?;}
    if let (Some(old),Some(new))=(base.local.base_damage,upgraded.local.base_damage){card.local.base_damage=Some(card.local.base_damage.unwrap_or(old).wrapping_add(new.wrapping_sub(old)));}
    if let (Some(old),Some(new))=(base.local.base_block,upgraded.local.base_block){card.local.base_block=Some(card.local.base_block.unwrap_or(old).wrapping_add(new.wrapping_sub(old)));}
    card.local.self_retain=upgraded.local.self_retain;card.local.innate=upgraded.local.innate;
    if card.local.m21{card.local.magic_number=upgraded.local.magic_number;}
    // An unplayable kind (Deus Ex Machina) has no cost on either face.
    if let (Some(old),Some(new))=(base.local.mutable_combat_cost.or_else(||starting_cost(card.id,0)),upgraded.local.mutable_combat_cost.or_else(||starting_cost(card.id,1))){
        if old!=new{super::super::card_state::upgrade_base_cost(card,old,new)?;}
    }
    Ok(())
}

pub(in crate::combat) fn meditate(state:&mut State,effect:&Effect,count:usize)->Result<(),EffectError>{
    let candidates=state.piles.discard.iter().map(|c|c.handle()).collect::<Vec<_>>();
    if candidates.is_empty(){return Ok(())}
    if candidates.len()<=count{
        for card in &mut state.piles.discard{card.full_mut().unwrap().local.retain=true;}
        for card in candidates{super::super::piles::retrieve_ordinary_from_discard(state,&card)?;}
        return Ok(())
    }
    let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("Meditate has no physical card parent".into()).into())};
    super::super::effects::begin_selection_range(state,SelectionParent::Card(parent.clone()),SelectionSourceZone::Discard,candidates,count,count,SelectionDisposition::Hand,EmptySelection::Continue)?;
    state.pending_selection.as_mut().unwrap().operation=SelectionOperation::MeditateReturn;Ok(())
}
pub(in crate::combat) fn construct_mantra(state:&mut State,amount:i32)->Result<EffectKind,InputError>{
    state.character.mantra_gained=state.character.mantra_gained.checked_add(amount).ok_or_else(||InputError::Unsupported("Watcher Mantra history int32 exhausted".into()))?;
    Ok(EffectKind::ApplyFullPower{id:FullPowerId("mantra"),amount})
}
pub(in crate::combat) fn capture(state:&mut State,effects:&mut [Effect])->Result<(),EffectError>{
    if !cards_active(state){return Ok(())}
    for effect in effects{
        if let EffectKind::Watcher(WatcherEffect::GainMantra{amount})=effect.kind{effect.kind=construct_mantra(state,amount)?;}
        if let EffectKind::Watcher(WatcherEffect::CreateRequest{id,count})=effect.kind{effect.kind=create_action(state,id,count)?;}
        if let EffectKind::CreateCard{id:crate::content::CardId::Miracle,destination:CardDestination::Hand,upgrades:0}=effect.kind{
            if effect.source==Source::None&&state.current.as_ref().is_some_and(|e|e.kind==EffectKind::Hook(Hook::CombatStartedPreDraw)){
                effect.kind=create_action(state,crate::content::CardId::Miracle,1)?;
            }
        }
    }Ok(())
}
pub(in crate::combat) fn hook(state:&mut State,hook:Hook,_target:Option<&Actor>,_work:&mut Vec<Effect>)->Result<(),EffectError>{
    if hook==Hook::CardAfterUse{
        let source=state.current.as_ref().map_or(Source::None,|effect|effect.source.clone());
        m25_powers::after_card_use(state,&source);
    }
    // Like Water, Study and Omega are end-of-turn callbacks in the shared player pass
    // (`hooks::player_end_power`); the turn-start recharge is `turns::recharge`.
    Ok(())
}
/// Study and Omega at their place in the player's atEndOfTurn pass.
pub(in crate::combat) fn end_turn_power(state:&State,power:&FullPowerEntry,work:&mut Vec<Effect>)->Result<(),EffectError>{
    m25_generation::end_turn_power(state,power,work)
}

/// The stance start hook (Divinity's exit) runs ahead of the power callbacks.
pub(in crate::combat) fn stance_start(state:&State,work:&mut Vec<Effect>){
    if state.character.stance==Stance::Divinity{work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Character(CharacterEffect::Stance(Stance::Neutral))});}
}

/// Watcher power callbacks in the two start slots, run in power-list order by `hooks::dispatch`.
/// M26 §6.3 split: the power callbacks are L (`cards_active`, each still requiring its power).
pub(in crate::combat) fn start_power(state:&mut State,hook:Hook,power:&FullPowerEntry,work:&mut Vec<Effect>)->Result<(),EffectError>{
    if !cards_active(state){return Ok(())}
    let source=Source::Actor(Actor::Player);
    match (hook,power.id.0) {
        (Hook::TurnStartedPreDraw,"blasphemer") if state.turn>1&&power.state.amount>0=>
            work.push(Effect{source,target:Some(Actor::Player),kind:EffectKind::Damage(super::super::DamagePayload::HpLoss(99_999))}),
        (Hook::TurnStartedPreDraw,"simmering_fury") if power.state.amount>0=>{
            work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Character(CharacterEffect::Stance(Stance::Wrath))});
            work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Draw(power.state.amount)});
            work.push(Effect{source,target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(power.id)});
        },
        // Only Foresight's EmptyDeckShuffle is addToTop; its Scry stays addToBot in callback order.
        (Hook::TurnStartedPreDraw,"foresight")=>{
            if state.piles.draw.is_empty(){work.insert(0,Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Recycle});}
            let bonus=if state.item_context.as_ref().is_some_and(|i|i.owns_relic(crate::content::RelicId::GoldenEye)){2}else{0};
            work.push(Effect{source,target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::Scry{amount:power.state.amount.wrapping_add(bonus)})});
        },
        (Hook::TurnStartedPreDraw,"battle_hymn") if !super::super::enemies::act2::combat_victory_ready(state)=>
            work.push(Effect{source,target:Some(Actor::Player),kind:create_action(state,crate::content::CardId::Smite,power.state.amount.max(0) as u32)?}),
        // DevotionPower.atStartOfTurnPostDraw (no turn-1 call: `effects::player_turn_start`).
        (Hook::TurnStarted,"devotion")=>{
            let kind=if !state.player_full_powers.iter().any(|p|p.id.0=="mantra")&&power.state.amount>=10{
                EffectKind::Character(CharacterEffect::Stance(Stance::Divinity))
            }else{construct_mantra(state,power.state.amount)?};
            work.push(Effect{source,target:Some(Actor::Player),kind});
        },
        _=>(),
    }
    Ok(())
}

pub(super) fn actual_draw(state:&mut State,owner:&CardHandle)->Result<(),EffectError>{m25_powers::actual_draw(state,owner)}

pub(in crate::combat) fn after_damage(state:&mut State,result:&super::super::model::DamageResult)->Result<(),EffectError>{
    if !state.full_runtime||result.last_damage_taken()<=0||!matches!(&result.target,Actor::Enemy(_)){return Ok(())}
    let Source::Card(owner)=&result.source else{return Ok(())};
    if !state.in_play.as_ref().is_some_and(|card|card.handle()==*owner&&card.full().is_some_and(|live|live.id==crate::content::CardId::Wallop)){
        return Ok(())
    }
    state.pending.push_front(Effect{source:result.source.clone(),target:Some(Actor::Player),kind:EffectKind::GainBlock(result.last_damage_taken())});
    Ok(())
}

/// Explicit profile initialization on the existing physical constructor owner.
/// Old bare factories retain their accepted fallback fields and Debug output.
pub(in crate::combat) fn initialize_profile(card:&mut LiveCard)->Result<(),InputError>{
    use crate::content::CardId::*;
    // A Curse permanent (e.g. Writhing Mass's Parasite) keeps the shared constructor's
    // unplayable, costless fields in every Watcher combat profile; it gets no Watcher state.
    if crate::content::Catalog::standard().card(card.id).is_some_and(|def|def.card_type==crate::content::CardType::Curse){return Ok(())}
    if !crate::content::watcher::supports_generated(card.id)||card.upgrades>1{return Err(InputError::Unsupported("card outside finite M21 construction profile".into()))}
    initialize(card);let up=card.upgrades>0;card.local.m21=true;
    let cost=match card.id{StrikePurple|DefendPurple|EmptyFist=>1,Eruption=>if up{1}else{2},Vigilance=>2,Miracle=>0,_=>starting_cost(card.id,card.upgrades).unwrap()};
    card.local.mutable_combat_cost=Some(cost);card.local.turn_cost=Some(cost);
    match card.id{StrikePurple=>card.local.base_damage=Some(if up{9}else{6}),DefendPurple=>card.local.base_block=Some(if up{8}else{5}),Eruption=>card.local.base_damage=Some(9),Vigilance=>card.local.base_block=Some(if up{12}else{8}),EmptyFist=>card.local.base_damage=Some(if up{14}else{9}),Miracle=>card.local.exhaust_on_finish=true,_=>()}
    card.local.magic_number=match card.id{
        Prostrate|Devotion=>Some(if up{3}else{2}),Worship=>Some(5),Foresight|Nirvana=>Some(if up{4}else{3}),
        ThirdEye=>Some(if up{5}else{3}),CutThroughFate=>Some(if up{3}else{2}),JustLucky|Meditate=>Some(if up{2}else{1}),
        WindmillStrike|MentalFortress=>Some(if up{if card.id==WindmillStrike{5}else{6}}else{4}),Perseverance=>Some(if up{3}else{2}),
        Rushdown=>Some(2),LikeWater=>Some(if up{7}else{5}),
        Establishment|BattleHymn=>Some(1),_=>None,
    };Ok(())
}

fn installed_master_reality(state:&State)->bool{state.player_full_powers.iter().any(|p|p.id.0=="master_reality")}
pub(in crate::combat) fn create_action(state:&State,id:crate::content::CardId,count:u32)->Result<EffectKind,InputError>{
    let m25=state.item_context.as_ref().is_some_and(|items|items.m25_cards());
    if !crate::content::watcher::M21_WATCHER_GENERATED.contains(&id)
        &&!(m25&&crate::content::watcher::supports_m25_generated(id)){
        return Err(InputError::Invariant("fixed Watcher child route kind".into()))
    }
    // MakeTempCardInHandAction's constructor checks installed Master Reality and
    // upgrades the template, for every profile; the hand helper checks again later.
    let upgrades=u32::from(installed_master_reality(state));
    let mut template=super::super::card_state::generated(id,CardInstanceId(0),upgrades).ok_or_else(||InputError::Unsupported("fixed Watcher child constructor unavailable".into()))?;
    if m25{initialize_m25_profile(&mut template)?}else{initialize_profile(&mut template)?}
    Ok(EffectKind::Watcher(WatcherEffect::Create{template,count,next:0}))
}
fn deliver(state:&mut State,effect:&Effect,template:&LiveCard,count:u32,next:u32)->Result<(),EffectError>{
    if next>count{return Err(InputError::Invariant("fixed creation cursor exceeds captured count".into()).into())}
    if next==count{return Ok(())}
    let instance=super::super::piles::next_generated_instance(state)?;
    let mut child=template.stat_equivalent_copy(instance);
    // The hand helper upgrades its actual inserted child; the invoked overflow
    // helper upgrades only a separate display copy, so the argument stays early.
    if state.piles.hand.len()<10&&installed_master_reality(state)&&child.upgrades==0{
        upgrade_local(&mut child).map_err(|_|InputError::Invariant("fixed child upgrade failed".into()))?;
        child.upgrades=1;
    }
    super::super::piles::insert_generated_with(state,child,CardDestination::Hand,super::super::piles::MasterRealityRoute::RouteChecked)?;
    if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Watcher(WatcherEffect::Create{template:template.clone(),count,next:next+1});}
    super::super::effects::refresh_enemy_values(state)?;
    state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::Create{template:template.clone(),count,next:next+1})});Ok(())
}
fn deliver_shuffled(state:&mut State,effect:&Effect,template:&LiveCard,count:u32,next:u32)->Result<(),EffectError>{
    if next>count{return Err(InputError::Invariant("M25 shuffled creation cursor exceeds captured count".into()).into())}
    if next==count{return Ok(())}
    let instance=super::super::piles::next_generated_instance(state)?;
    let mut child=template.stat_equivalent_copy(instance);
    initialize_m25_profile(&mut child)?;
    super::super::piles::insert_generated(state,child,CardDestination::ShuffledDraw)?;
    if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Watcher(WatcherEffect::CreateShuffled{template:template.clone(),count,next:next+1});}
    super::super::effects::refresh_enemy_values(state)?;
    state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Watcher(WatcherEffect::CreateShuffled{template:template.clone(),count,next:next+1})});
    Ok(())
}
pub(in crate::combat) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    if !card.local.m21&&starting_cost(card.id,card.upgrades).is_none(){return None}
    Some(CardPublicValues{base_damage:card.local.base_damage,base_block:card.local.base_block,magic_number:card.local.magic_number,damage_bonus:None,
        cost_kind:if matches!(card.id,crate::content::CardId::Collect|crate::content::CardId::ConjureBlade){CardCostKind::X}
            else if card.local.turn_cost.is_none(){CardCostKind::Unplayable}else{CardCostKind::Fixed},
        free_play:card.local.free_play||card.local.free_to_play_once,exhaust_on_finish:card.local.exhaust_on_finish,
        ethereal:card.local.ethereal,innate:card.local.innate,temporary_cost:card.local.temporary_cost})
}
pub(in crate::combat) fn card_values(state:&State,card:&LiveCard)->Option<super::super::view::WatcherCardValues>{
    if !active(state){return None}
    let combat=super::base_cost(state,card).and_then(|base|super::super::card_state::combat_price(card,base).ok());
    Some(super::super::view::WatcherCardValues{combat_cost:combat,turn_cost:card.local.turn_cost.or(combat),self_retain:card.local.self_retain,retain:card.local.retain})
}
pub(in crate::combat) fn validate_choice(state:&State)->Result<(),InputError>{
    let choice=state.watcher.stance_choice.as_ref().ok_or_else(||InputError::Invariant("stance choice absent".into()))?;
    let valid=match choice.parent{SelectionParent::Potion{slot,id:crate::content::PotionId::StancePotion}=>state.item_context.as_ref().is_some_and(|i|i.consumed_potion_potency(slot,crate::content::PotionId::StancePotion)==Some(0)),_=>false};
    if !active(state)||!valid||state.phase!=Phase::Selection||state.pending_selection.is_some()||state.silent.card_offer.is_some()||state.in_play.is_some()
        ||choice.options!=[Stance::Wrath,Stance::Calm]||matches!(choice.resume_phase,Phase::Selection|Phase::Terminal){return Err(InputError::Invariant("stance choice exact potion/phase/options owner".into()))}Ok(())
}

/// Miracle reaches the same source addEnergy helper as stance and potion gains.
/// The opt-in binding leaves legacy bare-profile overflow behavior unchanged.
pub(in crate::combat) fn capped_energy_source(state:&State,source:&Source,amount:i32)->bool{
    cards_active(state)&&matches!(source,Source::Card(owner) if state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full()).is_some_and(|c|c.id==crate::content::CardId::Miracle&&amount==if c.upgrades>0{2}else{1}))
}

pub(in crate::combat) fn power_id(id:FullPowerId)->bool{matches!(id.0,"devotion"|"foresight"|"nirvana"|"establishment"|"master_reality"|"battle_hymn"|"mental_fortress"|"rushdown"|"like_water"|"mantra"
    |"vigor"|"wave_of_the_hand"|"free_attack"|"simmering_fury"|"energy_down"|"deva_form"|"collect"|"blasphemer"|"study"|"omega")}

pub(in crate::combat) fn valid_power_application(state:&State,effect:&Effect,id:FullPowerId,amount:i32)->bool{
    if matches!(effect.source,Source::Relic(_)){
        return id.0=="vigor"&&effect.target==Some(Actor::Player)&&(1..=999).contains(&amount)
            &&state.item_context.as_ref().is_some_and(|items|items.complete_items());
    }
    if !cards_active(state)||effect.target!=Some(Actor::Player){return false}
    if !matches!(id.0,"vigor"|"wave_of_the_hand"|"free_attack"|"simmering_fury"|"energy_down"|"deva_form"|"collect"|"blasphemer"|"study"|"omega"){
        return if id.0=="master_reality"{amount== -1}else{amount>0};
    }
    let Source::Card(card_owner)=&effect.source else{return false};
    let card=state.in_play.as_ref().filter(|owned|owned.handle()==*card_owner).or_else(||
        state.piles.hand.iter().find(|owned|owned.handle()==*card_owner));
    let Some(card)=card.and_then(OwnedCard::full) else{return false};
    if super::super::card_state::handle(card.instance)!=*card_owner{return false}
    let Some(items)=state.item_context.as_ref().filter(|items|items.m25_cards()) else{return false};
    let _=items;
    match id.0{
        "vigor"=>card.id==crate::content::CardId::WreathOfFlame&&card.local.magic_number==Some(amount)&&amount>0,
        "wave_of_the_hand"=>card.id==crate::content::CardId::WaveOfTheHand&&card.local.magic_number==Some(amount)&&amount>0,
        "free_attack"=>card.id==crate::content::CardId::Swivel&&amount==1,
        "simmering_fury"=>card.id==crate::content::CardId::SimmeringFury&&card.local.magic_number==Some(amount)&&amount>0,
        "energy_down"=>card.id==crate::content::CardId::Fasting&&amount==1,
        "deva_form"=>card.id==crate::content::CardId::DevaForm&&amount==1,
        "collect"=>card.id==crate::content::CardId::Collect&&{
            super::captured_x(state,card).ok().and_then(|energy|super::chemical_x_bonus(state).ok().and_then(|bonus|energy.checked_add(bonus)))
                .and_then(|x|x.checked_add(i32::from(card.upgrades>0)))==Some(amount)&&amount>0
        },
        "blasphemer"=>card.id==crate::content::CardId::Blasphemy&&amount==1,
        "study"=>card.id==crate::content::CardId::Study&&amount==1,
        "omega"=>card.id==crate::content::CardId::Omega&&card.local.magic_number==Some(amount)&&amount>0,
        _=>true,
    }
}

pub(in crate::combat) fn valid_mutable_attack_source(state:&State,source:&Source)->bool{
    cards_active(state)&&matches!(source,Source::Card(owner) if state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full()).is_some_and(|c|crate::content::watcher::supports_generated(c.id)&&c.local.base_damage.is_some()))
}
