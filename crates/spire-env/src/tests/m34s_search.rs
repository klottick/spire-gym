//! M34s-b b3: the search arena (design §4.7) and the env-level privacy oracle (§4.10 P1, P3, P7). Unit tests (not
//! `tests/`) because fault injection and the true-world precondition checks need crate-private hooks.
use super::m34s_transplant::{standard_combat_session, standard_entry};
use super::standard_traversal::{short_creation, simple_action, standard_session};
use crate::*;
use serde_json::json;
use spire_core::{combat::rng::Partition, content::Character, run::Run};

fn limits(live: usize, branches: u64, per_branch: u64, total: u64) -> SearchLimits {
    SearchLimits { max_live_branches: live, max_branches: branches, max_steps_per_branch: per_branch, max_total_steps: total }
}
const WIDE: SearchLimits = SearchLimits { max_live_branches: 64, max_branches: 1_000, max_steps_per_branch: 5_000, max_total_steps: 1_000_000 };
fn search(root: PublicRoot, seed: u64, limits: SearchLimits) -> PublicSearch { PublicSearch::new(root, SearchConfig { research_seed: seed, limits }).unwrap() }
fn legal(o: &RunObservation) -> Vec<usize> { o.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect() }
fn verb(o: &RunObservation, i: usize) -> Option<String> {
    let g = o.group("actions").unwrap();
    let v = g.values[i * g.shape.last().unwrap()];
    standard_vocab_manifest()["families"]["verb"].as_array().unwrap().get(usize::try_from(v).ok()?).and_then(|x| x.as_str()).map(str::to_owned)
}
/// Play a card when possible, else end the turn (selections take their first row).
fn aggressive(o: &RunObservation) -> usize {
    let l = legal(o);
    l.iter().copied().find(|i| verb(o, *i).as_deref() == Some("play_card")).or_else(|| l.iter().copied().find(|i| verb(o, *i).as_deref() == Some("end_turn"))).unwrap_or(l[0])
}
/// A research-seeded random legal policy (P3).
struct Xorshift(u64);
impl Xorshift { fn pick(&mut self, mask: &[i8]) -> usize { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17;
    let l = mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect::<Vec<_>>(); l[(self.0 % l.len() as u64) as usize] } }
fn standalone(character: Character, ascension: u8, encounter: &str, seed: u64) -> (CombatTrainingSession, CombatPolicyBoundary) {
    let mut s = standard_combat_session(vec![standard_entry("e", character, ascension, encounter, None)]);
    let b = s.reset(Some(seed), None).unwrap();
    (s, b)
}
/// A Standard Run session at its first combat decision.
fn run_in_combat(character: Character, ascension: u8, seed: u8) -> (RunSession, RunBoundary) {
    let (mut s, _) = standard_session(character, ascension, seed, short_creation());
    let mut b = s.decision().unwrap();
    while s.run_for_test().unwrap().decision().unwrap().combat.is_none() { b = s.step(b.token, RunActionIndex(simple_action(&b.observation) as i64)).unwrap().boundary; }
    (s, b)
}
/// Rolls `b` out to its end with a seeded random legal policy; returns the (mask, reward, terminated, truncated) trace.
fn rollout(search: &mut PublicSearch, b: BranchId, seed: u64, view: BranchView) -> Vec<(Vec<i8>, i32, bool, bool)> {
    let mut rng = Xorshift(seed | 1);
    let mut d = search.decision(b).unwrap();
    let (mut mask, mut token, mut out) = (d.observation.action_mask.clone(), d.token, vec![]);
    loop {
        let r = search.step(b, token, RunActionIndex(rng.pick(&mask) as i64), view).unwrap();
        out.push((r.action_mask.clone(), r.reward, r.terminated, r.truncated));
        if r.terminated || r.truncated { return out }
        (mask, token) = (r.action_mask, r.token);
        if view == BranchView::Full { d.observation = r.observation.unwrap(); }
    }
}

