use spire_core::{
    combat::rng::Partition,
    content::Character,
    run::RunTerminal,
};
use spire_env::{
    replay_run, RunAct1BossTarget, RunAct2BossTarget, RunAct3BossTarget, RunActionIndex,
    RunBossHistory, RunCapacities, RunSession, RunSessionConfig, RunSessionError,
    RUN_CONTENT_ID_M14, RUN_RECORD_SCHEMA_ID_M14, RUN_RULES_ID_M14, RUN_SEED_ADAPTER_ID,
};

fn config(history: RunBossHistory) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: history,
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn history() -> RunBossHistory {
    RunBossHistory::M14SilentAct3 {
        act1: RunAct1BossTarget::Guardian,
        act2: RunAct2BossTarget::Champ,
        act3: RunAct3BossTarget::AwakenedOne,
    }
}

#[test]
fn m14_profile_has_exact_native_identity_and_no_ironclad_sidecar() {
    let mut session = RunSession::new_m14(config(history())).unwrap();
    let boundary = session.reset(Some(7)).unwrap();
    let spec = session.observation_spec().unwrap();
    assert_eq!(spec.descriptor()["schema_id"], "spire-gym-m14-silent-act3-observation-v1");
    assert_eq!(spec.descriptor()["profile_id"], "M14SilentAct3");

    let environment = session.environment_spec().unwrap();
    assert_eq!(environment["profile_id"], "M14SilentAct3");
    assert_eq!(environment["schema_id"], "spire-gym-m14-silent-act3-observation-v1");
    assert_eq!(environment["rules_id"], RUN_RULES_ID_M14);
    assert_eq!(environment["content_id"], RUN_CONTENT_ID_M14);
    assert_eq!(environment["seed_adapter_id"], RUN_SEED_ADAPTER_ID);
    assert!(environment["combat_view_id"].is_null());
    assert_eq!(environment["finite_content"]["permanent_cards"].as_array().unwrap().len(), 20);
    assert_eq!(environment["finite_content"]["generated_and_status_cards"].as_array().unwrap().len(), 6);
    assert_eq!(environment["finite_content"]["act3_enemy_species"].as_array().unwrap().len(), 20);
    assert_eq!(environment["finite_content"]["playable_enemies"].as_array().unwrap().len(), 54);
    assert_eq!(environment["finite_content"]["act3_encounters"].as_array().unwrap().len(), 16);

    let expected = RunSessionError::UnsupportedBoundary(
        "M14 does not provide the Ironclad combat policy sidecar",
    );
    assert_eq!(session.combat_observation_spec(), Err(expected.clone()));
    assert_eq!(session.combat_view(), Err(expected));
    assert_eq!(session.decision().unwrap(), boundary, "sidecar probes leave the healthy session unchanged");
    assert!(session.reproduction().unwrap().fault.is_none());

    let mut wrong_character = config(history());
    wrong_character.character = Character::Ironclad;
    assert!(matches!(RunSession::new_m14(wrong_character), Err(RunSessionError::Input("M14 requires Silent A0"))));
    let mut wrong_ascension = config(history());
    wrong_ascension.ascension = 1;
    assert!(matches!(RunSession::new_m14(wrong_ascension), Err(RunSessionError::Input("M14 requires Silent A0"))));
    let mut wrong_history = config(RunBossHistory::Act1GuardianUnseen);
    assert!(matches!(RunSession::new_m14(wrong_history.clone()), Err(RunSessionError::Input("boss history does not match Run profile"))));
    assert!(matches!(RunSession::new_m13(wrong_history.clone()), Err(RunSessionError::Input("boss history does not match Run profile"))));
    wrong_history.boss_history = RunBossHistory::M13SilentAct2 {
        act1: RunAct1BossTarget::Guardian,
        act2: RunAct2BossTarget::Champ,
    };
    assert!(matches!(RunSession::new_m14(wrong_history), Err(RunSessionError::Input("boss history does not match Run profile"))));
}

