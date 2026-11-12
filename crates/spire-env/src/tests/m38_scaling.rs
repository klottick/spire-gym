//! M38a component-scaling harness (diagnostic only; no thresholds, no gameplay or API change). Run in release:
//! `cargo test --release --locked --offline -p spire-env m38_component_scaling -- --ignored --nocapture`.
//!
//! It separates game logic from observation building as threads are added, without the batch scheduler. Each of `T`
//! plain threads (8 MiB stacks, no rayon) drives `M38_SLOTS / T` Standard A20 runs of its own round-robin, through the
//! steps `m37_phase_timing`'s scalar pass 2 times: `Run::decision`, `Run::public_map`, `encode_run_standard`, the reward
//! snapshot, `Run::step_offered` and the reward items. A terminal run is replaced by a fresh one (`create_run`, timed
//! apart). Seeds, characters and the seeded uniform choice among the decision's candidates depend only on the run's
//! index among all `M38_SLOTS` runs and its own history. So every mode, and every thread count `T` that divides
//! `M38_SLOTS` (with `T <= M38_SLOTS`, true for the defaults), plays the same trajectories (the same total work). Per
//! thread it times the game logic and the encode (with the drop of the observation it replaces) separately. All threads
//! start together and play the same number of rounds; a round is one turn of each of the thread's runs, and a turn is
//! one step, or one terminal run replaced.
//!
//! Modes (`M38_MODES`, comma-separated; default all eight):
//! - `logic`: no encode; the floor without observations.
//! - `encode`: encode, then drop the observation at once (the allocator can hand the same memory back next step).
//! - `hold`: encode and keep each run's last observation until that run's next step (at 64 runs about 49 MB of live
//!   observations across the threads).
//! - `hold-compact`: `hold` with the compact capacities `m37_phase_timing` uses for `M37_CAPACITIES=compact` (an opt-in
//!   capacity choice with a different shape and fingerprint; here only to size the effect of fewer bytes).
//! - `hold-sync`: `hold`, but the threads wait at a barrier after each round, as a batch call ends when its last job ends.
//! - `hold-rotate`: `hold-sync`, but each round a thread takes the next thread's block of runs, so a run's state and
//!   observation move between cores every round, as batch slots can move between rayon workers.
//! - `hold-sync-batch` and `hold-rotate-batch`: as `hold-sync` and `hold-rotate`, with the batch's buffer lifetime. A
//!   batch caller keeps the previous call's rows until the next call returns, then drops them on its own thread. So
//!   here an observation replaced during a round stays alive until the round's barrier; thread 0 then frees all of
//!   them, and the threads wait again. At 64 runs about 98 MB is live at the peak, as in the batch.
//!
//! Environment: `M38_THREADS` (default `1,2,4,8,16,32`), `M38_SLOTS` (runs across all threads, default 64; each thread
//! gets `M38_SLOTS / T`, at least one), `M38_ROUNDS` (default 500), `M38_MODES` and `M38_OUT` (JSON path; otherwise JSON
//! on stdout).
use crate::run_observation::{encode_run_standard, StandardContext};
use crate::run_session::{checked_reward_items, create_run, RunProfile};
use crate::{RunCapacities, RunObservation, StandardRunConfig};
use serde_json::{json, Value};
use spire_core::{combat::rng::Partition, content::Character, run::Run};
use std::{hint::black_box, sync::{Barrier, Mutex}, time::Instant};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
const STEP_LIMIT: usize = 20_000;

/// The compact capacities: about twice the most rows random A20 play and the supplied routes occupy
/// (docs/performance.md, capacity headroom), with the small fixed groups unchanged.
pub(super) fn compact_capacities() -> RunCapacities {
    RunCapacities { map_nodes: 107, map_edges: 256, permanent_cards: 128, combat_cards: 128, enemies: 12, powers: 64,
        relics: 64, potions: 16, offers: 64, selection: 64, orbs: 10, actions: 128 }
}
fn observation_bytes(o: &RunObservation) -> usize {
    o.groups.iter().map(|g| g.values.len() * 8 + g.known.len() + g.occupied.as_ref().map_or(0, Vec::len)).sum::<usize>() + o.action_mask.len()
}
/// splitmix64, for the seeded action choice.
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { Logic, Encode, Hold, HoldCompact, HoldSync, HoldRotate, HoldSyncBatch, HoldRotateBatch }
impl Mode {
    fn parse(name: &str) -> Self {
        match name { "logic" => Self::Logic, "encode" => Self::Encode, "hold" => Self::Hold, "hold-compact" => Self::HoldCompact,
            "hold-sync" => Self::HoldSync, "hold-rotate" => Self::HoldRotate,
            "hold-sync-batch" => Self::HoldSyncBatch, "hold-rotate-batch" => Self::HoldRotateBatch,
            _ => panic!("M38_MODES entries are logic, encode, hold, hold-compact, hold-sync, hold-rotate, hold-sync-batch or hold-rotate-batch") }
    }
    fn name(self) -> &'static str {
        match self { Self::Logic => "logic", Self::Encode => "encode", Self::Hold => "hold", Self::HoldCompact => "hold-compact",
            Self::HoldSync => "hold-sync", Self::HoldRotate => "hold-rotate", Self::HoldSyncBatch => "hold-sync-batch",
            Self::HoldRotateBatch => "hold-rotate-batch" }
    }
    fn synced(self) -> bool { matches!(self, Self::HoldSync | Self::HoldRotate | Self::HoldSyncBatch | Self::HoldRotateBatch) }
    fn rotates(self) -> bool { matches!(self, Self::HoldRotate | Self::HoldRotateBatch) }
    /// The batch's buffer lifetime: a replaced observation stays alive until the round ends, then one thread frees it.
    fn caller_frees(self) -> bool { matches!(self, Self::HoldSyncBatch | Self::HoldRotateBatch) }
}

