use crate::{combat::CombatAction,content::{CardId,EncounterId,PotionId,RelicId},run::*};
use super::rooms::choose;
use super::super::rooms::act3;

// Supplied source-room diagnostics; Task12 owns normal-start episode evidence.
fn source()->Run{let mut r=super::m7_rewards::chest();choose(&mut r,RoomChoiceKind::Relic(RelicId::BlackBlood));r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M8Act3Prefix;r}
fn ready()->Run{let mut r=source();choose(&mut r,RoomChoiceKind::Proceed);r}
fn before(r:&mut Run,target:NodeId){let parent=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.edges.contains(&target)).map(|n|n.id);let map=r.state.map.as_mut().unwrap();map.current=parent;if let Some(id)=parent{map.nodes.iter_mut().find(|n|n.id==id).unwrap().visited=true;}r.state.room_loop.as_mut().unwrap().screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};}

#[test]
fn m8_transition_continues_same_player_without_rehealing(){
    let mut r=source();r.state.hp=40;r.state.deck[0].upgrades=1;r.state.deck.push(CardInstance{id:CardId::Parasite,instance:900,upgrades:0});r.state.next_instance=901;r.state.keys=[true,false,true];r.state.room_loop.as_mut().unwrap().purge_cost=125;let resources=(r.state.deck.clone(),r.state.gold,r.state.relics.clone(),r.state.relic_counters.clone(),r.state.potions.clone(),r.state.keys,r.state.next_instance,r.state.combat_progress);let reward=r.state.reward_ledger.total;
    let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0);assert_eq!((r.state.act,r.state.floor,r.state.hp),(3,34,r.state.max_hp));assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);assert_eq!(r.state.reward_ledger.total,reward);assert_eq!((r.state.deck.clone(),r.state.gold,r.state.relics.clone(),r.state.relic_counters.clone(),r.state.potions.clone(),r.state.keys,r.state.next_instance,r.state.combat_progress),resources);assert_eq!(r.state.room_loop.as_ref().unwrap().purge_cost,125);assert!(r.state.room_loop.as_ref().unwrap().act2.is_none());
    let map=r.public_map().unwrap();assert!(map.nodes.iter().filter(|n|n.y==0).all(|n|n.floor==35));assert_eq!(map.nodes.iter().find(|n|n.kind==MapNodeKind::Boss).unwrap().floor,50);assert!(!r.decision().unwrap().candidates.is_empty());let frozen=r.clone();assert!(super::super::rooms::transition::drive(&mut r).is_err());assert!(r==frozen);
}

#[test]
fn m8_all_prefix_edges_keep_local_owners(){
    let r=ready();let nodes=r.state.map.as_ref().unwrap().nodes.iter().filter(|n|(35..=43).contains(&n.floor)).cloned().collect::<Vec<_>>();assert!(!nodes.is_empty());
    for node in nodes{let mut c=r.clone();before(&mut c,node.id);let captured=act3::capture_room(&c.state,node.id).unwrap();let gold=c.state.gold;let deck=c.state.deck.clone();let t=c.step(&GameAction::Move(node.id)).unwrap();assert_eq!(t.reward,1);assert_eq!((c.state.act,c.state.floor,c.state.gold),(3,node.floor,gold));assert_eq!(c.state.deck,deck);
        if let Some(encounter)=captured.encounter{let a=c.state.active_combat.as_ref().unwrap();assert_eq!(a.encounter,encounter);assert!(matches!(a.origin,super::super::CombatOrigin::MapRoom{node:id,kind} if id==node.id&&kind==captured.kind));assert_eq!(c.state.combat_progress.entered,c.state.combat_progress.settled+1);}else{assert!(c.state.combat.is_none());assert_eq!(c.decision().unwrap().room.unwrap().kind,if captured.kind==MapNodeKind::Treasure{RoomViewKind::Treasure}else if captured.kind==MapNodeKind::Shop{RoomViewKind::Shop}else if captured.kind==MapNodeKind::Rest{RoomViewKind::Campfire}else{RoomViewKind::Event(captured.event.unwrap())});}
    }
}

