//! M37a phase-timing harness (diagnostic only; no thresholds, no gameplay or API change). Run in release:
//! `cargo test --release --locked --offline -p spire-env m37_phase_timing -- --ignored --nocapture`.
//!
//! Workload: the `a20_checks` one, Standard A20, the four characters and the seeded R1 legal-action rule.
//! - `scalar`: pass 1 times the public `RunSession::reset`/`step`; pass 2 replays the same seeds and actions through
//!   the steps `RunSession::step_with_status` and `publish` run, timing each one: reward snapshot, `Run::step` (game
//!   logic), reward items, `Run::decision` (public decision and legal candidates), `Run::public_map`,
//!   `encode_run_standard` (dense observation and action mask), `RunObservation::owned` (the returned copy), the action
//!   list and the observation drops, holding observations as long as the session and caller do. Pass 2 must reproduce
//!   pass 1's episode lengths and terminals. Layout probes then time the fixed Standard layout alone: fresh zeroed
//!   allocation, zeroing a reused buffer, a fresh copy and a copy into a reused buffer.
//! - `batch`: a `SessionBatch` over a timing wrapper of `RunSession`. Per job it records the slot, worker thread and
//!   start/end times, and per call it reports the serial head (before the first job starts), the parallel span, the
//!   serial tail (after the last job ends), per-worker busy time and the critical path. It also times a copy of the
//!   rows into `[B, ...]` columns in the order of `spire-python`'s `stacking::stack` (the per-call copy before numpy).
//!
//! Environment: `M37_PART` (`scalar`, `batch` or `all`; default `all`), `M37_SEEDS` (seeds per character, default 256),
//! `M37_SLOTS` (default 64), `M37_BATCH_STEPS` (default 100), `M37_WORKERS` (default `1,4,16`), `M37_RECORD`
//! (`on`, `off` or `both`; default `both`), `M37_STACK` (`off` skips the stacking copy) and `M37_OUT` (JSON path;
//! otherwise JSON on stdout).
use crate::run_observation::{encode_run_standard, StandardContext};
use crate::run_session::{checked_reward_items, create_run, RunProfile};
use crate::session_batch::{sealed::Sealed, BatchSession, BatchStep, BatchStepFailure};
use crate::{
    AutoresetMode, BatchConfig, FaultPolicy, RunActionIndex, RunBoundary, RunDecisionToken, RunEpisodeRecord, RunObservation,
    RunSession, RunSessionError, SessionBatch, StandardRunConfig,
};
use serde_json::{json, Value};
use spire_core::{combat::rng::Partition, content::Character};
use std::{
    collections::{BTreeMap, BTreeSet},
    hint::black_box,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, OnceLock,
    },
    time::{Duration, Instant},
};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
const STEP_LIMIT: usize = 20_000;

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key).ok().map(|v| v.parse().unwrap_or_else(|_| panic!("{key} must be a positive integer"))).unwrap_or(default)
}
fn character_index(character: Character) -> u64 {
    CHARACTERS.iter().position(|c| *c == character).unwrap() as u64
}
fn observation_bytes(o: &RunObservation) -> usize {
    o.groups.iter().map(|g| g.values.len() * 8 + g.known.len() + g.occupied.as_ref().map_or(0, Vec::len)).sum::<usize>() + o.action_mask.len()
}
/// Accumulated nanoseconds and call count per named phase, in first-seen order.
#[derive(Default)]
struct Phases { order: Vec<&'static str>, totals: BTreeMap<&'static str, (u128, u64)> }
impl Phases {
    fn add(&mut self, name: &'static str, elapsed: Duration) {
        if !self.totals.contains_key(name) { self.order.push(name) }
        let entry = self.totals.entry(name).or_default();
        entry.0 += elapsed.as_nanos();
        entry.1 += 1;
    }
    fn time<T>(&mut self, name: &'static str, f: impl FnOnce() -> T) -> T {
        let start = Instant::now();
        let value = f();
        self.add(name, start.elapsed());
        value
    }
    fn json(&self, steps: u64) -> Value {
        let total: u128 = self.totals.values().map(|(ns, _)| ns).sum();
        Value::Array(self.order.iter().map(|name| {
            let (ns, calls) = self.totals[*name];
            json!({"phase": name, "calls": calls, "total_ms": ns as f64 / 1e6, "mean_us": ns as f64 / 1e3 / calls.max(1) as f64,
                "us_per_step": ns as f64 / 1e3 / steps.max(1) as f64,
                "share_of_measured": ns as f64 / total.max(1) as f64})
        }).collect())
    }
}

