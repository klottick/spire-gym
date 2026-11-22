//! M40a (spire-61l stage 1) coverage diagnostic: how often `PublicRoot` exists at a combat decision, by refusal class,
//! and how often a fixed playout rule meets an in-branch refusal. Diagnostic only: no thresholds, nothing asserted about
//! the counts. Ignored release test:
//! `cargo test --release -p spire-env m40a_coverage -- --ignored --nocapture`.
//! Uses public items only, so the same file runs on the base revision (before) and the branch (after).
//!
//! Corpus A (natural runs): supplied Standard Runs at A20, 900 HP and all keys, no wound hook, four characters, seeds
//! `0..SEEDS_A`, two variants (A1 seeded boss-relic choice, A2 prefers Runic Dome); seeded policy takes card rewards, boss
//! relics, random map routes. Corpus B (targeted decks): `standard-combat-v1` at A20, every elite and boss encounter the
//! Standard catalog admits, four characters, a fixed deck addition per character, seeds `0..SEEDS_B`, seeded uniform
//! random legal play for up to `B_DECISIONS` decisions.
use super::m34s_transplant::{standard_combat_session, standard_entry};
use super::standard_traversal::{context, current_node_is_boss, field, legal_indices, short_creation, simple_action, CHARACTERS};
use crate::*;
use spire_core::{combat::{enemies::{EncounterKind, A0_ENCOUNTER_SETUPS}, rng::Partition, CombatAction}, content::Character, run::{diagnostics::standard_supplied_run_with, standard_combat_encounter, GameAction, Run, RunRng}};
use std::collections::BTreeMap;

fn env_or(name: &str, default: u64) -> u64 { std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default) }
/// Seeds per character for corpus A and per (character, encounter) for corpus B; `M40A_SEEDS_A` / `M40A_SEEDS_B` override.
fn seeds_a() -> u8 { env_or("M40A_SEEDS_A", 24) as u8 }
fn seeds_b() -> u64 { env_or("M40A_SEEDS_B", 32) }
const B_DECISIONS: usize = 400;
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

#[derive(Default)]
struct Tally {
    roots: BTreeMap<String, u64>,
    branch: BTreeMap<String, u64>,
}
impl Tally {
    fn root(&mut self, cell: &str, outcome: &Result<PublicRoot, RunSessionError>) {
        let class = match outcome { Ok(_) => "supported".to_owned(), Err(RunSessionError::UnsupportedBoundary(reason)) => format!("refused: {reason}"), Err(e) => format!("other: {e:?}") };
        *self.roots.entry(format!("{cell} | {class}")).or_default() += 1;
    }
    fn count(&mut self, cell: &str, what: &str, n: u64) { *self.branch.entry(format!("{cell} | {what}")).or_default() += n; }
}

/// A seeded uniform-random-legal rollout of one branch of `root`: steps, hidden-order refusals, applied identical-copy picks.
fn rollout(tally: &mut Tally, cell: &str, root: PublicRoot, seed: u64) {
    let mut search = PublicSearch::new(root, SearchConfig { research_seed: seed, limits: LIMITS }).unwrap();
    let Ok(b) = search.branch(Some(0)) else { return };
    let mut rng = Rng::new(seed ^ 0xB5AD_4ECE_DA1C_E2A9);
    let d = search.decision(b).unwrap();
    let (mut mask, mut token) = (d.observation.action_mask.clone(), d.token);
    for _ in 0..ROLLOUT_STEPS {
        let legal = mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect::<Vec<_>>();
        if legal.is_empty() { break }
        let first = rng.below(legal.len());
        let mut taken = None;
        for k in 0..legal.len() {
            let index = legal[(first + k) % legal.len()];
            let hidden = {
                let run = search.branch_run_for_test(b);
                let action = run.decision().ok().and_then(|d| d.candidates.get(index).map(|c| c.action.clone()));
                action.is_some_and(|a| run.pick_depends_on_hidden_order(&a))
            };
            tally.count(cell, "branch steps attempted", 1);
            match search.step(b, token, RunActionIndex(index as i64), BranchView::Mask) {
                Ok(r) => { if hidden { tally.count(cell, "identical-copy picks applied", 1) } taken = Some(r); break }
                Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK)) => tally.count(cell, "branch refusals: hidden-order pick", 1),
                Err(e) => { tally.count(cell, &format!("branch stop: {e:?}"), 1); return }
            }
        }
        let Some(r) = taken else { tally.count(cell, "branch stop: every legal action refused", 1); return };
        if r.terminated || r.truncated { tally.count(cell, "branch playouts ended", 1); return }
        (mask, token) = (r.action_mask, r.token);
    }
}

