use super::super::*;
use crate::content::{CardId,EncounterId,EnemyId};
use super::m5_elites::{amount,hit};
use super::m8_shapes::entry;

fn combat()->Combat{Combat::enter(entry(EncounterId::ThreeDarklings)).unwrap()}
fn set(c:&mut Combat,slot:usize,name:&'static str){
    let actor=&mut c.state.enemies[slot];let intent=ActorIntentId::Full(FullIntentId::from_name(name));actor.intent_id=intent.clone();actor.selected_move=intent.clone();actor.move_history=vec![intent];effects::refresh_enemy_values(&mut c.state).unwrap();
}
fn quiet_allies(c:&mut Combat){for slot in 0..3{if c.state.enemies[slot].hp>0{set(c,slot,"darkling_harden");}}}

#[test]
fn m8_darkling_center_identity_and_fixed_nip(){
    let e=entry(EncounterId::ThreeDarklings);assert_eq!(e.setup.enemies.len(),3);
    for (slot,actor) in e.setup.enemies.iter().enumerate(){
        assert!((48..=56).contains(&actor.hp));assert_eq!(actor.enemy_id,EnemyId::Darkling);
        assert!(matches!(actor.intent.as_str(),"darkling_harden"|"darkling_nip"));
        let nip=e.setup.enemy_damage_bases[&(slot as u32,FullIntentId::from_name("darkling_nip"))];assert!((7..=11).contains(&nip));
    }
    let mut c=Combat::enter(e).unwrap();let nip=c.state.enemies[1].full_damage_entries[&FullIntentId::from_name("darkling_nip")].base;
    quiet_allies(&mut c);set(&mut c,1,"darkling_nip");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,300-nip);
    // Reordering stored actors must not turn the original center into an outer.
    let center=c.state.enemies[1].instance_id.clone();c.state.enemies.swap(0,1);
    for _ in 0..8 {quiet_allies(&mut c);c.step_full(&CombatAction::EndTurn).unwrap();assert_ne!(c.state.enemies.iter().find(|e|e.instance_id==center).unwrap().intent_id.as_str(),"darkling_chomp");}
    assert_eq!(c.state.enemies.iter().find(|e|e.instance_id==center).unwrap().full_damage_entries[&FullIntentId::from_name("darkling_nip")].base,nip);
}

#[test]
fn m8_half_dead_is_untargetable_but_scheduled(){
    let mut c=combat();let id=c.state.enemies[0].instance_id.clone();let max=c.state.enemies[0].max_hp;
    hit(&mut c,0,max);assert_eq!((c.state.enemies[0].life,c.state.outcome),(ActorLife::HalfDead,None));
    assert_eq!(c.state.enemies[0].intent_id.as_str(),"darkling_regrowing");
    let view=c.public_full().unwrap();assert!(view.enemies[0].half_dead);assert!(!view.enemies[0].escaped&&!view.enemies[0].replaced);
    let snapshot=c.snapshot();assert!(c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(id.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(1))}],None).is_err());assert_eq!(c.snapshot(),snapshot);
    quiet_allies(&mut c);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].intent_id.as_str()),(0,"darkling_reincarnate"));
    quiet_allies(&mut c);c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!((&c.state.enemies[0].instance_id,c.state.enemies[0].hp,c.state.enemies[0].max_hp,c.state.enemies[0].life),(&id,max/2,max,ActorLife::Active));
    assert_eq!(amount(&c,0,"life_link"),1);assert!(!c.public_full().unwrap().enemies[0].half_dead);
}

#[test]
fn m8_darklings_require_simultaneous_down(){
    let mut c=combat();
    for slot in 0..2 {let hp=c.state.enemies[slot].hp;hit(&mut c,slot,hp);assert_eq!(c.state.outcome,None);}
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::HalfDeath).count(),2);
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death||e.hook==Hook::CombatSettled).count(),0);
    hit(&mut c,2,56);assert_eq!(c.state.outcome,Some(Outcome::Victory));
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatSettled).count(),1);
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),3);assert!(c.state.enemies.iter().all(|e|e.hp==0&&e.life==ActorLife::Active));c.exit().unwrap();
}

