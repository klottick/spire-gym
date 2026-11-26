//! M41 (spire-n6b, design-v2 §7): native batched playouts on `PublicSearch` under the declared public rule
//! `spire-gym-public-playout-v1`. T2 verb parity, T4 equivalence with one-action-at-a-time Mask stepping (`playout_action`
//! then `step`), T5 an independent reference rebuilt from the Full observation, T6 summary parity, T7 CRN and seeds, T8
//! privacy, T9 stops, T10 preflight atomicity, T11 fault isolation, T12 truncation and call caps.
use super::m34s_transplant::{standard_combat_session, standard_entry};
use super::standard_traversal::{short_creation, simple_action, standard_session, CHARACTERS};
use crate::*;
use serde_json::{json, Value};
use spire_core::combat::{CardDestination, Combat, CombatAction};
use spire_core::content::{Catalog, CardId, Character};
use spire_core::run::{diagnostics, GameAction, Run};

const SEED: u64 = 0x41_0001;
const WIDE: SearchLimits = SearchLimits { max_live_branches: 64, max_branches: 2_000, max_steps_per_branch: 5_000, max_total_steps: 10_000_000 };
fn limits(each: u64) -> SearchLimits { SearchLimits { max_steps_per_branch: each, ..WIDE } }

fn standalone_root(character: Character, ascension: u8, encounter: &str, seed: u64) -> PublicRoot {
    let mut s = standard_combat_session(vec![standard_entry("e", character, ascension, encounter, None)]);
    s.reset(Some(seed), None).unwrap();
    s.public_root().unwrap()
}
fn potion_root(character: Character, seed: u64) -> PublicRoot {
    let mut entry = standard_entry("p", character, 0, "jaw_worm", None);
    entry["potions"] = json!(["attack_potion", "block_potion", if character == Character::Watcher { json!("stance_potion") } else { Value::Null }]);
    let mut s = standard_combat_session(vec![entry]);
    s.reset(Some(seed), None).unwrap();
    s.public_root().unwrap()
}
fn run_root(character: Character, ascension: u8, seed: u8) -> PublicRoot {
    let (mut s, _) = standard_session(character, ascension, seed, short_creation());
    let mut b = s.decision().unwrap();
    while s.run_for_test().unwrap().decision().unwrap().combat.is_none() {
        b = s.step(b.token, RunActionIndex(simple_action(&b.observation) as i64)).unwrap().boundary;
    }
    s.public_root().unwrap()
}
fn dome_root(character: Character, seed: u64) -> PublicRoot {
    let mut e = standard_entry("dome", character, 20, "jaw_worm", None);
    e["relics"].as_array_mut().unwrap().push(json!("runic_dome"));
    let mut s = standard_combat_session(vec![e]);
    s.reset(Some(seed), None).unwrap();
    s.public_root().unwrap()
}
/// 4 characters × {A0, A20} × {standalone start, Standard Run combat}.
fn roots() -> Vec<(String, PublicRoot)> {
    let mut out = vec![];
    for (ci, c) in CHARACTERS.into_iter().enumerate() { for (ai, a) in [0u8, 20].into_iter().enumerate() {
        let seed = 0x50 + (ci * 2 + ai) as u8;
        out.push((format!("{c:?} A{a} Standard Run"), run_root(c, a, seed)));
        let encounter = ["jaw_worm", "cultist", "gremlin_nob", "two_louse"][(ci + ai) % 4];
        out.push((format!("{c:?} A{a} {encounter}"), standalone_root(c, a, encounter, u64::from(seed))));
    }}
    out
}
#[derive(Clone, Copy, Debug)]
enum Layout { Dense, Bounds, Play }
fn search(root: PublicRoot, layout: Layout, workers: usize, l: SearchLimits) -> PublicSearch {
    let config = SearchConfig { research_seed: SEED, limits: l };
    let s = match layout {
        Layout::Dense => PublicSearch::new(root, config).unwrap(),
        Layout::Bounds => PublicSearch::new_with_combat_layout(root, config, StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Bounds)).unwrap(),
        Layout::Play => PublicSearch::new_with_combat_layout(root, config, StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play)).unwrap(),
    };
    s.with_workers(workers).unwrap()
}
/// Samples 0..6, a repeated sample and two forks (as M35's branch set).
fn branch_set(s: &mut PublicSearch) -> Vec<BranchId> {
    let mut b = (0..6).map(|k| s.branch(Some(k)).unwrap()).collect::<Vec<_>>();
    b.push(s.branch(Some(3)).unwrap());
    b.push(s.fork(b[0]).unwrap());
    b.push(s.fork(b[1]).unwrap());
    b
}
fn reqs(s: &PublicSearch, ids: &[BranchId]) -> Vec<BranchPlayoutRequest> {
    ids.iter().map(|b| BranchPlayoutRequest { branch: *b, token: s.decision(*b).unwrap().token }).collect()
}
fn serial(t: RunDecisionToken) -> RunDecisionToken { RunDecisionToken([0, t.0[1]]) }
/// An outcome without the branch-id half of its token (two searches allocate different ids).
fn normalized(o: &BranchPlayoutOutcome) -> BranchPlayoutOutcome {
    match o.clone() { BranchPlayoutOutcome::Finished(mut p) => { p.token = serial(p.token); BranchPlayoutOutcome::Finished(p) } f => f }
}
fn normalized_decision(s: &PublicSearch, b: BranchId) -> Result<BranchBoundary, RunSessionError> {
    s.decision(b).map(|mut d| { d.token = serial(d.token); d })
}
fn same_state(a: &PublicSearch, ab: &[BranchId], b: &PublicSearch, bb: &[BranchId]) {
    assert_eq!(a.counters(), b.counters());
    for (x, y) in ab.iter().zip(bb) {
        assert_eq!(a.actions(*x).unwrap(), b.actions(*y).unwrap());
        assert_eq!(normalized_decision(a, *x), normalized_decision(b, *y));
    }
}

