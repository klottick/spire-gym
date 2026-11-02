//! Standard card rewards (design §6.1). M28a lands the read-only rarity classifier shared with
//! the standard shop; M28b adds the reward builder and the offset writes.
use crate::content::CardRarity;

/// The offset `b` at Run start and after a rare (economy research :105-107).
pub(in crate::run) const OFFSET_START:i32=5;

/// The room kinds whose card thresholds differ (economy research :103-106).
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(in crate::run) enum RarityRoom { Ordinary, Elite, Boss }

/// Classify one 0..100 draw `n` with the live offset `b`: rare if `n + b < R × rare_factor`,
/// uncommon if `n + b < R × rare_factor + U`, else common. Ordinary (monster, event, shop, rest)
/// R 3, U 37; elite R 10, U 40; a boss slot is always rare. `rare_factor` is M27's N'loth's Gift
/// multiplier on the rare chance (`CardRewardShape::rare_factor`, offset unchanged, no clamp).
pub(in crate::run) fn classify(room:RarityRoom,n:u32,offset:i32,rare_factor:u32)->CardRarity{
    let (rare,uncommon)=match room{RarityRoom::Ordinary=>(3,37),RarityRoom::Elite=>(10,40),RarityRoom::Boss=>return CardRarity::Rare};
    let roll=n as i64+i64::from(offset);
    let rare=rare*i64::from(rare_factor);
    if roll<rare{CardRarity::Rare}else if roll<rare+uncommon{CardRarity::Uncommon}else{CardRarity::Common}
}
