//! Explicit native support for the M10 Ironclad card capture workload.
use crate::{content::{CardId,CardRarity},run::{RunError,RunRng,RunRngScope}};

const COMMON:&[CardId]=&[CardId::Armaments,CardId::BodySlam,CardId::Clash,CardId::SwordBoomerang,CardId::WildStrike,CardId::TrueGrit,CardId::Headbutt,CardId::Warcry,CardId::Havoc,CardId::Anger,CardId::IronWave,CardId::Cleave,CardId::PommelStrike,CardId::ShrugItOff,CardId::TwinStrike,CardId::Clothesline,CardId::HeavyBlade,CardId::PerfectedStrike,CardId::Thunderclap,CardId::Flex];
const UNCOMMON:&[CardId]=&[CardId::SearingBlow,CardId::Carnage,CardId::Hemokinesis,CardId::Pummel,CardId::Uppercut,CardId::Entrench,CardId::GhostlyArmor,CardId::Intimidate,CardId::PowerThrough,CardId::RecklessCharge,CardId::SeeingRed,CardId::Shockwave,CardId::SpotWeakness,CardId::BattleTrance,CardId::DualWield,CardId::Whirlwind,CardId::Inflame,CardId::Evolve,CardId::FireBreathing,CardId::FlameBarrier,CardId::Rage,CardId::Rupture,CardId::DarkEmbrace,CardId::FeelNoPain,CardId::Sentinel,CardId::SecondWind,CardId::SeverSoul,CardId::Metallicize,CardId::Combust,CardId::Bloodletting,CardId::BurningPact,CardId::Disarm,CardId::BloodForBlood,CardId::Rampage,CardId::Dropkick,CardId::InfernalBlade];
const RARE:&[CardId]=&[CardId::Immolate,CardId::Exhume,CardId::DoubleTap,CardId::Juggernaut,CardId::Corruption,CardId::FiendFire,CardId::Barricade,CardId::Berserk,CardId::Brutality,CardId::DemonForm,CardId::Bludgeon,CardId::Feed,CardId::Reaper,CardId::Impervious,CardId::Offering,CardId::LimitBreak];
const BASIC:&[CardId]=&[CardId::Strike,CardId::Defend,CardId::Bash];

pub fn red_candidates(rarity:CardRarity)->&'static [CardId]{match rarity{CardRarity::Basic=>BASIC,CardRarity::Common=>COMMON,CardRarity::Uncommon=>UNCOMMON,CardRarity::Rare=>RARE,_=>&[]}}

/// All 28 executable ordinary red nonbasic attacks; Feed and Reaper retain
/// their special run-owned kill/healing semantics and are not generated.
pub fn infernal_candidates()->&'static [CardId]{&[CardId::SearingBlow,CardId::BodySlam,CardId::Carnage,CardId::Clash,CardId::Hemokinesis,CardId::Pummel,CardId::Uppercut,CardId::Immolate,CardId::SwordBoomerang,CardId::WildStrike,CardId::RecklessCharge,CardId::Headbutt,CardId::Whirlwind,CardId::SeverSoul,CardId::FiendFire,CardId::Anger,CardId::IronWave,CardId::Cleave,CardId::PommelStrike,CardId::TwinStrike,CardId::Clothesline,CardId::HeavyBlade,CardId::PerfectedStrike,CardId::Thunderclap,CardId::Bludgeon,CardId::BloodForBlood,CardId::Rampage,CardId::Dropkick]}

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