struct Slot { run: Run, seed: u64, character: Character, length: usize, resets: u64, held: Option<RunObservation>, replaced: Option<RunObservation> }
#[derive(Default)]
struct ThreadStats { steps: u64, resets: u64, logic_ns: u128, encode_ns: u128, reset_ns: u128, busy_ns: u128, caller_free_ns: u128, observation_bytes: usize }

fn new_run(character: Character, seed: u64) -> Run {
    let config = StandardRunConfig::new(character, 20, Partition::Test).session_config();
    create_run(RunProfile::Standard, &config, seed, 0).unwrap()
}

/// One turn of `slot`: decision and public map, the encode (unless `Logic`), then a step or a replacement run.
fn turn(slot: &mut Slot, mode: Mode, caps: &RunCapacities, stats: &mut ThreadStats) {
    let start = Instant::now();
    let decision = slot.run.decision().unwrap();
    let map = slot.run.public_map();
    let decided = Instant::now();
    stats.logic_ns += (decided - start).as_nanos();
    if mode != Mode::Logic {
        let context = StandardContext { ascension: slot.run.ascension(), run_selection: slot.run.room_selection_operation() };
        let observation = encode_run_standard(&decision, map.as_ref(), caps, context).unwrap();
        stats.observation_bytes = observation_bytes(&observation);
        if mode == Mode::Encode { drop(black_box(observation)) }
        else if mode.caller_frees() { slot.replaced = slot.held.replace(observation) }
        else { drop(slot.held.replace(observation)) }
        stats.encode_ns += decided.elapsed().as_nanos();
    }
    if decision.terminal.is_some() || slot.length >= STEP_LIMIT {
        let reset = Instant::now();
        slot.resets += 1;
        slot.seed = mix(slot.seed ^ slot.resets);
        slot.run = new_run(slot.character, slot.seed);
        slot.length = 0;
        stats.resets += 1;
        stats.reset_ns += reset.elapsed().as_nanos();
        return;
    }
    let stepping = Instant::now();
    let choice = (mix(slot.seed ^ (slot.length as u64).wrapping_mul(0x100_0000_01B3)) % decision.candidates.len() as u64) as usize;
    let action = decision.candidates.into_iter().nth(choice).unwrap().action;
    let ledger = slot.run.reward_snapshot();
    let transition = slot.run.step_offered(&action).unwrap();
    black_box(checked_reward_items(&ledger, &transition).unwrap());
    stats.logic_ns += stepping.elapsed().as_nanos();
    slot.length += 1;
    stats.steps += 1;
}

/// Thread `thread` creates its own block of runs, then plays `rounds` rounds of one turn per run of a block: its own
/// block, or in the rotating modes the block `(thread + round) % threads`. The synchronised modes wait at
/// `round_barrier` after each round. In the `-batch` modes thread 0 then frees every observation replaced in that round,
/// as a batch caller drops the previous call's rows, and all threads wait again before the next round. Waiting is not in
/// the timed phases but is in `busy_ns`; the freeing is `caller_free_ns`.
#[allow(clippy::too_many_arguments)]
fn thread_work(thread: usize, threads: usize, rounds: usize, runs: usize, mode: Mode, blocks: &[Mutex<Vec<Slot>>], start: &Barrier, round_barrier: &Barrier) -> ThreadStats {
    let caps = if mode == Mode::HoldCompact { compact_capacities() } else { RunCapacities::standard_default() };
    *blocks[thread].lock().unwrap() = (0..runs).map(|r| {
        let index = thread * runs + r;
        let (character, seed) = (CHARACTERS[index % 4], 0xA38_0000 + index as u64);
        Slot { run: new_run(character, seed), seed, character, length: 0, resets: 0, held: None, replaced: None }
    }).collect();
    let mut stats = ThreadStats::default();
    start.wait();
    let busy = Instant::now();
    for round in 0..rounds {
        let block = if mode.rotates() { (thread + round) % threads } else { thread };
        let mut slots = blocks[block].lock().unwrap();
        for slot in slots.iter_mut() { turn(slot, mode, &caps, &mut stats) }
        drop(slots);
        if mode.synced() { round_barrier.wait(); }
        if mode.caller_frees() {
            if thread == 0 {
                let freeing = Instant::now();
                for block in blocks { for slot in block.lock().unwrap().iter_mut() { drop(slot.replaced.take()) } }
                stats.caller_free_ns += freeing.elapsed().as_nanos();
            }
            round_barrier.wait();
        }
    }
    stats.busy_ns = busy.elapsed().as_nanos();
    stats
}