/// Pass 1: the public scalar API, as `a20_checks` S times it (step plus boundary read), plus reset.
fn scalar_public(seeds: usize) -> (Value, Vec<(usize, String)>) {
    let mut phases = Phases::default();
    let mut episodes = Vec::new();
    let mut steps = 0u64;
    let mut max_obs_bytes = 0usize;
    for character in CHARACTERS {
        for seed_index in 0..seeds {
            let seed = (character_index(character) << 32) | seed_index as u64;
            let mut session = RunSession::new_standard(StandardRunConfig::new(character, 20, Partition::Test)).unwrap();
            let mut boundary = phases.time("session_reset", || session.reset(Some(seed)).unwrap());
            max_obs_bytes = max_obs_bytes.max(observation_bytes(&boundary.observation));
            let mut length = 0usize;
            while boundary.terminal.is_none() && length < STEP_LIMIT {
                let action = phases.time("policy_select", || crate::tests::m35_r2::r1(seed, length as u64, &boundary.observation.action_mask));
                let result = phases.time("session_step", || session.step(boundary.token, RunActionIndex(action as i64)).unwrap());
                black_box(result.boundary.observation.action_mask.len());
                let previous = std::mem::replace(&mut boundary, result.boundary);
                phases.time("drop_previous_boundary", || drop(previous));
                length += 1;
                steps += 1;
            }
            episodes.push((length, format!("{:?}", boundary.terminal)));
        }
    }
    (json!({"episodes": episodes.len(), "steps": steps, "max_observation_bytes": max_obs_bytes, "phases": phases.json(steps)}), episodes)
}

/// Pass 2: the same trajectories through the session's own steps, one timer per step.
fn scalar_decomposed(seeds: usize, expected: &[(usize, String)]) -> Value {
    let mut phases = Phases::default();
    let mut steps = 0u64;
    let mut episode = 0usize;
    let mut group_bytes: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut last_layout = None;
    for character in CHARACTERS {
        for seed_index in 0..seeds {
            let seed = (character_index(character) << 32) | seed_index as u64;
            let config = StandardRunConfig::new(character, 20, Partition::Test).session_config();
            let caps = config.capacities.clone();
            let mut run = phases.time("reset_create_run", || create_run(RunProfile::Standard, &config, seed, 0).unwrap());
            let mut length = 0usize;
            // Held as the session holds them: the published observation until the next publish, and the returned
            // copy until the caller's next step returns.
            let mut published: Option<RunObservation> = None;
            let mut returned_previous: Option<RunObservation> = None;
            let terminal;
            loop {
                let decision = phases.time("core_decision", || run.decision().unwrap());
                let map = phases.time("core_public_map", || run.public_map());
                let context = StandardContext { ascension: run.ascension(), run_selection: run.room_selection_operation() };
                let observation = phases.time("encode_observation", || encode_run_standard(&decision, map.as_ref(), &caps, context).unwrap());
                let actions = phases.time("candidate_actions", || decision.candidates.into_iter().map(|c| c.action).collect::<Vec<_>>());
                let returned = phases.time("owned_copy", || observation.owned().unwrap());
                let previous = published.replace(observation);
                phases.time("drop_previous_published", || drop(previous));
                if steps == 0 && length == 0 {
                    for g in &returned.groups { group_bytes.insert(g.key, g.values.len() * 8 + g.known.len() + g.occupied.as_ref().map_or(0, Vec::len)); }
                    group_bytes.insert("action_mask", returned.action_mask.len());
                }
                let previous = returned_previous.replace(returned);
                phases.time("drop_previous_returned", || drop(previous));
                if decision.terminal.is_some() || length >= STEP_LIMIT { terminal = decision.terminal; break }
                let mask = &returned_previous.as_ref().unwrap().action_mask;
                let action = phases.time("policy_select", || crate::tests::m35_r2::r1(seed, length as u64, mask));
                let ledger = phases.time("reward_snapshot", || run.reward_snapshot());
                let transition = phases.time("core_step", || run.step(&actions[action]).unwrap());
                let items = phases.time("reward_items", || checked_reward_items(&ledger, &transition).unwrap());
                black_box(items);
                length += 1;
                steps += 1;
            }
            last_layout = returned_previous.take();
            assert_eq!((length, format!("{terminal:?}")), expected[episode], "decomposed pass diverged from the public pass at episode {episode}");
            episode += 1;
        }
    }
    json!({"episodes": episode, "steps": steps, "observation_bytes_by_group": group_bytes, "phases": phases.json(steps),
        "layout_probes": layout_probes(&last_layout.expect("at least one episode"))})
}

