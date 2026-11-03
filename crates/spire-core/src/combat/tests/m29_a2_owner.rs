//! M29a2: one combat-level ascension owner, per-species ascension functions,
//! spawn closure and playable ceilings. Literals are the pre-fold A0/legacy
//! values (enemies/mod.rs on 943ec3ac) and inventory §3a ceilings.
use super::super::*;
use super::super::enemies::{self, ascension, EnemyMemory, EnemyProfile};
use crate::content::{Character, CardId, EncounterId, EnemyId};
use crate::run::RunError;

pub(super) fn context(visit: u64) -> enemies::SetupContext {
    enemies::SetupContext {
        player: PlayerState { hp: 999, max_hp: 999, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
        player_powers: vec![], character: Character::Ironclad,
        cards: (0..10).map(|i| permanent_live_card(if i < 5 { CardId::Strike } else { CardId::Defend }, CardInstanceId(900 + i), 0)).collect(),
        opening_draw: 5, encounter_visit: visit,
    }
}

fn act_of(encounter: EncounterId) -> u8 {
    enemies::A0_ENCOUNTER_SETUPS.iter().find(|(id, _, _)| *id == encounter).unwrap().1
}

pub(super) fn setup(encounter: EncounterId, ascension: u8, seed: u8, visit: u64) -> Result<CombatEntry, RunError> {
    enemies::setup(encounter, act_of(encounter), ascension,
        &mut crate::run::RunRng::new([seed; 32], rng::Partition::Test, 0), context(visit))
}

pub(super) fn ai_stream(actor: &str) -> rng::StreamState {
    let id = rng::StreamId::new(rng::Domain::RunCombatEnemyAi, vec!["m29a2".into(), actor.into()]).unwrap();
    rng::make_stream(&[29; 32], rng::Partition::Test, "m29a2", &id).unwrap()
}

#[test]
fn m29_combat_ascension_owner_single_source() {
    let combat = Combat::enter(setup(EncounterId::ThreeCultists, 2, 1, 0).unwrap()).unwrap();
    assert_eq!(combat.state.ascension, 2);
    assert!(format!("{:?}", combat.state).contains("ascension: 2"), "state text carries the combat ascension");
    assert!(combat.state.enemies.iter().all(|enemy| enemy.full_memory == Some(EnemyMemory::Cultist)),
        "no per-actor ascension copy remains after the fold");

    let mut entry = setup(EncounterId::ThreeCultists, 2, 1, 0).unwrap();
    entry.setup.enemies[1].ascension = 3;
    assert_eq!(Combat::enter(entry).err(), Some(RunError::Unsupported("combat enemies disagree on ascension")));
}

#[test]
fn m29_legacy_species_unchanged_after_fold() {
    let actor = |id: EnemyId, intent: FullIntentId| {
        let mut entry = setup(match id {
            EnemyId::Cultist => EncounterId::Cultist, EnemyId::JawWorm => EncounterId::JawWorm,
            EnemyId::GremlinNob => EncounterId::GremlinNob, EnemyId::FungiBeast => EncounterId::TwoFungiBeasts,
            _ => EncounterId::BlueSlaver,
        }, 0, 3, 0).unwrap();
        entry.setup.enemies.truncate(1);
        let mut enemy = Combat::enter(entry).unwrap().state.enemies.remove(0);
        enemy.intent_id = ActorIntentId::Full(intent);
        enemy
    };
    let amount = |work: Vec<Effect>, power: &str| work.iter().find_map(|effect| match &effect.kind {
        EffectKind::ApplyFullPower { id, amount } if id.0 == power => Some(*amount), _ => None }).unwrap();
    let block = |work: Vec<Effect>| work.iter().find_map(|effect| match effect.kind {
        EffectKind::GainBlock(amount) => Some(amount), _ => None }).unwrap();
    let base = |id: EnemyId, a: u8, intent: &'static str| enemies::damage_bases_at_ascension(id, a).unwrap()[&FullIntentId::from_name(intent)];
    // (A, Cultist Ritual, Jaw Bellow Str/Block, Jaw Chomp, Nob Enrage, Nob Skull Bash/Bull Rush, Fungi Grow,
    //  Slaver Stab/Rake/Weak, HP ranges Cultist/Jaw/Nob/Fungi/Slaver Blue)
    type Row = (u8, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, [(i32, i32); 5]);
    let rows: [Row; 10] = [
        (0, 3, 3, 6, 11, 2, 6, 14, 3, 12, 7, 1, [(48, 54), (40, 44), (82, 86), (22, 28), (46, 50)]),
        (1, 3, 3, 6, 11, 2, 6, 14, 3, 12, 7, 1, [(48, 54), (40, 44), (82, 86), (22, 28), (46, 50)]),
        (2, 4, 4, 6, 12, 2, 6, 14, 4, 13, 8, 1, [(48, 54), (40, 44), (82, 86), (22, 28), (46, 50)]),
        (3, 4, 4, 6, 12, 2, 8, 16, 4, 13, 8, 1, [(48, 54), (40, 44), (82, 86), (22, 28), (46, 50)]),
        (6, 4, 4, 6, 12, 2, 8, 16, 4, 13, 8, 1, [(48, 54), (40, 44), (82, 86), (22, 28), (46, 50)]),
        (7, 4, 4, 6, 12, 2, 8, 16, 4, 13, 8, 1, [(50, 56), (42, 46), (82, 86), (24, 28), (48, 52)]),
        (8, 4, 4, 6, 12, 2, 8, 16, 4, 13, 8, 1, [(50, 56), (42, 46), (85, 90), (24, 28), (48, 52)]),
        (16, 4, 4, 6, 12, 2, 8, 16, 4, 13, 8, 1, [(50, 56), (42, 46), (85, 90), (24, 28), (48, 52)]),
        (17, 5, 5, 9, 12, 2, 8, 16, 5, 13, 8, 2, [(50, 56), (42, 46), (85, 90), (24, 28), (48, 52)]),
        (18, 5, 5, 9, 12, 3, 8, 16, 5, 13, 8, 2, [(50, 56), (42, 46), (85, 90), (24, 28), (48, 52)]),
    ];
    for (a, ritual, jaw_str, jaw_block, chomp, enrage, bash, rush, grow, stab, rake, weak, hp) in rows {
        assert_eq!(amount(enemies::move_work_at(&actor(EnemyId::Cultist, FullIntentId::Incantation), a).unwrap(), "ritual"), ritual, "A{a}");
        let jaw = actor(EnemyId::JawWorm, FullIntentId::Bellow);
        assert_eq!(amount(enemies::move_work_at(&jaw, a).unwrap(), "strength"), jaw_str, "A{a}");
        assert_eq!(block(enemies::move_work_at(&jaw, a).unwrap()), jaw_block, "A{a}");
        assert_eq!(base(EnemyId::JawWorm, a, "chomp"), chomp, "A{a}");
        assert_eq!(amount(enemies::move_work_at(&actor(EnemyId::GremlinNob, FullIntentId::Bellow), a).unwrap(), "enrage"), enrage, "A{a}");
        assert_eq!((base(EnemyId::GremlinNob, a, "skull_bash"), base(EnemyId::GremlinNob, a, "bull_rush")), (bash, rush), "A{a}");
        assert_eq!(amount(enemies::move_work_at(&actor(EnemyId::FungiBeast, FullIntentId::Grow), a).unwrap(), "strength"), grow, "A{a}");
        assert_eq!((base(EnemyId::SlaverBlue, a, "slaver_stab"), base(EnemyId::SlaverBlue, a, "slaver_rake")), (stab, rake), "A{a}");
        let mut slaver = actor(EnemyId::SlaverBlue, FullIntentId::from_name("slaver_rake"));
        let mut state = Combat::enter(setup(EncounterId::BlueSlaver, a, 3, 0).unwrap()).unwrap().state;
        slaver.instance_id = state.enemies[0].instance_id.clone();
        state.enemies[0] = slaver;
        let owner = state.enemies[0].instance_id.clone();
        assert_eq!(amount(enemies::act2::take_turn(&mut state, &owner).unwrap(), "weak"), weak, "A{a}");
        for (id, range) in [EnemyId::Cultist, EnemyId::JawWorm, EnemyId::GremlinNob, EnemyId::FungiBeast, EnemyId::SlaverBlue].into_iter().zip(hp) {
            assert_eq!(enemies::hp_range(id, a), range, "{id:?} A{a}");
        }
    }
    // Nob's A18 sequence reads the combat value, not memory.
    let memory = EnemyMemory::GremlinNob;
    let history = [FullIntentId::from_name("bellow")];
    assert_eq!(enemies::select_move_at(EnemyId::GremlinNob, &history, &memory, &ai_stream("nob"), 18).unwrap().intent.as_str(), "skull_bash");
}

#[test]
fn m29_spawned_and_split_children_read_combat_ascension() {
    let handle = EnemyHandle::parse("slime/split/0").unwrap();
    let child = enemies::prepare_split_child(EnemyId::AcidSlimeM, handle.clone(), 30, ai_stream("slime"), 0).ok().unwrap();
    assert_eq!(child.full_damage_entries.iter().map(|(id, entry)| (id.as_str().to_owned(), entry.base)).collect::<Vec<_>>(),
        vec![("acid_tackle".to_owned(), 10), ("acid_spit".to_owned(), 7)], "A0 split child values unchanged");
    // M29b landed the Acid Slime M ladder: the child is built at the requested ascension (A2 bases).
    let child = enemies::prepare_split_child(EnemyId::AcidSlimeM, handle, 30, ai_stream("slime"), 2).ok().unwrap();
    assert_eq!(child.full_damage_entries.iter().map(|(id, entry)| (id.as_str().to_owned(), entry.base)).collect::<Vec<_>>(),
        vec![("acid_tackle".to_owned(), 12), ("acid_spit".to_owned(), 8)],
        "a split child is built at the combat ascension, never silently A0");

    // Summons: the Collector summons Torch Heads on its first turn.
    let at = |forced: Option<u8>| {
        let mut combat = Combat::enter(setup(EncounterId::Collector, 0, 5, 0).unwrap()).unwrap();
        if let Some(a) = forced { combat.state.ascension = a; }
        let result = combat.step_full(&CombatAction::EndTurn);
        (combat, result)
    };
    let (combat, result) = at(None);
    result.unwrap();
    let torches = combat.state.enemies.iter().filter(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::TorchHead)).collect::<Vec<_>>();
    assert_eq!(torches.len(), 2);
    assert!(torches.iter().all(|t| (38..=40).contains(&t.max_hp) && t.full_damage_entries[&FullIntentId::from_name("torch_attack")].base == 7));
    // M29d landed the Torch Head: forced to A9, the summons carry the A9 HP band.
    let (combat, result) = at(Some(9));
    result.unwrap();
    assert!(combat.state.fault.is_none(), "{:?}", combat.state.fault);
    let torches = combat.state.enemies.iter().filter(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::TorchHead)).collect::<Vec<_>>();
    assert_eq!(torches.len(), 2);
    assert!(torches.iter().all(|t| (40..=45).contains(&t.max_hp)), "summons read the combat ascension");
    // The Mad Gremlin's Angry and the Awakened One's rebirth HP are per-species functions (A0 here).
    assert_eq!((ascension::gremlin_mad_angry(0), ascension::awakened_rebirth_max_hp(0)), (1, 300));
}

