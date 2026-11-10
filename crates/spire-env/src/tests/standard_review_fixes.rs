//! M34b final review fixes: m1 (a reward-item sum fault is recorded and replays) and m2 (exact Standard record shape).
use crate::*;
use crate::run_session::{REWARD_ITEMS_FAULT, TEST_REWARD_SKEW};
use spire_core::{combat::rng::Partition, content::Character};

fn session() -> RunSession {
    let mut config = StandardRunConfig::new(Character::Ironclad, 0, Partition::Test);
    config.record_replay = true;
    RunSession::new_standard(config).unwrap()
}

struct Skew;
impl Drop for Skew { fn drop(&mut self) { TEST_REWARD_SKEW.with(|s| s.set(false)) } }

#[test]
fn reward_item_sum_fault_is_recorded_and_replays() {
    let mut s = session();
    let mut boundary = s.reset(Some(5)).unwrap();
    let _skew = Skew;
    TEST_REWARD_SKEW.with(|v| v.set(true));
    let error = loop {
        let index = boundary.observation.action_mask.iter().position(|m| *m == 1).unwrap();
        match s.step(boundary.token, RunActionIndex(index as i64)) {
            Ok(result) => { assert!(result.reward_events.is_empty(), "a credited step must fault under the skew"); boundary = result.boundary; }
            Err(error) => break error,
        }
    };
    assert_eq!(error, RunSessionError::DeliveryFault(REWARD_ITEMS_FAULT));
    let record = s.record().unwrap().unwrap();
    let fault = record.fault.clone().unwrap();
    assert!(fault.committed && fault.gameplay_recorded);
    assert_eq!(fault.action.as_ref(), Some(&record.steps.last().unwrap().action), "the committed step is recorded");
    assert!(s.step(boundary.token, RunActionIndex(0)).is_err() && s.reset(None).is_err(), "frozen: no retry");
    let decoded = RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
    let report = replay_run(&decoded, decoded.config.capacities.clone()).unwrap();
    assert_eq!((report.accepted_count, report.reported_fault.as_ref()), (record.steps.len(), Some(&fault)));
    TEST_REWARD_SKEW.with(|v| v.set(false));
    assert_eq!(replay_run(&decoded, decoded.config.capacities.clone()).unwrap_err(),
        RunSessionError::IncompatibleReplay("recorded reward item fault no longer occurs"));
}

#[test]
fn standard_record_import_rejects_unknown_keys() {
    let mut s = session();
    let boundary = s.reset(Some(6)).unwrap();
    let index = boundary.observation.action_mask.iter().position(|m| *m == 1).unwrap();
    s.step(boundary.token, RunActionIndex(index as i64)).unwrap();
    let bytes = s.record().unwrap().unwrap().to_json().unwrap();
    assert!(RunEpisodeRecord::from_json(&bytes).is_ok());
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mutate = |f: &dyn Fn(&mut serde_json::Value)| { let mut v = value.clone(); f(&mut v); serde_json::to_vec(&v).unwrap() };
    for (name, bytes) in [
        ("top level", mutate(&|v| { v["unexpected_extra"] = serde_json::json!({"rng": [1, 2, 3]}); })),
        ("step", mutate(&|v| { v["steps"][0]["extra"] = serde_json::json!(1); })),
        ("config", mutate(&|v| { v["config"]["extra"] = serde_json::json!(1); })),
        ("creation", mutate(&|v| { v["config"]["boss_history"]["standard"]["extra"] = serde_json::json!(1); })),
        ("boss history", mutate(&|v| { v["config"]["boss_history"]["extra"] = serde_json::json!(1); })),
    ] {
        assert!(RunEpisodeRecord::from_json(&bytes).is_err(), "{name}: an unknown key must be rejected");
    }
}
