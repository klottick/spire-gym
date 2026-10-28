//! Source-qualified M15 Silent card, relic, and potion admission data.
use crate::{
    content::{CardId, CardRarity, Catalog, PotionId, RelicId},
    run::{RunError, RunRng, RunRngScope},
};

const BASIC: &[CardId] = &[
    CardId::DefendGreen,
    CardId::Neutralize,
    CardId::StrikeGreen,
    CardId::Survivor,
];

const COMMON: &[CardId] = &[
    CardId::Acrobatics,
    CardId::Backflip,
    CardId::Bane,
    CardId::BladeDance,
    CardId::CloakAndDagger,
    CardId::DaggerSpray,
    CardId::DaggerThrow,
    CardId::DeadlyPoison,
    CardId::Deflect,
    CardId::DodgeAndRoll,
    CardId::FlyingKnee,
    CardId::Outmaneuver,
    CardId::PiercingWail,
    CardId::PoisonedStab,
    CardId::Prepared,
    CardId::QuickSlash,
    CardId::Slice,
    CardId::SneakyStrike,
    CardId::SuckerPunch,
];

const UNCOMMON: &[CardId] = &[
    CardId::Accuracy,
    CardId::AllOutAttack,
    CardId::Backstab,
    CardId::Blur,
    CardId::BouncingFlask,
    CardId::CalculatedGamble,
    CardId::Caltrops,
    CardId::Catalyst,
    CardId::Choke,
    CardId::Concentrate,
    CardId::CripplingCloud,
    CardId::Dash,
    CardId::Distraction,
    CardId::EndlessAgony,
    CardId::EscapePlan,
    CardId::Eviscerate,
    CardId::Expertise,
    CardId::Finisher,
    CardId::Flechettes,
    CardId::Footwork,
    CardId::HeelHook,
    CardId::InfiniteBlades,
    CardId::LegSweep,
    CardId::MasterfulStab,
    CardId::NoxiousFumes,
    CardId::Predator,
    CardId::Reflex,
    CardId::RiddleWithHoles,
    CardId::Setup,
    CardId::Skewer,
    CardId::Tactician,
    CardId::Terror,
    CardId::WellLaidPlans,
];

const RARE: &[CardId] = &[
    CardId::AThousandCuts,
    CardId::Adrenaline,
    CardId::AfterImage,
    CardId::Alchemize,
    CardId::BulletTime,
    CardId::Burst,
    CardId::CorpseExplosion,
    CardId::DieDieDie,
    CardId::Doppelganger,
    CardId::Envenom,
    CardId::GlassKnife,
    CardId::GrandFinale,
    CardId::Malaise,
    CardId::Nightmare,
    CardId::PhantasmalKiller,
    CardId::StormOfSteel,
    CardId::ToolsOfTheTrade,
    CardId::Unload,
    CardId::WraithForm,
];

pub(crate) fn green_candidates(rarity: CardRarity) -> &'static [CardId] {
    match rarity {
        CardRarity::Basic => BASIC,
        CardRarity::Common => COMMON,
        CardRarity::Uncommon => UNCOMMON,
        CardRarity::Rare => RARE,
        CardRarity::Special | CardRarity::Status | CardRarity::Curse => &[],
    }
}

/// All 75 ordinary permanent Silent Green kinds, including Basics.
pub(crate) const M15_GREEN_PERMANENT_CARDS: &[CardId] = &[
    CardId::DefendGreen, CardId::Neutralize, CardId::StrikeGreen, CardId::Survivor,
    CardId::Acrobatics, CardId::Backflip, CardId::Bane, CardId::BladeDance,
    CardId::CloakAndDagger, CardId::DaggerSpray, CardId::DaggerThrow,
    CardId::DeadlyPoison, CardId::Deflect, CardId::DodgeAndRoll, CardId::FlyingKnee,
    CardId::Outmaneuver, CardId::PiercingWail, CardId::PoisonedStab, CardId::Prepared,
    CardId::QuickSlash, CardId::Slice, CardId::SneakyStrike, CardId::SuckerPunch,
    CardId::Accuracy, CardId::AllOutAttack, CardId::Backstab, CardId::Blur,
    CardId::BouncingFlask, CardId::CalculatedGamble, CardId::Caltrops, CardId::Catalyst,
    CardId::Choke, CardId::Concentrate, CardId::CripplingCloud, CardId::Dash,
    CardId::Distraction, CardId::EndlessAgony, CardId::EscapePlan, CardId::Eviscerate,
    CardId::Expertise, CardId::Finisher, CardId::Flechettes, CardId::Footwork,
    CardId::HeelHook, CardId::InfiniteBlades, CardId::LegSweep, CardId::MasterfulStab,
    CardId::NoxiousFumes, CardId::Predator, CardId::Reflex, CardId::RiddleWithHoles,
    CardId::Setup, CardId::Skewer, CardId::Tactician, CardId::Terror,
    CardId::WellLaidPlans, CardId::AThousandCuts, CardId::Adrenaline, CardId::AfterImage,
    CardId::Alchemize, CardId::BulletTime, CardId::Burst, CardId::CorpseExplosion,
    CardId::DieDieDie, CardId::Doppelganger, CardId::Envenom, CardId::GlassKnife,
    CardId::GrandFinale, CardId::Malaise, CardId::Nightmare, CardId::PhantasmalKiller,
    CardId::StormOfSteel, CardId::ToolsOfTheTrade, CardId::Unload, CardId::WraithForm,
];

