use super::{Character, PotionId, RelicId};

#[derive(Clone, Copy, Debug)]
pub struct RelicDefinition { pub id: RelicId, pub definition_only: bool }
#[derive(Clone, Copy, Debug)]
pub struct PotionDefinition { pub id: PotionId, pub definition_only: bool }

/// Catalog tiers for registered relic metadata; this is not a selector pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelicTier { Starter, Common, Uncommon, Rare, Shop, Boss }

/// Rarity metadata for registered potions; this is not a selector pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PotionRarity { Common, Uncommon, Rare }

pub(crate) const SILENT_RELIC_TIERS: &[(RelicId, RelicTier)] = &[
    (RelicId::RingOfTheSnake, RelicTier::Starter),
    (RelicId::SneckoSkull, RelicTier::Common),
    (RelicId::NinjaScroll, RelicTier::Uncommon),
    (RelicId::PaperKrane, RelicTier::Uncommon),
    (RelicId::TheSpecimen, RelicTier::Rare),
    (RelicId::Tingsha, RelicTier::Rare),
    (RelicId::ToughBandages, RelicTier::Rare),
    (RelicId::TwistedFunnel, RelicTier::Shop),
    (RelicId::HoveringKite, RelicTier::Boss),
    (RelicId::RingOfTheSerpent, RelicTier::Boss),
    (RelicId::WristBlade, RelicTier::Boss),
];

pub(crate) const DEFECT_RELIC_TIERS: &[(RelicId, RelicTier)] = &[
    (RelicId::CrackedCore, RelicTier::Starter),
    (RelicId::DataDisk, RelicTier::Common),
    (RelicId::GoldPlatedCables, RelicTier::Uncommon),
    (RelicId::EmotionChip, RelicTier::Rare),
    (RelicId::SymbioticVirus, RelicTier::Uncommon),
    (RelicId::FrozenCore, RelicTier::Boss),
    (RelicId::Inserter, RelicTier::Boss),
    (RelicId::NuclearBattery, RelicTier::Boss),
    (RelicId::RunicCapacitor, RelicTier::Shop),
];

pub(crate) const EXCLUSIVE_RELIC_CHARACTERS: &[(RelicId, Character)] = &[
    (RelicId::HolyWater, Character::Watcher),
    (RelicId::CrackedCore, Character::Defect),
    (RelicId::DataDisk, Character::Defect),
    (RelicId::GoldPlatedCables, Character::Defect),
    (RelicId::EmotionChip, Character::Defect),
    (RelicId::SymbioticVirus, Character::Defect),
    (RelicId::FrozenCore, Character::Defect),
    (RelicId::Inserter, Character::Defect),
    (RelicId::NuclearBattery, Character::Defect),
    (RelicId::RunicCapacitor, Character::Defect),
];

pub(crate) const SILENT_POTION_RARITIES: &[(PotionId, PotionRarity)] = &[
    (PotionId::PoisonPotion, PotionRarity::Common),
    (PotionId::CunningPotion, PotionRarity::Uncommon),
    (PotionId::GhostInAJar, PotionRarity::Rare),
    (PotionId::BlockPotion, PotionRarity::Common),
    (PotionId::DexterityPotion, PotionRarity::Common),
    (PotionId::EnergyPotion, PotionRarity::Common),
    (PotionId::ExplosivePotion, PotionRarity::Common),
    (PotionId::FirePotion, PotionRarity::Common),
    (PotionId::StrengthPotion, PotionRarity::Common),
    (PotionId::SwiftPotion, PotionRarity::Common),
    (PotionId::WeakPotion, PotionRarity::Common),
    (PotionId::FearPotion, PotionRarity::Common),
    (PotionId::AttackPotion, PotionRarity::Common),
    (PotionId::SkillPotion, PotionRarity::Common),
    (PotionId::PowerPotion, PotionRarity::Common),
    (PotionId::ColorlessPotion, PotionRarity::Common),
    (PotionId::FlexPotion, PotionRarity::Common),
    (PotionId::SpeedPotion, PotionRarity::Common),
    (PotionId::BlessingOfTheForge, PotionRarity::Common),
    (PotionId::RegenPotion, PotionRarity::Uncommon),
    (PotionId::AncientPotion, PotionRarity::Uncommon),
    (PotionId::LiquidBronze, PotionRarity::Uncommon),
    (PotionId::GamblersBrew, PotionRarity::Uncommon),
    (PotionId::EssenceOfSteel, PotionRarity::Uncommon),
    (PotionId::DuplicationPotion, PotionRarity::Uncommon),
    (PotionId::DistilledChaos, PotionRarity::Uncommon),
    (PotionId::LiquidMemories, PotionRarity::Uncommon),
    (PotionId::CultistPotion, PotionRarity::Rare),
    (PotionId::FruitJuice, PotionRarity::Rare),
    (PotionId::SneckoOil, PotionRarity::Rare),
    (PotionId::FairyPotion, PotionRarity::Rare),
    (PotionId::SmokeBomb, PotionRarity::Rare),
    (PotionId::EntropicBrew, PotionRarity::Rare),
];

