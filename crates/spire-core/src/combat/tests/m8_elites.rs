use super::super::*;
use crate::content::{CardId,EncounterId};
use super::m8_shapes::entry;
use super::m5_elites::amount;

fn power(id:&'static str,amount:i32)->FullPowerEntry{FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}}}
fn play(c:&mut Combat,id:u64,target:bool){c.step_full(&CombatAction::PlayCard{instance:id,target:target.then_some(TargetId(1))}).unwrap();}

// Counts all completed plays, not hits, and only after the triggering card.
#[test]
fn m8_slow_counts_cards_and_resets(){
    let mut e=entry(EncounterId::GiantHead);e.cards=vec![permanent_live_card(CardId::Strike,CardInstanceId(1),0),permanent_live_card(CardId::Defend,CardInstanceId(2),0),permanent_live_card(CardId::TwinStrike,CardInstanceId(3),0)];e.opening_draw=3;
    let mut c=Combat::enter(e).unwrap();assert_eq!((c.state.enemies[0].hp,amount(&c,0,"slow")),(500,0));
    let frozen=c.snapshot();assert!(c.step_full(&CombatAction::PlayCard{instance:2,target:Some(TargetId(1))}).is_err());assert_eq!(c.snapshot(),frozen);
    play(&mut c,1,true);assert_eq!((c.state.enemies[0].hp,amount(&c,0,"slow")),(494,1));
    play(&mut c,2,false);assert_eq!(amount(&c,0,"slow"),2);
    play(&mut c,3,true);assert_eq!((c.state.enemies[0].hp,amount(&c,0,"slow")),(482,3));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(amount(&c,0,"slow"),0);assert!(c.state.enemies[0].full_powers.iter().any(|p|p.id.0=="slow"));
}

#[test]
fn m8_slow_keeps_one_final_floor_after_weak_strength_and_vulnerable(){
    let mut e=entry(EncounterId::GiantHead);e.cards=vec![permanent_live_card(CardId::Defend,CardInstanceId(1),0),permanent_live_card(CardId::Strike,CardInstanceId(2),0)];e.opening_draw=2;
    e.setup.player_powers=vec![power("strength",1),power("weak",2)];e.setup.enemies[0].powers.push(power("vulnerable",2));
    let mut c=Combat::enter(e).unwrap();play(&mut c,1,false);play(&mut c,2,true);
    // floor((6+1)*0.75*1.5*1.1)=8, NOT floor(7*1.1)=7.
    assert_eq!(c.state.enemies[0].hp,492);assert_eq!(amount(&c,0,"slow"),2);
}

#[test]
fn m8_head_countdown_and_damage_cap(){
    let mut e=entry(EncounterId::GiantHead);e.setup.player.hp=999;e.setup.player.max_hp=999;
    let mut c=Combat::enter(e).unwrap();
    for turn in 1..=12{
        assert!(engine::validate_runtime(&c.state).is_ok(),"canonical Head turn {turn}");
        if turn<5{assert!(matches!(c.state.enemies[0].intent_id.as_str(),"head_count"|"head_glare"));}
        else{assert_eq!(c.state.enemies[0].intent_id.as_str(),"head_time");assert_eq!(c.state.enemies[0].intent_damage,Some([30,35,40,45,50,55,60,60][turn-5]));}
        let before=c.state.player.hp;let intent=c.state.enemies[0].intent_id.as_str();c.step_full(&CombatAction::EndTurn).unwrap();
        let want=if turn>=5{[30,35,40,45,50,55,60,60][turn-5]}else if intent=="head_count"{13}else{0};assert_eq!(before-c.state.player.hp,want);
        if intent=="head_glare"{assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="weak"&&p.state.amount>0));}
    }
}

#[test]
fn m8_nemesis_intangible_is_per_hit(){
    let mut e=entry(EncounterId::Nemesis);e.cards=vec![permanent_live_card(CardId::TwinStrike,CardInstanceId(1),0)];for id in 2..=4{e.cards.push(permanent_live_card(CardId::Defend,CardInstanceId(id),0));}e.opening_draw=4;e.setup.player.draw_per_turn=10;let mut c=Combat::enter(e).unwrap();
    assert_eq!((c.state.enemies[0].hp,amount(&c,0,"intangible")),(185,0));assert_ne!(c.state.enemies[0].intent_id.as_str(),"nemesis_scythe");
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.turn,amount(&c,0,"intangible")),(2,1));play(&mut c,1,true);assert_eq!(c.state.enemies[0].hp,183);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.turn,amount(&c,0,"intangible")),(3,0));play(&mut c,1,true);assert_eq!(c.state.enemies[0].hp,173);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.turn,amount(&c,0,"intangible")),(4,1));
}

