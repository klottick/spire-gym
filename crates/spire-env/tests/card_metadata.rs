//! spire-90f opt-in card metadata: identity, bounded deterministic bytes, joins,
//! native consistency, public-information safety and frozen-schema preservation.
use spire_env::*;
use spire_core::content::{CardCost, CardId, CardTarget, Catalog};
use serde_json::Value;
use sha2::{Digest, Sha256};

const UNRESOLVED: &[&str] = &["mind_blast", "secret_technique", "secret_weapon", "distraction", "alchemize", "discovery",
    "chrysalis", "metamorphosis", "stack", "blizzard", "white_noise", "deus_ex_machina"];

fn hex(bytes: [u8; 32]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn row<'a>(d: &'a Value, id: &str) -> &'a Value { d["cards"].as_array().unwrap().iter().find(|r| r["id"] == id).expect("row") }
fn face<'a>(d: &'a Value, id: &str, upgrades: usize) -> &'a Value { &row(d, id)["faces"][upgrades] }

#[test]
fn descriptor_is_versioned_deterministic_bounded_and_self_fingerprinted() {
    let d = card_metadata_descriptor();
    assert_eq!(d["schema_id"], CARD_METADATA_SCHEMA_ID);
    let bytes = canonical_json(&d).unwrap();
    assert_eq!(bytes, canonical_json(&card_metadata_descriptor()).unwrap());
    assert!(bytes.len() < 512 * 1024, "bounded export: {} bytes", bytes.len());
    let mut body = d.clone();
    body.as_object_mut().unwrap().remove("fingerprint");
    let mut hash = Sha256::new();
    hash.update(b"spire-lab/card-metadata/v1\0");
    hash.update(canonical_json(&body).unwrap());
    assert_eq!(d["fingerprint"], hex(hash.finalize().into()));
    let mut cards = Sha256::new();
    cards.update(b"spire-lab/card-metadata/v1/cards\0");
    cards.update(canonical_json(&d["cards"]).unwrap());
    let faces = d["cards"].as_array().unwrap().iter().map(|r| r["faces"].as_array().unwrap().len()).sum::<usize>();
    assert_eq!((&d["content_revision"]["cards_sha256"], &d["content_revision"]["rows"], &d["content_revision"]["faces"]),
        (&Value::from(hex(cards.finalize().into())), &Value::from(369), &Value::from(faces)));
    // Pinned: any content, constructor, cost, payload or schema change must be an explicit update.
    assert_eq!(d["fingerprint"], "3e922792b73e10ce2214d1e97a8bbcd46af1dad1320b85d41c2310c6b855071c");
}

