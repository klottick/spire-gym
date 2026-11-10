//! M34s-b b5: measured search costs (design §4.11); ignored release tests (assistant b56 fills this file).
use super::m34s_transplant::{standard_combat_session, standard_entry};
use super::standard_traversal::{context, current_node_is_boss, drive, short_creation, simple_action, standard_session, CHARACTERS};
use crate::*;
use spire_core::{combat::rng::Partition, content::Character, run::{CombatTrainingEntry, CreationProfile, GameAction, RoomScenario, Run, RunConfig, RunRng, TRAINING_CARDS, TRAINING_ENCOUNTERS_V2}};
use std::{mem::size_of, time::Instant};

const LIMITS: SearchLimits = SearchLimits { max_live_branches: 8, max_branches: 1_000, max_steps_per_branch: 5_000, max_total_steps: 1_000_000 };
fn median(v: &mut [u128]) -> u128 { v.sort_unstable(); v[v.len()/2] }
/// Debug-length proxy of a Run plus its draw knowledge's heap (the combat Debug omits the knowledge; final review I2).
fn debug_proxy(run: &Run) -> usize { format!("{run:?}").len() + run.public_knowledge_heap_bytes().unwrap_or(0) }
fn run_boss(character: Character, ascension: u8, seed: u8) -> (RunSession, PublicRoot, usize) {
    let mut last_reason = String::new();
    for offset in 0..32u8 {
        let (mut session, _) = standard_session(character, ascension, seed.wrapping_add(offset), short_creation());
        if let Err(error) = drive(&mut session, 12_000, |o| { let (act, _, room) = context(o); act == 3 && room == "combat" && current_node_is_boss(o) }, simple_action) {
            last_reason = format!("route: {error:?}"); continue;
        }
        match session.public_root() {
            Ok(root) => { let bytes = debug_proxy(session.run_for_test().unwrap()); return (session, root, bytes); }
            Err(error) => last_reason = format!("root: {error:?}"),
        }
    }
    panic!("no supported first Act 3 boss root for {character:?} A{ascension} after 32 supplied routes: {last_reason}")
}
fn measure_root(label: &str, root: PublicRoot, root_proxy: usize, mut public_root_times: Vec<u128>) {
    let (mut roots, mut branches, mut full, mut mask, mut forks) = (vec![], vec![], vec![], vec![], vec![]);
    let mut max_branch_proxy = 0usize;
    for i in 0..9 {
        let start = Instant::now(); let mut search = PublicSearch::new(root.clone(), SearchConfig { research_seed: i as u64 + 7, limits: LIMITS }).unwrap();
        roots.push(start.elapsed().as_nanos());
        let start = Instant::now(); let b = search.branch(Some(i as u64)).unwrap(); branches.push(start.elapsed().as_nanos());
        let branch_proxy = size_of::<Run>() + debug_proxy(search.branch_run_for_test(b));
        max_branch_proxy = max_branch_proxy.max(branch_proxy);
        let d = search.decision(b).unwrap();
        let action = d.observation.action_mask.iter().position(|m| *m == 1).unwrap();
        let start = Instant::now(); let _ = search.step(b, d.token, RunActionIndex(action as i64), BranchView::Full).unwrap(); full.push(start.elapsed().as_nanos());
        let b2 = search.branch(Some(100 + i as u64)).unwrap(); let d2 = search.decision(b2).unwrap(); let a2 = d2.observation.action_mask.iter().position(|m| *m == 1).unwrap();
        let start = Instant::now(); let _ = search.step(b2, d2.token, RunActionIndex(a2 as i64), BranchView::Mask).unwrap(); mask.push(start.elapsed().as_nanos());
        let b3 = search.branch(Some(200 + i as u64)).unwrap();
        let start = Instant::now(); let child = search.fork(b3).unwrap(); forks.push(start.elapsed().as_nanos());
        search.close(child).unwrap();
        if i == 0 {
            println!("search_measure {label}: retained_proxy root={} branch={} bytes (Run inline size + sanitized Run Debug string length + draw-knowledge heap; diagnostic proxy, not allocator measurement)", size_of::<Run>() + root_proxy, branch_proxy);
            // Final review v2 I2: allocator accounting (this test binary's counting allocator). The root's heap is its Run's;
            // a second fresh branch's is its box (inline Run), its Run's heap, its legal-action table and its map slot.
            let (_, root_heap) = super::standard_exact_cost::retained_bytes(|| root.clone());
            let (_, branch_heap) = super::standard_exact_cost::retained_bytes(|| search.branch(Some(300)).unwrap());
            let knowledge = search.branch_run_for_test(b).public_knowledge_heap_bytes().unwrap_or(0);
            println!("search_measure {label}: allocator root_heap={root_heap} fresh_branch_heap={branch_heap} knowledge_heap={knowledge} branch_record={} run_estimate={BRANCH_RUN_BYTES_ESTIMATE}",
                size_of::<Run>());
        }
    }
    println!("search_measure {label}: medians_ns public_root={} search_new={} branch={} step_full={} step_mask={} fork={} root_inline={} branch_inline={} max_branch_retained_proxy={}",
        median(&mut public_root_times), median(&mut roots), median(&mut branches), median(&mut full), median(&mut mask), median(&mut forks), size_of::<PublicRoot>(), size_of::<Run>(),
        max_branch_proxy.max(size_of::<Run>()));
}
fn root_times(mut f: impl FnMut() -> PublicRoot) -> Vec<u128> {
    (0..9).map(|_| { let start = Instant::now(); let _ = f(); start.elapsed().as_nanos() }).collect()
}

