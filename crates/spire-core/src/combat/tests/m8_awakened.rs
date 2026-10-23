use super::super::*;
use crate::content::{CardId,EncounterId};
use super::m8_shapes::entry;
use super::m5_elites::{amount,hit};

fn set(c:&mut Combat,name:&'static str){let i=ActorIntentId::Full(FullIntentId::from_name(name));let e=&mut c.state.enemies[2];e.intent_id=i.clone();e.selected_move=i.clone();e.move_history.push(i);effects::refresh_enemy_values(&mut c.state).unwrap();}
fn apply(c:&mut Combat,id:&'static str,n:i32){let actor=Actor::Enemy(c.state.enemies[2].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(actor),kind:EffectKind::ApplyFullPower{id:FullPowerId(id),amount:n}}],None).unwrap();}

#[test]
fn m8_inflame_is_a_real_power_producer(){
    for (upgrades,strength) in [(0,2),(1,3)]{
        let mut e=entry(EncounterId::OrbWalker);e.cards=vec![permanent_live_card(CardId::Inflame,CardInstanceId(1),upgrades),permanent_live_card(CardId::Defend,CardInstanceId(2),0)];e.opening_draw=2;
        let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
        assert_eq!((c.state.player.energy,c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount),(2,strength));assert!(c.state.in_play.is_none());
        let public=c.public_full().unwrap();assert_eq!(public.played_powers.len(),1);assert_eq!((public.played_powers[0].instance,public.played_powers[0].id,public.played_powers[0].upgrades),(CardInstanceId(1),CardId::Inflame,upgrades));assert!(public.discard.is_empty()&&public.exhaust.is_empty());
        let frozen=c.state.clone();assert!(c.step_full(&CombatAction::PlayCard{instance:1,target:None}).is_err());assert_eq!(c.state,frozen);c.step_full(&CombatAction::EndTurn).unwrap();assert!(c.public_full().unwrap().hand.iter().all(|c|c.id!=CardId::Inflame));assert!(engine::validate_runtime(&c.state).is_ok());
        let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(owner),kind:EffectKind::Damage(DamagePayload::Fixed(2000))}],None).unwrap();let exit=c.exit().unwrap();assert_eq!(exit.played_powers,vec![permanent_live_card(CardId::Inflame,CardInstanceId(1),upgrades)]);
    }
}

#[test]
fn m8_awakened_last_actor_card_collapse_keeps_a_live_rebirth_decision(){
    let mut e=entry(EncounterId::AwakenedOne);e.cards=vec![permanent_live_card(CardId::Strike,CardInstanceId(1),0)];e.opening_draw=1;
    let mut c=Combat::enter(e).unwrap();hit(&mut c,0,60);hit(&mut c,1,60);hit(&mut c,2,294);
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(3))}).unwrap();
    assert_eq!(c.state.outcome,None);assert_eq!(c.state.enemies[2].life,ActorLife::HalfDead);assert!(c.state.enemies.iter().all(|e|e.hp==0));assert!(c.state.in_play.is_none());
    assert!(c.public_full().unwrap().enemies[2].half_dead);assert!(c.legal_actions_full().unwrap().contains(&CombatAction::EndTurn));assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[2].hp,300);assert!(!c.public_full().unwrap().enemies[2].half_dead);assert_eq!(c.state.enemies[2].intent_id.as_str(),"awa_echo");assert_eq!(c.state.outcome,None);
}

#[test]
fn m8_void_loses_energy_only_on_real_draw_and_exhausts_ethereally(){
    for initial in [0,3,999]{
        let mut e=entry(EncounterId::OrbWalker);e.setup.player.max_energy=initial.max(1);e.setup.player.energy=initial.max(1);let mut c=Combat::enter(e).unwrap();
        // Entry's actual turn-start owner refills energy. Spend the one energy
        // legitimately to exercise drawing at zero, rather than editing it.
        if initial==0{c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();}
        for destination in [CardDestination::Hand,CardDestination::Draw]{c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::CreateCard{id:CardId::Void,destination,upgrades:0}}],None).unwrap();assert_eq!(c.public_full().unwrap().player.energy,initial);}
        c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();assert_eq!(c.state.player.energy,(initial-1).max(0));assert_eq!(c.state.piles.hand.iter().filter(|c|c.full().unwrap().id==CardId::Void).count(),2);
        assert!(!c.legal_actions_full().unwrap().iter().any(|a|matches!(a,CombatAction::PlayCard{instance:2|3,..})));c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.piles.exhaust.iter().filter(|c|c.full().unwrap().id==CardId::Void).count(),2);assert!(c.permanent_gains().is_empty());
    }
}

