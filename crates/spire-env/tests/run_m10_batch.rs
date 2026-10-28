use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    encode_run_m10, M9BossHistory, RunActionIndex, RunBatch, RunBatchError,
    RunCapacities, RunRecordErrorKind, RunSession, RunSessionConfig,
};

#[path = "../../spire-core/examples/m10_support/mod.rs"]
mod support;

fn config(history: M9BossHistory) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Ironclad,
        ascension: 0,
        partition: Partition::Test,
        boss_history: history,
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn source_history(history: M9BossHistory) -> support::M9BossHistory {
    match history {
        M9BossHistory::AwakenedUnseen => support::M9BossHistory::AwakenedUnseen,
        M9BossHistory::TimeEaterUnseen => support::M9BossHistory::TimeEaterUnseen,
        M9BossHistory::DonuDecaUnseen => support::M9BossHistory::DonuDecaUnseen,
        M9BossHistory::Act1GuardianUnseen | M9BossHistory::Act1HexaghostUnseen | M9BossHistory::Act1SlimeBossUnseen | M9BossHistory::M13SilentAct2 { .. } | M9BossHistory::M14SilentAct3 { .. } | M9BossHistory::M15SilentHeart { .. } | M9BossHistory::M17DefectAct1 { .. } | M9BossHistory::M18DefectAct2 { .. } =>
            unreachable!("M10 batch reference only accepts Act 3 history variants"),
    }
}

#[test]
fn batch_reset_returns_real_m10_boundaries_in_stable_slot_order() {
    let configs = vec![
        config(M9BossHistory::AwakenedUnseen),
        config(M9BossHistory::DonuDecaUnseen),
    ];
    let seeds = [Some(7), Some(11)];
    let independent = configs
        .iter()
        .cloned()
        .zip(seeds)
        .map(|(config, seed)| {
            let mut session = RunSession::new_m10(config).unwrap();
            let boundary = session.reset(seed).unwrap();
            (session, boundary)
        })
        .collect::<Vec<_>>();

    let mut batch = RunBatch::new_m10(configs, 2).unwrap();
    let boundaries = batch.reset(&seeds).unwrap();
    assert_eq!(boundaries.len(), 2);
    assert_eq!(boundaries[0].observation, independent[0].1.observation);
    assert_eq!(boundaries[1].observation, independent[1].1.observation);
    assert_eq!(boundaries[0].terminal, None);
    assert_eq!(boundaries[1].terminal, None);
    assert_ne!(boundaries[0].token, boundaries[1].token);

    for slot in 0..2 {
        let batch_record = batch.record(slot).unwrap().unwrap();
        let independent_record = independent[slot].0.record().unwrap().unwrap();
        assert_eq!(batch_record, independent_record);
        assert_eq!(batch.reproduction(slot).unwrap().seed, seeds[slot]);
    }
}