/// The player and enemy rows of a published observation, as the playout summary reports them (Learn's `balance` reads
/// rows with `max_hp > 0`).
fn observed_summary(o: &RunObservation) -> PlayoutSummary {
    let p = o.group("player").unwrap();
    let e = o.group("enemies").unwrap();
    let cols = *e.shape.last().unwrap();
    let enemies = e.values.chunks_exact(cols).filter(|row| row[2] > 0).map(|row| EnemyHp { hp: row[1] as i32, max_hp: row[2] as i32 }).collect();
    let pot = o.group("potions").unwrap();
    let potions = pot.values.iter().zip(&pot.known).filter(|(_, k)| **k == 1).map(|(v, _)| *v as i32).collect();
    PlayoutSummary { player_hp: p.values[0] as i32, player_max_hp: p.values[1] as i32, player_gold: p.values[2] as u32, enemies, potions }
}

/// T4 reference: the same rule one action at a time — `playout_action`, then `step(.., Mask)` — until terminal,
/// truncation, the call cap, a refusal or a fault. Returns the expected outcome (summary from the Full decision).
fn reference(s: &mut PublicSearch, b: BranchId, config: &PlayoutConfig) -> BranchPlayoutOutcome {
    let start = s.decision(b).unwrap();
    reference_from(s, b, config, start.token, start.info["branch_steps"].as_u64().unwrap())
}
fn reference_from(s: &mut PublicSearch, b: BranchId, config: &PlayoutConfig, mut token: RunDecisionToken, mut branch_steps: u64) -> BranchPlayoutOutcome {
    let cap = config.max_steps.unwrap_or(u64::MAX);
    let (mut steps, mut reward, mut terminated, mut truncated) = (0u64, 0, false, false);
    let (end_reason, refusal): (&'static str, Option<&'static str>) = loop {
        if steps == cap { break ("max_steps", None) }
        let index = match s.playout_action(b, &config.rule, config.playout_seed) {
            Ok(i) => i,
            Err(RunSessionError::Input("branch has no legal action")) => break ("refused", Some(REFUSAL_NO_LEGAL_ACTION)),
            Err(e) => panic!("playout_action: {e:?}"),
        };
        match s.step(b, token, index, BranchView::Mask) {
            Ok(r) => {
                steps += 1; branch_steps += 1;
                (reward, terminated, truncated, token) = (r.reward, r.terminated, r.truncated, r.token);
                if r.terminated { break (match r.info["terminal_reason"].as_str().unwrap() { "victory" => "victory", "death" => "death", _ => "escaped" }, None) }
                if r.truncated { break ("step_limit", None) }
            }
            Err(RunSessionError::UnsupportedBoundary(HIDDEN_ORDER_PICK)) => break ("refused", Some(REFUSAL_HIDDEN_ORDER_PICK)),
            Err(error) => {
                // A live branch with its own token and a rule-picked index fails only inside the step (a fault).
                steps += 1; branch_steps += 1;
                let stage = if s.actions(b).unwrap().len() as u64 == branch_steps { "encode" } else { "step" };
                return BranchPlayoutOutcome::Fault { committed: true, error, stage, steps, branch_steps }
            }
        }
    };
    let summary = observed_summary(&s.decision(b).unwrap().observation);
    BranchPlayoutOutcome::Finished(BranchPlayout { token, reward, terminated, truncated, end_reason, refusal, steps, branch_steps, summary })
}
/// Compare a fault outcome on the error class and step counts (the reference cannot see `stage`/`committed` directly).
fn assert_same(label: &str, got: &BranchPlayoutOutcome, want: &BranchPlayoutOutcome) {
    match (got, want) {
        (BranchPlayoutOutcome::Fault { error: e1, steps: s1, branch_steps: b1, .. }, BranchPlayoutOutcome::Fault { error: e2, steps: s2, branch_steps: b2, .. }) =>
            assert_eq!((e1, s1, b1), (e2, s2, b2), "{label}"),
        _ => assert_eq!(normalized(got), normalized(want), "{label}"),
    }
}

/// Runs `playout_many` on one search and the reference on a twin, and checks outcomes, accepted actions, decisions and
/// counters agree. Returns the outcomes.
fn check_equivalence(label: &str, root: &PublicRoot, layout: Layout, workers: usize, l: SearchLimits, config: &PlayoutConfig) -> Vec<BranchPlayoutOutcome> {
    let (mut a, mut b) = (search(root.clone(), layout, workers, l), search(root.clone(), layout, 1, l));
    let (ia, ib) = (branch_set(&mut a), branch_set(&mut b));
    let got = a.playout_many(&reqs(&a, &ia), config).unwrap();
    let want = ib.iter().map(|x| reference(&mut b, *x, config)).collect::<Vec<_>>();
    for (i, (g, w)) in got.iter().zip(&want).enumerate() { assert_same(&format!("{label} entry {i}"), g, w); }
    same_state(&a, &ia, &b, &ib);
    got
}

#[test]
fn t2_verb_names_are_the_standard_vocab_and_rules_validate() {
    let vocab = crate::standard_vocab_manifest()["families"]["verb"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_owned()).collect::<Vec<_>>();
    assert_eq!(vocab[0], "padding");
    assert_eq!(STANDARD_PLAYOUT_VERBS.iter().map(|v| v.to_string()).collect::<Vec<_>>(), vocab[1..], "every Standard verb, in vocab order");
    assert_eq!(PlayoutRule::default().excluded_verbs(), vec!["discard_potion", "end_turn"]);
    assert_eq!(PlayoutRule::default(), PlayoutRule::uniform_exclude_verbs(["end_turn", "discard_potion", "end_turn"]).unwrap());
    assert_eq!(PlayoutRule::uniform_exclude_verbs(Vec::<String>::new()).unwrap().excluded_verbs(), Vec::<&str>::new());
    for bad in ["padding", "End_Turn", "", "end_turn "] {
        assert_eq!(PlayoutRule::uniform_exclude_verbs([bad]).err(), Some(RunSessionError::Input("unknown playout verb")), "{bad:?}");
    }
    assert_eq!(PlayoutConfig::default(), PlayoutConfig { rule: PlayoutRule::default(), max_steps: None, playout_seed: 0 });
    assert_eq!(PUBLIC_PLAYOUT_RULE_ID, "spire-gym-public-playout-v1");
    assert_ne!(REFUSAL_HIDDEN_ORDER_PICK, HIDDEN_ORDER_PICK, "machine-readable refusal codes, not the error prose");
    assert_eq!((REFUSAL_HIDDEN_ORDER_PICK, REFUSAL_NO_LEGAL_ACTION), ("HIDDEN_ORDER_PICK", "NO_LEGAL_ACTION"));
}

