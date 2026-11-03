//! M29d (rest of M29 and the close): Run combats enter at the Run's ascension (M29c review m1), the
//! remaining event fights (Masked Bandits, Mysterious Sphere) at A1–A5, Act 2 bosses at A4 for each
//! character, and Act 4 (Shield and Spear at A3, the Heart at A4) on public A1–A5 Standard Runs.
//! Combats are won with the constructed lethal frontier (supplied, not a strategy claim).
use super::super::*;
use super::m28_event_core::at_event;
use super::m28_world::{drive, win};
use crate::combat::enemies;
use crate::combat::rng::Partition;
use crate::content::{Character, EncounterId, EnemyId, EventId};
use rooms::standard::event_exec::Outcome;
use rooms::standard::event_frame::{self, DONE};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

/// A public standard Run at `ascension`, past the short Neow, on the Act 1 map.
fn on_map_at(character: Character, ascension: u8, seed: u8) -> Run {
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    let mut run = Run::new_room_scenario(RunConfig { character, ascension }, profile, RunRng::new([seed; 32], Partition::Test, 0), RoomScenario::Standard)
        .unwrap_or_else(|e| panic!("{character:?} A{ascension}: {e:?}"));
    let id = rooms::public(&run).unwrap().choices[1].id;
    run.step(&GameAction::Choose(id)).unwrap();
    run
}

/// The active combat entered at `ascension`, with every enemy's bases equal to its table at
/// `ascension`; returns whether some enemy's bases differ from A0 (the routing witness).
fn assert_combat_at(run: &Run, ascension: u8, context: &str) -> bool {
    let combat = run.state.combat.as_ref().unwrap_or_else(|| panic!("{context}: no combat"));
    assert_eq!(combat.ascension(), ascension, "{context}: the combat enters at the Run's ascension");
    let mut differs = false;
    for (id, bases) in combat.damage_bases_for_test() {
        let table = enemies::damage_bases_at_ascension(id, ascension).unwrap();
        // Rolled bases (Louse Bite, Darkling Nip) are checked by their own range tests.
        if matches!(id, EnemyId::LouseRed | EnemyId::LouseGreen | EnemyId::Darkling) { continue }
        assert_eq!(bases, table, "{context}: {id:?} bases at A{ascension}");
        differs |= table != enemies::damage_bases_at_ascension(id, 0).unwrap();
    }
    differs
}

fn origin_kind(run: &Run) -> Option<MapNodeKind> {
    match run.state.active_combat.as_ref()?.origin { CombatOrigin::MapRoom { kind, .. } => Some(kind), _ => None }
}

#[test]
fn m29_run_combats_use_run_ascension() {
    // Map room: the first elite on an A3 Run (Act 1 elites all change at A3).
    let mut run = on_map_at(Character::Ironclad, 3, 0x51);
    drive(&mut run, |r| r.state.combat.is_some() && origin_kind(r) == Some(MapNodeKind::Elite));
    assert!(assert_combat_at(&run, 3, "map-room elite"), "an A3 elite differs from A0");
    // Event fight: Three Sentries (Beam 9 → A3 10) through the event executor.
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenShrine, 0x52);
    run.state.ascension = 3;
    event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::ThreeSentries, rewards: None, next_stage: DONE, elite: true }]).unwrap();
    assert!(assert_combat_at(&run, 3, "event fight"));
}

#[test]
fn m29_d_event_fights_accepted_a1_to_a5() {
    for (encounter, act) in [(EncounterId::MaskedBandits, 2), (EncounterId::TwoOrbWalkers, 3)] {
        for ascension in 1..=5u8 {
            let mut run = at_event(Character::Silent, act, EventId::GoldenShrine, 0x60 + ascension);
            run.state.ascension = ascension;
            event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter, rewards: None, next_stage: DONE, elite: false }])
                .unwrap_or_else(|e| panic!("{encounter:?} A{ascension}: {e:?}"));
            let differs = assert_combat_at(&run, ascension, &format!("{encounter:?}"));
            assert_eq!(differs, ascension >= 2, "{encounter:?} A{ascension}: A2 is the normals' threshold");
            win(&mut run);
            assert!(run.state.combat.is_none() && run.state.fault.is_none(), "{encounter:?} A{ascension} settles");
        }
    }
}

#[test]
fn m29_act2_bosses_a4_each_character() {
    for (i, character) in CHARACTERS.into_iter().enumerate() {
        // Every Act 2 boss through the event executor at A4 (A3 is the control: no A4 change yet).
        for boss in [EncounterId::Champ, EncounterId::Collector, EncounterId::BronzeAutomaton] {
            for ascension in [3, 4] {
                let mut run = at_event(character, 2, EventId::GoldenShrine, 0x70 + i as u8);
                run.state.ascension = ascension;
                event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: boss, rewards: None, next_stage: DONE, elite: false }]).unwrap();
                assert_eq!(assert_combat_at(&run, ascension, &format!("{character:?} {boss:?}")), ascension == 4, "{character:?} {boss:?} A{ascension}");
                win(&mut run);
                assert!(run.state.fault.is_none(), "{character:?} {boss:?} A{ascension}");
            }
        }
        // The map's own Act 2 boss on a public A4 Run (supplied HP, constructed lethal frontier).
        let mut run = on_map_at(character, 4, 0x78 + i as u8);
        run.state.max_hp = 999; run.state.hp = 999;
        drive(&mut run, |r| r.state.act == 2 && r.state.combat.is_some() && origin_kind(r) == Some(MapNodeKind::Boss));
        assert!(assert_combat_at(&run, 4, &format!("{character:?} Act 2 map boss")));
        win(&mut run);
        assert!(run.state.fault.is_none(), "{character:?}");
    }
}

