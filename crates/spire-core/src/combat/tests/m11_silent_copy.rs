use super::super::*;
use super::super::model::OwnedCard;
use super::full_runtime::entered_combat;

fn silent_copy_fixture() -> Combat {
    let mut combat = entered_combat(3, 3, |card| {
        card.id = crate::content::CardId::Strike;
        card.upgrades = 0;
    });
    combat.state.character = super::super::characters::CharacterState::for_character(
        crate::content::Character::Silent);
    combat
}

fn add_shiv(combat: &mut Combat, instance: u64, zone: CardDestination) {
    let card = super::super::card_state::generated(crate::content::CardId::Shiv,
        CardInstanceId(instance), 0).expect("Shiv has its generated constructor");
    assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(card.instance)));
    match zone {
        CardDestination::Hand => combat.state.piles.hand.push(OwnedCard::Full(card)),
        CardDestination::Discard => combat.state.piles.discard.push(OwnedCard::Full(card)),
        _ => panic!("fixture places Shivs in hand or discard"),
    }
}

fn apply_accuracy(combat: &mut Combat) {
    let effect = Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("accuracy"), amount: 4 } };
    super::super::powers::apply_effect(&mut combat.state, &effect)
        .expect("the real Accuracy owner accepts its represented power");
}

#[test]
fn m11_silent_accuracy_rewrites_hand_on_stat_copy_commit_without_draw_or_discard_events() {
    let mut combat = silent_copy_fixture();
    add_shiv(&mut combat, 80, CardDestination::Hand);
    add_shiv(&mut combat, 81, CardDestination::Discard);
    apply_accuracy(&mut combat);

    // Labelled native other-writer fixture: an active Accuracy power coexists
    // with hand and non-hand Shivs whose represented local damage was rewritten.
    for owned in combat.state.piles.hand.iter_mut().chain(&mut combat.state.piles.discard) {
        if let Some(card) = owned.full_mut().filter(|card| card.id == crate::content::CardId::Shiv) {
            card.local.base_damage = Some(77);
        }
    }
    let source = combat.state.piles.discard.iter().find_map(OwnedCard::full)
        .filter(|card| card.id == crate::content::CardId::Shiv).unwrap().clone();
    let revealed_before = combat.state.public.revealed_order.clone();
    let drawn_before = combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardDrawn).count();
    let discarded_before = combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardDiscarded).count();

    super::super::piles::stat_copy_to_hand(&mut combat.state, &source, 1)
        .expect("the existing stat-equivalent generated-copy insertion commits to hand");

    let hand_shivs = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == crate::content::CardId::Shiv).collect::<Vec<_>>();
    assert_eq!(hand_shivs.len(), 2);
    assert!(hand_shivs.iter().all(|card| card.local.base_damage == Some(8)),
        "hand insertion runs Accuracy's qualified hand-state recomputation for the copied Shiv and remaining hand Shivs");
    assert_eq!(combat.state.piles.discard.iter().find_map(OwnedCard::full)
        .filter(|card| card.instance == CardInstanceId(81)).unwrap().local.base_damage, Some(77),
        "the copy callback does not rewrite a source Shiv outside hand");
    assert_eq!(combat.state.public.revealed_order, revealed_before,
        "a generated copy is not an actual draw");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardDrawn).count(), drawn_before,
        "copy insertion does not emit the actual-draw hook");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardDiscarded).count(), discarded_before,
        "copy insertion does not emit a discard hook");
}

fn nightmare_id() -> crate::content::CardId {
    crate::content::Catalog::standard().cards().iter()
        .find(|definition| definition.id.as_str() == "nightmare")
        .map(|definition| definition.id)
        .expect("Nightmare has a registered native card definition")
}

fn nightmare_combat(upgrades: u32, energy: i32) -> Combat {
    let id = nightmare_id();
    let mut combat = entered_combat(energy, energy, |card| {
        card.id = id;
        card.upgrades = upgrades;
        card.local.exhaust_on_finish = true;
    });
    combat.state.character = super::super::characters::CharacterState::for_character(
        crate::content::Character::Silent);
    for instance in 100..105 {
        let card = super::super::card_state::permanent(crate::content::CardId::Strike,
            CardInstanceId(instance), 0);
        assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(card.instance)));
        combat.state.piles.draw.push(OwnedCard::Full(card));
    }
    combat
}

