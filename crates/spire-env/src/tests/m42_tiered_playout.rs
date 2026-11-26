//! M42 (spire-9wa): tiered-verb native playout rule (`spire-gym-public-playout-tiered-v1`) and additive resource
//! summary fields (`player_gold`, `potions`, and terminal `player_hp == hp_after_cleanup`).
use super::m34s_transplant::{standard_combat_session, standard_entry};
use super::standard_traversal::{short_creation, simple_action, standard_session, CHARACTERS};
use crate::*;
use serde_json::{json, Value};
use spire_core::content::{Catalog, Character};
use spire_core::run::Run;

const SEED: u64 = 0x42_0001;
const WIDE: SearchLimits = SearchLimits { max_live_branches: 64, max_branches: 2_000, max_steps_per_branch: 5_000, max_total_steps: 10_000_000 };
fn limits(each: u64) -> SearchLimits { SearchLimits { max_steps_per_branch: each, ..WIDE } }

fn standalone_root(character: Character, ascension: u8, encounter: &str, seed: u64) -> PublicRoot {
    let mut s = standard_combat_session(vec![standard_entry("e", character, ascension, encounter, None)]);
    s.reset(Some(seed), None).unwrap();
    s.public_root().unwrap()
}
fn potion_root(character: Character, ascension: u8, seed: u64, potions: Value) -> PublicRoot {
    let mut entry = standard_entry("p", character, ascension, "jaw_worm", None);
    entry["potions"] = potions;
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
fn roots() -> Vec<(String, PublicRoot)> {
    let mut out = vec![];
    for (ci, c) in CHARACTERS.into_iter().enumerate() {
        for (ai, a) in [0u8, 20].into_iter().enumerate() {
            let seed = 0x60 + (ci * 2 + ai) as u8;
            out.push((format!("{c:?} A{a} Standard Run"), run_root(c, a, seed)));
            let encounter = ["jaw_worm", "cultist", "gremlin_nob", "two_louse"][(ci + ai) % 4];
            out.push((format!("{c:?} A{a} {encounter}"), standalone_root(c, a, encounter, u64::from(seed))));
        }
    }
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

fn observed_summary(o: &RunObservation) -> PlayoutSummary {
    let p = o.group("player").unwrap();
    let e = o.group("enemies").unwrap();
    let cols = *e.shape.last().unwrap();
    let enemies = e.values.chunks_exact(cols).filter(|row| row[2] > 0).map(|row| EnemyHp { hp: row[1] as i32, max_hp: row[2] as i32 }).collect();
    let pot = o.group("potions").unwrap();
    let potions = pot.values.iter().zip(&pot.known).filter(|(_, k)| **k == 1).map(|(v, _)| *v as i32).collect();
    PlayoutSummary { player_hp: p.values[0] as i32, player_max_hp: p.values[1] as i32, player_gold: p.values[2] as u32, enemies, potions }
}

fn reference(s: &mut PublicSearch, b: BranchId, config: &PlayoutConfig) -> BranchPlayoutOutcome {
    let start = s.decision(b).unwrap();
    let mut token = start.token;
    let mut branch_steps = start.info["branch_steps"].as_u64().unwrap();
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
                steps += 1; branch_steps += 1;
                let stage = if s.actions(b).unwrap().len() as u64 == branch_steps { "encode" } else { "step" };
                return BranchPlayoutOutcome::Fault { committed: true, error, stage, steps, branch_steps }
            }
        }
    };
    let summary = observed_summary(&s.decision(b).unwrap().observation);
    BranchPlayoutOutcome::Finished(BranchPlayout { token, reward, terminated, truncated, end_reason, refusal, steps, branch_steps, summary })
}

fn check_equivalence(label: &str, root: &PublicRoot, layout: Layout, workers: usize, l: SearchLimits, config: &PlayoutConfig) -> Vec<BranchPlayoutOutcome> {
    let (mut a, mut b) = (search(root.clone(), layout, workers, l), search(root.clone(), layout, 1, l));
    let (ia, ib) = (branch_set(&mut a), branch_set(&mut b));
    let got = a.playout_many(&reqs(&a, &ia), config).unwrap();
    let want = ib.iter().map(|x| reference(&mut b, *x, config)).collect::<Vec<_>>();
    for (i, (g, w)) in got.iter().zip(&want).enumerate() {
        assert_eq!(normalized(g), normalized(w), "{label} entry {i}");
    }
    same_state(&a, &ia, &b, &ib);
    got
}

