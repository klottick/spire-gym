//! M40c (spire-61l stage 3) coverage diagnostic: how often `PublicRoot` exists at a combat decision with Runic Dome, by
//! refusal class, act, room kind and enemy set, and how branch rollouts from those roots behave. Diagnostic only: no
//! thresholds, nothing asserted about the counts. Ignored release test:
//! `cargo test --release -p spire-env m40c_coverage -- --ignored --nocapture`.
//! Uses public items only, so the same file runs on the base revision (before) and the branch (after).
//!
//! Corpus A (natural runs, as M40a): supplied Standard Runs at A20, 900 HP refilled at map decisions, all keys, four
//! characters, seeds `0..SEEDS_A`, two variants (A1 seeded boss-relic choice, A2 prefers Runic Dome). Corpus C (forced
//! Dome): `standard-combat-v1` at A20 with Runic Dome added, every encounter the Standard catalog admits (monster, elite,
//! boss), four characters, the starter deck, seeds `0..SEEDS_C`, seeded uniform random legal play for up to
//! `C_DECISIONS` decisions. Rows are tagged `dome`/`no-dome` by the relic rows of the observation.
use super::m34s_transplant::{standard_combat_session, standard_entry};
use super::standard_traversal::{context, current_node_is_boss, field, legal_indices, short_creation, simple_action, CHARACTERS};
use crate::*;
use spire_core::{combat::{enemies::{EncounterKind, A0_ENCOUNTER_SETUPS}, rng::Partition}, content::Character, run::{diagnostics::standard_supplied_run_with, standard_combat_encounter, Run, RunRng}};
use std::collections::BTreeMap;

fn env_or(name: &str, default: u64) -> u64 { std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default) }
fn seeds_a() -> u8 { env_or("M40C_SEEDS_A", 24) as u8 }
fn seeds_c() -> u64 { env_or("M40C_SEEDS_C", 16) }
fn c_decisions() -> usize { env_or("M40C_C_DECISIONS", 300) as usize }
const A_STEP_LIMIT: usize = 20_000;
const ROLLOUT_EVERY: usize = 8;
const ROLLOUT_STEPS: usize = 64;
const LIMITS: SearchLimits = SearchLimits { max_live_branches: 8, max_branches: 100_000, max_steps_per_branch: 5_000, max_total_steps: 100_000_000 };

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self { Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1) }
    fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 }
    fn below(&mut self, n: usize) -> usize { (self.next() % n as u64) as usize }
}
fn vocab(family: &str, index: i64) -> String {
    usize::try_from(index).ok().and_then(|i| crate::standard_vocab_manifest()["families"][family].as_array()?.get(i)?.as_str().map(str::to_owned)).unwrap_or_default()
}
fn rows(obs: &RunObservation, group: &str) -> usize {
    obs.group(group).and_then(|g| g.occupied.as_ref()).map_or(0, |o| o.iter().take_while(|v| **v == 1).count())
}
fn has_dome(obs: &RunObservation) -> bool { (0..rows(obs, "relics")).any(|r| vocab("relic", field(obs, "relics", r, 0)) == "runic_dome") }
/// The sorted enemy kinds of the view (living or not): the fight's enemy set.
fn enemy_set(obs: &RunObservation) -> String {
    let mut kinds = (0..rows(obs, "enemies")).map(|r| vocab("enemy", field(obs, "enemies", r, 0))).collect::<Vec<_>>();
    kinds.sort();
    kinds.dedup();
    kinds.join("+")
}

#[derive(Default)]
struct Tally { roots: BTreeMap<String, u64>, enemies: BTreeMap<String, u64>, branch: BTreeMap<String, u64>, digests: Vec<(String, [u8; 32])> }
impl Tally {
    fn class(outcome: &Result<PublicRoot, RunSessionError>) -> String {
        match outcome { Ok(_) => "supported".to_owned(), Err(RunSessionError::UnsupportedBoundary(reason)) => format!("refused: {reason}"), Err(e) => format!("other: {e:?}") }
    }
    fn root(&mut self, cell: &str, set: &str, dome: bool, outcome: &Result<PublicRoot, RunSessionError>) {
        let class = Self::class(outcome);
        // Every supported root's digest in visit order, split by Dome: the no-Dome sequence must be unchanged by M40c.
        if let Ok(root) = outcome { self.digests.push((if dome { "dome" } else { "no-dome" }.to_owned(), root.digest())); }
        *self.roots.entry(format!("{cell} | {} | {class}", if dome { "dome" } else { "no-dome" })).or_default() += 1;
        if dome { *self.enemies.entry(format!("{set} | {class}")).or_default() += 1; }
    }
    fn count(&mut self, cell: &str, what: &str, n: u64) { *self.branch.entry(format!("{cell} | {what}")).or_default() += n; }
}

