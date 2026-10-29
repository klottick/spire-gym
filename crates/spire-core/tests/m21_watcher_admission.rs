use spire_core::content::watcher as watcher_admission;

use spire_core::{
    combat::{m21_watcher_live_card, CardInstanceId, CombatCardRef},
    content::{
        CardColor, CardId, CardRarity, CardType, Catalog, Character, ContentProducer,
        NoteStorage, RegistryError,
    },
};

const EXPECTED_PERMANENT: &[CardId] = &[
    CardId::StrikePurple,
    CardId::DefendPurple,
    CardId::Eruption,
    CardId::Vigilance,
    CardId::EmptyFist,
    CardId::FlurryOfBlows,
    CardId::Prostrate,
    CardId::Worship,
    CardId::Brilliance,
    CardId::Devotion,
    CardId::ThirdEye,
    CardId::CutThroughFate,
    CardId::JustLucky,
    CardId::Weave,
    CardId::Foresight,
    CardId::Nirvana,
    CardId::WindmillStrike,
    CardId::Perseverance,
    CardId::SandsOfTime,
    CardId::Establishment,
    CardId::Meditate,
    CardId::CarveReality,
    CardId::DeceiveReality,
    CardId::MasterReality,
    CardId::BattleHymn,
    CardId::MentalFortress,
    CardId::Rushdown,
    CardId::LikeWater,
];

const EXPECTED_GENERATED_ONLY: &[CardId] = &[CardId::Miracle, CardId::Smite, CardId::Safety];

#[derive(Clone, Copy)]
struct MetadataExpectation {
    id: CardId,
    color: CardColor,
    rarity: CardRarity,
    card_type: CardType,
    storage: NoteStorage,
}

const METADATA: &[MetadataExpectation] = &[
    MetadataExpectation { id: CardId::StrikePurple, color: CardColor::Purple, rarity: CardRarity::Basic, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::DefendPurple, color: CardColor::Purple, rarity: CardRarity::Basic, card_type: CardType::Skill, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Eruption, color: CardColor::Purple, rarity: CardRarity::Basic, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Vigilance, color: CardColor::Purple, rarity: CardRarity::Basic, card_type: CardType::Skill, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::EmptyFist, color: CardColor::Purple, rarity: CardRarity::Common, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::FlurryOfBlows, color: CardColor::Purple, rarity: CardRarity::Common, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Prostrate, color: CardColor::Purple, rarity: CardRarity::Common, card_type: CardType::Skill, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Worship, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Skill, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Brilliance, color: CardColor::Purple, rarity: CardRarity::Rare, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Devotion, color: CardColor::Purple, rarity: CardRarity::Rare, card_type: CardType::Power, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::ThirdEye, color: CardColor::Purple, rarity: CardRarity::Common, card_type: CardType::Skill, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::CutThroughFate, color: CardColor::Purple, rarity: CardRarity::Common, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::JustLucky, color: CardColor::Purple, rarity: CardRarity::Common, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Weave, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Foresight, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Power, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Nirvana, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Power, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::WindmillStrike, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Perseverance, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Skill, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::SandsOfTime, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Establishment, color: CardColor::Purple, rarity: CardRarity::Rare, card_type: CardType::Power, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Meditate, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Skill, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::CarveReality, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Attack, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::DeceiveReality, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Skill, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::MasterReality, color: CardColor::Purple, rarity: CardRarity::Rare, card_type: CardType::Power, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::BattleHymn, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Power, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::MentalFortress, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Power, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Rushdown, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Power, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::LikeWater, color: CardColor::Purple, rarity: CardRarity::Uncommon, card_type: CardType::Power, storage: NoteStorage::RemovablePermanent },
    MetadataExpectation { id: CardId::Miracle, color: CardColor::Colorless, rarity: CardRarity::Special, card_type: CardType::Skill, storage: NoteStorage::TemporaryOnly },
    MetadataExpectation { id: CardId::Smite, color: CardColor::Colorless, rarity: CardRarity::Special, card_type: CardType::Attack, storage: NoteStorage::TemporaryOnly },
    MetadataExpectation { id: CardId::Safety, color: CardColor::Colorless, rarity: CardRarity::Special, card_type: CardType::Skill, storage: NoteStorage::TemporaryOnly },
];

