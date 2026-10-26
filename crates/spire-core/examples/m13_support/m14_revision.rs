//! Finite M14-only choices over the shared public Run decision.
//!
//! This module does not simulate purchases, deck changes, or hidden room state.
//! Every returned action is taken from the decision's currently offered
//! candidates.
use super::*;
use spire_core::run::PublicObject;

const DEFENSIVE_CARD_ORDER: [CardId; 3] = [
    CardId::Backflip,
    CardId::CloakAndDagger,
    CardId::Deflect,
];

const GENERAL_REWARD_ORDER: [CardId; 14] = [
    CardId::BladeDance,
    CardId::Backflip,
    CardId::CloakAndDagger,
    CardId::DeadlyPoison,
    CardId::PoisonedStab,
    CardId::NoxiousFumes,
    CardId::DaggerThrow,
    CardId::Deflect,
    CardId::Accuracy,
    CardId::WellLaidPlans,
    CardId::Catalyst,
    CardId::QuickSlash,
    CardId::Reflex,
    CardId::Tactician,
];

const UPGRADE_ORDER: [CardId; 6] = [
    CardId::Backflip,
    CardId::BladeDance,
    CardId::NoxiousFumes,
    CardId::Survivor,
    CardId::CloakAndDagger,
    CardId::DeadlyPoison,
];

pub(super) fn card_reward_action(decision: &PublicDecision) -> Option<GameAction> {
    if card_count(decision, CardId::NoxiousFumes) == 0 {
        if let Some(action) = card_choice(decision, CardId::NoxiousFumes) {
            return Some(action);
        }
    }

    if defense_is_short(decision) {
        if let Some(action) = first_admissible_card(decision, &DEFENSIVE_CARD_ORDER) {
            return Some(action);
        }
    } else if let Some(action) = first_admissible_card(decision, &GENERAL_REWARD_ORDER) {
        return Some(action);
    }

    offered_choice(decision, |kind| kind == &RoomChoiceKind::Skip, false)
}

pub(super) fn shop_action(decision: &PublicDecision) -> Option<GameAction> {
    if !decision.player.relics.contains(&RelicId::SneckoSkull) {
        if let Some(action) = offered_choice(
            decision,
            |kind| kind == &RoomChoiceKind::Relic(RelicId::SneckoSkull),
            true,
        ) {
            return Some(action);
        }
    }

    if defense_is_short(decision) {
        if let Some(action) = first_admissible_card(decision, &DEFENSIVE_CARD_ORDER) {
            return Some(action);
        }
    }

    let has_ratio_safe_strike = decision.objects.iter().any(|card| {
        card.id == CardId::StrikeGreen && removal_preserves_defense(decision, card, &[])
    });
    if has_ratio_safe_strike {
        if let Some(action) = offered_choice(
            decision,
            |kind| kind == &RoomChoiceKind::Purge,
            true,
        ) {
            return Some(action);
        }
    }

    let has_belt_space = decision.player.potions.iter().any(Option::is_none);
    if has_belt_space {
        for potion in [
            PotionId::DexterityPotion,
            PotionId::FirePotion,
            PotionId::WeakPotion,
            PotionId::StrengthPotion,
        ] {
            if let Some(action) = offered_choice(
                decision,
                |kind| kind == &RoomChoiceKind::Potion(potion),
                true,
            ) {
                return Some(action);
            }
        }
    }

    if !decision.player.relics.contains(&RelicId::BagOfPreparation) {
        if let Some(action) = offered_choice(
            decision,
            |kind| kind == &RoomChoiceKind::Relic(RelicId::BagOfPreparation),
            true,
        ) {
            return Some(action);
        }
    }

    if let Some(action) = reward_card_choice(decision) {
        return Some(action);
    }
    offered_choice(decision, |kind| kind == &RoomChoiceKind::Leave, false)
}

pub(super) fn room_selection_action(decision: &PublicDecision) -> Result<GameAction, String> {
    let selection = decision.selection.as_ref().ok_or("missing public room selection")?;
    if selection.eligible.is_empty() {
        return Err("public room selection has no eligible card".into());
    }

    let candidates = legal_selection_cards(decision);
    let room_kind = decision.room.as_ref().map(|room| room.kind);
    if is_removal_selection(decision) {
        return choose_removal(decision, &candidates)
            .map(|(_, _, action)| action)
            .ok_or_else(|| "public room selection has no offered card".into());
    }

    if is_upgrade_selection(decision) {
        return choose_upgrade(&candidates)
            .map(|(_, _, action)| action)
            .ok_or_else(|| "public room selection has no offered card".into());
    }

    // Keep the frozen public caller's tie/value behavior for selections that
    // are outside M14's approved removal and upgrade preferences. In
    // particular, this does not add a shop purge policy.
    let prefer_high = matches!(
        room_kind,
        Some(RoomViewKind::Event(EventId::UpgradeShrine) | RoomViewKind::Campfire)
    );
    let chosen = candidates
        .into_iter()
        .min_by_key(|(card, instance, _)| {
            (
                if prefer_high { -card_value(card.id) } else { card_value(card.id) },
                *instance,
            )
        });
    chosen
        .map(|(_, _, action)| action)
        .ok_or_else(|| "public room selection has no offered card".into())
}

