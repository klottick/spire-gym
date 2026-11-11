//! M35 task 1 (design §3.1, §4 R1/R6): the shared `SessionBatch` owner agrees with single sessions, for any worker count
//! and slot order, on mixed Standard batches and the finite-profile wrappers.
mod m35_support;
use m35_support::*;
use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    AutoresetMode, BatchConfig, FaultPolicy, M9BossHistory, RunActionIndex, RunBatch, RunBoundary, RunCapacities, RunSession, RunSessionBatch,
    RunSessionConfig, RunSessionError, SessionBatchError, StandardRunConfig, CombatTrainingBatch, CombatTrainingCatalog,
};

/// One slot's trajectory row: observation, reward, reward events, flags, terminal (tokens are owner-scoped).
#[derive(Debug, PartialEq)]
struct Row { observation: spire_env::RunObservation, reward: i32, events: Vec<spire_core::run::RewardItem>, terminated: bool, truncated: bool, terminal: Option<spire_core::run::RunTerminal> }
fn row(b: &RunBoundary, reward: i32, events: &[spire_core::run::RewardItem], terminated: bool, truncated: bool) -> Row {
    Row { observation: b.observation.clone(), reward, events: events.to_vec(), terminated, truncated, terminal: b.terminal }
}

/// The single-session reference: reset with `seed`, then R1 steps; the step after a terminal is `reset(None)` (the
/// NEXT_STEP protocol: the next ordinal of the same seed).
fn single_trajectory(config: StandardRunConfig, seed: u64, steps: u64) -> (Vec<Row>, Option<spire_env::RunEpisodeRecord>) {
    let mut s = RunSession::new_standard(config).unwrap();
    let mut b = s.reset(Some(seed)).unwrap();
    let mut rows = vec![row(&b, 0, &[], false, false)];
    for step in 1..=steps {
        if b.terminal.is_some() { b = s.reset(None).unwrap(); rows.push(row(&b, 0, &[], false, false)); continue }
        let i = r1_index(seed, step, &b.observation.action_mask);
        let r = s.step(b.token, RunActionIndex(i as i64)).unwrap();
        rows.push(row(&r.boundary, r.reward, &r.reward_events, r.terminated, r.truncated));
        b = r.boundary;
    }
    (rows, s.record().unwrap())
}

/// The batch path (NEXT_STEP), rows per config (slot `k` of the batch runs config `order[k]`). Autoreset rows carry the
/// reset boundary with reward 0 and no flags, as the reference's reset row.
fn batch_trajectories(configs: &[StandardRunConfig], seeds: &[u64], order: &[usize], workers: usize, steps: u64) -> (Vec<Vec<Row>>, Vec<Option<spire_env::RunEpisodeRecord>>) {
    let n = configs.len();
    let config = BatchConfig { workers, autoreset: AutoresetMode::NextStep, fault_policy: FaultPolicy::Freeze };
    let mut batch = RunSessionBatch::new_run(order.iter().map(|&k| configs[k].clone()).collect(), RunSession::new_standard, config).unwrap();
    let slot_seeds: Vec<u64> = order.iter().map(|&k| seeds[k]).collect();
    let mut current: Vec<RunBoundary> = batch.reset(&slot_seeds.iter().map(|s| Some(*s)).collect::<Vec<_>>(), &vec![None; n], None).unwrap()
        .into_iter().map(|r| r.boundary).collect();
    let mut rows: Vec<Vec<Row>> = current.iter().map(|b| vec![row(b, 0, &[], false, false)]).collect();
    for step in 1..=steps {
        let tokens: Vec<_> = current.iter().map(|b| b.token).collect();
        // A terminal slot's action is ignored (its step is the autoreset); any index will do.
        let actions: Vec<_> = (0..n).map(|k| RunActionIndex(if current[k].terminal.is_some() { 0 } else { r1_index(slot_seeds[k], step, &current[k].observation.action_mask) as i64 })).collect();
        let out = batch.step(&tokens, &actions).unwrap();
        for (k, r) in out.into_iter().enumerate() {
            assert_eq!(r.autoreset, current[k].terminal.is_some(), "autoreset exactly after a terminal");
            rows[k].push(row(&r.boundary, r.reward, &r.reward_events, r.terminated, r.truncated));
            current[k] = r.boundary;
        }
    }
    let mut by_config: Vec<Option<Vec<Row>>> = (0..n).map(|_| None).collect();
    let mut records: Vec<Option<spire_env::RunEpisodeRecord>> = vec![None; n];
    for (k, r) in rows.into_iter().enumerate() { records[order[k]] = batch.record(k).unwrap(); by_config[order[k]] = Some(r); }
    (by_config.into_iter().map(Option::unwrap).collect(), records)
}