#[test]
fn public_root_requires_a_supported_combat_decision() {
    let unsupported = |r: Result<PublicRoot, RunSessionError>| matches!(r, Err(RunSessionError::UnsupportedBoundary(_)));
    // Noncombat Standard decision; captured profile; Gym66 session; Runic Dome.
    let (s, _) = standard_session(Character::Ironclad, 0, 0xB0, short_creation());
    assert!(unsupported(s.public_root()), "noncombat");
    let m10 = RunSession::new_m10(RunSessionConfig { character: Character::Ironclad, ascension: 0, partition: Partition::Test,
        boss_history: RunBossHistory::AwakenedUnseen, capacities: RunCapacities::m10_default(), record_replay: false }).unwrap();
    assert!(unsupported(m10.public_root()), "captured profile");
    let gym66 = CombatTrainingSession::new(CombatTrainingCatalog::from_json(br#"{"entries":[{"id":"a","encounter":"cultist","hp":80,"max_hp":80,"deck":[{"card":"strike","upgrades":0}],"relics":[],"potions":[null,null,null]}]}"#).unwrap(), Partition::Test).unwrap();
    assert!(unsupported(gym66.public_root()), "Gym66");
    let mut dome = standard_entry("d", Character::Ironclad, 0, "jaw_worm", None);
    dome["relics"] = json!(["burning_blood", "runic_dome"]);
    let mut s = standard_combat_session(vec![dome]);
    s.reset(Some(1), None).unwrap();
    assert_eq!(s.public_root().err(), Some(RunSessionError::UnsupportedBoundary("hidden intents are not modelled")));
    // Not reset; terminal; faulted (frozen); closed.
    let fresh = standard_combat_session(vec![standard_entry("e", Character::Ironclad, 0, "jaw_worm", None)]);
    assert_eq!(fresh.public_root().err(), Some(RunSessionError::NotReset));
    let (mut s, mut b) = standalone(Character::Ironclad, 0, "cultist", 2);
    loop { let r = s.step(b.token, RunActionIndex(aggressive(&b.observation) as i64)).unwrap(); if r.terminated { break } b = r.boundary; }
    assert_eq!(s.public_root().err(), Some(RunSessionError::Terminal));
    let (mut s, _) = run_in_combat(Character::Silent, 0, 0xB1);
    assert!(s.public_root().is_ok());
    let token = s.decision().unwrap().token;
    s.apply_supplied(|run| { spire_core::run::diagnostics::supplied_fault(run); Ok(()) }).ok();
    let _ = s.step(token, RunActionIndex(0));
    assert!(s.public_root().is_err(), "a faulted session has no root");
    let (mut s, _) = standalone(Character::Ironclad, 0, "jaw_worm", 3);
    s.close().unwrap();
    assert_eq!(s.public_root().err(), Some(RunSessionError::Closed));
}

#[test]
fn branch_first_boundary_equals_root_view_and_candidates() {
    // Run roots: the Standard combat sidecar view is the branch's first view, with the same action rows.
    for (i, character) in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher].into_iter().enumerate() {
        let (mut s, _) = run_in_combat(character, 20, 0xB8 + i as u8);
        let root_view = s.combat_view().unwrap().observation;
        let mut search = search(s.public_root().unwrap(), 9, WIDE);
        for k in 0..4 {
            let b = search.branch(Some(k)).unwrap();
            assert_eq!(search.decision(b).unwrap().observation, root_view, "{character:?} sample {k}");
        }
    }
    // Standalone roots.
    let (s, b) = standalone(Character::Watcher, 0, "two_louse", 4);
    let mut search = search(s.public_root().unwrap(), 9, WIDE);
    let branch = search.branch(None).unwrap();
    assert_eq!(search.decision(branch).unwrap().observation, b.observation);
}

fn search_between_live_steps(root: PublicRoot, seed: u64) {
    let mut arena = search(root, seed, WIDE);
    let branch = arena.branch(None).unwrap();
    let boundary = arena.decision(branch).unwrap();
    let fork = arena.fork(branch).unwrap();
    for (id, token, mask) in [(branch, boundary.token, boundary.observation.action_mask.clone()),
        (fork, RunDecisionToken([fork.0, 0]), boundary.observation.action_mask)] {
        let mut succeeded = false;
        for action in legal(&RunObservation { groups: vec![], action_mask: mask.clone() }) {
            match arena.step(id, token, RunActionIndex(action as i64), BranchView::Mask) {
                Ok(step) => { assert!(step.observation.is_none()); succeeded = true; break; }
                Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK)) => continue,
                Err(error) => panic!("search branch step: {error:?}"),
            }
        }
        assert!(succeeded, "all search actions were refused for hidden order");
    }
}

#[test]
fn live_session_is_unchanged_by_search() {
    let run = |with_search: bool| {
        let (mut session, mut boundary) = run_in_combat(Character::Ironclad, 20, 11);
        let mut trace = vec![];
        for index in 0..10_000 {
            let in_combat = session.run_for_test().unwrap().decision().unwrap().combat.is_some();
            if !in_combat { return (trace, session.record().unwrap().unwrap()); }
            if with_search { search_between_live_steps(session.public_root().unwrap(), 3 + index as u64); }
            let info = session.combat_view().unwrap().info;
            let action = simple_action(&boundary.observation);
            let result = session.step(boundary.token, RunActionIndex(action as i64)).unwrap();
            trace.push((result.boundary.observation.clone(), result.reward, result.terminated, result.truncated, info));
            if result.terminated { return (trace, session.record().unwrap().unwrap()); }
            boundary = result.boundary;
        }
        panic!("Standard Run combat did not settle within 10,000 actions")
    };
    let (plain, searched) = (run(false), run(true));
    assert_eq!(plain, searched, "searches leave the Standard Run combat and its record unchanged");

    let standalone_run = |with_search: bool| {
        let (mut session, mut boundary) = standalone(Character::Ironclad, 20, "gremlin_nob", 11);
        let mut trace = vec![];
        for index in 0..10_000 {
            if with_search { search_between_live_steps(session.public_root().unwrap(), 7 + index as u64); }
            let action = aggressive(&boundary.observation);
            let result = session.step(boundary.token, RunActionIndex(action as i64)).unwrap();
            trace.push((result.boundary.observation.clone(), result.reward, result.terminated, result.truncated,
                result.boundary.info.clone()));
            if result.terminated || result.truncated { return (trace, session.record().unwrap()); }
            boundary = result.boundary;
        }
        panic!("standalone combat did not end within 10,000 actions")
    };
    let (plain, searched) = (standalone_run(false), standalone_run(true));
    // `standard-combat-v1` keeps no episode record (only the M21 catalog does): both twins report none.
    assert!(plain.1.is_none(), "standard-combat-v1 sessions keep no episode record");
    assert_eq!(plain, searched, "searches leave standalone views, rewards, flags, info and the (absent) record unchanged");
}