pub(crate) const DEFECT_POTION_RARITIES: &[(PotionId, PotionRarity)] = &[
    (PotionId::FocusPotion, PotionRarity::Common),
    (PotionId::PotionOfCapacity, PotionRarity::Uncommon),
    (PotionId::EssenceOfDarkness, PotionRarity::Rare),
];

pub(crate) const EXCLUSIVE_POTION_CHARACTERS: &[(PotionId, Character)] = &[
    (PotionId::BloodPotion, Character::Ironclad),
    (PotionId::Elixir, Character::Ironclad),
    (PotionId::HeartOfIron, Character::Ironclad),
    (PotionId::PoisonPotion, Character::Silent),
    (PotionId::CunningPotion, Character::Silent),
    (PotionId::GhostInAJar, Character::Silent),
    (PotionId::FocusPotion, Character::Defect),
    (PotionId::PotionOfCapacity, Character::Defect),
    (PotionId::EssenceOfDarkness, Character::Defect),
];

pub(crate) const RELICS: &[RelicDefinition] = &[
    RelicDefinition{id:RelicId::HolyWater,definition_only:false},
    RelicDefinition { id:RelicId::ChemicalX,definition_only:false },
    RelicDefinition{id:RelicId::Vajra,definition_only:false},
    RelicDefinition{id:RelicId::OddlySmoothStone,definition_only:false},
    RelicDefinition{id:RelicId::BagOfPreparation,definition_only:false},
    RelicDefinition{id:RelicId::BagOfMarbles,definition_only:false},
    RelicDefinition{id:RelicId::BronzeScales,definition_only:false},
    RelicDefinition{id:RelicId::RedMask,definition_only:false},
    RelicDefinition{id:RelicId::RedSkull,definition_only:false},
    RelicDefinition{id:RelicId::PaperPhrog,definition_only:false},
    RelicDefinition{id:RelicId::ChampionsBelt,definition_only:false},
    RelicDefinition{id:RelicId::CharonsAshes,definition_only:false},
    RelicDefinition{id:RelicId::SelfFormingClay,definition_only:false},
    RelicDefinition{id:RelicId::Orichalcum,definition_only:false},
    RelicDefinition{id:RelicId::Brimstone,definition_only:false},
    RelicDefinition{id:RelicId::MarkOfPain,definition_only:false},
    RelicDefinition{id:RelicId::RunicCube,definition_only:false},
    RelicDefinition{id:RelicId::SneckoSkull,definition_only:false},
    RelicDefinition{id:RelicId::NinjaScroll,definition_only:false},
    RelicDefinition{id:RelicId::PaperKrane,definition_only:false},
    RelicDefinition{id:RelicId::TheSpecimen,definition_only:false},
    RelicDefinition{id:RelicId::Tingsha,definition_only:false},
    RelicDefinition{id:RelicId::ToughBandages,definition_only:false},
    RelicDefinition{id:RelicId::TwistedFunnel,definition_only:false},
    RelicDefinition{id:RelicId::HoveringKite,definition_only:false},
    RelicDefinition{id:RelicId::RingOfTheSerpent,definition_only:false},
    RelicDefinition{id:RelicId::WristBlade,definition_only:false},
    RelicDefinition { id: RelicId::BlackBlood, definition_only: false },
    RelicDefinition { id: RelicId::EmptyCage, definition_only: false },
    RelicDefinition { id: RelicId::SlaversCollar, definition_only: false },
    RelicDefinition { id: RelicId::NeowsLament, definition_only: false },
    RelicDefinition { id: RelicId::BurningBlood, definition_only: true },
    RelicDefinition { id: RelicId::RingOfTheSnake, definition_only: true },
    RelicDefinition { id: RelicId::CrackedCore, definition_only: true },
    RelicDefinition { id: RelicId::PureWater, definition_only: true },
    RelicDefinition { id: RelicId::BloodVial, definition_only: true },
    RelicDefinition { id: RelicId::Anchor, definition_only: true },
    RelicDefinition { id: RelicId::Lantern, definition_only: true },
    RelicDefinition { id: RelicId::Strawberry, definition_only: true },
    RelicDefinition { id: RelicId::Pear, definition_only: true },
    RelicDefinition { id: RelicId::Mango, definition_only: true },
    RelicDefinition { id: RelicId::SacredBark, definition_only: true },
    RelicDefinition { id: RelicId::MarkOfBloom, definition_only: true },
    RelicDefinition { id: RelicId::MagicFlower, definition_only: true },
    RelicDefinition { id: RelicId::ToyOrnithopter, definition_only: true },
    RelicDefinition { id: RelicId::Sozu, definition_only: false },
    RelicDefinition { id: RelicId::PotionBelt, definition_only: false },
    RelicDefinition { id: RelicId::DataDisk, definition_only: false },
    RelicDefinition { id: RelicId::GoldPlatedCables, definition_only: false },
    RelicDefinition { id: RelicId::EmotionChip, definition_only: false },
    RelicDefinition { id: RelicId::SymbioticVirus, definition_only: false },
    RelicDefinition { id: RelicId::FrozenCore, definition_only: false },
    RelicDefinition { id: RelicId::Inserter, definition_only: false },
    RelicDefinition { id: RelicId::NuclearBattery, definition_only: false },
    RelicDefinition { id: RelicId::RunicCapacitor, definition_only: false },
    RelicDefinition{id:RelicId::VioletLotus,definition_only:false},
    RelicDefinition{id:RelicId::GoldenEye,definition_only:false},
    RelicDefinition{id:RelicId::TeardropLocket,definition_only:false},
];
pub(crate) const STARTER_RELICS: &[(Character, RelicId)] = &[
    (Character::Ironclad, RelicId::BurningBlood),
    (Character::Silent, RelicId::RingOfTheSnake),
    (Character::Defect, RelicId::CrackedCore),
    (Character::Watcher, RelicId::PureWater),
];
pub(crate) const POTIONS: &[PotionDefinition] = &[
    PotionDefinition { id: PotionId::FirePotion, definition_only: true },
    PotionDefinition { id: PotionId::BloodPotion, definition_only: false },
    PotionDefinition { id: PotionId::FruitJuice, definition_only: true },
    PotionDefinition { id: PotionId::BlockPotion, definition_only: true },
    PotionDefinition { id: PotionId::EnergyPotion, definition_only: true },
    PotionDefinition { id: PotionId::StrengthPotion, definition_only: true },
    PotionDefinition { id: PotionId::SwiftPotion, definition_only: true },
    PotionDefinition { id: PotionId::WeakPotion, definition_only: true },
    PotionDefinition { id: PotionId::FearPotion, definition_only: true },
    PotionDefinition { id: PotionId::DexterityPotion, definition_only: false },
    PotionDefinition { id: PotionId::AncientPotion, definition_only: false },
    PotionDefinition { id: PotionId::Elixir, definition_only: false },
    PotionDefinition { id: PotionId::HeartOfIron, definition_only: false },
    PotionDefinition { id: PotionId::PoisonPotion, definition_only: false },
    PotionDefinition { id: PotionId::CunningPotion, definition_only: false },
    PotionDefinition { id: PotionId::GhostInAJar, definition_only: false },
    PotionDefinition { id: PotionId::FlexPotion, definition_only: false },
    PotionDefinition { id: PotionId::SpeedPotion, definition_only: false },
    PotionDefinition { id: PotionId::RegenPotion, definition_only: false },
    PotionDefinition { id: PotionId::LiquidBronze, definition_only: false },
    PotionDefinition { id: PotionId::EssenceOfSteel, definition_only: false },
    PotionDefinition { id: PotionId::CultistPotion, definition_only: false },
    PotionDefinition { id: PotionId::BlessingOfTheForge, definition_only: false },
    PotionDefinition { id: PotionId::SneckoOil, definition_only: false },
    PotionDefinition {id:PotionId::AttackPotion,definition_only:false},
    PotionDefinition {id:PotionId::SkillPotion,definition_only:false},
    PotionDefinition {id:PotionId::PowerPotion,definition_only:false},
    PotionDefinition {id:PotionId::ColorlessPotion,definition_only:false},
    PotionDefinition {id:PotionId::SmokeBomb,definition_only:false},
    PotionDefinition { id: PotionId::FairyPotion, definition_only: false },
    PotionDefinition { id: PotionId::ExplosivePotion, definition_only: false },
    PotionDefinition { id: PotionId::GamblersBrew, definition_only: false },
    PotionDefinition { id: PotionId::LiquidMemories, definition_only: false },
    PotionDefinition { id: PotionId::DuplicationPotion, definition_only: false },
    PotionDefinition { id: PotionId::DistilledChaos, definition_only: false },
    PotionDefinition { id: PotionId::EntropicBrew, definition_only: false },
    PotionDefinition { id: PotionId::FocusPotion, definition_only: false },
    PotionDefinition { id: PotionId::PotionOfCapacity, definition_only: false },
    PotionDefinition { id: PotionId::EssenceOfDarkness, definition_only: false },
    PotionDefinition{id:PotionId::StancePotion,definition_only:false},
    PotionDefinition{id:PotionId::Ambrosia,definition_only:false},
    PotionDefinition{id:PotionId::BottledMiracle,definition_only:false},
];

