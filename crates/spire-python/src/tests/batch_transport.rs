//! M35 task 6 binding seams, tested without the extension feature: row stacking, per-slot status fields, recovery
//! repackaging and the GIL-detach structure of the batch transport.
use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{BatchConfig, BatchFaultStage, RunActionIndex, RunBoundary, RunObservation, RunSession, RunSessionBatch,
    RunSessionError, SlotDelivery, SlotReport, SlotRow, StandardRunConfig};
use crate::stacking::{self, Column};

fn standard_batch(count: usize) -> (RunSessionBatch, Vec<SlotRow<RunBoundary>>) {
    let characters = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
    let configs = (0..count).map(|slot| StandardRunConfig::new(characters[slot % 4], if slot % 2 == 0 { 0 } else { 20 }, Partition::Test))
        .collect::<Vec<_>>();
    let mut batch = RunSessionBatch::new_run(configs, RunSession::new_standard, BatchConfig::new(1)).unwrap();
    let seeds = (0..count as u64).map(|s| Some(9_400 + s)).collect::<Vec<_>>();
    let rows = batch.reset(&seeds, &vec![None; count], None).unwrap();
    (batch, rows)
}
fn first_legal(row: &SlotRow<RunBoundary>) -> RunActionIndex {
    RunActionIndex(row.boundary.observation.action_mask.iter().position(|v| *v != 0).unwrap() as i64)
}
/// The scalar observation dict's key order (`run_env::observation`).
fn scalar_keys(o: &RunObservation) -> Vec<String> {
    let mut keys = Vec::new();
    for g in &o.groups {
        keys.push(g.key.to_owned());
        keys.push(format!("{}_known", g.key));
        if g.occupied.is_some() { keys.push(format!("{}_occupied", g.key)); }
    }
    keys.push("action_mask".to_owned());
    keys
}
/// Slot `slot`'s slice of every stacked column, as (key, i64 values).
fn slot_slice(columns: &[stacking::Stacked], slot: usize) -> Vec<(String, Vec<i64>)> {
    columns.iter().map(|c| {
        let width = c.shape[1..].iter().product::<usize>();
        let values = match &c.column {
            Column::I64(v) => v[slot * width..(slot + 1) * width].to_vec(),
            Column::I8(v) => v[slot * width..(slot + 1) * width].iter().map(|x| i64::from(*x)).collect(),
        };
        (c.key.clone(), values)
    }).collect()
}
fn scalar_slice(o: &RunObservation) -> Vec<(String, Vec<i64>)> {
    let mut out = Vec::new();
    for g in &o.groups {
        out.push((g.key.to_owned(), g.values.clone()));
        out.push((format!("{}_known", g.key), g.known.iter().map(|x| i64::from(*x)).collect()));
        if let Some(occ) = &g.occupied { out.push((format!("{}_occupied", g.key), occ.iter().map(|x| i64::from(*x)).collect())); }
    }
    out.push(("action_mask".to_owned(), o.action_mask.iter().map(|x| i64::from(*x)).collect()));
    out
}

#[test]
fn stacking_matches_per_slot_observation_rows() {
    let (_, rows) = standard_batch(4);
    let observations = rows.iter().map(|r| &r.boundary.observation).collect::<Vec<_>>();
    let columns = stacking::stack(&observations).unwrap();
    assert_eq!(columns.iter().map(|c| c.key.clone()).collect::<Vec<_>>(), scalar_keys(observations[0]));
    for c in &columns {
        assert_eq!(c.shape[0], 4, "leading batch dimension on {}", c.key);
        let len = match &c.column { Column::I64(v) => v.len(), Column::I8(v) => v.len() };
        assert_eq!(len, c.shape.iter().product::<usize>(), "{} is C-contiguous [B, ...]", c.key);
    }
    for (slot, o) in observations.iter().enumerate() { assert_eq!(slot_slice(&columns, slot), scalar_slice(o), "slot {slot}"); }
}

#[test]
fn stacking_rejects_mismatched_layouts() {
    let (_, rows) = standard_batch(2);
    let mut other = rows[1].boundary.observation.clone();
    other.groups[0].shape.push(1);
    assert!(stacking::stack(&[&rows[0].boundary.observation, &other]).is_err());
    let mut short = rows[1].boundary.observation.clone();
    short.action_mask.pop();
    assert!(stacking::stack(&[&rows[0].boundary.observation, &short]).is_err());
    assert!(stacking::stack(&[]).is_err());
}

