//! M28f2 close (plan Task X1, design §14): the traversal smoke test and the global fixes the
//! traversal found. The 25-seed runs and their census are `examples/m28_traversal.rs`.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{Character, RelicId};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn standard(character: Character, full: bool, seed: u8) -> Run {
    let profile = CreationProfile { neow_eligible: full, ..CreationProfile::experienced() };
    Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile, RunRng::new([seed; 32], Partition::Test, 0), RoomScenario::Standard).unwrap()
}

#[test]
fn m28_g14_combat_won_during_entry_settles_all_characters() {
    // Traversal uniform/Watcher/16: Neow's Lament (enemies at 1 HP) and Mercury Hourglass (3 to all at
    // turn start) kill every enemy during entry. The source ends the battle as soon as the monsters
    // are dead (AbstractRoom.update), so the Run settles at once; before, it kept a won combat with
    // no candidates (a softlock).
    for character in CHARACTERS {
        let mut run = standard(character, false, 0x31);
        let lament = rooms::public(&run).unwrap().choices[0].id;
        run.step(&GameAction::Choose(lament)).unwrap();
        run.state.relics.push(RelicId::MercuryHourglass);
        let target = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(n) => Some(n), _ => None }).unwrap();
        let transition = run.step(&GameAction::Move(target)).unwrap();
        assert!(run.state.combat.is_none() && run.state.active_combat.is_none(), "{character:?}: settled at entry");
        assert_eq!((run.state.combat_progress.entered, run.state.combat_progress.settled), (1, 1), "{character:?}");
        assert!(transition.committed_events.iter().any(|e| matches!(e, CommittedEvent::CombatFinished(record) if record.outcome == crate::combat::Outcome::Victory)), "{character:?}");
        let decision = run.decision().unwrap();
        assert_eq!(decision.room.map(|room| room.kind), Some(RoomViewKind::Rewards), "{character:?}: the normal reward screen");
        assert!(!decision.candidates.is_empty(), "{character:?}");
    }
}

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[test]
fn m28_traversal_smoke() {
    // Design §14 / plan X1: one seed per character, short and full Neow, a seeded uniform random legal
    // policy to a terminal under the 20,000-action cap. Every decision is the standard scenario, every
    // advertised candidate is accepted on a clone, and the combat runtime validator holds after every step.
    for character in CHARACTERS {
        for full in [false, true] {
            let mut run = standard(character, full, 0x28);
            let mut policy = 0x4D32_3846_u64 ^ u64::from(full) ^ ((character as u64) << 8);
            let mut steps = 0;
            loop {
                let decision = run.decision().unwrap();
                assert_eq!(decision.content_profile, Some(RoomScenario::Standard));
                if decision.terminal.is_some() { break } // a death is a policy outcome
                assert!(!decision.candidates.is_empty(), "{character:?} full={full} step {steps}: no candidates at {:?}", decision.room);
                assert!(steps < 20_000, "{character:?} full={full}: action cap");
                for candidate in &decision.candidates {
                    let mut clone = run.clone();
                    clone.step(&candidate.action).unwrap_or_else(|error| panic!("{character:?} full={full} step {steps}: {:?} rejected: {error:?}", candidate.action));
                }
                let index = (splitmix(&mut policy) % decision.candidates.len() as u64) as usize;
                run.step(&decision.candidates[index].action).unwrap();
                assert_eq!(run.state.combat.is_some(), run.state.active_combat.is_some(), "{character:?} full={full}: combat owner");
                if let Some(combat) = &run.state.combat { combat.validate_runtime_for_test().unwrap(); combat.validate_permanent_mutations_for_run().unwrap(); }
                steps += 1;
            }
        }
    }
}

