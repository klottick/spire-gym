//! M36 from-start Heart acceptance (design §3.1). Every route starts from public seed `s` with the default Standard
//! creation (`StandardRunConfig::new`: experienced, full Neow) and exactly this supply: current and maximum HP 900 and
//! all three keys before the first boundary (`diagnostics::supply_hp_and_keys`), plus the wound hook
//! (`diagnostics::supplied_wound_enemies`) at combat decisions. Supplied, native and source-grounded; never a normal-reach
//! or strategy claim. Seeds may change only on Death or the action cap, with a comment here and a profile note.
use super::m36_freeze::{EpisodeDigest, HEART_PINS};
use super::standard_traversal::{assert_heart_rewards, drive, needs_supplied_wound, replay_wounded_supplied, simple_action, CHARACTERS};
use crate::run_session::{create_run, RunProfile};
use crate::{AutoresetMode, BatchConfig, FaultPolicy, RunActionIndex, RunBoundary, RunObservation, RunSession, SessionBatch, StandardRunConfig};
use spire_core::{combat::rng::Partition, content::Character, run::{diagnostics::{supplied_wound_enemies, supply_hp_and_keys}, Run, RunTerminal}};

const CAP: usize = 12_000;
const SUPPLIED_HP: u32 = 900;

/// The public seed of a cell: `0xB0 + character index`, `+8` at A20 (no cell has been re-seeded).
pub(super) fn cell_seed(character_index: usize, ascension: u8) -> u64 { 0xB0 + character_index as u64 + if ascension == 20 { 8 } else { 0 } }
fn config(character: Character, ascension: u8) -> StandardRunConfig {
    let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
    config.record_replay = true;
    config
}
/// The supplied start: the public seed world (`create_run`, as `reset(Some(s))` builds it) plus the supply.
pub(super) fn supplied_start(character: Character, ascension: u8, seed: u64) -> Run {
    let mut run = create_run(RunProfile::Standard, &config(character, ascension).session_config(), seed, 0).unwrap();
    supply_hp_and_keys(&mut run, SUPPLIED_HP, true);
    run
}
pub(super) fn supplied_session(character: Character, ascension: u8, seed: u64) -> RunSession {
    RunSession::from_standard_supplied(config(character, ascension), supplied_start(character, ascension, seed), seed).unwrap()
}

/// Observation columns the supply may change: player HP and maximum HP, and the three key flags.
fn supply_columns(group: &str, field: &str) -> bool {
    matches!((group, field), ("player", "hp" | "max_hp") | ("context", "ruby" | "sapphire" | "emerald"))
}
/// Asserts the public `reset(Some(s))` Run plus the same supply equals the supplied start, and the first public
/// observations agree outside the supplied columns.
fn assert_public_start(character: Character, ascension: u8, seed: u64, start: &Run, supplied_first: &RunObservation) {
    let mut public = RunSession::new_standard(config(character, ascension)).unwrap();
    let public_first = public.reset(Some(seed)).unwrap().observation;
    let mut public_run = public.run_for_test().unwrap().clone();
    supply_hp_and_keys(&mut public_run, SUPPLIED_HP, true);
    assert!(public_run == *start, "{character:?} A{ascension} s={seed}: the public seed world plus the supply is the supplied start");
    let spec = crate::standard_run_observation_spec(&crate::RunCapacities::standard_default()).unwrap().descriptor();
    let mut differing = Vec::new();
    assert_eq!(public_first.action_mask, supplied_first.action_mask);
    for (p, s) in public_first.groups.iter().zip(&supplied_first.groups) {
        assert_eq!((p.key, &p.shape, &p.known, &p.occupied), (s.key, &s.shape, &s.known, &s.occupied));
        let fields = spec["tensors"].as_array().unwrap().iter().find(|t| t["key"] == p.key).unwrap()["fields"].as_array().unwrap().clone();
        let cols = *p.shape.last().unwrap();
        for (i, (a, b)) in p.values.iter().zip(&s.values).enumerate() {
            let field = fields[i % cols].as_str().unwrap();
            if a != b && !supply_columns(p.key, field) { differing.push(format!("{}[{}].{field}: {a} vs {b}", p.key, i / cols)); }
        }
    }
    assert!(differing.is_empty(), "{character:?} A{ascension}: first observations differ outside the supply: {differing:?}");
}

