//! M29d: full numeric ascension ladders (A0–A20) for the Act 2 normals and bosses, the Act 3
//! normals, elites and bosses, and Act 4. Literals are the reviewed sheets `m29d/spec/sheet-{act2,
//! act3n,act3b4}.md` (research b/c, source-graded); ladders list `(threshold, value)` and are checked
//! at every level 0..=20. Levels above an encounter's playable ceiling are table-level only (plan §1 I2).
use super::super::*;
use super::super::enemies::{self, ascension};
use super::m29_a2_owner::setup;
use super::m29_c_elites_bosses::{actor, blocks, check, created, power_of, powers};
use super::super::model::State;
use crate::content::{CardId, EncounterId, EnemyId};

/// Turn work of the encounter's own `id` actor with `intent` selected, through `owner`, on a live
/// state of `encounter` entered at A0 and then set to combat ascension `a` (table-level above a ceiling).
fn live_turn(encounter: EncounterId, id: EnemyId, intent: &'static str, a: u8,
    owner: fn(&mut State, &EnemyHandle) -> Vec<Effect>) -> Vec<Effect> {
    let mut state = Combat::enter(setup(encounter, 0, 1, 0).unwrap()).unwrap().state;
    state.ascension = a;
    let slot = state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(id)).expect("species in the formation");
    let selected = ActorIntentId::Full(FullIntentId::from_name(intent));
    state.enemies[slot].intent_id = selected.clone();
    state.enemies[slot].selected_move = selected;
    let handle = state.enemies[slot].instance_id.clone();
    owner(&mut state, &handle)
}

fn act2(state: &mut State, owner: &EnemyHandle) -> Vec<Effect> { enemies::act2::take_turn(state, owner).unwrap() }

fn act3(state: &mut State, owner: &EnemyHandle) -> Vec<Effect> { enemies::act3::take_turn(state, owner).unwrap() }
fn act4(state: &mut State, owner: &EnemyHandle) -> Vec<Effect> { enemies::act4::move_work(state, owner).unwrap() }
fn automaton(state: &mut State, owner: &EnemyHandle) -> Vec<Effect> { enemies::act2::automaton::move_work(state, owner).unwrap() }

fn hits(work: &[Effect]) -> usize {
    work.iter().filter(|effect| matches!(effect.kind, EffectKind::Damage(_))).count()
}

fn setup_powers(encounter: EncounterId, id: EnemyId, a: u8) -> Vec<(&'static str, i32)> {
    ascension::setup_powers(encounter, id, a).into_iter().map(|p| (p.id.0, p.state.amount)).collect()
}

// ---- Act 2 normals and bosses (sheet-act2) ----

#[test]
fn m29_bandit_pointy_ascension_values() {
    check(EnemyId::BanditPointy, &[(0, (30, 30)), (7, (34, 34))], &[("pointy_special", &[(0, 5), (2, 6)])]);
}

#[test]
fn m29_bandit_leader_ascension_values() {
    check(EnemyId::BanditLeader, &[(0, (35, 39)), (7, (37, 41))],
        &[("romeo_agonizing_slash", &[(0, 10), (2, 12)]), ("romeo_cross_slash", &[(0, 15), (2, 17)])]);
    // Agonizing Slash Weak 2 → A17 3 (table-level: Romeo's ceiling is 16).
    for (a, weak) in [(0, 2), (16, 2), (17, 3), (20, 3)] {
        assert_eq!(powers(&live_turn(EncounterId::MaskedBandits, EnemyId::BanditLeader, "romeo_agonizing_slash", a, act2)), vec![("weak", weak)], "A{a}");
    }
}

#[test]
fn m29_bandit_bear_ascension_values() {
    check(EnemyId::BanditBear, &[(0, (38, 42)), (7, (40, 44))], &[("bear_lunge", &[(0, 9), (2, 10)]), ("bear_maul", &[(0, 18), (2, 20)])]);
    for (a, dexterity) in [(0, -2), (16, -2), (17, -4), (20, -4)] {
        assert_eq!(powers(&live_turn(EncounterId::MaskedBandits, EnemyId::BanditBear, "bear_hug", a, act2)), vec![("dexterity", dexterity)], "A{a}");
    }
}

#[test]
fn m29_byrd_ascension_values() {
    check(EnemyId::Byrd, &[(0, (25, 31)), (7, (26, 33))],
        &[("byrd_peck", &[(0, 1)]), ("byrd_swoop", &[(0, 12), (2, 14)]), ("byrd_headbutt", &[(0, 3)])]);
    for (a, flight) in [(0, 3), (16, 3), (17, 4), (20, 4)] {
        assert_eq!(ascension::byrd_flight(a), flight, "A{a}");
        assert!(setup_powers(EncounterId::ThreeByrds, EnemyId::Byrd, a).contains(&("flight", flight)), "A{a} setup");
        assert_eq!(powers(&enemies::move_work_at(&actor(EnemyId::Byrd, "byrd_go_airborne"), a).unwrap()), vec![("flight", flight)], "A{a}");
    }
}

#[test]
fn m29_byrd_peck_hit_count() {
    for (a, count) in [(0, 5), (1, 5), (2, 6), (20, 6)] {
        assert_eq!(hits(&enemies::move_work_at(&actor(EnemyId::Byrd, "byrd_peck"), a).unwrap()), count, "A{a} table");
    }
    // Live: a Byrd opening with Peck at A1/A2 deals 5/6 hits of 1.
    for (a, count) in [(1, 5), (2, 6)] {
        let (mut combat, slot) = (0..64u64).find_map(|visit| {
            let combat = Combat::enter(setup(EncounterId::ThreeByrds, a, 1, visit).unwrap()).unwrap();
            let slot = combat.state.enemies.iter().position(|e| e.intent_id.as_str() == "byrd_peck")?;
            Some((combat, slot))
        }).expect("a Byrd opens with Peck");
        for (i, enemy) in combat.state.enemies.iter_mut().enumerate() {
            if i != slot { enemy.intent_id = ActorIntentId::Full(FullIntentId::from_name("byrd_caw")); enemy.selected_move = enemy.intent_id.clone(); }
        }
        combat.state.player.block = 0;
        let hp = combat.state.player.hp;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(hp - combat.state.player.hp, count, "A{a} Peck hits");
    }
}

