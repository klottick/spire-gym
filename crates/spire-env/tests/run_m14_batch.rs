//! M14 uses the shared ordered scheduler, including retained execution faults.
use spire_core::{combat::rng::Partition, content::{Character, EncounterId}};
use spire_env::{
    encode_run_m14, replay_run, RunAct1BossTarget, RunAct2BossTarget, RunAct3BossTarget,
    RunActionIndex, RunBatch, RunBatchError, RunBossHistory, RunCapacities,
    RunEpisodeRecord, RunRecordErrorKind, RunSession, RunSessionConfig, RunSessionError,
};

#[path = "../../spire-core/examples/m14_support/mod.rs"]
#[allow(dead_code)]
mod m14_support;

fn config(act1: EncounterId, act2: EncounterId, act3: EncounterId) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: RunBossHistory::M14SilentAct3 {
            act1: match act1 {
                EncounterId::Guardian => RunAct1BossTarget::Guardian,
                EncounterId::Hexaghost => RunAct1BossTarget::Hexaghost,
                EncounterId::SlimeBoss => RunAct1BossTarget::SlimeBoss,
                _ => panic!("foreign Act1 target"),
            },
            act2: match act2 {
                EncounterId::Champ => RunAct2BossTarget::Champ,
                EncounterId::BronzeAutomaton => RunAct2BossTarget::BronzeAutomaton,
                EncounterId::Collector => RunAct2BossTarget::Collector,
                _ => panic!("foreign Act2 target"),
            },
            act3: match act3 {
                EncounterId::AwakenedOne => RunAct3BossTarget::AwakenedOne,
                EncounterId::TimeEater => RunAct3BossTarget::TimeEater,
                EncounterId::DonuAndDeca => RunAct3BossTarget::DonuAndDeca,
                _ => panic!("foreign Act3 target"),
            },
        },
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}
fn default_config() -> RunSessionConfig {
    config(EncounterId::Guardian, EncounterId::Champ, EncounterId::AwakenedOne)
}

#[test]
fn m14_batch_heterogeneous_prefix_preserves_slot_and_candidate_order() {
    let configs = vec![
        default_config(),
        config(EncounterId::Hexaghost, EncounterId::BronzeAutomaton, EncounterId::TimeEater),
    ];
    let mut worker_records = Vec::new();
    for workers in [1, 2] {
        let mut batch = RunBatch::new_m14(configs.clone(), workers).unwrap();
        let mut singles = configs.iter().cloned().map(|c| RunSession::new_m14(c).unwrap()).collect::<Vec<_>>();
        let mut boundaries = batch.reset(&[Some(7), Some(11)]).unwrap();
        let mut single_boundaries = singles.iter_mut().zip([7, 11]).map(|(s, seed)| s.reset(Some(seed)).unwrap()).collect::<Vec<_>>();
        assert_ne!(boundaries[0].token, boundaries[1].token);
        for slot in 0..2 { assert_eq!(boundaries[slot].observation, single_boundaries[slot].observation); }
        for _ in 0..32 {
            if boundaries.iter().any(|b| b.terminal.is_some() || b.observation.action_mask.iter().all(|v| *v == 0)) { break; }
            let tokens = boundaries.iter().map(|b| b.token).collect::<Vec<_>>();
            let steps = batch.step(&tokens, &[RunActionIndex(0); 2]).unwrap();
            for slot in 0..2 {
                let single = singles[slot].step(single_boundaries[slot].token, RunActionIndex(0)).unwrap();
                assert_eq!((steps[slot].reward, steps[slot].terminated, steps[slot].truncated), (single.reward, single.terminated, single.truncated));
                assert_eq!(steps[slot].boundary.observation, single.boundary.observation);
                assert_eq!(steps[slot].boundary.terminal, single.boundary.terminal);
                boundaries[slot] = steps[slot].boundary.clone();
                single_boundaries[slot] = single.boundary;
            }
        }
        let records = (0..2).map(|slot| {
            assert_eq!(batch.record(slot).unwrap(), singles[slot].record().unwrap());
            assert_eq!(batch.reproduction(slot).unwrap(), singles[slot].reproduction().unwrap());
            batch.record(slot).unwrap().unwrap()
        }).collect::<Vec<_>>();
        assert!(records.iter().all(|r| !r.steps.is_empty()));
        worker_records.push(records);
    }
    assert_eq!(worker_records[0], worker_records[1]);
}