#[test]
fn batch_sizes_workers_histories_and_seeds_match_independent_sessions_through_endings() {
    let all_histories = [
        M9BossHistory::AwakenedUnseen,
        M9BossHistory::TimeEaterUnseen,
        M9BossHistory::DonuDecaUnseen,
    ];
    for size in [1, 3] {
        let histories = if size == 1 { &all_histories[..1] } else { &all_histories[..] };
        for workers in [1, 2] {
            for seed in [7, 11] {
                let configs = histories.iter().copied().map(config).collect::<Vec<_>>();
                let reports = histories.iter().copied().map(|history| {
                    support::rollout(seed, source_history(history), 8192).unwrap()
                }).collect::<Vec<_>>();
                let trace_len = reports[0].trace.len();
                assert!(reports.iter().all(|report| report.trace.len() == trace_len));

                let mut batch = RunBatch::new_m10(configs.clone(), workers).unwrap();
                let seeds = vec![Some(seed); size];
                let mut boundaries = batch.reset(&seeds).unwrap();
                let mut independent = configs.iter().cloned().map(|config| {
                    let mut session = RunSession::new_m10(config).unwrap();
                    let boundary = session.reset(Some(seed)).unwrap();
                    (session, boundary)
                }).collect::<Vec<_>>();
                let mut cores = histories.iter().copied().map(|history| {
                    support::start(seed, source_history(history)).unwrap()
                }).collect::<Vec<_>>();
                for slot in 0..size {
                    assert_eq!(boundaries[slot].observation, independent[slot].1.observation);
                }
                for left in 0..size {
                    for right in left + 1..size {
                        assert_ne!(boundaries[left].token, boundaries[right].token);
                    }
                }

                for step_index in 0..trace_len {
                    let indices = cores.iter().enumerate().map(|(slot, core)| {
                        let decision = core.decision().unwrap();
                        let expected = &reports[slot].trace[step_index];
                        decision.candidates.iter().position(|candidate| candidate.action == expected.action)
                            .map(|index| RunActionIndex(index as i64))
                            .unwrap_or_else(|| panic!("seed={seed}, slot={slot}, step={step_index}: expected action not offered"))
                    }).collect::<Vec<_>>();
                    let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
                    let batch_steps = batch.step(&tokens, &indices).unwrap();
                    assert_eq!(batch_steps.len(), size, "batch must deliver one step result per stable slot");
                    for slot in 0..size {
                        let expected = &reports[slot].trace[step_index];
                        let individual_token = independent[slot].1.token;
                        let individual = independent[slot].0.step(individual_token, indices[slot]).unwrap();
                        let core_transition = cores[slot].step(&expected.action).unwrap();
                        assert_eq!(batch_steps[slot].reward, expected.reward, "batch reward at {seed}/{slot}/{step_index}");
                        assert_eq!(batch_steps[slot].reward, individual.reward);
                        assert_eq!(batch_steps[slot].reward, core_transition.reward);
                        assert_eq!(batch_steps[slot].boundary.terminal, expected.decision.terminal);
                        assert_eq!(batch_steps[slot].terminated, expected.decision.terminal.is_some());
                        assert!(!batch_steps[slot].truncated);
                        assert_eq!(batch_steps[slot].boundary.observation, individual.boundary.observation);
                        assert_eq!(batch_steps[slot].boundary.observation.action_mask, individual.boundary.observation.action_mask);
                        assert_eq!(batch_steps[slot].boundary.observation,
                            encode_run_m10(&expected.decision, expected.map.as_ref(), &RunCapacities::m10_default()).unwrap());
                        boundaries[slot] = batch_steps[slot].boundary.clone();
                        independent[slot].1 = individual.boundary;
                    }
                }

                for slot in 0..size {
                    let batch_record = batch.record(slot).unwrap().unwrap();
                    let individual_record = independent[slot].0.record().unwrap().unwrap();
                    assert_eq!(batch_record, individual_record);
                    assert_eq!(batch_record.steps.len(), reports[slot].trace.len());
                    assert_eq!(batch_record.terminal, reports[slot].final_decision.terminal);
                    for (recorded, expected) in batch_record.steps.iter().zip(&reports[slot].trace) {
                        assert_eq!((&recorded.action, recorded.reward, recorded.terminal),
                            (&expected.action, expected.reward, expected.decision.terminal));
                    }
                    if reports[slot].completion == support::M10Completion::Death {
                        assert_eq!(batch_record.terminal, Some(spire_core::run::RunTerminal::Death));
                    }
                }

                // Every selected public-only trace ends at a real terminal.
                // Replaying its final token must reject before changing any
                // accepted terminal record or faulting the batch.
                assert!(reports.iter().all(|report| report.final_decision.terminal.is_some()));
                let terminal_tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
                let terminal_records = (0..size).map(|slot| batch.record(slot).unwrap().unwrap()).collect::<Vec<_>>();
                let terminal_indices = vec![RunActionIndex(0); size];
                assert_eq!(batch.step(&terminal_tokens, &terminal_indices), Err(RunBatchError::Input {
                    slot: Some(0), error: spire_env::RunSessionError::Terminal,
                }));
                for slot in 0..size {
                    assert_eq!(batch.record(slot).unwrap().unwrap(), terminal_records[slot]);
                }

                let continued = batch.reset(&vec![None; size]).unwrap();
                let mut continued_boundaries = continued;
                for slot in 0..size {
                    let individual = independent[slot].0.reset(None).unwrap();
                    assert_eq!(continued_boundaries[slot].observation, individual.observation);
                    let record = batch.record(slot).unwrap().unwrap();
                    assert_eq!((record.seed, record.ordinal), (seed, 1));
                    assert_eq!(batch.reproduction(slot).unwrap().record, Some(record));
                    independent[slot].1 = individual;
                }

                // Continue the ordinal-1 episodes with real actions chosen
                // from each owner's current ordered menu. Compare every
                // returned transition and retained semantic action prefix.
                let mut continued_steps = 0;
                for _ in 0..16 {
                    if continued_boundaries.iter().any(|boundary| boundary.terminal.is_some()) {
                        break;
                    }
                    let indices = vec![RunActionIndex(0); size];
                    let tokens = continued_boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
                    let batch_results = batch.step(&tokens, &indices).unwrap();
                    for slot in 0..size {
                        let individual_token = independent[slot].1.token;
                        let individual = independent[slot].0.step(individual_token, indices[slot]).unwrap();
                        assert_eq!(batch_results[slot].reward, individual.reward);
                        assert_eq!(batch_results[slot].terminated, individual.terminated);
                        assert_eq!(batch_results[slot].truncated, individual.truncated);
                        assert_eq!(batch_results[slot].boundary.observation, individual.boundary.observation);
                        assert_eq!(batch_results[slot].boundary.observation.action_mask,
                            individual.boundary.observation.action_mask);
                        let batch_record = batch.record(slot).unwrap().unwrap();
                        let individual_record = independent[slot].0.record().unwrap().unwrap();
                        assert_eq!(batch_record, individual_record);
                        assert_eq!(batch_record.ordinal, 1);
                        assert!(batch_record.steps.last().is_some());
                        continued_boundaries[slot] = batch_results[slot].boundary.clone();
                        independent[slot].1 = individual.boundary;
                    }
                    continued_steps += 1;
                }
                assert!(continued_steps > 0, "continued episode must accept a real step");
            }
        }
    }
}

