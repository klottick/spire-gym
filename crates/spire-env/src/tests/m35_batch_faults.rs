//! M35 task 3 (design §3.3, §4 fault corpus): per-slot statuses, committed-result recovery, no retry, the isolate policy
//! and its resume rule, the SAME_STEP / NEXT_STEP autoreset-fault composites and export failures, under both policies.
use super::m35_batch_unit::standard_catalog;
use super::m35_r2::r1;
use crate::session_batch::TestInject;
use crate::{
    AutoresetMode, BatchConfig, BatchFaultStage, BatchOperation, CombatPolicyBoundary, CombatTrainingBatch, CombatTrainingCatalog,
    CombatTrainingSession, FaultPolicy, RunActionIndex, RunDecisionToken, RunSession, RunSessionBatch, RunSessionError, SessionBatchError,
    SlotDelivery, SlotRow, StandardRunConfig,
};
use spire_core::{combat::rng::Partition, content::Character};
use std::sync::Arc;

fn cfg(workers: usize, autoreset: AutoresetMode, fault_policy: FaultPolicy) -> BatchConfig { BatchConfig { workers, autoreset, fault_policy } }
fn catalog() -> Arc<CombatTrainingCatalog> { Arc::new(CombatTrainingCatalog::standard_from_json(&standard_catalog()).unwrap()) }
fn single() -> CombatTrainingSession { CombatTrainingSession::new_standard(CombatTrainingCatalog::standard_from_json(&standard_catalog()).unwrap(), Partition::Test).unwrap() }
const SEEDS: [u64; 3] = [501, 502, 503];

/// A started 3-slot combat batch, its current rows and three single twins at the same boundary.
fn started(config: BatchConfig) -> (CombatTrainingBatch, Vec<CombatPolicyBoundary>, Vec<CombatTrainingSession>, Vec<CombatPolicyBoundary>) {
    let mut batch = CombatTrainingBatch::new_combat(catalog(), Partition::Test, 3, None, config).unwrap();
    let rows = batch.reset(&SEEDS.map(Some), &[None, None, None], None).unwrap().into_iter().map(|r| r.boundary).collect();
    let mut singles: Vec<_> = (0..3).map(|_| single()).collect();
    let sb = singles.iter_mut().zip(SEEDS).map(|(s, seed)| s.reset(Some(seed), None).unwrap()).collect();
    (batch, rows, singles, sb)
}
fn actions(rows: &[CombatPolicyBoundary], step: u64) -> Vec<RunActionIndex> {
    rows.iter().enumerate().map(|(k, b)| RunActionIndex(if b.observation.action_mask.iter().any(|v| *v != 0) { r1(SEEDS[k], step, &b.observation.action_mask) as i64 } else { 0 })).collect()
}
fn tokens(rows: &[CombatPolicyBoundary]) -> Vec<RunDecisionToken> { rows.iter().map(|b| b.token).collect() }
fn same_row(row: &SlotRow<CombatPolicyBoundary>, b: &CombatPolicyBoundary, reward: i32, terminated: bool) {
    assert_eq!((&row.boundary.observation, &row.boundary.info, row.reward, row.terminated, row.truncated), (&b.observation, &b.info, reward, terminated, false));
}
fn fault_of(err: SessionBatchError) -> crate::BatchFault { match err { SessionBatchError::Fault(f) => f, e => panic!("expected a batch fault, got {e:?}") } }