#[test]
fn m8_regrowth_fault_retains_actor(){
    let mut c=combat();let max=c.state.enemies[0].max_hp;let id=c.state.enemies[0].instance_id.clone();let nip=c.state.enemies[0].full_damage_entries[&FullIntentId::from_name("darkling_nip")].base;
    hit(&mut c,0,max);quiet_allies(&mut c);c.step_full(&CombatAction::EndTurn).unwrap();
    let stream=c.state.enemies[0].full_ai_rng.clone();c.state.phase=Phase::EnemyTurn;enemies::act(&mut c.state,&id).unwrap();
    for _ in 0..16 {if c.state.enemies[0].hp>0{break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].life),(max/2,ActorLife::HalfDead));
    c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_some());
    assert_eq!((&c.state.enemies[0].instance_id,c.state.enemies[0].hp,c.state.enemies[0].full_ai_rng.clone()),(&id,max/2,stream));
    assert_eq!(c.state.enemies[0].full_damage_entries[&FullIntentId::from_name("darkling_nip")].base,nip);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m8_darkling_aoe_prunes_only_corpse_hits_and_finalizes_once(){
    let mut e=entry(EncounterId::ThreeDarklings);e.cards[0]=permanent_live_card(CardId::Cleave,CardInstanceId(1),0);
    let mut c=Combat::enter(e).unwrap();for slot in 0..3{let hp=c.state.enemies[slot].hp;hit(&mut c,slot,hp-5);}
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatSettled).count(),1);
    let mut c=combat();let id=c.state.enemies[0].instance_id.clone();
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(id.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(56))},Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(id)),kind:EffectKind::Damage(DamagePayload::Fixed(56))}],None).unwrap();
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::HalfDeath).count(),1);assert_eq!(c.state.outcome,None);
}

#[test]
fn m8_darkling_down_clears_all_powers_and_repeated_revive_keeps_odd_max_hp(){
    let mut e=entry(EncounterId::ThreeDarklings);e.setup.enemies[0].hp=51;e.setup.enemies[0].max_hp=51;
    let mut c=Combat::enter(e).unwrap();let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    for _ in 0..2 {
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(owner.clone()),target:Some(owner.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3}},Effect{source:Source::Actor(Actor::Player),target:Some(owner.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}}],None).unwrap();
        let hp=c.state.enemies[0].hp;hit(&mut c,0,hp+12);assert!(c.state.enemies[0].full_powers.is_empty());
        for _ in 0..2 {quiet_allies(&mut c);c.step_full(&CombatAction::EndTurn).unwrap();}
        assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].max_hp,c.state.enemies[0].life),(25,51,ActorLife::Active));assert_eq!(amount(&c,0,"strength"),0);assert_eq!(amount(&c,0,"weak"),0);
    }
}

#[test]
fn m8_half_dead_ready_state_rejects_unqualified_lifecycle_and_changed_native_identity(){
    let mut c=combat();c.state.enemies[0].life=ActorLife::HalfDead;
    assert!(engine::validate_runtime(&c.state).is_err(),"positive HP without an accepted healing continuation is not a ready HalfDead");
    let mut c=combat();let hp=c.state.enemies[0].hp;hit(&mut c,0,hp);
    c.state.enemies[0].intent_id=ActorIntentId::Full(FullIntentId::from_name("darkling_nip"));c.state.enemies[0].selected_move=c.state.enemies[0].intent_id.clone();c.state.enemies[0].move_history=vec![c.state.enemies[0].intent_id.clone()];
    assert!(engine::validate_runtime(&c.state).is_err(),"HalfDead cannot advertise an ordinary Attack");
    let mut c=combat();let nip=c.state.enemies[0].full_damage_entries[&FullIntentId::from_name("darkling_nip")].base;
    c.state.enemies[0].full_memory=Some(EnemyMemory::Darkling{center:true,nip,first:false});
    assert!(engine::validate_runtime(&c.state).is_err(),"outer actor cannot silently become center");
}

#[test]
fn m8_darkling_native_ai_preserves_center_redirects_and_half_dead_no_draw(){
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"darkling".into()]).unwrap();
    for center in [false,true] {
        let memory=EnemyMemory::Darkling{center,nip:9,first:false};
        for cursor in 0..16 {
            let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();
            for history in [vec!["darkling_chomp"],vec!["darkling_nip","darkling_nip"],vec!["darkling_harden"]] {
                let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let selected=enemies::select_move(EnemyId::Darkling,&history,&memory,&stream).unwrap();
                if center||history.last().unwrap().as_str()=="darkling_chomp"{assert_ne!(selected.intent.as_str(),"darkling_chomp");}
                if history.len()==2 {assert_ne!(selected.intent.as_str(),"darkling_nip");}
                if history.last().unwrap().as_str()=="darkling_harden"{assert_ne!(selected.intent.as_str(),"darkling_harden");}
                assert_eq!(selected.memory,memory);
            }
        }
    }
    let mut c=combat();let hp=c.state.enemies[0].hp;hit(&mut c,0,hp);
    let id=c.state.enemies[0].full_ai_rng.as_ref().unwrap().id().clone();let exhausted=rng::StreamState::new(id,[0;32],1_u128<<64).unwrap();c.state.enemies[0].full_ai_rng=Some(exhausted.clone());
    quiet_allies(&mut c);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].intent_id.as_str(),"darkling_reincarnate");assert_eq!(c.state.enemies[0].full_ai_rng,Some(exhausted));
}
