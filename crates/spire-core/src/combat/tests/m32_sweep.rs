//! M32 admission and sweep (plan items 3 and 9; design §6.2, §7): no playable ceiling remains, so
//! every encounter setup enters at A0–A20 and A21 stays the typed `OUT_OF_RANGE`.
use super::super::enemies::{self, ascension};
use super::super::*;
use super::m29_a2_owner::setup;
use crate::content::{EncounterId, EnemyId};
use crate::run::RunError;

#[test]
fn m32_every_setup_admitted_a0_to_a20() {
    for (encounter, _, _) in enemies::A0_ENCOUNTER_SETUPS {
        for a in 0..=20u8 {
            assert_eq!(ascension::encounter_admission(*encounter, a), Ok(()), "{encounter:?} A{a}");
            assert!(enemies::setup_supports_ascension(*encounter, a), "{encounter:?} A{a}");
            for species in ascension::encounter_species(*encounter) {
                assert_eq!(ascension::species_admission(species, a), Ok(()), "{encounter:?} {species:?} A{a}");
            }
            let combat = Combat::enter(setup(*encounter, a, a + 1, u64::from(a)).unwrap_or_else(|e| panic!("{encounter:?} A{a}: {e:?}")))
                .unwrap_or_else(|e| panic!("{encounter:?} A{a}: {e:?}"));
            assert_eq!(combat.state.ascension, a);
        }
        assert_eq!(ascension::encounter_admission(*encounter, 21), Err(ascension::OUT_OF_RANGE), "{encounter:?}");
        assert_eq!(setup(*encounter, 21, 1, 0).err(), Some(RunError::Unsupported(ascension::OUT_OF_RANGE)), "{encounter:?}");
    }
    // A solo summoner or splitter enters at A17–A20 too (its spawn closure is admitted).
    for (encounter, owner) in [(EncounterId::SlimeBoss, EnemyId::SlimeBoss), (EncounterId::GremlinLeader, EnemyId::GremlinLeader),
        (EncounterId::Collector, EnemyId::Collector), (EncounterId::BronzeAutomaton, EnemyId::BronzeAutomaton)] {
        for a in 17..=20u8 {
            let mut entry = setup(encounter, 0, 1, 0).unwrap();
            entry.setup.enemies.retain(|enemy| enemy.enemy_id == owner);
            for enemy in &mut entry.setup.enemies { enemy.ascension = a; }
            let combat = Combat::enter(entry).unwrap_or_else(|e| panic!("{owner:?} A{a}: {e:?}"));
            assert_eq!(combat.state.ascension, a);
        }
    }
}

fn m32_rng_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn m32_assert_combat_ready(combat: &Combat, context: &str) {
    assert!(combat.state.fault.is_none(), "{context}: {:?}", combat.state.fault);
    engine::validate_runtime(&combat.state).unwrap_or_else(|error| panic!("{context}: {error:?}"));
    combat.public_full().unwrap_or_else(|error| panic!("{context}: public view: {error:?}"));
}

fn m32_assert_public_actors(combat: &Combat, actors: &[EnemyHandle], context: &str) {
    let public = combat.public_full().unwrap_or_else(|error| panic!("{context}: public view: {error:?}"));
    for actor in actors {
        let enemy = public.enemies.iter().find(|enemy| &enemy.instance_id == actor)
            .unwrap_or_else(|| panic!("{context}: {actor:?} absent from public view"));
        assert!(!enemy.intent.as_str().is_empty(), "{context}: {actor:?} has a public intent");
    }
}