#[test]
fn m14_batch_matches_single_full_recorded_endings_across_workers() {
    // The same frozen public caller supplies every action. Identical paired
    // lanes allow each real terminal to be delivered without resetting a peer.
    let cases = m14_support::evidence_cases();
    assert_eq!(cases.len(), 42, "the approved M14 evidence suite has exactly 42 rows");
    let mut completed = 0;
    for (case_index, (seed, scenario)) in cases.iter().copied().enumerate() {
        eprintln!(
            "M14 full-recorded row {}/{} START seed={} scenario={scenario:?}",
            case_index + 1,
            cases.len(),
            seed,
        );
        let mut source = m14_support::make_run(seed, scenario).unwrap();
        let mut caller = m14_support::Caller::new_m14(8192).unwrap();
        let outcome = m14_support::rollout_m14(&mut source, &mut caller, false).unwrap();
        assert!(matches!(outcome, m14_support::CallerOutcome::Death | m14_support::CallerOutcome::Act3NoKeys | m14_support::CallerOutcome::EmptyDeck));
        let final_decision = caller.final_decision.as_ref().unwrap();
        let c = config(scenario.act1_boss, scenario.act2_boss, scenario.act3_boss);
        let mut worker_records: Vec<Vec<RunEpisodeRecord>> = Vec::new();
        for workers in [1, 2] {
            let mut batch = RunBatch::new_m14(vec![c.clone(); 2], workers).unwrap();
            let mut boundaries = batch.reset(&[Some(seed); 2]).unwrap();
            let mut singles = (0..2).map(|_| RunSession::new_m14(c.clone()).unwrap()).collect::<Vec<_>>();
            let mut single_boundaries = singles.iter_mut().map(|s| s.reset(Some(seed)).unwrap()).collect::<Vec<_>>();
            for (index, expected) in caller.trace.iter().enumerate() {
                let action = RunActionIndex(expected.decision.candidates.iter().position(|row| row.action == expected.action).unwrap() as i64);
                let tokens = boundaries.iter().map(|b| b.token).collect::<Vec<_>>();
                let steps = batch.step(&tokens, &[action; 2]).unwrap();
                let next_decision = caller.trace.get(index + 1).map_or(final_decision, |s| &s.decision);
                let next_map = caller.trace.get(index + 1).map_or(caller.final_map.as_ref(), |s| s.map.as_ref());
                let observation = encode_run_m14(next_decision, next_map, &c.capacities).unwrap();
                for slot in 0..2 {
                    let single = singles[slot].step(single_boundaries[slot].token, action).unwrap();
                    assert_eq!(steps[slot].reward, expected.transition.reward);
                    assert_eq!(steps[slot].boundary.terminal, expected.transition.terminal);
                    assert_eq!(steps[slot].terminated, expected.transition.terminal.is_some());
                    assert!(!steps[slot].truncated);
                    assert_eq!(steps[slot].boundary.observation, observation);
                    assert_eq!(steps[slot].boundary.observation, single.boundary.observation);
                    assert_eq!((steps[slot].reward, steps[slot].terminated, steps[slot].truncated), (single.reward, single.terminated, single.truncated));
                    boundaries[slot] = steps[slot].boundary.clone();
                    single_boundaries[slot] = single.boundary;
                }
            }
            let records = (0..2).map(|slot| {
                let record = batch.record(slot).unwrap().unwrap();
                assert_eq!(Some(record.clone()), singles[slot].record().unwrap());
                assert_eq!(batch.reproduction(slot).unwrap(), singles[slot].reproduction().unwrap());
                assert_eq!(record.steps.len(), caller.trace.len());
                assert_eq!(record.terminal, final_decision.terminal);
                for (recorded, expected) in record.steps.iter().zip(&caller.trace) {
                    assert_eq!((&recorded.action, recorded.reward, recorded.terminal), (&expected.action, expected.transition.reward, expected.transition.terminal));
                }
                let decoded = RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
                assert_eq!(decoded, record);
                let report = replay_run(&decoded, c.capacities.clone()).unwrap();
                assert_eq!(report.final_decision, *final_decision);
                assert_eq!(report.final_map, caller.final_map);
                assert_eq!(report.rewards, caller.trace.iter().map(|s| s.transition.reward).collect::<Vec<_>>());
                record
            }).collect::<Vec<_>>();
            worker_records.push(records);
        }
        assert_eq!(worker_records[0], worker_records[1]);
        eprintln!(
            "M14 full-recorded row {}/{} END seed={} scenario={scenario:?} accepted={} terminal={:?}",
            case_index + 1,
            cases.len(),
            seed,
            caller.accepted(),
            final_decision.terminal,
        );
        // `source`, `caller`, and the two worker-count record pairs are local
        // to this row and are dropped before the next declared case starts.
        completed += 1;
    }
    assert_eq!(completed, 42, "the native parity loop must finish every approved row");
}