#[test]
#[ignore = "measurement; run with --release --ignored --nocapture"]
fn m34s_search_measure() {
    for (ci, character) in CHARACTERS.into_iter().enumerate() { for ascension in [0, 20] {
        let (session, root, bytes) = run_boss(character, ascension, 0xB0 + ci as u8 + if ascension == 20 { 8 } else { 0 });
        let times = root_times(|| session.public_root().expect("public root repeat"));
        measure_root(&format!("Run {character:?} A{ascension} first Act 3 boss"), root, bytes, times);
        drop(session);
    }}
    for character in CHARACTERS { for ascension in [0, 20] {
        let session = standard_combat_session(vec![standard_entry("measure", character, ascension, "jaw_worm", None)]);
        let mut session = session; session.reset(Some(0x51), Some("measure")).unwrap();
        let bytes = debug_proxy(session.run_for_test().unwrap());
        let times = root_times(|| session.public_root().expect("public root repeat"));
        measure_root(&format!("standalone {character:?} A{ascension}"), session.public_root().unwrap(), bytes, times);
    }}
}

/// Same step driver is copied to the archived base build for a direct per-step comparison.
fn maintenance_measure() {
    fn drive_steps(run: &mut Run, cap: usize) -> (usize, u128) {
        let (mut steps, mut ns) = (0usize, 0u128);
        while steps < cap && run.decision().is_ok_and(|d| d.terminal.is_none() && !d.candidates.is_empty()) {
            let d = run.decision().unwrap();
            let action = d.candidates.iter().find(|c| matches!(c.action, GameAction::Combat(spire_core::combat::CombatAction::PlayCard { .. })))
                .unwrap_or(&d.candidates[0]).action.clone();
            let start = Instant::now(); if run.step(&action).is_err() { break } ns += start.elapsed().as_nanos(); steps += 1;
        }
        (steps, ns)
    }
    let mut gym = (0usize, 0u128);
    for (index, (encounter, _, _)) in TRAINING_ENCOUNTERS_V2.iter().enumerate().take(24) {
        let deck = TRAINING_CARDS.iter().cycle().skip(index).take(14).map(|c| (*c, 0)).collect();
        let entry = CombatTrainingEntry { encounter: *encounter, hp: 80, max_hp: 80, deck, relics: vec![], potions: vec![None; 3], neows_lament_remaining: None };
        let mut run = Run::prepare_combat_training_v2_with_gold(entry, 0, RunRng::new([index as u8; 32], Partition::Test, 0)).unwrap();
        if run.enter_training_combat(*encounter).is_ok() { let r = drive_steps(&mut run, 2_000); gym.0 += r.0; gym.1 += r.1; }
    }
    println!("search_measure_step Gym66 v2 24 encounters: steps={} mean_us={:.3}", gym.0, gym.1 as f64 / gym.0.max(1) as f64 / 1_000.0);
    for seed in [1u8, 2] {
        let mut run = Run::new_room_scenario(RunConfig { character: Character::Ironclad, ascension: 0 }, CreationProfile { neow_eligible: false, ..CreationProfile::experienced() },
            RunRng::new([seed; 32], Partition::Test, 0), RoomScenario::M10IroncladHeart).unwrap();
        let (steps, ns) = drive_steps(&mut run, 3_000);
        println!("search_measure_step M10 Ironclad Run seed {seed}: steps={steps} mean_us={:.3}", ns as f64 / steps.max(1) as f64 / 1_000.0);
    }
}

#[test]
#[ignore = "measurement; run with --release --ignored --nocapture"]
fn m34s_search_nonstandard_step_cost() { maintenance_measure(); }
