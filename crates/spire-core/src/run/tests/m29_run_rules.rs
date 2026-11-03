//! M29a1 run rules (plan M29a1, erratum v3): Standard at A0–A5 (A1–A5 diagnostics-only until
//! M29f), the A1 elite quota (binary32), the A5 act-transition heal and the A0-only gate audit.
//! Expected values are literals from `m29a1/spec/sheet.md` (an independent binary32 emulation,
//! SHA256SUMS 9a9b6bcc…), never computed by the code under test.
use super::super::*;
use super::m28_world::{drive, on_map, win};
use crate::combat::rng::Partition;
use crate::content::{Character, EncounterId, EventId, RelicId};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
fn profile(full: bool) -> CreationProfile { CreationProfile { neow_eligible: full, ..CreationProfile::experienced() } }
fn kind(run: &Run) -> RoomViewKind { rooms::public(run).unwrap().kind }

#[cfg(feature = "m29-diagnostics")]
#[test]
fn m29_standard_creation_all_characters_a0_to_a5() {
    // Base max HP (run/engine.rs:13-17); A1–A5 change no creation value (inventory §1).
    for (character, max_hp) in [(Character::Ironclad, 80), (Character::Silent, 70), (Character::Defect, 75), (Character::Watcher, 72)] {
        for ascension in 0..=5 {
            for full in [false, true] {
                let run = Run::new_room_scenario(RunConfig { character, ascension }, profile(full), rng(0x29), RoomScenario::Standard).unwrap();
                let decision = run.decision().unwrap();
                assert_eq!(decision.content_profile, Some(RoomScenario::Standard));
                assert_eq!(run.state.ascension, ascension);
                assert_eq!((run.state.hp, run.state.max_hp, run.state.gold, run.state.potions.len()), (max_hp, max_hp, 99, 3), "{character:?} A{ascension}");
                assert_eq!(kind(&run), RoomViewKind::Neow);
                assert_eq!(rooms::public(&run).unwrap().choices.len(), if full { 4 } else { 2 }, "{character:?} A{ascension} full {full}");
            }
        }
    }
}

#[cfg(not(feature = "m29-diagnostics"))]
#[test]
fn m29_standard_default_build_rejects_a1() {
    for character in CHARACTERS {
        for ascension in 1..=5 {
            for full in [false, true] {
                assert_eq!(Run::new_room_scenario(RunConfig { character, ascension }, profile(full), rng(1), RoomScenario::Standard).err(),
                    Some(RunError::Unsupported("Standard is A0 only until M29f")), "{character:?} A{ascension}");
            }
        }
        assert!(Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile(false), rng(1), RoomScenario::Standard).is_ok());
    }
}

#[test]
fn m29_standard_rejects_a6_and_above() {
    for character in CHARACTERS {
        for ascension in [6, 7, 15, 20] {
            assert_eq!(Run::new_room_scenario(RunConfig { character, ascension }, profile(false), rng(1), RoomScenario::Standard).err(),
                Some(RunError::Unsupported("Standard supports A0–A5")), "{character:?} A{ascension}");
            assert_eq!(rooms::standard::build(RunConfig { character, ascension }, profile(false), rng(1)).err(),
                Some(RunError::Unsupported("Standard supports A0–A5")));
        }
        assert_eq!(Run::new_room_scenario(RunConfig { character, ascension: 21 }, profile(false), rng(1), RoomScenario::Standard).err(),
            Some(RunError::Unsupported("Standard supports A0–A5")));
    }
}

#[test]
fn m29_frozen_profiles_still_a0_only() {
    use RoomScenario::*;
    for (scenario, character) in [(M4Act1Prefix, Character::Ironclad), (M10IroncladHeart, Character::Ironclad), (M12SilentAct1, Character::Silent),
        (M15SilentHeart, Character::Silent), (M17DefectAct1, Character::Defect), (M20DefectHeart, Character::Defect),
        (M22WatcherAct1, Character::Watcher), (M25WatcherHeart, Character::Watcher)] {
        assert!(Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile(false), rng(2), scenario).is_ok(), "{scenario:?}");
        for ascension in [1, 5] {
            assert!(matches!(Run::new_room_scenario(RunConfig { character, ascension }, profile(false), rng(2), scenario),
                Err(RunError::Unsupported(message)) if message.contains("A0 short Neow")), "{scenario:?} A{ascension}");
        }
    }
}