/// Costs of the fixed Standard layout alone (every Standard observation has the same shape): a fresh zeroed allocation
/// and its drop (the encoder's fill-free floor), zeroing a reused buffer, a fresh copy (`owned`) and a copy into a reused
/// buffer. Means over 2,000 repetitions, in microseconds.
fn layout_probes(o: &RunObservation) -> Value {
    const REPS: u32 = 2_000;
    let fresh = || (o.groups.iter().map(|g| (vec![0i64; g.values.len()], vec![0i8; g.known.len()], g.occupied.as_ref().map(|v| vec![0i8; v.len()]))).collect::<Vec<_>>(), vec![0i8; o.action_mask.len()]);
    let mean = |f: &mut dyn FnMut()| { let start = Instant::now(); for _ in 0..REPS { f() } start.elapsed().as_secs_f64() * 1e6 / f64::from(REPS) };
    let alloc_zero_drop = mean(&mut || { black_box(fresh()); });
    let mut reused = fresh();
    let zero_reused = mean(&mut || { for (v, k, occ) in &mut reused.0 { v.fill(0); k.fill(0); if let Some(occ) = occ { occ.fill(0) } } reused.1.fill(0); black_box(&reused); });
    let owned_copy_drop = mean(&mut || { black_box(o.owned().unwrap()); });
    let mut target = o.owned().unwrap();
    let copy_reused = mean(&mut || { for (t, g) in target.groups.iter_mut().zip(&o.groups) { t.values.copy_from_slice(&g.values); t.known.copy_from_slice(&g.known); if let (Some(t), Some(s)) = (&mut t.occupied, &g.occupied) { t.copy_from_slice(s) } } target.action_mask.copy_from_slice(&o.action_mask); black_box(&target); });
    json!({"observation_bytes": observation_bytes(o), "repetitions": REPS, "fresh_zeroed_alloc_and_drop_us": alloc_zero_drop, "zero_reused_buffer_us": zero_reused,
        "owned_copy_and_drop_us": owned_copy_drop, "copy_into_reused_buffer_us": copy_reused})
}

/// Per-job timings a wrapped session leaves for the harness (relative to the process epoch, in ns).
#[derive(Default)]
struct JobClock { start: AtomicU64, end: AtomicU64, thread: AtomicU64, kind: AtomicU64, record_export_ns: AtomicU64 }
fn epoch() -> Instant { static EPOCH: OnceLock<Instant> = OnceLock::new(); *EPOCH.get_or_init(Instant::now) }
fn now_ns() -> u64 { epoch().elapsed().as_nanos() as u64 }
fn thread_tag() -> u64 { rayon::current_thread_index().map_or(0, |i| i as u64 + 1) }
const KIND_STEP: u64 = 1;
const KIND_RESET: u64 = 2;