#[test]
fn mixed_standard_batch_matches_single_sessions() {
    let configs = r1_standard_configs();
    let seeds = r1_seeds(configs.len());
    let order: Vec<usize> = (0..configs.len()).collect();
    let (rows, records) = batch_trajectories(&configs, &seeds, &order, 1, 200);
    for k in 0..configs.len() {
        let (single, record) = single_trajectory(configs[k].clone(), seeds[k], 200);
        assert_eq!(rows[k], single, "slot {k} rows differ from its single session");
        assert_eq!(records[k], record, "slot {k} record differs");
    }
}

#[test]
fn worker_count_and_slot_permutation_do_not_change_rows() {
    let configs = r1_standard_configs();
    let seeds = r1_seeds(configs.len());
    let identity: Vec<usize> = (0..configs.len()).collect();
    let permuted = vec![5, 2, 7, 0, 3, 6, 1, 4];
    let reference = batch_trajectories(&configs, &seeds, &identity, 1, 120);
    for (workers, order) in [(4, &identity), (4, &permuted), (1, &permuted), (3, &permuted)] {
        let other = batch_trajectories(&configs, &seeds, order, workers, 120);
        assert_eq!(other.0, reference.0, "rows differ at workers {workers} order {order:?}");
        assert_eq!(other.1, reference.1, "records differ at workers {workers} order {order:?}");
    }
}

#[test]
fn descriptor_mismatch_rejected_before_construction() {
    let mut configs = r1_standard_configs();
    configs[3].capacities.actions += 1;
    let result = RunSessionBatch::new_run(configs, RunSession::new_standard, BatchConfig::new(1));
    assert!(matches!(result, Err(SessionBatchError::Input { slot: Some(3), error: RunSessionError::Input(m) }) if m.contains("one observation descriptor")));
}

#[test]
fn batch_bounds_and_zero_workers_rejected() {
    let one = vec![StandardRunConfig::new(Character::Ironclad, 0, Partition::Test)];
    assert!(matches!(RunSessionBatch::new_run(Vec::<StandardRunConfig>::new(), RunSession::new_standard, BatchConfig::new(1)), Err(SessionBatchError::Input { slot: None, .. })));
    assert!(matches!(RunSessionBatch::new_run(one.clone(), RunSession::new_standard, BatchConfig::new(0)), Err(SessionBatchError::Input { slot: None, .. })));
    let many = vec![one[0].clone(); 257];
    assert!(matches!(RunSessionBatch::new_run(many, RunSession::new_standard, BatchConfig::new(2)), Err(SessionBatchError::Input { slot: None, .. })));
    let max = vec![one[0].clone(); 256];
    assert_eq!(RunSessionBatch::new_run(max, RunSession::new_standard, BatchConfig::new(2)).unwrap().len(), 256);
}

fn m10_config(history: M9BossHistory) -> RunSessionConfig {
    RunSessionConfig { character: Character::Ironclad, ascension: 0, partition: Partition::Test, boss_history: history, capacities: RunCapacities::m10_default(), record_replay: true }
}