#[test]
fn m29_java_round_f32_edges() {
    use crate::run::numeric::{java_round_f32, libgdx_round};
    // Sheet §(d): Math.round(float) is the exact floor(x + 1/2); a float add of 0.5f gives 1 and 8388610.
    for (bits, expected) in [(0x3effffff_u32, 0), (0x3f000000, 1), (0x3fc00000, 2), (0x40200000, 3), (0x00000000, 0),
        (0x4b000001, 8388609), (0x4a800001, 4194305), (0x3fbfffff, 1)] {
        assert_eq!(java_round_f32(f32::from_bits(bits)), expected, "{bits:#x}");
    }
    assert_eq!(java_round_f32(f32::NAN), 0);
    assert_eq!(java_round_f32(f32::INFINITY), i32::MAX);
    assert_eq!(java_round_f32(-0.5), 0);
    assert_eq!(java_round_f32(-1.5), -1);
    // Sheet §(e): MathUtils.round, moved unchanged.
    assert_eq!([libgdx_round(0.75), libgdx_round(1.5), libgdx_round(2.25), libgdx_round(2.5), libgdx_round(3.5)], [1, 2, 2, 3, 4]);
}

#[test]
fn m29_map_quota_binary32_matches_a0_law() {
    // The pre-M29 integer law `(n * percent + 50) / 100` (shop 5, rest 12, elite 8, event 22); the
    // sheet records no n in 1..=105 where binary32 differs, so A0 maps are unchanged.
    for n in 0..=105 {
        let law = |percent: usize| (n * percent + 50) / 100;
        assert_eq!(map::room_quotas(n, 0), [law(5), law(12), law(8), law(22)], "n {n}");
    }
}

/// Sheet §(b) `elite_a1` for n = 1..=105.
const ELITE_A1: [usize; 105] = [0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
    5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 10, 10, 10, 10,
    10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 13, 13, 13];

#[test]
fn m29_a1_elite_quota_table() {
    for n in 1..=105 {
        let a0 = map::room_quotas(n, 0);
        for ascension in [1, 2, 5, 20] {
            let a1 = map::room_quotas(n, ascension);
            assert_eq!(a1[2], ELITE_A1[n - 1], "n {n} A{ascension}");
            assert_eq!([a1[0], a1[1], a1[3]], [a0[0], a0[1], a0[3]], "n {n}: only the elite target changes");
        }
    }
    // Sheet spot values: n 40/50/60/70 → 5/6/8/9 (A0: 3/4/5/6).
    assert_eq!([40, 50, 60, 70].map(|n| map::room_quotas(n, 1)[2]), [5, 6, 8, 9]);
    assert_eq!([40, 50, 60, 70].map(|n| map::room_quotas(n, 0)[2]), [3, 4, 5, 6]);
}

/// The quota input of a generated map: connected nodes in rows 0–14 except row 13 (`map.rs`).
fn quota_nodes(map: &RunMap) -> usize { map.nodes.iter().filter(|n| n.y <= 14 && n.y != 13).count() }

