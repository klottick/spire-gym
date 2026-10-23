use super::super::*;
use crate::content::{CardId,EncounterId};
use super::m8_shapes::entry;
use super::m5_elites::{amount,hit};

// Explicit supplied native hands/counter; never a normal-start policy fixture.
fn fixture(cards:&[CardId],hand:&[u64],count:i32)->Combat{
    let mut e=entry(EncounterId::TimeEater);e.cards=cards.iter().enumerate().map(|(i,id)|permanent_live_card(*id,CardInstanceId(i as u64+1),0)).collect();e.opening_draw=0;e.setup.enemies[0].powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=count;e.setup.enemies[0].intent=FullIntentId::from_name("time_ripple");
    let mut c=Combat::enter(e).unwrap();for id in hand{let at=c.state.piles.draw.iter().position(|c|c.full().unwrap().instance==CardInstanceId(*id)).unwrap();let card=c.state.piles.draw.remove(at);c.state.piles.hand.push(card);}c
}
fn set(c:&mut Combat,name:&'static str){let i=ActorIntentId::Full(FullIntentId::from_name(name));let e=&mut c.state.enemies[0];e.intent_id=i.clone();e.selected_move=i.clone();e.move_history.push(i);effects::refresh_enemy_values(&mut c.state).unwrap();}
fn own(c:&mut Combat,kind:EffectKind){let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind}],None).unwrap();}
fn play(c:&mut Combat,id:u64){let card=c.state.piles.hand.iter().find(|c|c.full().unwrap().instance==CardInstanceId(id)).unwrap().full().unwrap();let attack=crate::content::Catalog::standard().card(card.id).unwrap().card_type==crate::content::CardType::Attack;c.step_full(&CombatAction::PlayCard{instance:id,target:attack.then_some(TargetId(1))}).unwrap();}

#[test]
fn m8_time_warp_twelfth_card_finishes_owned_work(){
    let mut c=fixture(&[CardId::TwinStrike],&[1],11);play(&mut c,1);assert_eq!((c.state.enemies[0].hp,c.state.turn,amount(&c,0,"time_warp"),amount(&c,0,"strength")),(446,2,0,2));assert!(c.state.in_play.is_none()&&c.state.pending_time_warp.is_none());assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),1);
    let mut c=fixture(&[CardId::Offering,CardId::Defend,CardId::Defend,CardId::Defend,CardId::Defend,CardId::Defend],&[1],11);play(&mut c,1);assert_eq!((c.state.player.hp,c.state.turn),(294,2));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CardDrawn&&e.source==Source::Card(CardHandle::parse("1").unwrap())).count(),3);assert!(c.state.piles.exhaust.iter().any(|c|c.full().unwrap().instance==CardInstanceId(1)));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),1);
}

#[test]
fn m8_time_counter_survives_turns_and_invalid_actions(){
    let mut items=Some(crate::items::CombatItemContext::new(vec![],vec![Some(crate::content::PotionId::FirePotion)]));let mut e=entry(EncounterId::TimeEater);e.cards=vec![permanent_live_card(CardId::Anger,CardInstanceId(1),0)];for id in 2..=5{e.cards.push(permanent_live_card(CardId::Defend,CardInstanceId(id),0));}e.opening_draw=0;e.setup.enemies[0].intent=FullIntentId::from_name("time_ripple");e.setup.enemies[0].powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=9;let mut c=Combat::enter_with_item_context(e,&mut items).unwrap();let at=c.state.piles.draw.iter().position(|c|c.full().unwrap().instance==CardInstanceId(1)).unwrap();let card=c.state.piles.draw.remove(at);c.state.piles.hand.push(card);
    play(&mut c,1);assert_eq!(amount(&c,0,"time_warp"),10);set(&mut c,"time_ripple");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,0,"time_warp"),10);let frozen=c.state.clone();assert!(c.step_full(&CombatAction::PlayCard{instance:99,target:None}).is_err());assert_eq!(c.state,frozen);c.step_full(&CombatAction::UsePotion{slot:0,target:Some(TargetId(1))}).unwrap();assert_eq!(amount(&c,0,"time_warp"),10);
    let id=c.state.piles.hand[0].full().unwrap().instance.0;play(&mut c,id);assert_eq!(amount(&c,0,"time_warp"),11);set(&mut c,"time_ripple");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,0,"time_warp"),11);let id=c.state.piles.hand[0].full().unwrap().instance.0;play(&mut c,id);assert_eq!((amount(&c,0,"time_warp"),c.state.turn),(0,4));
}