/// A seeded uniform-random-legal rollout of one branch of `root` (step only; every stop is counted by its kind).
fn rollout(tally: &mut Tally, cell: &str, root: PublicRoot, seed: u64) {
    let mut search = PublicSearch::new(root, SearchConfig { research_seed: seed, limits: LIMITS }).unwrap();
    let b = match search.branch(Some(0)) { Ok(b) => b, Err(e) => { tally.count(cell, &format!("branch refused: {e:?}"), 1); return } };
    let mut rng = Rng::new(seed ^ 0xB5AD_4ECE_DA1C_E2A9);
    let d = search.decision(b).unwrap();
    let (mut mask, mut token) = (d.observation.action_mask.clone(), d.token);
    tally.count(cell, "branches opened", 1);
    for _ in 0..ROLLOUT_STEPS {
        let legal = mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect::<Vec<_>>();
        if legal.is_empty() { break }
        let first = rng.below(legal.len());
        let mut taken = None;
        for k in 0..legal.len() {
            let index = legal[(first + k) % legal.len()];
            tally.count(cell, "branch steps attempted", 1);
            match search.step(b, token, RunActionIndex(index as i64), BranchView::Mask) {
                Ok(r) => { taken = Some(r); break }
                Err(RunSessionError::UnsupportedBoundary(reason)) => tally.count(cell, &format!("branch refusals: {reason}"), 1),
                Err(e) => { tally.count(cell, &format!("branch stop: {e:?}"), 1); return }
            }
        }
        let Some(r) = taken else { tally.count(cell, "branch stop: every legal action refused", 1); return };
        if r.terminated || r.truncated { tally.count(cell, "branch playouts ended", 1); return }
        (mask, token) = (r.action_mask, r.token);
    }
}

fn supplied_session(character: Character, seed: u8) -> RunSession {
    let creation = short_creation();
    let run = standard_supplied_run_with(character, 20, creation.profile(), 900, true, RunRng::new([seed; 32], Partition::Test, 0)).unwrap();
    let mut config = StandardRunConfig::new(character, 20, Partition::Test);
    config.creation = creation;
    RunSession::from_standard_supplied(config, run, u64::from(seed)).unwrap()
}

fn policy_a(obs: &RunObservation, rng: &mut Rng, prefer_dome: bool) -> usize {
    let legal = legal_indices(obs);
    let (_, _, room) = context(obs);
    let choice = |i: usize| vocab("choice", field(obs, "actions", i, 5));
    let verb = |i: usize| vocab("verb", field(obs, "actions", i, 0));
    match room.as_str() {
        "map" => legal[rng.below(legal.len())],
        "card_reward" => {
            let picks = legal.iter().copied().filter(|i| verb(*i) != "skip_card_offer" && !matches!(choice(*i).as_str(), "skip" | "leave")).collect::<Vec<_>>();
            if picks.is_empty() { simple_action(obs) } else { picks[rng.below(picks.len())] }
        }
        "boss_relic_choice" | "boss_relic_child" => {
            let relic = |i: usize| usize::try_from(field(obs, "actions", i, 2)).map(|row| vocab("relic", field(obs, "offers", row, 6))).unwrap_or_default();
            if prefer_dome { if let Some(i) = legal.iter().copied().find(|i| relic(*i) == "runic_dome") { return i } }
            let picks = legal.iter().copied().filter(|i| !choice(*i).contains("skip")).collect::<Vec<_>>();
            if picks.is_empty() { legal[0] } else { picks[rng.below(picks.len())] }
        }
        "combat" => {
            let free = legal.iter().any(|i| verb(*i) == "end_turn");
            if free && rng.below(4) == 0 { legal[rng.below(legal.len())] } else { simple_action(obs) }
        }
        _ => simple_action(obs),
    }
}
fn kind_of(obs: &RunObservation) -> &'static str {
    if current_node_is_boss(obs) { "boss" }
    else {
        let row = usize::try_from(field(obs, "current_node", 0, 0)).ok();
        match row.map(|r| vocab("map_kind", field(obs, "map_nodes", r, 3))).as_deref() { Some("elite") => "elite", Some("monster") => "normal", Some("boss") => "boss", _ => "event/other" }
    }
}

