use crate::{content::{EncounterId,EventId,RelicId},run::*};
use super::rooms::choose;

fn source()->Run{let mut r=super::m5_rewards::boss_chest();choose(&mut r,RoomChoiceKind::Relic(RelicId::SlaversCollar));r.state.seen_bosses[1]=vec![];r}
fn continuing()->Run{let mut r=source();r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M6Act2Prefix;choose(&mut r,RoomChoiceKind::Proceed);r}
fn capture_fixture()->Run{let mut r=source();choose(&mut r,RoomChoiceKind::Proceed);let room=r.state.room_loop.as_mut().unwrap();room.scenario=RoomScenario::M6Act2Prefix;room.act2=Some(Default::default());room.screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};r}

#[test]
fn m6_continuation_uses_earned_inventory_and_same_map(){
    let mut r=source();r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M6Act2Prefix;r.state.room_loop.as_mut().unwrap().purge_cost=125;r.state.hp=40;
    let resources=(r.state.deck.clone(),r.state.gold,r.state.relics.clone(),r.state.potions.clone(),r.state.relic_counters.clone(),r.state.next_instance);let reward=r.state.reward_ledger.total;let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0);assert_eq!((r.state.act,r.state.floor,r.state.hp,r.state.reward_ledger.total),(2,17,80,reward));assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    assert_eq!((r.state.deck.clone(),r.state.gold,r.state.relics.clone(),r.state.potions.clone(),r.state.relic_counters.clone(),r.state.next_instance),resources);assert_eq!(r.state.room_loop.as_ref().unwrap().purge_cost,125);assert!(r.state.room_loop.as_ref().unwrap().act2.is_some());let map=r.public_map().unwrap();let rng=r.state.rng.clone();let old=r.clone();let _=r.decision().unwrap();assert!(r==old);assert_eq!(r.state.rng,rng);assert_eq!(r.public_map().unwrap(),map);
    let action=r.decision().unwrap().candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).expect("M6 must publish first Act2 move").action;r.step(&action).unwrap();assert_eq!(r.state.floor,18);assert_eq!(r.state.map.as_ref().unwrap().regular_encounter_history,vec![EncounterId::SphericGuardian]);assert!(r.state.combat.is_some());assert!(r.state.seen_bosses[1].is_empty());
}

#[test]
fn m6_capture_respects_first_strong_and_question_eligibility(){
    let mut r=capture_fixture();let ordinary=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.kind==MapNodeKind::Monster).unwrap().id;let question=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.kind==MapNodeKind::Question).unwrap().id;
    let mut history=vec![];for want in [EncounterId::SphericGuardian,EncounterId::ThreeByrds,EncounterId::CenturionAndHealer,EncounterId::Snecko,EncounterId::ChosenAndByrds,EncounterId::ThreeCultists,EncounterId::CenturionAndHealer]{
        r.state.map.as_mut().unwrap().regular_encounter_history=history.clone();assert_eq!(super::super::rooms::act2::capture_room(&r.state,ordinary).unwrap().encounter,Some(want));history.push(want);
    }
    assert_eq!(super::super::rooms::act2::capture_room(&r.state,question).unwrap().event,Some(EventId::AncientWriting));r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().question_visits=1;
    for (gold,event) in [(49,EventId::Purifier),(50,EventId::Joust)]{r.state.gold=gold;assert_eq!(super::super::rooms::act2::capture_room(&r.state,question).unwrap().event,Some(event));}
    r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().question_visits=2;assert!(super::super::rooms::act2::capture_room(&r.state,question).unwrap().encounter.is_some());
}

#[test]
fn m6_prefix_cutoff_waits_for_room_children(){
    let mut r=continuing();let map=r.state.map.as_mut().unwrap();let chest=map.nodes.iter().find(|n|n.floor==26).unwrap().id;let prev=map.nodes.iter_mut().find(|n|n.edges.contains(&chest)).unwrap();prev.visited=true;map.current=Some(prev.id);r.state.floor=25;
    r.step(&GameAction::Move(chest)).unwrap();assert_eq!(r.state.floor,26);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Treasure);let before=r.state.reward_ledger.total;
    choose(&mut r,RoomChoiceKind::OpenChest);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Treasure);let relic=r.decision().unwrap().room.unwrap().choices.into_iter().find_map(|c|match c.kind{RoomChoiceKind::Relic(id)=>Some(id),_=>None}).unwrap();choose(&mut r,RoomChoiceKind::Relic(relic));
    let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!((t.reward,r.state.reward_ledger.total,r.state.terminal),(0,before,None));assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act2PrefixComplete);assert!(r.decision().unwrap().candidates.is_empty());let frozen=r.clone();assert!(r.step(&GameAction::Move(NodeId(108))).is_err());assert!(r==frozen);
}

