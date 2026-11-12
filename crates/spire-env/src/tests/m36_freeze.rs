//! M36 freeze (design §3.8, F-a): the semantic episode digest (I3) and the pinned tables. A pin may change only through
//! the change protocol in `docs/delivery.md` ("Freeze and change protocol"): bump the affected rules id, re-pin here and
//! in `m36_heart.rs`, update the pre-existing pins and add a `CHANGES.md` entry.
use super::m35_r2::r1;
use crate::*;
use sha2::{Digest, Sha256};
use spire_core::{combat::rng::Partition, content::Character};

/// Canonical bytes of one public observation: each group in published (descriptor) order as key, shape, values (i64 LE),
/// `known`, the occupied flag and rows; then the action mask. Integers only; no record bytes.
pub(super) fn hash_run_observation(h: &mut Sha256, o: &RunObservation) {
    for g in &o.groups {
        h.update(g.key.as_bytes()); h.update([0]);
        h.update((g.shape.len() as u64).to_le_bytes());
        for d in &g.shape { h.update((*d as u64).to_le_bytes()); }
        for v in &g.values { h.update(v.to_le_bytes()); }
        h.update(g.known.iter().map(|k| *k as u8).collect::<Vec<_>>());
        match &g.occupied { None => h.update([0]), Some(rows) => { h.update([1]); h.update(rows.iter().map(|r| *r as u8).collect::<Vec<_>>()); } }
    }
    h.update(b"action_mask"); h.update(o.action_mask.iter().map(|m| *m as u8).collect::<Vec<_>>());
}

/// The per-episode semantic digest: for every accepted step, the boundary the policy acted on, then the step tuple
/// `(action index, reward, terminal)`; finally the terminal (or last) boundary. Steps, terminal and return are pinned beside it.
pub(super) struct EpisodeDigest { h: Sha256, pub(super) steps: usize, pub(super) ret: i64 }
impl EpisodeDigest {
    pub(super) fn new(surface: &str) -> Self { let mut h = Sha256::new(); h.update(surface.as_bytes()); h.update([0]); Self { h, steps: 0, ret: 0 } }
    pub(super) fn boundary(&mut self, o: &RunObservation) { self.h.update(b"B"); hash_run_observation(&mut self.h, o); }
    /// Raw tensor bytes (legacy combat-v1 rows), in descriptor order.
    pub(super) fn bytes(&mut self, tag: &str, bytes: &[u8]) { self.h.update(b"T"); self.h.update(tag.as_bytes()); self.h.update([0]); self.h.update((bytes.len() as u64).to_le_bytes()); self.h.update(bytes); }
    pub(super) fn step(&mut self, action: i64, reward: i64, terminal: &str) {
        self.h.update(b"S"); self.h.update(action.to_le_bytes()); self.h.update(reward.to_le_bytes()); self.h.update(terminal.as_bytes()); self.h.update([0]);
        self.steps += 1; self.ret += reward;
    }
    /// combat-v1 rewards are f32: the tuple carries the bit pattern; the return accumulates milli-reward units.
    pub(super) fn step_f32(&mut self, action: i64, reward: f32, terminal: &str) {
        self.h.update(b"S"); self.h.update(action.to_le_bytes()); self.h.update(reward.to_bits().to_le_bytes()); self.h.update(terminal.as_bytes()); self.h.update([0]);
        self.steps += 1; self.ret += (f64::from(reward) * 1000.0).round() as i64;
    }
    pub(super) fn finish(self) -> String { self.h.finalize().iter().map(|b| format!("{b:02x}")).collect() }
}

/// One pinned episode: `(name, seed, steps, terminal, return, semantic digest)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Pin { pub name: &'static str, pub seed: u64, pub steps: usize, pub terminal: &'static str, pub ret: i64, pub digest: &'static str }

