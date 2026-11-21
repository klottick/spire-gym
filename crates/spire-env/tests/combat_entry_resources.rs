use serde_json::{Value, json};
use spire_env::{CombatTrainingCatalog, RunSessionError, STANDARD_COMBAT_CATALOG_SCHEMA_ID, STANDARD_COMBAT_PROFILE_ID};

fn entry(character: &str) -> Value {
    json!({
        "id": "probe", "character": character, "ascension": 0, "encounter": "jaw_worm",
        "hp": 50, "max_hp": 70, "gold": 0, "deck": [{"card": "strike", "upgrades": 0}],
        "relics": [], "potions": [null, null, null]
    })
}

fn parse(entry: &Value) -> Result<CombatTrainingCatalog, RunSessionError> {
    CombatTrainingCatalog::standard_from_json(&catalog_bytes(&[entry]))
}

fn catalog_bytes<T: serde::Serialize>(entries: &[T]) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID,
        "profile_id": STANDARD_COMBAT_PROFILE_ID,
        "entries": entries,
    })).unwrap()
}

fn rejects(entry: &Value, field: &str) {
    let result = parse(entry);
    assert_rejection(result, field);
}

fn assert_rejection(result: Result<CombatTrainingCatalog, RunSessionError>, message_prefix: &str) {
    assert!(
        matches!(&result, Err(RunSessionError::Input(message)) if message.starts_with(message_prefix)),
        "expected an Input naming {message_prefix}, got {:?}", result.err()
    );
}

#[test]
fn resource_errors_name_the_offending_field() {
    let mut probe = entry("ironclad");
    probe["relics"] = json!(["neows_lament"]);
    rejects(&probe, "relic_counters:");
    for (field, value, diagnostic) in [
        ("hp", json!(0), "hp:"),
        ("max_hp", json!(1000), "max_hp:"),
        ("deck", json!([]), "deck:"),
        ("deck", json!([{"card": "unknown", "upgrades": 0}]), "deck:"),
        ("deck", json!([{"card": "wound", "upgrades": 0}]), "deck:"),
        ("relics", json!(["vajra", "vajra"]), "relics:"),
        ("relic_counters", json!({"frozen_core": 0}), "relic_counters:"),
        ("burning", json!("strength"), "burning "),
    ] {
        let mut probe = entry("ironclad");
        probe[field] = value;
        rejects(&probe, diagnostic);
    }
}

fn rows(value: &Value) -> &[Value] {
    value.as_array().unwrap()
}

fn relic_row<'a>(descriptor: &'a Value, id: &str) -> &'a Value {
    rows(&descriptor["relics"]).iter().find(|row| row["id"] == id).unwrap()
}

fn own_relic(entry: &mut Value, row: &Value) {
    let id = row["id"].as_str().unwrap();
    entry["relics"] = json!([id]);
    let counter = &row["counter"];
    if counter["rule"] == "required" {
        entry["relic_counters"][id] = counter["values"][0][0].clone();
    }
    if !row["companion"].is_null() {
        entry["bottled_cards"] = json!({id: null});
    }
}

fn accepts(entry: &Value) {
    if let Err(error) = parse(entry) {
        panic!("rejected {entry}: {error:?}");
    }
}

#[test]
fn catalog_admission_matches_every_character_item_union() {
    let descriptor = spire_env::combat_entry_resources_descriptor();
    let catalog = spire_core::content::Catalog::standard();
    let characters = rows(&descriptor["characters"]);
    assert_eq!(characters.iter().map(|row| row["character"].as_str().unwrap()).collect::<Vec<_>>(),
        ["ironclad", "silent", "defect", "watcher"]);
    for character in characters {
        let name = character["character"].as_str().unwrap();
        // Union rows must also be registered. The registry walk below covers unsupported IDs.
        for relic in rows(&character["relics"]) {
            assert!(catalog.relics().iter().any(|row| relic == row.id.as_str()));
        }
        for potion in rows(&character["potions"]) {
            assert!(catalog.potions().iter().any(|row| potion == row.id.as_str()));
        }
        let mut probe = entry(name);
        probe["relics"] = json!(["unregistered_relic"]);
        rejects(&probe, "relics:");
        let mut probe = entry(name);
        probe["potions"][0] = json!("unregistered_potion");
        rejects(&probe, "potions:");

        for relic in catalog.relics() {
            let id = relic.id.as_str();
            let mut probe = entry(name);
            if rows(&character["relics"]).contains(&json!(id)) {
                let row = relic_row(&descriptor, id);
                assert!(rows(&row["characters"]).contains(&json!(name)));
                own_relic(&mut probe, row);
                if id == descriptor["potion_slots"]["potion_belt"] {
                    let count = descriptor["potion_slots"]["allowed"][0]["with_potion_belt"][0].as_u64().unwrap() as usize;
                    probe["potions"] = json!(vec![Value::Null; count]);
                }
                accepts(&probe);
            } else {
                probe["relics"] = json!([id]);
                rejects(&probe, "relics:");
            }
        }
        for potion in catalog.potions() {
            let id = potion.id.as_str();
            let mut probe = entry(name);
            probe["potions"][0] = json!(id);
            if rows(&character["potions"]).contains(&json!(id)) {
                accepts(&probe);
            } else {
                rejects(&probe, "potions:");
            }
        }
    }
    // Check the other direction of the character-to-relic join.
    for relic in rows(&descriptor["relics"]) {
        let owners: Vec<_> = characters.iter().filter(|row| rows(&row["relics"]).contains(&relic["id"]))
            .map(|row| row["character"].clone()).collect();
        assert_eq!(json!(owners), relic["characters"]);
    }
}