/// Ordinary rewards include all nonbasic Green source IDs, including Alchemize.
/// Capture admission below preflights every requested executable rarity family.
pub(crate) const M15_ORDINARY_GREEN_CARDS: &[CardId] = &[
    CardId::Acrobatics, CardId::Backflip, CardId::Bane, CardId::BladeDance,
    CardId::CloakAndDagger, CardId::DaggerSpray, CardId::DaggerThrow,
    CardId::DeadlyPoison, CardId::Deflect, CardId::DodgeAndRoll, CardId::FlyingKnee,
    CardId::Outmaneuver, CardId::PiercingWail, CardId::PoisonedStab, CardId::Prepared,
    CardId::QuickSlash, CardId::Slice, CardId::SneakyStrike, CardId::SuckerPunch,
    CardId::Accuracy, CardId::AllOutAttack, CardId::Backstab, CardId::Blur,
    CardId::BouncingFlask, CardId::CalculatedGamble, CardId::Caltrops, CardId::Catalyst,
    CardId::Choke, CardId::Concentrate, CardId::CripplingCloud, CardId::Dash,
    CardId::Distraction, CardId::EndlessAgony, CardId::EscapePlan, CardId::Eviscerate,
    CardId::Expertise, CardId::Finisher, CardId::Flechettes, CardId::Footwork,
    CardId::HeelHook, CardId::InfiniteBlades, CardId::LegSweep, CardId::MasterfulStab,
    CardId::NoxiousFumes, CardId::Predator, CardId::Reflex, CardId::RiddleWithHoles,
    CardId::Setup, CardId::Skewer, CardId::Tactician, CardId::Terror,
    CardId::WellLaidPlans, CardId::AThousandCuts, CardId::Adrenaline, CardId::AfterImage,
    CardId::Alchemize, CardId::BulletTime, CardId::Burst, CardId::CorpseExplosion,
    CardId::DieDieDie, CardId::Doppelganger, CardId::Envenom, CardId::GlassKnife,
    CardId::GrandFinale, CardId::Malaise, CardId::Nightmare, CardId::PhantasmalKiller,
    CardId::StormOfSteel, CardId::ToolsOfTheTrade, CardId::Unload, CardId::WraithForm,
];

/// Permanent run cards admitted by this content boundary, including Parasite.
pub(crate) const M15_PERMANENT_CARDS: &[CardId] = &[
    CardId::DefendGreen, CardId::Neutralize, CardId::StrikeGreen, CardId::Survivor,
    CardId::Acrobatics, CardId::Backflip, CardId::Bane, CardId::BladeDance,
    CardId::CloakAndDagger, CardId::DaggerSpray, CardId::DaggerThrow,
    CardId::DeadlyPoison, CardId::Deflect, CardId::DodgeAndRoll, CardId::FlyingKnee,
    CardId::Outmaneuver, CardId::PiercingWail, CardId::PoisonedStab, CardId::Prepared,
    CardId::QuickSlash, CardId::Slice, CardId::SneakyStrike, CardId::SuckerPunch,
    CardId::Accuracy, CardId::AllOutAttack, CardId::Backstab, CardId::Blur,
    CardId::BouncingFlask, CardId::CalculatedGamble, CardId::Caltrops, CardId::Catalyst,
    CardId::Choke, CardId::Concentrate, CardId::CripplingCloud, CardId::Dash,
    CardId::Distraction, CardId::EndlessAgony, CardId::EscapePlan, CardId::Eviscerate,
    CardId::Expertise, CardId::Finisher, CardId::Flechettes, CardId::Footwork,
    CardId::HeelHook, CardId::InfiniteBlades, CardId::LegSweep, CardId::MasterfulStab,
    CardId::NoxiousFumes, CardId::Predator, CardId::Reflex, CardId::RiddleWithHoles,
    CardId::Setup, CardId::Skewer, CardId::Tactician, CardId::Terror,
    CardId::WellLaidPlans, CardId::AThousandCuts, CardId::Adrenaline, CardId::AfterImage,
    CardId::Alchemize, CardId::BulletTime, CardId::Burst, CardId::CorpseExplosion,
    CardId::DieDieDie, CardId::Doppelganger, CardId::Envenom, CardId::GlassKnife,
    CardId::GrandFinale, CardId::Malaise, CardId::Nightmare, CardId::PhantasmalKiller,
    CardId::StormOfSteel, CardId::ToolsOfTheTrade, CardId::Unload, CardId::WraithForm,
    CardId::Parasite,
];

