//! A power-owned selection may interrupt a card play: the play's CardAfterUse and FinishCard
//! continuations stay queued and the selection keeps its own parent (M25 once-only fault F1).
use super::super::*;
use crate::{content::{CardId, Character, EncounterId, RelicId}, items::CombatItemContext};
use std::collections::BTreeMap;

/// Hand instances 1–10, discard 11–20, empty draw pile.
fn arrange(c: &mut Combat) {
    let mut all = Vec::new();
    all.append(&mut c.state.piles.hand);
    all.append(&mut c.state.piles.draw);
    all.append(&mut c.state.piles.discard);
    all.append(&mut c.state.piles.exhaust);
    let mut by_instance = BTreeMap::new();
    for card in all {
        let id = card.full().expect("full card").instance.0;
        by_instance.insert(id, card);
    }
    c.state.piles.hand = (1..=10).map(|id| by_instance.remove(&id).expect("hand owner")).collect();
    c.state.piles.discard = (11..=20).map(|id| by_instance.remove(&id).expect("discard owner")).collect();
    c.state.piles.draw.clear();
    assert!(by_instance.is_empty(), "every constructed card has one pile owner");
    piles::refresh_hand_values(&mut c.state);
}

/// Calm Watcher, InnerPeace (instance 1) in a full hand, an empty draw pile and ten discards.
fn calm_watcher(relic: Option<RelicId>) -> Combat {
    let mut entry = super::m8_shapes::entry(EncounterId::OrbWalker);
    entry.setup.character = Character::Watcher;
    entry.setup.player.energy = 3;
    entry.setup.player.max_energy = 3;
    entry.setup.player.draw_per_turn = 1;
    entry.cards = (1..=20).map(|instance| {
        let id = if instance == 1 { CardId::InnerPeace } else { CardId::DefendPurple };
        crate::combat::m25_watcher_live_card(id, CardInstanceId(instance), 0).unwrap()
    }).collect();
    entry.opening_draw = 10;
    let relics = relic.into_iter().collect();
    let mut items = Some(CombatItemContext::new(relics, vec![None; 3]).with_m25_profile());
    let mut combat = Combat::enter_with_item_context(entry, &mut items).expect("ordinary Watcher combat enters");
    assert!(items.is_none());
    arrange(&mut combat);
    combat.state.character.stance = Stance::Calm;
    combat
}

#[test]
fn m25_melange_shuffle_scry_during_inner_peace_keeps_the_play_valid() {
    let mut combat = calm_watcher(Some(RelicId::Melange));
    let action = CombatAction::PlayCard { instance: 1, target: None };
    assert!(combat.legal_actions_full().unwrap().contains(&action));
    assert_eq!(combat.step_full(&action), Ok(()), "draw suspends at Melange Scry");
    assert_eq!(combat.state.piles.hand.len(), 10, "the one free hand slot is filled");
    let selection = combat.state.pending_selection.as_ref().expect("Melange owns a Scry selection");
    assert_eq!(selection.operation, SelectionOperation::Scry);
    assert!(matches!(&selection.parent, SelectionParent::Power { id, .. } if id.0 == "melange"));
    assert!(combat.state.pending.iter().any(|effect| effect.kind == EffectKind::Hook(Hook::CardAfterUse)));
    assert!(combat.state.pending.iter().any(|effect| effect.kind == EffectKind::FinishCard));
    let runtime = engine::validate_runtime(&combat.state);
    assert!(runtime.is_ok(), "card and relic continuations coexist: {runtime:?}");
    assert!(combat.public_full().is_ok(), "the Scry selection is publicly observable");

    // Closing the Scry resumes and finishes the suspended play.
    assert!(combat.legal_actions_full().unwrap().contains(&CombatAction::ConfirmSelection));
    combat.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(combat.state.pending_selection.is_none());
    assert!(combat.state.in_play.is_none(), "InnerPeace finished");
    assert!(combat.state.piles.discard.iter().any(|card| card.full().unwrap().instance == CardInstanceId(1)));
    assert_eq!(combat.state.piles.hand.len(), 10);
    assert_eq!(combat.state.player.energy, 2);
    assert!(combat.public_full().is_ok());
}

#[test]
fn m25_inner_peace_shuffle_without_melange_finishes_at_hand_cap() {
    let mut combat = calm_watcher(None);
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    assert_eq!(combat.state.piles.hand.len(), 10);
    assert_eq!(combat.state.piles.draw.len(), 9, "one card was drawn from the recycled ten");
    assert!(combat.state.pending_selection.is_none());
    assert!(combat.public_full().is_ok());
}

#[test]
fn m25_ironclad_full_hand_draw_after_empty_draw_shuffle_succeeds() {
    let mut entry = super::m8_shapes::entry(EncounterId::OrbWalker);
    entry.setup.character = Character::Ironclad;
    entry.setup.player.energy = 3;
    entry.setup.player.max_energy = 3;
    entry.cards = (1..=20).map(|instance| {
        let id = if instance == 1 { CardId::ShrugItOff } else { CardId::Defend };
        permanent_live_card(id, CardInstanceId(instance), 0)
    }).collect();
    entry.opening_draw = 10;
    let mut combat = Combat::enter(entry).expect("ordinary Ironclad combat enters");
    arrange(&mut combat);
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    assert_eq!(combat.state.piles.hand.len(), 10, "draw stops at the ten-card cap");
    assert_eq!(combat.state.piles.draw.len(), 9, "the reshuffled draw pile supplied one card");
    assert!(combat.public_full().is_ok());
}
