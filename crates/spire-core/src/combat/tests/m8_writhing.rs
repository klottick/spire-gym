use super::super::*;
use crate::content::{CardId,EncounterId};
use super::m5_elites::{amount,hit};
use super::m8_shapes::entry;

fn set(c:&mut Combat,name:&'static str){let i=ActorIntentId::Full(FullIntentId::from_name(name));let a=&mut c.state.enemies[0];a.intent_id=i.clone();a.selected_move=i.clone();a.move_history=vec![i];effects::refresh_enemy_values(&mut c.state).unwrap();}

#[test]
fn m8_writhing_rerolls_each_qualifying_hit(){
    let mut e=entry(EncounterId::WrithingMass);e.cards[0]=permanent_live_card(CardId::TwinStrike,CardInstanceId(1),0);
    let mut c=Combat::enter(e).unwrap();assert_eq!((c.state.enemies[0].hp,amount(&c,0,"malleable")),(160,4));
    let stream=c.state.enemies[0].full_ai_rng.clone().unwrap();
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,amount(&c,0,"malleable")),(150,9,6));
    assert_eq!(c.state.enemies[0].move_history.len(),3);let after=c.state.enemies[0].full_ai_rng.as_ref().unwrap();assert_eq!(after.id(),stream.id());assert!(after.word_index()>stream.word_index());
    set(&mut c,"writhing_block");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,0,"malleable"),4);
    assert_eq!((c.state.player.hp,c.state.enemies[0].block),(285,15));
}

#[test]
fn m8_writhing_blocked_and_lethal_hits_do_not_reroll_or_gain_malleable_block(){
    let mut c=Combat::enter(entry(EncounterId::WrithingMass)).unwrap();let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(owner),kind:EffectKind::GainBlock(6)}],None).unwrap();
    let stream=c.state.enemies[0].full_ai_rng.clone();hit(&mut c,0,6);assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,c.state.enemies[0].move_history.len(),amount(&c,0,"malleable")),(160,0,1,4));assert_eq!(c.state.enemies[0].full_ai_rng,stream);
    hit(&mut c,0,160);assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.enemies[0].block,0);assert_eq!(c.state.enemies[0].full_ai_rng,stream);
}

#[test]
fn m8_parasite_used_not_selected(){
    let mut e=entry(EncounterId::WrithingMass);e.setup.enemies[0].intent=FullIntentId::from_name("writhing_parasite");
    let mut c=Combat::enter(e).unwrap();hit(&mut c,0,1);
    assert_ne!(c.state.enemies[0].intent_id.as_str(),"writhing_parasite");assert!(matches!(c.state.enemies[0].full_memory,Some(EnemyMemory::WrithingMass{parasite_used:false,..})));assert!(c.permanent_gains().is_empty());
    set(&mut c,"writhing_parasite");c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(matches!(c.state.enemies[0].full_memory,Some(EnemyMemory::WrithingMass{parasite_used:true,..})));assert_eq!(c.permanent_gains().len(),1);assert_eq!((c.permanent_gains()[0].ordinal,c.permanent_gains()[0].id,c.permanent_gains()[0].upgrades),(1,CardId::Parasite,0));
    assert!(c.state.piles.all(c.state.in_play.as_ref()).all(|c|c.full().unwrap().id!=CardId::Parasite));
    let actor=Actor::Enemy(c.state.enemies[0].instance_id.clone());for _ in 0..5{c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor.clone()),kind:EffectKind::RollMove}],None).unwrap();assert_ne!(c.state.enemies[0].intent_id.as_str(),"writhing_parasite");}
    assert_eq!(c.permanent_gains().len(),1);
}

#[test]
fn m8_parasite_removal_not_combat_exhaust(){
    let mut e=entry(EncounterId::WrithingMass);e.cards=vec![permanent_live_card(CardId::BurningPact,CardInstanceId(1),0),permanent_live_card(CardId::Parasite,CardInstanceId(2),0)];e.opening_draw=2;
    let mut c=Combat::enter(e).unwrap();assert!(!c.legal_actions_full().unwrap().iter().any(|a|matches!(a,CombatAction::PlayCard{instance:2,..})));
    // The existing Burning Pact producer auto-exhausts its sole other hand card.
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert!(c.state.piles.exhaust.iter().any(|c|c.full().unwrap().id==CardId::Parasite));assert_eq!((c.state.player.hp,c.state.player.max_hp),(300,300));
}

