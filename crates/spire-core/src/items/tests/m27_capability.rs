//! M27 complete item capability (plan Task M1, design §5.1–§5.3).
//!
//! Literal sizes are design §4.4's; finite unions are the accepted M10–M25 constants.
use super::*;
use crate::content::{complete, Catalog, Character, PotionId, RelicId, RelicTier};

pub(crate) const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn starter(character: Character) -> RelicId {
    match character {
        Character::Ironclad => RelicId::BurningBlood, Character::Silent => RelicId::RingOfTheSnake,
        Character::Defect => RelicId::CrackedCore, Character::Watcher => RelicId::PureWater,
    }
}
fn class_potion(character: Character) -> PotionId {
    match character {
        Character::Ironclad => PotionId::BloodPotion, Character::Silent => PotionId::PoisonPotion,
        Character::Defect => PotionId::FocusPotion, Character::Watcher => PotionId::BottledMiracle,
    }
}
/// Another class's relic and potion for each character.
fn foreign(character: Character) -> (RelicId, PotionId) {
    match character {
        Character::Ironclad => (RelicId::TwistedFunnel, PotionId::PoisonPotion),
        Character::Silent => (RelicId::RedSkull, PotionId::FocusPotion),
        Character::Defect => (RelicId::Damaru, PotionId::BottledMiracle),
        Character::Watcher => (RelicId::DataDisk, PotionId::BloodPotion),
    }
}
/// The 20 new Common relics of design §4.2, all shared.
const NEW_SHARED: [RelicId; 20] = [RelicId::Akabeko, RelicId::AncientTeaSet, RelicId::ArtOfWar, RelicId::TheBoot,
    RelicId::CentennialPuzzle, RelicId::CeramicFish, RelicId::DreamCatcher, RelicId::HappyFlower, RelicId::JuzuBracelet,
    RelicId::MawBank, RelicId::MealTicket, RelicId::Nunchaku, RelicId::Omamori, RelicId::PenNib, RelicId::PreservedInsect,
    RelicId::RegalPillow, RelicId::SmilingMask, RelicId::TinyChest, RelicId::WarPaint, RelicId::Whetstone];

