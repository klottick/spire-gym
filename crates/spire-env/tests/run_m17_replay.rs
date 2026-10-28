mod m17_support;
use m17_support::*;
use spire_env::*;
#[test]
fn m17_single_and_portable_replay_match_all_core_cases(){
    for (seed,scenario,run,caller) in retained(){
        let cfg=config(scenario);let mut s=RunSession::new_m17(cfg.clone()).unwrap();let mut b=s.reset(Some(seed)).unwrap();
        for (i,step) in caller.trace.iter().enumerate(){
            let reproduction=s.reproduction().unwrap();assert_eq!(reproduction.latest_decision.unwrap().unwrap(),step.decision);assert_eq!(reproduction.latest_map,step.map);
            assert_eq!(b.observation,encode_run_m17(&step.decision,step.map.as_ref(),&cfg.capacities).unwrap());
            let index=step.decision.candidates.iter().position(|c|c.action==step.action).unwrap();let result=s.step(b.token,RunActionIndex(index as i64)).unwrap();assert_eq!(result.reward,step.transition.reward);assert_eq!(result.boundary.terminal,step.transition.terminal);assert!(!result.truncated);b=result.boundary;
            assert_eq!(s.record().unwrap().unwrap().steps.len(),i+1);
        }
        let final_d=run.decision().unwrap();assert_eq!(s.reproduction().unwrap().latest_decision.unwrap().unwrap(),final_d);assert_eq!(s.reproduction().unwrap().latest_map,run.public_map());
        let record=s.record().unwrap().unwrap();let bytes=record.to_json().unwrap();let decoded=RunEpisodeRecord::from_json(&bytes).unwrap();assert_eq!(record,decoded);let replay=replay_run(&decoded,cfg.capacities.clone()).unwrap();assert_eq!(replay.final_decision,final_d);assert_eq!(replay.final_map,run.public_map());assert_eq!(replay.rewards,caller.trace.iter().map(|s|s.transition.reward).collect::<Vec<_>>());assert_eq!(replay.terminals,caller.trace.iter().map(|s|s.transition.terminal).collect::<Vec<_>>());assert_eq!(replay.accepted_count,caller.accepted());assert!(replay.reported_fault.is_none());
        if let Some(dir)=std::env::var_os("M17_NATIVE_EVIDENCE_DIR"){let dir=std::path::PathBuf::from(dir);std::fs::create_dir_all(&dir).unwrap();std::fs::write(dir.join(format!("{seed}-{}-portable.json",scenario.label())),&bytes).unwrap();}
        let mut wrong=record.clone();wrong.profile_id="M12SilentAct1".into();assert!(replay_run(&wrong,cfg.capacities.clone()).is_err());
        let before=s.reproduction().unwrap();let rejected=s.step(b.token,RunActionIndex(0));
        if final_d.terminal.is_some(){assert_eq!(rejected,Err(RunSessionError::Terminal))}else{assert_eq!(rejected,Err(RunSessionError::UnsupportedBoundary("M17 stops at Act2Ready")));}
        assert_eq!(s.reproduction().unwrap(),before);let reset=s.reset(Some(seed)).unwrap();assert_ne!(reset.token,b.token);assert_eq!(s.reproduction().unwrap().latest_decision.unwrap().unwrap().player.hp,75);
    }
}