#[test]
fn m8_source_loot_and_capture_exhaustion(){
    let mut r=ready();if !r.state.relics.contains(&RelicId::NeowsLament){r.state.relics.push(RelicId::NeowsLament);}r.state.relic_counters.insert(RelicId::NeowsLament,3);let target=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.y==0).unwrap().id;r.step(&GameAction::Move(target)).unwrap();assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,EncounterId::OrbWalker);
    let d=r.decision().unwrap();let a=d.candidates.into_iter().find_map(|c|match c.action{GameAction::Combat(a@CombatAction::PlayCard{target:Some(_),..})=>Some(a),_=>None}).unwrap();r.choose_combat(a).unwrap();assert!(r.state.combat.is_none());let potion=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|matches!(c.kind,RoomChoiceKind::Potion(_))).unwrap();assert_eq!(potion.kind,RoomChoiceKind::Potion(PotionId::FirePotion));assert_eq!(potion.status,RoomChoiceStatus::Disabled);let frozen=r.clone();assert!(r.step(&GameAction::Choose(potion.id)).is_err());assert!(r==frozen);
    choose(&mut r,RoomChoiceKind::CardReward);assert!(r.decision().unwrap().room.unwrap().choices.iter().any(|c|c.kind==RoomChoiceKind::Card{id:CardId::Inflame,upgrades:0}));choose(&mut r,RoomChoiceKind::Card{id:CardId::Inflame,upgrades:0});assert!(r.state.deck.iter().any(|c|c.id==CardId::Inflame));
    assert!(super::super::rooms::rewards::capture_elite_relic(8,&r.state.relics).is_err());
}

#[test]
fn m8_chest_proceed_is_actual_cutoff(){
    for take in [0,1,2]{let mut r=ready();let chest=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.floor==43&&n.kind==MapNodeKind::Treasure).unwrap().id;before(&mut r,chest);r.step(&GameAction::Move(chest)).unwrap();assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Treasure);assert!(!r.decision().unwrap().candidates.is_empty());let reward=r.state.reward_ledger.total;
        if take>0{choose(&mut r,RoomChoiceKind::OpenChest);if take==2{r.state.keys[1]=false;choose(&mut r,RoomChoiceKind::Sapphire);}else{let relic=r.decision().unwrap().room.unwrap().choices.iter().find_map(|c|if let RoomChoiceKind::Relic(id)=c.kind{Some(id)}else{None}).unwrap();choose(&mut r,RoomChoiceKind::Relic(relic));}}
        let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0);assert_eq!((r.state.floor,r.state.terminal),(43,None));assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act3PrefixComplete);assert!(r.decision().unwrap().candidates.is_empty());assert_eq!(r.state.reward_ledger.total,reward+if take==2{5}else{0});let frozen=r.clone();assert!(r.step(&GameAction::DiscardPotion{slot:0}).is_err());assert!(r==frozen);
    }
}

#[test]
fn m8_old_ready_and_prefix_profiles_unchanged(){
    let mut r=super::m7_rewards::chest();choose(&mut r,RoomChoiceKind::Relic(RelicId::BlackBlood));choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act3Ready);assert!(r.decision().unwrap().candidates.is_empty());
    let mut r=super::m5_rewards::boss_chest();choose(&mut r,RoomChoiceKind::Relic(RelicId::BlackBlood));choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act2Ready);assert!(r.decision().unwrap().candidates.is_empty());
}

