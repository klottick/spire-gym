//! M35 task 4: native Gym66 and Standard combat batches agree with scalar sessions.
mod m35_support;
use m35_support::*;
use serde_json::{json, Value};
use spire_core::combat::rng::Partition;
use spire_env::{AutoresetMode, BatchConfig, CombatTrainingBatch, CombatTrainingCatalog,
    CombatTrainingSession, EnvError, FaultPolicy, InputReason, RunActionIndex, RunSession, RunSessionBatch, RunSessionError,
    SeedBatch, IntegerSeed, expand_seeds, expand_scalar_seed, SessionBatchError};
use std::sync::Arc;

fn config(workers: usize, autoreset: AutoresetMode) -> BatchConfig {
    BatchConfig { workers, autoreset, fault_policy: FaultPolicy::Freeze }
}
fn gym_catalog(v2: bool) -> Arc<CombatTrainingCatalog> {
    let entries = ["cultist", "jaw_worm", "three_louse"].iter().enumerate().map(|(i, encounter)| {
        let mut e = json!({"id":format!("entry-{i}"), "encounter":encounter, "hp":80-i*5, "max_hp":80,
            "deck":[{"card":"strike","upgrades":0},{"card":"defend","upgrades":0},{"card":"bash","upgrades":0}],
            "relics":[], "potions":[null,null,null]});
        if v2 { e["gold"] = json!(7+i); }
        e
    }).collect::<Vec<_>>();
    let mut c = json!({"entries":entries});
    if v2 { c["profile_id"] = json!("ironclad-combat-v2"); }
    Arc::new(CombatTrainingCatalog::from_json(&serde_json::to_vec(&c).unwrap()).unwrap())
}
fn legal(o: &spire_env::RunObservation, seed: u64, step: u64) -> i64 {
    r1_index(seed, step, &o.action_mask) as i64
}
#[test]
fn gym66_batch_matches_scalar_sessions() {
    for v2 in [false, true] {
        let catalog = gym_catalog(v2);
        let ids = (0..6).map(|i| format!("entry-{}", i % 3)).collect::<Vec<_>>();
        let seeds = (0..6).map(|i| 8300 + i as u64 * 19).collect::<Vec<_>>();
        for workers in [1, 4] {
            let mut batch = CombatTrainingBatch::new_combat(catalog.clone(), Partition::Test, 6, None,
                config(workers, AutoresetMode::Disabled)).unwrap();
            let mut current = batch.reset(&seeds.iter().copied().map(Some).collect::<Vec<_>>(),
                &ids.iter().cloned().map(Some).collect::<Vec<_>>(), None).unwrap()
                .into_iter().map(|r| r.boundary).collect::<Vec<_>>();
            let mut singles = (0..6).map(|k| {
                let mut s = CombatTrainingSession::new((*catalog).clone(), Partition::Test).unwrap();
                let b = s.reset(Some(seeds[k]), Some(&ids[k])).unwrap();
                (s, b)
            }).collect::<Vec<_>>();
            let mut masked_terminal_resets = 0usize;
            for k in 0..6 {
                assert_eq!(current[k].observation, singles[k].1.observation);
                assert_eq!(current[k].info, singles[k].1.info);
                assert_eq!(current[k].info["entry_id"], ids[k]);
                assert_eq!(batch.session(k).unwrap().effective_seed(), Some(seeds[k]));
            }
            for step in 1..=300u64 {
                let mut mask = vec![false; 6];
                for k in 0..6 {
                    if singles[k].1.info["outcome"] != Value::Null {
                        mask[k] = true;
                    }
                }
                if mask.iter().any(|x| *x) {
                    let rows = batch.reset(&vec![None; 6], &ids.iter().cloned().map(Some).collect::<Vec<_>>(), Some(&mask)).unwrap();
                    for k in 0..6 { if mask[k] {
                        masked_terminal_resets += 1;
                        singles[k].1 = singles[k].0.reset(None, Some(&ids[k])).unwrap();
                        current[k] = rows[k].boundary.clone();
                        assert_eq!(current[k].observation, singles[k].1.observation);
                        assert_eq!(current[k].info, singles[k].1.info);
                    }}
                }
                let tokens = current.iter().map(|b| b.token).collect::<Vec<_>>();
                let acts = (0..6).map(|k| RunActionIndex(legal(&current[k].observation, seeds[k], step))).collect::<Vec<_>>();
                let rows = batch.step(&tokens, &acts).unwrap();
                for k in 0..6 {
                    let token = singles[k].1.token;
                    let single = singles[k].0.step(token, acts[k]).unwrap();
                    assert_eq!(rows[k].boundary.observation, single.boundary.observation, "v2={v2} worker={workers} slot={k} step={step} obs");
                    assert_eq!(rows[k].boundary.info, single.boundary.info, "v2={v2} worker={workers} slot={k} step={step} info");
                    assert_eq!(rows[k].reward, single.reward);
                    assert_eq!(rows[k].terminated, single.terminated);
                    assert_eq!(rows[k].boundary.info["entry_id"], single.boundary.info["entry_id"]);
                    assert_eq!(batch.session(k).unwrap().effective_seed(), singles[k].0.effective_seed());
                    singles[k].1 = single.boundary;
                    current[k] = rows[k].boundary.clone();
                }
            }
            assert!(masked_terminal_resets > 0, "v2={v2} worker={workers} must exercise masked terminal resets");
        }
    }
}

