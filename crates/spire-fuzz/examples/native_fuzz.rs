use std::{collections::BTreeMap, env, fs, path::PathBuf, process::Command, thread, time::{Duration, Instant}};
use serde_json::json;
use spire_core::content::Character;
use spire_fuzz::{CatalogCombat, EpisodeOptions, FullRun, Policy, RunSummary, RUN_PROFILES,
    failure_key, generated_catalog_json, generated_standard_catalog_json, mix64, read_repro, run_episode, run_repro, shrink_failure_case, write_failure};

#[cfg(feature = "diagnostics")]
use spire_core::combat::enemies::A0_ENCOUNTER_SETUPS;
#[cfg(feature = "diagnostics")]
use spire_fuzz::{CompleteCombat, CompleteItemsRun, COMPLETE_ITEMS_RUN_PROFILES};

#[derive(Clone)]
struct Options {
    seed_start: u64,
    seed_end: u64,
    threads: usize,
    budget: Duration,
    out: PathBuf,
    drivers: Vec<String>,
    weights: DriverWeights,
    policy: Option<Policy>,
    catalogs: Vec<PathBuf>,
    combat_max_steps: usize,
    run_max_steps: usize,
    all_legal_every: usize,
    determinism_every: u64,
    replay_every: u64,
    min_combat: u64,
    min_runs: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DriverWeights { catalog: u32, complete: u32, run: u32, items: u32, standard_combat: u32, search: u32 }
impl Default for DriverWeights { fn default() -> Self { Self { catalog: 1, complete: 1, run: 1, items: 1, standard_combat: 1,
    search: if cfg!(feature = "diagnostics") { 1 } else { 0 } } } }
impl DriverWeights {
    fn get(self, driver: &str) -> u32 {
        match driver { "combat" => self.catalog, "complete" => self.complete, "run" => self.run, "items" => self.items,
            "standard-combat" => self.standard_combat, "search" => self.search, _ => 0 }
    }
}

fn main() {
    if let Err(error) = run() { eprintln!("native_fuzz: {error}"); std::process::exit(2); }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".into());
    if command == "repro" {
        let path = args.next().ok_or_else(usage)?;
        let case = read_repro(path)?;
        run_repro(&case)?;
        println!("PASS {} seed={} policy={} actions={}", case.driver, case.seed, case.policy.name(), case.actions.len());
        return Ok(());
    }
    if command != "run" { return Err(usage()); }
    let options = parse_options(args.collect())?;
    run_batch(options)
}

fn parse_options(args: Vec<String>) -> Result<Options, String> {
    let mut options = Options { seed_start: 0, seed_end: 0, threads: 2, budget: Duration::ZERO,
        out: PathBuf::from("native-fuzz-out"),
        drivers: if cfg!(feature = "diagnostics") { vec!["combat".into(), "complete".into(), "run".into(), "items".into(), "standard-combat".into(), "search".into()] } else { vec!["combat".into(), "run".into(), "standard-combat".into()] },
        weights: DriverWeights::default(), policy: None, catalogs: Vec::new(), combat_max_steps: 2_000,
        run_max_steps: 20_000, all_legal_every: 0, determinism_every: 1, replay_every: 1, min_combat: 0, min_runs: 0 };
    let mut max_steps_override = None;
    let mut combat_steps_override = None;
    let mut run_steps_override = None;
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        let value = || args.get(index + 1).ok_or_else(|| format!("missing value for {flag}"));
        match flag {
            "--seed-start" => options.seed_start = value()?.parse().map_err(|_| "invalid --seed-start")?,
            "--seed-end" => options.seed_end = value()?.parse().map_err(|_| "invalid --seed-end")?,
            "--threads" => options.threads = value()?.parse().map_err(|_| "invalid --threads")?,
            "--budget-secs" => options.budget = Duration::from_secs(value()?.parse().map_err(|_| "invalid --budget-secs")?),
            "--out" => options.out = PathBuf::from(value()?),
            "--drivers" => options.drivers = value()?.split(',').map(str::to_owned).collect(),
            "--weights" => options.weights = parse_weights(value()?)?,
            "--policy" => options.policy = Some(Policy::from_name(value()?).ok_or_else(|| "unknown policy".to_owned())?),
            "--catalog" => options.catalogs.push(PathBuf::from(value()?)),
            "--max-steps" => max_steps_override = Some(parse_positive(value()?, "--max-steps")?),
            "--combat-max-steps" => combat_steps_override = Some(parse_positive(value()?, "--combat-max-steps")?),
            "--run-max-steps" => run_steps_override = Some(parse_positive(value()?, "--run-max-steps")?),
            "--all-legal-every" => options.all_legal_every = value()?.parse().map_err(|_| "invalid --all-legal-every")?,
            "--determinism-every" => options.determinism_every = value()?.parse().map_err(|_| "invalid --determinism-every")?,
            "--replay-every" => options.replay_every = value()?.parse().map_err(|_| "invalid --replay-every")?,
            "--min-combat" => options.min_combat = value()?.parse().map_err(|_| "invalid --min-combat")?,
            "--min-runs" => options.min_runs = value()?.parse().map_err(|_| "invalid --min-runs")?,
            other => return Err(format!("unknown option {other}\n{}", usage())),
        }
        index += 2;
    }
    if options.threads == 0 { return Err("--threads must be positive".into()); }
    if options.seed_start > options.seed_end { return Err("--seed-start must be <= --seed-end".into()); }
    if options.drivers.iter().any(|driver| !["combat", "complete", "run", "items", "standard-combat", "search"].contains(&driver.as_str())) { return Err("--drivers accepts combat,complete,run,items,standard-combat,search".into()); }
    if options.drivers.iter().collect::<std::collections::BTreeSet<_>>().len() != options.drivers.len() {
        return Err("--drivers cannot contain duplicates".into());
    }
    if options.drivers.iter().any(|driver| ["complete", "items", "search"].contains(&driver.as_str())) && !cfg!(feature = "diagnostics") {
        return Err("complete, items, and search drivers require --features diagnostics".into());
    }
    if weighted_driver_schedule(&options.drivers, options.weights)?.is_empty() { return Err("selected drivers have zero total weight".into()); }
    options.combat_max_steps = combat_steps_override.or(max_steps_override).unwrap_or(options.combat_max_steps);
    options.run_max_steps = run_steps_override.or(max_steps_override).unwrap_or(options.run_max_steps);
    Ok(options)
}