/// `RunSession` with its step and reset jobs timed. Every other method delegates unchanged.
struct TimedSession { inner: RunSession, clock: Arc<JobClock> }
impl TimedSession {
    fn stamp(&self, start: u64, kind: u64) {
        self.clock.start.store(start, Ordering::Relaxed);
        self.clock.end.store(now_ns(), Ordering::Relaxed);
        self.clock.thread.store(thread_tag(), Ordering::Relaxed);
        self.clock.kind.store(kind, Ordering::Relaxed);
    }
}
impl Sealed for TimedSession {}
impl BatchSession for TimedSession {
    type Boundary = RunBoundary;
    type Record = RunEpisodeRecord;
    fn batch_preflight_reset(&self, seed: Option<u64>, entry: Option<&str>) -> Result<(), RunSessionError> { self.inner.batch_preflight_reset(seed, entry) }
    fn batch_preflight_fresh(&self, entry: Option<&str>) -> Result<(), RunSessionError> { self.inner.batch_preflight_fresh(entry) }
    fn batch_reset(&mut self, seed: Option<u64>, entry: Option<&str>) -> Result<RunBoundary, RunSessionError> {
        let start = now_ns();
        let result = self.inner.batch_reset(seed, entry);
        self.stamp(start, KIND_RESET);
        result
    }
    fn batch_preflight_step(&self, token: RunDecisionToken, index: RunActionIndex) -> Result<(), RunSessionError> { self.inner.batch_preflight_step(token, index) }
    fn batch_step(&mut self, token: RunDecisionToken, index: RunActionIndex) -> Result<BatchStep<RunBoundary>, BatchStepFailure> {
        let start = now_ns();
        let result = self.inner.batch_step(token, index);
        self.stamp(start, KIND_STEP);
        result
    }
    fn batch_publication_serial(&self) -> u64 { self.inner.batch_publication_serial() }
    fn batch_retain_error(&mut self, error: RunSessionError, committed: bool) -> RunSessionError { self.inner.batch_retain_error(error, committed) }
    fn batch_retain_panic(&mut self, committed: bool) -> RunSessionError { self.inner.batch_retain_panic(committed) }
    fn batch_decision(&self) -> Result<RunBoundary, RunSessionError> { self.inner.batch_decision() }
    fn batch_record(&self) -> Result<Option<RunEpisodeRecord>, RunSessionError> {
        let start = now_ns();
        let record = self.inner.batch_record();
        self.clock.record_export_ns.fetch_add(now_ns() - start, Ordering::Relaxed);
        record
    }
    fn batch_fault(&self) -> Option<RunSessionError> { self.inner.batch_fault() }
    fn batch_close(&mut self) -> Result<(), RunSessionError> { self.inner.batch_close() }
    fn batch_fingerprint(&self) -> Result<[u8; 32], RunSessionError> { self.inner.batch_fingerprint() }
    fn batch_current_entry(&self) -> Option<String> { self.inner.batch_current_entry() }
    fn batch_is_terminal(&self) -> bool { self.inner.batch_is_terminal() }
    fn batch_fresh(&self) -> Result<Self, RunSessionError> { Ok(Self { inner: self.inner.batch_fresh()?, clock: self.clock.clone() }) }
    fn batch_zeroed(boundary: &RunBoundary) -> Result<RunBoundary, RunSessionError> { RunSession::batch_zeroed(boundary) }
}

/// The copy `spire-python`'s `stacking::stack` makes per call: each group's values, known flags and occupancy, then the
/// action mask, appended row by row into one `[B, ...]` column each (its numpy hand-off is not included).
fn stack_columns(rows: &[&RunObservation]) -> usize {
    let first = rows[0];
    let mut columns = Vec::new();
    for (index, group) in first.groups.iter().enumerate() {
        let mut values = Vec::with_capacity(group.values.len() * rows.len());
        let mut known = Vec::with_capacity(group.known.len() * rows.len());
        let mut occupied = group.occupied.as_ref().map(|o| Vec::with_capacity(o.len() * rows.len()));
        for row in rows {
            let other = &row.groups[index];
            values.extend_from_slice(&other.values);
            known.extend_from_slice(&other.known);
            if let (Some(target), Some(source)) = (&mut occupied, &other.occupied) { target.extend_from_slice(source) }
        }
        columns.push((values, known, occupied));
    }
    let mut mask = Vec::with_capacity(first.action_mask.len() * rows.len());
    for row in rows { mask.extend_from_slice(&row.action_mask) }
    black_box(&columns);
    columns.len() + mask.len()
}