#[test]
fn m8_awakened_curiosity_and_regenerate(){
    let mut e=entry(EncounterId::AwakenedOne);assert_eq!(e.setup.enemies.iter().map(|e|e.enemy_id).collect::<Vec<_>>(),vec![crate::content::EnemyId::Cultist,crate::content::EnemyId::Cultist,crate::content::EnemyId::AwakenedOne]);e.setup.enemies[2].hp=290;e.cards=vec![permanent_live_card(CardId::Inflame,CardInstanceId(1),0),permanent_live_card(CardId::Defend,CardInstanceId(2),0)];e.opening_draw=2;
    let mut c=Combat::enter(e).unwrap();assert_eq!((c.state.enemies[2].max_hp,amount(&c,2,"curiosity"),amount(&c,2,"regenerate")),(300,1,10));assert_eq!(amount(&c,2,"strength"),0);
    c.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();assert_eq!(amount(&c,2,"strength"),0);
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();assert_eq!((amount(&c,2,"strength"),c.state.enemies[2].intent_damage),(1,Some(21)));c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.enemies[2].hp,c.state.player.hp,amount(&c,2,"regenerate")),(300,284,10));
}

#[test]
fn m8_awakened_rebirth_is_not_victory(){
    let mut e=entry(EncounterId::AwakenedOne);e.cards=vec![permanent_live_card(CardId::TwinStrike,CardInstanceId(1),0)];e.setup.enemies[2].hp=5;
    let mut c=Combat::enter(e).unwrap();let original=c.state.enemies[2].instance_id.clone();let stream=c.state.enemies[2].full_ai_rng.clone();let actor=Actor::Enemy(original.clone());c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(actor),kind:EffectKind::GainBlock(10)}],None).unwrap();hit(&mut c,2,10);assert_eq!((c.state.enemies[2].hp,c.state.enemies[2].life,amount(&c,2,"curiosity")),(5,ActorLife::Active,1));apply(&mut c,"strength",4);apply(&mut c,"weak",2);apply(&mut c,"vulnerable",2);apply(&mut c,"shackled",3);
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(3))}).unwrap();assert_eq!((c.state.enemies[2].hp,c.state.enemies[2].life,c.state.outcome),(0,ActorLife::HalfDead,None));assert!(!c.public_full().unwrap().targets[2].1);assert_eq!(c.state.enemies[2].intent_id.as_str(),"awa_rebirth");assert_eq!(c.state.enemies[2].full_ai_rng,stream);assert!(c.state.in_play.is_none());assert_eq!(c.state.piles.discard.len(),1);
    assert_eq!(amount(&c,2,"strength"),4);for id in ["weak","vulnerable","shackled","curiosity","unawakened"]{assert_eq!(amount(&c,2,id),0);}
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.enemies[2].hp,c.state.enemies[2].instance_id.clone(),c.state.enemies[2].life,c.state.enemies[2].intent_id.as_str()),(300,original,ActorLife::Active,"awa_echo"));assert_eq!(c.state.enemies[2].intent_damage,Some(44));assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m8_void_draw_and_final_cultist_escape(){
    let mut e=entry(EncounterId::AwakenedOne);for id in 2..=6{e.cards.push(permanent_live_card(CardId::Defend,CardInstanceId(id),0));}
    let mut c=Combat::enter(e).unwrap();hit(&mut c,2,300);c.step_full(&CombatAction::EndTurn).unwrap();set(&mut c,"awa_sludge");let stream=c.state.card_rng.clone();c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.piles.draw.first().unwrap().full().unwrap().id,CardId::Void,"Sludge inserts at BOTTOM, not top/random");assert_eq!(c.state.card_rng,stream);assert_eq!(c.state.player.energy,3,"undrawn Void does not drain energy");
    let cultists=c.state.enemies[..2].iter().map(|e|e.hp).collect::<Vec<_>>();hit(&mut c,2,300);assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.enemies[..2].iter().map(|e|e.hp).collect::<Vec<_>>(),cultists);assert!(c.state.enemies[..2].iter().all(|e|e.life==ActorLife::Escaped));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::HalfDeath).count(),1);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),1);
}

#[test]
fn m8_awakened_negative_strength_cleanse_early_cultist_death_and_accepted_healing(){
    let mut c=Combat::enter(entry(EncounterId::AwakenedOne)).unwrap();hit(&mut c,0,60);let dead=c.state.enemies[0].instance_id.clone();apply(&mut c,"strength",-4);c.state.player.hp=200;let owner=Actor::Enemy(c.state.enemies[2].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(owner),kind:EffectKind::Damage(DamagePayload::Fixed(300))},Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Heal(9)}],None).unwrap();assert_eq!((c.state.player.hp,c.state.outcome,amount(&c,2,"strength")),(209,None,0));c.step_full(&CombatAction::EndTurn).unwrap();hit(&mut c,2,300);assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].instance_id.clone()),(0,dead));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),2);
}