#[test]
fn siblings_and_forks_are_isolated() {
    let (s, _) = standalone(Character::Defect, 0, "jaw_worm", 5);
    let mut search = search(s.public_root().unwrap(), 1, WIDE);
    let (a, b) = (search.branch(Some(0)).unwrap(), search.branch(Some(1)).unwrap());
    let before_b = search.decision(b).unwrap();
    let d = search.decision(a).unwrap();
    search.step(a, d.token, RunActionIndex(aggressive(&d.observation) as i64), BranchView::Full).unwrap();
    assert_eq!(search.decision(b).unwrap(), before_b, "a sibling is untouched");
    let f = search.fork(a).unwrap();
    let (fa, ff) = (search.decision(a).unwrap(), search.decision(f).unwrap());
    assert_eq!(fa.observation, ff.observation, "a fork starts in its parent's world");
    assert_ne!(fa.token, ff.token);
    rollout(&mut search, f, 9, BranchView::Mask);
    assert_eq!(search.decision(a).unwrap(), fa, "the parent is untouched by its fork");
}

#[test]
fn stale_foreign_and_closed_tokens_are_rejected_without_effect() {
    let (s, live) = standalone(Character::Silent, 0, "cultist", 6);
    let mut search = search(s.public_root().unwrap(), 1, WIDE);
    let (a, b) = (search.branch(None).unwrap(), search.branch(None).unwrap());
    let (da, db) = (search.decision(a).unwrap(), search.decision(b).unwrap());
    let counters = search.counters();
    for token in [db.token, live.token, RunDecisionToken([da.token.0[0], da.token.0[1] + 1])] {
        assert_eq!(search.step(a, token, RunActionIndex(0), BranchView::Full).err(), Some(RunSessionError::StaleToken));
    }
    assert_eq!((search.decision(a).unwrap(), search.counters()), (da.clone(), counters));
    search.step(a, da.token, RunActionIndex(aggressive(&da.observation) as i64), BranchView::Full).unwrap();
    assert_eq!(search.step(a, da.token, RunActionIndex(0), BranchView::Full).err(), Some(RunSessionError::StaleToken), "a spent token");
    search.close(b).unwrap();
    for e in [search.step(b, db.token, RunActionIndex(0), BranchView::Full).err(), search.decision(b).err(), search.fork(b).err(), search.actions(b).err(), search.close(b).err()] {
        assert_eq!(e, Some(RunSessionError::Closed));
    }
    assert!(search.public_root(b).is_err());
    assert!(matches!(search.decision(BranchId(u64::MAX)), Err(RunSessionError::Input(_))), "a foreign id");
}

#[test]
fn budgets_are_enforced_without_effect() {
    let (s, _) = standalone(Character::Ironclad, 0, "jaw_worm", 7);
    let mut search = search(s.public_root().unwrap(), 1, limits(2, 3, 100, 3));
    let a = search.branch(None).unwrap();
    let _b = search.branch(None).unwrap();
    let c = search.counters();
    assert!(matches!(search.branch(None), Err(RunSessionError::Budget(_))), "live limit");
    assert!(matches!(search.fork(a), Err(RunSessionError::Budget(_))));
    assert_eq!(search.counters(), c);
    search.close(a).unwrap();
    let a2 = search.branch(None).unwrap();
    assert!(matches!(search.branch(None), Err(RunSessionError::Budget(_))), "total branch limit");
    let mut d = search.decision(a2).unwrap();
    for _ in 0..3 { let r = search.step(a2, d.token, RunActionIndex(aggressive(&d.observation) as i64), BranchView::Full).unwrap(); d = search.decision(a2).unwrap(); assert!(!r.terminated); }
    let (c, before) = (search.counters(), search.decision(a2).unwrap());
    assert!(matches!(search.step(a2, d.token, RunActionIndex(0), BranchView::Full), Err(RunSessionError::Budget(_))), "total step limit");
    assert_eq!((search.counters(), search.decision(a2).unwrap()), (c, before));
}

#[test]
fn step_limit_truncates_with_its_own_end_reason() {
    let (s, _) = standalone(Character::Watcher, 0, "jaw_worm", 8);
    let mut search = search(s.public_root().unwrap(), 1, limits(4, 10, 3, 100));
    let a = search.branch(None).unwrap();
    let trace = rollout(&mut search, a, 1, BranchView::Full);
    let last = trace.last().unwrap();
    assert_eq!((trace.len(), last.1, last.2, last.3), (3, 0, false, true));
    assert_eq!(search.decision(a).unwrap().info["end_reason"], "step_limit");
    let d = search.decision(a).unwrap();
    assert_eq!(search.step(a, d.token, RunActionIndex(0), BranchView::Full).err(), Some(RunSessionError::Terminal));
    assert_eq!(search.fork(a).err(), Some(RunSessionError::Terminal), "fork of a truncated branch");
}

#[test]
fn fork_inherits_step_count() {
    let (s, _) = standalone(Character::Watcher, 0, "jaw_worm", 8);
    let mut search = search(s.public_root().unwrap(), 1, limits(4, 10, 4, 100));
    let a = search.branch(None).unwrap();
    let d = search.decision(a).unwrap();
    search.step(a, d.token, RunActionIndex(aggressive(&d.observation) as i64), BranchView::Mask).unwrap();
    let f = search.fork(a).unwrap();
    assert_eq!(rollout(&mut search, f, 3, BranchView::Mask).len(), 3, "the fork has 4 - 1 steps left");
    assert_eq!(search.actions(f).unwrap().len(), 4);
}

