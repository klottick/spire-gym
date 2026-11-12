//! spire-3xq: engine-level checks behind the M38e capacity findings (the catalog replays are in
//! `spire-env/tests/standard_3xq_findings.rs`).
use super::super::*;
use crate::content::{CardId,EncounterId};

/// F2-B: FireBreathingPower snapshots its amount into the queued damage. A stack added before that damage resolves raises
/// the live amount but not the queued hit, which still resolves for the queued value (it used to fault the Thorns
/// producer check, which required the live amount).
#[test]
fn x3q_fire_breathing_queued_thorns_resolve_for_the_queued_amount_after_a_later_stack(){
    let mut entry=super::m8_shapes::entry(EncounterId::ThreeShapes);
    entry.cards=vec![permanent_live_card(CardId::FireBreathing,CardInstanceId(1),0),permanent_live_card(CardId::Strike,CardInstanceId(2),0)];
    entry.opening_draw=2;
    let mut c=Combat::enter(entry).unwrap();
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="fire_breathing"&&p.state.amount==6));
    let target=c.state.enemies.iter().find(|e|e.enemy_id!=ActorEnemyId::Full(crate::content::EnemyId::Spiker)).unwrap();
    let (enemy,hp)=(target.instance_id.clone(),target.hp);
    // A status draw queued Thorns(6); before it resolves a second Fire Breathing stacks (as Mayhem's autoplay does).
    c.state.pending.push_front(Effect{source:Source::None,target:Some(Actor::Enemy(enemy.clone())),kind:EffectKind::Damage(DamagePayload::Thorns(6))});
    c.state.player_full_powers.iter_mut().find(|p|p.id.0=="fire_breathing").unwrap().state.amount=12;
    engine::resolve(&mut c.state);
    assert!(c.state.fault.is_none(),"{:?}",c.state.fault);
    assert_eq!(c.state.enemies.iter().find(|e|e.instance_id==enemy).unwrap().hp,hp-6,"the queued 6, not the live 12");
}

/// Cross-cutting: a step that stops at a selection has accepted its action; an inconsistent boundary there is a retained
/// Invariant fault, and later reads report that fault instead of `InvalidAction`. (The Reptomancer case that used to reach
/// it is fixed, so the permanent-gain frontier, which the step itself never reads, is broken by hand.)
#[test]
fn x3q_inconsistent_selection_boundary_is_a_retained_invariant_fault(){
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=card_state::permanent(CardId::TrueGrit,card.instance,1));
    let mut ok=c.clone();
    ok.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(ok.state.phase,Phase::Selection);
    assert!(ok.public_full().is_ok());
    c.state.next_permanent_gain+=1;
    let error=c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap_err();
    let crate::run::RunError::Faulted(fault)=&error else{panic!("{error:?}")};
    let crate::run::RunFaultKind::Combat(combat)=&fault.kind else{panic!("{fault:?}")};
    assert_eq!(combat.record.kind,FaultKind::Invariant);
    assert!(combat.record.message.contains("inconsistent selection"),"{}",combat.record.message);
    assert!(matches!(c.public_full(),Err(crate::run::RunError::Faulted(_))));
}

/// F2-D: a card offer (Discovery-style or Wish's choice) opens no choice once every enemy is dead; with a living enemy the
/// same offer opens. Synthetic: no engine path is known to reach an offer in a won combat, so the guard is checked on the
/// offer owner directly.
#[test]
fn x3q_card_offer_opens_no_choice_in_a_won_combat(){
    let offer=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
        kind:EffectKind::CardOffer{cards:vec![CardId::Strike,CardId::Defend,CardId::Bash],optional:true,copies:1,zero_cost:false}};
    let mut live=Combat::enter(super::m8_shapes::entry(EncounterId::ThreeShapes)).unwrap();
    super::super::card_offers::apply(&mut live.state,&offer).unwrap();
    assert!(live.state.silent.card_offer.is_some());
    assert_eq!(live.state.phase,Phase::Selection);
    let mut won=Combat::enter(super::m8_shapes::entry(EncounterId::ThreeShapes)).unwrap();
    for enemy in &mut won.state.enemies{enemy.hp=0;}
    let phase=won.state.phase;
    super::super::card_offers::apply(&mut won.state,&offer).unwrap();
    assert!(won.state.silent.card_offer.is_none());
    assert_eq!(won.state.phase,phase);
}
