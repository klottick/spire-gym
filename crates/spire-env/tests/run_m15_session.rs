use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    RunAct1BossTarget, RunAct2BossTarget, RunAct3BossTarget, RunBossHistory,
    RunCapacities, RunSession, RunSessionConfig, RunSessionError,
};

fn history(
    act1: RunAct1BossTarget,
    act2: RunAct2BossTarget,
    act3: RunAct3BossTarget,
) -> RunBossHistory {
    RunBossHistory::M15SilentHeart { act1, act2, act3 }
}

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

fn default_config() -> RunSessionConfig {
    config(history(
        RunAct1BossTarget::Guardian,
        RunAct2BossTarget::Champ,
        RunAct3BossTarget::AwakenedOne,
    ))
}

#[test]
fn m15_profile_uses_exact_identity_and_rejects_the_ironclad_policy_sidecar() {
    let mut session = RunSession::new_m15(default_config()).unwrap();
    let observation = session.observation_spec().unwrap();
    assert_eq!(
        observation.descriptor()["schema_id"],
        "spire-gym-m15-silent-heart-observation-v1"
    );
    assert_eq!(observation.descriptor()["profile_id"], "M15SilentHeart");

    let environment = session.environment_spec().unwrap();
    assert_eq!(environment["profile_id"], "M15SilentHeart");
    assert_eq!(environment["schema_id"], "spire-gym-m15-silent-heart-observation-v1");
    assert_eq!(environment["rules_id"], "spire-gym-native-m15-rules-v1");
    assert_eq!(environment["content_id"], "spire-gym-m15-silent-a0-heart-captured-v1");
    assert!(environment["combat_view_id"].is_null());
    let finite=&environment["finite_content"];
    assert_eq!(finite["permanent_cards"].as_array().unwrap().len(),76);
    assert_eq!(finite["generated_and_status_cards"].as_array().unwrap().len(),40);
    assert_eq!(finite["relics"].as_array().unwrap().len(),28);
    assert_eq!(finite["potions"].as_array().unwrap().len(),33);
    assert_eq!(finite["playable_enemies"].as_array().unwrap().len(),57);
    assert_eq!(finite["playable_encounters"].as_array().unwrap().len(),42);


    let expected = RunSessionError::UnsupportedBoundary(
        "M15 does not provide the Ironclad combat policy sidecar",
    );
    assert_eq!(session.combat_observation_spec(), Err(expected.clone()));
    assert_eq!(session.combat_view(), Err(expected));
    assert!(session.reproduction().unwrap().fault.is_none());
}

#[test]
fn m15_session_accepts_all_twenty_seven_typed_histories() {
    use spire_core::content::EncounterId;

    let act1 = [
        RunAct1BossTarget::Guardian,
        RunAct1BossTarget::Hexaghost,
        RunAct1BossTarget::SlimeBoss,
    ];
    let act2 = [
        RunAct2BossTarget::Champ,
        RunAct2BossTarget::BronzeAutomaton,
        RunAct2BossTarget::Collector,
    ];
    let act3 = [
        RunAct3BossTarget::AwakenedOne,
        RunAct3BossTarget::TimeEater,
        RunAct3BossTarget::DonuAndDeca,
    ];
    let mut accepted = 0;
    for first in act1 {
        for second in act2 {
            for third in act3 {
                let mut session = RunSession::new_m15(config(history(first, second, third))).unwrap();
                let initial = session.reset(Some(7)).unwrap();
                let zero_step = session.record().unwrap().unwrap();
                let decoded = spire_env::RunEpisodeRecord::from_json(&zero_step.to_json().unwrap()).unwrap();
                assert_eq!(decoded, zero_step);
                let replay = spire_env::replay_run(&decoded, default_config().capacities).unwrap();
                assert_eq!(replay.accepted_count, 0);
                assert_eq!(replay.final_decision, session.reproduction().unwrap().latest_decision.unwrap().unwrap());

                let next = session.step(initial.token, spire_env::RunActionIndex(0)).unwrap();
                assert!(!next.terminated);
                let expected_boss = match first {
                    RunAct1BossTarget::Guardian => EncounterId::Guardian,
                    RunAct1BossTarget::Hexaghost => EncounterId::Hexaghost,
                    RunAct1BossTarget::SlimeBoss => EncounterId::SlimeBoss,
                };
                assert_eq!(session.reproduction().unwrap().latest_map.unwrap().boss, Some(expected_boss));
                accepted += 1;
            }
        }
    }
    assert_eq!(accepted, 27);
}
#[test]
fn m15_session_rejects_foreign_character_ascension_and_history() {
    let mut wrong_character = default_config();
    wrong_character.character = Character::Ironclad;
    assert!(matches!(
        RunSession::new_m15(wrong_character),
        Err(RunSessionError::Input("M15 requires Silent A0"))
    ));

    let mut wrong_ascension = default_config();
    wrong_ascension.ascension = 1;
    assert!(matches!(
        RunSession::new_m15(wrong_ascension),
        Err(RunSessionError::Input("M15 requires Silent A0"))
    ));

    let m14_history = RunBossHistory::M14SilentAct3 {
        act1: RunAct1BossTarget::Guardian,
        act2: RunAct2BossTarget::Champ,
        act3: RunAct3BossTarget::AwakenedOne,
    };
    assert!(matches!(
        RunSession::new_m15(config(m14_history)),
        Err(RunSessionError::Input("boss history does not match Run profile"))
    ));
}