pub(crate) const M15_RELIC_STOCK: [RelicId; 19] = [
    RelicId::Lantern,
    RelicId::Pear,
    RelicId::Mango,
    RelicId::ToyOrnithopter,
    RelicId::Strawberry,
    RelicId::Anchor,
    RelicId::BloodVial,
    RelicId::Vajra,
    RelicId::OddlySmoothStone,
    RelicId::BagOfPreparation,
    RelicId::BagOfMarbles,
    RelicId::BronzeScales,
    RelicId::Orichalcum,
    RelicId::SneckoSkull,
    RelicId::NinjaScroll,
    RelicId::PaperKrane,
    RelicId::TheSpecimen,
    RelicId::Tingsha,
    RelicId::ToughBandages,
];

pub(crate) const M15_RELICS28: [RelicId; 28] = [
    RelicId::Lantern, RelicId::Pear, RelicId::Mango, RelicId::ToyOrnithopter,
    RelicId::Strawberry, RelicId::Anchor, RelicId::BloodVial, RelicId::Vajra,
    RelicId::OddlySmoothStone, RelicId::BagOfPreparation, RelicId::BagOfMarbles,
    RelicId::BronzeScales, RelicId::Orichalcum, RelicId::SneckoSkull,
    RelicId::NinjaScroll, RelicId::PaperKrane, RelicId::TheSpecimen,
    RelicId::Tingsha, RelicId::ToughBandages, RelicId::RingOfTheSnake,
    RelicId::NeowsLament, RelicId::EmptyCage, RelicId::SlaversCollar,
    RelicId::RedMask, RelicId::TwistedFunnel, RelicId::RingOfTheSerpent,
    RelicId::HoveringKite, RelicId::WristBlade,
];

pub(crate) const M15_BOSS_RELICS: [RelicId; 5] = [
    RelicId::EmptyCage,
    RelicId::SlaversCollar,
    RelicId::RingOfTheSerpent,
    RelicId::HoveringKite,
    RelicId::WristBlade,
];

pub(crate) const M15_POTION_CYCLE: [PotionId; 6] = [
    PotionId::FirePotion,
    PotionId::StrengthPotion,
    PotionId::WeakPotion,
    PotionId::PoisonPotion,
    PotionId::CunningPotion,
    PotionId::GhostInAJar,
];

/// Capture the next unowned relic in the one finite normal M15 stock.
/// Skipped owned entries still advance the returned cursor; there is no refill.
pub(in crate::run) fn capture_relic(
    start: usize,
    owned: &[RelicId],
) -> Result<(RelicId, usize), RunError> {
    M15_RELIC_STOCK
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, relic)| !owned.contains(relic))
        .map(|(index, relic)| (*relic, index + 1))
        .ok_or(RunError::Unsupported("M15 normal relic stock exhausted"))
}

/// Capture one ordered rarity row without replacement. Every requested family
/// is preflighted as a whole: a definition-only member closes that family.
pub(in crate::run) fn sample_capture_row(
    rng: &mut RunRng,
    semantic: &[u64],
    rarities: &[CardRarity],
) -> Result<Vec<CardId>, RunError> {
    let catalog = Catalog::standard();
    for rarity in rarities {
        let candidates = green_candidates(*rarity);
        if candidates.is_empty()
            || candidates.iter().any(|id| {
                catalog.card(*id).is_none_or(|definition| definition.definition_only)
            })
        {
            return Err(RunError::Unsupported(
                "M15 captured card row has an incomplete Green rarity family",
            ));
        }
    }

    let mut staged = rng.clone();
    let mut chosen = Vec::with_capacity(rarities.len());
    for (slot, rarity) in rarities.iter().enumerate() {
        let remaining = green_candidates(*rarity)
            .iter()
            .copied()
            .filter(|id| !chosen.contains(id))
            .collect::<Vec<_>>();
        if remaining.is_empty() {
            return Err(RunError::Unsupported(
                "M15 captured card row has no remaining Green candidate",
            ));
        }
        let mut key = semantic.to_vec();
        key.push(slot as u64);
        let index = staged.bounded_scoped_checked(
            RunRngScope::M15CardOffer,
            &key,
            remaining.len() as u64,
        )?;
        chosen.push(remaining[index]);
    }
    *rng = staged;
    Ok(chosen)
}