#[test]
fn catalog_counter_rules_match_presence_and_interval_boundaries() {
    let descriptor = spire_env::combat_entry_resources_descriptor();
    for row in rows(&descriptor["relics"]) {
        let mut probe = entry(row["characters"][0].as_str().unwrap());
        let id = row["id"].as_str().unwrap();
        own_relic(&mut probe, row);
        if id == "potion_belt" {
            probe["potions"] = json!([null, null, null, null, null]);
        }
        probe.as_object_mut().unwrap().remove("relic_counters");
        let counter = &row["counter"];
        if counter["rule"] == "required" {
            rejects(&probe, "relic_counters:");
        } else {
            let normalized = parse(&probe).unwrap().descriptor();
            assert_eq!(normalized["entries"][0]["relic_counters"], json!({}), "{id}");
        }
        if counter.is_null() {
            probe["relic_counters"][id] = json!(0);
            rejects(&probe, "relic_counters:");
            continue;
        }
        assert!(counter["default"].is_null());
        let intervals: Vec<_> = rows(&counter["values"]).iter()
            .map(|range| range[0].as_i64().unwrap()..=range[1].as_i64().unwrap())
            .collect();
        for interval in &intervals {
            for edge in [*interval.start(), *interval.end()] {
                for value in (edge - 1)..=(edge + 1) {
                    if i32::try_from(value).is_err() {
                        continue;
                    }
                    probe["relic_counters"][id] = json!(value);
                    if intervals.iter().any(|interval| interval.contains(&value)) {
                        accepts(&probe);
                    } else {
                        rejects(&probe, "relic_counters:");
                    }
                }
            }
        }
    }
    let mut probe = entry("ironclad");
    probe["relic_counters"] = json!({"pen_nib": 0});
    rejects(&probe, "relic_counters:");
}

#[test]
fn catalog_potion_slots_match_at_every_ascension() {
    let descriptor = spire_env::combat_entry_resources_descriptor();
    let slots = &descriptor["potion_slots"];
    let ascensions = descriptor["bounds"]["ascension"]["maximum"].as_u64().unwrap() as usize + 1;
    assert_eq!(rows(&slots["allowed"]).len(), ascensions);
    assert_eq!(rows(&slots["base_by_ascension"]).len(), ascensions);
    for (ascension, allowance) in rows(&slots["allowed"]).iter().enumerate() {
        assert_eq!(allowance["ascension"], ascension);
        let base = slots["base_by_ascension"][ascension].as_u64().unwrap();
        let expanded = base + slots["belt_extra_slots"].as_u64().unwrap();
        assert_eq!(allowance["with_potion_belt"], json!([expanded]));
        assert_eq!(allowance["without_potion_belt"], json!([base, expanded]));
        for (relics, allowed) in [
            (json!([]), &allowance["without_potion_belt"]),
            (json!([slots["potion_belt"]]), &allowance["with_potion_belt"]),
        ] {
            let mut probe = entry("ironclad");
            probe["ascension"] = json!(ascension);
            probe["relics"] = relics;
            for count in 0..=8 {
                probe["potions"] = json!(vec![Value::Null; count]);
                if rows(allowed).contains(&json!(count)) {
                    accepts(&probe);
                } else {
                    rejects(&probe, "potions:");
                }
            }
        }
    }
}