#[test]
fn m14_tokens_first_step_semantic_replay_and_capacity_fault_custody() {
    let mut first = RunSession::new_m14(config(history())).unwrap();
    let mut second = RunSession::new_m14(config(history())).unwrap();
    let before = first.reset(Some(11)).unwrap();
    let other = second.reset(Some(11)).unwrap();
    assert_ne!(before.token, other.token, "session identity separates equal-seed decisions");

    let before_record = first.record().unwrap().unwrap();
    assert_eq!(first.preflight_step(other.token, RunActionIndex(0)), Err(RunSessionError::StaleToken));
    assert!(matches!(first.preflight_step(before.token, RunActionIndex(-1)), Err(RunSessionError::Input(_))));
    assert_eq!(first.record().unwrap().unwrap(), before_record);
    assert_eq!(first.decision().unwrap(), before);

    let step = first.step(before.token, RunActionIndex(0)).unwrap();
    assert!(!step.terminated && !step.truncated);
    assert_ne!(step.boundary.token, before.token);
    assert_eq!(first.preflight_step(before.token, RunActionIndex(0)), Err(RunSessionError::StaleToken));
    let record = first.record().unwrap().unwrap();
    assert_eq!(record.steps.len(), 1);
    assert_eq!(record.schema_id, RUN_RECORD_SCHEMA_ID_M14);
    assert_eq!(record.profile_id, "M14SilentAct3");
    assert_eq!(record.rules_id, RUN_RULES_ID_M14);
    assert_eq!(record.content_id, RUN_CONTENT_ID_M14);
    assert_eq!(record.terminal, None);
    let wire = record.to_json().unwrap();
    let restored = spire_env::RunEpisodeRecord::from_json(&wire).unwrap();
    assert_eq!(restored, record);
    let replay = replay_run(&restored, RunCapacities::m10_default()).unwrap();
    assert_eq!(replay.accepted_count, 1);
    assert_eq!(replay.final_decision, first.reproduction().unwrap().latest_decision.unwrap().unwrap());
    assert_eq!(replay.final_decision.terminal, None::<RunTerminal>);

    let mut too_small = config(history());
    too_small.capacities.offers = 1;
    let mut faulted = RunSession::new_m14(too_small).unwrap();
    assert!(matches!(
        faulted.reset(Some(11)),
        Err(RunSessionError::Capacity { group: "offers", required, capacity: 1 }) if required > 1
    ));
    let reproduction = faulted.reproduction().unwrap();
    assert!(matches!(reproduction.fault, Some(RunSessionError::Capacity { group: "offers", .. })));
    let fault_record = reproduction.record.unwrap();
    let fault = fault_record.fault.as_ref().unwrap();
    assert_eq!(fault.action, None);
    assert!(!fault.committed && !fault.gameplay_recorded);
    assert_eq!(fault.error.kind, spire_env::RunRecordErrorKind::Capacity);
    assert_eq!(fault.error.capacity.as_ref().unwrap().group, "offers");
    let fault_wire = fault_record.to_json().unwrap();
    assert_eq!(spire_env::RunEpisodeRecord::from_json(&fault_wire).unwrap(), fault_record);
}

#[test]
fn m14_session_accepts_all_twenty_seven_typed_histories() {
    for act1 in [
        RunAct1BossTarget::Guardian,
        RunAct1BossTarget::Hexaghost,
        RunAct1BossTarget::SlimeBoss,
    ] {
        for act2 in [
            RunAct2BossTarget::Champ,
            RunAct2BossTarget::BronzeAutomaton,
            RunAct2BossTarget::Collector,
        ] {
            for act3 in [
                RunAct3BossTarget::AwakenedOne,
                RunAct3BossTarget::TimeEater,
                RunAct3BossTarget::DonuAndDeca,
            ] {
                assert!(RunSession::new_m14(config(RunBossHistory::M14SilentAct3 {
                    act1,
                    act2,
                    act3,
                })).is_ok(), "typed history {act1:?}/{act2:?}/{act3:?}");
            }
        }
    }
}

#[test]
fn m14_profile_rejects_m10_and_legacy_profile_publication_boundaries() {
    let mut m10 = config(history());
    m10.boss_history = RunBossHistory::AwakenedUnseen;
    assert!(matches!(RunSession::new_m14(m10), Err(RunSessionError::Input(_))));

    let mut m13 = config(history());
    m13.boss_history = RunBossHistory::M13SilentAct2 {
        act1: RunAct1BossTarget::SlimeBoss,
        act2: RunAct2BossTarget::Collector,
    };
    assert!(matches!(RunSession::new_m14(m13.clone()), Err(RunSessionError::Input(_))));
    assert!(matches!(RunSession::new_m13(config(history())), Err(RunSessionError::Input(_))));

    let mut current = RunSession::new_m14(config(history())).unwrap();
    let boundary = current.reset(Some(7)).unwrap();
    assert!(boundary.terminal.is_none());
    assert!(boundary.observation.action_mask.iter().any(|value| *value == 1));
    assert_eq!(current.decision().unwrap().observation.group("context").unwrap().values[4], 1, "new_m14 begins at the real Neow room row");
}
