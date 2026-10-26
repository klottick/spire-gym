use super::super::*;
use super::full_runtime::entered_combat;
use super::super::model::OwnedCard;
use crate::content::{CardColor,CardId,CardRarity,CardType,EncounterId,NoteStorage};

fn silent_combat() -> Combat {
    let mut combat = entered_combat(3, 3, |_| {});
    for instance in 4..=18 {
        let card = OwnedCard::Full(card_state::permanent(CardId::Defend,CardInstanceId(instance),0));
        combat.state.card_ids_issued.insert(card.handle());
        combat.state.piles.draw.push(card);
    }
    combat.state.character = characters::CharacterState::for_character(crate::content::Character::Silent);
    combat
}

fn combat_with_retain_power(amount: i32) -> Combat {
    let mut combat = silent_combat();
    combat.state.player_full_powers.push(FullPowerEntry {
        id: FullPowerId("retain_cards"),
        state: PowerState { amount, skip_first: false, monster_source_grace: false },
    });
    combat
}

fn open_selection(combat: &mut Combat) -> Vec<CardHandle> {
    let hand = combat.state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>();
    assert!(!hand.is_empty());
    assert_eq!(combat.step_full(&CombatAction::EndTurn), Ok(()));
    let selection = combat.state.pending_selection.as_ref().expect("retain power opens a selection");
    assert_eq!(selection.parent,SelectionParent::EndTurn{turn:combat.state.turn});
    assert_eq!(selection.operation,SelectionOperation::Retain);
    assert_eq!(selection.source_zone, SelectionSourceZone::Hand);
    assert_eq!(selection.minimum, 0);
    assert_eq!(selection.candidates, hand);
    assert!(engine::validate_runtime(&combat.state).is_ok(),"the typed EndTurn parent validates while selection is suspended");
    hand
}

fn select_and_confirm(combat: &mut Combat, handle: &CardHandle) {
    let instance = handle.as_str().parse().unwrap();
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance }), Ok(()));
    assert_eq!(combat.step_full(&CombatAction::ConfirmSelection), Ok(()));
}

fn play_well_laid_plans(combat:&mut Combat,upgrades:u32)->u64{
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    combat.state.piles.hand[0]=OwnedCard::Full(card_state::permanent(CardId::WellLaidPlans,instance,upgrades));
    assert_eq!(combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}),Ok(()));
    instance.0
}

#[test]
fn m11_silent_retain_preserves_selected_instances_at_end_turn() {
    let mut one = combat_with_retain_power(1);
    let hand = open_selection(&mut one);
    assert_eq!(one.state.pending_selection.as_ref().unwrap().maximum, 1);
    let actions = one.legal_actions_full().unwrap();
    assert!(actions.contains(&CombatAction::ConfirmSelection), "a live optional selector permits Confirm0");
    for card in &hand { assert!(actions.contains(&CombatAction::SelectCard { instance: card.as_str().parse().unwrap() })); }
    assert_eq!(one.state.phase, Phase::Selection,
        "Well-Laid Plans opens its optional selection in the real pre-discard end-turn checkpoint");

    let mut two = combat_with_retain_power(2);
    open_selection(&mut two);
    assert_eq!(two.state.pending_selection.as_ref().unwrap().maximum, 2,
        "the selected-card maximum is bounded by the current hand and upgraded power amount");
}

#[test]
fn m11_silent_retain_confirm_zero_and_empty_hand_are_distinct() {
    let mut choose_zero = combat_with_retain_power(1);
    let original = open_selection(&mut choose_zero);
    assert_eq!(choose_zero.step_full(&CombatAction::ConfirmSelection), Ok(()));
    assert_eq!(choose_zero.state.phase, Phase::Decision);
    assert!(choose_zero.state.pending_selection.is_none());
    for card in original {
        assert!(choose_zero.state.piles.discard.iter().any(|owned| owned.handle() == card),
            "Confirm0 discards each original hand instance: {card:?}");
    }

    let mut empty = combat_with_retain_power(2);
    empty.state.piles.discard.append(&mut empty.state.piles.hand);
    assert_eq!(empty.step_full(&CombatAction::EndTurn), Ok(()));
    assert_eq!(empty.state.phase, Phase::Decision);
    assert!(empty.state.pending_selection.is_none(), "an empty hand does not create a zero-candidate prompt");
    assert!(!empty.legal_actions_full().unwrap().contains(&CombatAction::ConfirmSelection));
}