#[test]
fn m29_a1_elite_placement_on_generated_maps_acts_1_3() {
    let (mut placed, mut unplaced) = ([0usize; 2], 0);
    for act in 1..=3 {
        for seed in 0..40u8 {
            let a0 = map::generate(act, &mut rng(seed), map::MapEligibility::new(true, false)).unwrap();
            let a1 = map::generate(act, &mut rng(seed), map::MapEligibility::new(true, false).with_ascension(1)).unwrap();
            let elites = |m: &RunMap| m.nodes.iter().filter(|n| n.kind == MapNodeKind::Elite).count();
            let requested = ELITE_A1[quota_nodes(&a1) - 1];
            assert!(elites(&a1) >= 1 && elites(&a1) <= requested, "act {act} seed {seed}: placed {} of {requested}", elites(&a1));
            // Leftover requested elites become Monster rooms, never another kind (m9).
            let count = |m: &RunMap, k: MapNodeKind| m.nodes.iter().filter(|n| n.kind == k).count();
            let q = map::room_quotas(quota_nodes(&a1), 1);
            assert!(count(&a1, MapNodeKind::Shop) <= q[0] && count(&a1, MapNodeKind::Rest) <= q[1] + 7 && count(&a1, MapNodeKind::Question) <= q[3]);
            assert!(a1.nodes.iter().all(|n| matches!(n.kind, MapNodeKind::Monster | MapNodeKind::Elite | MapNodeKind::Shop | MapNodeKind::Rest
                | MapNodeKind::Question | MapNodeKind::Treasure | MapNodeKind::Boss)));
            placed[0] += elites(&a0); placed[1] += elites(&a1);
            unplaced += requested - elites(&a1);
        }
    }
    // A1's larger target places more elites overall (the traversal reports unplaced ones).
    assert!(placed[1] > placed[0], "placed A0 {} A1 {} (A1 unplaced {unplaced})", placed[0], placed[1]);
}

#[cfg(feature = "m29-diagnostics")]
#[test]
fn m29_standard_run_maps_use_run_ascension() {
    let has_a1_excess = |map: &crate::run::map::PublicMap| {
        let n = map.nodes.iter().filter(|node| node.y <= 14 && node.y != 13).count();
        let elites = map.nodes.iter().filter(|node| node.kind == MapNodeKind::Elite).count();
        elites > map::room_quotas(n, 0)[2]
    };

    let mut act1_uses_a1 = false;
    for seed in 0..40u8 {
        let mut run = Run::new_room_scenario(RunConfig { character: Character::Ironclad, ascension: 1 }, profile(false), rng(seed), RoomScenario::Standard).unwrap();
        let neow = rooms::public(&run).unwrap().choices[1].id;
        run.step(&GameAction::Choose(neow)).unwrap();
        let map = run.public_map().expect("Standard A1 Act 1 map");
        assert_eq!(map.act, 1, "seed {seed}");
        act1_uses_a1 |= has_a1_excess(&map);
    }
    assert!(act1_uses_a1, "no Standard A1 Act 1 Run map exceeded its A0 elite quota");

    let mut act2_uses_a1 = false;
    for seed in 0..10u8 {
        let mut run = on_map(Character::Ironclad, seed);
        at_boss_chest(&mut run, 1);
        let (hp, max_hp) = (run.state.hp, run.state.max_hp);
        set(&mut run, 1, hp, max_hp);
        drive(&mut run, |run| run.state.act == 2);
        let map = run.public_map().expect("successor Act 2 map");
        assert_eq!(map.act, 2, "seed {seed}");
        act2_uses_a1 |= has_a1_excess(&map);
        if act2_uses_a1 { break; }
    }
    assert!(act2_uses_a1, "no successor A1 Act 2 map exceeded its A0 elite quota");
}

/// A standard A0 Run driven to `act`'s boss chest (A0 combats), then given `ascension`, the HP
/// state and relics; `run.state.act` is still `act` (cfg(test) builder, plan item 10).
fn at_boss_chest(run: &mut Run, act: u8) {
    drive(run, |r| r.state.act == act && r.state.combat.is_none() && matches!(kind(r), RoomViewKind::BossTreasure | RoomViewKind::BossRelicChoice));
    assert_eq!(run.state.act, act);
}
/// The Act 3 boss victory screen of a supplied A0 Run with all keys.
fn at_act3_boss_complete(run: &mut Run) {
    drive(run, |r| r.state.act == 3 && kind(r) == RoomViewKind::Act3BossComplete);
    assert_eq!(kind(run), RoomViewKind::Act3BossComplete);
}
fn set(run: &mut Run, ascension: u8, hp: u32, max_hp: u32) { run.state.ascension = ascension; run.state.hp = hp; run.state.max_hp = max_hp; }

