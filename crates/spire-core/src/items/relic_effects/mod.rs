//! Per-relic handler files (M27 design §6.2). One file per implementation lane, so no two
//! agents edit one file; `items/dispatch.rs` and `items/laws.rs` (Main) own the loops.
pub(crate) mod accepted_law;
pub(crate) mod play;
pub(crate) mod run;
pub(crate) mod structural;
pub(crate) mod turns;

use crate::content::RelicId;

/// Accepted relics (Appendix A status Lift or M25): handled by the accepted owners in
/// `items/{relics,silent,ironclad,ironclad_turns,mod}.rs` and their inline single-relic sites.
pub(crate) const ACCEPTED_RELICS: &[RelicId] = &[
    RelicId::BurningBlood, RelicId::CrackedCore, RelicId::PureWater, RelicId::RingOfTheSnake,
    RelicId::Anchor, RelicId::BagOfMarbles, RelicId::BagOfPreparation, RelicId::BloodVial,
    RelicId::BronzeScales, RelicId::DataDisk, RelicId::Lantern, RelicId::OddlySmoothStone,
    RelicId::Orichalcum, RelicId::PotionBelt, RelicId::RedSkull, RelicId::SneckoSkull, RelicId::Strawberry,
    RelicId::ToyOrnithopter, RelicId::Vajra, RelicId::GoldPlatedCables, RelicId::NinjaScroll,
    RelicId::PaperKrane, RelicId::PaperPhrog, RelicId::Pear, RelicId::SelfFormingClay,
    RelicId::SymbioticVirus, RelicId::TeardropLocket, RelicId::ChampionsBelt, RelicId::CharonsAshes,
    RelicId::EmotionChip, RelicId::GoldenEye, RelicId::MagicFlower, RelicId::Mango, RelicId::TheSpecimen,
    RelicId::Tingsha, RelicId::ToughBandages, RelicId::BlackBlood, RelicId::EmptyCage, RelicId::FrozenCore,
    RelicId::HolyWater, RelicId::HoveringKite, RelicId::Inserter, RelicId::MarkOfPain,
    RelicId::NuclearBattery, RelicId::RingOfTheSerpent, RelicId::RunicCube, RelicId::SacredBark,
    RelicId::SlaversCollar, RelicId::Sozu, RelicId::VioletLotus, RelicId::WristBlade, RelicId::Brimstone,
    RelicId::ChemicalX, RelicId::RunicCapacitor, RelicId::TwistedFunnel, RelicId::MarkOfBloom,
    RelicId::NeowsLament, RelicId::RedMask, RelicId::Damaru, RelicId::Duality, RelicId::CloakClasp,
    RelicId::Melange,
];

/// The five new relics Main implements (Lizard Tail, Ice Cream, Calipers, Prismatic Shard,
/// Philosopher's Stone; plan Task M5).
pub(crate) const MAIN_RELICS: &[RelicId] = &[
    RelicId::Calipers, RelicId::IceCream, RelicId::LizardTail, RelicId::PhilosophersStone,
    RelicId::PrismaticShard,
];
