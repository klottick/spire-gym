//! M40c (spire-61l stage 3, design-v4 §8): Runic Dome public roots on the Standard combat surface. T1 natural pairs,
//! T7 branch views, T8 determinism, branch re-roots and `step_many`, plus the ignored rule-2d truncation count.
use super::m34s_transplant::{standard_combat_session, standard_entry};
use super::standard_traversal::CHARACTERS;
use crate::*;
use serde_json::json;
use spire_core::content::Character;

const WIDE: SearchLimits = SearchLimits { max_live_branches: 64, max_branches: 1_000, max_steps_per_branch: 5_000, max_total_steps: 1_000_000 };
struct Rng(u64);
impl Rng { fn pick(&mut self, mask: &[i8]) -> usize { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; let legal = mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect::<Vec<_>>(); legal[(self.0 % legal.len() as u64) as usize] } }

/// A one-card Runic Dome entry (identical-copy labels never differ between seeds, so `same_world` is the full check).
fn dome_entry(character: Character, encounter: &str) -> serde_json::Value {
    let mut e = standard_entry("dome", character, 20, encounter, None);
    let defend = match character { Character::Ironclad => "defend", Character::Silent => "defend_g", Character::Defect => "defend_b", Character::Watcher => "defend_p" };
    e["deck"] = json!([{"card": defend, "upgrades": 0}]);
    e["relics"].as_array_mut().unwrap().push(json!("runic_dome"));
    e
}
fn end_turn_index(o: &RunObservation) -> i64 {
    let actions = o.group("actions").unwrap();
    let cols = *actions.shape.last().unwrap();
    let verb = |i: usize| usize::try_from(actions.values[i * cols]).ok().and_then(|v| crate::standard_vocab_manifest()["families"]["verb"].as_array()?.get(v)?.as_str().map(str::to_owned));
    o.action_mask.iter().enumerate().find(|(i, m)| **m == 1 && verb(*i).as_deref() == Some("end_turn")).map(|(i, _)| i as i64).unwrap()
}
fn rollout(search: &mut PublicSearch, b: BranchId, seed: u64) -> Vec<(Vec<i8>, i32, bool)> {
    let mut rng = Rng(seed | 1);
    let d = search.decision(b).unwrap();
    let (mut mask, mut token, mut out) = (d.observation.action_mask.clone(), d.token, vec![]);
    for _ in 0..200 {
        let r = search.step(b, token, RunActionIndex(rng.pick(&mask) as i64), BranchView::Mask).unwrap();
        out.push((r.action_mask.clone(), r.reward, r.terminated));
        if r.terminated || r.truncated { break }
        (mask, token) = (r.action_mask, r.token);
    }
    out
}