#[derive(Clone, Copy)]
struct ConstructorExpectation {
    id: CardId,
    base_cost: i32,
    upgraded_cost: i32,
    base_damage: Option<i32>,
    upgraded_damage: Option<i32>,
    base_block: Option<i32>,
    upgraded_block: Option<i32>,
    upgraded_innate: bool,
}

const CONSTRUCTORS: &[ConstructorExpectation] = &[
    ConstructorExpectation { id: CardId::StrikePurple, base_cost: 1, upgraded_cost: 1, base_damage: Some(6), upgraded_damage: Some(9), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::DefendPurple, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: Some(5), upgraded_block: Some(8), upgraded_innate: false },
    ConstructorExpectation { id: CardId::Eruption, base_cost: 2, upgraded_cost: 1, base_damage: Some(9), upgraded_damage: Some(9), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::Vigilance, base_cost: 2, upgraded_cost: 2, base_damage: None, upgraded_damage: None, base_block: Some(8), upgraded_block: Some(12), upgraded_innate: false },
    ConstructorExpectation { id: CardId::EmptyFist, base_cost: 1, upgraded_cost: 1, base_damage: Some(9), upgraded_damage: Some(14), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::FlurryOfBlows, base_cost: 0, upgraded_cost: 0, base_damage: Some(4), upgraded_damage: Some(6), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::Prostrate, base_cost: 0, upgraded_cost: 0, base_damage: None, upgraded_damage: None, base_block: Some(4), upgraded_block: Some(4), upgraded_innate: false },
    ConstructorExpectation { id: CardId::Worship, base_cost: 2, upgraded_cost: 2, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::Brilliance, base_cost: 1, upgraded_cost: 1, base_damage: Some(12), upgraded_damage: Some(16), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::Devotion, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::ThirdEye, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: Some(7), upgraded_block: Some(9), upgraded_innate: false },
    ConstructorExpectation { id: CardId::CutThroughFate, base_cost: 1, upgraded_cost: 1, base_damage: Some(7), upgraded_damage: Some(9), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::JustLucky, base_cost: 0, upgraded_cost: 0, base_damage: Some(3), upgraded_damage: Some(4), base_block: Some(2), upgraded_block: Some(3), upgraded_innate: false },
    ConstructorExpectation { id: CardId::Weave, base_cost: 0, upgraded_cost: 0, base_damage: Some(4), upgraded_damage: Some(6), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::Foresight, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::Nirvana, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::WindmillStrike, base_cost: 2, upgraded_cost: 2, base_damage: Some(7), upgraded_damage: Some(10), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::Perseverance, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: Some(5), upgraded_block: Some(7), upgraded_innate: false },
    ConstructorExpectation { id: CardId::SandsOfTime, base_cost: 4, upgraded_cost: 4, base_damage: Some(20), upgraded_damage: Some(26), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::Establishment, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: true },
    ConstructorExpectation { id: CardId::Meditate, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::CarveReality, base_cost: 1, upgraded_cost: 1, base_damage: Some(6), upgraded_damage: Some(10), base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::DeceiveReality, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: Some(4), upgraded_block: Some(7), upgraded_innate: false },
    ConstructorExpectation { id: CardId::MasterReality, base_cost: 1, upgraded_cost: 0, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::BattleHymn, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: true },
    ConstructorExpectation { id: CardId::MentalFortress, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::Rushdown, base_cost: 1, upgraded_cost: 0, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: false },
    ConstructorExpectation { id: CardId::LikeWater, base_cost: 1, upgraded_cost: 1, base_damage: None, upgraded_damage: None, base_block: None, upgraded_block: None, upgraded_innate: false },
];