fn parse_positive(value: &str, option: &str) -> Result<usize, String> {
    let parsed = value.parse::<usize>().map_err(|_| format!("invalid {option}"))?;
    if parsed == 0 { return Err(format!("{option} must be positive")); }
    Ok(parsed)
}

fn parse_weights(value: &str) -> Result<DriverWeights, String> {
    let mut weights = DriverWeights::default();
    let mut seen = std::collections::BTreeSet::new();
    for assignment in value.split(',') {
        let (name, raw) = assignment.split_once('=').ok_or_else(|| "--weights uses name=weight pairs".to_owned())?;
        if !seen.insert(name) { return Err(format!("duplicate driver weight {name}")); }
        let weight = raw.parse::<u32>().map_err(|_| format!("invalid weight for {name}"))?;
        match name {
            "catalog" => weights.catalog = weight,
            "complete" => weights.complete = weight,
            "run" => weights.run = weight,
            "items" => weights.items = weight,
            "standard-combat" => weights.standard_combat = weight,
            "search" if cfg!(feature = "diagnostics") => weights.search = weight,
            "search" => return Err("search weight requires --features diagnostics".into()),
            _ => return Err(format!("unknown driver weight {name}")),
        }
    }
    Ok(weights)
}

fn weighted_driver_schedule(drivers: &[String], weights: DriverWeights) -> Result<Vec<String>, String> {
    let active = drivers.iter().filter_map(|driver| {
        let weight = weights.get(driver);
        (weight > 0).then_some((driver.clone(), weight))
    }).collect::<Vec<_>>();
    let total = active.iter().try_fold(0u32, |total, (_, weight)| total.checked_add(*weight))
        .ok_or_else(|| "driver weights overflow".to_owned())?;
    if total > 4096 { return Err("sum of --weights must be at most 4096".into()); }
    if total == 0 { return Ok(Vec::new()); }
    let mut current = vec![0i64; active.len()];
    let mut schedule = Vec::with_capacity(total as usize);
    for _ in 0..total {
        for (slot, (_, weight)) in active.iter().enumerate() { current[slot] += i64::from(*weight); }
        let selected = (0..active.len()).max_by_key(|slot| (current[*slot], std::cmp::Reverse(*slot))).unwrap();
        current[selected] -= i64::from(total);
        schedule.push(active[selected].0.clone());
    }
    Ok(schedule)
}

