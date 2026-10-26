use super::super::*;
use super::full_runtime::entered_combat;
use crate::{combat::model::OwnedCard, content::CardId};
// The suite covers actual typed identities and runtime behavior.
const REFLEX: CardId = CardId::Reflex;
const TACTICIAN: CardId = CardId::Tactician;

fn fixture(reaction: CardId, upgrades: u32, draw_ids: &[CardId]) -> (Combat, u64, u64) {
    let mut combat = entered_combat(3, 3, |card| {
        if card.instance == CardInstanceId(3) {
            *card = card_state::permanent(CardId::DaggerThrow, card.instance, 0);
        }
    });
    combat.state.character = characters::CharacterState::for_character(crate::content::Character::Silent);
    combat.state.player.max_energy = 9;
    combat.state.player.energy = 9;

    let selected = combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
        .find(|card| card.instance != CardInstanceId(3)).expect("one non-parent hand card");
    selected.id = reaction;
    selected.upgrades = upgrades;
    let selected_id = selected.instance.0;
    let parent_id = 3;

    for (index, id) in draw_ids.iter().copied().enumerate() {
        let card = card_state::permanent(id, CardInstanceId(100 + index as u64), 0);
        let handle = card_state::handle(card.instance);
        assert!(combat.state.card_ids_issued.insert(handle), "fixture identity is unique");
        combat.state.piles.draw.push(OwnedCard::Full(card));
    }
    (combat, parent_id, selected_id)
}

fn play_dagger_throw(combat: &mut Combat, instance: u64) {
    combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.phase, Phase::Selection);
}

fn choose(combat: &mut Combat, instance: u64) {
    combat.step_full(&CombatAction::SelectCard { instance }).unwrap();
}

fn draw_ids(id: CardId, count: usize) -> Vec<CardId> {
    vec![id; count]
}

#[test]
fn m11_silent_discard_card_definitions_are_real_unplayable_silent_skills() {
    use crate::content::{CardColor, CardRarity, CardType, Catalog};

    for id in [REFLEX, TACTICIAN] {
        let definition = Catalog::standard().card(id).expect("registered Silent reaction card");
        assert_eq!(definition.color, CardColor::Green);
        assert_eq!(definition.rarity, CardRarity::Uncommon);
        assert_eq!(definition.card_type, CardType::Skill);
        assert!(!definition.definition_only, "the discard listener has executable behavior");
    }
}

#[test]
fn m11_silent_discard_triggers_follow_the_committed_owner() {
    for (upgrades, expected) in [(0, 2), (1, 3)] {
        let ids = draw_ids(CardId::Strike, expected + 2);
        let (mut combat, dagger, reflex) = fixture(REFLEX, upgrades, &ids);
        let first_reaction_draw = CardInstanceId(100 + ids.len() as u64 - 1);
        let hp_before = combat.state.enemies[0].hp;

        play_dagger_throw(&mut combat, dagger);
        assert_eq!(combat.state.enemies[0].hp, hp_before - 9, "Dagger Throw damage resolves before selection");
        assert_eq!(combat.state.player.energy, 8, "the parent action pays once");
        let selection = combat.state.pending_selection.as_ref().unwrap();
        assert!(selection.candidates.contains(&card_state::handle(CardInstanceId(reflex))));
        assert!(selection.candidates.contains(&card_state::handle(first_reaction_draw)),
            "Dagger Throw captures its own drawn card before the decision");

        choose(&mut combat, reflex);
        assert_eq!(combat.state.enemies[0].hp, hp_before - 9, "resuming the discard does not repeat damage");
        assert_eq!(combat.state.player.energy, 8, "resuming the discard does not repay Dagger Throw");
        assert_eq!(combat.state.piles.hand.len(), 2 + expected,
            "only the selected Reflex draws its base or upgraded amount");
        assert_eq!(combat.state.piles.draw.len(), 1, "one initial draw plus only the Reflex draw resolve");
        assert_eq!(combat.state.piles.discard.iter().filter(|card| card.handle().as_str() == reflex.to_string()).count(), 1);
        assert_eq!(combat.state.piles.discard.iter().filter(|card| card.handle().as_str() == dagger.to_string()).count(), 1);
        let selected_source = Source::Card(card_state::handle(CardInstanceId(reflex)));
        let parent_source = Source::Card(card_state::handle(CardInstanceId(dagger)));
        let events = combat.state.hook_events.iter().enumerate().filter(|(_, event)| event.hook == Hook::CardDiscarded)
            .collect::<Vec<_>>();
        let selected_event = events.iter().find(|(_, event)| event.source == selected_source).expect("manual discard notification");
        let parent_event = events.iter().find(|(_, event)| event.source == parent_source).expect("parent natural-finish notification");
        assert_eq!(events.iter().filter(|(_, event)| event.source == selected_source).count(), 1,
            "the committed selected instance notifies exactly once");
        assert!(selected_event.0 < parent_event.0,
            "selected discard notification occurs before the played parent naturally finishes");
        assert_eq!(combat.state.fault_selection, None, "successful resolution releases the retained selection");
    }
}