#[test]
fn batch_preflight_rejects_bad_shapes_indices_and_foreign_tokens_without_mutation() {
    let configs = vec![
        config(M9BossHistory::AwakenedUnseen),
        config(M9BossHistory::TimeEaterUnseen),
        config(M9BossHistory::DonuDecaUnseen),
    ];
    let seeds = [Some(7), Some(11), Some(0)];
    let mut batch = RunBatch::new_m10(configs, 2).unwrap();
    let boundaries = batch.reset(&seeds).unwrap();
    let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
    let valid_indices = vec![RunActionIndex(0); 3];
    let before = (0..3).map(|slot| batch.reproduction(slot).unwrap()).collect::<Vec<_>>();

    assert!(matches!(batch.reset(&seeds[..2]), Err(RunBatchError::Input { slot: None, .. })));
    assert!(matches!(batch.step(&tokens[..2], &valid_indices), Err(RunBatchError::Input { slot: None, .. })));
    let invalid_indices = [RunActionIndex(0), RunActionIndex(0), RunActionIndex(-1)];
    assert_eq!(batch.step(&tokens, &invalid_indices), Err(RunBatchError::Input {
        slot: Some(2), error: spire_env::RunSessionError::Input("action index is not legal"),
    }));

    let mut foreign = RunSession::new_m10(config(M9BossHistory::TimeEaterUnseen)).unwrap();
    let foreign_token = foreign.reset(Some(11)).unwrap().token;
    let mut stale_tokens = tokens.clone();
    stale_tokens[1] = foreign_token;
    assert_eq!(batch.step(&stale_tokens, &valid_indices), Err(RunBatchError::Input {
        slot: Some(1), error: spire_env::RunSessionError::StaleToken,
    }));
    for slot in 0..3 {
        assert_eq!(batch.reproduction(slot).unwrap(), before[slot], "preflight must not change slot {slot}");
    }

    let advanced = batch.step(&tokens, &valid_indices).unwrap();
    assert_eq!(advanced.len(), 3, "a rejected preflight does not fault the batch");
    let older_records = (0..3).map(|slot| batch.record(slot).unwrap().unwrap()).collect::<Vec<_>>();
    let advanced_tokens = advanced.iter().map(|step| step.boundary.token).collect::<Vec<_>>();
    assert!(matches!(batch.step(&advanced_tokens[..2], &valid_indices), Err(RunBatchError::Input { slot: None, .. })));
    let bad_after_accepted = [RunActionIndex(0), RunActionIndex(-1), RunActionIndex(0)];
    assert!(matches!(batch.step(&advanced_tokens, &bad_after_accepted), Err(RunBatchError::Input { slot: Some(1), .. })));
    for slot in 0..3 {
        assert_eq!(batch.record(slot).unwrap().unwrap(), older_records[slot], "rejected input must not rewrite older slot {slot} records");
    }
    batch.close().unwrap();
    batch.close().unwrap();
    assert!(batch.reproduction(0).unwrap().closed);
    assert_eq!(batch.reset(&seeds), Err(RunBatchError::Input {
        slot: Some(0), error: spire_env::RunSessionError::Closed,
    }));
}

