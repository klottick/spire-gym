//! M35 agreement helpers (test-only): the R1 policy (design §4) and the mixed Standard slot set.
#![allow(dead_code)]
use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{RunObservation, StandardRunConfig};

/// The R1 policy: the `splitmix64(seed, step) % legal_count`-th legal index of `mask` (the same function the Python
/// lanes use; `step` counts from 1).
pub fn r1_index(seed: u64, step: u64, mask: &[i8]) -> usize {
    let legal: Vec<usize> = mask.iter().enumerate().filter(|(_, v)| **v != 0).map(|(i, _)| i).collect();
    assert!(!legal.is_empty(), "R1 policy needs a legal action");
    let mut z = seed.wrapping_add((step + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    legal[(z % legal.len() as u64) as usize]
}

/// R1 slots: Ironclad, Silent, Defect, Watcher × {A0, A20}, recording on.
pub fn r1_standard_configs() -> Vec<StandardRunConfig> {
    let mut configs = Vec::new();
    for character in Character::ALL {
        for ascension in [0, 20] {
            let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
            config.record_replay = true;
            configs.push(config);
        }
    }
    configs
}
pub fn r1_seeds(count: usize) -> Vec<u64> { (0..count as u64).map(|i| 9_350_000 + 17 * i).collect() }

/// The comparison key of one observation (every group's values, known flags and occupancy, and the mask).
pub fn same_observation(a: &RunObservation, b: &RunObservation) -> bool { a == b }

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_env::{RunActionIndex, RunSession, CombatTrainingCatalog, STANDARD_COMBAT_CATALOG_SCHEMA_ID, STANDARD_COMBAT_PROFILE_ID};

fn starter_relic(c: Character) -> &'static str {
    match c { Character::Ironclad => "burning_blood", Character::Silent => "ring_of_the_snake", Character::Defect => "cracked_core", Character::Watcher => "pure_water" }
}
/// A `standard-combat-v1` fresh-start entry with starter resources.
pub fn standard_entry(id: &str, character: Character, ascension: u8, encounter: &str) -> Value {
    let deck: Vec<_> = spire_core::content::Catalog::standard().starter_deck(character).unwrap().iter().map(|c| json!({"card": c.as_str(), "upgrades": 0})).collect();
    json!({"id": id, "character": character.as_str(), "ascension": ascension, "encounter": encounter, "hp": 70, "max_hp": 80, "gold": 99,
        "deck": deck, "relics": [starter_relic(character)], "potions": vec![Value::Null; if ascension >= 11 { 2 } else { 3 }]})
}
pub fn sha_hex(bytes: &[u8]) -> String { Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect() }
/// A Standard record (first legal action) reaching its first combat decision: bytes and prefix length.
pub fn record_into_combat(character: Character, ascension: u8, seed: u64) -> (Vec<u8>, usize) {
    let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
    config.record_replay = true;
    let mut s = RunSession::new_standard(config).unwrap();
    let mut b = s.reset(Some(seed)).unwrap();
    let mut steps = 0;
    while s.public_root().is_err() {
        assert!(steps < 400, "no combat decision");
        let i = b.observation.action_mask.iter().position(|&v| v != 0).unwrap();
        b = s.step(b.token, RunActionIndex(i as i64)).unwrap().boundary;
        steps += 1;
    }
    (s.record().unwrap().unwrap().to_json().unwrap(), steps)
}
/// R4: 4 characters × {A0, A20} fresh starts, with the A0 Defect entry a `synthetic_position` and the A20 Watcher entry a
/// `reachable_recipe` (Watcher A20 Standard record into its first combat). Returns the catalog, records and entry ids.
pub fn r4_catalog() -> (Vec<u8>, Vec<Vec<u8>>, Vec<String>) {
    let mut entries = Vec::new();
    let mut ids = Vec::new();
    let encounters = ["cultist", "jaw_worm"];
    let (record, steps) = record_into_combat(Character::Watcher, 20, 4242);
    for (c, character) in Character::ALL.into_iter().enumerate() {
        for (a, ascension) in [0u8, 20].into_iter().enumerate() {
            let id = format!("{}_a{}", character.as_str(), ascension);
            let mut entry = standard_entry(&id, character, ascension, encounters[(c + a) % 2]);
            if character == Character::Defect && ascension == 0 {
                entry["position"] = json!({"player": {"powers": {"focus": 1}, "orb_slots": 3, "orbs": ["frost", "lightning"]}});
            }
            if character == Character::Watcher && ascension == 20 {
                entry = json!({"id": id, "recipe": {"record_sha256": sha_hex(&record), "steps": steps}});
            }
            entries.push(entry);
            ids.push(id);
        }
    }
    let catalog = serde_json::to_vec(&json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": entries})).unwrap();
    (catalog, vec![record], ids)
}
pub fn r4_shared_catalog() -> (std::sync::Arc<CombatTrainingCatalog>, Vec<String>) {
    let (catalog, records, ids) = r4_catalog();
    (std::sync::Arc::new(CombatTrainingCatalog::standard_from_json_with_records(&catalog, &records).unwrap()), ids)
}