#[test]
fn m11_silent_retain_keeps_exact_nonethereal_instances_and_expires_after_one_use() {
    let mut combat = combat_with_retain_power(1);
    let original = open_selection(&mut combat);
    let selected = original[0].clone();
    select_and_confirm(&mut combat, &selected);

    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(combat.state.turn, 2);
    assert!(combat.state.piles.hand.iter().any(|owned| owned.handle() == selected),
        "the exact selected card remains in hand after end-turn cleanup");
    for card in original.iter().filter(|card| **card != selected) {
        assert!(combat.state.piles.discard.iter().any(|owned| owned.handle() == *card),
            "nonselected card is discarded without replacing the retained instance");
    }

    let second_turn = open_selection(&mut combat);
    assert!(second_turn.contains(&selected));
    assert_eq!(combat.step_full(&CombatAction::ConfirmSelection), Ok(()));
    assert!(combat.state.piles.discard.iter().any(|owned| owned.handle() == selected),
        "the first turn's temporary retain does not protect the card a second time");
}

#[test]
fn m11_silent_retain_keeps_ethereal_in_the_choice_but_it_still_exhausts() {
    let mut combat = combat_with_retain_power(1);
    let ethereal = combat.state.piles.hand[0].handle();
    combat.state.piles.hand[0].full_mut().unwrap().local.ethereal = true;
    let candidates = open_selection(&mut combat);
    assert!(candidates.contains(&ethereal), "Ethereal remains among the legal choices");
    select_and_confirm(&mut combat, &ethereal);
    assert!(combat.state.piles.exhaust.iter().any(|owned| owned.handle() == ethereal),
        "Well-Laid Plans does not turn Ethereal into immunity");
    assert!(!combat.state.piles.hand.iter().any(|owned| owned.handle() == ethereal));
}

#[test]
fn m11_silent_retain_stale_and_duplicate_requests_are_atomic() {
    let mut duplicate = combat_with_retain_power(2);
    let hand = open_selection(&mut duplicate);
    let instance = hand[0].as_str().parse().unwrap();
    assert_eq!(duplicate.step_full(&CombatAction::SelectCard { instance }), Ok(()));
    let before_duplicate = duplicate.state.clone();
    assert_eq!(duplicate.step_full(&CombatAction::SelectCard { instance }), Err(crate::run::RunError::InvalidAction));
    assert_eq!(duplicate.state, before_duplicate, "duplicate pick is preflight-atomic");

    let mut stale = combat_with_retain_power(1);
    let hand = open_selection(&mut stale);
    let candidate = hand[0].clone();
    let moved = stale.state.piles.hand.iter().position(|owned| owned.handle() == candidate).unwrap();
    let card = stale.state.piles.hand.remove(moved);
    stale.state.piles.discard.push(card);
    let before_stale = stale.state.clone();
    assert_eq!(stale.step_full(&CombatAction::SelectCard { instance: candidate.as_str().parse().unwrap() }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(stale.state, before_stale, "a candidate no longer in the captured hand is rejected without mutation");
}

#[test]
fn m11_silent_retain_snapshot_comparison_includes_pre_discard_membership() {
    let mut combat = combat_with_retain_power(1);
    let hand = open_selection(&mut combat);
    let selected = hand[0].clone();
    assert_eq!(combat.step_full(&CombatAction::SelectCard {
        instance: selected.as_str().parse().unwrap(),
    }), Ok(()));
    let checkpoint = combat.snapshot();

    // The EndTurn continuation's accepted pre-discard checkpoint owns this
    // selected instance. Construct the compared sibling from the real live
    // selector so only its retained-membership field differs.
    let mut retained = combat.clone();
    retained.state.temporary_retain = Some(super::super::model::TemporaryRetain {
        turn: retained.state.turn,
        cards: std::collections::BTreeSet::from([selected]),
    });
    assert_ne!(checkpoint, retained.snapshot(), "detached checkpoints preserve the membership marker");
    assert_eq!(super::super::snapshot::compare(&checkpoint, &retained.snapshot())
        .map(|divergence| divergence.component), Some("temporary_retain"),
        "the component comparator reports retained membership as a runtime state divergence");
}

#[test]
fn m11_silent_retain_accepted_fault_keeps_parent_and_selected_handles() {
    let mut combat = silent_combat();
    let power_instance=play_well_laid_plans(&mut combat,0);
    assert_eq!(combat.state.player.energy,2);
    let hand = open_selection(&mut combat);
    let selected = hand[0].clone();
    let instance = selected.as_str().parse().unwrap();
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance }), Ok(()));
    let discard_at = combat.state.pending.iter().position(|effect|effect.kind==EffectKind::DiscardHand).unwrap();
    combat.state.pending.insert(discard_at+1,Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
        kind:EffectKind::Damage(DamagePayload::Fixed(-1))});

    assert!(matches!(combat.step_full(&CombatAction::ConfirmSelection), Err(crate::run::RunError::Faulted(_))));

    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::Invariant);
    let retained = combat.state.fault_selection.as_ref().expect("accepted selection is retained for fault diagnostics");
    assert_eq!(retained.selected, vec![selected.clone()]);
    assert_eq!(retained.parent,SelectionParent::EndTurn{turn:1});
    assert!(combat.state.piles.hand.iter().any(|owned| owned.handle() == selected),
        "the accepted selection keeps the exact card through cleanup");
    assert_eq!(combat.state.player.energy,2,"an accepted selection fault does not replay the paid Power card");
    assert_eq!(combat.state.played_powers.iter().filter(|card|card.full().is_some_and(|card|
        card.instance.0==power_instance&&card.id==CardId::WellLaidPlans)).count(),1);
    assert!(combat.state.temporary_retain.is_none(),"the temporary owner expires after its actual DiscardHand pass");
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardDiscarded).count(),
        hand.len()-1,"the selected card is excluded while each nonselected card completes its discard callback once");
    assert_eq!(combat.state.current.as_ref().unwrap().kind,EffectKind::Damage(DamagePayload::Fixed(-1)));
    let faulted = combat.state.clone();
    assert!(matches!(combat.step_full(&CombatAction::ConfirmSelection),Err(crate::run::RunError::Faulted(_))));
    assert_eq!(combat.state,faulted,"a repeated request cannot rerun the accepted cleanup or replace the first fault");
}

