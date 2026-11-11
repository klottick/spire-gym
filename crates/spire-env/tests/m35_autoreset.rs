//! M35 task 2 (design §3.2, §4 R3/R4/R4-neg): masked resets, sticky automatic entries, the explicit-reset entry rule,
//! the three autoreset modes and final observations, completed records and `combat_views()`, each against single
//! sessions.
mod m35_support;
use m35_support::*;
use serde_json::Value;
use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    AutoresetMode, BatchConfig, CombatPolicyBoundary, CombatTrainingBatch, CombatTrainingCatalog, CombatTrainingSession, FaultPolicy,
    RunActionIndex, RunObservation, RunSession, RunSessionBatch, RunSessionError, SessionBatchError, StandardRunConfig,
};
use std::sync::Arc;

fn config(workers: usize, autoreset: AutoresetMode) -> BatchConfig { BatchConfig { workers, autoreset, fault_policy: FaultPolicy::Freeze } }

/// One combat slot's row: what a consumer sees, tokens excluded.
#[derive(Clone, Debug, PartialEq)]
struct CRow { obs: RunObservation, info: Value, reward: i32, terminated: bool, autoreset: bool, final_obs: Option<(RunObservation, Value)> }
fn crow(b: &CombatPolicyBoundary, reward: i32, terminated: bool, autoreset: bool, final_obs: Option<&CombatPolicyBoundary>) -> CRow {
    CRow { obs: b.observation.clone(), info: b.info.clone(), reward, terminated, autoreset, final_obs: final_obs.map(|f| (f.observation.clone(), f.info.clone())) }
}

/// The batch path: explicit initial entries, R1 actions, `steps` batch steps.
fn batch_rows(catalog: &Arc<CombatTrainingCatalog>, ids: &[String], seeds: &[u64], mode: AutoresetMode, workers: usize, steps: u64) -> Vec<Vec<CRow>> {
    let n = ids.len();
    let mut batch = CombatTrainingBatch::new_combat(catalog.clone(), Partition::Test, n, None, config(workers, mode)).unwrap();
    let entries: Vec<_> = ids.iter().map(|id| Some(id.clone())).collect();
    let out = batch.reset(&seeds.iter().map(|s| Some(*s)).collect::<Vec<_>>(), &entries, None).unwrap();
    let mut current: Vec<CombatPolicyBoundary> = out.iter().map(|r| r.boundary.clone()).collect();
    let mut rows: Vec<Vec<CRow>> = current.iter().map(|b| vec![crow(b, 0, false, false, None)]).collect();
    let mut pending = vec![false; n];
    for step in 1..=steps {
        let tokens: Vec<_> = current.iter().map(|b| b.token).collect();
        let actions: Vec<_> = (0..n).map(|k| RunActionIndex(if pending[k] { 0 } else { r1_index(seeds[k], step, &current[k].observation.action_mask) as i64 })).collect();
        for (k, r) in batch.step(&tokens, &actions).unwrap().into_iter().enumerate() {
            rows[k].push(crow(&r.boundary, r.reward, r.terminated, r.autoreset, r.final_boundary.as_ref()));
            pending[k] = mode == AutoresetMode::NextStep && r.terminated;
            current[k] = r.boundary;
        }
    }
    rows
}

/// The single-session reference for one slot. `auto_entry` is the entry an automatic reset passes: the slot's current
/// entry (the M35 rule) or `None` (the old seedless rule, `entries[0]`; R4-neg).
fn single_rows(catalog: &Arc<CombatTrainingCatalog>, id: &str, seed: u64, mode: AutoresetMode, steps: u64, sticky: bool) -> Vec<CRow> {
    let mut s = CombatTrainingSession::new_standard((**catalog).clone(), Partition::Test).unwrap();
    let mut b = s.reset(Some(seed), Some(id)).unwrap();
    let mut rows = vec![crow(&b, 0, false, false, None)];
    let auto = |s: &CombatTrainingSession| if sticky { s.current_entry().map(str::to_owned) } else { None };
    let mut pending = false;
    for step in 1..=steps {
        if pending {
            let entry = auto(&s);
            b = s.reset(None, entry.as_deref()).unwrap();
            rows.push(crow(&b, 0, false, true, None));
            pending = false;
            continue
        }
        let r = s.step(b.token, RunActionIndex(r1_index(seed, step, &b.observation.action_mask) as i64)).unwrap();
        if r.terminated && mode == AutoresetMode::SameStep {
            let entry = auto(&s);
            let reset = s.reset(None, entry.as_deref()).unwrap();
            rows.push(crow(&reset, r.reward, true, false, Some(&r.boundary)));
            b = reset;
        } else {
            rows.push(crow(&r.boundary, r.reward, r.terminated, false, None));
            pending = mode == AutoresetMode::NextStep && r.terminated;
            b = r.boundary;
        }
    }
    rows
}

