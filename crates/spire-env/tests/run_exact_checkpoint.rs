//! M34c exact checkpoints (restore/fork) and replay to a chosen decision (design §4.7, §4.8).
use spire_core::{combat::rng::Partition, content::Character, run::{GameAction, RunTerminal}};
use spire_env::{replay_run, RunActionIndex, RunBoundary, RunCapacities, RunEpisodeRecord, RunObservation, RunSessionConfig, M9BossHistory, RunSession,
    RunSessionError, StandardCreation, StandardNeow, StandardRunConfig};

fn standard(character: Character, ascension: u8) -> StandardRunConfig {
    let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
    config.record_replay = true;
    config
}

/// A deterministic mixed policy over the published legal mask (varies the chosen row with the step number).
fn pick(boundary: &RunBoundary, step: usize) -> usize {
    let legal = boundary.observation.action_mask.iter().enumerate().filter_map(|(i, v)| (*v != 0).then_some(i)).collect::<Vec<_>>();
    legal[(step * 7 + 3) % legal.len()]
}

type Trace = Vec<(RunObservation, i32, bool)>;

fn advance(session: &mut RunSession, mut boundary: RunBoundary, steps: usize, salt: usize) -> (RunBoundary, Trace) {
    let mut trace = Vec::new();
    for step in 0..steps {
        if boundary.terminal.is_some() { break; }
        let result = session.step(boundary.token, RunActionIndex(pick(&boundary, step + salt) as i64)).unwrap();
        trace.push((result.boundary.observation.clone(), result.reward, result.terminated));
        boundary = result.boundary;
    }
    (boundary, trace)
}

fn m10(record_replay: bool) -> RunSessionConfig {
    RunSessionConfig { character: Character::Ironclad, ascension: 0, partition: Partition::Test, boss_history: M9BossHistory::AwakenedUnseen,
        capacities: RunCapacities::m10_default(), record_replay }
}

fn fresh(character: Character, seed: u64, prefix: usize) -> (RunSession, RunBoundary) {
    let mut session = RunSession::new_standard(standard(character, 0)).unwrap();
    let boundary = session.reset(Some(seed)).unwrap();
    let (boundary, _) = advance(&mut session, boundary, prefix, 0);
    (session, boundary)
}

/// A Standard session frozen by a post-commit map capacity fault, plus a checkpoint taken just before the faulting step.
fn frozen_with_checkpoint() -> (RunSession, spire_env::ExactCheckpoint, RunSessionError) {
    let mut config = standard(Character::Silent, 0);
    config.capacities.map_nodes = 1;
    let mut session = RunSession::new_standard(config).unwrap();
    let mut boundary = session.reset(Some(22)).unwrap();
    for _ in 0..8 {
        let checkpoint = session.exact_checkpoint().unwrap();
        let index = boundary.observation.action_mask.iter().position(|&v| v != 0).unwrap();
        match session.step(boundary.token, RunActionIndex(index as i64)) {
            Ok(result) => boundary = result.boundary,
            Err(error) => return (session, checkpoint, error),
        }
    }
    panic!("map capacity should be exceeded when Neow enters the map");
}

#[test]
fn exact_restore_replays_identically() {
    for character in [Character::Ironclad, Character::Watcher] {
        let (mut session, boundary) = fresh(character, 41, 12);
        let checkpoint = session.exact_checkpoint().unwrap();
        let (_, first) = advance(&mut session, boundary.clone(), 80, 5);
        let restored = session.exact_restore(&checkpoint).unwrap();
        assert_eq!(restored.observation, boundary.observation);
        assert_eq!(restored.terminal, boundary.terminal);
        let (_, second) = advance(&mut session, restored, 80, 5);
        assert!(!first.is_empty());
        assert_eq!(first, second, "{character:?}: the same actions after restore must give the same hidden future");
    }
}

#[test]
fn exact_restore_stales_old_tokens() {
    let mut session = RunSession::new_standard(standard(Character::Defect, 0)).unwrap();
    let start = session.reset(Some(5)).unwrap();
    let (at_checkpoint, _) = advance(&mut session, start.clone(), 3, 0);
    let checkpoint = session.exact_checkpoint().unwrap();
    let (later, _) = advance(&mut session, at_checkpoint.clone(), 3, 0);
    let restored = session.exact_restore(&checkpoint).unwrap();
    for stale in [start.token, at_checkpoint.token, later.token] {
        assert_eq!(session.step(stale, RunActionIndex(pick(&restored, 0) as i64)), Err(RunSessionError::StaleToken));
    }
    assert_ne!(restored.token, at_checkpoint.token);
    assert_eq!(session.decision().unwrap(), restored);
    assert!(session.step(restored.token, RunActionIndex(pick(&restored, 0) as i64)).is_ok());
}