#[test]
fn m14_batch_retains_first_capacity_fault_and_accepted_peer_work() {
    for workers in [1, 2] {
        let mut constrained = default_config();
        constrained.capacities.map_nodes = 1;
        let mut unrecorded = default_config();
        unrecorded.record_replay = false;
        let configs = vec![constrained.clone(), default_config(), unrecorded];
        let mut batch = RunBatch::new_m14(configs.clone(), workers).unwrap();
        let boundaries = batch.reset(&[Some(7); 3]).unwrap();
        let tokens = boundaries.iter().map(|b| b.token).collect::<Vec<_>>();
        let indices = [RunActionIndex(0); 3];
        let faults = match batch.step(&tokens, &indices).unwrap_err() {
            RunBatchError::Fault { slots } => slots,
            error => panic!("expected accepted publication fault, got {error:?}"),
        };
        assert_eq!(faults.len(), 3);
        assert_eq!((faults[0].slot, faults[0].committed, faults[0].delivery_lost), (0, true, false));
        assert_eq!((faults[1].slot, faults[1].committed, faults[1].delivery_lost), (1, true, true));
        assert_eq!((faults[2].slot, faults[2].committed, faults[2].delivery_lost), (2, true, true));
        let record = batch.record(0).unwrap().unwrap();
        assert_eq!(record.steps.len(), 1);
        assert!(record.fault.as_ref().is_some_and(|f| f.committed && f.gameplay_recorded && f.error.kind == RunRecordErrorKind::Capacity && f.error.capacity.as_ref().is_some_and(|c| c.group == "map_nodes")));
        let mut single = RunSession::new_m14(constrained.clone()).unwrap();
        let initial = single.reset(Some(7)).unwrap();
        assert_eq!(single.step(initial.token, RunActionIndex(0)).unwrap_err(), RunSessionError::DeliveryFault("post-step Run encoding failed"));
        assert_eq!(batch.record(0).unwrap(), single.record().unwrap());
        assert_eq!(batch.reproduction(0).unwrap(), single.reproduction().unwrap());
        let decoded = RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
        assert!(matches!(replay_run(&decoded, constrained.capacities), Err(RunSessionError::Capacity { group: "map_nodes", .. })));
        let report = replay_run(&decoded, RunCapacities::m10_default()).unwrap();
        assert_eq!(report.accepted_count, 1);
        assert_eq!(report.reported_fault, record.fault);
        for slot in 1..3 {
            let mut peer = RunSession::new_m14(configs[slot].clone()).unwrap();
            let initial = peer.reset(Some(7)).unwrap();
            peer.step(initial.token, RunActionIndex(0)).unwrap();
            let expected = peer.reproduction().unwrap();
            let actual = batch.reproduction(slot).unwrap();
            assert_eq!(actual.latest_decision, expected.latest_decision);
            assert_eq!(actual.latest_map, expected.latest_map);
            assert_eq!(actual.last_accepted_step, expected.last_accepted_step);
            assert!(matches!(actual.fault, Some(RunSessionError::DeliveryFault(_))));
        }
        let retained = (0..3).map(|slot| batch.reproduction(slot).unwrap()).collect::<Vec<_>>();
        assert_eq!(batch.step(&tokens, &indices), Err(RunBatchError::Fault { slots: faults.clone() }));
        assert_eq!(batch.reset(&[Some(11); 3]), Err(RunBatchError::Fault { slots: faults }));
        for slot in 0..3 { assert_eq!(batch.reproduction(slot).unwrap(), retained[slot]); }
    }
}

#[test]
fn m14_batch_rejects_invalid_config_before_reset() {
    let mut invalid = default_config();
    invalid.character = Character::Ironclad;
    assert!(matches!(RunBatch::new_m14(vec![default_config(), invalid], 2), Err(RunBatchError::Input { slot: Some(1), error: RunSessionError::Input("M14 requires Silent A0") })));
    let mut wrong_history = default_config();
    wrong_history.boss_history = RunBossHistory::M13SilentAct2 { act1: RunAct1BossTarget::Guardian, act2: RunAct2BossTarget::Champ };
    assert!(matches!(RunBatch::new_m14(vec![wrong_history], 1), Err(RunBatchError::Input { slot: Some(0), error: RunSessionError::Input("boss history does not match Run profile") })));
}
