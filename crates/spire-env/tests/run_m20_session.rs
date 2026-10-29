mod m20_support;
use m20_support::*;
use spire_env::*;
#[test]
fn m20_sidecar_is_absent_and_rejection_does_not_freeze() {
    let mut s=RunSession::new_m20(default_config()).unwrap();let b=s.reset(Some(7)).unwrap();let before=s.reproduction().unwrap();
    assert!(s.environment_spec().unwrap()["combat_view_id"].is_null());assert!(s.combat_observation_spec().is_err());assert!(s.combat_view().is_err());
    assert_eq!(s.reproduction().unwrap(),before);assert_eq!(s.decision().unwrap(),b);
    s.close().unwrap();assert_eq!(s.decision(),Err(RunSessionError::Closed));
}

#[test]
fn m20_postaccepted_capacity_and_delivery_failures_keep_one_prefix() {
    let mut cfg=default_config();cfg.capacities.map_nodes=1;let mut s=RunSession::new_m20(cfg).unwrap();let b=s.reset(Some(7)).unwrap();
    assert_eq!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::DeliveryFault("post-step Run encoding failed")));
    let record=s.record().unwrap().unwrap();assert_eq!(record.steps.len(),1);let fault=record.fault.as_ref().unwrap();
    assert!(fault.committed&&fault.gameplay_recorded);assert_eq!(fault.error.kind,RunRecordErrorKind::Capacity);
    let before=s.reproduction().unwrap();assert!(s.step(b.token,RunActionIndex(0)).is_err());assert!(s.reset(None).is_err());assert_eq!(s.reproduction().unwrap(),before);
    let replay=replay_run(&RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap(),RunCapacities::m10_default()).unwrap();
    assert_eq!(replay.accepted_count,1);assert_eq!(replay.final_decision,before.latest_decision.unwrap().unwrap());assert_eq!(replay.reported_fault,record.fault);
    s.close().unwrap();assert!(s.reproduction().unwrap().closed);
    let mut s=RunSession::new_m20(default_config()).unwrap();let b=s.reset(Some(11)).unwrap();s.step(b.token,RunActionIndex(0)).unwrap();
    let accepted=s.record().unwrap().unwrap().steps;s.fail_delivery("supplied delivery probe").unwrap();let before=s.reproduction().unwrap();
    assert!(s.step(b.token,RunActionIndex(0)).is_err());assert_eq!(s.record().unwrap().unwrap().steps,accepted);assert_eq!(s.reproduction().unwrap(),before);
    let r=s.record().unwrap().unwrap();assert_eq!(replay_run(&RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap(),RunCapacities::m10_default()).unwrap().accepted_count,1);
}

#[test]
fn m20_config_history_is_typed_and_profile_bound() {
    for a in [RunAct1BossTarget::Guardian,RunAct1BossTarget::Hexaghost,RunAct1BossTarget::SlimeBoss] {
        for b in [RunAct2BossTarget::Champ,RunAct2BossTarget::BronzeAutomaton,RunAct2BossTarget::Collector] {
            for c in [RunAct3BossTarget::AwakenedOne,RunAct3BossTarget::TimeEater,RunAct3BossTarget::DonuAndDeca] {
            let cfg=config(a,b,c);let mut s=RunSession::new_m20(cfg.clone()).unwrap();let boundary=s.reset(Some(7)).unwrap();
            let before=s.reproduction().unwrap();assert_eq!((before.latest_decision.as_ref().unwrap().as_ref().unwrap().act,before.latest_decision.as_ref().unwrap().as_ref().unwrap().floor),(1,0));
            let record=s.record().unwrap().unwrap();assert_eq!(RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap(),record);
            let wire:serde_json::Value=serde_json::from_slice(&record.to_json().unwrap()).unwrap();
            assert!(wire["config"]["boss_history"]["m20-defect-heart"].get("act3").is_some());
            // Actual canonical first map, without running the public caller.
            let action=before.latest_decision.unwrap().unwrap().candidates[0].action.clone();
            let mut core=diagnostic_run_for_config(&cfg,7);core.step(&action).unwrap();s.step(boundary.token,RunActionIndex(0)).unwrap();
            assert_eq!(s.reproduction().unwrap().latest_map,core.public_map());
            let expected=match a{RunAct1BossTarget::Guardian=>spire_core::content::EncounterId::Guardian,RunAct1BossTarget::Hexaghost=>spire_core::content::EncounterId::Hexaghost,RunAct1BossTarget::SlimeBoss=>spire_core::content::EncounterId::SlimeBoss};
            assert_eq!(core.public_map().unwrap().boss,Some(expected));
            let mut wrong=cfg.clone();wrong.character=spire_core::content::Character::Silent;assert!(RunSession::new_m20(wrong).is_err());
            let mut wrong=cfg.clone();wrong.ascension=1;assert!(RunSession::new_m20(wrong).is_err());
            for old in [RunSession::new,RunSession::new_m10,RunSession::new_m12,RunSession::new_m13,RunSession::new_m14,RunSession::new_m15,RunSession::new_m17,RunSession::new_m18,RunSession::new_m19] {assert!(old(cfg.clone()).is_err());}
            }
        }
    }
}

