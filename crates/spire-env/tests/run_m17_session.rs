mod m17_support;
use m17_support::{caller,config};
use spire_env::*;
use spire_core::run::*;
#[test]
fn m17_history_is_typed_strict_and_profile_bound(){
    for scenario in [caller::Scenario::Guardian,caller::Scenario::Hexaghost,caller::Scenario::SlimeBoss]{
        let cfg=config(scenario);let mut session=RunSession::new_m17(cfg.clone()).unwrap();let boundary=session.reset(Some(7)).unwrap();let d=session.reproduction().unwrap().latest_decision.unwrap().unwrap();assert_eq!((d.player.hp,d.player.gold,d.objects.len()),(75,99,10));assert_eq!(d.room.unwrap().kind,RoomViewKind::Neow);
        let json=session.record().unwrap().unwrap().to_json().unwrap();assert_eq!(RunEpisodeRecord::from_json(&json).unwrap(),session.record().unwrap().unwrap());
        let mut value:serde_json::Value=serde_json::from_slice(&json).unwrap();assert!(value["config"]["boss_history"]["m17-defect-act1"]["act1"].is_string());
        for history in [serde_json::json!({}),serde_json::json!({"m17-defect-act1":{}}),serde_json::json!({"m17-defect-act1":{"act1":"guardian","act2":"champ"}}),serde_json::json!({"m17-defect-act1":{"act1":"unknown"}}),serde_json::json!("act1-guardian-unseen")]{value["config"]["boss_history"]=history;assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&value).unwrap()).is_err());}
        let text=String::from_utf8(json.clone()).unwrap();
        let start=text.find("\"boss_history\":").unwrap();let tail=&text[start..];let target_start=tail.find("\"act1\":").unwrap()+start;let target_end=text[target_start..].find('}').unwrap()+target_start;
        let fragment=&text[target_start..target_end];let duplicate=text.replacen(fragment,&format!("{fragment},{fragment}"),1);assert!(RunEpisodeRecord::from_json(duplicate.as_bytes()).is_err(),"raw duplicate target rejected");
        let wrapper="\"m17-defect-act1\"";let duplicated=text.replacen(wrapper,&format!("{wrapper}:{{\"act1\":\"guardian\"}},{wrapper}"),1);assert!(RunEpisodeRecord::from_json(duplicated.as_bytes()).is_err(),"raw duplicate wrapper rejected");
        let before=session.reproduction().unwrap();assert_eq!(session.step(boundary.token,RunActionIndex(-1)),Err(RunSessionError::Input("action index is not legal")));assert_eq!(session.reproduction().unwrap(),before);
        let a=dummy_lament(&session);let next=session.step(boundary.token,RunActionIndex(a)).unwrap();assert_eq!(session.reproduction().unwrap().latest_map.unwrap().boss,Some(scenario.encounter()));assert_eq!(next.truncated,false);
        let reset=session.reset(Some(7)).unwrap();assert_ne!(reset.token,boundary.token);assert_eq!(session.step(boundary.token,RunActionIndex(0)),Err(RunSessionError::StaleToken));
        let mut bad=cfg.clone();bad.boss_history=RunBossHistory::Act1GuardianUnseen;assert!(RunSession::new_m17(bad).is_err());assert!(RunSession::new_m12(cfg.clone()).is_err());
    }
    for (character,ascension) in [(spire_core::content::Character::Silent,0),(spire_core::content::Character::Defect,1)]{let mut cfg=config(caller::Scenario::Guardian);cfg.character=character;cfg.ascension=ascension;assert!(RunSession::new_m17(cfg).is_err());}
}
fn dummy_lament(s:&RunSession)->i64{let d=s.reproduction().unwrap().latest_decision.unwrap().unwrap();let id=d.room.as_ref().unwrap().choices.iter().find(|r|r.kind==RoomChoiceKind::NeowLament).unwrap().id;d.candidates.iter().position(|c|c.action==GameAction::Choose(id)).unwrap() as i64}
#[test]
fn m17_native_sidecar_is_rejected_without_freezing(){
    let mut s=RunSession::new_m17(config(caller::Scenario::Guardian)).unwrap();let b=s.reset(Some(7)).unwrap();let before=s.reproduction().unwrap();assert!(s.combat_observation_spec().is_err());assert!(s.combat_view().is_err());assert_eq!(s.reproduction().unwrap(),before);assert_eq!(s.decision().unwrap(),b);assert!(s.environment_spec().unwrap()["combat_view_id"].is_null());s.close().unwrap();
}
#[test]
fn m17_postaccepted_delivery_fault_retains_one_semantic_prefix(){
    let mut cfg=config(caller::Scenario::Guardian);cfg.capacities.map_nodes=1;let mut s=RunSession::new_m17(cfg.clone()).unwrap();let initial=s.reset(Some(7)).unwrap();
    assert_eq!(s.step(initial.token,RunActionIndex(0)),Err(RunSessionError::DeliveryFault("post-step Run encoding failed")));
    let record=s.record().unwrap().unwrap();assert_eq!(record.steps.len(),1);assert!(record.fault.as_ref().is_some_and(|f|f.committed&&f.gameplay_recorded&&f.error.kind==RunRecordErrorKind::Capacity));
    let before=s.reproduction().unwrap();assert!(s.step(initial.token,RunActionIndex(0)).is_err());assert_eq!(s.reproduction().unwrap(),before);
    let decoded=RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();let replay=replay_run(&decoded,RunCapacities::m10_default()).unwrap();assert_eq!(replay.accepted_count,1);assert_eq!(replay.final_decision,s.reproduction().unwrap().latest_decision.unwrap().unwrap());assert_eq!(replay.reported_fault,record.fault);
}
