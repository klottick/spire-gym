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

// ---- M28b Lane C contract (bodies by the lane; signatures fixed by Main) ----
use crate::content::CardId;
use crate::run::{RunError, RunState};

/// The automatic card-upgrade chance by act (F3): Act 1 0, Act 2 0.25, Act 3 0.5 (binary32 A0
/// constructor parameters, acts23 research :51); Act 4 0.5 below A12 (PM source check
/// m28b/source-check-act4-upgrade-v1.md, F3).
pub(in crate::run) fn upgrade_chance(act:u8)->f32{
    match act {
        1 => 0.0_f32,
        2 => 0.25_f32,
        3 | 4 => 0.5_f32,
        _ => 0.0_f32,
    }
}

/// The upgrades a card row shows when offered: M27's egg preview (Molten Attack, Toxic Skill,
/// Frozen Power) on an upgradeable card. Used by reward and shop card rows.
pub(in crate::run) fn preview_upgrades(state:&RunState,id:CardId,upgrades:u32)->u32{
    if upgrades != 0 { return upgrades; }
    let card_type = crate::content::Catalog::standard().card(id).map(|card| card.card_type);
    let egg = match card_type {
        Some(crate::content::CardType::Attack) => Some(crate::content::RelicId::MoltenEgg),
        Some(crate::content::CardType::Skill) => Some(crate::content::RelicId::ToxicEgg),
        Some(crate::content::CardType::Power) => Some(crate::content::RelicId::FrozenEgg),
        _ => None,
    };
    let matching_egg = egg.is_some_and(|egg| state.relics.contains(&egg));
    if matching_egg
        && crate::combat::cards::ironclad::rules::can_upgrade(id, 0)
        && crate::content::complete::supports_permanent(id)
    { 1 } else { upgrades }
}

/// One ordinary card offer of `choices` slots (economy research :102-110).
pub(in crate::run) fn ordinary_offer(state:&mut RunState,room:RarityRoom,choices:usize,rare_factor:u32,key:&[u64])
    ->Result<Vec<(CardId,u32)>,RunError>{
    let mut staged = state.clone();
    let mut offset = staged.card_rarity_offset.ok_or(RunError::InvalidAction)?;
    let mut offered = Vec::with_capacity(choices);
    let has_shard = staged.relics.contains(&crate::content::RelicId::PrismaticShard);
    let chance = upgrade_chance(staged.act);

    for slot in 0..choices {
        let slot = u64::try_from(slot).map_err(|_| RunError::Unsupported("reward slot exhausted"))?;
        let rarity = if room == RarityRoom::Boss {
            crate::content::CardRarity::Rare
        } else {
            let mut rarity_key = key.to_vec();
            rarity_key.extend([0x5241, slot]);
            let n = staged.rng.bounded_scoped_checked(crate::run::RunRngScope::M28CardRarity, &rarity_key, 100)? as u32;
            classify(room, n, offset, rare_factor)
        };

        match rarity {
            crate::content::CardRarity::Rare => offset = 5,
            crate::content::CardRarity::Common => offset = offset.saturating_sub(1).max(-40),
            crate::content::CardRarity::Uncommon => (),
            _ => return Err(RunError::InvalidAction),
        }
        staged.card_rarity_offset = Some(offset);

        let pool = if has_shard {
            crate::content::pools::CardPool::AnyColour { rarity: Some(rarity) }
        } else {
            crate::content::pools::CardPool::Ordinary { character: staged.character, rarity: Some(rarity), card_type: None }
        };
        let cards = crate::content::pools::cards(pool).map_err(|_| RunError::Unsupported("card pool is incomplete"))?;
        if cards.is_empty() { return Err(RunError::Unsupported("card pool is empty")); }

        let mut attempt = 0_u64;
        let id = loop {
            let mut identity_key = key.to_vec();
            identity_key.extend([0x1D, slot, attempt]);
            let index = staged.rng.bounded_scoped_checked(
                crate::run::RunRngScope::M28CardRarity, &identity_key, cards.len() as u64)?;
            let id = cards[index];
            if !offered.iter().any(|(offered_id, _)| *offered_id == id) { break id; }
            if offered.len() >= cards.len() { return Err(RunError::Unsupported("reward card family has too few cards")); }
            attempt = attempt.checked_add(1).ok_or(RunError::Unsupported("reward identity retries exhausted"))?;
        };

        // F3b (PM source check m28b/source-check-act4-upgrade-v1.md): every Common and Uncommon
        // consumes one upgrade draw, even at chance 0 and before the upgradeability check; a Rare
        // draws nothing and is never automatically upgraded.
        let mut upgrades = 0;
        if rarity != crate::content::CardRarity::Rare {
            let mut upgrade_key = key.to_vec();
            upgrade_key.extend([0x0F, slot]);
            let u = staged.rng.bounded_scoped_checked(crate::run::RunRngScope::M28CardRarity, &upgrade_key, 1_u64 << 24)? as u32;
            let roll = u as f32 / 16_777_216.0_f32;
            if roll < chance
                && crate::combat::cards::ironclad::rules::can_upgrade(id, 0)
                && crate::content::complete::supports_permanent(id)
            { upgrades = 1; }
        }
        upgrades = preview_upgrades(&staged, id, upgrades);
        offered.push((id, upgrades));
    }

    *state = staged;
    Ok(offered)
}

/// One colourless card offer of `choices` slots (economy research :109).
pub(in crate::run) fn colourless_offer(state:&mut RunState,choices:usize,key:&[u64])
    ->Result<Vec<(CardId,u32)>,RunError>{
    let mut staged = state.clone();
    let mut offset = staged.card_rarity_offset;
    let mut offered = Vec::with_capacity(choices);

    for slot in 0..choices {
        let slot = u64::try_from(slot).map_err(|_| RunError::Unsupported("reward slot exhausted"))?;
        let mut rarity_key = key.to_vec();
        rarity_key.extend([0xC0, slot]);
        let u = staged.rng.bounded_scoped_checked(crate::run::RunRngScope::M28CardRarity, &rarity_key, 1_u64 << 24)? as u32;
        let roll = u as f32 / 16_777_216.0_f32;
        let rarity = if roll < 0.3_f32 { crate::content::CardRarity::Rare } else { crate::content::CardRarity::Uncommon };
        if rarity == crate::content::CardRarity::Rare { offset = Some(5); }
        staged.card_rarity_offset = offset;

        let cards = crate::content::pools::cards(crate::content::pools::CardPool::Colourless { rarity: Some(rarity) })
            .map_err(|_| RunError::Unsupported("card pool is incomplete"))?;
        if cards.is_empty() { return Err(RunError::Unsupported("card pool is empty")); }
        let mut attempt = 0_u64;
        let id = loop {
            let mut identity_key = key.to_vec();
            identity_key.extend([0x1D, slot, attempt]);
            let index = staged.rng.bounded_scoped_checked(
                crate::run::RunRngScope::M28CardRarity, &identity_key, cards.len() as u64)?;
            let id = cards[index];
            if !offered.iter().any(|(offered_id, _)| *offered_id == id) { break id; }
            if offered.len() >= cards.len() { return Err(RunError::Unsupported("reward card family has too few cards")); }
            attempt = attempt.checked_add(1).ok_or(RunError::Unsupported("reward identity retries exhausted"))?;
        };
        offered.push((id, preview_upgrades(&staged, id, 0)));
    }

    *state = staged;
    Ok(offered)
}