#[test]
fn m29_centurion_ascension_values() {
    check(EnemyId::Centurion, &[(0, (76, 80)), (7, (78, 83))], &[("centurion_slash", &[(0, 12), (2, 14)]), ("centurion_fury", &[(0, 6), (2, 7)])]);
    for a in [0, 2, 20] { assert_eq!(hits(&live_turn(EncounterId::CenturionAndHealer, EnemyId::Centurion, "centurion_fury", a, act2)), 3, "A{a}"); }
    // Protect 15 → A17 20 (table-level: the Centurion's formation has the Mystic, ceiling 16).
    for (a, block) in [(0, 15), (16, 15), (17, 20), (20, 20)] {
        assert_eq!(blocks(&live_turn(EncounterId::CenturionAndHealer, EnemyId::Centurion, "centurion_protect", a, act2)), vec![block], "A{a}");
    }
}

#[test]
fn m29_mystic_ascension_values() {
    check(EnemyId::Mystic, &[(0, (48, 56)), (7, (50, 58))], &[("mystic_attack", &[(0, 8), (2, 9)])]);
    let heal = |work: &[Effect]| work.iter().filter_map(|e| match e.kind { EffectKind::Heal(n) => Some(n), _ => None }).collect::<Vec<_>>();
    for (a, strength, amount) in [(0, 2, 16), (1, 2, 16), (2, 3, 16), (16, 3, 16), (17, 4, 20), (20, 4, 20)] {
        let buff = live_turn(EncounterId::CenturionAndHealer, EnemyId::Mystic, "mystic_buff", a, act2);
        assert!(!buff.is_empty() && powers(&buff).iter().all(|p| *p == ("strength", strength)), "A{a} Strength");
        let work = live_turn(EncounterId::CenturionAndHealer, EnemyId::Mystic, "mystic_heal", a, act2);
        assert!(!work.is_empty() && heal(&work).iter().all(|n| *n == amount), "A{a} heal");
    }
    for a in [0, 2, 17] {
        assert!(powers(&live_turn(EncounterId::CenturionAndHealer, EnemyId::Mystic, "mystic_attack", a, act2)).contains(&("frail", 2)), "A{a}");
    }
}

#[test]
fn m29_chosen_ascension_values() {
    check(EnemyId::Chosen, &[(0, (95, 99)), (7, (98, 103))],
        &[("chosen_poke", &[(0, 5), (2, 6)]), ("chosen_debilitate", &[(0, 10), (2, 12)]), ("chosen_zap", &[(0, 18), (2, 21)])]);
}

#[test]
fn m29_mugger_ascension_values() {
    check(EnemyId::Mugger, &[(0, (48, 52)), (7, (50, 54))], &[("mugger_mug", &[(0, 10), (2, 11)]), ("mugger_lunge", &[(0, 16), (2, 18)])]);
    for (a, gold) in [(0, 15), (16, 15), (17, 20), (20, 20)] {
        assert_eq!(ascension::thievery_gold(EnemyId::Mugger, a), gold, "A{a}");
        assert!(setup_powers(EncounterId::TwoThieves, EnemyId::Mugger, a).contains(&("thievery", gold as i32)), "A{a}");
    }
}

#[test]
fn m29_mugger_escape_block_live() {
    // Smoke Bomb 11 → A17 17 (Mugger ceiling 20: live at A16/A17).
    for (a, block) in [(16, 11), (17, 17)] {
        let mut combat = Combat::enter(setup(EncounterId::TwoThieves, a, 1, 0).unwrap()).unwrap();
        let slot = combat.state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::Mugger)).unwrap();
        let owner = combat.state.enemies[slot].instance_id.clone();
        let smoke = ActorIntentId::Full(FullIntentId::from_name("mugger_smoke_bomb"));
        combat.state.enemies[slot].intent_id = smoke.clone();
        combat.state.enemies[slot].selected_move = smoke;
        let work = enemies::act1::thieves::take_turn(&mut combat.state, &owner).unwrap();
        assert_eq!(blocks(&work), vec![block], "A{a}");
    }
}

#[test]
fn m29_shelled_parasite_ascension_values() {
    check(EnemyId::ShelledParasite, &[(0, (68, 72)), (7, (70, 75))], &[("shelled_parasite_double_strike", &[(0, 6), (2, 7)]),
        ("shelled_parasite_fell", &[(0, 18), (2, 21)]), ("shelled_parasite_life_suck", &[(0, 10), (2, 12)])]);
    for a in [0, 7, 20] { assert!(setup_powers(EncounterId::ShelledParasite, EnemyId::ShelledParasite, a).contains(&("plated_armor", 14)), "A{a}"); }
}

#[test]
fn m29_snake_plant_ascension_values() {
    check(EnemyId::SnakePlant, &[(0, (75, 79)), (7, (78, 82))], &[("snake_plant_chomp_chomp", &[(0, 7), (2, 8)])]);
    for a in [0, 2, 20] { assert!(setup_powers(EncounterId::SnakePlant, EnemyId::SnakePlant, a).contains(&("malleable", 3)), "A{a}"); }
}