/// T2: for every legal row the rule sees, its verb id equals the encoder's `actions[:, verb]` column, in dense and compact
/// layouts, over random playouts on many starts (with potions for the potion, stance and card-offer verbs).
#[test]
fn t2_verb_ids_match_the_encoder_column() {
    let vocab = crate::standard_vocab_manifest()["families"]["verb"].as_array().unwrap().clone();
    let mut seen = std::collections::BTreeSet::new();
    let mut starts = roots();
    for c in CHARACTERS { starts.push((format!("{c:?} potions"), potion_root(c, 7))); }
    for (label, root) in starts {
        for layout in [Layout::Dense, Layout::Bounds] {
            let mut s = search(root.clone(), layout, 1, limits(60));
            for k in 0..3 {
                let b = s.branch(Some(k)).unwrap();
                let rule = PlayoutRule::uniform_exclude_verbs(Vec::<String>::new()).unwrap();
                loop {
                    let Ok(d) = s.decision(b) else { break };
                    let legal = s.legal_for_test(b);
                    let actions = d.observation.group("actions").unwrap();
                    let cols = *actions.shape.last().unwrap();
                    for (i, action) in legal.iter().enumerate() {
                        let id = crate::public_search::standard_verb_id(action);
                        assert_eq!(id, actions.values[i * cols], "{label} {layout:?}: row {i} {action:?}");
                        seen.insert(vocab[id as usize].as_str().unwrap().to_owned());
                    }
                    if legal.is_empty() || d.info["end_reason"] == "step_limit" { break }
                    let i = s.playout_action(b, &rule, k).unwrap();
                    match s.step(b, d.token, i, BranchView::Mask) {
                        Ok(r) if r.terminated || r.truncated => break,
                        Ok(_) => {}
                        Err(RunSessionError::UnsupportedBoundary(HIDDEN_ORDER_PICK)) => break,
                        Err(e) => panic!("{label}: {e:?}"),
                    }
                }
            }
        }
    }
    eprintln!("M41 T2 verbs reached: {seen:?}");
    for verb in ["play_card", "end_turn", "use_potion", "discard_potion", "combat_select_card", "choose_card_offer"] {
        assert!(seen.contains(verb), "{verb} reached (seen {seen:?})");
    }
}

/// T4: `playout_many` equals one-action-at-a-time Mask stepping under the same rule and stream, on every start, dense
/// and compact, at 1/4/16 workers, with the default rule, the all-legal rule, a custom exclusion, call caps and seeds.
#[test]
fn t4_playouts_equal_one_action_at_a_time() {
    let configs = [
        PlayoutConfig::default(),
        PlayoutConfig { rule: PlayoutRule::uniform_exclude_verbs(Vec::<String>::new()).unwrap(), max_steps: Some(7), playout_seed: 3 },
        PlayoutConfig { rule: PlayoutRule::uniform_exclude_verbs(["play_card"]).unwrap(), max_steps: None, playout_seed: 11 },
    ];
    let mut reasons = std::collections::BTreeMap::<&'static str, usize>::new();
    for (i, (label, root)) in roots().into_iter().enumerate() {
        let layout = [Layout::Dense, Layout::Bounds, Layout::Play][i % 3];
        let workers = [1, 4, 16][i % 3];
        for (j, config) in configs.iter().enumerate() {
            for o in check_equivalence(&format!("{label} {layout:?} w{workers} c{j}"), &root, layout, workers, limits(40), config) {
                if let BranchPlayoutOutcome::Finished(p) = o { *reasons.entry(p.end_reason).or_default() += 1; }
            }
        }
    }
    // M40c Runic Dome roots (hidden intents), dense and compact.
    for (i, c) in CHARACTERS.into_iter().enumerate() {
        let layout = [Layout::Dense, Layout::Bounds][i % 2];
        for o in check_equivalence(&format!("{c:?} dome {layout:?}"), &dome_root(c, 3 + i as u64), layout, 4, limits(40), &configs[0]) {
            if let BranchPlayoutOutcome::Finished(p) = o { *reasons.entry(p.end_reason).or_default() += 1; }
        }
    }
    eprintln!("M41 T4 end reasons: {reasons:?}");
    for reason in ["victory", "step_limit", "max_steps"] { assert!(reasons.contains_key(reason), "{reason} covered: {reasons:?}"); }
}

/// T4: outcomes and final state do not depend on workers, input order, grouping or splitting the call into
/// `max_steps = 1` calls (only the per-call `steps` and end reasons of intermediate calls differ).
#[test]
fn t4_worker_order_grouping_and_chunking_invariance() {
    let root = standalone_root(Character::Silent, 20, "gremlin_nob", 5);
    let config = PlayoutConfig::default();
    let l = limits(50);
    let mut base = search(root.clone(), Layout::Dense, 1, l);
    let ib = branch_set(&mut base);
    let want = base.playout_many(&reqs(&base, &ib), &config).unwrap();
    for workers in [2, 4, 16] {
        let mut s = search(root.clone(), Layout::Dense, workers, l);
        let ids = branch_set(&mut s);
        // Reversed input order, in two groups.
        let order = (0..ids.len()).rev().collect::<Vec<_>>();
        let (first, second) = order.split_at(4);
        let mut got = vec![None; ids.len()];
        for group in [first, second] {
            let r = s.playout_many(&group.iter().map(|i| BranchPlayoutRequest { branch: ids[*i], token: s.decision(ids[*i]).unwrap().token }).collect::<Vec<_>>(), &config).unwrap();
            for (i, o) in group.iter().zip(r) { got[*i] = Some(o); }
        }
        for (g, w) in got.iter().zip(&want) { assert_eq!(normalized(g.as_ref().unwrap()), normalized(w)); }
        same_state(&s, &ids, &base, &ib);
    }
    // Chunked: max_steps = 1 calls until every branch ends; the final state equals one unbounded call.
    let mut s = search(root, Layout::Dense, 4, l);
    let ids = branch_set(&mut s);
    let one = PlayoutConfig { max_steps: Some(1), ..config.clone() };
    let mut live = ids.clone();
    let mut last = std::collections::BTreeMap::new();
    while !live.is_empty() {
        let out = s.playout_many(&reqs(&s, &live), &one).unwrap();
        let mut next = vec![];
        for (b, o) in live.iter().zip(out) {
            match &o { BranchPlayoutOutcome::Finished(p) if p.end_reason == "max_steps" => next.push(*b), _ => {} }
            last.insert(*b, o);
        }
        live = next;
    }
    same_state(&s, &ids, &base, &ib);
    for (b, w) in ids.iter().zip(&want) {
        match (&last[b], w) {
            (BranchPlayoutOutcome::Finished(g), BranchPlayoutOutcome::Finished(w)) => {
                assert_eq!((g.reward, g.terminated, g.truncated, g.end_reason, g.refusal, g.branch_steps, &g.summary), (w.reward, w.terminated, w.truncated, w.end_reason, w.refusal, w.branch_steps, &w.summary));
            }
            (g, w) => panic!("{g:?} vs {w:?}"),
        }
    }
}