fn batch_part(slots: usize, steps: usize, workers: usize, record: bool, stacking: bool) -> Value {
    let clocks = (0..slots).map(|_| Arc::new(JobClock::default())).collect::<Vec<_>>();
    let sessions = (0..slots).map(|slot| {
        let mut config = StandardRunConfig::new(CHARACTERS[(slot / (slots / 4).max(1)) % 4], 20, Partition::Test);
        config.record_replay = record;
        TimedSession { inner: RunSession::new_standard(config).unwrap(), clock: clocks[slot].clone() }
    }).collect::<Vec<_>>();
    let config = BatchConfig { workers, autoreset: AutoresetMode::NextStep, fault_policy: FaultPolicy::Freeze };
    let mut batch = SessionBatch::from_sessions(sessions, vec![None; slots], config).unwrap();
    let seeds = (0..slots).map(|slot| Some(0xA20_0000 + slot as u64)).collect::<Vec<_>>();
    let mut rows = batch.reset(&seeds, &vec![None; slots], None).unwrap();
    let before_record_ns = clocks.iter().map(|c| c.record_export_ns.load(Ordering::Relaxed)).sum::<u64>();
    let mut tokens = rows.iter().map(|row| row.boundary.token).collect::<Vec<_>>();
    let (mut wall, mut head, mut span, mut tail, mut busy_sum, mut critical, mut stack) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut step_jobs, mut step_ns, mut reset_jobs, mut reset_ns, mut max_reset_ns) = (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut call_max_job = 0u64;
    let mut drop_rows = 0u64;
    let mut threads_seen = BTreeSet::new();
    for tick in 0..steps {
        let actions = rows.iter().enumerate().map(|(slot, row)| {
            if row.boundary.terminal.is_some() { RunActionIndex(0) }
            else { RunActionIndex(crate::tests::m35_r2::r1(seeds[slot].unwrap(), tick as u64, &row.boundary.observation.action_mask) as i64) }
        }).collect::<Vec<_>>();
        let call_start = now_ns();
        let next = batch.step(&tokens, &actions).unwrap();
        let call_end = now_ns();
        let previous = std::mem::replace(&mut rows, next);
        let drop_start = now_ns();
        drop(previous);
        drop_rows += now_ns() - drop_start;
        let (mut first, mut last) = (u64::MAX, 0u64);
        let mut per_thread: BTreeMap<u64, u64> = BTreeMap::new();
        let mut longest = 0u64;
        for clock in &clocks {
            let (start, end) = (clock.start.load(Ordering::Relaxed), clock.end.load(Ordering::Relaxed));
            if start < call_start { continue }
            let duration = end - start;
            first = first.min(start);
            last = last.max(end);
            longest = longest.max(duration);
            let thread = clock.thread.load(Ordering::Relaxed);
            threads_seen.insert(thread);
            *per_thread.entry(thread).or_default() += duration;
            if clock.kind.load(Ordering::Relaxed) == KIND_RESET { reset_jobs += 1; reset_ns += duration; max_reset_ns = max_reset_ns.max(duration) }
            else { step_jobs += 1; step_ns += duration }
        }
        wall += call_end - call_start;
        head += first - call_start;
        span += last - first;
        tail += call_end - last;
        busy_sum += per_thread.values().sum::<u64>();
        critical += per_thread.values().copied().max().unwrap_or(0);
        call_max_job += longest;
        if stacking {
            let stack_start = now_ns();
            let observations = rows.iter().map(|row| &row.boundary.observation).collect::<Vec<_>>();
            black_box(stack_columns(&observations));
            stack += now_ns() - stack_start;
        }
        tokens.clear();
        tokens.extend(rows.iter().map(|row| row.boundary.token));
    }
    let record_ns = clocks.iter().map(|c| c.record_export_ns.load(Ordering::Relaxed)).sum::<u64>() - before_record_ns;
    let ms = |ns: u64| ns as f64 / 1e6;
    let slot_steps = (slots * steps) as f64;
    json!({"slots": slots, "workers": workers, "record_replay": record, "stacking": stacking, "batch_calls": steps, "slot_steps": slot_steps,
        "worker_threads_seen": threads_seen.len(),
        "slot_steps_per_second_step_only": slot_steps / (wall as f64 / 1e9),
        "slot_steps_per_second_with_stacking": slot_steps / ((wall + stack) as f64 / 1e9),
        "wall_ms": ms(wall), "serial_head_ms": ms(head), "parallel_span_ms": ms(span), "serial_tail_ms": ms(tail),
        "job_busy_sum_ms": ms(busy_sum), "critical_path_ms": ms(critical), "sum_of_longest_job_per_call_ms": ms(call_max_job),
        "ideal_span_ms_busy_over_workers": ms(busy_sum) / workers.min(slots) as f64,
        "step_jobs": step_jobs, "step_job_mean_us": step_ns as f64 / 1e3 / step_jobs.max(1) as f64,
        "reset_jobs": reset_jobs, "reset_job_mean_us": reset_ns as f64 / 1e3 / reset_jobs.max(1) as f64, "reset_job_max_us": max_reset_ns as f64 / 1e3,
        "reset_job_total_ms": ms(reset_ns), "record_export_in_autoreset_ms": ms(record_ns),
        "drop_previous_rows_ms": ms(drop_rows), "stack_copy_ms": ms(stack), "stack_copy_us_per_call": stack as f64 / 1e3 / steps as f64})
}

