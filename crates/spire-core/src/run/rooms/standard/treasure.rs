//! Standard treasure (design §6.4; economy research :260-298). Lane C bodies; signatures fixed by Main.
use crate::content::{RelicId, RelicTier};
use crate::run::{RoomChoiceKind, RunError, RunState};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(in crate::run) enum ChestSize { Small, Medium, Large }

/// Size from one 0..100 draw: small 0–49, medium 50–82, large 83–99.
pub(in crate::run) fn chest_size(roll:u32)->ChestSize{
    if roll < 50 { ChestSize::Small } else if roll < 83 { ChestSize::Medium } else { ChestSize::Large }
}
/// Tier and gold presence from the second, shared 0..100 draw (economy research :260-267).
pub(in crate::run) fn chest_tier_gold(size:ChestSize,roll:u32)->(RelicTier,bool){
    match size {
        ChestSize::Small if roll < 50 => (RelicTier::Common, true),
        ChestSize::Small if roll < 75 => (RelicTier::Common, false),
        ChestSize::Small => (RelicTier::Uncommon, false),
        ChestSize::Medium if roll < 35 => (RelicTier::Common, true),
        ChestSize::Medium if roll < 85 => (RelicTier::Uncommon, false),
        ChestSize::Medium => (RelicTier::Rare, false),
        ChestSize::Large if roll < 50 => (RelicTier::Uncommon, true),
        ChestSize::Large if roll < 75 => (RelicTier::Uncommon, false),
        ChestSize::Large => (RelicTier::Rare, false),
    }
}
/// Chest gold `J(L(F(N×F(0.9)), F(N×F(1.1)), k))` for nominal N, k in 0..2^24 (:269-273).
pub(in crate::run) fn chest_gold(nominal:u32,k:u32)->u32{
    let a = (nominal as f32) * 0.9_f32;
    let b = (nominal as f32) * 1.1_f32;
    let ratio = (k as f32) / 16_777_216.0_f32;
    let span = b - a;
    let interpolation = ratio * span;
    let x = a + interpolation;
    ((x + 0.5_f32).floor() as i32) as u32
}
/// Open an ordinary chest: rows in source order (economy research :275-291). Atomic on error.
pub(in crate::run) fn open_rows(state:&mut RunState,key:&[u64])->Result<Vec<RoomChoiceKind>,RunError>{
    let mut staged = state.clone();
    let rules = crate::items::run_laws::chest_contents(&mut staged, false)?;

    if rules.curse {
        let mut curse_key = key.to_vec();
        curse_key.push(0xC5);
        let curse = crate::run::mutations::draw_card(&mut staged, crate::content::pools::CardPool::Curse, &curse_key)?;
        crate::run::mutations::obtain_card_or_negated(&mut staged, curse, 0)?;
    }

    let mut size_key = key.to_vec();
    size_key.push(0x51);
    let size_roll = staged.rng.bounded_scoped_checked(crate::run::RunRngScope::M28Chest, &size_key, 100)? as u32;
    let size = chest_size(size_roll);
    let mut tier_key = key.to_vec();
    tier_key.push(0x71);
    let tier_roll = staged.rng.bounded_scoped_checked(crate::run::RunRngScope::M28Chest, &tier_key, 100)? as u32;
    let (tier, gold) = chest_tier_gold(size, tier_roll);

    let mut rows = Vec::with_capacity(3);
    if gold {
        let mut gold_key = key.to_vec();
        gold_key.push(0x601);
        let k = staged.rng.bounded_scoped_checked(crate::run::RunRngScope::M28Chest, &gold_key, 1_u64 << 24)? as u32;
        let nominal = match size { ChestSize::Small => 25, ChestSize::Medium => 50, ChestSize::Large => 75 };
        rows.push(RoomChoiceKind::Gold(chest_gold(nominal, k)));
    }

    let mut relic_key = key.to_vec();
    relic_key.push(0xB5);
    let relic = crate::items::relic_pools::take(&mut staged, tier, crate::items::relic_pools::Selector::Front,
        crate::items::lifecycle::RelicSource::Chest, &relic_key)?;
    rows.push(RoomChoiceKind::Relic(relic));
    // Sapphire only while Act 4 is reachable and the key is unowned (economy research :275-291).
    if staged.act4_enabled && !staged.keys[1] { rows.push(RoomChoiceKind::Sapphire); }

    crate::items::run_laws::apply_chest_rules(&rules, &mut rows);
    *state = staged;
    Ok(rows)
}
/// Open the boss chest: three sequential Boss-tier offers (economy research :293-298). Atomic on error.
pub(in crate::run) fn boss_offers(state:&mut RunState,key:&[u64])->Result<Vec<RelicId>,RunError>{
    let mut staged = state.clone();
    let _rules = crate::items::run_laws::chest_contents(&mut staged, true)?;
    let mut offers = Vec::with_capacity(3);
    for index in 0..3_u64 {
        let mut relic_key = key.to_vec();
        relic_key.extend([0xB055, index]);
        offers.push(crate::items::relic_pools::take(&mut staged, RelicTier::Boss,
            crate::items::relic_pools::Selector::Front, crate::items::lifecycle::RelicSource::Boss, &relic_key)?);
    }
    *state = staged;
    Ok(offers)
}