#[test]
fn m8_nemesis_all_typed_positive_hits_cap_before_block_zero_stays_zero(){
    let mut c=Combat::enter(entry(EncounterId::Nemesis)).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    for payload in [DamagePayload::Fixed(0),DamagePayload::Orb(0),DamagePayload::HpLoss(0)]{c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(owner.clone()),kind:EffectKind::Damage(payload)}],None).unwrap();}assert_eq!(c.state.enemies[0].hp,185);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(owner.clone()),kind:EffectKind::GainBlock(2)},Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("thorns"),amount:7}}],None).unwrap();
    for payload in [DamagePayload::Fixed(9),DamagePayload::Orb(8),DamagePayload::Thorns(7),DamagePayload::HpLoss(6)]{c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(owner.clone()),kind:EffectKind::Damage(payload)}],None).unwrap();}
    assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block),(183,0));
}

#[test]
fn m8_nemesis_payloads_are_real_and_burns_enter_discard(){
    for (name,want_hp,want_burns) in [("nemesis_tri",282,0),("nemesis_scythe",255,0),("nemesis_burn",300,3)]{
        let mut e=entry(EncounterId::Nemesis);for id in 2..=4{e.cards.push(permanent_live_card(CardId::Defend,CardInstanceId(id),0));}let mut c=Combat::enter(e).unwrap();let i=ActorIntentId::Full(FullIntentId::from_name(name));c.state.enemies[0].intent_id=i.clone();c.state.enemies[0].selected_move=i;effects::refresh_enemy_values(&mut c.state).unwrap();
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,want_hp);assert_eq!(c.state.piles.discard.iter().filter(|c|c.full().unwrap().id==CardId::Burn).count(),want_burns);
    }
}

#[test]
fn m8_nemesis_history_redirect_tickets(){
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"nemesis".into()]).unwrap();
    let find=|lo,hi,coin:Option<u64>|(0..2048).find_map(|index|{let stream=rng::StreamState::new(id.clone(),[0;32],index).unwrap();let (n,next)=rng::uniform_below(&stream,100).unwrap();if n<lo||n>hi{return None}let after=if let Some(want)=coin{let (v,next)=rng::uniform_below(&next,2).unwrap();if v!=want{return None}next}else{next};Some((stream,after))}).unwrap();
    for (lo,hi,coin,history,cooldown,want,want_cooldown) in [
        (0,29,None,vec!["nemesis_tri"],1,"nemesis_scythe",2),
        (0,29,Some(0),vec!["nemesis_scythe"],2,"nemesis_tri",1),
        (0,29,Some(1),vec!["nemesis_scythe"],2,"nemesis_burn",1),
        (30,64,None,vec!["nemesis_burn"],0,"nemesis_tri",0),
        (30,64,Some(0),vec!["nemesis_tri";2],1,"nemesis_scythe",2),
        (30,64,Some(1),vec!["nemesis_tri";2],1,"nemesis_burn",0),
        (30,64,Some(0),vec!["nemesis_tri";2],2,"nemesis_burn",1),
        (65,99,None,vec!["nemesis_tri"],0,"nemesis_burn",0),
        (65,99,Some(0),vec!["nemesis_burn"],1,"nemesis_scythe",2),
        (65,99,Some(1),vec!["nemesis_burn"],1,"nemesis_tri",0),
        (65,99,Some(0),vec!["nemesis_burn"],2,"nemesis_tri",1),
    ]{
        let (stream,after)=find(lo,hi,coin);let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();
        let selected=enemies::select_move(crate::content::EnemyId::Nemesis,&history,&EnemyMemory::Nemesis{first:false,cooldown},&stream).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.memory,EnemyMemory::Nemesis{first:false,cooldown:want_cooldown});assert_eq!(selected.stream,after);
    }
    for (lo,hi,want) in [(0,49,"nemesis_tri"),(50,99,"nemesis_burn")]{let (stream,after)=find(lo,hi,None);let selected=enemies::select_move(crate::content::EnemyId::Nemesis,&[],&EnemyMemory::Nemesis{first:true,cooldown:0},&stream).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.stream,after);}
}