#[test]
fn m11_silent_tactician_discard_triggers_follow_the_committed_owner() {
    for (upgrades, expected_gain) in [(0, 1), (1, 2)] {
        let (mut combat, dagger, tactician) = fixture(TACTICIAN, upgrades, &[]);
        play_dagger_throw(&mut combat, dagger);
        assert_eq!(combat.state.player.energy, 8);
        choose(&mut combat, tactician);
        assert_eq!(combat.state.player.energy, 8 + expected_gain,
            "Tactician's energy reaction is applied once to the selected instance");
        assert_eq!(combat.state.piles.discard.iter().filter(|card|
            card.handle().as_str() == tactician.to_string()).count(), 1);
        assert_eq!(combat.state.piles.discard.iter().filter(|card|
            card.handle().as_str() == dagger.to_string()).count(), 1);
    }
}

#[test]
fn m11_silent_dagger_throw_with_empty_post_draw_hand_finishes_once() {
    let mut combat = entered_combat(3, 3, |card| {
        if card.instance == CardInstanceId(3) {
            *card = card_state::permanent(CardId::DaggerThrow, card.instance, 0);
        }
    });
    combat.state.character = characters::CharacterState::for_character(crate::content::Character::Silent);
    let parent = card_state::handle(CardInstanceId(3));
    let mut only_parent = Vec::new();
    while let Some(card) = combat.state.piles.hand.pop() {
        if card.handle() == parent {
            only_parent.push(card);
        } else {
            combat.state.piles.exhaust.push(card);
        }
    }
    combat.state.piles.hand = only_parent;

    combat.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.phase, Phase::Decision, "an empty post-draw selection continues automatically");
    assert!(combat.state.pending_selection.is_none());
    assert_eq!(combat.state.player.energy, 2);
    assert_eq!(combat.state.enemies[0].hp, 39);
    assert_eq!(combat.state.piles.discard.iter().filter(|card| card.handle() == parent).count(), 1);
}

#[test]
fn m11_silent_discard_reaction_draws_do_not_restart_the_original_selection() {
    let ids = vec![CardId::Strike, REFLEX, REFLEX, REFLEX];
    let (mut combat, dagger, selected) = fixture(REFLEX, 0, &ids);
    play_dagger_throw(&mut combat, dagger);
    let drawn_reflex = CardInstanceId(100 + ids.len() as u64 - 1);
    assert!(combat.state.pending_selection.as_ref().unwrap().candidates.contains(&card_state::handle(drawn_reflex)));

    choose(&mut combat, selected);
    assert_eq!(combat.state.phase, Phase::Decision, "reaction draws do not open a second discard selection");
    assert_eq!(combat.state.piles.hand.iter().filter(|card|
        card.full().is_some_and(|live| live.id == REFLEX)).count(), 3,
        "the unselected Reflexes drawn by Dagger Throw and the selected reaction remain in hand");
    assert_eq!(combat.state.piles.discard.iter().filter(|card|
        card.full().is_some_and(|live| live.id == REFLEX)).count(), 1,
        "only the exact selected Reflex moves to discard");
}

