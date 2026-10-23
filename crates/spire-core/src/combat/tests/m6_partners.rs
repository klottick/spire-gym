use super::super::*;
use crate::content::{EncounterId,RelicId};
use super::m5_elites::{hit,amount};

fn entry()->CombatEntry{super::m6_ordinary::entry(EncounterId::CenturionAndHealer)}
fn stream(c:&mut Combat,slot:usize,cursor:u128){let old=c.state.enemies[slot].full_ai_rng.as_ref().unwrap();c.state.enemies[slot].full_ai_rng=Some(rng::StreamState::new(old.id().clone(),[0;32],cursor).unwrap());}
fn roll(c:&mut Combat,slot:usize){let actor=Actor::Enemy(c.state.enemies[slot].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RollMove}],None).unwrap();}
fn act(c:&mut Combat,slot:usize){let actor=Actor::Enemy(c.state.enemies[slot].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove}],None).unwrap();}

#[test]
fn m6_partner_choice_reads_live_survivors() {
    let mut c=Combat::enter(entry()).unwrap();stream(&mut c,0,33);roll(&mut c,0);assert_eq!(c.state.enemies[0].intent_id.as_str(),"centurion_protect");
    hit(&mut c,1,100);stream(&mut c,0,33);c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!((c.state.player.hp,c.state.enemies[0].block,c.state.enemies[1].hp),(80,15,0),"captured Protect falls back to self Block, not a dead partner");assert_eq!(c.state.enemies[0].intent_id.as_str(),"centurion_fury");
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,62,"subsequently selected Fury is three separate hit6s");
}

#[test]
fn m6_mystic_heal_threshold_and_repeat_limit() {
    for swapped in [false,true] {
        let mut e=entry();if swapped{e.setup.enemies.swap(0,1)}
        let mut items=Some(crate::items::CombatItemContext::new(vec![RelicId::MagicFlower,RelicId::MarkOfBloom],vec![]));let mut c=Combat::enter_with_item_context(e,&mut items).unwrap();
        let fighter=usize::from(swapped);let mystic=1-fighter;assert!((76..=80).contains(&c.state.enemies[fighter].hp));assert!((48..=56).contains(&c.state.enemies[mystic].hp));
        hit(&mut c,fighter,15);roll(&mut c,mystic);assert_ne!(c.state.enemies[mystic].intent_id.as_str(),"mystic_heal");
        hit(&mut c,fighter,1);let before=c.state.enemies[mystic].full_ai_rng.clone();roll(&mut c,mystic);assert_eq!(c.state.enemies[mystic].full_ai_rng,before,"forced Heal needs no probabilistic branch");assert_eq!(c.state.enemies[mystic].intent_id.as_str(),"mystic_heal");
        hit(&mut c,mystic,3);act(&mut c,mystic);for slot in [fighter,mystic]{assert_eq!(c.state.enemies[slot].hp,c.state.enemies[slot].max_hp,"native enemy Heal16 caps and ignores PLAYER Bloom/Flower");}
        hit(&mut c,fighter,100);hit(&mut c,mystic,16);roll(&mut c,mystic);act(&mut c,mystic);assert_eq!(c.state.enemies[fighter].hp,0);assert_eq!(c.state.enemies[mystic].hp,c.state.enemies[mystic].max_hp);
    }
    let mut c=Combat::enter(entry()).unwrap();let max=c.state.enemies[0].max_hp;hit(&mut c,0,60);roll(&mut c,1);
    for remaining in [44,28]{assert_eq!(c.state.enemies[1].intent_id.as_str(),"mystic_heal");c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.enemies[0].hp,max-remaining);}
    assert!(matches!(c.state.enemies[1].intent_id.as_str(),"mystic_attack"|"mystic_buff"),"missing28 still cannot cause a third consecutive Heal");
}