fn record_mismatch<T: std::fmt::Debug + PartialEq>(
    mismatches: &mut Vec<String>,
    label: impl std::fmt::Display,
    actual: T,
    expected: T,
) {
    if actual != expected {
        mismatches.push(format!("{label}: expected {expected:?}, got {actual:?}"));
    }
}

#[test]
fn m21_watcher_admission_is_exact_and_finite() {
    use watcher_admission::{supports_generated, supports_permanent, M21_WATCHER_CARDS, M21_WATCHER_GENERATED};

    assert_eq!(M21_WATCHER_CARDS, EXPECTED_PERMANENT);
    assert_eq!(M21_WATCHER_GENERATED, EXPECTED_GENERATED_ONLY);
    assert_eq!(M21_WATCHER_CARDS.len(), 28);
    assert_eq!(M21_WATCHER_GENERATED.len(), 3);
    assert_eq!(M21_WATCHER_CARDS.iter().copied().collect::<std::collections::HashSet<_>>().len(), 28);
    assert_eq!(M21_WATCHER_GENERATED.iter().copied().collect::<std::collections::HashSet<_>>().len(), 3);

    for id in EXPECTED_PERMANENT {
        assert!(supports_permanent(*id), "expected permanent admission for {id:?}");
        assert!(supports_generated(*id), "permanent kinds remain stat-equivalent-copy eligible: {id:?}");
    }
    for id in EXPECTED_GENERATED_ONLY {
        assert!(!supports_permanent(*id), "generated-only kind must not enter permanent deck: {id:?}");
        assert!(supports_generated(*id), "expected generated admission for {id:?}");
    }
    for excluded in [CardId::EmptyBody, CardId::EmptyMind, CardId::Miracle, CardId::Unknown("outside_m21")] {
        assert!(!supports_permanent(excluded), "unexpected permanent admission for {excluded:?}");
    }
    for excluded in [CardId::EmptyBody, CardId::EmptyMind, CardId::Parasite, CardId::Wound, CardId::Dazed, CardId::Unknown("outside_m21")] {
        assert!(!supports_generated(excluded), "unexpected generated admission for {excluded:?}");
    }
}

#[test]
fn m21_watcher_catalog_metadata_matches_source_rarity_and_card_kinds() {
    let catalog = Catalog::standard();
    let mut mismatches = Vec::new();

    for expected in METADATA {
        match catalog.card(expected.id) {
            Some(actual) => {
                record_mismatch(&mut mismatches, format!("{:?}.color", expected.id), actual.color, expected.color);
                record_mismatch(&mut mismatches, format!("{:?}.rarity", expected.id), actual.rarity, expected.rarity);
                record_mismatch(&mut mismatches, format!("{:?}.card_type", expected.id), actual.card_type, expected.card_type);
                record_mismatch(&mut mismatches, format!("{:?}.note_storage", expected.id), actual.note_storage, expected.storage);
                record_mismatch(&mut mismatches, format!("{:?}.definition_only", expected.id), actual.definition_only, false);
                record_mismatch(&mut mismatches, format!("{:?}.blue_metadata", expected.id), actual.blue_metadata.is_some(), false);
            }
            None => mismatches.push(format!("{:?}: missing catalog row", expected.id)),
        }

        record_mismatch(
            &mut mismatches,
            format!("{:?}.ordinary_reward", expected.id),
            catalog.eligibility(expected.id, ContentProducer::OrdinaryReward, Character::Watcher),
            Ok(expected.id == CardId::EmptyFist),
        );
        record_mismatch(
            &mut mismatches,
            format!("{:?}.shard_ordinary_reward", expected.id),
            catalog.eligibility(expected.id, ContentProducer::ShardOrdinaryReward, Character::Watcher),
            Ok(expected.id == CardId::EmptyFist),
        );
        record_mismatch(
            &mut mismatches,
            format!("{:?}.note_for_yourself", expected.id),
            catalog.eligibility(
                expected.id,
                ContentProducer::NoteForYourself,
                Character::Watcher,
            ),
            Ok(expected.storage == NoteStorage::RemovablePermanent),
        );
        record_mismatch(
            &mut mismatches,
            format!("{:?}.pure_water", expected.id),
            catalog.eligibility(expected.id, ContentProducer::PureWater, Character::Watcher),
            Ok(expected.id == CardId::Miracle),
        );
    }

    assert_eq!(
        catalog.selectable_pool(ContentProducer::OrdinaryReward, Character::Watcher),
        Err(RegistryError::IncompletePool),
        "finite admission must not turn into an ordinary reward selector",
    );
    assert!(mismatches.is_empty(), "source-backed catalog mismatch:\n{}", mismatches.join("\n"));
}