fn entry_stable(rows: &[CRow], id: &str) -> bool {
    rows.iter().all(|r| r.info["entry_id"] == Value::from(id) && r.final_obs.as_ref().is_none_or(|(_, info)| info["entry_id"] == Value::from(id)))
}
fn episodes(rows: &[CRow]) -> usize { rows.iter().filter(|r| r.terminated).count() }

const R4_STEPS: u64 = 450;

fn r4(mode: AutoresetMode) {
    let (catalog, ids) = r4_shared_catalog();
    let seeds: Vec<u64> = (0..ids.len() as u64).map(|k| 7_100 + k).collect();
    let reference = batch_rows(&catalog, &ids, &seeds, mode, 1, R4_STEPS);
    assert_eq!(batch_rows(&catalog, &ids, &seeds, mode, 4, R4_STEPS), reference, "workers 4 equal workers 1");
    let kinds: Vec<_> = reference.iter().map(|rows| rows[0].info["start_kind"].clone()).collect();
    assert!(kinds.contains(&Value::from("synthetic_position")) && kinds.contains(&Value::from("reachable_recipe")) && kinds.contains(&Value::from("fresh_start")), "{kinds:?}");
    for (k, id) in ids.iter().enumerate() {
        assert!(episodes(&reference[k]) >= 3, "slot {k} ({id}) needs at least 3 episodes, got {}", episodes(&reference[k]));
        assert!(entry_stable(&reference[k], id), "slot {k} keeps its entry across automatic resets");
        assert!(reference[k].iter().all(|r| r.info["start_kind"] == kinds[k]), "slot {k} keeps its start kind");
        assert_eq!(reference[k], single_rows(&catalog, id, seeds[k], mode, R4_STEPS, true), "slot {k} equals its single session");
    }
}

#[test]
fn next_step_autoreset_keeps_slot_entry() { r4(AutoresetMode::NextStep) }

#[test]
fn same_step_autoreset_keeps_slot_entry() { r4(AutoresetMode::SameStep) }

/// R4-neg: a comparator applying the old seedless rule (`entries[0]`) on automatic resets must fail the per-slot entry
/// and agreement assertions, which shows the R4 comparator is independent of the rule it checks.
#[test]
fn r4_negative_control_detects_entry_zero_rule() {
    let (catalog, ids) = r4_shared_catalog();
    let seeds: Vec<u64> = (0..ids.len() as u64).map(|k| 7_100 + k).collect();
    for mode in [AutoresetMode::NextStep, AutoresetMode::SameStep] {
        let batch = batch_rows(&catalog, &ids, &seeds, mode, 1, R4_STEPS);
        let mut detected = 0;
        for (k, id) in ids.iter().enumerate().skip(1) {
            let old = single_rows(&catalog, id, seeds[k], mode, R4_STEPS, false);
            assert!(!entry_stable(&old, id), "the old rule moves slot {k} to entries[0]");
            assert_ne!(old, batch[k], "the agreement assertion must fail against the old rule (slot {k})");
            detected += 1;
        }
        assert_eq!(detected, ids.len() - 1);
        // Slot 0's entry is entries[0], where the two rules coincide.
        assert_eq!(single_rows(&catalog, &ids[0], seeds[0], mode, R4_STEPS, false), batch[0]);
    }
}

