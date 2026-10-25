use super::{Character, PotionId, RelicId};

#[derive(Clone, Copy, Debug)]
pub struct RelicDefinition { pub id: RelicId, pub definition_only: bool }
#[derive(Clone, Copy, Debug)]
pub struct PotionDefinition { pub id: PotionId, pub definition_only: bool }

pub(crate) const RELICS: &[RelicDefinition] = &[
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
];