#[test]
fn m29_snecko_ascension_values() {
    check(EnemyId::Snecko, &[(0, (114, 120)), (7, (120, 125))], &[("snecko_bite", &[(0, 15), (2, 18)]), ("snecko_tail_whip", &[(0, 8), (2, 10)])]);
}

#[test]
fn m29_spheric_guardian_ascension_values() {
    check(EnemyId::SphericGuardian, &[(0, (20, 20))],
        &[("sphere_debuff", &[(0, 10), (2, 11)]), ("sphere_slam", &[(0, 10), (2, 11)]), ("sphere_harden", &[(0, 10), (2, 11)])]);
    for (a, block) in [(0, 25), (16, 25), (17, 35), (20, 35)] {
        assert_eq!(blocks(&enemies::move_work_at(&actor(EnemyId::SphericGuardian, "sphere_activate"), a).unwrap()), vec![block], "A{a}");
    }
    for a in [0, 17] { assert_eq!(blocks(&enemies::move_work_at(&actor(EnemyId::SphericGuardian, "sphere_harden"), a).unwrap()), vec![15], "A{a}"); }
}

#[test]
fn m29_bronze_automaton_ascension_values() {
    check(EnemyId::BronzeAutomaton, &[(0, (300, 300)), (9, (320, 320))],
        &[("automaton_flail", &[(0, 7), (4, 8)]), ("automaton_beam", &[(0, 45), (4, 50)])]);
    for (a, block, strength) in [(0, 9, 3), (3, 9, 3), (4, 9, 4), (8, 9, 4), (9, 12, 4), (20, 12, 4)] {
        let work = live_turn(EncounterId::BronzeAutomaton, EnemyId::BronzeAutomaton, "automaton_boost", a, automaton);
        assert_eq!((blocks(&work), powers(&work)), (vec![block], vec![("strength", strength)]), "A{a}");
    }
}

#[test]
fn m29_bronze_orb_ascension_values() {
    check(EnemyId::BronzeOrb, &[(0, (52, 58)), (9, (54, 60))], &[("orb_beam", &[(0, 8)])]);
}