fn select_nightmare(combat: &mut Combat, nightmare: u64, selected: u64) {
    combat.step_full(&CombatAction::PlayCard { instance: nightmare, target: None })
        .expect("Nightmare opens its real remaining-hand selection");
    assert_eq!(combat.state.phase, Phase::Selection);
    combat.step_full(&CombatAction::SelectCard { instance: selected })
        .expect("one actual remaining-hand card is selected");
    assert_eq!(combat.state.phase, Phase::Decision);
}

#[test]
fn m11_silent_nightmare_copies_are_owned_and_delivered_once() {
    let definition = crate::content::Catalog::standard().card(nightmare_id()).unwrap();
    assert_eq!(definition.color, crate::content::CardColor::Green);
    assert_eq!(definition.rarity, crate::content::CardRarity::Rare);
    assert_eq!(definition.card_type, crate::content::CardType::Skill);
    assert!(!definition.definition_only);
    assert!(super::super::card_state::permanent(nightmare_id(),CardInstanceId(900),0)
        .local.exhaust_on_finish, "the ordinary Nightmare constructor owns its Exhaust disposition");
    assert!(super::super::card_state::generated(nightmare_id(),CardInstanceId(901),0)
        .unwrap().local.exhaust_on_finish, "the generated Nightmare constructor preserves its Exhaust disposition");

    let mut combat = nightmare_combat(0, 3);
    let nightmare = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == nightmare_id()).unwrap().instance.0;
    let source = combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "1")
        .and_then(OwnedCard::full_mut).expect("fixture has a selected card");
    source.id = crate::content::CardId::BloodForBlood;
    source.upgrades = 1;
    source.local.base_damage = Some(22);
    source.local.combat_cost_reduction = 1;
    source.local.temporary_cost = -1;
    source.local.free_play = true;
    source.local.temporary_upgrades = 2;
    let source_before = combat.state.piles.hand.iter().find(|owned| owned.handle().as_str() == "1")
        .and_then(OwnedCard::full).unwrap().clone();
    let issued_before = combat.state.card_ids_issued.clone();
    let first_draw = combat.state.piles.draw.last().unwrap().handle();
    let drawn_events_before = combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardDrawn).count();

    let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard { instance: nightmare, target: None })
        .expect("base Nightmare prepares as a targetless cost-three Skill");
    assert_eq!(prepared.paid, 3);
    select_nightmare(&mut combat, nightmare, 1);
    assert_eq!(combat.state.player.energy, 0);
    assert_eq!(combat.state.card_ids_issued, issued_before, "capturing a template creates no immediate card identities");
    assert_eq!(combat.state.piles.hand.iter().find(|owned| owned.handle().as_str() == "1")
        .and_then(OwnedCard::full), Some(&source_before), "the exact selected original card returns to hand");

    let original = combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "1")
        .and_then(OwnedCard::full_mut).unwrap();
    original.local.base_damage = Some(99);
    original.local.combat_cost_reduction = 3;
    original.local.temporary_cost = 2;
    original.local.free_play = false;
    original.upgrades = 0;

    let reveals_before_turn = combat.state.public.revealed_order.clone();
    combat.step_full(&CombatAction::EndTurn).expect("the live End Turn pipeline reaches the next player start");
    assert_eq!(combat.state.turn, 2);
    assert_eq!(combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == crate::content::CardId::BloodForBlood && card.instance.0 != 1).count(), 3,
        "exactly three independent children are delivered on the next player turn");
    let copies = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == crate::content::CardId::BloodForBlood && card.instance.0 != 1)
        .collect::<Vec<_>>();
    let copy_ids = copies.iter().map(|card| card.instance).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(copy_ids.len(), 3, "each child has a distinct generated identity");
    assert!(copies.iter().all(|card| card.upgrades == 1
        && card.local.base_damage == Some(22)
        && card.local.combat_cost_reduction == 1
        && card.local.temporary_cost == 0
        && card.local.free_play
        && card.local.temporary_upgrades == 0),
        "copies use the captured stat-equivalent values and reset only temporary copy state");
    let first_draw_position = combat.state.piles.hand.iter().position(|owned| owned.handle() == first_draw).unwrap();
    assert!(copies.iter().all(|copy|combat.state.piles.hand.iter().position(|owned|
        owned.handle().as_str() == copy.instance.0.to_string()).unwrap() < first_draw_position),
        "the committed copies precede the ordinary next-turn draw");
    assert!(combat.state.public.revealed_order.iter().skip(reveals_before_turn.len())
        .any(|handle| handle == &first_draw), "the normal draw remains publicly recorded");
    assert!(copies.iter().all(|copy| !combat.state.public.revealed_order.contains(&super::super::card_state::handle(copy.instance))),
        "generated copies do not fabricate draw reveal history");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardDrawn).count()
        - drawn_events_before, 5, "only the five ordinary cards emit actual CardDrawn hooks");
    assert!(combat.state.piles.discard.iter().filter_map(OwnedCard::full)
        .any(|card| card.instance.0 == 1 && card.local.base_damage == Some(99)),
        "the later-mutated original remains a separate owned card");
    assert!(combat.state.piles.exhaust.iter().filter_map(OwnedCard::full)
        .any(|card| card.instance.0 == nightmare), "Nightmare keeps its normal Exhaust disposition");
    assert!(combat.permanent_gains().is_empty());

    let mut upgraded = nightmare_combat(1, 3);
    let upgraded_id = upgraded.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == nightmare_id()).unwrap().instance.0;
    let prepared = super::super::cards::prepare(&upgraded,
        &CombatAction::PlayCard { instance: upgraded_id, target: None })
        .expect("upgraded Nightmare prepares as a targetless cost-two Skill");
    assert_eq!(prepared.paid, 2);
    select_nightmare(&mut upgraded, upgraded_id, 1);
    assert_eq!(upgraded.state.player.energy, 1, "the upgraded real play pays two energy");
    let issued_before = upgraded.state.card_ids_issued.clone();
    upgraded.step_full(&CombatAction::EndTurn).expect("upgraded Nightmare reaches the next qualified player turn");
    let new_ids = upgraded.state.card_ids_issued.difference(&issued_before).cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(new_ids.len(), 3);
    assert!(upgraded.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| new_ids.contains(&super::super::card_state::handle(card.instance)))
        .all(|card| card.id == crate::content::CardId::Strike),
        "the upgraded card also delivers exactly three copies on the next player start");
}