#[test]
fn m32_every_encounter_a0_to_a20_sweep() {
    let started = std::time::Instant::now();
    for (setup_index, (encounter, _, _)) in enemies::A0_ENCOUNTER_SETUPS.iter().enumerate() {
        for ascension in 0..=20u8 {
            for seed in 0..3u8 {
                let mut combat = Combat::enter(setup(*encounter, ascension, 0x32 + seed, u64::from(seed))
                    .unwrap_or_else(|error| panic!("{encounter:?} A{ascension} seed {seed} setup: {error:?}")))
                    .unwrap_or_else(|error| panic!("{encounter:?} A{ascension} seed {seed} enter: {error:?}"));
                let context = format!("{encounter:?} A{ascension} seed {seed}");
                let mut policy = 0x4D32_5357_4545_5000u64
                    ^ ((setup_index as u64) << 24)
                    ^ (u64::from(ascension) << 8)
                    ^ u64::from(seed);
                m32_assert_combat_ready(&combat, &context);
                for step in 0..120 {
                    if combat.state.outcome.is_some() { break; }
                    let legal = combat.legal_actions_full()
                        .unwrap_or_else(|error| panic!("{context} step {step} legal actions: {error:?}"));
                    if legal.is_empty() { break; }
                    let index = (m32_rng_next(&mut policy) % legal.len() as u64) as usize;
                    let action = legal[index].clone();
                    combat.step_full(&action)
                        .unwrap_or_else(|error| panic!("{context} step {step} {action:?}: {error:?}"));
                    m32_assert_combat_ready(&combat, &format!("{context} step {step} {action:?}"));
                }
            }
        }
    }
    eprintln!("m32_every_encounter_a0_to_a20_sweep elapsed={:.3}s", started.elapsed().as_secs_f64());
}

