//! Bounded composition fixtures, never prescribed caller outcomes or traversal.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId,RelicId,PotionId},items::{CombatItemContext,NativeBattleContext},run::RunRng};
use characters::{CharacterEffect as C,OrbPassiveGroup};
use super::m5_elites::{amount,hit};
fn power(id:&'static str,n:i32)->FullPowerEntry{FullPowerEntry{id:FullPowerId(id),state:PowerState{amount:n,skip_first:false,monster_source_grace:false}}}
fn enter(encounter:EncounterId,ids:&[CardId],relics:Vec<RelicId>,potions:Vec<Option<PotionId>>)->Combat{
    let act=if matches!(encounter,EncounterId::ShieldAndSpear|EncounterId::CorruptHeart){4}else{3};
    let e=enemies::setup(encounter,act,0,&mut RunRng::new([81;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:999,max_hp:999,block:0,energy:9,max_energy:9,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Defect,
        cards:ids.iter().enumerate().map(|(i,id)|permanent_live_card(*id,CardInstanceId(i as u64+1),0)).collect(),opening_draw:ids.len(),encounter_visit:0}).unwrap();
    let mut owner=Some(CombatItemContext::new(relics,potions).with_battle_context(NativeBattleContext{elite_trigger:encounter==EncounterId::ShieldAndSpear,boss:encounter==EncounterId::CorruptHeart,burning:None,m15:false,m20:true}));
    Combat::enter_with_item_context(e,&mut owner).unwrap()
}
fn play(c:&mut Combat,n:u64,target:Option<TargetId>){c.step_full(&CombatAction::PlayCard{instance:n,target}).unwrap();}
fn orb(c:&mut Combat,work:C){c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Character(work)}],None).unwrap();}
fn set_intent(c:&mut Combat,slot:usize,name:&'static str){let id=ActorIntentId::Full(FullIntentId::from_name(name));let e=&mut c.state.enemies[slot];e.intent_id=id.clone();e.selected_move=id.clone();e.move_history.push(id);effects::refresh_enemy_values(&mut c.state).unwrap();}
#[test]
fn m20_defect_interactions_orbs_normal_hits_and_shared_heart_allowance(){
    let mut c=enter(EncounterId::CorruptHeart,&[CardId::ThunderStrike,CardId::ReinforcedBody],vec![],vec![None;3]);
    assert_eq!(c.state.enemies[0].hp,750);assert_eq!(amount(&c,0,"invincible"),300);
    c.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(299)}),Some(OrbKind::Lightning.into()),None];
    orb(&mut c,C::EvokeFront{remove:true});assert_eq!(c.state.enemies[0].hp,451);assert_eq!(amount(&c,0,"invincible"),1);
    orb(&mut c,C::EvokeFront{remove:true});assert_eq!(c.state.enemies[0].hp,450);assert_eq!(amount(&c,0,"invincible"),0);
    c.state.defect.lightning_channeled=3;for card in c.state.piles.hand.iter_mut().filter_map(model::OwnedCard::full_mut){cards::defect::refresh_values(card,0,3,0);}play(&mut c,1,None);
    assert_eq!(c.state.enemies[0].hp,450,"each normal Thunder hit shares the exhausted allowance");
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardPlayed).count(),1,"orb effects never notify a card use");
    let mut c=enter(EncounterId::Nemesis,&[CardId::RipAndTear],vec![],vec![None;3]);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,0,"intangible"),1);let hp=c.state.enemies[0].hp;
    play(&mut c,1,None);assert_eq!(c.state.enemies[0].hp,hp-2);
    c.state.character.orb_slots=vec![Some(OrbKind::Lightning.into()),None,None];orb(&mut c,C::EvokeFront{remove:true});assert_eq!(c.state.enemies[0].hp,hp-3);
}
#[test]
fn m20_defect_interactions_frost_ignores_dexterity_frail_ordinary_x_block_does_not(){
    let mut c=enter(EncounterId::CorruptHeart,&[CardId::ReinforcedBody],vec![],vec![None;3]);
    c.state.player_full_powers.extend([power("dexterity",3),power("frail",2)]);c.state.player.energy=2;
    play(&mut c,1,None);assert_eq!(c.state.player.block,13,"two floor((7+3)*.75) gains, then Beat spends one actual Block");
    c.state.character.orb_slots=vec![Some(OrbKind::Frost.into()),None,None];orb(&mut c,C::EvokeFront{remove:true});assert_eq!(c.state.player.block,18,"Frost evoke5 ignores card Dexterity/Frail");
}
#[test]
fn m20_defect_interactions_shield_spear_facing_and_multi_hit_reactions(){
    let mut c=enter(EncounterId::ShieldAndSpear,&[CardId::BallLightning,CardId::Buffer,CardId::Buffer],vec![],vec![None;3]);
    assert_eq!(c.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>(),vec![110,160]);
    let initial=c.state.facing;c.state.character.orb_slots=vec![Some(OrbKind::Lightning.into()),None,None];orb(&mut c,C::PassiveGroup(OrbPassiveGroup::NaturalEnd));assert_eq!(c.state.facing,initial);
    play(&mut c,1,Some(TargetId(2)));assert_eq!(c.state.facing,Some(PlayerFacing::Right));
    play(&mut c,2,None);assert_eq!(c.state.facing,Some(PlayerFacing::Right),"Power target is no facing attack");
    hit(&mut c,0,999);assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="surrounded"));assert_eq!(amount(&c,1,"back_attack"),0);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[1].intent_id.as_str(),"spear_skewer");play(&mut c,3,None);
    c.state.player_full_powers.push(power("static_discharge",1));c.state.character.orb_slots=vec![None;3];
    let before_channels=c.state.defect.lightning_channeled;let hp=c.state.player.hp;let owner=c.state.enemies[1].instance_id.clone();enemies::act(&mut c.state,&owner).unwrap();assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Decision);
    assert!(c.state.player.hp<hp);assert_eq!(c.state.defect.lightning_channeled-before_channels,2,"post-change-damage onAttacked sees zero for the Buffer-prevented first hit");assert!(engine::validate_runtime(&c.state).is_ok());
}
#[test]
fn m20_defect_interactions_beat_buffer_repeat_and_lethal_cancellation(){
    let mut c=enter(EncounterId::CorruptHeart,&[CardId::Buffer,CardId::Glacier],vec![],vec![None;3]);
    let hp=c.state.player.hp;play(&mut c,1,None);assert_eq!(c.state.player.hp,hp,"new Buffer is installed before its own after-use Beat and prevents it");
    c.state.player_full_powers.push(power("echo_form",2));play(&mut c,2,None);
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CardPlayed).count(),3);assert_eq!(c.state.player.hp,hp,"both Glacier after-use Beats spend the actual gained card Block");
    assert_eq!(c.state.defect.frost_channeled,4);assert_eq!(c.state.play_frames.len(),0);assert_eq!(c.state.inactive_cards.len(),1);assert!(engine::validate_runtime(&c.state).is_ok());
    let mut lethal=enter(EncounterId::CorruptHeart,&[CardId::HelloWorld],vec![],vec![None;3]);lethal.state.player.hp=1;lethal.state.player_full_powers.push(power("echo_form",1));
    play(&mut lethal,1,None);assert_eq!(lethal.state.outcome,Some(Outcome::Death));assert_eq!(lethal.state.defect.frost_channeled,0);assert_eq!(lethal.state.queued_autoplay.len(),1,"terminal retains unstarted child custody without dispatch");assert_eq!(lethal.state.play_frames.len(),1,"finished parent remains in the existing deferred census");assert_eq!(lethal.state.hook_events.iter().filter(|e|e.hook==Hook::CardPlayed).count(),1);assert!(engine::validate_runtime(&lethal.state).is_ok());
}
#[test]
fn m20_defect_interactions_timewarp_new_power_generator_and_equilibrium_cleanup(){
    let mut c=enter(EncounterId::TimeEater,&[CardId::HelloWorld,CardId::Equilibrium,CardId::Void],vec![RelicId::FrozenCore],vec![None;3]);
    c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=11;
    c.state.player_full_powers.extend([power("echo_form",1),power("heatsinks",1),power("equilibrium",1)]);
    set_intent(&mut c,0,"time_ripple");play(&mut c,1,None);
    assert_eq!(c.state.turn,2);assert_eq!(amount(&c,0,"time_warp"),0);assert_eq!(amount(&c,0,"strength"),2);
    assert!(c.state.play_frames.is_empty()&&c.state.queued_autoplay.is_empty());assert!(c.state.in_play.is_none());
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CardPlayed).count(),1);
    assert_eq!(c.state.defect.frost_channeled,1,"FrozenCore executes once in real forced end-turn");
    assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="equilibrium"));assert!(c.state.piles.exhaust.iter().any(|card|card.full().is_some_and(|card|card.id==CardId::Void)));
    let generated=c.state.piles.hand.iter().filter_map(model::OwnedCard::full).filter(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).collect::<Vec<_>>();assert_eq!(generated.len(),1,"Hello callback on next real turn; suppressed repeat never installs another intensity");assert!(engine::validate_runtime(&c.state).is_ok());
}
#[test]
fn m20_defect_interactions_repair_victory_smoke_and_death_use_distinct_callbacks(){
    for smoke in [false,true]{
        let mut c=enter(EncounterId::GiantHead,&[CardId::SelfRepair,CardId::SelfRepair],vec![],vec![Some(PotionId::SmokeBomb),None,None]);
        play(&mut c,1,None);play(&mut c,2,None);c.state.player.hp=10;
        if smoke{c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();assert_eq!(c.state.outcome,Some(Outcome::Escaped));assert_eq!(c.state.player.hp,24,"native Smoke checkpoint invokes the direct battle-end Repair callback before clear; no frame-timing claim");}
        else{hit(&mut c,0,999);assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.player.hp,24,"stacked direct Repair heals14 once");assert_eq!(c.state.hp_before_healing,Some(10));}
        engine::validate_runtime(&c.state).unwrap();assert!(c.exit().is_ok());
    }
}
#[test]
fn m20_defect_interactions_buffer_discharge_block_type_intangible_and_chip_checkpoints(){
    for (payload,block,intangible)in [(DamagePayload::Fixed(5),0,false),(DamagePayload::Fixed(5),5,false),(DamagePayload::Fixed(5),0,true),(DamagePayload::Thorns(1),0,false),(DamagePayload::HpLoss(1),0,false)]{
        let mut c=enter(EncounterId::CorruptHeart,&[CardId::Buffer],vec![RelicId::EmotionChip],vec![None;3]);
        cards::ironclad::rules::upgrade(c.state.piles.hand[0].full_mut().unwrap()).unwrap();play(&mut c,1,None);
        assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="buffer").unwrap().state.amount,1,"Buffer+ survives its own one-point Beat with one charge");
        c.state.player_full_powers.push(power("static_discharge",1));if intangible{c.state.player_full_powers.push(power("intangible_player",1));}c.state.player.block=block;let hp=c.state.player.hp;
        let source=if matches!(payload,DamagePayload::HpLoss(_)){Source::None}else{Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone()))};
        c.enqueue_native_effects(vec![Effect{source,target:Some(Actor::Player),kind:EffectKind::Damage(payload)}],None).unwrap();
        assert_eq!(c.state.player.hp,hp);assert_eq!(c.state.defect.lightning_channeled,0);assert!(!c.state.item_context.as_ref().unwrap().emotion_chip_armed);
        let buffer=c.state.player_full_powers.iter().find(|p|p.id.0=="buffer").map_or(0,|p|p.state.amount);assert_eq!(buffer,i32::from(block==5),"only a positive post-Block change-damage spends Buffer");
    }
    let mut c=enter(EncounterId::CorruptHeart,&[CardId::DefendBlue],vec![RelicId::EmotionChip],vec![None;3]);c.state.player_full_powers.extend([power("static_discharge",1),power("intangible_player",1)]);let hp=c.state.player.hp;
    let enemy=c.state.enemies[0].instance_id.clone();c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(enemy)),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(5))}],None).unwrap();
    assert_eq!(c.state.player.hp,hp-1);assert_eq!(c.state.defect.lightning_channeled,1);assert!(c.state.item_context.as_ref().unwrap().emotion_chip_armed);
}
#[test]
fn m20_defect_interactions_new_blue_and_essence_dark_capture_keep_later_halfdead_reference(){
    for potion in [false,true]{
        let mut c=enter(EncounterId::ThreeDarklings,&[CardId::MultiCast],vec![],vec![None;3]);
        let source=if potion{Source::Potion{slot:0,id:PotionId::EssenceOfDarkness}}else{Source::Card(card_state::handle(CardInstanceId(1)))};
        let slot=c.state.enemies.iter().enumerate().min_by_key(|(i,e)|(e.hp,*i)).unwrap().0;
        c.state.character.orb_slots=vec![Some(OrbKind::Dark.into()),None,None];characters::apply(&mut c.state,&source,C::EvokeFront{remove:true}).unwrap();
        let captured=c.state.pending.pop_front().unwrap();assert_eq!(captured.target,Some(Actor::Enemy(c.state.enemies[slot].instance_id.clone())));
        let hp=c.state.enemies[slot].hp;hit(&mut c,slot,hp);assert_eq!(c.state.enemies[slot].life,ActorLife::HalfDead);
        let before=c.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>();let deaths=c.state.hook_events.iter().filter(|e|matches!(e.hook,Hook::Death|Hook::HalfDeath)).count();
        engine::validate_effect(&captured,&c.state).expect("M20 supported producer retains the already captured Dark reference");
        c.state.pending.push_front(captured);assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Decision);assert_eq!(c.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>(),before);assert_eq!(c.state.hook_events.iter().filter(|e|matches!(e.hook,Hook::Death|Hook::HalfDeath)).count(),deaths);assert_eq!(c.state.outcome,None);assert!(engine::validate_runtime(&c.state).is_ok());
    }
}
#[test]
fn m20_defect_interactions_awakened_new_powers_repeat_phase_and_retirement(){
    let mut c=enter(EncounterId::AwakenedOne,&[CardId::Buffer,CardId::HelloWorld],vec![],vec![None;3]);c.state.player_full_powers.extend([power("echo_form",1),power("amplify",1)]);play(&mut c,1,None);
    assert_eq!(amount(&c,2,"strength"),3);assert_eq!(c.state.silent.powers_played_this_combat,3);assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="buffer").unwrap().state.amount,3);assert_eq!(c.state.played_powers.len(),1);assert_eq!(c.state.inactive_cards.len(),2);assert!(c.state.play_frames.is_empty());
    for slot in [0,1,2]{let hp=c.state.enemies[slot].hp;hit(&mut c,slot,hp);}assert_eq!(c.state.enemies[2].life,ActorLife::HalfDead);assert_eq!(c.state.outcome,None);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[2].hp,300);assert_eq!(amount(&c,2,"curiosity"),0);let strength=amount(&c,2,"strength");play(&mut c,2,None);assert_eq!(amount(&c,2,"strength"),strength,"reborn phase has no Curiosity callback");
    hit(&mut c,2,300);assert_eq!(c.state.outcome,Some(Outcome::Victory));assert!(engine::validate_runtime(&c.state).is_ok());
}
#[test]
fn m20_defect_interactions_sunder_halfdeath_and_last_enemy_have_distinct_continuations(){
    let mut c=enter(EncounterId::ThreeDarklings,&[CardId::Sunder],vec![],vec![None;3]);c.state.enemies[0].hp=20;play(&mut c,1,Some(TargetId(1)));assert_eq!(c.state.enemies[0].life,ActorLife::HalfDead);assert_eq!(c.state.player.energy,9,"captured target HP<=0 grants3 while other actors keep the combat live");assert_eq!(c.state.outcome,None);
    let mut c=enter(EncounterId::GiantHead,&[CardId::Sunder],vec![],vec![None;3]);c.state.enemies[0].hp=20;play(&mut c,1,Some(TargetId(1)));assert_eq!(c.state.outcome,Some(Outcome::Victory));assert_eq!(c.state.player.energy,6,"existing terminal checkpoint cancels the later energy action; no forced drain");assert!(engine::validate_runtime(&c.state).is_ok());assert!(c.exit().is_ok());
}