#[test]
fn m42_tiered_tier_priority_and_validation() {
    assert_eq!(PUBLIC_PLAYOUT_RULE_ID, "spire-gym-public-playout-v1");
    assert_eq!(PUBLIC_PLAYOUT_TIERED_RULE_ID, "spire-gym-public-playout-tiered-v1");
    assert_eq!(DEFAULT_PLAYOUT_NEVER_VERBS, ["use_potion", "discard_potion"]);
    assert_eq!(DEFAULT_PLAYOUT_LAST_RESORT_VERBS, ["end_turn"]);

    let uniform = PlayoutRule::default();
    assert_eq!(uniform.rule_id(), PUBLIC_PLAYOUT_RULE_ID);
    assert_eq!(uniform.never_verbs(), Vec::<&str>::new());
    assert_eq!(uniform.last_resort_verbs(), Vec::<&str>::new());

    let tiered = PlayoutRule::default_tiered();
    assert_eq!(tiered.rule_id(), PUBLIC_PLAYOUT_TIERED_RULE_ID);
    assert_eq!(tiered.never_verbs(), vec!["use_potion", "discard_potion"]);
    assert_eq!(tiered.last_resort_verbs(), vec!["end_turn"]);
    assert_eq!(tiered.excluded_verbs(), Vec::<&str>::new());
    assert_eq!(tiered, PlayoutRule::tiered_verbs(["discard_potion", "use_potion", "use_potion"], ["end_turn", "end_turn"]).unwrap());

    for bad in ["padding", "Use_Potion", "", "end_turn "] {
        assert_eq!(PlayoutRule::tiered_verbs([bad], ["end_turn"]).err(), Some(RunSessionError::Input("unknown playout verb")));
        assert_eq!(PlayoutRule::tiered_verbs(["use_potion"], [bad]).err(), Some(RunSessionError::Input("unknown playout verb")));
    }
    assert_eq!(
        PlayoutRule::tiered_verbs(["use_potion", "end_turn"], ["end_turn"]).err(),
        Some(RunSessionError::Input("playout verb cannot be both never-pick and last-resort")),
    );

    // Tier 0 > Tier 1 > Tier 2 on a potion-holding start.
    let root = potion_root(Character::Ironclad, 0, 7, json!(["block_potion", "fire_potion", Value::Null]));
    let mut s = search(root, Layout::Dense, 1, limits(60));
    let b = s.branch(Some(0)).unwrap();
    // Play cards until only end_turn + potion actions remain (no cards playable).
    let vocab = crate::standard_vocab_manifest()["families"]["verb"].as_array().unwrap().clone();
    let verb_of = |obs: &RunObservation, i: usize| -> String {
        let a = obs.group("actions").unwrap();
        let cols = *a.shape.last().unwrap();
        vocab[a.values[i * cols] as usize].as_str().unwrap().to_owned()
    };
    let mut reached_tier1 = false;
    for _ in 0..20 {
        let d = s.decision(b).unwrap();
        let legal: Vec<usize> = d.observation.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect();
        let verbs: Vec<String> = legal.iter().map(|i| verb_of(&d.observation, *i)).collect();
        let has_card = verbs.iter().any(|v| v == "play_card");
        let has_end = verbs.iter().any(|v| v == "end_turn");
        let has_potion = verbs.iter().any(|v| v == "use_potion" || v == "discard_potion");
        if !has_card && has_end && has_potion {
            reached_tier1 = true;
            // Tiered rule MUST pick end_turn across all seeds (never drinks or discards a potion).
            for seed in 0..32u64 {
                let picked = s.playout_action(b, &tiered, seed).unwrap().0 as usize;
                assert_eq!(verb_of(&d.observation, picked), "end_turn", "seed {seed}: Tier 1 picks end_turn over potions");
            }
            // Tier 2 fallback: when every legal verb is in never_verbs (and last_resort empty), picks from all legal.
            let all_never = PlayoutRule::tiered_verbs(["use_potion", "discard_potion", "end_turn"], Vec::<&str>::new()).unwrap();
            let picked = s.playout_action(b, &all_never, 0).unwrap().0 as usize;
            assert!(legal.contains(&picked));
            break;
        }
        let pick = s.playout_action(b, &tiered, 0).unwrap();
        assert_ne!(verb_of(&d.observation, pick.0 as usize), "use_potion");
        assert_ne!(verb_of(&d.observation, pick.0 as usize), "discard_potion");
        if s.step(b, d.token, pick, BranchView::Mask).unwrap().terminated { break }
    }
    assert!(reached_tier1, "reached a boundary with only end_turn and potions legal");
}

