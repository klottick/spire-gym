//! Finite M14 eligibility and Act3 rows, using the accepted green reward owner.
use super::*;

// Permanent eligibility includes the actual Writhing Mass curse; reward/shop
// selection deliberately delegates to the separate green rows below.
pub const M14_SILENT_ACT3_CARDS:&[CardId]=&[
    CardId::StrikeGreen,CardId::DefendGreen,CardId::Survivor,CardId::Neutralize,
    CardId::PoisonedStab,CardId::Backflip,CardId::BladeDance,
    CardId::DeadlyPoison,CardId::CloakAndDagger,CardId::DaggerThrow,
    CardId::NoxiousFumes,CardId::Accuracy,CardId::WellLaidPlans,
    CardId::QuickSlash,CardId::Reflex,CardId::Catalyst,
    CardId::Deflect,CardId::Tactician,CardId::Nightmare,CardId::Parasite,
];
pub const M14_SILENT_ACT3_RELICS:&[RelicId]=&[
    RelicId::RingOfTheSnake,RelicId::NeowsLament,RelicId::Lantern,RelicId::Pear,
    RelicId::Mango,RelicId::SneckoSkull,RelicId::ToyOrnithopter,RelicId::Strawberry,
    RelicId::Anchor,RelicId::BloodVial,RelicId::BagOfPreparation,
    RelicId::EmptyCage,RelicId::SlaversCollar,
    RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfMarbles,
    RelicId::BronzeScales,RelicId::RedMask,
];
pub const M14_SILENT_ACT3_POTIONS:&[PotionId]=super::m12_content::M12_SILENT_ACT1_POTIONS;
const ELITE_RELICS:&[RelicId]=&[
    RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::SneckoSkull,
    RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial,
    RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,
    RelicId::BagOfMarbles,RelicId::BronzeScales,
];
pub(in crate::run) fn reward_cards(local_visit:usize)->Vec<CardId>{super::m12_content::reward_cards(false,local_visit)}
pub(in crate::run) fn combat_potion(local_visit:u64)->PotionId{super::m13_content::combat_potion(local_visit)}
pub(in crate::run) fn elite_relic(start:usize,owned:&[RelicId])->Result<(RelicId,usize),RunError>{
    ELITE_RELICS.iter().enumerate().skip(start).find(|(_,id)|!owned.contains(id))
        .map(|(i,id)|(*id,i+1)).ok_or(RunError::Unsupported("captured Silent Act3 elite relic sequence exhausted"))
}
