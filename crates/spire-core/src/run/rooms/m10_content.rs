//! Explicit native support for the M10 Ironclad card capture workload.
use crate::{content::{CardId,CardRarity},run::{RunError,RunRng,RunRngScope}};

const COMMON:&[CardId]=crate::content::pools::RED_COMMON;
const UNCOMMON:&[CardId]=crate::content::pools::RED_UNCOMMON;
const RARE:&[CardId]=crate::content::pools::RED_RARE;
const BASIC:&[CardId]=&[CardId::Strike,CardId::Defend,CardId::Bash];

pub fn red_candidates(rarity:CardRarity)->&'static [CardId]{match rarity{CardRarity::Basic=>BASIC,CardRarity::Common=>COMMON,CardRarity::Uncommon=>UNCOMMON,CardRarity::Rare=>RARE,_=>&[]}}

/// All 28 executable ordinary red nonbasic attacks; Feed and Reaper retain
/// their special run-owned kill/healing semantics and are not generated.
/// The list is owned by `content::pools` (the Ironclad combat Attack order).
pub fn infernal_candidates()->&'static [CardId]{crate::content::pools::IRONCLAD_COMBAT_ATTACKS}

pub(crate) fn sample_capture_row(rng:&mut RunRng,semantic:&[u64],rarities:&[CardRarity])->Result<Vec<CardId>,RunError>{
    let mut chosen=Vec::with_capacity(rarities.len());
    for (slot,rarity) in rarities.iter().enumerate(){
        let remaining=red_candidates(*rarity).iter().copied().filter(|id|!chosen.contains(id)).collect::<Vec<_>>();
        if remaining.is_empty(){return Err(RunError::Unsupported("M10 captured card row has no eligible support"))}
        let mut key=semantic.to_vec();key.push(slot as u64);
        let index=rng.bounded_scoped(RunRngScope::M10CardOffer,&key,remaining.len() as u64);
        chosen.push(remaining[index]);
    }
    Ok(chosen)
}