/// N1: an explicit reset that omits a slot's entry uses the constructor's `entry_ids[slot]` if given, else `entries[0]`,
/// exactly as the scalar `reset(None, entry)`; a slot's current entry never leaks into an explicit reset.
#[test]
fn explicit_reset_omitted_entry_uses_constructor_or_first_entry() {
    let (catalog, ids) = r4_shared_catalog();
    let defaults = vec![None, Some(ids[5].clone()), None, Some(ids[2].clone())];
    let mut batch = CombatTrainingBatch::new_combat(catalog.clone(), Partition::Test, 4, Some(defaults.clone()), config(2, AutoresetMode::Disabled)).unwrap();
    let explicit = vec![Some(ids[3].clone()), Some(ids[4].clone()), Some(ids[6].clone()), Some(ids[1].clone())];
    let seeds = [11u64, 12, 13, 14];
    let first = batch.reset(&seeds.map(Some), &explicit, None).unwrap();
    for k in 0..4 { assert_eq!(first[k].boundary.info["entry_id"], Value::from(explicit[k].clone().unwrap())); }
    let mask = [true, true, true, false];
    let rows = batch.reset(&[None; 4], &[None, None, None, None], Some(&mask)).unwrap();
    let expect = [ids[0].clone(), ids[5].clone(), ids[0].clone(), ids[1].clone()];
    for k in 0..4 { assert_eq!(rows[k].boundary.info["entry_id"], Value::from(expect[k].clone()), "slot {k}"); }
    for k in 0..3 {
        let mut s = CombatTrainingSession::new_standard((*catalog).clone(), Partition::Test).unwrap();
        s.reset(Some(seeds[k]), explicit[k].as_deref()).unwrap();
        let b = s.reset(None, defaults[k].as_deref()).unwrap();
        assert_eq!((b.observation, b.info), (rows[k].boundary.observation.clone(), rows[k].boundary.info.clone()), "slot {k} equals the scalar reset(None, entry)");
    }
}

#[test]
fn masked_reset_leaves_unmasked_slots_identical() {
    let (catalog, ids) = r4_shared_catalog();
    let mut batch = CombatTrainingBatch::new_combat(catalog.clone(), Partition::Test, 4, None, config(3, AutoresetMode::Disabled)).unwrap();
    let entries: Vec<_> = ids[..4].iter().map(|i| Some(i.clone())).collect();
    let rows = batch.reset(&[Some(1), Some(2), Some(3), Some(4)], &entries, None).unwrap();
    let tokens: Vec<_> = rows.iter().map(|r| r.boundary.token).collect();
    let stepped = batch.step(&tokens, &rows.iter().map(|r| RunActionIndex(r1_index(9, 1, &r.boundary.observation.action_mask) as i64)).collect::<Vec<_>>()).unwrap();
    let before: Vec<_> = (0..4).map(|k| batch.decision(k).unwrap()).collect();
    let records_before: Vec<_> = (0..4).map(|k| batch.record(k).unwrap()).collect();
    let mask = [false, true, false, true];
    let out = batch.reset(&[Some(99), Some(5), None, None], &[None, None, None, None], Some(&mask)).unwrap();
    for k in [0, 2] {
        assert_eq!(out[k].boundary, before[k], "unmasked slot {k}: same boundary and token");
        assert_eq!(out[k].boundary, stepped[k].boundary);
        assert_eq!(batch.decision(k).unwrap(), before[k]);
        assert_eq!(batch.record(k).unwrap(), records_before[k]);
        assert_eq!(out[k].boundary.info["entry_id"], Value::from(ids[k].clone()), "entry kept");
    }
    for k in [1, 3] { assert_ne!(out[k].boundary.token, before[k].token); }
}