#[test]
fn core_fault_reports_every_slot_freeze() {
    for workers in [1, 3] {
        let (mut batch, rows, mut singles, sb) = started(cfg(workers, AutoresetMode::Disabled, FaultPolicy::Freeze));
        batch.inject(1, TestInject::StepError);
        let a = actions(&rows, 1);
        let fault = fault_of(batch.step(&tokens(&rows), &a).unwrap_err());
        assert_eq!((fault.operation, fault.policy), (BatchOperation::Step, FaultPolicy::Freeze));
        let d: Vec<_> = fault.slots.iter().map(|s| s.delivery).collect();
        assert_eq!(d, [SlotDelivery::CommittedUndelivered { recoverable: true }, SlotDelivery::Faulted { stage: BatchFaultStage::Step, committed: false, gameplay_recorded: false },
            SlotDelivery::CommittedUndelivered { recoverable: true }]);
        assert_eq!(fault.slots[1].error, Some(RunSessionError::DeliveryFault("TEST-ONLY injected step fault")));
        // Frozen: no reset or step until close; reads stay allowed.
        assert_eq!(batch.step(&tokens(&rows), &a), Err(SessionBatchError::Fault(fault.clone())));
        assert_eq!(batch.reset(&[Some(1); 3], &[None, None, None], None), Err(SessionBatchError::Fault(fault.clone())));
        for k in [0, 2] {
            let r = singles[k].step(sb[k].token, a[k]).unwrap();
            assert_eq!(batch.decision(k).unwrap().observation, r.boundary.observation, "healthy peer {k} readable");
        }
        assert!(batch.decision(1).is_err());
        assert_eq!(batch.diagnostics()["status"], "faulted");
        batch.close().unwrap();
        assert!(matches!(batch.step(&tokens(&rows), &a), Err(SessionBatchError::Input { error: RunSessionError::Closed, .. })));
    }
}

#[test]
fn panic_reports_every_slot() {
    for (inject, committed) in [(TestInject::StepPanic, false), (TestInject::StepPanicAfterCommit, true)] {
        let (mut batch, rows, _, _) = started(cfg(2, AutoresetMode::Disabled, FaultPolicy::Freeze));
        batch.inject(2, inject);
        let fault = fault_of(batch.step(&tokens(&rows), &actions(&rows, 1)).unwrap_err());
        assert_eq!(fault.slots[2].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Panic, committed, gameplay_recorded: committed });
        assert_eq!(fault.slots[2].error, Some(RunSessionError::DeliveryFault("native combat session operation panicked")));
        assert!(fault.slots[..2].iter().all(|s| s.delivery == SlotDelivery::CommittedUndelivered { recoverable: true }));
        assert_eq!(batch.session(2).unwrap().diagnostic()["last_step_committed"], committed);
    }
    // A reset panic.
    let mut batch = CombatTrainingBatch::new_combat(catalog(), Partition::Test, 2, None, cfg(2, AutoresetMode::Disabled, FaultPolicy::Freeze)).unwrap();
    batch.inject(0, TestInject::ResetPanic);
    let fault = fault_of(batch.reset(&[Some(1), Some(2)], &[None, None], None).unwrap_err());
    assert_eq!(fault.slots[0].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Panic, committed: false, gameplay_recorded: false });
    assert_eq!(fault.slots[1].delivery, SlotDelivery::CommittedUndelivered { recoverable: true });
}

#[test]
fn committed_results_recoverable_after_fault() {
    let (mut batch, rows, mut singles, sb) = started(cfg(3, AutoresetMode::Disabled, FaultPolicy::Freeze));
    batch.inject(0, TestInject::StepErrorAfterCommit);
    let a = actions(&rows, 1);
    let fault = fault_of(batch.step(&tokens(&rows), &a).unwrap_err());
    assert_eq!(fault.slots[0].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Encode, committed: true, gameplay_recorded: true });
    let recovered = batch.recover_committed().unwrap();
    assert!(recovered[0].is_none(), "the faulted slot's post-commit state has no deliverable row");
    for k in [1, 2] {
        let r = singles[k].step(sb[k].token, a[k]).unwrap();
        let once = batch.committed_result(k).unwrap();
        same_row(&once, &r.boundary, r.reward, r.terminated);
        assert_eq!(batch.committed_result(k).unwrap(), once, "repeatable");
        assert_eq!(recovered[k].as_ref().unwrap(), &once);
        assert_eq!(batch.retained_views(k), 0, "N4: no boundary retained by value for an ordinary step");
    }
    // Nothing re-executed: the healthy peers' sessions advanced exactly once.
    for k in [1, 2] { assert_eq!(batch.session(k).unwrap().diagnostic()["accepted_steps"], 1); }
}

