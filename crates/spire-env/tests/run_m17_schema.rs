mod m17_support;
use m17_support::caller;
use spire_core::{combat::OrbKind,content::{CardId,RelicId,PotionId},run::*};
use spire_env::*;
fn initial()->Run{caller::make_run(7,caller::Scenario::Guardian).unwrap()}
#[test]
fn m17_dynamic_values_and_actions_use_shared_defect_projection(){
    let caps=RunCapacities::m10_default();let spec=m17_run_observation_spec(&caps).unwrap().descriptor();
    assert_eq!(spec["schema_id"],"spire-gym-m17-defect-act1-observation-v1");assert_eq!(spec["profile_id"],"M17DefectAct1");assert_eq!(spec["action_space"]["n"],caps.actions);
    for (key,values) in [("card",M17_DEFECT_ACT1_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>()),("relic",M17_DEFECT_ACT1_RELICS.iter().map(|id|id.as_str()).collect()),("potion",M17_DEFECT_ACT1_POTIONS.iter().map(|id|id.as_str()).collect())]{for value in values{assert!(spec["vocabularies"][key].as_array().unwrap().iter().any(|v|v==value));}}
    assert!(!spec["vocabularies"]["card"].as_array().unwrap().iter().any(|v|v==CardId::EchoForm.as_str()));
    let mut run=initial();let d=run.decision().unwrap();let action=d.candidates[0].action.clone();run.step(&action).unwrap();let d=run.decision().unwrap();let action=d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();run.step(&action).unwrap();let mut d=run.decision().unwrap();
    let obs=encode_run_m17(&d,run.public_map().as_ref(),&caps).unwrap();assert_eq!(obs.action_mask.len(),caps.actions);assert!(obs.action_mask[..d.candidates.len()].iter().all(|v|*v==1));assert!(obs.action_mask[d.candidates.len()..].iter().all(|v|*v==0));
    let rows=obs.group("actions").unwrap();for (i,a) in d.candidates.iter().enumerate(){if let GameAction::Combat(spire_core::combat::CombatAction::PlayCard{instance,..})=a.action{let hand=&d.combat.as_ref().unwrap().hand;assert_eq!(&rows.values[i*8..i*8+3],&[8,4,hand.iter().position(|c|c.instance.0==instance).unwrap() as i64]);}}
    let c=d.combat.as_mut().unwrap();c.character.orbs=vec![Some(OrbKind::Lightning),None,Some(OrbKind::Frost),Some(OrbKind::Dark),Some(OrbKind::Plasma),None,None,None,None,None];c.character.orb_dark_damage=vec![None,None,None,Some(i32::MAX),None,None,None,None,None,None];c.character.focus=-3;
    d.objects[0].misc=i32::MIN;
    let obs=encode_run_m17(&d,run.public_map().as_ref(),&caps).unwrap();let orbs=obs.group("orbs").unwrap();assert_eq!(&orbs.values[..10],&[1,0,0,0,2,0,3,i64::from(i32::MAX),4,0]);assert_eq!(orbs.known.iter().step_by(2).sum::<i8>(),10);assert_eq!(obs.group("permanent_cards").unwrap().values[2],i64::from(i32::MIN));
    assert!(matches!(encode_run_m17(&d,None,&RunCapacities{orbs:9,..caps.clone()}),Err(RunSessionError::Capacity{group:"orbs",required:10,capacity:9})));
    assert!(matches!(encode_run_m17(&d,None,&RunCapacities{actions:d.candidates.len()-1,..caps.clone()}),Err(RunSessionError::Capacity{group:"actions",..})));
    assert!(matches!(encode_run_m17(&d,None,&RunCapacities{permanent_cards:9,..caps.clone()}),Err(RunSessionError::Capacity{group:"permanent_cards",..})));
    for id in [CardId::EchoForm,CardId::Leap,CardId::Unknown("foreign")]{let mut bad=d.clone();bad.objects[0].id=id;assert!(encode_run_m17(&bad,None,&caps).is_err());}
    let mut bad=d.clone();bad.combat.as_mut().unwrap().inventory.as_mut().unwrap().relics.push(RelicId::BurningBlood);assert!(encode_run_m17(&bad,None,&caps).is_err());let mut bad=d.clone();bad.combat.as_mut().unwrap().inventory.as_mut().unwrap().potions[0]=Some(PotionId::DexterityPotion);assert!(encode_run_m17(&bad,None,&caps).is_err());
    let mut bad=d.clone();bad.content_profile=Some(RoomScenario::M12SilentAct1);assert!(encode_run_m17(&bad,None,&caps).is_err());let mut bad=d.clone();bad.terminal=Some(RunTerminal::HeartVictory);assert!(encode_run_m17(&bad,None,&caps).is_err());
    for hidden in ["copy_family","applied_mutation_ordinal","rng_key","draw_order"]{assert!(!format!("{spec}").contains(hidden));}
    for (key,field) in [("permanent_cards","misc"),("combat_cards","misc"),("orbs","dark_damage")]{let tensor=spec["tensors"].as_array().unwrap().iter().find(|t|t["key"]==key).unwrap();let column=tensor["columns"].as_array().unwrap().iter().find(|c|c["field"]==field).unwrap();assert_eq!(column["low"],i32::MIN);assert_eq!(column["high"],i32::MAX);}
}
#[test]
fn m17_map_icons_and_statuses_cannot_forge_permanent_playable_content(){
    let caps=RunCapacities::m10_default();let d=initial().decision().unwrap();
    for encounter in [spire_core::content::EncounterId::Champ,spire_core::content::EncounterId::BronzeAutomaton,spire_core::content::EncounterId::Collector]{let mut bad=d.clone();bad.last_combat=Some(CombatRecord{visit:0,encounter,outcome:spire_core::combat::Outcome::Victory});assert!(encode_run_m17(&bad,None,&caps).is_err(),"map-only icon forged as played encounter {encounter:?}");}
    for id in [CardId::Dazed,CardId::Burn,CardId::Slimed]{let mut bad=d.clone();bad.objects[0].id=id;assert!(encode_run_m17(&bad,None,&caps).is_err(),"generated status forged as permanent {id:?}");}
    let mut bad=d.clone();bad.objects[0].upgrades=2;assert!(encode_run_m17(&bad,None,&caps).is_err());
}
