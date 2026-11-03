use crate::{
    combat::{Actor, Effect, EffectKind, Hook, Source},
    content::{Catalog, NoteStorage, RelicId},
    run::{RunError, RunState},
};
use super::lifecycle::{EquipWork, SelectionSpec};

use super::CombatItemContext;

/// Run event work of the accepted relics: none. Burning Blood and Black Blood heal inside the
/// combat (`Hook::CombatSettled` phase below); the Run-side settlement heal survives only as the
/// `items::on_hook` bridge (`settlement_heal_bridge`), so the M27 `CombatSettled` Run event never
/// heals a second time (Main-2, A13 duplicate-heal finding).
pub(super) fn run_event(_id: RelicId, _state: &mut RunState, _event: &mut super::dispatch::RunItemEvent) -> Result<bool, RunError> {
    Ok(false)
}

/// The accepted Run-hook settlement heal (bodies unchanged): one mutually exclusive base heal
/// (Black Blood wins).
pub(super) fn settlement_heal_bridge(state: &mut RunState) {
    if state.hp>0&&(state.relics.contains(&RelicId::BurningBlood)||state.relics.contains(&RelicId::BlackBlood)) {
        let base=if state.relics.contains(&RelicId::BlackBlood){12}else{6};
        let heal = super::ordinary_heal_amount(&state.relics, base).max(0) as u32;
        state.hp = state.hp.saturating_add(heal).min(state.max_hp);
    }
}

fn player(kind:EffectKind)->Effect{Effect{source:Source::None,target:Some(Actor::Player),kind}}

fn complete_miracle_creation(count:u32)->Result<EffectKind,crate::combat::InputError>{
    let mut template=crate::combat::m21_watcher_live_card(crate::content::CardId::Miracle,
        crate::combat::CardInstanceId(1),0)?;
    template.instance=crate::combat::CardInstanceId(0);
    template.origin=crate::combat::CombatCardRef::CombatLocal(0);
    template.copy_family=0;
    template.copy_provenance=crate::combat::card_state::CopyProvenance::Generated;
    Ok(EffectKind::Watcher(crate::combat::cards::watcher::WatcherEffect::Create{
        template,count,next:0}))
}