#[test]
fn standard_combat_mixed_entries_match_single() {
    let (catalog, ids) = r4_shared_catalog();
    let seeds = (0..8).map(|k| 7100 + k as u64).collect::<Vec<_>>();
    for workers in [1, 4] {
        let mut batch = CombatTrainingBatch::new_combat(catalog.clone(), Partition::Test, 8, None,
            config(workers, AutoresetMode::NextStep)).unwrap();
        let entries = ids.iter().cloned().map(Some).collect::<Vec<_>>();
        let mut current = batch.reset(&seeds.iter().copied().map(Some).collect::<Vec<_>>(), &entries, None).unwrap()
            .into_iter().map(|r| r.boundary).collect::<Vec<_>>();
        let mut singles = ids.iter().enumerate().map(|(k,id)| {
            let mut s = CombatTrainingSession::new_standard((*catalog).clone(), Partition::Test).unwrap();
            let b = s.reset(Some(seeds[k]), Some(id)).unwrap(); (s,b)
        }).collect::<Vec<_>>();
        let kinds = current.iter().map(|b| b.info["start_kind"].clone()).collect::<Vec<_>>();
        assert!(kinds.iter().any(|v| v == "fresh_start") && kinds.iter().any(|v| v == "synthetic_position")
            && kinds.iter().any(|v| v == "reachable_recipe"));
        let mut episodes = [0usize; 8];
        for step in 1..=450u64 {
            let tokens = current.iter().map(|b| b.token).collect::<Vec<_>>();
            let actions = (0..8).map(|k| RunActionIndex(if batch.pending_autoreset(k) { -999 } else { legal(&current[k].observation, seeds[k], step) })).collect::<Vec<_>>();
            let rows = batch.step(&tokens, &actions).unwrap();
            for k in 0..8 {
                assert!(!rows[k].truncated, "worker={workers}, slot={k}, step={step}");
                let expected = if singles[k].1.info["terminal_reason"] != Value::Null {
                    let b = singles[k].0.reset(None, Some(&ids[k])).unwrap();
                    assert!(rows[k].autoreset);
                    (b, 0, false)
                } else {
                    assert!(!rows[k].autoreset, "worker={workers}, slot={k}, step={step}");
                    let token = singles[k].1.token;
                    let r = singles[k].0.step(token, actions[k]).unwrap();
                    (r.boundary, r.reward, r.terminated)
                };
                assert_eq!(rows[k].boundary.observation, expected.0.observation, "worker={workers}, slot={k}, step={step}");
                assert_eq!(rows[k].boundary.info, expected.0.info);
                assert_eq!(rows[k].reward, expected.1);
                assert_eq!(rows[k].terminated, expected.2);
                episodes[k] += usize::from(rows[k].terminated);
                assert_eq!(rows[k].boundary.info["entry_id"], ids[k]);
                assert_eq!(rows[k].boundary.info["start_kind"], expected.0.info["start_kind"]);
                singles[k].1 = expected.0;
                current[k] = rows[k].boundary.clone();
            }
        }
        for (k, count) in episodes.into_iter().enumerate() { assert!(count >= 3, "slot {k} had only {count} terminal episodes"); }
    }
}

