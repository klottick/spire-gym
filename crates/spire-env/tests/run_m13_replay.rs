use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    RunAct1BossTarget, RunAct2BossTarget, RunBossHistory, RunCapacities, RunEpisodeRecord,
    RunSession, RunSessionConfig, RunSessionError,
};

fn m13_config(act1: RunAct1BossTarget, act2: RunAct2BossTarget) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: RunBossHistory::M13SilentAct2 { act1, act2 },
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn m13_record() -> RunEpisodeRecord {
    let mut session = RunSession::new_m13(m13_config(
        RunAct1BossTarget::Guardian,
        RunAct2BossTarget::Champ,
    )).unwrap();
    session.reset(Some(7)).unwrap();
    session.record().unwrap().unwrap()
}

fn old_record(character: Character, history: RunBossHistory, profile: &str) -> RunEpisodeRecord {
    let config = RunSessionConfig {
        character,
        ascension: 0,
        partition: Partition::Test,
        boss_history: history,
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    };
    let mut session = match profile {
        "m9" => RunSession::new(config).unwrap(),
        "m10" => RunSession::new_m10(config).unwrap(),
        "m12" => RunSession::new_m12(config).unwrap(),
        _ => unreachable!(),
    };
    session.reset(Some(7)).unwrap();
    session.record().unwrap().unwrap()
}

#[test]
fn m13_boss_history_json_roundtrips_typed_pair_and_rejects_malformed_objects() {
    let record = m13_record();
    assert_eq!(record.schema_id, "spire-gym-m13-silent-act2-replay-v1");
    assert_eq!(record.profile_id, "M13SilentAct2");
    assert_eq!(record.rules_id, "spire-gym-native-m13-rules-v1");
    assert_eq!(record.content_id, "spire-gym-m13-silent-a0-acts1-2-captured-v1");
    let bytes = record.to_json().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        value["config"]["boss_history"],
        serde_json::json!({
            "m13-silent-act2": { "act1": "guardian", "act2": "champ" }
        })
    );
    assert_eq!(RunEpisodeRecord::from_json(&bytes).unwrap(), record);

    for malformed in [
        serde_json::json!({"m13-silent-act2": {"act1": "guardian"}}),
        serde_json::json!({"m13-silent-act2": {"act1": "guardian", "act2": "champ", "act3": "time_eater"}}),
        serde_json::json!({"m13-silent-act2": {"act1": "guardian", "act2": "champ"}, "extra": true}),
        serde_json::json!({"m13-silent-act2": {"act1": "unknown", "act2": "champ"}}),
        serde_json::json!({"m13-silent-act2": {"act1": "guardian", "act2": "unknown"}}),
        serde_json::json!("act1-guardian-unseen"),
    ] {
        value["config"]["boss_history"] = malformed;
        assert!(
            matches!(
                RunEpisodeRecord::from_json(&serde_json::to_vec(&value).unwrap()),
                Err(RunSessionError::IncompatibleReplay(_))
            ),
            "M13 must reject malformed or legacy history JSON: {}",
            value["config"]["boss_history"]
        );
    }
}

#[test]
fn m13_history_object_is_rejected_by_old_profiles_and_old_history_strings_stay_stable() {
    let composite = serde_json::json!({
        "m13-silent-act2": { "act1": "guardian", "act2": "champ" }
    });
    let m12 = old_record(
        Character::Silent,
        RunBossHistory::Act1GuardianUnseen,
        "m12",
    );
    let mut value: serde_json::Value = serde_json::from_slice(&m12.to_json().unwrap()).unwrap();
    value["config"]["boss_history"] = composite.clone();
    assert!(matches!(
        RunEpisodeRecord::from_json(&serde_json::to_vec(&value).unwrap()),
        Err(RunSessionError::IncompatibleReplay(_))
    ));

    let m9 = old_record(
        Character::Ironclad,
        RunBossHistory::AwakenedUnseen,
        "m9",
    );
    let mut value: serde_json::Value = serde_json::from_slice(&m9.to_json().unwrap()).unwrap();
    value["config"]["boss_history"] = composite;
    assert!(matches!(
        RunEpisodeRecord::from_json(&serde_json::to_vec(&value).unwrap()),
        Err(RunSessionError::IncompatibleReplay(_))
    ));

    let cases = [
        (Character::Ironclad, RunBossHistory::AwakenedUnseen, "m9", "awakened-unseen"),
        (Character::Ironclad, RunBossHistory::TimeEaterUnseen, "m10", "time-eater-unseen"),
        (Character::Ironclad, RunBossHistory::DonuDecaUnseen, "m9", "donu-deca-unseen"),
        (Character::Silent, RunBossHistory::Act1GuardianUnseen, "m12", "act1-guardian-unseen"),
        (Character::Silent, RunBossHistory::Act1HexaghostUnseen, "m12", "act1-hexaghost-unseen"),
        (Character::Silent, RunBossHistory::Act1SlimeBossUnseen, "m12", "act1-slime-boss-unseen"),
    ];
    for (character, history, profile, spelling) in cases {
        let record = old_record(character, history, profile);
        let value: serde_json::Value = serde_json::from_slice(&record.to_json().unwrap()).unwrap();
        assert_eq!(value["config"]["boss_history"], spelling);
        assert_eq!(RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap(), record);
    }
}