#[test]
fn healthy_peers_readable_after_fault_and_no_retry() {
    let configs: Vec<StandardRunConfig> = [Character::Ironclad, Character::Watcher, Character::Silent].into_iter().map(|c| {
        let mut config = StandardRunConfig::new(c, 0, Partition::Test);
        config.record_replay = true;
        config
    }).collect();
    let mut batch = RunSessionBatch::new_run(configs.clone(), RunSession::new_standard, cfg(3, AutoresetMode::Disabled, FaultPolicy::Freeze)).unwrap();
    let rows: Vec<_> = batch.reset(&[Some(1), Some(2), Some(3)], &[None, None, None], None).unwrap().into_iter().map(|r| r.boundary).collect();
    batch.inject(1, TestInject::StepPanicAfterCommit);
    let a: Vec<_> = rows.iter().enumerate().map(|(k, b)| RunActionIndex(r1(k as u64, 1, &b.observation.action_mask) as i64)).collect();
    fault_of(batch.step(&rows.iter().map(|b| b.token).collect::<Vec<_>>(), &a).unwrap_err());
    for k in 0..3 {
        let record = batch.record(k).unwrap().unwrap();
        assert_eq!(record.steps.len(), 1, "slot {k}: the action ran once (no retry)");
        assert_eq!(record.fault.is_some(), k == 1, "only the faulted slot's record has a fault");
    }
    let f = batch.record(1).unwrap().unwrap().fault.unwrap();
    assert!(f.committed && f.gameplay_recorded);
    for k in [0, 2] { assert!(batch.decision(k).is_ok() && batch.session(k).unwrap().reproduction().unwrap().fault.is_none()); }
    // Retries are refused and change nothing.
    let before: Vec<_> = (0..3).map(|k| batch.record(k).unwrap()).collect();
    assert!(batch.step(&rows.iter().map(|b| b.token).collect::<Vec<_>>(), &a).is_err());
    assert_eq!((0..3).map(|k| batch.record(k).unwrap()).collect::<Vec<_>>(), before);
}

#[test]
fn isolate_policy_fault_row_every_step() {
    for mode in [AutoresetMode::Disabled, AutoresetMode::NextStep, AutoresetMode::SameStep] {
        let (mut batch, mut rows, mut singles, mut sb) = started(cfg(2, mode, FaultPolicy::Isolate));
        batch.inject(1, TestInject::StepError);
        for step in 1..=200u64 {
            let mut a = actions(&rows, step);
            let mut t = tokens(&rows);
            if step > 1 { a[1] = RunActionIndex(12_345); t[1] = RunDecisionToken([0, 0]); } // ignored, not validated
            let out = batch.step(&t, &a).unwrap();
            let f = out[1].fault.as_ref().expect("the fault row repeats every step");
            assert_eq!((f.stage, f.committed), (BatchFaultStage::Step, false));
            assert!(out[1].boundary.observation.action_mask.iter().all(|v| *v == 0) && out[1].boundary.observation.groups.iter().all(|g| g.values.iter().all(|v| *v == 0)));
            assert!(!out[1].terminated && !out[1].truncated && out[1].reward == 0 && !out[1].autoreset && out[1].final_boundary.is_none());
            if mode == AutoresetMode::Disabled {
                for k in [0, 2] { sb[k] = singles[k].step(sb[k].token, a[k]).unwrap().boundary; assert_eq!(out[k].boundary.observation, sb[k].observation, "peer {k} step {step}"); }
            }
            assert!(out[0].fault.is_none() && out[2].fault.is_none(), "peers are never fault rows");
            if mode == AutoresetMode::Disabled && out.iter().any(|r| r.terminated) { break }
            rows = out.into_iter().map(|r| r.boundary).collect();
        }
        assert!(batch.slot_fault(1).is_some() && batch.fault().is_none(), "the batch continues; slot 1 stays isolated");
    }
}

