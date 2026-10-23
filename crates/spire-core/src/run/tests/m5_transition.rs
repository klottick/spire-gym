use crate::{content::{CardId,EncounterId,RelicId},run::*};
use super::rooms::choose;

fn source()->Run {
    let mut r=super::m5_rewards::victory(MapNodeKind::Boss,false);choose(&mut r,RoomChoiceKind::Gold(100));choose(&mut r,RoomChoiceKind::CardReward);choose(&mut r,RoomChoiceKind::Card{id:CardId::Offering,upgrades:0});
    choose(&mut r,RoomChoiceKind::Proceed);choose(&mut r,RoomChoiceKind::OpenChest);choose(&mut r,RoomChoiceKind::Relic(RelicId::SlaversCollar));r
}

#[test]
fn act2_ready_requires_resolved_source_children() {
    let mut r=super::m5_rewards::boss_chest();choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));
    let before=r.clone();assert!(r.step(&GameAction::Choose(ChoiceId(u64::MAX))).is_err());assert!(r==before);assert_eq!(r.state.act,1);
    for _ in 0..2{let id=r.state.pending.as_ref().unwrap().selection.candidates[0].id;r.step(&GameAction::Select(id)).unwrap();}
    let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0);assert_eq!(t.committed_events,vec![CommittedEvent::ActAdvanced{from:1,to:2}]);
    assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act2Ready);assert_eq!(r.state.terminal,None);
}

#[test]
fn act_transition_preserves_identity_and_heals_once() {
    for bloom in [false,true] {
        let mut r=source();r.state.hp=40;r.state.max_hp=88;r.state.deck[0].upgrades=1;r.state.room_loop.as_mut().unwrap().purge_cost=100;
        if bloom{r.state.relics.push(RelicId::MarkOfBloom)}
        let resources=(r.state.deck.clone(),r.state.gold,r.state.relics.clone(),r.state.potions.clone(),r.state.relic_counters.clone(),r.state.keys,r.state.next_instance,r.next_choice,r.state.combat_progress);
        let before_reward=r.state.reward_ledger.total;let t=choose(&mut r,RoomChoiceKind::Proceed);
        assert_eq!((r.state.act,r.state.floor,r.state.hp,r.state.max_hp),(2,17,if bloom{40}else{88},88));assert_eq!((t.reward,r.state.reward_ledger.total),(0,before_reward));
        assert_eq!((r.state.deck.clone(),r.state.gold,r.state.relics.clone(),r.state.potions.clone(),r.state.relic_counters.clone(),r.state.keys,r.state.next_instance,r.next_choice,r.state.combat_progress),resources);
        assert_eq!(r.state.room_loop.as_ref().unwrap().purge_cost,100);let before=r.clone();assert!(r.step(&GameAction::Choose(ChoiceId(0))).is_err());assert!(r==before);
    }
}

#[test]
fn act_transition_fault_is_not_terminal() {
    let mut r=source();r.state.hp=40;r.state.max_hp=88;
    // Internal retained-frame corruption after real source relic acquisition.
    // No public action accepts a target act override.
    r.state.room_loop.as_mut().unwrap().transition=Some(ActTransition{from:1,to:3,floor:17,reset_applied:false,heal_applied:false,initialized:false});
    let result=super::super::rooms::transition::drive(&mut r);assert!(matches!(result,Err(RunError::Faulted(_))));assert_eq!(r.state.hp,88);assert_eq!(r.state.terminal,None);
    let frame=r.state.room_loop.as_ref().unwrap().transition.as_ref().unwrap();assert!(frame.reset_applied&&frame.heal_applied&&!frame.initialized);
    let before=r.clone();assert_eq!(r.step(&GameAction::Choose(ChoiceId(0))),result);assert!(r==before);assert!(!r.state.history.events.iter().any(|e|matches!(e,CommittedEvent::ActAdvanced{..})));
}

#[test]
fn act2_initialization_has_distinct_rng_and_no_menu() {
    for (seen,boss) in [(vec![],EncounterId::Champ),(vec!["Champ"],EncounterId::BronzeAutomaton),(vec!["Champ","Automaton"],EncounterId::Collector)] {
        let mut r=source();r.state.seen_bosses[1]=seen.clone();let mut before=r.state.rng.clone();choose(&mut r,RoomChoiceKind::Proceed);
        let mut after=r.state.rng.clone();assert_eq!(before.draw(RunRngScope::Act2BossOrder),after.draw(RunRngScope::Act2BossOrder));
        assert_eq!(before.draw_scoped(RunRngScope::MapTopology,&[1,0,0,0]),after.draw_scoped(RunRngScope::MapTopology,&[1,0,0,0]));
        let map=r.public_map().unwrap();assert_eq!(map.act,2);assert_eq!(map.boss,Some(boss));assert!(map.current.is_none());assert_eq!(map.nodes.iter().filter(|n|n.y==0).map(|n|n.floor).collect::<Vec<_>>(),vec![18;6]);assert_eq!(map.nodes.iter().find(|n|n.kind==MapNodeKind::Boss).unwrap().floor,33);
        assert_eq!(r.state.seen_bosses[1],seen,"initialization is not Act2 boss discovery");assert!(r.decision().unwrap().candidates.is_empty(),"full potion belt must not leak an Act2 action");
        let before=r.clone();assert!(r.step(&GameAction::Move(NodeId(0))).is_err());assert!(r==before);
    }
}

#[test]
fn act2_all_seen_shuffle_is_once_and_skip_confirmation_is_a_real_source_action() {
    let mut r=super::m5_rewards::boss_chest();choose(&mut r,RoomChoiceKind::Skip);
    let mut expected=r.state.rng.clone();expected.bounded(RunRngScope::Act2BossOrder,3);expected.bounded(RunRngScope::Act2BossOrder,2);
    let t=choose(&mut r,RoomChoiceKind::ConfirmSkip);assert_eq!(t.reward,0);let mut after=r.state.rng.clone();assert_eq!(expected.draw(RunRngScope::Act2BossOrder),after.draw(RunRngScope::Act2BossOrder));
    let order=r.state.room_loop.as_ref().unwrap().act2_boss_order.as_ref().unwrap();assert_eq!(order.len(),3);for boss in [EncounterId::Champ,EncounterId::BronzeAutomaton,EncounterId::Collector]{assert_eq!(order.iter().filter(|b|**b==boss).count(),1);}
    assert_eq!(r.state.act,2);assert_eq!(r.state.terminal,None);assert!(!r.state.relics.iter().any(|id|matches!(id,RelicId::BlackBlood|RelicId::EmptyCage|RelicId::SlaversCollar)));
}

#[test]
fn invalid_pre_proceed_resources_leave_the_source_and_rng_unchanged() {
    let mut r=source();r.state.hp=0;let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Proceed).unwrap().id;
    let before=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before);assert!(r.state.room_loop.as_ref().unwrap().transition.is_none());
}
