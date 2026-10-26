use super::super::*;
use super::super::model::OwnedCard;
use super::full_runtime::entered_combat;

fn with_silent_hand_card(id: crate::content::CardId, upgrades: u32) -> Combat {
    let mut combat = entered_combat(3, 3, |card| {
        card.id = id;
        card.upgrades = upgrades;
    });
    combat.state.character = super::super::characters::CharacterState::for_character(
        crate::content::Character::Silent);
    combat
}

fn set_hand_card(combat: &mut Combat, instance: u64, id: crate::content::CardId, upgrades: u32) {
    let card = combat.state.piles.hand.iter_mut()
        .find(|owned| owned.handle().as_str() == instance.to_string())
        .and_then(OwnedCard::full_mut)
        .expect("fixture owns the requested hand instance");
    card.id = id;
    card.upgrades = upgrades;
}

fn apply_accuracy(combat: &mut Combat, amount: i32) {
    let effect = Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("accuracy"), amount } };
    super::super::powers::apply_effect(&mut combat.state, &effect)
        .expect("real full-power application accepts Accuracy");
}

fn add_shiv(combat: &mut Combat, instance: u64, upgrades: u32, zone: u8) {
    let card = super::super::card_state::generated(crate::content::CardId::Shiv,
        CardInstanceId(instance), upgrades).expect("Shiv has its existing generated constructor");
    assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(card.instance)));
    let owned = OwnedCard::Full(card);
    match zone {
        0 => combat.state.piles.hand.push(owned),
        1 => combat.state.piles.draw.push(owned),
        2 => combat.state.piles.discard.push(owned),
        3 => combat.state.piles.exhaust.push(owned),
        _ => unreachable!("test zone is hand/draw/discard/exhaust"),
    }
}

fn shivs(combat: &Combat) -> Vec<&super::super::card_state::LiveCard> {
    combat.state.piles.hand.iter().chain(&combat.state.piles.draw)
        .chain(&combat.state.piles.discard).chain(&combat.state.piles.exhaust)
        .filter_map(OwnedCard::full)
        .filter(|card| card.id == crate::content::CardId::Shiv).collect()
}

#[test]
fn m11_silent_shiv_generation_and_accuracy_preserve_instance_ownership() {
    for (id, upgrades, expected_damage) in [
        (crate::content::CardId::Shiv, 0, 8),
        (crate::content::CardId::Shiv, 1, 10),
        (crate::content::CardId::Neutralize, 0, 3),
    ] {
        let mut combat = with_silent_hand_card(id, upgrades);
        apply_accuracy(&mut combat, 4);
        let instance = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card| card.id == id).expect("fixture owns the requested real card").instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: Some(TargetId(1)),
        }).expect("actual Silent attack card prepares with its owned target");
        assert_eq!(prepared.paid, 0, "ordinary Shiv attacks cost zero");
        let damage = prepared.effects.iter().find_map(|effect| match &effect.kind {
            EffectKind::Damage(DamagePayload::Fixed(amount)) => Some(*amount),
            _ => None,
        }).expect("attack payload contains actual fixed damage");
        assert_eq!(damage, expected_damage,
            "Accuracy adds four only to Shiv, not to every zero-cost attack");
        let hp = combat.state.enemies[0].hp;
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) })
            .expect("the actual card action resolves through the shared combat queue");
        assert_eq!(combat.state.enemies[0].hp, hp - expected_damage);
    }

    let mut stacked = with_silent_hand_card(crate::content::CardId::Shiv, 1);
    apply_accuracy(&mut stacked, 4);
    apply_accuracy(&mut stacked, 6);
    assert_eq!(stacked.state.player_full_powers.iter()
        .find(|power| power.id == FullPowerId("accuracy")).map(|power| power.state.amount), Some(10));
    let instance = stacked.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == crate::content::CardId::Shiv).unwrap().instance.0;
    let shiv = stacked.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == crate::content::CardId::Shiv).unwrap();
    assert_eq!(shiv.local.base_damage, Some(16), "upgraded Shiv base plus stacked Accuracy is qualified before modifiers");
    let prepared = super::super::cards::prepare(&stacked, &CombatAction::PlayCard {
        instance, target: Some(TargetId(1)),
    }).unwrap();
    assert_eq!(prepared.effects.iter().find_map(|effect| match &effect.kind {
        EffectKind::Damage(DamagePayload::Fixed(amount)) => Some(*amount), _ => None,
    }), Some(16));
    let hp = stacked.state.enemies[0].hp;
    stacked.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(stacked.state.enemies[0].hp, hp - 16);
}