#[test]
fn kind_rejects_mismatched_catalog() {
    let (std_json, records, _) = r4_catalog();
    let mut gym = gym_catalog(true).descriptor();
    gym.as_object_mut().unwrap().remove("catalog_schema_id");
    let gym_bytes = serde_json::to_vec(&gym).unwrap();
    let std = CombatTrainingCatalog::standard_from_json_with_records(&std_json, &records).unwrap();
    assert!(matches!(CombatTrainingCatalog::from_json(&std_json), Err(RunSessionError::UnsupportedBoundary(_))));
    assert!(matches!(CombatTrainingCatalog::standard_from_json(&gym_bytes), Err(RunSessionError::UnsupportedBoundary(_))));
    assert!(matches!(CombatTrainingSession::new_shared(Arc::new(std.clone()), Partition::Test), Err(RunSessionError::UnsupportedBoundary(_))));
    assert!(matches!(CombatTrainingSession::new_standard((*gym_catalog(true)).clone(), Partition::Test), Err(RunSessionError::UnsupportedBoundary(_))));
    // The binding rejects kind="ironclad" with non-empty records; native from_json has no records parameter, so test
    // the native layer's available counterpart: the Standard parser rejects the Gym66 catalog kind.
    assert!(matches!(CombatTrainingCatalog::standard_from_json_with_records(&gym_bytes, &records), Err(RunSessionError::UnsupportedBoundary(_))));
}

#[test]
fn records_validated_as_scalar() {
    let (bytes, records, ids) = r4_catalog();
    let parsed = Arc::new(CombatTrainingCatalog::standard_from_json_with_records(&bytes, &records).unwrap());
    let batch = CombatTrainingBatch::new_combat(parsed.clone(), Partition::Test, 8, Some(ids.into_iter().map(Some).collect()), config(2, AutoresetMode::Disabled)).unwrap();
    for k in 0..8 { assert!(Arc::ptr_eq(&parsed, batch.session(k).unwrap().shared_catalog())); }
    let missing = CombatTrainingCatalog::standard_from_json(&bytes).unwrap_err();
    let too_big = CombatTrainingCatalog::standard_from_json_with_records(&bytes, &[vec![0; 8 * 1024 * 1024 + 1]]).unwrap_err();
    let wrong_sha = CombatTrainingCatalog::standard_from_json_with_records(&bytes, &[b"{}".to_vec()]).unwrap_err();
    assert_eq!(format!("{missing:?}"), "Input(\"recipe.record_sha256: record not supplied\")");
    assert_eq!(format!("{too_big:?}"), "Input(\"records: a record exceeds 8 MiB\")");
    assert_eq!(wrong_sha, missing, "a supplied record with the wrong digest is reported as missing, matching scalar validation");
}

