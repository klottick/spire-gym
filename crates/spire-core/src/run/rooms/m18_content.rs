//! Finite Act2 selection only. M17 owns the unchanged Act1 prefix and common rows.
use super::*;
pub const M18_DEFECT_ACT2_CARDS:&[CardId]=crate::content::defect::M16_DEFECT_CARDS;
pub const M18_DEFECT_ACT2_RELICS:&[RelicId]=super::m17_content::M17_DEFECT_ACT1_RELICS;
pub const M18_DEFECT_ACT2_POTIONS:&[PotionId]=super::m17_content::M17_DEFECT_ACT1_POTIONS;
pub(in crate::run) fn reward_cards(boss:bool,local_visit:u64)->Vec<CardId>{
    if boss{vec![CardId::Electrodynamics,CardId::Amplify,CardId::EchoForm]}else{super::m17_content::reward_cards(false,local_visit)}
}
pub(in crate::run) fn elite_relic(start:usize,owned:&[RelicId])->Result<(RelicId,usize),RunError>{
    super::m17_content::elite_relic(start,owned).map_err(|error|match error{
        RunError::Unsupported("captured Defect Act1 elite relic sequence exhausted")=>RunError::Unsupported("captured Defect Act2 elite relic sequence exhausted"),
        other=>other,
    })
}
