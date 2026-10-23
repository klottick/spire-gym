use super::super::*;
use crate::content::{EncounterId,EnemyId};
use super::m5_elites::{hit,amount};

fn entry()->CombatEntry{super::m6_ordinary::entry(EncounterId::GremlinLeader)}
fn roll(c:&mut Combat,slot:usize,cursor:u128){
    let old=c.state.enemies[slot].full_ai_rng.as_ref().unwrap();c.state.enemies[slot].full_ai_rng=Some(rng::StreamState::new(old.id().clone(),[0;32],cursor).unwrap());
    let actor=Actor::Enemy(c.state.enemies[slot].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RollMove}],None).unwrap();
}

#[test]
fn m6_rally_keeps_parent_and_spawns_fresh_native_children() {
    let mut c=Combat::enter(entry()).unwrap();assert_eq!(c.state.enemies.len(),3);assert!((140..=148).contains(&c.state.enemies[2].hp));
    assert_eq!((amount(&c,0,"minion"),amount(&c,1,"minion"),amount(&c,2,"minion")),(1,1,0));
    hit(&mut c,0,30);hit(&mut c,1,30);let parent=c.state.enemies[2].clone();roll(&mut c,2,0);assert_eq!(c.state.enemies[2].intent_id.as_str(),"leader_rally");
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,80);assert_eq!(c.state.enemies.len(),5);
    assert_eq!((c.state.enemies[2].hp,c.state.enemies[2].life,c.state.enemies[2].full_powers.clone()),(parent.hp,ActorLife::Active,parent.full_powers));
    for child in &c.state.enemies[3..]{
        let range=match child.enemy_id.as_str(){"gremlin_mad"=>20..=24,"gremlin_sneaky"=>10..=14,"gremlin_fat"=>13..=17,"gremlin_shield"=>12..=15,"gremlin_wizard"=>21..=25,_=>panic!("Rally must construct an actual small Gremlin")};
        assert!(range.contains(&child.max_hp));assert_eq!(child.hp,child.max_hp);assert_eq!(child.move_history.len(),1);assert!(child.full_powers.iter().any(|p|p.id.0=="minion"));
        assert_ne!(child.full_ai_rng.as_ref().unwrap().id(),parent.full_ai_rng.as_ref().unwrap().id());
    }
}

#[test]
fn m6_respawn_never_retargets_old_handles_or_exhausts_27_history_slots() {
    let mut c=Combat::enter(entry()).unwrap();let mut current=vec![0,1];
    for _ in 0..14 {
        for slot in current{hit(&mut c,slot,30)}roll(&mut c,2,0);c.step_full(&CombatAction::EndTurn).unwrap();
        current=vec![c.state.enemies.len()-2,c.state.enemies.len()-1];
        assert_eq!(c.state.player.hp,80);assert_eq!(c.state.enemies.iter().filter(|e|e.hp>0).count(),3);
    }
    assert_eq!(c.state.enemies.len(),31);let view=c.public_full().unwrap();assert!(!view.targets.iter().find(|(id,_)|*id==TargetId(1)).unwrap().1);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).is_err());assert_eq!(c.snapshot(),before);
    let handles=c.state.enemies.iter().map(|e|e.instance_id.clone()).collect::<std::collections::BTreeSet<_>>();assert_eq!(handles.len(),31);
}

#[test]
fn m6_new_summons_wait_for_next_turn_snapshot() {
    let mut c=Combat::enter(entry()).unwrap();hit(&mut c,0,30);hit(&mut c,1,30);roll(&mut c,2,0);c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.player.hp,80);let child_histories=c.state.enemies[3..].iter().map(|e|e.move_history.len()).collect::<Vec<_>>();assert_eq!(child_histories,vec![1,1]);
    c.step_full(&CombatAction::EndTurn).unwrap();assert!(c.state.enemies[3..].iter().all(|e|e.move_history.len()==2),"initial intents execute on the following captured turn only");
}