/// Combat phase work of the accepted generic relics, one relic per call (M27 design §6.2;
/// bodies unchanged from the former single `on_combat_hook`, Anchor's bottom side kept).
pub(super) fn phase(id: RelicId, context: &mut CombatItemContext, phase: super::dispatch::ItemPhase,
    facts: &super::dispatch::PhaseFacts<'_>, work: &mut super::RelicHookWork) -> Result<bool, crate::combat::InputError>
{
    if phase==super::dispatch::ItemPhase::PotionUsed {
        if id==RelicId::ToyOrnithopter&&context.complete_items() {
            work.bottom.push(player(EffectKind::Heal(5)));
            return Ok(true)
        }
        return Ok(false)
    }
    // Philosopher's Stone (Main relic, plan Task M5): every enemy gains 1 Strength at combat
    // start and every enemy spawned later gains 1 (catalog `atBattleStart` + `onSpawnMonster`).
    if id==RelicId::PhilosophersStone {
        if context.complete_items() {
            let strength=|owner:&crate::combat::EnemyHandle|Effect{source:Source::Relic(RelicId::PhilosophersStone),
                target:Some(Actor::Enemy(owner.clone())),
                kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("strength"),amount:1}};
            match phase {
                // Immediate in the source; top is the nearest queued position.
                super::dispatch::ItemPhase::CombatStart if facts.hook==Some(Hook::CombatStarted)=>
                    work.top.extend(facts.live_enemies.iter().map(strength)),
                super::dispatch::ItemPhase::EnemySpawned=>if let Some(owner)=facts.spawned {work.bottom.push(strength(owner));},
                _=>{}
            }
        }
        return Ok(true)
    }
    // Pure Water and Holy Water use the single pre-draw owner selected by combat
    // entry: either the M21 queued hook or hook-free startup dispatch, never both.
    if phase==super::dispatch::ItemPhase::CombatStartPreDraw&&context.is_pre_draw_visit(facts.hook){
        match id {
            // Cracked Core channels at atPreBattle, before the opening hand.
            RelicId::CrackedCore=>{
                work.bottom.push(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Character(
                    crate::combat::CharacterEffect::Channel(crate::combat::OrbKind::Lightning))});
                return Ok(true)
            }
            RelicId::PureWater if context.complete_items()||!context.m21_profile()=>{
                let creation=if !context.complete_items()
                    ||context.m21_profile()&&facts.hook==Some(Hook::CombatStartedPreDraw){
                    EffectKind::CreateCard{id:crate::content::CardId::Miracle,
                        destination:crate::combat::CardDestination::Hand,upgrades:0}
                }else{complete_miracle_creation(1)?};
                work.top.push(player(creation));
                return Ok(true)
            }
            RelicId::HolyWater if context.complete_items()||!context.m21_profile()=>{
                let creation=if context.m22_profile()&&facts.hook==Some(Hook::CombatStartedPreDraw){
                    EffectKind::Watcher(crate::combat::cards::watcher::WatcherEffect::CreateRequest{
                        id:crate::content::CardId::Miracle,count:3})
                }else{complete_miracle_creation(3)?};
                work.top.push(player(creation));
                return Ok(true)
            }
            _=>{}
        }
    }
    let Some(hook)=facts.hook else { return Ok(matches!(id,RelicId::Orichalcum|RelicId::Damaru|RelicId::Duality|RelicId::CloakClasp
        |RelicId::Melange|RelicId::GoldenEye|RelicId::Lantern|RelicId::NeowsLament|RelicId::HolyWater|RelicId::TeardropLocket
        |RelicId::DataDisk|RelicId::Vajra|RelicId::OddlySmoothStone|RelicId::BagOfPreparation|RelicId::BronzeScales
        |RelicId::BloodVial|RelicId::Anchor|RelicId::RingOfTheSnake|RelicId::CrackedCore|RelicId::PureWater
        |RelicId::BurningBlood|RelicId::BlackBlood|RelicId::RedMask|RelicId::BagOfMarbles)) };
    let block=facts.block;
    match id {
        RelicId::Orichalcum=>{
            if hook==Hook::TurnStartedPreDraw {context.orichalcum_trigger=false;}
            // AbstractRoom.applyEndOfTurnRelics runs before the player's pre-card powers
            // (source-check-endturn-v1), so the Block read here precedes Metallicize.
            if hook==Hook::PlayerPreEndTurnCards&&(block==0||context.orichalcum_trigger) {
                context.orichalcum_trigger=false;
                work.top.push(player(EffectKind::GainBlock(6)));
            }
        }
        RelicId::Damaru=>if hook==Hook::TurnStartedPreDraw {
            work.bottom.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
                kind:EffectKind::Watcher(crate::combat::cards::watcher::WatcherEffect::GainMantra{amount:1})});
        },
        RelicId::Duality=>if hook==Hook::AttackResolved {
            work.top.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
                kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("dexterity"),amount:1}});
            work.top.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
                kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("lose_dexterity"),amount:1}});
        },
        // CloakClasp.onPlayerEndTurn: addToBot Block equal to the hand size at the callback.
        RelicId::CloakClasp=>if hook==Hook::PlayerPreEndTurnCards&&facts.hand_size>0 {
            work.bottom.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
                kind:EffectKind::GainBlock(facts.hand_size as i32)});
        },
        RelicId::Melange=>if hook==Hook::DrawPileShuffled {
            let amount=3+i32::from(context.owns_relic(RelicId::GoldenEye))*2;
            work.top.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
                kind:EffectKind::Watcher(crate::combat::cards::watcher::WatcherEffect::ScryFromRelic{relic:RelicId::Melange,amount})});
        },
        // Golden Eye is an amount modifier read by Scry owners (and Melange above).
        RelicId::GoldenEye=>{}
        RelicId::Lantern=>{
            if hook==Hook::CombatStarted {context.lantern_armed=true;}
            if hook==Hook::TurnStartedPreDraw&&context.lantern_armed {
                context.lantern_armed=false;
                work.top.push(player(EffectKind::GainEnergy(1)));
            }
        }
        RelicId::NeowsLament=>if hook==Hook::CombatStarted {
            if let Some(counter)=context.counters.get_mut(&RelicId::NeowsLament) {
                if *counter>0 {
                    *counter=if *counter==1{-2}else{*counter-1};
                    work.top.push(player(EffectKind::NeowsLament));
                }
            }
        },
        RelicId::HolyWater|RelicId::TeardropLocket|RelicId::DataDisk|RelicId::Vajra|RelicId::OddlySmoothStone
            |RelicId::BagOfPreparation|RelicId::BronzeScales|RelicId::BloodVial|RelicId::Anchor|RelicId::RingOfTheSnake
            |RelicId::CrackedCore|RelicId::PureWater|RelicId::BurningBlood|RelicId::BlackBlood=>{
            let hp=facts.hp;
            let kind = match (hook, id) {
                (Hook::CombatStartedPreDraw,RelicId::HolyWater) if context.m22_profile()||context.complete_items()=>Some(EffectKind::Watcher(crate::combat::cards::watcher::WatcherEffect::CreateRequest{id:crate::content::CardId::Miracle,count:3})),
                (Hook::CombatStarted,RelicId::TeardropLocket) if context.m21_profile()||context.complete_items()=>Some(EffectKind::Character(crate::combat::CharacterEffect::Stance(crate::combat::Stance::Calm))),
                (Hook::CombatStarted, RelicId::DataDisk) => Some(EffectKind::Character(crate::combat::CharacterEffect::Focus(1))),
                (Hook::CombatStarted, RelicId::Vajra) => Some(EffectKind::ApplyFullPower { id:crate::combat::FullPowerId("strength"),amount:1 }),
                (Hook::CombatStarted, RelicId::OddlySmoothStone) => Some(EffectKind::ApplyFullPower { id:crate::combat::FullPowerId("dexterity"),amount:1 }),
                (Hook::CombatStarted, RelicId::BagOfPreparation) => Some(EffectKind::Draw(2)),
                (Hook::CombatStarted, RelicId::BronzeScales) => Some(EffectKind::ApplyFullPower { id:crate::combat::FullPowerId("thorns"),amount:3 }),
                (Hook::CombatStarted, RelicId::BloodVial) => Some(EffectKind::Heal(2)),
                (Hook::CombatStarted, RelicId::Anchor) => Some(EffectKind::GainBlock(10)),
                (Hook::CombatStarted, RelicId::RingOfTheSnake) => Some(EffectKind::Draw(2)),
                (Hook::CombatStarted|Hook::CombatStartedPreDraw, RelicId::PureWater) if context.is_pre_draw_visit(Some(hook)) => Some(EffectKind::CreateCard {
                    id: crate::content::CardId::Miracle,
                    destination: crate::combat::CardDestination::Hand,
                    upgrades: 0,
                }),
                (Hook::CombatSettled, RelicId::BurningBlood) if hp > 0&&!context.owns_relic(RelicId::BlackBlood)
                    &&!matches!(facts.source,Some(Source::Potion{id:crate::content::PotionId::SmokeBomb,..})) => Some(EffectKind::Heal(6)),
                (Hook::CombatSettled, RelicId::BlackBlood) if hp>0
                    &&!matches!(facts.source,Some(Source::Potion{id:crate::content::PotionId::SmokeBomb,..})) => Some(EffectKind::Heal(12)),
                _ => None,
            };
            if let Some(kind) = kind {
                if matches!(id,RelicId::Anchor|RelicId::BagOfPreparation|RelicId::RingOfTheSnake) { work.bottom.push(player(kind)); }
                else { work.top.push(player(kind)); }
            }
        }
        // Battle-start enemy debuffs (formerly a separate front-pushed pass; G4).
        RelicId::RedMask|RelicId::BagOfMarbles=>if hook==Hook::CombatStarted {
            let power=if id==RelicId::RedMask{"weak"}else{"vulnerable"};
            work.bottom.extend(facts.live_enemies.iter().map(|owner|Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId(power),amount:1}}));
        },
        _=>return Ok(false),
    }
    Ok(true)
}