fn sampled(seed: u64, every: u64, domain: u64) -> bool {
    every > 0 && mix64(seed ^ domain) % every == 0
}

fn run_batch(options: Options) -> Result<(), String> {
    fs::create_dir_all(&options.out).map_err(|error| format!("create output directory: {error}"))?;
    let revision = env::var("SPIRE_FUZZ_REVISION").unwrap_or_else(|_| git_revision());
    let started = Instant::now();
    let worker_count = options.threads;
    let mut workers = Vec::new();
    for worker in 0..worker_count {
        let options = options.clone();
        let started = started;
        workers.push(thread::spawn(move || worker_run(worker, worker_count, options, started)));
    }
    let mut results = Vec::new();
    for worker in workers { results.extend(worker.join().map_err(|_| "native fuzz worker panicked".to_owned())??); }
    results.sort_by(|left, right| (left.seed, left.driver.as_str(), left.labels.get("profile"), left.labels.get("character"), left.labels.get("encounter"), left.policy.name())
        .cmp(&(right.seed, right.driver.as_str(), right.labels.get("profile"), right.labels.get("character"), right.labels.get("encounter"), right.policy.name())));
    let mut summary = RunSummary::new(revision);
    summary.weights = BTreeMap::from([
        ("catalog".into(), options.weights.catalog),
        ("complete".into(), options.weights.complete),
        ("run".into(), options.weights.run),
        ("items".into(), options.weights.items),
    ]);
    if options.drivers.iter().any(|driver| driver == "standard-combat") {
        summary.weights.insert("standard-combat".into(), options.weights.standard_combat);
        summary.driver_descriptions.insert("standard-combat".into(), json!({"driver":"CatalogCombat","source":"generated standard-combat-v1 catalogs","specs":"four characters at A0 and A20"}));
    }
    if options.drivers.iter().any(|driver| driver == "search") {
        summary.weights.insert("search".into(), options.weights.search);
    }
    summary.combat_max_steps = options.combat_max_steps;
    summary.run_max_steps = options.run_max_steps;
    summary.all_legal_every = options.all_legal_every;
    summary.determinism_every = options.determinism_every;
    summary.replay_every = options.replay_every;
    for result in results { summary.add(result); }
    for failure in &mut summary.failures {
        if let Err(error) = shrink_failure_case(failure, Duration::from_secs(60)) {
            eprintln!("native_fuzz: shrink failed for {}: {error}", failure_key(failure));
        }
        failure.repro_path = Some(write_failure(&options.out, failure)?);
    }
    summary.elapsed_secs = started.elapsed().as_secs_f64();
    let bytes = serde_json::to_vec_pretty(&summary).map_err(|error| error.to_string())?;
    fs::write(options.out.join("summary.json"), bytes).map_err(|error| format!("write summary: {error}"))?;
    let combat = summary.episodes_by_driver.get("catalog").copied().unwrap_or(0)
        + summary.episodes_by_driver.get("complete").copied().unwrap_or(0)
        + summary.episodes_by_driver.iter().filter(|(driver, _)| driver.starts_with("standard-combat-")).map(|(_, episodes)| *episodes).sum::<u64>();
    let runs = summary.episodes_by_driver.get("run").copied().unwrap_or(0) + summary.episodes_by_driver.get("items").copied().unwrap_or(0);
    let items = summary.episodes_by_driver.get("items").copied().unwrap_or(0);
    println!("summary={} episodes={} steps={} combat={} runs={} items={} failures={} elapsed={:.3}s hash={}", options.out.join("summary.json").display(), summary.episodes, summary.steps, combat, runs, items, summary.failures.len(), summary.elapsed_secs, summary.stable_hash());
    if combat < options.min_combat || runs < options.min_runs {
        return Err(format!("minimum lanes unmet: combat {combat}/{} runs {runs}/{}", options.min_combat, options.min_runs));
    }
    Ok(())
}

struct CatalogSource { bytes: Vec<u8>, entries: usize }