#[test]
fn execution_fault_keeps_failing_owner_and_marks_only_committed_undelivered_peers() {
    let mut constrained = config(M9BossHistory::AwakenedUnseen);
    constrained.capacities.map_nodes = 1;
    let mut unrecorded = config(M9BossHistory::DonuDecaUnseen);
    unrecorded.record_replay = false;
    let mut batch = RunBatch::new_m10(vec![
        constrained,
        config(M9BossHistory::TimeEaterUnseen),
        unrecorded,
    ], 2).unwrap();
    let boundaries = batch.reset(&[Some(11), Some(11), Some(11)]).unwrap();
    let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
    let indices = [RunActionIndex(0); 3];

    let faults = match batch.step(&tokens, &indices).unwrap_err() {
        RunBatchError::Fault { slots } => slots,
        other => panic!("expected retained indexed execution faults, got {other:?}"),
    };
    assert_eq!(faults.len(), 3);
    assert_eq!((faults[0].slot, faults[0].committed, faults[0].delivery_lost), (0, true, false));
    assert_eq!((faults[1].slot, faults[1].committed, faults[1].delivery_lost), (1, true, true));
    assert_eq!((faults[2].slot, faults[2].committed, faults[2].delivery_lost), (2, true, true));

    let failed_owner = batch.record(0).unwrap().unwrap();
    assert_eq!(failed_owner.steps.len(), 1);
    assert!(failed_owner.fault.as_ref().is_some_and(|fault|
        fault.committed && fault.gameplay_recorded && fault.error.kind == RunRecordErrorKind::Capacity
            && fault.error.capacity.as_ref().is_some_and(|capacity| capacity.group == "map_nodes")));
    let delivered_peer = batch.record(1).unwrap().unwrap();
    assert_eq!(delivered_peer.steps.len(), 1);
    assert!(delivered_peer.fault.as_ref().is_some_and(|fault|
        fault.committed && fault.gameplay_recorded && fault.error.kind == RunRecordErrorKind::DeliveryFault));
    let unrecorded_peer = batch.reproduction(2).unwrap();
    assert!(unrecorded_peer.record.is_none());
    assert!(unrecorded_peer.last_accepted_step.is_some());
    assert!(matches!(unrecorded_peer.fault, Some(spire_env::RunSessionError::DeliveryFault(_))));

    let retained = (0..3).map(|slot| batch.reproduction(slot).unwrap()).collect::<Vec<_>>();
    assert_eq!(batch.step(&tokens, &indices).unwrap_err(), RunBatchError::Fault { slots: faults.clone() });
    assert_eq!(batch.reset(&[Some(7), Some(7), Some(7)]).unwrap_err(), RunBatchError::Fault { slots: faults });
    for slot in 0..3 {
        assert_eq!(batch.reproduction(slot).unwrap(), retained[slot], "faulted owner {slot} must not retry or reset");
    }
    batch.close().unwrap();
    batch.close().unwrap();
    assert!(batch.reproduction(0).unwrap().closed);
    assert!(batch.reproduction(1).unwrap().closed);
    assert!(batch.reproduction(2).unwrap().closed);
}

