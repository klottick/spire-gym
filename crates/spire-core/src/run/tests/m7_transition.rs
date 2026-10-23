use crate::{content::{EncounterId,RelicId},run::*};
use super::rooms::choose;

fn source()->Run {
    let mut r=super::m7_rewards::chest();choose(&mut r,RoomChoiceKind::Relic(RelicId::BlackBlood));r
}

#[test]
fn m7_act3_ready_waits_for_actual_source_proceed() {
    for take in [true,false] {
        let mut r=super::m7_rewards::chest();
        if take {
            choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));
            assert_eq!((r.state.act,r.state.floor),(2,34));
            let old=r.clone();assert!(r.step(&GameAction::Choose(ChoiceId(u64::MAX))).is_err());assert!(r==old);
            for _ in 0..2 {let id=r.state.pending.as_ref().unwrap().selection.candidates[0].id;r.step(&GameAction::Select(id)).unwrap();}
        } else {choose(&mut r,RoomChoiceKind::Skip);}
        assert!(r.state.room_loop.as_ref().unwrap().transition.as_ref().unwrap().initialized,"actual predecessor1→2 frame must still be retained");
        let t=choose(&mut r,if take{RoomChoiceKind::Proceed}else{RoomChoiceKind::ConfirmSkip});
        assert_eq!(t.reward,0);assert_eq!(t.committed_events,vec![CommittedEvent::ActAdvanced{from:2,to:3}]);
        let d=r.decision().unwrap();assert_eq!((d.act,d.floor,d.terminal),(3,34,None));
        assert_eq!(d.room.unwrap().kind,RoomViewKind::Act3Ready);assert!(d.candidates.is_empty());
        let old=r.clone();assert!(r.step(&GameAction::DiscardPotion{slot:0}).is_err());assert!(r==old);
    }
}

#[test]
fn m7_act3_transition_preserves_resources_and_heals_once() {
    for bloom in [false,true] {
        let mut r=source();r.state.hp=40;r.state.max_hp=88;r.state.deck[0].upgrades=1;r.state.keys=[true,true,true];
        r.state.room_loop.as_mut().unwrap().purge_cost=125;if bloom{r.state.relics.push(RelicId::MarkOfBloom);}
        let resources=(r.state.deck.clone(),r.state.gold,r.state.relics.clone(),r.state.potions.clone(),r.state.relic_counters.clone(),r.state.keys,r.state.next_instance,r.next_choice,r.state.combat_progress);
        let reward=r.state.reward_ledger.total;let history=r.state.history.events.clone();
        choose(&mut r,RoomChoiceKind::Proceed);
        assert_eq!((r.state.act,r.state.floor,r.state.hp,r.state.max_hp,r.state.reward_ledger.total),(3,34,if bloom{40}else{88},88,reward));
        assert_eq!((r.state.deck.clone(),r.state.gold,r.state.relics.clone(),r.state.potions.clone(),r.state.relic_counters.clone(),r.state.keys,r.state.next_instance,r.next_choice,r.state.combat_progress),resources);
        assert_eq!(r.state.room_loop.as_ref().unwrap().purge_cost,125);assert_eq!(&r.state.history.events[..history.len()],history.as_slice());
        assert!(r.state.room_loop.as_ref().unwrap().act2.is_none());assert_eq!(r.state.room_loop.as_ref().unwrap().joust_visits,0);
        let old=r.clone();assert!(r.step(&GameAction::Choose(ChoiceId(0))).is_err());assert!(r==old);
        assert_eq!(r.state.history.events.iter().filter(|e|matches!(e,CommittedEvent::ActAdvanced{from:2,to:3})).count(),1);
    }
    let mut r=source();r.state.hp=0;let old=r.clone();let id=r.decision().unwrap().room.unwrap().choices[0].id;
    assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==old);
}

#[test]
fn m7_act3_initialization_has_distinct_map_and_boss_streams() {
    let scope=RunRngScope::parse("run.act3.boss_order").expect("registered third-act native scope");
    for (seen,want) in [(vec![],EncounterId::AwakenedOne),(vec!["AwakenedOne"],EncounterId::DonuAndDeca),(vec!["AwakenedOne","DonuAndDeca"],EncounterId::TimeEater)] {
        let mut r=source();r.state.seen_bosses[2]=seen.clone();let mut before=r.state.rng.clone();
        choose(&mut r,RoomChoiceKind::Proceed);let map=r.public_map().unwrap();assert_eq!((map.act,map.boss,map.current),(3,Some(want),None));
        assert!(map.nodes.iter().filter(|n|n.y==0).all(|n|n.floor==35));assert_eq!(map.nodes.iter().find(|n|n.kind==MapNodeKind::Boss).unwrap().floor,50);
        assert_eq!(r.state.seen_bosses[2],seen);let mut after=r.state.rng.clone();
        assert_eq!(before.draw(scope),after.draw(scope),"unseen selection draws no shuffle");
        assert_eq!(before.draw(RunRngScope::Act2BossOrder),after.draw(RunRngScope::Act2BossOrder));
        assert_eq!(before.draw_scoped(RunRngScope::MapTopology,&[2,0,0,0]),after.draw_scoped(RunRngScope::MapTopology,&[2,0,0,0]));
    }
    let mut r=source();r.state.seen_bosses[2]=vec!["AwakenedOne","DonuAndDeca","TimeEater"];
    let mut expected=r.state.rng.clone();let mut order=vec![EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca];
    let j=expected.bounded(scope,3);order.swap(2,j);let j=expected.bounded(scope,2);order.swap(1,j);
    choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.public_map().unwrap().boss,Some(order[0]));
    let mut after=r.state.rng.clone();assert_eq!(expected.draw(scope),after.draw(scope));
}

#[test]
fn m7_partial_transition_fault_retains_progress_not_success() {
    let mut r=source();r.state.hp=40;r.state.max_hp=88;
    // Internal damaged accepted frame: reset/heal work is real and cannot roll back.
    r.state.room_loop.as_mut().unwrap().transition=Some(ActTransition{from:2,to:3,floor:35,reset_applied:false,heal_applied:false,initialized:false});
    let err=super::super::rooms::transition::drive(&mut r).unwrap_err();assert!(matches!(err,RunError::Faulted(_)));
    assert_eq!((r.state.act,r.state.floor,r.state.hp,r.state.terminal),(3,34,88,None));
    let frame=r.state.room_loop.as_ref().unwrap().transition.as_ref().unwrap();assert!(frame.reset_applied&&frame.heal_applied&&!frame.initialized);
    assert!(!r.state.history.events.iter().any(|e|matches!(e,CommittedEvent::ActAdvanced{from:2,to:3})));
    let old=r.clone();assert_eq!(r.step(&GameAction::Choose(ChoiceId(0))),Err(err));assert!(r==old);
    let mut r=source();r.state.room_loop.as_mut().unwrap().transition.as_mut().unwrap().initialized=false;
    let old=r.clone();let id=r.decision().unwrap().room.unwrap().choices[0].id;assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==old,"cannot overwrite an incomplete predecessor");
}
