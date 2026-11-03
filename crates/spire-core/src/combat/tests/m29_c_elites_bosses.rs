//! M29c: full numeric ascension ladders (A0–A20) for the Act 1 elites and bosses, the Slavers
//! and the Act 2 elites. Literals are the reviewed sheet `m29c/spec/sheet.md` (research b,
//! source-graded); ladders list `(threshold, value)` and are checked at every level 0..=20.
//! Levels above an encounter's playable ceiling are table-level only (plan §1 I2).
use super::super::*;
use super::super::enemies::{self, ascension};
use super::m29_a2_owner::{ai_stream, setup};
use crate::content::{CardId, EncounterId, EnemyId};
use std::collections::BTreeMap;

type Ladder<T> = &'static [(u8, T)];

fn at<T: Copy>(a: u8, ladder: Ladder<T>) -> T {
    ladder.iter().rev().find(|(threshold, _)| *threshold <= a).unwrap().1
}

fn check(id: EnemyId, hp: Ladder<(i32, i32)>, damage: &[(&'static str, Ladder<i32>)]) {
    for a in 0..=20u8 {
        assert!(ascension::species_supports_ascension(id, a), "{id:?} A{a} landed");
        assert_eq!(enemies::hp_range(id, a), at(a, hp), "{id:?} A{a} HP");
        let bases = enemies::damage_bases_at_ascension(id, a).unwrap().into_iter()
            .map(|(intent, base)| (intent.as_str(), base)).collect::<BTreeMap<_, _>>();
        let want = damage.iter().map(|(intent, ladder)| (*intent, at(a, ladder))).collect::<BTreeMap<_, _>>();
        assert_eq!(bases, want, "{id:?} A{a} damage bases");
    }
}

/// An A0-built actor of `id` with `intent` selected (table-level move work).
fn actor(id: EnemyId, intent: &'static str) -> EnemyState {
    let handle = EnemyHandle::parse("m29c/actor/0").unwrap();
    let mut enemy = enemies::prepare_split_child(id, handle, 10, ai_stream("m29c"), 0).ok().unwrap();
    enemy.intent_id = ActorIntentId::Full(FullIntentId::from_name(intent));
    enemy
}

fn powers(work: &[Effect]) -> Vec<(&'static str, i32)> {
    work.iter().filter_map(|effect| match &effect.kind {
        EffectKind::ApplyFullPower { id, amount } => Some((id.0, *amount)), _ => None }).collect()
}

fn created(work: &[Effect], card: CardId) -> usize {
    work.iter().filter(|effect| matches!(&effect.kind, EffectKind::CreateCard { id, .. } if *id == card)).count()
}

fn blocks(work: &[Effect]) -> Vec<i32> {
    work.iter().filter_map(|effect| match effect.kind { EffectKind::GainBlock(n) => Some(n), _ => None }).collect()
}

/// Act-2 turn work of `id` with `intent` on a live state at combat ascension `a` (slot 0 replaced).
fn act2_turn(encounter: EncounterId, id: EnemyId, intent: &'static str, a: u8) -> Vec<Effect> {
    let mut state = Combat::enter(setup(encounter, 0, 1, 0).unwrap()).unwrap().state;
    state.ascension = a;
    let owner = state.enemies[0].instance_id.clone();
    state.enemies[0] = actor(id, intent);
    state.enemies[0].instance_id = owner.clone();
    enemies::act2::take_turn(&mut state, &owner).unwrap()
}

fn cards(state: &super::super::model::State, card: CardId) -> usize {
    state.piles.hand.iter().chain(&state.piles.draw).chain(&state.piles.discard).chain(&state.piles.exhaust)
        .filter(|owned| owned.full().is_some_and(|live| live.id == card)).count()
}

fn power_of(enemy: &EnemyState, id: &str) -> Option<i32> {
    enemy.full_powers.iter().find(|p| p.id.0 == id).map(|p| p.state.amount)
}

#[test]
fn m29_gremlin_nob_ascension_values() {
    check(EnemyId::GremlinNob, &[(0, (82, 86)), (8, (85, 90))], &[("skull_bash", &[(0, 6), (3, 8)]), ("bull_rush", &[(0, 14), (3, 16)])]);
    for (a, enrage) in [(0, 2), (17, 2), (18, 3), (20, 3)] {
        assert_eq!(powers(&enemies::move_work_at(&actor(EnemyId::GremlinNob, "bellow"), a).unwrap()), vec![("enrage", enrage)], "A{a}");
    }
    for a in [0, 3, 18] {
        assert!(powers(&enemies::move_work_at(&actor(EnemyId::GremlinNob, "skull_bash"), a).unwrap()).contains(&("vulnerable", 2)), "A{a}");
    }
}

#[test]
fn m29_lagavulin_ascension_values() {
    check(EnemyId::Lagavulin, &[(0, (109, 111)), (8, (112, 115))], &[("attack", &[(0, 18), (3, 20)])]);
    // Siphon Soul: Dexterity then Strength, the same amount (−1; A18 −2).
    for (a, amount) in [(0, -1), (17, -1), (18, -2), (20, -2)] {
        assert_eq!(powers(&enemies::move_work_at(&actor(EnemyId::Lagavulin, "siphon"), a).unwrap()),
            vec![("dexterity", amount), ("strength", amount)], "A{a}");
    }
    // Live, awake (Dead Adventurer profile): A2/A3 attack 18/20 on the second turn.
    for (a, attack) in [(2, 18), (3, 20)] {
        let mut combat = Combat::enter(setup(EncounterId::LagavulinEvent, a, 1, 0).unwrap()).unwrap();
        assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Attack].base, attack, "A{a}");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        let hp = combat.state.player.hp;
        assert_eq!(combat.state.enemies[0].intent_id.as_str(), "attack");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(hp - combat.state.player.hp, attack, "A{a} attack");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_lagavulin_siphon_soul_order_a17_a18() {
    // Live at A17/A18 (ceiling 20): the awake Lagavulin opens with Siphon Soul.
    for (a, amount) in [(17, -1), (18, -2)] {
        let mut combat = Combat::enter(setup(EncounterId::LagavulinEvent, a, 1, 0).unwrap()).unwrap();
        assert!((112..=115).contains(&combat.state.enemies[0].max_hp), "A{a} HP");
        assert_eq!(combat.state.enemies[0].intent_id.as_str(), "siphon");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        let player = combat.state.player_full_powers.iter().map(|p| (p.id.0, p.state.amount)).collect::<Vec<_>>();
        assert_eq!(player, vec![("dexterity", amount), ("strength", amount)], "A{a}: Dexterity, then Strength");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_sentry_ascension_values() {
    check(EnemyId::Sentry, &[(0, (38, 42)), (8, (39, 45))], &[("beam", &[(0, 9), (3, 10)])]);
    for (a, dazed) in [(0, 2), (17, 2), (18, 3), (20, 3)] {
        assert_eq!(created(&enemies::move_work_at(&actor(EnemyId::Sentry, "bolt"), a).unwrap(), CardId::Dazed), dazed, "A{a}");
    }
    // Live: the centre Sentry beams (A2 9, A3 10); the two outer Sentries bolt (Dazed 2 each; A18 3).
    for (a, beam, dazed) in [(2, 9, 2), (3, 10, 2), (17, 10, 2), (18, 10, 3)] {
        let mut combat = Combat::enter(setup(EncounterId::ThreeSentries, a, 1, 0).unwrap()).unwrap();
        let hp = combat.state.player.hp;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(hp - combat.state.player.hp, beam, "A{a} Beam");
        assert_eq!(cards(&combat.state, CardId::Dazed), 2 * dazed, "A{a} Dazed");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_sentry_in_sentry_and_sphere_species_not_room() {
    // "Species, not room" (m7): the Sentry beside the Spheric Guardian uses the Sentry ladder (A2 9,
    // A3 10). The Spheric Guardian lands in M29d, so the formation stays interim; table-level here.
    for (a, beam) in [(2, 9), (3, 10)] {
        assert_eq!(enemies::damage_bases_at_ascension(EnemyId::Sentry, a).unwrap()[&FullIntentId::from_name("beam")], beam);
        assert_eq!(ascension::species_admission(EnemyId::Sentry, a), Ok(()));
        assert_eq!(ascension::encounter_admission(EncounterId::SentryAndSphere, a), Err(ascension::INTERIM));
    }
}

#[test]
fn m29_hexaghost_ascension_values() {
    check(EnemyId::Hexaghost, &[(0, (250, 250)), (9, (264, 264))],
        &[("divider", &[(0, 1)]), ("sear", &[(0, 6)]), ("tackle", &[(0, 5), (4, 6)]), ("inferno", &[(0, 2), (4, 3)])]);
    for (a, strength, burns) in [(0, 2, 1), (4, 2, 1), (18, 2, 1), (19, 3, 2), (20, 3, 2)] {
        let inflame = enemies::move_work_at(&actor(EnemyId::Hexaghost, "inflame"), a).unwrap();
        assert_eq!((powers(&inflame), blocks(&inflame)), (vec![("strength", strength)], vec![12]), "A{a} Inflame");
        assert_eq!(created(&enemies::move_work_at(&actor(EnemyId::Hexaghost, "sear"), a).unwrap(), CardId::Burn), burns, "A{a} Sear");
    }
    // Live entry at t−1/t (A3/A4, A8/A9): damage table and HP.
    for (a, tackle, inferno, hp) in [(3, 5, 2, 250), (4, 6, 3, 250), (8, 6, 3, 250), (9, 6, 3, 264)] {
        let combat = Combat::enter(setup(EncounterId::Hexaghost, a, 1, 0).unwrap()).unwrap();
        let hex = &combat.state.enemies[0];
        assert_eq!((hex.full_damage_entries[&FullIntentId::from_name("tackle")].base,
            hex.full_damage_entries[&FullIntentId::from_name("inferno")].base, hex.max_hp), (tackle, inferno, hp), "A{a}");
    }
}

#[test]
fn m29_hexaghost_live_sear_and_inflame_a18_a19() {
    // Live at A18/A19 (ceiling 20): Activate, Divider, Sear (Burns 1/2), Fire Tackle, Sear, Inflame (Strength 2/3).
    for (a, burns, strength) in [(18, 1, 2), (19, 2, 3)] {
        let mut combat = Combat::enter(setup(EncounterId::Hexaghost, a, 1, 0).unwrap()).unwrap();
        for turn in 1..=6 {
            combat.step_full(&CombatAction::EndTurn).unwrap();
            assert!(combat.state.fault.is_none(), "A{a} turn {turn}: {:?}", combat.state.fault);
            if turn == 3 { assert_eq!(cards(&combat.state, CardId::Burn), burns, "A{a} first Sear"); }
        }
        assert_eq!(cards(&combat.state, CardId::Burn), 2 * burns, "A{a} two Sears");
        assert_eq!(power_of(&combat.state.enemies[0], "strength"), Some(strength), "A{a} Inflame");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_slime_boss_ascension_values() {
    check(EnemyId::SlimeBoss, &[(0, (140, 140)), (9, (150, 150))], &[("slam", &[(0, 35), (4, 38)])]);
    // Goop Spray Slimed 3; A19 5 (table-level: the encounter ceiling is 16 through the children).
    for (a, slimed) in [(0, 3), (16, 3), (18, 3), (19, 5), (20, 5)] {
        assert_eq!(created(&enemies::move_work_at(&actor(EnemyId::SlimeBoss, "sticky"), a).unwrap(), CardId::Slimed), slimed, "A{a}");
    }
    // Live at A16: the opening Goop Spray adds 3 Slimed.
    let mut combat = Combat::enter(setup(EncounterId::SlimeBoss, 16, 1, 0).unwrap()).unwrap();
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.state.fault.is_none(), "{:?}", combat.state.fault);
    assert_eq!(cards(&combat.state, CardId::Slimed), 3);
}

#[test]
fn m29_slime_boss_splits_at_combat_ascension() {
    // A natural split at A4 and A9: the children are built at the combat ascension (A2+ bases).
    for a in [4, 9] {
        let mut combat = Combat::enter(setup(EncounterId::SlimeBoss, a, 1, 0).unwrap()).unwrap();
        assert_eq!(combat.state.enemies[0].max_hp, if a >= 9 { 150 } else { 140 });
        assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::from_name("slam")].base, 38);
        let max = combat.state.enemies[0].max_hp;
        combat.state.enemies[0].hp = max / 2 + 1;
        super::m5_elites::hit(&mut combat, 0, 6);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        let children = combat.state.enemies.iter().filter(|e| e.life == ActorLife::Active).collect::<Vec<_>>();
        assert_eq!(children.len(), 2, "A{a} split");
        for child in children {
            let bases = child.full_damage_entries.iter().map(|(id, e)| (id.as_str(), e.base)).collect::<BTreeMap<_, _>>();
            let want: BTreeMap<&str, i32> = match child.enemy_id {
                ActorEnemyId::Full(EnemyId::AcidSlimeL) => [("acid_spit", 12), ("acid_tackle", 18)].into(),
                ActorEnemyId::Full(EnemyId::SpikeSlimeL) => [("spike_flame_tackle", 18)].into(),
                ref other => panic!("unexpected child {other:?}"),
            };
            assert_eq!(bases, want, "A{a} child");
            assert_eq!(child.max_hp, max / 2 + 1 - 6, "A{a} child HP is the boss's current HP");
        }
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_guardian_ascension_values() {
    check(EnemyId::Guardian, &[(0, (240, 240)), (9, (250, 250))], &[("fierce_bash", &[(0, 32), (4, 36)]),
        ("roll_attack", &[(0, 9), (4, 10)]), ("whirlwind", &[(0, 5)]), ("twin_slam", &[(0, 8)])]);
    for (a, shift) in [(0, 30), (8, 30), (9, 35), (18, 35), (19, 40), (20, 40)] {
        assert_eq!(ascension::guardian_mode_shift(a), shift, "A{a}");
    }
    for (a, hide) in [(0, 3), (18, 3), (19, 4), (20, 4)] {
        assert_eq!(powers(&enemies::move_work_at(&actor(EnemyId::Guardian, "defensive_mode"), a).unwrap()), vec![("sharp_hide", hide)], "A{a}");
    }
    for a in [0, 4, 19] {
        let work = enemies::move_work_at(&actor(EnemyId::Guardian, "charging_up"), a).unwrap();
        assert_eq!(blocks(&work), vec![9], "A{a}");
        let vent = enemies::move_work_at(&actor(EnemyId::Guardian, "vent_steam"), a).unwrap();
        assert_eq!(powers(&vent), vec![("weak", 2), ("vulnerable", 2)], "A{a}");
    }
}

#[test]
fn m29_guardian_mode_shift_live_a8_a9_a18_a19() {
    for (a, shift, hide) in [(8, 30, 3), (9, 35, 3), (18, 35, 3), (19, 40, 4)] {
        let mut combat = Combat::enter(setup(EncounterId::Guardian, a, 1, 0).unwrap()).unwrap();
        let guardian = &combat.state.enemies[0];
        assert_eq!(power_of(guardian, "mode_shift"), Some(shift), "A{a} Mode Shift");
        assert!(matches!(guardian.full_memory, Some(enemies::EnemyMemory::Guardian { quota, .. }) if quota == shift), "A{a} quota");
        // One HP short of the threshold keeps it offensive; the next point shifts it.
        super::m5_elites::hit(&mut combat, 0, shift - 1);
        assert_eq!(power_of(&combat.state.enemies[0], "mode_shift"), Some(1), "A{a}");
        assert_ne!(combat.state.enemies[0].intent_id.as_str(), "defensive_mode", "A{a}");
        super::m5_elites::hit(&mut combat, 0, 1);
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(combat.state.enemies[0].intent_id.as_str(), "defensive_mode", "A{a}");
        assert!(matches!(combat.state.enemies[0].full_memory, Some(enemies::EnemyMemory::Guardian { quota, .. }) if quota == shift + 10), "A{a} next quota");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(power_of(&combat.state.enemies[0], "sharp_hide"), Some(hide), "A{a} Sharp Hide");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_slaver_blue_ascension_values() {
    check(EnemyId::SlaverBlue, &[(0, (46, 50)), (7, (48, 52))], &[("slaver_stab", &[(0, 12), (2, 13)]), ("slaver_rake", &[(0, 7), (2, 8)])]);
    for (a, weak) in [(0, 1), (16, 1), (17, 2), (20, 2)] {
        assert_eq!(powers(&act2_turn(EncounterId::BlueSlaver, EnemyId::SlaverBlue, "slaver_rake", a)), vec![("weak", weak)], "A{a}");
    }
}

#[test]
fn m29_slaver_red_ascension_values() {
    check(EnemyId::SlaverRed, &[(0, (46, 50)), (7, (48, 52))], &[("slaver_stab", &[(0, 13), (2, 14)]), ("slaver_scrape", &[(0, 8), (2, 9)])]);
    // Scrape Vulnerable 1; A17 2 (table-level: the Slaver Red ceiling is 16).
    for (a, vulnerable) in [(0, 1), (16, 1), (17, 2), (20, 2)] {
        assert_eq!(powers(&act2_turn(EncounterId::RedSlaver, EnemyId::SlaverRed, "slaver_scrape", a)), vec![("vulnerable", vulnerable)], "A{a}");
    }
    assert_eq!(powers(&act2_turn(EncounterId::RedSlaver, EnemyId::SlaverRed, "slaver_entangle", 16)), vec![("entangled", 1)]);
    // Live: the opening Stab at A1/A2 (13/14); HP at A6/A7.
    for (a, stab, hp) in [(1, 13, 46..=50), (2, 14, 46..=50), (6, 14, 46..=50), (7, 14, 48..=52)] {
        let mut combat = Combat::enter(setup(EncounterId::RedSlaver, a, 1, 0).unwrap()).unwrap();
        assert!(hp.contains(&combat.state.enemies[0].max_hp), "A{a} HP");
        let before = combat.state.player.hp;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(before - combat.state.player.hp, stab, "A{a} Stab");
    }
}

#[test]
fn m29_book_of_stabbing_ascension_values() {
    check(EnemyId::BookOfStabbing, &[(0, (160, 164)), (8, (168, 172))], &[("book_multi", &[(0, 6), (3, 7)]), ("book_big", &[(0, 21), (3, 24)])]);
    for (a, multi, big) in [(2, 6, 21), (3, 7, 24)] {
        let combat = Combat::enter(setup(EncounterId::BookOfStabbing, a, 1, 0).unwrap()).unwrap();
        let book = &combat.state.enemies[0];
        assert_eq!((book.full_damage_entries[&FullIntentId::from_name("book_multi")].base,
            book.full_damage_entries[&FullIntentId::from_name("book_big")].base), (multi, big), "A{a}");
        assert_eq!(power_of(book, "painful_stabs"), Some(1));
    }
}

#[test]
fn m29_gremlin_leader_ascension_values() {
    check(EnemyId::GremlinLeader, &[(0, (140, 148)), (8, (145, 155))], &[("leader_stab", &[(0, 6)])]);
    // Encourage: Strength to every living monster (Leader included), Block to the others.
    // A17/A18 are table-level (the Leader encounter's ceiling is 16 through Fat and Wizard).
    for (a, strength, block) in [(0, 3, 6), (2, 3, 6), (3, 4, 6), (17, 4, 6), (18, 5, 10), (20, 5, 10)] {
        let mut state = Combat::enter(setup(EncounterId::GremlinLeader, 0, 1, 0).unwrap()).unwrap().state;
        state.ascension = a;
        let leader = state.enemies.iter().find(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::GremlinLeader)).unwrap();
        let owner = leader.instance_id.clone();
        let idx = state.enemies.iter().position(|e| e.instance_id == owner).unwrap();
        state.enemies[idx].intent_id = ActorIntentId::Full(FullIntentId::from_name("leader_encourage"));
        let work = enemies::act2::take_turn(&mut state, &owner).unwrap();
        assert_eq!(powers(&work), vec![("strength", strength); 3], "A{a} Strength");
        assert_eq!(blocks(&work), vec![block; 2], "A{a} Block");
    }
}

#[test]
fn m29_gremlin_leader_live_summons_read_combat_ascension() {
    // Carried from M29b: a live, admitted Leader fight (A2/A7/A16). The Rally summons are built at the
    // combat ascension (HP, bases, Minion; a summoned Mad Gremlin has Angry 1 below A17).
    for a in [2u8, 7, 16] {
        let mut mad = 0;
        for seed in 0..24u8 {
            let mut combat = Combat::enter(setup(EncounterId::GremlinLeader, a, seed, 0).unwrap()).unwrap();
            assert_eq!(combat.state.ascension, a);
            let leader = combat.state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::GremlinLeader)).unwrap();
            let (lo, hi) = enemies::hp_range(EnemyId::GremlinLeader, a);
            assert!((lo..=hi).contains(&combat.state.enemies[leader].max_hp), "A{a} Leader HP");
            for slot in 0..combat.state.enemies.len() { if slot != leader { super::m5_elites::hit(&mut combat, slot, 999); } }
            let owner = Actor::Enemy(combat.state.enemies[leader].instance_id.clone());
            combat.enqueue_native_effects(vec![Effect { source: Source::Actor(owner.clone()), target: Some(owner),
                kind: EffectKind::RallyGremlins { remaining: 2 } }], None).unwrap();
            assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
            let summoned = combat.state.enemies.iter().filter(|e| e.life == ActorLife::Active && e.instance_id.as_str().contains("/spawn/")).collect::<Vec<_>>();
            assert_eq!(summoned.len(), 2, "A{a}: Rally summons two");
            for child in &summoned {
                let ActorEnemyId::Full(id) = child.enemy_id else { unreachable!() };
                let (lo, hi) = enemies::hp_range(id, a);
                assert!((lo..=hi).contains(&child.max_hp), "A{a} {id:?} HP {}", child.max_hp);
                let bases = child.full_damage_entries.iter().map(|(intent, e)| (*intent, e.base)).collect::<BTreeMap<_, _>>();
                assert_eq!(bases, enemies::damage_bases_at_ascension(id, a).unwrap(), "A{a} {id:?} bases");
                assert_eq!(power_of(child, "minion"), Some(1), "A{a} {id:?} minion");
                if id == EnemyId::GremlinMad { mad += 1; assert_eq!(power_of(child, "angry"), Some(1), "A{a} Angry"); }
            }
            engine::validate_runtime(&combat.state).unwrap();
        }
        assert!(mad > 0, "A{a}: some Rally summons a Mad Gremlin");
    }
}

#[test]
fn m29_taskmaster_ascension_values() {
    check(EnemyId::Taskmaster, &[(0, (54, 60)), (8, (57, 64))], &[("taskmaster_whip", &[(0, 7)])]);
    // Whip Wounds 1, A3 2, A18 3 (A18 table-level: Taskmaster's ceiling is 17; its A18 Strength is M32).
    for (a, wounds) in [(0, 1), (2, 1), (3, 2), (17, 2), (18, 3), (20, 3)] {
        let work = act2_turn(EncounterId::Slavers, EnemyId::Taskmaster, "taskmaster_whip", a);
        assert_eq!(created(&work, CardId::Wound), wounds, "A{a}");
        assert!(powers(&work).is_empty(), "A{a}: no Strength below M32");
    }
    // Live in the Slavers formation (ceiling 16) at A2/A3: the opening Whip adds 1/2 Wounds.
    for (a, wounds) in [(2, 1), (3, 2)] {
        let mut combat = Combat::enter(setup(EncounterId::Slavers, a, 1, 0).unwrap()).unwrap();
        assert_eq!(combat.state.enemies[1].intent_id.as_str(), "taskmaster_whip");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(cards(&combat.state, CardId::Wound), wounds, "A{a} Wounds");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_c_formations_leave_interim() {
    use EncounterId::*;
    for (encounter, ceiling) in [(GremlinNob, 20), (Lagavulin, 20), (LagavulinEvent, 20), (ThreeSentries, 20), (SlimeBoss, 16),
        (Guardian, 20), (Hexaghost, 20), (RedSlaver, 16), (ExordiumThugs, 16), (GremlinLeader, 16), (Slavers, 16),
        (BookOfStabbing, 17), (ColosseumSlavers, 16), (ColosseumNobs, 17)] {
        assert_eq!(ascension::encounter_ceiling(encounter), ceiling, "{encounter:?}");
        for a in 0..=ceiling { assert_eq!(ascension::encounter_admission(encounter, a), Ok(()), "{encounter:?} A{a}"); }
        if ceiling < 20 { assert_eq!(ascension::encounter_admission(encounter, ceiling + 1), Err(ascension::ABOVE_CEILING), "{encounter:?}"); }
    }
    // Spheric Guardian and Mugger land in M29d.
    for encounter in [SentryAndSphere, TwoThieves] {
        assert_eq!(ascension::encounter_admission(encounter, 1), Err(ascension::INTERIM), "{encounter:?}");
    }
}