#[test]
fn masked_reset_and_entry_ids() {
    let (catalog, ids) = r4_shared_catalog();
    let defaults = vec![Some(ids[3].clone()), Some(ids[5].clone()), Some(ids[7].clone())];
    let mut batch = CombatTrainingBatch::new_combat(catalog.clone(), Partition::Test, 3, Some(defaults.clone()), config(2, AutoresetMode::NextStep)).unwrap();
    let mut rows = batch.reset(&[Some(13),Some(14),Some(15)], &[None,None,None], None).unwrap();
    for k in 0..3 { assert_eq!(rows[k].boundary.info["entry_id"], Value::from(defaults[k].clone().unwrap())); }
    let mask = [true, false, true];
    let explicit = [Some(ids[1].clone()), None, Some(ids[2].clone())];
    rows = batch.reset(&[Some(20),None,Some(22)], &explicit, Some(&mask)).unwrap();
    assert_eq!(rows[0].boundary.info["entry_id"], ids[1]);
    assert_eq!(rows[1].boundary.info["entry_id"], Value::from(defaults[1].clone().unwrap()));
    assert_eq!(rows[2].boundary.info["entry_id"], ids[2]);
    // A terminal slot's NEXT_STEP reset retains the entry of its most recent explicit reset.
    let mut current = rows.into_iter().map(|r| r.boundary).collect::<Vec<_>>();
    let mut autoresets = [0usize; 3];
    let mut sticky_entry_checks = [0usize; 3];
    for step in 1..=500u64 {
        let pending = (0..3).map(|k| batch.pending_autoreset(k)).collect::<Vec<_>>();
        let tokens = current.iter().map(|b| b.token).collect::<Vec<_>>();
        let actions = (0..3).map(|k| RunActionIndex(if pending[k] { -1 } else { legal(&current[k].observation, 100+k as u64, step) })).collect::<Vec<_>>();
        let out = batch.step(&tokens, &actions).unwrap();
        for k in 0..3 {
            assert_eq!(out[k].autoreset, pending[k], "slot={k} step={step}");
            let expected = if k==0 { &ids[1] } else if k==1 { &ids[5] } else { &ids[2] };
            if out[k].autoreset {
                autoresets[k] += 1;
                assert_eq!(out[k].boundary.info["entry_id"], Value::from(expected.clone()), "sticky entry after autoreset, slot={k} step={step}");
                sticky_entry_checks[k] += 1;
            }
            current[k] = out[k].boundary.clone();
        }
    }
    assert!(autoresets.iter().sum::<usize>() > 0, "NEXT_STEP autoresets must occur");
    for k in 0..3 {
        if autoresets[k] > 0 {
            assert!(sticky_entry_checks[k] > 0, "slot {k} autoreset without a sticky-entry check");
        }
    }
}

fn input_parts(error: &SessionBatchError) -> (Option<usize>, &RunSessionError) {
    match error {
        SessionBatchError::Input { slot, error } => (*slot, error),
        other => panic!("expected SessionBatchError::Input, got {other:?}"),
    }
}

#[test]
fn scalar_seed_expansion_shared() {
    for seed in [0, 1, 42, u64::MAX - 8] {
        let expanded = expand_scalar_seed(seed, 8).unwrap();
        let parsed = IntegerSeed::from_be_bytes(&seed.to_be_bytes()).unwrap();
        let legacy = expand_seeds(SeedBatch::Scalar(parsed), 8).unwrap().into_iter()
            .map(|s| s.unwrap().as_be_bytes().iter().fold(0u64, |n,b| (n << 8) | u64::from(*b))).collect::<Vec<_>>();
        assert_eq!(expanded, legacy);
    }
    assert!(matches!(expand_scalar_seed(u64::MAX - 1, 3), Err(EnvError::Input { reason: InputReason::Seed, .. })));
}