#[test]
fn m11_silent_nightmare_keeps_same_display_templates_independent() {
    let id = nightmare_id();
    let mut combat = nightmare_combat(0, 6);
    combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "2")
        .and_then(OwnedCard::full_mut).unwrap().id = id;
    combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "2")
        .and_then(OwnedCard::full_mut).unwrap().local.exhaust_on_finish = true;
    let source = combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "1")
        .and_then(OwnedCard::full_mut).unwrap();
    source.id = crate::content::CardId::BloodForBlood;
    source.upgrades = 1;
    source.local.base_damage = Some(22);
    source.local.combat_cost_reduction = 1;
    source.local.temporary_cost = -1;
    source.local.free_play = true;

    select_nightmare(&mut combat, 3, 1);
    assert_eq!(combat.state.nightmare_templates.len(), 1);
    let after_first = combat.snapshot();
    let mut changed_cursor = combat.clone();
    changed_cursor.state.nightmare_templates[0].delivered = 1;
    assert_eq!(super::super::snapshot::compare(&after_first, &changed_cursor.snapshot())
        .map(|divergence| divergence.component), Some("nightmare_templates"),
        "snapshots compare the captured template and its independent delivery cursor");

    let source = combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "1")
        .and_then(OwnedCard::full_mut).unwrap();
    source.local.base_damage = Some(90);
    source.local.combat_cost_reduction = 2;
    source.local.temporary_cost = 4;
    source.local.free_play = false;
    source.local.temporary_upgrades = 3;
    select_nightmare(&mut combat, 2, 1);
    assert_eq!(combat.state.nightmare_templates.len(), 2,
        "same-display captures use distinct Nightmare installation owners");

    let source = combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "1")
        .and_then(OwnedCard::full_mut).unwrap();
    source.local.base_damage = Some(111);
    source.local.combat_cost_reduction = 3;
    source.local.temporary_cost = -2;
    source.local.free_play = true;
    combat.validate_runtime_for_test().expect("two captured snapshots are not duplicate card owners");
    combat.step_full(&CombatAction::EndTurn).expect("both distinct templates reach the next player start");

    let copies = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == crate::content::CardId::BloodForBlood && card.instance.0 != 1)
        .collect::<Vec<_>>();
    assert_eq!(copies.len(), 6);
    assert_eq!(copies.iter().map(|card|card.instance).collect::<std::collections::BTreeSet<_>>().len(), 6);
    assert_eq!(copies.iter().filter(|card| card.local.base_damage == Some(22)
        && card.local.combat_cost_reduction == 1 && card.local.free_play && card.local.temporary_cost == 0
        && card.local.temporary_upgrades == 0 && card.upgrades == 1).count(), 3,
        "the first installation delivers its own earlier snapshot");
    assert_eq!(copies.iter().filter(|card| card.local.base_damage == Some(90)
        && card.local.combat_cost_reduction == 2 && !card.local.free_play && card.local.temporary_cost == 0
        && card.local.temporary_upgrades == 0 && card.upgrades == 1).count(), 3,
        "the second installation delivers its own later snapshot");
    assert!(combat.state.nightmare_templates.is_empty(), "both owner cursors retire after exactly three committed children");
    assert!(combat.permanent_gains().is_empty());
}