#[test]
fn finite_profile_wrappers_unchanged() {
    // R6 natively: the M10 wrapper, the generic owner and single sessions agree (B=4, 150 R1 steps, workers 1 and 2).
    let histories = [M9BossHistory::AwakenedUnseen, M9BossHistory::TimeEaterUnseen, M9BossHistory::DonuDecaUnseen, M9BossHistory::AwakenedUnseen];
    let configs: Vec<_> = histories.iter().map(|h| m10_config(*h)).collect();
    let seeds = [31u64, 32, 33, 34];
    for workers in [1, 2] {
        let mut wrapper = RunBatch::new_m10(configs.clone(), workers).unwrap();
        let mut generic = RunSessionBatch::new_run(configs.clone(), RunSession::new_m10, BatchConfig::new(workers)).unwrap();
        let mut singles: Vec<RunSession> = configs.iter().map(|c| RunSession::new_m10(c.clone()).unwrap()).collect();
        let seeds_opt: Vec<_> = seeds.iter().map(|s| Some(*s)).collect();
        let a = wrapper.reset(&seeds_opt).unwrap();
        let b: Vec<_> = generic.reset(&seeds_opt, &[None, None, None, None], None).unwrap().into_iter().map(|r| r.boundary).collect();
        let c: Vec<_> = singles.iter_mut().zip(&seeds).map(|(s, seed)| s.reset(Some(*seed)).unwrap()).collect();
        let (mut ta, mut tb, mut tc) = (a, b, c);
        for step in 1..=150u64 {
            if tc.iter().any(|b| b.terminal.is_some()) { break }
            let actions: Vec<_> = (0..4).map(|k| RunActionIndex(r1_index(seeds[k], step, &tc[k].observation.action_mask) as i64)).collect();
            for k in 0..4 { assert_eq!(ta[k].observation, tc[k].observation); assert_eq!(tb[k].observation, tc[k].observation); }
            let ra = wrapper.step(&ta.iter().map(|b| b.token).collect::<Vec<_>>(), &actions).unwrap();
            let rb = generic.step(&tb.iter().map(|b| b.token).collect::<Vec<_>>(), &actions).unwrap();
            let rc: Vec<_> = (0..4).map(|k| singles[k].step(tc[k].token, actions[k]).unwrap()).collect();
            for k in 0..4 {
                assert_eq!((ra[k].reward, ra[k].terminated, &ra[k].reward_events), (rc[k].reward, rc[k].terminated, &rc[k].reward_events));
                assert_eq!((rb[k].reward, rb[k].terminated, &rb[k].reward_events), (rc[k].reward, rc[k].terminated, &rc[k].reward_events));
            }
            ta = ra.into_iter().map(|r| r.boundary).collect();
            tb = rb.into_iter().map(|r| r.boundary).collect();
            tc = rc.into_iter().map(|r| r.boundary).collect();
        }
        for k in 0..4 {
            assert_eq!(wrapper.record(k).unwrap(), singles[k].record().unwrap());
            assert_eq!(generic.record(k).unwrap(), singles[k].record().unwrap());
        }
    }
}

const GYM66_V2: &str = r#"{"profile_id":"ironclad-combat-v2","entries":[{"id":"a","encounter":"cultist","hp":80,"max_hp":80,"gold":0,"deck":[{"card":"strike","upgrades":0},{"card":"defend","upgrades":0},{"card":"bash","upgrades":0}],"relics":[],"potions":[null,null,null],"relic_counters":{}},{"id":"b","encounter":"jaw_worm","hp":70,"max_hp":80,"gold":5,"deck":[{"card":"strike","upgrades":0},{"card":"defend","upgrades":0}],"relics":[],"potions":[null,null,null],"relic_counters":{}}]}"#;

#[test]
fn combat_training_catalog_is_shared() {
    let catalog = std::sync::Arc::new(CombatTrainingCatalog::from_json(GYM66_V2.as_bytes()).unwrap());
    let batch = CombatTrainingBatch::new_combat(catalog.clone(), Partition::Test, 6, None, BatchConfig::new(2)).unwrap();
    for slot in 0..6 { assert!(std::sync::Arc::ptr_eq(batch.session(slot).unwrap().shared_catalog(), &catalog)); }
    assert_eq!(std::sync::Arc::strong_count(&catalog), 6 + 1);
}

