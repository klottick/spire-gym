//! Qualified final-source/dialog regressions; supplied late-state diagnostics only.
use crate::{combat::{CombatAction,Outcome},content::{EncounterId,RelicId},run::*};
use super::{rooms::choose,m9_boss::{source,hit}};

fn defeated(boss:EncounterId)->Run{
    let(mut r,node)=source(boss);r.step(&GameAction::Move(node)).unwrap();
    match boss{
        EncounterId::AwakenedOne=>{hit(&mut r,2,300);r.choose_combat(CombatAction::EndTurn).unwrap();hit(&mut r,2,300);},
        EncounterId::TimeEater=>hit(&mut r,0,456),
        EncounterId::DonuAndDeca=>{hit(&mut r,0,250);hit(&mut r,1,250);},
        _=>unreachable!(),
    }
    assert_eq!(r.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));r
}
fn victory(boss:EncounterId)->Run{let mut r=defeated(boss);r.settle_combat().unwrap();r}

#[test]
fn m9_boss_source_has_qualified_no_rewards_not_generic_chest(){
    for boss in [EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca]{
        let mut r=defeated(boss);let gold=r.state.gold;let rng=r.state.rng.clone();let deck=r.state.deck.clone();let settled=r.state.combat_progress.settled;
        let t=r.settle_combat().unwrap();assert_eq!(t.reward,0);
        assert_eq!((r.state.floor,r.state.gold,r.state.deck.clone()),(50,gold,deck));assert_eq!(r.state.rng,rng);
        assert_eq!(r.state.combat_progress.settled,settled+1);assert!(r.state.relics.contains(&RelicId::BlackBlood));
        let room=r.decision().unwrap().room.unwrap();assert_eq!(room.kind,RoomViewKind::Act3BossComplete);
        assert_eq!(room.choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Proceed]);
        assert!(r.state.room_loop.as_ref().unwrap().active_rewards.is_none());
    }
}

#[test]
fn m9_no_keys_waits_for_four_real_heart_choices_and_commits_non_goal_once(){
    let mut r=victory(EncounterId::TimeEater);assert_eq!(r.state.keys,[false;3]);
    let resources=(r.state.hp,r.state.max_hp,r.state.gold,r.state.deck.clone(),r.state.relics.clone(),r.state.potions.clone(),r.state.reward_ledger.total);
    let old=GameAction::Choose(r.decision().unwrap().room.unwrap().choices[0].id);
    assert_eq!(choose(&mut r,RoomChoiceKind::Proceed).reward,0);
    let frozen=r.clone();assert!(r.step(&old).is_err());assert!(r==frozen);
    for stage in 1..=4 {
        let screen=r.decision().unwrap().room.unwrap();assert_eq!(screen.kind,RoomViewKind::Act3Ending);
        assert_eq!(screen.choices[0].kind,RoomChoiceKind::Act3Continue{stage});
        let t=choose(&mut r,RoomChoiceKind::Act3Continue{stage});assert_eq!(t.reward,0);
        assert_eq!(t.terminal,if stage==4{Some(RunTerminal::Act3NoKeys)}else{None});
    }
    assert_eq!(r.state.terminal,Some(RunTerminal::Act3NoKeys));assert_eq!(r.state.floor,50);
    assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.deck.clone(),r.state.relics.clone(),r.state.potions.clone(),r.state.reward_ledger.total),resources);
    assert!(r.decision().unwrap().candidates.is_empty());
    assert_eq!(r.state.history.events.iter().filter(|e|matches!(e,CommittedEvent::Ended(RunTerminal::Act3NoKeys))).count(),1);
    let frozen=r.clone();assert!(r.step(&old).is_err());assert!(r==frozen);
}

