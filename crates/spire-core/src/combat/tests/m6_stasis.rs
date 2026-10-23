use super::super::*;
use crate::content::{CardId,EncounterId};
use super::m5_elites::hit;

pub(super) fn combat(cards:Vec<LiveCard>)->Combat {
    let mut e=super::m6_ordinary::entry(EncounterId::BronzeAutomaton);e.setup.player.hp=300;e.setup.player.max_hp=300;e.cards=cards;e.opening_draw=0;
    let mut c=Combat::enter(e).unwrap();act(&mut c,0);assert_eq!(c.state.enemies.len(),3);c
}
fn card(id:CardId,n:u64)->LiveCard{permanent_live_card(id,CardInstanceId(n),0)}
fn cursor(c:&mut Combat,slot:usize,index:u128){let old=c.state.enemies[slot].full_ai_rng.as_ref().unwrap();c.state.enemies[slot].full_ai_rng=Some(rng::StreamState::new(old.id().clone(),[0;32],index).unwrap());}
fn roll(c:&mut Combat,slot:usize){let actor=Actor::Enemy(c.state.enemies[slot].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RollMove}],None).unwrap();}
fn act(c:&mut Combat,slot:usize){let actor=Actor::Enemy(c.state.enemies[slot].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove}],None).unwrap();}
fn capture(c:&mut Combat,slot:usize){if c.state.enemies[slot].intent_id.as_str()!="orb_stasis"{cursor(c,slot,1);roll(c,slot);}assert_eq!(c.state.enemies[slot].intent_id.as_str(),"orb_stasis");act(c,slot);}
fn card_rng(c:&mut Combat,index:u128){let old=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(old.id().clone(),[0;32],index).unwrap());}
fn all(c:&Combat)->Vec<CardInstanceId>{c.state.piles.all(c.state.in_play.as_ref()).chain(c.state.stasis.values()).map(|c|c.full().unwrap().instance).collect()}

#[test]
fn m6_stasis_moves_one_highest_rarity_live_card() {
    let mut rare=card(CardId::Bludgeon,4);rare.upgrades=1;rare.local.randomized_cost=Some(2);rare.local.temporary_cost=-1;rare.local.free_play=true;
    let mut c=combat(vec![card(CardId::Strike,1),card(CardId::TwinStrike,2),card(CardId::Disarm,3),rare.clone(),card(CardId::Offering,5)]);card_rng(&mut c,0);capture(&mut c,1);
    let orb=c.state.enemies[1].instance_id.clone();assert_eq!(c.state.stasis[&orb].full(),Some(&rare));assert_eq!(c.state.card_rng.as_ref().unwrap().word_index(),1);assert_eq!(c.state.piles.draw.len(),4);assert_eq!(all(&c).len(),5);
    let view=c.public_full().unwrap();assert_eq!(view.stasis.len(),1);assert_eq!((view.stasis[0].orb.clone(),view.stasis[0].card.instance,view.stasis[0].card.origin.clone(),view.stasis[0].card.upgrades),(orb,CardInstanceId(4),CombatCardRef::Permanent(CardInstanceId(4)),1));
    assert!(!c.legal_actions_full().unwrap().iter().any(|a|matches!(a,CombatAction::PlayCard{instance:4,..})));
    hit(&mut c,1,100);assert!(c.state.stasis.is_empty());assert_eq!(c.state.piles.hand[0].full(),Some(&rare));assert_eq!(c.state.card_ids_issued.len(),5);assert!(!c.public_full().unwrap().targets[1].1);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::PlayCard{instance:4,target:Some(TargetId(2))}).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_stasis_draw_priority_discard_fallback_and_single_card_two_orbs() {
    let mut c=combat(vec![card(CardId::Strike,1),card(CardId::Impervious,2)]);
    let index=c.state.piles.draw.iter().position(|c|c.full().unwrap().id==CardId::Impervious).unwrap();let rare=c.state.piles.draw.remove(index);c.state.piles.discard.push(rare);
    capture(&mut c,1);assert_eq!(c.state.stasis.values().next().unwrap().full().unwrap().id,CardId::Strike,"nonempty draw outranks even a Rare in discard");capture(&mut c,2);assert_eq!(c.state.stasis.len(),2);assert!(c.state.piles.draw.is_empty()&&c.state.piles.discard.is_empty());
    let mut one=combat(vec![card(CardId::Strike,1)]);capture(&mut one,1);let before=one.state.card_rng.clone();capture(&mut one,2);assert_eq!(one.state.stasis.len(),1);assert_eq!(one.state.card_rng,before);assert_eq!(all(&one),vec![CardInstanceId(1)]);
}