#[test]
fn restore_then_reset_none_continues_checkpoint_ordinal() {
    let mut session = RunSession::new_standard(standard(Character::Ironclad, 0)).unwrap();
    session.reset(Some(77)).unwrap();
    session.reset(None).unwrap();
    let checkpoint = session.exact_checkpoint().unwrap();
    session.reset(None).unwrap();
    session.reset(Some(9)).unwrap();
    session.exact_restore(&checkpoint).unwrap();
    let record = session.record().unwrap().unwrap();
    assert_eq!((record.seed, record.ordinal), (77, 1));
    let next = session.reset(None).unwrap();
    let record = session.record().unwrap().unwrap();
    assert_eq!((record.seed, record.ordinal), (77, 2));
    let mut reference = RunSession::new_standard(standard(Character::Ironclad, 0)).unwrap();
    reference.reset(Some(77)).unwrap();
    reference.reset(None).unwrap();
    assert_eq!(reference.reset(None).unwrap().observation, next.observation);
}

#[test]
fn exact_fork_is_independent_of_source_and_siblings() {
    let (mut source, boundary) = fresh(Character::Silent, 63, 10);
    let checkpoint = source.exact_checkpoint().unwrap();
    let mut a = RunSession::exact_fork(&checkpoint).unwrap();
    let mut b = RunSession::exact_fork(&checkpoint).unwrap();
    let a_start = a.decision().unwrap();
    let b_start = b.decision().unwrap();
    assert_eq!(a_start.observation, boundary.observation);
    assert_eq!(b_start.observation, boundary.observation);
    for token in [boundary.token, b_start.token] {
        assert_eq!(a.step(token, RunActionIndex(pick(&a_start, 0) as i64)), Err(RunSessionError::StaleToken));
    }
    let (_, a_trace) = advance(&mut a, a_start, 30, 1);
    let (_, source_trace) = advance(&mut source, boundary.clone(), 30, 2);
    assert_eq!(b.decision().unwrap().observation, boundary.observation, "sibling unchanged by its sibling and the source");
    let b_now = b.decision().unwrap();
    let (_, b_trace) = advance(&mut b, b_now, 30, 1);
    assert_eq!(a_trace, b_trace, "two forks given the same actions agree");
    assert_ne!(source_trace, a_trace);
    let prefix = checkpoint_record_len(&a, a_trace.len());
    assert_eq!(b.record().unwrap().unwrap().steps.len(), prefix + b_trace.len());
}

fn checkpoint_record_len(session: &RunSession, suffix: usize) -> usize {
    session.record().unwrap().unwrap().steps.len() - suffix
}

#[test]
fn exact_checkpoint_is_unchanged_by_later_steps() {
    let (mut session, boundary) = fresh(Character::Watcher, 88, 15);
    let record_then = session.record().unwrap().unwrap();
    let checkpoint = session.exact_checkpoint().unwrap();
    let (later, _) = advance(&mut session, boundary.clone(), 40, 3);
    session.reset(Some(1)).unwrap();
    let fork = RunSession::exact_fork(&checkpoint).unwrap();
    assert_eq!(fork.decision().unwrap().observation, boundary.observation);
    assert_ne!(later.observation, boundary.observation);
    assert_eq!(fork.record().unwrap().unwrap(), record_then);
    let again = RunSession::exact_fork(&checkpoint).unwrap();
    assert_eq!(again.decision().unwrap().observation, boundary.observation, "a checkpoint restores any number of times");
}

