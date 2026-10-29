//! Supplied public-state checks for the M14 combat-selection caller.
use super::*;
use spire_core::{
    combat::{
        enemies, CardHandle, CardInstanceId, Combat, LiveCardView, PlayerState,
        SelectionParent, SelectionSourceZone, SelectionView,
    },
    items::CombatItemContext,
    run::{Candidate, ChoiceId, GameAction, RoomScenario},
};

fn supplied(
    cards: &[(u64, CardId, u32)],
    energy: i32,
    max_energy: i32,
    incoming: i32,
) -> PublicDecision {
    let mut setup_rng = RunRng::new([42; 32], Partition::Test, 0);
    let entry = enemies::setup(
        EncounterId::Slavers,
        2,
        0,
        &mut setup_rng,
        enemies::SetupContext {
            player: PlayerState {
                hp: 70,
                max_hp: 70,
                block: 0,
                energy,
                max_energy,
                draw_per_turn: 5,
                powers: vec![],
            },
            player_powers: vec![],
            character: Character::Silent,
            cards: cards
                .iter()
                .map(|(instance, id, upgrades)| {
                    spire_core::combat::permanent_live_card(
                        *id,
                        CardInstanceId(*instance),
                        *upgrades,
                    )
                })
                .collect(),
            opening_draw: cards.len(),
            encounter_visit: 0,
        },
    )
    .unwrap();
    let mut items = Some(CombatItemContext::new(vec![], vec![]));
    let combat = Combat::enter_with_item_context(entry, &mut items).unwrap();
    let legal = combat.legal_actions_full().unwrap();
    let mut public = combat.public_full().unwrap();
    for (index, enemy) in public.enemies.iter_mut().enumerate() {
        enemy.intent_damage = Some(if index == 0 { incoming } else { 0 });
    }

    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    let mut decision = Run::new_room_scenario(
        RunConfig {
            character: Character::Silent,
            ascension: 0,
        },
        profile,
        RunRng::new([42; 32], Partition::Test, 0),
        RoomScenario::M14SilentAct3,
    )
    .unwrap()
    .decision()
    .unwrap();
    decision.candidates = legal
        .into_iter()
        .enumerate()
        .map(|(index, action)| Candidate {
            id: ChoiceId(index as u64 + 1),
            action: GameAction::Combat(action),
            label: "supplied legal combat action",
            neow_offer: None,
            object: None,
        })
        .collect();
    decision.combat = Some(public);
    decision
}

fn supplied_selection(
    cards: &[(u64, CardId, u32)],
    parent_id: Option<u64>,
    operation: SelectionOperation,
    disposition: SelectionDisposition,
    source_zone: SelectionSourceZone,
    candidate_ids: &[u64],
    selected_ids: &[u64],
    minimum: usize,
    maximum: usize,
    energy: i32,
    max_energy: i32,
    incoming: i32,
) -> PublicDecision {
    let mut decision = supplied(cards, energy, max_energy, incoming);
    let combat = decision.combat.as_mut().unwrap();
    let parent = if operation == SelectionOperation::Retain {
        SelectionParent::EndTurn { turn: combat.turn }
    } else {
        let instance = parent_id.expect("card selection has a visible owner");
        let position = combat
            .hand
            .iter()
            .position(|card| card.instance.0 == instance);
        let owner = if let Some(position) = position {
            combat.hand.remove(position)
        } else {
            // A supplied selection may already be past the parent's play and
            // draw checkpoint, with a full ten-card hand and the owner in play.
            let live = spire_core::combat::permanent_live_card(
                CardId::DaggerThrow,
                CardInstanceId(instance),
                0,
            );
            LiveCardView { misc:live.misc, watcher:None,
                instance: live.instance,
                id: live.id,
                origin: live.origin,
                upgrades: live.upgrades,
                effective_cost: Some(1),
                values: None,
            }
        };
        combat.playing_cards.push(owner);
        SelectionParent::Card(CardHandle::parse(&instance.to_string()).unwrap())
    };

    let mut candidates = Vec::with_capacity(candidate_ids.len());
    for instance in candidate_ids {
        let position = combat
            .hand
            .iter()
            .position(|card| card.instance.0 == *instance)
            .expect("selection source is in the supplied hand");
        if source_zone == SelectionSourceZone::Exhaust {
            combat.exhaust.push(combat.hand.remove(position));
            candidates.push(combat.exhaust.last().unwrap().clone());
        } else {
            candidates.push(combat.hand[position].clone());
        }
    }

    combat.selection = Some(SelectionView {
        parent,
        operation,
        candidates,
        required: minimum,
        selected: selected_ids
            .iter()
            .copied()
            .map(CardInstanceId)
            .collect(),
        disposition,
        source_zone,
        minimum,
        maximum,
    });
    let mut legal = Vec::new();
    if selected_ids.len() < maximum {
        legal.extend(
            candidate_ids
                .iter()
                .filter(|instance| !selected_ids.contains(instance))
                .map(|instance| CombatAction::SelectCard { instance: *instance }),
        );
    }
    if minimum < maximum && selected_ids.len() >= minimum {
        legal.push(CombatAction::ConfirmSelection);
    }
    decision.candidates = legal
        .into_iter()
        .enumerate()
        .map(|(index, action)| Candidate {
            id: ChoiceId(index as u64 + 100),
            action: GameAction::Combat(action),
            label: "supplied legal selection action",
            neow_offer: None,
            object: None,
        })
        .collect();
    decision
}

