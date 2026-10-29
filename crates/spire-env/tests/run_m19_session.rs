mod m19_support;
use m19_support::*;
use spire_env::*;
#[test]
fn m19_config_history_is_typed_and_profile_bound() {
    for a in [RunAct1BossTarget::Guardian,RunAct1BossTarget::Hexaghost,RunAct1BossTarget::SlimeBoss] {
        for b in [RunAct2BossTarget::Champ,RunAct2BossTarget::BronzeAutomaton,RunAct2BossTarget::Collector] {
            for c in [RunAct3BossTarget::AwakenedOne,RunAct3BossTarget::TimeEater,RunAct3BossTarget::DonuAndDeca] {
            let cfg=config(a,b,c);let mut s=RunSession::new_m19(cfg.clone()).unwrap();let boundary=s.reset(Some(7)).unwrap();
            let before=s.reproduction().unwrap();assert_eq!((before.latest_decision.as_ref().unwrap().as_ref().unwrap().act,before.latest_decision.as_ref().unwrap().as_ref().unwrap().floor),(1,0));
            let record=s.record().unwrap().unwrap();assert_eq!(RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap(),record);
            let wire:serde_json::Value=serde_json::from_slice(&record.to_json().unwrap()).unwrap();
            assert!(wire["config"]["boss_history"]["m19-defect-act3"].get("act3").is_some());
            // Actual canonical first map, without running the public caller.
            let action=before.latest_decision.unwrap().unwrap().candidates[0].action.clone();
            let mut core=diagnostic_run_for_config(&cfg,7);core.step(&action).unwrap();s.step(boundary.token,RunActionIndex(0)).unwrap();
            assert_eq!(s.reproduction().unwrap().latest_map,core.public_map());
            let expected=match a{RunAct1BossTarget::Guardian=>spire_core::content::EncounterId::Guardian,RunAct1BossTarget::Hexaghost=>spire_core::content::EncounterId::Hexaghost,RunAct1BossTarget::SlimeBoss=>spire_core::content::EncounterId::SlimeBoss};
            assert_eq!(core.public_map().unwrap().boss,Some(expected));
            let mut wrong=cfg.clone();wrong.character=spire_core::content::Character::Silent;assert!(RunSession::new_m19(wrong).is_err());
            let mut wrong=cfg.clone();wrong.ascension=1;assert!(RunSession::new_m19(wrong).is_err());
            for old in [RunSession::new,RunSession::new_m10,RunSession::new_m12,RunSession::new_m13,RunSession::new_m14,RunSession::new_m15,RunSession::new_m17,RunSession::new_m18] {assert!(old(cfg.clone()).is_err());}
            }
        }
    }
}

