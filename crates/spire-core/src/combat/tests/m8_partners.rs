use super::super::*;
use crate::content::EncounterId;
use super::m8_shapes::entry;
use super::m5_elites::{amount,hit};

#[test]
fn m8_deca_donu_order_and_independent_cycles(){
    let mut c=Combat::enter(entry(EncounterId::DonuAndDeca)).unwrap();
    assert_eq!(c.state.enemies.iter().map(|e|(e.enemy_id.as_str(),e.hp)).collect::<Vec<_>>(),[("deca",250),("donu",250)]);
    assert_eq!(c.state.enemies.iter().map(|e|e.intent_id.as_str()).collect::<Vec<_>>(),["deca_beam","donu_circle"]);for i in 0..2{assert_eq!(amount(&c,i,"artifact"),2);}
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,280);for i in 0..2{assert_eq!(amount(&c,i,"strength"),3);}assert_eq!(c.state.piles.all(c.state.in_play.as_ref()).filter(|c|c.full().unwrap().id==crate::content::CardId::Dazed).count(),2);
    assert_eq!(c.state.enemies.iter().map(|e|e.intent_id.as_str()).collect::<Vec<_>>(),["deca_square","donu_beam"]);assert_eq!(c.state.enemies[1].intent_damage,Some(13));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,254);assert_eq!(c.state.enemies.iter().map(|e|e.block).collect::<Vec<_>>(),[16,16]);assert_eq!(c.state.enemies[0].intent_damage,Some(13));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,228);for i in 0..2{assert_eq!(amount(&c,i,"strength"),6);}assert_eq!(c.state.enemies[1].intent_damage,Some(16));assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m8_pair_artifact_and_either_death_first(){
    for first in 0..2{
        let mut c=Combat::enter(entry(EncounterId::DonuAndDeca)).unwrap();let owner=Actor::Enemy(c.state.enemies[1-first].instance_id.clone());
        for left in [1,0]{c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(owner.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:1}}],None).unwrap();assert_eq!(amount(&c,1-first,"artifact"),left);assert_eq!(amount(&c,1-first,"weak"),0);}
        hit(&mut c,first,250);assert_eq!(c.state.outcome,None);assert_eq!(c.state.enemies[first].hp,0);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,if first==0{300}else{280});assert_eq!(amount(&c,first,"strength"),0);
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,if first==0{274}else{280});c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,if first==0{274}else{260});assert_eq!(amount(&c,1-first,"strength"),if first==0{6}else{0});
        hit(&mut c,1-first,999);assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatSettled).count(),1);let frozen=c.state.clone();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.state,frozen);
    }
}

#[test]
fn m8_pair_selection_does_not_advance_unexecuted_own_cycle(){
    let mut c=Combat::enter(entry(EncounterId::DonuAndDeca)).unwrap();
    for i in 0..2{let start=c.state.enemies[i].intent_id.clone();let actor=Actor::Enemy(c.state.enemies[i].instance_id.clone());for _ in 0..2{c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor.clone()),kind:EffectKind::RollMove}],None).unwrap();assert_eq!(c.state.enemies[i].intent_id,start);}}
}

#[test]
fn m8_pair_foreign_intent_rejected_before_item_custody_transfer(){
    for i in 0..2{let mut e=entry(EncounterId::DonuAndDeca);e.setup.enemies[i].intent=FullIntentId::from_name(if i==0{"donu_circle"}else{"deca_square"});let mut items=Some(crate::items::CombatItemContext::new(vec![crate::content::RelicId::Anchor],vec![]));assert!(Combat::enter_with_item_context(e,&mut items).is_err());assert!(items.is_some());}
}

#[test]
fn m8_pair_published_cycle_and_remaining_targets_survive_fault(){
    let mut c=Combat::enter(entry(EncounterId::DonuAndDeca)).unwrap();let owner=c.state.enemies[1].instance_id.clone();c.state.phase=Phase::EnemyTurn;c.state.pending.push_back(Effect{source:Source::None,target:Some(Actor::Enemy(owner)),kind:EffectKind::EnemyMove});c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});engine::dispatch_one(&mut c.state);engine::dispatch_one(&mut c.state);assert_eq!(amount(&c,0,"strength"),3);assert_eq!(amount(&c,1,"strength"),0);assert!(matches!(c.state.enemies[1].full_memory,Some(EnemyMemory::Donu{attacking:true})));
    c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);assert_eq!(c.state.current.as_ref().unwrap().target,Some(Actor::Enemy(c.state.enemies[1].instance_id.clone())));assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3});assert_eq!(amount(&c,0,"strength"),3);assert_eq!(c.state.outcome,None);let frozen=c.state.clone();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.state,frozen);
}

#[test]
fn m8_pair_buffs_target_live_actors(){
    for producer in 0..2{for killed in 0..2{
        let mut c=Combat::enter(entry(EncounterId::DonuAndDeca)).unwrap();if producer==0{c.step_full(&CombatAction::EndTurn).unwrap();}
        let owner=c.state.enemies[producer].instance_id.clone();c.state.phase=Phase::EnemyTurn;c.state.pending.push_back(Effect{source:Source::None,target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::EnemyMove});c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());
        let buffs=c.state.pending.iter().filter(|e|matches!(e.kind,EffectKind::GainBlock(16)|EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3})).map(|e|e.target.clone()).collect::<Vec<_>>();assert_eq!(buffs,vec![Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),Some(Actor::Enemy(c.state.enemies[1].instance_id.clone()))]);
        // Real lethal damage AFTER publication must neither buff the corpse nor
        // cancel the surviving target's earned action if its source was killed.
        c.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(c.state.enemies[killed].instance_id.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(250))});engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.outcome,None);assert_eq!(c.state.enemies[killed].hp,0);
        if producer==0{assert_eq!(c.state.enemies[killed].block,0);assert_eq!(c.state.enemies[1-killed].block,16);}else{assert_eq!(amount(&c,killed,"strength"),0);assert_eq!(amount(&c,1-killed,"strength"),3);}
    }}
    for producer in 0..2{
        let mut c=Combat::enter(entry(EncounterId::DonuAndDeca)).unwrap();if producer==0{c.step_full(&CombatAction::EndTurn).unwrap();}hit(&mut c,1-producer,250);
        let actor=Actor::Enemy(c.state.enemies[producer].instance_id.clone());c.state.phase=Phase::EnemyTurn;c.state.pending.push_back(Effect{source:Source::None,target:Some(actor),kind:EffectKind::EnemyMove});c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.pending.iter().filter(|e|matches!(e.kind,EffectKind::GainBlock(16)|EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3})).count(),2,"publication includes the already dead list entry");engine::resolve(&mut c.state);assert!(c.state.fault.is_none());assert_eq!(c.state.enemies[1-producer].block,0);assert_eq!(amount(&c,1-producer,"strength"),if producer==0{3}else{0});
    }
}