fn scaling_point(threads: usize, slots: usize, rounds: usize, mode: Mode) -> Value {
    let runs = (slots / threads).max(1);
    let blocks = (0..threads).map(|_| Mutex::new(Vec::new())).collect::<Vec<Mutex<Vec<Slot>>>>();
    let (start, round_barrier) = (Barrier::new(threads + 1), Barrier::new(threads));
    let (wall, stats) = std::thread::scope(|scope| {
        let handles = (0..threads).map(|t| {
            let (blocks, start, round_barrier) = (&blocks, &start, &round_barrier);
            std::thread::Builder::new().stack_size(crate::workers::WORKER_STACK_BYTES)
                .spawn_scoped(scope, move || thread_work(t, threads, rounds, runs, mode, blocks, start, round_barrier)).unwrap()
        }).collect::<Vec<_>>();
        start.wait();
        let begin = Instant::now();
        let stats = handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<_>>();
        (begin.elapsed(), stats)
    });
    let turns = (threads * runs * rounds) as u64;
    let total_steps: u64 = stats.iter().map(|s| s.steps).sum();
    let sum = |f: fn(&ThreadStats) -> u128| stats.iter().map(f).sum::<u128>() as f64;
    let per_turn_us = |ns: f64| ns / 1e3 / turns as f64;
    json!({"mode": mode.name(), "threads": threads, "runs_per_thread": runs, "rounds": rounds, "turns": turns, "accepted_steps": total_steps,
        "resets": stats.iter().map(|s| s.resets).sum::<u64>(), "observation_bytes": stats[0].observation_bytes,
        "wall_ms": wall.as_secs_f64() * 1e3, "turns_per_second": turns as f64 / wall.as_secs_f64(),
        "steps_per_second": total_steps as f64 / wall.as_secs_f64(),
        "logic_us_per_turn": per_turn_us(sum(|s| s.logic_ns)), "encode_us_per_turn": per_turn_us(sum(|s| s.encode_ns)),
        "reset_us_per_turn": per_turn_us(sum(|s| s.reset_ns)), "busy_us_per_turn": per_turn_us(sum(|s| s.busy_ns)),
        "caller_free_ms_per_round": sum(|s| s.caller_free_ns) / 1e6 / rounds as f64,
        "slowest_thread_busy_ms": stats.iter().map(|s| s.busy_ns).max().unwrap_or(0) as f64 / 1e6})
}

#[test]
#[ignore]
fn m38_component_scaling() {
    let list = |key: &str, default: &str| std::env::var(key).unwrap_or_else(|_| default.to_owned());
    let threads = list("M38_THREADS", "1,2,4,8,16,32").split(',').map(|t| t.parse::<usize>().expect("M38_THREADS: positive integers")).collect::<Vec<_>>();
    let slots = list("M38_SLOTS", "64").parse::<usize>().expect("M38_SLOTS: a positive integer");
    let rounds = list("M38_ROUNDS", "500").parse::<usize>().expect("M38_ROUNDS: a positive integer");
    let modes = list("M38_MODES", "logic,encode,hold,hold-compact,hold-sync,hold-rotate,hold-sync-batch,hold-rotate-batch").split(',').map(Mode::parse).collect::<Vec<_>>();
    let mut points = Vec::new();
    for &mode in &modes { for &t in &threads { points.push(scaling_point(t, slots, rounds, mode)) } }
    let out = json!({"schema_id": "spire-gym-m38-component-scaling-v1", "rules_id": crate::RUN_RULES_ID_STANDARD,
        "parameters": {"threads": threads, "slots": slots, "rounds": rounds, "modes": modes.iter().map(|m| m.name()).collect::<Vec<_>>()},
        "host_cpu_count": std::thread::available_parallelism().map(usize::from).ok(), "debug_assertions": cfg!(debug_assertions),
        "points": points});
    let text = serde_json::to_string_pretty(&out).unwrap();
    match std::env::var("M38_OUT") { Ok(path) => std::fs::write(path, format!("{text}\n")).unwrap(), Err(_) => println!("{text}") }
}