#[test]
fn m8_head_supplied_countdown_restores_dynamic_preview_and_live_modifiers(){
    let mut e=entry(EncounterId::GiantHead);e.setup.enemies[0].initial_memory=Some(EnemyMemory::GiantHead{count:-3});e.setup.enemies[0].intent=FullIntentId::from_name("head_time");e.setup.enemies[0].powers.extend([power("weak",1),power("strength",2)]);
    let mut c=Combat::enter(e).unwrap();assert_eq!(c.state.enemies[0].intent_damage,Some(35));let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(owner),kind:EffectKind::RemoveFullPower(FullPowerId("weak"))}],None).unwrap();assert_eq!(c.state.enemies[0].intent_damage,Some(47));c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,c.state.enemies[0].intent_damage),(253,Some(52)));
}

#[test]
fn m8_elite_selectors_reject_other_species_memory(){
    let e=entry(EncounterId::GiantHead);let stream=&e.setup.enemies[0].ai_rng;
    assert!(enemies::select_move(crate::content::EnemyId::GiantHead,&[],&EnemyMemory::Nemesis{first:true,cooldown:0},stream).is_err());
    assert!(enemies::select_move(crate::content::EnemyId::Nemesis,&[],&EnemyMemory::GiantHead{count:5},stream).is_err());
    for (encounter,name) in [(EncounterId::GiantHead,"nemesis_burn"),(EncounterId::Nemesis,"head_glare")]{let mut e=entry(encounter);e.setup.enemies[0].intent=FullIntentId::from_name(name);assert!(Combat::enter(e).is_err(),"foreign intent must not execute another species' nonattack");}
}

#[test]
fn m8_slow_blocked_and_zero_cards_count_but_typed_nonattacks_do_not_amplify(){
    let mut e=entry(EncounterId::GiantHead);e.cards=vec![permanent_live_card(CardId::Defend,CardInstanceId(1),0),permanent_live_card(CardId::Strike,CardInstanceId(2),0)];e.opening_draw=2;
    let mut c=Combat::enter(e.clone()).unwrap();play(&mut c,1,false);let owner=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(owner.clone()),kind:EffectKind::GainBlock(20)}],None).unwrap();play(&mut c,2,true);assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,amount(&c,0,"slow")),(500,14,2));
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("thorns"),amount:10}}],None).unwrap();
    for payload in [DamagePayload::HpLoss(7),DamagePayload::Orb(20),DamagePayload::Thorns(10)]{c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(owner.clone()),kind:EffectKind::Damage(payload)}],None).unwrap();}assert_eq!((c.state.enemies[0].hp,c.state.enemies[0].block,amount(&c,0,"slow")),(477,0,2));
    e.setup.player_powers=vec![power("strength",-999)];let mut zero=Combat::enter(e).unwrap();play(&mut zero,2,true);assert_eq!((zero.state.enemies[0].hp,amount(&zero,0,"slow")),(500,1));
}

#[test]
fn m8_nemesis_potion_and_late_roll_fault_retain_the_actual_power_owner(){
    let mut items=Some(crate::items::CombatItemContext::new(vec![],vec![Some(crate::content::PotionId::FirePotion)]));let mut c=Combat::enter_with_item_context(entry(EncounterId::Nemesis),&mut items).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();c.step_full(&CombatAction::UsePotion{slot:0,target:Some(TargetId(1))}).unwrap();assert_eq!(c.state.enemies[0].hp,184);assert!(c.item_context().unwrap().potions()[0].is_none());
    let mut e=entry(EncounterId::Nemesis);e.setup.enemies[0].intent=FullIntentId::from_name("nemesis_tri");let id=e.setup.enemies[0].ai_rng.id().clone();e.setup.enemies[0].ai_rng=rng::StreamState::new(id,[0;32],1_u128<<64).unwrap();let mut c=Combat::enter(e).unwrap();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!((c.state.player.hp,amount(&c,0,"intangible")),(282,1));assert!(c.state.enemies[0].full_powers.iter().any(|p|p.id.0=="intangible"&&p.state.skip_first));let frozen=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),frozen);
}
