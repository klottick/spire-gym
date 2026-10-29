//! Source-qualified Silent potion factory over the shared card/run random owners.
use crate::{
    combat::{self, EffectError, InputError},
    content::{Character, PotionId, PotionRarity},
    run::{RunError, RunRngScope, RunState},
};

pub(crate) const SILENT_POTIONS: &[PotionId] = &[
    PotionId::PoisonPotion,
    PotionId::CunningPotion,
    PotionId::GhostInAJar,
    PotionId::BlockPotion,
    PotionId::DexterityPotion,
    PotionId::EnergyPotion,
    PotionId::ExplosivePotion,
    PotionId::FirePotion,
    PotionId::StrengthPotion,
    PotionId::SwiftPotion,
    PotionId::WeakPotion,
    PotionId::FearPotion,
    PotionId::AttackPotion,
    PotionId::SkillPotion,
    PotionId::PowerPotion,
    PotionId::ColorlessPotion,
    PotionId::FlexPotion,
    PotionId::SpeedPotion,
    PotionId::BlessingOfTheForge,
    PotionId::RegenPotion,
    PotionId::AncientPotion,
    PotionId::LiquidBronze,
    PotionId::GamblersBrew,
    PotionId::EssenceOfSteel,
    PotionId::DuplicationPotion,
    PotionId::DistilledChaos,
    PotionId::LiquidMemories,
    PotionId::CultistPotion,
    PotionId::FruitJuice,
    PotionId::SneckoOil,
    PotionId::FairyPotion,
    PotionId::SmokeBomb,
    PotionId::EntropicBrew,
];
const SILENT_LIMITED_POTIONS: &[PotionId] = &[
    PotionId::PoisonPotion,
    PotionId::CunningPotion,
    PotionId::GhostInAJar,
    PotionId::BlockPotion,
    PotionId::DexterityPotion,
    PotionId::EnergyPotion,
    PotionId::ExplosivePotion,
    PotionId::FirePotion,
    PotionId::StrengthPotion,
    PotionId::SwiftPotion,
    PotionId::WeakPotion,
    PotionId::FearPotion,
    PotionId::AttackPotion,
    PotionId::SkillPotion,
    PotionId::PowerPotion,
    PotionId::ColorlessPotion,
    PotionId::FlexPotion,
    PotionId::SpeedPotion,
    PotionId::BlessingOfTheForge,
    PotionId::RegenPotion,
    PotionId::AncientPotion,
    PotionId::LiquidBronze,
    PotionId::GamblersBrew,
    PotionId::EssenceOfSteel,
    PotionId::DuplicationPotion,
    PotionId::DistilledChaos,
    PotionId::LiquidMemories,
    PotionId::CultistPotion,
    PotionId::SneckoOil,
    PotionId::FairyPotion,
    PotionId::SmokeBomb,
    PotionId::EntropicBrew,
];

/// Complete constructor pool. Other characters are rejected rather than mapped
/// to a partial shared subset.
pub(crate) const DEFECT_POTIONS:&[PotionId]=&[
    PotionId::FocusPotion,PotionId::PotionOfCapacity,PotionId::EssenceOfDarkness,
    PotionId::BlockPotion,PotionId::DexterityPotion,PotionId::EnergyPotion,PotionId::ExplosivePotion,
    PotionId::FirePotion,PotionId::StrengthPotion,PotionId::SwiftPotion,PotionId::WeakPotion,PotionId::FearPotion,
    PotionId::AttackPotion,PotionId::SkillPotion,PotionId::PowerPotion,PotionId::ColorlessPotion,PotionId::FlexPotion,
    PotionId::SpeedPotion,PotionId::BlessingOfTheForge,PotionId::RegenPotion,PotionId::AncientPotion,
    PotionId::LiquidBronze,PotionId::GamblersBrew,PotionId::EssenceOfSteel,PotionId::DuplicationPotion,
    PotionId::DistilledChaos,PotionId::LiquidMemories,PotionId::CultistPotion,PotionId::FruitJuice,
    PotionId::SneckoOil,PotionId::FairyPotion,PotionId::SmokeBomb,PotionId::EntropicBrew,
];
const DEFECT_LIMITED_POTIONS:&[PotionId]=&[
    PotionId::FocusPotion,PotionId::PotionOfCapacity,PotionId::EssenceOfDarkness,
    PotionId::BlockPotion,PotionId::DexterityPotion,PotionId::EnergyPotion,PotionId::ExplosivePotion,
    PotionId::FirePotion,PotionId::StrengthPotion,PotionId::SwiftPotion,PotionId::WeakPotion,PotionId::FearPotion,
    PotionId::AttackPotion,PotionId::SkillPotion,PotionId::PowerPotion,PotionId::ColorlessPotion,PotionId::FlexPotion,
    PotionId::SpeedPotion,PotionId::BlessingOfTheForge,PotionId::RegenPotion,PotionId::AncientPotion,
    PotionId::LiquidBronze,PotionId::GamblersBrew,PotionId::EssenceOfSteel,PotionId::DuplicationPotion,
    PotionId::DistilledChaos,PotionId::LiquidMemories,PotionId::CultistPotion,
    PotionId::SneckoOil,PotionId::FairyPotion,PotionId::SmokeBomb,PotionId::EntropicBrew,
];

