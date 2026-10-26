use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    encode_run_m12, RunActionIndex, RunBatch, RunBatchError, RunBossHistory, RunCapacities,
    RunReproduction, RunSession, RunSessionConfig, RunSessionError,
};

#[path = "../../spire-core/examples/m12_support/mod.rs"]
#[allow(dead_code)]
mod m12_support;

fn history(scenario: m12_support::Scenario) -> RunBossHistory {
    match scenario {
        m12_support::Scenario::Guardian | m12_support::Scenario::Death => RunBossHistory::Act1GuardianUnseen,
        m12_support::Scenario::Hexaghost => RunBossHistory::Act1HexaghostUnseen,
        m12_support::Scenario::SlimeBoss => RunBossHistory::Act1SlimeBossUnseen,
    }
}

fn config(scenario: m12_support::Scenario) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: history(scenario),
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn caller(seed: u64, scenario: m12_support::Scenario) -> m12_support::Caller {
    let mut run = m12_support::make_run(seed, scenario).unwrap();
    let mut caller = m12_support::Caller::new(4096).unwrap();
    assert_eq!(m12_support::rollout(&mut run, &mut caller, scenario).unwrap(), m12_support::CallerOutcome::Act2Ready);
    caller
}

fn action_index(step: &m12_support::TraceStep) -> RunActionIndex {
    let index = step.decision.candidates.iter().position(|candidate| candidate.action == step.action)
        .expect("caller action must be in the published candidate list");
    RunActionIndex(index as i64)
}

fn expected_observation(caller: &m12_support::Caller, step: usize) -> spire_env::RunObservation {
    let decision = caller.trace.get(step + 1).map(|next| &next.decision)
        .unwrap_or_else(|| caller.final_decision.as_ref().unwrap());
    let map = caller.trace.get(step + 1).and_then(|next| next.map.as_ref()).or(caller.final_map.as_ref());
    encode_run_m12(decision, map, &RunCapacities::m10_default()).unwrap()
}

fn run_two_slots(workers: usize) -> (Vec<spire_env::RunEpisodeRecord>, Vec<RunReproduction>) {
    let seed = 7;
    let callers = [
        caller(seed, m12_support::Scenario::Guardian),
        caller(seed, m12_support::Scenario::SlimeBoss),
    ];
    assert!(callers[0].trace.len() < callers[1].trace.len(), "the cutoff slot is shorter than the still-active peer");

    let mut batch = RunBatch::new_m12(vec![
        config(m12_support::Scenario::Guardian),
        config(m12_support::Scenario::SlimeBoss),
    ], workers).unwrap();
    let mut boundaries = batch.reset(&[Some(seed), Some(seed)]).unwrap();
    assert_eq!(boundaries[0].observation,
        encode_run_m12(&callers[0].trace[0].decision, callers[0].trace[0].map.as_ref(), &RunCapacities::m10_default()).unwrap());
    assert_eq!(boundaries[1].observation,
        encode_run_m12(&callers[1].trace[0].decision, callers[1].trace[0].map.as_ref(), &RunCapacities::m10_default()).unwrap());

    let paired_steps = callers[0].trace.len();
    for index in 0..paired_steps {
        let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
        let actions = [action_index(&callers[0].trace[index]), action_index(&callers[1].trace[index])];
        let steps = batch.step(&tokens, &actions).unwrap();
        for slot in 0..2 {
            assert_eq!(steps[slot].reward, callers[slot].trace[index].transition.reward, "slot {slot} reward at {index}");
            assert_eq!(steps[slot].boundary.terminal, callers[slot].trace[index].transition.terminal, "slot {slot} terminal at {index}");
            assert_eq!(steps[slot].boundary.observation, expected_observation(&callers[slot], index), "slot {slot} observation at {index}");
            assert!(!steps[slot].terminated && !steps[slot].truncated);
            boundaries[slot] = steps[slot].boundary.clone();
        }
    }

    assert!(boundaries[0].observation.action_mask.iter().all(|value| *value == 0));
    assert!(boundaries[1].observation.action_mask.iter().any(|value| *value == 1));
    let before = [batch.reproduction(0).unwrap(), batch.reproduction(1).unwrap()];
    let records_before = [batch.record(0).unwrap().unwrap(), batch.record(1).unwrap().unwrap()];
    let peer_action = action_index(&callers[1].trace[paired_steps]);
    assert_eq!(batch.step(
        &[boundaries[0].token, boundaries[1].token],
        &[RunActionIndex(0), peer_action],
    ), Err(RunBatchError::Input {
        slot: Some(0), error: RunSessionError::UnsupportedBoundary("M12 stops at Act2Ready"),
    }));
    assert_eq!(batch.reproduction(0).unwrap(), before[0]);
    assert_eq!(batch.reproduction(1).unwrap(), before[1], "all-slot preflight rejects before advancing the valid peer");
    assert_eq!(batch.record(0).unwrap().unwrap(), records_before[0]);
    assert_eq!(batch.record(1).unwrap().unwrap(), records_before[1]);
    assert!(batch.reproduction(0).unwrap().fault.is_none());
    assert!(batch.reproduction(1).unwrap().fault.is_none());
    (records_before.to_vec(), vec![batch.reproduction(0).unwrap(), batch.reproduction(1).unwrap()])
}

fn run_single_prefix(scenario: m12_support::Scenario, count: usize) -> (spire_env::RunEpisodeRecord, RunReproduction) {
    let seed = 7;
    let caller = caller(seed, scenario);
    let mut session = RunSession::new_m12(config(scenario)).unwrap();
    let mut boundary = session.reset(Some(seed)).unwrap();
    for index in 0..count {
        let step = session.step(boundary.token, action_index(&caller.trace[index])).unwrap();
        assert_eq!(step.reward, caller.trace[index].transition.reward);
        assert_eq!(step.boundary.observation, expected_observation(&caller, index));
        boundary = step.boundary;
    }
    (session.record().unwrap().unwrap(), session.reproduction().unwrap())
}

#[test]
fn m12_native_batch_matches_single_sessions_and_is_worker_count_independent() {
    let (one_records, one_reproductions) = run_two_slots(1);
    let (two_records, two_reproductions) = run_two_slots(2);
    let singles = [
        run_single_prefix(m12_support::Scenario::Guardian, 269),
        run_single_prefix(m12_support::Scenario::SlimeBoss, 269),
    ];
    let single_records = singles.iter().map(|(record, _)| record.clone()).collect::<Vec<_>>();
    let single_reproductions = singles.iter().map(|(_, reproduction)| reproduction.clone()).collect::<Vec<_>>();
    assert_eq!(one_records, single_records);
    assert_eq!(two_records, single_records);
    assert_eq!(one_reproductions, single_reproductions);
    assert_eq!(two_reproductions, single_reproductions);
    assert_eq!(one_records[0].steps.len(), 269);
    assert_eq!(one_records[1].steps.len(), 269, "the peer remains live at the paired cutoff");
    assert_eq!(one_records[0].terminal, None);
    assert_eq!(one_records[1].terminal, None);
    assert!(one_records.iter().all(|record| record.fault.is_none()));
}