fn terminal_name(t: Option<RunTerminal>) -> String { t.map(|t| format!("{t:?}")).unwrap_or_else(|| "none".to_owned()) }

/// One supplied route driven by `simple_action`; returns the session, its start, the pin and the wound schedule.
struct Route { session: RunSession, start: Run, pin: (usize, String, i64, String), wounds: Vec<usize>, boss_fights: usize, reward_total: i32, floors: (usize, usize) }
fn route(character: Character, ascension: u8, seed: u64) -> Route {
    let start = supplied_start(character, ascension, seed);
    let mut session = RunSession::from_standard_supplied(config(character, ascension), start.clone(), seed).unwrap();
    let mut seen: Vec<(RunObservation, usize)> = Vec::new();
    let progress = drive(&mut session, CAP, |_| false, |obs| { let i = simple_action(obs); seen.push((obs.clone(), i)); i })
        .unwrap_or_else(|f| panic!("{character:?} A{ascension} s={seed}: route failed after {} steps at {}: {} ({})", f.progress.steps, f.checkpoint, f.error, f.action));
    let record = session.record().unwrap().expect("record_replay is enabled");
    assert_eq!((record.steps.len(), seen.len()), (progress.steps, progress.steps));
    if record.terminal != Some(RunTerminal::HeartVictory) {
        let at = session.reproduction().unwrap().latest_decision.and_then(Result::ok).map(|d| format!("act={} floor={}", d.act, d.floor));
        panic!("{character:?} A{ascension} s={seed:#x}: terminal {:?} after {} steps at {at:?}", record.terminal, progress.steps);
    }
    let mut digest = EpisodeDigest::new("standard-heart-supplied");
    for ((obs, index), step) in seen.iter().zip(&record.steps) {
        digest.boundary(obs);
        digest.step(*index as i64, i64::from(step.reward), &terminal_name(step.terminal));
    }
    let last = session.decision().unwrap();
    digest.boundary(&last.observation);
    let (steps, ret) = (digest.steps, digest.ret);
    let (reward_total, normal, act4) = assert_heart_rewards(&progress, ascension);
    Route { pin: (steps, terminal_name(last.terminal), ret, digest.finish()), session, start, wounds: progress.wound_counts, boss_fights: progress.boss_fights, reward_total, floors: (normal, act4) }
}

