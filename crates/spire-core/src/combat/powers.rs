//! Finite full-mode power operations on the existing actor-owned storage lanes.
use super::*;
use super::model::State;

fn lanes<'a>(state: &'a mut State, actor: &Actor) -> (&'a mut Vec<PowerEntry>, &'a mut Vec<FullPowerEntry>) {
    match actor {
        Actor::Player => (&mut state.player.powers, &mut state.player_full_powers),
        Actor::Enemy(id) => {
            let enemy = state.enemies.iter_mut().find(|e| &e.instance_id == id).expect("validated actor");
            (&mut enemy.powers, &mut enemy.full_powers)
        }
    }
}

fn amount(state: &State, actor: &Actor, id: FullPowerId) -> Result<i32, InputError> {
    let (legacy, full) = match actor {
        Actor::Player => (&state.player.powers, &state.player_full_powers),
        Actor::Enemy(owner) => {
            let enemy = state.enemies.iter().find(|e| &e.instance_id == owner).expect("validated actor");
            (&enemy.powers, &enemy.full_powers)
        }
    };
    legacy.iter().filter(|p| p.id.as_str() == id.0).map(|p| p.state.amount)
        .chain(full.iter().filter(|p| p.id == id).map(|p| p.state.amount))
        .try_fold(0_i32, |a, b| a.checked_add(b))
        .ok_or_else(|| InputError::Invariant("power sum overflow".into()))
}

fn remove(state: &mut State, actor: &Actor, id: FullPowerId) {
    let (legacy, full) = lanes(state, actor);
    legacy.retain(|p| p.id.as_str() != id.0);
    full.retain(|p| p.id != id);
}

fn set(state: &mut State, effect: &Effect, id: FullPowerId, value: i32) {
    let actor = effect.target.as_ref().expect("validated power target");
    let (legacy, full) = lanes(state, actor);
    if let Some(existing) = legacy.iter_mut().find(|p| p.id.as_str() == id.0) {
        existing.state.amount = value;
        full.retain(|p| p.id != id);
    } else if let Some(existing) = full.iter_mut().find(|p| p.id == id) {
        existing.state.amount = value;
    } else if let EffectKind::ApplyPower { id: legacy_id, .. } = effect.kind
        && legacy_id.as_str() == id.0 {
        legacy.push(PowerEntry { id: legacy_id, state: PowerState { amount: value,
            skip_first: legacy_id == PowerId::Ritual,
            monster_source_grace: legacy_id == PowerId::Vulnerable
                && matches!(effect.source, Source::Actor(Actor::Enemy(_))),
        }});
    } else {
        full.push(FullPowerEntry { id, state: PowerState { amount: value,
            skip_first: id.0 == "ritual",
            monster_source_grace: matches!(id.0, "weak" | "vulnerable" | "frail")
                && matches!(effect.source, Source::Actor(Actor::Enemy(_))),
        }});
    }
    if value == 0 {
        state.pending.push_front(Effect { source: effect.source.clone(), target: effect.target.clone(),
            kind: EffectKind::RemoveFullPower(id) });
    }
}

/// Only ordinary supported applications spend Artifact. Reductions/removals
/// reach this same owner without being reclassified as fresh applications.
pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), InputError> {
    let (id, delta, application) = match effect.kind {
        EffectKind::ApplyFullPower { id, amount } => (id, amount, true),
        EffectKind::ApplyPower { id, amount } => (FullPowerId(id.as_str()), amount, true),
        EffectKind::ReduceFullPower { id, amount } => (id, -amount, false),
        EffectKind::ReducePower { id, amount } => (FullPowerId(id.as_str()), -amount, false),
        EffectKind::RemoveFullPower(id) => {
            remove(state, effect.target.as_ref().unwrap(), id);
            return super::effects::refresh_enemy_values(state);
        }
        EffectKind::RemovePower(id) => {
            remove(state, effect.target.as_ref().unwrap(), FullPowerId(id.as_str()));
            return super::effects::refresh_enemy_values(state);
        }
        _ => return Err(InputError::Invariant("power owner received non-power work".into())),
    };
    let actor = effect.target.as_ref().unwrap();
    let alive = match actor {
        Actor::Player => state.player.hp > 0,
        Actor::Enemy(owner) => state.enemies.iter().any(|e| &e.instance_id == owner
            && e.life == ActorLife::Active && e.hp > 0),
    };
    if application && !alive { return Ok(()); }
    let debuff = (matches!(id.0, "weak" | "vulnerable" | "frail") && delta > 0)
        || (matches!(id.0, "strength" | "dexterity") && delta < 0);
    let artifact = amount(state, actor, FullPowerId("artifact"))?;
    if application && debuff && artifact > 0 {
        set(state, effect, FullPowerId("artifact"), artifact - 1);
        return Ok(());
    }
    let previous = amount(state, actor, id)?;
    if !application && previous == 0 { return Ok(()); }
    let sum = previous.checked_add(delta).ok_or_else(|| InputError::Invariant("power amount overflow".into()))?;
    let next = match id.0 {
        "strength" => sum.clamp(-999, 999),
        "dexterity" if (-999..=999).contains(&sum) => sum,
        "dexterity" => return Err(InputError::Unsupported("Dexterity outside supported native surface".into())),
        _ if sum <= 999 => sum.max(0),
        _ => return Err(InputError::Unsupported("power outside supported native surface".into())),
    };
    set(state, effect, id, next);
    super::effects::refresh_enemy_values(state)
}