fn worker_run(worker: usize, workers: usize, options: Options, started: Instant) -> Result<Vec<spire_fuzz::EpisodeResult>, String> {
    let use_catalog = options.drivers.iter().any(|driver| driver == "combat");
    let catalog_bytes = if !use_catalog {
        Vec::new()
    } else if options.catalogs.is_empty() {
        vec![generated_catalog_json(options.seed_start.wrapping_add(worker as u64))]
    } else {
        options.catalogs.iter().map(|path| fs::read(path).map_err(|error| format!("read catalog {}: {error}", path.display())))
            .collect::<Result<Vec<_>, _>>()?
    };
    let catalogs = catalog_bytes.into_iter().map(|bytes| {
        let driver = CatalogCombat::from_json(&bytes).map_err(|error| format!("catalog rejected: {}", error.message))?;
        Ok(CatalogSource { bytes, entries: driver.entry_count() })
    }).collect::<Result<Vec<_>, String>>()?;
    let schedule = weighted_driver_schedule(&options.drivers, options.weights)?;
    let total_catalog_entries = catalogs.iter().map(|catalog| catalog.entries).sum::<usize>();
    #[cfg(feature = "diagnostics")]
    let complete_cases = Character::ALL.into_iter().flat_map(|character|
        A0_ENCOUNTER_SETUPS.iter().map(move |(encounter, _, _)| (character, *encounter))).collect::<Vec<_>>();
    let mut results = Vec::new();
    let mut seed = match options.seed_start.checked_add(worker as u64) { Some(value) => value, None => return Ok(results) };
    let mut catalog_cursor = 0usize;
    #[cfg(feature = "diagnostics")]
    let mut complete_cursor = 0usize;
    #[cfg(feature = "diagnostics")]
    let mut items_cursor = 0usize;
    let mut run_cursor = 0usize;
    let mut standard_cursor = 0usize;
    let mut episode_ordinal = 0u64;
    let budgeted = !options.budget.is_zero();
    'seeds: while seed <= options.seed_end {
        if budgeted && started.elapsed() >= options.budget { break; }
        for scheduled_driver in &schedule {
            if budgeted && started.elapsed() >= options.budget { break 'seeds; }
            let episode_seed = mix64(seed ^ episode_ordinal.wrapping_mul(0xD6E8_FEB8_6659_FD93));
            match scheduled_driver.as_str() {
                "combat" => {
                    let entry = catalog_cursor % total_catalog_entries;
                    catalog_cursor += 1;
                    let (source, local_entry) = catalog_at(&catalogs, entry);
                    let mut driver = CatalogCombat::from_json(&source.bytes).map_err(|error| error.message)?;
                    driver.set_entry_cursor(local_entry);
                    results.push(run_one(Box::new(driver), episode_seed, episode_ordinal, &options));
                }
                "complete" => {
                    #[cfg(feature = "diagnostics")]
                    {
                        let (character, encounter) = complete_cases[complete_cursor % complete_cases.len()];
                        complete_cursor += 1;
                        results.push(run_one(Box::new(CompleteCombat::new(character, encounter)), episode_seed, episode_ordinal, &options));
                    }
                    #[cfg(not(feature = "diagnostics"))]
                    return Err("complete driver requires diagnostics".into());
                }
                "run" => {
                    let spec = RUN_PROFILES[run_cursor % RUN_PROFILES.len()];
                    run_cursor += 1;
                    let driver = FullRun::new(spec).map_err(|error| error.message)?;
                    results.push(run_one(Box::new(driver), episode_seed, episode_ordinal, &options));
                }
                "standard-combat" => {
                    let character = Character::ALL[standard_cursor % Character::ALL.len()];
                    let ascension = if standard_cursor / Character::ALL.len() % 2 == 0 { 0 } else { 20 };
                    standard_cursor += 1;
                    let bytes = generated_standard_catalog_json(episode_seed, character, ascension);
                    let mut driver = CatalogCombat::standard_from_json(&bytes, character, ascension).map_err(|error| error.message)?;
                    driver.set_entry_cursor(episode_seed as usize);
                    results.push(run_one(Box::new(driver), episode_seed, episode_ordinal, &options));
                }
                "search" => {
                    #[cfg(feature = "diagnostics")]
                    results.push(run_one(Box::new(spire_fuzz::SearchDriver::new()), episode_seed, episode_ordinal, &options));
                    #[cfg(not(feature = "diagnostics"))]
                    return Err("search driver requires --features diagnostics".into());
                }
                "items" => {
                    #[cfg(feature = "diagnostics")]
                    {
                        let profile = COMPLETE_ITEMS_RUN_PROFILES[items_cursor % COMPLETE_ITEMS_RUN_PROFILES.len()];
                        items_cursor += 1;
                        results.push(run_one(Box::new(CompleteItemsRun::new(profile)), episode_seed, episode_ordinal, &options));
                    }
                    #[cfg(not(feature = "diagnostics"))]
                    return Err("items driver requires --features diagnostics".into());
                }
                _ => unreachable!("validated driver name"),
            }
            episode_ordinal = episode_ordinal.wrapping_add(1);
        }
        seed = match seed.checked_add(workers as u64) { Some(value) => value, None => break };
    }
    Ok(results)
}

