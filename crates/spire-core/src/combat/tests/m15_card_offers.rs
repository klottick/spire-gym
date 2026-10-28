use super::super::*;
use crate::content::CardId;

fn offer(optional:bool)->Effect {
    Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
        kind:EffectKind::CardOffer{cards:vec![CardId::Slice,CardId::Deflect,CardId::Backflip],optional,copies:2,zero_cost:true}}
}

#[test]
fn m15_card_offers_are_templates_with_atomic_indices_and_live_hand_overflow() {
    let mut c=super::full_runtime::entered_combat(3,3,|_|{});
    let old=c.public_full().unwrap();assert!(!format!("{old:?}").contains("card_offer"));
    let issued=c.state.card_ids_issued.clone();let rng=c.state.card_rng.clone();
    c.enqueue_native_effects(vec![offer(true)],None).unwrap();
    let view=c.public_full().unwrap();let offers=view.card_offer.unwrap();
    assert_eq!(offers.cards,vec![CardId::Slice,CardId::Deflect,CardId::Backflip]);assert!(offers.optional);
    assert_eq!(c.state.card_ids_issued,issued,"templates do not issue owned instance handles");
    assert_eq!(c.state.card_rng,rng,"inspection never rerolls");
    assert_eq!(c.legal_actions_full().unwrap(),vec![CombatAction::ChooseCardOffer{index:0},CombatAction::ChooseCardOffer{index:1},CombatAction::ChooseCardOffer{index:2},CombatAction::SkipCardOffer]);
    let before=c.snapshot();assert_eq!(c.step_full(&CombatAction::ChooseCardOffer{index:3}),Err(crate::run::RunError::InvalidAction));assert_eq!(c.snapshot(),before);
    for instance in 100..106 {
        let card=permanent_live_card(CardId::DefendGreen,CardInstanceId(instance),0);
        c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(model::OwnedCard::Full(card));
    }
    c.step_full(&CombatAction::ChooseCardOffer{index:2}).unwrap();
    assert_eq!(c.state.piles.hand.len(),10);assert_eq!(c.state.piles.discard.len(),1);
    let generated=c.state.piles.hand.iter().chain(&c.state.piles.discard).filter_map(model::OwnedCard::full)
        .filter(|card|card.id==CardId::Backflip).collect::<Vec<_>>();
    assert_eq!(generated.len(),2);assert_ne!(generated[0].instance,generated[1].instance);
    assert!(generated.iter().all(|card|card.local.turn_cost==Some(0)));
    let before=c.snapshot();assert_eq!(c.step_full(&CombatAction::ChooseCardOffer{index:2}),Err(crate::run::RunError::InvalidAction));assert_eq!(c.snapshot(),before);
}

#[test]
fn m15_card_offer_skip_is_source_specific_and_resumes_the_existing_queue_once() {
    for optional in [false,true] {
        let mut c=super::full_runtime::entered_combat(3,3,|_|{});
        let issued=c.state.card_ids_issued.clone();
        c.enqueue_native_effects(vec![offer(optional),Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainEnergy(2)}],None).unwrap();
        assert_eq!(c.state.player.energy,3);
        if optional {
            c.step_full(&CombatAction::SkipCardOffer).unwrap();
            assert_eq!(c.state.player.energy,5);assert_eq!(c.state.card_ids_issued,issued);
            assert!(c.public_full().unwrap().card_offer.is_none());
        } else {
            let before=c.snapshot();assert_eq!(c.step_full(&CombatAction::SkipCardOffer),Err(crate::run::RunError::InvalidAction));assert_eq!(c.snapshot(),before);
            c.step_full(&CombatAction::ChooseCardOffer{index:0}).unwrap();assert_eq!(c.state.player.energy,5);
        }
    }
}

#[test]
fn m15_card_offer_delivery_fault_keeps_first_copy_cursor_and_first_fault() {
    let mut c=super::full_runtime::entered_combat(3,3,|_|{});
    c.enqueue_native_effects(vec![offer(true)],None).unwrap();
    for instance in 1000..1995 {
        let card=card_state::generated(CardId::Slice,CardInstanceId(instance),0).unwrap();
        c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.inactive_cards.push(model::OwnedCard::Full(card));
    }
    assert_eq!(c.state.card_ids_issued.len(),998);
    let error=c.step_full(&CombatAction::ChooseCardOffer{index:0}).unwrap_err();
    assert!(matches!(error,crate::run::RunError::Faulted(_)));
    assert_eq!(c.state.card_ids_issued.len(),999);
    assert_eq!(c.state.piles.hand.iter().filter_map(model::OwnedCard::full).filter(|card|card.id==CardId::Slice).count(),1);
    assert!(matches!(c.state.current.as_ref().map(|effect|&effect.kind),Some(EffectKind::DeliverCardOffer{copies:2,next:1,..})));
    let before=c.snapshot();assert_eq!(c.step_full(&CombatAction::ChooseCardOffer{index:0}),Err(error));assert_eq!(c.snapshot(),before);
}

#[test]
fn m15_card_offer_zero_cost_guard_preserves_x_and_unplayable_templates() {
    for index in 0..3 {
        let mut c=super::full_runtime::entered_combat(3,3,|_|{});
        let mut work=offer(true);
        if let EffectKind::CardOffer{cards,copies,..}=&mut work.kind{*cards=vec![CardId::Skewer,CardId::Reflex,CardId::Tactician];*copies=1;}
        c.enqueue_native_effects(vec![work],None).unwrap();c.step_full(&CombatAction::ChooseCardOffer{index}).unwrap();
        let card=c.state.piles.hand.last().unwrap().full().unwrap();
        if index==0 {assert!(card.local.x_cost);assert_eq!(card.local.turn_cost,Some(-1));}
        else {assert_eq!(card.local.turn_cost,None);assert_eq!(cards::public_values(card).unwrap().cost_kind,CardCostKind::Unplayable);}
    }
}