#[test]
fn m11_silent_reaction_cards_are_not_ordinary_legal_plays() {
    for id in [REFLEX, TACTICIAN] {
        let mut combat = entered_combat(3, 3, |card| {
            if card.instance == CardInstanceId(3) { card.id = id; }
        });
        combat.state.character = characters::CharacterState::for_character(crate::content::Character::Silent);
        let instance = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card| card.id == id).unwrap().instance.0;
        let menu = combat.legal_actions_full().expect("unplayable reaction cards are supported menu entries");
        assert!(!menu.contains(&CombatAction::PlayCard { instance, target: None }));
        assert!(!menu.contains(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }));
        let before = combat.state.clone();
        assert_eq!(combat.step_full(&CombatAction::PlayCard { instance, target: None }),
            Err(crate::run::RunError::InvalidAction));
        assert_eq!(combat.state, before, "unplayable cards reject a direct ordinary play atomically");
    }
}

#[test]
fn m11_silent_distinct_reflex_instances_and_stale_choices_keep_exact_ownership() {
    let (mut combat, dagger, first) = fixture(REFLEX, 0, &draw_ids(CardId::Strike, 4));
    let second = combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
        .find(|card| card.instance != CardInstanceId(dagger) && card.instance != CardInstanceId(first)).unwrap();
    second.id = REFLEX;
    let second_id = second.instance.0;
    play_dagger_throw(&mut combat, dagger);
    let actions = combat.legal_actions_full().unwrap();
    assert!(actions.contains(&CombatAction::SelectCard { instance: first }));
    assert!(actions.contains(&CombatAction::SelectCard { instance: second_id }), "same-kind copies retain distinct handles");

    let before_stale = combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance: 9999 }), Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state, before_stale, "stale choice leaves the paid parent and captured candidates untouched");

    choose(&mut combat, first);
    assert_eq!(combat.state.piles.discard.iter().filter(|card| card.handle().as_str() == first.to_string()).count(), 1);
    assert_eq!(combat.state.piles.hand.iter().filter(|card| card.handle().as_str() == second_id.to_string()).count(), 1);
    assert_eq!(combat.state.piles.discard.iter().filter(|card| card.handle().as_str() == second_id.to_string()).count(), 0);
    let after_commit = combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance: first }), Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state, after_commit, "a completed choice cannot be delivered a second time");
}

#[test]
fn m11_silent_survivor_blocks_before_its_selected_discard() {
    for (upgrades, block) in [(0, 8), (1, 11)] {
        let mut combat = entered_combat(3, 3, |card| {
            if card.instance == CardInstanceId(3) { *card = card_state::permanent(CardId::Survivor, card.instance, upgrades); }
        });
        combat.state.character = characters::CharacterState::for_character(crate::content::Character::Silent);
        let selected = combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
            .find(|card| card.instance != CardInstanceId(3)).unwrap();
        selected.id = REFLEX;
        let selected_id = selected.instance.0;
        for index in 0..4 {
            let card = card_state::permanent(CardId::Strike, CardInstanceId(100 + index), 0);
            combat.state.card_ids_issued.insert(card_state::handle(card.instance));
            combat.state.piles.draw.push(OwnedCard::Full(card));
        }
        combat.step_full(&CombatAction::PlayCard { instance: 3, target: None }).unwrap();
        assert_eq!(combat.state.player.block, block, "Survivor grants Block before its choice opens");
        assert_eq!(combat.state.phase, Phase::Selection);
        choose(&mut combat, selected_id);
        assert_eq!(combat.state.player.block, block);
        assert!(combat.state.piles.discard.iter().any(|card| card.handle().as_str() == selected_id.to_string()));
    }
}

