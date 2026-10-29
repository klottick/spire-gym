//! Finite Act3 placement data. The accepted M17/M18 prefix remains authoritative.
use super::*;
pub const M19_DEFECT_ACT3_CARDS:&[CardId]=&[
    CardId::StrikeBlue,CardId::DefendBlue,CardId::Zap,CardId::Dualcast,
    CardId::BallLightning,CardId::ColdSnap,CardId::Coolheaded,
    CardId::Barrage,CardId::CompileDriver,CardId::Darkness,
    CardId::Fusion,CardId::Defragment,CardId::GeneticAlgorithm,
    CardId::Claw,CardId::SteamBarrier,CardId::Streamline,
    CardId::Storm,CardId::Recursion,CardId::BiasedCognition,
    CardId::Electrodynamics,CardId::Fission,CardId::Amplify,CardId::EchoForm,
    CardId::Capacitor,CardId::Consume,CardId::Loop,
    CardId::Bullseye,CardId::StaticDischarge,CardId::Heatsinks,CardId::Parasite,
];
pub const M19_DEFECT_ACT3_RELICS:&[RelicId]=&[
    RelicId::CrackedCore,RelicId::NeowsLament,RelicId::Lantern,RelicId::Pear,
    RelicId::Mango,RelicId::DataDisk,RelicId::ToyOrnithopter,RelicId::Strawberry,
    RelicId::Anchor,RelicId::BloodVial,RelicId::GoldPlatedCables,RelicId::EmotionChip,
    RelicId::EmptyCage,RelicId::SlaversCollar,RelicId::Vajra,RelicId::OddlySmoothStone,
    RelicId::BagOfMarbles,RelicId::BronzeScales,RelicId::RedMask,
];
pub const M19_DEFECT_ACT3_POTIONS:&[PotionId]=super::m17_content::M17_DEFECT_ACT1_POTIONS;
const ELITE_RELICS:&[RelicId]=&[
    RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::DataDisk,
    RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial,
    RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfMarbles,RelicId::BronzeScales,
];
pub(in crate::run) fn reward_cards(local_visit:u64)->Vec<CardId>{super::m17_content::reward_cards(false,local_visit)}
pub(in crate::run) fn combat_potion(local_visit:u64)->PotionId{super::m17_content::combat_potion(local_visit)}
pub(in crate::run) fn shop_cards(later:bool)->[CardId;3]{super::m17_content::shop_cards(later)}
pub(in crate::run) fn elite_relic(start:usize,owned:&[RelicId])->Result<(RelicId,usize),RunError>{
    ELITE_RELICS.iter().enumerate().skip(start).find(|(_,id)|!owned.contains(id))
        .map(|(i,id)|(*id,i+1)).ok_or(RunError::Unsupported("captured Defect Act3 elite relic sequence exhausted"))
}
