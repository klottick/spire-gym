//! Silent relic callback owners. Work leaves this module as ordinary or typed
//! effects for the shared combat queue and resolver.
use crate::combat::{Actor, CardDestination, Effect, EffectKind, EnemyHandle, Hook,
    SilentItemEffect, Source};
use crate::content::{CardId, RelicId};

use super::{CombatItemContext, RelicHookWork};

/// Combat phase work of the M15 Silent relics, one relic per call (bodies unchanged).
pub(super) fn phase(id: RelicId, context: &mut CombatItemContext, phase: super::dispatch::ItemPhase,
    facts: &super::dispatch::PhaseFacts<'_>, work: &mut RelicHookWork) -> Result<bool, crate::combat::InputError>
{
    use super::dispatch::ItemPhase;
    if !matches!(id,RelicId::ToughBandages|RelicId::Tingsha|RelicId::HoveringKite|RelicId::TwistedFunnel|RelicId::NinjaScroll|RelicId::TheSpecimen){return Ok(false)}
    if !(context.m15_profile()||context.complete_items()) { return Ok(true); }
    match (id,phase) {
        (RelicId::HoveringKite,_) if facts.hook==Some(Hook::TurnStarted) => context.kite_triggered = false,
        (RelicId::ToughBandages,_) if facts.hook==Some(Hook::CardManuallyDiscarded) => work.top.push(Effect {
            source: Source::Actor(Actor::Player), target: Some(Actor::Player), kind: EffectKind::GainBlock(3) }),
        (RelicId::Tingsha,_) if facts.hook==Some(Hook::CardManuallyDiscarded) => work.top.push(Effect {
            source: Source::Actor(Actor::Player), target: None, kind: EffectKind::JuggernautRandomDamage(3) }),
        (RelicId::HoveringKite,_) if facts.hook==Some(Hook::CardManuallyDiscarded)&&!context.kite_triggered => {
            context.kite_triggered=true;
            work.top.push(Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player), kind: EffectKind::GainEnergy(1) });
        }
        // Battle-start Poison (formerly a separate front-pushed pass after Red Mask/Marbles; G4).
        (RelicId::TwistedFunnel,_) if facts.hook==Some(Hook::CombatStarted) => work.top.extend(facts.live_enemies.iter().cloned().map(|owner| Effect {
            source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(owner)),
            kind: EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("poison"), amount: 4 } })),
        // Generated hand work at the combat-entry startup dispatch, before the opening hand.
        (RelicId::NinjaScroll,ItemPhase::CombatStartPreDraw) if facts.hook.is_none() => work.bottom.extend((0..3).map(|_| Effect {
            source: Source::Actor(Actor::Player), target: Some(Actor::Player),
            kind: EffectKind::CreateCard { id: CardId::Shiv, destination: CardDestination::Hand, upgrades: 0 } })),
        (RelicId::TheSpecimen,ItemPhase::EnemyDied) => if let Some((owner,poison,live))=facts.dead_enemy {
            work.top.extend(death_work(context,owner.clone(),poison,live));
        },
        _ => (),
    }
    Ok(true)
}

fn death_work(context: &CombatItemContext, dead_owner: EnemyHandle,
    callback_poison: Option<i32>, live_eligible_count: usize) -> Vec<Effect> {
    let Some(amount) = callback_poison.filter(|amount| *amount > 0) else { return Vec::new() };
    if !(context.m15_profile()||context.complete_items()) || !context.owns_relic(RelicId::TheSpecimen)
        || live_eligible_count == 0 {
        return Vec::new();
    }
    vec![Effect { source: Source::Actor(Actor::Player), target: None,
        kind: EffectKind::SilentItem(SilentItemEffect::SpecimenPoisonTransfer {
            callback_owner: dead_owner, amount,
        }) }]
}