#[test]
fn catalog_enforces_companions_and_starter_replacements() {
    let descriptor = spire_env::combat_entry_resources_descriptor();
    for row in rows(&descriptor["relics"]) {
        let id = row["id"].as_str().unwrap();
        let mut probe = entry(row["characters"][0].as_str().unwrap());
        for incompatible in rows(&row["exclusive_with"]) {
            let other = relic_row(&descriptor, incompatible.as_str().unwrap());
            assert!(rows(&other["exclusive_with"]).contains(&row["id"]));
            probe["relics"] = json!([id, incompatible]);
            rejects(&probe, "relics:");
        }
        let companion = &row["companion"];
        if companion.is_null() {
            continue;
        }
        assert_eq!(companion["field"], "bottled_cards");
        assert_eq!(companion["required"], true);
        assert_eq!(companion["nullable"], true);
        assert_eq!(companion["removable"], true);
        let field = companion["field"].as_str().unwrap();
        probe["relics"] = json!([id]);
        rejects(&probe, "bottled_cards:");
        own_relic(&mut probe, row);
        accepts(&probe);

        let mut unowned = probe.clone();
        unowned["relics"] = json!([]);
        rejects(&unowned, "bottled_cards: relic is not an owned Bottled relic");
        let mut non_bottle = probe.clone();
        let vajra = relic_row(&descriptor, "vajra");
        assert!(vajra["companion"].is_null());
        own_relic(&mut non_bottle, vajra);
        non_bottle[field] = json!({"vajra": null});
        rejects(&non_bottle, "bottled_cards: relic is not an owned Bottled relic");
        for (kind, card) in [("attack", "bash"), ("skill", "shrug_it_off"), ("power", "inflame")] {
            let card = json!({"card": card, "upgrades": 0});
            probe["deck"] = json!([card]);
            probe["bottled_cards"][id] = card;
            if companion["card_type"] == kind {
                accepts(&probe);
                let mut absent = probe.clone();
                absent["deck"] = entry("ironclad")["deck"].clone();
                rejects(&absent, "bottled_cards: card is not in the deck");
            } else {
                rejects(&probe, "bottled_cards:");
            }
        }
    }
}

#[test]
fn catalog_card_numeric_bounds_match_the_descriptor() {
    let descriptor = spire_env::combat_entry_resources_descriptor();
    let cards = rows(&descriptor["bounds"]["cards"]);
    for row in cards {
        let mut probe = entry("ironclad");
        let lower = row["misc"][0].as_i64().unwrap();
        let upper = row["misc"][1].as_i64().unwrap();
        let max_upgrades = row["max_upgrades"].as_u64().unwrap();
        probe["deck"][0] = json!({"card": row["card"], "upgrades": max_upgrades, "misc": lower});
        accepts(&probe);
        probe["deck"][0]["misc"] = json!(upper);
        accepts(&probe);
        probe["deck"][0]["upgrades"] = json!(max_upgrades + 1);
        rejects(&probe, "deck.upgrades:");
        probe["deck"][0]["upgrades"] = json!(0);
        for misc in [lower - 1, upper + 1] {
            probe["deck"][0]["misc"] = json!(misc);
            rejects(&probe, "deck.misc:");
        }
    }
}

#[test]
fn descriptor_identity_is_static_and_canonical() {
    use sha2::{Digest, Sha256};
    let mut descriptor = spire_env::combat_entry_resources_descriptor();
    assert_eq!(descriptor, spire_env::combat_entry_resources_descriptor());
    assert_eq!(descriptor["schema_id"], "spire-lab-combat-entry-resources-v1");
    let catalog = parse(&entry("ironclad")).unwrap().descriptor();
    assert_eq!(descriptor["profile_id"], catalog["profile_id"]);
    assert_eq!(descriptor["catalog_schema_id"], catalog["catalog_schema_id"]);
    let fingerprint = descriptor.as_object_mut().unwrap().remove("fingerprint").unwrap();
    let mut hash = Sha256::new();
    hash.update(b"spire-lab/combat-entry-resources/v1\0");
    hash.update(spire_env::canonical_json(&descriptor).unwrap());
    assert_eq!(fingerprint, format!("{:x}", hash.finalize()));
    assert_eq!(fingerprint, "4eafca3b0089e15735fb6aef24d594a084030438daa9738a1ac8cf6eab75dc5a");
    let text = descriptor.to_string();
    for forbidden in ["\"seed\"", "\"rng\"", "\"draw_order\""] {
        assert!(!text.contains(forbidden));
    }
}