/// T5: an independent reference — each choice rebuilt from the Full observation (mask, the verb column read through the
/// vocab manifest, the excluded names) and the public pick function — gives the same trajectory and summary.
#[test]
fn t5_independent_reference_from_the_full_observation() {
    let vocab = crate::standard_vocab_manifest()["families"]["verb"].as_array().unwrap().clone();
    for (label, root) in roots().into_iter().step_by(3).chain([("dome".to_owned(), dome_root(Character::Defect, 4))]) {
        for (layout, excluded, seed) in [(Layout::Dense, vec!["end_turn", "discard_potion"], 0u64), (Layout::Bounds, vec!["play_card"], 9)] {
            let config = PlayoutConfig { rule: PlayoutRule::uniform_exclude_verbs(excluded.clone()).unwrap(), max_steps: None, playout_seed: seed };
            let (mut a, mut f) = (search(root.clone(), layout, 1, limits(40)), search(root.clone(), layout, 1, limits(40)));
            let (ia, i_f) = (branch_set(&mut a), branch_set(&mut f));
            let got = a.playout_many(&reqs(&a, &ia), &config).unwrap();
            for ((b, g), x) in i_f.iter().zip(&got).zip(&ia) {
                let origin = f.origin_for_test(*b);
                let mut d = f.decision(*b).unwrap();
                let mut refused = None;
                loop {
                    if d.info["end_reason"] == "step_limit" || d.observation.action_mask.iter().all(|m| *m == 0) { break }
                    let actions = d.observation.group("actions").unwrap();
                    let cols = *actions.shape.last().unwrap();
                    let legal = d.observation.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect::<Vec<_>>();
                    let name = |i: usize| vocab[actions.values[i * cols] as usize].as_str().unwrap();
                    let preferred = legal.iter().copied().filter(|i| !excluded.contains(&name(*i))).collect::<Vec<_>>();
                    let pool = if preferred.is_empty() { legal } else { preferred };
                    let ordinal = f.actions(*b).unwrap().len() as u64;
                    let i = pool[spire_core::run::research_playout_pick(&origin, seed, ordinal, pool.len())];
                    match f.step(*b, d.token, RunActionIndex(i as i64), BranchView::Full) {
                        Ok(r) if r.terminated || r.truncated => break,
                        Ok(_) => d = f.decision(*b).unwrap(),
                        Err(RunSessionError::UnsupportedBoundary(HIDDEN_ORDER_PICK)) => { refused = Some(REFUSAL_HIDDEN_ORDER_PICK); break }
                        Err(e) => panic!("{label}: {e:?}"),
                    }
                }
                assert_eq!(a.actions(*x).unwrap(), f.actions(*b).unwrap(), "{label} {layout:?}: same trajectory");
                let BranchPlayoutOutcome::Finished(p) = g else { panic!("{label}: {g:?}") };
                assert_eq!(p.refusal, refused);
                assert_eq!(p.summary, observed_summary(&f.decision(*b).unwrap().observation), "{label} {layout:?}: T6 summary");
            }
        }
    }
}

/// T5/D1: a fault only the Full publication raises is the declared divergence — the Full step faults (`encode`) where
/// the playout (which encodes no view) continues exactly as Mask stepping does.
#[test]
fn t5_full_only_encode_fault_is_the_declared_divergence() {
    use crate::run_observation::test_range;
    let root = standalone_root(Character::Ironclad, 0, "jaw_worm", 12);
    // A compact layout (its publication range-checks values) with capacities unique to this test, so the process-wide
    // range hook touches no other test's publications.
    let mut compact = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Bounds);
    compact.capacities = RunCapacities::standard_default();
    compact.capacities.powers -= 41;
    let caps = compact.capacities.clone();
    let config = SearchConfig { research_seed: SEED, limits: limits(30) };
    let (mut full, mut play) = (PublicSearch::new_with_combat_layout(root.clone(), config, compact.clone()).unwrap(), PublicSearch::new_with_combat_layout(root, config, compact).unwrap());
    let (fb, pb) = (full.branch(Some(0)).unwrap(), play.branch(Some(0)).unwrap());
    let rule = PlayoutRule::default();
    let first = full.playout_action(fb, &rule, 0).unwrap();
    let d = full.decision(fb).unwrap();
    let play_token = play.decision(pb).unwrap().token;
    let hook = test_range::inflate(caps, "player", 2, i64::from(i32::MAX), (99, 99));
    assert!(matches!(full.step(fb, d.token, first, BranchView::Full), Err(RunSessionError::Capacity { group: "player.gold", .. })));
    let out = play.playout_many(&[BranchPlayoutRequest { branch: pb, token: play_token }], &PlayoutConfig { max_steps: Some(5), ..PlayoutConfig::default() }).unwrap();
    drop(hook);
    let BranchPlayoutOutcome::Finished(p) = &out[0] else { panic!("{out:?}") };
    assert_eq!((p.steps, p.end_reason), (5, "max_steps"));
    assert_eq!(play.actions(pb).unwrap()[0], first.0);
}

