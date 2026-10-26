//! Finite M14 choices over the supplied public combat selection.
//!
//! Discard order is: useful Reflex/Tactician triggers, unplayable/status/curse,
//! an already-active supported power, then lowest fixed `card_value`. Reflex
//! is useful when a draw remains and `hand.len().saturating_sub(1) < 10`;
//! Tactician always grants its printed energy, including above max energy. In
//! the final discard tier only, protect affordable positive-block cards when
//! targetable published incoming damage exceeds current block, if another
//! unprotected card is available. Ties in low-value choices use lower value,
//! then lower instance id.
//!
//! Retain first chooses a legal usable block card when no already-selected
//! retained card supplies block, then the highest-valued legal useful card.
//! It skips unplayable/status/curse cards and powers already active, and
//! confirms when no useful choice remains or the maximum is reached, but only
//! when confirmation is offered. Copy, Nightmare, return and topdeck choose a
//! usable block card under published pressure; otherwise they choose highest
//! fixed card value. Optional utility skips current unplayables; mandatory
//! selections fall back to the highest-valued legal card. All high-value ties
//! use the lower instance id. No branch simulates an action or future draw.

use std::cmp::Reverse;

use spire_core::{
    combat::{
        CardCostKind, CombatAction, CombatPublic, LiveCardView, SelectionDisposition,
        SelectionOperation, SelectionView,
    },
    content::{CardId, Catalog, CardType},
};

pub(super) fn selection_action(
    combat: &CombatPublic,
    legal: &[CombatAction],
) -> Option<CombatAction> {
    let selection = combat.selection.as_ref()?;
    if selection.minimum > selection.maximum
        || selection.required > selection.maximum
        || selection.selected.len() > selection.maximum
    {
        return None;
    }

    match selection.operation {
        SelectionOperation::Move(SelectionDisposition::Discard) => {
            discard_action(combat, selection, legal)
        }
        SelectionOperation::Retain => retain_action(combat, selection, legal),
        SelectionOperation::CopyToHand { .. }
        | SelectionOperation::NightmareCapture
        | SelectionOperation::ReturnFromExhaust
        | SelectionOperation::Topdeck => transfer_action(combat, selection, legal),
        _ => None,
    }
}

fn discard_action(
    combat: &CombatPublic,
    selection: &SelectionView,
    legal: &[CombatAction],
) -> Option<CombatAction> {
    if selection.selected.len() >= selection.maximum {
        return legal_confirmation(selection, legal);
    }
    let candidates = legal_candidates(selection, legal);

    let triggered = candidates
        .iter()
        .copied()
        .filter(|card| discard_trigger_is_useful(card, combat))
        .collect::<Vec<_>>();
    if !triggered.is_empty() {
        return select(lowest_value(&triggered)?, legal);
    }

    let junk = candidates
        .iter()
        .copied()
        .filter(|card| discard_junk(card))
        .collect::<Vec<_>>();
    if !junk.is_empty() {
        return select(lowest_value(&junk)?, legal);
    }

    let redundant = candidates
        .iter()
        .copied()
        .filter(|card| power_is_active(card, combat))
        .collect::<Vec<_>>();
    if !redundant.is_empty() {
        return select(lowest_value(&redundant)?, legal);
    }

    let pressured = public_pressure(combat);
    let ordinary = candidates
        .iter()
        .copied()
        .filter(|card| !power_is_active(card, combat) && !discard_junk(card))
        .collect::<Vec<_>>();
    if pressured {
        let unprotected = ordinary
            .iter()
            .copied()
            .filter(|card| !usable_block(card, combat.player.energy))
            .collect::<Vec<_>>();
        if !unprotected.is_empty() {
            return select(lowest_value(&unprotected)?, legal);
        }
    }
    if let Some(card) = lowest_value(&ordinary) {
        return select(card, legal);
    }

    if required_more(selection) {
        return select(lowest_value(&candidates)?, legal);
    }
    legal_confirmation(selection, legal)
}

fn retain_action(
    combat: &CombatPublic,
    selection: &SelectionView,
    legal: &[CombatAction],
) -> Option<CombatAction> {
    if selection.selected.len() >= selection.maximum {
        return legal_confirmation(selection, legal);
    }
    let candidates = legal_candidates(selection, legal);
    let useful = candidates
        .iter()
        .copied()
        .filter(|card| !is_current_unplayable(card) && !power_is_active(card, combat))
        .collect::<Vec<_>>();

    let retained_block = selection.selected.iter().any(|instance| {
        selection
            .candidates
            .iter()
            .find(|card| card.instance == *instance)
            .is_some_and(|card| usable_block(card, combat.player.max_energy))
    });
    if !retained_block {
        let blocks = useful
            .iter()
            .copied()
            .filter(|card| usable_block(card, combat.player.max_energy))
            .collect::<Vec<_>>();
        if let Some(card) = highest_value(&blocks) {
            return select(card, legal);
        }
    }
    if let Some(card) = highest_value(&useful) {
        return select(card, legal);
    }
    legal_confirmation(selection, legal)
}

