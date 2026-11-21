//! Static resource admission for owned Standard combat entries (M39a).
use crate::standard_combat::{
    CHARACTERS, MAX_CATALOG_BYTES, MAX_CATALOG_ENTRIES, MAX_ENTRY_ID_BYTES,
    STANDARD_COMBAT_CATALOG_SCHEMA_ID, STANDARD_COMBAT_PROFILE_ID,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use spire_core::{
    content::{Catalog, RelicId, complete},
    items::{
        POTION_BELT_EXTRA_SLOTS, STARTER_REPLACEMENTS, complete_counter_intervals,
        complete_counter_required,
    },
    run::{
        STANDARD_COMBAT_MAX_DECK, STANDARD_COMBAT_MAX_HP, STANDARD_MAX_ASCENSION, potion_base_slots,
        standard_combat_bottle_type, standard_combat_misc_range, standard_combat_upgrade_limit,
    },
};

pub const COMBAT_ENTRY_RESOURCES_SCHEMA_ID: &str = "spire-lab-combat-entry-resources-v1";
const FINGERPRINT_DOMAIN: &[u8] = b"spire-lab/combat-entry-resources/v1\0";

/// The public constraints for an owned Standard combat start, with a hex SHA-256
/// fingerprint over the domain tag and the canonical JSON of the other fields.
pub fn combat_entry_resources_descriptor() -> Value {
    let catalog = Catalog::standard();
    let relic_unions = CHARACTERS.map(complete::entry_relic_union);
    let characters: Vec<_> = CHARACTERS.iter().zip(&relic_unions).map(|(character, relics)| {
        json!({
            "character": character.as_str(),
            "relics": relics.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
            "potions": complete::potion_union(*character).iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        })
    }).collect();

    let registered = catalog.relics().iter().map(|row| row.id);
    let relics: Vec<_> = registered.filter_map(|id| {
        let owners: Vec<_> = CHARACTERS.iter().zip(&relic_unions)
            .filter(|(_, union)| union.contains(&id))
            .map(|(character, _)| character.as_str()).collect();
        (!owners.is_empty()).then(|| relic_row(id, &owners))
    }).collect();

    let base_by_ascension: Vec<_> = (0..=STANDARD_MAX_ASCENSION).map(potion_base_slots).collect();
    let allowed: Vec<_> = base_by_ascension.iter().enumerate().map(|(ascension, &base)| {
        let expanded = base + POTION_BELT_EXTRA_SLOTS;
        json!({"ascension": ascension, "without_potion_belt": [base, expanded], "with_potion_belt": [expanded]})
    }).collect();
    let cards: Vec<_> = catalog.cards().iter().map(|row| row.id)
        .filter(|&id| complete::supports_permanent(id))
        .map(|id| {
            let misc = standard_combat_misc_range(id);
            json!({"card": id.as_str(), "max_upgrades": standard_combat_upgrade_limit(id), "misc": [misc.start(), misc.end()]})
        }).collect();
    let mut descriptor = json!({
        "schema_id": COMBAT_ENTRY_RESOURCES_SCHEMA_ID,
        "profile_id": STANDARD_COMBAT_PROFILE_ID,
        "catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID,
        "characters": characters,
        "relics": relics,
        "potion_slots": {
            "base_by_ascension": base_by_ascension,
            "potion_belt": RelicId::PotionBelt.as_str(),
            "belt_extra_slots": POTION_BELT_EXTRA_SLOTS,
            "allowed": allowed,
        },
        "bounds": {
            "ascension": {"minimum": 0, "maximum": STANDARD_MAX_ASCENSION},
            "hp": {"minimum": 1, "maximum_field": "max_hp"},
            "max_hp": {"minimum": 1, "maximum": STANDARD_COMBAT_MAX_HP},
            "gold": {"minimum": 0, "maximum": u32::MAX},
            "deck": {"minimum": 1, "maximum": STANDARD_COMBAT_MAX_DECK, "maximum_includes_entry_generated_cards": true},
            "entry_id": {"minimum_bytes": 1, "maximum_bytes": MAX_ENTRY_ID_BYTES, "pattern": "^[A-Za-z0-9_-]+$", "unique": true},
            "catalog_entries": {"minimum": 1, "maximum": MAX_CATALOG_ENTRIES},
            "catalog_bytes": {"maximum": MAX_CATALOG_BYTES},
            "cards": cards,
        },
    });
    let mut hash = Sha256::new();
    hash.update(FINGERPRINT_DOMAIN);
    hash.update(crate::canonical_json(&descriptor).expect("integer-only combat entry resources"));
    let digest: [u8; 32] = hash.finalize().into();
    descriptor["fingerprint"] = json!(crate::combat_policy::hex(&digest));
    descriptor
}

fn relic_row(id: RelicId, owners: &[&str]) -> Value {
    let intervals = complete_counter_intervals(id);
    let counter = (!intervals.is_empty()).then(|| json!({
        "rule": if complete_counter_required(id) { "required" } else { "optional" },
        "values": intervals.iter().map(|r| [*r.start(), *r.end()]).collect::<Vec<_>>(),
        "default": null,
    }));
    let exclusive_with: Vec<_> = STARTER_REPLACEMENTS.iter().filter_map(|&(starter, replacement)| {
        if starter == id { Some(replacement.as_str()) }
        else if replacement == id { Some(starter.as_str()) }
        else { None }
    }).collect();
    let companion = standard_combat_bottle_type(id).map(|card_type| json!({
        "field": "bottled_cards",
        "required": true,
        "nullable": true,
        "card_type": crate::card_metadata::card_type(card_type),
        "removable": true,
    }));
    json!({
        "id": id.as_str(), "characters": owners, "counter": counter,
        "exclusive_with": exclusive_with, "companion": companion,
    })
}
