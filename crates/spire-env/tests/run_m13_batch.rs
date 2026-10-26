use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    RunAct1BossTarget, RunAct2BossTarget, RunActionIndex, RunBatch, RunBatchError,
    RunBossHistory, RunCapacities, RunEpisodeRecord, RunReproduction, RunSession,
    RunSessionConfig, RunSessionError, RunStepResult,
};

fn config(act1: RunAct1BossTarget, act2: RunAct2BossTarget) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: RunBossHistory::M13SilentAct2 { act1, act2 },
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Prefix {
    observations: Vec<Vec<spire_env::RunObservation>>,
    rewards: Vec<Vec<i32>>,
    terminals: Vec<Vec<Option<spire_core::run::RunTerminal>>>,
    records: Vec<Option<RunEpisodeRecord>>,
    reproductions: Vec<RunReproduction>,
    actions_per_slot: Vec<usize>,
}

fn prefix(workers: usize) -> Prefix {
    let configs = vec![
        config(RunAct1BossTarget::Guardian, RunAct2BossTarget::Champ),
        config(RunAct1BossTarget::Hexaghost, RunAct2BossTarget::BronzeAutomaton),
    ];
    let mut batch = RunBatch::new_m13(configs.clone(), workers).unwrap();
    let mut singles = configs.into_iter()
        .map(|config| RunSession::new_m13(config).unwrap())
        .collect::<Vec<_>>();
    let mut boundaries = batch.reset(&[Some(7), Some(7)]).unwrap();
    let mut single_boundaries = singles.iter_mut()
        .map(|session| session.reset(Some(7)).unwrap())
        .collect::<Vec<_>>();
    let mut observations = vec![Vec::new(); 2];
    let mut rewards = vec![Vec::new(); 2];
    let mut terminals = vec![Vec::new(); 2];
    let mut actions_per_slot = vec![0; 2];

    for slot in 0..2 {
        assert_eq!(boundaries[slot].observation, single_boundaries[slot].observation);
        observations[slot].push(boundaries[slot].observation.clone());
    }
    for _ in 0..32 {
        if boundaries.iter().any(|boundary| boundary.observation.action_mask.iter().all(|value| *value == 0)) {
            break;
        }
        let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
        let indices = vec![RunActionIndex(0); 2];
        let batch_steps: Vec<RunStepResult> = batch.step(&tokens, &indices).unwrap();
        for slot in 0..2 {
            let single_step = singles[slot].step(
                single_boundaries[slot].token,
                RunActionIndex(0),
            ).unwrap();
            assert_eq!(batch_steps[slot].reward, single_step.reward, "reward at slot {slot}");
            assert_eq!(batch_steps[slot].terminated, single_step.terminated, "termination at slot {slot}");
            assert_eq!(batch_steps[slot].truncated, single_step.truncated, "truncation at slot {slot}");
            assert_eq!(batch_steps[slot].boundary.terminal, single_step.boundary.terminal);
            assert_eq!(batch_steps[slot].boundary.observation, single_step.boundary.observation);
            assert_eq!(batch.record(slot).unwrap(), singles[slot].record().unwrap());
            assert_eq!(batch.reproduction(slot).unwrap(), singles[slot].reproduction().unwrap());
            rewards[slot].push(batch_steps[slot].reward);
            terminals[slot].push(batch_steps[slot].boundary.terminal);
            observations[slot].push(batch_steps[slot].boundary.observation.clone());
            actions_per_slot[slot] += 1;
            boundaries[slot] = batch_steps[slot].boundary.clone();
            single_boundaries[slot] = single_step.boundary;
        }
    }

    Prefix {
        observations,
        rewards,
        terminals,
        records: (0..2).map(|slot| batch.record(slot).unwrap()).collect(),
        reproductions: (0..2).map(|slot| batch.reproduction(slot).unwrap()).collect(),
        actions_per_slot,
    }
}

#[test]
fn m13_batch_matches_single_prefix_and_is_worker_count_independent() {
    let one_worker = prefix(1);
    let two_workers = prefix(2);
    assert_eq!(one_worker, two_workers);
    assert!(one_worker.actions_per_slot.iter().all(|actions| *actions > 0));
    assert!(one_worker.observations.iter().zip(&one_worker.actions_per_slot).all(|(rows, actions)| rows.len() == 1 + actions));
}

#[test]
fn m13_batch_rejects_invalid_config_before_any_reset() {
    let mut invalid = config(RunAct1BossTarget::Guardian, RunAct2BossTarget::Champ);
    invalid.character = Character::Ironclad;
    assert!(matches!(
        RunBatch::new_m13(
            vec![config(RunAct1BossTarget::Guardian, RunAct2BossTarget::Champ), invalid],
            2,
        ),
        Err(RunBatchError::Input {
            slot: Some(1),
            error: RunSessionError::Input("M13 requires Silent A0"),
        })
    ));
}