#[test]
fn faulted_branches_are_never_revived() {
    let (s, _) = standalone(Character::Silent, 0, "jaw_worm", 9);
    let mut search = search(s.public_root().unwrap(), 1, WIDE);
    let (a, sibling) = (search.branch(None).unwrap(), search.branch(None).unwrap());
    let d = search.decision(a).unwrap();
    let first = aggressive(&d.observation) as i64;
    let r = search.step(a, d.token, RunActionIndex(first), BranchView::Full).unwrap();
    let last = search.decision(a).unwrap();
    spire_core::run::diagnostics::supplied_fault(search.branch_run_for_test(a));
    // Final review v3: the core fault (which carries the attempted action and its card instance) is redacted to a class.
    let fault = search.step(a, r.token, RunActionIndex(0), BranchView::Full).unwrap_err();
    assert_eq!(fault, RunSessionError::DeliveryFault(BRANCH_CORE_FAULT));
    assert!(!format!("{fault:?}").contains("instance"), "no attempted action in the error");
    // fork_of_faulted_branch_is_rejected, public_root_of_faulted_branch_is_rejected, no retry.
    assert_eq!(search.fork(a).err(), Some(RunSessionError::BranchFaulted));
    assert_eq!(search.public_root(a).err(), Some(RunSessionError::BranchFaulted));
    assert_eq!(search.step(a, r.token, RunActionIndex(0), BranchView::Full).err(), Some(RunSessionError::BranchFaulted));
    // branch_fault_freezes_only_that_branch_and_keeps_its_prefix: the accepted prefix and the last boundary stay readable.
    assert_eq!(search.actions(a).unwrap(), vec![first]);
    assert_eq!(search.decision(a).unwrap().observation, last.observation);
    assert!(rollout(&mut search, sibling, 4, BranchView::Mask).last().unwrap().2 || true, "the sibling still steps");
    // A terminal branch cannot be forked.
    let t = search.branch(None).unwrap();
    let trace = rollout(&mut search, t, 5, BranchView::Mask);
    assert!(trace.last().unwrap().2);
    assert_eq!(search.fork(t).err(), Some(RunSessionError::Terminal));
}

#[test]
fn a_publication_error_after_the_engine_commits_faults_the_branch() {
    // Final review M3: the step committed, so the branch must not stay Live with the previous boundary's action table.
    let (s, _) = standalone(Character::Ironclad, 0, "jaw_worm", 9);
    let mut search = search(s.public_root().unwrap(), 1, WIDE);
    let a = search.branch(None).unwrap();
    let d = search.decision(a).unwrap();
    let first = aggressive(&d.observation) as i64;
    search.root_caps_for_test().actions = 1;
    assert!(matches!(search.step(a, d.token, RunActionIndex(first), BranchView::Mask), Err(RunSessionError::Capacity { .. })));
    assert_eq!(search.actions(a).unwrap(), vec![first], "the committed action is kept");
    let token = RunDecisionToken([d.token.0[0], d.token.0[1] + 1]);
    assert_eq!(search.step(a, token, RunActionIndex(0), BranchView::Mask).err(), Some(RunSessionError::BranchFaulted));
    assert_eq!(search.fork(a).err(), Some(RunSessionError::BranchFaulted));
}

#[test]
fn branch_actions_are_the_accepted_prefix_and_trajectories_are_deterministic() {
    let (s, _) = standalone(Character::Defect, 20, "two_louse", 10);
    let root = s.public_root().unwrap();
    let mut x = search(root.clone(), 77, WIDE);
    let mut y = search(root, 77, WIDE);
    let (a, b) = (x.branch(Some(4)).unwrap(), y.branch(Some(4)).unwrap());
    let (ta, tb) = (rollout(&mut x, a, 5, BranchView::Full), rollout(&mut y, b, 5, BranchView::Full));
    assert_eq!(ta, tb);
    assert_eq!(x.actions(a).unwrap(), y.actions(b).unwrap());
    assert_eq!(x.actions(a).unwrap().len(), ta.len());
}

#[test]
fn duplicate_and_auto_sample_indices() {
    let (s, _) = standalone(Character::Ironclad, 0, "jaw_worm", 12);
    let mut search = search(s.public_root().unwrap(), 5, WIDE);
    // duplicate_sample_index_gives_identical_world
    let (a, b) = (search.branch(Some(3)).unwrap(), search.branch(Some(3)).unwrap());
    assert_eq!(rollout(&mut search, a, 2, BranchView::Full), rollout(&mut search, b, 2, BranchView::Full));
    // auto_sample_index_never_collides: auto picks skip every used index (0, 1, 2 here; 3 is taken).
    let worlds = (0..4).map(|_| { let x = search.branch(None).unwrap(); search.branch_run_for_test(x).clone() }).collect::<Vec<Run>>();
    let three = search.branch(Some(3)).unwrap();
    let three = search.branch_run_for_test(three).clone();
    assert!(worlds.iter().all(|w| *w != three), "no auto index reuses 3");
    for i in 0..4 { for j in i + 1..4 { assert!(worlds[i] != worlds[j]); } }
}

#[test]
fn nested_search_budgets_are_separate() {
    let (s, _) = standalone(Character::Ironclad, 0, "jaw_worm", 13);
    let mut outer = search(s.public_root().unwrap(), 5, limits(2, 2, 100, 100));
    let a = outer.branch(None).unwrap();
    let d = outer.decision(a).unwrap();
    outer.step(a, d.token, RunActionIndex(aggressive(&d.observation) as i64), BranchView::Mask).unwrap();
    let before = outer.counters();
    let mut inner = search(outer.public_root(a).unwrap(), 6, limits(2, 2, 100, 100));
    let i = inner.branch(None).unwrap();
    rollout(&mut inner, i, 3, BranchView::Mask);
    assert_eq!(outer.counters(), before, "the parent's counters exclude the nested search");
    assert_eq!(inner.counters()["branches_created"], 1);
}