#[test]
fn isolate_reset_of_faulted_slot_requires_seed() {
    let (mut batch, rows, _, _) = started(cfg(2, AutoresetMode::NextStep, FaultPolicy::Isolate));
    batch.inject(2, TestInject::StepError);
    let out = batch.step(&tokens(&rows), &actions(&rows, 1)).unwrap();
    let before: Vec<_> = (0..2).map(|k| batch.decision(k).unwrap()).collect();
    for (seeds, mask) in [([None, None, None], None), ([Some(5), Some(6), None], None), ([None, None, None], Some([false, false, true]))] {
        let err = batch.reset(&seeds, &[None, None, None], mask.as_ref().map(|m| &m[..])).unwrap_err();
        assert_eq!(err, SessionBatchError::Input { slot: Some(2), error: RunSessionError::Input("faulted slot needs an explicit seed") });
        assert_eq!((0..2).map(|k| batch.decision(k).unwrap()).collect::<Vec<_>>(), before, "no effect on any slot");
        assert!(batch.slot_fault(2).is_some());
    }
    // A mask that excludes the faulted slot works and keeps its fault row.
    let kept = batch.reset(&[Some(9), None, None], &[None, None, None], Some(&[true, false, false])).unwrap();
    assert!(kept[2].fault.is_some() && kept[1].boundary == out[1].boundary);
}

#[test]
fn isolate_reset_with_seed_equals_fresh_single() {
    let (mut batch, rows, _, _) = started(cfg(2, AutoresetMode::NextStep, FaultPolicy::Isolate));
    batch.inject(0, TestInject::StepError);
    batch.step(&tokens(&rows), &actions(&rows, 1)).unwrap();
    // Explicit seed, and the scalar `seed + slot` derivation (base 1000 → slot 0 gets 1000).
    for seed in [77u64, 1000] {
        let out = batch.reset(&[Some(seed), None, None], &[None, None, None], Some(&[true, false, false])).unwrap();
        assert!(out[0].fault.is_none() && batch.slot_fault(0).is_none());
        let mut fresh = single();
        let b = fresh.reset(Some(seed), None).unwrap();
        assert_eq!((&out[0].boundary.observation, &out[0].boundary.info), (&b.observation, &b.info), "a fresh single session with that seed and entry");
        assert_eq!(out[0].boundary.info["episode_ordinal"], 0);
        batch.inject(0, TestInject::StepError);
        let cur: Vec<_> = (0..3).map(|k| batch.decision(k).unwrap_or_else(|_| out[k].boundary.clone())).collect();
        batch.step(&tokens(&cur), &actions(&cur, 2)).unwrap();
    }
}

#[test]
fn isolate_old_fault_in_fault_records() {
    let configs: Vec<StandardRunConfig> = [Character::Defect, Character::Ironclad].into_iter().map(|c| { let mut x = StandardRunConfig::new(c, 0, Partition::Test); x.record_replay = true; x }).collect();
    let mut batch = RunSessionBatch::new_run(configs, RunSession::new_standard, cfg(2, AutoresetMode::NextStep, FaultPolicy::Isolate)).unwrap();
    let rows: Vec<_> = batch.reset(&[Some(1), Some(2)], &[None, None], None).unwrap().into_iter().map(|r| r.boundary).collect();
    batch.inject(0, TestInject::StepPanicAfterCommit);
    let out = batch.step(&rows.iter().map(|b| b.token).collect::<Vec<_>>(), &[RunActionIndex(0), RunActionIndex(0)]).unwrap();
    assert_eq!(out[0].fault.as_ref().unwrap().stage, BatchFaultStage::Panic);
    assert!(batch.take_fault_records().is_empty(), "retired only on resume");
    batch.reset(&[Some(3), None], &[None, None], Some(&[true, false])).unwrap();
    let retired = batch.take_fault_records();
    assert_eq!(retired.len(), 1);
    assert_eq!(retired[0].slot, 0);
    assert_eq!(retired[0].error, Some(RunSessionError::DeliveryFault("Run batch session operation panicked")));
    let record = retired[0].record.as_ref().unwrap();
    assert_eq!((record.seed, record.steps.len()), (1, 1));
    assert!(record.fault.as_ref().unwrap().committed);
    assert!(batch.take_completed_records().is_empty(), "a faulted episode is a fault record, never a completed (scored) one");
}