#[test]
fn first_reset_must_select_all() {
    let (catalog, _) = r4_shared_catalog();
    let mut batch = CombatTrainingBatch::new_combat(catalog, Partition::Test, 3, None, config(1, AutoresetMode::NextStep)).unwrap();
    let err = batch.reset(&[Some(1); 3], &[None, None, None], Some(&[true, false, true])).unwrap_err();
    assert!(matches!(err, SessionBatchError::Input { slot: None, error: RunSessionError::Input(m) } if m.contains("first batch reset")));
    assert!(matches!(batch.step(&[spire_env::RunDecisionToken::from([0, 0]); 3], &[RunActionIndex(0); 3]), Err(SessionBatchError::Input { error: RunSessionError::NotReset, .. })));
    batch.reset(&[Some(1); 3], &[None, None, None], Some(&[true, true, true])).unwrap();
}

/// Runs slot 0 to its terminal with R1 actions (other slots take R1 actions too); returns the batch and its rows.
fn to_first_terminal(mode: AutoresetMode) -> (CombatTrainingBatch, Vec<CombatPolicyBoundary>, Vec<spire_env::SlotRow<CombatPolicyBoundary>>, Arc<CombatTrainingCatalog>, Vec<String>) {
    let (catalog, ids) = r4_shared_catalog();
    let mut batch = CombatTrainingBatch::new_combat(catalog.clone(), Partition::Test, 2, None, config(2, mode)).unwrap();
    let mut current: Vec<_> = batch.reset(&[Some(31), Some(32)], &[Some(ids[0].clone()), Some(ids[2].clone())], None).unwrap().into_iter().map(|r| r.boundary).collect();
    for step in 1..=400 {
        let actions: Vec<_> = (0..2).map(|k| RunActionIndex(r1_index(31 + k as u64, step, &current[k].observation.action_mask) as i64)).collect();
        let rows = batch.step(&current.iter().map(|b| b.token).collect::<Vec<_>>(), &actions).unwrap();
        if rows[0].terminated { let before = current.clone(); return (batch, before, rows, catalog, ids) }
        assert!(!rows[1].terminated || mode != AutoresetMode::Disabled, "slot 1 ends first; pick other seeds");
        current = rows.into_iter().map(|r| r.boundary).collect();
    }
    panic!("slot 0 must terminate within 400 steps")
}

#[test]
fn disabled_rejects_terminal_slot_without_effects() {
    let (mut batch, _, rows, _, _) = to_first_terminal(AutoresetMode::Disabled);
    assert!(rows[0].final_boundary.is_none() && !rows[0].autoreset);
    let before: Vec<_> = (0..2).map(|k| batch.decision(k).unwrap()).collect();
    let tokens: Vec<_> = rows.iter().map(|r| r.boundary.token).collect();
    let err = batch.step(&tokens, &[RunActionIndex(0); 2]).unwrap_err();
    assert_eq!(err, SessionBatchError::Input { slot: Some(0), error: RunSessionError::Terminal });
    for k in 0..2 { assert_eq!(batch.decision(k).unwrap(), before[k], "no effect on slot {k}"); }
    // A masked reset recovers the slot.
    let out = batch.reset(&[None, None], &[None, None], Some(&[true, false])).unwrap();
    assert_eq!(out[1].boundary, before[1]);
}

#[test]
fn next_step_resets_and_ignores_action() {
    let (mut batch, _, rows, catalog, ids) = to_first_terminal(AutoresetMode::NextStep);
    assert!(rows[0].terminated && rows[0].final_boundary.is_none());
    assert!(batch.pending_autoreset(0));
    // Slot 0's token and action are ignored (a stale token and an out-of-range index are fine).
    let tokens = [spire_env::RunDecisionToken::from([0, 0]), rows[1].boundary.token];
    let a1 = r1_index(7, 1, &rows[1].boundary.observation.action_mask) as i64;
    let next = batch.step(&tokens, &[RunActionIndex(999), RunActionIndex(a1)]).unwrap();
    assert!(next[0].autoreset && next[0].reward == 0 && !next[0].terminated && !next[0].truncated);
    assert!(!batch.pending_autoreset(0));
    assert_eq!(next[0].boundary.info["entry_id"], Value::from(ids[0].clone()));
    assert_eq!(next[0].boundary.info["episode_ordinal"], Value::from(1));
    // The reset row equals a single session's reset(None, entry) after the same episode.
    let mut s = CombatTrainingSession::new_standard((*catalog).clone(), Partition::Test).unwrap();
    let mut b = s.reset(Some(31), Some(&ids[0])).unwrap();
    let mut step = 1;
    loop {
        let r = s.step(b.token, RunActionIndex(r1_index(31, step, &b.observation.action_mask) as i64)).unwrap();
        step += 1;
        if r.terminated { assert_eq!(r.boundary.observation, rows[0].boundary.observation); break }
        b = r.boundary;
    }
    let reset = s.reset(None, Some(&ids[0])).unwrap();
    assert_eq!((reset.observation, reset.info), (next[0].boundary.observation.clone(), next[0].boundary.info.clone()));
}