#[test]
fn m11_silent_accuracy_card_rewrites_existing_piles_and_future_shivs() {
    for (accuracy_upgrade, amount) in [(0, 4), (1, 6)] {
        let mut combat = with_silent_hand_card(crate::content::CardId::Accuracy, accuracy_upgrade);
        set_hand_card(&mut combat, 1, crate::content::CardId::Neutralize, 0);
        set_hand_card(&mut combat, 2, crate::content::CardId::BladeDance, 0);
        set_hand_card(&mut combat, 3, crate::content::CardId::Accuracy, accuracy_upgrade);
        add_shiv(&mut combat, 70, 0, 0);
        add_shiv(&mut combat, 71, 1, 0);
        add_shiv(&mut combat, 72, 0, 1);
        add_shiv(&mut combat, 73, 0, 2);
        add_shiv(&mut combat, 74, 0, 3);
        combat.state.enemies[0].hp = 14 + 2 * amount;
        let strike = super::super::card_state::permanent(
            crate::content::CardId::Strike, CardInstanceId(90), 0);
        assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(strike.instance)));
        combat.state.piles.hand.push(OwnedCard::Full(strike));
        let accuracy = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card| card.id == crate::content::CardId::Accuracy).unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance: accuracy, target: None })
            .expect("the real Accuracy card applies its upgraded power");
        assert_eq!(combat.state.player_full_powers.iter()
            .find(|power| power.id == FullPowerId("accuracy")).map(|power| power.state.amount), Some(amount));
        let observed = shivs(&combat).into_iter()
            .map(|card| (card.instance.0, card.local.base_damage))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(observed.get(&70), Some(&Some(4 + amount)));
        assert_eq!(observed.get(&71), Some(&Some(6 + amount)));
        for instance in [72, 73, 74] {
            assert_eq!(observed.get(&instance), Some(&Some(4 + amount)),
                "Accuracy rewrites actual Shiv identity in every eligible pile");
        }

        let blade_dance = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card| card.id == crate::content::CardId::BladeDance).unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance: blade_dance, target: None })
            .expect("Blade Dance uses the shared generated-Shiv constructor after Accuracy");
        let new_shivs = shivs(&combat).into_iter().filter(|card| card.instance.0 > 74)
            .collect::<Vec<_>>();
        assert_eq!(new_shivs.len(), 3);
        assert!(new_shivs.iter().all(|card| card.upgrades == 0
            && card.local.base_damage == Some(4 + amount)
            && card.local.exhaust_on_finish
            && matches!(card.origin, CombatCardRef::CombatLocal(_))),
            "Blade Dance creates distinct ordinary Shivs with the current Accuracy base");

        let neutralize = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card| card.id == crate::content::CardId::Neutralize).unwrap().instance.0;
        let before_neutralize = combat.state.enemies[0].hp;
        combat.step_full(&CombatAction::PlayCard { instance: neutralize, target: Some(TargetId(1)) })
            .expect("Accuracy does not invalidate another legal zero-cost Attack");
        assert_eq!(combat.state.enemies[0].hp, before_neutralize - 3,
            "Accuracy does not add damage to Neutralize");

        for (instance, expected) in [(70_u64, 4 + amount), (71, 6 + amount)] {
            let before = combat.state.enemies[0].hp;
            combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) })
                .expect("the same Shiv instance remains playable after Accuracy rewrites it");
            assert_eq!(combat.state.enemies[0].hp, before - expected);
            assert!(combat.state.piles.exhaust.iter()
                .any(|owned| owned.handle().as_str() == instance.to_string()));
        }
        let strike = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card| card.id == crate::content::CardId::Strike).unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance: strike, target: Some(TargetId(1)) })
            .expect("a separate ordinary Attack can settle the diagnostic combat after both Shivs finish");
        assert_eq!(combat.state.outcome, Some(Outcome::Victory));

        let exit = combat.exit().expect("actual Victory exports its native card census");
        let exported = exit.hand.iter().chain(&exit.draw).chain(&exit.discard).chain(&exit.exhaust)
            .chain(exit.in_play.iter()).chain(&exit.play_frames).chain(&exit.queued_autoplay)
            .chain(&exit.stasis).chain(&exit.played_powers).chain(&exit.inactive_cards)
            .collect::<Vec<_>>();
        let ids = exported.iter().map(|card| card.instance.0).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), exported.len(), "every permanent/generated card has one exit owner");
        assert_eq!(exported.iter().filter(|card| matches!(card.origin, CombatCardRef::Permanent(_))).count(), 4);
        assert_eq!(exported.iter().filter(|card| matches!(card.origin, CombatCardRef::CombatLocal(_))).count(), 8);
        assert!(combat.permanent_gains().is_empty(), "Accuracy and generated Shivs never enter the permanent deck");
    }
}