/// Sheet §(c) A5 heals: (max HP, HP before, HP after) for missing 39→29, 10→8, 1→1, 6→5, 399→299.
const A5_ROWS: [(u32, u32, u32); 5] = [(80, 41, 70), (80, 70, 78), (75, 74, 75), (72, 66, 71), (400, 1, 300)];

#[test]
fn m29_a5_heal_transitions_1_2_2_3_3_4_per_character() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = on_map(character, 0x51 + index as u8);
        run.state.max_hp = 999; run.state.hp = 999; run.state.keys = [true; 3];
        for (act, (max_hp, before, after)) in [(1u8, A5_ROWS[index]), (2, A5_ROWS[(index + 1) % 5]), (3, A5_ROWS[4])] {
            if act < 3 { at_boss_chest(&mut run, act) } else { at_act3_boss_complete(&mut run) }
            set(&mut run, 5, before, max_hp);
            drive(&mut run, |r| r.state.act == act + 1);
            assert_eq!((run.state.act, run.state.hp, run.state.max_hp), (act + 1, after, max_hp), "{character:?} {act}→{}", act + 1);
            assert!(run.state.history.events.contains(&CommittedEvent::ActAdvanced { from: act, to: act + 1 }));
            set(&mut run, 0, 999, 999);
        }
    }
}

#[test]
fn m29_a4_heals_full() {
    for character in CHARACTERS {
        let mut run = on_map(character, 0x61);
        run.state.max_hp = 999; run.state.hp = 999; run.state.keys = [true; 3];
        for act in 1..=3u8 {
            if act < 3 { at_boss_chest(&mut run, act) } else { at_act3_boss_complete(&mut run) }
            set(&mut run, 4, 41, 80);
            drive(&mut run, |r| r.state.act == act + 1);
            assert_eq!(run.state.hp, 80, "{character:?} A4 {act}→{}", act + 1);
            set(&mut run, 0, 999, 999);
        }
    }
}

#[test]
fn m29_a5_heal_respects_heal_hooks() {
    // Mark of the Bloom: no heal; Magic Flower acts only in live combat, so the transition heal is
    // the plain 75% (the ordinary heal owner, items::ordinary_heal_amount).
    for (character, relic, after) in [(Character::Ironclad, RelicId::MarkOfBloom, 41), (Character::Silent, RelicId::MarkOfBloom, 41),
        (Character::Defect, RelicId::MagicFlower, 70), (Character::Watcher, RelicId::MagicFlower, 70)] {
        let mut run = on_map(character, 0x62);
        run.state.max_hp = 999; run.state.hp = 999; run.state.keys = [true; 3];
        at_boss_chest(&mut run, 1);
        set(&mut run, 5, 41, 80);
        run.state.relics.push(relic);
        drive(&mut run, |r| r.state.act == 2);
        assert_eq!(run.state.hp, after, "{character:?} {relic:?} 1→2");
        if relic == RelicId::MarkOfBloom {
            set(&mut run, 0, 999, 999);
            at_act3_boss_complete(&mut run);
            set(&mut run, 5, 41, 80);
            drive(&mut run, |r| r.state.act == 4);
            assert_eq!(run.state.hp, 41, "{character:?} Mark of the Bloom 3→4");
        }
    }
}

#[test]
fn m29_no_heal_at_act3_proceed_or_after_heart() {
    for character in CHARACTERS {
        // No keys: the Act 3 boss's Proceed ends the Run with no transition heal.
        let mut run = on_map(character, 0x63);
        run.state.max_hp = 999; run.state.hp = 999;
        at_act3_boss_complete(&mut run);
        set(&mut run, 5, 41, 80);
        drive(&mut run, |_| false);
        assert_eq!((run.state.terminal, run.state.hp), (Some(RunTerminal::Act3NoKeys), 41), "{character:?}");
        // Keys: after the Heart victory nothing heals.
        let mut run = on_map(character, 0x64);
        run.state.max_hp = 999; run.state.hp = 999; run.state.keys = [true; 3];
        drive(&mut run, |r| r.state.act == 4 && kind(r) == RoomViewKind::HeartVictory);
        set(&mut run, 5, 41, 80);
        drive(&mut run, |_| false);
        assert_eq!((run.state.terminal, run.state.hp), (Some(RunTerminal::HeartVictory), 41), "{character:?}");
    }
}

