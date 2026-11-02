//! M27 complete-admission relic pool history (design §7.4).
use crate::{
    content::{Catalog, CardRarity, CardType, RelicId, RelicTier},
    items::lifecycle::RelicSource,
    run::{RunError, RunRngScope, RunState},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RelicPools {
    pub(crate) common: Vec<RelicId>,
    pub(crate) uncommon: Vec<RelicId>,
    pub(crate) rare: Vec<RelicId>,
    pub(crate) shop: Vec<RelicId>,
    pub(crate) boss: Vec<RelicId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TierTable { Ordinary, Merchant }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Selector { Front, Back }

pub(crate) fn build(state: &RunState, key: &[u64]) -> Result<RelicPools, RunError> {
    let catalog = Catalog::standard();
    let mut pools = RelicPools::default();
    for id in crate::content::complete::relic_union(state.character) {
        let tier = catalog.relic_tier(id).ok_or(RunError::Registry("complete relic tier missing"))?;
        let target = match tier {
            RelicTier::Common => Some(&mut pools.common),
            RelicTier::Uncommon => Some(&mut pools.uncommon),
            RelicTier::Rare => Some(&mut pools.rare),
            RelicTier::Shop => Some(&mut pools.shop),
            RelicTier::Boss => Some(&mut pools.boss),
            RelicTier::Starter | RelicTier::Special => None,
        };
        if let Some(target) = target { target.push(id); }
    }

    // Research §4.1: initialize each catalog-ordered tier by a scoped shuffle, then remove owned relics.
    // `build` is intentionally read-only; clone the RunRng so the initializer's keys and stream are
    // reproducible without adding draws to the supplied RunState.
    let mut rng = state.rng.clone();
    for tier in [&mut pools.common, &mut pools.uncommon, &mut pools.rare, &mut pools.shop, &mut pools.boss] {
        for upper in (1..tier.len()).rev() {
            let selected = rng.bounded_scoped_checked(RunRngScope::M27RelicPools, key, (upper + 1) as u64)?;
            tier.swap(upper, selected);
        }
    }
    for owned in &state.relics { remove(&mut pools, *owned); }
    Ok(pools)
}

pub(crate) fn roll_tier(state: &mut RunState, table: TierTable, key: &[u64]) -> RelicTier {
    let roll = state.rng.bounded_scoped(RunRngScope::M27RelicTier, key, 100) as u8;
    tier_for_roll(table, roll)
}

pub(crate) fn take(state: &mut RunState, tier: RelicTier, selector: Selector,
    source: RelicSource, key: &[u64]) -> Result<RelicId, RunError> {
    take_with_exclusions(state, tier, selector, source, &[], key)
}

pub(crate) fn remove(pools: &mut RelicPools, id: RelicId) {
    pools.common.retain(|candidate| *candidate != id);
    pools.uncommon.retain(|candidate| *candidate != id);
    pools.rare.retain(|candidate| *candidate != id);
    pools.shop.retain(|candidate| *candidate != id);
    pools.boss.retain(|candidate| *candidate != id);
}

pub(crate) fn exclude_take(state: &mut RunState, tier: crate::content::RelicTier, selector: Selector,
    source: RelicSource, excluded: &[RelicId], key: &[u64]) -> Result<RelicId, RunError> {
    take_with_exclusions(state, tier, selector, source, excluded, key)
}

fn take_with_exclusions(state: &mut RunState, tier: RelicTier, selector: Selector,
    source: RelicSource, excluded: &[RelicId], _key: &[u64]) -> Result<RelicId, RunError> {
    take_from_requested_tier(state, tier, selector, source, excluded)
}

fn take_from_requested_tier(state: &mut RunState, tier: RelicTier, selector: Selector,
    source: RelicSource, excluded: &[RelicId]) -> Result<RelicId, RunError> {
    let pools = state.relic_pools.as_mut().ok_or(RunError::InvalidAction)?;
    let candidates = match tier {
        RelicTier::Common => &mut pools.common,
        RelicTier::Uncommon => &mut pools.uncommon,
        RelicTier::Rare => &mut pools.rare,
        RelicTier::Shop => &mut pools.shop,
        RelicTier::Boss => &mut pools.boss,
        RelicTier::Starter | RelicTier::Special => return Err(RunError::InvalidAction),
    };
    if candidates.is_empty() {
        return match tier {
            // §4.1: depletion promotions use the normal/front helper even after an end request.
            RelicTier::Common => take_from_requested_tier(state, RelicTier::Uncommon, Selector::Front, source, excluded),
            RelicTier::Uncommon => take_from_requested_tier(state, RelicTier::Rare, Selector::Front, source, excluded),
            RelicTier::Shop => take_from_requested_tier(state, RelicTier::Uncommon, Selector::Front, source, excluded),
            RelicTier::Rare => Ok(RelicId::Circlet),
            // F7: the source fallback is Red Circlet; it remains outside ordinary pool unions.
            RelicTier::Boss => Ok(RelicId::RedCirclet),
            RelicTier::Starter | RelicTier::Special => Err(RunError::InvalidAction),
        };
    }

    // §4.1: Boss is front for both selectors; other tiers honor front or back.
    let index = if tier == RelicTier::Boss || selector == Selector::Front { 0 } else { candidates.len() - 1 };
    let candidate = candidates.remove(index);
    if excluded.contains(&candidate) || !spawn_allowed(state, candidate, source) {
        // §4.1: rejected entries stay consumed, then retry from the end of the original tier.
        return take_from_requested_tier(state, tier, Selector::Back, source, excluded);
    }
    Ok(candidate)
}

fn spawn_allowed(state: &RunState, id: RelicId, source: RelicSource) -> bool {
    let catalog = Catalog::standard();
    if catalog.relic_eligible_for(id, state.character) != Some(true) { return false; }

    // Catalog character restrictions are enforced before pool construction; replacements still
    // require their named starter, matching data/relics/relics.json `acquisition.replaces`.
    let required_starter = match id {
        RelicId::BlackBlood => Some(RelicId::BurningBlood),
        RelicId::FrozenCore => Some(RelicId::CrackedCore),
        RelicId::HolyWater => Some(RelicId::PureWater),
        RelicId::RingOfTheSerpent => Some(RelicId::RingOfTheSnake),
        _ => None,
    };
    if required_starter.is_some_and(|starter| !state.relics.contains(&starter)) { return false; }

    // Economy research §4.1: not ShopRoom and floor <= 48 (docs/research/2026-10-19-act1-content-economy-potions.md:226).
    if matches!(id, RelicId::TheCourier | RelicId::MawBank | RelicId::OldCoin | RelicId::SmilingMask)
        && (state.floor > 48 || matches!(source, RelicSource::Shop)) { return false; }
    // Economy research §4.1: the sixteen floor <= 48 relics (docs/research/2026-10-19-act1-content-economy-potions.md:230).
    if matches!(id, RelicId::AncientTeaSet | RelicId::CeramicFish | RelicId::DarkstonePeriapt | RelicId::DreamCatcher
        | RelicId::FrozenEgg | RelicId::MoltenEgg | RelicId::ToxicEgg | RelicId::JuzuBracelet | RelicId::MealTicket
        | RelicId::MeatOnTheBone | RelicId::Omamori | RelicId::PotionBelt | RelicId::PrayerWheel
        | RelicId::QuestionCard | RelicId::RegalPillow | RelicId::SingingBowl) && state.floor > 48 { return false; }
    // Economy research §4.1: Matryoshka/Wing Boots <= 40, Tiny Chest <= 35, Insect <= 52
    // (docs/research/2026-10-19-act1-content-economy-potions.md:231).
    if matches!(id, RelicId::Matryoshka | RelicId::WingBoots) && state.floor > 40 { return false; }
    if id == RelicId::TinyChest && state.floor > 35 { return false; }
    if id == RelicId::PreservedInsect && state.floor > 52 { return false; }
    // Economy research §4.1: Ectoplasm is legal only through Act 1
    // (docs/research/2026-10-19-act1-content-economy-potions.md:228).
    if id == RelicId::Ectoplasm && state.act > 1 { return false; }

    // Economy research §4.1: floor < 48, fewer than two owned
    // (docs/research/2026-10-19-act1-content-economy-potions.md:227).
    let campfire_relic = matches!(id, RelicId::Girya | RelicId::PeacePipe | RelicId::Shovel);
    if campfire_relic {
        if state.floor >= 48 { return false; }
        let owned = [RelicId::Girya, RelicId::PeacePipe, RelicId::Shovel]
            .into_iter().filter(|candidate| state.relics.contains(candidate)).count();
        // The noncampfire selector family (Black Star's second elite relic) passes these three
        // as `exclude_take` exclusions; ordinary sources (chests, elites, shops, Dig) keep them.
        if owned >= 2 { return false; }
    }

    // Economy research §4.1 excludes bottles and Whetstone from screenless sources
    // (docs/research/2026-10-19-act1-content-economy-potions.md:216-221);
    // data/relics/relics.json excludes them from Calling Bell and noncombat direct-event offers.
    // Replacement rows and source restrictions: data/relics/relics.json:621-628, 3179-3186,
    // 3964-3971, 7472-7479.
    // Direct event/equip offers are screenless here;
    // visible reward, shop, chest, boss, and Neow sources are screenful. Whetstone shares the ban.
    if source_is_screenless(source) {
        if matches!(id, RelicId::BottledFlame | RelicId::BottledLightning | RelicId::BottledTornado | RelicId::Whetstone) { return false; }
    }
    let has_permanent = |predicate: fn(&crate::content::CardDefinition) -> bool| {
        state.deck.iter().filter(|card| crate::content::complete::supports_permanent(card.id))
            .filter_map(|card| catalog.card(card.id))
            .any(|definition| !definition.definition_only && predicate(definition))
    };
    match id {
        RelicId::BottledFlame => has_permanent(|card| card.card_type == CardType::Attack && card.rarity != CardRarity::Basic),
        RelicId::BottledLightning => has_permanent(|card| card.card_type == CardType::Skill && card.rarity != CardRarity::Basic),
        RelicId::BottledTornado => has_permanent(|card| card.card_type == CardType::Power),
        _ => true,
    }
}

// The research calls out selector families, not this native enum's individual variants
// (docs/research/2026-10-19-act1-content-economy-potions.md:216-217).
// Keep these mappings explicit and report their source-level limits in A19's Questions.
fn source_is_screenless(source: RelicSource) -> bool {
    matches!(source, RelicSource::Event | RelicSource::Equip(_))
}


#[cfg(test)]
pub(crate) fn explicit_for_test(common: Vec<RelicId>, uncommon: Vec<RelicId>, rare: Vec<RelicId>,
    shop: Vec<RelicId>, boss: Vec<RelicId>) -> RelicPools {
    RelicPools { common, uncommon, rare, shop, boss }
}

fn tier_for_roll(table: TierTable, roll: u8) -> RelicTier {
    assert!(roll <= 99, "tier roll is an integer from 0 through 99");
    match table {
        TierTable::Ordinary => match roll { 0..=49 => RelicTier::Common, 50..=82 => RelicTier::Uncommon, _ => RelicTier::Rare },
        TierTable::Merchant => match roll { 0..=47 => RelicTier::Common, 48..=81 => RelicTier::Uncommon, _ => RelicTier::Rare },
    }
}

#[cfg(test)]
pub(crate) fn roll_tier_with_roll(table: TierTable, roll: u8) -> RelicTier {
    tier_for_roll(table, roll)
}

#[cfg(test)]
pub(crate) fn spawn_allowed_for_test(state: &RunState, id: RelicId, source: RelicSource) -> bool {
    spawn_allowed(state, id, source)
}