#[test]
fn spec_publishes_max_retained_bytes_and_validates_limits() {
    let (s, _) = standalone(Character::Ironclad, 0, "jaw_worm", 14);
    let root = s.public_root().unwrap();
    let search = search(root.clone(), 5, limits(8, 10, 10, 10));
    let spec = search.spec();
    // Final review I2: the bound includes the enforced knowledge cap, the action history (doubled for vector growth), the
    // legal-action table and the branch-map entry; the arena total adds its root, collection roots and per-created-branch
    // closed/used-sample entries (final review v2).
    let parts = &spec["branch_retained_bytes"];
    assert_eq!((parts["run_estimate"].as_u64(), parts["knowledge_cap"].as_u64(), parts["action_history"].as_u64()),
        (Some(BRANCH_RUN_BYTES_ESTIMATE), Some(spire_core::combat::knowledge::KNOWLEDGE_BYTES_CAP as u64), Some(2 * 10 * 8)));
    let per_branch = ["run_estimate", "knowledge_cap", "action_history", "legal_actions", "branch_entry"].iter().map(|k| parts[*k].as_u64().unwrap()).sum::<u64>();
    assert!(parts["legal_actions"].as_u64().unwrap() > 0 && parts["branch_entry"].as_u64().unwrap() > 0);
    assert_eq!(spec["branch_retained_bytes_bound"], json!(per_branch));
    let total = &spec["retained_bytes"];
    assert_eq!(total["per_live_branch"], json!(per_branch));
    let (fixed, per_created) = (total["fixed"].as_u64().unwrap(), total["per_created_branch"].as_u64().unwrap());
    assert!(fixed > BRANCH_RUN_BYTES_ESTIMATE && per_created > 0);
    assert_eq!(spec["max_retained_bytes"], json!(fixed + 8 * per_branch + 10 * per_created));
    assert_eq!((spec["search_id"].as_str(), spec["belief_id"].as_str(), spec["knowledge_id"].as_str()), (Some(PUBLIC_SEARCH_ID), Some(PUBLIC_BELIEF_ID), Some(PUBLIC_KNOWLEDGE_ID)));
    assert_eq!(spec["root_digest"], json!(crate::combat_policy::hex(&root.digest())));
    // limits_are_validated_against_hard_caps
    let bad = [limits(0, 1, 1, 1), limits(SEARCH_HARD_CAPS.max_live_branches + 1, 1, 1, 1), limits(1, SEARCH_HARD_CAPS.max_branches + 1, 1, 1),
        limits(1, 1, SEARCH_HARD_CAPS.max_steps_per_branch + 1, 1), limits(1, 1, 1, SEARCH_HARD_CAPS.max_total_steps + 1)];
    for l in bad { assert!(matches!(PublicSearch::new(root.clone(), SearchConfig { research_seed: 0, limits: l }), Err(RunSessionError::Input(_))), "{l:?}"); }
    assert!(PublicSearch::new(root, SearchConfig { research_seed: 0, limits: SEARCH_HARD_CAPS }).is_ok());
}

#[test]
fn branch_terminals_are_distinct() {
    let mut win = standard_entry("win", Character::Ironclad, 0, "cultist", None);
    win["deck"] = json!(vec![json!({"card": "bludgeon", "upgrades": 1}); 10]);
    let mut lose = standard_entry("lose", Character::Silent, 0, "jaw_worm", None); lose["hp"] = json!(1);
    let mut escape = standard_entry("escape", Character::Defect, 0, "cultist", None); escape["potions"] = json!(["smoke_bomb", null, null]);
    let mut s = standard_combat_session(vec![win, lose, escape]);
    let end = |s: &mut CombatTrainingSession, id: &str, policy: fn(&RunObservation) -> usize| {
        s.reset(Some(5), Some(id)).unwrap();
        let mut search = search(s.public_root().unwrap(), 2, WIDE);
        let b = search.branch(None).unwrap();
        let mut d = search.decision(b).unwrap();
        loop {
            let r = search.step(b, d.token, RunActionIndex(policy(&d.observation) as i64), BranchView::Full).unwrap();
            if r.terminated { return (r.reward, r.info["terminal_reason"].clone(), r.truncated) }
            d = search.decision(b).unwrap();
        }
    };
    let passive = |o: &RunObservation| legal(o).into_iter().find(|i| verb(o, *i).as_deref() == Some("end_turn")).unwrap_or(legal(o)[0]);
    let smoke = |o: &RunObservation| legal(o).into_iter().find(|i| verb(o, *i).as_deref() == Some("use_potion")).unwrap_or(legal(o)[0]);
    assert_eq!(end(&mut s, "win", aggressive), (1, json!("victory"), false));
    assert_eq!(end(&mut s, "lose", passive), (-1, json!("death"), false));
    assert_eq!(end(&mut s, "escape", smoke), (0, json!("escaped"), false));
}

#[test]
fn mask_only_steps_match_full_view_masks() {
    let (s, _) = standalone(Character::Silent, 20, "gremlin_nob", 15);
    let root = s.public_root().unwrap();
    let (mut x, mut y) = (search(root.clone(), 4, WIDE), search(root, 4, WIDE));
    let (a, b) = (x.branch(Some(0)).unwrap(), y.branch(Some(0)).unwrap());
    let (full, mask) = (rollout(&mut x, a, 8, BranchView::Full), rollout(&mut y, b, 8, BranchView::Mask));
    assert_eq!(full, mask);
}