#[test]
fn m29_act4_map_unchanged_by_ascension() {
    let mut views = vec![];
    for ascension in [0, 1, 5] {
        let mut run = on_map(Character::Defect, 0x65);
        run.state.max_hp = 999; run.state.hp = 999; run.state.keys = [true; 3];
        at_act3_boss_complete(&mut run);
        run.state.ascension = ascension;
        drive(&mut run, |r| r.state.act == 4);
        views.push(run.public_map().unwrap());
    }
    assert!(views.iter().all(|view| *view == views[0]));
}

#[test]
fn m29_act3_events_standard_a1() {
    // `rooms/events/act3.rs:11` was A0-only; Standard reads the Run's ascension (no check at A1–A5).
    for character in CHARACTERS {
        for event in [EventId::Falling, EventId::TombOfLordRedMask] {
            let (mut run, node) = rooms::standard::diagnostic_event_run(character, 3, event, rng(0x66)).unwrap();
            run.state.ascension = 1;
            run.step(&GameAction::Move(node)).unwrap_or_else(|error| panic!("{character:?} {event:?}: {error:?}"));
            assert_eq!(kind(&run), RoomViewKind::Event(event), "{character:?}");
        }
    }
}

/// Erratum n8: an A0 Standard Run fights its Act 3 boss; the Run's ascension is set before the
/// settlement, which must reach the ending (`act3_end.rs:22` was A0-only). A20 is M33's interim.
#[test]
fn m29_act3_boss_ending_standard_a1() {
    let at_boss = |character: Character, seed: u8| {
        let mut run = on_map(character, seed);
        run.state.max_hp = 999; run.state.hp = 999;
        drive(&mut run, |r| r.state.act == 3 && r.state.active_combat.as_ref().is_some_and(|a| matches!(a.origin, CombatOrigin::MapRoom { kind: MapNodeKind::Boss, .. })));
        assert!(matches!(run.state.map.as_ref().unwrap().boss_icon, Some(EncounterId::AwakenedOne | EncounterId::TimeEater | EncounterId::DonuAndDeca)));
        run
    };
    for character in CHARACTERS {
        let mut run = at_boss(character, 0x67);
        run.state.ascension = 1;
        win(&mut run);
        assert_eq!(kind(&run), RoomViewKind::Act3BossComplete, "{character:?}");
        drive(&mut run, |_| false);
        assert_eq!(run.state.terminal, Some(RunTerminal::Act3NoKeys), "{character:?}");
    }
    let mut run = at_boss(Character::Ironclad, 0x68);
    run.state.ascension = 20;
    let full = |run: &Run| run.state.combat.as_ref().unwrap().public_full().unwrap();
    for _ in 0..12 {
        let alive: Vec<usize> = full(&run).enemies.iter().enumerate().filter(|(_, e)| e.hp > 0).map(|(i, _)| i).collect();
        for slot in alive { if full(&run).outcome.is_none() { super::m9_boss::hit(&mut run, slot, 2_000) } }
        if full(&run).outcome.is_some() { break }
        run.step(&GameAction::Combat(crate::combat::CombatAction::EndTurn)).unwrap();
    }
    assert_eq!(full(&run).outcome, Some(crate::combat::Outcome::Victory));
    let error = run.settle_combat().err();
    assert!(matches!(error, Some(RunError::Faulted(ref fault)) if fault.kind == RunFaultKind::UnsupportedBoundary("A20 double boss lands in M33")), "{error:?}");
}