#[test]
fn m29_spawn_closure_gates_encounter() {
    for (encounter, ceiling) in [
        (EncounterId::SlimeBoss, 16), (EncounterId::GremlinLeader, 16), (EncounterId::GremlinGang, 16),
        (EncounterId::DonuAndDeca, 18), (EncounterId::ShieldAndSpear, 17), (EncounterId::Slavers, 16),
        (EncounterId::ColosseumSlavers, 16), (EncounterId::ColosseumNobs, 17), (EncounterId::ExordiumThugs, 16),
        (EncounterId::ExordiumWildlife, 16), (EncounterId::BronzeAutomaton, 18), (EncounterId::Collector, 20),
        (EncounterId::Reptomancer, 20), (EncounterId::AwakenedOne, 20), (EncounterId::CorruptHeart, 20),
        (EncounterId::Cultist, 20), (EncounterId::BlueSlaver, 20), (EncounterId::TimeEater, 18),
    ] {
        assert_eq!(ascension::encounter_ceiling(encounter), ceiling, "{encounter:?}");
    }
    assert_eq!(ascension::playable_ceiling(EnemyId::SlimeBoss), 20);
    assert_eq!(ascension::encounter_admission(EncounterId::SlimeBoss, 17), Err(ascension::ABOVE_CEILING),
        "the children's ceiling bounds the Slime Boss encounter");
    for id in [EnemyId::AcidSlimeL, EnemyId::SpikeSlimeL, EnemyId::AcidSlimeM, EnemyId::SpikeSlimeM] {
        assert!(ascension::encounter_species(EncounterId::SlimeBoss).contains(&id));
    }
    for (encounter, child) in [(EncounterId::Collector, EnemyId::TorchHead), (EncounterId::BronzeAutomaton, EnemyId::BronzeOrb),
        (EncounterId::Reptomancer, EnemyId::Dagger), (EncounterId::GremlinLeader, EnemyId::GremlinWizard)] {
        assert!(ascension::encounter_species(encounter).contains(&child), "{encounter:?}");
    }
    // n6: every composition draw stays inside the static species set.
    for (encounter, _, _) in enemies::A0_ENCOUNTER_SETUPS {
        let species = ascension::encounter_species(*encounter);
        assert!(!species.is_empty(), "{encounter:?}");
        for visit in 0..48 {
            let entry = setup(*encounter, 0, (visit % 7) as u8, visit).unwrap();
            for enemy in &entry.setup.enemies {
                assert!(species.contains(&enemy.enemy_id), "{encounter:?} drew {:?}", enemy.enemy_id);
            }
        }
    }
}

