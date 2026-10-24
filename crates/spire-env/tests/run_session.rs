use spire_env::*;
use spire_core::{combat::rng::Partition,content::Character,run::*};
#[path="../../spire-core/examples/m9_support/mod.rs"] mod support;
fn config(record:bool)->RunSessionConfig{RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m9_default(),record_replay:record}}

#[test]
fn run_session_reset_uses_real_m9_start_and_owned_public_encoding(){
    let mut s=RunSession::new(config(true)).unwrap();assert_eq!(s.decision(),Err(RunSessionError::NotReset));
    for seed in [7,11]{
        let b=s.reset(Some(seed)).unwrap();let r=support::start(seed,support::M9BossHistory::AwakenedUnseen).unwrap();
        assert_eq!(b.observation,encode_run(&r.decision().unwrap(),r.public_map().as_ref(),&config(true).capacities).unwrap());assert_eq!(b.terminal,None);
        assert_eq!(s.decision().unwrap(),b);let mut detached=s.decision().unwrap();detached.observation.groups[0].values[0]=999;assert_eq!(s.decision().unwrap(),b);
    }
    let mut bad=config(false);bad.character=Character::Silent;assert!(matches!(RunSession::new(bad),Err(RunSessionError::Input(_))));
    let mut bad=config(false);bad.ascension=1;assert!(matches!(RunSession::new(bad),Err(RunSessionError::Input(_))));
    s.close().unwrap();assert_eq!(s.decision(),Err(RunSessionError::Closed));assert_eq!(s.reset(None),Err(RunSessionError::Closed));assert!(s.record().unwrap().unwrap().closed);
}

#[test]
fn run_session_stale_cross_session_and_masked_indices_do_not_mutate(){
    let mut s=RunSession::new(config(true)).unwrap();let b=s.reset(Some(11)).unwrap();
    let mut other=RunSession::new(config(true)).unwrap();let foreign=other.reset(Some(11)).unwrap();
    for (token,index,error) in [(foreign.token,0,RunSessionError::StaleToken),(b.token,-1,RunSessionError::Input("action index is not legal")),(b.token,2,RunSessionError::Input("action index is not legal")),(b.token,2048,RunSessionError::Input("action index is not legal"))]{
        let before=s.record().unwrap();assert_eq!(s.step(token,RunActionIndex(index)),Err(error));assert_eq!(s.decision().unwrap(),b);assert_eq!(s.record().unwrap(),before);
    }
    s.step(b.token,RunActionIndex(0)).unwrap();let next=s.decision().unwrap();assert_ne!(next.token,b.token);
    assert_eq!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::StaleToken));assert_eq!(s.decision().unwrap(),next);
}

#[test]
fn run_session_reset_reseeds_or_continues_ordinal_without_hidden_resources(){
    let mut s=RunSession::new(config(true)).unwrap();s.reset(None).unwrap();let first=s.record().unwrap().unwrap();assert_eq!((first.seed,first.ordinal),(0,0));
    s.reset(None).unwrap();let second=s.record().unwrap().unwrap();assert_eq!((second.seed,second.ordinal),(0,1));
    let b=s.reset(Some(11)).unwrap();let second=s.reset(Some(11)).unwrap();assert_eq!(b.observation,second.observation);assert_ne!(b.token,second.token);assert_eq!(s.record().unwrap().unwrap().ordinal,0);
    s.reset(None).unwrap();let record=s.record().unwrap().unwrap();assert_eq!((record.seed,record.ordinal),(11,1));
    let mut root=[0;32];root[..8].copy_from_slice(&11_u64.to_le_bytes());let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];p.seen_bosses[1]=vec!["Champ","Collector"];p.seen_bosses[2]=vec!["TimeEater","DonuAndDeca"];
    let mut r=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},p,RunRng::new(root,Partition::Test,1),RoomScenario::M9IroncladAct3).unwrap();
    assert_eq!(s.decision().unwrap().observation,encode_run(&r.decision().unwrap(),r.public_map().as_ref(),&record.config.capacities).unwrap());
    let b=s.decision().unwrap();r.step(&r.decision().unwrap().candidates[0].action).unwrap();
    let t=s.step(b.token,RunActionIndex(0)).unwrap();assert_eq!(t.boundary.observation,encode_run(&r.decision().unwrap(),r.public_map().as_ref(),&record.config.capacities).unwrap());
}

