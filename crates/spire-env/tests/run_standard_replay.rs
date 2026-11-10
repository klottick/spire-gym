use spire_core::{combat::rng::Partition, content::{CardId, Character}};
use spire_env::{replay_run, RunActionIndex, RunEpisodeRecord, RunSession, RunSessionError, StandardCreation, StandardNeow, StandardRunConfig};

fn config(character: Character, ascension: u8) -> StandardRunConfig {
    let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
    config.record_replay = true;
    config.creation = StandardCreation { neow: StandardNeow::Full, seen_bosses: [[false; 3]; 3], stored_note: (CardId::IronWave, 0) };
    config
}

fn run_one_step(character: Character, ascension: u8, seed: u64) -> RunEpisodeRecord {
    let mut session = RunSession::new_standard(config(character, ascension)).unwrap();
    let boundary = session.reset(Some(seed)).unwrap();
    let index = boundary.observation.action_mask.iter().position(|&v| v != 0).unwrap();
    session.step(boundary.token, RunActionIndex(index as i64)).unwrap();
    session.record().unwrap().unwrap()
}

#[test]
fn standard_record_json_round_trips() {
    for character in Character::ALL {
        for ascension in [0, 20] {
            let record = run_one_step(character, ascension, 1000 + u64::from(ascension));
            assert_eq!(record.config.boss_history, spire_env::RunBossHistory::Standard(config(character, ascension).creation));
            let bytes = record.to_json().unwrap();
            let decoded = RunEpisodeRecord::from_json(&bytes).unwrap();
            assert_eq!(decoded, record);
            let report = replay_run(&decoded, decoded.config.capacities.clone()).unwrap();
            assert_eq!(report.accepted_count, 1);
            assert_eq!(report.rewards, vec![record.steps[0].reward]);
        }
    }
}

#[test]
fn standard_replay_rejects_mismatched_ids_and_creation() {
    let original = run_one_step(Character::Ironclad, 0, 511);
    for field in ["rules_id", "content_id", "schema_id"] {
        let mut tampered = original.clone();
        match field { "rules_id" => tampered.rules_id.push_str("-wrong"), "content_id" => tampered.content_id.push_str("-wrong"), _ => tampered.schema_id.push_str("-wrong") }
        assert!(matches!(replay_run(&tampered, tampered.config.capacities.clone()), Err(RunSessionError::IncompatibleReplay(_))));
    }

    let mut distinguished = false;
    let mut source = RunSession::new_standard(config(Character::Ironclad, 0)).unwrap();
    let initial = source.reset(Some(511)).unwrap();
    let legal = initial.observation.action_mask.iter().enumerate().filter_map(|(i, &v)| (v != 0).then_some(i)).collect::<Vec<_>>();
    for index in legal {
        let mut branch = RunSession::new_standard(config(Character::Ironclad, 0)).unwrap();
        let boundary = branch.reset(Some(511)).unwrap();
        if branch.step(boundary.token, RunActionIndex(index as i64)).is_err() { continue; }
        let mut changed = branch.record().unwrap().unwrap();
        if let spire_env::RunBossHistory::Standard(mut creation) = changed.config.boss_history {
            creation.neow = StandardNeow::Short;
            changed.config.boss_history = spire_env::RunBossHistory::Standard(creation);
        }
        if matches!(replay_run(&changed, changed.config.capacities.clone()), Err(RunSessionError::IncompatibleReplay(_))) {
            distinguished = true;
            break;
        }
    }
    assert!(distinguished, "no offered Neow action distinguished full from short creation (initial record: {original:?})");
}

#[test]
fn standard_replay_rejects_captured_boss_history_under_standard_profile() {
    let record = run_one_step(Character::Ironclad, 0, 612);
    let mut json: serde_json::Value = serde_json::from_slice(&record.to_json().unwrap()).unwrap();
    let mut wrong_act = json.clone();
    wrong_act["config"]["boss_history"]["standard"]["seen_bosses"][1] = serde_json::json!(["Guardian"]);
    assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&wrong_act).unwrap()).is_err());
    let mut unknown_boss = json.clone();
    unknown_boss["config"]["boss_history"]["standard"]["seen_bosses"][0] = serde_json::json!(["UnknownBoss"]);
    assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&unknown_boss).unwrap()).is_err());
    json["config"]["boss_history"] = serde_json::json!({"awakened-unseen": {}});
    assert!(matches!(RunEpisodeRecord::from_json(&serde_json::to_vec(&json).unwrap()), Err(RunSessionError::IncompatibleReplay(_))));
}

#[test]
fn standard_replay_reproduces_faults() {
    let mut config = config(Character::Silent, 0);
    config.capacities.map_nodes = 1;
    let mut session = RunSession::new_standard(config).unwrap();
    let mut boundary = session.reset(Some(22)).unwrap();
    let mut fault = None;
    for _ in 0..8 {
        let index = boundary.observation.action_mask.iter().position(|&v| v != 0).unwrap();
        match session.step(boundary.token, RunActionIndex(index as i64)) {
            Ok(result) => boundary = result.boundary,
            Err(error) => { fault = Some(error); break; },
        }
    }
    let fault = fault.expect("map capacity should be exceeded when Neow enters the map");
    // Shared session contract (M22–M25 tests): a post-commit publication failure returns DeliveryFault (the step committed);
    // the record and reproduction keep the typed Capacity failure with its group.
    assert_eq!(fault, RunSessionError::DeliveryFault("post-step Run encoding failed"));
    let record = session.record().unwrap().unwrap();
    assert_eq!(record.fault.as_ref().map(|f| (f.committed, f.gameplay_recorded)), Some((true, true)));
    assert_eq!(record.fault.as_ref().unwrap().error.kind, spire_env::RunRecordErrorKind::Capacity);
    assert_eq!(record.fault.as_ref().unwrap().error.capacity.as_ref().map(|c| c.group.as_str()), Some("map_nodes"));
    let report = replay_run(&record, record.config.capacities.clone());
    assert!(report.is_ok(), "recorded capacity fault did not replay: {report:?}");
    let report = report.unwrap();
    assert_eq!(report.accepted_count, record.steps.len());
    assert_eq!(report.reported_fault, record.fault);
    assert_eq!(record.terminal, None);
}
