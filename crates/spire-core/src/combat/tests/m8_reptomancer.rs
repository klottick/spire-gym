use super::super::*;
use crate::content::{CardId,EncounterId,EnemyId};
use super::m8_shapes::entry;
use super::m5_elites::{hit,amount};

fn dagger(hp:i32)->Combat{
    let mut e=entry(EncounterId::OrbWalker);let a=&mut e.setup.enemies[0];a.enemy_id=EnemyId::Dagger;a.hp=hp;a.max_hp=hp;a.powers.clear();a.intent=FullIntentId::from_name("dagger_wound");a.initial_memory=Some(EnemyMemory::Dagger{first:false});
    for instance in 2..=4{e.cards.push(permanent_live_card(CardId::Defend,CardInstanceId(instance),0));}Combat::enter(e).unwrap()
}

#[test]
fn m8_dagger_wound_then_explosion(){
    let mut c=dagger(20);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].hp,c.state.enemies[0].intent_id.as_str()),(291,20,"dagger_explode"));
    assert_eq!(c.state.piles.discard.iter().filter(|c|c.full().unwrap().id==CardId::Wound).count(),1);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].hp,c.state.outcome),(266,0,Some(Outcome::Victory)));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),1);
    let mut dying=dagger(20);dying.step_full(&CombatAction::EndTurn).unwrap();dying.state.player.hp=25;dying.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(dying.state.outcome,Some(Outcome::Death));assert_eq!(dying.state.enemies[0].hp,20,"lethal player hit precedes queued dagger self-loss");
}

#[test]
fn m8_reptomancer_initial_slots_and_live_cap(){
    let mut c=Combat::enter(entry(EncounterId::Reptomancer)).unwrap();assert_eq!(c.state.enemies.len(),3);assert_eq!(c.state.enemies.iter().map(|e|e.enemy_id.clone()).collect::<Vec<_>>(),vec![ActorEnemyId::Full(EnemyId::Dagger),ActorEnemyId::Full(EnemyId::Reptomancer),ActorEnemyId::Full(EnemyId::Dagger)]);
    assert!((180..=190).contains(&c.state.enemies[1].hp));for index in [0,2]{assert!((20..=25).contains(&c.state.enemies[index].hp));assert_eq!(amount(&c,index,"minion"),1);}
    let parent=c.state.enemies[1].instance_id.clone();let hp=c.state.enemies[1].hp;c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies.len(),4);assert_eq!((c.state.enemies[1].hp,c.state.enemies[1].instance_id.clone()),(hp,parent));assert_eq!(c.state.enemies[3].move_history.len(),1,"fresh dagger waits for next enemy snapshot");
    assert_eq!(c.state.player.hp,282,"two existing Dagger9 attacks, no same-turn newborn attack");
    assert_eq!(c.state.enemies[3].enemy_id,ActorEnemyId::Full(EnemyId::Dagger));assert_eq!(amount(&c,3,"minion"),1);
    assert_eq!(c.public_full().unwrap().enemies.iter().map(|e|e.logical_slot).collect::<Vec<_>>(),vec![1,2,4,3]);
    spawn(&mut c);assert_eq!(c.state.enemies.len(),5);assert_eq!(c.public_full().unwrap().enemies[4].logical_slot,0);spawn(&mut c);assert_eq!(c.state.enemies.len(),5,"four tracked live daggers never admits a fifth");
    let old=c.state.enemies[0].instance_id.clone();hit(&mut c,0,30);let survivors=c.state.enemies[2..].to_vec();spawn(&mut c);assert_eq!(c.state.enemies.len(),6);assert_eq!(c.state.enemies[2..5],survivors);assert_ne!(c.state.enemies[5].instance_id,old);assert_eq!(c.public_full().unwrap().enemies[5].logical_slot,1);assert!(!c.public_full().unwrap().targets[0].1);
    assert!(engine::validate_runtime(&c.state).is_ok());
}

