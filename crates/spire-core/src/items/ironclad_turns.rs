//! Ironclad relic-owned player-turn and positive HP-loss callbacks.

use crate::{
    combat::{Actor, Effect, EffectKind, Hook, Source},
    content::{CardId, RelicId},
};

/// Ordered item work for the existing full-combat lifecycle checkpoints.
pub(super) fn relic_hook(context:&super::CombatItemContext,hook:Hook)->Vec<Effect>{
    let mut effects=Vec::new();
    match (hook,context.owns_relic(RelicId::MarkOfPain)) {
        (Hook::CombatStarted,true)=>{
            effects.push(Effect{source:Source::None,target:Some(Actor::Player),
                kind:EffectKind::CreateCard{id:CardId::Wound,destination:crate::combat::CardDestination::ShuffledDraw,upgrades:0}});
            effects.push(Effect{source:Source::None,target:Some(Actor::Player),
                kind:EffectKind::CreateCard{id:CardId::Wound,destination:crate::combat::CardDestination::ShuffledDraw,upgrades:0}});
        }
        _=>(),
    }
    if hook==Hook::TurnStartedPreDraw&&context.owns_relic(RelicId::Brimstone){
        effects.push(Effect{source:Source::None,target:Some(Actor::Player),
            kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("strength"),amount:2}});
    }
    effects
}

pub(super) fn brimstone_enemy_work(context:&super::CombatItemContext,hook:Hook,
    enemies:&[crate::combat::EnemyHandle])->Vec<Effect>{
    if hook!=Hook::TurnStartedPreDraw||!context.owns_relic(RelicId::Brimstone){return Vec::new()}
    // `enemies` is the live subset of the exact formation at this checkpoint;
    // the caller owns the snapshot so this module does not create a second state.
    enemies.iter().map(|owner|Effect{
        source:Source::None,target:Some(Actor::Enemy(owner.clone())),
        kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("strength"),amount:1},
    }).collect()
}

pub(super) fn runic_cube_draw(context:&super::CombatItemContext,positive_hp_lost:i32)->Option<Effect>{
    (positive_hp_lost>0&&context.owns_relic(RelicId::RunicCube)).then_some(Effect{
        source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)})
}
