//! Bounded Standard A20 throughput, memory, and observation-capacity check.
//!
//! JSON schema: `spire-gym-a20-checks-v1` with `identity` (revision, Standard rules ID,
//! observation fingerprint, host CPU count), phases `S`, `B`, and `M`, and `capacity`
//! rows (`group`, `capacity`, `max_occupied`, `headroom`). `S` is four characters by
//! `--seeds`; `B` measures 64 mixed slots for `--batch-steps` batch calls at each
//! requested worker count; `M` records process RSS and runs a 256-slot reset plus 50
//! batch calls. RSS values are KiB, or `"unavailable"` off Linux.
//!
//! Usage: `a20_checks [--seeds N] [--batch-steps K] [--workers 1,4,16] [--out PATH] [--smoke]`.
use serde_json::{json, Value};
use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    standard_run_observation_spec, AutoresetMode, BatchConfig, FaultPolicy, RunActionIndex,
    RunBoundary, RunCapacities, RunSession, RunSessionError, SessionBatch, StandardRunConfig,
    RUN_RULES_ID_STANDARD,
};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    time::{Duration, Instant},
};

const STEP_LIMIT: usize = 20_000;
const DEFAULT_SEEDS: usize = 256;
const DEFAULT_BATCH_STEPS: usize = 100;
const CAPACITY_NAMES: [&str; 12] = [
    "map_nodes", "map_edges", "permanent_cards", "combat_cards", "enemies", "powers",
    "relics", "potions", "offers", "selection", "orbs", "actions",
];

/// M35 R1 seeded SplitMix action selection (copied from `tests/m35_r2.rs`).
fn r1(seed: u64, step: u64, mask: &[i8]) -> usize {
    let legal: Vec<usize> = mask.iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, _)| i).collect();
    let mut z = seed.wrapping_add((step + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    legal[(z % legal.len() as u64) as usize]
}

#[derive(Clone)]
struct Options {
    seeds: usize,
    batch_steps: usize,
    workers: [usize; 3],
    out: Option<PathBuf>,
}

fn usage() -> &'static str {
    "usage: a20_checks [--seeds N] [--batch-steps K] [--workers 1,4,16] [--out PATH] [--smoke]"
}

fn parse_args() -> Result<Options, String> {
    let mut options = Options { seeds: DEFAULT_SEEDS, batch_steps: DEFAULT_BATCH_STEPS, workers: [1, 4, 16], out: None };
    let mut smoke = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke" => smoke = true,
            "--seeds" => options.seeds = args.next().ok_or_else(|| usage().to_owned())?.parse().map_err(|_| "--seeds must be a positive integer".to_owned())?,
            "--batch-steps" => options.batch_steps = args.next().ok_or_else(|| usage().to_owned())?.parse().map_err(|_| "--batch-steps must be a positive integer".to_owned())?,
            "--workers" => {
                let values = args.next().ok_or_else(|| usage().to_owned())?;
                let parsed = values.split(',').map(str::parse::<usize>).collect::<Result<Vec<_>, _>>().map_err(|_| "--workers must be three positive comma-separated integers".to_owned())?;
                if parsed.len() != 3 || parsed.contains(&0) { return Err("--workers must be three positive comma-separated integers".into()) }
                options.workers = [parsed[0], parsed[1], parsed[2]];
            }
            "--out" => options.out = Some(PathBuf::from(args.next().ok_or_else(|| usage().to_owned())?)),
            _ => return Err(format!("unknown argument {arg}; {}", usage())),
        }
    }
    if smoke {
        options.seeds = 1;
        options.batch_steps = 2;
        options.workers = [1, 1, 1];
    }
    if options.seeds == 0 || options.batch_steps == 0 { return Err("--seeds and --batch-steps must be positive".into()) }
    Ok(options)
}

fn capacities() -> RunCapacities { RunCapacities::standard_default() }

fn capacity_limits(c: &RunCapacities) -> [usize; 12] {
    [c.map_nodes, c.map_edges, c.permanent_cards, c.combat_cards, c.enemies, c.powers,
     c.relics, c.potions, c.offers, c.selection, c.orbs, c.actions]
}

fn record_boundary(boundary: &RunBoundary, maxima: &mut BTreeMap<String, usize>) {
    for name in CAPACITY_NAMES {
        if let Some(group) = boundary.observation.group(name) {
            let used = group.occupied.as_ref().map_or(0, |rows| rows.iter().filter(|v| **v != 0).count());
            maxima.entry(name.to_owned()).and_modify(|value| *value = (*value).max(used)).or_insert(used);
        }
    }
}

fn character_name(character: Character) -> &'static str {
    match character { Character::Ironclad => "ironclad", Character::Silent => "silent", Character::Defect => "defect", Character::Watcher => "watcher" }
}

fn character_index(character: Character) -> u64 {
    match character { Character::Ironclad => 0, Character::Silent => 1, Character::Defect => 2, Character::Watcher => 3 }
}

