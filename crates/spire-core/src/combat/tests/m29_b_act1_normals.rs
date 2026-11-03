//! M29b: full numeric ascension ladders (A0–A20) for the 17 Act 1 normal species.
//! Literals are the reviewed sheet `m29b/spec/sheet.md` (research b, source-graded);
//! ladders list `(threshold, value)` and are checked at every level 0..=20. Levels
//! above a species' playable ceiling (16) are table-level only (plan §1 I2).
use super::super::*;
use super::super::enemies::{self, ascension};
use super::m29_a2_owner::{ai_stream, setup};
use crate::content::{EncounterId, EnemyId};
use std::collections::{BTreeMap, BTreeSet};

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
    let handle = EnemyHandle::parse("m29b/actor/0").unwrap();
    let mut enemy = enemies::prepare_split_child(id, handle, 10, ai_stream("m29b"), 0).ok().unwrap();
    enemy.intent_id = ActorIntentId::Full(FullIntentId::from_name(intent));
    enemy
}

fn power_amount(work: &[Effect], power: &str) -> i32 {
    work.iter().find_map(|effect| match &effect.kind {
        EffectKind::ApplyFullPower { id, amount } if id.0 == power => Some(*amount), _ => None }).unwrap()
}

fn setup_power(entry: &CombatEntry, slot: usize, power: &str) -> Option<i32> {
    entry.setup.enemies[slot].powers.iter().find(|p| p.id.0 == power).map(|p| p.state.amount)
}

#[test]
fn m29_acid_slime_s_ascension_values() {
    check(EnemyId::AcidSlimeS, &[(0, (8, 12)), (7, (9, 13))], &[("acid_tackle", &[(0, 3), (2, 4)])]);
    for a in [0, 2, 17, 20] { assert_eq!(power_amount(&enemies::move_work_at(&actor(EnemyId::AcidSlimeS, "acid_lick"), a).unwrap(), "weak"), 1); }
}

#[test]
fn m29_acid_slime_m_ascension_values() {
    check(EnemyId::AcidSlimeM, &[(0, (28, 32)), (7, (29, 34))],
        &[("acid_spit", &[(0, 7), (2, 8)]), ("acid_tackle", &[(0, 10), (2, 12)])]);
    for a in [0, 2, 17, 20] { assert_eq!(power_amount(&enemies::move_work_at(&actor(EnemyId::AcidSlimeM, "acid_lick"), a).unwrap(), "weak"), 1); }
}

#[test]
fn m29_acid_slime_l_ascension_values() {
    check(EnemyId::AcidSlimeL, &[(0, (65, 69)), (7, (68, 72))],
        &[("acid_spit", &[(0, 11), (2, 12)]), ("acid_tackle", &[(0, 16), (2, 18)])]);
    for a in [0, 2, 17, 20] { assert_eq!(power_amount(&enemies::move_work_at(&actor(EnemyId::AcidSlimeL, "acid_lick"), a).unwrap(), "weak"), 2); }
}

#[test]
fn m29_spike_slime_s_ascension_values() {
    check(EnemyId::SpikeSlimeS, &[(0, (10, 14)), (7, (11, 15))], &[("spike_tackle", &[(0, 5), (2, 6)])]);
}

#[test]
fn m29_spike_slime_m_ascension_values() {
    check(EnemyId::SpikeSlimeM, &[(0, (28, 32)), (7, (29, 34))], &[("spike_flame_tackle", &[(0, 8), (2, 10)])]);
    for a in [0, 2, 16, 17, 20] { assert_eq!(power_amount(&enemies::move_work_at(&actor(EnemyId::SpikeSlimeM, "spike_lick"), a).unwrap(), "frail"), 1); }
}

#[test]
fn m29_spike_slime_l_ascension_values() {
    check(EnemyId::SpikeSlimeL, &[(0, (64, 70)), (7, (67, 73))], &[("spike_flame_tackle", &[(0, 16), (2, 18)])]);
    // Frail 2 → 3 at A17: above the ceiling (16), so table-level.
    for (a, frail) in [(0, 2), (2, 2), (16, 2), (17, 3), (20, 3)] {
        assert_eq!(power_amount(&enemies::move_work_at(&actor(EnemyId::SpikeSlimeL, "spike_lick"), a).unwrap(), "frail"), frail, "A{a}");
    }
}