/// T6: the summary equals the published player and enemy rows at terminal, truncated, call-cap and refused ends, dense
/// and compact; Learn's `balance` from either agrees.
#[test]
fn t6_summary_matches_the_published_rows() {
    fn balance(s: &PlayoutSummary) -> f64 {
        let hp: i64 = s.enemies.iter().map(|e| i64::from(e.hp.max(0))).sum();
        let max: i64 = s.enemies.iter().map(|e| i64::from(e.max_hp)).sum();
        f64::from(s.player_hp.max(0)) / f64::from(s.player_max_hp.max(1)) - hp as f64 / if max == 0 { 1.0 } else { max as f64 }
    }
    let mut ends = std::collections::BTreeSet::new();
    for (root, layout, cap) in [(standalone_root(Character::Ironclad, 0, "cultist", 3), Layout::Dense, None), (standalone_root(Character::Watcher, 20, "two_louse", 4), Layout::Play, Some(6)),
                                (run_root(Character::Defect, 0, 0x61), Layout::Bounds, None), (standalone_root(Character::Silent, 0, "gremlin_nob", 2), Layout::Dense, Some(3))] {
        let mut s = search(root, layout, 4, limits(25));
        let ids = branch_set(&mut s);
        let out = s.playout_many(&reqs(&s, &ids), &PlayoutConfig { max_steps: cap, ..PlayoutConfig::default() }).unwrap();
        for (b, o) in ids.iter().zip(&out) {
            let BranchPlayoutOutcome::Finished(p) = o else { panic!("{o:?}") };
            let observed = observed_summary(&s.decision(*b).unwrap().observation);
            assert_eq!(p.summary, observed);
            assert!((balance(&p.summary) - balance(&observed)).abs() < 1e-12);
            assert!(p.summary.enemies.len() <= s.enemy_capacity());
            ends.insert(p.end_reason);
        }
    }
    assert!(ends.contains("max_steps") && ends.contains("step_limit"), "{ends:?}");
}

/// T7: common random numbers per (research_seed, k): equal samples draw equally; the playout seed and the sample change
/// the trace; a fork under seed 0 replays its parent's choices.
#[test]
fn t7_common_random_numbers_and_seeds() {
    let root = standalone_root(Character::Defect, 0, "jaw_worm", 8);
    let mut s = search(root, Layout::Dense, 1, limits(60));
    let trace = |s: &mut PublicSearch, b: BranchId, seed: u64| {
        s.playout_many(&[BranchPlayoutRequest { branch: b, token: s.decision(b).unwrap().token }], &PlayoutConfig { playout_seed: seed, ..PlayoutConfig::default() }).unwrap();
        s.actions(b).unwrap()
    };
    let (a, b) = (s.branch(Some(4)).unwrap(), s.branch(Some(4)).unwrap());
    assert_eq!(trace(&mut s, a, 0), trace(&mut s, b, 0), "same k, same seed: identical playouts");
    let mut differ_seed = false;
    let mut differ_k = false;
    for k in 10..14 {
        let (x, y, z) = (s.branch(Some(k)).unwrap(), s.branch(Some(k)).unwrap(), s.branch(Some(k + 100)).unwrap());
        let tx = trace(&mut s, x, 0);
        differ_seed |= tx != trace(&mut s, y, 1);
        differ_k |= tx != trace(&mut s, z, 0);
    }
    assert!(differ_seed && differ_k);
    // CRN across two root actions: the same draws at equal ordinals, so the first playout pick after a shared prefix agrees
    // with a fork's.
    let p = s.branch(Some(20)).unwrap();
    let rule = PlayoutRule::default();
    let first = s.playout_action(p, &rule, 0).unwrap();
    let d = s.decision(p).unwrap();
    s.step(p, d.token, first, BranchView::Mask).unwrap();
    let f = s.fork(p).unwrap();
    assert_eq!(trace(&mut s, p, 0), trace(&mut s, f, 0), "a fork under seed 0 replays its parent");
}

/// T8: natural pairs (different true worlds, equal public roots) give identical playouts; outputs carry no key material.
#[test]
fn t8_privacy_natural_pairs_and_outputs() {
    let strike = Catalog::standard().starter_deck(Character::Silent).unwrap()[0];
    let mut entry = standard_entry("e", Character::Silent, 0, "spheric_guardian", None);
    entry["deck"] = json!(vec![json!({"card": strike.as_str(), "upgrades": 0}); 7]);
    entry["relics"] = json!([]);
    let mut seen = std::collections::BTreeMap::<[u8; 32], (PublicRoot, Run)>::new();
    let pair = (0..400u64).find_map(|seed| {
        let mut session = standard_combat_session(vec![entry.clone()]);
        session.reset(Some(seed), None).unwrap();
        let root = session.public_root().unwrap();
        let world = session.run_for_test().unwrap().clone();
        match seen.get(&root.digest()) {
            Some((prior, prior_world)) if prior == &root && prior_world != &world => Some((prior.clone(), root)),
            Some(_) => None,
            None => { seen.insert(root.digest(), (root, world)); None }
        }
    }).expect("a natural pair");
    let (mut x, mut y) = (search(pair.0, Layout::Dense, 4, limits(80)), search(pair.1, Layout::Dense, 1, limits(80)));
    let (ix, iy) = (branch_set(&mut x), branch_set(&mut y));
    let (ox, oy) = (x.playout_many(&reqs(&x, &ix), &PlayoutConfig::default()).unwrap(), y.playout_many(&reqs(&y, &iy), &PlayoutConfig::default()).unwrap());
    assert_eq!(ox.iter().map(normalized).collect::<Vec<_>>(), oy.iter().map(normalized).collect::<Vec<_>>());
    same_state(&x, &ix, &y, &iy);
    let text = format!("{ox:?}{:?}", PlayoutConfig::default());
    for b in &ix { assert!(!text.contains(&super::support::hex(&x.origin_for_test(*b))), "no key material in outputs"); }
}

