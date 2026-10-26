use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    replay_run, RunActionIndex, RunBossHistory, RunCapacities, RunEpisodeRecord, RunSession,
    RunSessionConfig, RunSessionError,
};

fn config() -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: RunBossHistory::Act1GuardianUnseen,
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn recorded_neow_step() -> RunEpisodeRecord {
    let mut session = RunSession::new_m12(config()).unwrap();
    let boundary = session.reset(Some(11)).unwrap();
    session.step(boundary.token, RunActionIndex(0)).unwrap();
    session.record().unwrap().unwrap()
}

#[test]
fn m12_replay_roundtrip_keeps_the_act1_boss_mapping_and_distinct_identity() {
    let record = recorded_neow_step();
    assert_eq!(record.schema_id, "spire-gym-m12-silent-act1-replay-v1");
    assert_eq!(record.profile_id, "M12SilentAct1");
    assert_eq!(record.rules_id, "spire-gym-native-m12-rules-v1");
    assert_eq!(record.content_id, "spire-gym-m12-silent-a0-act1-captured-v1");
    assert_eq!(record.config.boss_history, RunBossHistory::Act1GuardianUnseen);
    let json = record.to_json().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&json).unwrap();
    assert_eq!(value["schema_id"], "spire-gym-m12-silent-act1-replay-v1");
    assert_eq!(value["config"]["boss_history"], "act1-guardian-unseen");
    let decoded = RunEpisodeRecord::from_json(&json).unwrap();
    assert_eq!(decoded, record);
    let replay = replay_run(&decoded, RunCapacities::m10_default()).unwrap();
    assert_eq!(replay.accepted_count, 1);
    assert_eq!(replay.final_decision.room.as_ref().unwrap().kind, spire_core::run::RoomViewKind::Map);
    assert_eq!(replay.final_map.unwrap().boss, Some(spire_core::content::EncounterId::Guardian));
}

#[test]
fn m12_replay_fails_closed_on_identity_history_config_and_fingerprint_tampering() {
    let record = recorded_neow_step();
    let mut candidates = Vec::new();
    let mut bad = record.clone(); bad.schema_id.push('x'); candidates.push(bad);
    let mut bad = record.clone(); bad.profile_id.push('x'); candidates.push(bad);
    let mut bad = record.clone(); bad.rules_id.push('x'); candidates.push(bad);
    let mut bad = record.clone(); bad.content_id.push('x'); candidates.push(bad);
    let mut bad = record.clone(); bad.observation_fingerprint[0] ^= 1; candidates.push(bad);
    let mut bad = record.clone(); bad.config.boss_history = RunBossHistory::AwakenedUnseen; candidates.push(bad);
    let mut bad = record.clone(); bad.config.character = Character::Ironclad; candidates.push(bad);
    let mut bad = record.clone(); bad.config.ascension = 1; candidates.push(bad);

    for bad in candidates {
        assert!(matches!(replay_run(&bad, RunCapacities::m10_default()), Err(RunSessionError::IncompatibleReplay(_))));
        assert!(matches!(bad.to_json(), Err(RunSessionError::IncompatibleReplay(_))));
    }

    let mut value: serde_json::Value = serde_json::from_slice(&record.to_json().unwrap()).unwrap();
    value["config"]["boss_history"] = serde_json::json!("awakened-unseen");
    assert!(matches!(RunEpisodeRecord::from_json(&serde_json::to_vec(&value).unwrap()), Err(RunSessionError::IncompatibleReplay(_))));
}