fn spawn(c:&mut Combat){let e=&mut c.state.enemies[1];let i=ActorIntentId::Full(FullIntentId::from_name("repto_spawn"));e.intent_id=i.clone();e.selected_move=i.clone();e.move_history.push(i);let actor=Actor::Enemy(e.instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(actor),kind:EffectKind::EnemyMove}],None).unwrap();}

#[test]
fn m8_dagger_spawn_fault_keeps_committed_child(){
    for exhausted in [false,true]{
        let mut c=Combat::enter(entry(EncounterId::Reptomancer)).unwrap();spawn(&mut c);let first=c.state.enemies[3].clone();let parent=c.state.enemies[1].instance_id.clone();
        let actor=Actor::Enemy(parent.clone());c.state.pending.push_back(Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove});
        // Prepare the next source spawn without acknowledging its insertion.
        let i=ActorIntentId::Full(FullIntentId::from_name("repto_spawn"));let e=&mut c.state.enemies[1];e.intent_id=i.clone();e.selected_move=i.clone();e.move_history.push(i);
        for _ in 0..12{if c.state.spawn_progress.is_some(){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
        if exhausted{let progress=c.state.spawn_progress.as_mut().unwrap();let old=&progress.constructor_rng;progress.constructor_rng=rng::StreamState::new(old.id().clone(),*old.key(),1_u128<<64).unwrap();}
        else{engine::dispatch_one(&mut c.state);assert!(c.state.spawn_progress.as_ref().unwrap().prepared_child.is_some());assert!(c.state.spawn_progress.as_ref().unwrap().rolled_hp.is_some());c.state.resolved_effects=99_999;}
        engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,if exhausted{FaultKind::RngExhausted}else{FaultKind::EffectGuard});assert_eq!(c.state.enemies[3],first);assert!(c.state.spawn_progress.is_some());assert_eq!(c.state.spawn_ordinals[&parent],2);assert_eq!(c.state.enemies.len(),4);let frozen=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),frozen);
    }
}

#[test]
fn m8_reptomancer_cleanup_not_global_minion_ignore(){
    let mut c=Combat::enter(entry(EncounterId::Reptomancer)).unwrap();let parent=Actor::Enemy(c.state.enemies[1].instance_id.clone());let deaths=c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count();
    c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(parent),kind:EffectKind::Damage(DamagePayload::Fixed(200))});engine::dispatch_one(&mut c.state);assert!(c.state.enemies[0].hp>0&&c.state.enemies[2].hp>0);assert!(!enemies::act2::combat_victory_ready(&c.state));assert_eq!(c.state.outcome,None);
    engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.outcome,Some(Outcome::Victory));assert!(c.state.enemies.iter().all(|e|e.hp==0));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),deaths+3);
    let actual=c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).map(|e|e.target.clone()).collect::<Vec<_>>();assert_eq!(actual,vec![Some(Actor::Enemy(c.state.enemies[1].instance_id.clone())),Some(Actor::Enemy(c.state.enemies[2].instance_id.clone())),Some(Actor::Enemy(c.state.enemies[0].instance_id.clone()))]);
    let mut standalone=dagger(20);standalone.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("minion"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});assert!(!enemies::act2::combat_victory_ready(&standalone.state));
}

#[test]
fn m8_prepared_dagger_inserts_after_parent_top_cleanup_and_keeps_combat_live(){
    let mut c=Combat::enter(entry(EncounterId::Reptomancer)).unwrap();let parent=c.state.enemies[1].instance_id.clone();let actor=Actor::Enemy(parent.clone());c.state.pending.push_back(Effect{source:Source::None,target:Some(actor.clone()),kind:EffectKind::EnemyMove});
    for _ in 0..12{if c.state.spawn_progress.as_ref().is_some_and(|p|p.prepared_child.is_some()){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    let prepared=c.state.spawn_progress.as_ref().unwrap().prepared_child.as_ref().unwrap().instance_id.clone();assert_eq!(c.state.enemies.len(),3);
    c.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(actor),kind:EffectKind::Damage(DamagePayload::Fixed(200))});engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.outcome,None);assert_eq!(c.state.enemies.len(),4);assert_eq!(c.state.enemies[3].instance_id,prepared);assert!(c.state.enemies[..3].iter().all(|e|e.hp==0));assert!(c.state.spawn_progress.is_none());assert_eq!(c.state.enemies[3].move_history.len(),1);assert_eq!(c.state.player.hp,300);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,291);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.outcome),(266,Some(Outcome::Victory)));
}