/// M40a fixture: Defect Seek over Wounds shuffled into the draw pile (`upgrades` 0: one pick; 1: two picks).
fn seek_root(seed: u64, upgrades: u32) -> PublicRoot {
    let mut s = standard_combat_session(vec![standard_entry("e", Character::Defect, 0, "jaw_worm", None)]);
    s.reset(Some(seed), None).unwrap();
    let mut run = s.run_for_test().unwrap().clone();
    let c: &mut Combat = diagnostics::supplied_combat(&mut run).unwrap();
    for _ in 0..2 { c.diagnostic_create_card(CardId::Wound, 0, CardDestination::ShuffledDraw).unwrap(); }
    let seek = c.diagnostic_create_card(CardId::Seek, upgrades, CardDestination::Hand).unwrap();
    let d = run.decision().unwrap();
    let play = d.candidates.iter().find(|c| matches!(c.action, GameAction::Combat(CombatAction::PlayCard { instance, .. }) if instance == seek)).unwrap().action.clone();
    run.step(&play).unwrap();
    PublicRoot::of(&run, &RunCapacities::standard_default()).unwrap()
}
fn wound_pick(s: &mut PublicSearch, b: BranchId, index: i64) -> bool {
    let run = s.branch_run_for_test(b).clone();
    let d = run.decision().unwrap();
    let wounds = d.combat.as_ref().unwrap().selection.as_ref().map(|sel| sel.candidates.iter().filter(|c| c.id == CardId::Wound).map(|c| c.instance.0).collect::<Vec<_>>()).unwrap_or_default();
    matches!(&d.candidates[index as usize].action, GameAction::Combat(CombatAction::SelectCard { instance }) if wounds.contains(instance))
}

/// T4/T9 (M40a): a resolvable single pick among identical Wounds is applied exactly as `step` applies it; a two-pick grid
/// refuses (`HIDDEN_ORDER_PICK`) with no effect, the branch stays live, siblings finish, and a repeat call refuses again.
#[test]
fn t9_identical_copy_picks_resolve_or_refuse() {
    let config = PlayoutConfig { rule: PlayoutRule::uniform_exclude_verbs(Vec::<String>::new()).unwrap(), max_steps: None, playout_seed: 0 };
    let mut resolved = 0;
    for seed in 1..=4u64 {
        let root = seek_root(seed, 0);
        let mut probe = search(root.clone(), Layout::Dense, 1, limits(40));
        for k in 0..6 { let b = probe.branch(Some(k)).unwrap(); let i = probe.playout_action(b, &config.rule, 0).unwrap(); resolved += usize::from(wound_pick(&mut probe, b, i.0)); }
        check_equivalence(&format!("seek seed {seed}"), &root, Layout::Dense, 4, limits(40), &config);
    }
    assert!(resolved > 0, "some first pick is a Wound (the M40a rule applies)");
    let mut refused = 0;
    for seed in 1..=4u64 {
        let root = seek_root(seed, 1);
        for o in check_equivalence(&format!("seek+ seed {seed}"), &root, Layout::Dense, 1, limits(40), &config) {
            let BranchPlayoutOutcome::Finished(p) = o else { panic!("{o:?}") };
            if p.end_reason == "refused" { assert_eq!(p.refusal, Some(REFUSAL_HIDDEN_ORDER_PICK)); refused += 1; }
        }
        let mut s = search(root, Layout::Dense, 1, limits(40));
        let ids = branch_set(&mut s);
        let first = s.playout_many(&reqs(&s, &ids), &config).unwrap();
        for (b, o) in ids.iter().zip(&first) {
            let BranchPlayoutOutcome::Finished(p) = o else { panic!("{o:?}") };
            if p.end_reason != "refused" { continue }
            let before = (s.actions(*b).unwrap(), s.decision(*b).unwrap(), s.counters());
            assert_eq!(p.token, before.1.token, "no effect: same token");
            assert!(!p.terminated && !p.truncated);
            let again = s.playout_many(&[BranchPlayoutRequest { branch: *b, token: p.token }], &config).unwrap();
            assert_eq!(again[0], BranchPlayoutOutcome::Finished(BranchPlayout { steps: 0, ..p.clone() }), "deterministic repeat refusal");
            assert_eq!((s.actions(*b).unwrap(), s.decision(*b).unwrap(), s.counters()), before);
        }
    }
    assert!(refused > 0, "some playout reaches an unresolvable two-pick grid");
}

/// T9: a live branch with an empty legal table stops (`NO_LEGAL_ACTION`) with no effect.
#[test]
fn t9_no_legal_action_stops_with_no_effect() {
    let root = standalone_root(Character::Ironclad, 0, "jaw_worm", 3);
    let mut s = search(root, Layout::Dense, 1, limits(30));
    let (a, b) = (s.branch(Some(0)).unwrap(), s.branch(Some(1)).unwrap());
    s.clear_legal_table_for_test(a);
    let before = s.counters();
    assert_eq!(s.playout_action(a, &PlayoutRule::default(), 0).err(), Some(RunSessionError::Input("branch has no legal action")));
    let out = s.playout_many(&reqs(&s, &[a, b]), &PlayoutConfig::default()).unwrap();
    let BranchPlayoutOutcome::Finished(p) = &out[0] else { panic!("{out:?}") };
    assert_eq!((p.end_reason, p.refusal, p.steps, p.terminated, p.truncated), ("refused", Some(REFUSAL_NO_LEGAL_ACTION), 0, false, false));
    let BranchPlayoutOutcome::Finished(q) = &out[1] else { panic!("{out:?}") };
    assert!(q.steps > 0, "the sibling plays");
    assert_eq!(s.counters()["total_steps"], before["total_steps"].as_u64().unwrap() + q.steps);
    assert!(s.actions(a).unwrap().is_empty());
}