#[test]
fn same_step_final_obs_equals_single_terminal() {
    let (batch, _, rows, catalog, ids) = to_first_terminal(AutoresetMode::SameStep);
    let terminal = rows[0].final_boundary.clone().expect("SAME_STEP carries the terminal boundary");
    assert!(rows[0].terminated && !batch.pending_autoreset(0));
    assert_eq!(rows[0].boundary.info["episode_ordinal"], Value::from(1), "the row is the next episode's reset");
    let mut s = CombatTrainingSession::new_standard((*catalog).clone(), Partition::Test).unwrap();
    let mut b = s.reset(Some(31), Some(&ids[0])).unwrap();
    let mut step = 1;
    loop {
        let r = s.step(b.token, RunActionIndex(r1_index(31, step, &b.observation.action_mask) as i64)).unwrap();
        step += 1;
        if r.terminated {
            assert_eq!((r.boundary.observation, r.boundary.info, r.reward), (terminal.observation.clone(), terminal.info.clone(), rows[0].reward));
            break
        }
        b = r.boundary;
    }
}

#[test]
fn masked_reset_clears_pending_autoreset() {
    let (mut batch, _, rows, _, ids) = to_first_terminal(AutoresetMode::NextStep);
    assert!(batch.pending_autoreset(0));
    let out = batch.reset(&[Some(77), None], &[None, None], Some(&[true, false])).unwrap();
    assert!(!batch.pending_autoreset(0), "a masked reset clears the pending autoreset");
    assert_eq!(out[0].boundary.info["entry_id"], Value::from(ids[0].clone()));
    // The next step steps the slot (it is not reset a second time).
    let tokens = [out[0].boundary.token, rows[1].boundary.token];
    let a = [RunActionIndex(r1_index(77, 1, &out[0].boundary.observation.action_mask) as i64), RunActionIndex(r1_index(1, 1, &rows[1].boundary.observation.action_mask) as i64)];
    let next = batch.step(&tokens, &a).unwrap();
    assert!(!next[0].autoreset);
    assert_eq!(next[0].boundary.info["episode_ordinal"], Value::from(0));
}