#[test]
fn m8_reptomancer_real_snake_bite_and_recursive_redirects(){
    for (name,loss,weak) in [("repto_snake",26,1),("repto_bite",30,0)]{
        let mut c=Combat::enter(entry(EncounterId::Reptomancer)).unwrap();let parent=&mut c.state.enemies[1];let i=ActorIntentId::Full(FullIntentId::from_name(name));parent.intent_id=i.clone();parent.selected_move=i.clone();parent.move_history.push(i);effects::refresh_enemy_values(&mut c.state).unwrap();let actor=Actor::Enemy(c.state.enemies[1].instance_id.clone());
        c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(actor),kind:EffectKind::EnemyMove}],None).unwrap();assert_eq!((c.state.player.hp,c.state.player_full_powers.iter().find(|p|p.id.0=="weak").map_or(0,|p|p.state.amount)),(300-loss,weak));assert!(engine::validate_runtime(&c.state).is_ok());
    }
    let owner=EnemyHandle::parse("repto-ai").unwrap();let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),owner.as_str().into()]).unwrap();let context=enemies::act2::EnemyRollContext{owner:owner.clone(),turn:2,actors:vec![enemies::act2::ActorSummary{owner,species:EnemyId::Reptomancer,hp:180,max_hp:180,slot:2}]};
    for (history,first_lo,first_hi,retry_bound,retry_offset,lo,hi,want) in [("repto_snake",0,32,67,33,33,65,"repto_spawn"),("repto_snake",0,32,67,33,66,99,"repto_bite"),("repto_bite",66,99,66,0,0,32,"repto_snake"),("repto_bite",66,99,66,0,33,65,"repto_spawn")]{
        let (stream,after)=(0..4096).find_map(|cursor|{let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();let(n,next)=rng::uniform_below(&stream,100).unwrap();if !(first_lo..=first_hi).contains(&n){return None}let(n,after)=rng::uniform_below(&next,retry_bound).unwrap();let n=n+retry_offset;(lo..=hi).contains(&n).then_some((stream,after))}).unwrap();
        let selected=enemies::act2::select_move(EnemyId::Reptomancer,&[FullIntentId::from_name(history)],&EnemyMemory::Reptomancer{first:false,slots:Default::default()},&stream,&context).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream,after);
    }
}

#[test]
fn m8_reptomancer_custody_admission_and_invariant_fail_closed(){
    let mut e=entry(EncounterId::Reptomancer);if let Some(EnemyMemory::Reptomancer{slots,..})=&mut e.setup.enemies[1].initial_memory{slots.insert(2,EnemyHandle::parse("foreign-child").unwrap());}
    let mut items=Some(crate::items::CombatItemContext::new(vec![crate::content::RelicId::Anchor],vec![]));let frozen=items.clone();assert!(matches!(Combat::enter_with_item_context(e,&mut items),Err(CombatEntryFailure::Rejected(_))));assert_eq!(items,frozen);
    let mut c=Combat::enter(entry(EncounterId::Reptomancer)).unwrap();let child=c.state.enemies[0].instance_id.clone();if let Some(EnemyMemory::Reptomancer{slots,..})=&mut c.state.enemies[1].full_memory{slots.insert(4,child);}assert!(engine::validate_runtime(&c.state).is_err());
}

