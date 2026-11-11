//! M36 spire-dmm M5 (design §3.5): the public-search worker cap and the shared pool lifetime.
//! Race rule (R-I1): each test owns worker counts no other test in the crate uses (except as noted), so tests running in parallel cannot
//! hold, share or make hot one of these pools: the lifetime test uses 7 (and 13 for the hot slot), pools-shared uses 11
//! and the cap test only 0, 256 and 257 (it never steps). The reconfigure test uses 5, which `m35_batch_unit` also takes
//! through `Scheduler::shared(5)`, so it asserts only on its own search's pool field. No assertion reads the whole registry.
use super::m34s_transplant::{standard_combat_session, standard_entry};
use crate::*;
use spire_core::content::Character;

const LIMITS: SearchLimits = SearchLimits { max_live_branches: 4, max_branches: 8, max_steps_per_branch: 16, max_total_steps: 64 };
fn root(seed: u64) -> PublicRoot {
    let mut s = standard_combat_session(vec![standard_entry("e", Character::Ironclad, 0, "jaw_worm", None)]);
    s.reset(Some(seed), None).unwrap();
    s.public_root().unwrap()
}
fn search(workers: usize) -> PublicSearch {
    PublicSearch::new(root(0x36_05), SearchConfig { research_seed: 1, limits: LIMITS }).unwrap().with_workers(workers).unwrap()
}
/// One parallel `step_many` call on a fresh branch (the first parallel call takes the search's pool).
fn step_once(s: &mut PublicSearch) {
    let b = s.branch(None).unwrap();
    let d = s.decision(b).unwrap();
    let index = d.observation.action_mask.iter().position(|m| *m != 0).unwrap();
    s.step_many(&[BranchStepRequest { branch: b, token: d.token, index: RunActionIndex(index as i64) }], BranchView::Mask).unwrap();
}

#[test]
fn m36_search_workers_cap() {
    assert_eq!(MAX_SEARCH_WORKERS, BATCH_MAX_SLOTS, "the search cap equals the batch slot ceiling");
    let new = || PublicSearch::new(root(0x36_06), SearchConfig { research_seed: 1, limits: LIMITS }).unwrap();
    for bad in [0, MAX_SEARCH_WORKERS + 1] {
        assert_eq!(new().with_workers(bad).err(), Some(RunSessionError::Input("search workers must be 1..=256")), "workers={bad}");
    }
    assert_eq!(new().with_workers(MAX_SEARCH_WORKERS).unwrap().workers(), 256);
}

#[test]
fn m36_search_pool_lifetime() {
    let mut s = search(7);
    assert!(s.pool_for_test().is_none(), "no pool before the first parallel step_many");
    step_once(&mut s);
    let weak = s.pool_for_test().expect("a parallel step_many holds its pool");
    assert!(weak.upgrade().is_some_and(|p| p.current_num_threads() == 7), "the pool lives while its search does");
    assert!(crate::workers::registry_contains_for_test(7));
    drop(s);
    let mut hot = search(13);
    step_once(&mut hot);
    assert!(weak.upgrade().is_none(), "the dropped search's pool is freed once another pool is hot");
    assert!(!crate::workers::registry_contains_for_test(7), "dead handles are pruned");
}

#[test]
fn m36_search_pools_shared() {
    let (mut a, mut b) = (search(11), search(11));
    step_once(&mut a);
    step_once(&mut b);
    let (pa, pb) = (a.pool_for_test().unwrap().upgrade().unwrap(), b.pool_for_test().unwrap().upgrade().unwrap());
    assert!(std::sync::Arc::ptr_eq(&pa, &pb), "live searches with one count share one pool");
    let mut serial = search(1);
    step_once(&mut serial);
    assert!(serial.pool_for_test().is_none(), "workers=1 touches no pool");
}

#[test]
fn m36_search_with_workers_releases_the_taken_pool() {
    // Reconfiguring the worker count after a parallel `step_many` drops the held pool, so the next call takes the pool
    // for the new count (or none for 1); keeping the same count keeps the pool.
    let mut s = search(5);
    step_once(&mut s);
    assert!(s.pool_for_test().is_some(), "the first parallel step_many takes the 5-worker pool");
    let mut s = s.with_workers(5).unwrap();
    assert!(s.pool_for_test().is_some(), "the same count keeps the held pool");
    s = s.with_workers(1).unwrap();
    assert!(s.pool_for_test().is_none(), "a new count releases the held pool");
    step_once(&mut s);
    assert!(s.pool_for_test().is_none(), "workers = 1 steps serially and takes no pool");
}