#[test]
fn m29_automaton_orbs_at_combat_ascension() {
    // The Automaton summons two Orbs on its first turn; they carry the combat ascension's HP band.
    for (a, low, high) in [(8, 52, 58), (9, 54, 60)] {
        let mut combat = Combat::enter(setup(EncounterId::BronzeAutomaton, a, 1, 0).unwrap()).unwrap();
        assert_eq!(combat.state.enemies[0].max_hp, if a >= 9 { 320 } else { 300 });
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        let orbs = combat.state.enemies.iter().filter(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::BronzeOrb)).collect::<Vec<_>>();
        assert_eq!(orbs.len(), 2, "A{a}");
        assert!(orbs.iter().all(|o| (low..=high).contains(&o.max_hp) && o.full_damage_entries[&FullIntentId::from_name("orb_beam")].base == 8), "A{a}");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_champ_ascension_values() {
    check(EnemyId::Champ, &[(0, (420, 420)), (9, (440, 440))],
        &[("champ_slash", &[(0, 16), (4, 18)]), ("champ_execute", &[(0, 10)]), ("champ_slap", &[(0, 12), (4, 14)])]);
    let champ = |intent, a| live_turn(EncounterId::Champ, EnemyId::Champ, intent, a, act2);
    // Gloat 2/3/4, Anger 6/9/12, Stance Block 15/18/20 and Metallicize 5/6/7 (A19 table-level: ceiling 18).
    for (a, gloat, anger, block, metallicize) in [(0, 2, 6, 15, 5), (3, 2, 6, 15, 5), (4, 3, 9, 15, 5), (8, 3, 9, 15, 5),
        (9, 3, 9, 18, 6), (18, 3, 9, 18, 6), (19, 4, 12, 20, 7), (20, 4, 12, 20, 7)] {
        assert_eq!(powers(&champ("champ_gloat", a)), vec![("strength", gloat)], "A{a} Gloat");
        assert_eq!(powers(&champ("champ_anger", a)), vec![("strength", anger)], "A{a} Anger");
        let stance = champ("champ_stance", a);
        assert_eq!((blocks(&stance), powers(&stance)), (vec![block], vec![("metallicize", metallicize)]), "A{a} Stance");
    }
}

#[test]
fn m29_collector_ascension_values() {
    check(EnemyId::Collector, &[(0, (282, 282)), (9, (300, 300))], &[("collector_rake", &[(0, 18), (4, 21)])]);
    let collector = |intent, a| live_turn(EncounterId::Collector, EnemyId::Collector, intent, a, act2);
    for (a, strength, block, debuff) in [(0, 3, 15, 3), (3, 3, 15, 3), (4, 4, 15, 3), (8, 4, 15, 3), (9, 4, 18, 3),
        (18, 4, 18, 3), (19, 5, 23, 5), (20, 5, 23, 5)] {
        let buff = collector("collector_buff", a);
        assert_eq!(blocks(&buff), vec![block], "A{a} Buff Block");
        assert!(!powers(&buff).is_empty() && powers(&buff).iter().all(|p| *p == ("strength", strength)), "A{a} Buff Strength");
        assert_eq!(powers(&collector("collector_ultimate", a)), vec![("weak", debuff), ("vulnerable", debuff), ("frail", debuff)], "A{a}");
    }
}

#[test]
fn m29_torch_head_ascension_values() {
    check(EnemyId::TorchHead, &[(0, (38, 40)), (9, (40, 45))], &[("torch_attack", &[(0, 7)])]);
}

#[test]
fn m29_collector_torch_heads_at_combat_ascension() {
    for (a, low, high) in [(8, 38, 40), (9, 40, 45)] {
        let mut combat = Combat::enter(setup(EncounterId::Collector, a, 1, 0).unwrap()).unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        let torches = combat.state.enemies.iter().filter(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::TorchHead)).collect::<Vec<_>>();
        assert_eq!(torches.len(), 2, "A{a}");
        assert!(torches.iter().all(|t| (low..=high).contains(&t.max_hp)), "A{a}");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

// ---- Act 3 normals and elites (sheet-act3n) ----

#[test]
fn m29_darkling_ascension_values() {
    check(EnemyId::Darkling, &[(0, (48, 56)), (7, (50, 59))], &[("darkling_nip", &[(0, 7), (2, 9)]), ("darkling_chomp", &[(0, 8), (2, 9)])]);
    for (a, range) in [(0, (7, 11)), (1, (7, 11)), (2, (9, 13)), (20, (9, 13))] {
        assert_eq!(ascension::darkling_nip_range(a), range, "A{a}");
    }
}

#[test]
fn m29_darkling_nip_roll_and_validator() {
    // Setup rolls inside the ascension's range; entry and the live validator check it at the combat ascension.
    for (a, low, high) in [(1, 7, 11), (2, 9, 13)] {
        let mut seen = std::collections::BTreeSet::new();
        for visit in 0..48 {
            let combat = Combat::enter(setup(EncounterId::ThreeDarklings, a, 1, visit).unwrap()).unwrap();
            for enemy in &combat.state.enemies {
                let nip = enemy.full_damage_entries[&FullIntentId::from_name("darkling_nip")].base;
                assert!((low..=high).contains(&nip), "A{a} Nip {nip}");
                seen.insert(nip);
            }
            engine::validate_runtime(&combat.state).unwrap();
        }
        assert_eq!(seen.into_iter().collect::<Vec<_>>(), (low..=high).collect::<Vec<_>>(), "A{a}: every value of the range is drawn");
    }
    // A2 Nip 13 is valid at A2 and rejected once the combat is A1 (memory and entry agree).
    let mut combat = (0..64).map(|visit| Combat::enter(setup(EncounterId::ThreeDarklings, 2, 1, visit).unwrap()).unwrap())
        .find(|c| c.state.enemies.iter().any(|e| e.full_damage_entries[&FullIntentId::from_name("darkling_nip")].base == 13)).unwrap();
    engine::validate_runtime(&combat.state).unwrap();
    combat.state.ascension = 1;
    assert!(engine::validate_runtime(&combat.state).is_err(), "Nip 13 is outside the A1 range");
    let mut entry = (0..64).map(|visit| setup(EncounterId::ThreeDarklings, 2, 1, visit).unwrap())
        .find(|e| e.setup.enemies.iter().any(|enemy| matches!(enemy.initial_memory, Some(EnemyMemory::Darkling { nip: 12..=13, .. })))).unwrap();
    for enemy in &mut entry.setup.enemies { enemy.ascension = 1; }
    let reason = Combat::enter(entry).err();
    assert!(reason.is_some(), "an A2 roll entered at A1 is rejected");
    let memory = EnemyMemory::Darkling { center: false, nip: 13, first: true };
    assert!(memory.valid_for(EnemyId::Darkling), "13 is structurally valid (A2+)");
    assert!(!EnemyMemory::Darkling { center: false, nip: 14, first: true }.valid_for(EnemyId::Darkling));
}

#[test]
fn m29_exploder_ascension_values() {
    check(EnemyId::Exploder, &[(0, (30, 30)), (7, (30, 35))], &[("exploder_attack", &[(0, 9), (2, 11)])]);
    for a in [0, 7, 20] { assert!(setup_powers(EncounterId::ThreeShapes, EnemyId::Exploder, a).contains(&("explosive", 3)), "A{a}"); }
}

#[test]
fn m29_maw_ascension_values() {
    check(EnemyId::Maw, &[(0, (300, 300))], &[("maw_slam", &[(0, 25), (2, 30)]), ("maw_nom", &[(0, 5)])]);
    for (a, amount) in [(0, 3), (16, 3), (17, 5), (20, 5)] {
        assert_eq!(powers(&live_turn(EncounterId::Maw, EnemyId::Maw, "maw_drool", a, act3)), vec![("strength", amount)], "A{a} Drool");
        assert_eq!(powers(&live_turn(EncounterId::Maw, EnemyId::Maw, "maw_roar", a, act3)), vec![("weak", amount), ("frail", amount)], "A{a} Roar");
    }
}

#[test]
fn m29_orb_walker_ascension_values() {
    check(EnemyId::OrbWalker, &[(0, (90, 96)), (7, (92, 102))], &[("orb_laser", &[(0, 10), (2, 11)]), ("orb_claw", &[(0, 15), (2, 16)])]);
    for (a, amount) in [(0, 3), (16, 3), (17, 5), (20, 5)] {
        assert!(setup_powers(EncounterId::OrbWalker, EnemyId::OrbWalker, a).contains(&("strength_up", amount)), "A{a}");
    }
}

#[test]
fn m29_repulsor_ascension_values() {
    check(EnemyId::Repulsor, &[(0, (29, 35)), (7, (31, 38))], &[("repulsor_attack", &[(0, 11), (2, 13)])]);
}

#[test]
fn m29_spiker_ascension_values() {
    check(EnemyId::Spiker, &[(0, (42, 56)), (7, (44, 60))], &[("spiker_attack", &[(0, 7), (2, 9)])]);
    for (a, thorns) in [(0, 3), (1, 3), (2, 4), (16, 4), (17, 7), (20, 7)] {
        assert!(setup_powers(EncounterId::ThreeShapes, EnemyId::Spiker, a).contains(&("thorns", thorns)), "A{a}");
    }
}

#[test]
fn m29_spire_growth_ascension_values() {
    check(EnemyId::SpireGrowth, &[(0, (170, 170)), (7, (190, 190))], &[("growth_quick", &[(0, 16), (2, 18)]), ("growth_smash", &[(0, 22), (2, 25)])]);
    // Constricted 10 → A17 12 (table-level: ceiling 16).
    for (a, amount) in [(0, 10), (16, 10), (17, 12), (20, 12)] {
        assert_eq!(powers(&live_turn(EncounterId::SpireGrowth, EnemyId::SpireGrowth, "growth_constrict", a, act3)), vec![("constricted", amount)], "A{a}");
    }
}

#[test]
fn m29_transient_ascension_values() {
    check(EnemyId::Transient, &[(0, (999, 999))], &[("transient_attack", &[(0, 30), (2, 40)])]);
    for (a, fading) in [(0, 5), (16, 5), (17, 6), (20, 6)] {
        assert_eq!(setup_powers(EncounterId::Transient, EnemyId::Transient, a), vec![("fading", fading), ("shifting", 1)], "A{a}");
    }
}

#[test]
fn m29_transient_sequence_and_validator() {
    // Live: attack bases 30, 40, … (A1) and 40, 50, … (A2); it fades after Fading attacks (5; A17 6).
    for (a, start, attacks) in [(1, 30, 5), (2, 40, 5), (16, 40, 5), (17, 40, 6)] {
        let mut combat = Combat::enter(setup(EncounterId::Transient, a, 1, 0).unwrap()).unwrap();
        let mut seen = vec![];
        for _ in 0..12 {
            if combat.state.outcome.is_some() || combat.state.enemies[0].life != ActorLife::Active { break; }
            seen.push(combat.state.enemies[0].full_damage_entries[&FullIntentId::from_name("transient_attack")].base);
            combat.state.player.hp = combat.state.player.max_hp;
            combat.state.player.block = 999;
            combat.step_full(&CombatAction::EndTurn).unwrap();
            assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
            engine::validate_runtime(&combat.state).unwrap();
        }
        assert_eq!(seen, (0..attacks).map(|n| start + 10 * n).collect::<Vec<_>>(), "A{a}");
    }
    // Memory: structurally valid for either start; the validator checks the combat ascension.
    assert!(EnemyMemory::Transient { turns: 0, base: 40 }.valid_for(EnemyId::Transient));
    assert!(!EnemyMemory::Transient { turns: 0, base: 50 }.valid_for(EnemyId::Transient));
    let mut combat = Combat::enter(setup(EncounterId::Transient, 2, 1, 0).unwrap()).unwrap();
    engine::validate_runtime(&combat.state).unwrap();
    combat.state.ascension = 1;
    assert!(engine::validate_runtime(&combat.state).is_err(), "base 40 is not the A1 start");
}

#[test]
fn m29_writhing_mass_ascension_values() {
    check(EnemyId::WrithingMass, &[(0, (160, 160)), (7, (175, 175))], &[("writhing_big", &[(0, 32), (2, 38)]),
        ("writhing_multi", &[(0, 7), (2, 9)]), ("writhing_block", &[(0, 15), (2, 16)]), ("writhing_debuff", &[(0, 10), (2, 12)])]);
}

#[test]
fn m29_writhing_mass_malleable_is_3() {
    // spire-oto.24 (global A0 fix): Writhing Mass's Malleable uses the one-argument constructor (3), at setup
    // and at every reset; the Snake Plant's stays 3.
    for a in [0, 2, 17, 20] {
        assert_eq!(setup_powers(EncounterId::WrithingMass, EnemyId::WrithingMass, a), vec![("reactive", 1), ("malleable", 3)], "A{a}");
    }
    let mut combat = Combat::enter(setup(EncounterId::WrithingMass, 0, 1, 0).unwrap()).unwrap();
    assert_eq!(power_of(&combat.state.enemies[0], "malleable"), Some(3));
    combat.state.player.block = 999;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.state.fault.is_none(), "{:?}", combat.state.fault);
    assert_eq!(power_of(&combat.state.enemies[0], "malleable"), Some(3), "reset at the owner's end of turn");
}

#[test]
fn m29_giant_head_ascension_values() {
    check(EnemyId::GiantHead, &[(0, (500, 500)), (8, (520, 520))], &[("head_count", &[(0, 13)]), ("head_time", &[(0, 30), (3, 40)])]);
    for (a, sequence) in [(2, [30, 35, 40, 45, 50, 55, 60]), (3, [40, 45, 50, 55, 60, 65, 70])] {
        assert_eq!((0..=6i8).map(|n| ascension::giant_head_time_base(-n, a)).collect::<Vec<_>>(), sequence, "A{a}");
    }
    for (a, count) in [(0, 5), (17, 5), (18, 4), (20, 4)] {
        assert_eq!(ascension::giant_head_count(a), count, "A{a}");
        assert_eq!(EnemyMemory::for_profile(EnemyId::GiantHead, a, enemies::EnemyProfile::Standard).unwrap(), EnemyMemory::GiantHead { count }, "A{a}");
    }
}

#[test]
fn m29_giant_head_time_sequence_live() {
    // It Is Time first comes after the countdown (A17: 5th move; A18: 4th move), at the A3+ start 40.
    for (a, first_time, base) in [(2, 5, 30), (3, 5, 40), (17, 5, 40), (18, 4, 40)] {
        let mut combat = Combat::enter(setup(EncounterId::GiantHead, a, 1, 0).unwrap()).unwrap();
        let mut moves = vec![combat.state.enemies[0].intent_id.as_str().to_owned()];
        while moves.len() < first_time {
            combat.state.player.hp = combat.state.player.max_hp;
            combat.state.player.block = 999;
            combat.step_full(&CombatAction::EndTurn).unwrap();
            assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
            engine::validate_runtime(&combat.state).unwrap();
            moves.push(combat.state.enemies[0].intent_id.as_str().to_owned());
        }
        assert!(moves[..first_time - 1].iter().all(|m| m != "head_time"), "A{a}: {moves:?}");
        assert_eq!(moves[first_time - 1], "head_time", "A{a}: {moves:?}");
        assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::from_name("head_time")].base, base, "A{a}");
    }
}

