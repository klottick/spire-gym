//! M31 event rows (plan items 5 and 7): A13 Mind Bloom gold, A15 Designer Punch, the A14 arms of the
//! A15-only M28 tests and the Standard-route A15 reachability (source check §D, §F). Each value is
//! pinned at its t−1/t boundary; literals come from the source check, never the body under test.
use super::super::*;
use super::m28_event_core::{before_event, row, view};
use crate::content::{Character, EventId};
use crate::run::{GameAction, RoomChoiceKind, RoomViewKind, Run};
use rooms::standard::event_exec::Outcome;
use rooms::standard::events;

fn entered(character: Character, event: EventId, act: u8, ascension: u8, seed: u8) -> Run {
    let (mut run, node) = before_event(character, act, event, seed);
    run.state.ascension = ascension;
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(event));
    run
}
fn outcomes(run: &Run, stage: u8, option: u8) -> Vec<Outcome> {
    let frame = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap();
    (events::body(frame.event).unwrap().choose)(&run.state, &frame.captured, stage, option).unwrap()
}

/// MindBloom.java (source check §D, §F): "I am War" pays 50 gold, 25 from A13.
#[test]
fn m31_a13_mind_bloom_gold() {
    for character in [Character::Ironclad, Character::Watcher] {
        for (ascension, gold) in [(0_u8, 50_u32), (12, 50), (13, 25), (15, 25)] {
            let run = entered(character, EventId::MindBloom, 3, ascension, 40 + ascension);
            let fight = outcomes(&run, 0, 0);
            assert!(matches!(fight.first(), Some(Outcome::Combat { rewards: Some(spec), .. }) if spec.gold == gold),
                "{character:?} A{ascension}: I am War gold {gold}, saw {fight:?}");
            let _ = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 });
        }
    }
}