/// Steps slot 0 to its terminal step with `inject` armed on every step (a reset inject fires only on the autoreset).
fn to_terminal_with(config: BatchConfig, inject: Option<TestInject>) -> (CombatTrainingBatch, Vec<SlotRow<CombatPolicyBoundary>>, CombatPolicyBoundary, i32) {
    let (mut batch, mut rows, mut singles, mut sb) = started(config);
    for step in 1..=400u64 {
        if let Some(inject) = inject { batch.inject(0, inject); }
        let a = actions(&rows, step);
        let out = batch.step(&tokens(&rows), &a);
        let r0 = singles[0].step(sb[0].token, a[0]).unwrap();
        for k in 1..3 { if let Ok(s) = singles[k].step(sb[k].token, a[k]) { sb[k] = s.boundary; } }
        if r0.terminated { return (batch, out.map_or_else(|_| vec![], |o| o), r0.boundary, r0.reward) }
        let out = out.unwrap();
        assert!(out.iter().all(|r| r.fault.is_none()), "no fault before slot 0's terminal");
        sb[0] = r0.boundary;
        rows = out.into_iter().map(|r| r.boundary).collect();
    }
    panic!("slot 0 must terminate")
}

#[test]
fn same_step_reset_fault_delivers_terminal_then_fault_row() {
    let (mut batch, out, terminal, reward) = to_terminal_with(cfg(2, AutoresetMode::SameStep, FaultPolicy::Isolate), Some(TestInject::ResetError));
    let row = &out[0];
    assert_eq!((row.reward, row.terminated), (reward, true), "the terminal transition and its reward are delivered");
    assert_eq!(row.final_boundary.as_ref().unwrap().observation, terminal.observation);
    let f = row.fault.as_ref().unwrap();
    assert_eq!((f.stage, f.committed), (BatchFaultStage::Autoreset, true));
    assert!(row.boundary.observation.action_mask.iter().all(|v| *v == 0));
    // Then frozen behind the fault row; never reset again automatically.
    let cur: Vec<_> = out.iter().map(|r| r.boundary.clone()).collect();
    let frozen = batch.session(0).unwrap().diagnostic();
    let next = batch.step(&tokens(&cur), &actions(&cur, 999)).unwrap();
    assert!(next[0].fault.is_some() && next[0].reward == 0 && !next[0].terminated && next[0].final_boundary.is_none());
    assert_eq!(batch.session(0).unwrap().diagnostic(), frozen, "the frozen slot never runs again (no retry, no new episode)");
    assert_eq!(frozen["episode_ordinal"], 0);
}

#[test]
fn same_step_reset_fault_freeze_reports_autoreset_committed() {
    let (batch, out, terminal, reward) = to_terminal_with(cfg(2, AutoresetMode::SameStep, FaultPolicy::Freeze), Some(TestInject::ResetError));
    assert!(out.is_empty(), "freeze: the operation fails");
    let fault = batch.fault().unwrap().clone();
    assert_eq!(fault.slots[0].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Autoreset, committed: true, gameplay_recorded: true });
    let committed = batch.committed_result(0).unwrap();
    assert_eq!((committed.reward, committed.terminated), (reward, true));
    assert_eq!(committed.final_boundary.as_ref().unwrap().observation, terminal.observation);
    assert_eq!(committed.boundary.observation, terminal.observation);
    assert_eq!(batch.retained_views(0), 1, "N4: the autoreset-fault terminal is retained by value, once");
    assert!(batch.recover_committed().unwrap()[0].is_some());
}

