//! Silent relic callback owners. Work leaves this module as ordinary or typed
//! effects for the shared combat queue and resolver.
use crate::combat::{Actor, CardDestination, Effect, EffectKind, EnemyHandle, Hook,
    SilentItemEffect, Source};
use crate::content::{CardId, RelicId};

use super::{CombatItemContext, RelicHookWork};

pub(super) fn on_combat_hook(context: &mut CombatItemContext, hook: Hook) -> RelicHookWork {
    let mut work = RelicHookWork::default();
    if !context.m15_profile() { return work; }

    if hook == Hook::TurnStarted {
        context.kite_triggered = false;
    }
    if hook == Hook::CardManuallyDiscarded {
        let kite_eligible=context.owns_relic(RelicId::HoveringKite)&&!context.kite_triggered;
        if kite_eligible{context.kite_triggered=true;}
        for relic in context.relics() {
            match relic {
                RelicId::ToughBandages => work.top.push(Effect {
                    source: Source::Actor(Actor::Player), target: Some(Actor::Player),
                    kind: EffectKind::GainBlock(3),
                }),
                RelicId::Tingsha => work.top.push(Effect {
                    source: Source::Actor(Actor::Player), target: None,
                    kind: EffectKind::JuggernautRandomDamage(3),
                }),
                RelicId::HoveringKite if kite_eligible => {
                    work.top.push(Effect { source: Source::Actor(Actor::Player),
                        target: Some(Actor::Player), kind: EffectKind::GainEnergy(1) });
                }
                _ => (),
            }
        }
    }
    work
}

pub(super) fn battle_start_enemy_effects(context: &CombatItemContext,
    targets: &[EnemyHandle]) -> Vec<Effect> {
    if !context.m15_profile() || !context.owns_relic(RelicId::TwistedFunnel) {
        return Vec::new();
    }
    targets.iter().cloned().map(|owner| Effect {
        source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(owner)),
        kind: EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("poison"), amount: 4 },
    }).collect()
}

pub(super) fn pre_draw_work(context: &CombatItemContext) -> Vec<Effect> {
    if !context.m15_profile() || !context.owns_relic(RelicId::NinjaScroll) {
        return Vec::new();
    }
    (0..3).map(|_| Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::CreateCard {
            id: CardId::Shiv, destination: CardDestination::Hand, upgrades: 0,
        } }).collect()
}

pub(super) fn death_work(context: &CombatItemContext, dead_owner: EnemyHandle,
    callback_poison: Option<i32>, live_eligible_count: usize) -> Vec<Effect> {
    let Some(amount) = callback_poison.filter(|amount| *amount > 0) else { return Vec::new() };
    if !context.m15_profile() || !context.owns_relic(RelicId::TheSpecimen)
        || live_eligible_count == 0 {
        return Vec::new();
    }
    vec![Effect { source: Source::Actor(Actor::Player), target: None,
        kind: EffectKind::SilentItem(SilentItemEffect::SpecimenPoisonTransfer {
            callback_owner: dead_owner, amount,
        }) }]
}