#[test]
fn m11_silent_retain_uses_time_eaters_forced_end_turn_pipeline() {
    let mut entry = super::m8_shapes::entry(EncounterId::TimeEater);
    entry.cards = [CardId::TwinStrike, CardId::Defend, CardId::Defend].into_iter()
        .enumerate().map(|(index, id)| crate::combat::card_state::permanent(id, CardInstanceId(index as u64 + 1), 0)).collect();
    entry.opening_draw = 3;
    entry.setup.player_powers.push(FullPowerEntry { id: FullPowerId("retain_cards"),
        state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
    entry.setup.enemies[0].powers.iter_mut().find(|power| power.id == FullPowerId("time_warp")).unwrap()
        .state.amount = 11;
    entry.setup.enemies[0].intent = FullIntentId::from_name("time_ripple");
    let mut combat = Combat::enter(entry).unwrap();
    let remaining = combat.state.piles.hand.iter().find(|owned| owned.full().unwrap().id == CardId::Defend)
        .unwrap().handle();

    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }), Ok(()));

    let selection = combat.state.pending_selection.as_ref().expect("Time Warp routes through the shared end-turn retain hook");
    assert!(selection.candidates.contains(&remaining));
    assert_eq!(selection.maximum, 1);
}

#[test]
fn m11_well_laid_plans_plays_as_an_owned_power_and_sets_the_real_retain_limit() {
    for (upgrades,expected) in [(0,1),(1,2)] {
        let mut combat=silent_combat();
        let instance=combat.state.piles.hand[0].full().unwrap().instance;
        combat.state.piles.hand[0]=OwnedCard::Full(card_state::permanent(CardId::WellLaidPlans,instance,upgrades));
        let definition=crate::content::Catalog::standard().card(CardId::WellLaidPlans).expect("registered WLP card");
        assert_eq!((definition.color,definition.rarity,definition.card_type,definition.note_storage,definition.definition_only),
            (CardColor::Green,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent,false));
        assert_eq!(cards::visible_cost(&combat.state,combat.state.piles.hand[0].full().unwrap()),Some(1));

        assert_eq!(combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}),Ok(()));

        assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id==FullPowerId("retain_cards"))
            .map(|power|power.state.amount),Some(expected));
        assert_eq!(combat.state.player.energy,2);
        assert_eq!(combat.state.played_powers.iter().filter(|card|card.full().is_some_and(|card|
            card.instance==instance&&card.id==CardId::WellLaidPlans)).count(),1,
            "the played Power has one actual owner in the existing Power zone");
        assert_eq!(combat.step_full(&CombatAction::EndTurn),Ok(()));
        assert_eq!(combat.state.pending_selection.as_ref().unwrap().maximum,expected as usize);
    }
}

#[test]
fn m11_well_laid_plans_rejects_an_enemy_target_before_payment_or_mutation() {
    let mut combat = silent_combat();
    let instance = combat.state.piles.hand[0].full().unwrap().instance;
    combat.state.piles.hand[0] = OwnedCard::Full(card_state::permanent(CardId::WellLaidPlans, instance, 0));
    let before = combat.state.clone();

    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance: instance.0, target: Some(TargetId(1)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state, before, "an invalid target neither pays nor consumes the Power");
}