#[test]
fn fault_envelope_matches_run_batch() {
    let run_configs = r1_standard_configs()[..2].to_vec();
    let run_catalog = {
        let e = json!({"entries":[
            {"id":"a","encounter":"cultist","hp":80,"max_hp":80,"deck":[{"card":"strike","upgrades":0}],"relics":[],"potions":[null,null,null]},
            {"id":"b","encounter":"jaw_worm","hp":80,"max_hp":80,"deck":[{"card":"strike","upgrades":0}],"relics":[],"potions":[null,null,null]}]});
        Arc::new(CombatTrainingCatalog::from_json(&serde_json::to_vec(&e).unwrap()).unwrap())
    };
    let entries = vec![Some("a".to_owned()),Some("b".to_owned())];
    let mut run = RunSessionBatch::new_run(run_configs, RunSession::new_standard, config(1, AutoresetMode::Disabled)).unwrap();
    let mut combat = CombatTrainingBatch::new_combat(run_catalog, Partition::Test, 2, None, config(1, AutoresetMode::Disabled)).unwrap();
    let partial = [true,false];
    let re = run.reset(&[Some(1),Some(2)], &vec![None;2], Some(&partial)).unwrap_err();
    let ce = combat.reset(&[Some(1),Some(2)], &entries, Some(&partial)).unwrap_err();
    assert_eq!(re, ce);
    assert_eq!(input_parts(&re), (None, &RunSessionError::Input("the first batch reset must select every slot")));
    let rr = run.reset(&[Some(1),Some(2)], &vec![None;2], None).unwrap();
    let cr = combat.reset(&[Some(1),Some(2)], &entries, None).unwrap();
    let stale_run = run.reset(&[Some(3),Some(4)], &vec![None;2], None).unwrap();
    let stale_combat = combat.reset(&[Some(3),Some(4)], &entries, None).unwrap();
    let re = run.step(&[rr[0].boundary.token,rr[1].boundary.token], &[RunActionIndex(0);2]).unwrap_err();
    let ce = combat.step(&[cr[0].boundary.token,cr[1].boundary.token], &[RunActionIndex(0);2]).unwrap_err();
    assert_eq!(re, ce);
    assert_eq!(input_parts(&re), (Some(0), &RunSessionError::StaleToken));
    let re = run.step(&stale_run.iter().map(|b|b.boundary.token).collect::<Vec<_>>(), &[RunActionIndex(i64::MAX);2]).unwrap_err();
    let ce = combat.step(&stale_combat.iter().map(|r|r.boundary.token).collect::<Vec<_>>(), &[RunActionIndex(i64::MAX);2]).unwrap_err();
    assert_eq!(input_parts(&re), (Some(0), &RunSessionError::Input("action index is not legal")));
    assert_eq!(input_parts(&ce), (Some(0), &RunSessionError::Input("combat action index is not legal")));
    run.close().unwrap(); combat.close().unwrap();
    let re = run.reset(&[None;2], &vec![None;2], None).unwrap_err();
    let ce = combat.reset(&[None;2], &entries, None).unwrap_err();
    assert_eq!(re, ce);
    assert_eq!(input_parts(&re), (None, &RunSessionError::Closed));
}
/// An entry id given for an unselected slot is still validated, and a bad one rejects the whole reset with no effect
/// (the pre-M35 Gym66 batch rule, kept for `test_batch_reset_preflights_unselected_entry_ids_before_resetting_selected_rows`).
#[test]
fn unselected_entry_ids_are_validated_atomically() {
    let mut batch = CombatTrainingBatch::new_combat(gym_catalog(false), Partition::Test, 2, None, config(1, AutoresetMode::Disabled)).unwrap();
    let first = batch.reset(&[Some(5), Some(6)], &[None, None], None).unwrap();
    let error = batch.reset(&[Some(9), Some(10)], &[Some("entry-1".into()), Some("unknown".into())], Some(&[true, false])).unwrap_err();
    assert!(matches!(error, SessionBatchError::Input { slot: Some(1), .. }), "{error:?}");
    for (slot, row) in first.iter().enumerate() { assert_eq!(batch.decision(slot).unwrap(), row.boundary, "slot {slot} unchanged"); }
    let rows = batch.reset(&[Some(9), Some(10)], &[Some("entry-1".into()), Some("entry-2".into())], Some(&[true, false])).unwrap();
    assert_eq!(rows[1].boundary, first[1].boundary, "a valid unselected entry leaves the slot untouched");
    assert_eq!(rows[0].boundary.info["entry_id"], "entry-1");
}