#[test]
fn m9_all_actual_key_combinations_choose_only_the_qualified_boundary(){
    for mask in 0..8{
        let(mut r,node)=source(EncounterId::DonuAndDeca);assert_eq!(r.state.keys,[false;3]);
        for (index,key) in [Key::Ruby,Key::Sapphire,Key::Emerald].into_iter().enumerate(){if mask&(1<<index)!=0{let event=super::super::mutations::acquire_key(&mut r.state,key).unwrap();assert_eq!(super::super::reward::commit(&mut r.state,&[event]).unwrap(),5);}}
        r.step(&GameAction::Move(node)).unwrap();hit(&mut r,0,250);hit(&mut r,1,250);r.settle_combat().unwrap();
        let before=(r.state.hp,r.state.max_hp,r.state.gold,r.state.relics.clone(),r.state.potions.clone(),r.state.keys,r.state.reward_ledger.total);
        choose(&mut r,RoomChoiceKind::Proceed);for stage in 1..=3{assert_eq!(choose(&mut r,RoomChoiceKind::Act3Continue{stage}).reward,0);}
        assert_eq!((r.state.act,r.state.floor,r.state.terminal),(3,50,None));
        if mask==7{
            assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act4Ready);assert!(r.decision().unwrap().candidates.is_empty());
            let frozen=r.clone();assert!(r.step(&GameAction::DiscardPotion{slot:0}).is_err());assert!(r==frozen);
        }else{assert_eq!(r.decision().unwrap().room.unwrap().choices[0].kind,RoomChoiceKind::Act3Continue{stage:4});choose(&mut r,RoomChoiceKind::Act3Continue{stage:4});assert_eq!(r.state.terminal,Some(RunTerminal::Act3NoKeys));}
        assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.relics.clone(),r.state.potions.clone(),r.state.keys,r.state.reward_ledger.total),before);
        assert!(!r.state.history.events.iter().any(|e|matches!(e,CommittedEvent::ActAdvanced{from:3,..}|CommittedEvent::FloorEntered{floor:51,..}|CommittedEvent::Ended(RunTerminal::HeartVictory))));
    }
}

#[test]
fn m9_final_preparation_fault_keeps_actual_victory_inventory_and_census(){
    let mut r=defeated(EncounterId::TimeEater);r.next_choice=u64::MAX;let visits=r.state.combat_progress;let rng=r.state.rng.clone();let deck=r.state.deck.clone();
    let error=r.settle_combat().unwrap_err();assert!(matches!(&error,RunError::Faulted(f) if f.checkpoint=="run.m9.act3.end"));
    assert_eq!(r.state.combat_progress,visits);assert_eq!(r.state.rng,rng);assert_eq!(r.state.deck,deck);
    assert!(r.state.active_combat.is_some()&&r.state.relics.is_empty());assert!(r.state.combat.as_ref().unwrap().item_context().is_some());
    assert_eq!(r.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));
    let frozen=r.clone();assert_eq!(r.choose_combat(CombatAction::EndTurn),Err(error));assert!(r==frozen);
}

#[test]
fn m9_ending_cursor_preflight_is_atomic_and_accepted_frame_fault_is_retained(){
    let mut r=victory(EncounterId::TimeEater);r.next_choice=u64::MAX;let id=r.decision().unwrap().room.unwrap().choices[0].id;
    let frozen=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==frozen);
    let mut r=victory(EncounterId::TimeEater);choose(&mut r,RoomChoiceKind::Proceed);
    let screen=r.state.room_loop.as_ref().unwrap().screen.clone();let rng=r.state.rng.clone();r.state.map.as_mut().unwrap().current=None;
    let action=GameAction::Choose(screen.choices[0].id);let error=r.step(&action).unwrap_err();
    assert!(matches!(&error,RunError::Faulted(f) if f.checkpoint=="run.m9.act3.end"));
    assert_eq!(r.state.room_loop.as_ref().unwrap().screen,screen);assert_eq!(r.state.rng,rng);assert_eq!(r.state.terminal,None);
    let frozen=r.clone();assert_eq!(r.step(&action),Err(error));assert!(r==frozen);
}

#[test]
fn m9_full_belt_and_disabled_final_act_do_not_fabricate_readiness(){
    let(mut r,node)=source(EncounterId::TimeEater);r.state.keys=[true;3];r.state.act4_enabled=false;
    r.state.potions=vec![Some(crate::content::PotionId::FirePotion);3];
    r.step(&GameAction::Move(node)).unwrap();hit(&mut r,0,456);r.settle_combat().unwrap();
    let belt=r.state.potions.clone();choose(&mut r,RoomChoiceKind::Proceed);
    for stage in 1..=4{choose(&mut r,RoomChoiceKind::Act3Continue{stage});}
    assert_eq!(r.state.terminal,Some(RunTerminal::Act3NoKeys));assert_eq!(r.state.potions,belt);
    assert!(r.decision().unwrap().candidates.is_empty());
}

#[test]
fn m9_corrupt_empty_deck_cannot_be_promoted_from_victory_to_ending(){
    let mut r=defeated(EncounterId::TimeEater);r.state.deck.clear();let visits=r.state.combat_progress;
    assert!(matches!(r.settle_combat(),Err(RunError::Faulted(_))));
    assert_eq!(r.state.combat_progress,visits);assert!(r.state.combat.is_some());
    assert!(r.state.room_loop.as_ref().unwrap().act3_end.is_none());assert_eq!(r.state.terminal,None);
}
