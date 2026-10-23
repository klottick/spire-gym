use super::super::*;
use crate::content::CardId;

fn generated(c:&mut Combat,id:CardId,upgrades:u32,destination:CardDestination){c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::CreateCard{id,upgrades,destination}}],None).unwrap();}

#[test]
fn dazed_is_unplayable_and_exhausts_once_instead_of_discarding_at_turn_end() {
    let mut c=Combat::enter(super::shared_mechanics::entry()).unwrap();generated(&mut c,CardId::Dazed,0,CardDestination::Hand);
    let card=c.state.piles.hand.last().unwrap().full().unwrap().clone();
    assert!(matches!(card.origin,CombatCardRef::CombatLocal(_)));
    assert!(!c.legal_actions_full().unwrap().iter().any(|a|matches!(a,CombatAction::PlayCard{instance,..}if *instance==card.instance.0)));
    let before=c.snapshot();assert!(c.step_full(&CombatAction::PlayCard{instance:card.instance.0,target:None}).is_err());assert_eq!(c.snapshot(),before);
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.piles.exhaust.iter().filter(|c|c.full().unwrap().instance==card.instance).count(),1);
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CardExhausted&&e.source==Source::Card(card_state::handle(card.instance))).count(),1);
    assert!(!c.state.hook_events.iter().any(|e|matches!(e.hook,Hook::CardPlayed|Hook::CardDiscarded)&&e.source==Source::Card(card_state::handle(card.instance))));
}

#[test]
fn burn_end_turn_uses_block_without_playing_the_card_or_normal_modifiers() {
    for (upgrades,want) in [(0,80),(1,79)] {
        let mut c=Combat::enter(super::m5_elites::entry(crate::content::EncounterId::Cultist)).unwrap();
        generated(&mut c,CardId::Burn,upgrades,CardDestination::Hand);
        c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainBlock(3)},Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:2}},Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}}],None).unwrap();
        c.step_full(&CombatAction::EndTurn).unwrap();
        // Cultist's first move is non-damaging: isolate Burn2/4 vs Block3.
        assert_eq!(c.state.player.hp,want);assert!(!c.state.hook_events.iter().any(|e|e.hook==Hook::CardPlayed));
    }
}

#[test]
fn generated_statuses_have_fresh_local_ids_and_do_not_emit_acquisition() {
    let mut c=Combat::enter(super::shared_mechanics::entry()).unwrap();
    for id in [CardId::Dazed,CardId::Burn,CardId::Dazed] {generated(&mut c,id,0,CardDestination::Discard);}
    assert_eq!(c.state.piles.discard.iter().map(|c|c.full().unwrap().instance.0).collect::<Vec<_>>(),vec![4,5,6]);
    assert!(c.state.piles.discard.iter().all(|c|matches!(c.full().unwrap().origin,CombatCardRef::CombatLocal(_))));
    assert!(!c.state.hook_events.iter().any(|e|e.hook==Hook::CardAcquired));
}