#[test]
fn m6_rally_fills_two_missing_slots_without_resetting_a_surviving_ally() {
    let mut c=Combat::enter(entry()).unwrap();hit(&mut c,0,30);hit(&mut c,1,1);
    let ally=Actor::Enemy(c.state.enemies[1].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(ally),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:2}}],None).unwrap();
    let surviving=c.state.enemies[1].clone();roll(&mut c,2,0);assert_eq!(c.state.enemies[2].intent_id.as_str(),"leader_rally");
    let leader=Actor::Enemy(c.state.enemies[2].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(leader.clone()),target:Some(leader),kind:EffectKind::EnemyMove}],None).unwrap();
    assert_eq!(c.state.enemies[1],surviving);assert_eq!(c.state.enemies.len(),5);assert_eq!(c.state.enemies.iter().filter(|e|e.hp>0).count(),4);
    assert_eq!(c.public_full().unwrap().enemies[3..].iter().map(|e|e.logical_slot).collect::<Vec<_>>(),vec![1,0]);
}

#[test]
fn m6_leader_death_escapes_living_adds_without_fabricated_deaths() {
    let mut c=Combat::enter(entry()).unwrap();let living=c.state.enemies[..2].iter().map(|e|e.hp).collect::<Vec<_>>();
    let deaths=c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count();hit(&mut c,2,200);
    assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.enemies[..2].iter().map(|e|e.hp).collect::<Vec<_>>(),living);
    assert!(c.state.enemies[..2].iter().all(|e|e.life!=ActorLife::Active));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),deaths+1);
    assert!(c.public_full().unwrap().targets.iter().all(|(_,active)|!*active));
}

#[test]
fn m6_twin_strike_lethal_first_hit_prunes_only_dead_add_not_combat_or_card_finish(){
    let mut e=entry();e.cards[0]=permanent_live_card(crate::content::CardId::TwinStrike,CardInstanceId(1),0);e.setup.enemies[0].hp=5;
    let mut c=Combat::enter(e).unwrap();let others=c.state.enemies[1..].iter().map(|e|e.hp).collect::<Vec<_>>();let damage=c.state.hook_events.iter().filter(|e|e.hook==Hook::Damage).count();
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();assert_eq!((c.state.enemies[0].hp,c.state.outcome,c.state.player.energy),(0,None,2));assert_eq!(c.state.enemies[1..].iter().map(|e|e.hp).collect::<Vec<_>>(),others);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Damage).count(),damage+1);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count(),1);assert!(c.state.in_play.is_none());assert_eq!(c.state.piles.discard.len(),1);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).is_err());assert_eq!(c.snapshot(),before);
}

fn small(id:EnemyId,intent:&'static str)->Combat {
    let mut e=super::m5_elites::entry(EncounterId::Cultist);let actor=&mut e.setup.enemies[0];actor.enemy_id=id;actor.hp=24;actor.max_hp=24;actor.intent=FullIntentId::from_name(intent);actor.initial_memory=None;actor.powers.clear();
    if id==EnemyId::GremlinMad{actor.powers.push(FullPowerEntry{id:FullPowerId("angry"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}})}
    e.setup.player.hp=160;e.setup.player.max_hp=160;Combat::enter(e).unwrap()
}

#[test]
fn m6_small_gremlins_execute_distinct_attack_charge_and_protect_cycles() {
    for (id,intent,loss,weak) in [(EnemyId::GremlinMad,"gremlin_scratch",4,0),(EnemyId::GremlinSneaky,"gremlin_puncture",9,0),(EnemyId::GremlinFat,"gremlin_smash",4,1)] {
        let mut c=small(id,intent);let before=c.state.enemies[0].full_ai_rng.clone();c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,160-loss);assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="weak").map_or(0,|p|p.state.amount),weak);assert_eq!(c.state.enemies[0].full_ai_rng,before);
    }
    let mut c=small(EnemyId::GremlinWizard,"gremlin_charge");
    for (hp,next) in [(160,"gremlin_charge"),(160,"gremlin_blast"),(135,"gremlin_charge"),(135,"gremlin_charge"),(135,"gremlin_charge"),(135,"gremlin_blast"),(110,"gremlin_charge")] {
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].intent_id.as_str()),(hp,next));
    }
    let mut c=small(EnemyId::GremlinShield,"gremlin_protect");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].block,c.state.enemies[0].intent_id.as_str()),(160,7,"gremlin_bash"));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,154);
}