#[test]
fn m11_silent_end_turn_cleanup_does_not_fire_manual_discard_reactions() {
    for (reaction, ethereal, expected_zone) in [
        (REFLEX, false, SelectionDisposition::Discard),
        (TACTICIAN, false, SelectionDisposition::Discard),
        (REFLEX, true, SelectionDisposition::Exhaust),
    ] {
        let (mut combat, _, selected) = fixture(reaction, 0, &draw_ids(CardId::Strike, 4));
        let owner = card_state::handle(CardInstanceId(selected));
        if ethereal {
            combat.state.piles.hand.iter_mut().find(|card| card.handle() == owner).unwrap()
                .full_mut().unwrap().local.ethereal = true;
        }
        let energy_before = combat.state.player.energy;
        let draw_before = combat.state.piles.draw.len();
        piles::discard_card(&mut combat.state, &owner, true).unwrap();
        engine::resolve(&mut combat.state);
        assert!(combat.state.fault.is_none());
        assert_eq!(combat.state.player.energy, energy_before, "cleanup does not grant Tactician energy");
        assert_eq!(combat.state.piles.draw.len(), draw_before, "cleanup does not trigger Reflex draws");
        match expected_zone {
            SelectionDisposition::Discard => assert!(combat.state.piles.discard.iter().any(|card| card.handle() == owner)),
            SelectionDisposition::Exhaust => assert!(combat.state.piles.exhaust.iter().any(|card| card.handle() == owner)),
            _ => unreachable!(),
        }
    }

    let (mut combat, _, reflex) = fixture(REFLEX, 0, &draw_ids(CardId::Strike, 5));
    let other = combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
        .find(|card| card.instance != CardInstanceId(3) && card.instance != CardInstanceId(reflex)).unwrap();
    other.id = TACTICIAN;
    let reflex_handle = card_state::handle(CardInstanceId(reflex));
    let tactician_handle = card_state::handle(other.instance);
    combat.state.piles.hand.iter_mut().find(|card| card.handle() == reflex_handle).unwrap()
        .full_mut().unwrap().local.ethereal = true;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.turn, 2);
    assert_eq!(combat.state.phase, Phase::Decision);
    assert!(combat.state.piles.exhaust.iter().any(|card| card.handle() == reflex_handle),
        "Ethereal follows its actual end-turn exhaust path");
    assert!(combat.state.piles.discard.iter().any(|card| card.handle() == tactician_handle));
    assert_eq!(combat.state.piles.hand.len(), 5,
        "the ordinary next-turn draw is not enlarged by Reflex cleanup reactions");
}

#[test]
fn m11_silent_natural_finish_without_selected_movement_is_not_manual_discard() {
    let (mut combat, _, selected) = fixture(REFLEX, 0, &draw_ids(CardId::Strike, 3));
    let owner = card_state::handle(CardInstanceId(selected));
    let index = combat.state.piles.hand.iter().position(|card| card.handle() == owner).unwrap();
    combat.state.in_play = Some(combat.state.piles.hand.remove(index));
    combat.state.phase = Phase::PlayerAction;
    let energy_before = combat.state.player.energy;
    let draw_before = combat.state.piles.draw.len();

    piles::discard_card(&mut combat.state, &owner, false).unwrap();
    engine::resolve(&mut combat.state);
    assert!(combat.state.piles.discard.iter().any(|card| card.handle() == owner));
    assert_eq!(combat.state.player.energy, energy_before);
    assert_eq!(combat.state.piles.draw.len(), draw_before);
    assert!(combat.state.hook_events.iter().any(|event|
        event.hook == Hook::CardDiscarded && event.source == Source::Card(owner.clone())));
}

#[test]
fn m11_silent_exhaust_selection_does_not_fire_manual_discard_reactions() {
    let (mut combat, dagger, reflex) = fixture(REFLEX, 0, &draw_ids(CardId::Strike, 3));
    let parent = card_state::handle(CardInstanceId(dagger));
    let selected = card_state::handle(CardInstanceId(reflex));
    let parent_index = combat.state.piles.hand.iter().position(|card| card.handle() == parent).unwrap();
    combat.state.in_play = Some(combat.state.piles.hand.remove(parent_index));
    combat.state.phase = Phase::Selection;
    combat.state.pending_selection = Some(PendingSelection {
        parent: SelectionParent::Card(parent.clone()),
        source_zone: SelectionSourceZone::Hand,
        candidates: vec![selected.clone()],
        operation: SelectionOperation::Move(SelectionDisposition::Exhaust),
        selected: Vec::new(), minimum: 1, maximum: 1,
        disposition: SelectionDisposition::Exhaust, empty: EmptySelection::Continue,
    });
    combat.state.pending.extend([
        Effect { source: Source::Card(parent.clone()), target: None, kind: EffectKind::Hook(Hook::CardAfterUse) },
        Effect { source: Source::Card(parent.clone()), target: None, kind: EffectKind::FinishCard },
        Effect { source: Source::None, target: None, kind: EffectKind::Decision },
    ]);
    let energy_before = combat.state.player.energy;
    let draw_before = combat.state.piles.draw.len();

    choose(&mut combat, reflex);
    assert!(combat.state.piles.exhaust.iter().any(|card| card.handle() == selected));
    assert_eq!(combat.state.player.energy, energy_before);
    assert_eq!(combat.state.piles.draw.len(), draw_before);
    assert_eq!(combat.state.phase, Phase::Decision);
}