#[test]
fn m15_short_prefix_records_and_reproduces_the_shared_session_lifecycle() {
    let mut session = RunSession::new_m15(default_config()).unwrap();
    let first = session.reset(Some(7)).unwrap();
    let mut peer = RunSession::new_m15(default_config()).unwrap();
    let peer_boundary = peer.reset(Some(7)).unwrap();
    let before_boundary = session.decision().unwrap();
    assert_eq!(
        session.preflight_step(peer_boundary.token, spire_env::RunActionIndex(0)),
        Err(RunSessionError::StaleToken)
    );
    assert_eq!(session.decision().unwrap(), before_boundary);
    let mut detached = before_boundary.clone();
    detached.observation.action_mask.fill(0);
    if before_boundary.observation.action_mask.iter().any(|value| *value != 0) {
        assert_ne!(detached.observation, session.decision().unwrap().observation);
    }
    let before = session.record().unwrap().unwrap();
    assert_eq!(before.schema_id, "spire-gym-m15-silent-heart-replay-v1");
    assert_eq!(before.profile_id, "M15SilentHeart");
    assert_eq!(before.rules_id, "spire-gym-native-m15-rules-v1");
    assert_eq!(before.content_id, "spire-gym-m15-silent-a0-heart-captured-v1");
    assert!(session
        .preflight_step(first.token, spire_env::RunActionIndex(-1))
        .is_err());
    assert_eq!(session.record().unwrap().unwrap(), before);

    let accepted = session
        .step(first.token, spire_env::RunActionIndex(0))
        .unwrap();
    assert!(!accepted.terminated);
    assert!(!accepted.truncated);
    assert_ne!(accepted.boundary.token, first.token);

    let record = session.record().unwrap().unwrap();
    assert_eq!(record.steps.len(), 1);
    let decoded = spire_env::RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
    assert_eq!(decoded, record);
    let replay = spire_env::replay_run(&decoded, default_config().capacities).unwrap();
    let reproduction = session.reproduction().unwrap();
    assert_eq!(replay.accepted_count, 1);
    assert_eq!(replay.final_decision, reproduction.latest_decision.unwrap().unwrap());
    assert_eq!(replay.final_map, reproduction.latest_map);

    let reset = session.reset(None).unwrap();
    assert_eq!(session.effective_seed(), Some(7));
    let next_record = session.record().unwrap().unwrap();
    assert_eq!(next_record.ordinal, 1);
    assert!(reset.terminal.is_none());
}

#[test]
fn m15_recording_can_be_disabled_without_losing_reproduction_prefix() {
    let mut no_record = default_config();
    no_record.record_replay = false;
    let mut session = RunSession::new_m15(no_record).unwrap();
    let initial = session.reset(Some(11)).unwrap();
    session
        .step(initial.token, spire_env::RunActionIndex(0))
        .unwrap();

    assert_eq!(session.record().unwrap(), None);
    let reproduction = session.reproduction().unwrap();
    assert!(reproduction.fault.is_none());
    assert!(reproduction.latest_decision.unwrap().is_ok());
    assert!(reproduction.last_accepted_step.is_some());
}