#[test]
fn m8_new_leftmost_dagger_acts_before_older_exploding_left_dagger(){
    let mut c=Combat::enter(entry(EncounterId::Reptomancer)).unwrap();spawn(&mut c);spawn(&mut c);assert_eq!(c.public_full().unwrap().enemies[4].logical_slot,0);
    let i=ActorIntentId::Full(FullIntentId::from_name("dagger_explode"));let old=&mut c.state.enemies[0];old.intent_id=i.clone();old.selected_move=i.clone();old.move_history.push(i);effects::refresh_enemy_values(&mut c.state).unwrap();
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::EndPlayerTurn});
    for _ in 0..24{if c.state.player.hp<300{break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    assert_eq!(c.state.player.hp,291,"new trackedslot3/physical0 Wound9, not stored-index0 Explode25");engine::resolve(&mut c.state);assert!(c.state.fault.is_none());
}

#[test]
fn m8_reptomancer_top_cleanup_kills_foreign_group_actor_without_losing_earned_heal(){
    let mut c=Combat::enter(entry(EncounterId::Reptomancer)).unwrap();c.state.player.hp=250;
    let mut foreign=Combat::enter(entry(EncounterId::OrbWalker)).unwrap().state.enemies.remove(0);foreign.instance_id=EnemyHandle::parse("foreign-ordinary").unwrap();let scope=c.state.enemies[1].full_ai_rng.as_ref().unwrap().id().scope()[0].clone();foreign.full_ai_rng=Some(rng::StreamState::new(rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec![scope,"foreign-ordinary".into()]).unwrap(),[19;32],0).unwrap());assert!(!foreign.full_powers.iter().any(|p|p.id.0=="minion"));c.state.actor_slots.insert(foreign.instance_id.clone(),5);c.state.enemies.push(foreign);
    let parent=Actor::Enemy(c.state.enemies[1].instance_id.clone());let mut work=vec![Effect{source:Source::Actor(Actor::Player),target:Some(parent),kind:EffectKind::Damage(DamagePayload::Fixed(200))},Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Heal(9)}];for index in [0,2,3]{work.push(Effect{source:Source::None,target:Some(Actor::Enemy(c.state.enemies[index].instance_id.clone())),kind:EffectKind::EnemyMove});}
    c.enqueue_native_effects(work,None).unwrap();assert_eq!((c.state.outcome,c.state.player.hp),(Some(Outcome::Victory),259));assert!(c.state.enemies.iter().all(|e|e.hp==0));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),4);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatSettled).count(),1);
}

#[test]
fn m8_reptomancer_history_and_foreign_live_count_redirects(){
    let owner=EnemyHandle::parse("repto-fixture").unwrap();let stream_id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),owner.as_str().into()]).unwrap();
    let find=|lo,hi|(0..1024).find_map(|index|{let stream=rng::StreamState::new(stream_id.clone(),[0;32],index).unwrap();let (n,next)=rng::uniform_below(&stream,100).unwrap();(n>=lo&&n<=hi).then_some((stream,next))}).unwrap();
    for (others,history,want) in [(3,vec!["repto_bite"],"repto_spawn"),(4,vec!["repto_bite"],"repto_snake"),(0,vec!["repto_spawn";2],"repto_snake")]{
        let mut actors=vec![enemies::act2::ActorSummary{owner:owner.clone(),species:EnemyId::Reptomancer,hp:180,max_hp:180,slot:4}];for slot in 0..others{actors.push(enemies::act2::ActorSummary{owner:EnemyHandle::parse(&format!("foreign-{slot}")).unwrap(),species:EnemyId::Cultist,hp:48,max_hp:48,slot});}
        let context=enemies::act2::EnemyRollContext{owner:owner.clone(),turn:2,actors};let (stream,next)=find(33,65);let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();
        let selected=enemies::act2::select_move(EnemyId::Reptomancer,&history,&EnemyMemory::Reptomancer{first:false,slots:Default::default()},&stream,&context).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream,next);
    }
}