#[test]
fn m11_silent_blade_dance_creates_three_or_four_distinct_unupgraded_shivs() {
    for (generator_upgrade, count) in [(0, 3), (1, 4)] {
        let mut combat = with_silent_hand_card(crate::content::CardId::BladeDance, generator_upgrade);
        combat.state.enemies[0].hp = count as i32 * 4 + 2;
        let instance = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card| card.id == crate::content::CardId::BladeDance).unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: None,
        }).expect("Blade Dance is a real one-energy Silent skill");
        assert_eq!(prepared.paid, 1);
        let created = prepared.effects.iter().filter_map(|effect| match &effect.kind {
            EffectKind::CreateCard { id, destination: CardDestination::Hand, upgrades } => Some((*id, *upgrades)),
            _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(created, vec![(crate::content::CardId::Shiv, 0); count as usize]);
        combat.step_full(&CombatAction::PlayCard { instance, target: None })
            .expect("Blade Dance creates ordinary Shivs through the shared queue");
        let generated = shivs(&combat);
        assert_eq!(generated.len(), count);
        let ids = generated.iter().map(|card| card.instance.0).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), count, "each Shiv is a distinct owned instance");
        assert!(generated.iter().all(|card| card.upgrades == 0 && card.local.exhaust_on_finish
            && matches!(card.origin, CombatCardRef::CombatLocal(_))));
        let generated_instances = generated.iter().map(|card| card.instance.0).collect::<Vec<_>>();
        assert_eq!(combat.public_full().unwrap().player.energy, 2);
        let mut expected_hp = combat.state.enemies[0].hp;
        for instance in generated_instances {
            let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
                instance, target: Some(TargetId(1)),
            }).expect("generated ordinary Shiv is a legal targeted attack");
            assert_eq!(prepared.paid, 0);
            let damage = prepared.effects.iter().find_map(|effect| match &effect.kind {
                EffectKind::Damage(DamagePayload::Fixed(amount)) => Some(*amount),
                _ => None,
            }).unwrap();
            assert_eq!(damage, 4, "generated Shivs retain ordinary damage");
            combat.step_full(&CombatAction::PlayCard { instance,
                target: Some(TargetId(1)) }).expect("ordinary Shiv resolves through shared queue");
            expected_hp -= 4;
            assert_eq!(combat.state.enemies[0].hp, expected_hp);
            assert!(combat.state.piles.exhaust.iter()
                .any(|owned| owned.handle().as_str() == instance.to_string()));
        }
        assert_eq!(expected_hp, 2, "all generated Shivs finish before separate settlement");
        let strike = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card| card.id == crate::content::CardId::Strike).unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance: strike, target: Some(TargetId(1)) })
            .expect("a separate Attack settles after each Shiv has exhausted");
        assert_eq!(combat.state.enemies[0].hp, 0);
        assert_eq!(combat.state.outcome, Some(Outcome::Victory));
        assert!(combat.permanent_gains().is_empty());
        let exit = combat.exit().expect("Victory exports all surviving generated card identities");
        let exported = exit.hand.iter().chain(&exit.draw).chain(&exit.discard).chain(&exit.exhaust)
            .chain(exit.in_play.iter()).chain(&exit.play_frames).chain(&exit.queued_autoplay)
            .chain(&exit.stasis).chain(&exit.played_powers).chain(&exit.inactive_cards)
            .collect::<Vec<_>>();
        let exported_ids = exported.iter().map(|card| card.instance.0).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(exported_ids.len(), exported.len());
        assert_eq!(exported.iter().filter(|card| matches!(card.origin, CombatCardRef::CombatLocal(_))).count(), count);
    }
}

