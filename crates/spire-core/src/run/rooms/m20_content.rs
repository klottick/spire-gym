//! Complete-blue finite M20 placement data on the shared staged sampler.
use super::*;
use crate::content::{CardColor,CardRarity,Catalog};
const fn permanent_cards()->[CardId;76]{
    let mut cards=[CardId::Parasite;76];let mut index=0;
    while index<75{cards[index]=crate::content::defect::M20_DEFECT_CARDS[index];index+=1;}cards
}
pub const M20_DEFECT_HEART_CARDS:&[CardId]=&permanent_cards();
pub const NORMAL_RELIC_STOCK:&[RelicId]=&[
    RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::ToyOrnithopter,RelicId::Strawberry,
    RelicId::Anchor,RelicId::BloodVial,RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,
    RelicId::BagOfMarbles,RelicId::BronzeScales,RelicId::Orichalcum,RelicId::DataDisk,
    RelicId::GoldPlatedCables,RelicId::SymbioticVirus,RelicId::EmotionChip,
];
pub const BOSS_RELICS:&[RelicId]=&[RelicId::EmptyCage,RelicId::SlaversCollar,RelicId::FrozenCore,RelicId::Inserter,RelicId::NuclearBattery];
pub const M20_DEFECT_HEART_RELICS:&[RelicId]=&[
    RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::ToyOrnithopter,RelicId::Strawberry,
    RelicId::Anchor,RelicId::BloodVial,RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,
    RelicId::BagOfMarbles,RelicId::BronzeScales,RelicId::Orichalcum,RelicId::DataDisk,
    RelicId::GoldPlatedCables,RelicId::SymbioticVirus,RelicId::EmotionChip,
    RelicId::CrackedCore,RelicId::NeowsLament,RelicId::EmptyCage,RelicId::SlaversCollar,
    RelicId::FrozenCore,RelicId::Inserter,RelicId::NuclearBattery,RelicId::RunicCapacitor,RelicId::RedMask,
];
pub const M20_DEFECT_HEART_POTIONS:&[PotionId]=crate::items::potion_generation::DEFECT_POTIONS;
pub const POTION_CYCLE:&[PotionId]=&[PotionId::FirePotion,PotionId::StrengthPotion,PotionId::WeakPotion,
    PotionId::FocusPotion,PotionId::PotionOfCapacity,PotionId::EssenceOfDarkness,PotionId::AttackPotion,
    PotionId::SkillPotion,PotionId::PowerPotion,PotionId::ColorlessPotion,PotionId::EntropicBrew];
pub(in crate::run) fn blue_candidates(rarity:CardRarity)->&'static [CardId]{
    static FAMILIES:std::sync::OnceLock<[Vec<CardId>;3]>=std::sync::OnceLock::new();
    let families=FAMILIES.get_or_init(||[CardRarity::Common,CardRarity::Uncommon,CardRarity::Rare].map(|rarity|{
        let mut cards=Catalog::standard().cards().iter().filter(|row|row.color==CardColor::Blue&&row.rarity==rarity)
            .map(|row|row.id).collect::<Vec<_>>();cards.sort_by_key(|id|id.as_str());cards
    }));
    match rarity{CardRarity::Common=>&families[0],CardRarity::Uncommon=>&families[1],CardRarity::Rare=>&families[2],_=>&[]}
}
pub(in crate::run) fn sample_capture_row(rng:&mut RunRng,semantic:&[u64],rarities:&[CardRarity])->Result<Vec<CardId>,RunError>{
    for rarity in rarities {
        let expected=match rarity{CardRarity::Common=>18,CardRarity::Uncommon=>36,CardRarity::Rare=>17,_=>0};
        if blue_candidates(*rarity).len()!=expected||expected==0{return Err(RunError::Unsupported("M20 incomplete Blue rarity family"))}
    }
    super::m15_content::sample_capture_row_for(rng,semantic,rarities,blue_candidates,RunRngScope::M20CardOffer,
        "M20 incomplete Blue rarity family","M20 captured card row has no remaining Blue candidate")
}
pub(in crate::run) fn normal_relic(start:usize,owned:&[RelicId])->Result<(RelicId,usize),RunError>{
    NORMAL_RELIC_STOCK.iter().enumerate().skip(start).find(|(_,id)|!owned.contains(id))
        .map(|(index,id)|(*id,index+1)).ok_or(RunError::Unsupported("M20 normal relic stock exhausted"))
}