#[test]
fn next_step_autoreset_fault_stage_and_policy() {
    for policy in [FaultPolicy::Isolate, FaultPolicy::Freeze] {
        let (mut batch, out, _, _) = to_terminal_with(cfg(2, AutoresetMode::NextStep, policy), None);
        assert!(out[0].terminated && out[0].fault.is_none(), "the terminal row was delivered");
        batch.inject(0, TestInject::ResetError);
        let cur: Vec<_> = out.iter().map(|r| r.boundary.clone()).collect();
        let next = batch.step(&tokens(&cur), &actions(&cur, 998));
        match policy {
            FaultPolicy::Isolate => {
                let f = next.unwrap()[0].fault.clone().unwrap();
                assert_eq!((f.stage, f.committed), (BatchFaultStage::Autoreset, false));
            }
            FaultPolicy::Freeze => {
                let fault = fault_of(next.unwrap_err());
                assert_eq!(fault.slots[0].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Autoreset, committed: false, gameplay_recorded: false });
            }
        }
    }
}

#[test]
fn export_failure_marks_committed_undelivered_and_recovers() {
    for policy in [FaultPolicy::Freeze, FaultPolicy::Isolate] {
        let (mut batch, rows, _, _) = started(cfg(2, AutoresetMode::Disabled, policy));
        let out = batch.step(&tokens(&rows), &actions(&rows, 1)).unwrap();
        let fault = batch.fail_export();
        assert_eq!((fault.operation, fault.policy), (BatchOperation::Export, policy), "export failures are batch-level, never isolated");
        assert!(fault.slots.iter().all(|s| s.delivery == SlotDelivery::CommittedUndelivered { recoverable: true }));
        let recovered = batch.recover_committed().unwrap();
        for k in 0..3 { assert_eq!(recovered[k].as_ref().unwrap(), &out[k], "recovery repackages the committed row"); }
        assert!(matches!(batch.step(&tokens(&rows), &actions(&rows, 2)), Err(SessionBatchError::Fault(_))));
        assert_eq!(batch.fail_export(), fault, "idempotent");
    }
}

#[test]
fn preflight_errors_report_nothing_and_change_nothing() {
    let (mut batch, rows, _, _) = started(cfg(2, AutoresetMode::Disabled, FaultPolicy::Freeze));
    let before: Vec<_> = (0..3).map(|k| batch.decision(k).unwrap()).collect();
    let mut t = tokens(&rows);
    t[2] = RunDecisionToken([t[2].0[0], t[2].0[1] + 1]);
    assert_eq!(batch.step(&t, &actions(&rows, 1)), Err(SessionBatchError::Input { slot: Some(2), error: RunSessionError::StaleToken }));
    let mut a = actions(&rows, 1);
    a[0] = RunActionIndex(9_999);
    assert!(matches!(batch.step(&tokens(&rows), &a), Err(SessionBatchError::Input { slot: Some(0), .. })));
    assert!(matches!(batch.reset(&[Some(1)], &[None], None), Err(SessionBatchError::Input { slot: None, .. })));
    assert!(matches!(batch.reset(&[None; 3], &[Some("missing".into()), None, None], None), Err(SessionBatchError::Input { slot: Some(0), .. })));
    assert_eq!((0..3).map(|k| batch.decision(k).unwrap()).collect::<Vec<_>>(), before);
    assert!(batch.fault().is_none() && batch.last_statuses().iter().all(|s| *s != SlotDelivery::NotCommitted));
}

#[test]
fn capture_thread_local_none_after_worker_panic() {
    let (mut batch, rows, _, _) = started(cfg(3, AutoresetMode::Disabled, FaultPolicy::Isolate));
    batch.inject(1, TestInject::StepPanicAfterCommit);
    batch.step(&tokens(&rows), &actions(&rows, 1)).unwrap();
    let crate::workers::Scheduler::Parallel(pool) = batch.scheduler_for_test() else { panic!("three workers") };
    assert!(pool.broadcast(|_| spire_core::combat::capture_active()).iter().all(|active| !active), "no observer collector survives on a worker");
}