#[test]
fn m42_uniform_rule_is_byte_identical() {
    // tiered_verbs([], excluded) equals uniform_exclude_verbs(excluded) on trajectories and outcomes.
    let root = potion_root(Character::Silent, 0, 13, json!(["attack_potion", "block_potion", Value::Null]));
    let u_cfg = PlayoutConfig { rule: PlayoutRule::default(), max_steps: Some(30), playout_seed: 5 };
    let t_cfg = PlayoutConfig { rule: PlayoutRule::tiered_verbs(Vec::<&str>::new(), DEFAULT_PLAYOUT_EXCLUDED_VERBS).unwrap(), max_steps: Some(30), playout_seed: 5 };
    let (mut su, mut st) = (search(root.clone(), Layout::Dense, 2, limits(40)), search(root, Layout::Dense, 2, limits(40)));
    let (bu, bt) = (branch_set(&mut su), branch_set(&mut st));
    let (ou, ot) = (su.playout_many(&reqs(&su, &bu), &u_cfg).unwrap(), st.playout_many(&reqs(&st, &bt), &t_cfg).unwrap());
    for (g, w) in ou.iter().zip(&ot) { assert_eq!(normalized(g), normalized(w)); }
    same_state(&su, &bu, &st, &bt);
}

#[test]
fn m42_tiered_equals_one_action_at_a_time() {
    let configs = [
        PlayoutConfig { rule: PlayoutRule::default_tiered(), max_steps: None, playout_seed: 0 },
        PlayoutConfig { rule: PlayoutRule::tiered_verbs(["use_potion", "discard_potion"], ["end_turn"]).unwrap(), max_steps: Some(9), playout_seed: 7 },
        PlayoutConfig { rule: PlayoutRule::tiered_verbs(["play_card"], ["end_turn"]).unwrap(), max_steps: Some(12), playout_seed: 19 },
    ];
    let mut starts = roots();
    for c in CHARACTERS {
        starts.push((format!("{c:?} potions"), potion_root(c, 0, 11, json!(["attack_potion", "block_potion", Value::Null]))));
    }
    for (i, (label, root)) in starts.into_iter().enumerate() {
        let layout = [Layout::Dense, Layout::Bounds, Layout::Play][i % 3];
        let workers = [1, 4, 16][i % 3];
        for (j, config) in configs.iter().enumerate() {
            check_equivalence(&format!("{label} {layout:?} w{workers} c{j}"), &root, layout, workers, limits(35), config);
        }
    }
    for (i, c) in CHARACTERS.into_iter().enumerate() {
        let layout = [Layout::Dense, Layout::Bounds][i % 2];
        check_equivalence(&format!("{c:?} dome {layout:?}"), &dome_root(c, 3 + i as u64), layout, 4, limits(35), &configs[0]);
    }
}