/// R3: Standard Run R1 slots, 600 batch steps, every mode against single sessions doing `reset(None)` after a terminal;
/// masked resets at fixed steps (one on a slot with a pending NEXT_STEP autoreset when one exists); completed records
/// equal the single records.
#[test]
fn autoreset_cycles_match_single_reset_none() {
    let configs = r1_standard_configs();
    let n = configs.len();
    let seeds = r1_seeds(n);
    let masked_at = [150u64, 400];
    for mode in [AutoresetMode::Disabled, AutoresetMode::NextStep, AutoresetMode::SameStep] {
        let mut batch = RunSessionBatch::new_run(configs.clone(), RunSession::new_standard, config(4, mode)).unwrap();
        let mut singles: Vec<RunSession> = configs.iter().map(|c| RunSession::new_standard(c.clone()).unwrap()).collect();
        let mut single_records = Vec::new();
        let mut cur: Vec<_> = batch.reset(&seeds.iter().map(|s| Some(*s)).collect::<Vec<_>>(), &vec![None; n], None).unwrap().into_iter().map(|r| r.boundary).collect();
        let mut sb: Vec<_> = singles.iter_mut().zip(&seeds).map(|(s, seed)| s.reset(Some(*seed)).unwrap()).collect();
        let mut pending = vec![false; n];
        let mut terminals = 0;
        for step in 1..=600u64 {
            if masked_at.contains(&step) || (mode == AutoresetMode::Disabled && cur.iter().any(|b| b.terminal.is_some())) {
                // Masked reset: odd slots at fixed steps, plus every terminal slot under DISABLED.
                let mask: Vec<bool> = (0..n).map(|k| (masked_at.contains(&step) && k % 2 == 1) || cur[k].terminal.is_some() && mode == AutoresetMode::Disabled).collect();
                let out = batch.reset(&vec![None; n], &vec![None; n], Some(&mask)).unwrap();
                for k in 0..n {
                    if mask[k] {
                        if let Some(r) = singles[k].record().unwrap() { single_records.push((k, r)); }
                        sb[k] = singles[k].reset(None).unwrap();
                        pending[k] = false;
                    }
                    assert_eq!(out[k].boundary.observation, sb[k].observation, "{mode:?} masked reset slot {k} step {step}");
                    cur[k] = out[k].boundary.clone();
                }
                continue
            }
            let actions: Vec<_> = (0..n).map(|k| RunActionIndex(if pending[k] { 0 } else { r1_index(seeds[k], step, &sb[k].observation.action_mask) as i64 })).collect();
            let rows = batch.step(&cur.iter().map(|b| b.token).collect::<Vec<_>>(), &actions).unwrap();
            for k in 0..n {
                if pending[k] {
                    if let Some(r) = singles[k].record().unwrap() { single_records.push((k, r)); }
                    sb[k] = singles[k].reset(None).unwrap();
                    pending[k] = false;
                    assert!(rows[k].autoreset);
                    assert_eq!(rows[k].boundary.observation, sb[k].observation);
                } else {
                    let r = singles[k].step(sb[k].token, actions[k]).unwrap();
                    assert_eq!((rows[k].reward, &rows[k].reward_events, rows[k].terminated), (r.reward, &r.reward_events, r.terminated), "{mode:?} slot {k} step {step}");
                    if r.terminated {
                        terminals += 1;
                        if mode == AutoresetMode::SameStep {
                            assert_eq!(rows[k].final_boundary.as_ref().unwrap().observation, r.boundary.observation);
                            assert_eq!(rows[k].final_boundary.as_ref().unwrap().terminal, r.boundary.terminal);
                            if let Some(rec) = singles[k].record().unwrap() { single_records.push((k, rec)); }
                            sb[k] = singles[k].reset(None).unwrap();
                        } else {
                            sb[k] = r.boundary;
                            pending[k] = mode == AutoresetMode::NextStep;
                        }
                    } else { sb[k] = r.boundary; }
                    assert_eq!(rows[k].boundary.observation, sb[k].observation, "{mode:?} slot {k} step {step}");
                }
                cur[k] = rows[k].boundary.clone();
            }
        }
        let completed: Vec<_> = batch.take_completed_records().into_iter().map(|c| (c.slot, c.record)).collect();
        // Order: by operation, slot order within one operation; compare as per-slot sequences.
        for k in 0..n {
            let a: Vec<_> = completed.iter().filter(|(s, _)| *s == k).map(|(_, r)| r).collect();
            let b: Vec<_> = single_records.iter().filter(|(s, _)| *s == k).map(|(_, r)| r).collect();
            assert_eq!(a, b, "{mode:?} completed records slot {k}");
        }
        assert!(batch.take_completed_records().is_empty(), "drained");
        eprintln!("R3 {mode:?}: {terminals} terminals, {} completed records", completed.len());
    }
}

