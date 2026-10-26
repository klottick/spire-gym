use spire_core::{
    combat::rng::Partition,
    content::Character,
    run::{CreationProfile, Run, RunConfig, RunRng, RoomScenario, RunTerminal},
};
use spire_env::{
    encode_run_m10, m10_run_observation_spec, replay_run,
    run_observation_spec, M9BossHistory, RunCapacities, RunEpisodeRecord,
    RunSession, RunSessionConfig, RunSessionError,
};

#[path = "../../spire-core/examples/m10_support/mod.rs"]
mod support;

type HistoryPair = (M9BossHistory, support::M9BossHistory);

fn histories() -> [HistoryPair; 3] {
    [
        (M9BossHistory::AwakenedUnseen, support::M9BossHistory::AwakenedUnseen),
        (M9BossHistory::TimeEaterUnseen, support::M9BossHistory::TimeEaterUnseen),
        (M9BossHistory::DonuDecaUnseen, support::M9BossHistory::DonuDecaUnseen),
    ]
}

fn config(history: M9BossHistory, recording: bool) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Ironclad,
        ascension: 0,
        partition: Partition::Test,
        boss_history: history,
        capacities: RunCapacities::m10_default(),
        record_replay: recording,
    }
}

fn direct_m10(seed: u64, history: M9BossHistory, partition: Partition, ordinal: u64) -> Run {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    let mut creation = CreationProfile::experienced();
    creation.neow_eligible = false;
    creation.seen_bosses[0] = vec!["Hexaghost", "SlimeBoss"];
    creation.seen_bosses[1] = vec!["Champ", "Collector"];
    creation.seen_bosses[2] = match history {
        M9BossHistory::AwakenedUnseen => vec!["TimeEater", "DonuAndDeca"],
        M9BossHistory::TimeEaterUnseen => vec!["AwakenedOne", "DonuAndDeca"],
        M9BossHistory::DonuDecaUnseen => vec!["AwakenedOne", "TimeEater"],
        M9BossHistory::Act1GuardianUnseen | M9BossHistory::Act1HexaghostUnseen | M9BossHistory::Act1SlimeBossUnseen =>
            unreachable!("M10 direct-run reference only accepts Act 3 history variants"),
    };
    Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 0 },
        creation,
        RunRng::new(root, partition, ordinal),
        RoomScenario::M10IroncladHeart,
    )
    .unwrap()
}