/// T1 (env): seeds with an identical public history (every observation and info over 0, 1 and 3 end turns) have equal
/// roots and equal canonical worlds, and identical branch rollouts for equal `(research_seed, k)`.
#[test]
fn dome_natural_pairs_have_equal_roots_and_branches() {
    let mut total = 0;
    for character in CHARACTERS {
        for encounter in ["jaw_worm", "chosen", "three_byrds"] {
            for depth in [0usize, 1, 3] {
                let entry = dome_entry(character, encounter);
                let mut groups: std::collections::BTreeMap<String, Vec<u64>> = Default::default();
                for seed in 0..200u64 {
                    let mut s = standard_combat_session(vec![entry.clone()]);
                    let mut b = s.reset(Some(seed), None).unwrap();
                    let mut h = format!("{:?}{}", b.observation, b.info);
                    let mut ok = true;
                    for _ in 0..depth { match s.step(b.token, RunActionIndex(end_turn_index(&b.observation))) { Ok(r) if !r.terminated => { b = r.boundary; h.push_str(&format!("{:?}{}", b.observation, b.info)); } _ => { ok = false; break } } }
                    if ok { groups.entry(h).or_default().push(seed); }
                }
                let at = |seed: u64| { let mut s = standard_combat_session(vec![entry.clone()]); let mut b = s.reset(Some(seed), None).unwrap();
                    for _ in 0..depth { b = s.step(b.token, RunActionIndex(end_turn_index(&b.observation))).unwrap().boundary; } s };
                for seeds in groups.values().filter(|g| g.len() > 1).take(6) {
                    let (s1, s2) = (at(seeds[0]), at(seeds[1]));
                    assert!(s1.run_for_test().unwrap() != s2.run_for_test().unwrap(), "the true worlds differ");
                    match (s1.public_root(), s2.public_root()) {
                        (Ok(r1), Ok(r2)) => {
                            assert_eq!(r1, r2, "{character:?} {encounter} depth {depth}: equal roots");
                            assert!(r1.same_world(&r2), "{character:?} {encounter} depth {depth}: equal canonical worlds");
                            for k in 0..2 {
                                let mut x = PublicSearch::new(r1.clone(), SearchConfig { research_seed: 41, limits: WIDE }).unwrap();
                                let mut y = PublicSearch::new(r2.clone(), SearchConfig { research_seed: 41, limits: WIDE }).unwrap();
                                let (a, b) = (x.branch(Some(k)).unwrap(), y.branch(Some(k)).unwrap());
                                assert!(x.decision(a).unwrap().observation == y.decision(b).unwrap().observation);
                                assert_eq!(rollout(&mut x, a, 7 + k), rollout(&mut y, b, 7 + k));
                            }
                            total += 1;
                        }
                        (Err(x), Err(y)) => assert_eq!(x, y),
                        (x, y) => panic!("{character:?} {encounter} depth {depth}: one refused: {:?} {:?}", x.err(), y.err()),
                    }
                }
            }
        }
    }
    assert!(total > 0, "no natural pair");
    eprintln!("m40c env natural pairs: {total}");
}

/// T7: a branch's view hides intents exactly like the live Runic Dome view (the root's first decision equals live).
#[test]
fn dome_branch_views_hide_intents_like_the_live_view() {
    for character in CHARACTERS {
        let mut s = standard_combat_session(vec![dome_entry(character, "jaw_worm")]);
        let live = s.reset(Some(3), None).unwrap();
        let root = s.public_root().unwrap();
        let mut search = PublicSearch::new(root, SearchConfig { research_seed: 5, limits: WIDE }).unwrap();
        for k in 0..4 {
            let b = search.branch(Some(k)).unwrap();
            let d = search.decision(b).unwrap();
            assert!(d.observation.group("enemies").unwrap().known == live.observation.group("enemies").unwrap().known, "{character:?} k={k}");
            assert!(d.observation.group("enemies").unwrap().values == live.observation.group("enemies").unwrap().values, "{character:?} k={k}: hidden intent columns equal the live view");
        }
    }
}