/// Apply the native acquisition/equipment child for relics whose effect is
/// defined at obtain time. RunState is the sole owner outside active combat.
pub(super) fn acquire(state: &mut RunState, id: RelicId) -> Result<(), RunError> {
    if id==RelicId::HolyWater{return replace_pure_water(state)}
    if id==RelicId::FrozenCore{return replace_cracked_core(state)}
    if matches!(id,RelicId::SymbioticVirus|RelicId::Inserter|RelicId::NuclearBattery|RelicId::RunicCapacitor)
        &&(state.character!=crate::content::Character::Defect||state.relics.contains(&id)){return Err(RunError::InvalidAction)}
    if id==RelicId::BlackBlood{return replace_burning_blood(state)}
    if id==RelicId::RingOfTheSerpent{return replace_ring_of_the_snake(state)}
    if matches!(id,RelicId::EmptyCage|RelicId::SlaversCollar|RelicId::RedMask)&&state.relics.contains(&id){return Err(RunError::InvalidAction)}
    if matches!(id,RelicId::NinjaScroll|RelicId::PaperKrane|RelicId::TheSpecimen|RelicId::Tingsha|RelicId::ToughBandages
        |RelicId::TwistedFunnel|RelicId::HoveringKite|RelicId::WristBlade)&&state.relics.contains(&id){return Err(RunError::InvalidAction)}
    if state.combat.is_some() { return Err(RunError::InvalidAction); }
    if crate::content::Catalog::standard().relic(id).is_none() {
        return Err(RunError::Registry("unknown relic"));
    }
    if matches!(id,RelicId::RedSkull|RelicId::PaperPhrog|RelicId::ChampionsBelt|RelicId::CharonsAshes|RelicId::SelfFormingClay)
        &&state.character!=crate::content::Character::Ironclad{return Err(RunError::InvalidAction)}
    if matches!(id,RelicId::NinjaScroll|RelicId::PaperKrane|RelicId::TheSpecimen|RelicId::Tingsha|RelicId::ToughBandages
        |RelicId::TwistedFunnel|RelicId::HoveringKite|RelicId::RingOfTheSerpent|RelicId::WristBlade)
        &&state.character!=crate::content::Character::Silent{return Err(RunError::InvalidAction)}
    let max_hp_gain = match id {
        RelicId::Strawberry => 7,
        RelicId::Pear => 10,
        RelicId::Mango => 14,
        _ => 0,
    };
    if state.hp > state.max_hp { return Err(RunError::InvalidAction); }
    let max_hp = state.max_hp.checked_add(max_hp_gain).filter(|max|*max<=999)
        .ok_or(RunError::Unsupported("maximum HP overflow"))?;
    let hp = if state.hp == 0 {
        // increaseMaxHp still raises the ceiling, while the delegated heal
        // returns without reviving a dying creature.
        0
    } else {
        let heal = super::ordinary_heal_amount(&state.relics, max_hp_gain as i32).max(0) as u32;
        state.hp.checked_add(heal)
            .ok_or(RunError::Unsupported("current HP overflow"))?.min(max_hp)
    };

    state.relics.push(id);
    if id==RelicId::NeowsLament{state.relic_counters.insert(id,3);}
    if id==RelicId::Inserter{state.relic_counters.insert(id,0);}
    state.max_hp = max_hp;
    let hp_before=state.hp;
    state.hp = hp;
    if max_hp_gain>0 {crate::combat::observe::run_heal(state,hp_before);}
    Ok(())
}