#[test]
fn m29_louse_red_ascension_values() {
    check(EnemyId::LouseRed, &[(0, (10, 15)), (7, (11, 16))], &[("bite", &[(0, 5), (2, 6)])]);
    // Grow Strength 3 → 4 at A17 (table-level above the ceiling).
    for (a, grow) in [(0, 3), (2, 3), (16, 3), (17, 4), (20, 4)] {
        assert_eq!(power_amount(&enemies::move_work_at(&actor(EnemyId::LouseRed, "grow"), a).unwrap(), "strength"), grow, "A{a}");
    }
}

#[test]
fn m29_louse_green_ascension_values() {
    check(EnemyId::LouseGreen, &[(0, (11, 17)), (7, (12, 18))], &[("bite", &[(0, 5), (2, 6)])]);
    for a in [0, 2, 17, 20] { assert_eq!(power_amount(&enemies::move_work_at(&actor(EnemyId::LouseGreen, "spit_web"), a).unwrap(), "weak"), 2); }
}

#[test]
fn m29_louse_bite_and_curl_up_ranges() {
    for (a, bite, curl) in [(0, (5, 7), (3, 7)), (1, (5, 7), (3, 7)), (2, (6, 8), (3, 7)), (6, (6, 8), (3, 7)),
        (7, (6, 8), (4, 8)), (16, (6, 8), (4, 8)), (17, (6, 8), (9, 12)), (20, (6, 8), (9, 12))] {
        assert_eq!((ascension::louse_bite_range(a), ascension::louse_curl_up_range(a)), (bite, curl), "A{a}");
    }
    // Live setup draws cover exactly the range at t−1/t, and entry admits each.
    for (a, bite, curl) in [(1, 5..=7, 3..=7), (2, 6..=8, 3..=7), (6, 6..=8, 3..=7), (7, 6..=8, 4..=8)] {
        let (mut bites, mut curls) = (BTreeSet::new(), BTreeSet::new());
        for seed in 0..30u8 {
            for visit in 0..4u64 {
                let entry = setup(EncounterId::TwoLouse, a, seed, visit).unwrap();
                for slot in 0..2 {
                    bites.insert(entry.setup.enemy_damage_bases[&(slot as u32, FullIntentId::Bite)]);
                    curls.insert(setup_power(&entry, slot, "curl_up").unwrap());
                }
                let combat = Combat::enter(entry).unwrap();
                assert_eq!(combat.state.ascension, a);
                engine::validate_runtime(&combat.state).unwrap();
            }
        }
        assert_eq!(bites, bite.collect::<BTreeSet<_>>(), "A{a} bite draws");
        assert_eq!(curls, curl.collect::<BTreeSet<_>>(), "A{a} Curl Up draws");
    }
}

#[test]
fn m29_cultist_ascension_values() {
    check(EnemyId::Cultist, &[(0, (48, 54)), (7, (50, 56))], &[("attack", &[(0, 6)])]);
    for (a, ritual) in [(0, 3), (1, 3), (2, 4), (16, 4), (17, 5), (20, 5)] {
        assert_eq!(power_amount(&enemies::move_work_at(&actor(EnemyId::Cultist, "incantation"), a).unwrap(), "ritual"), ritual, "A{a}");
    }
}

#[test]
fn m29_jaw_worm_ascension_values() {
    check(EnemyId::JawWorm, &[(0, (40, 44)), (7, (42, 46))], &[("chomp", &[(0, 11), (2, 12)]), ("thrash", &[(0, 7)])]);
    let block = |work: &[Effect]| work.iter().find_map(|e| match e.kind { EffectKind::GainBlock(n) => Some(n), _ => None }).unwrap();
    for (a, strength, bellow_block) in [(0, 3, 6), (1, 3, 6), (2, 4, 6), (16, 4, 6), (17, 5, 9), (20, 5, 9)] {
        let work = enemies::move_work_at(&actor(EnemyId::JawWorm, "bellow"), a).unwrap();
        assert_eq!((power_amount(&work, "strength"), block(&work)), (strength, bellow_block), "A{a}");
        assert_eq!(block(&enemies::move_work_at(&actor(EnemyId::JawWorm, "thrash"), a).unwrap()), 5, "A{a}");
    }
}