#[test]
fn taking_an_exact_checkpoint_does_not_change_the_live_trajectory() {
    let mut plain = RunSession::new_standard(standard(Character::Defect, 0)).unwrap();
    let mut probed = RunSession::new_standard(standard(Character::Defect, 0)).unwrap();
    let mut a = plain.reset(Some(314)).unwrap();
    let mut b = probed.reset(Some(314)).unwrap();
    let mut kept = Vec::new();
    for step in 0..60 {
        if a.terminal.is_some() { break; }
        kept.push(probed.exact_checkpoint().unwrap());
        let index = pick(&a, step);
        let ra = plain.step(a.token, RunActionIndex(index as i64)).unwrap();
        let rb = probed.step(b.token, RunActionIndex(index as i64)).unwrap();
        assert_eq!((&ra.boundary.observation, ra.reward, &ra.reward_events), (&rb.boundary.observation, rb.reward, &rb.reward_events));
        a = ra.boundary;
        b = rb.boundary;
    }
    assert!(!kept.is_empty());
    let (pr, br) = (plain.record().unwrap().unwrap(), probed.record().unwrap().unwrap());
    assert_eq!((pr.steps, pr.terminal), (br.steps, br.terminal));
}

#[test]
fn exact_checkpoint_rejected_on_frozen_or_closed() {
    let unreset = RunSession::new_standard(standard(Character::Ironclad, 0)).unwrap();
    assert_eq!(unreset.exact_checkpoint().err(), Some(RunSessionError::NotReset));
    let (frozen, _, fault) = frozen_with_checkpoint();
    assert_eq!(frozen.exact_checkpoint().err(), Some(fault));
    let (mut closed, _) = fresh(Character::Ironclad, 3, 2);
    closed.close().unwrap();
    assert_eq!(closed.exact_checkpoint().err(), Some(RunSessionError::Closed));
}

#[test]
fn exact_checkpoint_on_a_terminal_boundary() {
    let mut session = RunSession::new_standard(standard(Character::Ironclad, 0)).unwrap();
    let mut boundary = session.reset(Some(2)).unwrap();
    for step in 0..3000 {
        if boundary.terminal.is_some() { break; }
        boundary = session.step(boundary.token, RunActionIndex(pick(&boundary, step) as i64)).unwrap().boundary;
    }
    let terminal = boundary.terminal.expect("a mixed policy ends the Run");
    let checkpoint = session.exact_checkpoint().unwrap();
    let fork = RunSession::exact_fork(&checkpoint).unwrap();
    assert_eq!(fork.decision().unwrap().terminal, Some(terminal));
    assert!(matches!(terminal, RunTerminal::Death | RunTerminal::EmptyDeck | RunTerminal::Act3NoKeys | RunTerminal::HeartVictory));
}

#[test]
fn exact_restore_rejected_on_frozen_session() {
    let (mut frozen, checkpoint, fault) = frozen_with_checkpoint();
    assert_eq!(frozen.exact_restore(&checkpoint).err(), Some(fault.clone()));
    assert_eq!(frozen.decision().err(), Some(fault), "a rejected restore leaves the frozen session unchanged");
    let mut closed = RunSession::exact_fork(&checkpoint).unwrap();
    closed.close().unwrap();
    assert_eq!(closed.exact_restore(&checkpoint).err(), Some(RunSessionError::Closed));
}

#[test]
fn exact_fork_allowed_after_source_froze() {
    let (frozen, checkpoint, fault) = frozen_with_checkpoint();
    assert!(frozen.decision().is_err());
    let mut fork = RunSession::exact_fork(&checkpoint).unwrap();
    let boundary = fork.decision().unwrap();
    let reproduction = fork.reproduction().unwrap();
    assert_eq!(reproduction.fault, None);
    assert_eq!(reproduction.record.unwrap().fault, None, "the fork does not inherit the source's later fault");
    // An explicit caller choice, not a retry: the same action faults the fork in the same deterministic way.
    let index = boundary.observation.action_mask.iter().position(|&v| v != 0).unwrap();
    assert_eq!(fork.step(boundary.token, RunActionIndex(index as i64)).err(), Some(fault));
}

#[test]
fn exact_restore_rejects_other_config() {
    let (session, _) = fresh(Character::Ironclad, 12, 4);
    let checkpoint = session.exact_checkpoint().unwrap();
    let mut capacities = standard(Character::Ironclad, 0);
    capacities.capacities.offers += 1;
    let mut creation = standard(Character::Ironclad, 0);
    creation.creation = StandardCreation { neow: StandardNeow::Short, ..StandardCreation::experienced() };
    let mut no_record = standard(Character::Ironclad, 0);
    no_record.record_replay = false;
    for config in [standard(Character::Silent, 0), standard(Character::Ironclad, 1), capacities, creation, no_record] {
        let mut other = RunSession::new_standard(config).unwrap();
        let before = other.reset(Some(12)).unwrap();
        assert!(matches!(other.exact_restore(&checkpoint), Err(RunSessionError::Input(_))));
        assert_eq!(other.decision().unwrap(), before, "a rejected restore changes nothing");
    }
    let mut captured = RunSession::new_m10(m10(true)).unwrap();
    captured.reset(Some(12)).unwrap();
    assert!(matches!(captured.exact_restore(&checkpoint), Err(RunSessionError::Input(_))));
}

