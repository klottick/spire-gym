//! M34c measurements (reported, never gated; run with `--ignored --nocapture`): exact checkpoint time and heap size at a
//! late-Act-3 Standard state (the Act 3 boss), and replay-to-decision (`from_record`) cost per accepted step.
use super::standard_traversal::{context, current_node_is_boss, drive, short_creation, simple_action, standard_session, CHARACTERS};
use crate::{RunActionIndex, RunCapacities, RunSession, StandardRunConfig};
use spire_core::combat::rng::Partition;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::time::Instant;

/// Test-binary allocator: the system allocator plus a per-thread net byte count while a measurement is armed.
struct Counting;
thread_local! {
    static ARMED: Cell<bool> = const { Cell::new(false) };
    static NET: Cell<isize> = const { Cell::new(0) };
}
fn track(delta: isize) {
    let _ = ARMED.try_with(|armed| if armed.get() { let _ = NET.try_with(|net| net.set(net.get() + delta)); });
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { track(layout.size() as isize); unsafe { System.alloc(layout) } }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 { track(layout.size() as isize); unsafe { System.alloc_zeroed(layout) } }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) { track(-(layout.size() as isize)); unsafe { System.dealloc(ptr, layout) } }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        track(new_size as isize - layout.size() as isize);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Net heap bytes still held by the value `make` returns (also the M34s-b search retained-bytes check).
pub(super) fn retained_bytes<T>(make: impl FnOnce() -> T) -> (T, isize) {
    NET.with(|net| net.set(0));
    ARMED.with(|armed| armed.set(true));
    let value = make();
    ARMED.with(|armed| armed.set(false));
    (value, NET.with(Cell::get))
}

fn median_micros(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

#[test]
#[ignore = "M34c measurement; reported, not gated"]
fn measure_exact_checkpoint_and_from_record_cost() {
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0, 20] {
            let seed = 0xB0 + index as u8 + if ascension == 20 { 8 } else { 0 };
            let (mut session, _) = standard_session(character, ascension, seed, short_creation());
            // Late Act 3: the first Act 3 boss combat (the largest Act 3 state: full map, deck and relics).
            let late_act3 = |obs: &crate::RunObservation| context(obs).0 == 3 && current_node_is_boss(obs);
            drive(&mut session, 12_000, late_act3, simple_action).unwrap_or_else(|f| panic!("{character:?} A{ascension}: {f:?}"));
            let recorded = session.record().unwrap().unwrap().steps.len();
            let (act, floor, room) = context(&session.decision().unwrap().observation);
            let (checkpoint, bytes) = retained_bytes(|| session.exact_checkpoint().unwrap());
            let take = median_micros((0..31).map(|_| { let t = Instant::now(); let c = session.exact_checkpoint().unwrap(); let e = t.elapsed(); drop(c); e.as_secs_f64() * 1e6 }).collect());
            let fork = median_micros((0..31).map(|_| { let t = Instant::now(); let s = RunSession::exact_fork(&checkpoint).unwrap(); let e = t.elapsed(); drop(s); e.as_secs_f64() * 1e6 }).collect());
            let restore = median_micros((0..31).map(|_| { let t = Instant::now(); session.exact_restore(&checkpoint).unwrap(); t.elapsed().as_secs_f64() * 1e6 }).collect());
            eprintln!("M34C-CHECKPOINT {character:?} A{ascension}: act {act} floor {floor} room {room}, {} recorded steps, checkpoint {bytes} heap bytes, take {take:.1} us, fork {fork:.1} us, restore {restore:.1} us (median of 31)", recorded);
        }
    }
    // Replay to a decision from public seeded rollouts (records of supplied starts are test-only, so a public random
    // rollout to a terminal or 3,000 steps; deterministic mixed policy over the legal mask).
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        let mut config = StandardRunConfig::new(character, 0, Partition::Test);
        config.record_replay = true;
        let mut session = RunSession::new_standard(config).unwrap();
        let mut boundary = session.reset(Some(900 + index as u64)).unwrap();
        for step in 0..3_000 {
            if boundary.terminal.is_some() { break; }
            let legal = boundary.observation.action_mask.iter().enumerate().filter_map(|(i, v)| (*v != 0).then_some(i)).collect::<Vec<_>>();
            boundary = session.step(boundary.token, RunActionIndex(legal[(step * 7 + 3) % legal.len()] as i64)).unwrap().boundary;
        }
        let record = session.record().unwrap().unwrap();
        let steps = record.steps.len();
        let total = median_micros((0..5).map(|_| { let t = Instant::now(); let s = RunSession::from_record(&record, steps, RunCapacities::standard_default()).unwrap(); let e = t.elapsed(); drop(s); e.as_secs_f64() * 1e6 }).collect());
        let replay = median_micros((0..5).map(|_| { let t = Instant::now(); crate::replay_run(&record, RunCapacities::standard_default()).unwrap(); t.elapsed().as_secs_f64() * 1e6 }).collect());
        eprintln!("M34C-FROM-RECORD {character:?} A0: {steps} steps, from_record(k=n) {total:.0} us = {:.1} us per accepted step; replay_run {replay:.0} us = {:.1} us per step (median of 5)", total / steps as f64, replay / steps as f64);
    }
}