#[test]
fn m29_nemesis_ascension_values() {
    check(EnemyId::Nemesis, &[(0, (185, 185)), (8, (200, 200))], &[("nemesis_tri", &[(0, 6), (3, 7)]), ("nemesis_scythe", &[(0, 45)])]);
    for (a, burns) in [(0, 3), (17, 3), (18, 5), (20, 5)] {
        assert_eq!(created(&live_turn(EncounterId::Nemesis, EnemyId::Nemesis, "nemesis_burn", a, act3), CardId::Burn), burns, "A{a}");
    }
}

#[test]
fn m29_reptomancer_ascension_values() {
    check(EnemyId::Reptomancer, &[(0, (180, 190)), (8, (190, 200))], &[("repto_snake", &[(0, 13), (3, 16)]), ("repto_bite", &[(0, 30), (3, 34)])]);
    for (a, daggers) in [(0, 1), (17, 1), (18, 2), (20, 2)] { assert_eq!(ascension::reptomancer_daggers_per_spawn(a), daggers, "A{a}"); }
}

#[test]
fn m29_dagger_ascension_values() {
    check(EnemyId::Dagger, &[(0, (20, 25))], &[("dagger_wound", &[(0, 9)]), ("dagger_explode", &[(0, 25)])]);
}

#[test]
fn m29_reptomancer_daggers_at_combat_ascension() {
    // Spawn Daggers summons 1 (A17) or 2 (A18) into the first free slots, capped at four live Daggers.
    for (a, per_spawn) in [(17, 1), (18, 2)] {
        let mut combat = Combat::enter(setup(EncounterId::Reptomancer, a, 1, 0).unwrap()).unwrap();
        let repto = combat.state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::Reptomancer)).unwrap();
        let before = combat.state.enemies.iter().filter(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::Dagger) && e.life == ActorLife::Active).count();
        for (i, enemy) in combat.state.enemies.iter_mut().enumerate() {
            let intent = if i == repto { "repto_spawn" } else { "dagger_wound" };
            enemy.intent_id = ActorIntentId::Full(FullIntentId::from_name(intent));
            enemy.selected_move = enemy.intent_id.clone();
        }
        combat.state.player.block = 999;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        let daggers = combat.state.enemies.iter().filter(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::Dagger) && e.life == ActorLife::Active).collect::<Vec<_>>();
        assert_eq!(daggers.len(), (before + per_spawn).min(4), "A{a}");
        assert!(daggers.iter().all(|d| (20..=25).contains(&d.max_hp)), "A{a}");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