/// Review m1 (reviewer probes `m28f2-review-v1/g14-probe-tests-v1.rs`, adopted): G14 on the elite
/// and boss entry paths. A Standard elite and Act 1 boss decided at entry settle to their normal
/// reward screens.
#[test]
fn m28_g14_elite_and_boss_won_during_entry_settle() {
    for character in [Character::Ironclad, Character::Watcher] {
        for kind in [MapNodeKind::Elite, MapNodeKind::Boss] {
            let mut run = standard(character, false, 0x45);
            let lament = rooms::public(&run).unwrap().choices[0].id;
            run.step(&GameAction::Choose(lament)).unwrap();
            run.state.relics.push(RelicId::MercuryHourglass);
            let target = if kind == MapNodeKind::Elite {
                let t = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(n) => Some(n), _ => None }).unwrap();
                run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == t).unwrap().kind = MapNodeKind::Elite;
                t
            } else {
                let map = run.state.map.as_mut().unwrap();
                let boss = map.nodes.iter().find(|n| n.kind == MapNodeKind::Boss).unwrap().clone();
                let from = map.nodes.iter().find(|n| n.edges.contains(&boss.id)).unwrap().clone();
                map.current = Some(from.id);
                for n in map.nodes.iter_mut() { if n.id == from.id { n.visited = true; } }
                run.state.floor = from.floor;
                run.state.room_loop.as_mut().unwrap().screen = RoomView { kind: RoomViewKind::Map, choices: vec![] };
                run.candidates.clear();
                boss.id
            };
            let transition = run.step(&GameAction::Move(target)).unwrap_or_else(|e| panic!("{character:?} {kind:?}: {e:?} fault {:?}", run.state.fault));
            assert!(run.state.fault.is_none() && run.state.combat.is_none() && run.state.active_combat.is_none(), "{character:?} {kind:?}: settled");
            assert!(transition.committed_events.iter().any(|e| matches!(e, CommittedEvent::CombatFinished(record) if record.outcome == crate::combat::Outcome::Victory)), "{character:?} {kind:?}");
            let room = run.decision().unwrap().room.unwrap();
            assert_eq!(room.kind, RoomViewKind::Rewards, "{character:?} {kind:?}");
            let shape: Vec<_> = room.choices.iter().map(|c| match c.kind { RoomChoiceKind::Gold(_) => "gold", RoomChoiceKind::Relic(_) => "relic", RoomChoiceKind::Potion(_) => "potion", RoomChoiceKind::CardReward => "card", RoomChoiceKind::Proceed => "proceed", _ => "other" }).collect();
            let expected = if kind == MapNodeKind::Elite { vec!["gold", "relic", "potion", "card", "proceed"] } else { vec!["gold", "card", "proceed"] };
            assert_eq!(shape, expected, "{character:?} {kind:?}");
        }
    }
}

/// Review m1 (adopted probe): G14 inside an event. The Colosseum's first fight (rewards disabled)
/// settles during entry through the nested `event_frame::after_combat` and reopens the event at
/// stage 1; the second fight (rewards enabled) settles to the reward screen.
#[test]
fn m28_g14_colosseum_fights_won_during_entry_settle_all_characters() {
    use super::m28_event_core::view;
    use super::m28_events_act2::{colosseum, option_row};
    for character in CHARACTERS {
        let mut run = colosseum(character, 104);
        if !run.state.relics.contains(&RelicId::NeowsLament) { run.state.relics.push(RelicId::NeowsLament); }
        run.state.relic_counters.insert(RelicId::NeowsLament, 3);
        run.state.relics.push(RelicId::MercuryHourglass);
        option_row(&mut run, 0, 0);
        assert!(run.state.fault.is_none(), "{character:?}: {:?}", run.state.fault);
        assert!(run.state.combat.is_none() && run.state.active_combat.is_none(), "{character:?}: settled");
        let rows = view(&run);
        assert_eq!(rows.kind, RoomViewKind::Event(crate::content::EventId::Colosseum), "{character:?}");
        let kinds: Vec<_> = rows.choices.iter().map(|c| c.kind.clone()).collect();
        assert_eq!(kinds, vec![RoomChoiceKind::EventOption { stage: 1, option: 0 }, RoomChoiceKind::EventOption { stage: 1, option: 1 }], "{character:?}");
        option_row(&mut run, 1, 1);
        assert!(run.state.fault.is_none() && run.state.combat.is_none(), "{character:?}: the Nobs fight settled");
        let decision = run.decision().unwrap();
        assert_eq!(decision.room.map(|room| room.kind), Some(RoomViewKind::Rewards), "{character:?}");
        assert!(!decision.candidates.is_empty(), "{character:?}");
    }
}

/// Review m8: the public Standard constructor rejects a stored note that Note For Yourself's
/// take row could not obtain (not storable, or more upgrades than the card allows).
#[test]
fn m28_standard_a0_rejects_an_unobtainable_stored_note() {
    use crate::content::CardId;
    let create = |stored_note| {
        let profile = CreationProfile { neow_eligible: false, stored_note, ..CreationProfile::experienced() };
        Run::new_room_scenario(RunConfig { character: Character::Silent, ascension: 0 }, profile, RunRng::new([0x46; 32], Partition::Test, 0), RoomScenario::Standard).map(|_| ())
    };
    let rejected = Err(RunError::Unsupported("Standard stored note must be a storable card with legal upgrades"));
    for bad in [(CardId::IronWave, 2), (CardId::Dazed, 0)] { assert_eq!(create(bad), rejected, "{bad:?}"); }
    for good in [(CardId::IronWave, 0), (CardId::IronWave, 1), (CardId::SearingBlow, 5)] { assert_eq!(create(good), Ok(()), "{good:?}"); }
}