#[test]
fn m8_awakened_actual_soul_echo_tackle_and_empty_draw_sludge(){
    let mut c=Combat::enter(entry(EncounterId::AwakenedOne)).unwrap();hit(&mut c,0,60);hit(&mut c,1,60);set(&mut c,"awa_soul");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,276);
    hit(&mut c,2,300);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[2].intent_id.as_str(),"awa_echo");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,236);set(&mut c,"awa_tackle");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,206);set(&mut c,"awa_sludge");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.player.energy),(188,2));assert!(c.public_full().unwrap().hand.iter().any(|c|c.id==CardId::Void));
}

#[test]
fn m8_awakened_unknown_cleanse_retains_all_powers_and_partial_rebirth_retains_owner(){
    let mut c=Combat::enter(entry(EncounterId::AwakenedOne)).unwrap();apply(&mut c,"weak",2);apply(&mut c,"unqualified_power_type",1);let powers=c.state.enemies[2].full_powers.clone();let owner=Actor::Enemy(c.state.enemies[2].instance_id.clone());assert!(c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(owner),kind:EffectKind::Damage(DamagePayload::Fixed(300))}],None).is_err());assert_eq!(c.state.enemies[2].full_powers,powers);assert_eq!(c.state.enemies[2].life,ActorLife::HalfDead);assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::AwakenedCollapse);
    let mut c=Combat::enter(entry(EncounterId::AwakenedOne)).unwrap();hit(&mut c,2,300);let owner=Actor::Enemy(c.state.enemies[2].instance_id.clone());c.state.pending.push_back(Effect{source:Source::None,target:Some(owner),kind:EffectKind::EnemyMove});for _ in 0..8{if c.state.pending.front().is_some_and(|e|e.kind==EffectKind::RevivalHeal(300)){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}assert!(c.state.pending.front().is_some_and(|e|e.kind==EffectKind::RevivalHeal(300)));c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);assert_eq!((c.state.enemies[2].hp,c.state.enemies[2].life,c.state.outcome),(0,ActorLife::Active,None));assert!(matches!(c.state.enemies[2].full_memory,Some(EnemyMemory::AwakenedOne{phase:enemies::AwakenedPhase::Rebirth,..})));let frozen=c.state.clone();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.state,frozen);
}

#[test]
fn m8_awakened_history_tickets_keep_phase_openers_and_restrictions(){
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"awakened".into()]).unwrap();
    for (phase,history,lo,hi,want) in [(enemies::AwakenedPhase::First,vec!["awa_soul"],0,24,"awa_slash"),(enemies::AwakenedPhase::First,vec!["awa_slash";2],25,99,"awa_soul"),(enemies::AwakenedPhase::Second,vec!["awa_sludge";2],0,49,"awa_tackle"),(enemies::AwakenedPhase::Second,vec!["awa_tackle";2],50,99,"awa_sludge")]{
        let (stream,next)=(0..1024).find_map(|cursor|{let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();let(n,next)=rng::uniform_below(&stream,100).unwrap();(lo..=hi).contains(&n).then_some((stream,next))}).unwrap();let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let selected=enemies::select_move(crate::content::EnemyId::AwakenedOne,&history,&EnemyMemory::AwakenedOne{phase,first:false},&stream).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream,next);
    }
}

#[test]
fn m8_curiosity_resolves_after_power_payload_before_its_disposition(){
    let mut e=entry(EncounterId::AwakenedOne);e.cards[0]=permanent_live_card(CardId::Inflame,CardInstanceId(1),0);let mut c=Combat::enter(e).unwrap();
    // Drive the actual prepared producer at individual native dispatch boundaries.
    let prepared=cards::prepare(&c,&CombatAction::PlayCard{instance:1,target:None}).unwrap();let card=c.state.piles.hand.remove(prepared.hand_index);let source=Source::Card(card.handle());c.state.in_play=Some(card);c.state.player.energy=prepared.remaining_energy;c.state.phase=Phase::PlayerAction;
    c.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardPlayed)});c.state.pending.extend(prepared.effects);c.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::FinishCard});c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    engine::dispatch_one(&mut c.state);engine::dispatch_one(&mut c.state);assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,2);assert_eq!(amount(&c,2,"strength"),0);assert!(c.state.in_play.is_some()&&c.state.played_powers.is_empty());
    engine::dispatch_one(&mut c.state);assert_eq!(amount(&c,2,"strength"),1);assert!(c.state.in_play.is_some());engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert!(c.state.in_play.is_none());assert_eq!(c.public_full().unwrap().played_powers[0].id,CardId::Inflame);
}