#[test]
fn m6_mystic_group_heal_combines_living_injuries_at_the_threshold() {
    for swapped in [false, true] {
        let mut e = entry();
        if swapped {
            e.setup.enemies.swap(0, 1);
        }
        let mut c = Combat::enter(e).unwrap();
        let fighter = usize::from(swapped);
        let mystic = 1 - fighter;

        hit(&mut c, fighter, 7);
        hit(&mut c, mystic, 8);
        roll(&mut c, mystic);
        assert_ne!(c.state.enemies[mystic].intent_id.as_str(), "mystic_heal");

        hit(&mut c, fighter, 1);
        let before = c.state.enemies[mystic].full_ai_rng.clone();
        roll(&mut c, mystic);
        assert_eq!(c.state.enemies[mystic].intent_id.as_str(), "mystic_heal",
            "two injuries of 8 meet the group threshold even though neither reaches 16");
        assert_eq!(c.state.enemies[mystic].full_ai_rng, before,
            "forced healing does not consume an ordinary Attack/Buff draw");
        act(&mut c, mystic);
        for slot in [fighter, mystic] {
            assert_eq!(c.state.enemies[slot].hp, c.state.enemies[slot].max_hp);
        }

        hit(&mut c, fighter, 100);
        hit(&mut c, mystic, 15);
        roll(&mut c, mystic);
        assert_ne!(c.state.enemies[mystic].intent_id.as_str(), "mystic_heal",
            "the dead partner's missing HP must not contribute");
        hit(&mut c, mystic, 1);
        roll(&mut c, mystic);
        assert_eq!(c.state.enemies[mystic].intent_id.as_str(), "mystic_heal");
    }
}

#[test]
fn m6_mystic_group_heal_priority_respects_selected_move_history() {
    let mut c = Combat::enter(entry()).unwrap();
    hit(&mut c, 0, 15);
    hit(&mut c, 1, 15);
    for _ in 0..2 {
        roll(&mut c, 1);
        assert_eq!(c.state.enemies[1].intent_id.as_str(), "mystic_heal");
    }
    // These were selections, not executed heals: the group is still injured.
    roll(&mut c, 1);
    assert!(matches!(c.state.enemies[1].intent_id.as_str(), "mystic_attack" | "mystic_buff"),
        "group-wide injury does not permit a third consecutive selected Heal");
}

#[test]
fn m6_mystic_real_group_buff_and_attack_use_shared_strength_then_frail() {
    let mut c=Combat::enter(entry()).unwrap();stream(&mut c,1,33);roll(&mut c,1);assert_eq!(c.state.enemies[1].intent_id.as_str(),"mystic_buff");act(&mut c,1);
    assert_eq!((amount(&c,0,"strength"),amount(&c,1,"strength")),(2,2));stream(&mut c,1,23);roll(&mut c,1);assert_eq!(c.state.enemies[1].intent_id.as_str(),"mystic_attack");
    let hp=c.state.player.hp;act(&mut c,1);assert_eq!(c.state.player.hp,hp-10);assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="frail"&&p.state.amount==2));
}

#[test]
fn m6_partner_roll_exhaustion_does_not_publish_or_mutate_sibling_cursor() {
    let mut c=Combat::enter(entry()).unwrap();stream(&mut c,1,1_u128<<64);let actor=Actor::Enemy(c.state.enemies[1].instance_id.clone());
    let before=(c.state.enemies[1].intent_id.clone(),c.state.enemies[1].move_history.clone(),c.state.enemies[1].full_memory.clone(),c.state.enemies[0].full_ai_rng.clone());
    assert!(c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RollMove}],None).is_err());
    assert_eq!((c.state.enemies[1].intent_id.clone(),c.state.enemies[1].move_history.clone(),c.state.enemies[1].full_memory.clone(),c.state.enemies[0].full_ai_rng.clone()),before);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::RngExhausted);
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);
}

#[test]
fn m6_selected_initial_memory_is_transferred_and_foreign_memory_rejects_before_items() {
    let mut e=super::m6_ordinary::entry(EncounterId::ThreeByrds);e.setup.enemies.truncate(1);
    e.setup.enemies[0].initial_memory=Some(EnemyMemory::Byrd{flying:false});e.setup.enemies[0].intent=FullIntentId::from_name("byrd_stunned");e.setup.enemies[0].powers.retain(|p|p.id.0!="flight");
    let mut c=Combat::enter(e).unwrap();c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.player.hp,80);assert_eq!(c.state.enemies[0].intent_id.as_str(),"byrd_headbutt","entry must not reconstruct flying=true and randomize instead");
    let mut e=entry();e.setup.enemies[1].initial_memory=Some(EnemyMemory::Cultist{ascension:0});
    let mut items=Some(crate::items::CombatItemContext::new(vec![RelicId::BurningBlood],vec![]));let before=items.clone();assert!(matches!(Combat::enter_with_item_context(e,&mut items),Err(CombatEntryFailure::Rejected(_))));assert_eq!(items,before);
}