fn transfer_action(
    combat: &CombatPublic,
    selection: &SelectionView,
    legal: &[CombatAction],
) -> Option<CombatAction> {
    if selection.selected.len() >= selection.maximum {
        return legal_confirmation(selection, legal);
    }
    let candidates = legal_candidates(selection, legal);
    let useful = candidates
        .iter()
        .copied()
        .filter(|card| !is_current_unplayable(card))
        .collect::<Vec<_>>();

    if public_pressure(combat) {
        let blocks = useful
            .iter()
            .copied()
            .filter(|card| usable_block(card, combat.player.energy))
            .collect::<Vec<_>>();
        if let Some(card) = highest_value(&blocks) {
            return select(card, legal);
        }
    }
    if let Some(card) = highest_value(&useful) {
        return select(card, legal);
    }
    if !required_more(selection) {
        return legal_confirmation(selection, legal);
    }

    // A required selection cannot be skipped; choose only from the current
    // legal menu even when every remaining source is currently unplayable.
    select(highest_value(&candidates)?, legal)
}

fn legal_candidates<'a>(
    selection: &'a SelectionView,
    legal: &[CombatAction],
) -> Vec<&'a LiveCardView> {
    if selection.selected.len() >= selection.maximum {
        return Vec::new();
    }
    selection
        .candidates
        .iter()
        .filter(|card| !selection.selected.contains(&card.instance))
        .filter(|card| has_select(legal, card.instance.0))
        .collect()
}

fn has_select(legal: &[CombatAction], instance: u64) -> bool {
    legal.iter().any(|action| {
        matches!(action, CombatAction::SelectCard { instance: offered } if *offered == instance)
    })
}

fn select(card: &LiveCardView, legal: &[CombatAction]) -> Option<CombatAction> {
    legal
        .iter()
        .find(|action| {
            matches!(action, CombatAction::SelectCard { instance } if *instance == card.instance.0)
        })
        .cloned()
}

fn legal_confirmation(
    selection: &SelectionView,
    legal: &[CombatAction],
) -> Option<CombatAction> {
    if selection.selected.len() < selection.required.max(selection.minimum)
        || selection.selected.len() > selection.maximum
    {
        return None;
    }
    legal
        .iter()
        .find(|action| matches!(action, CombatAction::ConfirmSelection))
        .cloned()
}

fn required_more(selection: &SelectionView) -> bool {
    selection.selected.len() < selection.required.max(selection.minimum)
}

fn lowest_value<'a>(cards: &[&'a LiveCardView]) -> Option<&'a LiveCardView> {
    cards
        .iter()
        .copied()
        .min_by_key(|card| (super::card_value(card.id), card.instance.0))
}

fn highest_value<'a>(cards: &[&'a LiveCardView]) -> Option<&'a LiveCardView> {
    cards
        .iter()
        .copied()
        .max_by_key(|card| (super::card_value(card.id), Reverse(card.instance.0)))
}

fn discard_trigger_is_useful(card: &LiveCardView, combat: &CombatPublic) -> bool {
    match card.id {
        CardId::Reflex => {
            combat.draw_count > 0 && combat.hand.len().saturating_sub(1) < 10
        }
        // GainEnergy is not capped at max energy in the shared resolver.
        CardId::Tactician => true,
        _ => false,
    }
}

fn discard_junk(card: &LiveCardView) -> bool {
    // Reflex and Tactician have an unplayable play-cost, but their discard
    // trigger/value is handled above and otherwise by the fixed card value.
    if matches!(card.id, CardId::Reflex | CardId::Tactician) {
        return is_status_or_curse(card);
    }
    is_current_unplayable(card)
}

fn is_current_unplayable(card: &LiveCardView) -> bool {
    is_status_or_curse(card)
        || card
            .values
            .as_ref()
            .is_some_and(|values| values.cost_kind == CardCostKind::Unplayable)
}

fn is_status_or_curse(card: &LiveCardView) -> bool {
    Catalog::standard().card(card.id).is_some_and(|definition| {
        matches!(definition.card_type, CardType::Status | CardType::Curse)
    })
}

fn redundant_power_id(card: &LiveCardView) -> Option<&'static str> {
    match card.id {
        CardId::NoxiousFumes => Some("noxious_fumes"),
        CardId::Accuracy => Some("accuracy"),
        CardId::WellLaidPlans => Some("retain_cards"),
        _ => None,
    }
}

fn power_is_active(card: &LiveCardView, combat: &CombatPublic) -> bool {
    let Some(id) = redundant_power_id(card) else {
        return false;
    };
    combat
        .player
        .powers
        .iter()
        .any(|power| power.id.0 == id && power.amount > 0)
}

fn has_printed_block(card: &LiveCardView) -> bool {
    card.values
        .as_ref()
        .is_some_and(|values| values.base_block.is_some_and(|block| block > 0))
}

fn usable_block(card: &LiveCardView, energy: i32) -> bool {
    has_printed_block(card)
        && card.values.as_ref().is_some_and(|values| {
            values.free_play || card.effective_cost.is_some_and(|cost| cost <= energy)
        })
}

fn public_pressure(combat: &CombatPublic) -> bool {
    super::m14_threat::incoming(combat) > combat.player.block
}
