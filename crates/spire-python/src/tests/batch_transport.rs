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

/// M37b: the parallel per-slot stacking (`stack_slots`) gives `stack`'s tensors exactly, for owned and shared rows, run
/// inline, on scoped threads or on a batch's own workers, after steps that change occupancy; and it rejects the same
/// mismatched layouts with the same messages.
#[test]
fn m37b_slot_stacking_matches_serial_stacking() {
    fn serial(jobs: Vec<stacking::SlotCopy<'_>>) -> Result<(), ()> { jobs.into_iter().for_each(stacking::SlotCopy::run); Ok(()) }
    fn threads(jobs: Vec<stacking::SlotCopy<'_>>) -> Result<(), ()> {
        std::thread::scope(|scope| { for job in jobs { scope.spawn(move || job.run()); } });
        Ok(())
    }
    let (mut batch, mut rows) = standard_batch(8);
    for round in 0..6 {
        let observations = rows.iter().map(|r| &r.boundary.observation).collect::<Vec<_>>();
        let expected = stacking::stack(&observations).unwrap();
        let owned = || rows.iter().map(|r| stacking::SlotSource::Owned(r.boundary.observation.clone())).collect::<Vec<_>>();
        assert_eq!(stacking::stack_slots(owned(), serial).unwrap(), expected, "round {round} inline");
        assert_eq!(stacking::stack_slots(owned(), threads).unwrap(), expected, "round {round} threads");
        assert_eq!(stacking::stack_slots(owned(), |jobs| batch.map_jobs(jobs, stacking::SlotCopy::run).map(|_| ())).unwrap(), expected,
            "round {round} batch workers");
        let zero = <RunSession as spire_env::BatchSession>::batch_zeroed(&rows[0].boundary).unwrap();
        let mixed = rows.iter().enumerate().map(|(slot, r)| if slot % 3 == 1 { stacking::SlotSource::Shared(&zero.observation) }
            else { stacking::SlotSource::Owned(r.boundary.observation.clone()) }).collect::<Vec<_>>();
        let mixed_views = rows.iter().enumerate().map(|(slot, r)| if slot % 3 == 1 { &zero.observation } else { &r.boundary.observation })
            .collect::<Vec<_>>();
        assert_eq!(stacking::stack_slots(mixed, threads).unwrap(), stacking::stack(&mixed_views).unwrap(), "round {round} shared rows");
        let tokens = rows.iter().map(|r| r.boundary.token).collect::<Vec<_>>();
        let actions = rows.iter().map(first_legal).collect::<Vec<_>>();
        rows = batch.step(&tokens, &actions).unwrap();
    }
    let first = rows[0].boundary.observation.clone();
    let check = |other: RunObservation| {
        let views = [&first, &other];
        let expected = stacking::stack(&views).unwrap_err();
        let got = stacking::stack_slots(vec![stacking::SlotSource::Shared(&first), stacking::SlotSource::Owned(other)], serial).unwrap_err();
        assert_eq!(got, expected);
    };
    let mut shape = first.clone(); shape.groups[0].shape.push(1); check(shape);
    let mut short = first.clone(); short.action_mask.pop(); check(short);
    let mut fewer = first.clone(); fewer.groups.pop(); check(fewer);
    let mut occupancy = first.clone();
    let index = occupancy.groups.iter().position(|g| g.occupied.is_some()).unwrap();
    occupancy.groups[index].occupied = None; check(occupancy);
    assert_eq!(stacking::stack_slots(Vec::new(), serial).unwrap_err(), stacking::stack(&[]).unwrap_err());
    // A runner that reports success without running every job is rejected before any column length is set.
    let skip = |jobs: Vec<stacking::SlotCopy<'_>>| -> Result<(), ()> { jobs.into_iter().skip(1).for_each(stacking::SlotCopy::run); Ok(()) };
    let rows = [stacking::SlotSource::Shared(&first), stacking::SlotSource::Shared(&first)];
    assert_eq!(stacking::stack_slots(rows.into_iter().collect(), skip).unwrap_err(), "observation stacking did not copy every slot");
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
    let calls = [".batch.reset(", ".batch.step(", ".batch.step_into(", ".batch.recover_committed()", ".batch.close()", ".batch.combat_views()",
        "RunSessionBatch::new_run(", "CombatTrainingBatch::new_combat("];
    for call in calls {
        let found = lines.iter().enumerate().filter(|(_, l)| l.contains(call)).collect::<Vec<_>>();
        assert!(!found.is_empty(), "{call} is used");
        for (index, _) in found {
            // M38b: the call must sit inside the argument of the nearest `py.detach(` before it in the same function: from
            // that `py.detach(` to the call, more parentheses are opened than closed (the detach call is still open).
            let start = lines[..=index].iter().rposition(|l| l.contains("py.detach("));
            let function = lines[..=index].iter().rposition(|l| l.trim_start().starts_with("fn ") || l.contains(" fn "));
            let detached = start.filter(|s| function.is_none_or(|f| f <= *s)).is_some_and(|s| {
                let mut text = lines[s..=index].join("\n");
                text.drain(..text.find("py.detach(").unwrap());
                let text = &text[..text.rfind(call).unwrap()];
                text.matches('(').count() > text.matches(')').count()
            });
            assert!(detached, "{call} at batch.rs:{} runs with the GIL held", index + 1);
        }
    }
    let run_env = include_str!("../run_env.rs");
    let step_many = run_env.lines().find(|l| l.contains("search.step_many(")).expect("step_many binding");
    assert!(step_many.contains("py.detach("), "step_many runs with the GIL held");
}

/// M38b fixes 1 and 3(a): a step encoded straight into pooled stacked columns (`SessionBatch::step_into` + `Slab`)
/// packages exactly what the row path (`SessionBatch::step` + `stack_slots`) packages, call after call: same tensors,
/// keys, shapes and bytes, same rows, statuses, faults and finals. Two identical batches run side by side over four
/// characters at A0 and A20, with NEXT_STEP and SAME_STEP autoreset, both fault policies, capacity faults (encode stage,
/// after commit), recording on and off, and 1 or 4 workers. Returned columns are scribbled on before they go back to the
/// pool, so reused memory starts dirty.
#[test]
fn m38b_slab_step_matches_row_stacking() {
    use spire_env::{AutoresetMode, FaultPolicy, RunCapacities, SessionBatchError};
    let characters = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
    let cases = [
        (8, 1, AutoresetMode::NextStep, FaultPolicy::Freeze, None, false, 120),
        (8, 4, AutoresetMode::SameStep, FaultPolicy::Isolate, None, true, 160),
        (8, 4, AutoresetMode::NextStep, FaultPolicy::Isolate, Some(6), false, 60),
        (8, 1, AutoresetMode::SameStep, FaultPolicy::Isolate, Some(8), true, 60),
        (6, 4, AutoresetMode::NextStep, FaultPolicy::Freeze, Some(6), false, 60),
    ];
    let pool = stacking::ColumnPool::default();
    let (mut encoded_rows, mut copied_rows, mut finals, mut faults) = (0, 0, 0, 0);
    for (case, &(count, workers, autoreset, policy, actions, record, steps)) in cases.iter().enumerate() {
        let make = || {
            let configs = (0..count).map(|slot| {
                let mut c = StandardRunConfig::new(characters[slot % 4], if slot % 2 == 0 { 0 } else { 20 }, Partition::Test);
                if let Some(actions) = actions { c.capacities = RunCapacities { actions, ..c.capacities.clone() }; }
                c.record_replay = record;
                c
            }).collect::<Vec<_>>();
            let mut batch = RunSessionBatch::new_run(configs, RunSession::new_standard, BatchConfig { workers, autoreset, fault_policy: policy }).unwrap();
            let rows = batch.reset(&(0..count as u64).map(|s| Some(7_000 + s)).collect::<Vec<_>>(), &vec![None; count], None).unwrap();
            (batch, rows)
        };
        let ((mut old, mut rows), (mut new, new_rows)) = (make(), make());
        // Session ids (the first token word) differ between the two batches; serials agree.
        let mut new_tokens = new_rows.iter().map(|r| r.boundary.token).collect::<Vec<_>>();
        for step in 0..steps {
            let tokens = rows.iter().map(|r| r.boundary.token).collect::<Vec<_>>();
            let actions = rows.iter().enumerate().map(|(slot, r)| {
                let legal = r.boundary.observation.action_mask.iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, _)| i).collect::<Vec<_>>();
                RunActionIndex(legal.get((step * 7 + slot * 3) % legal.len().max(1)).copied().unwrap_or(0) as i64)
            }).collect::<Vec<_>>();
            let expected = old.step(&tokens, &actions);
            let layout = new.zero_boundary().unwrap().observation.clone();
            let mut slab = stacking::Slab::new(&layout, count, &pool).unwrap();
            let got = new.step_into(&new_tokens, &actions, slab.targets());
            let (mut expected, mut got) = match (expected, got) {
                (Ok(e), Ok(g)) => (e, g),
                (Err(e), Err(g)) => {
                    assert_eq!(format!("{g:?}"), format!("{e:?}"), "case {case} step {step}: same failure");
                    if let SessionBatchError::Fault(_) = e {
                    faults += 1;
                    let strip = |rows: Vec<Option<SlotRow<RunBoundary>>>| rows.into_iter().map(|r| r.map(|mut r| { r.boundary.token = spire_env::RunDecisionToken::from([0, <[u64; 2]>::from(r.boundary.token)[1]]); r })).collect::<Vec<_>>();
                    assert_eq!(strip(new.recover_committed().unwrap()), strip(old.recover_committed().unwrap()), "case {case}: recovery");
                }
                    break
                }
                (e, g) => panic!("case {case} step {step}: {e:?} vs {g:?}"),
            };
            let fill = got.iter_mut().map(|r| if r.fault.is_none() && r.boundary.observation.groups.is_empty() { encoded_rows += 1; None }
                else { copied_rows += 1; Some(stacking::SlotSource::Owned(std::mem::replace(&mut r.boundary.observation, RunObservation { groups: vec![], action_mask: vec![] }))) })
                .collect::<Vec<_>>();
            let columns = slab.finish(fill, |jobs| new.map_jobs(jobs, stacking::SlotCopy::run).map(|_| ())).unwrap();
            let sources = expected.iter_mut().map(|r| stacking::SlotSource::Owned(r.boundary.observation.clone())).collect::<Vec<_>>();
            assert_eq!(columns, stacking::stack_slots(sources, |jobs| old.map_jobs(jobs, stacking::SlotCopy::run).map(|_| ())).unwrap(),
                "case {case} step {step}: stacked obs");
            for (slot, (e, g)) in expected.iter().zip(&got).enumerate() {
                let serial = |t: spire_env::RunDecisionToken| <[u64; 2]>::from(t)[1];
                assert_eq!((serial(e.boundary.token), e.boundary.terminal, e.reward, &e.reward_events, e.terminated, e.truncated, e.autoreset, &e.fault),
                    (serial(g.boundary.token), g.boundary.terminal, g.reward, &g.reward_events, g.terminated, g.truncated, g.autoreset, &g.fault),
                    "case {case} step {step} slot {slot}: row");
                assert_eq!(e.final_boundary.as_ref().map(|b| (&b.observation, b.terminal, serial(b.token))),
                    g.final_boundary.as_ref().map(|b| (&b.observation, b.terminal, serial(b.token))), "case {case} step {step} slot {slot}: owned final boundary");
                faults += usize::from(g.fault.is_some());
                finals += usize::from(g.final_boundary.is_some());
            }
            assert_eq!(old.last_statuses(), new.last_statuses());
            for c in columns {
                pool.put(match c.column { Column::I64(mut v) => { v.fill(0x5555); Column::I64(v) } Column::I8(mut v) => { v.fill(0x55); Column::I8(v) } });
            }
            new_tokens = got.iter().map(|r| r.boundary.token).collect();
            rows = expected;
        }
    }
    assert!(pool.pooled_bytes() > 0, "returned columns are pooled");
    assert!(encoded_rows > 1_000 && copied_rows > 0 && finals > 0 && faults > 0,
        "paths exercised: encoded {encoded_rows}, copied {copied_rows}, finals {finals}, faults {faults}");
}