fn rss() -> Value {
    #[cfg(target_os = "linux")]
    {
        let text = fs::read_to_string("/proc/self/status").ok();
        let read = |key: &str| -> Value {
            text.as_deref().and_then(|contents| contents.lines().find_map(|line| line.strip_prefix(key)))
                .and_then(|value| value.split_whitespace().next())
                .and_then(|value| value.parse::<u64>().ok()).map(|value| json!(value)).unwrap_or_else(|| json!("unavailable"))
        };
        json!({"VmHWM_kib": read("VmHWM:"), "VmRSS_kib": read("VmRSS:")})
    }
    #[cfg(not(target_os = "linux"))]
    { json!({"VmHWM_kib": "unavailable", "VmRSS_kib": "unavailable"}) }
}

fn occupied(boundary: &RunBoundary, maxima: &mut BTreeMap<String, usize>) {
    record_boundary(boundary, maxima);
}

fn run_single(seeds: usize, maxima: &mut BTreeMap<String, usize>) -> Result<Value, String> {
    let characters = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
    let mut accepted = 0usize;
    let mut measured = Duration::ZERO;
    let mut lengths = Vec::new();
    let mut terminals = BTreeMap::<String, usize>::new();
    let wall = Instant::now();
    for character in characters {
        for seed_index in 0..seeds {
            let seed = (character_index(character) << 32) | seed_index as u64;
            let mut session = RunSession::new_standard(StandardRunConfig::new(character, 20, Partition::Test))
                .map_err(|error| format!("S create {}/seed {seed}: {error:?}", character_name(character)))?;
            let mut boundary = session.reset(Some(seed)).map_err(|error| format!("S reset {}/seed {seed}: {error:?}", character_name(character)))?;
            occupied(&boundary, maxima);
            let mut length = 0usize;
            while boundary.terminal.is_none() && length < STEP_LIMIT {
                let action = r1(seed, length as u64, &boundary.observation.action_mask);
                let start = Instant::now();
                let result = session.step(boundary.token, RunActionIndex(action as i64))
                    .map_err(|error: RunSessionError| format!("S step {}/seed {seed}: {error:?}", character_name(character)))?;
                let _observation_read = result.boundary.observation.action_mask.len();
                measured += start.elapsed();
                accepted += 1;
                length += 1;
                boundary = result.boundary;
                occupied(&boundary, maxima);
            }
            lengths.push(length);
            let terminal = boundary.terminal.map_or_else(|| "step_limit".to_owned(), |value| format!("{value:?}"));
            *terminals.entry(terminal).or_default() += 1;
        }
    }
    let elapsed = wall.elapsed();
    let total_length: usize = lengths.iter().sum();
    Ok(json!({
        "episodes": lengths.len(), "accepted_actions": accepted, "step_observation_seconds": measured.as_secs_f64(),
        "wall_seconds": elapsed.as_secs_f64(), "accepted_actions_per_second": accepted as f64 / measured.as_secs_f64().max(f64::MIN_POSITIVE),
        "episodes_per_second": lengths.len() as f64 / elapsed.as_secs_f64().max(f64::MIN_POSITIVE),
        "mean_length": total_length as f64 / lengths.len() as f64, "max_length": lengths.iter().copied().max().unwrap_or(0),
        "terminal_histogram": terminals,
    }))
}

fn batch_configs(slots: usize) -> Vec<StandardRunConfig> {
    let characters = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
    (0..slots).map(|slot| {
        let mut config = StandardRunConfig::new(characters[(slot / (slots / 4)) % 4], 20, Partition::Test);
        config.record_replay = true;
        config
    }).collect()
}

fn batch_phase(slots: usize, steps: usize, workers: usize, maxima: &mut BTreeMap<String, usize>) -> Result<Value, String> {
    let configs = batch_configs(slots);
    let batch_config = BatchConfig { workers, autoreset: AutoresetMode::NextStep, fault_policy: FaultPolicy::Freeze };
    let mut batch = SessionBatch::new_run(configs, RunSession::new_standard, batch_config).map_err(|error| format!("batch create: {error:?}"))?;
    let seeds = (0..slots).map(|slot| Some(0xA20_0000 + slot as u64)).collect::<Vec<_>>();
    let entries = vec![None; slots];
    let mut rows = batch.reset(&seeds, &entries, None).map_err(|error| format!("batch reset: {error:?}"))?;
    for row in &rows {
        if row.fault.is_some() { return Err("batch reset produced a fault row".into()) }
        occupied(&row.boundary, maxima);
    }
    let mut tokens = rows.iter().map(|row| row.boundary.token).collect::<Vec<_>>();
    let mut measured = Duration::ZERO;
    let wall = Instant::now();
    for tick in 0..steps {
        let actions = rows.iter().enumerate().map(|(slot, row)| {
            if row.boundary.terminal.is_some() { RunActionIndex(0) }
            else { RunActionIndex(r1(seeds[slot].unwrap(), tick as u64, &row.boundary.observation.action_mask) as i64) }
        }).collect::<Vec<_>>();
        let start = Instant::now();
        rows = batch.step(&tokens, &actions).map_err(|error| format!("batch step {tick}: {error:?}"))?;
        for row in &rows {
            if row.fault.is_some() { return Err(format!("batch step {tick} produced a fault row")) }
            let _observation_read = row.boundary.observation.action_mask.len();
        }
        measured += start.elapsed();
        for row in &rows { occupied(&row.boundary, maxima); }
        tokens.clear();
        tokens.extend(rows.iter().map(|row| row.boundary.token));
    }
    let elapsed = wall.elapsed();
    let rss_after_budget = rss();
    Ok(json!({ "slots": slots, "batch_calls": steps, "slot_steps": slots * steps, "workers": workers,
        "step_observation_seconds": measured.as_secs_f64(), "wall_seconds": elapsed.as_secs_f64(),
        "slot_steps_per_second": (slots * steps) as f64 / measured.as_secs_f64().max(f64::MIN_POSITIVE),
        "rss_after_budget": rss_after_budget }))
}

