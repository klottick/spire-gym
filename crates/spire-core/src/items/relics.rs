use crate::{
    combat::{Actor, Effect, EffectKind, Hook, Source},
    content::RelicId,
    run::{RunError, RunState},
};

use super::CombatItemContext;

pub(super) fn on_run_hook(state: &mut RunState, hook: Hook) -> Result<(), RunError> {
    if hook == Hook::CombatSettled && state.hp > 0 && (state.relics.contains(&RelicId::BurningBlood)||state.relics.contains(&RelicId::BlackBlood)) {
        let base=if state.relics.contains(&RelicId::BlackBlood){12}else{6};
        let heal = super::ordinary_heal_amount(&state.relics, base).max(0) as u32;
        state.hp = state.hp.saturating_add(heal).min(state.max_hp);
    }
    Ok(())
}

pub(super) fn on_combat_hook(context: &mut CombatItemContext, hook: Hook, hp: i32)
    -> super::RelicHookWork
{
    let mut work = super::RelicHookWork::default();
    if hook == Hook::CombatStarted {
        context.lantern_armed = context.owns_relic(RelicId::Lantern);
        if context.owns_relic(RelicId::NeowsLament) {
            if let Some(counter)=context.counters.get_mut(&RelicId::NeowsLament) {
                if *counter>0 {
                    *counter=if *counter==1{-2}else{*counter-1};
                    work.top.push(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::NeowsLament});
                }
            }
        }
    }
    if hook == Hook::TurnStarted && context.lantern_armed {
        context.lantern_armed = false;
        work.top.push(Effect { source: Source::None, target: Some(Actor::Player),
            kind: EffectKind::GainEnergy(1) });
    }
    // Emit top-queued work before Anchor's bottom-queued Block regardless of
    // relic acquisition order.
    for anchor_pass in [false, true] {
        for relic in context.relics() {
            if (*relic == RelicId::Anchor) != anchor_pass { continue; }
            let kind = match (hook, *relic) {
                (Hook::CombatStarted, RelicId::Vajra) => Some(EffectKind::ApplyFullPower { id:crate::combat::FullPowerId("strength"),amount:1 }),
                (Hook::CombatStarted, RelicId::OddlySmoothStone) => Some(EffectKind::ApplyFullPower { id:crate::combat::FullPowerId("dexterity"),amount:1 }),
                (Hook::CombatStarted, RelicId::BagOfPreparation) => Some(EffectKind::Draw(2)),
                (Hook::CombatStarted, RelicId::BronzeScales) => Some(EffectKind::ApplyFullPower { id:crate::combat::FullPowerId("thorns"),amount:3 }),
                (Hook::CombatStarted, RelicId::BloodVial) => Some(EffectKind::Heal(2)),
                (Hook::CombatStarted, RelicId::Anchor) => Some(EffectKind::GainBlock(10)),
                (Hook::CombatStarted, RelicId::RingOfTheSnake) => Some(EffectKind::Draw(2)),
                (Hook::CombatStarted, RelicId::CrackedCore) => Some(EffectKind::Character(
                    crate::combat::CharacterEffect::Channel(crate::combat::OrbKind::Lightning))),
                (Hook::CombatStarted, RelicId::PureWater) => Some(EffectKind::CreateCard {
                    id: crate::content::CardId::Miracle,
                    destination: crate::combat::CardDestination::Hand,
                    upgrades: 0,
                }),
                (Hook::CombatSettled, RelicId::BurningBlood) if hp > 0&&!context.owns_relic(RelicId::BlackBlood) => Some(EffectKind::Heal(6)),
                (Hook::CombatSettled, RelicId::BlackBlood) if hp>0=>Some(EffectKind::Heal(12)),
                _ => None,
            };
            if let Some(kind) = kind {
                let effect = Effect { source: Source::None, target: Some(Actor::Player), kind };
                if *relic == RelicId::Anchor { work.bottom.push(effect); }
                else { work.top.push(effect); }
            }
        }
    }
    work
}

/// Apply the native acquisition/equipment child for relics whose effect is
/// defined at obtain time. RunState is the sole owner outside active combat.
pub(super) fn acquire(state: &mut RunState, id: RelicId) -> Result<(), RunError> {
    if id==RelicId::BlackBlood{return replace_burning_blood(state)}
    if matches!(id,RelicId::EmptyCage|RelicId::SlaversCollar|RelicId::RedMask)&&state.relics.contains(&id){return Err(RunError::InvalidAction)}
    if state.combat.is_some() { return Err(RunError::InvalidAction); }
    if crate::content::Catalog::standard().relic(id).is_none() {
        return Err(RunError::Registry("unknown relic"));
    }
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
    state.max_hp = max_hp;
    state.hp = hp;
    Ok(())
}

pub(super) fn battle_start_enemy_effects(context:&CombatItemContext,targets:&[crate::combat::EnemyHandle])->Vec<Effect>{
    let mut work=vec![];
    for relic in context.relics(){
        let id=match relic{RelicId::RedMask=>"weak",RelicId::BagOfMarbles=>"vulnerable",_=>continue};
        work.extend(targets.iter().map(|owner|Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId(id),amount:1}}));
    }
    work
}

pub(super) fn replace_burning_blood(state:&mut RunState)->Result<(),RunError>{
    if state.combat.is_some()||state.relics.first()!=Some(&RelicId::BurningBlood)||state.relics.contains(&RelicId::BlackBlood){return Err(RunError::InvalidAction)}
    state.relics[0]=RelicId::BlackBlood;state.relic_counters.remove(&RelicId::BurningBlood);Ok(())
}
