//! Captured-room diagnostics; never substitute these supplied late sources for normal-start evidence.
use crate::{combat::CombatAction,content::{EncounterId,RelicId},run::*};
use super::rooms::choose;
use super::super::rooms::act3;

pub(super) fn ready()->Run {
    let mut r=super::m7_rewards::chest();choose(&mut r,RoomChoiceKind::Relic(RelicId::BlackBlood));
    r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M8Act3Prefix;
    choose(&mut r,RoomChoiceKind::Proceed);
    r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M9IroncladAct3;r
}
pub(super) fn before(r:&mut Run,target:NodeId){
    let parent=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.edges.contains(&target)).map(|n|n.id);
    let map=r.state.map.as_mut().unwrap();map.current=parent;
    if let Some(id)=parent{map.nodes.iter_mut().find(|n|n.id==id).unwrap().visited=true;}
    r.state.room_loop.as_mut().unwrap().screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};
}

#[test]
fn m9_transition_reuses_earned_player_and_maps_instead_of_old_readiness(){
    let mut r=super::m7_rewards::chest();choose(&mut r,RoomChoiceKind::Relic(RelicId::BlackBlood));
    r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M9IroncladAct3;
    let deck=r.state.deck.clone();let gold=r.state.gold;let keys=r.state.keys;
    let t=choose(&mut r,RoomChoiceKind::Proceed);
    assert_eq!((r.state.act,r.state.floor,t.reward),(3,34,0));
    assert_eq!((r.state.deck.clone(),r.state.gold,r.state.keys),(deck,gold,keys));
    assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    assert!(r.state.room_loop.as_ref().unwrap().act3.is_some());
}

#[test]
fn m9_all_late_act3_edges_preserve_capture_owners(){
    let r=ready();let nodes=r.state.map.as_ref().unwrap().nodes.iter().filter(|n|(44..=49).contains(&n.floor)).cloned().collect::<Vec<_>>();
    assert!(!nodes.is_empty());
    for node in nodes {
        let mut c=r.clone();before(&mut c,node.id);
        c.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().question_visits=2;
        let p=c.state.room_loop.as_ref().unwrap().act3.clone().unwrap();let purge=c.state.room_loop.as_ref().unwrap().purge_cost;
        let captured=act3::capture_room(&c.state,node.id).unwrap();
        let t=c.step(&GameAction::Move(node.id)).unwrap();assert_eq!((c.state.floor,t.reward),(node.floor,1));
        assert_eq!(c.state.room_loop.as_ref().unwrap().purge_cost,purge);
        let after=c.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap();
        assert_eq!(after.elite_relic_cursor,p.elite_relic_cursor);
        assert_eq!(after.question_visits,p.question_visits+usize::from(node.kind==MapNodeKind::Question));
        assert_eq!(after.shop_visits,p.shop_visits+usize::from(captured.kind==MapNodeKind::Shop));
        if let Some(encounter)=captured.encounter{assert_eq!(c.state.active_combat.as_ref().unwrap().encounter,encounter);}
        else{assert!(c.state.combat.is_none());assert_eq!(c.decision().unwrap().room.unwrap().kind,match captured.kind{MapNodeKind::Rest=>RoomViewKind::Campfire,MapNodeKind::Shop=>RoomViewKind::Shop,_=>panic!("unexpected late capture")});}
        if node.floor==49{assert_eq!(captured.kind,MapNodeKind::Rest);}
    }
}

#[test]
fn m9_chest43_continues_but_m8_still_stops_and_stale_move_rejects(){
    for scenario in [RoomScenario::M9IroncladAct3,RoomScenario::M8Act3Prefix]{
        let mut r=ready();r.state.room_loop.as_mut().unwrap().scenario=scenario;
        let chest=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.floor==43).unwrap().id;
        before(&mut r,chest);r.step(&GameAction::Move(chest)).unwrap();let visits=r.state.room_loop.as_ref().unwrap().act3.clone();
        let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0);
        assert_eq!(r.decision().unwrap().room.unwrap().kind,if scenario==RoomScenario::M8Act3Prefix{RoomViewKind::Act3PrefixComplete}else{RoomViewKind::Map});
        assert_eq!(r.state.room_loop.as_ref().unwrap().act3,visits);
        if scenario==RoomScenario::M8Act3Prefix{assert!(r.decision().unwrap().candidates.is_empty());}
        let frozen=r.clone();assert!(r.step(&GameAction::Move(chest)).is_err());assert!(r==frozen);
    }
}