pub(crate) fn complete_context(character: Character, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> CombatItemContext {
    crate::run::complete_heart_context(CombatItemContext::new(relics, potions), character).with_complete_items()
}
pub(crate) fn inventory(character: Character) -> (Vec<RelicId>, Vec<Option<PotionId>>) {
    let mut relics = vec![starter(character)];
    relics.extend(NEW_SHARED);
    (relics, vec![Some(PotionId::FirePotion), Some(PotionId::BlockPotion), Some(class_potion(character))])
}

#[test]
fn m27_identity_complete_unions() {
    let all: std::collections::BTreeSet<_> = CHARACTERS.iter().flat_map(|c| complete::relic_union(*c)).collect();
    // F7 Red Circlet remains outside the ordinary complete unions.
    assert_eq!(all.len(), 179);
    assert!(all.iter().all(|id| complete::supports_relic(*id)));
    assert!(!complete::supports_relic(RelicId::Unknown("x")));
    assert_eq!(CHARACTERS.map(|c| complete::relic_union(c).len()), [150, 150, 148, 148]);
    let shared = all.iter().filter(|id| CHARACTERS.iter().all(|c| complete::relic_union(*c).contains(id))).count();
    assert_eq!(shared, 139);
    assert_eq!(CHARACTERS.map(|c| complete::potion_union(c).len()), [33; 4]);
    assert!(!complete::supports_potion(PotionId::Unknown("x")));
    let shared_potions = CHARACTERS.iter().flat_map(|c| complete::potion_union(*c))
        .filter(|id| CHARACTERS.iter().all(|c| complete::potion_union(*c).contains(id))).collect::<Vec<_>>();
    assert_eq!(shared_potions.len(), 30 * 4, "each shared potion appears once per character");
    // Every accepted finite union is unchanged in size and inside its character's complete union.
    let relic_unions: [(&str, &[RelicId], Character, usize); 11] = [
        ("M10 TRAINING_RELICS", crate::run::TRAINING_RELICS, Character::Ironclad, 30),
        ("M12", crate::run::rooms::m12_content::M12_SILENT_ACT1_RELICS, Character::Silent, 13),
        ("M14", crate::run::rooms::m14_content::M14_SILENT_ACT3_RELICS, Character::Silent, 18),
        ("M15 M15_RELICS28", crate::run::M15_SILENT_HEART_RELICS, Character::Silent, 28),
        ("M17", crate::run::rooms::m17_content::M17_DEFECT_ACT1_RELICS, Character::Defect, 14),
        ("M19", crate::run::rooms::m19_content::M19_DEFECT_ACT3_RELICS, Character::Defect, 19),
        ("M20", crate::run::rooms::m20_content::M20_DEFECT_HEART_RELICS, Character::Defect, 26),
        ("M21", crate::run::M21_WATCHER_RELICS, Character::Watcher, 7),
        ("M22", crate::run::rooms::m22_content::M22_WATCHER_ACT1_RELICS, Character::Watcher, 18),
        ("M24", crate::run::rooms::m24_content::M24_WATCHER_ACT3_RELICS, Character::Watcher, 19),
        ("M25", crate::run::rooms::m25_content::M25_WATCHER_HEART_RELICS, Character::Watcher, 23),
    ];
    for (name, union, character, size) in relic_unions {
        assert_eq!(union.len(), size, "{name}");
        let complete = complete::relic_union(character);
        assert!(union.iter().all(|id| complete.contains(id)), "{name} not inside the {character:?} complete union");
    }
    let potion_unions: [(&str, &[PotionId], Character); 7] = [
        ("M10 TRAINING_POTIONS", crate::run::TRAINING_POTIONS, Character::Ironclad),
        ("M12", crate::run::rooms::m12_content::M12_SILENT_ACT1_POTIONS, Character::Silent),
        ("M15", crate::run::M15_SILENT_HEART_POTIONS, Character::Silent),
        ("M17", crate::run::rooms::m17_content::M17_DEFECT_ACT1_POTIONS, Character::Defect),
        ("M20", crate::run::rooms::m20_content::M20_DEFECT_HEART_POTIONS, Character::Defect),
        ("M21", crate::run::M21_WATCHER_POTIONS, Character::Watcher),
        ("M22", crate::run::rooms::m22_content::M22_WATCHER_ACT1_POTIONS, Character::Watcher),
    ];
    for (name, union, character) in potion_unions {
        let complete = complete::potion_union(character);
        assert!(union.iter().all(|id| complete.contains(id)), "{name} not inside the {character:?} complete union");
    }
}

#[test]
fn m27_red_circlet_catalog_row_not_in_union() {
    let catalog = Catalog::standard();
    let row = catalog.relic(RelicId::RedCirclet).expect("F7 Red Circlet catalog row");
    assert!(!row.definition_only);
    assert_eq!(catalog.relic_tier(RelicId::RedCirclet), Some(RelicTier::Special));
    assert_eq!(RelicId::RedCirclet.as_str(), "red_circlet");
    assert!(!complete::supports_relic(RelicId::RedCirclet), "Red Circlet has no effect handler");
    for character in CHARACTERS {
        assert_eq!(catalog.relic_eligible_for(RelicId::RedCirclet, character), Some(true));
        assert!(!complete::relic_union(character).contains(&RelicId::RedCirclet));
        let context = complete_context(character, vec![RelicId::RedCirclet], vec![None; 3]);
        assert_eq!(validate_complete_inventory(&context, character), Ok(()), "F7 owned fallback for {character:?}");
    }
}

#[test]
fn m27_capability_combat_admission() {
    for character in CHARACTERS {
        let (relics, potions) = inventory(character);
        assert_eq!(validate_combat_inventory(&complete_context(character, relics.clone(), potions.clone()), character), Ok(()), "{character:?}");
        let (other_relic, other_potion) = foreign(character);
        let mut with = relics.clone(); with.push(other_relic);
        assert_eq!(validate_combat_inventory(&complete_context(character, with, potions.clone()), character), Err(RunError::InvalidAction), "{character:?} foreign relic");
        let mut duplicate = relics.clone(); duplicate.push(RelicId::PenNib);
        assert_eq!(validate_combat_inventory(&complete_context(character, duplicate, potions.clone()), character), Err(RunError::InvalidAction), "{character:?} duplicate");
        let mut bad_potions = potions.clone(); bad_potions[0] = Some(other_potion);
        assert_eq!(validate_combat_inventory(&complete_context(character, relics.clone(), bad_potions), character), Err(RunError::InvalidAction), "{character:?} foreign potion");
        let mut four = potions.clone(); four.push(None);
        assert_eq!(validate_combat_inventory(&complete_context(character, relics.clone(), four), character), Err(RunError::InvalidAction), "{character:?} 4 slots");
        let mut belt = relics.clone(); belt.push(RelicId::PotionBelt);
        let mut five = potions.clone(); five.extend([None, None]);
        assert_eq!(validate_combat_inventory(&complete_context(character, belt, five), character), Ok(()), "{character:?} belt 5 slots");
        let counted = |id: RelicId, value: i32| complete_context(character, relics.clone(), potions.clone())
            .with_counters([(id, value)].into_iter().collect());
        assert_eq!(validate_combat_inventory(&counted(RelicId::PenNib, 9), character), Ok(()));
        assert_eq!(validate_combat_inventory(&counted(RelicId::PenNib, 10), character), Err(RunError::InvalidAction), "Pen Nib 10");
        assert_eq!(validate_combat_inventory(&counted(RelicId::Girya, 1), character), Err(RunError::InvalidAction), "unowned counter");
        assert_eq!(validate_combat_inventory(&counted(RelicId::Akabeko, 1), character), Err(RunError::InvalidAction), "counterless relic");
        // Complete items without complete cards is an invariant error.
        let bare = CombatItemContext::new(relics.clone(), potions.clone()).with_complete_items();
        assert!(matches!(validate_combat_inventory(&bare, character), Err(RunError::Unsupported(_))), "{character:?}");
        // The starter with its boss replacement.
        let replacement = match character { Character::Ironclad => RelicId::BlackBlood, Character::Silent => RelicId::RingOfTheSerpent,
            Character::Defect => RelicId::FrozenCore, Character::Watcher => RelicId::HolyWater };
        let mut both = relics.clone(); both.push(replacement);
        assert_eq!(validate_combat_inventory(&complete_context(character, both, potions.clone()), character), Err(RunError::InvalidAction), "{character:?} starter + boss");
    }
    // Without the flag the finite rejections and text are unchanged.
    let starter_only = |c| CombatItemContext::new(vec![starter(c), RelicId::Akabeko], vec![None; 3]);
    assert_eq!(validate_combat_inventory(&starter_only(Character::Ironclad), Character::Ironclad), Err(RunError::Unsupported("unimplemented combat relic")));
    assert_eq!(validate_combat_inventory(&starter_only(Character::Silent).with_m15_profile(), Character::Silent), Err(RunError::Unsupported("unimplemented combat relic")));
    assert_eq!(validate_combat_inventory(&starter_only(Character::Defect).with_m20_profile(), Character::Defect), Err(RunError::Unsupported("unimplemented combat relic")));
    assert_eq!(validate_combat_inventory(&starter_only(Character::Watcher).with_m25_profile(), Character::Watcher), Err(RunError::InvalidAction));
    let frozen = CombatItemContext::new(vec![RelicId::BurningBlood], vec![None; 3]);
    assert!(!format!("{frozen:?}").contains("complete_items"));
    assert!(format!("{:?}", frozen.clone().with_complete_cards().with_complete_items()).contains("complete_items: true"));
    assert_eq!(format!("{frozen:?}"), format!("{:?}", CombatItemContext::new(vec![RelicId::BurningBlood], vec![None; 3])));
}