#[test]
fn new_m10_is_opt_in_and_m9_new_descriptor_and_wire_stay_frozen() {
    assert_eq!(RunCapacities::m10_default(), RunCapacities::m9_default());
    let m9_config = RunSessionConfig { capacities: RunCapacities::m9_default(), ..config(M9BossHistory::AwakenedUnseen, true) };
    let m9 = RunSession::new(m9_config.clone()).unwrap();
    assert_eq!(m9.observation_spec().unwrap(), run_observation_spec(&m9_config.capacities).unwrap());
    assert_eq!(m9.observation_spec().unwrap().fingerprint().iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999");
    let mut m9 = RunSession::new(m9_config).unwrap();
    m9.reset(Some(11)).unwrap();
    let m9_record = m9.record().unwrap().unwrap();
    assert_eq!(m9_record.schema_id, "spire-gym-run-replay-v1");
    assert_eq!(m9_record.profile_id, "M9IroncladAct3");
    assert_eq!(m9_record.rules_id, "spire-gym-native-m9-rules-v2");
    assert_eq!(m9_record.content_id, "spire-gym-m9-ironclad-a0-captured-v1");

    let (history, reference_history) = histories()[0];
    let mut m10 = RunSession::new_m10(config(history, true)).unwrap();
    assert_eq!(m10.decision(), Err(RunSessionError::NotReset));
    let m10_spec = m10.observation_spec().unwrap();
    assert_eq!(m10_spec, m10_run_observation_spec(&RunCapacities::m10_default()).unwrap());
    assert_eq!(m10_spec.descriptor()["schema_id"], "spire-gym-run-observation-v2");
    assert_eq!(m10_spec.descriptor()["profile_id"], "M10IroncladHeart");
    for seed in [7, 11] {
        let boundary = m10.reset(Some(seed)).unwrap();
        let core = support::start(seed, reference_history).unwrap();
        assert_eq!(
            boundary.observation,
            encode_run_m10(&core.decision().unwrap(), core.public_map().as_ref(), &RunCapacities::m10_default()).unwrap(),
        );
        let record = m10.record().unwrap().unwrap();
        assert_eq!(record.schema_id, "spire-gym-run-replay-v2");
        assert_eq!(record.profile_id, "M10IroncladHeart");
        assert_eq!(record.rules_id, "spire-gym-native-m10-rules-v1");
        assert_eq!(record.content_id, "spire-gym-m10-ironclad-a0-captured-v1");
        assert_eq!(record.seed_adapter_id, "spire-gym-run-u64-le-v1");
        assert_eq!(record.reward_id, "spire-gym-floor-key-terminal-reward-v1");
        assert_eq!(record.ordinal, 0);
        assert_eq!(record.observation_fingerprint, m10_spec.fingerprint());
    }
}

#[test]
fn m10_seed7_and_seed11_all_histories_match_and_replay_every_boundary() {
    for seed in [7, 11] {
        for (history, reference_history) in histories() {
            let source = support::rollout(seed, reference_history, 8192).unwrap();
            assert!(!matches!(source.completion, support::M10Completion::Unfinished | support::M10Completion::Act4Ready));
            let mut core = support::start(seed, reference_history).unwrap();
            let mut session = RunSession::new_m10(config(history, true)).unwrap();
            let mut boundary = session.reset(Some(seed)).unwrap();
            assert_eq!(boundary.observation, encode_run_m10(&core.decision().unwrap(), core.public_map().as_ref(), &RunCapacities::m10_default()).unwrap());
            for (index, expected) in source.trace.iter().enumerate() {
                let before = core.decision().unwrap();
                let action_index = before.candidates.iter().position(|candidate| candidate.action == expected.action)
                    .unwrap_or_else(|| panic!("source action not offered at {seed}/{history:?}/{index}"));
                let step = session.step(boundary.token, spire_env::RunActionIndex(action_index as i64)).unwrap();
                let transition = core.step(&expected.action).unwrap();
                let after = core.decision().unwrap();
                assert_eq!(step.reward, transition.reward, "reward at {seed}/{history:?}/{index}");
                assert_eq!(step.boundary.terminal, after.terminal, "terminal at {seed}/{history:?}/{index}");
                assert!(!step.truncated, "manual-reset Run has no base truncation");
                assert_eq!(step.terminated, after.terminal.is_some());
                assert_eq!(step.boundary.observation, encode_run_m10(&after, core.public_map().as_ref(), &RunCapacities::m10_default()).unwrap(),
                    "session/core observation at {seed}/{history:?}/{index}");
                assert_eq!(step.boundary.observation, encode_run_m10(&expected.decision, expected.map.as_ref(), &RunCapacities::m10_default()).unwrap(),
                    "source trace boundary at {seed}/{history:?}/{index}");
                boundary = step.boundary;
            }
            assert_eq!(boundary.terminal, source.final_decision.terminal);
            if source.completion == support::M10Completion::Death {
                assert_eq!(boundary.terminal, Some(RunTerminal::Death), "preserve the actual untuned Death");
            }
            let record = session.record().unwrap().unwrap();
            assert_eq!(record.steps.len(), source.trace.len());
            assert_eq!(record.terminal, source.final_decision.terminal);
            let decoded = RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
            assert_eq!(decoded, record);
            let replay = replay_run(&decoded, RunCapacities::m10_default()).unwrap();
            assert_eq!(replay.accepted_count, source.trace.len());
            assert_eq!(replay.final_decision, source.final_decision);
            assert_eq!(replay.final_map, source.final_map);
            assert_eq!(replay.rewards, source.trace.iter().map(|step| step.reward).collect::<Vec<_>>());
            assert_eq!(replay.terminals, source.trace.iter().map(|step| step.decision.terminal).collect::<Vec<_>>());
            assert_eq!(replay.reported_fault, None);
            if boundary.terminal.is_some() {
                assert_eq!(
                    session.step(boundary.token, spire_env::RunActionIndex(0)),
                    Err(RunSessionError::Terminal),
                    "terminal M10 sessions require an explicit manual reset",
                );
                let next = session.reset(None).unwrap();
                let next_record = session.record().unwrap().unwrap();
                assert_eq!((next_record.seed, next_record.ordinal), (seed, 1));
                let core = direct_m10(seed, history, Partition::Test, 1);
                assert_eq!(
                    next.observation,
                    encode_run_m10(
                        &core.decision().unwrap(),
                        core.public_map().as_ref(),
                        &RunCapacities::m10_default()
                    )
                    .unwrap(),
                );
            }
        }
    }
}

#[test]
fn m10_reset_seed_root_partition_and_ordinal_are_core_identical() {
    for partition in [Partition::Test, Partition::Train, Partition::Selection, Partition::Final] {
        let history = M9BossHistory::DonuDecaUnseen;
        let mut cfg = config(history, true);
        cfg.partition = partition;
        let mut session = RunSession::new_m10(cfg).unwrap();
        for (seed, ordinal, explicit) in [(0, 0, None), (0, 1, None), (11, 0, Some(11)), (11, 1, None)] {
            let boundary = session.reset(explicit).unwrap();
            let record = session.record().unwrap().unwrap();
            assert_eq!((record.seed, record.ordinal), (seed, ordinal));
            let core = direct_m10(seed, history, partition, ordinal);
            assert_eq!(boundary.observation, encode_run_m10(&core.decision().unwrap(), core.public_map().as_ref(), &RunCapacities::m10_default()).unwrap());
        }
    }
}

#[test]
fn m10_recording_is_passive_for_the_same_seeded_gameplay_trace() {
    let (history, reference_history) = histories()[0];
    let source = support::rollout(11, reference_history, 8192).unwrap();
    let mut core = support::start(11, reference_history).unwrap();
    let mut recorded = RunSession::new_m10(config(history, true)).unwrap();
    let mut unrecorded = RunSession::new_m10(config(history, false)).unwrap();
    let mut recorded_boundary = recorded.reset(Some(11)).unwrap();
    let mut unrecorded_boundary = unrecorded.reset(Some(11)).unwrap();
    assert_eq!(recorded_boundary.observation, unrecorded_boundary.observation);

    for (index, expected) in source.trace.iter().enumerate() {
        let decision = core.decision().unwrap();
        let action_index = decision
            .candidates
            .iter()
            .position(|candidate| candidate.action == expected.action)
            .unwrap_or_else(|| panic!("source action not offered at {index}"));
        let with_recording = recorded
            .step(recorded_boundary.token, spire_env::RunActionIndex(action_index as i64))
            .unwrap();
        let without_recording = unrecorded
            .step(unrecorded_boundary.token, spire_env::RunActionIndex(action_index as i64))
            .unwrap();
        let transition = core.step(&expected.action).unwrap();
        assert_eq!(with_recording.reward, transition.reward, "recorded reward at {index}");
        assert_eq!(without_recording.reward, transition.reward, "unrecorded reward at {index}");
        assert_eq!(with_recording.terminated, without_recording.terminated);
        assert!(!with_recording.truncated && !without_recording.truncated);
        let decision = core.decision().unwrap();
        let expected_observation = encode_run_m10(
            &decision,
            core.public_map().as_ref(),
            &RunCapacities::m10_default(),
        )
        .unwrap();
        assert_eq!(with_recording.boundary.observation, expected_observation, "recorded boundary at {index}");
        assert_eq!(without_recording.boundary.observation, expected_observation, "unrecorded boundary at {index}");
        recorded_boundary = with_recording.boundary;
        unrecorded_boundary = without_recording.boundary;
    }

    assert_eq!(recorded.record().unwrap().unwrap().steps.len(), source.trace.len());
    assert_eq!(unrecorded.record().unwrap(), None);
    assert_eq!(recorded_boundary.terminal, unrecorded_boundary.terminal);
}

#[test]
fn m10_rejects_invalid_profiles_and_capacities_and_stale_cross_session_actions_atomically() {
    let mut bad = config(M9BossHistory::AwakenedUnseen, false);
    bad.character = Character::Silent;
    assert!(matches!(RunSession::new_m10(bad), Err(RunSessionError::Input(_))));
    let mut bad = config(M9BossHistory::AwakenedUnseen, false);
    bad.ascension = 1;
    assert!(matches!(RunSession::new_m10(bad), Err(RunSessionError::Input(_))));
    let mut bad = config(M9BossHistory::AwakenedUnseen, false);
    bad.capacities.actions = 0;
    assert!(matches!(RunSession::new_m10(bad), Err(RunSessionError::Input(_))));

    let mut session = RunSession::new_m10(config(M9BossHistory::AwakenedUnseen, true)).unwrap();
    let boundary = session.reset(Some(11)).unwrap();
    let mut other = RunSession::new_m10(config(M9BossHistory::AwakenedUnseen, true)).unwrap();
    let foreign = other.reset(Some(11)).unwrap();
    let before = session.record().unwrap();
    assert_eq!(session.step(foreign.token, spire_env::RunActionIndex(0)), Err(RunSessionError::StaleToken));
    assert_eq!(session.step(boundary.token, spire_env::RunActionIndex(-1)), Err(RunSessionError::Input("action index is not legal")));
    assert_eq!(session.record().unwrap(), before);
    assert_eq!(session.decision().unwrap(), boundary);
}

#[test]
fn m10_initial_post_step_capacity_and_external_delivery_fault_keep_one_owner() {
    let mut cfg = config(M9BossHistory::AwakenedUnseen, true);
    cfg.capacities.permanent_cards = 1;
    let mut initial = RunSession::new_m10(cfg).unwrap();
    let error = initial.reset(Some(11)).unwrap_err();
    assert!(matches!(error, RunSessionError::Capacity { group: "permanent_cards", .. }));
    assert!(initial.reproduction().unwrap().record.unwrap().fault.is_some());

    let mut cfg = config(M9BossHistory::AwakenedUnseen, true);
    cfg.capacities.map_nodes = 1;
    let mut post_step = RunSession::new_m10(cfg).unwrap();
    let first = post_step.reset(Some(11)).unwrap();
    let error = post_step.step(first.token, spire_env::RunActionIndex(0)).unwrap_err();
    assert!(matches!(error, RunSessionError::DeliveryFault(_)));
    let record = post_step.record().unwrap().unwrap();
    assert_eq!(record.steps.len(), 1);
    assert!(record.fault.as_ref().is_some_and(|fault| fault.committed && fault.gameplay_recorded
        && fault.error.kind == spire_env::RunRecordErrorKind::Capacity
        && fault.error.capacity.as_ref().is_some_and(|capacity| capacity.group == "map_nodes")));
    let reproduction = post_step.reproduction().unwrap();
    assert!(reproduction.latest_decision.unwrap().is_ok());
    assert_eq!(reproduction.last_accepted_step, record.steps.last().cloned());
    assert_eq!(reproduction.record, Some(record.clone()));

    let mut delivery = RunSession::new_m10(config(M9BossHistory::AwakenedUnseen, true)).unwrap();
    let first = delivery.reset(Some(11)).unwrap();
    delivery.step(first.token, spire_env::RunActionIndex(0)).unwrap();
    delivery.fail_delivery("M10 tuple packaging failed").unwrap();
    let before = delivery.record().unwrap().unwrap();
    let reproduction = delivery.reproduction().unwrap();
    assert_eq!(reproduction.record, Some(before.clone()));
    assert!(matches!(reproduction.fault, Some(RunSessionError::DeliveryFault(_))));
    let decoded = RunEpisodeRecord::from_json(&before.to_json().unwrap()).unwrap();
    assert_eq!(decoded, before);
    let replay = replay_run(&decoded, RunCapacities::m10_default()).unwrap();
    assert_eq!(replay.reported_fault, before.fault, "external export fault is reported, not required to recur");
}
