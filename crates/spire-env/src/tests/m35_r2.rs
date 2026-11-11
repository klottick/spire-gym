//! M35 R2 (design §4): Standard terminals through the batch — supplied HeartVictory routes (TrueVictory final view), a
//! keys-off supplied route (Act3NoKeys) and an A20 R1-policy death — equal the single sessions' terminal boundaries under
//! SAME_STEP (`final_obs`) and NEXT_STEP (next-step reset), at workers 1 and 4 (the worker stack guard); DISABLED rejects
//! the terminal slot. No Standard EmptyDeck route is retained (census B2), so none is checked.
use super::standard_traversal::{needs_supplied_wound, short_creation, simple_action};
use crate::{AutoresetMode, BatchConfig, FaultPolicy, RunActionIndex, RunBoundary, RunSession, RunSessionError, SessionBatch, SessionBatchError, StandardRunConfig};
use spire_core::{combat::rng::Partition, content::Character, run::{diagnostics::{standard_supplied_run_with, supplied_wound_enemies}, RunRng, RunTerminal}};

fn supplied(character: Character, ascension: u8, seed: u8, keys: bool) -> RunSession {
    let creation = short_creation();
    let run = standard_supplied_run_with(character, ascension, creation.profile(), 900, keys, RunRng::new([seed; 32], Partition::Test, 0)).unwrap();
    let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
    config.creation = creation;
    config.record_replay = true;
    RunSession::from_standard_supplied(config, run, u64::from(seed)).unwrap()
}
fn death_slot() -> RunSession {
    let mut config = StandardRunConfig::new(Character::Defect, 20, Partition::Test);
    config.creation = short_creation();
    let mut s = RunSession::new_standard(config).unwrap();
    s.reset(Some(DEATH_SEED)).unwrap();
    s
}
const DEATH_SEED: u64 = 0x35_0002;
/// Slot kinds: two HeartVictory routes, a keys-off route, a normal A20 run.
fn slots() -> Vec<RunSession> {
    vec![supplied(Character::Ironclad, 0, 0xB0, true), supplied(Character::Watcher, 20, 0xBB, true), supplied(Character::Silent, 20, 0xB9, false), death_slot()]
}
fn policy(slot: usize, step: u64, b: &RunBoundary) -> usize {
    if slot == 3 { crate::tests::m35_r2::r1(DEATH_SEED, step, &b.observation.action_mask) } else { simple_action(&b.observation) }
}
pub(crate) fn r1(seed: u64, step: u64, mask: &[i8]) -> usize {
    let legal: Vec<usize> = mask.iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, _)| i).collect();
    let mut z = seed.wrapping_add((step + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    legal[(z % legal.len() as u64) as usize]
}

/// Drives every slot to its first terminal through the batch and its single twin; returns each slot's terminal.
fn terminals_through_batch(mode: AutoresetMode, workers: usize) -> Vec<Option<RunTerminal>> {
    let n = 4;
    let mut batch = SessionBatch::build(slots(), vec![None; n], BatchConfig { workers, autoreset: mode, fault_policy: FaultPolicy::Freeze }, true).unwrap();
    batch.adopt_started();
    let mut singles = slots();
    let mut cur: Vec<RunBoundary> = (0..n).map(|k| batch.decision(k).unwrap()).collect();
    let mut sb: Vec<RunBoundary> = singles.iter().map(|s| s.decision().unwrap()).collect();
    let mut done: Vec<Option<RunTerminal>> = vec![None; n];
    let mut pending = vec![false; n];
    for step in 1..=20_000u64 {
        if done.iter().all(Option::is_some) { break }
        for k in 0..n {
            if done[k].is_none() && k != 3 && needs_supplied_wound(&sb[k].observation) {
                sb[k] = singles[k].apply_supplied(supplied_wound_enemies).unwrap();
                cur[k] = batch.sessions[k].apply_supplied(supplied_wound_enemies).unwrap();
            }
            if done[k].is_none() { assert_eq!(cur[k].observation, sb[k].observation, "{mode:?} slot {k} step {step}"); }
        }
        let actions: Vec<_> = (0..n).map(|k| RunActionIndex(if pending[k] { 0 } else { policy(k, step, &cur[k]) as i64 })).collect();
        let rows = batch.step(&cur.iter().map(|b| b.token).collect::<Vec<_>>(), &actions).unwrap();
        for k in 0..n {
            if done[k].is_none() {
                let r = singles[k].step(sb[k].token, actions[k]).unwrap();
                assert_eq!((rows[k].reward, &rows[k].reward_events, rows[k].terminated), (r.reward, &r.reward_events, r.terminated), "{mode:?} slot {k} step {step}");
                if r.terminated {
                    done[k] = r.boundary.terminal;
                    match mode {
                        AutoresetMode::SameStep => {
                            let fin = rows[k].final_boundary.as_ref().expect("final_obs");
                            assert_eq!((&fin.observation, fin.terminal), (&r.boundary.observation, r.boundary.terminal), "{mode:?} slot {k}: final_obs is the single terminal");
                            let reset = singles[k].reset(None).unwrap();
                            assert_eq!(rows[k].boundary.observation, reset.observation, "the row is the next episode's reset");
                        }
                        _ => {
                            assert_eq!((&rows[k].boundary.observation, rows[k].boundary.terminal), (&r.boundary.observation, r.boundary.terminal));
                            pending[k] = true;
                        }
                    }
                } else { sb[k] = r.boundary; }
            } else if pending[k] {
                assert!(rows[k].autoreset);
                if mode == AutoresetMode::NextStep && singles[k].decision().is_ok_and(|b| b.terminal.is_some()) {
                    let reset = singles[k].reset(None).unwrap();
                    assert_eq!(rows[k].boundary.observation, reset.observation, "NEXT_STEP: the reset of the next step");
                }
                pending[k] = false;
            }
            pending[k] = pending[k] || (mode == AutoresetMode::NextStep && rows[k].terminated && done[k].is_some() && rows[k].boundary.terminal.is_some());
            cur[k] = rows[k].boundary.clone();
        }
    }
    done
}

#[test]
fn standard_terminal_views_match_single() {
    for (mode, workers) in [(AutoresetMode::NextStep, 1), (AutoresetMode::SameStep, 4)] {
        let terminals = terminals_through_batch(mode, workers);
        eprintln!("R2 {mode:?} workers {workers}: {terminals:?}");
        assert_eq!(terminals, [Some(RunTerminal::HeartVictory), Some(RunTerminal::HeartVictory), Some(RunTerminal::Act3NoKeys), Some(RunTerminal::Death)]);
    }
}

#[test]
fn disabled_rejects_a_terminal_standard_slot() {
    let mut batch = SessionBatch::build(vec![death_slot()], vec![None], BatchConfig::new(1), true).unwrap();
    batch.adopt_started();
    let mut b = batch.decision(0).unwrap();
    for step in 1..=5_000u64 {
        let rows = batch.step(&[b.token], &[RunActionIndex(r1(DEATH_SEED, step, &b.observation.action_mask) as i64)]).unwrap();
        b = rows[0].boundary.clone();
        if rows[0].terminated {
            assert_eq!(b.terminal, Some(RunTerminal::Death));
            assert_eq!(batch.step(&[b.token], &[RunActionIndex(0)]), Err(SessionBatchError::Input { slot: Some(0), error: RunSessionError::Terminal }));
            return
        }
    }
    panic!("the A20 R1 run must die");
}