#[test]
fn m29_act4_shield_spear_a3_and_heart_a4() {
    for (character, ascension, encounter) in [(Character::Defect, 3, EncounterId::ShieldAndSpear), (Character::Watcher, 4, EncounterId::CorruptHeart)] {
        let mut run = on_map_at(character, ascension, 0x80 + ascension);
        run.state.max_hp = 999; run.state.hp = 999; run.state.keys = [true; 3];
        let mut act3_boss = false;
        drive(&mut run, |r| {
            if r.state.act == 3 && r.state.combat.is_some() && origin_kind(r) == Some(MapNodeKind::Boss) && !act3_boss {
                act3_boss = true;
                assert_combat_at(r, ascension, &format!("{character:?} Act 3 boss"));
            }
            r.state.act == 4 && r.state.combat.is_some() && r.state.active_combat.as_ref().is_some_and(|c| c.encounter == encounter)
        });
        assert!(act3_boss, "{character:?} fought the Act 3 boss");
        assert!(assert_combat_at(&run, ascension, &format!("{character:?} {encounter:?}")), "{encounter:?} A{ascension} differs from A0");
        win(&mut run);
        assert!(run.state.fault.is_none(), "{character:?} {encounter:?}");
    }
}

/// Review m1: the traversal reaches no Act 4 at any level (also not at base A0), so a constructed public Run
/// covers the Act 3 boss and all of Act 4 (Shield and Spear, then the Heart) at every one of A1–A5 (supplied
/// HP and keys, constructed lethal frontier). Every advertised Act 4 candidate is stepped on a clone and
/// none is `Unsupported`.
#[test]
fn m29_act3_boss_and_act4_a1_to_a5() {
    for ascension in 1..=5u8 {
        let character = CHARACTERS[usize::from(ascension - 1) % CHARACTERS.len()];
        let mut run = on_map_at(character, ascension, 0x90 + ascension);
        // 400 HP (as the supplied mode): 999 overflows a maximum-HP gain offered in Act 4.
        run.state.max_hp = 400; run.state.hp = 400; run.state.keys = [true; 3];
        let (mut act3_boss, mut fought, mut checked) = (false, Vec::new(), 0);
        drive(&mut run, |r| {
            if r.state.act == 3 && r.state.combat.is_some() && origin_kind(r) == Some(MapNodeKind::Boss) && !act3_boss {
                act3_boss = true;
                assert_combat_at(r, ascension, &format!("{character:?} A{ascension} Act 3 boss"));
            }
            if r.state.act == 4 {
                for candidate in r.decision().unwrap().candidates {
                    if let Err(RunError::Unsupported(message)) = r.clone().step(&candidate.action) {
                        panic!("{character:?} A{ascension}: Act 4 advertised {:?} is Unsupported: {message}", candidate.action);
                    }
                    checked += 1;
                }
                if let Some(encounter) = r.state.active_combat.as_ref().filter(|_| r.state.combat.is_some()).map(|c| c.encounter) {
                    if fought.last() != Some(&encounter) {
                        assert_combat_at(r, ascension, &format!("{character:?} A{ascension} {encounter:?}"));
                        fought.push(encounter);
                    }
                }
            }
            r.state.terminal.is_some() || fought.last() == Some(&EncounterId::CorruptHeart)
        });
        assert!(act3_boss, "{character:?} A{ascension} fought the Act 3 boss");
        assert_eq!(fought, [EncounterId::ShieldAndSpear, EncounterId::CorruptHeart], "{character:?} A{ascension} Act 4");
        win(&mut run);
        assert!(run.state.fault.is_none(), "{character:?} A{ascension} Heart settles");
        assert!(checked > 0, "{character:?} A{ascension}: Act 4 candidates checked");
    }
}

/// M29f gate "A1–A5 public" (spot check; the traversal is the full proof): in any build, a public
/// Standard Run at A1–A5 for each character advertises no candidate that steps to `Unsupported`,
/// through the Act 1 boss (route driver, constructed lethal frontier).
#[test]
fn m29_public_a1_a5_gate() {
    for character in CHARACTERS {
        for ascension in 1..=5 {
            let mut run = on_map_at(character, ascension, 0x29 + ascension);
            run.state.max_hp = 400; run.state.hp = 400;
            let mut checked = 0;
            drive(&mut run, |r| {
                for candidate in r.decision().unwrap().candidates {
                    if let Err(RunError::Unsupported(message)) = r.clone().step(&candidate.action) {
                        panic!("{character:?} A{ascension}: advertised {:?} is Unsupported: {message}", candidate.action);
                    }
                    checked += 1;
                }
                r.state.act == 2
            });
            assert_eq!(run.state.act, 2, "{character:?} A{ascension} reaches Act 2");
            assert!(checked > 50, "{character:?} A{ascension}: {checked} candidates checked");
        }
    }
}
