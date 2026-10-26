//! Finite Silent Act1 captures. These are declared rows, not standard random pools.
//! Identity/rarity/character inputs: data/cards, data/relics, data/README and the
//! accepted M11 profile. Shared room, acquisition and card owners execute them.
use super::*;

pub const M12_SILENT_ACT1_CARDS:&[CardId]=&[
    CardId::StrikeGreen,CardId::DefendGreen,CardId::Survivor,CardId::Neutralize,
    CardId::PoisonedStab,CardId::Backflip,CardId::BladeDance,
    CardId::DeadlyPoison,CardId::CloakAndDagger,CardId::DaggerThrow,
    CardId::NoxiousFumes,CardId::Accuracy,CardId::WellLaidPlans,
    CardId::QuickSlash,CardId::Reflex,CardId::Catalyst,
    CardId::Deflect,CardId::Tactician,CardId::Nightmare,
];
pub const M12_SILENT_ACT1_RELICS:&[RelicId]=&[
    RelicId::RingOfTheSnake,RelicId::NeowsLament,RelicId::Lantern,RelicId::Pear,
    RelicId::Mango,RelicId::SneckoSkull,RelicId::ToyOrnithopter,RelicId::Strawberry,
    RelicId::Anchor,RelicId::BloodVial,RelicId::BagOfPreparation,
    RelicId::EmptyCage,RelicId::SlaversCollar,
];
pub const M12_SILENT_ACT1_POTIONS:&[PotionId]=&[
    PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion,PotionId::DexterityPotion,
];
const REWARDS:[[CardId;3];3]=[
    [CardId::PoisonedStab,CardId::Backflip,CardId::BladeDance],
    [CardId::DeadlyPoison,CardId::CloakAndDagger,CardId::DaggerThrow],
    [CardId::NoxiousFumes,CardId::Accuracy,CardId::WellLaidPlans],
];
const ELITE_RELICS:&[RelicId]=&[
    RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::SneckoSkull,
    RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial,
];
pub(super) fn reward_cards(boss:bool,ordinal:usize)->Vec<CardId>{
    if boss{vec![CardId::Nightmare]}else{REWARDS[ordinal%REWARDS.len()].to_vec()}
}
pub(in crate::run) fn elite_relic(start:usize,owned:&[RelicId])->Result<(RelicId,usize),RunError>{
    elite_relic_with_exhaustion(start,owned,"captured Silent Act1 elite relic sequence exhausted")
}
pub(in crate::run) fn elite_relic_with_exhaustion(start:usize,owned:&[RelicId],message:&'static str)->Result<(RelicId,usize),RunError>{
    ELITE_RELICS.iter().enumerate().skip(start).find(|(_,id)|!owned.contains(id))
        .map(|(i,id)|(*id,i+1)).ok_or(RunError::Unsupported(message))
}
pub(super) fn shop_cards(later:bool)->[CardId;3]{
    if later{[CardId::Deflect,CardId::Tactician,CardId::Nightmare]}else{[CardId::QuickSlash,CardId::Reflex,CardId::Catalyst]}
}

// The shared captured Act1 reward already uses this order. Keep its M12 data
// decision explicit while leaving reward preparation/claim ownership shared.
pub(super) fn combat_potion(visit:u64)->PotionId{
    [PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion][(visit%3) as usize]
}