#[test]
fn m21_watcher_permanent_constructor_pairs_match_approved_values() {
    // Explicit M21 profile state; the shared bare factory remains frozen.
    let mut mismatches = Vec::new();

    for (ordinal, expected) in CONSTRUCTORS.iter().enumerate() {
        let base_instance = CardInstanceId(10_000 + ordinal as u64 * 4);
        let upgraded_instance = CardInstanceId(base_instance.0 + 1);
        let base = m21_watcher_live_card(expected.id, base_instance, 0).unwrap();
        let upgraded = m21_watcher_live_card(expected.id, upgraded_instance, 1).unwrap();

        for (label, card, cost, damage, block, innate) in [
            ("base", &base, expected.base_cost, expected.base_damage, expected.base_block, false),
            ("upgraded", &upgraded, expected.upgraded_cost, expected.upgraded_damage, expected.upgraded_block, expected.upgraded_innate),
        ] {
            let prefix = format!("{:?}.{label}", expected.id);
            record_mismatch(&mut mismatches, format!("{prefix}.upgrade_count"), card.upgrades, if label == "base" { 0 } else { 1 });
            record_mismatch(&mut mismatches, format!("{prefix}.mutable_combat_cost"), card.local.mutable_combat_cost, Some(cost));
            record_mismatch(&mut mismatches, format!("{prefix}.turn_cost"), card.local.turn_cost, Some(cost));
            record_mismatch(&mut mismatches, format!("{prefix}.base_damage"), card.local.base_damage, damage);
            record_mismatch(&mut mismatches, format!("{prefix}.base_block"), card.local.base_block, block);
            record_mismatch(&mut mismatches, format!("{prefix}.innate"), card.local.innate, innate);
            record_mismatch(&mut mismatches, format!("{prefix}.exhaust"), card.local.exhaust_on_finish, false);
            record_mismatch(&mut mismatches, format!("{prefix}.ethereal"), card.local.ethereal, false);
        }
    }

    // CombatCardData and the public card view in this input do not expose a
    // Retain flag, and Watcher catalog rows have no target field. Those source
    // facts are retained in METADATA and the paired research; no API is added
    // here to make an inaccessible assertion compile.
    assert!(mismatches.is_empty(), "constructor pair mismatch:\n{}", mismatches.join("\n"));
}

