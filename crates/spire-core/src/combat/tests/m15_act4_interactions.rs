//! Native supplied Act4 composition diagnostics; no normal-start outcome claims.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId,PotionId,RelicId},run::{RunRng},items::{CombatItemContext,NativeBattleContext}};
fn entered(encounter:EncounterId,cards:&[CardId],relics:Vec<RelicId>,potions:Vec<Option<PotionId>>)->Combat{
    let entry=enemies::setup(encounter,4,0,&mut RunRng::new([81;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:999,max_hp:999,block:0,energy:9,max_energy:9,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Silent,
        cards:cards.iter().enumerate().map(|(index,id)|permanent_live_card(*id,CardInstanceId(index as u64+1),0)).collect(),opening_draw:cards.len(),encounter_visit:0}).unwrap();
    let mut items=Some(CombatItemContext::new(relics,potions).with_battle_context(NativeBattleContext{elite_trigger:encounter==EncounterId::ShieldAndSpear,boss:encounter==EncounterId::CorruptHeart,burning:None,m15:true}));
    Combat::enter_with_item_context(entry,&mut items).unwrap()
}
fn amount(c:&Combat,actor:Actor,name:&str)->i32{match actor{Actor::Player=>&c.state.player_full_powers,Actor::Enemy(ref owner)=>&c.state.enemies.iter().find(|enemy|&enemy.instance_id==owner).unwrap().full_powers}.iter().find(|power|power.id.0==name).map_or(0,|power|power.state.amount)}
fn enemy(c:&Combat,index:usize)->Actor{Actor::Enemy(c.state.enemies[index].instance_id.clone())}
fn apply(c:&mut Combat,target:Actor,id:&'static str,n:i32){c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::ApplyFullPower{id:FullPowerId(id),amount:n}}],None).unwrap();}
fn play(c:&mut Combat,instance:u64,target:Option<TargetId>){c.step_full(&CombatAction::PlayCard{instance,target}).unwrap();}
#[test]
fn m15_act4_flask_artifact_specimen_and_poison_survivor_share_facing_owner(){
    let mut c=entered(EncounterId::ShieldAndSpear,&[CardId::CripplingCloud,CardId::BouncingFlask,CardId::Deflect],vec![RelicId::TheSpecimen],vec![None;3]);
    assert_eq!(c.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),vec![110,160]);
    for slot in 0..2{assert_eq!(amount(&c,enemy(&c,slot),"artifact"),1);}
    play(&mut c,1,None);for slot in 0..2{assert_eq!(amount(&c,enemy(&c,slot),"artifact"),0);assert_eq!(amount(&c,enemy(&c,slot),"poison"),0);}
    play(&mut c,2,None);assert_eq!((0..2).map(|slot|amount(&c,enemy(&c,slot),"poison")).sum::<i32>(),9);
    let shield=enemy(&c,0);apply(&mut c,shield,"poison",110);c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.enemies[0].hp,0);assert!(c.state.enemies[1].hp>0&&c.state.enemies[1].hp<=50);assert_eq!(amount(&c,Actor::Player,"surrounded"),0);assert_eq!(amount(&c,enemy(&c,1),"back_attack"),0);assert_eq!(c.state.facing,Some(PlayerFacing::Right));assert!(c.validate_runtime_for_test().is_ok());
}
#[test]
fn m15_act4_burst_actual_skill_uses_beat_after_afterimage_and_shared_damage_callbacks(){
    let mut c=entered(EncounterId::CorruptHeart,&[CardId::Burst,CardId::Deflect,CardId::Deflect],vec![],vec![None;3]);
    apply(&mut c,Actor::Player,"after_image",1);apply(&mut c,Actor::Player,"a_thousand_cuts",1);let heart=enemy(&c,0);apply(&mut c,heart,"choked",3);
    let before=c.state.player.hp;play(&mut c,1,None);play(&mut c,2,None);
    assert_eq!(c.state.player.hp,before,"each actual use gains AfterImage Block before its Beat hit");assert_eq!(c.state.player.block,8,"original and Burst child each grant Deflect4 + AfterImage1 - Beat1");
    assert_eq!(c.state.enemies[0].hp,738,"three actual uses each commit Choke3 and Cuts1");assert_eq!(amount(&c,enemy(&c,0),"invincible"),288);assert!(c.validate_runtime_for_test().is_ok());
}
#[test]
fn m15_act4_poison_normal_and_thorns_spend_one_shared_allowance_and_reset_at_monster_start(){
    let mut c=entered(EncounterId::CorruptHeart,&[CardId::Deflect],vec![],vec![None;3]);let heart=enemy(&c,0);apply(&mut c,heart.clone(),"poison",400);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].hp,450);assert_eq!(amount(&c,heart.clone(),"invincible"),0);
    for payload in [DamagePayload::Fixed(100),DamagePayload::Thorns(100),DamagePayload::HpLoss(100)]{c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(heart.clone()),kind:EffectKind::Damage(payload)}],None).unwrap();assert_eq!(c.state.enemies[0].hp,450);}
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].hp,150);assert_eq!(amount(&c,heart,"invincible"),0);assert!(c.validate_runtime_for_test().is_ok());
}
#[test]
fn m15_act4_wraith_and_ghost_intangible_cap_each_real_heart_multihit(){
    for wraith in [false,true]{
        let mut c=entered(EncounterId::CorruptHeart,&[CardId::WraithForm,CardId::Deflect],vec![],vec![Some(PotionId::GhostInAJar),None,None]);
        if wraith{play(&mut c,1,None);}
        for _ in 0..3{if c.state.enemies[0].intent_id.as_str()=="heart_blood_shots"{break}c.step_full(&CombatAction::EndTurn).unwrap();}
        assert_eq!(c.state.enemies[0].intent_id.as_str(),"heart_blood_shots");
        if wraith{assert!(amount(&c,Actor::Player,"dexterity")<0);}else{c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();}
        // Wraith's base two-turn duration may have expired if Echo was first;
        // use the real Ghost slot in that case, preserving its ordinary stack.
        if amount(&c,Actor::Player,"intangible_player")==0{c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();}
        let before=c.state.player.hp;c.state.player.block=0;let owner=c.state.enemies[0].instance_id.clone();enemies::act(&mut c.state,&owner).unwrap();super::super::engine::resolve(&mut c.state);
        assert_eq!(c.state.player.hp,before-12,"each of twelve actual Heart hits is capped at1");assert!(c.fault_record().is_none());
    }
}
#[test]
fn m15_act4_corpse_last_heart_and_specimen_have_no_target_or_extra_allowance(){
    let mut c=entered(EncounterId::CorruptHeart,&[CardId::CorpseExplosion],vec![RelicId::TheSpecimen],vec![None;3]);play(&mut c,1,Some(TargetId(1)));
    let heart=enemy(&c,0);apply(&mut c,heart.clone(),"poison",500);
    for expected in [450,150,0]{let rng_before=c.state.card_rng.clone();c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].hp,expected);if expected>0{assert_eq!(amount(&c,heart.clone(),"invincible"),0);}else{assert_eq!(c.state.card_rng,rng_before,"last-enemy cleanup has no surviving target or random draw");}}
    assert_eq!(c.state.outcome,Some(Outcome::Victory));assert!(c.fault_record().is_none());
}