/// Final review c1 finding 1a: a SAME_STEP reset that PANICS after a committed terminal step is the same composite as a
/// reset error: stage `autoreset`, committed, and the terminal transition is delivered (isolate) or recoverable (freeze).
#[test]
fn same_step_reset_panic_is_an_autoreset_composite() {
    for policy in [FaultPolicy::Freeze, FaultPolicy::Isolate] {
        let (batch, out, terminal, reward) = to_terminal_with(cfg(2, AutoresetMode::SameStep, policy), Some(TestInject::ResetPanic));
        match policy {
            FaultPolicy::Isolate => {
                let row = &out[0];
                assert_eq!((row.reward, row.terminated), (reward, true));
                assert_eq!(row.final_boundary.as_ref().unwrap().observation, terminal.observation);
                let f = row.fault.as_ref().unwrap();
                assert_eq!((f.stage, f.committed), (BatchFaultStage::Autoreset, true));
            }
            FaultPolicy::Freeze => {
                let fault = batch.fault().unwrap().clone();
                assert_eq!(fault.slots[0].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Autoreset, committed: true, gameplay_recorded: true });
                let recovered = batch.recover_committed().unwrap()[0].clone().expect("the committed terminal is recoverable");
                assert_eq!((recovered.reward, recovered.terminated), (reward, true));
                assert_eq!(recovered.final_boundary.unwrap().observation, terminal.observation);
            }
        }
    }
}

/// Final review c1 finding 1b: the isolate composite row whose export then fails stays recoverable.
#[test]
fn isolate_composite_then_export_failure_recovers_the_terminal() {
    let (mut batch, out, terminal, reward) = to_terminal_with(cfg(2, AutoresetMode::SameStep, FaultPolicy::Isolate), Some(TestInject::ResetError));
    assert!(out[0].fault.is_some());
    let fault = batch.fail_export();
    assert_eq!(fault.slots[0].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Autoreset, committed: true, gameplay_recorded: true });
    let recovered = batch.recover_committed().unwrap()[0].clone().expect("recoverable as reported");
    assert_eq!((recovered.reward, recovered.terminated), (reward, true));
    assert_eq!(recovered.final_boundary.unwrap().observation, terminal.observation);
    assert_eq!(batch.committed_result(0).unwrap().reward, reward, "repeatable");
}

/// Final review c1 finding 6: a panic outside every per-slot guard (here after the join) freezes the batch under either
/// policy, with each slot's status from its publication serial; nothing is left uncertain or retried.
#[test]
fn panic_outside_slot_jobs_freezes_with_serial_statuses() {
    for policy in [FaultPolicy::Freeze, FaultPolicy::Isolate] {
        let (mut batch, rows, _, _) = started(cfg(2, AutoresetMode::Disabled, policy));
        batch.inject(1, TestInject::PostJoinPanic);
        let fault = fault_of(batch.step(&tokens(&rows), &actions(&rows, 1)).unwrap_err());
        assert_eq!((fault.operation, fault.policy), (BatchOperation::Step, policy));
        assert!(fault.slots.iter().all(|s| s.delivery == SlotDelivery::CommittedUndelivered { recoverable: false }), "{:?}", fault.slots);
        for k in 0..3 { assert!(batch.committed_result(k).is_err(), "slot {k}: no stale retained result survives the panic freeze"); }
        assert_eq!(batch.step(&tokens(&rows), &actions(&rows, 2)), Err(SessionBatchError::Fault(fault.clone())));
        assert_eq!(batch.diagnostics()["status"], "faulted");
    }
}