#[test]
fn public_root_from_a_branch() {
    let (s, _) = standalone(Character::Watcher, 0, "jaw_worm", 16);
    let root = s.public_root().unwrap();
    let mut search = search(root.clone(), 4, WIDE);
    let a = search.branch(None).unwrap();
    assert_eq!(search.public_root(a).unwrap(), root, "a fresh branch's root is its root");
    let d = search.decision(a).unwrap();
    search.step(a, d.token, RunActionIndex(aggressive(&d.observation) as i64), BranchView::Mask).unwrap();
    let next = search.public_root(a).unwrap();
    assert_ne!(next.digest(), root.digest());
    let mut nested = super::m34s_search::search(next, 4, WIDE);
    let n = nested.branch(None).unwrap();
    assert_eq!(nested.decision(n).unwrap().observation, search.decision(a).unwrap().observation);
}

/// P7: no output channel carries a seed, key, cursor, pre-roll value, research seed or sample index.
#[test]
fn search_outputs_carry_no_hidden_values() {
    const RESEARCH_SEED: u64 = 0x5EED_1234_ABCD_9876;
    const SAMPLE: u64 = 987_654_321_123;
    const LIVE_SEED: u64 = 0x0DDB_A11C_0FFE_E123;
    let (s, _) = standalone(Character::Ironclad, 20, "two_louse", LIVE_SEED);
    let root = s.public_root().unwrap();
    let mut search = search(root.clone(), RESEARCH_SEED, WIDE);
    let a = search.branch(Some(SAMPLE)).unwrap();
    let key = spire_core::run::research_sample_key(&spire_core::run::research_root(RESEARCH_SEED), &root.digest(), SAMPLE);
    let forbidden = [RESEARCH_SEED.to_string(), format!("{RESEARCH_SEED:x}"), SAMPLE.to_string(), format!("{SAMPLE:x}"), LIVE_SEED.to_string(),
        format!("{LIVE_SEED:x}"), crate::combat_policy::hex(&key)];
    let mut texts = vec![format!("{search:?}"), format!("{root:?}"), search.spec().to_string(), search.counters().to_string()];
    let d = search.decision(a).unwrap();
    texts.push(d.info.to_string());
    texts.push(format!("{:?}", search.step(a, RunDecisionToken([0, 0]), RunActionIndex(0), BranchView::Full).err()));
    texts.push(format!("{:?}", PublicSearch::new(root.clone(), SearchConfig { research_seed: RESEARCH_SEED, limits: limits(0, 1, 1, 1) }).err()));
    let r = search.step(a, d.token, RunActionIndex(aggressive(&d.observation) as i64), BranchView::Full).unwrap();
    texts.push(r.info.to_string());
    for t in &texts { for f in &forbidden { assert!(!t.contains(f.as_str()), "{f} appears in {t}"); } }
    // Views: every branch shows the root's first view, whatever its sampled pre-rolls and order (no hidden value leaks).
    for k in 0..6 { let b = search.branch(Some(k)).unwrap(); assert_eq!(search.decision(b).unwrap().observation, d.observation); }
    let small = [RESEARCH_SEED, SAMPLE, LIVE_SEED].map(|x| x as i64);
    assert!(r.observation.unwrap().groups.iter().all(|g| g.values.iter().all(|v| !small.contains(v))));
}

/// P1/P3 (design §4.10; b0 scan plan): standalone starts from different seeds whose public histories are identical
/// (byte-equal Standard combat views and info, token excluded, at every step of the shared prefix) give equal public
/// roots, and the same `(research_seed, k)` with a seeded random legal policy gives identical branches to combat end.
/// Each pair first proves the true worlds differ: the Runs differ, and stepping exact copies with the same actions
/// diverges publicly.
#[test]
fn natural_pairs_have_equal_public_roots_and_identical_branches() {
    use spire_core::content::Catalog;
    let passive = |o: &RunObservation| legal(o).into_iter().find(|i| verb(o, *i).as_deref() == Some("end_turn")).unwrap_or(legal(o)[0]);
    let mut summary = vec![];
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let strike = Catalog::standard().starter_deck(character).unwrap()[0];
        let defend = *Catalog::standard().starter_deck(character).unwrap().iter().find(|c| **c != strike).unwrap();
        let card = |c: spire_core::content::CardId| json!({"card": c.as_str(), "upgrades": 0});
        // (entry, prefix steps): 7 Strikes vs Spheric Guardian at the opening; 1 Strike vs Jaw Worm after two turns;
        // Strike + Defend vs Cultist after one turn.
        let entries = [("guardian", "spheric_guardian", vec![card(strike); 7], 0), ("worm", "jaw_worm", vec![card(strike)], 2), ("cultist", "cultist", vec![card(strike), card(defend)], 1)];
        let mut pairs = 0;
        let mut per_entry = vec![];
        for (id, encounter, deck, prefix) in entries {
            let before = pairs;
            let mut entry = standard_entry(id, character, 0, encounter, None);
            entry["deck"] = json!(deck);
            entry["relics"] = json!([]);
            let mut groups: std::collections::BTreeMap<String, Vec<u64>> = Default::default();
            for seed in 0..400u64 {
                let mut s = standard_combat_session(vec![entry.clone()]);
                let mut b = s.reset(Some(seed), None).unwrap();
                let mut history = format!("{:?}{}", b.observation, b.info);
                for _ in 0..prefix { b = s.step(b.token, RunActionIndex(passive(&b.observation) as i64)).unwrap().boundary; history.push_str(&format!("{:?}{}", b.observation, b.info)); }
                groups.entry(history).or_default().push(seed);
            }
            for seeds in groups.values().filter(|g| g.len() > 1) {
                let at = |seed: u64| { let mut s = standard_combat_session(vec![entry.clone()]); let mut b = s.reset(Some(seed), None).unwrap();
                    for _ in 0..prefix { b = s.step(b.token, RunActionIndex(passive(&b.observation) as i64)).unwrap().boundary; } (s, b) };
                let ((s1, _), (s2, _)) = (at(seeds[0]), at(seeds[1]));
                // Negative control (a): the true worlds differ (Runs differ; exact copies stepped alike diverge publicly).
                let (mut w1, mut w2) = (s1.run_for_test().unwrap().clone(), s2.run_for_test().unwrap().clone());
                assert!(w1 != w2);
                let mut diverged = false;
                for _ in 0..60 {
                    let (d1, d2) = (w1.decision().unwrap(), w2.decision().unwrap());
                    if d1.combat != d2.combat || d1.candidates.len() != d2.candidates.len() { diverged = true; break }
                    let Some(c) = d1.candidates.iter().find(|c| matches!(c.action, spire_core::run::GameAction::Combat(spire_core::combat::CombatAction::EndTurn))).or(d1.candidates.first()) else { break };
                    let a = c.action.clone();
                    if w1.step(&a).is_err() || w2.step(&a).is_err() || w1.combat_scope_terminal().is_some() { break }
                }
                if !diverged { continue }
                let (r1, r2) = (s1.public_root().unwrap(), s2.public_root().unwrap());
                assert_eq!(r1, r2, "{character:?} {id}: seeds {seeds:?} have equal public roots");
                assert!(r1.same_world(&r2), "{character:?} {id}: seeds {seeds:?}: permanent-only pairs have equal canonical worlds");
                // P3: identical branches for the same (research_seed, k) and policy seed.
                for k in 0..3 {
                    let (mut x, mut y) = (search(r1.clone(), 31, WIDE), search(r2.clone(), 31, WIDE));
                    let (a, b) = (x.branch(Some(k)).unwrap(), y.branch(Some(k)).unwrap());
                    assert_eq!(rollout(&mut x, a, 100 + k, BranchView::Full), rollout(&mut y, b, 100 + k, BranchView::Full));
                }
                pairs += 1;
            }
            per_entry.push(format!("{id} {}", pairs - before));
            assert!(pairs > before, "{character:?} {id}: no natural pair");
        }
        summary.push(format!("{character:?}: {pairs} pairs ({})", per_entry.join(", ")));
        assert!(pairs >= 3, "{character:?}: {pairs} natural pairs");
    }
    eprintln!("natural pairs: {}", summary.join("; "));
}