#[test]
fn m42_tiered_independent_reference_from_full_obs() {
    let vocab = crate::standard_vocab_manifest()["families"]["verb"].as_array().unwrap().clone();
    let never = ["use_potion", "discard_potion"];
    let last_resort = ["end_turn"];
    let config = PlayoutConfig { rule: PlayoutRule::tiered_verbs(never, last_resort).unwrap(), max_steps: None, playout_seed: 3 };
    let starts = vec![
        ("ironclad potions".to_owned(), potion_root(Character::Ironclad, 0, 9, json!(["fire_potion", "block_potion", Value::Null]))),
        ("silent run".to_owned(), run_root(Character::Silent, 20, 0x62)),
        ("defect dome".to_owned(), dome_root(Character::Defect, 4)),
    ];
    for (label, root) in starts {
        for layout in [Layout::Dense, Layout::Bounds] {
            let (mut a, mut f) = (search(root.clone(), layout, 2, limits(35)), search(root.clone(), layout, 1, limits(35)));
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
                    let legal: Vec<usize> = d.observation.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect();
                    let name = |i: usize| vocab[actions.values[i * cols] as usize].as_str().unwrap();
                    let tier0: Vec<usize> = legal.iter().copied().filter(|i| !never.contains(&name(*i)) && !last_resort.contains(&name(*i))).collect();
                    let tier1: Vec<usize> = legal.iter().copied().filter(|i| last_resort.contains(&name(*i))).collect();
                    let pool = if !tier0.is_empty() { tier0 } else if !tier1.is_empty() { tier1 } else { legal };
                    let ordinal = f.actions(*b).unwrap().len() as u64;
                    let i = pool[spire_core::run::research_playout_pick(&origin, config.playout_seed, ordinal, pool.len())];
                    match f.step(*b, d.token, RunActionIndex(i as i64), BranchView::Full) {
                        Ok(r) if r.terminated || r.truncated => break,
                        Ok(_) => d = f.decision(*b).unwrap(),
                        Err(RunSessionError::UnsupportedBoundary(HIDDEN_ORDER_PICK)) => { refused = Some(REFUSAL_HIDDEN_ORDER_PICK); break }
                        Err(e) => panic!("{label}: {e:?}"),
                    }
                }
                assert_eq!(a.actions(*x).unwrap(), f.actions(*b).unwrap(), "{label} {layout:?}: trajectory");
                let BranchPlayoutOutcome::Finished(p) = g else { panic!("{label}: {g:?}") };
                assert_eq!(p.refusal, refused);
                assert_eq!(p.summary, observed_summary(&f.decision(*b).unwrap().observation), "{label} {layout:?}: summary");
            }
        }
    }
}

#[test]
fn m42_tiered_worker_order_and_chunking_invariance() {
    let root = potion_root(Character::Silent, 20, 5, json!(["fire_potion", "block_potion"]));
    let config = PlayoutConfig { rule: PlayoutRule::default_tiered(), max_steps: None, playout_seed: 0 };
    let l = limits(40);
    let mut base = search(root.clone(), Layout::Dense, 1, l);
    let ib = branch_set(&mut base);
    let want = base.playout_many(&reqs(&base, &ib), &config).unwrap();
    for workers in [2, 4, 8] {
        let mut s = search(root.clone(), Layout::Dense, workers, l);
        let ids = branch_set(&mut s);
        let order: Vec<usize> = (0..ids.len()).rev().collect();
        let (first, second) = order.split_at(4);
        let mut got = vec![None; ids.len()];
        for group in [first, second] {
            let r = s.playout_many(&group.iter().map(|i| BranchPlayoutRequest { branch: ids[*i], token: s.decision(ids[*i]).unwrap().token }).collect::<Vec<_>>(), &config).unwrap();
            for (i, o) in group.iter().zip(r) { got[*i] = Some(o); }
        }
        for (g, w) in got.iter().zip(&want) { assert_eq!(normalized(g.as_ref().unwrap()), normalized(w)); }
        same_state(&s, &ids, &base, &ib);
    }
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
                assert_eq!((g.reward, g.terminated, g.truncated, g.end_reason, g.refusal, g.branch_steps, &g.summary),
                           (w.reward, w.terminated, w.truncated, w.end_reason, w.refusal, w.branch_steps, &w.summary));
            }
            (g, w) => panic!("{g:?} vs {w:?}"),
        }
    }
}

#[test]
fn m42_tiered_crn_and_seeds() {
    let root = potion_root(Character::Defect, 0, 8, json!(["block_potion", Value::Null, Value::Null]));
    let mut s = search(root, Layout::Dense, 1, limits(50));
    let trace = |s: &mut PublicSearch, b: BranchId, seed: u64| {
        let cfg = PlayoutConfig { rule: PlayoutRule::default_tiered(), max_steps: None, playout_seed: seed };
        s.playout_many(&[BranchPlayoutRequest { branch: b, token: s.decision(b).unwrap().token }], &cfg).unwrap();
        s.actions(b).unwrap()
    };
    let (a, b) = (s.branch(Some(4)).unwrap(), s.branch(Some(4)).unwrap());
    assert_eq!(trace(&mut s, a, 0), trace(&mut s, b, 0));
    let (mut differ_seed, mut differ_k) = (false, false);
    for k in 10..16 {
        let (x, y, z) = (s.branch(Some(k)).unwrap(), s.branch(Some(k)).unwrap(), s.branch(Some(k + 100)).unwrap());
        let tx = trace(&mut s, x, 0);
        differ_seed |= tx != trace(&mut s, y, 1);
        differ_k |= tx != trace(&mut s, z, 0);
    }
    assert!(differ_seed && differ_k);
    let p = s.branch(Some(20)).unwrap();
    let rule = PlayoutRule::default_tiered();
    let first = s.playout_action(p, &rule, 0).unwrap();
    let d = s.decision(p).unwrap();
    s.step(p, d.token, first, BranchView::Mask).unwrap();
    let f = s.fork(p).unwrap();
    assert_eq!(trace(&mut s, p, 0), trace(&mut s, f, 0));
}