pub(super) fn falling_action(decision: &PublicDecision) -> Option<GameAction> {
    let room = decision.room.as_ref()?;
    let choices = room
        .choices
        .iter()
        .filter(|choice| choice.status == RoomChoiceStatus::Offered)
        .filter_map(|choice| {
            let RoomChoiceKind::FallingLose { instance: Some(instance), .. } = choice.kind else {
                return None;
            };
            let card = decision.objects.iter().find(|card| card.instance == instance)?;
            let action = GameAction::Choose(choice.id);
            decision
                .candidates
                .iter()
                .any(|candidate| candidate.action == action)
                .then(|| (card.clone(), instance, action, choice.id))
        })
        .collect::<Vec<_>>();

    let preserves_ratio = choices.iter().any(|(card, _, _, _)| {
        removal_preserves_defense(decision, card, &[])
    });
    let chosen = choices
        .into_iter()
        .filter(|(card, _, _, _)| {
            !preserves_ratio || removal_preserves_defense(decision, card, &[])
        })
        .min_by_key(|(card, instance, _, choice_id)| {
            (
                u8::from(card.id != CardId::StrikeGreen),
                if card.id == CardId::StrikeGreen { 0 } else { card_value(card.id) },
                *instance,
                *choice_id,
            )
        })
        .map(|(_, _, action, _)| action);

    chosen.or_else(|| room_choice(decision, |kind| kind == &RoomChoiceKind::Leave))
}

fn reward_card_choice(decision: &PublicDecision) -> Option<GameAction> {
    if card_count(decision, CardId::NoxiousFumes) == 0 {
        if let Some(action) = card_choice(decision, CardId::NoxiousFumes) {
            return Some(action);
        }
    }

    let order = if defense_is_short(decision) {
        &DEFENSIVE_CARD_ORDER[..]
    } else {
        &GENERAL_REWARD_ORDER[..]
    };
    first_admissible_card(decision, order)
}

fn first_admissible_card(decision: &PublicDecision, order: &[CardId]) -> Option<GameAction> {
    for &wanted in order {
        if let Some(action) = card_choice(decision, wanted) {
            return Some(action);
        }
    }
    None
}

fn card_choice(decision: &PublicDecision, wanted: CardId) -> Option<GameAction> {
    if !admissible_card(decision, wanted) {
        return None;
    }
    offered_choice(
        decision,
        |kind| matches!(kind, RoomChoiceKind::Card { id, .. } if *id == wanted),
        true,
    )
}

fn admissible_card(decision: &PublicDecision, card: CardId) -> bool {
    let cap = match card {
        CardId::Backflip => 3,
        CardId::BladeDance | CardId::DeadlyPoison | CardId::PoisonedStab => 1,
        CardId::CloakAndDagger | CardId::Deflect | CardId::DaggerThrow => 2,
        CardId::NoxiousFumes
        | CardId::Accuracy
        | CardId::WellLaidPlans
        | CardId::QuickSlash
        | CardId::Catalyst
        | CardId::Reflex
        | CardId::Tactician => 1,
        CardId::Nightmare => 0,
        _ => return false,
    };
    if card_count(decision, card) >= cap {
        return false;
    }

    match card {
        CardId::Accuracy => {
            card_count(decision, CardId::BladeDance)
                + card_count(decision, CardId::CloakAndDagger)
                >= 2
        }
        CardId::Catalyst => {
            card_count(decision, CardId::NoxiousFumes)
                + card_count(decision, CardId::DeadlyPoison)
                + card_count(decision, CardId::PoisonedStab)
                >= 2
        }
        CardId::Reflex | CardId::Tactician => {
            card_count(decision, CardId::Survivor) + card_count(decision, CardId::DaggerThrow) >= 3
        }
        _ => true,
    }
}

fn card_count(decision: &PublicDecision, card: CardId) -> usize {
    decision.objects.iter().filter(|object| object.id == card).count()
}

fn defense_is_short(decision: &PublicDecision) -> bool {
    let defense = decision
        .objects
        .iter()
        .filter(|object| {
            matches!(
                object.id,
                CardId::DefendGreen
                    | CardId::Survivor
                    | CardId::Backflip
                    | CardId::CloakAndDagger
                    | CardId::Deflect
            )
        })
        .count();
    defense.saturating_mul(3) < decision.objects.len()
}

fn offered_choice(
    decision: &PublicDecision,
    wanted: impl Fn(&RoomChoiceKind) -> bool,
    require_price: bool,
) -> Option<GameAction> {
    let room = decision.room.as_ref()?;
    room.choices
        .iter()
        .filter(|choice| choice.status == RoomChoiceStatus::Offered && wanted(&choice.kind))
        .filter(|choice| {
            !require_price || choice.price.is_none_or(|price| price <= decision.player.gold)
        })
        .filter_map(|choice| {
            let action = GameAction::Choose(choice.id);
            decision
                .candidates
                .iter()
                .any(|candidate| candidate.action == action)
                .then_some((choice.id, action))
        })
        .min_by_key(|(id, _)| *id)
        .map(|(_, action)| action)
}