// ---- Act 3 bosses and Act 4 (sheet-act3b4) ----

#[test]
fn m29_awakened_one_ascension_values() {
    check(EnemyId::AwakenedOne, &[(0, (300, 300)), (9, (320, 320))], &[("awa_slash", &[(0, 20)]), ("awa_soul", &[(0, 6)]),
        ("awa_echo", &[(0, 40)]), ("awa_sludge", &[(0, 18)]), ("awa_tackle", &[(0, 10)])]);
    // Pre-battle in source queue order (AwakenedOne.usePreBattleAction; all four powers have priority 5).
    for (a, want) in [(0, vec![("regenerate", 10), ("curiosity", 1), ("unawakened", 1)]),
        (3, vec![("regenerate", 10), ("curiosity", 1), ("unawakened", 1)]),
        (4, vec![("regenerate", 10), ("curiosity", 1), ("unawakened", 1), ("strength", 2)]),
        (18, vec![("regenerate", 10), ("curiosity", 1), ("unawakened", 1), ("strength", 2)]),
        (19, vec![("regenerate", 15), ("curiosity", 2), ("unawakened", 1), ("strength", 2)])] {
        assert_eq!(setup_powers(EncounterId::AwakenedOne, EnemyId::AwakenedOne, a), want, "A{a}");
    }
    for (a, hp) in [(0, 300), (8, 300), (9, 320), (20, 320)] { assert_eq!(ascension::awakened_rebirth_max_hp(a), hp, "A{a}"); }
}

#[test]
fn m29_awakened_one_live_a4_strength_and_a9_rebirth() {
    for (a, strength, rebirth) in [(3, None, 300), (4, Some(2), 300), (8, Some(2), 300), (9, Some(2), 320)] {
        let mut combat = Combat::enter(setup(EncounterId::AwakenedOne, a, 1, 0).unwrap()).unwrap();
        let slot = combat.state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::AwakenedOne)).unwrap();
        assert_eq!(power_of(&combat.state.enemies[slot], "strength"), strength, "A{a}");
        let order: Vec<&str> = combat.state.enemies[slot].full_powers.iter().map(|p| p.id.0).collect();
        let want: &[&str] = if a >= 4 { &["regenerate", "curiosity", "unawakened", "strength"] } else { &["regenerate", "curiosity", "unawakened"] };
        assert_eq!(order, want, "A{a} live power order (source queue order)");
        assert_eq!(combat.state.enemies[slot].max_hp, rebirth, "A{a} first phase");
        let hp = combat.state.enemies[slot].hp;
        super::m5_elites::hit(&mut combat, slot, hp);
        assert_eq!(combat.state.enemies[slot].life, ActorLife::HalfDead, "A{a}");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        let awakened = &combat.state.enemies[slot];
        assert_eq!((awakened.hp, awakened.max_hp, awakened.life), (rebirth, rebirth, ActorLife::Active), "A{a} rebirth");
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_awakened_one_cultists_species_not_room() {
    // "Species, not room" (m7): the Cultists beside the Awakened One use the Cultist ladder (A2 Ritual 4).
    for (a, ritual) in [(1, 3), (2, 4)] {
        let work = live_turn(EncounterId::AwakenedOne, EnemyId::Cultist, "incantation", a, act3_or_core);
        assert_eq!(powers(&work), vec![("ritual", ritual)], "A{a}");
    }
}

