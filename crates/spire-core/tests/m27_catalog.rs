//! M27 item identities and catalog rows (design §4, §5.1; corrections G5–G8).
//!
//! Expected values are literals from design §4.1–§4.3, `data/relics/relics.json` and the
//! planning potion inventory (`planning-v1/research-B/potion-inventory.tsv`), never values
//! computed by the helpers under test.

use spire_core::content::{complete, Catalog, Character, PotionId, PotionRarity, RelicId, RelicTier};
use std::collections::BTreeSet;

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn all_relics() -> Vec<RelicId> {
    let mut seen = BTreeSet::new();
    CHARACTERS.iter().flat_map(|c| complete::relic_union(*c)).filter(|id| seen.insert(*id)).collect()
}

fn all_potions() -> Vec<PotionId> {
    let mut seen = Vec::new();
    for id in CHARACTERS.iter().flat_map(|c| complete::potion_union(*c)) {
        if !seen.contains(&id) { seen.push(id); }
    }
    seen
}

/// Design §4.3 / economy research §6: rarity and class (None = shared) of every potion.
const POTIONS: [(PotionId, PotionRarity, Option<Character>); 42] = {
    use PotionId::*; use PotionRarity::*;
    const I: Option<Character> = Some(Character::Ironclad);
    const S: Option<Character> = Some(Character::Silent);
    const D: Option<Character> = Some(Character::Defect);
    const W: Option<Character> = Some(Character::Watcher);
    [
        (FirePotion, Common, None), (BloodPotion, Common, I), (FruitJuice, Rare, None), (BlockPotion, Common, None),
        (EnergyPotion, Common, None), (StrengthPotion, Common, None), (SwiftPotion, Common, None), (WeakPotion, Common, None),
        (FearPotion, Common, None), (DexterityPotion, Common, None), (AncientPotion, Uncommon, None), (Elixir, Uncommon, I),
        (HeartOfIron, Rare, I), (PoisonPotion, Common, S), (CunningPotion, Uncommon, S), (GhostInAJar, Rare, S),
        (SmokeBomb, Rare, None), (AttackPotion, Common, None), (SkillPotion, Common, None), (PowerPotion, Common, None),
        (ColorlessPotion, Common, None), (FlexPotion, Common, None), (SpeedPotion, Common, None), (RegenPotion, Uncommon, None),
        (LiquidBronze, Uncommon, None), (EssenceOfSteel, Uncommon, None), (CultistPotion, Rare, None), (BlessingOfTheForge, Common, None),
        (SneckoOil, Rare, None), (FairyPotion, Rare, None), (ExplosivePotion, Common, None), (GamblersBrew, Uncommon, None),
        (LiquidMemories, Uncommon, None), (DuplicationPotion, Uncommon, None), (DistilledChaos, Uncommon, None), (EntropicBrew, Rare, None),
        (FocusPotion, Common, D), (PotionOfCapacity, Uncommon, D), (EssenceOfDarkness, Rare, D),
        (StancePotion, Uncommon, W), (Ambrosia, Rare, W), (BottledMiracle, Common, W),
    ]
};

#[test]
fn m27_identity_catalog_rows() {
    let catalog = Catalog::standard();
    let relics = all_relics();
    assert_eq!(relics.len(), 179);
    let mut tiers = std::collections::BTreeMap::new();
    for id in &relics {
        let row = catalog.relic(*id).unwrap_or_else(|| panic!("{id:?} has no row"));
        assert!(!row.definition_only, "{id:?} is definition_only (G5)");
        let tier = catalog.relic_tier(*id).unwrap_or_else(|| panic!("{id:?} has no tier (G6)"));
        *tiers.entry(format!("{tier:?}")).or_insert(0) += 1;
        assert!(CHARACTERS.iter().any(|c| catalog.relic_eligible_for(*id, *c) == Some(true)), "{id:?} has no class");
    }
    let expected = [("Boss", 30), ("Common", 36), ("Rare", 34), ("Shop", 20), ("Special", 19), ("Starter", 4), ("Uncommon", 36)];
    assert_eq!(tiers.into_iter().collect::<Vec<_>>(), expected.map(|(t, n)| (t.to_string(), n)).to_vec());
    let potions = all_potions();
    assert_eq!(potions.len(), 42);
    for id in &potions {
        let row = catalog.potion(*id).unwrap_or_else(|| panic!("{id:?} has no row"));
        assert!(!row.definition_only, "{id:?} is definition_only (G5)");
        assert!(catalog.potion_rarity(*id).is_some(), "{id:?} has no rarity (G6)");
    }
    // The 22 rows design §5.1 names, explicitly (G5).
    for id in [RelicId::BurningBlood, RelicId::RingOfTheSnake, RelicId::CrackedCore, RelicId::PureWater, RelicId::BloodVial,
        RelicId::Anchor, RelicId::Lantern, RelicId::Strawberry, RelicId::Pear, RelicId::Mango, RelicId::SacredBark,
        RelicId::MarkOfBloom, RelicId::MagicFlower, RelicId::ToyOrnithopter] {
        assert_eq!(catalog.relic(id).map(|row| row.definition_only), Some(false), "{id:?}");
    }
    for id in [PotionId::FirePotion, PotionId::FruitJuice, PotionId::BlockPotion, PotionId::EnergyPotion,
        PotionId::StrengthPotion, PotionId::SwiftPotion, PotionId::WeakPotion, PotionId::FearPotion] {
        assert_eq!(catalog.potion(id).map(|row| row.definition_only), Some(false), "{id:?}");
    }
    // Red Circlet has no identity; Unknown is never an item.
    assert!(!relics.iter().any(|id| id.as_str() == "red_circlet"));
    assert!(!complete::supports_relic(RelicId::Unknown("red_circlet")));
    assert!(!complete::supports_potion(PotionId::Unknown("x")));
    // as_str is unique.
    assert_eq!(relics.iter().map(|id| id.as_str()).collect::<BTreeSet<_>>().len(), 179);
}

