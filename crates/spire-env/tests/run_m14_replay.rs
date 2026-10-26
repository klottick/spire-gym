use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    replay_run, RunAct1BossTarget, RunAct2BossTarget, RunAct3BossTarget, RunBossHistory,
    RunCapacities, RunEpisodeRecord, RunSession, RunSessionConfig, RunSessionError,
    RUN_CONTENT_ID_M14, RUN_RECORD_SCHEMA_ID_M14, RUN_RULES_ID_M14,
};

fn config(character: Character, history: RunBossHistory) -> RunSessionConfig {
    RunSessionConfig {
        character,
        ascension: 0,
        partition: Partition::Test,
        boss_history: history,
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn template_record() -> RunEpisodeRecord {
    let history = RunBossHistory::M14SilentAct3 {
        act1: RunAct1BossTarget::Guardian,
        act2: RunAct2BossTarget::Champ,
        act3: RunAct3BossTarget::AwakenedOne,
    };
    let mut session = RunSession::new_m14(config(Character::Silent, history)).unwrap();
    session.reset(Some(7)).unwrap();
    session.record().unwrap().unwrap()
}

fn typed_triples() -> Vec<(RunAct1BossTarget, RunAct2BossTarget, RunAct3BossTarget)> {
    use RunAct1BossTarget as A1;
    use RunAct2BossTarget as A2;
    use RunAct3BossTarget as A3;
    [A1::Guardian, A1::Hexaghost, A1::SlimeBoss].into_iter().flat_map(|act1|
        [A2::Champ, A2::BronzeAutomaton, A2::Collector].into_iter().flat_map(move |act2|
            [A3::AwakenedOne, A3::TimeEater, A3::DonuAndDeca].into_iter().map(move |act3| (act1, act2, act3))
        )
    ).collect()
}

#[test]
fn m14_exact_triple_json_accepts_all_27_combinations() {
    let template = template_record();
    assert_eq!(template.schema_id, RUN_RECORD_SCHEMA_ID_M14);
    assert_eq!(template.profile_id, "M14SilentAct3");
    assert_eq!(template.rules_id, RUN_RULES_ID_M14);
    assert_eq!(template.content_id, RUN_CONTENT_ID_M14);

    for (act1, act2, act3) in typed_triples() {
        let mut record = template.clone();
        record.config.boss_history = RunBossHistory::M14SilentAct3 { act1, act2, act3 };
        let bytes = record.to_json().unwrap();
        let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let history = &wire["config"]["boss_history"]["m14-silent-act3"];
        assert_eq!(history.as_object().unwrap().len(), 3);
        assert!(history.get("act1").is_some());
        assert!(history.get("act2").is_some());
        assert!(history.get("act3").is_some());
        assert_eq!(RunEpisodeRecord::from_json(&bytes).unwrap(), record);
    }

    let bytes = template.to_json().unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(wire["config"]["boss_history"], serde_json::json!({
        "m14-silent-act3":{"act1":"guardian","act2":"champ","act3":"awakened-one"}
    }));
}

#[test]
fn m14_triple_json_is_strict_and_profile_bound() {
    let template = template_record();
    let bytes = template.to_json().unwrap();
    let mut valid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let invalid_histories = [
        serde_json::json!({"m14-silent-act3":{"act2":"champ","act3":"awakened-one"}}),
        serde_json::json!({"m14-silent-act3":{"act1":"guardian","act2":"champ","act3":"awakened-one","act4":"heart"}}),
        serde_json::json!({"m14-silent-act2":{"act1":"guardian","act2":"champ"}}),
        serde_json::json!({"m14-silent-act3":{"act1":"guardian","act2":"champ","act3":"awakened_one"}}),
        serde_json::json!({"m14-silent-act3":{"act1":"unknown","act2":"champ","act3":"awakened-one"}}),
        serde_json::json!({"m14-silent-act3":{"act1":"guardian","act2":"bad","act3":"awakened-one"}}),
        serde_json::json!({"m14-silent-act3":{"act1":"guardian","act2":"champ","act3":"awakened-one"},"extra":"field"}),
        serde_json::json!({"m13-silent-act2":{"act1":"guardian","act2":"champ"}}),
    ];
    for invalid in invalid_histories {
        valid["config"]["boss_history"] = invalid;
        assert!(matches!(
            RunEpisodeRecord::from_json(&serde_json::to_vec(&valid).unwrap()),
            Err(RunSessionError::IncompatibleReplay(_))
        ), "invalid triple accepted: {}", valid["config"]["boss_history"]);
        valid = serde_json::from_slice(&bytes).unwrap();
    }

    valid["schema_id"] = serde_json::json!("spire-gym-m13-silent-act2-replay-v1");
    assert!(matches!(
        RunEpisodeRecord::from_json(&serde_json::to_vec(&valid).unwrap()),
        Err(RunSessionError::IncompatibleReplay(_))
    ));
    valid = serde_json::from_slice(&bytes).unwrap();
    valid["config"]["boss_history"] = serde_json::json!({
        "m13-silent-act2":{"act1":"guardian","act2":"champ"}
    });
    assert!(matches!(
        RunEpisodeRecord::from_json(&serde_json::to_vec(&valid).unwrap()),
        Err(RunSessionError::IncompatibleReplay(_))
    ), "M13 pair cannot be imported as M14 config");

    let mut changed_fingerprint = template.clone();
    changed_fingerprint.observation_fingerprint[0] ^= 1;
    assert!(matches!(
        replay_run(&changed_fingerprint, RunCapacities::m10_default()),
        Err(RunSessionError::IncompatibleReplay("Run observation fingerprint mismatch"))
    ));
    let mut changed_identity = template;
    changed_identity.content_id.push_str("-wrong");
    assert!(matches!(
        replay_run(&changed_identity, RunCapacities::m10_default()),
        Err(RunSessionError::IncompatibleReplay("Run identity mismatch"))
    ));
}

#[test]
fn old_scalar_and_m13_pair_json_keep_their_original_shapes() {
    use RunAct1BossTarget as A1;
    use RunAct2BossTarget as A2;
    use RunBossHistory as H;

    for (history, wire) in [
        (H::AwakenedUnseen, "awakened-unseen"),
        (H::TimeEaterUnseen, "time-eater-unseen"),
        (H::DonuDecaUnseen, "donu-deca-unseen"),
    ] {
        let mut session = RunSession::new(config(Character::Ironclad, history)).unwrap();
        session.reset(Some(5)).unwrap();
        let record = session.record().unwrap().unwrap();
        let bytes = record.to_json().unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["config"]["boss_history"], wire);
        assert_eq!(RunEpisodeRecord::from_json(&bytes).unwrap(), record);
    }

    for (history, wire) in [
        (H::Act1GuardianUnseen, "act1-guardian-unseen"),
        (H::Act1HexaghostUnseen, "act1-hexaghost-unseen"),
        (H::Act1SlimeBossUnseen, "act1-slime-boss-unseen"),
    ] {
        let mut session = RunSession::new_m12(config(Character::Silent, history)).unwrap();
        session.reset(Some(5)).unwrap();
        let record = session.record().unwrap().unwrap();
        let bytes = record.to_json().unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["config"]["boss_history"], wire);
        assert_eq!(RunEpisodeRecord::from_json(&bytes).unwrap(), record);
    }

    let history = H::M13SilentAct2 { act1: A1::Hexaghost, act2: A2::BronzeAutomaton };
    let mut session = RunSession::new_m13(config(Character::Silent, history)).unwrap();
    session.reset(Some(5)).unwrap();
    let record = session.record().unwrap().unwrap();
    let bytes = record.to_json().unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["config"]["boss_history"], serde_json::json!({
        "m13-silent-act2":{"act1":"hexaghost","act2":"bronze_automaton"}
    }));
    assert_eq!(RunEpisodeRecord::from_json(&bytes).unwrap(), record);
}

#[test]
fn m14_zero_action_record_replays_its_normal_start_without_completing_the_run() {
    let record = template_record();
    let replay = replay_run(&record, RunCapacities::m10_default()).unwrap();
    assert_eq!(replay.accepted_count, 0);
    assert_eq!(replay.final_decision.terminal, None);
    assert!(replay.final_decision.room.is_some());
}