#[cfg(test)]
mod tests {
    use super::{Character, PotionId, PotionRarity, RelicId, RelicTier};

    #[test]
    fn m20_defect_item_rows_are_implemented_and_character_scoped() {
        let catalog = crate::content::Catalog::standard();
        let relics = [
            (RelicId::SymbioticVirus, RelicTier::Uncommon),
            (RelicId::FrozenCore, RelicTier::Boss),
            (RelicId::Inserter, RelicTier::Boss),
            (RelicId::NuclearBattery, RelicTier::Boss),
            (RelicId::RunicCapacitor, RelicTier::Shop),
        ];
        for (id, tier) in relics {
            let definition = catalog.relic(id).expect("M20 relic definition");
            assert!(!definition.definition_only);
            assert_eq!(catalog.relic_tier(id), Some(tier));
            assert_eq!(catalog.relic_eligible_for(id, Character::Defect), Some(true));
            assert_eq!(catalog.relic_eligible_for(id, Character::Silent), Some(false));
        }

        let potion = catalog.potion(PotionId::EssenceOfDarkness).expect("M20 potion definition");
        assert!(!potion.definition_only);
        assert_eq!(catalog.potion_rarity(PotionId::EssenceOfDarkness), Some(PotionRarity::Rare));
        assert_eq!(catalog.potion_eligible_for(PotionId::EssenceOfDarkness, Character::Defect), Some(true));
        assert_eq!(catalog.potion_eligible_for(PotionId::EssenceOfDarkness, Character::Silent), Some(false));
    }
}
