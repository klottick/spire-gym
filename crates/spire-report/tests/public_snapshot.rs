#[path="common/native.rs"] mod native;
use spire_core::{run::*,combat::*,content::PotionId};
use spire_report::*;
use serde_json::Value;

#[test]
fn run_snapshot_current_owner() {
    let mut run=native::lethal();
    run.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:Some(TargetId(1))})).unwrap();
    let d=run.decision().unwrap();let c=d.combat.as_ref().unwrap();
    let state=run_snapshot(&d,run.public_map().as_ref()).unwrap();
    assert_eq!(state["player"]["hp"],c.player.hp);assert_eq!(state["player"]["energy"],c.player.energy);
    assert_eq!(state["inventory"]["potions"].as_array().unwrap().len(),3);assert!(state["inventory"]["potions"][0].is_null());
    assert_eq!(state["combat"]["hand"][0]["origin"]["instance"],"1");
    assert_eq!(state["combat"]["hand"][0]["currentCost"],c.hand[0].effective_cost.unwrap());
}
#[test]
fn run_option_namespace() {
    let mut d=native::m4::run(7).decision().unwrap();
    let use_action=GameAction::UsePotion{slot:0,target:None};let discard=GameAction::DiscardPotion{slot:0};
    d.player.potions[0]=Some(PotionId::BloodPotion);
    d.candidates=vec![Candidate{id:ChoiceId(42),action:use_action.clone(),label:"Use",neow_offer:None,object:None},Candidate{id:ChoiceId(42),action:discard.clone(),label:"Discard",neow_offer:None,object:None}];
    let a=describe_run_action(&d,&use_action,"combat-0").unwrap();let b=describe_run_action(&d,&discard,"combat-0").unwrap();
    assert_ne!(a["optionKey"],b["optionKey"]);
    assert_eq!(a["preActionOffer"]["potionId"],"blood_potion");
}
#[test]
fn run_target_lifetimes() {
    let mut d=native::lethal().decision().unwrap();let c=d.combat.as_mut().unwrap();
    let mut dead=c.enemies[0].clone();dead.hp=0;dead.instance_id=EnemyHandle::parse("dead-first").unwrap();c.enemies.insert(0,dead);
    c.targets=vec![(TargetId(99),false),(TargetId(73),true)];
    let action=GameAction::Combat(CombatAction::UsePotion{slot:0,target:Some(TargetId(73))});
    d.candidates=vec![Candidate{id:ChoiceId(0),action:action.clone(),label:"Potion",neow_offer:None,object:None}];
    let described=describe_run_action(&d,&action,"combat-9").unwrap();
    assert!(described["targets"][0].as_str().unwrap().ends_with(d.combat.as_ref().unwrap().enemies[1].instance_id.as_str()));
    d.combat.as_mut().unwrap().targets.pop();assert!(describe_run_action(&d,&action,"combat-9").is_err());
}

#[test]
fn run_snapshot_keeps_half_dead_distinct_from_permanent_defeat() {
    let mut d=native::lethal().decision().unwrap();d.candidates.clear();let c=d.combat.as_mut().unwrap();
    c.enemies[0].hp=0;c.enemies[0].half_dead=true;c.targets[0].1=false;
    let state=run_snapshot(&d,None).unwrap();
    assert_eq!(state["combat"]["enemies"][0]["lifeState"],"half_dead");
    assert_eq!(state["combat"]["enemies"][0]["targetEligible"],false);
}

#[test]
fn newer_public_room_variants_are_named_without_inventing_completion() {
    let mut d=native::m4::run(7).decision().unwrap();d.candidates.clear();
    for (kind,want) in [(RoomViewKind::Act3Ready,"act3_ready"),(RoomViewKind::Act3PrefixComplete,"act3_prefix_complete")] {
        d.room=Some(RoomView{kind,choices:vec![]});let state=run_snapshot(&d,None).unwrap();
        assert_eq!(state["screen"]["kind"],want);assert!(state["terminal"].is_null());
    }
    d.room=Some(RoomView{kind:RoomViewKind::Event(spire_core::content::EventId::Falling),choices:vec![
        RoomChoiceView{id:ChoiceId(1),kind:RoomChoiceKind::FallingLose{card_type:spire_core::content::CardType::Attack,instance:Some(42)},status:RoomChoiceStatus::Offered,price:None},
        RoomChoiceView{id:ChoiceId(2),kind:RoomChoiceKind::TombGold,status:RoomChoiceStatus::Offered,price:None},
        RoomChoiceView{id:ChoiceId(3),kind:RoomChoiceKind::TombOfferGold,status:RoomChoiceStatus::Disabled,price:Some(99)},
    ]});
    let state=run_snapshot(&d,None).unwrap();let options=state["screen"]["options"].as_array().unwrap();
    assert_eq!(options[0]["data"]["instance"],"42");assert_eq!(options[0]["data"]["cardType"],"attack");
    assert_eq!(options[1]["label"],"Take gold");assert_eq!(options[2]["label"],"Offer gold");
    assert_eq!(options[2]["status"],"disabled");assert_eq!(options[2]["data"]["price"],99);
}
#[test]
fn run_public_map() {
    let mut run=native::m4::run(7);
    let action=run.decision().unwrap().candidates[0].action.clone();run.step(&action).unwrap();
    let map=run.public_map().unwrap();
    let state=run_snapshot(&run.decision().unwrap(),Some(&map)).unwrap();
    assert_eq!(state["map"]["nodes"].as_array().unwrap().len(),map.nodes.len());
    assert!(state["map"]["edges"].is_array());
    for n in state["map"]["nodes"].as_array().unwrap() {if n["kind"]=="?" {assert_eq!(n["resolvedRoom"],Value::Null);}}
    assert!(!state.to_string().contains("encounter_history"));
}
