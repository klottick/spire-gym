use super::super::*;
use crate::content::{EncounterId,EnemyId};
use super::m5_elites::{amount,hit};
use super::m8_shapes::entry;

fn set(c:&mut Combat,name:&'static str){let i=ActorIntentId::Full(FullIntentId::from_name(name));let a=&mut c.state.enemies[0];a.intent_id=i.clone();a.selected_move=i.clone();a.move_history=vec![i];effects::refresh_enemy_values(&mut c.state).unwrap();}
fn own(c:&mut Combat,kind:EffectKind){let a=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(a.clone()),target:Some(a),kind}],None).unwrap();}
fn player_power(c:&Combat,id:&str)->i32{c.state.player_full_powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.state.amount)}

#[test]
fn m8_growth_checks_actual_constricted(){
    for artifact in [false,true] {
        let mut e=entry(EncounterId::SpireGrowth);if artifact{e.setup.player_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        let mut c=Combat::enter(e).unwrap();assert_eq!(c.state.enemies[0].hp,170);set(&mut c,"growth_constrict");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,300);
        assert_eq!(player_power(&c,"constricted"),if artifact{0}else{10});
        let q=ActorIntentId::Full(FullIntentId::from_name("growth_quick"));c.state.enemies[0].move_history=vec![q.clone(),q.clone()];c.state.enemies[0].intent_id=q.clone();c.state.enemies[0].selected_move=q;effects::refresh_enemy_values(&mut c.state).unwrap();
        own(&mut c,EffectKind::RollMove);assert_eq!(c.state.enemies[0].intent_id.as_str(),if artifact{"growth_constrict"}else{"growth_smash"});
        if !artifact {
            c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainBlock(7)}],None).unwrap();set(&mut c,"growth_quick");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,281,"Constricted10 absorbs Block7, then actual Quick16");
        }
    }
}

#[test]
fn m8_transient_shifts_by_hp_loss_and_fades(){
    let mut c=Combat::enter(entry(EncounterId::Transient)).unwrap();assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].intent_damage),(999,Some(30)));
    own(&mut c,EffectKind::GainBlock(7));hit(&mut c,0,7);assert_eq!(amount(&c,0,"strength"),0);
    hit(&mut c,0,3);hit(&mut c,0,4);assert_eq!((c.state.enemies[0].hp,amount(&c,0,"strength"),amount(&c,0,"shackled")),(992,-7,7));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].intent_damage,amount(&c,0,"strength"),amount(&c,0,"shackled")),(277,Some(40),0,0));
    for (turn,hp) in [(2,237),(3,187),(4,127),(5,57)]{c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,hp,"actual action {turn}");}
    assert_eq!((c.state.outcome,c.state.enemies[0].hp,c.state.enemies[0].life),(Some(Outcome::Victory),0,ActorLife::Active));
    assert!(c.state.hook_events.iter().any(|h|h.hook==Hook::Death&&h.target==Some(Actor::Enemy(c.state.enemies[0].instance_id.clone()))&&h.source==Source::None));
}

#[test]
fn m8_shifting_respects_artifact_and_player_lethal_fifth_attack_precedes_expiry(){
    let mut c=Combat::enter(entry(EncounterId::Transient)).unwrap();own(&mut c,EffectKind::ApplyFullPower{id:FullPowerId("artifact"),amount:1});hit(&mut c,0,5);assert_eq!((amount(&c,0,"artifact"),amount(&c,0,"strength"),amount(&c,0,"shackled")),(0,0,0));
    let mut c=Combat::enter(entry(EncounterId::Transient)).unwrap();for _ in 0..4{c.step_full(&CombatAction::EndTurn).unwrap();}
    c.state.player.hp=70;c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.outcome,c.state.player.hp,c.state.enemies[0].hp),(Some(Outcome::Death),0,999));c.exit().unwrap();
}

#[test]
fn m8_maw_uses_enemy_turn_not_card_count(){
    let mut c=Combat::enter(entry(EncounterId::Maw)).unwrap();assert_eq!(c.state.enemies[0].hp,300);assert_eq!(c.state.enemies[0].intent_id.as_str(),"maw_roar");
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((player_power(&c,"weak"),player_power(&c,"frail")),(3,3));
    for (name,hp) in [("maw_nom",295),("maw_drool",295),("maw_nom",279),("maw_nom",255)]{
        set(&mut c,name);let before=c.snapshot();c.public_full().unwrap();c.legal_actions_full().unwrap();assert_eq!(c.snapshot(),before);c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,hp);
    }
    assert_eq!(amount(&c,0,"strength"),3);
}

#[test]
fn m8_hard_jaw_is_reused(){
    let e=entry(EncounterId::JawWormHorde);assert_eq!(e.setup.enemies.len(),3);assert!(e.setup.enemies.iter().all(|e|e.enemy_id==EnemyId::JawWorm&&e.profile==enemies::EnemyProfile::JawWormHorde));
    let c=Combat::enter(e).unwrap();assert!(c.state.enemies.iter().all(|e|e.block>0));assert!(amount(&c,0,"strength")>0);
}

#[test]
fn m8_transient_multihit_card_reductions_accumulate_before_actual_enemy_action(){
    let mut e=entry(EncounterId::Transient);e.cards[0]=permanent_live_card(crate::content::CardId::TwinStrike,CardInstanceId(1),0);
    let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!((c.state.enemies[0].hp,amount(&c,0,"strength"),amount(&c,0,"shackled")),(989,-10,10));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,amount(&c,0,"strength"),amount(&c,0,"shackled")),(280,0,0));
}

#[test]
fn m8_fading_fault_retains_fifth_attack_and_pending_actual_suicide(){
    let mut c=Combat::enter(entry(EncounterId::Transient)).unwrap();for _ in 0..4{c.step_full(&CombatAction::EndTurn).unwrap();}
    let owner=c.state.enemies[0].instance_id.clone();c.state.phase=Phase::EnemyTurn;enemies::act(&mut c.state,&owner).unwrap();
    for _ in 0..16{if c.state.player.hp==50{break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    assert_eq!((c.state.player.hp,c.state.enemies[0].hp,c.state.outcome),(50,999,None));
    c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_some());assert!(c.state.pending.iter().any(|e|e.kind==EffectKind::TransientFade));
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}
