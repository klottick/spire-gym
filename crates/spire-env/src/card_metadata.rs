//! Opt-in static public card metadata (spire-90f). Generated on read from the
//! native catalog, complete constructor, cost/payload probe and upgrade owners;
//! no card table is kept here. Existing observation schemas are not changed.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use spire_core::combat::{static_card_face, static_card_upgrade_limit, StaticCardFace, StaticCost, StaticTarget};
use spire_core::content::{card_tags, complete, CardColor, CardDefinition, CardRarity, CardTag, CardTarget,
    CardType, Catalog, NoteStorage};

pub const CARD_METADATA_SCHEMA_ID: &str = "spire-lab-card-metadata-v1";
const FINGERPRINT_DOMAIN: &[u8] = b"spire-lab/card-metadata/v1\0";

/// The complete descriptor with its `fingerprint` (hex SHA-256 over the domain tag
/// and the canonical JSON of every other field).
pub fn card_metadata_descriptor() -> Value {
    let cards = Catalog::standard().cards().iter().map(row).collect::<Vec<_>>();
    let unresolved = cards.iter().filter(|card| card["faces"].as_array().unwrap().iter()
        .any(|face| face["cost"] == "unresolved" || face["target"] == "unresolved" && face["cost"] != "unplayable")).map(|card| card["id"].clone()).collect::<Vec<_>>();
    let mut rows = Sha256::new();
    rows.update(b"spire-lab/card-metadata/v1/cards\0");
    rows.update(crate::canonical_json(&Value::Array(cards.clone())).expect("integer-only static card rows"));
    let rows: [u8; 32] = rows.finalize().into();
    let faces = cards.iter().map(|card| card["faces"].as_array().unwrap().len()).sum::<usize>();
    let mut descriptor = json!({
        "schema_id": CARD_METADATA_SCHEMA_ID,
        "content_revision": {"cards_sha256": crate::combat_policy::hex(&rows), "rows": cards.len(), "faces": faces,
            "semantics": "SHA-256 over spire-lab/card-metadata/v1/cards\\0 and the canonical cards array only: it moves with any native content, constructor, public-value, cost, payload or upgrade-bound change and not with descriptor prose; compare it to detect a content or rules revision"},
        "key": "card id string; equals the entries of observation_spec()['vocabularies']['card'], so a category index i joins to the row whose id is vocabularies.card[i]; 'padding' and ids without a row are unknown",
        "content_basis": "spire-core Catalog::standard rows plus the complete (M26) card constructor, cost and payload owners and upgrade limits; regenerated on read, the fingerprint changes with any of them",
        "precedence": "static printed faces only; live observations, card views and legal actions are authoritative for current cost, upgrades, reductions, powers, targets and playability, including relic or potion effects the item-free probe cannot show, and this descriptor never overrides them",
        "support_join": "a row is registry metadata, not admission: support.complete_* describe the complete construction union only; a session's environment_spec() support/finite_content and its observation vocabulary decide admission per profile, character and ascension; a card absent there is unsupported in that profile; profile upgrade bounds may be narrower than max_upgrades; item admission (profile or M27 complete items) never widens card admission; a session's own content_id/rules_id identify its profile",
        "public_information": "static and run-independent: no seed, RNG cursor, hidden card identity or order, queue/listener state, unrevealed outcome or diagnostic",
        "provenance": "project-generated structured fields from native engine owners; no game text, wiki prose, binaries or assets are included; actual effects remain the engine's",
        "faces": "upgrades 0 and, when max_upgrades allows, 1; cost/target are probed read-only from the payload owner under one neutral complete-card state (one full-HP enemy, no powers, relics or potions, three energy)",
        "field_values": {
            "cost": "integer energy, 'x', 'unplayable' (the public value owner's unplayable class, or a status/curse with no playable payload) or 'unresolved'",
            "target": "'enemy' (needs an enemy target), 'untargeted' or 'unresolved'",
            "catalog_target": "blue rows only: the M20 catalog target ('enemy','all_enemies','self','none')",
            "max_upgrades": "integer, or null when unbounded",
            "base_damage/base_block/magic_number": "printed numbers from the public card-value owner that live card views use, else the constructor; null when neither projects one"
        },
        "unresolved": "the neutral probe cannot resolve a play condition (pile contents, generation state) or a text-unplayable kind; use live legal actions and card views",
        "unresolved_cards": unresolved,
        "unmodeled": ["display names (no existing public name owner; ids are snake_case)", "natural-language mechanic summaries (not authored in v1)", "effect structure beyond cost, target, keywords and constructor numbers", "per-profile admission (see support_join)"],
        "cards": cards,
    });
    let mut hash = Sha256::new();
    hash.update(FINGERPRINT_DOMAIN);
    hash.update(crate::canonical_json(&descriptor).expect("integer-only static card metadata"));
    let digest: [u8; 32] = hash.finalize().into();
    descriptor["fingerprint"] = json!(crate::combat_policy::hex(&digest));
    descriptor
}