/// Design §4.6.4 (review M4): different public roots — including a re-determinised root from a branch — get independent
/// research streams (their digests differ, so every sample key differs), while equal roots share them.
#[test]
fn different_roots_get_independent_streams() {
    let (s1, _) = standalone(Character::Ironclad, 0, "jaw_worm", 17);
    let (s2, _) = standalone(Character::Ironclad, 0, "cultist", 17);
    let (r1, r2) = (s1.public_root().unwrap(), s2.public_root().unwrap());
    assert_ne!(r1.digest(), r2.digest());
    assert_eq!(r1.digest(), s1.public_root().unwrap().digest(), "equal roots, equal digests");
    let mut x = search(r1.clone(), 3, WIDE);
    let a = x.branch(Some(0)).unwrap();
    let d = x.decision(a).unwrap();
    x.step(a, d.token, RunActionIndex(aggressive(&d.observation) as i64), BranchView::Mask).unwrap();
    let nested = x.public_root(a).unwrap();
    assert_ne!(nested.digest(), r1.digest());
    let key = |r: &PublicRoot, k| spire_core::run::research_sample_key(&spire_core::run::research_root(3), &r.digest(), k);
    for k in 0..4 { assert_ne!(key(&r1, k), key(&r2, k)); assert_ne!(key(&r1, k), key(&nested, k)); }
}

/// Design §4.6.3: the digest is SHA-256 over the domain, the encoded Standard combat view and the public knowledge in normal
/// form (draw knowledge and pre-roll candidates), so roots whose views agree but whose knowledge differs (L6) differ.
#[test]
fn root_digest_covers_the_view_and_the_public_knowledge() {
    use sha2::{Digest, Sha256};
    let (s, _) = standalone(Character::Ironclad, 20, "two_louse", 18);
    let root = s.public_root().unwrap();
    let canonical = s.run_for_test().unwrap().public_root_run().unwrap();
    let (view, _, _) = crate::combat_training::standard_combat_publication(&canonical, &RunCapacities::standard_default()).unwrap();
    let knowledge = canonical.public_knowledge_bytes().unwrap();
    assert!(knowledge.len() > 32, "the knowledge bytes carry the draw knowledge and the pre-roll candidates");
    let mut h = Sha256::new();
    h.update(b"spire-gym-public-belief-v1\0root\0");
    super::super::public_search::hash_observation_for_test(&mut h, &view);
    h.update((knowledge.len() as u64).to_le_bytes());
    h.update(&knowledge);
    assert_eq!(root.digest(), <[u8; 32]>::from(h.finalize()));
}