#[test]
fn m8_time_haste_once_strict_threshold(){
    let mut c=fixture(&[CardId::Defend],&[1],0);hit(&mut c,0,228);own(&mut c,EffectKind::RollMove);assert_ne!(c.state.enemies[0].intent_id.as_str(),"time_haste");hit(&mut c,0,1);own(&mut c,EffectKind::RollMove);assert_eq!(c.state.enemies[0].intent_id.as_str(),"time_haste");assert!(matches!(c.state.enemies[0].full_memory,Some(EnemyMemory::TimeEater{haste_used:true})));
    let mut second=c.clone();own(&mut second,EffectKind::RollMove);assert_ne!(second.state.enemies[0].intent_id.as_str(),"time_haste","selection, not execution, spent its latch");
    own(&mut c,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:4});own(&mut c,EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2});own(&mut c,EffectKind::ApplyFullPower{id:FullPowerId("shackled"),amount:5});own(&mut c,EffectKind::GainBlock(9));own(&mut c,EffectKind::EnemyMove);assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,amount(&c,0,"strength"),amount(&c,0,"weak"),amount(&c,0,"shackled")),(228,9,4,0,0));hit(&mut c,0,10);own(&mut c,EffectKind::RollMove);assert_ne!(c.state.enemies[0].intent_id.as_str(),"time_haste");
}

#[test]
fn m8_time_draw_reduction_lifecycle(){
    for base in [1,5]{let mut e=entry(EncounterId::TimeEater);e.cards=(1..=8).map(|id|permanent_live_card(CardId::Defend,CardInstanceId(id),0)).collect();e.opening_draw=0;e.setup.player.draw_per_turn=base;e.setup.enemies[0].intent=FullIntentId::from_name("time_slam");let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.piles.hand.len(),(base-1) as usize);assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="draw_reduction").unwrap().state.amount,1);set(&mut c,"time_ripple");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.piles.hand.len(),base as usize);assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="draw_reduction"));assert!(engine::validate_runtime(&c.state).is_ok());}
}

#[test]
fn m8_time_real_attack_bases_ripple_and_artifact_draw_prevention(){
    for (intent,hp) in [("time_reverb",279),("time_slam",274),("time_ripple",300)]{
        let mut c=fixture(&[CardId::Defend],&[1],0);set(&mut c,intent);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,hp);
        if intent=="time_ripple"{assert_eq!(c.state.enemies[0].block,20);for id in ["weak","vulnerable"]{assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0==id).unwrap().state.amount,1);}}
    }
    let mut c=fixture(&[CardId::Defend],&[1],0);c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("artifact"),amount:1}}],None).unwrap();set(&mut c,"time_slam");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,274);assert_eq!(c.state.piles.hand.len(),1);assert!(c.state.player_full_powers.iter().all(|p|!matches!(p.id.0,"artifact"|"draw_reduction")));
}

#[test]
fn m8_time_selection_finishes_before_the_barrier(){
    let mut c=fixture(&[CardId::BurningPact,CardId::Defend,CardId::Strike,CardId::Defend,CardId::Defend,CardId::Defend],&[1,2,3],11);play(&mut c,1);assert_eq!(c.state.phase,Phase::Selection);assert_eq!((amount(&c,0,"time_warp"),c.state.turn),(11,1));assert!(c.state.pending_time_warp.is_none());let frozen=c.state.clone();assert!(c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).is_err());assert_eq!(c.state,frozen);c.step_full(&CombatAction::SelectCard{instance:2}).unwrap();assert_eq!((c.state.turn,amount(&c,0,"time_warp"),amount(&c,0,"strength")),(2,0,2));assert!(c.state.pending_selection.is_none()&&c.state.in_play.is_none());assert!(c.state.piles.exhaust.iter().any(|c|c.full().unwrap().instance==CardInstanceId(2)));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),1);
}

#[test]
fn m8_time_terminal_current_card_wins_over_forced_turn_request(){
    for death in [false,true]{let mut c=fixture(&[if death{CardId::Offering}else{CardId::Strike},CardId::Defend],&[1],11);if death{c.state.player.hp=6;}else{hit(&mut c,0,450);}play(&mut c,1);assert_eq!(c.state.outcome,Some(if death{Outcome::Death}else{Outcome::Victory}));assert_eq!(c.state.turn,1);assert!(c.state.pending_time_warp.is_none());assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),0);assert_eq!(c.exit().unwrap().in_play.unwrap().instance,CardInstanceId(1));}
}

// A missing barrier owner must never become a ready decision or an injected
// turn-end/cleanse producer. These catch absent native custody validation.
#[test]
fn m8_time_orphan_request_and_forged_producers_fail_closed(){
    let c=fixture(&[CardId::Defend],&[1],11);let owner=c.state.enemies[0].instance_id.clone();
    let mut orphan=c.clone();orphan.state.pending_time_warp=Some(owner.clone());let mut admitted=vec![engine::validate_runtime(&orphan.state).is_ok()];
    for kind in [EffectKind::TimeWarpEndTurn,EffectKind::TimeEaterCleanse]{admitted.push(engine::validate_effect(&Effect{source:Source::None,target:None,kind},&c.state).is_ok());}
    let e=Effect{source:Source::Actor(Actor::Enemy(owner)),target:None,kind:EffectKind::TimeWarpEndTurn};admitted.push(engine::validate_effect(&e,&c.state).is_ok());assert_eq!(admitted,[false;4]);
}

