//! Ironclad relic-owned player-turn and positive HP-loss callbacks.

use crate::{
    combat::{Actor, Effect, EffectKind, Hook, Source},
    content::{CardId, RelicId},
};

/// Combat phase work of Mark of Pain, Brimstone and Runic Cube (bodies unchanged).
pub(super) fn phase(id:RelicId,_context:&mut super::CombatItemContext,phase:super::dispatch::ItemPhase,
    facts:&super::dispatch::PhaseFacts<'_>,work:&mut super::RelicHookWork)->Result<bool,crate::combat::InputError>{
    use super::dispatch::ItemPhase;
    match id{
        RelicId::MarkOfPain=>if facts.hook==Some(Hook::CombatStarted){
            for _ in 0..2{work.bottom.push(Effect{source:Source::None,target:Some(Actor::Player),
                kind:EffectKind::CreateCard{id:CardId::Wound,destination:crate::combat::CardDestination::ShuffledDraw,upgrades:0}});}
        },
        RelicId::Brimstone=>if facts.hook==Some(Hook::TurnStartedPreDraw){
            // Monster Strength first in reverse encounter order (top insertion), then the
            // player's; the live subset is the caller's snapshot of the exact formation.
            if facts.target==Some(&Actor::Player){
                work.top.extend(facts.live_enemies.iter().rev().map(|owner|Effect{
                    source:Source::None,target:Some(Actor::Enemy(owner.clone())),
                    kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("strength"),amount:1}}));
            }
            work.top.push(Effect{source:Source::None,target:Some(Actor::Player),
                kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("strength"),amount:2}});
        },
        RelicId::RunicCube=>if phase==ItemPhase::HpLost&&facts.hp_lost>0{
            work.top.push(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)});
        },
        _=>return Ok(false),
    }
    Ok(true)
}