#[test]
fn restored_record_is_prefix_plus_suffix_and_replays() {
    let (mut session, boundary) = fresh(Character::Watcher, 19, 9);
    let prefix = session.record().unwrap().unwrap();
    let checkpoint = session.exact_checkpoint().unwrap();
    advance(&mut session, boundary, 25, 4);
    let restored = session.exact_restore(&checkpoint).unwrap();
    assert_eq!(session.record().unwrap().unwrap(), prefix);
    let (_, suffix) = advance(&mut session, restored, 20, 11);
    let record = session.record().unwrap().unwrap();
    assert_eq!(record.steps.len(), prefix.steps.len() + suffix.len());
    assert_eq!(record.steps[..prefix.steps.len()], prefix.steps[..]);
    assert_eq!(record.steps[prefix.steps.len()..].iter().map(|s| s.reward).collect::<Vec<_>>(), suffix.iter().map(|s| s.1).collect::<Vec<_>>());
    let bytes = record.to_json().unwrap();
    let report = replay_run(&RunEpisodeRecord::from_json(&bytes).unwrap(), record.config.capacities.clone()).unwrap();
    assert_eq!(report.accepted_count, record.steps.len());
}

#[test]
fn exact_checkpoint_debug_is_redacted() {
    let (session, _) = fresh(Character::Defect, 123456789, 6);
    let checkpoint = session.exact_checkpoint().unwrap();
    let text = format!("{checkpoint:?} {checkpoint:#?}");
    assert_eq!(format!("{checkpoint:?}"), "ExactCheckpoint { .. }");
    for leak in ["123456789", "seed", "hp", "Run", "rng"] { assert!(!text.contains(leak), "{leak} leaked: {text}"); }
}

#[test]
fn exact_checkpoint_captured_profile_smoke() {
    let mut session = RunSession::new_m10(m10(true)).unwrap();
    let start = session.reset(Some(31)).unwrap();
    let (boundary, _) = advance(&mut session, start, 6, 0);
    let checkpoint = session.exact_checkpoint().unwrap();
    let (_, first) = advance(&mut session, boundary.clone(), 40, 2);
    let restored = session.exact_restore(&checkpoint).unwrap();
    assert_eq!(restored.observation, boundary.observation);
    let (_, second) = advance(&mut session, restored, 40, 2);
    assert_eq!(first, second);
    let record = session.record().unwrap().unwrap();
    assert_eq!(replay_run(&record, record.config.capacities.clone()).unwrap().accepted_count, record.steps.len());
}

// ---- c2: replay to a chosen decision ----

fn live_boundaries(character: Character, seed: u64, steps: usize) -> (RunEpisodeRecord, Vec<RunBoundary>) {
    let mut session = RunSession::new_standard(standard(character, 0)).unwrap();
    let mut boundary = session.reset(Some(seed)).unwrap();
    let mut seen = vec![boundary.clone()];
    for step in 0..steps {
        if boundary.terminal.is_some() { break; }
        boundary = session.step(boundary.token, RunActionIndex(pick(&boundary, step) as i64)).unwrap().boundary;
        seen.push(boundary.clone());
    }
    (session.record().unwrap().unwrap(), seen)
}

