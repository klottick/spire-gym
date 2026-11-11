//! M35 task 5 (R7): stable cross-layer fixtures for the first 200 public batch rows.
mod m35_support;
use m35_support::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{combat::rng::Partition, run::RunTerminal};
use spire_env::{AutoresetMode, BatchConfig, CombatTrainingBatch, FaultPolicy, RunActionIndex, RunSession,
    RunSessionBatch};
use std::{fs, path::PathBuf};

const STEPS: usize = 200;
const FIXTURE: &str = "tests/fixtures/m35_agreement_digests.json";

// R7 frame order, mirrored by the Python lane: for each row and each descriptor-ordered group, append the UTF-8 group
// key as u32-LE byte length + bytes, then rank and dimensions as u32 LE, then group values, known flags and optional
// occupied flags as int64-LE arrays in C order. Append the action mask with key `action_mask`, rank/shape and int64-LE
// values. Then append reward as int64 LE; terminated, truncated and autoreset bytes; terminal reason as u32-LE length
// + UTF-8 (empty for none); and, for combat only, entry_id in that same length-prefixed UTF-8 form. Initial reset rows
// are row 0. Decision tokens and reward_events are intentionally excluded, per design §4 R7.
fn u32le(h: &mut Sha256, n: usize) { h.update(u32::try_from(n).unwrap().to_le_bytes()); }
fn text(h: &mut Sha256, s: &str) { u32le(h, s.len()); h.update(s.as_bytes()); }
fn i64s(h: &mut Sha256, values: impl IntoIterator<Item=i64>) { for v in values { h.update(v.to_le_bytes()); } }
fn obs(h: &mut Sha256, o: &spire_env::RunObservation) {
    for g in &o.groups {
        text(h, g.key);
        u32le(h, g.shape.len()); for d in &g.shape { u32le(h, *d); }
        i64s(h, g.values.iter().copied());
        i64s(h, g.known.iter().map(|v| i64::from(*v)));
        if let Some(occupied) = &g.occupied { i64s(h, occupied.iter().map(|v| i64::from(*v))); }
    }
    text(h, "action_mask"); u32le(h, 1); u32le(h, o.action_mask.len());
    i64s(h, o.action_mask.iter().map(|v| i64::from(*v)));
}
fn digest_row(h: &mut Sha256, o: &spire_env::RunObservation, reward: i32, terminated: bool,
    truncated: bool, autoreset: bool, reason: Option<&str>, entry: Option<&str>) {
    obs(h, o); h.update(i64::from(reward).to_le_bytes());
    h.update([u8::from(terminated), u8::from(truncated), u8::from(autoreset)]);
    text(h, reason.unwrap_or("")); if let Some(id) = entry { text(h, id); }
}
fn hex(d: &[u8]) -> String { d.iter().map(|b| format!("{b:02x}")).collect() }
fn terminal(t: Option<RunTerminal>) -> Option<&'static str> { Some(match t? {
    RunTerminal::Death => "death", RunTerminal::EmptyDeck => "empty_deck", RunTerminal::Act3NoKeys => "act3_no_keys",
    RunTerminal::HeartVictory => "heart_victory",
}) }

fn r1(workers: usize) -> Vec<String> {
    let configs = r1_standard_configs(); let seeds = r1_seeds(8);
    let mut batch = RunSessionBatch::new_run(configs, RunSession::new_standard, BatchConfig {
        workers, autoreset: AutoresetMode::NextStep, fault_policy: FaultPolicy::Freeze,
    }).unwrap();
    let mut current = batch.reset(&seeds.iter().copied().map(Some).collect::<Vec<_>>(), &vec![None;8], None).unwrap()
        .into_iter().map(|r| r.boundary).collect::<Vec<_>>();
    let mut hashes = (0..8).map(|_| Sha256::new()).collect::<Vec<_>>();
    for k in 0..8 { digest_row(&mut hashes[k], &current[k].observation, 0, false, false, false, None, None); }
    for step in 1..STEPS {
        let tokens = current.iter().map(|b| b.token).collect::<Vec<_>>();
        let actions = (0..8).map(|k| RunActionIndex(if current[k].terminal.is_some() { -1 } else { r1_index(seeds[k], step as u64, &current[k].observation.action_mask) as i64 })).collect::<Vec<_>>();
        let rows = batch.step(&tokens, &actions).unwrap();
        for k in 0..8 {
            let reason = terminal(rows[k].boundary.terminal);
            digest_row(&mut hashes[k], &rows[k].boundary.observation, rows[k].reward, rows[k].terminated,
                rows[k].truncated, rows[k].autoreset, reason, None);
            current[k] = rows[k].boundary.clone();
        }
    }
    hashes.into_iter().map(|h| hex(&h.finalize())).collect()
}
fn r4(workers: usize) -> Vec<String> {
    let (catalog, ids) = r4_shared_catalog(); let seeds = (0..8).map(|k| 7100 + k as u64).collect::<Vec<_>>();
    let mut batch = CombatTrainingBatch::new_combat(catalog, Partition::Test, 8, None, BatchConfig {
        workers, autoreset: AutoresetMode::NextStep, fault_policy: FaultPolicy::Freeze,
    }).unwrap();
    let mut current = batch.reset(&seeds.iter().copied().map(Some).collect::<Vec<_>>(),
        &ids.iter().cloned().map(Some).collect::<Vec<_>>(), None).unwrap().into_iter().map(|r|r.boundary).collect::<Vec<_>>();
    let mut hashes = (0..8).map(|_| Sha256::new()).collect::<Vec<_>>();
    for k in 0..8 { digest_row(&mut hashes[k], &current[k].observation, 0, false, false, false,
        current[k].info["terminal_reason"].as_str(), Some(&ids[k])); }
    for step in 1..STEPS {
        let tokens = current.iter().map(|b|b.token).collect::<Vec<_>>();
        let actions = (0..8).map(|k| RunActionIndex(if batch.pending_autoreset(k) { -1 } else {
            r1_index(seeds[k], step as u64, &current[k].observation.action_mask) as i64 })).collect::<Vec<_>>();
        let rows = batch.step(&tokens, &actions).unwrap();
        for k in 0..8 {
            digest_row(&mut hashes[k], &rows[k].boundary.observation, rows[k].reward, rows[k].terminated, rows[k].truncated,
                rows[k].autoreset, rows[k].boundary.info["terminal_reason"].as_str(), rows[k].boundary.info["entry_id"].as_str());
            current[k] = rows[k].boundary.clone();
        }
    }
    hashes.into_iter().map(|h| hex(&h.finalize())).collect()
}

#[test]
fn m35_agreement_digests() {
    let r1_one = r1(1); let r1_four = r1(4); let r4_one = r4(1); let r4_four = r4(4);
    assert_eq!(r1_one, r1_four, "R1 digests are independent of worker count");
    assert_eq!(r4_one, r4_four, "R4 digests are independent of worker count");
    let expected = json!({"schema":"m35-agreement-digests-v1", "steps":STEPS, "corpora":{"r1":r1_one,"r4":r4_one}});
    if std::env::var_os("SPIRE_M35_WRITE_DIGESTS").is_some_and(|v| v == "1") {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE); fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec_pretty(&expected).unwrap()).unwrap();
    } else {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
        let fixture: Value = serde_json::from_slice(&fs::read(path).expect("R7 fixture present; set SPIRE_M35_WRITE_DIGESTS=1 to regenerate")).unwrap();
        assert_eq!(fixture, expected);
    }
}