#[test]
fn m11_silent_accuracy_actual_draw_rewrites_only_hand_shivs() {
    let mut combat = with_silent_hand_card(crate::content::CardId::Strike, 0);
    apply_accuracy(&mut combat, 4);
    add_shiv(&mut combat, 70, 0, 0);
    add_shiv(&mut combat, 71, 0, 1);
    add_shiv(&mut combat, 72, 0, 2);
    add_shiv(&mut combat, 73, 0, 3);

    // Labelled native other-writer fixture: Accuracy has already applied, then
    // another owner changes all four live Shiv base-damage values.
    for owned in combat.state.piles.hand.iter_mut()
        .chain(&mut combat.state.piles.draw)
        .chain(&mut combat.state.piles.discard)
        .chain(&mut combat.state.piles.exhaust)
    {
        if let Some(card) = owned.full_mut().filter(|card| card.id == crate::content::CardId::Shiv) {
            card.local.base_damage = Some(77);
        }
    }

    // Put a real non-Shiv card on top of the draw pile. The actual draw path
    // moves it into hand and dispatches CardDrawn.
    let strike = combat.state.piles.hand.iter().find(|owned|
        owned.full().is_some_and(|card| card.id == crate::content::CardId::Strike))
        .map(OwnedCard::handle).expect("fixture starts with a non-Shiv Strike");
    let strike_index = combat.state.piles.hand.iter().position(|owned| owned.handle() == strike).unwrap();
    let strike = combat.state.piles.hand.remove(strike_index);
    combat.state.piles.draw.push(strike);

    combat.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Draw(1) }], None).expect("one real draw resolves through the shared pile owner");

    let damage = |pile: &[OwnedCard], instance| pile.iter().filter_map(OwnedCard::full)
        .find(|card| card.instance.0 == instance && card.id == crate::content::CardId::Shiv)
        .and_then(|card| card.local.base_damage);
    assert_eq!(damage(&combat.state.piles.hand, 70), Some(8),
        "the actual non-Shiv draw callback restores the hand Shiv's own 4 plus Accuracy 4");
    assert_eq!(damage(&combat.state.piles.draw, 71), Some(77),
        "a Shiv remaining in draw is not rewritten by this hand-only callback");
    assert_eq!(damage(&combat.state.piles.discard, 72), Some(77),
        "a Shiv in discard is not rewritten by this hand-only callback");
    assert_eq!(damage(&combat.state.piles.exhaust, 73), Some(77),
        "a Shiv in exhaust is not rewritten by this hand-only callback");
    assert!(combat.state.hook_events.iter().any(|event| event.hook == Hook::CardDrawn),
        "the hand rewrite follows the real draw hook rather than card construction");
}