#[test]
fn m42_tiered_privacy_natural_pairs_and_dome() {
    let strike = Catalog::standard().starter_deck(Character::Silent).unwrap()[0];
    let mut entry = standard_entry("e", Character::Silent, 0, "spheric_guardian", None);
    entry["deck"] = json!(vec![json!({"card": strike.as_str(), "upgrades": 0}); 7]);
    entry["relics"] = json!([]);
    entry["potions"] = json!(["block_potion", Value::Null, Value::Null]);
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
    let cfg = PlayoutConfig { rule: PlayoutRule::default_tiered(), max_steps: None, playout_seed: 0 };
    let (mut x, mut y) = (search(pair.0, Layout::Dense, 4, limits(60)), search(pair.1, Layout::Dense, 1, limits(60)));
    let (ix, iy) = (branch_set(&mut x), branch_set(&mut y));
    let (ox, oy) = (x.playout_many(&reqs(&x, &ix), &cfg).unwrap(), y.playout_many(&reqs(&y, &iy), &cfg).unwrap());
    assert_eq!(ox.iter().map(normalized).collect::<Vec<_>>(), oy.iter().map(normalized).collect::<Vec<_>>());
    same_state(&x, &ix, &y, &iy);
    let text = format!("{ox:?}{cfg:?}");
    for b in &ix { assert!(!text.contains(&super::support::hex(&x.origin_for_test(*b)))); }
}