#[test]
fn m29_fungi_beast_ascension_values() {
    check(EnemyId::FungiBeast, &[(0, (22, 28)), (7, (24, 28))], &[("bite", &[(0, 6)])]);
    for (a, grow) in [(0, 3), (1, 3), (2, 4), (16, 4), (17, 5), (20, 5)] {
        assert_eq!(power_amount(&enemies::move_work_at(&actor(EnemyId::FungiBeast, "grow"), a).unwrap(), "strength"), grow, "A{a}");
    }
    for a in [0, 2, 7, 17, 20] {
        assert_eq!(setup_power(&setup(EncounterId::TwoFungiBeasts, a, 2, 0).unwrap(), 0, "spore_cloud"), Some(2), "A{a}");
    }
}

#[test]
fn m29_looter_ascension_values() {
    check(EnemyId::Looter, &[(0, (44, 48)), (7, (46, 50))], &[("looter_mug", &[(0, 10), (2, 11)]), ("looter_lunge", &[(0, 12), (2, 14)])]);
    for (a, gold) in [(0, 15), (16, 15), (17, 20), (20, 20)] {
        assert_eq!(ascension::thievery_gold(EnemyId::Looter, a), gold as u32, "A{a}");
        let entry = setup(EncounterId::Looter, a, 3, 0).unwrap();
        assert_eq!(setup_power(&entry, 0, "thievery"), Some(gold), "A{a} Thievery shows the gold amount");
        // Live: the first Mug steals the A-specific amount and deals the A-specific base.
        let mut combat = Combat::enter(entry).unwrap();
        combat.set_thief_gold_for_entry(99).unwrap();
        let hp = combat.state.player.hp;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(combat.current_combat_gold(), Some(99 - gold as u32), "A{a} gold stolen");
        assert_eq!(hp - combat.state.player.hp, if a >= 2 { 11 } else { 10 }, "A{a} Mug damage");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_gremlin_mad_ascension_values() {
    check(EnemyId::GremlinMad, &[(0, (20, 24)), (7, (21, 25))], &[("gremlin_scratch", &[(0, 4), (2, 5)])]);
    for (a, angry) in [(0, 1), (16, 1), (17, 2), (20, 2)] {
        assert_eq!(ascension::gremlin_mad_angry(a), angry, "A{a}");
        let powers = ascension::setup_powers(EncounterId::GremlinGang, EnemyId::GremlinMad, a);
        assert_eq!(powers.iter().find(|p| p.id.0 == "angry").map(|p| p.state.amount), Some(angry), "A{a}");
    }
    // Live at the Gremlin Gang ceiling (16; Fat and Wizard): Angry 1.
    let entry = (0..40u8).map(|seed| setup(EncounterId::GremlinGang, 16, seed, 0).unwrap())
        .find(|entry| entry.setup.enemies.iter().any(|e| e.enemy_id == EnemyId::GremlinMad)).unwrap();
    let slot = entry.setup.enemies.iter().position(|e| e.enemy_id == EnemyId::GremlinMad).unwrap();
    assert_eq!(setup_power(&entry, slot, "angry"), Some(1));
}

#[test]
fn m29_gremlin_sneaky_ascension_values() {
    check(EnemyId::GremlinSneaky, &[(0, (10, 14)), (7, (11, 15))], &[("gremlin_puncture", &[(0, 9), (2, 10)])]);
}

#[test]
fn m29_gremlin_fat_ascension_values() {
    check(EnemyId::GremlinFat, &[(0, (13, 17)), (7, (14, 18))], &[("gremlin_smash", &[(0, 4), (2, 5)])]);
    // Smash applies Weak 1 at every playable level (A0–A16, the combat ascension set on the state);
    // the A17 Frail is behaviour (M32), above the ceiling.
    for a in [0, 1, 2, 6, 7, 16] {
        let mut state = Combat::enter(setup(EncounterId::GremlinGang, 0, 1, 0).unwrap()).unwrap().state;
        state.ascension = a;
        let owner = state.enemies[0].instance_id.clone();
        state.enemies[0] = actor(EnemyId::GremlinFat, "gremlin_smash");
        state.enemies[0].instance_id = owner.clone();
        let work = enemies::act2::take_turn(&mut state, &owner).unwrap();
        assert_eq!(power_amount(&work, "weak"), 1, "A{a}");
        assert!(!work.iter().any(|e| matches!(&e.kind, EffectKind::ApplyFullPower { id, .. } if id.0 == "frail")), "A{a}");
    }
}

#[test]
fn m29_gremlin_shield_ascension_values() {
    check(EnemyId::GremlinShield, &[(0, (12, 15)), (7, (13, 17))], &[("gremlin_bash", &[(0, 6), (2, 8)])]);
    for (a, block) in [(0, 7), (6, 7), (7, 8), (16, 8), (17, 11), (20, 11)] {
        assert_eq!(ascension::gremlin_shield_protect(a), block, "A{a}");
    }
    // Live at t−1/t (A6/A7): the opening Protect gives exactly one ally the A-specific Block.
    for (a, block) in [(6, 7), (7, 8), (16, 8)] {
        let entry = (0..60u8).map(|seed| setup(EncounterId::GremlinGang, a, seed, 0).unwrap())
            .find(|entry| entry.setup.enemies.iter().any(|e| e.enemy_id == EnemyId::GremlinShield)).unwrap();
        let mut combat = Combat::enter(entry).unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(combat.state.enemies.iter().map(|e| e.block).sum::<i32>(), block, "A{a} Protect block");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_gremlin_wizard_ascension_values() {
    check(EnemyId::GremlinWizard, &[(0, (21, 25)), (7, (22, 26))], &[("gremlin_blast", &[(0, 25), (2, 30)])]);
}

#[test]
fn m29_slime_splits_keep_combat_ascension() {
    // A natural A2 Large Slime split: children are built at the combat ascension (A2 bases).
    for seed in 0..4u8 {
        let mut combat = Combat::enter(setup(EncounterId::LargeSlime, 2, seed, 0).unwrap()).unwrap();
        let parent = match combat.state.enemies[0].enemy_id { ActorEnemyId::Full(id) => id, _ => unreachable!() };
        let max = combat.state.enemies[0].max_hp;
        combat.state.enemies[0].hp = max / 2 + 1;
        let strike = combat.legal_actions_full().unwrap().into_iter()
            .find(|action| matches!(action, CombatAction::PlayCard { instance, target: Some(_) } if *instance < 905)).unwrap();
        combat.step_full(&strike).unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "{parent:?}: {:?}", combat.state.fault);
        let children = combat.state.enemies.iter().filter(|e| e.life == ActorLife::Active).collect::<Vec<_>>();
        assert_eq!(children.len(), 2, "{parent:?} split");
        for child in children {
            let bases = child.full_damage_entries.iter().map(|(id, e)| (id.as_str(), e.base)).collect::<BTreeMap<_, _>>();
            let want: BTreeMap<&str, i32> = match parent {
                EnemyId::AcidSlimeL => [("acid_spit", 8), ("acid_tackle", 12)].into(),
                EnemyId::SpikeSlimeL => [("spike_flame_tackle", 10)].into(),
                other => panic!("unexpected Large Slime {other:?}"),
            };
            assert_eq!(bases, want, "{parent:?} child at A2");
        }
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_jaw_worm_horde_act3_a2() {
    // "Species, not room" (m7): Act 3's Jaw Worm Horde uses the Act 1 species' A2 values.
    let combat = Combat::enter(setup(EncounterId::JawWormHorde, 2, 1, 0).unwrap()).unwrap();
    assert_eq!(combat.state.ascension, 2);
    for jaw in &combat.state.enemies {
        assert_eq!(jaw.full_damage_entries[&FullIntentId::Chomp].base, 12);
        assert_eq!(jaw.full_powers.iter().find(|p| p.id.0 == "strength").map(|p| p.state.amount), Some(4), "pre-battle Bellow at A2");
        assert_eq!(jaw.block, 6);
    }
    engine::validate_runtime(&combat.state).unwrap();
}

#[test]
fn m29_act1_normal_formations_leave_interim() {
    for encounter in [EncounterId::Cultist, EncounterId::JawWorm, EncounterId::TwoLouse, EncounterId::ThreeLouse,
        EncounterId::SmallSlimes, EncounterId::LargeSlime, EncounterId::LotsOfSlimes, EncounterId::Looter,
        EncounterId::GremlinGang, EncounterId::ExordiumWildlife, EncounterId::TwoFungiBeasts, EncounterId::ThreeFungiBeasts,
        EncounterId::BlueSlaver, EncounterId::JawWormHorde] {
        let ceiling = ascension::encounter_ceiling(encounter);
        for a in 0..=ceiling { assert_eq!(ascension::encounter_admission(encounter, a), Ok(()), "{encounter:?} A{a}"); }
        if ceiling < 20 { assert_eq!(ascension::encounter_admission(encounter, ceiling + 1), Err(ascension::ABOVE_CEILING)); }
    }
    // Slaver Red lands in M29c; Mugger in M29d.
    for encounter in [EncounterId::ExordiumThugs, EncounterId::RedSlaver, EncounterId::TwoThieves] {
        assert_eq!(ascension::encounter_admission(encounter, 1), Err(ascension::INTERIM), "{encounter:?}");
    }
}

#[test]
fn m29_looter_mid_theft_state_reads_combat_ascension() {
    // Review v1 m1: between the ledger and the debit, the state check expects the queued debit to
    // carry the A-specific cap (A16 15, A17 20).
    for (a, cap) in [(16, 15u32), (17, 20)] {
        let mut combat = Combat::enter(setup(EncounterId::Looter, a, 3, 0).unwrap()).unwrap();
        combat.set_thief_gold_for_entry(99).unwrap();
        let owner = combat.state.enemies[0].instance_id.clone();
        let work = enemies::act1::thieves::take_turn(&mut combat.state, &owner).unwrap();
        for effect in work.into_iter().rev() { combat.state.pending.push_front(effect); }
        engine::dispatch_one(&mut combat.state);
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert!(matches!(combat.state.enemies[0].full_memory, Some(enemies::EnemyMemory::Thief(ref m)) if m.pending_theft.is_some()),
            "A{a}: the ledger committed and the debit is outstanding");
        assert!(combat.state.pending.iter().any(|e| matches!(&e.kind,
            EffectKind::Thief(enemies::act1::thieves::ThiefEffect::TheftDamage { cap: c, .. }) if *c == cap)), "A{a}");
        enemies::act1::thieves::validate_state(&combat.state).unwrap_or_else(|e| panic!("A{a}: {e:?}"));
    }
}

#[test]
fn m29_summoned_mad_gremlin_angry_reads_combat_ascension() {
    // Review v1 m2: a Mad Gremlin summoned by the Gremlin Leader gets Angry at the combat ascension.
    // The Leader encounter is interim until M29c, so the A0 fight is entered and the combat ascension
    // forced (the M29a2 spawn pattern); the Mad Gremlin itself has landed.
    for (forced, angry, hp) in [(0, 1, 20..=24), (16, 1, 21..=25), (17, 2, 21..=25)] {
        let mut c = Combat::enter(super::m6_ordinary::entry(EncounterId::GremlinLeader)).unwrap();
        super::m5_elites::hit(&mut c, 0, 30);
        c.state.ascension = forced;
        let summoner = c.state.enemies[2].instance_id.clone();
        c.enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Enemy(summoner.clone())), target: None,
            kind: EffectKind::SpawnEnemy(SpawnRequest { summoner, slot: 0, species: EnemyId::GremlinMad }) }], None).unwrap();
        assert!(c.state.fault.is_none(), "A{forced}: {:?}", c.state.fault);
        let child = c.state.enemies.iter().find(|e| e.instance_id.as_str().contains("/spawn/")).unwrap();
        assert_eq!(child.enemy_id, ActorEnemyId::Full(EnemyId::GremlinMad));
        assert_eq!(child.full_powers.iter().find(|p| p.id.0 == "angry").map(|p| p.state.amount), Some(angry), "A{forced}");
        assert!(hp.contains(&child.max_hp), "A{forced} HP {}", child.max_hp);
        assert_eq!(child.full_damage_entries[&FullIntentId::from_name("gremlin_scratch")].base, if forced >= 2 { 5 } else { 4 }, "A{forced}");
    }
}