#[test]
fn m11_silent_accuracy_manual_discard_rewrites_hand_but_end_turn_does_not() {
    let mut combat = with_silent_hand_card(crate::content::CardId::DaggerThrow, 0);
    set_hand_card(&mut combat, 2, crate::content::CardId::Strike, 0);
    add_shiv(&mut combat, 70, 0, 0);
    add_shiv(&mut combat, 71, 0, 1);
    add_shiv(&mut combat, 72, 0, 2);
    add_shiv(&mut combat, 73, 0, 3);

    // Keep a real non-Shiv on top for Dagger Throw's actual draw.
    let drawn = combat.state.piles.hand.iter().find(|owned|
        owned.handle().as_str() == "1").map(OwnedCard::handle).unwrap();
    let drawn_index = combat.state.piles.hand.iter().position(|owned| owned.handle() == drawn).unwrap();
    let drawn = combat.state.piles.hand.remove(drawn_index);
    combat.state.piles.draw.push(drawn);
    apply_accuracy(&mut combat, 4);
    for owned in combat.state.piles.hand.iter_mut()
        .chain(&mut combat.state.piles.draw)
        .chain(&mut combat.state.piles.discard)
        .chain(&mut combat.state.piles.exhaust)
    {
        if let Some(card) = owned.full_mut().filter(|card| card.id == crate::content::CardId::Shiv) {
            card.local.base_damage = Some(77);
        }
    }

    combat.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) })
        .expect("real Dagger Throw reaches its draw and selected-discard continuation");
    assert_eq!(combat.state.phase, Phase::Selection,
        "Dagger Throw's actual post-draw discard is a public hand selection");
    let damage = |pile: &[OwnedCard], instance| pile.iter().filter_map(OwnedCard::full)
        .find(|card| card.instance.0 == instance && card.id == crate::content::CardId::Shiv)
        .and_then(|card| card.local.base_damage);
    assert_eq!(damage(&combat.state.piles.hand, 70), Some(8),
        "Dagger Throw's actual non-Shiv draw rewrites the existing hand Shiv first");

    // Change the same hand Shiv after draw, then discard a different real card
    // through Dagger Throw's actual public selection. This is a labelled native
    // other-writer fixture for Accuracy's manual-discard callback.
    combat.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "70")
        .and_then(OwnedCard::full_mut).unwrap().local.base_damage = Some(77);
    combat.step_full(&CombatAction::SelectCard { instance: 2 })
        .expect("the selected Strike is an actual manual discard through the public action");
    assert_eq!(damage(&combat.state.piles.hand, 70), Some(8),
        "manual discard rewrites remaining hand Shivs only");
    assert_eq!(damage(&combat.state.piles.draw, 71), Some(77));
    assert_eq!(damage(&combat.state.piles.discard, 72), Some(77));
    assert_eq!(damage(&combat.state.piles.exhaust, 73), Some(77));
    assert!(combat.state.piles.discard.iter().any(|owned| owned.handle().as_str() == "2"),
        "the selected card reaches its real discard pile");

    let mut end_turn = with_silent_hand_card(crate::content::CardId::Strike, 0);
    apply_accuracy(&mut end_turn, 4);
    add_shiv(&mut end_turn, 80, 0, 0);
    end_turn.state.piles.hand.iter_mut().find(|owned| owned.handle().as_str() == "80")
        .and_then(OwnedCard::full_mut).unwrap().local.base_damage = Some(77);
    end_turn.enqueue_native_effects(vec![Effect { source: Source::None, target: None,
        kind: EffectKind::DiscardHand }], None)
        .expect("the actual end-turn DiscardHand owner resolves before a later turn draw");
    assert_eq!(damage(&end_turn.state.piles.discard, 80), Some(77),
        "end-turn discard suppresses the manual-discard listener");
}