#[test]
fn m11_silent_tactician_top_callback_commits_before_parent_and_retains_fault() {
    let (mut combat, dagger, tactician) = fixture(TACTICIAN, 0, &[]);
    play_dagger_throw(&mut combat, dagger);
    combat.state.resolved_effects = 99_997;
    let action = CombatAction::SelectCard { instance: tactician };
    let result = combat.step_full(&action);
    let crate::run::RunError::Faulted(first) = result.unwrap_err() else { panic!("expected retained callback fault") };
    let crate::run::RunFaultKind::Combat(native) = &first.kind else { panic!("expected combat fault packet") };
    assert_eq!(native.record.kind, FaultKind::EffectGuard);
    assert_eq!(native.current.as_ref().unwrap().kind, EffectKind::Hook(Hook::CardAfterUse));
    assert!(native.pending.iter().any(|effect| effect.kind == EffectKind::FinishCard),
        "the parent's existing finish remains queued after top work");
    assert_eq!(native.attempted_action, Some(action));
    assert_eq!(combat.state.player.energy, 9, "Tactician's top-queued gain committed before the parent continued");
    assert!(combat.state.piles.discard.iter().any(|card| card.handle().as_str() == tactician.to_string()));
    assert_eq!(combat.state.piles.discard.iter().filter(|card| card.handle().as_str() == dagger.to_string()).count(), 0);
    assert_eq!(combat.state.fault_selection.as_ref().unwrap().selected,
        vec![card_state::handle(CardInstanceId(tactician))]);
    let retained = combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::EndTurn), Err(crate::run::RunError::Faulted(first)));
    assert_eq!(combat.state, retained, "the top callback and accepted choice are not redelivered");
}

#[test]
fn m11_silent_discard_callback_fault_retains_committed_draw_and_first_fault() {
    let ids = draw_ids(CardId::Strike, 5);
    let (mut combat, dagger, reflex) = fixture(REFLEX, 0, &ids);
    play_dagger_throw(&mut combat, dagger);
    combat.state.resolved_effects = 99_993;
    let action = CombatAction::SelectCard { instance: reflex };
    let result = combat.step_full(&action);
    let crate::run::RunError::Faulted(first) = result.unwrap_err() else { panic!("expected retained callback fault") };
    let crate::run::RunFaultKind::Combat(native) = &first.kind else { panic!("expected combat fault packet") };
    assert_eq!(native.record.kind, FaultKind::EffectGuard);
    assert_eq!(native.current.as_ref().unwrap().kind, EffectKind::AfterCardDrawn { card: card_state::handle(CardInstanceId(103)) });
    assert!(native.pending.iter().any(|effect| effect.kind == EffectKind::Draw(1)),
        "the remaining Reflex child stays queued after its first draw commits");
    assert_eq!(native.attempted_action, Some(action));
    assert_eq!(native.selection.as_ref().unwrap().selected, vec![card_state::handle(CardInstanceId(reflex))]);
    assert_eq!(combat.state.fault_selection.as_ref().unwrap().selected, vec![card_state::handle(CardInstanceId(reflex))]);
    assert!(combat.state.piles.discard.iter().any(|card| card.handle().as_str() == reflex.to_string()));
    assert!(combat.state.piles.discard.iter().any(|card| card.handle().as_str() == dagger.to_string()),
        "the parent finish committed before the callback fault");
    assert!(combat.state.piles.hand.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(103))),
        "the first reaction draw remains committed");
    let retained = combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::EndTurn), Err(crate::run::RunError::Faulted(first)));
    assert_eq!(combat.state, retained, "faulted callback work and accepted selection are not redelivered");
}

#[test]
fn lethal_dagger_throw_suppresses_followup_selector_and_manual_discard() {
    let mut combat = entered_combat(3, 3, |_| {});
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.state.piles.hand[0].full_mut().unwrap().id = CardId::DaggerThrow;
    combat.state.enemies[0].hp = 1;
    combat.state.enemies[0].block = 0;

    combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();

    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert!(combat.state.pending_selection.is_none());
    assert!(!combat.state.pending.iter().any(|effect| matches!(effect.kind,
        EffectKind::SelectHandCardsAfterDraw { .. } | EffectKind::SelectCards { .. })));
}
