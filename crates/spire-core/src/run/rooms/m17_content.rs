//! Finite Defect Act1 placements, shared by later approved prefix profiles.
//! These rows describe acquisition, not mechanics or standard random pools.
use super::*;
pub const M17_DEFECT_ACT1_CARDS:&[CardId]=&[
    CardId::StrikeBlue,CardId::DefendBlue,CardId::Zap,CardId::Dualcast,
    CardId::BallLightning,CardId::ColdSnap,CardId::Coolheaded,
    CardId::Barrage,CardId::CompileDriver,CardId::Darkness,
    CardId::Fusion,CardId::Defragment,CardId::GeneticAlgorithm,
    CardId::Claw,CardId::SteamBarrier,CardId::Streamline,
    CardId::Storm,CardId::Recursion,CardId::BiasedCognition,
    CardId::Capacitor,CardId::Consume,CardId::Loop,
    CardId::Bullseye,CardId::StaticDischarge,CardId::Heatsinks,
    CardId::Electrodynamics,CardId::Fission,CardId::Amplify,
];
pub const M17_DEFECT_ACT1_RELICS:&[RelicId]=&[
    RelicId::CrackedCore,RelicId::NeowsLament,RelicId::Lantern,RelicId::Pear,
    RelicId::Mango,RelicId::DataDisk,RelicId::ToyOrnithopter,RelicId::Strawberry,
    RelicId::Anchor,RelicId::BloodVial,RelicId::GoldPlatedCables,RelicId::EmotionChip,
    RelicId::EmptyCage,RelicId::SlaversCollar,
];
pub const M17_DEFECT_ACT1_POTIONS:&[PotionId]=&[
    PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion,PotionId::FocusPotion,PotionId::PotionOfCapacity,
];
const REWARDS:[[CardId;3];5]=[
    [CardId::BallLightning,CardId::ColdSnap,CardId::Coolheaded],
    [CardId::Barrage,CardId::CompileDriver,CardId::Darkness],
    [CardId::Fusion,CardId::Defragment,CardId::GeneticAlgorithm],
    [CardId::Claw,CardId::SteamBarrier,CardId::Streamline],
    [CardId::Storm,CardId::Recursion,CardId::BiasedCognition],
];
const ELITE_RELICS:&[RelicId]=&[
    RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::DataDisk,
    RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial,
];
pub(in crate::run) fn reward_cards(boss:bool,local_visit:u64)->Vec<CardId>{
    if boss{vec![CardId::Electrodynamics,CardId::Fission,CardId::Amplify]}else{REWARDS[(local_visit%5) as usize].to_vec()}
}
pub(in crate::run) fn elite_relic(start:usize,owned:&[RelicId])->Result<(RelicId,usize),RunError>{
    ELITE_RELICS.iter().enumerate().skip(start).find(|(_,id)|!owned.contains(id))
        .map(|(i,id)|(*id,i+1)).ok_or(RunError::Unsupported("captured Defect Act1 elite relic sequence exhausted"))
}
pub(in crate::run) fn shop_cards(later:bool)->[CardId;3]{
    if later{[CardId::Bullseye,CardId::StaticDischarge,CardId::Heatsinks]}else{[CardId::Capacitor,CardId::Consume,CardId::Loop]}
}
pub(in crate::run) fn combat_potion(local_visit:u64)->PotionId{
    [PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion][(local_visit%3) as usize]
}