#[test]
fn reset_capacity_fault_retains_first_error_and_marks_successful_peer_resets_undelivered() {
    let mut constrained = config(M9BossHistory::AwakenedUnseen);
    constrained.capacities.permanent_cards = 1;
    let mut unrecorded = config(M9BossHistory::DonuDecaUnseen);
    unrecorded.record_replay = false;
    let mut batch = RunBatch::new_m10(
        vec![
            constrained,
            config(M9BossHistory::TimeEaterUnseen),
            unrecorded,
        ],
        2,
    )
    .unwrap();
    let seeds = [Some(11), Some(11), Some(11)];

    let faults = match batch.reset(&seeds).unwrap_err() {
        RunBatchError::Fault { slots } => slots,
        other => panic!("expected retained indexed reset fault, got {other:?}"),
    };
    assert_eq!(faults.len(), 3);
    assert_eq!(faults[0].slot, 0);
    assert!(!faults[0].delivery_lost);
    assert!(faults[0].committed == false);
    assert!(matches!(
        faults[0].error,
        spire_env::RunSessionError::Capacity {
            group: "permanent_cards",
            required,
            capacity: 1,
        } if required > 1
    ));
    for fault in &faults[1..] {
        assert!(fault.delivery_lost);
        assert!(!fault.committed);
    }

    let failing_record = batch.record(0).unwrap().unwrap();
    assert!(failing_record.steps.is_empty(), "reset failure must not fabricate gameplay");
    assert!(failing_record.fault.as_ref().is_some_and(|fault|
        !fault.committed && !fault.gameplay_recorded
            && fault.error.kind == RunRecordErrorKind::Capacity
            && fault.error.capacity.as_ref().is_some_and(|capacity|
                capacity.group == "permanent_cards" && capacity.capacity == 1 && capacity.required > 1)));

    let delivered_record = batch.record(1).unwrap().unwrap();
    assert!(delivered_record.steps.is_empty(), "a delivered reset is not a gameplay step");
    assert!(delivered_record.fault.as_ref().is_some_and(|fault|
        !fault.committed && !fault.gameplay_recorded
            && fault.error.kind == RunRecordErrorKind::DeliveryFault));

    let unrecorded_peer = batch.reproduction(2).unwrap();
    assert_eq!(unrecorded_peer.seed, Some(11));
    assert_eq!(unrecorded_peer.ordinal, 0);
    assert!(unrecorded_peer.record.is_none());
    assert!(unrecorded_peer.last_accepted_step.is_none());
    assert!(unrecorded_peer.latest_decision.as_ref().is_some_and(Result::is_ok));
    assert!(matches!(
        unrecorded_peer.fault,
        Some(spire_env::RunSessionError::DeliveryFault(_))
    ));

    let retained = (0..3)
        .map(|slot| batch.reproduction(slot).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(batch.reset(&seeds).unwrap_err(), RunBatchError::Fault { slots: faults.clone() });
    assert_eq!(
        batch.step(&[], &[]).unwrap_err(),
        RunBatchError::Fault { slots: faults },
    );
    for slot in 0..3 {
        assert_eq!(batch.reproduction(slot).unwrap(), retained[slot]);
    }

    batch.close().unwrap();
    batch.close().unwrap();
    assert!(batch.record(0).unwrap().unwrap().closed);
    assert!(batch.record(1).unwrap().unwrap().closed);
    assert!(batch.reproduction(2).unwrap().closed);
}