#[test]
fn m29_playable_ceiling_rejects_above() {
    assert_eq!(setup(EncounterId::TwoLouse, 17, 1, 0).err(), Some(RunError::Unsupported(ascension::ABOVE_CEILING)));
    assert!(ascension::ABOVE_CEILING.contains("M32"));
    // M29b: Louse landed, so A16 is admitted. M29c: Slaver Red landed too; M29d: the Mugger.
    assert!(setup(EncounterId::TwoLouse, 16, 1, 0).is_ok());
    assert!(setup(EncounterId::RedSlaver, 16, 1, 0).is_ok());
    assert!(setup(EncounterId::TwoThieves, 16, 1, 0).is_ok());
    assert_eq!(setup(EncounterId::Cultist, 21, 1, 0).err(), Some(RunError::Unsupported(ascension::OUT_OF_RANGE)));
    assert!(setup(EncounterId::Cultist, 20, 1, 0).is_ok());
    assert!(!enemies::setup_supports_ascension(EncounterId::TwoLouse, 17));
    // A rejected setup consumes no setup RNG.
    let mut rng = crate::run::RunRng::new([9; 32], rng::Partition::Test, 0);
    let before = rng.clone();
    assert!(enemies::setup(EncounterId::TwoLouse, 1, 17, &mut rng, context(0)).is_err());
    assert_eq!(rng, before);
    // Entry admission applies the same predicate to a hand-built setup.
    let mut entry = setup(EncounterId::TwoLouse, 0, 1, 0).unwrap();
    for enemy in &mut entry.setup.enemies { enemy.ascension = 17; }
    assert_eq!(Combat::enter(entry).err(), Some(RunError::Unsupported(ascension::ABOVE_CEILING)));
    // Entry bounds a summoner or splitter by its spawn closure's ceiling (review v1 m1): a solo Slime
    // Boss (children 16) or Gremlin Leader (Wizard, Fat 16) is rejected up front at A17, although
    // each species' own ceiling is 20. M29c: both landed, so A16 is admitted.
    for (encounter, owner) in [(EncounterId::SlimeBoss, EnemyId::SlimeBoss), (EncounterId::GremlinLeader, EnemyId::GremlinLeader)] {
        assert_eq!(ascension::playable_ceiling(owner), 20);
        for (a, reason) in [(17, Some(ascension::ABOVE_CEILING)), (16, None)] {
            let mut entry = setup(encounter, 0, 1, 0).unwrap();
            entry.setup.enemies.retain(|enemy| enemy.enemy_id == owner);
            for enemy in &mut entry.setup.enemies { enemy.ascension = a; }
            let Some(reason) = reason else {
                assert_eq!(ascension::species_admission(owner, a), Ok(()), "{owner:?} A{a}");
                // M29c review m2: the solo owner-only entry is admitted, not only predicted.
                let combat = Combat::enter(entry).unwrap_or_else(|e| panic!("{owner:?} A{a}: {e:?}"));
                assert_eq!(combat.state.ascension, a);
                continue;
            };
            assert_eq!(Combat::enter(entry).err(), Some(RunError::Unsupported(reason)), "{owner:?} A{a}");
            assert_eq!(ascension::species_admission(owner, a), Err(reason), "{owner:?} A{a}");
        }
    }
    // Genuine precondition failures are not the typed interim (M29a1 review m2).
    let mut bad = context(0);
    bad.cards.clear();
    assert_eq!(enemies::setup(EncounterId::Cultist, 1, 1, &mut crate::run::RunRng::new([9; 32], rng::Partition::Test, 0), bad).err(),
        Some(RunError::Unsupported("combat setup preconditions failed")));
}

