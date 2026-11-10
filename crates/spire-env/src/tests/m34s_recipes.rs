//! M34s-c c3: `reachable_recipe` starts (design §4.8; plan c3): a Standard record prefix validated once at load, its public
//! root cached, and each reset a world re-sampled from that root (never the record's hidden state).
use crate::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{combat::rng::Partition, content::Character, run::diagnostics};

/// A Standard Ironclad A0 record (seed `seed`, first legal action) that reaches its first combat decision and takes `extra`
/// more combat actions; returns the record bytes, its prefix length and the live session at that decision.
fn record_into_combat(seed: u64, extra: usize) -> (Vec<u8>, usize, RunSession) {
    let mut config = StandardRunConfig::new(Character::Ironclad, 0, Partition::Test);
    config.record_replay = true;
    let mut s = RunSession::new_standard(config).unwrap();
    let mut b = s.reset(Some(seed)).unwrap();
    let (mut steps, mut in_combat) = (0, 0);
    loop {
        if s.public_root().is_ok() { if in_combat == extra { break } in_combat += 1; }
        assert!(steps < 400, "no combat decision");
        let i = b.observation.action_mask.iter().position(|&v| v != 0).unwrap();
        b = s.step(b.token, RunActionIndex(i as i64)).unwrap().boundary;
        steps += 1;
    }
    (s.record().unwrap().unwrap().to_json().unwrap(), steps, s)
}
fn sha(bytes: &[u8]) -> String { Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect() }
fn recipe_catalog(entries: Vec<Value>) -> Vec<u8> {
    serde_json::to_vec(&json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": entries})).unwrap()
}
fn recipe(id: &str, record: &[u8], steps: usize) -> Value { json!({"id": id, "recipe": {"record_sha256": sha(record), "steps": steps}}) }
fn load(entries: Vec<Value>, records: &[Vec<u8>]) -> Result<CombatTrainingCatalog, RunSessionError> {
    CombatTrainingCatalog::standard_from_json_with_records(&recipe_catalog(entries), records)
}

/// c3: the record and its prefix are validated at load: bad bytes, a missing or unmatched record and the size caps are
/// `Input`; a pre-change record or a diverging prefix is `IncompatibleReplay`; a prefix outside a combat decision is
/// `UnsupportedBoundary`.
#[test]
fn recipe_validates_its_record_prefix_at_load() {
    let (record, steps, _) = record_into_combat(20261110, 0);
    assert!(load(vec![recipe("r", &record, steps)], std::slice::from_ref(&record)).is_ok());
    let input = |r: Result<CombatTrainingCatalog, RunSessionError>| matches!(r, Err(RunSessionError::Input(_)));
    assert!(input(load(vec![recipe("r", &record, steps)], &[])), "record not supplied");
    let bad = b"not json".to_vec();
    assert!(input(load(vec![recipe("r", &bad, steps)], std::slice::from_ref(&bad))), "bad record bytes");
    let mut other = record.clone();
    other.push(b' ');
    assert!(input(load(vec![json!({"id": "r", "recipe": {"record_sha256": sha(&record), "steps": steps}})], &[other])), "SHA mismatch");
    let records_error = |r: Result<CombatTrainingCatalog, RunSessionError>| matches!(r, Err(RunSessionError::Input(m)) if m.starts_with("records:"));
    assert!(records_error(load(vec![recipe("r", &record, steps)], &[record.clone(), vec![b' '; STANDARD_RECORD_MAX_BYTES + 1]])), "a record over 8 MiB");
    assert!(records_error(load(vec![recipe("r", &record, steps)], &[vec![record.clone()], vec![vec![b' '; STANDARD_RECORD_MAX_BYTES]; 8]].concat())), "records over 64 MiB in total");
    assert!(input(load(vec![recipe("r", &record, steps + 10_000)], std::slice::from_ref(&record))), "prefix past the record");
    assert!(input(load(vec![json!({"id": "r", "recipe": {"record_sha256": sha(&record), "steps": steps}, "hp": 3})], std::slice::from_ref(&record))), "unknown recipe field");
    // Pre-M34s record (old rules id and fingerprint): typed IncompatibleReplay, never a silent mismatch (design §4.12).
    let old = include_bytes!("fixtures/pre_m34s_standard_record_v1.json").to_vec();
    assert!(matches!(load(vec![recipe("r", &old, 3)], std::slice::from_ref(&old)), Err(RunSessionError::IncompatibleReplay(_))));
    // A diverging prefix (a recorded reward the replay does not reproduce).
    let mut v: Value = serde_json::from_slice(&record).unwrap();
    v["steps"][0]["reward"] = json!(5);
    let diverged = serde_json::to_vec(&v).unwrap();
    assert!(matches!(load(vec![recipe("r", &diverged, steps)], std::slice::from_ref(&diverged)), Err(RunSessionError::IncompatibleReplay(_))));
    // Not at a combat decision (the opening Neow decision).
    assert!(matches!(load(vec![recipe("r", &record, 0)], std::slice::from_ref(&record)), Err(RunSessionError::UnsupportedBoundary(_))));
}

/// c3: resets share the recipe's public root and re-sample its hidden state: different seeds give the same first view and
/// different hidden draw orders; the same (entry, seed) gives an identical start.
#[test]
fn recipe_resets_share_the_public_root_and_resample_hidden_state() {
    let (record, steps, _) = record_into_combat(20261111, 1);
    let mut s = CombatTrainingSession::new_standard(load(vec![recipe("r", &record, steps)], &[record]).unwrap(), Partition::Test).unwrap();
    let first = s.reset(Some(1), None).unwrap();
    let draws = |s: &mut CombatTrainingSession| diagnostics::supplied_combat(s.run_mut_for_test().unwrap()).unwrap().diagnostic_draw_classes();
    let order = draws(&mut s);
    let mut differs = false;
    for seed in 2..20 {
        let b = s.reset(Some(seed), None).unwrap();
        assert_eq!(b.observation, first.observation, "the first view is the root's");
        differs |= draws(&mut s) != order;
    }
    assert!(differs, "hidden draw orders are re-sampled per seed");
    let again = s.reset(Some(1), None).unwrap();
    assert_eq!((again.observation, draws(&mut s)), (first.observation.clone(), order), "same seed, same world");
    // Implicit resets follow Gym66 seed semantics (ordinal + 1), and stay inside the root's support.
    let next = s.reset(None, None).unwrap();
    assert_eq!((next.observation, next.info["episode_ordinal"].clone()), (first.observation, json!(1)));
}

/// c3: the start is labelled reachable in the descriptor and the info.
#[test]
fn recipe_start_is_labelled_reachable() {
    let (record, steps, _) = record_into_combat(20261112, 0);
    let cat = load(vec![recipe("r", &record, steps)], &[record]).unwrap();
    assert_eq!(cat.descriptor()["entries"][0]["start_kind"], "reachable_recipe");
    let mut s = CombatTrainingSession::new_standard(cat, Partition::Test).unwrap();
    assert_eq!(s.reset(Some(3), None).unwrap().info["start_kind"], "reachable_recipe");
}

/// c3 (privacy): resets never follow the record's hidden future: over many resets the sampled top card is not concentrated
/// on the recorded combat's true next card (it matches only at that class's share of the unknown pile).
#[test]
fn recipe_does_not_follow_the_record_hidden_future() {
    let (record, steps, mut live) = record_into_combat(20261113, 0);
    let truth = diagnostics::supplied_combat(live.run_mut_for_test().unwrap()).unwrap().diagnostic_draw_classes();
    let true_top = truth.first().cloned().flatten().expect("a non-empty draw pile");
    let share = truth.iter().filter(|c| c.as_ref() == Some(&true_top)).count() as f64 / truth.len() as f64;
    let mut s = CombatTrainingSession::new_standard(load(vec![recipe("r", &record, steps)], &[record]).unwrap(), Partition::Test).unwrap();
    let n = 300;
    let mut hits = 0;
    for seed in 0..n {
        s.reset(Some(seed), None).unwrap();
        let top = diagnostics::supplied_combat(s.run_mut_for_test().unwrap()).unwrap().diagnostic_draw_classes().first().cloned().flatten();
        hits += usize::from(top == Some(true_top));
    }
    let rate = hits as f64 / n as f64;
    assert!(share < 1.0, "control: the true next card is not forced by public knowledge");
    assert!((rate - share).abs() < 0.12, "sampled top rate {rate} vs its public share {share}: no concentration on the true next card");
}

/// c3: the prefix is replayed once, at load; resets only sample.
#[test]
fn recipe_prefix_cost_is_paid_once() {
    let (record, steps, _) = record_into_combat(20261114, 0);
    crate::standard_combat::RECIPE_LOADS.with(|n| n.set(0));
    let cat = load(vec![recipe("r", &record, steps), recipe("r2", &record, steps)], &[record]).unwrap();
    assert_eq!(crate::standard_combat::RECIPE_LOADS.with(|n| n.get()), 2, "one load per recipe entry");
    let mut s = CombatTrainingSession::new_standard(cat, Partition::Test).unwrap();
    for seed in 0..20 { s.reset(Some(seed), Some("r")).unwrap(); }
    assert_eq!(crate::standard_combat::RECIPE_LOADS.with(|n| n.get()), 2, "resets never replay the prefix");
}

/// c1 (plan `position_rejection_has_no_effect`): a position rejected at reset changes nothing — the session keeps its
/// boundary, token, serial, seed and ordinal, and no stream is consumed. (A catalog rejects such a position at parse; the
/// catalog's entry is replaced here, inside the crate, to reach the reset path.)
#[test]
fn position_rejected_at_reset_has_no_effect() {
    let deck = spire_core::content::Catalog::standard().starter_deck(Character::Ironclad).unwrap().iter().map(|c| json!({"card": c.as_str(), "upgrades": 0})).collect::<Vec<_>>();
    let entry = json!({"id": "a", "character": "ironclad", "ascension": 0, "encounter": "jaw_worm", "hp": 70, "max_hp": 80, "gold": 9, "deck": deck,
        "relics": ["burning_blood", "toolbox"], "potions": [null, null, null]});
    let mut cat = CombatTrainingCatalog::standard_from_json(&recipe_catalog(vec![entry])).unwrap();
    let mut s = CombatTrainingSession::new_standard(cat.clone(), Partition::Test).unwrap();
    let before = s.reset(Some(5), None).unwrap();
    let diag = s.diagnostic();
    // A statically valid position on a Toolbox start: the native opening stops at Toolbox's offer, so the core rejects it
    // after entry, on the scratch Run.
    cat.set_standard_position_for_test("a", spire_core::combat::position::Position { powers: vec![(spire_core::combat::position::power_id("strength").unwrap(), 2)], ..Default::default() });
    let mut bad = CombatTrainingSession::new_standard(cat, Partition::Test).unwrap();
    let start = bad.reset(Some(5), None);
    assert!(matches!(start, Err(RunSessionError::UnsupportedBoundary(_))), "{start:?}");
    assert_eq!(bad.diagnostic()["entry_id"], Value::Null, "a rejected first reset leaves the session unreset");
    let mut twin = CombatTrainingSession::new_standard(s.catalog_for_test(), Partition::Test).unwrap();
    twin.reset(Some(5), None).unwrap();
    s.set_catalog_for_test(bad.catalog_for_test());
    assert!(matches!(s.reset(Some(6), None), Err(RunSessionError::UnsupportedBoundary(_))));
    assert_eq!((s.decision().unwrap(), s.diagnostic()), (before.clone(), diag), "boundary, token, seed, ordinal and steps unchanged");
    // The episode continues exactly as an untouched twin's: the old token is valid and the Run and its streams are the same.
    let first = |b: &CombatPolicyBoundary| RunActionIndex(b.observation.action_mask.iter().position(|m| *m == 1).unwrap() as i64);
    let (a, b) = (s.step(before.token, first(&before)).unwrap(), twin.step(twin.decision().unwrap().token, first(&before)).unwrap());
    assert_eq!((a.boundary.observation, a.reward), (b.boundary.observation, b.reward));
}

/// Design §4.11 (measured, not gated): recipe load (prefix replay + public root, once per entry) and per-reset sampling, and
/// a synthetic position's reset (native opening + `apply_position`) against a fresh start. Release:
/// `cargo test -p spire-env --release --lib m34s_recipes::tactical_start_costs -- --ignored --nocapture`.
#[test]
#[ignore = "measurement"]
fn tactical_start_costs() {
    use std::time::Instant;
    let median = |mut v: Vec<u128>| { v.sort_unstable(); v[v.len() / 2] };
    for (character, seed) in [(Character::Ironclad, 20261120), (Character::Silent, 20261121), (Character::Defect, 20261122), (Character::Watcher, 20261123)] {
        let mut config = StandardRunConfig::new(character, 20, Partition::Test);
        config.record_replay = true;
        let mut s = RunSession::new_standard(config).unwrap();
        let mut b = s.reset(Some(seed)).unwrap();
        let mut steps = 0;
        while s.public_root().is_err() { let i = b.observation.action_mask.iter().position(|&v| v != 0).unwrap(); b = s.step(b.token, RunActionIndex(i as i64)).unwrap().boundary; steps += 1; }
        let record = s.record().unwrap().unwrap().to_json().unwrap();
        let loads = (0..5).map(|_| { let t = Instant::now(); load(vec![recipe("r", &record, steps)], std::slice::from_ref(&record)).unwrap(); t.elapsed().as_micros() }).collect();
        let mut session = CombatTrainingSession::new_standard(load(vec![recipe("r", &record, steps)], &[record.clone()]).unwrap(), Partition::Test).unwrap();
        let resets = (0..100).map(|seed| { let t = Instant::now(); session.reset(Some(seed), None).unwrap(); t.elapsed().as_micros() }).collect();
        println!("recipe {} A20 prefix {steps} steps, record {} bytes: load median {} µs, reset median {} µs", character.as_str(), record.len(), median(loads), median(resets));
    }
    let deck = spire_core::content::Catalog::standard().starter_deck(Character::Ironclad).unwrap().iter().map(|c| json!({"card": c.as_str(), "upgrades": 0})).collect::<Vec<_>>();
    let base = json!({"id": "fresh", "character": "ironclad", "ascension": 20, "encounter": "gremlin_nob", "hp": 70, "max_hp": 80, "gold": 9, "deck": deck,
        "relics": ["burning_blood"], "potions": [null, null]});
    let mut synthetic = base.clone();
    synthetic["id"] = json!("synthetic");
    synthetic["position"] = json!({"player": {"powers": {"strength": 2}}, "enemies": [{"slot": 0, "hp": 50}],
        "piles": {"hand": [{"card": "bash"}, {"card": "strike"}, {"card": "strike"}, {"card": "defend"}, {"card": "defend"}],
            "draw": [{"card": "strike"}, {"card": "strike"}, {"card": "strike"}, {"card": "defend"}, {"card": "defend"}]}});
    let mut session = CombatTrainingSession::new_standard(CombatTrainingCatalog::standard_from_json(&recipe_catalog(vec![base, synthetic])).unwrap(), Partition::Test).unwrap();
    for id in ["fresh", "synthetic"] {
        let resets = (0..200).map(|seed| { let t = Instant::now(); session.reset(Some(seed), Some(id)).unwrap(); t.elapsed().as_micros() }).collect();
        println!("{id} start (Ironclad A20 Gremlin Nob) reset median {} µs", median(resets));
    }
}