#[test]
fn m6_angry_uses_positive_normal_hp_damage_not_attack_play_or_thorns() {
    let mut c=small(EnemyId::GremlinMad,"gremlin_scratch");let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(actor.clone()),kind:EffectKind::GainBlock(4)}],None).unwrap();hit(&mut c,0,4);assert_eq!(amount(&c,0,"strength"),0);
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(actor.clone()),kind:EffectKind::Damage(DamagePayload::Thorns(2))}],None).unwrap();assert_eq!(amount(&c,0,"strength"),0);
    hit(&mut c,0,2);hit(&mut c,0,2);assert_eq!(amount(&c,0,"strength"),2);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,154);
}

#[test]
fn m6_leader_encourage_and_stab_use_native_per_hit_strength_and_other_only_block() {
    let mut c=Combat::enter(entry()).unwrap();roll(&mut c,2,0);assert_eq!(c.state.enemies[2].intent_id.as_str(),"leader_encourage");
    let actor=Actor::Enemy(c.state.enemies[2].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove}],None).unwrap();
    assert_eq!((amount(&c,0,"strength"),amount(&c,1,"strength"),amount(&c,2,"strength")),(3,3,3));assert_eq!((c.state.enemies[0].block,c.state.enemies[1].block,c.state.enemies[2].block),(6,6,0));
    assert_eq!(c.state.enemies[2].intent_id.as_str(),"leader_stab");let before=c.state.player.hp;let actor=Actor::Enemy(c.state.enemies[2].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove}],None).unwrap();assert_eq!(c.state.player.hp,before-27);
}

#[test]
fn m6_leader_ai_obeys_live_count_buckets_redirects_and_inclusive_reroll() {
    let owner=EnemyHandle::parse("leader").unwrap();let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"leader".into()]).unwrap();
    for (count,last,cursor,want,draws) in [(0,"leader_encourage",271,"leader_rally",1),(0,"leader_encourage",16,"leader_stab",1),(0,"leader_rally",0,"leader_stab",1),(0,"leader_stab",33,"leader_rally",1),
        (1,"leader_stab",138,"leader_rally",1),(1,"leader_rally",66,"leader_encourage",1),(1,"leader_rally",1,"leader_encourage",1),(1,"leader_rally",82,"leader_stab",1),(1,"leader_encourage",66,"leader_stab",1),
        (1,"leader_rally",0,"leader_encourage",2),(1,"leader_rally",23,"leader_stab",2),(1,"leader_stab",320,"leader_rally",3),
        (2,"leader_stab",19,"leader_encourage",1),(2,"leader_encourage",245,"leader_stab",1),(2,"leader_encourage",0,"leader_stab",1),(2,"leader_stab",33,"leader_encourage",1),
        (3,"leader_stab",0,"leader_encourage",1),(3,"leader_encourage",33,"leader_stab",1)] {
        let mut actors=vec![enemies::act2::ActorSummary{owner:owner.clone(),species:EnemyId::GremlinLeader,hp:140,max_hp:140,slot:3}];
        for slot in 0..count{actors.push(enemies::act2::ActorSummary{owner:EnemyHandle::parse(&format!("add-{slot}")).unwrap(),species:EnemyId::GremlinMad,hp:20,max_hp:20,slot})}
        let context=enemies::act2::EnemyRollContext{owner:owner.clone(),turn:2,actors};let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();
        let selected=enemies::act2::select_move(EnemyId::GremlinLeader,&[FullIntentId::from_name(last)],&EnemyMemory::GremlinLeader,&stream,&context).unwrap();assert_eq!(selected.intent.as_str(),want,"count{count}/{last}/cursor{cursor}");assert_eq!(selected.stream.word_index(),cursor+draws);
    }
}