#[test]
fn m36_heart_acceptance() {
    let mut lines = Vec::new();
    let mut pins = Vec::new();
    for (ci, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0u8, 20] {
            let seed = cell_seed(ci, ascension);
            let r = route(character, ascension, seed);
            let first = RunSession::from_standard_supplied(config(character, ascension), r.start.clone(), seed).unwrap().decision().unwrap().observation;
            assert_public_start(character, ascension, seed, &r.start, &first);
            let record = r.session.record().unwrap().unwrap();
            let last = r.session.decision().unwrap();
            // HeartVictory by the final Proceed (step reward 110 = Act 4 floor +10 and Heart +100), terminated not truncated.
            assert_eq!(record.terminal, Some(RunTerminal::HeartVictory), "{character:?} A{ascension}");
            assert_eq!(record.steps.last().map(|s| (s.reward, s.terminal)), Some((110, Some(RunTerminal::HeartVictory))));
            assert!(record.fault.is_none() && !record.closed);
            let final_view = r.session.reproduction().unwrap().latest_decision.unwrap().unwrap();
            assert_eq!(final_view.floor, if ascension == 20 { 56 } else { 55 }, "TrueVictory floor");
            assert_eq!(r.boss_fights, if ascension == 20 { 2 } else { 1 }, "Act 3 boss fights");
            assert_eq!(r.reward_total as i64, r.pin.2);
            // Supplied replay: same start and wound schedule reproduce the terminal, every reward and the final view.
            let replay = replay_wounded_supplied(&record, r.start.clone(), &r.wounds);
            assert_eq!(replay.accepted_count, record.steps.len());
            assert_eq!(replay.rewards, record.steps.iter().map(|s| s.reward).collect::<Vec<_>>());
            assert_eq!(replay.terminals.last().copied().flatten(), Some(RunTerminal::HeartVictory));
            assert_eq!(replay.final_decision, final_view, "replay reproduces the final view");
            // Determinism: a second construction gives a byte-identical record.
            let again = route(character, ascension, seed);
            assert_eq!(again.session.record().unwrap().unwrap().to_json().unwrap(), record.to_json().unwrap(), "{character:?} A{ascension}: second construction");
            assert_eq!(again.pin, r.pin);
            let (steps, terminal, ret, digest) = r.pin.clone();
            lines.push(format!("{character:?} A{ascension} s={seed:#x}: steps={steps} terminal={terminal} return={ret} floors={:?} wounds={} last={:?}",
                r.floors, r.wounds.len(), last.terminal));
            pins.push(format!("    Pin {{ name: \"{character:?}-A{ascension}\", seed: {seed:#x}, steps: {steps}, terminal: \"{terminal}\", ret: {ret}, digest: \"{digest}\" }},"));
            let pin = HEART_PINS.iter().find(|p| p.name == format!("{character:?}-A{ascension}"))
                .unwrap_or_else(|| panic!("{character:?} A{ascension}: no Heart pin; every acceptance cell is pinned"));
            assert_eq!((pin.seed, pin.steps, pin.terminal, pin.ret, pin.digest), (seed, steps, terminal.as_str(), ret, digest.as_str()),
                "{character:?} A{ascension}: Heart pin moved; follow the change protocol (docs/delivery.md)");
        }
    }
    for line in &lines { eprintln!("m36_heart_acceptance: {line}"); }
    eprintln!("m36_heart_acceptance pins:\n{}", pins.join("\n"));
    assert_eq!(HEART_PINS.len(), 8, "all 8 Heart routes are pinned");
}

/// The 8 routes as one SAME_STEP batch (workers 4, Freeze): each slot's terminal, return and `final_obs` equal its scalar route.
#[test]
fn m36_heart_acceptance_batch() {
    let cells: Vec<(Character, u8, u64)> = CHARACTERS.into_iter().enumerate()
        .flat_map(|(ci, c)| [0u8, 20].map(move |a| (c, a, cell_seed(ci, a)))).collect();
    let scalar: Vec<(Option<RunTerminal>, i64, RunObservation)> = cells.iter().map(|&(c, a, s)| {
        let r = route(c, a, s); let last = r.session.decision().unwrap(); (last.terminal, r.pin.2, last.observation)
    }).collect();
    let n = cells.len();
    let sessions = cells.iter().map(|&(c, a, s)| supplied_session(c, a, s)).collect();
    let mut batch = SessionBatch::build(sessions, vec![None; n], BatchConfig { workers: 4, autoreset: AutoresetMode::SameStep, fault_policy: FaultPolicy::Freeze }, true).unwrap();
    batch.adopt_started();
    let mut cur: Vec<RunBoundary> = (0..n).map(|k| batch.decision(k).unwrap()).collect();
    let mut done: Vec<Option<(Option<RunTerminal>, i64, RunObservation)>> = vec![None; n];
    let mut returns = vec![0i64; n];
    for _ in 0..CAP {
        if done.iter().all(Option::is_some) { break }
        for k in 0..n {
            if done[k].is_none() && needs_supplied_wound(&cur[k].observation) { cur[k] = batch.sessions[k].apply_supplied(supplied_wound_enemies).unwrap(); }
        }
        let actions: Vec<RunActionIndex> = (0..n).map(|k| RunActionIndex(if done[k].is_none() { simple_action(&cur[k].observation) }
            else { cur[k].observation.action_mask.iter().position(|m| *m != 0).unwrap() } as i64)).collect();
        let rows = batch.step(&cur.iter().map(|b| b.token).collect::<Vec<_>>(), &actions).unwrap();
        for k in 0..n {
            if done[k].is_none() {
                returns[k] += i64::from(rows[k].reward);
                if rows[k].terminated {
                    let fin = rows[k].final_boundary.as_ref().expect("SAME_STEP final_obs");
                    done[k] = Some((fin.terminal, returns[k], fin.observation.clone()));
                }
            }
            cur[k] = rows[k].boundary.clone();
        }
    }
    for k in 0..n {
        let (c, a, s) = cells[k];
        assert_eq!(done[k].as_ref(), Some(&scalar[k]), "{c:?} A{a} s={s:#x}: batch slot equals its scalar route");
    }
}