/// R6 natively (design §4): the M9, M10 Heart and M10 policy-v3 profiles (the Python Run profiles besides Standard) run
/// through the generic owner under NEXT_STEP at workers 1 and 4 and agree row for row with single sessions doing
/// `reset(None)` after a terminal (B=4, 200 R1 steps; records equal).
#[test]
fn r6_python_run_profiles_match_single_sessions() {
    type Constructor = fn(RunSessionConfig) -> Result<RunSession, RunSessionError>;
    let histories = [M9BossHistory::AwakenedUnseen, M9BossHistory::TimeEaterUnseen, M9BossHistory::DonuDecaUnseen, M9BossHistory::AwakenedUnseen];
    let profiles: [(&str, Constructor, fn(M9BossHistory) -> RunSessionConfig); 3] =
        [("m9", RunSession::new, |h| RunSessionConfig { capacities: RunCapacities::m9_default(), ..m10_config(h) }),
         ("m10", RunSession::new_m10, m10_config), ("m10-policy", RunSession::new_m10_policy, m10_config)];
    let seeds = [61u64, 62, 63, 64];
    for (name, constructor, make) in profiles {
        let configs: Vec<_> = histories.iter().map(|h| make(*h)).collect();
        let mut reference = Vec::new();
        let mut reference_records = Vec::new();
        for (k, config) in configs.iter().enumerate() {
            let mut s = constructor(config.clone()).unwrap();
            let mut b = s.reset(Some(seeds[k])).unwrap();
            let mut rows = vec![row(&b, 0, &[], false, false)];
            for step in 1..=200u64 {
                if b.terminal.is_some() { b = s.reset(None).unwrap(); rows.push(row(&b, 0, &[], false, false)); continue }
                let r = s.step(b.token, RunActionIndex(r1_index(seeds[k], step, &b.observation.action_mask) as i64)).unwrap();
                rows.push(row(&r.boundary, r.reward, &r.reward_events, r.terminated, r.truncated));
                b = r.boundary;
            }
            reference.push(rows);
            reference_records.push(s.record().unwrap());
        }
        for workers in [1, 4] {
            let config = BatchConfig { workers, autoreset: AutoresetMode::NextStep, fault_policy: FaultPolicy::Freeze };
            let mut batch = RunSessionBatch::new_run(configs.clone(), constructor, config).unwrap();
            let mut current: Vec<RunBoundary> = batch.reset(&seeds.iter().map(|s| Some(*s)).collect::<Vec<_>>(), &[None, None, None, None], None)
                .unwrap().into_iter().map(|r| r.boundary).collect();
            let mut rows: Vec<Vec<Row>> = current.iter().map(|b| vec![row(b, 0, &[], false, false)]).collect();
            for step in 1..=200u64 {
                let tokens: Vec<_> = current.iter().map(|b| b.token).collect();
                let actions: Vec<_> = (0..4).map(|k| RunActionIndex(if current[k].terminal.is_some() { 0 }
                    else { r1_index(seeds[k], step, &current[k].observation.action_mask) as i64 })).collect();
                for (k, r) in batch.step(&tokens, &actions).unwrap().into_iter().enumerate() {
                    assert_eq!(r.autoreset, current[k].terminal.is_some(), "{name} slot {k} step {step}");
                    rows[k].push(row(&r.boundary, r.reward, &r.reward_events, r.terminated, r.truncated));
                    current[k] = r.boundary;
                }
            }
            for k in 0..4 {
                assert_eq!(rows[k], reference[k], "{name} workers {workers} slot {k}");
                assert_eq!(batch.record(k).unwrap(), reference_records[k], "{name} workers {workers} slot {k} record");
            }
        }
    }
}