fn row(definition: &CardDefinition) -> Value {
    let id = definition.id;
    let limit = static_card_upgrade_limit(id);
    let faces = if complete::supports_generated(id) {
        (0..=limit.unwrap_or(1).min(1)).map(|upgrades| face(upgrades, static_card_face(id, upgrades)
            .expect("complete constructor admits its own upgrade limit"), definition)).collect()
    } else { vec![] };
    json!({"id": id.as_str(), "color": color(definition.color), "rarity": rarity(definition.rarity),
        "type": card_type(definition.card_type), "tags": card_tags(id).iter().map(|tag| tag_name(*tag)).collect::<Vec<_>>(),
        "max_upgrades": if faces.is_empty() { json!(0) } else { json!(limit) },
        "support": {"complete_permanent": complete::supports_permanent(id), "complete_generated": complete::supports_generated(id),
            "choice_only": complete::is_choice_only(id), "removable": definition.note_storage == NoteStorage::RemovablePermanent},
        "faces": faces})
}

fn face(upgrades: u32, face: StaticCardFace, definition: &CardDefinition) -> Value {
    let keywords = [("ethereal", face.ethereal), ("exhaust", face.exhaust), ("innate", face.innate), ("retain", face.retain)]
        .into_iter().filter(|(_, set)| *set).map(|(name, _)| name).collect::<Vec<_>>();
    let mut value = json!({"upgrades": upgrades, "keywords": keywords,
        "cost": match face.cost { StaticCost::Energy(n) => json!(n), StaticCost::X => json!("x"),
            StaticCost::Unplayable => json!("unplayable"), StaticCost::Unresolved => json!("unresolved") },
        "target": match face.target { StaticTarget::Enemy => "enemy", StaticTarget::Untargeted => "untargeted", StaticTarget::Unresolved => "unresolved" },
        "base_damage": face.base_damage, "base_block": face.base_block, "magic_number": face.magic_number});
    if let Some(blue) = definition.blue_metadata {
        let catalog = if upgrades == 0 { blue.base } else { blue.upgraded };
        value["catalog_target"] = json!(match catalog.target { CardTarget::Enemy => "enemy", CardTarget::AllEnemies => "all_enemies",
            CardTarget::SelfTarget => "self", CardTarget::None => "none" });
    }
    value
}

fn color(value: CardColor) -> &'static str { match value { CardColor::Red => "red", CardColor::Green => "green", CardColor::Blue => "blue",
    CardColor::Purple => "purple", CardColor::Colorless => "colorless", CardColor::Status => "status", CardColor::Curse => "curse", CardColor::Shared => "shared" } }
fn rarity(value: CardRarity) -> &'static str { match value { CardRarity::Basic => "basic", CardRarity::Common => "common", CardRarity::Uncommon => "uncommon",
    CardRarity::Rare => "rare", CardRarity::Special => "special", CardRarity::Status => "status", CardRarity::Curse => "curse" } }
fn card_type(value: CardType) -> &'static str { match value { CardType::Attack => "attack", CardType::Skill => "skill", CardType::Power => "power",
    CardType::Status => "status", CardType::Curse => "curse" } }
fn tag_name(value: CardTag) -> &'static str { match value { CardTag::Strike => "strike", CardTag::StarterStrike => "starter_strike",
    CardTag::StarterDefend => "starter_defend", CardTag::Healing => "healing" } }