/// P10 (design §3.1): the same supplied route for 4 characters × A0–A20, run once by runner B
/// (`cargo test --release --locked --offline -p spire-env m36_heart_sweep_all_ascensions -- --ignored --nocapture`).
/// A0 and A20 use the acceptance seeds; A1–A19 use `0x3600 + 16·ascension + character index`, except two cells re-seeded
/// (`+0x100`) because their first seed ended in Death under `simple_action`, a policy outcome (Silent A5 s=0x3651 at
/// act 3 floor 36; Defect A4 s=0x3642 at act 2 floor 22; M36 sweep run 2).
fn sweep_seed(character_index: usize, ascension: u8) -> u64 {
    if ascension == 0 || ascension == 20 { return cell_seed(character_index, ascension) }
    let seed = 0x3600 + 16 * u64::from(ascension) + character_index as u64;
    if matches!((character_index, ascension), (1, 5) | (2, 4)) { seed + 0x100 } else { seed }
}
#[test]
#[ignore]
fn m36_heart_sweep_all_ascensions() {
    let mut lines = Vec::new();
    let mut failures = Vec::new();
    for (ci, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in 0u8..=20 {
            let seed = sweep_seed(ci, ascension);
            let outcome = std::panic::catch_unwind(|| {
                let r = route(character, ascension, seed);
                let record = r.session.record().unwrap().unwrap();
                let final_view = r.session.reproduction().unwrap().latest_decision.unwrap().unwrap();
                let replay = replay_wounded_supplied(&record, r.start.clone(), &r.wounds);
                let mut problems = Vec::new();
                if record.terminal != Some(RunTerminal::HeartVictory) { problems.push(format!("terminal {:?}", record.terminal)); }
                if record.fault.is_some() { problems.push("fault".to_owned()); }
                // A Secret Portal (Act 3 event) skips to the boss, so the final floor may be lower; Act 4 credits are checked
                // by `assert_heart_rewards` inside `route`.
                if final_view.floor > if ascension == 20 { 56 } else { 55 } { problems.push(format!("final floor {}", final_view.floor)); }
                if r.boss_fights != if ascension == 20 { 2 } else { 1 } { problems.push(format!("{} Act 3 boss fights", r.boss_fights)); }
                if replay.final_decision != final_view || replay.rewards != record.steps.iter().map(|s| s.reward).collect::<Vec<_>>() { problems.push("replay differs".to_owned()); }
                (r.pin.0, r.pin.2, final_view.floor, problems)
            });
            match outcome {
                Ok((steps, ret, floor, problems)) => {
                    lines.push(format!("{character:?} A{ascension} s={seed:#x}: steps={steps} return={ret} final_floor={floor} {}", if problems.is_empty() { "OK".to_owned() } else { problems.join(", ") }));
                    if !problems.is_empty() { failures.push(format!("{character:?} A{ascension} s={seed:#x}: {}", problems.join(", "))); }
                }
                Err(panic) => {
                    let message = panic.downcast_ref::<String>().cloned().or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    lines.push(format!("{character:?} A{ascension} s={seed:#x}: PANIC {message}"));
                    failures.push(format!("{character:?} A{ascension} s={seed:#x}: {message}"));
                }
            }
        }
    }
    for line in &lines { eprintln!("m36_heart_sweep: {line}"); }
    assert!(failures.is_empty(), "sweep failures (a Death or cap re-seeds with disclosure; a fault is a bug): {failures:#?}");
}