#[test]
fn m11_silent_nightmare_replay_after_exhume_creates_a_second_installation() {
    // Supplied native diagnostic fixture: seven energy pays Nightmare, Exhume,
    // then Nightmare again; no outcome is forced by this setup.
    let mut combat = nightmare_combat(0, 7);
    let nightmare = combat.state.piles.hand.iter_mut()
        .find(|owned| owned.handle().as_str() == "3").unwrap().full_mut().unwrap();
    assert_eq!(nightmare.id, nightmare_id());
    let exhuming = combat.state.piles.hand.iter_mut()
        .find(|owned| owned.handle().as_str() == "2").unwrap().full_mut().unwrap();
    exhuming.id = crate::content::CardId::Exhume;
    exhuming.local.exhaust_on_finish = true;
    let target_a = combat.state.piles.hand.iter_mut()
        .find(|owned| owned.handle().as_str() == "1").unwrap().full_mut().unwrap();
    target_a.id = crate::content::CardId::BloodForBlood;
    target_a.upgrades = 1;
    target_a.local.base_damage = Some(22);
    target_a.local.combat_cost_reduction = 1;
    target_a.local.free_play = true;

    select_nightmare(&mut combat, 3, 1);
    assert!(combat.state.piles.exhaust.iter().any(|owned| owned.handle().as_str() == "3"));
    assert_eq!(combat.state.nightmare_templates.len(), 1);
    let physical_owner = combat.state.nightmare_templates[0].owner.clone();

    combat.step_full(&CombatAction::PlayCard { instance: 2, target: None })
        .expect("the actual Exhume play returns the only eligible Nightmare from Exhaust");
    assert!(combat.state.pending_selection.is_none(), "the single exhausted Nightmare returns without a chooser");
    assert!(combat.state.piles.hand.iter().any(|owned| owned.handle() == physical_owner));
    assert!(combat.state.piles.exhaust.iter().any(|owned| owned.handle().as_str() == "2"));

    // Supplied native fixture: reuse an already-issued card from Draw as a second
    // capture target without claiming this setup is an ordinary draw event.
    let mut target_b = combat.state.piles.draw.pop().expect("fixture has an existing issued target");
    let target_b_id = target_b.handle();
    let target_b_live = target_b.full_mut().expect("fixture target is a full card");
    target_b_live.id = crate::content::CardId::BloodForBlood;
    target_b_live.upgrades = 0;
    target_b_live.local.base_damage = Some(90);
    target_b_live.local.combat_cost_reduction = 2;
    combat.state.piles.hand.push(target_b);
    combat.validate_runtime_for_test().expect("the reused parent and second existing target retain exact custody");

    let mut exhausted_identity = combat.clone();
    exhausted_identity.state.next_nightmare_installation_id = u64::MAX;
    let source_b = exhausted_identity.state.piles.hand.iter().find(|owned| owned.handle() == target_b_id)
        .and_then(OwnedCard::full).unwrap().clone();
    let before_exhaustion = exhausted_identity.snapshot();
    assert!(super::super::cards::capture_silent_nightmare_template(
        &mut exhausted_identity.state, physical_owner.clone(), &source_b).is_err(),
        "installation identity exhaustion is a checked failure rather than wrapping to zero");
    assert_eq!(exhausted_identity.snapshot(), before_exhaustion,
        "rejected installation allocation preserves the prior template and cursor");

    combat.step_full(&CombatAction::PlayCard { instance: 3, target: None })
        .expect("the same physical Nightmare can be replayed after Exhume returns it");
    assert_eq!(combat.state.phase, Phase::Selection);
    let second_capture = combat.step_full(&CombatAction::SelectCard {
        instance: target_b_id.as_str().parse().unwrap(),
    });
    assert!(second_capture.is_ok(),
        "a second cast installs its own snapshot even while the first installation shares its physical owner; got {second_capture:?}");
    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(combat.state.nightmare_templates.len(), 2);
    assert!(combat.state.nightmare_templates.iter().all(|pending| pending.owner == physical_owner),
        "both installations retain attribution to the same physical Nightmare card");
    assert_ne!(combat.state.nightmare_templates[0].installation_id,
        combat.state.nightmare_templates[1].installation_id,
        "each accepted cast has an independent installation identity");
    let captured = combat.snapshot();
    let mut changed_counter = combat.clone();
    changed_counter.state.next_nightmare_installation_id += 1;
    assert_eq!(super::super::snapshot::compare(&captured, &changed_counter.snapshot())
        .map(|divergence| divergence.component), Some("next_nightmare_installation_id"),
        "snapshots include the checked installation allocation cursor");
    combat.validate_runtime_for_test().expect("same-parent installations validate as distinct captured owners");

    combat.step_full(&CombatAction::EndTurn).expect("both installations deliver on the next qualified player start");
    let copies = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == crate::content::CardId::BloodForBlood
            && card.instance.0 != 1 && card.instance.0 != target_b_id.as_str().parse::<u64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(copies.len(), 6, "the same physical owner delivers three copies per installation");
    assert_eq!(copies.iter().map(|card| card.instance).collect::<std::collections::BTreeSet<_>>().len(), 6,
        "both independent installations allocate distinct children");
    assert_eq!(copies.iter().filter(|card| card.upgrades == 1 && card.local.base_damage == Some(22)
        && card.local.combat_cost_reduction == 1 && card.local.free_play).count(), 3,
        "the first cast keeps its own earlier template");
    assert_eq!(copies.iter().filter(|card| card.upgrades == 0 && card.local.base_damage == Some(90)
        && card.local.combat_cost_reduction == 2 && !card.local.free_play).count(), 3,
        "the replay keeps its own later template");
    assert!(combat.state.nightmare_templates.is_empty(), "each installation retires after its third committed copy");
}