/// A supplied Standard session (all keys, `hp()` HP, no wound hook; the traversal helper without its wounding).
fn supplied_session(character: Character, seed: u8) -> RunSession {
    let creation = short_creation();
    let run = standard_supplied_run_with(character, 20, creation.profile(), hp() as u32, true, RunRng::new([seed; 32], Partition::Test, 0)).unwrap();
    let mut config = StandardRunConfig::new(character, 20, Partition::Test);
    config.creation = creation;
    RunSession::from_standard_supplied(config, run, u64::from(seed)).unwrap()
}
fn hp() -> u64 { env_or("M40A_HP", 900) }
fn refill() -> bool { env_or("M40A_REFILL", 1) != 0 }
fn character_name(c: Character) -> &'static str { c.as_str() }

// ---------------------------------------------------------------- corpus A
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
            // The offered relic is the action source row's `offers` row (column 6 holds the relic id).
            let relic = |i: usize| usize::try_from(field(obs, "actions", i, 2)).map(|row| vocab("relic", field(obs, "offers", row, 6))).unwrap_or_default();
            if prefer_dome { if let Some(i) = legal.iter().copied().find(|i| relic(*i) == "runic_dome") { return i } }
            let picks = legal.iter().copied().filter(|i| !choice(*i).contains("skip")).collect::<Vec<_>>();
            if picks.is_empty() { legal[0] } else { picks[rng.below(picks.len())] }
        }
        "combat" => {
            // Mostly the repo's simple combat policy; sometimes any legal action when a free choice is open (not in a selection).
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
                let (mut steps, mut combat_decisions, mut max_act) = (0usize, 0usize, 1i64);
                while boundary.terminal.is_none() && steps < A_STEP_LIMIT {
                    let (act, _, room) = context(&boundary.observation);
                    max_act = max_act.max(act);
                    if room == "map" && refill() {
                        // Stated supplied condition (not a wound hook): HP is refilled at each map decision, so a run keeps playing full-length fights through all three acts.
                        boundary = session.apply_supplied(|run| { spire_core::run::diagnostics::supply_hp_and_keys(run, hp() as u32, true); Ok(()) }).unwrap();
                    }
                    if room == "combat" {
                        let cell = format!("A {variant} {} act{act} {}", character_name(character), kind_of(&boundary.observation));
                        let outcome = session.public_root();
                        tally.root(&cell, &outcome);
                        combat_decisions += 1;
                        if let Ok(root) = outcome { if combat_decisions % ROLLOUT_EVERY == 1 { rollout(tally, &cell, root, seed as u64 * 7919 + combat_decisions as u64); } }
                    }
                    let index = policy_a(&boundary.observation, &mut rng, variant == "A2-dome");
                    boundary = session.step(boundary.token, RunActionIndex(index as i64)).unwrap().boundary;
                    steps += 1;
                }
                let end = match boundary.terminal { Some(t) => format!("{t:?}"), None => "step limit".to_owned() };
                *tally.branch.entry(format!("A {variant} {} run ends: {end} (max act {max_act})", character_name(character))).or_default() += 1;
            }
        }
    }
}