fn legal_selection_cards(decision: &PublicDecision) -> Vec<(PublicObject, u64, GameAction)> {
    let Some(selection) = decision.selection.as_ref() else {
        return Vec::new();
    };
    decision
        .candidates
        .iter()
        .filter_map(|candidate| {
            let GameAction::Select(_) = candidate.action else { return None };
            let card = candidate.object.as_ref()?;
            let eligible = selection.eligible.iter().any(|option| option == card);
            let in_deck = decision.objects.iter().any(|object| object == card);
            (eligible && in_deck).then(|| (card.clone(), card.instance, candidate.action.clone()))
        })
        .collect()
}

fn is_removal_selection(decision: &PublicDecision) -> bool {
    let Some(room) = decision.room.as_ref() else { return false };
    match room.kind {
        RoomViewKind::BossRelicChild => room.choices.iter().any(|choice| {
            choice.status == RoomChoiceStatus::Offered
                && choice.kind == RoomChoiceKind::Relic(RelicId::EmptyCage)
        }),
        RoomViewKind::Shop => room.choices.iter().any(|choice| {
            choice.status == RoomChoiceStatus::Offered && choice.kind == RoomChoiceKind::Purge
        }),
        RoomViewKind::Event(EventId::Purifier) => room.choices.iter().any(|choice| {
            choice.status == RoomChoiceStatus::Offered
                && choice.kind == RoomChoiceKind::ShrineRemove
        }),
        RoomViewKind::Event(EventId::AncientWriting) => room.choices.iter().any(|choice| {
            choice.status == RoomChoiceStatus::Offered
                && choice.kind == RoomChoiceKind::AncientElegance
        }),
        _ => false,
    }
}

fn is_upgrade_selection(decision: &PublicDecision) -> bool {
    let Some(room) = decision.room.as_ref() else { return false };
    match room.kind {
        RoomViewKind::Campfire => room.choices.iter().any(|choice| {
            choice.status == RoomChoiceStatus::Offered && choice.kind == RoomChoiceKind::Smith
        }),
        RoomViewKind::Event(EventId::UpgradeShrine) => room.choices.iter().any(|choice| {
            choice.status == RoomChoiceStatus::Offered
                && choice.kind == RoomChoiceKind::ShrineUpgrade
        }),
        _ => false,
    }
}

fn choose_upgrade(
    candidates: &[(PublicObject, u64, GameAction)],
) -> Option<(PublicObject, u64, GameAction)> {
    candidates.iter().min_by_key(|(card, instance, _)| {
        if let Some(rank) = UPGRADE_ORDER.iter().position(|wanted| *wanted == card.id) {
            (0_u8, rank, 0_i32, *instance)
        } else {
            (1_u8, usize::MAX, -card_value(card.id), *instance)
        }
    }).cloned()
}

fn choose_removal(
    decision: &PublicDecision,
    candidates: &[(PublicObject, u64, GameAction)],
) -> Option<(PublicObject, u64, GameAction)> {
    let prior_removed = already_selected_instances(decision);
    let can_preserve = candidates.iter().any(|(card, _, _)| {
        removal_preserves_defense(decision, card, &prior_removed)
    });
    candidates
        .iter()
        .filter(|(card, _, _)| {
            !can_preserve || removal_preserves_defense(decision, card, &prior_removed)
        })
        .min_by_key(|(card, instance, _)| {
            (
                u8::from(card.id != CardId::StrikeGreen),
                if card.id == CardId::StrikeGreen { 0 } else { card_value(card.id) },
                *instance,
            )
        })
        .cloned()
}

fn already_selected_instances(decision: &PublicDecision) -> Vec<u64> {
    let Some(selection) = decision.selection.as_ref() else {
        return Vec::new();
    };
    if selection.selected_count == 0 {
        return Vec::new();
    }
    let still_eligible = selection
        .eligible
        .iter()
        .map(|card| card.instance)
        .collect::<Vec<_>>();
    decision
        .objects
        .iter()
        .filter(|card| !still_eligible.contains(&card.instance))
        .map(|card| card.instance)
        .take(selection.selected_count)
        .collect()
}

fn removal_preserves_defense(
    decision: &PublicDecision,
    removed: &PublicObject,
    prior_removed: &[u64],
) -> bool {
    let remaining = decision
        .objects
        .iter()
        .filter(|card| !prior_removed.contains(&card.instance))
        .collect::<Vec<_>>();
    let deck_after = remaining.len().saturating_sub(1);
    let defense_before = remaining
        .iter()
        .filter(|card| is_defense(card.id))
        .count();
    let defense_after = defense_before.saturating_sub(usize::from(is_defense(removed.id)));
    defense_after.saturating_mul(3) >= deck_after
}

fn is_defense(card: CardId) -> bool {
    matches!(
        card,
        CardId::DefendGreen
            | CardId::Survivor
            | CardId::Backflip
            | CardId::CloakAndDagger
            | CardId::Deflect
    )
}