fn identity() -> Result<Value, String> {
    let revision = std::env::var("GIT_REVISION").ok().filter(|value| !value.is_empty()).or_else(|| {
        std::process::Command::new("git").args(["rev-parse", "HEAD"]).output().ok()
            .filter(|output| output.status.success()).map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    });
    let fingerprint = standard_run_observation_spec(&capacities()).map(|spec| spec.fingerprint().iter().map(|byte| format!("{byte:02x}")).collect::<String>())
        .map_err(|error| format!("observation spec: {error:?}"));
    let fingerprint = fingerprint?;
    Ok(json!({"revision": revision, "rules_id": RUN_RULES_ID_STANDARD,
        "observation_fingerprint": fingerprint,
        "host_cpu_count": std::thread::available_parallelism().map(usize::from).ok() })
    )
}

fn execute(options: Options) -> Result<Value, String> {
    let mut maxima = BTreeMap::new();
    let s = run_single(options.seeds, &mut maxima)?;
    let memory_after_s = rss();
    let mut batch_runs = Vec::new();
    for workers in options.workers {
        batch_runs.push(batch_phase(64, options.batch_steps, workers, &mut maxima)?);
    }
    let memory_after_b64 = batch_runs.last().expect("three batch worker configurations")["rss_after_budget"].clone();
    let mut m256 = batch_phase(256, 50, 16, &mut maxima)?;
    let memory_after_b256 = m256["rss_after_budget"].clone();
    m256["rss_after_reset_plus_50_steps"] = memory_after_b256.clone();
    m256.as_object_mut().expect("batch phase object").remove("rss_after_budget");
    let before = memory_after_b64["VmRSS_kib"].as_u64();
    let after = memory_after_b256["VmRSS_kib"].as_u64();
    let delta_per_slot = before.zip(after).map(|(a, b)| (b as f64 - a as f64) / 192.0);
    let limits = capacity_limits(&capacities());
    let capacity_rows = CAPACITY_NAMES.iter().zip(limits).map(|(name, capacity)| {
        let used = maxima.get(*name).copied().unwrap_or(0);
        json!({"group": name, "capacity": capacity, "max_occupied": used, "headroom": capacity.saturating_sub(used)})
    }).collect::<Vec<_>>();
    Ok(json!({"schema_id":"spire-gym-a20-checks-v1", "identity": identity()?, "S": s, "B": batch_runs,
        "M":{"rss_after_S": memory_after_s, "rss_after_B64": memory_after_b64, "B256": m256,
            "rss_delta_per_added_slot_kib": delta_per_slot}, "capacity": capacity_rows }))
}

fn main() {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let options = parse_args()?;
        let value = execute(options.clone())?;
        let summary = format!("a20_checks: S {} episodes / {} actions; B {} worker configurations; C {} capacity groups",
            value["S"]["episodes"], value["S"]["accepted_actions"], value["B"].as_array().map_or(0, Vec::len), value["capacity"].as_array().map_or(0, Vec::len));
        if let Some(path) = options.out {
                let text = serde_json::to_string_pretty(&value).expect("JSON value serializes");
            fs::write(path, format!("{text}\n")).map_err(|error| format!("write JSON output: {error}"))?;
        } else {
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();
            serde_json::to_writer_pretty(&mut lock, &value).map_err(|error| format!("serialize JSON: {error}"))?;
            writeln!(lock).map_err(|error| format!("write JSON newline: {error}"))?;
        }
        eprintln!("{summary}");
        Ok(())
    }));
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => { eprintln!("a20_checks: {error}"); std::process::exit(1); }
        Err(_) => { eprintln!("a20_checks: panic"); std::process::exit(1); }
    }
}