fn act3_or_core(state: &mut State, owner: &EnemyHandle) -> Vec<Effect> {
    let enemy = state.enemies.iter().find(|e| &e.instance_id == owner).unwrap();
    enemies::move_work_at(enemy, state.ascension).unwrap()
}

#[test]
fn m29_time_eater_ascension_values() {
    check(EnemyId::TimeEater, &[(0, (456, 456)), (9, (480, 480))], &[("time_reverb", &[(0, 7), (4, 8)]), ("time_slam", &[(0, 26), (4, 32)])]);
    for a in [0, 4, 18] { assert_eq!(hits(&live_turn(EncounterId::TimeEater, EnemyId::TimeEater, "time_reverb", a, act3)), 3, "A{a}"); }
}

#[test]
fn m29_deca_ascension_values() {
    check(EnemyId::Deca, &[(0, (250, 250)), (9, (265, 265))], &[("deca_beam", &[(0, 10), (4, 12)])]);
    for (a, artifact) in [(0, 2), (18, 2), (19, 3), (20, 3)] {
        assert_eq!(setup_powers(EncounterId::DonuAndDeca, EnemyId::Deca, a), vec![("artifact", artifact)], "A{a}");
    }
}

#[test]
fn m29_donu_ascension_values() {
    check(EnemyId::Donu, &[(0, (250, 250)), (9, (265, 265))], &[("donu_beam", &[(0, 10), (4, 12)])]);
    for (a, artifact) in [(0, 2), (18, 2), (19, 3), (20, 3)] {
        assert_eq!(setup_powers(EncounterId::DonuAndDeca, EnemyId::Donu, a), vec![("artifact", artifact)], "A{a}");
    }
}

#[test]
fn m29_spire_shield_ascension_values() {
    check(EnemyId::SpireShield, &[(0, (110, 110)), (8, (125, 125))], &[("shield_bash", &[(0, 12), (3, 14)]), ("shield_smash", &[(0, 34), (3, 38)])]);
    for (a, artifact) in [(0, 1), (17, 1), (18, 2), (20, 2)] {
        assert_eq!(setup_powers(EncounterId::ShieldAndSpear, EnemyId::SpireShield, a), vec![("artifact", artifact), ("back_attack", 1)], "A{a}");
    }
}

#[test]
fn m29_spire_spear_ascension_values() {
    check(EnemyId::SpireSpear, &[(0, (160, 160)), (8, (180, 180))], &[("spear_burn_strike", &[(0, 5), (3, 6)]), ("spear_skewer", &[(0, 10)])]);
    for (a, artifact) in [(0, 1), (17, 1), (18, 2), (20, 2)] {
        assert_eq!(setup_powers(EncounterId::ShieldAndSpear, EnemyId::SpireSpear, a), vec![("artifact", artifact), ("back_attack", 1)], "A{a}");
    }
    for (a, count) in [(0, 3), (2, 3), (3, 4), (20, 4)] {
        assert_eq!(hits(&live_turn(EncounterId::ShieldAndSpear, EnemyId::SpireSpear, "spear_skewer", a, act4)), count, "A{a} Skewer");
    }
    for a in [0, 3, 17] { assert_eq!(created(&live_turn(EncounterId::ShieldAndSpear, EnemyId::SpireSpear, "spear_burn_strike", a, act4), CardId::Burn), 2, "A{a}"); }
}

#[test]
fn m29_corrupt_heart_ascension_values() {
    check(EnemyId::CorruptHeart, &[(0, (750, 750)), (9, (800, 800))], &[("heart_blood_shots", &[(0, 2)]), ("heart_echo", &[(0, 40), (4, 45)])]);
    for (a, beat, invincible) in [(0, 1, 300), (18, 1, 300), (19, 2, 200), (20, 2, 200)] {
        assert_eq!(setup_powers(EncounterId::CorruptHeart, EnemyId::CorruptHeart, a), vec![("beat_of_death", beat), ("invincible", invincible)], "A{a}");
        assert_eq!(ascension::heart_invincible(a), invincible, "A{a}");
    }
    for (a, count) in [(0, 12), (3, 12), (4, 15), (20, 15)] {
        assert_eq!(hits(&live_turn(EncounterId::CorruptHeart, EnemyId::CorruptHeart, "heart_blood_shots", a, act3_or_core)), count, "A{a} Blood Shots");
    }
}

#[test]
fn m29_heart_invincible_reset_reads_ascension() {
    // Invincible starts at 300 (A19 200) and every monster turn refills it to the same cap (turns.rs).
    for (a, cap) in [(18, 300), (19, 200)] {
        let mut combat = Combat::enter(setup(EncounterId::CorruptHeart, a, 1, 0).unwrap()).unwrap();
        assert_eq!(power_of(&combat.state.enemies[0], "invincible"), Some(cap), "A{a} setup");
        super::m5_elites::hit(&mut combat, 0, 50);
        assert_eq!(power_of(&combat.state.enemies[0], "invincible"), Some(cap - 50), "A{a} after a hit");
        combat.state.player.hp = combat.state.player.max_hp;
        combat.state.player.block = 999;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(power_of(&combat.state.enemies[0], "invincible"), Some(cap), "A{a} refill");
        engine::validate_runtime(&combat.state).unwrap();
    }
}