pub(crate) fn candidates(character: Character, limited: bool) -> Result<&'static [PotionId], InputError> {
    if character==Character::Defect{return Ok(if limited{DEFECT_LIMITED_POTIONS}else{DEFECT_POTIONS})}
    if character != Character::Silent {
        return Err(InputError::Unsupported("Silent potion generation requires actual Silent character".into()));
    }
    Ok(if limited { SILENT_LIMITED_POTIONS } else { SILENT_POTIONS })
}

fn rarity(id: PotionId) -> PotionRarity {
    match id {
        PotionId::FocusPotion | PotionId::PoisonPotion | PotionId::BlockPotion | PotionId::DexterityPotion
        | PotionId::EnergyPotion | PotionId::ExplosivePotion | PotionId::FirePotion
        | PotionId::StrengthPotion | PotionId::SwiftPotion | PotionId::WeakPotion
        | PotionId::FearPotion | PotionId::AttackPotion | PotionId::SkillPotion
        | PotionId::PowerPotion | PotionId::ColorlessPotion | PotionId::FlexPotion
        | PotionId::SpeedPotion | PotionId::BlessingOfTheForge => PotionRarity::Common,
        PotionId::PotionOfCapacity | PotionId::CunningPotion | PotionId::RegenPotion | PotionId::AncientPotion
        | PotionId::LiquidBronze | PotionId::GamblersBrew | PotionId::EssenceOfSteel
        | PotionId::DuplicationPotion | PotionId::DistilledChaos | PotionId::LiquidMemories => PotionRarity::Uncommon,
        PotionId::EssenceOfDarkness | PotionId::GhostInAJar | PotionId::CultistPotion | PotionId::FruitJuice
        | PotionId::SneckoOil | PotionId::FairyPotion | PotionId::SmokeBomb
        | PotionId::EntropicBrew => PotionRarity::Rare,
        _ => unreachable!("only complete Silent potion pools reach the factory"),
    }
}

/// One source-law proposal: limited generation discards its first flat index
/// from the full pool, rolls rarity once, then rejects Fruit Juice and
/// wrong-rarity proposals without a cap.
fn sample_from_pool<E>(
    pool: &[PotionId],
    limited: bool,
    mut draw: impl FnMut(u64) -> Result<usize, E>,
) -> Result<PotionId, E> {
    let rarity_roll = draw(100)?;
    let wanted = match rarity_roll {
        0..=64 => PotionRarity::Common,
        65..=89 => PotionRarity::Uncommon,
        _ => PotionRarity::Rare,
    };
    if limited {
        let _discarded_initial_proposal = draw(pool.len() as u64)?;
    }
    loop {
        let proposal = pool[draw(pool.len() as u64)?];
        if (!limited || proposal != PotionId::FruitJuice) && rarity(proposal) == wanted {
            return Ok(proposal);
        }
    }
}

/// Sample from the actual combat card-random stream, returning its committed
/// successor. The source-qualified pool is never filtered by handler support.
pub(crate) fn sample(
    character: Character,
    limited: bool,
    stream: &combat::rng::StreamState,
) -> Result<(PotionId, combat::rng::StreamState), EffectError> {
    let pool = candidates(character, false).map_err(EffectError::Invalid)?;
    let mut cursor = stream.clone();
    let potion = sample_from_pool(pool, limited, |bound| {
        let (index, next) = combat::rng::uniform_below(&cursor, bound as u128)?;
        cursor = next;
        Ok::<usize, combat::rng::RngError>(index as usize)
    })?;
    Ok((potion, cursor))
}

/// Draw the complete proposal batch before any obtain action can be queued.
pub(crate) fn sample_many(
    character: Character,
    limited: bool,
    count: usize,
    stream: &combat::rng::StreamState,
) -> Result<(Vec<PotionId>, combat::rng::StreamState), EffectError> {
    let mut cursor = stream.clone();
    let mut potions = Vec::with_capacity(count);
    for _ in 0..count {
        let (potion, next) = sample(character, limited, &cursor)?;
        potions.push(potion);
        cursor = next;
    }
    Ok((potions, cursor))
}

/// Outside-combat proposals use Run's checked RNG scope and counter journal.
/// A single semantic key carries all draws for the accepted Brew use.
pub(crate) fn sample_many_for_run(
    state: &mut RunState,
    character: Character,
    limited: bool,
    count: usize,
    semantic: &[u64],
) -> Result<Vec<PotionId>, RunError> {
    let pool = candidates(character, false)
        .map_err(|_| RunError::Unsupported("Silent potion generation requires actual Silent character"))?;
    let mut potions = Vec::with_capacity(count);
    for _ in 0..count {
        let potion = sample_from_pool(pool, limited, |bound| {
            state.rng.bounded_scoped_checked(RunRngScope::M15PotionGeneration, semantic, bound)
        })?;
        potions.push(potion);
    }
    Ok(potions)
}

#[cfg(test)]
pub(crate) fn sample_scripted(
    character: Character,
    limited: bool,
    draws: &[usize],
) -> (PotionId, Vec<u64>, usize) {
    let pool = candidates(character, false).expect("Silent fixture has a complete proposal pool");
    let mut cursor = 0;
    let mut bounds = Vec::new();
    let potion = sample_from_pool(pool, limited, |bound| {
        bounds.push(bound);
        let value = *draws.get(cursor).expect("scripted draw stream exhausted");
        cursor += 1;
        Ok::<usize, ()>(value % bound as usize)
    })
    .unwrap();
    (potion, bounds, cursor)
}
