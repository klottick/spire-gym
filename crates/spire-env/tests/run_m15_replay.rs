use spire_core::{
    combat::{rng::Partition, CombatAction},
    content::Character,
    run::{GameAction, RunTerminal},
};
use spire_env::{
    RunAct1BossTarget, RunAct2BossTarget, RunAct3BossTarget, RunBossHistory,
    RunCapacities, RunEpisodeRecord, RunSession, RunSessionConfig,
};
use serde_json::Value;

fn m15_config() -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: RunBossHistory::M15SilentHeart {
            act1: RunAct1BossTarget::Guardian,
            act2: RunAct2BossTarget::Champ,
            act3: RunAct3BossTarget::AwakenedOne,
        },
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn short_m15_record() -> RunEpisodeRecord {
    let mut session = RunSession::new_m15(m15_config()).unwrap();
    let initial = session.reset(Some(7)).unwrap();
    session
        .step(initial.token, spire_env::RunActionIndex(0))
        .unwrap();
    session.record().unwrap().unwrap()
}

fn m14_record() -> RunEpisodeRecord {
    let mut config = m15_config();
    config.boss_history = RunBossHistory::M14SilentAct3 {
        act1: RunAct1BossTarget::Guardian,
        act2: RunAct2BossTarget::Champ,
        act3: RunAct3BossTarget::AwakenedOne,
    };
    let mut session = RunSession::new_m14(config).unwrap();
    let initial = session.reset(Some(7)).unwrap();
    session
        .step(initial.token, spire_env::RunActionIndex(0))
        .unwrap();
    session.record().unwrap().unwrap()
}

fn from_value(value: Value) -> Result<RunEpisodeRecord, spire_env::RunSessionError> {
    RunEpisodeRecord::from_json(&serde_json::to_vec(&value).unwrap())
}

#[test]
fn m15_portable_replay_round_trips_the_typed_history_and_short_prefix() {
    let record = short_m15_record();
    let wire = record.to_json().unwrap();
    let decoded = RunEpisodeRecord::from_json(&wire).unwrap();
    assert_eq!(decoded, record);
    assert_eq!(decoded.terminal, None::<RunTerminal>);

    let value: Value = serde_json::from_slice(&wire).unwrap();
    assert_eq!(value["schema_id"], "spire-gym-m15-silent-heart-replay-v1");
    assert_eq!(value["profile_id"], "M15SilentHeart");
    assert_eq!(value["rules_id"], "spire-gym-native-m15-rules-v1");
    assert_eq!(value["content_id"], "spire-gym-m15-silent-a0-heart-captured-v1");
    assert_eq!(
        value["config"]["boss_history"],
        serde_json::json!({"m15-silent-heart": {
            "act1":"guardian", "act2":"champ", "act3":"awakened-one"
        }})
    );
}

#[test]
fn m15_json_preflight_rejects_malformed_unknown_partial_cross_profile_and_duplicate_fields() {
    let wire = String::from_utf8(short_m15_record().to_json().unwrap()).unwrap();
    assert!(RunEpisodeRecord::from_json(b"{").is_err());

    let unknown = wire.replacen("\"closed\":false", "\"closed\":false,\"unexpected\":1", 1);
    assert!(RunEpisodeRecord::from_json(unknown.as_bytes()).is_err());

    let partial_history = wire.replacen(
        ",\"act3\":\"awakened-one\"",
        "",
        1,
    );
    assert_ne!(partial_history, wire);
    assert!(RunEpisodeRecord::from_json(partial_history.as_bytes()).is_err());

    let duplicate_top = wire.replacen(
        "\"closed\":false",
        "\"closed\":false,\"closed\":true",
        1,
    );
    assert!(RunEpisodeRecord::from_json(duplicate_top.as_bytes()).is_err());

    let duplicate_history = wire.replacen(
        "\"act1\":\"guardian\"",
        "\"act1\":\"guardian\",\"act1\":\"slime_boss\"",
        1,
    );
    assert!(RunEpisodeRecord::from_json(duplicate_history.as_bytes()).is_err());

    let mut cross_profile: Value = serde_json::from_str(&wire).unwrap();
    cross_profile["profile_id"] = Value::String("M14SilentAct3".into());
    assert!(from_value(cross_profile).is_err());

    let mut wrong_history: Value = serde_json::from_str(&wire).unwrap();
    wrong_history["config"]["boss_history"] = serde_json::json!({
        "m14-silent-act3": {"act1":"guardian", "act2":"champ", "act3":"awakened-one"}
    });
    assert!(from_value(wrong_history).is_err());
}

#[test]
fn m15_generated_offer_actions_round_trip_as_public_indices_only() {
    // These altered steps probe the portable codec only. They are not evidence
    // that an offer is reachable in a fresh M15 run; that trace lane is held.
    let mut choose = short_m15_record();
    choose.steps[0].action = GameAction::Combat(CombatAction::ChooseCardOffer { index: 2 });
    let wire = choose.to_json().unwrap();
    let value: Value = serde_json::from_slice(&wire).unwrap();
    assert_eq!(value["steps"][0]["action"], serde_json::json!({
        "verb":"combat.choose_card_offer", "index":2
    }));
    assert_eq!(RunEpisodeRecord::from_json(&wire).unwrap(), choose);

    let mut skip = short_m15_record();
    skip.steps[0].action = GameAction::Combat(CombatAction::SkipCardOffer);
    let wire = skip.to_json().unwrap();
    let value: Value = serde_json::from_slice(&wire).unwrap();
    assert_eq!(value["steps"][0]["action"], serde_json::json!({
        "verb":"combat.skip_card_offer"
    }));
    assert_eq!(RunEpisodeRecord::from_json(&wire).unwrap(), skip);

    let mut unknown_action_field: Value = serde_json::from_slice(&choose.to_json().unwrap()).unwrap();
    unknown_action_field["steps"][0]["action"]["offer_id"] = Value::String("private-template".into());
    assert!(from_value(unknown_action_field).is_err());
}

#[test]
fn m15_semantic_replay_decodes_offer_index_then_requires_current_public_candidate() {
    let mut record = short_m15_record();
    record.steps[0].action = GameAction::Combat(CombatAction::ChooseCardOffer { index: 1 });
    let decoded = RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
    let error = spire_env::replay_run(&decoded, m15_config().capacities).unwrap_err();
    assert!(matches!(
        error,
        spire_env::RunSessionError::IncompatibleReplay("Run action not currently offered")
    ));
}

#[test]
fn old_profiles_keep_their_export_shape_and_reject_m15_offer_actions() {
    let mut record = m14_record();
    let old_wire = record.to_json().unwrap();
    let old_value: Value = serde_json::from_slice(&old_wire).unwrap();
    assert_eq!(old_value["profile_id"], "M14SilentAct3");
    let mut forward_compatible: Value = old_value.clone();
    forward_compatible["older_reader_unknown"] = Value::Bool(true);
    assert_eq!(from_value(forward_compatible).unwrap(), record);

    record.steps[0].action = GameAction::Combat(CombatAction::ChooseCardOffer { index: 0 });
    assert!(record.to_json().is_err());

    let mut offer_action: Value = old_value;
    offer_action["steps"][0]["action"] = serde_json::json!({
        "verb":"combat.choose_card_offer", "index":0
    });
    assert!(from_value(offer_action).is_err());
}