fn prepare(c:&mut Combat){
    let prepared=cards::prepare(c,&CombatAction::PlayCard{instance:1,target:None}).unwrap();let card=c.state.piles.hand.remove(prepared.hand_index);let source=Source::Card(card.handle());c.state.in_play=Some(card);c.state.player.energy=prepared.remaining_energy;c.state.phase=Phase::PlayerAction;
    c.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardPlayed)});c.state.pending.extend(prepared.effects);c.state.pending.push_back(Effect{source,target:None,kind:EffectKind::FinishCard});c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
}

#[test]
fn m8_time_faults_preserve_current_card_request_and_single_end_turn(){
    for after_barrier in [false,true]{
        let mut c=fixture(&[CardId::Defend,CardId::Strike],&[1,2],11);prepare(&mut c);
        for _ in 0..16{if c.state.pending.front().is_some_and(|e|e.kind==EffectKind::TimeWarpEndTurn){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
        assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::TimeWarpEndTurn);assert_eq!(amount(&c,0,"strength"),2);assert_eq!(amount(&c,0,"time_warp"),0);assert!(c.state.in_play.is_none());assert!(c.state.piles.discard.iter().any(|c|c.full().unwrap().instance==CardInstanceId(1)));
        if after_barrier{engine::dispatch_one(&mut c.state);assert!(c.state.pending_time_warp.is_none());assert_eq!(c.state.pending.len(),1);assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::EndPlayerTurn);}else{assert_eq!(c.state.pending_time_warp,Some(c.state.enemies[0].instance_id.clone()));}
        c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);assert_eq!(c.state.current.as_ref().unwrap().kind,if after_barrier{EffectKind::EndPlayerTurn}else{EffectKind::TimeWarpEndTurn});assert_eq!(c.state.turn,1);assert_eq!(c.state.outcome,None);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::TurnEnded).count(),0);
        let frozen=c.state.clone();assert!(c.legal_actions_full().is_err());assert!(c.step_full(&CombatAction::PlayCard{instance:2,target:Some(TargetId(1))}).is_err());assert_eq!(c.state,frozen);assert!(engine::validate_runtime(&c.state).is_ok());
    }
    // Fault while the twelfth card's selection still owns its continuation:
    // no after-use callback or forced-end request has happened yet.
    let mut c=fixture(&[CardId::BurningPact,CardId::Defend,CardId::Strike],&[1,2,3],11);play(&mut c,1);c.state.resolved_effects=99_999;assert!(c.step_full(&CombatAction::SelectCard{instance:2}).is_err());assert_eq!(amount(&c,0,"time_warp"),11);assert!(c.state.pending_time_warp.is_none());assert_eq!(c.state.in_play.as_ref().unwrap().full().unwrap().instance,CardInstanceId(1));assert!(c.state.fault_selection.is_some());
}

#[test]
fn m8_time_exact_history_redirect_tickets_keep_native_stream(){
    let c=fixture(&[CardId::Defend],&[1],0);let owner=c.state.enemies[0].instance_id.clone();let context=enemies::act2::EnemyRollContext{owner:owner.clone(),turn:1,actors:vec![enemies::act2::ActorSummary{owner,species:crate::content::EnemyId::TimeEater,hp:456,max_hp:456,slot:0}]};
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"time".into()]).unwrap();
    // Fixed literal outcomes for each ordinary branch and both recursive ranges.
    for (history,lo,hi,range,second_lo,second_hi,want) in [
        (vec![],0,44,0,0,0,"time_reverb"),(vec![],45,79,0,0,0,"time_slam"),(vec![],80,99,0,0,0,"time_ripple"),
        (vec!["time_reverb";2],0,44,50,0,29,"time_slam"),(vec!["time_reverb";2],0,44,50,30,49,"time_ripple"),
        (vec!["time_ripple"],80,99,75,0,44,"time_reverb"),(vec!["time_ripple"],80,99,75,45,74,"time_slam"),
        (vec!["time_slam"],45,79,1<<24,0,11_072_962,"time_reverb"),(vec!["time_slam"],45,79,1<<24,11_072_963,16_777_215,"time_ripple"),
    ]{
        let (stream,next)=(0..2048).find_map(|cursor|{let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();let(n,next)=rng::uniform_below(&stream,100).unwrap();if !(lo..=hi).contains(&n){return None}if range==0{return Some((stream,next))}let(n,next)=rng::uniform_below(&next,range).unwrap();(second_lo..=second_hi).contains(&n).then_some((stream,next))}).unwrap();
        let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let selected=enemies::act2::select_move(crate::content::EnemyId::TimeEater,&history,&EnemyMemory::TimeEater{haste_used:false},&stream,&context).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream,next);
    }
}