#[test]
fn m6_invalid_move_preflight_is_atomic_but_transferred_entry_fault_is_retained(){
    let mut r=continuing();let frozen=r.clone();assert!(r.step(&GameAction::Move(NodeId(u64::MAX))).is_err());assert!(r==frozen);
    r.state.relics.push(RelicId::PureWater);r.state.deck=vec![CardInstance{ misc:0,instance:u64::MAX,id:crate::content::CardId::Strike,upgrades:0}];let action=r.decision().unwrap().candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action;let before=r.state.reward_ledger.total;
    let error=r.step(&action).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!((r.state.floor,r.state.reward_ledger.total,r.state.terminal),(18,before+1,None));assert!(r.state.combat.is_some());assert!(r.state.relics.is_empty());let frozen=r.clone();assert_eq!(r.step(&action),Err(error));assert!(r==frozen);
}

#[test]
fn m6_elite_role_controls_collar_burning_gold_and_independent_loot(){
    use crate::{combat::{CombatAction,TargetId},content::{CardId,EnemyId,PotionId}};
    let mut r=continuing();let map=r.state.map.as_mut().unwrap();let elite=map.nodes.iter().find(|n|n.kind==MapNodeKind::Elite&&n.floor<26).unwrap().id;let prev=map.nodes.iter_mut().find(|n|n.edges.contains(&elite)).unwrap();prev.visited=true;map.current=Some(prev.id);r.state.floor=prev.floor;for n in &mut map.nodes{n.burning=n.id==elite;}
    r.step(&GameAction::Move(elite)).unwrap();let d=r.decision().unwrap();let c=d.combat.unwrap();assert_eq!(c.player.max_energy,4);assert!(c.enemies.iter().all(|e|e.hp==1),"carried remaining Lament charges apply once to initial actual actors");let leader=c.enemies.iter().position(|e|e.enemy_id==EnemyId::GremlinLeader).unwrap();
    let target=TargetId(leader as u64+1);let a=d.candidates.iter().find_map(|c|match &c.action{GameAction::Combat(a@CombatAction::PlayCard{target:Some(t),..}) if *t==target=>Some(a.clone()),_=>None}).unwrap();r.choose_combat(a).unwrap();
    let room=r.decision().unwrap().room.unwrap();let gold=room.choices.iter().find_map(|c|match c.kind{RoomChoiceKind::Gold(n)=>Some(n),_=>None}).unwrap();assert!((25..=35).contains(&gold));assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Emerald));assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::Lantern)));assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Potion(PotionId::FirePotion)&&c.status==RoomChoiceStatus::Disabled));
    assert_eq!(r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().combat_visits,1);assert_eq!(r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().elite_relic_cursor,1);choose(&mut r,RoomChoiceKind::CardReward);assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Card{id:CardId::IronWave,upgrades:0}));choose(&mut r,RoomChoiceKind::Skip);choose(&mut r,RoomChoiceKind::Emerald);choose(&mut r,RoomChoiceKind::Proceed);assert!(!r.state.relics.contains(&RelicId::Lantern));assert!(r.state.keys[2]);
}

#[test]
fn m6_all_generated_edges_remain_visible_and_source_cage_child_blocks_advance(){
    let r=continuing();let map=r.public_map().unwrap();let offered=r.decision().unwrap().candidates.into_iter().filter_map(|c|match c.action{GameAction::Move(id)=>Some(id),_=>None}).collect::<Vec<_>>();let entries=map.nodes.iter().filter(|n|n.y==0).map(|n|n.id).collect::<Vec<_>>();assert_eq!(offered,entries);
    let mut r=super::m5_rewards::boss_chest();r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M6Act2Prefix;choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));let frozen=r.clone();assert!(r.step(&GameAction::Choose(ChoiceId(u64::MAX))).is_err());assert!(r==frozen);assert_eq!((r.state.act,r.state.floor),(1,17));for _ in 0..2{let id=r.state.pending.as_ref().unwrap().selection.candidates[0].id;r.step(&GameAction::Select(id)).unwrap();}choose(&mut r,RoomChoiceKind::Proceed);assert_eq!((r.state.act,r.state.floor),(2,17));assert!(r.decision().unwrap().candidates.iter().any(|c|matches!(c.action,GameAction::Move(_))));
}

#[test]
fn m6_chest_skip_is_a_real_source_proceed_not_a_published_rejected_action(){
    let mut r=continuing();let map=r.state.map.as_mut().unwrap();let chest=map.nodes.iter().find(|n|n.floor==26).unwrap().id;let prev=map.nodes.iter_mut().find(|n|n.edges.contains(&chest)).unwrap();prev.visited=true;map.current=Some(prev.id);r.state.floor=25;r.step(&GameAction::Move(chest)).unwrap();
    let relics=r.state.relics.clone();let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0);assert_eq!(r.state.relics,relics);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act2PrefixComplete);assert!(r.decision().unwrap().candidates.is_empty());
}