#[test]
#[ignore]
fn m37_phase_timing() {
    let part = std::env::var("M37_PART").unwrap_or_else(|_| "all".to_owned());
    let seeds = env_usize("M37_SEEDS", 256);
    let slots = env_usize("M37_SLOTS", 64);
    let steps = env_usize("M37_BATCH_STEPS", 100);
    let workers = std::env::var("M37_WORKERS").unwrap_or_else(|_| "1,4,16".to_owned()).split(',')
        .map(|w| w.parse::<usize>().expect("M37_WORKERS must be comma-separated positive integers")).collect::<Vec<_>>();
    let records: &[bool] = match std::env::var("M37_RECORD").as_deref().unwrap_or("both") { "on" => &[true], "off" => &[false], _ => &[true, false] };
    epoch();
    let mut out = json!({"schema_id": "spire-gym-m37-phase-timing-v1", "rules_id": crate::RUN_RULES_ID_STANDARD,
        "parameters": {"part": part, "seeds_per_character": seeds, "slots": slots, "batch_steps": steps, "workers": workers, "record_replay": records},
        "host_cpu_count": std::thread::available_parallelism().map(usize::from).ok(), "debug_assertions": cfg!(debug_assertions)});
    if part == "scalar" || part == "all" {
        let wall = Instant::now();
        let (public, episodes) = scalar_public(seeds);
        let decomposed = scalar_decomposed(seeds, &episodes);
        out["scalar"] = json!({"public": public, "decomposed": decomposed, "wall_seconds": wall.elapsed().as_secs_f64()});
    }
    if part == "batch" || part == "all" {
        let mut runs = Vec::new();
        let stacking = std::env::var("M37_STACK").as_deref() != Ok("off");
        for &record in records { for &w in &workers { runs.push(batch_part(slots, steps, w, record, stacking)) } }
        out["batch"] = Value::Array(runs);
    }
    let text = serde_json::to_string_pretty(&out).unwrap();
    match std::env::var("M37_OUT") { Ok(path) => std::fs::write(path, format!("{text}\n")).unwrap(), Err(_) => println!("{text}") }
}