/// Final review c1 finding 12: `combat_views()` is a read that stays allowed after a freeze; the faulted slot is a `None`
/// row and healthy peers keep their scalar views.
#[test]
fn combat_views_after_a_freeze_skip_the_faulted_slot() {
    let configs: Vec<StandardRunConfig> = [Character::Silent, Character::Watcher].into_iter().map(|c| StandardRunConfig::new(c, 0, Partition::Test)).collect();
    let mut batch = RunSessionBatch::new_run(configs, RunSession::new_standard, cfg(1, AutoresetMode::Disabled, FaultPolicy::Freeze)).unwrap();
    let rows: Vec<_> = batch.reset(&[Some(4), Some(5)], &[None, None], None).unwrap().into_iter().map(|r| r.boundary).collect();
    batch.inject(1, TestInject::StepError);
    assert!(batch.step(&rows.iter().map(|b| b.token).collect::<Vec<_>>(), &[RunActionIndex(0), RunActionIndex(0)]).is_err());
    let (views, in_combat) = batch.combat_views().unwrap();
    assert!(views[1].is_none() && !in_combat[1]);
    assert_eq!(views[0].as_ref().unwrap(), &batch.sessions[0].combat_view().unwrap());
}

/// Recheck c2 N2: in a panic freeze, an already-isolated slot (skipped by the operation) is `unaffected`.
#[test]
fn panic_freeze_reports_isolated_slots_unaffected() {
    let (mut batch, rows, _, _) = started(cfg(1, AutoresetMode::Disabled, FaultPolicy::Isolate));
    batch.inject(2, TestInject::StepError);
    let out = batch.step(&tokens(&rows), &actions(&rows, 1)).unwrap();
    assert!(out[2].fault.is_some());
    let cur: Vec<_> = out.into_iter().map(|r| r.boundary).collect();
    batch.inject(0, TestInject::PostJoinPanic);
    let fault = fault_of(batch.step(&tokens(&cur), &actions(&cur, 2)).unwrap_err());
    let d: Vec<_> = fault.slots.iter().map(|s| s.delivery).collect();
    assert_eq!(d, [SlotDelivery::CommittedUndelivered { recoverable: false }, SlotDelivery::CommittedUndelivered { recoverable: false }, SlotDelivery::Unaffected]);
}

/// PM final review M1: a reset that faults after the slot's finished episode was queued as completed must not file that
/// same episode again as a fault record on the later isolate resume; each episode lands in exactly one queue.
#[test]
fn isolate_resume_files_each_finished_episode_once() {
    for reset_kind in [TestInject::ResetError, TestInject::ResetPanic] {
        let configs: Vec<StandardRunConfig> = [Character::Ironclad, Character::Silent].into_iter()
            .map(|c| { let mut x = StandardRunConfig::new(c, 0, Partition::Test); x.record_replay = true; x }).collect();
        let mut batch = RunSessionBatch::new_run(configs, RunSession::new_standard, cfg(1, AutoresetMode::NextStep, FaultPolicy::Isolate)).unwrap();
        let rows: Vec<_> = batch.reset(&[Some(1), Some(2)], &[None, None], None).unwrap().into_iter().map(|r| r.boundary).collect();
        let a: Vec<_> = rows.iter().map(|b| RunActionIndex(b.observation.action_mask.iter().position(|v| *v != 0).unwrap() as i64)).collect();
        batch.step(&rows.iter().map(|b| b.token).collect::<Vec<_>>(), &a).unwrap();
        batch.inject(0, reset_kind);
        let out = batch.reset(&[Some(5), None], &[None, None], Some(&[true, false])).unwrap();
        assert!(out[0].fault.is_some(), "the masked reset of slot 0 faulted");
        batch.reset(&[Some(6), None], &[None, None], Some(&[true, false])).unwrap();
        let completed = batch.take_completed_records();
        let retired = batch.take_fault_records();
        assert_eq!(completed.len(), 1, "{reset_kind:?}: the finished episode is completed once");
        assert_eq!((completed[0].slot, completed[0].record.seed, completed[0].record.steps.len()), (0, 1, 1));
        assert!(completed[0].record.fault.is_none(), "the finished episode carries no reset fault");
        assert_eq!(retired.len(), 1, "{reset_kind:?}: the reset fault is still reported");
        assert!(retired[0].error.is_some());
        assert!(retired[0].record.as_ref().is_none_or(|r| (r.seed, r.ordinal) != (completed[0].record.seed, completed[0].record.ordinal)),
            "{reset_kind:?}: the same episode is not filed in both queues");
    }
}
