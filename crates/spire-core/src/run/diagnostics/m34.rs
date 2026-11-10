//! M34b diagnostics (env traversal tests): standalone, implies no other diagnostics feature, so enabling it for
//! spire-env tests does not change which core tests a default workspace build runs (the M28 default-build gate).
use crate::content::Character;
use crate::run::{CreationProfile, Run, RunConfig, RunError, RunRng, RoomScenario};

/// A Standard Run built through the public constructor with an explicit creation profile (seen bosses, Neow, Note), so an env
/// session's Standard creation and its supplied Run agree. Supplied: current/maximum HP `max_hp` and, when `keys`, all keys.
pub fn standard_supplied_run_with(character: Character, ascension: u8, profile: CreationProfile, max_hp: u32, keys: bool, rng: RunRng) -> Result<Run, RunError> {
    let mut run = Run::new_room_scenario(RunConfig { character, ascension }, profile, rng, RoomScenario::Standard)?;
    run.state.max_hp = max_hp;
    run.state.hp = max_hp;
    if keys { run.state.keys = [true; 3]; }
    Ok(run)
}

/// M34b supplied frontier for env traversals (same grade as the M28/M33 constructed lethal hit, never a policy
/// claim): one fixed hit on every living enemy of the current combat that leaves it at 1 HP, through the shared
/// resolver (so Invincible, Intangible and Buffer still apply). The policy's next real attack finishes it.
pub fn supplied_wound_enemies(run: &mut Run) -> Result<(), RunError> {
    use crate::combat::{Actor, DamagePayload, Effect, EffectKind, Source};
    let combat = run.state.combat.as_mut().ok_or(RunError::InvalidAction)?;
    let view = combat.public_full().map_err(|_| RunError::InvalidAction)?;
    if view.outcome.is_some() { return Ok(()) }
    let effects = view.enemies.iter().filter(|e| e.hp > 1 && !e.replaced && !e.escaped && !e.half_dead)
        .map(|e| Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(e.instance_id.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(e.hp - 1)) })
        .collect::<Vec<_>>();
    if effects.is_empty() { return Ok(()) }
    combat.enqueue_native_effects(effects, None)
}