/// Obtain-time equipment owned by the accepted relic handler.
pub(super) fn equip(id:RelicId,state:&mut RunState,work:&mut EquipWork)->Result<bool,RunError>{
    match id {
        RelicId::PotionBelt=>{
            if state.potions.len()!=3{return Err(RunError::InvalidAction)}
            state.potions.extend([None,None]);
            Ok(true)
        }
        RelicId::Circlet=>{
            state.relic_counters.insert(RelicId::Circlet,1);
            Ok(true)
        }
        RelicId::EmptyCage=>{
            let eligible=state.deck.iter().filter(|card|Catalog::standard().card(card.id)
                .is_some_and(|definition|definition.note_storage==NoteStorage::RemovablePermanent))
                .map(|card|card.instance).collect();
            work.select(SelectionSpec{eligible,count:2,cancellable:false});
            Ok(true)
        }
        _=>Ok(false),
    }
}

/// Complete an accepted relic's obtain-time selection.
pub(super) fn selection_done(id:RelicId,state:&mut RunState,chosen:&[u64])->Result<bool,RunError>{
    if id!=RelicId::EmptyCage{return Ok(false)}
    if chosen.len()>2||chosen.iter().any(|instance|!state.deck.iter().any(|card|card.instance==*instance)){
        return Err(RunError::InvalidAction)
    }
    for instance in chosen { crate::run::mutations::remove_card(state,*instance)?; }
    Ok(true)
}