/// The 8 from-start Heart routes, asserted inside `m36_heart::m36_heart_acceptance` (I2, Mn5).
pub(super) const HEART_PINS: &[Pin] = &[
    Pin { name: "Ironclad-A0", seed: 0xb0, steps: 326, terminal: "HeartVictory", ret: 200, digest: "5f31c0cd66674f0f0fca44bf3fb3f176d056c00e2f86570b6d850a680a69e9b8" },
    Pin { name: "Ironclad-A20", seed: 0xb8, steps: 345, terminal: "HeartVictory", ret: 201, digest: "7121467e9138a6b270a403c2bb7b541dd58379cbf3f32d385238b74a825483ed" },
    Pin { name: "Silent-A0", seed: 0xb1, steps: 331, terminal: "HeartVictory", ret: 200, digest: "07700971c8d2854c82a90805a4a8b6e76f2ea1ce9b8a24c04415910533428c70" },
    Pin { name: "Silent-A20", seed: 0xb9, steps: 333, terminal: "HeartVictory", ret: 201, digest: "6485e256ffbcc50a17ca6a69b36122aa0f9c2cebb8b910b6e4d07aa0021ea791" },
    Pin { name: "Defect-A0", seed: 0xb2, steps: 309, terminal: "HeartVictory", ret: 200, digest: "70adf7fcf5394c2e26e449ae367a45d14b813050c7499c4ec75e69dbdf757a87" },
    Pin { name: "Defect-A20", seed: 0xba, steps: 329, terminal: "HeartVictory", ret: 201, digest: "645ef434ffcff74f9944fb7737f8b7d991d31a6c161688b63f9963a54975ab00" },
    Pin { name: "Watcher-A0", seed: 0xb3, steps: 351, terminal: "HeartVictory", ret: 200, digest: "5a939bebfb8713da7c29084cf415f7098c21443dcff62e2b09e9e848cb7ec6a9" },
    Pin { name: "Watcher-A20", seed: 0xbb, steps: 419, terminal: "HeartVictory", ret: 201, digest: "01521e81d75e3d05cbd93beb93cc15dddbd631c28132b41d47bc612089d15907" },
];