#[test]
fn m11_silent_nightmare_empty_and_stale_selection_keep_exact_ownership() {
    let mut invalid_target = nightmare_combat(0, 3);
    let nightmare = invalid_target.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == nightmare_id()).unwrap().instance.0;
    let before = invalid_target.snapshot();
    assert_eq!(invalid_target.step_full(&CombatAction::PlayCard {
        instance: nightmare, target: Some(TargetId(1)),
    }), Err(crate::run::RunError::InvalidAction));
    assert_eq!(invalid_target.snapshot(), before,
        "targetless Nightmare rejects an enemy target before payment, movement, or selection");
    invalid_target.validate_runtime_for_test().expect("rejected target leaves exact ownership");

    let mut empty = nightmare_combat(0, 3);
    let nightmare = empty.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == nightmare_id()).unwrap().instance.0;
    let all = std::mem::take(&mut empty.state.piles.hand);
    for card in all {
        if card.full().is_some_and(|instance| instance.instance.0 == nightmare) {
            empty.state.piles.hand.push(card);
        } else {
            empty.state.piles.discard.push(card);
        }
    }
    empty.validate_runtime_for_test().expect("empty-hand Nightmare fixture still has exact card custody");
    let ids_before = empty.state.card_ids_issued.clone();
    empty.step_full(&CombatAction::PlayCard { instance: nightmare, target: None })
        .expect("Nightmare with no eligible remaining hand card completes without a selection");
    assert_eq!(empty.state.phase, Phase::Decision);
    assert!(empty.state.pending_selection.is_none(), "empty eligibility creates no Confirm menu");
    assert!(empty.state.nightmare_templates.is_empty());
    assert_eq!(empty.state.card_ids_issued, ids_before);
    assert!(empty.state.piles.exhaust.iter().any(|owned| owned.handle().as_str() == nightmare.to_string()));

    let mut stale = nightmare_combat(0, 3);
    let nightmare = stale.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == nightmare_id()).unwrap().instance.0;
    stale.step_full(&CombatAction::PlayCard { instance: nightmare, target: None })
        .expect("real Nightmare selection opens");
    assert_eq!(stale.state.pending_selection.as_ref().unwrap().minimum, 1);
    assert_eq!(stale.state.pending_selection.as_ref().unwrap().maximum, 1);
    let before = stale.snapshot();
    assert_eq!(stale.step_full(&CombatAction::SelectCard { instance: 999 }), Err(crate::run::RunError::InvalidAction));
    assert_eq!(stale.snapshot(), before, "stale selection is rejected before moving a card or capturing a template");
}