#[test]
fn completed_records_drain_in_order() {
    let configs: Vec<StandardRunConfig> = r1_standard_configs().into_iter().take(3).collect();
    let mut batch = RunSessionBatch::new_run(configs, RunSession::new_standard, config(2, AutoresetMode::Disabled)).unwrap();
    batch.reset(&[Some(1), Some(2), Some(3)], &[None, None, None], None).unwrap();
    assert!(batch.take_completed_records().is_empty(), "a first reset finishes no episode");
    batch.reset(&[None, None, None], &[None, None, None], Some(&[true, false, true])).unwrap();
    batch.reset(&[None, None, None], &[None, None, None], Some(&[false, true, true])).unwrap();
    let order: Vec<_> = batch.take_completed_records().into_iter().map(|c| (c.slot, c.record.ordinal)).collect();
    assert_eq!(order, [(0, 0), (2, 0), (1, 0), (2, 1)]);
    assert!(batch.take_completed_records().is_empty());
}

#[test]
fn combat_views_match_scalar_combat_view() {
    // Standard (mixed characters) and policy-v3 (M10 Ironclad); other profiles are Unsupported.
    let configs: Vec<StandardRunConfig> = r1_standard_configs();
    let n = configs.len();
    let seeds = r1_seeds(n);
    let mut batch = RunSessionBatch::new_run(configs.clone(), RunSession::new_standard, config(4, AutoresetMode::NextStep)).unwrap();
    let mut singles: Vec<RunSession> = configs.iter().map(|c| RunSession::new_standard(c.clone()).unwrap()).collect();
    let mut cur: Vec<_> = batch.reset(&seeds.iter().map(|s| Some(*s)).collect::<Vec<_>>(), &vec![None; n], None).unwrap().into_iter().map(|r| r.boundary).collect();
    let mut sb: Vec<_> = singles.iter_mut().zip(&seeds).map(|(s, seed)| s.reset(Some(*seed)).unwrap()).collect();
    let mut saw_combat = vec![false; n];
    for step in 1..=60u64 {
        let (views, in_combat) = batch.combat_views().unwrap();
        for k in 0..n {
            let scalar = singles[k].combat_view().unwrap();
            let view = views[k].as_ref().unwrap();
            assert_eq!((&view.observation, &view.info, view.token), (&scalar.observation, &scalar.info, cur[k].token), "slot {k} step {step}");
            assert_eq!(in_combat[k], scalar.info["active"] == Value::Bool(true));
            saw_combat[k] |= in_combat[k];
        }
        if cur.iter().any(|b| b.terminal.is_some()) { break }
        let actions: Vec<_> = (0..n).map(|k| RunActionIndex(r1_index(seeds[k], step, &sb[k].observation.action_mask) as i64)).collect();
        let rows = batch.step(&cur.iter().map(|b| b.token).collect::<Vec<_>>(), &actions).unwrap();
        for k in 0..n { sb[k] = singles[k].step(sb[k].token, actions[k]).unwrap().boundary; cur[k] = rows[k].boundary.clone(); }
    }
    assert!(saw_combat.iter().any(|x| *x), "some slot reached combat");
    // policy-v3
    let m10 = spire_env::RunSessionConfig { character: Character::Ironclad, ascension: 0, partition: Partition::Test,
        boss_history: spire_env::M9BossHistory::AwakenedUnseen, capacities: spire_env::RunCapacities::m10_default(), record_replay: false };
    let mut policy = RunSessionBatch::new_run(vec![m10.clone(); 2], RunSession::new_m10_policy, config(1, AutoresetMode::NextStep)).unwrap();
    policy.reset(&[Some(5), Some(6)], &[None, None], None).unwrap();
    let (views, _) = policy.combat_views().unwrap();
    let mut s = RunSession::new_m10_policy(m10.clone()).unwrap();
    s.reset(Some(6)).unwrap();
    assert_eq!(views[1].as_ref().unwrap().observation, s.combat_view().unwrap().observation);
    // Captured profiles are Unsupported.
    let mut m12 = RunSessionBatch::new_run(vec![spire_env::RunSessionConfig { character: Character::Silent, boss_history: spire_env::M9BossHistory::Act1GuardianUnseen, ..m10 }], RunSession::new_m12, config(1, AutoresetMode::Disabled)).unwrap();
    m12.reset(&[Some(1)], &[None], None).unwrap();
    assert!(matches!(m12.combat_views(), Err(SessionBatchError::Input { error: RunSessionError::UnsupportedBoundary(_), .. })));
}