fn hex(bytes: [u8; 32]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
const CHANGE_PROTOCOL: &str = "a frozen identity changed: follow the change protocol in docs/delivery.md (\"Freeze and change protocol\"): \
bump the affected rules/schema id, re-pin m36_freeze.rs and the Heart pins, update the pre-existing pins \
(src/tests/m34s_records.rs:29-31, tests/run_m25_schema.rs:49-74, tests/card_metadata.rs:15-35, src/tests/standard_vocab*) and add a CHANGES.md entry";

/// F-a (design §3.8, I4): every frozen surface's ids, capacity preset and descriptor fingerprints.
#[test]
fn frozen_identities() {
    let fp = |spec: Result<RunObservationSpec, RunSessionError>| hex(spec.unwrap().fingerprint());
    let standard_caps = RunCapacities::standard_default();
    let metadata = card_metadata_descriptor();
    let actual: Vec<(&str, String)> = vec![
        // Standard family
        ("standard.observation_schema", RUN_OBSERVATION_STANDARD_SCHEMA_ID.into()),
        ("standard.observation_fingerprint", fp(standard_run_observation_spec(&standard_caps))),
        ("standard.rules", crate::run_session::RUN_RULES_ID_STANDARD.into()),
        ("standard.content", crate::run_session::RUN_CONTENT_ID_STANDARD.into()),
        ("standard.record", crate::run_session::RUN_RECORD_SCHEMA_ID_STANDARD.into()),
        ("standard.reward", crate::run_session::RUN_REWARD_ID.into()),
        ("standard.seed_adapter", RUN_SEED_ADAPTER_ID.into()),
        ("standard.rng", spire_core::combat::rng::RNG_CONTRACT_ID.into()),
        ("standard.capacity_preset", crate::run_schema_standard::STANDARD_CAPACITY_PRESET_ID.into()),
        ("standard.vocabulary_manifest", crate::run_schema_standard::STANDARD_VOCAB_MANIFEST_ID.into()),
        ("standard.combat_view", crate::combat_policy::STANDARD_COMBAT_VIEW_ID.into()),
        ("standard.combat_view_fingerprint", fp(crate::combat_policy::standard_combat_observation_spec(&standard_caps))),
        ("standard.combat_profile", STANDARD_COMBAT_PROFILE_ID.into()),
        ("standard.combat_catalog_schema", STANDARD_COMBAT_CATALOG_SCHEMA_ID.into()),
        ("standard.combat_reward", COMBAT_TRAINING_REWARD_ID.into()),
        ("standard.position", spire_core::combat::position::POSITION_SCHEMA_ID.into()),
        ("standard.belief", PUBLIC_BELIEF_ID.into()),
        ("standard.search", PUBLIC_SEARCH_ID.into()),
        ("standard.knowledge", PUBLIC_KNOWLEDGE_ID.into()),
        // M38e opt-in compact Standard observation (frozen from the M38e merge)
        ("standard.compact_observation_schema", RUN_OBSERVATION_STANDARD_COMPACT_SCHEMA_ID.into()),
        ("standard.compact_observation_fingerprint", fp(standard_compact_run_observation_spec(&RunCapacities::standard_compact()))),
        ("standard.compact_capacity_preset", STANDARD_COMPACT_CAPACITY_PRESET_ID.into()),
        // Card metadata
        ("card_metadata.schema", CARD_METADATA_SCHEMA_ID.into()),
        ("card_metadata.fingerprint", metadata["fingerprint"].as_str().unwrap().into()),
        // M9 Run-v1
        ("m9.observation_schema", RUN_OBSERVATION_SCHEMA_ID.into()),
        ("m9.observation_fingerprint", fp(run_observation_spec(&RunCapacities::m9_default()))),
        ("m9.rules", RUN_RULES_ID.into()),
        ("m9.content", crate::run_session::RUN_CONTENT_ID.into()),
        ("m9.record", RUN_RECORD_SCHEMA_ID.into()),
        // M10 Heart Run-v2
        ("m10.observation_schema", RUN_OBSERVATION_M10_SCHEMA_ID.into()),
        ("m10.observation_fingerprint", fp(m10_run_observation_spec(&RunCapacities::m10_default()))),
        ("m10.rules", RUN_RULES_ID_M10.into()),
        ("m10.content", RUN_CONTENT_ID_M10.into()),
        ("m10.record", RUN_RECORD_SCHEMA_ID_M10.into()),
        // M10 policy Run-v3 and the Gym66 views (Gym66 reports the M10 rules id)
        ("m10_policy.observation_schema", RUN_OBSERVATION_POLICY_SCHEMA_ID.into()),
        ("m10_policy.observation_fingerprint", fp(policy_run_observation_spec(&RunCapacities::m10_default()))),
        ("m10_policy.record", RUN_RECORD_SCHEMA_ID_POLICY.into()),
        ("gym66.view_v1", COMBAT_POLICY_VIEW_ID.into()),
        ("gym66.view_v1_fingerprint", fp(combat_policy_observation_spec())),
        ("gym66.view_v2", COMBAT_POLICY_VIEW_V2_ID.into()),
        ("gym66.view_v2_fingerprint", fp(combat_policy_observation_spec_v2())),
        ("gym66.profile_v1", COMBAT_TRAINING_PROFILE_ID.into()),
        ("gym66.profile_v2", COMBAT_TRAINING_PROFILE_V2_ID.into()),
        ("gym66.content_v1", COMBAT_TRAINING_CONTENT_ID.into()),
        ("gym66.content_v2", COMBAT_TRAINING_CONTENT_V2_ID.into()),
        ("gym66.reward", COMBAT_TRAINING_REWARD_ID.into()),
        // Legacy combat-v1 (no rules id: CHANGES.md entry and re-pin)
        ("combat_v1.observation_schema", OBSERVATION_SCHEMA_ID.into()),
        ("combat_v1.observation_fingerprint", hex(observation_spec().fingerprint())),
        ("combat_v1.record", RECORD_SCHEMA_ID.into()),
        ("combat_v1.seed_adapter", SEED_ADAPTER_ID.into()),
        ("combat_v1.content", spire_core::combat::CONTENT_ID.into()),
    ];
    let actual: Vec<(&str, &str)> = actual.iter().map(|(k, v)| (*k, v.as_str())).collect();
    assert_eq!(actual, FROZEN_IDENTITIES, "{CHANGE_PROTOCOL}");
}

/// Values at base 09b8fc81 + M36 (each fingerprint equals its pre-existing pin; Mn4).
pub(super) const FROZEN_IDENTITIES: &[(&str, &str)] = &[
    ("standard.observation_schema", "spire-gym-standard-run-observation-v1"),
    ("standard.observation_fingerprint", "bf8e1d111bbf2171594231b99d8d6b701244249abfbb90a11e86c4b4a405f7ae"),
    ("standard.rules", "spire-gym-native-standard-rules-v3"),
    ("standard.content", "spire-gym-standard-content-v1"),
    ("standard.record", "spire-gym-standard-run-replay-v1"),
    ("standard.reward", "spire-gym-floor-key-terminal-reward-v1"),
    ("standard.seed_adapter", "spire-gym-run-u64-le-v1"),
    ("standard.rng", "spire-lab-rng-hmac-sha256-v1"),
    ("standard.capacity_preset", "standard-default-v1"),
    ("standard.vocabulary_manifest", "standard-vocab-v1"),
    ("standard.combat_view", "spire-gym-standard-combat-view-v1"),
    ("standard.combat_view_fingerprint", "5d8147e13ded030c7faa501a8202eb2588915c2e1330b30cac5b06afc9f9916e"),
    ("standard.combat_profile", "standard-combat-v1"),
    ("standard.combat_catalog_schema", "spire-gym-standard-combat-catalog-v1"),
    ("standard.combat_reward", "spire-gym-combat-outcome-reward-v1"),
    ("standard.position", "spire-gym-combat-position-v1"),
    ("standard.belief", "spire-gym-public-belief-v1"),
    ("standard.search", "spire-gym-public-search-v1"),
    ("standard.knowledge", "spire-gym-public-knowledge-v1"),
    ("standard.compact_observation_schema", "spire-gym-standard-run-observation-compact-v1"),
    ("standard.compact_observation_fingerprint", "633b9424be5070a2890b01725949447badba919c5c8c5bd798922f8c2d2a2b2d"),
    ("standard.compact_capacity_preset", "standard-compact-v1"),
    ("card_metadata.schema", "spire-lab-card-metadata-v1"),
    ("card_metadata.fingerprint", "effbb810d345e906f6e20b47bd972e35d72ef48141378d54bd28b54afac5f406"),
    ("m9.observation_schema", "spire-gym-run-observation-v1"),
    ("m9.observation_fingerprint", "270163f700c7ca19556898d14c22ca8d641b0ef6f5c757d80d0ca7983989981c"),
    ("m9.rules", "spire-gym-native-m9-rules-v2"),
    ("m9.content", "spire-gym-m9-ironclad-a0-captured-v1"),
    ("m9.record", "spire-gym-run-replay-v1"),
    ("m10.observation_schema", "spire-gym-run-observation-v2"),
    ("m10.observation_fingerprint", "dccc7624c45898bc33e3fa682b620fd78ce70fe558d6db7624a888a7ed068c21"),
    ("m10.rules", "spire-gym-native-m10-rules-v1"),
    ("m10.content", "spire-gym-m10-ironclad-a0-captured-v1"),
    ("m10.record", "spire-gym-run-replay-v2"),
    ("m10_policy.observation_schema", "spire-gym-run-observation-v3"),
    ("m10_policy.observation_fingerprint", "28d92d89c437eea7fdc260240585bca466dee1fd74b9b3eb38513d2b34b0be64"),
    ("m10_policy.record", "spire-gym-run-replay-v3"),
    ("gym66.view_v1", "spire-gym-combat-policy-view-v1"),
    ("gym66.view_v1_fingerprint", "4c73f7de04d7b2875f473f8034c1cbacf984ea2d52d0a9428289c30cf1dff463"),
    ("gym66.view_v2", "spire-gym-combat-policy-view-v2"),
    ("gym66.view_v2_fingerprint", "5ba729fd1b10a067fb8fc5d777d78b47fc9526ec0af0030d6868a32b52a7824b"),
    ("gym66.profile_v1", "ironclad-combat-v1"),
    ("gym66.profile_v2", "ironclad-combat-v2"),
    ("gym66.content_v1", "spire-gym-ironclad-combat-content-v1"),
    ("gym66.content_v2", "spire-gym-ironclad-combat-content-v2"),
    ("gym66.reward", "spire-gym-combat-outcome-reward-v1"),
    ("combat_v1.observation_schema", "spire-lab-combat-observation-v1"),
    ("combat_v1.observation_fingerprint", "8fd66d212259d2bdd27139988a772e1a6c223f8d65524f38c74174a0d0843357"),
    ("combat_v1.record", "spire-lab-combat-reproduction-v1"),
    ("combat_v1.seed_adapter", "spire-lab-env-seed-sha256-v1"),
    ("combat_v1.content", "spire-lab-combat-reference-v2"),
];

fn run_terminal(t: Option<spire_core::run::RunTerminal>) -> String { t.map(|t| format!("{t:?}")).unwrap_or_else(|| "none".to_owned()) }
/// One `RunSession` episode under the R1 policy from `reset(Some(seed))`, to its terminal or `cap` steps.
fn run_episode(surface: &str, name: String, mut session: RunSession, seed: u64, cap: usize) -> (String, u64, usize, String, i64, String) {
    let mut d = EpisodeDigest::new(surface);
    let mut b = session.reset(Some(seed)).unwrap();
    let mut step = 0u64;
    while b.terminal.is_none() && d.steps < cap {
        step += 1;
        let index = r1(seed, step, &b.observation.action_mask);
        d.boundary(&b.observation);
        let r = session.step(b.token, RunActionIndex(index as i64)).unwrap();
        d.step(index as i64, i64::from(r.reward), &run_terminal(r.boundary.terminal));
        b = r.boundary;
    }
    d.boundary(&b.observation);
    let (steps, ret) = (d.steps, d.ret);
    (name, seed, steps, run_terminal(b.terminal), ret, d.finish())
}
fn gym66_episode(profile: &str, entry: &str, seed: u64) -> (String, u64, usize, String, i64, String) {
    // Owned gold is a v2 field.
    let (g0, g5) = if profile == COMBAT_TRAINING_PROFILE_V2_ID { (r#""gold":0,"#, r#""gold":5,"#) } else { ("", "") };
    let catalog = format!(r#"{{"profile_id":"{profile}","entries":[{{"id":"a","encounter":"cultist","hp":80,"max_hp":80,{g0}"deck":[{{"card":"strike","upgrades":0}},{{"card":"strike","upgrades":0}},{{"card":"defend","upgrades":0}},{{"card":"bash","upgrades":0}}],"relics":["burning_blood"],"potions":[null,null,null]}},{{"id":"b","encounter":"jaw_worm","hp":70,"max_hp":80,{g5}"deck":[{{"card":"strike","upgrades":0}},{{"card":"defend","upgrades":0}},{{"card":"anger","upgrades":1}}],"relics":[],"potions":[null,null,null]}}]}}"#);
    let mut session = CombatTrainingSession::new(CombatTrainingCatalog::from_json(catalog.as_bytes()).unwrap(), Partition::Test).unwrap();
    let mut d = EpisodeDigest::new(profile);
    let mut b = session.reset(Some(seed), Some(entry)).unwrap();
    let (mut step, mut end) = (0u64, "none");
    while end == "none" && d.steps < 400 {
        step += 1;
        let index = r1(seed, step, &b.observation.action_mask);
        d.boundary(&b.observation);
        let r = session.step(b.token, RunActionIndex(index as i64)).unwrap();
        end = if r.terminated { "terminated" } else if r.truncated { "truncated" } else { "none" };
        d.step(index as i64, i64::from(r.reward), end);
        b = r.boundary;
    }
    d.boundary(&b.observation);
    let (steps, ret) = (d.steps, d.ret);
    (format!("{profile}/{entry}"), seed, steps, end.to_owned(), ret, d.finish())
}
fn combat_v1_rows(d: &mut EpisodeDigest, rows: &DecisionRows) {
    for t in &rows.observations.tensors {
        let bytes: Vec<u8> = match &t.values { TensorValues::I32(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(), TensorValues::I8(v) => v.iter().map(|x| *x as u8).collect() };
        d.bytes(t.key, &bytes);
    }
}
fn combat_v1_episode(seed_byte: u8) -> (String, u64, usize, String, i64, String) {
    let mut t = super::support::template();
    t.hand = vec![CardSpec { kind: spire_core::combat::CardKind::Strike, upgraded: false }; 5];
    let pool = CombatPool::new(std::sync::Arc::new(ScenarioCatalog::new(vec![t]).unwrap()), PoolConfig { batch_size: 1, workers: 1, partition: Partition::Test, ..PoolConfig::default() }).unwrap();
    let mut d = EpisodeDigest::new(OBSERVATION_SCHEMA_ID);
    let mut rows = pool.reset_batch(&[0], &[ResetRequest { seed: Some(IntegerSeed::from_be_bytes(&[0x36, seed_byte]).unwrap()), scenario_id: None }]).unwrap().decisions;
    let (mut step, mut end) = (0u64, "none");
    while end == "none" && d.steps < 400 {
        step += 1;
        let mask = rows.observations.tensors.iter().find(|t| t.key == "action_mask").unwrap();
        let TensorValues::I8(mask) = &mask.values else { panic!("action_mask is i8") };
        let index = r1(u64::from(seed_byte), step, mask);
        combat_v1_rows(&mut d, &rows);
        let r = pool.step_batch(&[index as i64], &rows.tokens).unwrap();
        end = if r.terminated[0] { "terminated" } else if r.truncated[0] { "truncated" } else { "none" };
        d.step_f32(index as i64, r.rewards[0], end);
        rows = if end == "none" { r.current } else { r.finals };
    }
    combat_v1_rows(&mut d, &rows);
    let (steps, ret) = (d.steps, d.ret);
    pool.close().unwrap();
    (format!("combat-v1/{seed_byte}"), u64::from(seed_byte), steps, end.to_owned(), ret, d.finish())
}

/// F-a (design §3.8, I3): per-episode semantic digests over the Standard corpus (4 characters × {A0, A10, A20} × 2 seeds,
/// default creation, R1 policy, to the terminal or 3,000 steps) and short episodes on M9 Run-v1, M10 Heart Run-v2,
/// M10 policy Run-v3, Gym66 v1/v2 and combat-v1. The 8 Heart routes are pinned in `m36_heart_acceptance`.
#[test]
fn frozen_replay_digests() {
    let mut actual = Vec::new();
    for character in super::standard_traversal::CHARACTERS {
        for ascension in [0u8, 10, 20] {
            for seed in [0x36_01u64, 0x36_02] {
                let session = RunSession::new_standard(StandardRunConfig::new(character, ascension, Partition::Test)).unwrap();
                actual.push(run_episode("standard", format!("standard/{character:?}/A{ascension}"), session, seed, 3_000));
            }
        }
    }
    let legacy = |h: M9BossHistory, caps: RunCapacities| RunSessionConfig { character: Character::Ironclad, ascension: 0, partition: Partition::Test, boss_history: h, capacities: caps, record_replay: false };
    for seed in [61u64, 62] {
        actual.push(run_episode("m9-run-v1", "m9".into(), RunSession::new(legacy(M9BossHistory::TimeEaterUnseen, RunCapacities::m9_default())).unwrap(), seed, 600));
        actual.push(run_episode("m10-heart-run-v2", "m10".into(), RunSession::new_m10(legacy(M9BossHistory::AwakenedUnseen, RunCapacities::m10_default())).unwrap(), seed, 600));
        actual.push(run_episode("m10-policy-run-v3", "m10-policy".into(), RunSession::new_m10_policy(legacy(M9BossHistory::DonuDecaUnseen, RunCapacities::m10_default())).unwrap(), seed, 600));
    }
    for profile in [COMBAT_TRAINING_PROFILE_ID, COMBAT_TRAINING_PROFILE_V2_ID] {
        actual.push(gym66_episode(profile, "a", 71));
        actual.push(gym66_episode(profile, "b", 72));
    }
    actual.push(combat_v1_episode(1));
    actual.push(combat_v1_episode(2));
    let pins: Vec<String> = actual.iter().map(|(name, seed, steps, terminal, ret, digest)|
        format!("    (\"{name}\", {seed:#x}, {steps}, \"{terminal}\", {ret}, \"{digest}\"),")).collect();
    eprintln!("frozen_replay_digests:\n{}", pins.join("\n"));
    let actual: Vec<(&str, u64, usize, &str, i64, &str)> = actual.iter().map(|(n, s, k, t, r, d)| (n.as_str(), *s, *k, t.as_str(), *r, d.as_str())).collect();
    assert_eq!(actual, FROZEN_REPLAY_DIGESTS, "{CHANGE_PROTOCOL}");
}

/// `(episode, seed, steps, terminal, return, semantic digest)`; computed on the M36 candidate (base 09b8fc81). The
/// combat-v1 return is in milli-reward units.
pub(super) const FROZEN_REPLAY_DIGESTS: &[(&str, u64, usize, &str, i64, &str)] = &[
    ("standard/Ironclad/A0", 0x3601, 65, "Death", 3, "a967c7928bf7cc930a52fb3e1bae7ac0a0069832d5296680871109fa3838573f"),
    ("standard/Ironclad/A0", 0x3602, 56, "Death", 1, "70ea64026eaa9d9a1f6d99bc8395ae06ecdb14342a5c9ded132b87d987eea7b1"),
    ("standard/Ironclad/A10", 0x3601, 54, "Death", 3, "ffeacaba5c25e8a807cacc28893527a2931be1fd61db631d69cd779d21126593"),
    ("standard/Ironclad/A10", 0x3602, 26, "Death", 0, "5285684b1dcb9e7fd92a83eaea01f239ed06a0d69ce4c3b2cc3ddf755993d542"),
    ("standard/Ironclad/A20", 0x3601, 54, "Death", 3, "531a097d64a832f09170cc32b8b3c65d32a63755f01cca4b4737e5b05f6d2eb1"),
    ("standard/Ironclad/A20", 0x3602, 26, "Death", 0, "cddedfaead5298f32c008c2e54d34f3a8dcbf109027de6f448322a8f51892b56"),
    ("standard/Silent/A0", 0x3601, 95, "Death", 4, "93a7500aefb526953a6ad93bca3dcdeee7f66d96e3a2b70ce13e4e75a01df8e0"),
    ("standard/Silent/A0", 0x3602, 59, "Death", 1, "2d9bddfbb33171d5ee33f240e72a0b0ca16fc0d84b3d1c86fe433922dc51754c"),
    ("standard/Silent/A10", 0x3601, 45, "Death", 2, "8b0514780e3a92a78fc23acb2bb55d729c7f408056ef0de0eda4025735ed86b4"),
    ("standard/Silent/A10", 0x3602, 28, "Death", 0, "02f9c4df8b3324ec5619d1dfdb945c8adcd2aafb8a2d2c9e7371c8c169cb8d09"),
    ("standard/Silent/A20", 0x3601, 41, "Death", 2, "51c1b9cb8335f28f246cbde2d6c6199fb57a027d486574d321caa4024ef36741"),
    ("standard/Silent/A20", 0x3602, 28, "Death", 0, "4dfbf10a60c9348a93a94db49adadddccb6bb32b01959bef70dbf4606fbc00e4"),
    ("standard/Defect/A0", 0x3601, 45, "Death", 2, "114e85e0bf4a51b6648d30c6584def35325444edd8aa8e5826f22f000c5aaeeb"),
    ("standard/Defect/A0", 0x3602, 86, "Death", 6, "b6a5df4774aacb33e038114789cb17ea7db8b89945f1e85b54cb9e7fe7e9b172"),
    ("standard/Defect/A10", 0x3601, 61, "Death", 3, "94a6ff98df770b51c995fa8efd30665211bfec9174e40e75590d80b4afdd1c2f"),
    ("standard/Defect/A10", 0x3602, 36, "Death", 0, "594eb3f0c532b1b9e34a9c3711b7f9b7671da7e57b7fdebc1c297cf8a056143d"),
    ("standard/Defect/A20", 0x3601, 45, "Death", 2, "22c8f06c493521880c892999a6b8690167092c8875cc96edec3116ff0379d2b1"),
    ("standard/Defect/A20", 0x3602, 28, "Death", 0, "68b2cd2479653656df807397bc6a28eccf015f2476d36f5431d6f452c46ce5d6"),
    ("standard/Watcher/A0", 0x3601, 41, "Death", 2, "6dfa92cce8240c4fcec763aa108f3f2aa722a080af6c026c3b9755f2f2611f52"),
    ("standard/Watcher/A0", 0x3602, 26, "Death", 0, "54a1f3766283c810f6e8c5cdee7a9bed26370304f573fded4b77cb1e60231e7b"),
    ("standard/Watcher/A10", 0x3601, 11, "Death", 0, "1d260ca5429d9028613d987e54a89660f9cb91b68c8e6835d6049b899650cdf1"),
    ("standard/Watcher/A10", 0x3602, 28, "Death", 0, "6d7a2a51bd6a941a582130873fd68ec9c2472f7736a423895f71afdd5121d590"),
    ("standard/Watcher/A20", 0x3601, 11, "Death", 0, "16962d7a0a65359d392c31dbd96f4a30f3cdefbdd59f6f7a439ff391ecda1d56"),
    ("standard/Watcher/A20", 0x3602, 26, "Death", 0, "00d3987e09e629061a6b7b70a39f4543ea8f48166059ab859a5886b4a9c25573"),
    ("m9", 0x3d, 90, "Death", 11, "1b934716df37dbb533f4d365845a3ca8060f9fe61ae1f265da48286f47f80fc5"),
    ("m10", 0x3d, 36, "Death", 1, "09f1895faace4b8a09e588a5f898bf2777564dad3585941c50e913720f28ff8e"),
    ("m10-policy", 0x3d, 36, "Death", 1, "5a6143ff45e970b64b362527fc0099760bee9c650cbad8dac2fb15c082fc4a6b"),
    ("m9", 0x3e, 60, "Death", 12, "ddd2b08119d85fc3c68a1c7d0048681fe048630edf56e37dba89ad6eebc3f343"),
    ("m10", 0x3e, 45, "Death", 6, "bce35b68609f71c17a12df3ca6f8d0edc9494cdf1193751f15e802d21d81915d"),
    ("m10-policy", 0x3e, 45, "Death", 6, "0b79c8768bd4084ae55c12c67783d937c13c22d20d22539df24b41fa5c0c4b46"),
    ("ironclad-combat-v1/a", 0x47, 14, "terminated", 1, "94c4cb9294ee578674c89356ec4a27b64320e9cdc6bc4aa553a61604a01be754"),
    ("ironclad-combat-v1/b", 0x48, 11, "terminated", 1, "2591ac4cd9bc1741f0bf556ea0bcdc041a2bfdbc57de9840aa20544313863af0"),
    ("ironclad-combat-v2/a", 0x47, 14, "terminated", 1, "34cc39805b048bd1022d92ef6e181db07d2a63c7a41ecfc89e374b5b9938ca70"),
    ("ironclad-combat-v2/b", 0x48, 11, "terminated", 1, "52e525a54f3d97f19a710c3ceffab25f97cfba279691d0a1be6acc963cf97ab5"),
    ("combat-v1/1", 0x1, 13, "terminated", 1000, "8bac07c2b82a0defe113084ce38f0891a4e57452b1e4b2e3e580b057cd78c10c"),
    ("combat-v1/2", 0x2, 12, "terminated", 1000, "4a49dbd30b73bb0dabed2ddae62a7a30aac1afece3c9c46cc626d96b311d030b"),
];