#[test]
fn m11_silent_nightmare_partial_delivery_fault_keeps_cursor_and_first_child() {
    let mut combat = nightmare_combat(0, 3);
    let nightmare = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == nightmare_id()).unwrap().instance.0;
    combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "1")
        .and_then(OwnedCard::full_mut).unwrap().id = crate::content::CardId::Shiv;
    select_nightmare(&mut combat, nightmare, 1);
    assert_eq!(combat.state.nightmare_templates.len(), 1);
    let blocker = super::super::card_state::permanent(crate::content::CardId::Strike,
        CardInstanceId(u64::MAX - 1), 0);
    assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(blocker.instance)));
    combat.state.piles.discard.push(OwnedCard::Full(blocker));
    combat.validate_runtime_for_test().expect("the high-ID diagnostic owner remains a valid issued card");

    assert!(combat.step_full(&CombatAction::EndTurn).is_err(),
        "the second child reaches the actual unsigned identity overflow after one commit");
    assert_eq!(combat.fault_record().unwrap().kind, FaultKind::Unsupported);
    assert_eq!(combat.state.nightmare_templates.len(), 1);
    assert_eq!(combat.state.nightmare_templates[0].delivered, 1,
        "the owner cursor advances exactly with the committed first child");
    assert!(combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .any(|card| card.id == crate::content::CardId::Shiv && card.instance.0 == u64::MAX),
        "the first generated child remains in its actual hand destination");
    assert_eq!(combat.state.public.revealed_order.iter()
        .filter(|handle| handle.as_str() == u64::MAX.to_string()).count(), 0);
    assert!(combat.permanent_gains().is_empty());
    let frozen = combat.snapshot();
    assert!(combat.step_full(&CombatAction::EndTurn).is_err());
    assert_eq!(combat.snapshot(), frozen, "faulted copy work cannot retry or duplicate the accepted child");
}

#[test]
fn m11_silent_nightmare_terminal_cleanup_drops_templates_without_gains() {
    let mut combat = nightmare_combat(0, 3);
    let nightmare = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == nightmare_id()).unwrap().instance.0;
    select_nightmare(&mut combat, nightmare, 1);
    let issued_before = combat.state.card_ids_issued.clone();
    assert_eq!(combat.state.nightmare_templates.len(), 1);
    combat.state.enemies[0].hp = 1;
    combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "2")
        .and_then(OwnedCard::full_mut).unwrap().id = crate::content::CardId::Neutralize;
    combat.step_full(&CombatAction::PlayCard { instance: 2, target: Some(TargetId(1)) })
        .expect("ordinary lethal Neutralize settles this supplied diagnostic combat");
    assert_eq!(combat.state.phase, Phase::Terminal);
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert!(combat.state.nightmare_templates.is_empty(), "terminal cleanup abandons combat-scoped templates");
    assert_eq!(combat.state.card_ids_issued, issued_before, "cleanup creates no copies or phantom card owners");
    assert!(combat.permanent_gains().is_empty());
}

