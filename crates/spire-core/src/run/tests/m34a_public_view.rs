//! M34a a4: displayed values that must reach `PublicDecision` (bottled-card marker, chest size).
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{CardId, Character, RelicId};

fn standard_on_map(character: Character) -> Run {
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    let mut run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile,
        RunRng::new([0x34; 32], Partition::Test, 0), RoomScenario::Standard).unwrap();
    let choice = rooms::public(&run).unwrap().choices[1].id;
    run.step(&GameAction::Choose(choice)).unwrap();
    assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Map);
    run
}

fn enter_first(run: &mut Run, kind: MapNodeKind) {
    let target = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(id) => Some(id), _ => None }).unwrap();
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == target).unwrap().kind = kind;
    run.step(&GameAction::Move(target)).unwrap();
}

/// The game draws a bottle on the bottled card; the link is player knowledge out of and in combat.
#[test]
fn bottled_card_marker_is_public() {
    let mut run = standard_on_map(Character::Ironclad);
    let bash = run.state.deck.iter().find(|card| card.id == CardId::Bash).unwrap().instance;
    run.state.relics.push(RelicId::BottledFlame);
    run.state.relic_cards.insert(RelicId::BottledFlame, bash);
    assert_eq!(run.decision().unwrap().player.bottled_cards, vec![(RelicId::BottledFlame, bash)]);
    enter_first(&mut run, MapNodeKind::Monster);
    let decision = run.decision().unwrap();
    assert!(decision.combat.is_some(), "entered a combat");
    assert_eq!(decision.player.bottled_cards, vec![(RelicId::BottledFlame, bash)], "in combat");
    let plain = standard_on_map(Character::Silent);
    assert!(plain.decision().unwrap().player.bottled_cards.is_empty());
}

use crate::content::EventId;
use super::m28_event_core::{before_event, row, leave};

/// Params published on an event option row (`[None; 3]` when the row publishes none).
fn option_params(run: &Run, stage: u8, option: u8) -> [Option<i32>; 3] {
    let id = row(run, &RoomChoiceKind::EventOption { stage, option }).id;
    let params = run.decision().unwrap().room_params.expect("Standard event rooms publish room params");
    params.rows.iter().find(|(choice, _)| *choice == id).map_or([None; 3], |(_, values)| *values)
}

/// a3 class (ii): World of Goop's Leave shows the gold it will take, captured at entry and capped by
/// the gold held then (A0 20–50, A15 35–75). The shown value is exactly what Leave removes.
#[test]
fn world_of_goop_leave_publishes_captured_gold_loss() {
    for ascension in [0u8, 15] {
        let (low, high) = if ascension >= 15 { (35, 75) } else { (20, 50) };
        for seed in 0..24u8 {
            for entry_gold in [10u32, 30, 99] {
                let (mut run, node) = before_event(Character::Silent, 1, EventId::WorldOfGoop, seed);
                run.state.ascension = ascension;
                run.state.gold = entry_gold;
                run.step(&GameAction::Move(node)).unwrap();
                let [shown, none1, none2] = option_params(&run, 0, 1);
                let shown = shown.expect("Leave shows its gold loss") as u32;
                assert_eq!((none1, none2), (None, None));
                assert!(shown <= entry_gold && (shown == entry_gold || (low..=high).contains(&shown)),
                    "A{ascension} gold {entry_gold}: shown {shown}");
                assert_eq!(option_params(&run, 0, 0), [Some(75), Some(11), None], "Gather: gold gained, damage taken");
                let before = run.state.gold;
                let id = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 }).id;
                run.step(&GameAction::Choose(id)).unwrap();
                assert_eq!(before - run.state.gold, shown, "Leave removes exactly the shown loss");
                leave(&mut run);
            }
        }
    }
}

/// a4: the chest's size is visible on entering a Treasure room, before it is opened; reading it
/// consumes no randomness and opening uses the same size.
#[test]
fn treasure_chest_size_is_public_before_opening() {
    let mut seen = [false; 3];
    for seed in 0..40u8 {
        let mut run = standard_on_map_seed(Character::Defect, seed);
        enter_first(&mut run, MapNodeKind::Treasure);
        assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Treasure);
        let before_rng = run.state.rng.clone();
        let size = run.decision().unwrap().room_params.expect("unopened chest publishes its size").stage[0].expect("size");
        assert_eq!(run.state.rng, before_rng, "reading the size consumes no randomness");
        assert!((1..=3).contains(&size));
        seen[(size - 1) as usize] = true;
        let opened_size = rooms::standard::treasure::opened_size_for_test(&run).expect("same key");
        assert_eq!(size, opened_size, "published size = the size opening draws");
        let open = rooms::public(&run).unwrap().choices.into_iter().find(|c| c.kind == RoomChoiceKind::OpenChest).unwrap().id;
        run.step(&GameAction::Choose(open)).unwrap();
        assert!(run.decision().unwrap().room_params.map_or(true, |p| p.stage[0].is_none()), "an opened chest no longer shows a size row");
    }
    assert!(seen.iter().all(|s| *s), "small, medium and large all occur: {seen:?}");
}

fn standard_on_map_seed(character: Character, seed: u8) -> Run {
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    let mut run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile,
        RunRng::new([seed; 32], Partition::Test, 0), RoomScenario::Standard).unwrap();
    let choice = rooms::public(&run).unwrap().choices[1].id;
    run.step(&GameAction::Choose(choice)).unwrap();
    run
}