#[test]
fn m8_capture_cycles_keep_history_exclusions_and_question_sources(){
    let r=ready();let ordinary=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.floor==35).unwrap().id;let elite=r.state.map.as_ref().unwrap().nodes.iter().find(|n|(35..43).contains(&n.floor)&&n.kind==MapNodeKind::Elite).unwrap().id;let question=r.state.map.as_ref().unwrap().nodes.iter().find(|n|(35..43).contains(&n.floor)&&n.kind==MapNodeKind::Question).unwrap().id;
    let mut c=r.clone();let mut history=vec![];for want in [EncounterId::OrbWalker,EncounterId::ThreeShapes,EncounterId::SpireGrowth,EncounterId::Transient,EncounterId::FourShapes,EncounterId::Maw,EncounterId::SphereAndTwoShapes,EncounterId::JawWormHorde,EncounterId::ThreeDarklings,EncounterId::WrithingMass]{c.state.map.as_mut().unwrap().regular_encounter_history=history.clone();assert_eq!(act3::capture_room(&c.state,ordinary).unwrap().encounter,Some(want));history.push(want);}
    for (boss,want) in [(EncounterId::AwakenedOne,EncounterId::GiantHead),(EncounterId::TimeEater,EncounterId::Nemesis),(EncounterId::DonuAndDeca,EncounterId::Reptomancer)]{c.state.map.as_mut().unwrap().boss_icon=Some(boss);c.state.map.as_mut().unwrap().elite_encounter_history.clear();assert_eq!(act3::capture_room(&c.state,elite).unwrap().encounter,Some(want));c.state.map.as_mut().unwrap().elite_encounter_history.push(want);assert_ne!(act3::capture_room(&c.state,elite).unwrap().encounter,Some(want));}
    for (visit,event) in [(0,Some(crate::content::EventId::Falling)),(1,Some(crate::content::EventId::TombOfLordRedMask)),(2,None)]{let mut c=r.clone();c.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().question_visits=visit;before(&mut c,question);c.step(&GameAction::Move(question)).unwrap();assert_eq!(c.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().question_visits,visit+1);if let Some(event)=event{assert_eq!(c.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(event));assert_eq!(c.state.room_loop.as_ref().unwrap().act3_event.as_ref().unwrap().node,question);}else{assert!(c.state.combat.is_some());}}
}

#[test]
fn m8_event_move_preflight_is_atomic_and_does_not_reheal(){
    let mut r=ready();let question=r.state.map.as_ref().unwrap().nodes.iter().find(|n|(35..43).contains(&n.floor)&&n.kind==MapNodeKind::Question).unwrap().id;before(&mut r,question);r.state.hp=40;r.next_choice=u64::MAX;let frozen=r.clone();assert!(r.step(&GameAction::Move(question)).is_err());assert!(r==frozen);r.next_choice=1000;r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().question_visits=usize::MAX;let frozen=r.clone();assert!(r.step(&GameAction::Move(question)).is_err());assert!(r==frozen);
    r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().question_visits=0;let rng=r.state.rng.clone();r.step(&GameAction::Move(question)).unwrap();assert_eq!(r.state.hp,40);assert_ne!(r.state.rng,rng);let frozen=r.clone();assert!(r.step(&GameAction::Move(question)).is_err());assert!(r==frozen);
}

#[test]
fn m8_owned_shop_stock_is_disabled_and_later_capture_is_retained(){
    let mut r=ready();if !r.state.relics.contains(&RelicId::Anchor){r.state.relics.push(RelicId::Anchor);}let node=r.state.map.as_ref().unwrap().nodes.iter().find(|n|(35..43).contains(&n.floor)&&n.kind==MapNodeKind::Shop).unwrap().id;before(&mut r,node);r.step(&GameAction::Move(node)).unwrap();let room=r.decision().unwrap().room.unwrap();assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Card{id:CardId::Clothesline,upgrades:0}&&c.price==Some(50)));assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::Anchor)&&c.status==RoomChoiceStatus::Disabled));assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().shop_visits,1);
}

#[test]
fn m8_exhausted_chest_can_still_proceed_but_other_source_cannot_complete(){
    let mut r=ready();for relic in [RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial]{if !r.state.relics.contains(&relic){r.state.relics.push(relic);}}let node=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.floor==43&&n.kind==MapNodeKind::Treasure).unwrap().id;before(&mut r,node);r.step(&GameAction::Move(node)).unwrap();let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::OpenChest).unwrap().id;let frozen=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==frozen);
    let mut wrong=r.clone();wrong.state.room_loop.as_mut().unwrap().screen.kind=RoomViewKind::Event(crate::content::EventId::Purifier);let frozen=wrong.clone();assert!(super::super::rooms::finish(&mut wrong).is_err());assert!(wrong==frozen);choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act3PrefixComplete);
}

