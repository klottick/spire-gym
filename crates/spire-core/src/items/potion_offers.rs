//! Complete-admission combat potion appearance law (M27 design §8.5).
use crate::{
    content::{PotionId, RelicId},
    run::{ItemAdmission, RunError, RunRngScope, RunState},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RewardRoom {
    Ordinary { enemy_escaped: bool },
    Elite,
    Event,
    Boss,
}

fn appearance(
    character: crate::content::Character,
    white_beast_statue: bool,
    offset: i32,
    room: RewardRoom,
    existing_rows: usize,
    mut draw: impl FnMut(u64) -> Result<usize, RunError>,
) -> Result<(Option<PotionId>, i32), RunError> {
    // PM source check 2026-11-02 (m6): when every monster escaped only the chance is 0; the roll
    // still happens and the offset keeps its history (+10 on a miss, -10 on a hit).
    let escaped = matches!(room, RewardRoom::Ordinary { enemy_escaped: true });
    let attempt_offset = offset;
    let mut chance = if escaped { 0 } else { 40i32.checked_add(attempt_offset)
        .ok_or(RunError::Unsupported("potion appearance chance overflow"))? };
    if white_beast_statue { chance = 100; }
    if existing_rows >= 4 { chance = 0; }

    let roll = draw(100)?;
    if roll >= 100 { return Err(RunError::InvalidAction); }
    if (roll as i32) < chance {
        let next_offset = attempt_offset.checked_sub(10)
            .ok_or(RunError::Unsupported("potion appearance offset exhausted"))?;
        let potion = super::potion_generation::rarity_first(character, |bound| {
            let index = draw(bound)?;
            if index >= bound as usize { return Err(RunError::InvalidAction); }
            Ok(index)
        })?;
        Ok((Some(potion), next_offset))
    } else {
        let next_offset = attempt_offset.checked_add(10)
            .ok_or(RunError::Unsupported("potion appearance offset exhausted"))?;
        Ok((None, next_offset))
    }
}

/// Roll appearance and, on success, sample the player's class pool. All draws for
/// the offer use the semantic M27 scope and the caller's key. `enemy_escaped` means every
/// monster escaped. A player Smoke Bomb escape rolls as a won ordinary fight (offset updated as
/// usual) and the caller discards any rolled potion with the hidden rewards (PM source check m6).
pub(crate) fn combat_appearance(
    state: &mut RunState,
    room: RewardRoom,
    existing_rows: usize,
    key: &[u64],
) -> Result<Option<PotionId>, RunError> {
    if state.item_admission != ItemAdmission::Complete { return Err(RunError::InvalidAction); }
    let mut staged = state.clone();
    let (potion, next_offset) = appearance(
        staged.character,
        staged.relics.contains(&RelicId::WhiteBeastStatue),
        staged.potion_offset,
        room,
        existing_rows,
        |bound| staged.rng.bounded_scoped_checked(RunRngScope::M27PotionOffer, key, bound),
    )?;
    staged.potion_offset = next_offset;
    *state = staged;
    Ok(potion)
}

#[cfg(test)]
pub(crate) fn combat_appearance_scripted(
    state: &mut RunState,
    room: RewardRoom,
    existing_rows: usize,
    draws: &[usize],
) -> Result<Option<PotionId>, RunError> {
    if state.item_admission != ItemAdmission::Complete { return Err(RunError::InvalidAction); }
    let mut cursor = 0;
    let (potion, next_offset) = appearance(
        state.character,
        state.relics.contains(&RelicId::WhiteBeastStatue),
        state.potion_offset,
        room,
        existing_rows,
        |bound| {
            let index = *draws.get(cursor).ok_or(RunError::InvalidAction)?;
            cursor += 1;
            if index >= bound as usize { return Err(RunError::InvalidAction); }
            Ok(index)
        },
    )?;
    state.potion_offset = next_offset;
    Ok(potion)
}
