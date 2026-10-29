mod m19_support;
use m19_support::*;
use spire_env::*;
#[test]
fn m19_short_diagnostic_portable_replay_uses_same_choices() {
    let cfg=default_config();let mut core=diagnostic_run();let mut s=RunSession::new_m19(cfg.clone()).unwrap();let b=s.reset(Some(7)).unwrap();
    assert_eq!(s.reproduction().unwrap().latest_decision.unwrap().unwrap(),core.decision().unwrap());
    let action=core.decision().unwrap().candidates[0].action.clone();let t=core.step(&action).unwrap();
    let result=s.step(b.token,RunActionIndex(0)).unwrap();assert_eq!(result.reward,t.reward);
    let record=s.record().unwrap().unwrap();let bytes=record.to_json().unwrap();let decoded=RunEpisodeRecord::from_json(&bytes).unwrap();
    let report=replay_run(&decoded,cfg.capacities).unwrap();assert_eq!(report.final_decision,core.decision().unwrap());assert_eq!(report.final_map,core.public_map());assert_eq!(report.accepted_count,1);
}

#[test]
fn m19_session_and_portable_replay_match_all_fixed_core_transcripts() {
    for (seed,scenario,core,trace) in retained() {
        let (mut session,record)=consume_single(seed,scenario,&core,&trace);
        let mut wrong=record.clone();wrong.profile_id="M17DefectAct1".into();assert!(replay_run(&wrong,RunCapacities::m10_default()).is_err());
        if trace.fault_attempt.is_none() {
            let d=core.decision().unwrap();let b=session.decision().unwrap();let before=session.reproduction().unwrap();
            if d.terminal.is_some() {assert_eq!(session.step(b.token,RunActionIndex(0)),Err(RunSessionError::Terminal));}
            else if d.room.as_ref().is_some_and(|r|r.kind==spire_core::run::RoomViewKind::Act4Ready) {assert_eq!(session.step(b.token,RunActionIndex(0)),Err(RunSessionError::UnsupportedBoundary("M19 stops before the Act4 door")));}
            assert_eq!(session.reproduction().unwrap(),before);
        }
    }
}