#[test]
fn m6_stasis_empty_full_hand_and_boss_exit_preserve_membership() {
    let mut c=combat(vec![card(CardId::Impervious,1)]);capture(&mut c,1);
    for _ in 0..10{c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::CreateCard{id:CardId::Shiv,destination:CardDestination::Hand,upgrades:0}}],None).unwrap();}
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("confused"),amount:1}}],None).unwrap();let stream=c.state.card_rng.clone();let draws=c.state.hook_events.iter().filter(|e|e.hook==Hook::CardDrawn).count();
    hit(&mut c,1,100);assert_eq!(c.state.piles.hand.len(),10);assert!(c.state.piles.discard.iter().any(|c|c.full().is_some_and(|c|c.instance==CardInstanceId(1))));assert_eq!(c.state.card_rng,stream);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CardDrawn).count(),draws);
    let mut boss=combat(vec![card(CardId::Impervious,1),card(CardId::Disarm,2)]);capture(&mut boss,1);capture(&mut boss,2);assert_eq!(boss.state.stasis.len(),2);hit(&mut boss,0,300);
    let exit=boss.exit().unwrap();assert_eq!(exit.outcome,Outcome::Victory);assert!(exit.stasis.is_empty());let mut ids=exit.hand.iter().chain(&exit.draw).chain(&exit.discard).chain(&exit.exhaust).chain(exit.in_play.iter()).map(|c|c.instance.0).collect::<Vec<_>>();ids.sort();assert_eq!(ids,vec![1,2]);assert_eq!(boss.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),3);
}

#[test]
fn m6_stasis_player_death_exports_living_orb_custody_once() {
    let mut c=combat(vec![card(CardId::Impervious,1),card(CardId::Strike,2)]);capture(&mut c,1);
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(300))}],None).unwrap();
    let exit=c.exit().unwrap();assert_eq!(exit.outcome,Outcome::Death);assert_eq!(exit.stasis.len(),1);assert_eq!(exit.stasis[0].origin,CombatCardRef::Permanent(CardInstanceId(1)));let mut ids=all(&c);ids.sort();assert_eq!(ids,vec![CardInstanceId(1),CardInstanceId(2)]);
}

#[test]
fn m6_stasis_candidate_rng_failure_keeps_pile_and_first_orb_owner() {
    let mut c=combat(vec![card(CardId::Impervious,1),card(CardId::Offering,2),card(CardId::Impervious,3)]);capture(&mut c,1);assert_eq!(c.state.stasis.len(),1);
    if c.state.enemies[2].intent_id.as_str()!="orb_stasis"{cursor(&mut c,2,1);roll(&mut c,2)}card_rng(&mut c,1_u128<<64);let before=(c.state.piles.clone(),c.state.stasis.clone(),c.state.card_ids_issued.clone());
    let orb=Actor::Enemy(c.state.enemies[2].instance_id.clone());assert!(c.enqueue_native_effects(vec![Effect{source:Source::Actor(orb.clone()),target:Some(orb),kind:EffectKind::EnemyMove}],None).is_err());assert_eq!((c.state.piles.clone(),c.state.stasis.clone(),c.state.card_ids_issued.clone()),before);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::RngExhausted);
    let snapshot=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),snapshot);
}