pub(super) fn replace_burning_blood(state:&mut RunState)->Result<(),RunError>{
    if state.combat.is_some()||state.relics.first()!=Some(&RelicId::BurningBlood)||state.relics.contains(&RelicId::BlackBlood){return Err(RunError::InvalidAction)}
    state.relics[0]=RelicId::BlackBlood;state.relic_counters.remove(&RelicId::BurningBlood);Ok(())
}

pub(super) fn replace_ring_of_the_snake(state:&mut RunState)->Result<(),RunError>{
    if state.combat.is_some()||state.character!=crate::content::Character::Silent
        ||state.relics.first()!=Some(&RelicId::RingOfTheSnake)
        ||state.relics.contains(&RelicId::RingOfTheSerpent){return Err(RunError::InvalidAction)}
    state.relics[0]=RelicId::RingOfTheSerpent;
    state.relic_counters.remove(&RelicId::RingOfTheSnake);
    Ok(())
}

fn replace_cracked_core(state:&mut RunState)->Result<(),RunError>{
    if state.combat.is_some()||state.character!=crate::content::Character::Defect||state.relics.contains(&RelicId::FrozenCore)
        ||state.relics.iter().filter(|id|**id==RelicId::CrackedCore).count()!=1{return Err(RunError::InvalidAction)}
    let slot=state.relics.iter().position(|id|*id==RelicId::CrackedCore).unwrap();
    state.relics[slot]=RelicId::FrozenCore;state.relic_counters.remove(&RelicId::CrackedCore);Ok(())
}

/// Boss obtain replaces the canonical starter slot and equips the new relic once.
fn replace_pure_water(state:&mut RunState)->Result<(),RunError>{
    if state.combat.is_some()||state.active_combat.is_some()||state.character!=crate::content::Character::Watcher
        ||state.relics.first()!=Some(&RelicId::PureWater)||state.relics.contains(&RelicId::HolyWater)
        ||state.relics.iter().filter(|id|**id==RelicId::PureWater).count()!=1{return Err(RunError::InvalidAction)}
    state.relics[0]=RelicId::HolyWater;
    state.relic_counters.remove(&RelicId::PureWater);
    Ok(())
}