#[test]
fn m29_live_validator_reads_combat_ascension() {
    let mut combat = Combat::enter(setup(EncounterId::JawWorm, 2, 4, 0).unwrap()).unwrap();
    assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Chomp].base, 12);
    engine::validate_runtime(&combat.state).unwrap();
    combat.state.ascension = 0;
    assert!(engine::validate_runtime(&combat.state).is_err(), "the validator expects the A0 Chomp 11 once the combat is A0");
}

#[test]
fn m29_every_encounter_every_ascension() {
    for (encounter, _, _) in enemies::A0_ENCOUNTER_SETUPS {
        let ceiling = ascension::encounter_ceiling(*encounter);
        for a in 0..=20u8 {
            match ascension::encounter_admission(*encounter, a) {
                Ok(()) => {
                    assert!(a <= ceiling);
                    let mut combat = Combat::enter(setup(*encounter, a, a + 1, u64::from(a)).unwrap())
                        .unwrap_or_else(|e| panic!("{encounter:?} A{a}: {e:?}"));
                    assert_eq!(combat.state.ascension, a);
                    let mut lcg = 0x2545_f491_u64 ^ u64::from(a);
                    for step in 0..60 {
                        if combat.state.outcome.is_some() { break; }
                        let legal = combat.legal_actions_full().unwrap_or_else(|e| panic!("{encounter:?} A{a} step {step}: {e:?}"));
                        if legal.is_empty() { break; }
                        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                        let action = legal[(lcg >> 33) as usize % legal.len()].clone();
                        combat.step_full(&action).unwrap_or_else(|e| panic!("{encounter:?} A{a} step {step} {action:?}: {e:?}"));
                        engine::validate_runtime(&combat.state).unwrap_or_else(|e| panic!("{encounter:?} A{a} step {step}: {e:?}"));
                    }
                },
                Err(reason) => {
                    // M29d (close): every species landed, so nothing is interim at or below the ceiling.
                    assert!(a > ceiling, "{encounter:?} A{a} rejected at or below its ceiling {ceiling}: {reason}");
                    assert_eq!(reason, ascension::ABOVE_CEILING, "{encounter:?} A{a}");
                    assert_eq!(setup(*encounter, a, 1, 0).err(), Some(RunError::Unsupported(reason)), "{encounter:?} A{a}");
                },
            }
        }
    }
}