#[test]
fn m20_raw_history_is_exact_duplicate_safe_and_validated_before_reset() {
    use serde_json::json;
    let mut s=RunSession::new_m20(default_config()).unwrap();let b=s.reset(Some(7)).unwrap();
    let record=s.record().unwrap().unwrap();let bytes=record.to_json().unwrap();let value:serde_json::Value=serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["config"]["boss_history"],json!({"m20-defect-heart":{"act1":"guardian","act2":"champ","act3":"awakened-one"}}));
    assert_eq!(record.schema_id,RUN_RECORD_SCHEMA_ID_M20);assert_eq!(record.rules_id,RUN_RULES_ID_M20);assert_eq!(record.content_id,RUN_CONTENT_ID_M20);
    let before=s.reproduction().unwrap();
    for history in [json!(null),json!({}),json!("act1-guardian-unseen"),json!({"m17-defect-act1":{"act1":"guardian"}}),
        json!({"m20-defect-heart":{"act1":"guardian","act2":"champ"}}),
        json!({"m20-defect-heart":{"act1":"guardian","act3":"awakened-one"}}),
        json!({"m20-defect-heart":{"act2":"champ","act3":"awakened-one"}}),
        json!({"m20-defect-heart":{"act1":"guardian","act2":"champ","act3":"unknown"}}),
        json!({"m20-defect-heart":{"act1":"guardian","act2":"champ","act3":"awakened-one","extra":0}}),
        json!({"m20-defect-heart":{}}),json!({"m20-defect-heart":{"act1":"guardian"}}),json!({"m20-defect-heart":{"act2":"champ"}}),
        json!({"m20-defect-heart":{"act1":"guardian","act2":"champ","extra":0}}),
        json!({"m20-defect-heart":{"act1":"unknown","act2":"champ"}}),json!({"m20-defect-heart":{"act1":"guardian","act2":"unknown"}}),
        json!({"m20-defect-heart":{"act1":"champ","act2":"guardian"}}),
        json!({"m20-defect-heart":{"act1":"guardian","act2":"champ"},"extra":{}})] {
        let mut bad=value.clone();bad["config"]["boss_history"]=history;assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let raw=String::from_utf8(bytes).unwrap();
    for (from,to) in [
        ("\"act1\":\"guardian\"","\"act1\":\"guardian\",\"act1\":\"guardian\""),
        ("\"act2\":\"champ\"","\"act2\":\"champ\",\"act2\":\"champ\""),
        ("\"act3\":\"awakened-one\"","\"act3\":\"awakened-one\",\"act3\":\"time-eater\""),
        ("\"m20-defect-heart\":{","\"m20-defect-heart\":{},\"m20-defect-heart\":{"),
        ("\"boss_history\":{","\"boss_history\":{},\"boss_history\":{"),
        ("\"character\":\"defect\"","\"character\":\"defect\",\"character\":\"defect\""),
    ] {
        let bad=raw.replacen(from,to,1);assert_ne!(bad,raw,"missing test field {from}");assert!(RunEpisodeRecord::from_json(bad.as_bytes()).is_err(),"raw duplicate {from}");
    }
    for path in ["config","capacities"] {
        let mut bad=value.clone();if path=="config" {bad["config"]["unexpected"]=json!(0)}else{bad["config"]["capacities"]["unexpected"]=json!(0)}
        assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    assert_eq!(s.reproduction().unwrap(),before);assert_eq!(s.decision().unwrap(),b);
    s.step(b.token,RunActionIndex(0)).unwrap();let reset=s.reset(None).unwrap();
    assert_ne!(reset.token,b.token);let reproduction=s.reproduction().unwrap();assert_eq!(reproduction.ordinal,1);
    let d=reproduction.latest_decision.unwrap().unwrap();assert_eq!((d.player.hp,d.player.gold,d.objects.len()),(75,99,10));assert!(reproduction.latest_map.is_none());
    assert_eq!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::StaleToken));
}


#[test]
fn m20_wrong_profile_history_is_rejected_before_any_session_reset(){
    let mut config=default_config();
    for history in [RunBossHistory::Act1GuardianUnseen,RunBossHistory::M15SilentHeart{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},RunBossHistory::M19DefectAct3{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne}]{
        config.boss_history=history;assert!(matches!(RunSession::new_m20(config.clone()),Err(RunSessionError::Input("boss history does not match Run profile"))));
    }
}