/// T10: every whole-call rejection names the first offending entry and changes nothing.
#[test]
fn t10_preflight_rejects_without_effect() {
    let fresh = || { let mut s = search(standalone_root(Character::Ironclad, 0, "jaw_worm", 31), Layout::Dense, 1, SearchLimits { max_steps_per_branch: 10, max_total_steps: 80, ..WIDE }); let ids = branch_set(&mut s); (s, ids) };
    let snapshot = |s: &PublicSearch, ids: &[BranchId]| (s.counters(), ids.iter().map(|b| (s.actions(*b).ok(), s.decision(*b).ok())).collect::<Vec<_>>());
    let (mut s, ids) = fresh();
    let before = snapshot(&s, &ids);
    let r = reqs(&s, &ids);
    let config = PlayoutConfig::default();
    assert!(matches!(s.playout_many(&[r[0], r[1], r[0]], &config), Err(RunSessionError::Input(_))), "duplicate");
    assert_eq!(s.playout_many(&[r[0]], &PlayoutConfig { max_steps: Some(0), ..config.clone() }).err(), Some(RunSessionError::Input("playout max_steps must be positive")));
    let stale = BranchPlayoutRequest { branch: ids[1], token: RunDecisionToken([ids[1].0, 99]) };
    assert_eq!(s.playout_many(&[r[0], stale], &config).err(), Some(RunSessionError::StaleToken));
    assert_eq!(s.playout_many(&[BranchPlayoutRequest { branch: BranchId(u64::MAX), token: r[0].token }], &config).err(), Some(RunSessionError::Input("unknown branch")));
    // Budget: after one step, 9 + 8 × 10 = 89 worst-case steps exceed the remaining 79.
    let spent = s.playout_many(&[r[0]], &PlayoutConfig { max_steps: Some(1), ..config.clone() }).unwrap();
    assert!(matches!(&spent[0], BranchPlayoutOutcome::Finished(p) if p.steps == 1));
    let before_budget = snapshot(&s, &ids);
    let r = reqs(&s, &ids);
    assert!(matches!(s.playout_many(&r, &config), Err(RunSessionError::Budget(_))));
    assert_eq!(snapshot(&s, &ids), before_budget);
    assert!(s.playout_many(&r, &PlayoutConfig { max_steps: Some(8), ..config.clone() }).is_ok(), "9 × 8 = 72 ≤ 79 fits");
    let _ = before;
    // Closed, faulted, terminal (truncated) and compact capacity.
    let (mut s, ids) = fresh();
    s.close(ids[2]).unwrap();
    let r = reqs(&s, &ids[..2]);
    let before = snapshot(&s, &ids[..2]);
    assert_eq!(s.playout_many(&[r[0], BranchPlayoutRequest { branch: ids[2], token: r[1].token }], &config).err(), Some(RunSessionError::Closed));
    assert_eq!(snapshot(&s, &ids[..2]), before);
    let t3 = s.decision(ids[3]).unwrap().token;
    diagnostics::supplied_fault(s.branch_run_for_test(ids[3]));
    let _ = s.playout_many(&[BranchPlayoutRequest { branch: ids[3], token: t3 }], &config).unwrap();
    let before = snapshot(&s, &ids[..2]);
    assert_eq!(s.playout_many(&[r[0], BranchPlayoutRequest { branch: ids[3], token: t3 }], &config).err(), Some(RunSessionError::BranchFaulted));
    assert_eq!(snapshot(&s, &ids[..2]), before);
    let out = s.playout_many(&[BranchPlayoutRequest { branch: ids[4], token: s.decision(ids[4]).unwrap().token }], &config).unwrap();
    let BranchPlayoutOutcome::Finished(p) = &out[0] else { panic!("{out:?}") };
    assert_eq!(p.end_reason, "step_limit");
    let before = snapshot(&s, &ids[..2]);
    assert_eq!(s.playout_many(&[r[0], BranchPlayoutRequest { branch: ids[4], token: p.token }], &config).err(), Some(RunSessionError::Terminal));
    assert_eq!(snapshot(&s, &ids[..2]), before);
    let mut c = search(standalone_root(Character::Ironclad, 0, "jaw_worm", 31), Layout::Bounds, 1, limits(10));
    let (x, y) = (c.branch(Some(0)).unwrap(), c.branch(Some(1)).unwrap());
    let width = c.action_capacity() + 1;
    c.widen_legal_table_for_test(y, width);
    let before = c.counters();
    assert!(matches!(c.playout_many(&reqs(&c, &[x, y]), &config), Err(RunSessionError::Capacity { group: "actions", .. })));
    assert!(matches!(c.playout_action(y, &PlayoutRule::default(), 0), Err(RunSessionError::Capacity { group: "actions", .. })));
    assert_eq!(c.counters(), before);
    assert!(c.actions(x).unwrap().is_empty());
}