#[test]
fn batch_fault_error_attributes() {
    let report = |delivery, error| SlotReport { delivery, error };
    let s = stacking::slot_status(&report(SlotDelivery::CommittedUndelivered { recoverable: true }, None), false);
    assert_eq!((s.status, s.stage, s.committed, s.recoverable, s.message), ("committed_undelivered", None, Some(true), Some(true), None));
    let s = stacking::slot_status(&report(SlotDelivery::NotCommitted, None), false);
    assert_eq!((s.status, s.committed, s.recoverable), ("not_committed", Some(false), None));
    let s = stacking::slot_status(&report(SlotDelivery::Faulted { stage: BatchFaultStage::Encode, committed: true, gameplay_recorded: true },
        Some(RunSessionError::Capacity { group: "hand", required: 11, capacity: 10 })), false);
    assert_eq!((s.status, s.stage, s.committed, s.recoverable), ("faulted", Some("encode"), Some(true), Some(false)));
    assert_eq!(s.message.as_deref(), Some("Run capacity hand: 11 > 10"));
    let s = stacking::slot_status(&report(SlotDelivery::Faulted { stage: BatchFaultStage::Autoreset, committed: true, gameplay_recorded: true }, None), true);
    assert_eq!((s.stage, s.recoverable), (Some("autoreset"), Some(true)), "a committed SAME_STEP terminal is recoverable");
    for (delivery, label) in [(SlotDelivery::Delivered, "delivered"), (SlotDelivery::Unaffected, "unaffected")] {
        let s = stacking::slot_status(&report(delivery, None), false);
        assert_eq!((s.status, s.stage, s.committed, s.recoverable), (label, None, None, None));
    }
}

#[test]
fn core_fault_messages_stay_redacted() {
    let core = RunSessionError::Core(spire_core::run::RunError::InvalidAction);
    assert_eq!(stacking::error_message(&core, false), "native Run fault; inspect privileged reproduction");
    assert_eq!(stacking::error_message(&core, true), "native combat fault; inspect privileged reproduction");
}

#[test]
fn recovery_repackages_the_retained_results() {
    let (mut batch, rows) = standard_batch(3);
    let tokens = rows.iter().map(|r| r.boundary.token).collect::<Vec<_>>();
    let indices = rows.iter().map(first_legal).collect::<Vec<_>>();
    let stepped = batch.step(&tokens, &indices).unwrap();
    let fault = batch.fail_export();
    assert!(fault.slots.iter().all(|s| s.delivery == SlotDelivery::CommittedUndelivered { recoverable: true }));
    for _ in 0..2 {
        let recovered = batch.recover_committed().unwrap().into_iter().map(Option::unwrap).collect::<Vec<_>>();
        assert_eq!(recovered, stepped, "recovery is a read of owned results, repeatable");
        let a = stacking::stack(&recovered.iter().map(|r| &r.boundary.observation).collect::<Vec<_>>()).unwrap();
        let b = stacking::stack(&stepped.iter().map(|r| &r.boundary.observation).collect::<Vec<_>>()).unwrap();
        assert_eq!(a, b);
    }
}

/// Every native batch operation in the transport runs inside `py.detach` (the GIL is released for native work).
#[test]
fn gil_is_detached_for_native_batch_work() {
    let source = include_str!("../batch.rs");
    let lines = source.lines().collect::<Vec<_>>();
    let calls = [".batch.reset(", ".batch.step(", ".batch.recover_committed()", ".batch.close()", ".batch.combat_views()", "RunSessionBatch::new_run(",
        "CombatTrainingBatch::new_combat("];
    for call in calls {
        let found = lines.iter().enumerate().filter(|(_, l)| l.contains(call)).collect::<Vec<_>>();
        assert!(!found.is_empty(), "{call} is used");
        for (index, _) in found {
            let window = &lines[index.saturating_sub(2)..=index];
            assert!(window.iter().any(|l| l.contains("py.detach(")), "{call} at batch.rs:{} runs with the GIL held", index + 1);
        }
    }
    let run_env = include_str!("../run_env.rs");
    let step_many = run_env.lines().find(|l| l.contains("search.step_many(")).expect("step_many binding");
    assert!(step_many.contains("py.detach("), "step_many runs with the GIL held");
}