/// T8: equal `(root, seed, k)` give equal worlds; a branch's own root after steps is supported and equal for
/// public-equivalent branches; `step_many` equals serial `step`.
#[test]
fn dome_branches_are_deterministic_re_rootable_and_batch_equal() {
    let mut s = standard_combat_session(vec![dome_entry(Character::Ironclad, "chosen")]);
    s.reset(Some(9), None).unwrap();
    let root = s.public_root().unwrap();
    let mut x = PublicSearch::new(root.clone(), SearchConfig { research_seed: 77, limits: WIDE }).unwrap();
    let mut y = PublicSearch::new(root.clone(), SearchConfig { research_seed: 77, limits: WIDE }).unwrap();
    let (a, b) = (x.branch(Some(3)).unwrap(), y.branch(Some(3)).unwrap());
    assert_eq!(rollout(&mut x, a, 11), rollout(&mut y, b, 11));
    // Re-root after two end turns.
    let mut z = PublicSearch::new(root.clone(), SearchConfig { research_seed: 78, limits: WIDE }).unwrap();
    let c = z.branch(Some(0)).unwrap();
    for _ in 0..2 {
        let d = z.decision(c).unwrap();
        let r = z.step(c, d.token, RunActionIndex(end_turn_index(&d.observation)), BranchView::Full).unwrap();
        if r.terminated { break }
    }
    let reroot = z.public_root(c);
    assert!(reroot.is_ok(), "a Dome branch re-roots: {:?}", reroot.err());
    // step_many equals serial step over several branches.
    let mut serial = PublicSearch::new(root.clone(), SearchConfig { research_seed: 90, limits: WIDE }).unwrap();
    let mut batch = PublicSearch::new(root, SearchConfig { research_seed: 90, limits: WIDE }).unwrap();
    let ids = (0..4).map(|k| (serial.branch(Some(k)).unwrap(), batch.branch(Some(k)).unwrap())).collect::<Vec<_>>();
    for _ in 0..6 {
        let mut requests = vec![];
        let mut expected = vec![];
        for (sa, ba) in &ids {
            let (ds, db) = (serial.decision(*sa).unwrap(), batch.decision(*ba).unwrap());
            assert!(ds.observation == db.observation);
            let index = RunActionIndex(end_turn_index(&ds.observation));
            expected.push(serial.step(*sa, ds.token, index, BranchView::Mask).unwrap());
            requests.push(BranchStepRequest { branch: *ba, token: db.token, index });
        }
        let got = batch.step_many(&requests, BranchView::Mask).unwrap();
        for (e, g) in expected.iter().zip(got) {
            let BranchStepOutcome::Stepped(g) = g else { panic!("batched step faulted") };
            assert_eq!((&e.action_mask, e.reward, e.terminated), (&g.action_mask, g.reward, g.terminated));
        }
        if expected.iter().any(|e| e.terminated) { break }
    }
}

/// Diagnostic (not a gate): rule-2d truncation marks on corpus C (every Standard encounter, four characters, Runic Dome,
/// random legal play), by species. `cargo test --release -p spire-env m40c_truncation_count -- --ignored --nocapture`.
#[test]
#[ignore = "M40c diagnostic; run explicitly in release"]
fn m40c_truncation_count() {
    use spire_core::combat::enemies::A0_ENCOUNTER_SETUPS;
    let encounters = A0_ENCOUNTER_SETUPS.iter().filter(|(e, _, _)| spire_core::run::standard_combat_encounter(*e).is_some()).map(|(e, act, _)| (e.as_str(), *act)).collect::<Vec<_>>();
    let seeds = std::env::var("M40C_SEEDS_C").ok().and_then(|v| v.parse().ok()).unwrap_or(16u64);
    let mut by_act: std::collections::BTreeMap<String, u64> = Default::default();
    for character in CHARACTERS {
        for (encounter, act) in &encounters {
            let mut entry = standard_entry("e", character, 20, encounter, None);
            entry["relics"].as_array_mut().unwrap().push(json!("runic_dome"));
            let mut session = standard_combat_session(vec![entry]);
            for seed in 0..seeds {
                session.reset(Some(seed), None).unwrap();
                let mut run = session.run_for_test().unwrap().clone();
                let mut rng = Rng(seed * 977 + character as u64 + 1);
                let before = spire_core::combat::belief::m40c_truncation_counts();
                for _ in 0..300 {
                    if run.combat_scope_terminal().is_some() { break }
                    let d = run.decision().unwrap();
                    let mask = vec![1i8; d.candidates.len()];
                    let a = d.candidates[rng.pick(&mask)].action.clone();
                    if run.step(&a).is_err() { break }
                }
                let after = spire_core::combat::belief::m40c_truncation_counts();
                for (species, n) in after { let added = n - before.get(species).copied().unwrap_or(0); if added > 0 { *by_act.entry(format!("act{act} {species}")).or_default() += added; } }
            }
        }
    }
    for (k, v) in &by_act { println!("m40c_trunc | {k} | {v}"); }
    println!("m40c_trunc total {}", by_act.values().sum::<u64>());
}