/// Design §3.3 late-game workload (I6), run in release by task 8
/// (`cargo test --release --locked --offline -p spire-env m36_a20_route_measure -- --ignored --nocapture`). Per character
/// at A20: the wounded acceptance route and a supplied full-length variant (same public seed, HP 900 and keys, but no
/// wound until a fight passes 15 player turns; cap 20,000 actions). One JSON line per variant: accepted actions/s (time
/// covers only `step` plus the boundary read), maximum occupied rows per group, record JSON bytes and exact-checkpoint
/// heap bytes at the last boundary. Diagnostic numbers, never gates.
#[test]
#[ignore]
fn m36_a20_route_measure() {
    use super::standard_exact_cost::retained_bytes;
    use super::standard_traversal::field;
    for (ci, character) in CHARACTERS.into_iter().enumerate() {
        let seed = cell_seed(ci, 20);
        for (variant, min_turn, cap) in [("wounded", 0i64, CAP), ("full_length", 15, 20_000)] {
            let mut session = supplied_session(character, 20, seed);
            let mut boundary = session.decision().unwrap();
            let (mut steps, mut wounds, mut timed) = (0usize, 0usize, std::time::Duration::ZERO);
            let mut max_rows: std::collections::BTreeMap<&'static str, usize> = Default::default();
            let mut observe = |o: &RunObservation| for g in &o.groups {
                if let Some(rows) = &g.occupied { let n = rows.iter().filter(|r| **r != 0).count(); let e = max_rows.entry(g.key).or_default(); *e = (*e).max(n); }
            };
            observe(&boundary.observation);
            while boundary.terminal.is_none() && steps < cap {
                if needs_supplied_wound(&boundary.observation) && field(&boundary.observation, "player", 0, 6) > min_turn {
                    boundary = session.apply_supplied(supplied_wound_enemies).unwrap(); wounds += 1;
                }
                let index = simple_action(&boundary.observation);
                let t = std::time::Instant::now();
                let result = session.step(boundary.token, RunActionIndex(index as i64)).unwrap();
                std::hint::black_box(result.boundary.observation.groups.len());
                timed += t.elapsed();
                boundary = result.boundary;
                observe(&boundary.observation);
                steps += 1;
            }
            let record_bytes = session.record().unwrap().unwrap().to_json().unwrap().len();
            let (_, checkpoint_bytes) = retained_bytes(|| session.exact_checkpoint().unwrap());
            let line = serde_json::json!({"m36_a20_route_measure": variant, "character": format!("{character:?}"), "seed": seed, "steps": steps,
                "terminal": terminal_name(boundary.terminal), "wounds": wounds, "accepted_actions_per_s": steps as f64 / timed.as_secs_f64(),
                "max_occupied_rows": max_rows, "record_json_bytes": record_bytes, "exact_checkpoint_heap_bytes": checkpoint_bytes});
            println!("{line}");
        }
    }
}

/// M36 sweep finding (red first): the supplied wound is a NORMAL hit of `hp - 1`, so a damage increase (The Boot, Wrath,
/// Vulnerable) could kill the target outside a step, leaving a won combat the Run never settles ("nonterminal Run has no
/// legal actions"; Ironclad A5 s=0x3650 with The Boot at a 3-HP Byrd). The hook must leave every wounded enemy alive.
#[test]
fn m36_supplied_wound_never_kills() {
    let mut session = supplied_session(Character::Ironclad, 5, 0x3650);
    let mut boundary = session.decision().unwrap();
    let mut steps = 0;
    while boundary.terminal.is_none() && steps < CAP {
        if needs_supplied_wound(&boundary.observation) {
            boundary = session.apply_supplied(supplied_wound_enemies).expect("a supplied wound leaves the combat undecided");
            let view = session.run_for_test().unwrap().decision().unwrap().combat.expect("still in combat");
            assert!(view.outcome.is_none(), "step {steps}: the wound decided the combat");
        }
        boundary = session.step(boundary.token, RunActionIndex(simple_action(&boundary.observation) as i64)).unwrap().boundary;
        steps += 1;
    }
    assert!(boundary.terminal.is_some(), "the route ends");
}