#[test]
fn m11_silent_blade_dance_target_payment_and_hand_overflow_are_atomic() {
    let mut combat = with_silent_hand_card(crate::content::CardId::BladeDance, 1);
    for instance in 80..87 {
        let card = super::super::card_state::generated(crate::content::CardId::Shiv,
            CardInstanceId(instance), 0).expect("diagnostic hand filler has an owned constructor");
        assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(card.instance)));
        combat.state.piles.hand.push(OwnedCard::Full(card));
    }
    assert_eq!(combat.state.piles.hand.len(), 10);
    let instance = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == crate::content::CardId::BladeDance).unwrap().instance.0;

    let before_wrong_target = combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.snapshot(), before_wrong_target, "no-target generator rejects a target atomically");

    let shiv = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == crate::content::CardId::Shiv).unwrap().instance.0;
    let before_stale_target = combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance: shiv, target: Some(TargetId(999)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.snapshot(), before_stale_target, "a stale Shiv target is rejected before payment or movement");

    combat.state.player.energy = 0;
    let before_unaffordable = combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance, target: None }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.snapshot(), before_unaffordable, "unaffordable generator leaves every owner unchanged");

    combat.state.player.energy = 1;
    let draw_before = combat.state.piles.draw.len();
    combat.step_full(&CombatAction::PlayCard { instance, target: None })
        .expect("paid Blade Dance resolves and uses current hand capacity");
    assert_eq!(combat.state.piles.hand.len(), 10);
    assert_eq!(combat.state.piles.draw.len(), draw_before, "created cards are not draws");
    assert_eq!(shivs(&combat).len(), 11, "seven supplied hand diagnostics remain plus four children");
    assert_eq!(combat.state.piles.hand.iter().filter(|owned| owned.full()
        .is_some_and(|card| card.id == crate::content::CardId::Shiv && card.instance.0 >= 87)).count(), 1);
    assert_eq!(combat.state.piles.discard.iter().filter(|owned| owned.full()
        .is_some_and(|card| card.id == crate::content::CardId::Shiv && card.instance.0 >= 87)).count(), 3,
        "the three generated children that exceed hand capacity go to discard");
    assert_eq!(combat.public_full().unwrap().player.energy, 0);
    assert!(combat.permanent_gains().is_empty());
    assert!(super::super::engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m11_silent_blade_dance_insertion_fault_retains_first_child_and_unfinished_parent() {
    let mut combat = with_silent_hand_card(crate::content::CardId::BladeDance, 1);
    let near_max = super::super::card_state::permanent(
        crate::content::CardId::Strike, CardInstanceId(u64::MAX - 1), 0);
    assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(near_max.instance)));
    combat.state.piles.hand.push(OwnedCard::Full(near_max));
    assert!(super::super::engine::validate_runtime(&combat.state).is_ok());
    let parent = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == crate::content::CardId::BladeDance).unwrap().instance.0;

    assert!(combat.step_full(&CombatAction::PlayCard { instance: parent, target: None }).is_err(),
        "the accepted generator reaches an actual generated-identity overflow");
    assert_eq!(combat.state.player.energy, 2, "accepted payment remains committed");
    assert_eq!(combat.state.in_play.as_ref().and_then(OwnedCard::full).map(|card| card.instance.0), Some(parent),
        "the failed generated insertion retains its in-flight parent");
    assert_eq!(combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == crate::content::CardId::Shiv).count(), 1,
        "the first child at u64::MAX remains owned after the second child overflows");
    assert!(combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .any(|card| card.id == crate::content::CardId::Shiv && card.instance.0 == u64::MAX));
    assert!(matches!(combat.state.current.as_ref().map(|effect| &effect.kind),
        Some(EffectKind::CreateCard { id: crate::content::CardId::Shiv,
            destination: CardDestination::Hand, upgrades: 0 })));
    assert_eq!(combat.state.pending.iter().filter(|effect| matches!(effect.kind,
        EffectKind::CreateCard { id: crate::content::CardId::Shiv, .. })).count(), 2,
        "later Blade Dance children remain queued behind the current fault");
    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::Unsupported);
    let frozen = combat.snapshot();
    assert!(combat.step_full(&CombatAction::EndTurn).is_err());
    assert_eq!(combat.snapshot(), frozen, "faulted parent and children are never replayed");
}

#[test]
fn m11_silent_cloak_and_dagger_keeps_block_and_creates_ordinary_shivs() {
    for (upgrades, count) in [(0, 1), (1, 2)] {
        let mut combat = with_silent_hand_card(crate::content::CardId::CloakAndDagger, upgrades);
        let instance = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .find(|card| card.id == crate::content::CardId::CloakAndDagger).unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: None,
        }).expect("Cloak and Dagger keeps its existing no-target payload");
        assert_eq!(prepared.paid, 1);
        assert!(prepared.effects.iter().any(|effect| effect.kind == EffectKind::GainBlock(6)));
        assert_eq!(prepared.effects.iter().filter(|effect|
            matches!(effect.kind, EffectKind::CreateCard { id: crate::content::CardId::Shiv,
                destination: CardDestination::Hand, upgrades: 0 })).count(), count);

        combat.step_full(&CombatAction::PlayCard { instance, target: None })
            .expect("existing Cloak and Dagger resolves through the shared queue");
        assert_eq!(combat.state.player.block, 6);
        let generated = shivs(&combat);
        assert_eq!(generated.len(), count);
        assert!(generated.iter().all(|card| card.upgrades == 0 && card.local.exhaust_on_finish));
        let ids = generated.iter().map(|card| card.instance.0).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), count, "each generated ordinary Shiv has its own instance");
    }
}