#[test]
fn m32_threshold_paths_a17_to_a20() {
    for ascension in 17..=20u8 {
        let context = |path: &str| format!("{path} A{ascension}");

        // Slime Boss: cross the 50% HP split with a small fixed hit, leaving its two actors alive.
        let mut combat = Combat::enter(setup(EncounterId::SlimeBoss, ascension, 0x41, u64::from(ascension)).unwrap()).unwrap();
        let boss = combat.state.enemies.iter().position(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::SlimeBoss)).unwrap();
        let owner = combat.state.enemies[boss].instance_id.clone();
        combat.state.enemies[boss].hp = combat.state.enemies[boss].max_hp / 2 + 1;
        super::m5_elites::hit(&mut combat, boss, 2);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        m32_assert_combat_ready(&combat, &context("Slime Boss split"));
        let children: Vec<_> = combat.state.enemies.iter().filter(|enemy| enemy.instance_id != owner && enemy.life == ActorLife::Active && enemy.hp > 0)
            .map(|enemy| enemy.instance_id.clone()).collect();
        assert!(children.len() >= 2, "{}: children after split", context("Slime Boss"));
        m32_assert_public_actors(&combat, &children, &context("Slime Boss children"));

        // Both large-slime species are selected from their real setup stream, then split at half HP.
        for large in [EnemyId::AcidSlimeL, EnemyId::SpikeSlimeL] {
            let entry = (0..=u8::MAX).find_map(|seed| {
                let entry = setup(EncounterId::LargeSlime, ascension, seed, u64::from(seed)).ok()?;
                entry.setup.enemies.iter().any(|enemy| enemy.enemy_id == large).then_some(entry)
            }).unwrap_or_else(|| panic!("{}: no Large Slime setup for {large:?}", context("Large Slime")));
            let mut combat = Combat::enter(entry).unwrap();
            let slot = combat.state.enemies.iter().position(|enemy| enemy.enemy_id == ActorEnemyId::Full(large)).unwrap();
            let owner = combat.state.enemies[slot].instance_id.clone();
            combat.state.enemies[slot].hp = combat.state.enemies[slot].max_hp / 2 + 1;
            super::m5_elites::hit(&mut combat, slot, 2);
            combat.step_full(&CombatAction::EndTurn).unwrap();
            m32_assert_combat_ready(&combat, &context("Large Slime split"));
            let children: Vec<_> = combat.state.enemies.iter().filter(|enemy| enemy.instance_id != owner && enemy.life == ActorLife::Active && enemy.hp > 0)
                .map(|enemy| enemy.instance_id.clone()).collect();
            assert!(children.len() >= 2, "{}: {large:?} children after split", context("Large Slime"));
            m32_assert_public_actors(&combat, &children, &context("Large Slime children"));
        }

        // Rally replaces the opening minions with two real Leader descendants.
        let mut combat = Combat::enter(setup(EncounterId::GremlinLeader, ascension, 0x42, u64::from(ascension)).unwrap()).unwrap();
        let leader = combat.state.enemies.iter().position(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::GremlinLeader)).unwrap();
        for slot in 0..combat.state.enemies.len() {
            if slot != leader { super::m5_elites::hit(&mut combat, slot, 999); }
        }
        let owner = combat.state.enemies[leader].instance_id.clone();
        let actor = Actor::Enemy(owner.clone());
        combat.enqueue_native_effects(vec![Effect { source: Source::Actor(actor.clone()), target: Some(actor), kind: EffectKind::RallyGremlins { remaining: 2 } }], None).unwrap();
        m32_assert_combat_ready(&combat, &context("Gremlin Leader Rally"));
        let summoned: Vec<_> = combat.state.enemies.iter().filter(|enemy| enemy.life == ActorLife::Active && enemy.hp > 0 && enemy.instance_id.as_str().contains("/spawn/"))
            .map(|enemy| enemy.instance_id.clone()).collect();
        assert_eq!(summoned.len(), 2, "{}: Rally descendants", context("Gremlin Leader"));
        m32_assert_public_actors(&combat, &summoned, &context("Gremlin Leader descendants"));

        // Collector and Automaton create their minions through their normal first turn.
        let mut combat = Combat::enter(setup(EncounterId::Collector, ascension, 0x43, u64::from(ascension)).unwrap()).unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        m32_assert_combat_ready(&combat, &context("Collector Torch Heads"));
        assert_eq!(combat.state.enemies.iter().filter(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::TorchHead) && enemy.life == ActorLife::Active && enemy.hp > 0).count(), 2,
            "{}: Torch Heads", context("Collector"));
        let heads: Vec<_> = combat.state.enemies.iter().filter(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::TorchHead) && enemy.life == ActorLife::Active && enemy.hp > 0)
            .map(|enemy| enemy.instance_id.clone()).collect();
        m32_assert_public_actors(&combat, &heads, &context("Torch Heads"));

        let mut combat = Combat::enter(setup(EncounterId::BronzeAutomaton, ascension, 0x44, u64::from(ascension)).unwrap()).unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        m32_assert_combat_ready(&combat, &context("Automaton orbs"));
        assert_eq!(combat.state.enemies.iter().filter(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::BronzeOrb) && enemy.life == ActorLife::Active && enemy.hp > 0).count(), 2,
            "{}: Orbs", context("Automaton"));
        let orbs: Vec<_> = combat.state.enemies.iter().filter(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::BronzeOrb) && enemy.life == ActorLife::Active && enemy.hp > 0)
            .map(|enemy| enemy.instance_id.clone()).collect();
        m32_assert_public_actors(&combat, &orbs, &context("Automaton Orbs"));

        // Reptomancer receives a fixed seed whose opening intent is its summon, then executes it.
        let entry = (0..=u8::MAX).find_map(|seed| {
            let entry = setup(EncounterId::Reptomancer, ascension, seed, u64::from(seed)).ok()?;
            entry.setup.enemies.iter().any(|enemy| enemy.enemy_id == EnemyId::Reptomancer && enemy.intent.as_str() == "repto_spawn").then_some(entry)
        }).unwrap_or_else(|| panic!("{}: no seeded Reptomancer opening summon", context("Reptomancer")));
        let mut combat = Combat::enter(entry).unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        m32_assert_combat_ready(&combat, &context("Reptomancer daggers"));
        assert!(combat.state.enemies.iter().filter(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::Dagger) && enemy.life == ActorLife::Active && enemy.hp > 0).count() >= 3,
            "{}: summoned Dagger", context("Reptomancer"));
        let daggers: Vec<_> = combat.state.enemies.iter().filter(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::Dagger) && enemy.life == ActorLife::Active && enemy.hp > 0)
            .map(|enemy| enemy.instance_id.clone()).collect();
        m32_assert_public_actors(&combat, &daggers, &context("Reptomancer Daggers"));

        // Time Eater's strict-below-half Haste and Champ's half-HP Anger are selected on the next turn.
        let mut combat = Combat::enter(setup(EncounterId::TimeEater, ascension, 0x45, u64::from(ascension)).unwrap()).unwrap();
        let slot = combat.state.enemies.iter().position(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::TimeEater)).unwrap();
        combat.state.enemies[slot].hp = combat.state.enemies[slot].max_hp / 2 + 1;
        super::m5_elites::hit(&mut combat, slot, 2);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        m32_assert_combat_ready(&combat, &context("Time Eater Haste"));
        assert_eq!(combat.state.enemies[slot].intent_id.as_str(), "time_haste", "{}", context("Time Eater"));

        let mut combat = Combat::enter(setup(EncounterId::Champ, ascension, 0x46, u64::from(ascension)).unwrap()).unwrap();
        let slot = combat.state.enemies.iter().position(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::Champ)).unwrap();
        combat.state.enemies[slot].hp = combat.state.enemies[slot].max_hp / 2 + 1;
        super::m5_elites::hit(&mut combat, slot, 2);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        m32_assert_combat_ready(&combat, &context("Champ Anger"));
        assert_eq!(combat.state.enemies[slot].intent_id.as_str(), "champ_anger", "{}", context("Champ"));

        // A Darkling and the Awakened One both pass through half-dead before the supplied revival turn.
        let mut combat = Combat::enter(setup(EncounterId::ThreeDarklings, ascension, 0x47, u64::from(ascension)).unwrap()).unwrap();
        let max_hp = combat.state.enemies[0].max_hp;
        super::m5_elites::hit(&mut combat, 0, max_hp);
        assert_eq!(combat.state.enemies[0].life, ActorLife::HalfDead, "{}: Darkling enters HalfDead", context("Darkling"));
        for _ in 0..2 { combat.step_full(&CombatAction::EndTurn).unwrap(); }
        m32_assert_combat_ready(&combat, &context("Darkling revive"));
        assert_eq!((combat.state.enemies[0].life, combat.state.enemies[0].hp), (ActorLife::Active, max_hp / 2), "{}", context("Darkling"));
        m32_assert_public_actors(&combat, &[combat.state.enemies[0].instance_id.clone()], &context("Darkling revived actor"));

        let mut combat = Combat::enter(setup(EncounterId::AwakenedOne, ascension, 0x48, u64::from(ascension)).unwrap()).unwrap();
        let slot = combat.state.enemies.iter().position(|enemy| enemy.enemy_id == ActorEnemyId::Full(EnemyId::AwakenedOne)).unwrap();
        let max_hp = combat.state.enemies[slot].max_hp;
        super::m5_elites::hit(&mut combat, slot, max_hp);
        assert_eq!(combat.state.enemies[slot].life, ActorLife::HalfDead, "{}: Awakened One enters HalfDead", context("Awakened One"));
        combat.step_full(&CombatAction::EndTurn).unwrap();
        m32_assert_combat_ready(&combat, &context("Awakened One rebirth"));
        assert_eq!((combat.state.enemies[slot].life, combat.state.enemies[slot].hp), (ActorLife::Active, max_hp), "{}", context("Awakened One"));
        m32_assert_public_actors(&combat, &[combat.state.enemies[slot].instance_id.clone()], &context("Awakened One reborn actor"));
    }
}