// ---------------------------------------------------------------- corpus B
fn additions(character: Character) -> (&'static [&'static str], &'static [&'static str], Option<&'static str>) {
    match character {
        Character::Ironclad => (&["double_tap", "headbutt", "havoc", "wild_strike", "anger", "secret_weapon", "headbutt", "double_tap"], &["necronomicon"], Some("duplication_potion")),
        Character::Silent => (&["dagger_throw", "burst", "survivor", "nightmare", "secret_technique", "acrobatics", "dagger_throw"], &["necronomicon"], Some("duplication_potion")),
        Character::Defect => (&["echo_form", "seek", "overclock", "hologram", "all_for_one", "seek", "scrape"], &["necronomicon"], Some("duplication_potion")),
        Character::Watcher => (&["just_lucky", "cut_through_fate", "omniscience", "evaluate", "study", "third_eye", "reach_heaven", "just_lucky"], &["melange"], Some("duplication_potion")),
    }
}
fn corpus_b(tally: &mut Tally) {
    let encounters = A0_ENCOUNTER_SETUPS.iter().filter(|(e, _, kind)| *kind != EncounterKind::Monster && standard_combat_encounter(*e).is_some()).map(|(e, _, kind)| (e.as_str(), *kind)).collect::<Vec<_>>();
    let caps = RunCapacities::standard_default();
    for character in CHARACTERS {
        let (cards, relics, potion) = additions(character);
        for (encounter, kind) in &encounters {
            let mut entry = standard_entry("e", character, 20, encounter, None);
            for card in cards { entry["deck"].as_array_mut().unwrap().push(serde_json::json!({"card": card, "upgrades": 0})); }
            for relic in relics { entry["relics"].as_array_mut().unwrap().push(serde_json::json!(relic)); }
            if let Some(p) = potion { entry["potions"][0] = serde_json::json!(p); }
            let mut session = standard_combat_session(vec![entry]);
            for seed in 0..seeds_b() {
                session.reset(Some(seed), None).unwrap();
                let mut run: Run = session.run_for_test().unwrap().clone();
                let mut rng = Rng::new(seed * 977 + character as u64);
                let cell = format!("B {} {}", character_name(character), if *kind == EncounterKind::Boss { "boss" } else { "elite" });
                for decision in 0..B_DECISIONS {
                    if run.combat_scope_terminal().is_some() { break }
                    let outcome = PublicRoot::of(&run, &caps);
                    tally.root(&cell, &outcome);
                    if let Ok(root) = outcome { if decision % ROLLOUT_EVERY == 0 { rollout(tally, &cell, root, seed * 31 + decision as u64); } }
                    let d = run.decision().unwrap();
                    let action = d.candidates[rng.below(d.candidates.len())].action.clone();
                    if run.step(&action).is_err() { *tally.branch.entry(format!("{cell} | core step error")).or_default() += 1; break }
                }
            }
        }
    }
    let _ = CombatAction::EndTurn;
    let _ = GameAction::Combat(CombatAction::EndTurn);
}

#[test]
#[ignore = "M40a diagnostic corpus; run explicitly in release"]
fn m40a_coverage() {
    let which = std::env::var("M40A_CORPUS").unwrap_or_else(|_| "AB".to_owned());
    println!("m40a_cov corpus: A = {} runs per variant (4 characters x seeds 0..{}), B = elite and boss encounters x 4 characters x seeds 0..{}, <= {B_DECISIONS} decisions; rollout every {ROLLOUT_EVERY}th root, {ROLLOUT_STEPS} steps", 4 * seeds_a() as usize, seeds_a(), seeds_b());
    let mut tally = Tally::default();
    let start = std::time::Instant::now();
    if which.contains('A') { corpus_a(&mut tally); }
    if which.contains('B') { corpus_b(&mut tally); }
    for (k, v) in &tally.roots { println!("m40a_cov root | {k} | {v}"); }
    for (k, v) in &tally.branch { println!("m40a_cov branch | {k} | {v}"); }
    println!("m40a_cov seconds {}", start.elapsed().as_secs());
}