#[test]
fn m19_raw_history_is_exact_duplicate_safe_and_validated_before_reset() {
    use serde_json::json;
    let mut s=RunSession::new_m19(default_config()).unwrap();let b=s.reset(Some(7)).unwrap();
    let record=s.record().unwrap().unwrap();let bytes=record.to_json().unwrap();let value:serde_json::Value=serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["config"]["boss_history"],json!({"m19-defect-act3":{"act1":"guardian","act2":"champ","act3":"awakened-one"}}));
    assert_eq!(record.schema_id,RUN_RECORD_SCHEMA_ID_M19);assert_eq!(record.rules_id,RUN_RULES_ID_M19);assert_eq!(record.content_id,RUN_CONTENT_ID_M19);
    let before=s.reproduction().unwrap();
    for history in [json!(null),json!({}),json!("act1-guardian-unseen"),json!({"m17-defect-act1":{"act1":"guardian"}}),
        json!({"m19-defect-act3":{"act1":"guardian","act2":"champ"}}),
        json!({"m19-defect-act3":{"act1":"guardian","act3":"awakened-one"}}),
        json!({"m19-defect-act3":{"act2":"champ","act3":"awakened-one"}}),
        json!({"m19-defect-act3":{"act1":"guardian","act2":"champ","act3":"unknown"}}),
        json!({"m19-defect-act3":{"act1":"guardian","act2":"champ","act3":"awakened-one","extra":0}}),
        json!({"m19-defect-act3":{}}),json!({"m19-defect-act3":{"act1":"guardian"}}),json!({"m19-defect-act3":{"act2":"champ"}}),
        json!({"m19-defect-act3":{"act1":"guardian","act2":"champ","extra":0}}),
        json!({"m19-defect-act3":{"act1":"unknown","act2":"champ"}}),json!({"m19-defect-act3":{"act1":"guardian","act2":"unknown"}}),
        json!({"m19-defect-act3":{"act1":"champ","act2":"guardian"}}),
        json!({"m19-defect-act3":{"act1":"guardian","act2":"champ"},"extra":{}})] {
        let mut bad=value.clone();bad["config"]["boss_history"]=history;assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let raw=String::from_utf8(bytes).unwrap();
    for (from,to) in [
        ("\"act1\":\"guardian\"","\"act1\":\"guardian\",\"act1\":\"guardian\""),
        ("\"act2\":\"champ\"","\"act2\":\"champ\",\"act2\":\"champ\""),
        ("\"act3\":\"awakened-one\"","\"act3\":\"awakened-one\",\"act3\":\"time-eater\""),
        ("\"m19-defect-act3\":{","\"m19-defect-act3\":{},\"m19-defect-act3\":{"),
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
fn m19_sidecar_is_absent_and_rejection_does_not_freeze() {
    let mut s=RunSession::new_m19(default_config()).unwrap();let b=s.reset(Some(7)).unwrap();let before=s.reproduction().unwrap();
    assert!(s.environment_spec().unwrap()["combat_view_id"].is_null());assert!(s.combat_observation_spec().is_err());assert!(s.combat_view().is_err());
    assert_eq!(s.reproduction().unwrap(),before);assert_eq!(s.decision().unwrap(),b);
    s.close().unwrap();assert_eq!(s.decision(),Err(RunSessionError::Closed));
}

#[test]
fn m19_postaccepted_capacity_and_delivery_failures_keep_one_prefix() {
    let mut cfg=default_config();cfg.capacities.map_nodes=1;let mut s=RunSession::new_m19(cfg).unwrap();let b=s.reset(Some(7)).unwrap();
    assert_eq!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::DeliveryFault("post-step Run encoding failed")));
    let record=s.record().unwrap().unwrap();assert_eq!(record.steps.len(),1);let fault=record.fault.as_ref().unwrap();
    assert!(fault.committed&&fault.gameplay_recorded);assert_eq!(fault.error.kind,RunRecordErrorKind::Capacity);
    let before=s.reproduction().unwrap();assert!(s.step(b.token,RunActionIndex(0)).is_err());assert!(s.reset(None).is_err());assert_eq!(s.reproduction().unwrap(),before);
    let replay=replay_run(&RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap(),RunCapacities::m10_default()).unwrap();
    assert_eq!(replay.accepted_count,1);assert_eq!(replay.final_decision,before.latest_decision.unwrap().unwrap());assert_eq!(replay.reported_fault,record.fault);
    s.close().unwrap();assert!(s.reproduction().unwrap().closed);
    let mut s=RunSession::new_m19(default_config()).unwrap();let b=s.reset(Some(11)).unwrap();s.step(b.token,RunActionIndex(0)).unwrap();
    let accepted=s.record().unwrap().unwrap().steps;s.fail_delivery("supplied delivery probe").unwrap();let before=s.reproduction().unwrap();
    assert!(s.step(b.token,RunActionIndex(0)).is_err());assert_eq!(s.record().unwrap().unwrap().steps,accepted);assert_eq!(s.reproduction().unwrap(),before);
    let r=s.record().unwrap().unwrap();assert_eq!(replay_run(&RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap(),RunCapacities::m10_default()).unwrap().accepted_count,1);
}

#[test]
fn m19_actual_endpoints_keep_terminal_reset_and_stale_precedence() {
    use spire_core::run::*;
    for (seed,scenario,core,trace) in retained() {
        if trace.fault_attempt.is_some(){continue}
        let d=core.decision().unwrap();let (mut s,_)=consume_single(seed,scenario,&core,&trace);
        let b=s.decision().unwrap();let before=s.reproduction().unwrap();
        if let Some(terminal)=d.terminal {
            assert!(matches!(terminal,RunTerminal::Death|RunTerminal::EmptyDeck|RunTerminal::Act3NoKeys));
            assert_eq!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::Terminal));
        } else if d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act4Ready){
            assert_eq!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::UnsupportedBoundary("M19 stops before the Act4 door")));
        }
        assert_eq!(s.reproduction().unwrap(),before);
        let reset=s.reset(None).unwrap();assert_ne!(reset.token,b.token);assert_eq!(s.reproduction().unwrap().ordinal,1);
        assert!(s.reproduction().unwrap().latest_map.is_none());
        assert_eq!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::StaleToken));s.close().unwrap();
    }
}