fn choose_m14(decision: &PublicDecision) -> GameAction {
    action_for(
        decision,
        None,
        &Caller::new_m14(M14_ACTION_LIMIT_MAX).unwrap(),
    )
    .unwrap()
}

fn choose_m13(decision: &PublicDecision) -> GameAction {
    action_for(decision, None, &Caller::new(ACTION_LIMIT_MAX).unwrap()).unwrap()
}

fn discard_fixture(
    discarded_candidate: CardId,
    energy: i32,
    max_energy: i32,
    draw_count: usize,
    filler_count: usize,
) -> PublicDecision {
    let mut cards = vec![(2, discarded_candidate, 0), (3, CardId::DefendGreen, 0)];
    cards.extend((0..filler_count).map(|offset| {
        (4 + offset as u64, CardId::StrikeGreen, 0)
    }));
    let candidate_ids = cards
        .iter()
        .map(|(instance, _, _)| *instance)
        .collect::<Vec<_>>();
    let mut decision = supplied_selection(
        &cards,
        Some(1),
        SelectionOperation::Move(SelectionDisposition::Discard),
        SelectionDisposition::Discard,
        SelectionSourceZone::Hand,
        &candidate_ids,
        &[],
        1,
        1,
        energy,
        max_energy,
        0,
    );
    decision.combat.as_mut().unwrap().draw_count = draw_count;
    decision
}

#[test]
fn m14_revision_selection_discard_protects_affordable_defense_against_strike_under_pressure() {
    let decision = supplied_selection(
        &[
            (1, CardId::DaggerThrow, 0),
            (2, CardId::DefendGreen, 0),
            (3, CardId::StrikeGreen, 0),
        ],
        Some(1),
        SelectionOperation::Move(SelectionDisposition::Discard),
        SelectionDisposition::Discard,
        SelectionSourceZone::Hand,
        &[2, 3],
        &[],
        1,
        1,
        3,
        3,
        20,
    );
    assert_eq!(
        choose_m14(&decision),
        GameAction::Combat(CombatAction::SelectCard { instance: 3 })
    );
}

#[test]
fn m14_revision_selection_discard_uses_printed_reflex_and_tactician_benefits() {
    let cases = [
        // Reflex draws when a card remains in the public draw pile. Discarding
        // Reflex frees a hand slot before its draw, including at a full hand.
        (CardId::Reflex, 3, 3, 2, 0, 2),
        (CardId::Reflex, 3, 3, 0, 0, 3),
        (CardId::Reflex, 3, 3, 2, 8, 2),
        // Tactician grants energy even when current energy is at its maximum.
        (CardId::Tactician, 2, 3, 0, 0, 2),
        (CardId::Tactician, 3, 3, 0, 0, 2),
    ];
    for (card, energy, max_energy, draw_count, fillers, expected) in cases {
        let decision = discard_fixture(card, energy, max_energy, draw_count, fillers);
        assert_eq!(
            choose_m14(&decision),
            GameAction::Combat(CombatAction::SelectCard { instance: expected }),
            "discard trigger for {card:?}, draw {draw_count}, hand {} and energy {energy}/{max_energy}",
            decision.combat.as_ref().unwrap().hand.len(),
        );
    }
}

#[test]
fn m14_revision_selection_retain_selects_useful_block_then_confirms_at_limit() {
    let cards = [(1, CardId::DefendGreen, 0), (2, CardId::StrikeGreen, 0)];
    let first = supplied_selection(
        &cards,
        None,
        SelectionOperation::Retain,
        SelectionDisposition::Hand,
        SelectionSourceZone::Hand,
        &[1, 2],
        &[],
        0,
        1,
        3,
        3,
        0,
    );
    assert_eq!(
        choose_m14(&first),
        GameAction::Combat(CombatAction::SelectCard { instance: 1 })
    );

    let at_limit = supplied_selection(
        &cards,
        None,
        SelectionOperation::Retain,
        SelectionDisposition::Hand,
        SelectionSourceZone::Hand,
        &[1, 2],
        &[1],
        0,
        1,
        3,
        3,
        0,
    );
    assert_eq!(
        choose_m14(&at_limit),
        GameAction::Combat(CombatAction::ConfirmSelection)
    );
}

#[test]
fn m14_revision_selection_retain_confirms_when_only_status_cards_are_offered() {
    let decision = supplied_selection(
        &[(1, CardId::Dazed, 0), (2, CardId::Wound, 0)],
        None,
        SelectionOperation::Retain,
        SelectionDisposition::Hand,
        SelectionSourceZone::Hand,
        &[1, 2],
        &[],
        0,
        2,
        3,
        3,
        0,
    );
    assert_eq!(
        choose_m14(&decision),
        GameAction::Combat(CombatAction::ConfirmSelection)
    );
}

