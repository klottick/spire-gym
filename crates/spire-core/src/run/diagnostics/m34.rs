//! M34b diagnostics (env traversal tests): standalone, implies no other diagnostics feature, so enabling it for
//! spire-env tests does not change which core tests a default workspace build runs (the M28 default-build gate).
use crate::content::Character;
use crate::run::{CreationProfile, Run, RunConfig, RunError, RunRng, RoomScenario};

/// A Standard Run built through the public constructor with an explicit creation profile (seen bosses, Neow, Note), so an env
/// session's Standard creation and its supplied Run agree. Supplied: current/maximum HP `max_hp` and, when `keys`, all keys.
pub fn standard_supplied_run_with(character: Character, ascension: u8, profile: CreationProfile, max_hp: u32, keys: bool, rng: RunRng) -> Result<Run, RunError> {
    let mut run = Run::new_room_scenario(RunConfig { character, ascension }, profile, rng, RoomScenario::Standard)?;
    supply_hp_and_keys(&mut run, max_hp, keys);
    Ok(run)
}

/// M36 (design §3.1, I1) supplied start on an existing Run, e.g. the public seed world of `reset(Some(s))`: current and
/// maximum HP `max_hp` and, when `keys`, all three keys. Nothing else changes.
pub fn supply_hp_and_keys(run: &mut Run, max_hp: u32, keys: bool) {
    run.state.max_hp = max_hp;
    run.state.hp = max_hp;
    if keys { run.state.keys = [true; 3]; }
}

/// M34b supplied frontier for env traversals (same grade as the M28/M33 constructed lethal hit, never a policy
/// claim): one fixed hit on every living enemy of the current combat that leaves it at 1 HP, through the shared
/// resolver (so Invincible, Intangible and Buffer still apply). The policy's next real attack finishes it.
/// M36: the hit is NORMAL damage, so a damage increase (The Boot, Wrath, Vulnerable) could make it lethal and decide the
/// combat outside a step. The hit is tried on a copy first; a hit that would kill its target is halved until every
/// target survives (and dropped at 0), so the hook never kills. Without such an increase the first try is committed
/// unchanged.
pub fn supplied_wound_enemies(run: &mut Run) -> Result<(), RunError> {
    use crate::combat::{Actor, DamagePayload, Effect, EffectKind, Source};
    let combat = run.state.combat.as_mut().ok_or(RunError::InvalidAction)?;
    let view = combat.public_full().map_err(|_| RunError::InvalidAction)?;
    if view.outcome.is_some() { return Ok(()) }
    let mut hits = view.enemies.iter().filter(|e| e.hp > 1 && !e.replaced && !e.escaped && !e.half_dead)
        .map(|e| (e.instance_id.clone(), e.hp - 1)).collect::<Vec<_>>();
    loop {
        hits.retain(|(_, amount)| *amount > 0);
        if hits.is_empty() { return Ok(()) }
        let effects = hits.iter().map(|(id, amount)| Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(id.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(*amount)) })
            .collect::<Vec<_>>();
        let mut trial = combat.clone();
        trial.enqueue_native_effects(effects, None)?;
        let after = trial.public_full().map_err(|_| RunError::InvalidAction)?;
        let killed = |id: &crate::combat::EnemyHandle| after.enemies.iter().any(|e| &e.instance_id == id && e.hp <= 0 && !e.replaced && !e.escaped);
        if after.outcome.is_none() && !hits.iter().any(|(id, _)| killed(id)) { *combat = trial; return Ok(()) }
        let decided = after.outcome.is_some();
        for (id, amount) in &mut hits { if decided || killed(id) { *amount /= 2; } }
    }
}

/// M34s-b diagnostics (search fault isolation tests): a Run fault retained now, so the next step reports it (never a
/// policy or gameplay path).
pub fn supplied_fault(run: &mut Run) {
    use crate::run::{RunFault, RunFaultKind};
    run.state.fault = Some(RunFault { kind: RunFaultKind::Invariant("supplied diagnostic fault"), checkpoint: "m34s.supplied-fault", committed_events: vec![] });
}

/// M34s-b final review I1 diagnostics (env P1 duplicate-copy families): the live combat, to create supplied copies
/// (`Combat::diagnostic_create_card`) and build public-equivalent worlds (`Combat::diagnostic_swap_identical_copies`).
pub fn supplied_combat(run: &mut Run) -> Option<&mut crate::combat::Combat> { run.state.combat.as_mut() }
