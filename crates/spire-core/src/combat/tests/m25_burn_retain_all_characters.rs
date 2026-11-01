//! R4 (all characters): Burn discards itself at end of turn even when a retain effect selected it
//! (implementation-notes BURN). This covers the generic temporary-retain path, so a Watcher-only
//! fix would fail it.
use super::super::*;
use super::full_runtime::entered_combat;
use super::super::model::OwnedCard;
use crate::content::CardId;

#[test]
fn m25_silent_well_laid_plans_cannot_retain_a_burn() {
    let mut combat = entered_combat(3, 3, |_| {});
    for instance in 4..=18 {
        let card = OwnedCard::Full(card_state::permanent(CardId::Defend, CardInstanceId(instance), 0));
        combat.state.card_ids_issued.insert(card.handle());
        combat.state.piles.draw.push(card);
    }
    combat.state.character = characters::CharacterState::for_character(crate::content::Character::Silent);
    combat.state.player_full_powers.push(FullPowerEntry {
        id: FullPowerId("retain_cards"),
        state: PowerState { amount: 1, skip_first: false, monster_source_grace: false },
    });
    let burn = OwnedCard::Full(card_state::generated(CardId::Burn, CardInstanceId(90), 0).expect("Burn constructor"));
    let handle = burn.handle();
    combat.state.card_ids_issued.insert(handle.clone());
    combat.state.piles.hand.push(burn);
    piles::refresh_hand_values(&mut combat.state);

    assert_eq!(combat.step_full(&CombatAction::EndTurn), Ok(()));
    let selection = combat.state.pending_selection.as_ref().expect("Well-Laid Plans opens its retain selection");
    assert!(selection.candidates.contains(&handle), "the Burn can be selected");
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance: 90 }), Ok(()));
    assert_eq!(combat.step_full(&CombatAction::ConfirmSelection), Ok(()));
    assert!(!combat.state.piles.hand.iter().any(|card| card.handle() == handle), "a retained Burn still leaves the hand");
    assert!(combat.state.piles.discard.iter().any(|card| card.handle() == handle), "the Burn discards itself");
}
