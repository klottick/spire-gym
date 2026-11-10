//! Small public-search lookahead demonstration. Rewards are estimates from random legal rollouts,
//! not a claim about policy strength.
use serde_json::json;
use spire_core::{combat::rng::Partition, content::{Catalog, Character}};
use spire_env::*;
use std::{env, time::Instant};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
fn starter_relic(c: Character) -> &'static str { match c { Character::Ironclad => "burning_blood", Character::Silent => "ring_of_the_snake", Character::Defect => "cracked_core", Character::Watcher => "pure_water" } }
fn legal(mask: &[i8]) -> Vec<usize> { mask.iter().enumerate().filter_map(|(i, m)| (*m == 1).then_some(i)).collect() }
fn rng(x: &mut u64) -> u64 { *x ^= *x << 13; *x ^= *x >> 7; *x ^= *x << 17; *x }
fn pick(mask: &[i8], seed: &mut u64) -> usize { let legal = legal(mask); legal[(rng(seed) as usize) % legal.len()] }
fn report(label: &str, root: PublicRoot, seed: u64, samples: usize) -> Result<(), String> {
    let first = Instant::now();
    let mut search = PublicSearch::new(root, SearchConfig { research_seed: seed, limits: SearchLimits { max_live_branches: 128,
        max_branches: 100_000, max_steps_per_branch: 5_000, max_total_steps: 1_000_000 } }).map_err(|e| format!("search: {e:?}"))?;
    let boundary = search.branch(Some(0)).and_then(|b| search.decision(b)).map_err(|e| format!("root branch: {e:?}"))?;
    let actions = legal(&boundary.observation.action_mask);
    println!("{label}: {} legal root actions", actions.len());
    for action in actions {
        let mut rewards = Vec::new();
        let mut wins = 0usize; let mut deaths = 0usize; let mut escapes = 0usize; let mut truncated = 0usize;
        for sample in 0..samples {
            let branch = search.branch(Some(sample as u64 + 1)).map_err(|e| format!("branch: {e:?}"))?;
            let d = search.decision(branch).map_err(|e| format!("decision: {e:?}"))?;
            let mut result = search.step(branch, d.token, RunActionIndex(action as i64), BranchView::Mask).map_err(|e| format!("step: {e:?}"))?;
            let mut local_seed = seed ^ (action as u64).wrapping_mul(0x9e3779b97f4a7c15) ^ sample as u64;
            while !result.terminated && !result.truncated {
                let next_action = pick(&result.action_mask, &mut local_seed);
                result = search.step(branch, result.token, RunActionIndex(next_action as i64), BranchView::Mask).map_err(|e| format!("rollout: {e:?}"))?;
            }
            rewards.push(result.reward);
            match result.info["terminal_reason"].as_str() { Some("victory") => wins += 1, Some("death") => deaths += 1, Some("escaped") => escapes += 1, _ if result.truncated => truncated += 1, _ => {} }
            search.close(branch).map_err(|e| format!("close: {e:?}"))?;
        }
        let mean = rewards.iter().sum::<i32>() as f64 / rewards.len().max(1) as f64;
        println!("  action={action} mean_reward={mean:.3} terminals={{victory:{wins},death:{deaths},escaped:{escapes},truncated:{truncated}}}");
    }
    println!("  counters={} elapsed_ms={}", search.counters(), first.elapsed().as_millis());
    Ok(())
}
fn main() { if let Err(error) = run() { eprintln!("public_lookahead: {error}"); std::process::exit(1); } }
fn run() -> Result<(), String> {
    let samples = env::args().nth(1).map(|s| s.parse::<usize>().map_err(|e| e.to_string())).transpose()?.unwrap_or(4).clamp(1, 64);
    // Move through a fresh Standard run with the first published legal action until its first combat.
    let character = Character::Ironclad;
    let mut run = RunSession::new_standard(StandardRunConfig::new(character, 0, Partition::Test)).map_err(|e| format!("session: {e:?}"))?;
    let mut d = run.reset(Some(1234)).map_err(|e| format!("reset: {e:?}"))?;
    let root = loop {
        if let Ok(root) = run.public_root() { break root }
        let index = legal(&d.observation.action_mask).first().copied().ok_or_else(|| "no legal action".to_owned())?;
        d = run.step(d.token, RunActionIndex(index as i64)).map_err(|e| format!("Run step: {e:?}"))?.boundary;
    };
    report("Standard run", root, 17, samples)?;

    let encounter = "jaw_worm";
    let entries = CHARACTERS[0].as_str();
    let deck = Catalog::standard().starter_deck(character).unwrap().iter().map(|c| json!({"card":c.as_str(),"upgrades":0})).collect::<Vec<_>>();
    let catalog = json!({"catalog_schema_id":STANDARD_COMBAT_CATALOG_SCHEMA_ID,"profile_id":STANDARD_COMBAT_PROFILE_ID,
        "entries":[{"id":"start","character":entries,"ascension":0,"encounter":encounter,"hp":70,"max_hp":80,"gold":0,"deck":deck,
            "relics":[starter_relic(character)],"potions":[null,null,null]}]});
    let bytes = serde_json::to_vec(&catalog).map_err(|e| e.to_string())?;
    let cat = CombatTrainingCatalog::standard_from_json(&bytes).map_err(|e| format!("catalog: {e:?}"))?;
    let mut standalone = CombatTrainingSession::new_standard(cat, Partition::Test).map_err(|e| format!("session: {e:?}"))?;
    standalone.reset(Some(4321), Some("start")).map_err(|e| format!("reset: {e:?}"))?;
    let root = standalone.public_root().map_err(|e| format!("public root: {e:?}"))?;
    report("standard-combat-v1", root, 23, samples)?;
    Ok(())
}
