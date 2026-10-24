//! Real acquired-relic producers; roomy supplied combat is diagnostic, not a normal run.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId,RelicId},items::CombatItemContext,run::*};

fn entry(encounter:EncounterId)->CombatEntry {
    let mut e=super::m8_shapes::entry(encounter);
    e.cards=(1..=12).map(|id|permanent_live_card(CardId::Defend,CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;e
}
fn acquired(ids:&[RelicId])->CombatItemContext {
    let mut run=Run::new(RunConfig{character:Character::Ironclad,ascension:0},CreationProfile::experienced(),RunRng::new([9;32],rng::Partition::Test,0)).unwrap();
    for id in ids {crate::items::acquire_relic(run.state_mut_for_test(),*id).unwrap();}
    CombatItemContext::take_from_run(run.state_mut_for_test())
}
fn enter(encounter:EncounterId,ids:&[RelicId])->Combat {
    Combat::enter_with_item_context(entry(encounter),&mut Some(acquired(ids))).unwrap()
}
fn player_power(c:&Combat,id:&str)->i32 {c.public_full().unwrap().player.powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.amount)}
fn hit(c:&mut Combat,source:Source,payload:DamagePayload){
    c.enqueue_native_effects(vec![Effect{source,target:Some(Actor::Player),kind:EffectKind::Damage(payload)}],None).unwrap();
}

// Missing a real startup producer, repeating it each turn, or bypassing Draw callbacks breaks this.
#[test]
fn m9_relic_startup_payloads_are_once_only(){
    let ids=[RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,RelicId::BronzeScales];
    let mut c=enter(EncounterId::OrbWalker,&ids);
    assert_eq!((player_power(&c,"strength"),player_power(&c,"dexterity"),player_power(&c,"thorns")),(1,1,3));
    assert_eq!(c.public_full().unwrap().hand.len(),7);
    let card=c.public_full().unwrap().hand[0].instance.0;
    c.step_full(&CombatAction::PlayCard{instance:card,target:None}).unwrap();
    assert_eq!(c.public_full().unwrap().player.block,6,"Defend5 gains the actual Dexterity1");
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.public_full().unwrap().hand.len(),5,"Preparation is not another normal-turn draw");
    assert_eq!((player_power(&c,"strength"),player_power(&c,"dexterity"),player_power(&c,"thorns")),(1,1,3));
    let before=c.public_full().unwrap();
    assert_eq!(c.enqueue_hook(HookEvent{source:Source::None,target:None,hook:Hook::CombatStarted}),Err(RunError::InvalidAction));
    assert_eq!(c.public_full().unwrap(),before,"duplicate startup hook does not grant relic work again");
    let absent=enter(EncounterId::OrbWalker,&[]);
    assert_eq!((player_power(&absent,"strength"),player_power(&absent,"dexterity"),player_power(&absent,"thorns"),absent.public_full().unwrap().hand.len()),(0,0,0,5));
    let next=enter(EncounterId::OrbWalker,&ids);
    assert_eq!((player_power(&next,"strength"),player_power(&next,"dexterity"),player_power(&next,"thorns"),next.public_full().unwrap().hand.len()),(1,1,3,7));
}

#[test]
fn m9_relic_marbles_uses_initial_group_and_individual_artifact(){
    let mut e=entry(EncounterId::DonuAndDeca);
    e.setup.enemies[0].powers.iter_mut().find(|p|p.id.0=="artifact").unwrap().state.amount=1;
    e.setup.enemies[1].powers.retain(|p|p.id.0!="artifact");
    let c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::BagOfMarbles]))).unwrap();
    assert_eq!(super::m5_elites::amount(&c,0,"artifact"),0);
    assert_eq!(super::m5_elites::amount(&c,0,"vulnerable"),0);
    assert_eq!(super::m5_elites::amount(&c,1,"vulnerable"),1);
}

#[test]
fn m9_relic_player_thorns_is_per_hit_not_attack_play(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::BronzeScales]);
    let attacker=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    let hp=c.state.enemies[0].hp;
    c.state.player.block=20;
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(attacker.clone()),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(4))};2],None).unwrap();
    assert_eq!((c.state.player.hp,c.state.player.block,c.state.enemies[0].hp),(300,12,hp-6),"two fully blocked hits each retaliate3");
    hit(&mut c,Source::Actor(attacker),DamagePayload::Fixed(0));
    assert_eq!(c.state.enemies[0].hp,hp-9,"owned zero NORMAL still invokes the power");
}

#[test]
fn m9_relic_thorns_excludes_null_self_orb_and_hp_loss(){
    let mut c=enter(EncounterId::OrbWalker,&[RelicId::BronzeScales]);
    let attacker=Actor::Enemy(c.state.enemies[0].instance_id.clone());let hp=c.state.enemies[0].hp;
    for (source,payload) in [(Source::None,DamagePayload::Fixed(2)),(Source::Actor(Actor::Player),DamagePayload::Fixed(2)),(Source::Actor(attacker.clone()),DamagePayload::Orb(2)),(Source::Actor(attacker),DamagePayload::HpLoss(2))] {hit(&mut c,source,payload);}
    assert_eq!((c.state.player.hp,c.state.enemies[0].hp),(292,hp));
}

#[test]
fn m9_relic_preparation_respects_real_hand_capacity(){
    let mut e=entry(EncounterId::OrbWalker);e.opening_draw=9;
    let c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::BagOfPreparation]))).unwrap();
    assert_eq!((c.state.piles.hand.len(),c.state.piles.draw.len()),(10,2));
}

#[test]
fn m9_relic_marbles_does_not_debuff_later_real_summons(){
    let mut c=enter(EncounterId::Reptomancer,&[RelicId::BagOfMarbles]);
    assert_eq!((0..3).map(|slot|super::m5_elites::amount(&c,slot,"vulnerable")).collect::<Vec<_>>(),vec![1,1,1]);
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.enemies.len(),4,"native Reptomancer really inserted the child");
    assert_eq!(super::m5_elites::amount(&c,3,"vulnerable"),0);
}

#[test]
fn m9_relic_thorns_does_not_recurse_with_actual_spiker(){
    let mut e=entry(EncounterId::OrbWalker);
    let a=&mut e.setup.enemies[0];a.enemy_id=crate::content::EnemyId::Spiker;a.hp=42;a.max_hp=42;
    a.powers=vec![FullPowerEntry{id:FullPowerId("thorns"),state:PowerState{amount:3,skip_first:false,monster_source_grace:false}}];
    let memory=EnemyMemory::for_enemy(crate::content::EnemyId::Spiker).unwrap();
    let selected=enemies::select_move(a.enemy_id,&[],&memory,&a.ai_rng).unwrap();
    a.intent=selected.intent;a.initial_memory=Some(selected.memory);a.ai_rng=selected.stream;e.setup.enemy_damage_bases.clear();
    let mut c=Combat::enter_with_item_context(e,&mut Some(acquired(&[RelicId::BronzeScales]))).unwrap();
    let attacker=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    hit(&mut c,Source::Actor(attacker.clone()),DamagePayload::Thorns(3));
    assert_eq!((c.state.player.hp,c.state.enemies[0].hp),(297,42),"incoming THORNS has no retaliation");
    hit(&mut c,Source::Actor(attacker),DamagePayload::Fixed(0));
    assert_eq!((c.state.player.hp,c.state.enemies[0].hp),(297,39),"player retaliation is THORNS, not a new NORMAL hit");
}