#[test]
fn run_session_errors_are_not_gameplay_outcomes_and_keep_accepted_prefix(){
    let mut cfg=config(true);cfg.capacities.map_nodes=1;
    let mut s=RunSession::new(cfg).unwrap();let b=s.reset(Some(11)).unwrap();
    let error=s.step(b.token,RunActionIndex(0)).unwrap_err();assert!(matches!(error,RunSessionError::DeliveryFault(_)));
    assert_eq!(s.decision(),Err(error.clone()));assert_eq!(s.reset(Some(7)),Err(error.clone()));assert_eq!(s.step(b.token,RunActionIndex(0)),Err(error));
    let record=s.record().unwrap().unwrap();assert_eq!(record.steps.len(),1);assert_eq!((record.steps[0].reward,record.terminal),(0,None));
    assert!(record.fault.as_ref().is_some_and(|f|f.committed&&f.gameplay_recorded&&f.error.kind==RunRecordErrorKind::Capacity&&f.error.capacity.as_ref().is_some_and(|c|c.group=="map_nodes")));
    let reproduction=s.reproduction().unwrap();assert_eq!(reproduction.record,Some(record.clone()));assert_eq!(reproduction.latest_decision.unwrap().unwrap().room.unwrap().kind,RoomViewKind::Map);assert!(reproduction.latest_map.unwrap().nodes.len()>1);
    s.close().unwrap();s.close().unwrap();let closed=s.record().unwrap().unwrap();assert_eq!(closed.steps,record.steps);assert_eq!(closed.fault,record.fault);

    let mut cfg=config(true);cfg.capacities.permanent_cards=1;let mut s=RunSession::new(cfg).unwrap();let error=s.reset(Some(11)).unwrap_err();assert!(matches!(error,RunSessionError::Capacity{..}));assert_eq!(s.decision(),Err(error.clone()));assert_eq!(s.reset(None),Err(error));assert!(s.record().unwrap().unwrap().fault.is_some());
    let mut s=RunSession::new(config(true)).unwrap();let b=s.reset(Some(11)).unwrap();s.step(b.token,RunActionIndex(0)).unwrap();s.fail_delivery("Python tuple packaging failed").unwrap();
    let before=s.record().unwrap();assert!(matches!(s.decision(),Err(RunSessionError::DeliveryFault(_))));assert!(s.reset(None).is_err());assert_eq!(s.record().unwrap(),before);
}

#[test]
fn run_session_natural_death_is_terminated_not_truncated_and_recording_is_passive(){
    let trace=support::death(11,8192).unwrap();let mut s=RunSession::new(config(true)).unwrap();let mut b=s.reset(Some(11)).unwrap();let mut plain=RunSession::new(config(false)).unwrap();let mut pb=plain.reset(Some(11)).unwrap();
    let mut core=support::start(11,support::M9BossHistory::AwakenedUnseen).unwrap();
    for step in &trace.trace{
        let d=core.decision().unwrap();let index=d.candidates.iter().position(|c|c.action==step.action).unwrap() as i64;
        let t=s.step(b.token,RunActionIndex(index)).unwrap();let p=plain.step(pb.token,RunActionIndex(index)).unwrap();assert_eq!(t.reward,step.reward);assert_eq!(t.boundary.terminal,step.decision.terminal);assert!(!t.truncated);
        assert_eq!(t.boundary.observation,p.boundary.observation);assert_eq!(t.terminated,step.decision.terminal.is_some());core.step(&step.action).unwrap();
        assert_eq!(t.boundary.observation,encode_run(&core.decision().unwrap(),core.public_map().as_ref(),&config(true).capacities).unwrap());b=t.boundary;pb=p.boundary;
    }
    assert_eq!(b.terminal,Some(RunTerminal::Death));assert!(b.observation.action_mask.iter().all(|v|*v==0));assert_eq!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::Terminal));
    let record=s.record().unwrap().unwrap();assert_eq!(record.steps.len(),trace.trace.len());assert_eq!(record.terminal,Some(RunTerminal::Death));assert!(plain.record().unwrap().is_none());
}