#[test]
fn m19_actual_empty_cage_child_delivery_retains_removed_ids_once() {
    use spire_core::{run::*,content::RelicId};
    let mut covered=0;
    for (seed,scenario,_,trace) in retained() {
        let Some(index)=trace.trace.iter().position(|step|step.decision.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::BossRelicChild)&&matches!(step.action,GameAction::Select(_))&&step.decision.selection.as_ref().is_some_and(|s|s.required==2&&s.selected_count==1)) else {continue};
        covered+=1;let cfg=case_config(scenario);let mut core=caller::make_run(seed,scenario).unwrap();let mut s=RunSession::new_m19(cfg.clone()).unwrap();let mut boundary=s.reset(Some(seed)).unwrap();
        for step in &trace.trace[..=index] {
            assert_eq!(core.decision().unwrap(),step.decision);let candidate=step.decision.candidates.iter().position(|c|c.action==step.action).unwrap();
            let transition=core.step(&step.action).unwrap();let native=s.step(boundary.token,RunActionIndex(candidate as i64)).unwrap();assert_eq!(native.reward,transition.reward);boundary=native.boundary;
        }
        let d=core.decision().unwrap();let before_ids=trace.trace[index].decision.objects.iter().map(|c|c.instance).collect::<Vec<_>>();let after_ids=d.objects.iter().map(|c|c.instance).collect::<Vec<_>>();assert_eq!(before_ids.len(),after_ids.len()+2);
        let first=&trace.trace[index-1];assert!(matches!(first.action,GameAction::Select(_)));assert_eq!(first.decision.selection.as_ref().unwrap().selected_count,0);assert_eq!(first.decision.objects,trace.trace[index].decision.objects,"first pick stages identity without removing");
        let selected=[first,&trace.trace[index]].map(|step|step.decision.candidates.iter().find(|c|c.action==step.action).unwrap().object.as_ref().unwrap().instance);
        let removed=before_ids.iter().filter(|id|!after_ids.contains(id)).copied().collect::<Vec<_>>();assert_eq!(removed.len(),2);assert!(selected.iter().all(|id|removed.contains(id)),"second pick commits precisely both selected permanent identities");
        assert_eq!(d.player.relics.iter().filter(|r|**r==RelicId::EmptyCage).count(),1);
        assert_eq!(s.reproduction().unwrap().latest_decision.unwrap().unwrap(),d);let record=s.record().unwrap().unwrap();
        s.fail_delivery("actual EmptyCage accepted child delivery probe").unwrap();let retained=s.reproduction().unwrap();
        assert!(s.step(boundary.token,RunActionIndex(0)).is_err());assert_eq!(s.reproduction().unwrap(),retained);assert_eq!(s.record().unwrap().unwrap().steps,record.steps);
        let r=s.record().unwrap().unwrap();let report=replay_run(&RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap(),cfg.capacities).unwrap();assert_eq!(report.final_decision,d);assert_eq!(report.final_map,core.public_map());assert_eq!(report.accepted_count,index+1);s.close().unwrap();
        // Static selection capacity fails after accepting the actual parent
        // choice. Every earlier choice is identical to the original receipt.
        let parent=trace.trace[..index].iter().rposition(|step|step.decision.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::BossRelicChoice)).unwrap();
        let required=trace.trace[parent+1].decision.selection.as_ref().expect("actual EmptyCage selection").eligible.len();
        let cap=required-1;
        assert!(trace.trace[..=parent].iter().all(|step|step.decision.selection.as_ref().is_none_or(|selection|selection.eligible.len()<=cap)),"UNMET: earlier selection exceeds static capacity needed for isolated actual child failure");
        let mut small=case_config(scenario);small.capacities.selection=cap;let mut capacity=RunSession::new_m19(small).unwrap();let mut b=capacity.reset(Some(seed)).unwrap();
        for (i,step) in trace.trace[..=parent].iter().enumerate() {
            let candidate=step.decision.candidates.iter().position(|c|c.action==step.action).unwrap();let result=capacity.step(b.token,RunActionIndex(candidate as i64));
            if i==parent {assert_eq!(result,Err(RunSessionError::DeliveryFault("post-step Run encoding failed")));}
            else {b=result.unwrap().boundary;}
        }
        let retained=capacity.reproduction().unwrap();let r=capacity.record().unwrap().unwrap();assert_eq!(r.steps.len(),parent+1);assert!(r.fault.as_ref().unwrap().committed);assert!(capacity.step(b.token,RunActionIndex(0)).is_err());assert_eq!(capacity.reproduction().unwrap(),retained);
    }
    assert!(covered>0,"UNMET: original six contain no actual EmptyCage child; supplied diagnostics cannot establish from-start custody");
}
