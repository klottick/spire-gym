//! M34s-a a1: the Standard Run fingerprint changes once (draw rows and card lifecycle columns, design §4.12) and the rules id
//! moves to v2 (G-a4, R-a1; PM ruling), so a Standard record made before M34s is a typed `IncompatibleReplay` on every
//! replay surface (no legacy replay path).
use crate::*;
use spire_core::{combat::rng::Partition, content::Character};

/// Made on base d6572940 by a Standard Ironclad A0 session (seed 20261110, first legal action to the first combat decision
/// and three more steps); there `replay_run` accepted all six steps.
const PRE_M34S_RECORD: &[u8] = include_bytes!("fixtures/pre_m34s_standard_record_v1.json");
const PRE_M34S_FINGERPRINT: &str = "efbc1253544a80c5c4a5b3a63cb8169a215229aaffa4ac2af5402252e195311a";
const STANDARD_FINGERPRINT: &str = "bf8e1d111bbf2171594231b99d8d6b701244249abfbb90a11e86c4b4a405f7ae";

fn hex(bytes: &[u8; 32]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }

/// The same episode on this tree: its record differs from the fixture only in the fingerprint.
fn same_episode_record() -> RunEpisodeRecord {
    let mut config = StandardRunConfig::new(Character::Ironclad, 0, Partition::Test);
    config.record_replay = true;
    let mut session = RunSession::new_standard(config).unwrap();
    let mut boundary = session.reset(Some(20261110)).unwrap();
    for _ in 0..6 {
        let index = boundary.observation.action_mask.iter().position(|&v| v != 0).unwrap();
        boundary = session.step(boundary.token, RunActionIndex(index as i64)).unwrap().boundary;
    }
    session.record().unwrap().unwrap()
}

#[test]
fn standard_fingerprint_is_pinned() {
    let spec = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap();
    assert_eq!(hex(&spec.fingerprint()), STANDARD_FINGERPRINT);
    assert_ne!(STANDARD_FINGERPRINT, PRE_M34S_FINGERPRINT);
}

#[test]
fn pre_m34s_standard_record_is_incompatible_replay() {
    let fixture: serde_json::Value = serde_json::from_slice(PRE_M34S_RECORD).unwrap();
    assert_eq!(fixture["observation_fingerprint"].as_array().map(|a| a.iter().map(|b| format!("{:02x}", b.as_u64().unwrap())).collect::<String>()),
        Some(PRE_M34S_FINGERPRINT.to_owned()));
    assert_eq!(fixture["rules_id"], "spire-gym-native-standard-rules-v1");
    assert_eq!(crate::RUN_RULES_ID_STANDARD, "spire-gym-native-standard-rules-v4");
    // The identity (rules id) is checked before the fingerprint.
    let identity_mismatch = RunSessionError::IncompatibleReplay("Run identity mismatch");
    let fingerprint_mismatch = RunSessionError::IncompatibleReplay("Run observation fingerprint mismatch");
    assert_eq!(RunEpisodeRecord::from_json(PRE_M34S_RECORD).unwrap_err(), identity_mismatch);

    // The episode itself still replays: only the fingerprint separates the old record from this tree's.
    let current = same_episode_record();
    assert_eq!(hex(&current.observation_fingerprint), STANDARD_FINGERPRINT);
    let mut old = current.clone();
    old.rules_id = fixture["rules_id"].as_str().unwrap().to_owned();
    assert_eq!(replay_run(&old, RunCapacities::standard_default()).unwrap_err(), identity_mismatch);
    assert_eq!(RunSession::from_record(&old, 0, RunCapacities::standard_default()).err(), Some(identity_mismatch));
    // With only the rules id updated, the old fingerprint still rejects the record.
    let mut old = current.clone();
    old.observation_fingerprint = fixture["observation_fingerprint"].as_array().unwrap().iter().map(|b| b.as_u64().unwrap() as u8)
        .collect::<Vec<_>>().try_into().unwrap();
    let mut expected = fixture.clone();
    expected["observation_fingerprint"] = serde_json::to_value(current.observation_fingerprint).unwrap();
    expected["rules_id"] = serde_json::json!(crate::RUN_RULES_ID_STANDARD);
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&current.to_json().unwrap()).unwrap(), expected);
    assert_eq!(replay_run(&current, RunCapacities::standard_default()).unwrap().accepted_count, 6);

    assert_eq!(replay_run(&old, RunCapacities::standard_default()).unwrap_err(), fingerprint_mismatch);
    assert_eq!(RunSession::from_record(&old, 0, RunCapacities::standard_default()).err(), Some(fingerprint_mismatch));
}