#[test]
fn m8_second_chest_selection_stays_owned_until_source_proceed(){
    let mut r=super::m7_rewards::chest();r.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M8Act3Prefix;choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));assert!(r.state.pending.is_some());assert_eq!((r.state.act,r.state.floor),(2,34));let frozen=r.clone();assert!(r.step(&GameAction::Choose(ChoiceId(u64::MAX))).is_err());assert!(r==frozen);for _ in 0..2{let id=r.state.pending.as_ref().unwrap().selection.candidates[0].id;r.step(&GameAction::Select(id)).unwrap();}assert_eq!(r.state.act,2);choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);assert_eq!(r.state.act,3);
}

#[test]
fn m8_elite_burning_collar_local_rewards_and_exhaustion_keep_real_owner(){
    for exhausted in [false,true]{
        let mut r=ready();let relics=if exhausted{vec![RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::MagicFlower,RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial]}else{vec![RelicId::Lantern]};for id in relics{if !r.state.relics.contains(&id){r.state.relics.push(id);}}
        if !r.state.relics.contains(&RelicId::NeowsLament){r.state.relics.push(RelicId::NeowsLament);}r.state.relic_counters.insert(RelicId::NeowsLament,3);r.state.map.as_mut().unwrap().boss_icon=Some(EncounterId::AwakenedOne);
        let node=r.state.map.as_ref().unwrap().nodes.iter().find(|n|(35..43).contains(&n.floor)&&n.kind==MapNodeKind::Elite).unwrap().id;before(&mut r,node);r.state.map.as_mut().unwrap().nodes.iter_mut().find(|n|n.id==node).unwrap().burning=true;r.step(&GameAction::Move(node)).unwrap();let d=r.decision().unwrap();let player=&d.combat.as_ref().unwrap().player;assert_eq!((player.max_energy,player.energy),(4,5),"Collar4 plus this owned Lantern's first-turn1");let item=r.state.combat.as_ref().unwrap().item_context().unwrap();assert!(item.battle.elite_trigger&&item.battle.burning.is_some());assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().combat_visits,0);
        let a=d.candidates.into_iter().find_map(|c|match c.action{GameAction::Combat(a@CombatAction::PlayCard{target:Some(_),..})=>Some(a),_=>None}).unwrap();let result=r.choose_combat(a);
        if exhausted{let error=result.unwrap_err();assert!(matches!(error,RunError::Faulted(_)));assert_eq!(r.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(crate::combat::Outcome::Victory));assert!(r.state.relics.is_empty());assert!(r.state.active_combat.is_some());assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().combat_visits,0);assert_eq!(r.state.terminal,None);let frozen=r.clone();assert_eq!(r.choose_combat(CombatAction::EndTurn),Err(error));assert!(r==frozen);}else{result.unwrap();assert!(r.state.combat.is_none());let p=r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap();assert_eq!((p.combat_visits,p.elite_relic_cursor),(1,2));let room=r.decision().unwrap().room.unwrap();assert!(room.choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::Pear)));assert!(room.choices.iter().any(|c|matches!(c.kind,RoomChoiceKind::Gold(25..=35))));}
    }
}

#[test]
fn m8_real_death_settles_act3_counter_without_prefix_completion(){
    let mut r=ready();r.state.hp=1;r.state.map.as_mut().unwrap().boss_icon=Some(EncounterId::AwakenedOne);let node=r.state.map.as_ref().unwrap().nodes.iter().find(|n|(35..43).contains(&n.floor)&&n.kind==MapNodeKind::Elite).unwrap().id;before(&mut r,node);r.step(&GameAction::Move(node)).unwrap();let settled=r.state.combat_progress.settled;
    let mut terminal=None;for _ in 0..8{let t=r.choose_combat(CombatAction::EndTurn).unwrap();if t.terminal.is_some(){assert_eq!(t.reward,-1);terminal=t.terminal;break}}
    assert_eq!(terminal,Some(RunTerminal::Death));assert_eq!(r.state.combat_progress.settled,settled+1);assert_eq!(r.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().combat_visits,1);assert!(r.decision().unwrap().candidates.is_empty());assert_ne!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Act3PrefixComplete);
}
