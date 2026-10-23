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