fn corpus_a(tally: &mut Tally) {
    for variant in ["A1", "A2-dome"] {
        for character in CHARACTERS {
            for seed in 0..seeds_a() {
                let mut session = supplied_session(character, seed);
                let mut rng = Rng::new(u64::from(seed) * 131 + character as u64 + 7);
                let mut boundary = session.decision().unwrap();
                let (mut steps, mut combat_decisions) = (0usize, 0usize);
                let (mut fight, mut set) = (-1i64, String::new());
                while boundary.terminal.is_none() && steps < A_STEP_LIMIT {
                    let (act, _, room) = context(&boundary.observation);
                    if room == "map" {
                        boundary = session.apply_supplied(|run| { spire_core::run::diagnostics::supply_hp_and_keys(run, 900, true); Ok(()) }).unwrap();
                    }
                    if room == "combat" {
                        let entered = field(&boundary.observation, "context", 0, 13);
                        if entered != fight { fight = entered; set = enemy_set(&boundary.observation); }
                        let kind = kind_of(&boundary.observation);
                        let cell = format!("A {variant} {} act{act} {kind}", character.as_str());
                        let outcome = session.public_root();
                        let dome = has_dome(&boundary.observation);
                        tally.root(&cell, &format!("A act{act} {kind} {set}"), dome, &outcome);
                        combat_decisions += 1;
                        if let Ok(root) = outcome { if dome && combat_decisions % ROLLOUT_EVERY == 1 { rollout(tally, &cell, root, seed as u64 * 7919 + combat_decisions as u64); } }
                    }
                    let index = policy_a(&boundary.observation, &mut rng, variant == "A2-dome");
                    boundary = session.step(boundary.token, RunActionIndex(index as i64)).unwrap().boundary;
                    steps += 1;
                }
            }
        }
    }
}

fn corpus_c(tally: &mut Tally) {
    let encounters = A0_ENCOUNTER_SETUPS.iter().filter(|(e, _, _)| standard_combat_encounter(*e).is_some()).map(|(e, act, kind)| (e.as_str(), *act, *kind)).collect::<Vec<_>>();
    let caps = RunCapacities::standard_default();
    for character in CHARACTERS {
        for (encounter, act, kind) in &encounters {
            let mut entry = standard_entry("e", character, 20, encounter, None);
            entry["relics"].as_array_mut().unwrap().push(serde_json::json!("runic_dome"));
            let mut session = standard_combat_session(vec![entry]);
            let kind = match kind { EncounterKind::Boss => "boss", EncounterKind::Elite => "elite", EncounterKind::Monster => "normal" };
            for seed in 0..seeds_c() {
                let b = session.reset(Some(seed), None).unwrap();
                let set = enemy_set(&b.observation);
                let dome = has_dome(&b.observation);
                let mut run: Run = session.run_for_test().unwrap().clone();
                let mut rng = Rng::new(seed * 977 + character as u64);
                let cell = format!("C {} act{act} {kind}", character.as_str());
                for decision in 0..c_decisions() {
                    if run.combat_scope_terminal().is_some() { break }
                    let outcome = PublicRoot::of(&run, &caps);
                    tally.root(&cell, &format!("C act{act} {kind} {encounter} {set}"), dome, &outcome);
                    if let Ok(root) = outcome { if decision % ROLLOUT_EVERY == 0 { rollout(tally, &cell, root, seed * 31 + decision as u64); } }
                    let d = run.decision().unwrap();
                    let action = d.candidates[rng.below(d.candidates.len())].action.clone();
                    if run.step(&action).is_err() { tally.count(&cell, "core step error", 1); break }
                }
            }
        }
    }
}

#[test]
#[ignore = "M40c diagnostic corpus; run explicitly in release"]
fn m40c_coverage() {
    let which = std::env::var("M40C_CORPUS").unwrap_or_else(|_| "AC".to_owned());
    println!("m40c_cov corpus: A = {} runs per variant (4 characters x seeds 0..{}), C = every Standard encounter x 4 characters x seeds 0..{} with Runic Dome, <= {} decisions; rollout every {ROLLOUT_EVERY}th root, {ROLLOUT_STEPS} steps", 4 * seeds_a() as usize, seeds_a(), seeds_c(), c_decisions());
    let mut tally = Tally::default();
    let start = std::time::Instant::now();
    if which.contains('A') { corpus_a(&mut tally); }
    if which.contains('C') { corpus_c(&mut tally); }
    for (k, v) in &tally.roots { println!("m40c_cov root | {k} | {v}"); }
    for (k, v) in &tally.enemies { println!("m40c_cov enemies | {k} | {v}"); }
    for (k, v) in &tally.branch { println!("m40c_cov branch | {k} | {v}"); }
    for side in ["no-dome", "dome"] {
        use sha2::Digest;
        let mut h = sha2::Sha256::new();
        let mut n = 0u64;
        for (s, d) in &tally.digests { if s == side { h.update(d); n += 1; } }
        println!("m40c_cov digests | {side} | {n} roots | sha256 {}", crate::combat_policy::hex(&h.finalize()[..]));
    }
    println!("m40c_cov seconds {}", start.elapsed().as_secs());
}