/// Final review v2 I2: the arena's real heap (counting allocator of this test binary) stays within `max_retained_bytes`, and
/// each live branch within its per-branch bound with the knowledge cap replaced by the knowledge's actual heap (so the Run
/// estimate itself is checked), after rollouts, forks and closes at standalone roots of every character at A0 and A20.
#[test]
fn arena_heap_stays_within_the_published_bound() {
    use super::standard_exact_cost::retained_bytes;
    let l = limits(6, 40, 60, 10_000);
    let cap = spire_core::combat::knowledge::KNOWLEDGE_BYTES_CAP as u64;
    let mut checked = 0;
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] { for ascension in [0, 20] {
        for (encounter, seed) in [("jaw_worm", 3u64), ("gremlin_nob", 5)] {
            let (s, _) = standalone(character, ascension, encounter, seed);
            let knowledge = s.run_for_test().unwrap().public_knowledge_heap_bytes().unwrap_or(0) as u64;
            // The Run estimate term on its own: a root's retained heap is its Run's; a second fresh branch's (the map's root
            // node exists) is its box, its Run's heap, its legal-action table and its map slot.
            let (root, root_heap) = retained_bytes(|| s.public_root().unwrap());
            let mut probe = search(root, seed, l);
            probe.branch(None).unwrap();
            let (_, branch_heap) = retained_bytes(|| probe.branch(None).unwrap());
            let fresh = probe.spec()["branch_retained_bytes"].clone();
            let fresh_bound = BRANCH_RUN_BYTES_ESTIMATE + knowledge + fresh["legal_actions"].as_u64().unwrap() + fresh["branch_entry"].as_u64().unwrap();
            assert!(root_heap >= 0 && (root_heap as u64) <= BRANCH_RUN_BYTES_ESTIMATE + knowledge, "{character:?} A{ascension} {encounter}: root heap {root_heap}");
            assert!(branch_heap >= 0 && (branch_heap as u64) <= fresh_bound, "{character:?} A{ascension} {encounter}: fresh branch heap {branch_heap} > {fresh_bound}");
            // One live branch rolled to the step limit or its end: its whole retained heap.
            let (arena, one) = retained_bytes(|| {
                let mut arena = search(s.public_root().unwrap(), seed, l);
                let b = arena.branch(None).unwrap();
                let mut mask = arena.decision(b).unwrap().observation.action_mask;
                let mut token = RunDecisionToken([b.0, 0]);
                for _ in 0..l.max_steps_per_branch {
                    let Some(r) = legal(&RunObservation { groups: vec![], action_mask: mask.clone() }).into_iter()
                        .find_map(|a| arena.step(b, token, RunActionIndex(a as i64), BranchView::Mask).ok()) else { break };
                    if r.terminated || r.truncated { break }
                    (mask, token) = (r.action_mask, r.token);
                }
                arena
            });
            let per_branch = arena.spec()["branch_retained_bytes_bound"].as_u64().unwrap() - cap + knowledge;
            let fixed = arena.spec()["retained_bytes"]["fixed"].as_u64().unwrap() - cap + knowledge;
            assert!(one >= 0 && (one as u64) <= fixed + per_branch, "{character:?} A{ascension} {encounter}: one branch {one} > {}", fixed + per_branch);
            // The full budget: live branches up to the limit, forks, and closes up to max_branches created.
            let (mut arena, all) = retained_bytes(|| {
                let mut arena = search(s.public_root().unwrap(), seed + 1, l);
                while arena.counters()["branches_created"].as_u64().unwrap() < l.max_branches {
                    if arena.counters()["live_branches"].as_u64().unwrap() as usize >= l.max_live_branches {
                        let live = arena.branches_for_test();
                        arena.close(live[0]).unwrap();
                    }
                    let b = arena.branch(None).unwrap();
                    let c = arena.counters();
                    if b.0 % 2 == 0 && (c["live_branches"].as_u64().unwrap() as usize) < l.max_live_branches && c["branches_created"].as_u64().unwrap() < l.max_branches {
                        arena.fork(b).unwrap();
                    }
                }
                arena
            });
            let spec = arena.spec();
            let bound = spec["max_retained_bytes"].as_u64().unwrap() - (l.max_live_branches as u64 + 1) * (cap - knowledge);
            assert!(all >= 0 && (all as u64) <= bound, "{character:?} A{ascension} {encounter}: arena {all} > {bound}");
            assert_eq!(arena.counters()["branches_created"], json!(l.max_branches));
            println!("arena_bound {character:?} A{ascension} {encounter}: root {root_heap} <= {} ; fresh_branch {branch_heap} <= {fresh_bound} ; root+rolled_branch {one} <= {} ; arena {all} <= {bound}",
                BRANCH_RUN_BYTES_ESTIMATE + knowledge, fixed + per_branch);
            arena.close(arena.branches_for_test()[0]).unwrap();
            checked += 1;
        }
    }}
    assert_eq!(checked, 16);
}

/// Final review v3: a branch creation that fails (here an exhausted session-id allocator) retains nothing: no sample index,
/// no branch, no counter, so `used_samples` stays within `max_branches` and the published bound.
#[test]
fn failed_branch_creation_retains_nothing() {
    let (s, _) = standalone(Character::Ironclad, 0, "jaw_worm", 4);
    let mut arena = search(s.public_root().unwrap(), 2, limits(4, 8, 10, 100));
    crate::public_search::FAIL_ID_ALLOCATION.with(|f| f.set(true));
    for k in 0..1_000 { assert!(matches!(arena.branch(Some(k)), Err(RunSessionError::DeliveryFault(_)))); }
    assert!(matches!(arena.branch(None), Err(RunSessionError::DeliveryFault(_))));
    crate::public_search::FAIL_ID_ALLOCATION.with(|f| f.set(false));
    assert_eq!((arena.used_samples_for_test(), arena.counters()["branches_created"].as_u64()), (0, Some(0)));
    let b = arena.branch(None).unwrap();
    assert_eq!(arena.used_samples_for_test(), 1);
    let zero = arena.branch(Some(0)).unwrap();
    assert_eq!(arena.decision(b).unwrap().observation, arena.decision(zero).unwrap().observation, "auto index 0 was not consumed by the failures");
}