#[test]
fn every_catalog_card_has_one_stable_row_and_upgrade_variants() {
    let d = card_metadata_descriptor();
    let ids = d["cards"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect::<Vec<_>>();
    let catalog = Catalog::standard().cards().iter().map(|c| c.id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, catalog);
    assert_eq!(ids.iter().collect::<std::collections::BTreeSet<_>>().len(), ids.len());
    for r in d["cards"].as_array().unwrap() {
        let upgrades = r["faces"].as_array().unwrap().iter().map(|f| f["upgrades"].as_u64().unwrap()).collect::<Vec<_>>();
        let expected: &[u64] = if !r["support"]["complete_generated"].as_bool().unwrap() { &[] }
            else if r["max_upgrades"] == 0 { &[0] } else { &[0, 1] };
        assert_eq!(upgrades, expected, "{}", r["id"]);
    }
    assert!(row(&d, "searing_blow")["max_upgrades"].is_null());
    assert_eq!((&row(&d, "strike")["max_upgrades"], &row(&d, "wound")["max_upgrades"]), (&Value::from(1), &Value::from(0)));
    assert!(row(&d, "become_almighty")["support"]["choice_only"].as_bool().unwrap());
}

#[test]
fn representative_faces_match_native_definitions() {
    let d = card_metadata_descriptor();
    assert_eq!((&face(&d, "strike", 0)["cost"], &face(&d, "strike", 0)["target"]), (&Value::from(1), &Value::from("enemy")));
    assert_eq!(row(&d, "strike")["tags"], serde_json::json!(["strike"]));
    assert_eq!(face(&d, "defend", 1)["target"], "untargeted");
    assert_eq!(face(&d, "whirlwind", 0)["cost"], "x");
    assert_eq!(face(&d, "carnage", 0)["keywords"], serde_json::json!(["ethereal"]));
    assert_eq!(face(&d, "limit_break", 0)["keywords"], serde_json::json!(["exhaust"]));
    assert_eq!(face(&d, "limit_break", 1)["keywords"], serde_json::json!([]));
    assert_eq!((&face(&d, "wound", 0)["cost"], &row(&d, "wound")["type"]), (&Value::from("unplayable"), &Value::from("status")));
    // Every blue row agrees with the independent M20 base/upgraded catalog.
    for definition in Catalog::standard().cards().iter().filter(|c| c.blue_metadata.is_some()) {
        let blue = definition.blue_metadata.unwrap();
        for (upgrades, catalog) in [(0, blue.base), (1, blue.upgraded)] {
            let f = face(&d, definition.id.as_str(), upgrades);
            let cost = match catalog.cost { CardCost::Energy(n) => Value::from(n), CardCost::X => Value::from("x") };
            assert_eq!(f["cost"], cost, "{} +{upgrades}", definition.id.as_str());
            let keywords = [("ethereal", catalog.keywords.ethereal), ("exhaust", catalog.keywords.exhaust), ("innate", catalog.keywords.innate)];
            for (name, set) in keywords { assert_eq!(f["keywords"].as_array().unwrap().contains(&Value::from(name)), set, "{} +{upgrades} {name}", definition.id.as_str()); }
            if f["target"] != "unresolved" {
                assert_eq!(f["target"] == "enemy", catalog.target == CardTarget::Enemy, "{} +{upgrades}", definition.id.as_str());
            }
        }
    }
    // Printed numbers come from the public card-value owner (spire-90f review B1).
    let numbers = |id: &str, u: usize| ["base_damage", "base_block", "magic_number"].map(|k| face(&d, id, u)[k].as_i64());
    assert_eq!((numbers("strike", 0), numbers("strike", 1)), ([Some(6), None, None], [Some(9), None, None]));
    assert_eq!((numbers("defend", 0), numbers("defend", 1)), ([None, Some(5), None], [None, Some(8), None]));
    assert_eq!((numbers("bash", 0), numbers("bash", 1)), ([Some(8), None, Some(2)], [Some(10), None, Some(3)]));
    assert_eq!((numbers("neutralize", 0), numbers("neutralize", 1)), ([Some(3), None, Some(1)], [Some(4), None, Some(2)]));
    for id in ["reflex", "tactician"] { assert_eq!(face(&d, id, 0)["cost"], "unplayable", "{id}"); }
    // Unresolved faces are listed exactly, never silently guessed.
    assert_eq!(d["unresolved_cards"], serde_json::json!(UNRESOLVED));
    assert!(!d["unresolved_cards"].as_array().unwrap().contains(&Value::from(CardId::Strike.as_str())));
}

#[test]
fn rows_join_to_vocabularies_and_support_without_implying_admission() {
    let d = card_metadata_descriptor();
    let known = |id: &str| d["cards"].as_array().unwrap().iter().any(|r| r["id"] == id);
    let caps = RunCapacities::m10_default();
    for spec in [combat_policy_observation_spec_v2().unwrap(), policy_run_observation_spec(&caps).unwrap()] {
        for name in spec.descriptor()["vocabularies"]["card"].as_array().unwrap().iter().filter_map(Value::as_str) {
            assert!(name == "padding" || known(name), "vocabulary card {name} has no metadata row");
        }
    }
    let support = combat_training_support();
    let admitted = support["cards"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect::<Vec<_>>();
    for id in &admitted { assert!(row(&d, id)["support"]["complete_permanent"].as_bool().unwrap(), "{id}"); }
    // A complete-union row with full faces is still outside the Ironclad v1 profile.
    assert!(row(&d, "eruption")["support"]["complete_permanent"].as_bool().unwrap() && !admitted.contains(&"eruption"));
    assert!(d["support_join"].as_str().unwrap().contains("environment_spec()"));
}

#[test]
fn descriptor_carries_no_run_private_or_diagnostic_fields() {
    fn keys(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::Object(map) => for (k, v) in map { out.push(k.clone()); keys(v, out); },
            Value::Array(items) => for v in items { keys(v, out); },
            _ => {}
        }
    }
    let mut all = vec![];
    keys(&card_metadata_descriptor(), &mut all);
    for key in all {
        for banned in ["seed", "rng", "cursor", "order", "queue", "listener", "diagnostic", "instance", "pile", "hidden"] {
            assert!(!key.contains(banned), "{key}");
        }
    }
}

#[test]
fn observation_fingerprints_match_current_pins() {
    let caps = RunCapacities::m10_default();
    let now = [
        (combat_policy_observation_spec().unwrap().fingerprint(), "08bb0e2244325c5ae33caea9b3c0f78ceda8cc79ea9100e77618b9d7a8d880db"),
        (combat_policy_observation_spec_v2().unwrap().fingerprint(), "a27083bad27ffdd1fafea4c8de2b1580edad3c9d10200695694e15858b6122f7"),
        (policy_run_observation_spec(&caps).unwrap().fingerprint(), "28d92d89c437eea7fdc260240585bca466dee1fd74b9b3eb38513d2b34b0be64"),
        (m10_run_observation_spec(&caps).unwrap().fingerprint(), "dccc7624c45898bc33e3fa682b620fd78ce70fe558d6db7624a888a7ed068c21"),
        (run_observation_spec(&RunCapacities::m9_default()).unwrap().fingerprint(), "270163f700c7ca19556898d14c22ca8d641b0ef6f5c757d80d0ca7983989981c"),
    ];
    for (actual, frozen) in now { assert_eq!(hex(actual), frozen); }
}
