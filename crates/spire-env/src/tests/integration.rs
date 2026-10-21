//! End-to-end native evidence; no extension feature or Python runtime required.
use std::sync::Arc;
use crate::*;
use spire_core::combat::{Action, CardKind, Combat, EnemyKind, TransitionStatus};
use spire_core::combat::rng::StreamState;

#[test]
fn native_fault_replay_and_python_independence() {
    let mut t = super::support::template();
    t.player_hp = 70;
    t.enemy_hp = 48;
    t.energy_per_turn = 8;
    t.hand = vec![CardSpec { kind: CardKind::Strike, upgraded: false }; 8];
    t.context = PublicContext { act: Some(1), next_encounter: Some(EnemyKind::JawWorm) };
    let catalog = Arc::new(ScenarioCatalog::new(vec![t]).unwrap());
    let requests: Vec<_> = (0..3).map(|i| ResetRequest {
        seed: Some(IntegerSeed::from_be_bytes(&[255, i]).unwrap()), scenario_id: None,
    }).collect();
    let mut baseline = None;
    for workers in [1, 2, 4] {
        let p = CombatPool::new(catalog.clone(), PoolConfig {
            batch_size: 3, workers, record_replay: true, ..PoolConfig::default()
        }).unwrap();
        let mut tokens = p.reset_batch(&[0, 1, 2], &requests).unwrap().decisions.tokens;
        let mut boundaries: Vec<Vec<_>> = {
            let state = p.state.lock().unwrap();
            state.slots.iter().map(|s| {
                let live = s.live.as_ref().unwrap();
                vec![ReplayBoundary { observation: live.row.clone(), info: live.info.clone(), reward: 0.0 }]
            }).collect()
        };
        for _ in 0..8 {
            let result = p.step_batch(&[1; 3], &tokens).unwrap();
            assert_eq!(result.truncated, [false; 3]);
            let state = p.state.lock().unwrap();
            for (i, s) in state.slots.iter().enumerate() {
                let live = s.live.as_ref().unwrap();
                boundaries[i].push(ReplayBoundary {
                    observation: live.row.clone(), info: live.info.clone(), reward: result.rewards[i],
                });
            }
            tokens = result.current.tokens;
        }
        let diagnostics = p.diagnostics().unwrap();
        let records = p.take_completed_records().unwrap();
        assert_eq!(records.len(), 3);
        let mut evidence = vec![];
        for (i, record) in records.iter().enumerate() {
            assert_eq!(record.status, RecordStatus::Completed);
            let replayed = replay(record).unwrap();
            assert_eq!(replayed.boundaries, boundaries[i]);
            assert_eq!(Some(&replayed.final_snapshot), diagnostics.slots[i].snapshot.as_ref());
            assert_eq!(replayed.fault, None);
            assert_eq!(boundaries[i].last().unwrap().info.hp_after_cleanup, Some(76));
            evidence.push((replayed.boundaries, replayed.final_snapshot));
        }
        if let Some(expected) = &baseline { assert_eq!(&evidence, expected); }
        else { baseline = Some(evidence); }
        p.close().unwrap();
    }
}

#[test]
fn complete_runtime_fault_diagnostics_match_across_workers() {
    let mut baseline = None;
    for workers in [1, 2, 4] {
        let p = CombatPool::new(Arc::new(ScenarioCatalog::starter()), PoolConfig {
            batch_size: 3, workers, record_replay: true, ..PoolConfig::default()
        }).unwrap();
        let requests: Vec<_> = (0..3).map(|_| ResetRequest {
            seed: Some(IntegerSeed::from_be_bytes(&[19]).unwrap()),
            scenario_id: Some("starter-jaw-worm-a0-v1".into()),
        }).collect();
        let rows = p.reset_batch(&[0, 1, 2], &requests).unwrap();
        // Private test-only core substitution; not a public fault/configuration API.
        let mut expected = vec![];
        {
            let mut state = p.state.lock().unwrap();
            for i in [0, 2] {
                let live = state.slots[i].live.as_mut().unwrap();
                let mut scenario = live.job.scenario().unwrap();
                scenario.rng.enemy_ai = StreamState::new(
                    scenario.rng.enemy_ai.id().clone(), [3; 32], 1u128 << 64).unwrap();
                live.combat = Combat::new(&scenario).unwrap();
                let mut reference = Combat::new(&scenario).unwrap();
                assert_eq!(reference.step(&Action::EndTurn).unwrap().status, TransitionStatus::Fault);
                expected.push((reference.snapshot(), reference.fault_record()));
            }
        }
        let faults = match p.step_batch(&[0; 3], &rows.decisions.tokens) {
            Err(EnvError::Fault { records }) => records, other => panic!("{other:?}"),
        };
        let diagnostics = p.diagnostics().unwrap();
        assert_eq!(diagnostics.faults, faults);
        assert_eq!(faults.iter().map(|f| f.slot).collect::<Vec<_>>(), [Some(0), Some(2)]);
        for (n, i) in [0, 2].into_iter().enumerate() {
            assert_eq!(diagnostics.slots[i].snapshot.as_ref(), Some(&expected[n].0));
            assert_eq!(faults[n].core_fault, expected[n].1);
        }
        // Tokens intentionally differ across pool instances; all diagnostic state
        // and complete fault provenance must otherwise match, not just messages.
        let mut normalized = diagnostics.clone();
        for slot in &mut normalized.slots { slot.token = [0, 0]; }
        if let Some(expected) = &baseline { assert_eq!(&normalized, expected); }
        else { baseline = Some(normalized); }
        assert!(matches!(p.reset_batch(&[0, 1, 2], &requests),
            Err(EnvError::State { reason: StateReason::Faulted, .. })));
        assert_eq!(p.diagnostics().unwrap(), diagnostics);
        p.close().unwrap();
    }
}