#[test]
fn m8_writhing_reactive_excludes_orb_thorns_hp_loss_and_null_owner(){
    let mut c=Combat::enter(entry(EncounterId::WrithingMass)).unwrap();let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    let start=c.state.enemies[0].full_ai_rng.clone();
    for (source,payload) in [(Source::None,DamagePayload::Fixed(1)),(Source::Actor(Actor::Player),DamagePayload::HpLoss(1))]{c.enqueue_native_effects(vec![Effect{source,target:Some(owner.clone()),kind:EffectKind::Damage(payload)}],None).unwrap();}
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("thorns"),amount:1}},Effect{source:Source::Actor(Actor::Player),target:Some(owner.clone()),kind:EffectKind::Damage(DamagePayload::Thorns(1))}],None).unwrap();
    assert_eq!(c.state.enemies[0].full_ai_rng,start);assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,amount(&c,0,"malleable")),(157,0,4));
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(owner),kind:EffectKind::Damage(DamagePayload::Orb(1))}],None).unwrap();
    assert_eq!(c.state.enemies[0].full_ai_rng,start);assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,amount(&c,0,"malleable")),(156,0,4));
    hit(&mut c,0,1);assert_ne!(c.state.enemies[0].full_ai_rng,start);assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,amount(&c,0,"malleable")),(155,4,5));
}

#[test]
fn m8_parasite_journal_ordinal_overflow_retains_execution_latch_without_gain(){
    let mut c=Combat::enter(entry(EncounterId::WrithingMass)).unwrap();set(&mut c,"writhing_parasite");c.state.next_permanent_gain=u64::MAX;
    assert!(c.step_full(&CombatAction::EndTurn).is_err());assert!(c.permanent_gains().is_empty());assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::ObtainParasite);
    assert!(matches!(c.state.enemies[0].full_memory,Some(EnemyMemory::WrithingMass{parasite_used:true,parasite_pending:true,..})));let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m8_writhing_block_uses_base15_not_modified_attack_and_debuff_uses_base10(){
    for strength in [0,3]{
        let mut c=Combat::enter(entry(EncounterId::WrithingMass)).unwrap();let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());
        if strength>0{c.enqueue_native_effects(vec![Effect{source:Source::Actor(owner.clone()),target:Some(owner),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:strength}}],None).unwrap();}
        set(&mut c,"writhing_block");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].block),(285-strength,15));
    }
    let mut c=Combat::enter(entry(EncounterId::WrithingMass)).unwrap();set(&mut c,"writhing_debuff");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,290);
    for id in ["weak","vulnerable"]{assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0==id).unwrap().state.amount,2);}
}

#[test]
fn m8_parasite_pending_obtain_cannot_be_supplied_or_restored_as_quiescent(){
    let mut e=entry(EncounterId::WrithingMass);e.setup.enemies[0].initial_memory=Some(EnemyMemory::WrithingMass{first:false,parasite_used:true,parasite_pending:true});assert!(Combat::enter(e).is_err());
    let mut c=Combat::enter(entry(EncounterId::WrithingMass)).unwrap();c.state.enemies[0].full_memory=Some(EnemyMemory::WrithingMass{first:false,parasite_used:true,parasite_pending:true});assert!(engine::validate_runtime(&c.state).is_err());
}

#[test]
fn m8_writhing_opening_partition_and_execution_gated_parasite_preserve_native_rng(){
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"writhing".into()]).unwrap();
    for cursor in 0..128{
        let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();let (n,next)=rng::uniform_below(&stream,100).unwrap();
        let initial=EnemyMemory::WrithingMass{first:true,parasite_used:false,parasite_pending:false};let selected=enemies::select_move(crate::content::EnemyId::WrithingMass,&[],&initial,&stream).unwrap();
        assert_eq!(selected.intent.as_str(),match n{0..=32=>"writhing_multi",33..=65=>"writhing_block",_=>"writhing_debuff"});assert_eq!(selected.stream,next);
        let used=EnemyMemory::WrithingMass{first:false,parasite_used:true,parasite_pending:false};let selected=enemies::select_move(crate::content::EnemyId::WrithingMass,&[FullIntentId::from_name("writhing_parasite")],&used,&stream).unwrap();assert_ne!(selected.intent.as_str(),"writhing_parasite");assert_eq!(selected.memory,used);
        if (10..20).contains(&n){let unused=EnemyMemory::WrithingMass{first:false,parasite_used:false,parasite_pending:false};let selected=enemies::select_move(crate::content::EnemyId::WrithingMass,&[FullIntentId::from_name("writhing_block")],&unused,&stream).unwrap();assert_eq!(selected.intent.as_str(),"writhing_parasite");assert_eq!(selected.stream,next);assert_eq!(selected.memory,unused);}
    }
}