#[test]
fn m6_stasis_detached_card_remains_owned_when_death_return_faults() {
    let mut c=combat(vec![card(CardId::Impervious,1)]);capture(&mut c,1);let orb=c.state.enemies[1].instance_id.clone();let held=c.state.stasis[&orb].clone();
    c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(orb.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(100))});
    for _ in 0..12{if c.state.pending.front().is_some_and(|e|matches!(e.kind,EffectKind::ReleaseStasis{..})){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);assert_eq!(c.state.stasis[&orb],held);assert!(c.state.piles.hand.is_empty());assert_eq!(all(&c),vec![CardInstanceId(1)]);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_stasis_selected_flag_and_exact_integer_orb_branches() {
    use crate::content::EnemyId;
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"orb".into()]).unwrap();
    for (history,used,cursor,want,after) in [
        (vec![],false,70,"orb_beam",false),(vec![],false,34,"orb_stasis",true),
        (vec![],true,257,"orb_beam",true),(vec![],true,285,"orb_support",true),
        (vec!["orb_support"],true,285,"orb_support",true),(vec!["orb_support","orb_support"],true,285,"orb_beam",true),
        (vec!["orb_support","orb_beam"],true,285,"orb_support",true),(vec!["orb_support","orb_beam"],true,257,"orb_beam",true),
        (vec!["orb_beam"],true,257,"orb_beam",true),(vec!["orb_beam","orb_stasis"],true,257,"orb_beam",true),
        (vec!["orb_beam","orb_beam"],true,257,"orb_support",true),
    ] {
        let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();let selected=enemies::select_move(EnemyId::BronzeOrb,&history,&EnemyMemory::BronzeOrb{used_stasis:used},&stream).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.memory,EnemyMemory::BronzeOrb{used_stasis:after});assert_eq!(selected.stream.word_index(),cursor+1);
    }
}

#[test]
fn m6_stasis_capture_fault_keeps_detached_card_before_power_application() {
    let mut c=combat(vec![card(CardId::Disarm,1)]);let owner=c.state.enemies[1].instance_id.clone();if c.state.enemies[1].intent_id.as_str()!="orb_stasis"{cursor(&mut c,1,34);roll(&mut c,1)}
    let actor=Actor::Enemy(owner.clone());c.state.pending.push_back(Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove});
    for _ in 0..8{if c.state.stasis.contains_key(&owner){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    assert!(c.state.piles.draw.is_empty());assert_eq!(all(&c),vec![CardInstanceId(1)]);assert!(matches!(c.state.enemies[1].full_memory,Some(EnemyMemory::BronzeOrb{used_stasis:true})));let rng=c.state.card_rng.clone();
    assert!(matches!(c.state.pending.front().unwrap().kind,EffectKind::ApplyFullPower{id:FullPowerId("stasis"),amount:1}));c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);assert_eq!(c.state.card_rng,rng);assert_eq!(all(&c),vec![CardInstanceId(1)]);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_stasis_orb_support_and_beam_use_independent_live_owner() {
    let mut c=combat(vec![card(CardId::Strike,1)]);capture(&mut c,1);let sibling=c.state.enemies[2].clone();let boss_ai=c.state.enemies[0].full_ai_rng.clone();
    cursor(&mut c,1,285);roll(&mut c,1);assert_eq!(c.state.enemies[1].intent_id.as_str(),"orb_support");cursor(&mut c,1,257);act(&mut c,1);assert_eq!((c.state.enemies[0].block,c.state.player.hp),(12,300));assert_eq!(c.state.enemies[1].intent_id.as_str(),"orb_beam");act(&mut c,1);assert_eq!(c.state.player.hp,292);assert_eq!(c.state.enemies[0].full_ai_rng,boss_ai);assert_eq!(c.state.enemies[2],sibling);
}

#[test]
fn m6_stasis_public_capture_does_not_publish_remaining_order_or_future_ai(){
    let mut c=combat(vec![card(CardId::Impervious,1),card(CardId::Strike,2),card(CardId::Defend,3),card(CardId::Bash,4)]);capture(&mut c,1);let public=c.public_full().unwrap();let actions=c.legal_actions_full().unwrap();let snapshot=c.snapshot();assert_eq!(public.stasis[0].card.instance,CardInstanceId(1));assert!(c.state.piles.draw.len()>=2);
    c.state.piles.draw.swap(0,1);let ai=c.state.enemies[2].full_ai_rng.as_ref().unwrap();c.state.enemies[2].full_ai_rng=Some(rng::StreamState::new(ai.id().clone(),*ai.key(),ai.word_index()+1).unwrap());
    assert_ne!(c.snapshot(),snapshot,"hidden state really differs");assert_eq!(c.public_full().unwrap(),public);assert_eq!(c.legal_actions_full().unwrap(),actions,"the stolen known identity is visible, not untouched candidates/order/future rolls");
}