#[test]
fn m21_watcher_stat_equivalent_copies_keep_the_paired_constructor_state() {
    // These assertions describe the M21 profile's desired source-equivalent
    // state. The pinned partial input has no public M21 constructor route, so
    // legacy-constructor REDs here are retained for main's integration join.
    let mut mismatches = Vec::new();

    for (ordinal, expected) in CONSTRUCTORS.iter().enumerate() {
        let source_instance = CardInstanceId(20_000 + ordinal as u64 * 3);
        let copy_instance = CardInstanceId(source_instance.0 + 1);
        let source = m21_watcher_live_card(expected.id, source_instance, 1).unwrap();
        let copy = source.stat_equivalent_copy(copy_instance);
        record_mismatch(&mut mismatches, format!("{:?}.copy.id", expected.id), copy.id, expected.id);
        record_mismatch(&mut mismatches, format!("{:?}.copy.upgrades", expected.id), copy.upgrades, 1);
        record_mismatch(&mut mismatches, format!("{:?}.copy.instance", expected.id), copy.instance, copy_instance);
        record_mismatch(&mut mismatches, format!("{:?}.copy.origin", expected.id), copy.origin, CombatCardRef::CombatLocal(copy_instance.0));
        record_mismatch(&mut mismatches, format!("{:?}.copy.cost", expected.id), copy.local.mutable_combat_cost, Some(expected.upgraded_cost));
        record_mismatch(&mut mismatches, format!("{:?}.copy.turn_cost", expected.id), copy.local.turn_cost, Some(expected.upgraded_cost));
        record_mismatch(&mut mismatches, format!("{:?}.copy.base_damage", expected.id), copy.local.base_damage, expected.upgraded_damage);
        record_mismatch(&mut mismatches, format!("{:?}.copy.base_block", expected.id), copy.local.base_block, expected.upgraded_block);
        record_mismatch(&mut mismatches, format!("{:?}.copy.innate", expected.id), copy.local.innate, expected.upgraded_innate);
        record_mismatch(&mut mismatches, format!("{:?}.copy.exhaust", expected.id), copy.local.exhaust_on_finish, false);
        record_mismatch(&mut mismatches, format!("{:?}.copy.ethereal", expected.id), copy.local.ethereal, false);
    }

    for (ordinal, (id, cost, damage, block)) in [
        (CardId::Miracle, 0, None, None),
        (CardId::Smite, 1, Some(16), None),
        (CardId::Safety, 1, None, Some(16)),
    ].into_iter().enumerate() {
        let source_instance = CardInstanceId(30_000 + ordinal as u64 * 3);
        let copy_instance = CardInstanceId(source_instance.0 + 1);
        let source = m21_watcher_live_card(id, source_instance, 1).unwrap();
        let copy = source.stat_equivalent_copy(copy_instance);
        record_mismatch(&mut mismatches, format!("{id:?}.generated_copy.cost"), copy.local.mutable_combat_cost, Some(cost));
        record_mismatch(&mut mismatches, format!("{id:?}.generated_copy.turn_cost"), copy.local.turn_cost, Some(cost));
        record_mismatch(&mut mismatches, format!("{id:?}.generated_copy.damage"), copy.local.base_damage, damage);
        record_mismatch(&mut mismatches, format!("{id:?}.generated_copy.block"), copy.local.base_block, block);
        record_mismatch(&mut mismatches, format!("{id:?}.generated_copy.exhaust"), copy.local.exhaust_on_finish, true);
        record_mismatch(&mut mismatches, format!("{id:?}.generated_copy.ethereal"), copy.local.ethereal, false);
    }

    // Costs, stats and identity come from the actual M21 initializer.
    assert!(mismatches.is_empty(), "stat-equivalent copy mismatch:\n{}", mismatches.join("\n"));
}

#[test]
fn m21_watcher_fixed_action_constants_are_not_fabricated_card_magic_metadata(){
    use spire_core::combat::{m21_watcher_live_card,CardInstanceId};
    for id in [CardId::MasterReality,CardId::CarveReality,CardId::DeceiveReality,CardId::SandsOfTime,CardId::Miracle]{for up in 0..=1{
        assert_eq!(m21_watcher_live_card(id,CardInstanceId(71),up).unwrap().local.magic_number,None,"{} / {up}",id.as_str());
    }}
    for id in [CardId::BattleHymn,CardId::Establishment]{assert_eq!(m21_watcher_live_card(id,CardInstanceId(71),0).unwrap().local.magic_number,Some(1));}
}
