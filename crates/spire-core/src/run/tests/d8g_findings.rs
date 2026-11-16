//! spire-d8g: Run-level checks behind the 2026-11-16 stress-campaign findings (the saved-prefix replays are in
//! `spire-env/src/tests/d8g_runs.rs`).
use super::super::*;
use super::m28_event_core::{at_event, before_event, option, view};
use crate::content::{CardId, Character, EventId, RelicId};

fn select_instance(run: &mut Run, instance: u64) {
    let candidate = run.state.pending.as_ref().unwrap().selection.candidates.iter()
        .find(|candidate| candidate.card.instance == instance).unwrap().id;
    run.step(&GameAction::Select(candidate)).unwrap();
}

/// A deck whose first card is `curse` (then Strikes), plus Omamori with `charges`.
fn curse_deck(run: &mut Run, curse: CardId, charges: i32) -> u64 {
    run.state.deck = [curse, CardId::Strike, CardId::Strike].into_iter().enumerate().map(|(index, id)|
        CardInstance { id, instance: 90_000 + index as u64, upgrades: 0, misc: 0 }).collect();
    run.state.relic_cards.clear();
    run.state.relics.push(RelicId::Omamori);
    run.state.relic_counters.insert(RelicId::Omamori, charges);
    90_000
}

fn curses(run: &Run) -> usize {
    run.state.deck.iter().filter(|card| crate::content::Catalog::standard().card(card.id)
        .is_some_and(|row| row.card_type == crate::content::CardType::Curse)).count()
}

/// E1: an event transform of a curse obtains its output through `ShowCardAndObtainEffect`, whose constructor spends an
/// Omamori charge on a curse; the input stays removed and the new curse is not added (lane D Q4). Living Wall
/// "Change" and Transmogrifier share the event selection transform; it used to fault
/// (`Unsupported("a negated card obtain needs an *_or_negated owner")`).
#[test]
fn d8g_e1_event_transform_of_a_curse_spends_omamori_and_adds_nothing() {
    for (event, seed, act) in [(EventId::LivingWall, 0xD1, 1u8), (EventId::Transmogrifier, 0xD2, 1)] {
        let (mut run, node) = before_event(Character::Ironclad, act, event, seed);
        let regret = curse_deck(&mut run, CardId::Regret, 2);
        run.step(&GameAction::Move(node)).unwrap();
        assert_eq!(view(&run).kind, RoomViewKind::Event(event));
        option(&mut run, if event == EventId::LivingWall { 1 } else { 0 }).unwrap();
        select_instance(&mut run, regret);
        assert_eq!(run.state.deck.iter().map(|card| card.id).collect::<Vec<_>>(), vec![CardId::Strike, CardId::Strike],
            "{event:?}: Regret removed, no curse added");
        assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&1), "{event:?}: one charge spent");
    }
}

/// E1: Drug Dealer transforms two cards one after another; one Omamori charge negates the first transformed curse
/// only, so the second transformed curse is added.
#[test]
fn d8g_e1_drug_dealer_two_curse_transforms_with_one_omamori_charge() {
    let (mut run, node) = before_event(Character::Ironclad, 2, EventId::DrugDealer, 0xD3);
    curse_deck(&mut run, CardId::Regret, 1);
    run.state.deck[1].id = CardId::Doubt;
    run.step(&GameAction::Move(node)).unwrap();
    option(&mut run, 1).unwrap();
    select_instance(&mut run, 90_000);
    select_instance(&mut run, 90_001);
    assert_eq!(run.state.deck.len(), 2, "two removed, one curse added");
    assert_eq!(curses(&run), 1);
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&0));
}

/// E3: Tomb of Lord Red Mask with Red Mask and Ectoplasm. The original's gainGold returns before changing gold, so the
/// 222 gold is blocked entirely and nothing else changes (lane D Q6); the event's frontier used to expect the gold
/// and fault (`accepted Act3 event mutation frontier changed`). Without Ectoplasm the gold is gained.
#[test]
fn d8g_e3_tomb_gold_is_blocked_by_ectoplasm() {
    for ectoplasm in [true, false] {
        let (mut run, node) = before_event(Character::Watcher, 3, EventId::TombOfLordRedMask, 0xD5);
        run.state.gold = 83;
        run.state.relics.push(RelicId::RedMask);
        if ectoplasm { run.state.relics.push(RelicId::Ectoplasm); }
        run.step(&GameAction::Move(node)).unwrap();
        let tomb = view(&run).choices.into_iter().find(|row| row.kind == RoomChoiceKind::TombGold).unwrap().id;
        let (relics, deck) = (run.state.relics.clone(), run.state.deck.clone());
        run.step(&GameAction::Choose(tomb)).unwrap();
        assert_eq!(run.state.gold, if ectoplasm { 83 } else { 305 });
        assert_eq!((&run.state.relics, &run.state.deck), (&relics, &deck));
        assert!(run.state.fault.is_none());
    }
}

/// E1: the shared transform owner itself (Neow and Astrolabe already call it): a negated output leaves the removal.
#[test]
fn d8g_e1_transform_owner_keeps_the_removal_when_omamori_negates() {
    let mut run = at_event(Character::Silent, 1, EventId::LivingWall, 0xD4);
    let doubt = curse_deck(&mut run, CardId::Doubt, 1);
    let created = mutations::transform_card_or_negated(&mut run.state, doubt, false, &[1, 2, 3]).unwrap();
    assert_eq!((created, run.state.deck.len(), curses(&run)), (None, 2, 0));
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&0));
}
