//! Blade Dance, Accuracy, and the ordinary generated Shiv card.
use super::super::{EffectKind, FullPowerId, LiveCard, PayloadTarget, State};
use crate::{content::CardId, run::RunError};

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, RunError>
{
    let upgraded = card.upgrades > 0;
    let payload = match card.id {
        CardId::Accuracy => {
            if target.is_some() { return Err(RunError::InvalidAction); }
            (1, vec![(Some(PayloadTarget::Player), EffectKind::ApplyFullPower {
                id: FullPowerId("accuracy"), amount: if upgraded { 6 } else { 4 },
            })])
        }
        CardId::BladeDance => {
            if target.is_some() { return Err(RunError::InvalidAction); }
            let count = if upgraded { 4 } else { 3 };
            (1, (0..count).map(|_| (Some(PayloadTarget::Player), EffectKind::CreateCard {
                id: CardId::Shiv, destination: crate::combat::CardDestination::Hand, upgrades: 0,
            })).collect())
        }
        CardId::Shiv => {
            let target_id = target.ok_or(RunError::InvalidAction)?;
            let base = card.local.base_damage.unwrap_or(if upgraded { 6 } else { 4 });
            let damage = super::super::modified_damage(state, target_id, base)?;
            (0, vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(
                super::super::DamagePayload::Fixed(damage)))])
        }
        _ => return Ok(None),
    };
    Ok(Some(payload))
}

pub(super) fn rewrite_existing(state: &mut State, amount: i32) {
    for owned in state.piles.hand.iter_mut().chain(&mut state.piles.draw)
        .chain(&mut state.piles.discard).chain(&mut state.piles.exhaust)
    {
        if let Some(card) = owned.full_mut().filter(|card| card.id == CardId::Shiv) {
            card.local.base_damage = Some((if card.upgrades > 0 { 6 } else { 4 }) + amount);
        }
    }
}

pub(super) fn rewrite_hand(state: &mut State, amount: i32) {
    for owned in &mut state.piles.hand {
        if let Some(card) = owned.full_mut().filter(|card| card.id == CardId::Shiv) {
            card.local.base_damage = Some((if card.upgrades > 0 { 6 } else { 4 }) + amount);
        }
    }
}