#[test]
fn m27_watcher_class_potions_are_class_restricted() {
    let catalog = Catalog::standard();
    let class_potions = POTIONS.iter().filter(|(_, _, class)| class.is_some()).count();
    assert_eq!(class_potions, 12);
    for id in [PotionId::BottledMiracle, PotionId::StancePotion, PotionId::Ambrosia] {
        for character in CHARACTERS {
            assert_eq!(catalog.potion_eligible_for(id, character), Some(character == Character::Watcher), "{id:?} {character:?} (G7)");
        }
    }
}

#[test]
fn m27_catalog_potion_table_matches_design() {
    let catalog = Catalog::standard();
    for (id, rarity, class) in POTIONS {
        assert_eq!(catalog.potion_rarity(id), Some(rarity), "{id:?}");
        for character in CHARACTERS {
            assert_eq!(catalog.potion_eligible_for(id, character), Some(class.is_none_or(|owner| owner == character)), "{id:?} {character:?}");
        }
    }
}

/// One `relics.json` row: catalog id, tier and character list (a minimal line scanner;
/// the file is pretty-printed with one field per line).
struct Row { id: String, tier: String, characters: Vec<String>, availability: String }

fn reference_rows() -> Vec<Row> {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/relics/relics.json")).unwrap();
    let value = |line: &str| line.split('"').nth(3).unwrap().to_string();
    let mut rows: Vec<Row> = Vec::new();
    let mut in_characters = false;
    for line in text.lines() {
        if line.starts_with("      \"id\": ") {
            rows.push(Row { id: value(line), tier: String::new(), characters: Vec::new(), availability: String::new() });
        } else if let Some(row) = rows.last_mut() {
            if line.starts_with("      \"tier\": ") { row.tier = value(line); }
            else if line.starts_with("      \"availability\": ") { row.availability = value(line); }
            else if line.starts_with("      \"characters\": [") { in_characters = true; }
            else if in_characters && line.trim_start().starts_with(']') { in_characters = false; }
            else if in_characters { row.characters.push(line.trim().trim_matches(|c| c == '"' || c == ',').to_string()); }
        }
    }
    rows
}

/// Catalog ids whose spelling differs from `as_str` upper-cased: Champion Belt
/// (`CHAMPION_BELT` / `ChampionsBelt`) and Mark of the Bloom (`MarkOfBloom`). The
/// Ring of the Snake, Paper Phrog and Paper Krane aliases are in the catalog `key`
/// (`snake_ring`, `paper_frog`, `paper_crane`), not the id.
fn reference_id(id: RelicId) -> String {
    match id {
        RelicId::ChampionsBelt => "CHAMPION_BELT".into(),
        RelicId::MarkOfBloom => "MARK_OF_THE_BLOOM".into(),
        _ => id.as_str().to_ascii_uppercase(),
    }
}

#[test]
fn m27_catalog_matches_reference() {
    let catalog = Catalog::standard();
    let rows = reference_rows();
    assert_eq!(rows.len(), 180);
    let relics = all_relics();
    let mut matched = BTreeSet::new();
    for row in &rows {
        let id = if row.id == "RED_CIRCLET" {
            RelicId::RedCirclet
        } else {
            let ids: Vec<_> = relics.iter().filter(|id| reference_id(**id) == row.id).collect();
            assert_eq!(ids.len(), 1, "{} maps to {ids:?}", row.id);
            *ids[0]
        };
        assert!(matched.insert(id));
        let tier = match row.tier.as_str() {
            "STARTER" => RelicTier::Starter, "COMMON" => RelicTier::Common, "UNCOMMON" => RelicTier::Uncommon,
            "RARE" => RelicTier::Rare, "BOSS" => RelicTier::Boss, "SHOP" => RelicTier::Shop, "SPECIAL" => RelicTier::Special,
            other => panic!("{} tier {other}", row.id),
        };
        assert_eq!(catalog.relic_tier(id), Some(tier), "{}", row.id);
        for character in CHARACTERS {
            let listed = row.characters.iter().any(|c| c.eq_ignore_ascii_case(&format!("{character:?}")));
            assert_eq!(catalog.relic_eligible_for(id, character), Some(listed), "{} {character:?}", row.id);
        }
        if id == RelicId::RedCirclet {
            assert_eq!(row.availability, "unobtainable", "F7 preserves fallback-only availability");
        } else {
            assert_ne!(row.availability, "unobtainable", "{}", row.id);
        }
    }
    // F7 Red Circlet adds one catalog row; ordinary union counts stay 179.
    assert_eq!(matched.len(), 180);
    let red = rows.iter().find(|row| row.id == "RED_CIRCLET").unwrap();
    assert_eq!(red.availability, "unobtainable");
    // The four aliases and Champion Belt map to their identities.
    for (catalog_id, id) in [("RING_OF_THE_SNAKE", RelicId::RingOfTheSnake), ("MARK_OF_THE_BLOOM", RelicId::MarkOfBloom),
        ("PAPER_PHROG", RelicId::PaperPhrog), ("PAPER_KRANE", RelicId::PaperKrane), ("CHAMPION_BELT", RelicId::ChampionsBelt)] {
        assert_eq!(reference_id(id), catalog_id);
        assert!(rows.iter().any(|row| row.id == catalog_id));
    }
}