#[test]
fn m9_elite_stock_covers_five_wins_after_old_stock_owned(){
    let mut owned=vec![RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::MagicFlower,RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial];
    let mut cursor=0;
    for (index,want) in [RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,RelicId::BagOfMarbles,RelicId::BronzeScales].into_iter().enumerate(){
        let mut r=ready();for id in &owned{if !r.state.relics.contains(id){crate::items::acquire_relic(&mut r.state,*id).unwrap();}}
        if !r.state.relics.contains(&RelicId::NeowsLament){crate::items::acquire_relic(&mut r.state,RelicId::NeowsLament).unwrap();}
        r.state.relic_counters.insert(RelicId::NeowsLament,3);r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().elite_relic_cursor=cursor;
        r.state.map.as_mut().unwrap().boss_icon=Some(EncounterId::AwakenedOne);
        let node=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.kind==MapNodeKind::Elite).unwrap().id;before(&mut r,node);r.step(&GameAction::Move(node)).unwrap();
        let action=r.decision().unwrap().candidates.into_iter().find_map(|c|match c.action{GameAction::Combat(a@CombatAction::PlayCard{target:Some(_),..})=>Some(a),_=>None}).unwrap();
        r.choose_combat(action).unwrap();assert!(r.state.combat.is_none());
        let offered=r.decision().unwrap().room.unwrap().choices.into_iter().find_map(|c|match c.kind{RoomChoiceKind::Relic(id)=>Some(id),_=>None}).unwrap();assert_eq!(offered,want);
        choose(&mut r,RoomChoiceKind::Relic(want));assert!(r.state.relics.contains(&want));owned.push(want);
        cursor=r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().elite_relic_cursor;assert_eq!(cursor,9+index);
    }
    assert!(super::super::rooms::rewards::capture_elite_relic(8,&owned).is_err(),"M8's capture did not grow");
    assert!(super::super::rooms::rewards::capture_m9_elite_relic(cursor,&owned).is_err(),"no fabricated refill after the finite thirteen identities");
}

#[test]
fn m9_exhausted_chest_opens_without_duplicate_key_or_relic(){
    let mut r=ready();for id in [RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial]{if !r.state.relics.contains(&id){crate::items::acquire_relic(&mut r.state,id).unwrap();}}
    r.state.keys[1]=true;let chest=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.floor==43).unwrap().id;before(&mut r,chest);r.step(&GameAction::Move(chest)).unwrap();
    let resources=(r.state.relics.clone(),r.state.keys,r.state.reward_ledger.total);
    choose(&mut r,RoomChoiceKind::OpenChest);
    let screen=r.decision().unwrap().room.unwrap();
    assert!(screen.choices.iter().all(|c|c.kind==RoomChoiceKind::Proceed||c.status==RoomChoiceStatus::Disabled));
    choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    assert_eq!((r.state.relics.clone(),r.state.keys,r.state.reward_ledger.total),resources);
}

#[test]
fn m9_late_preflight_overflow_does_not_move_or_reroll(){
    let mut r=ready();let node=r.state.map.as_ref().unwrap().nodes.iter().find(|n|(44..49).contains(&n.floor)&&n.kind==MapNodeKind::Shop).unwrap().id;
    before(&mut r,node);r.next_choice=u64::MAX;let frozen=r.clone();assert!(r.step(&GameAction::Move(node)).is_err());assert!(r==frozen);
}

#[test]
fn m9_generated_routes_obey_the_five_elite_stock_bound(){
    let r=ready();let map=r.public_map().unwrap();let mut most=std::collections::BTreeMap::new();
    for node in &map.nodes {
        if node.kind==MapNodeKind::Elite {
            assert!((5..=13).contains(&node.y)&&node.y!=8);
            assert!(node.edges.iter().all(|id|map.nodes.iter().find(|n|n.id==*id).unwrap().kind!=MapNodeKind::Elite));
        }
        let parent=map.nodes.iter().filter(|n|n.edges.contains(&node.id)).filter_map(|n|most.get(&n.id).copied()).max().unwrap_or(0);
        let count=parent+usize::from(node.kind==MapNodeKind::Elite);assert!(count<=5);most.insert(node.id,count);
    }
}