fn run_one(driver: Box<dyn spire_fuzz::Driver>, seed: u64, ordinal: u64, options: &Options) -> spire_fuzz::EpisodeResult {
    let policy = options.policy.unwrap_or_else(|| Policy::for_seed(seed ^ ordinal));
    let max_steps = if matches!(driver.class(), "run" | "items") { options.run_max_steps } else { options.combat_max_steps };
    let episode = EpisodeOptions { max_steps, all_legal_every: options.all_legal_every,
        determinism: sampled(seed, options.determinism_every, 0xA076_1D64_78BD_642F),
        replay: sampled(seed, options.replay_every, 0xE703_7ED1_A0B4_28DB), shrink_budget: Duration::ZERO,
        scripted_actions: None, continue_first: false };
    run_episode(driver, seed, policy, episode)
}

fn catalog_at(catalogs: &[CatalogSource], mut entry: usize) -> (&CatalogSource, usize) {
    for catalog in catalogs {
        if entry < catalog.entries { return (catalog, entry); }
        entry -= catalog.entries;
    }
    unreachable!("entry cursor is bounded by the combined catalog length")
}

fn git_revision() -> String {
    Command::new("git").args(["rev-parse", "HEAD"]).output().ok().filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok()).map(|revision| revision.trim().to_owned()).unwrap_or_else(|| "unknown".into())
}

fn usage() -> String {
        "usage: native_fuzz run --seed-start A --seed-end B --threads 2 --budget-secs S --out DIR [--drivers combat,complete,run,items,standard-combat,search] [--weights catalog=W,complete=W,run=W,items=W,standard-combat=W,search=W] [--policy P] [--catalog PATH]... [--max-steps N] [--all-legal-every N] [--min-combat N --min-runs N]\n       native_fuzz repro PATH".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "diagnostics")]
    #[test]
    fn search_is_selectable_and_included_in_default_diagnostics_drivers() {
        let defaults = parse_options(vec![]).unwrap();
        assert!(defaults.drivers.iter().any(|driver| driver == "search"));
        assert_eq!(defaults.weights.search, 1);
        let options = parse_options(vec!["--drivers".into(), "search".into(), "--seed-start".into(), "1".into(), "--seed-end".into(), "2".into()]).unwrap();
        assert_eq!(options.drivers, ["search"]);
        assert_eq!(weighted_driver_schedule(&options.drivers, options.weights).unwrap(), ["search"]);
    }

    #[test]
    fn cli_accepts_explicit_weighted_driver_schedule() {
        let drivers = if cfg!(feature = "diagnostics") { "combat,complete,run,items" } else { "combat,run" };
        let parsed = parse_options(vec![
            "--drivers".into(), drivers.into(),
            "--weights".into(), "catalog=2,complete=1,run=1,items=0".into(),
            "--seed-start".into(), "1".into(), "--seed-end".into(), "9".into(),
        ]);
        let options = parsed.unwrap();
        assert_eq!(options.threads, 2, "default campaign concurrency");
        let schedule = weighted_driver_schedule(&options.drivers, options.weights).unwrap();
        let expected = if cfg!(feature = "diagnostics") {
            vec!["combat", "complete", "run", "combat"]
        } else { vec!["combat", "run", "combat"] };
        assert_eq!(schedule, expected);
    }
}