#[test]
fn m29_heart_selected_refresh_reads_ascension() {
    // The selection refresh re-reads the damage table at the combat ascension (Echo 40 → A4 45):
    // live, every selected Echo carries the ascension's base.
    for (a, echo) in [(3, 40), (4, 45)] {
        let mut combat = Combat::enter(setup(EncounterId::CorruptHeart, a, 1, 0).unwrap()).unwrap();
        let mut echoes = 0;
        for _ in 0..9 {
            combat.state.player.hp = combat.state.player.max_hp;
            combat.state.player.block = 999;
            combat.step_full(&CombatAction::EndTurn).unwrap();
            assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
            if combat.state.enemies[0].intent_id.as_str() == "heart_echo" {
                echoes += 1;
                assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::from_name("heart_echo")].base, echo, "A{a}");
            }
        }
        assert!(echoes > 0, "A{a}: the Heart selected Echo");
    }
}

// ---- Formations and close ----

#[test]
fn m29_all_species_landed() {
    for (encounter, _, _) in enemies::A0_ENCOUNTER_SETUPS {
        for id in ascension::encounter_species(*encounter) {
            for a in 0..=20 { assert!(ascension::species_supports_ascension(id, a), "{id:?} A{a}"); }
        }
    }
}

/// Formerly `m29_d_formations_ceilings` (M32 plan item 3, design §6.2): the former ceilings (second
/// column, kept as the record) are gone, so every formation is admitted at A0–A20.
#[test]
fn m29_d_formations_admit_through_a20() {
    use EncounterId::*;
    for (encounter, ceiling) in [(MaskedBandits, 16), (ThreeByrds, 20), (CenturionAndHealer, 16), (CultistAndChosen, 16),
        (TwoThieves, 20), (ShelledParasite, 16), (ShelledParasiteAndFungi, 16), (SnakePlant, 16), (Snecko, 16), (SphericGuardian, 20),
        (SentryAndSphere, 20), (BronzeAutomaton, 18), (Champ, 18), (Collector, 20), (ThreeDarklings, 16), (ThreeShapes, 20),
        (FourShapes, 20), (SphereAndTwoShapes, 20), (Maw, 20), (OrbWalker, 20), (SpireGrowth, 16), (Transient, 20),
        (WrithingMass, 20), (GiantHead, 20), (Nemesis, 20), (Reptomancer, 20), (AwakenedOne, 20), (TimeEater, 18),
        (DonuAndDeca, 18), (ShieldAndSpear, 17), (CorruptHeart, 20)] {
        let _former_ceiling: u8 = ceiling;
        for a in 0..=20 { assert_eq!(ascension::encounter_admission(encounter, a), Ok(()), "{encounter:?} A{a}"); }
    }
}

// ---- Routing pins added after the mutation pass (mutations-v1 survivors M24, M25, M27, B35) ----

#[test]
fn m29_d_move_work_at_routes_combat_ascension() {
    // The shared per-intent reader passes the ascension to the Act 3 elite/ordinary and Heart owners.
    let read = |encounter, id, intent, a| {
        let combat = Combat::enter(setup(encounter, 0, 1, 0).unwrap()).unwrap();
        let mut enemy = combat.state.enemies.into_iter().find(|e| e.enemy_id == ActorEnemyId::Full(id)).expect("species in the formation");
        enemy.intent_id = ActorIntentId::Full(FullIntentId::from_name(intent));
        enemies::move_work_at(&enemy, a).unwrap()
    };
    let burns = |a| created(&read(EncounterId::Nemesis, EnemyId::Nemesis, "nemesis_burn", a), CardId::Burn);
    assert_eq!((burns(17), burns(18)), (3, 5), "Nemesis Burns");
    let drool = |a| powers(&read(EncounterId::Maw, EnemyId::Maw, "maw_drool", a));
    assert_eq!((drool(16), drool(17)), (vec![("strength", 3)], vec![("strength", 5)]), "Maw Drool");
    let shots = |a| hits(&read(EncounterId::CorruptHeart, EnemyId::CorruptHeart, "heart_blood_shots", a));
    assert_eq!((shots(3), shots(4)), (12, 15), "Heart Blood Shots");
}

#[test]
fn m29_heart_blood_shots_live_turn() {
    // Live enemy turn (the Heart's act path): Blood Shots 2 × 12 → A4 2 × 15.
    for (a, count) in [(3, 12), (4, 15)] {
        let mut combat = Combat::enter(setup(EncounterId::CorruptHeart, a, 1, 0).unwrap()).unwrap();
        let slot = combat.state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::CorruptHeart)).unwrap();
        let shots = ActorIntentId::Full(FullIntentId::from_name("heart_blood_shots"));
        combat.state.enemies[slot].intent_id = shots.clone();
        combat.state.enemies[slot].selected_move = shots;
        combat.state.player.block = 0;
        let hp = combat.state.player.hp;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        assert_eq!(hp - combat.state.player.hp, 2 * count, "A{a} Blood Shots hits");
    }
}

#[test]
fn m29_byrd_flight_refill_live() {
    // Flight refills at the monster start of turn to the ascension amount: 3 → A17 4.
    for (a, flight) in [(16, 3), (17, 4)] {
        let mut combat = Combat::enter(setup(EncounterId::ThreeByrds, a, 1, 0).unwrap()).unwrap();
        for enemy in combat.state.enemies.iter_mut() {
            enemy.intent_id = ActorIntentId::Full(FullIntentId::from_name("byrd_caw"));
            enemy.selected_move = enemy.intent_id.clone();
            enemy.full_powers.iter_mut().find(|p| p.id.0 == "flight").expect("Byrd starts flying").state.amount = 1;
        }
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{a}: {:?}", combat.state.fault);
        for enemy in &combat.state.enemies { assert_eq!(power_of(enemy, "flight"), Some(flight), "A{a} refill"); }
    }
}