#[test]
fn m11_silent_nightmare_hand_overflow_keeps_hand_callback_scope_before_draw() {
    let mut combat = nightmare_combat(0, 3);
    let nightmare = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == nightmare_id()).unwrap().instance.0;
    let accuracy = Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("accuracy"), amount: 4 } };
    super::super::powers::apply_effect(&mut combat.state, &accuracy)
        .expect("the native Accuracy owner is active for this generated-card fixture");
    let source = combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "1")
        .and_then(OwnedCard::full_mut).unwrap();
    source.id = crate::content::CardId::Shiv;
    source.local.base_damage = Some(77);
    select_nightmare(&mut combat, nightmare, 1);

    // Labelled native start-turn fixture: retain a nine-card hand at the live
    // pre-draw boundary so this same ordinary insertion path exercises overflow.
    let mut remaining_shiv = super::super::card_state::generated(crate::content::CardId::Shiv,
        CardInstanceId(200), 0).unwrap();
    remaining_shiv.local.base_damage = Some(77);
    assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(remaining_shiv.instance)));
    combat.state.piles.hand.push(OwnedCard::Full(remaining_shiv));
    for instance in 201..207 {
        let filler = super::super::card_state::permanent(crate::content::CardId::Strike,
            CardInstanceId(instance), 0);
        assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(filler.instance)));
        combat.state.piles.hand.push(OwnedCard::Full(filler));
    }
    assert_eq!(combat.state.piles.hand.len(), 9);
    combat.validate_runtime_for_test().expect("the supplied nine-card start hand has exact ownership");
    let issued_before = combat.state.card_ids_issued.clone();
    let reveals_before = combat.state.public.revealed_order.clone();
    let draw_events_before = combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardDrawn).count();
    let discard_events_before = combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardDiscarded).count();

    super::super::turns::schedule_next_player_turn(&mut combat.state).unwrap();
    let transition = super::super::engine::resolve(&mut combat.state);
    combat.refresh_legacy_cache();
    assert_eq!(transition.status, super::super::TransitionStatus::Decision);
    combat.validate_runtime_for_test().expect("overflow children and remaining templates preserve runtime ownership");

    let new_ids = combat.state.card_ids_issued.difference(&issued_before).cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(new_ids.len(), 3);
    let copies = combat.state.piles.hand.iter().chain(&combat.state.piles.discard)
        .filter_map(OwnedCard::full)
        .filter(|card| new_ids.contains(&super::super::card_state::handle(card.instance)))
        .collect::<Vec<_>>();
    assert_eq!(copies.len(), 3);
    assert_eq!(combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| new_ids.contains(&super::super::card_state::handle(card.instance))).count(), 1);
    assert_eq!(combat.state.piles.discard.iter().filter_map(OwnedCard::full)
        .filter(|card| new_ids.contains(&super::super::card_state::handle(card.instance))).count(), 2,
        "the ordinary hand-cap rule redirects the excess Nightmare children to discard");
    assert!(combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == crate::content::CardId::Shiv)
        .all(|card| card.local.base_damage == Some(8)),
        "the committed Hand child rewrites itself and the other live hand Shivs through Accuracy");
    assert!(combat.state.piles.discard.iter().filter_map(OwnedCard::full)
        .filter(|card| new_ids.contains(&super::super::card_state::handle(card.instance)))
        .all(|card| card.id == crate::content::CardId::Shiv && card.local.base_damage == Some(77)),
        "overflow constructors go directly to Discard and do not run the hand-only callback");
    assert_eq!(combat.state.public.revealed_order, reveals_before);
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardDrawn).count(),draw_events_before,
        "a full-hand ordinary draw is suppressed and generated copies are not actual draws");
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardDiscarded).count(),discard_events_before,
        "hand insertion and overflow do not fabricate a manual-discard notification");
}