#[test]
fn from_record_at_k_equals_live_boundary_k() {
    let (record, seen) = live_boundaries(Character::Silent, 404, 120);
    let n = record.steps.len();
    let mid_combat = (1..n).find(|&k| matches!(record.steps[k - 1].action, GameAction::Combat(_)) && matches!(record.steps[k].action, GameAction::Combat(_)))
        .expect("the rollout enters combat");
    for k in [0, 1, mid_combat, n / 2, n] {
        let mut session = RunSession::from_record(&record, k, RunCapacities::standard_default()).unwrap();
        let boundary = session.decision().unwrap();
        assert_eq!(boundary.observation, seen[k].observation, "boundary {k}");
        assert_eq!(boundary.terminal, seen[k].terminal);
        let prefix = session.record().unwrap().unwrap();
        assert_eq!(prefix.steps[..], record.steps[..k]);
        assert_eq!((prefix.seed, prefix.ordinal), (record.seed, record.ordinal));
        // The live session continues: the recorded suffix reproduces the remaining boundaries.
        let mut boundary = boundary;
        for (offset, step) in record.steps[k..].iter().take(10).enumerate() {
            let index = session.reproduction().unwrap().latest_decision.unwrap().unwrap().candidates.iter().position(|c| c.action == step.action).unwrap();
            let result = session.step(boundary.token, RunActionIndex(index as i64)).unwrap();
            assert_eq!((result.reward, result.boundary.terminal), (step.reward, step.terminal));
            assert_eq!(result.boundary.observation, seen[k + offset + 1].observation);
            boundary = result.boundary;
        }
    }
}

#[test]
fn from_record_continues_ordinal_and_rejects_divergence() {
    let mut session = RunSession::new_standard(standard(Character::Defect, 0)).unwrap();
    session.reset(Some(8)).unwrap();
    let boundary = session.reset(None).unwrap();
    advance(&mut session, boundary, 5, 0);
    let record = session.record().unwrap().unwrap();
    assert_eq!(record.ordinal, 1);
    let mut resumed = RunSession::from_record(&record, 5, RunCapacities::standard_default()).unwrap();
    resumed.reset(None).unwrap();
    assert_eq!(resumed.record().unwrap().unwrap().ordinal, 2);
    let mut tampered = record.clone();
    tampered.steps[2].reward += 1;
    assert!(matches!(RunSession::from_record(&tampered, 5, RunCapacities::standard_default()), Err(RunSessionError::IncompatibleReplay(_))));
    assert!(RunSession::from_record(&tampered, 2, RunCapacities::standard_default()).is_ok(), "only the replayed prefix is checked");
    let mut small = RunCapacities::standard_default();
    small.offers = 0;
    assert!(matches!(RunSession::from_record(&record, 1, small), Err(RunSessionError::Input(_))));
}

#[test]
fn from_record_rejects_k_past_end() {
    let (record, _) = live_boundaries(Character::Watcher, 9, 12);
    let n = record.steps.len();
    assert!(RunSession::from_record(&record, n, RunCapacities::standard_default()).is_ok());
    assert!(matches!(RunSession::from_record(&record, n + 1, RunCapacities::standard_default()), Err(RunSessionError::Input(_))));
    assert!(matches!(RunSession::from_record(&record, usize::MAX, RunCapacities::standard_default()), Err(RunSessionError::Input(_))));
}

#[test]
fn from_record_reproduces_a_recorded_fault_before_k() {
    let (frozen, _, fault) = frozen_with_checkpoint();
    let record = frozen.record().unwrap().unwrap();
    let n = record.steps.len();
    assert_eq!(record.fault.as_ref().map(|f| (f.committed, f.gameplay_recorded)), Some((true, true)));
    let caps = record.config.capacities.clone();
    assert_eq!(RunSession::from_record(&record, n, caps.clone()).err(), Some(fault), "the recorded fault reproduces, it is not skipped");
    let before = RunSession::from_record(&record, n - 1, caps).unwrap();
    assert!(before.decision().is_ok(), "the decision before the faulting step is live");
    assert_eq!(before.reproduction().unwrap().fault, None);
}

#[test]
fn from_record_resumes_after_a_recorded_transport_fault() {
    // A transport (delivery) fault is recorded after an accepted step but cannot recur on replay; as for `replay_run`,
    // the prefix still replays and the decision is live (the fault was the transport's, not the game's).
    let (mut session, _) = fresh(Character::Ironclad, 52, 6);
    let live = session.decision().unwrap();
    session.fail_delivery("Run transport packaging failed").unwrap();
    let record = session.record().unwrap().unwrap();
    assert_eq!(record.fault.as_ref().map(|f| (f.committed, f.gameplay_recorded)), Some((true, true)));
    assert!(replay_run(&record, record.config.capacities.clone()).is_ok());
    let resumed = RunSession::from_record(&record, record.steps.len(), record.config.capacities.clone()).unwrap();
    assert_eq!(resumed.decision().unwrap().observation, live.observation);
    assert_eq!(resumed.reproduction().unwrap().fault, None);
}