#[test]
fn m42_summary_potions_gold_and_cleanup_hp_match_observation() {
    // 1. Potion-holding combat: default tiered rule preserves potions; uniform rule drinks them.
    let p_root = potion_root(Character::Ironclad, 0, 17, json!(["fire_potion", "block_potion", Value::Null]));
    let mut st = search(p_root.clone(), Layout::Dense, 2, limits(80));
    let bt = branch_set(&mut st);
    let ot = st.playout_many(&reqs(&st, &bt), &PlayoutConfig { rule: PlayoutRule::default_tiered(), max_steps: None, playout_seed: 0 }).unwrap();
    let mut saw_victory = false;
    for (b, o) in bt.iter().zip(&ot) {
        let BranchPlayoutOutcome::Finished(p) = o else { panic!("{o:?}") };
        let d = st.decision(*b).unwrap();
        assert_eq!(p.summary, observed_summary(&d.observation));
        assert_eq!(p.summary.potions.len(), 3, "slot-indexed potions including empty slot");
        assert!(p.summary.potions[0] > 0 && p.summary.potions[1] > 0 && p.summary.potions[2] == 0, "tiered rule kept potions untouched: {:?}", p.summary.potions);
        if p.terminated {
            let player = d.observation.group("player").unwrap();
            assert_eq!(player.known[11], 1, "hp_after_cleanup known at terminal");
            assert_eq!(i64::from(p.summary.player_hp), player.values[11], "terminal player_hp == hp_after_cleanup");
            assert_eq!(Some(u64::from(p.summary.player_hp as u32)), d.info["hp_after_cleanup"].as_u64());
            if p.end_reason == "victory" { saw_victory = true; }
        }
    }
    assert!(saw_victory, "at least one victory checked for Burning Blood cleanup HP");

    // 2. Smoke Bomb escape: terminal player_hp == hp_after_cleanup (with Burning Blood), and Smoke Bomb removed from potions.
    let mut smoke_entry = standard_entry("smoke", Character::Ironclad, 0, "cultist", None);
    smoke_entry["hp"] = json!(50);
    smoke_entry["max_hp"] = json!(80);
    smoke_entry["gold"] = json!(123);
    smoke_entry["potions"] = json!(["smoke_bomb", "fairy_potion", Value::Null]);
    let mut s_env = standard_combat_session(vec![smoke_entry]);
    s_env.reset(Some(3), None).unwrap();
    let smoke_root = s_env.public_root().unwrap();
    let mut ss = search(smoke_root, Layout::Dense, 1, limits(20));
    let sb = ss.branch(Some(0)).unwrap();
    // Force only use_potion by putting play_card, discard_potion, end_turn in never_verbs.
    let smoke_cfg = PlayoutConfig { rule: PlayoutRule::tiered_verbs(["play_card", "discard_potion", "end_turn"], Vec::<&str>::new()).unwrap(), max_steps: Some(5), playout_seed: 0 };
    let out = ss.playout_many(&reqs(&ss, &[sb]), &smoke_cfg).unwrap();
    let BranchPlayoutOutcome::Finished(p) = &out[0] else { panic!("{out:?}") };
    assert_eq!((p.terminated, p.end_reason), (true, "escaped"));
    let d = ss.decision(sb).unwrap();
    assert_eq!(p.summary, observed_summary(&d.observation));
    assert_eq!(p.summary.player_gold, 123);
    assert_eq!(p.summary.player_hp, 56, "Burning Blood +6 applied on Smoke Bomb escape");
    assert_eq!(i64::from(p.summary.player_hp), d.observation.group("player").unwrap().values[11]);
    assert_eq!(p.summary.potions[0], 0, "Smoke Bomb consumed");
    assert!(p.summary.potions[1] > 0 && p.summary.potions[2] == 0);

    // 3. Hand of Greed gold gain inside combat reflected in player_gold.
    let mut hog_entry = standard_entry("hog", Character::Ironclad, 0, "cultist", None);
    hog_entry["gold"] = json!(77);
    hog_entry["deck"] = json!(vec![json!({"card": "hand_of_greed", "upgrades": 1}); 10]);
    let mut h_env = standard_combat_session(vec![hog_entry]);
    h_env.reset(Some(5), None).unwrap();
    let hog_root = h_env.public_root().unwrap();
    let mut sh = search(hog_root, Layout::Bounds, 1, limits(40));
    let hb = sh.branch(Some(0)).unwrap();
    let hout = sh.playout_many(&reqs(&sh, &[hb]), &PlayoutConfig { rule: PlayoutRule::default_tiered(), max_steps: None, playout_seed: 0 }).unwrap();
    let BranchPlayoutOutcome::Finished(hp) = &hout[0] else { panic!("{hout:?}") };
    assert_eq!((hp.terminated, hp.end_reason), (true, "victory"));
    let hd = sh.decision(hb).unwrap();
    assert_eq!(hp.summary, observed_summary(&hd.observation));
    assert_eq!(hp.summary.player_gold, 102, "Hand of Greed+ fatal +25 gold (77 -> 102)");
    assert!(sh.potion_capacity() >= 3);
}

#[test]
fn m42_post_step_summary_fault_freezes_and_redacts_branch() {
    // Plan review Important finding 1: if playout_summary fails in play_branch's Some(step) arm (steps > 0),
    // the branch must be frozen (Status::Faulted, legal cleared, redacted error) while siblings finish.
    let root = standalone_root(Character::Ironclad, 0, "jaw_worm", 19);
    let mut s = search(root, Layout::Dense, 1, limits(30));
    let (a, b) = (s.branch(Some(0)).unwrap(), s.branch(Some(1)).unwrap());
    let cfg = PlayoutConfig { rule: PlayoutRule::default_tiered(), max_steps: Some(3), playout_seed: 0 };
    crate::public_search::FAIL_PLAYOUT_SUMMARY_BRANCH.with(|c| c.set(Some(a.0)));
    let out = s.playout_many(&reqs(&s, &[a, b]), &cfg);
    crate::public_search::FAIL_PLAYOUT_SUMMARY_BRANCH.with(|c| c.set(None));
    let out = out.unwrap();
    assert!(
        matches!(&out[0], BranchPlayoutOutcome::Fault { error: RunSessionError::DeliveryFault(BRANCH_CORE_FAULT), stage: "encode", committed: true, steps, branch_steps } if *steps > 0 && *steps == *branch_steps),
        "{:?}", out[0],
    );
    assert!(matches!(&out[1], BranchPlayoutOutcome::Finished(p) if p.steps > 0), "sibling finishes cleanly");
    assert_eq!(s.playout_action(a, &cfg.rule, 0).err(), Some(RunSessionError::BranchFaulted), "faulted branch is frozen");
    assert!(s.legal_for_test(a).is_empty(), "faulted branch legal table is cleared");
}