#[test]
fn m6_rally_samples_the_weighted_pool_and_a_full_formation_draws_nothing() {
    let mut base=Combat::enter(entry()).unwrap();hit(&mut base,0,30);hit(&mut base,1,30);
    for (cursor,species) in [(14,EnemyId::GremlinMad),(13,EnemyId::GremlinMad),(0,EnemyId::GremlinSneaky),(6,EnemyId::GremlinSneaky),(9,EnemyId::GremlinFat),(3,EnemyId::GremlinFat),(4,EnemyId::GremlinShield),(1,EnemyId::GremlinWizard)] {
        let mut c=base.clone();let old=c.state.enemies[2].full_ai_rng.as_ref().unwrap();c.state.enemies[2].full_ai_rng=Some(rng::StreamState::new(old.id().clone(),[0;32],cursor).unwrap());
        let actor=Actor::Enemy(c.state.enemies[2].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RallyGremlins{remaining:1}}],None).unwrap();
        assert_eq!(c.state.enemies[3].enemy_id,ActorEnemyId::Full(species));assert_eq!(c.state.enemies[2].full_ai_rng.as_ref().unwrap().word_index(),cursor+1);
    }
    let mut c=Combat::enter(entry()).unwrap();let summoner=c.state.enemies[2].instance_id.clone();c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(summoner.clone())),target:None,kind:EffectKind::SpawnEnemy(SpawnRequest{summoner:summoner.clone(),slot:0,species:EnemyId::GremlinFat})}],None).unwrap();
    let before=(c.state.enemies.clone(),c.state.spawn_ordinals.clone());let actor=Actor::Enemy(summoner);c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RallyGremlins{remaining:2}}],None).unwrap();assert_eq!((c.state.enemies.clone(),c.state.spawn_ordinals.clone()),before);
}

#[test]
fn m6_append_only_third_slot_acts_leftmost_on_the_next_turn() {
    let mut e=entry();for (slot,id,hp,intent) in [(0,EnemyId::GremlinSneaky,12,"gremlin_puncture"),(1,EnemyId::GremlinWizard,23,"gremlin_charge")] {
        let a=&mut e.setup.enemies[slot];a.enemy_id=id;a.hp=hp;a.max_hp=hp;a.initial_memory=None;a.intent=FullIntentId::from_name(intent);a.powers.retain(|p|p.id.0=="minion");
    }
    let mut c=Combat::enter(e).unwrap();roll(&mut c,2,0);let summoner=c.state.enemies[2].instance_id.clone();
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(summoner.clone())),target:None,kind:EffectKind::SpawnEnemy(SpawnRequest{summoner,slot:0,species:EnemyId::GremlinFat})}],None).unwrap();
    assert_eq!((c.public_full().unwrap().enemies[3].logical_slot,c.state.enemies[3].move_history.len()),(0,1));
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::EndPlayerTurn});
    for _ in 0..30{if c.state.player.hp<80{break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    assert_eq!(c.state.player.hp,76,"new leftmost Fat4 must act before old Sneaky9, despite its later stored target index");
    engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.player.hp,67);
}

#[test]
fn m6_shield_targets_at_resolution_but_keeps_the_captured_next_protect() {
    let mut e=entry();e.setup.enemies.truncate(2);
    for (slot,id,intent) in [(0,EnemyId::GremlinShield,"gremlin_protect"),(1,EnemyId::GremlinWizard,"gremlin_charge")] {
        let a=&mut e.setup.enemies[slot];a.enemy_id=id;a.initial_memory=None;a.intent=FullIntentId::from_name(intent);a.powers.clear();
    }
    let mut c=Combat::enter(e).unwrap();let shield=Actor::Enemy(c.state.enemies[0].instance_id.clone());let wizard=Actor::Enemy(c.state.enemies[1].instance_id.clone());
    c.state.pending.push_front(Effect{source:Source::Actor(shield.clone()),target:Some(shield),kind:EffectKind::EnemyMove});engine::dispatch_one(&mut c.state);
    c.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(wizard),kind:EffectKind::Damage(DamagePayload::Fixed(30))});engine::resolve(&mut c.state);assert!(c.state.fault.is_none());
    assert_eq!((c.state.enemies[0].block,c.state.enemies[0].intent_id.as_str(),c.state.player.hp),(7,"gremlin_protect",80));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].intent_id.as_str(),"gremlin_bash");
}

