//! Native state/RNG equality, not only report/checksum equivalence.
#[path="../examples/run_driver.rs"] mod driver;
#[path="common/native.rs"] mod native;
use spire_report::*;
use spire_env::*;
use std::sync::Arc;

#[test]
fn determinism_legacy() {
    for seed in [1,2,3] {
        let pool=CombatPool::new(Arc::new(ScenarioCatalog::starter()),PoolConfig {record_replay:true,..PoolConfig::default()}).unwrap();
        let mut tokens=pool.reset_batch(&[0],&[ResetRequest {seed:Some(IntegerSeed::from_be_bytes(&[seed]).unwrap()),scenario_id:None}]).unwrap().decisions.tokens;
        for index in [0,1,0] {tokens=pool.step_batch(&[index],&tokens).unwrap().current.tokens;}
        let record=pool.reproduction(0).unwrap().unwrap();let copy=record.clone();let mut visits=vec![];
        let observed=replay_observed(&record,|v|{visits.push(v);Ok(())}).unwrap();let plain=replay(&record).unwrap();
        assert_eq!(observed,plain);assert_eq!(record,copy);
        assert_eq!(observed.final_snapshot,pool.diagnostics().unwrap().slots[0].snapshot.clone().unwrap());
        assert_eq!(visits.iter().map(|v|v.boundary.clone().unwrap()).collect::<Vec<_>>(),plain.boundaries);
    }
}
#[test]
fn determinism_run() {
    for seed in [7,11] {
        let source=driver::source::prefix(seed,vec!["Automaton","Collector"],4096).unwrap();
        let mut plain=driver::source::start(seed,vec!["Automaton","Collector"]).unwrap();let mut recorded=plain.clone();
        let mut writer=ReportWriter::new(std::io::sink(),native::header()).unwrap();let mut recorder=RunRecorder::new(&mut writer);recorder.start(&recorded).unwrap();
        for step in source.trace {
            let plain_transition=plain.step(&step.action).unwrap();let recorded_transition=recorder.step(&mut recorded,&step.action).unwrap();
            assert_eq!(plain,recorded);assert_eq!(plain_transition,recorded_transition);
            assert_eq!(plain.decision().unwrap(),step.decision);assert_eq!(plain.public_map(),step.map);assert_eq!(plain_transition.reward,step.reward);
        }
    }
}