#[test]
fn resource_decode_errors_name_the_offending_field() {
    for (path, value, field) in [
        ("/gold", json!(-1), "gold:"),
        ("/gold", json!(4_294_967_296_u64), "gold:"),
        ("/hp", json!(-1), "hp:"),
        ("/max_hp", json!(4_294_967_296_u64), "max_hp:"),
        ("/deck/0/upgrades", json!(-1), "deck.upgrades:"),
        ("/deck/0/misc", json!(2_147_483_648_u64), "deck.misc:"),
        ("/potions", json!(3), "potions:"),
    ] {
        let mut probe = entry("ironclad");
        probe["deck"][0]["misc"] = json!(0);
        *probe.pointer_mut(path).unwrap() = value;
        rejects(&probe, field);
    }
    let mut probe = entry("ironclad");
    probe["relics"] = json!(["circlet"]);
    probe["relic_counters"] = json!({"circlet": 2_147_483_648_u64});
    rejects(&probe, "relic_counters:");
    probe = entry("ironclad");
    probe.as_object_mut().unwrap().remove("hp");
    rejects(&probe, "hp:");
}


#[test]
fn catalog_decode_preserves_struct_sequences_and_hides_private_keys() {
    let encoded = json!([
        "probe", "ironclad", 0, "jaw_worm", null, 50, 70, 0,
        [["strike", 0]], [], {}, [null, null, null]
    ]);
    let expected = parse(&entry("ironclad")).unwrap().descriptor();
    assert_eq!(parse(&encoded).unwrap().descriptor(), expected);

    let generic = "invalid Standard combat catalog fields or types";
    for path in ["/", "/deck/0"] {
        let mut probe = entry("ironclad");
        let item = if path == "/" { &mut probe } else { probe.pointer_mut(path).unwrap() };
        item["hidden_sentinel"] = json!(123456);
        assert_eq!(parse(&probe).err(), Some(RunSessionError::Input(generic)));
    }
    assert!(matches!(CombatTrainingCatalog::standard_from_json(b"{"), Err(RunSessionError::Input(message)) if message == generic));

    let mut book: Value = serde_json::from_slice(&catalog_bytes(&[entry("ironclad")])).unwrap();
    book["entries"] = json!(1);
    assert_rejection(CombatTrainingCatalog::standard_from_json(&serde_json::to_vec(&book).unwrap()), "entries:");
}

fn bound(value: &Value, key: &str) -> usize {
    usize::try_from(value[key].as_u64().unwrap()).unwrap()
}

#[test]
fn catalog_deck_admission_matches_published_bounds() {
    let descriptor = spire_env::combat_entry_resources_descriptor();
    let deck = &descriptor["bounds"]["deck"];
    let minimum = bound(deck, "minimum");
    let maximum = bound(deck, "maximum");
    let mut probe = entry("ironclad");
    let card = probe["deck"][0].clone();
    for count in [minimum - 1, minimum, maximum, maximum + 1] {
        probe["deck"] = json!(vec![card.clone(); count]);
        if (minimum..=maximum).contains(&count) {
            accepts(&probe);
        } else {
            rejects(&probe, "deck:");
        }
    }

    own_relic(&mut probe, relic_row(&descriptor, "mark_of_pain"));
    let generated_cards = 2; // Mark of Pain puts two Wounds into the deck at entry.
    let capacity = if deck["maximum_includes_entry_generated_cards"].as_bool().unwrap() {
        maximum - generated_cards
    } else {
        maximum
    };
    probe["deck"] = json!(vec![card.clone(); capacity]);
    accepts(&probe);
    probe["deck"] = json!(vec![card; capacity + 1]);
    assert_rejection(parse(&probe), "Standard combat entry setup rejected");
}

#[test]
fn catalog_count_and_bytes_match_published_bounds() {
    let descriptor = spire_env::combat_entry_resources_descriptor();
    let count = &descriptor["bounds"]["catalog_entries"];
    let minimum = bound(count, "minimum");
    let maximum = bound(count, "maximum");
    let entries: Vec<_> = (0..=maximum).map(|n| {
        let mut probe = entry("ironclad");
        probe["id"] = json!(format!("probe-{n}"));
        probe
    }).collect();
    for size in [minimum - 1, minimum, maximum, maximum + 1] {
        let result = CombatTrainingCatalog::standard_from_json(&catalog_bytes(&entries[..size]));
        if (minimum..=maximum).contains(&size) {
            assert_eq!(rows(&result.unwrap().descriptor()["entries"]).len(), size);
        } else {
            assert_rejection(result, "catalog requires");
        }
    }

    let max_bytes = bound(&descriptor["bounds"]["catalog_bytes"], "maximum");
    let mut payload = catalog_bytes(&entries[..minimum]);
    assert!(payload.len() < max_bytes);
    payload.resize(max_bytes, b' ');
    CombatTrainingCatalog::standard_from_json(&payload).unwrap();
    payload.push(b' ');
    assert_rejection(CombatTrainingCatalog::standard_from_json(&payload), "combat catalog exceeds");
}