#[test]
fn m6_partial_second_spawn_keeps_first_child_ordinal_and_owned_rng_without_retry() {
    for exhausted_rng in [false,true] {
        let mut c=Combat::enter(entry()).unwrap();hit(&mut c,0,30);hit(&mut c,1,30);let summoner=c.state.enemies[2].instance_id.clone();
        c.state.pending.push_back(Effect{source:Source::Actor(Actor::Enemy(summoner.clone())),target:Some(Actor::Enemy(summoner.clone())),kind:EffectKind::RallyGremlins{remaining:2}});
        for _ in 0..12 {
            if c.state.enemies.len()==4&&c.state.pending.front().is_some_and(|e|e.kind==EffectKind::CompleteSpawn){break}
            engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());
        }
        assert_eq!(c.state.spawn_progress.as_ref().unwrap().ordinal,1);assert_eq!(c.state.enemies.len(),4);assert_eq!(c.state.spawn_ordinals[&summoner],2);
        let prior=(c.state.enemies[2].clone(),c.state.enemies[3].clone(),c.state.actor_slots.clone());
        if exhausted_rng {
            let stream=&c.state.spawn_progress.as_ref().unwrap().constructor_rng;c.state.spawn_progress.as_mut().unwrap().constructor_rng=rng::StreamState::new(stream.id().clone(),*stream.key(),1_u128<<64).unwrap();
        }else{c.state.resolved_effects=99_999;}
        engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,if exhausted_rng{FaultKind::RngExhausted}else{FaultKind::EffectGuard});
        assert_eq!((c.state.enemies[2].clone(),c.state.enemies[3].clone(),c.state.actor_slots.clone()),prior);assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::CompleteSpawn);
        let progress=c.state.spawn_progress.as_ref().unwrap();assert_eq!(progress.request.summoner,summoner);assert!(progress.prepared_child.is_none());assert_eq!(progress.rolled_hp,None);
        let before=c.snapshot();let error=c.step_full(&CombatAction::EndTurn).unwrap_err();assert_eq!(c.snapshot(),before);
        let crate::run::RunError::Faulted(fault)=error else{panic!("partial summon must retain its native fault")};
        let crate::run::RunFaultKind::Combat(diagnostic)=fault.kind else{panic!("expected native combat diagnostic")};
        let progress=diagnostic.spawn_progress.unwrap();assert_eq!((progress.request.summoner,progress.ordinal),(summoner,1));assert_eq!(progress.prepared_child,None);if exhausted_rng{assert_eq!(progress.constructor_cursor,1_u128<<64);}
    }
}

#[test]
fn m6_invalid_leader_formation_rejects_before_inventory_or_hooks() {
    let mut e=entry();
    for index in 0..2{let mut extra=e.setup.enemies[0].clone();extra.instance_id=EnemyHandle::parse(&format!("excess-gremlin-{index}")).unwrap();let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec![extra.ai_rng.id().scope()[0].clone(),extra.instance_id.as_str().into()]).unwrap();extra.ai_rng=rng::StreamState::new(id,*extra.ai_rng.key(),0).unwrap();e.setup.enemies.push(extra);}
    let mut items=Some(crate::items::CombatItemContext::new(vec![crate::content::RelicId::Anchor],vec![]));let before=items.clone();
    assert!(matches!(Combat::enter_with_item_context(e,&mut items),Err(CombatEntryFailure::Rejected(_))));assert_eq!(items,before);
}