#[test]
fn m14_revision_selection_copy_and_return_use_defense_under_pressure_and_value_otherwise() {
    let operations = [
        (
            CardId::Nightmare,
            SelectionOperation::CopyToHand { copies: 2 },
            SelectionDisposition::Hand,
            SelectionSourceZone::Hand,
        ),
        (
            CardId::Nightmare,
            SelectionOperation::NightmareCapture,
            SelectionDisposition::Hand,
            SelectionSourceZone::Hand,
        ),
        (
            CardId::Exhume,
            SelectionOperation::ReturnFromExhaust,
            SelectionDisposition::Hand,
            SelectionSourceZone::Exhaust,
        ),
        (
            CardId::Warcry,
            SelectionOperation::Topdeck,
            SelectionDisposition::DrawTop,
            SelectionSourceZone::Hand,
        ),
    ];
    for (parent, operation, disposition, source_zone) in operations {
        for (incoming, expected) in [(20, 2), (0, 3)] {
            let decision = supplied_selection(
                &[
                    (1, parent, 0),
                    (2, CardId::DefendGreen, 0),
                    (3, CardId::NoxiousFumes, 0),
                ],
                Some(1),
                operation.clone(),
                disposition,
                source_zone,
                &[2, 3],
                &[],
                1,
                1,
                3,
                3,
                incoming,
            );
            assert_eq!(
                choose_m14(&decision),
                GameAction::Combat(CombatAction::SelectCard { instance: expected }),
                "operation {operation:?} with incoming {incoming}",
            );
        }
    }
}

#[test]
fn m14_revision_selection_optional_topdeck_skips_unplayable_but_mandatory_choice_uses_legal_fallback() {
    let optional = supplied_selection(
        &[(1, CardId::Warcry, 0), (2, CardId::Dazed, 0)],
        Some(1),
        SelectionOperation::Topdeck,
        SelectionDisposition::DrawTop,
        SelectionSourceZone::Hand,
        &[2],
        &[],
        0,
        1,
        3,
        3,
        0,
    );
    assert_eq!(
        choose_m14(&optional),
        GameAction::Combat(CombatAction::ConfirmSelection)
    );

    let mandatory = supplied_selection(
        &[(1, CardId::Warcry, 0), (2, CardId::Dazed, 0)],
        Some(1),
        SelectionOperation::Topdeck,
        SelectionDisposition::DrawTop,
        SelectionSourceZone::Hand,
        &[2],
        &[],
        1,
        1,
        3,
        3,
        0,
    );
    assert_eq!(
        choose_m14(&mandatory),
        GameAction::Combat(CombatAction::SelectCard { instance: 2 })
    );
}

#[test]
fn m14_revision_selection_respects_required_selected_and_legal_action_limits() {
    let mut decision = supplied_selection(
        &[
            (1, CardId::DaggerThrow, 0),
            (2, CardId::StrikeGreen, 0),
            (3, CardId::DefendGreen, 0),
            (4, CardId::Dazed, 0),
        ],
        Some(1),
        SelectionOperation::Move(SelectionDisposition::Discard),
        SelectionDisposition::Discard,
        SelectionSourceZone::Hand,
        &[2, 3, 4],
        &[2],
        2,
        3,
        3,
        3,
        20,
    );
    // The selected Strike cannot be repeated, and the unoffered Dazed cannot
    // be chosen even though the supplied public card remains a candidate.
    decision.candidates.retain(|candidate| {
        candidate.action != GameAction::Combat(CombatAction::SelectCard { instance: 4 })
    });
    assert_eq!(
        choose_m14(&decision),
        GameAction::Combat(CombatAction::SelectCard { instance: 3 })
    );
}

#[test]
fn m14_revision_selection_m13_keeps_confirm_retain_and_low_value_discard_rules() {
    let retain = supplied_selection(
        &[(1, CardId::DefendGreen, 0), (2, CardId::StrikeGreen, 0)],
        None,
        SelectionOperation::Retain,
        SelectionDisposition::Hand,
        SelectionSourceZone::Hand,
        &[1, 2],
        &[],
        0,
        1,
        3,
        3,
        0,
    );
    assert_eq!(
        choose_m13(&retain),
        GameAction::Combat(CombatAction::ConfirmSelection)
    );

    let discard = supplied_selection(
        &[
            (1, CardId::DaggerThrow, 0),
            (2, CardId::DefendGreen, 0),
            (3, CardId::StrikeGreen, 0),
        ],
        Some(1),
        SelectionOperation::Move(SelectionDisposition::Discard),
        SelectionDisposition::Discard,
        SelectionSourceZone::Hand,
        &[2, 3],
        &[],
        1,
        1,
        3,
        3,
        20,
    );
    assert_eq!(
        choose_m13(&discard),
        GameAction::Combat(CombatAction::SelectCard { instance: 2 })
    );
}
