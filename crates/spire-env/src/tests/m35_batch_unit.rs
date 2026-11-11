//! M35 task 1 unit checks (crate-private seams): worker stacks, the shared pool, and the combat-training session's
//! batch status methods.
use crate::workers::{Scheduler, WORKER_STACK_BYTES};
use crate::{CombatTrainingCatalog, CombatTrainingSession, RunActionIndex, RunSessionError};
use spire_core::combat::rng::Partition;

/// Uses about `depth` × 64 KiB of stack.
#[inline(never)]
fn burn(depth: usize) -> u64 {
    let buf = std::hint::black_box([depth as u8; 64 * 1024]);
    if depth == 0 { return u64::from(buf[17]) }
    u64::from(buf[depth % buf.len()]) + std::hint::black_box(burn(depth - 1))
}

#[test]
fn scheduler_workers_have_8mib_stacks() {
    assert_eq!(WORKER_STACK_BYTES, 8 * 1024 * 1024);
    // About 5 MiB of frames: beyond rayon's 2 MiB default worker stack, within 8 MiB (removing the explicit stack size
    // makes this test abort; checked as a mutation).
    let Scheduler::Parallel(pool) = Scheduler::new(2, 2).unwrap() else { panic!("two workers must build a pool") };
    let total: Vec<u64> = pool.install(|| { use rayon::prelude::*; (0..2usize).into_par_iter().map(|_| burn(80)).collect() });
    assert_eq!(total.len(), 2);
    let Scheduler::Parallel(shared) = Scheduler::shared(2).unwrap() else { panic!("shared(2) must be parallel") };
    assert!(shared.install(|| burn(80)) > 0);
}

#[test]
fn shared_pool_is_reused() {
    assert!(matches!(Scheduler::shared(1).unwrap(), Scheduler::Serial), "one worker touches no pool");
    assert!(matches!(Scheduler::shared(0).unwrap(), Scheduler::Serial));
    let (Scheduler::Parallel(a), Scheduler::Parallel(b), Scheduler::Parallel(c)) = (Scheduler::shared(3).unwrap(), Scheduler::shared(3).unwrap(), Scheduler::shared(5).unwrap())
        else { panic!("shared pools must be parallel") };
    assert!(std::sync::Arc::ptr_eq(&a, &b), "the same worker count reuses one process-wide pool");
    assert!(!std::sync::Arc::ptr_eq(&a, &c));
    assert_eq!(a.current_num_threads(), 3);
    // Batches keep private pools.
    let (Scheduler::Parallel(p), Scheduler::Parallel(q)) = (Scheduler::new(3, 8).unwrap(), Scheduler::new(3, 8).unwrap()) else { panic!() };
    assert!(!std::sync::Arc::ptr_eq(&p, &q) && !std::sync::Arc::ptr_eq(&p, &a));
}

const GYM66: &str = r#"{"entries":[{"id":"fixture","encounter":"cultist","hp":80,"max_hp":80,"deck":[{"card":"strike","upgrades":0},{"card":"defend","upgrades":0},{"card":"bash","upgrades":0}],"relics":[],"potions":[null,null,null],"relic_counters":{}}]}"#;

/// A one-entry `standard-combat-v1` catalog (Ironclad starter resources against the Cultist).
pub(super) fn standard_catalog() -> Vec<u8> {
    use serde_json::json;
    let deck: Vec<_> = spire_core::content::Catalog::standard().starter_deck(spire_core::content::Character::Ironclad).unwrap().iter()
        .map(|c| json!({"card": c.as_str(), "upgrades": 0})).collect();
    serde_json::to_vec(&json!({"catalog_schema_id": crate::STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": crate::STANDARD_COMBAT_PROFILE_ID,
        "entries": [{"id": "a", "character": "ironclad", "ascension": 0, "encounter": "cultist", "hp": 70, "max_hp": 80, "gold": 99,
            "deck": deck, "relics": ["burning_blood"], "potions": [null, null, null]}]})).unwrap()
}

#[test]
fn combat_training_step_with_status_matches_last_step_committed() {
    let mut s = CombatTrainingSession::new(CombatTrainingCatalog::from_json(GYM66.as_bytes()).unwrap(), Partition::Test).unwrap();
    let b = s.reset(Some(7), None).unwrap();
    let serial = s.publication_serial();
    // A stale token: not committed, nothing changes, the session stays healthy (no freeze).
    let stale = crate::RunDecisionToken([b.token.0[0], b.token.0[1] + 9]);
    let failure = s.step_with_status(stale, RunActionIndex(0)).unwrap_err();
    assert_eq!((failure.error, failure.committed), (RunSessionError::StaleToken, false));
    assert_eq!(s.publication_serial(), serial, "no publication, no serial");
    assert!(s.retained_fault().is_none());
    // An accepted step: committed, and the serial advances with the publication.
    let step = s.step_with_status(b.token, RunActionIndex(0)).unwrap();
    assert_eq!(s.diagnostic()["last_step_committed"], true);
    assert_eq!(s.publication_serial(), serial + 1);
    assert_eq!(step.boundary.token.0[1], s.publication_serial());
    // A post-commit publication failure (the action table outgrows the capacity): committed, frozen, and equal to
    // `last_step_committed`.
    let mut found = false;
    'seeds: for seed in 0..40u64 {
        let mut s = CombatTrainingSession::new_standard(CombatTrainingCatalog::standard_from_json(&standard_catalog()).unwrap(), Partition::Test).unwrap();
        let mut b = s.reset(Some(seed), None).unwrap();
        s.caps_mut_for_test().actions = b.observation.action_mask.iter().filter(|v| **v != 0).count() - 1;
        for _ in 0..60 {
            let i = b.observation.action_mask.iter().rposition(|v| *v != 0).unwrap();
            let before = s.publication_serial();
            match s.step_with_status(b.token, RunActionIndex(i as i64)) {
                Ok(next) if next.terminated => continue 'seeds,
                Ok(next) => b = next.boundary,
                Err(failure) => {
                    assert_eq!(failure.committed, s.diagnostic()["last_step_committed"].as_bool().unwrap());
                    assert!(failure.committed, "a publication failure follows a committed action: {:?}", failure.error);
                    assert_eq!(s.publication_serial(), before + 1);
                    assert!(s.retained_fault().is_some());
                    found = true;
                    break 'seeds;
                }
            }
        }
    }
    assert!(found, "a seed must reach a post-commit capacity fault");
}