/// T11: an engine step fault, a mid-playout action-capacity (`encode`) fault, a panic and the 0-step summary read fault
/// freeze only their branch with an exact prefix and step count; siblings finish; nothing is retried.
#[test]
fn t11_faults_are_isolated_and_exact() {
    let root = standalone_root(Character::Silent, 0, "jaw_worm", 22);
    let config = PlayoutConfig::default();
    // Engine step fault after two accepted steps.
    let (mut s, mut clean) = (search(root.clone(), Layout::Dense, 4, limits(40)), search(root.clone(), Layout::Dense, 1, limits(40)));
    let (ids, cids) = (branch_set(&mut s), branch_set(&mut clean));
    let two = PlayoutConfig { max_steps: Some(2), ..config.clone() };
    s.playout_many(&reqs(&s, &ids[..1]), &two).unwrap();
    clean.playout_many(&reqs(&clean, &cids[..1]), &two).unwrap();
    let r = reqs(&s, &ids);
    diagnostics::supplied_fault(s.branch_run_for_test(ids[0]));
    let out = s.playout_many(&r, &config).unwrap();
    let sib = clean.playout_many(&reqs(&clean, &cids[1..]), &config).unwrap();
    assert!(matches!(&out[0], BranchPlayoutOutcome::Fault { error: RunSessionError::DeliveryFault(BRANCH_CORE_FAULT), stage: "step", committed: true, steps: 1, branch_steps: 3 }), "{:?}", out[0]);
    assert_eq!(s.actions(ids[0]).unwrap(), clean.actions(cids[0]).unwrap(), "prefix at the last published boundary");
    for (g, w) in out[1..].iter().zip(&sib) { assert_eq!(normalized(g), normalized(w), "siblings unaffected"); }
    assert_eq!(s.playout_many(&reqs(&s, &ids[..1]).iter().map(|r| BranchPlayoutRequest { token: RunDecisionToken([ids[0].0, 3]), ..*r }).collect::<Vec<_>>(), &config).err(), Some(RunSessionError::BranchFaulted), "never revived");
    // Mid-playout encode fault (action table wider than the capacity after a step): identical to Mask stepping.
    let mut narrow = search(root.clone(), Layout::Dense, 1, limits(40));
    let mut narrow_ref = search(root.clone(), Layout::Dense, 1, limits(40));
    let (nb, rb) = (narrow.branch(Some(2)).unwrap(), narrow_ref.branch(Some(2)).unwrap());
    let (nr, rt) = (reqs(&narrow, &[nb]), narrow_ref.decision(rb).unwrap().token);
    narrow.root_caps_for_test().actions = 4;
    narrow_ref.root_caps_for_test().actions = 4;
    let got = narrow.playout_many(&nr, &config).unwrap();
    let want = reference_from(&mut narrow_ref, rb, &config, rt, 0);
    assert!(matches!(&got[0], BranchPlayoutOutcome::Fault { error: RunSessionError::Capacity { group: "actions", .. }, stage: "encode", committed: true, .. }), "{got:?}");
    assert_same("encode fault", &got[0], &want);
    assert_eq!(narrow.actions(nb).unwrap(), narrow_ref.actions(rb).unwrap(), "the committed index is accepted");
    assert_eq!(narrow.counters(), narrow_ref.counters());
    // Panic at the third step: settled like `step_many`'s panic (serial/steps advance, no index), siblings finish.
    let mut p = search(root.clone(), Layout::Dense, 1, limits(40));
    let pids = branch_set(&mut p);
    crate::public_search::PANIC_PLAYOUT_AT.with(|c| c.set(Some((pids[1].0, 2))));
    let out = p.playout_many(&reqs(&p, &pids), &config);
    crate::public_search::PANIC_PLAYOUT_AT.with(|c| c.set(None));
    let out = out.unwrap();
    assert_eq!(out[1], BranchPlayoutOutcome::Fault { error: RunSessionError::DeliveryFault(BRANCH_CORE_FAULT), stage: "panic", committed: true, steps: 3, branch_steps: 3 });
    assert_eq!(p.actions(pids[1]).unwrap().len(), 2);
    assert!(out.iter().enumerate().all(|(i, o)| i == 1 || matches!(o, BranchPlayoutOutcome::Finished(_))));
    let charged: u64 = out.iter().map(|o| match o { BranchPlayoutOutcome::Finished(x) => x.steps, BranchPlayoutOutcome::Fault { steps, .. } => *steps }).sum();
    assert_eq!(p.counters()["total_steps"].as_u64().unwrap(), charged);
    // 0-step summary read fault (defensive hook): the branch is frozen with its record unchanged.
    let mut z = search(root, Layout::Dense, 1, limits(40));
    let (za, zb) = (z.branch(Some(0)).unwrap(), z.branch(Some(1)).unwrap());
    z.clear_legal_table_for_test(za);
    crate::public_search::FAIL_PLAYOUT_SUMMARY.with(|c| c.set(true));
    let out = z.playout_many(&reqs(&z, &[za, zb]), &config);
    crate::public_search::FAIL_PLAYOUT_SUMMARY.with(|c| c.set(false));
    let out = out.unwrap();
    assert!(matches!(&out[0], BranchPlayoutOutcome::Fault { stage: "encode", committed: false, steps: 0, branch_steps: 0, .. }), "{out:?}");
    assert!(matches!(&out[1], BranchPlayoutOutcome::Finished(_)));
    assert_eq!(z.playout_action(za, &PlayoutRule::default(), 0).err(), Some(RunSessionError::BranchFaulted));
    assert!(z.actions(za).unwrap().is_empty());
}

/// T12: the call cap leaves the branch live (`max_steps`, `truncated` false) and a second call continues exactly; the
/// branch limit truncates (`step_limit`, `truncated` true) and a further call is `Terminal`.
#[test]
fn t12_call_cap_and_branch_limit() {
    let root = standalone_root(Character::Watcher, 0, "gremlin_nob", 6);
    let config = PlayoutConfig::default();
    let (mut a, mut b) = (search(root.clone(), Layout::Dense, 1, limits(12)), search(root, Layout::Dense, 1, limits(12)));
    let (xa, xb) = (a.branch(Some(1)).unwrap(), b.branch(Some(1)).unwrap());
    let whole = a.playout_many(&reqs(&a, &[xa]), &config).unwrap();
    let BranchPlayoutOutcome::Finished(w) = &whole[0] else { panic!() };
    let first = b.playout_many(&reqs(&b, &[xb]), &PlayoutConfig { max_steps: Some(5), ..config.clone() }).unwrap();
    let BranchPlayoutOutcome::Finished(f) = &first[0] else { panic!() };
    assert_eq!((f.end_reason, f.truncated, f.terminated, f.steps, f.branch_steps), ("max_steps", false, false, 5, 5));
    let rest = b.playout_many(&[BranchPlayoutRequest { branch: xb, token: f.token }], &config).unwrap();
    let BranchPlayoutOutcome::Finished(r) = &rest[0] else { panic!() };
    assert_eq!((r.end_reason, r.truncated, r.branch_steps, r.steps), (w.end_reason, w.truncated, w.branch_steps, w.steps - 5));
    assert_eq!(a.actions(xa).unwrap(), b.actions(xb).unwrap());
    assert_eq!(normalized_decision(&a, xa), normalized_decision(&b, xb));
    if w.end_reason == "step_limit" {
        assert!(w.truncated && !w.terminated && w.branch_steps == 12);
        assert_eq!(a.playout_many(&[BranchPlayoutRequest { branch: xa, token: w.token }], &config).err(), Some(RunSessionError::Terminal));
        assert_eq!(a.decision(xa).unwrap().info["end_reason"], Value::from("step_limit"));
    } else {
        panic!("gremlin nob within 12 steps ended {:?}; pick a longer fixture", w.end_reason);
    }
}
