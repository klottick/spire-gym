//! M38b fix 4: the batch's home-range scheduling (`Scheduler::map_mut_sticky`). Jobs run once each, results keep slot
//! order, a panic still reaches the caller, idle workers take unclaimed slots, and batch results are the same at 1, 4 and
//! 16 workers (rows, statuses, faults and finals), with uneven slot ranges included.
use crate::workers::Scheduler;
use crate::{AutoresetMode, BatchConfig, FaultPolicy, RunActionIndex, RunCapacities, RunSession, RunSessionBatch, StandardRunConfig};
use spire_core::{combat::rng::Partition, content::Character};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn sticky_scheduler_keeps_order_and_runs_each_job_once() {
    for (workers, count) in [(4, 4), (4, 7), (3, 16), (16, 16)] {
        let scheduler = Scheduler::new(workers, count).unwrap();
        let mut values = (0..count).map(|i| i * 10).collect::<Vec<_>>();
        for _ in 0..3 {
            let out = scheduler.map_mut_sticky(&mut values, (0..count).collect(), |i, v, j| {
                assert_eq!(i, j, "job index matches its slot");
                *v += 1;
                (j, rayon::current_thread_index())
            }).unwrap();
            assert_eq!(out.iter().map(|o| o.0).collect::<Vec<_>>(), (0..count).collect::<Vec<_>>(), "slot order");
            assert!(out.iter().all(|o| o.1.is_some()), "runs on pool workers");
        }
        assert_eq!(values, (0..count).map(|i| i * 10 + 3).collect::<Vec<_>>(), "every job ran exactly once per call");
        // One slow job: the other workers take the rest of its range, and every slot still runs once, in order.
        let slow = scheduler.map_mut_sticky(&mut values, (0..count).collect(), |i, v, j| {
            if i == 0 { std::thread::sleep(std::time::Duration::from_millis(50)) }
            *v += 1;
            (j, rayon::current_thread_index())
        }).unwrap();
        assert_eq!(slow.iter().map(|o| o.0).collect::<Vec<_>>(), (0..count).collect::<Vec<_>>());
        if workers > 1 && count / workers > 1 {
            assert!(slow[1..count / workers].iter().any(|o| o.1 != slow[0].1), "workers {workers}, slots {count}: an idle worker took part of the slow range");
        }
        let panicked = catch_unwind(AssertUnwindSafe(|| scheduler.map_mut_sticky(&mut values, (0..count).collect(), |i, _, _| {
            if i == count - 1 { panic!("TEST-ONLY sticky job panic") }
        })));
        assert!(panicked.is_err(), "a job panic propagates to the caller");
    }
    let serial = Scheduler::new(1, 8).unwrap();
    let mut values = vec![0; 8];
    let out = serial.map_mut_sticky(&mut values, (0..8).collect(), |i, _, j| (i, j, rayon::current_thread_index())).unwrap();
    assert!(out.iter().enumerate().all(|(k, o)| o.0 == k && o.1 == k && o.2.is_none()), "one worker stays inline");
}

#[test]
fn sticky_batch_results_do_not_depend_on_worker_count() {
    let characters = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
    let cases = [
        (16, AutoresetMode::NextStep, FaultPolicy::Isolate, None, 90),
        (7, AutoresetMode::SameStep, FaultPolicy::Isolate, Some(8), 90),
        (16, AutoresetMode::SameStep, FaultPolicy::Freeze, None, 90),
    ];
    for (case, &(count, autoreset, fault_policy, actions, steps)) in cases.iter().enumerate() {
        let mut traces = Vec::new();
        for workers in [1, 4, 16] {
            let configs = (0..count).map(|slot| {
                let mut c = StandardRunConfig::new(characters[slot % 4], if slot % 2 == 0 { 0 } else { 20 }, Partition::Test);
                if let Some(actions) = actions { c.capacities = RunCapacities { actions, ..c.capacities.clone() }; }
                c
            }).collect::<Vec<_>>();
            let mut batch = RunSessionBatch::new_run(configs, RunSession::new_standard, BatchConfig { workers, autoreset, fault_policy }).unwrap();
            let mut rows = batch.reset(&(0..count as u64).map(|s| Some(31_000 + s)).collect::<Vec<_>>(), &vec![None; count], None).unwrap();
            let mut trace = Vec::new();
            for step in 0..steps {
                let tokens = rows.iter().map(|r| r.boundary.token).collect::<Vec<_>>();
                let picks = rows.iter().enumerate().map(|(slot, r)| {
                    let legal = r.boundary.observation.action_mask.iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, _)| i).collect::<Vec<_>>();
                    RunActionIndex(legal.get((step * 5 + slot) % legal.len().max(1)).copied().unwrap_or(0) as i64)
                }).collect::<Vec<_>>();
                match batch.step(&tokens, &picks) {
                    Ok(next) => {
                        // Session ids (token word 0) are per batch; serials and everything else must agree.
                        trace.push(format!("{:?}", next.iter().map(|r| (<[u64; 2]>::from(r.boundary.token)[1], &r.boundary.observation,
                            r.boundary.terminal, r.reward, &r.reward_events, r.terminated, r.autoreset, &r.fault,
                            r.final_boundary.as_ref().map(|b| (&b.observation, b.terminal)))).collect::<Vec<_>>()));
                        trace.push(format!("{:?}", batch.last_statuses()));
                        rows = next;
                    }
                    Err(error) => { trace.push(format!("{error:?}")); break }
                }
            }
            traces.push((workers, trace));
        }
        let (_, first) = &traces[0];
        for (workers, trace) in &traces[1..] { assert!(trace == first, "case {case}: workers {workers} differ from workers 1"); }
    }
}