#[test]
fn m6_room_token_and_visit_overflow_reject_before_floor_credit(){
    let mut r=continuing();let map=r.state.map.as_mut().unwrap();let question=map.nodes.iter().find(|n|n.kind==MapNodeKind::Question&&n.floor<26).unwrap().id;let prev=map.nodes.iter_mut().find(|n|n.edges.contains(&question)).unwrap();prev.visited=true;map.current=Some(prev.id);r.state.floor=prev.floor;
    r.next_choice=u64::MAX;let before=r.clone();assert!(r.step(&GameAction::Move(question)).is_err());assert!(r==before);r.next_choice=1000;r.state.room_loop.as_mut().unwrap().act2.as_mut().unwrap().question_visits=usize::MAX;let before=r.clone();assert!(r.step(&GameAction::Move(question)).is_err());assert!(r==before);
}

#[test]
fn m6_real_act1_prefix_choices_and_resources_match_m5(){
    use crate::{combat::CombatAction,content::Character};
    let mut a=super::m5_act1::run();let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=vec!["Guardian","Hexaghost","SlimeBoss"];let mut b=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},p,RunRng::new([7;32],crate::combat::rng::Partition::Test,0),RoomScenario::M6Act2Prefix).unwrap();choose(&mut a,RoomChoiceKind::NeowLament);choose(&mut b,RoomChoiceKind::NeowLament);
    for _ in 0..128 {
        let da=a.decision().unwrap();let mut db=b.decision().unwrap();db.content_profile=Some(RoomScenario::M5Act1Captured);assert_eq!(da,db);assert_eq!(a.public_map(),b.public_map());if a.state.combat_progress.settled==3{break}
        let action=if da.combat.is_some(){da.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(CombatAction::PlayCard{target:Some(_),..}))).map(|c|c.action.clone()).unwrap_or(GameAction::Combat(CombatAction::EndTurn))}else{match da.room.as_ref().unwrap().kind{
            RoomViewKind::Map=>da.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone(),
            kind=>{let wanted=if kind==RoomViewKind::Rewards{RoomChoiceKind::Proceed}else{RoomChoiceKind::Leave};let id=da.room.as_ref().unwrap().choices.iter().find(|c|c.kind==wanted).unwrap().id;GameAction::Choose(id)},
        }};assert_eq!(a.step(&action),b.step(&action));
    }
    assert_eq!(b.state.combat_progress,CombatProgress{entered:3,settled:3});
}

#[test]
fn m6_skipped_relic_offer_advances_and_combat_loot_uses_act_local_visit(){
    use crate::{combat::CombatAction,content::PotionId};
    let mut r=continuing();
    for (visit,relic,potion) in [(1,RelicId::Lantern,PotionId::FirePotion),(2,RelicId::Pear,PotionId::StrengthPotion)]{
        if visit==1{let map=r.state.map.as_mut().unwrap();let elite=map.nodes.iter().find(|n|n.kind==MapNodeKind::Elite&&n.floor<26).unwrap().id;let prev=map.nodes.iter_mut().find(|n|n.edges.contains(&elite)).unwrap();prev.visited=true;map.current=Some(prev.id);r.state.floor=prev.floor;r.step(&GameAction::Move(elite)).unwrap();}
        else{
            // Explicit native loot-consumer diagnostic, not a second legal map
            // visit: this generated prefix need not contain two Elite rooms.
            let node=r.state.map.as_ref().unwrap().current.unwrap();r.enter_owned(EncounterId::Slavers,CombatOrigin::MapRoom{node,kind:MapNodeKind::Elite}).unwrap();
        }
        for _ in 0..32{let d=r.decision().unwrap();if d.combat.is_none(){break}let action=d.candidates.iter().find_map(|c|match &c.action{GameAction::Combat(a@CombatAction::PlayCard{target:Some(_),..})=>Some(a.clone()),_=>None}).unwrap_or(CombatAction::EndTurn);r.choose_combat(action).unwrap();}
        assert!(r.state.combat.is_none());let room=r.decision().unwrap().room.unwrap();assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(relic)));assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Potion(potion)));assert_eq!(r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().combat_visits,visit);choose(&mut r,RoomChoiceKind::Proceed);assert!(!r.state.relics.contains(&relic));
    }
    assert_eq!(r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().elite_relic_cursor,2);
}

#[test]
fn m6_actual_death_settles_act_local_combat_visit_without_cutoff_or_loot(){
    let mut r=continuing();let action=r.decision().unwrap().candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action;r.step(&action).unwrap();
    for _ in 0..24{if r.state.terminal.is_some(){break}r.choose_combat(crate::combat::CombatAction::EndTurn).unwrap();}
    assert_eq!(r.state.terminal,Some(RunTerminal::Death));assert_eq!(r.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Death);assert_eq!(r.state.room_loop.as_ref().unwrap().act2.as_ref().unwrap().combat_visits,1);assert_ne!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act2PrefixComplete);assert!(r.decision().unwrap().candidates.is_empty());
}