#[test]
fn m29_split_call_site_reads_combat_ascension() {
    // Review v1 m3: the live split path (`apply_replacement`) builds children at the combat
    // ascension. A Large Slime entered at A0 and then forced to A2 must build A2 children
    // (M29b landed the slime ladders; before M29b the forced split failed with the interim).
    for forced in [None, Some(2)] {
        let mut combat = Combat::enter(setup(EncounterId::LargeSlime, 0, 6, 0).unwrap()).unwrap();
        if let Some(a) = forced { combat.state.ascension = a; }
        let max = combat.state.enemies[0].max_hp;
        combat.state.enemies[0].hp = max / 2 + 1;
        let strike = combat.legal_actions_full().unwrap().into_iter()
            .find(|action| matches!(action, CombatAction::PlayCard { instance, target: Some(_) } if *instance < 905)).unwrap();
        let mut text = format!("{:?}", combat.step_full(&strike));
        if combat.state.fault.is_none() { text += &format!(" {:?}", combat.step_full(&CombatAction::EndTurn)); }
        text += &format!(" {:?}", combat.state.fault);
        assert!(combat.state.fault.is_none(), "{text}");
        let children = combat.state.enemies.iter().filter(|e| e.life == ActorLife::Active).collect::<Vec<_>>();
        assert!(children.len() >= 2, "split produced children: {text}");
        for child in children {
            let (id, base) = child.full_damage_entries.iter().find(|(id, _)| matches!(id.as_str(), "acid_tackle" | "spike_flame_tackle"))
                .map(|(id, entry)| (id.as_str(), entry.base)).unwrap();
            let want = match (id, forced) { ("acid_tackle", None) => 10, ("acid_tackle", Some(_)) => 12, (_, None) => 8, (_, Some(_)) => 10 };
            assert_eq!(base, want, "{id} child at {forced:?}: {text}");
        }
    }
}

#[test]
fn m29_awakened_rebirth_reads_the_owner() {
    // Review v1 m2: max HP, the revival heal and both validator stages read
    // `awakened_rebirth_max_hp(state.ascension)`; any leftover literal faults once e2 changes it.
    // The A0 literal (300) is pinned in `m29_spawned_and_split_children_read_combat_ascension`.
    let mut combat = Combat::enter(super::m8_shapes::entry(EncounterId::AwakenedOne)).unwrap();
    super::m5_elites::hit(&mut combat, 2, 300);
    assert_eq!(combat.state.enemies[2].life, ActorLife::HalfDead);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    let rebirth = ascension::awakened_rebirth_max_hp(combat.state.ascension);
    let awakened = &combat.state.enemies[2];
    assert_eq!((awakened.hp, awakened.max_hp, awakened.life), (rebirth, rebirth, ActorLife::Active));
    assert!(combat.state.fault.is_none());
    engine::validate_runtime(&combat.state).unwrap();
}
