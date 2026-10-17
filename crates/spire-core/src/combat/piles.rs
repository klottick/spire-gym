//! Ordered pile membership and transactional discard recycling.
use super::*;
use super::model::State;

pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    super::engine::validate_effect(effect, state)?;
    match &effect.kind {
        EffectKind::Draw(requested) => {
            let room = 10usize.checked_sub(state.piles.hand.len())
                .ok_or_else(|| InputError::Invariant("hand exceeds cap".into()))?;
            // Read sizes at dispatch, not when this work was constructed. Total
            // cards is bounded by 999, and requested was validated in 0..=10.
            let count = (*requested as usize).min(room)
                .min(state.piles.draw.len() + state.piles.discard.len());
            if count > state.piles.draw.len() {
                let existing = state.piles.draw.len();
                // Front insertion reverses insertion order, not gameplay order.
                for kind in [EffectKind::Draw((count - existing) as i32),
                    EffectKind::Recycle, EffectKind::Draw(existing as i32)] {
                    if kind != EffectKind::Draw(0) {
                        state.pending.push_front(Effect { source: effect.source.clone(),
                            target: effect.target.clone(), kind });
                    }
                }
            } else {
                for _ in 0..count {
                    let card = state.piles.draw.pop().unwrap();
                    state.public.revealed_order.push(card.instance_id.clone());
                    state.piles.hand.push(card);
                }
            }
        }
        EffectKind::Recycle => {
            // shuffle owns only a detached permutation. On failure neither pile
            // commits; the resolver installs the error's latest exhausted cursor.
            let (cards, next) = super::rng::shuffle(&state.rng.shuffle, &state.piles.discard)?;
            state.rng.shuffle = next;
            state.piles.discard.clear();
            state.piles.draw.extend(cards);
        }
        EffectKind::DiscardHand => {
            while let Some(card) = state.piles.hand.last() {
                let id = card.instance_id.clone();
                discard_card(state, &id, true)?;
            }
        }
        _ => return Err(InputError::Unsupported("not a pile effect".into()).into()),
    }
    Ok(())
}

pub(super) fn discard_card(state: &mut State, card: &CardHandle, end_turn: bool) -> Result<(), InputError> {
    let moved = if end_turn {
        let index = state.piles.hand.iter().position(|c| &c.instance_id == card)
            .ok_or_else(|| InputError::Invariant("discard hand owner missing".into()))?;
        state.piles.hand.remove(index)
    } else {
        if state.in_play.as_ref().map(|c| &c.instance_id) != Some(card) {
            return Err(InputError::Invariant("discard in-play owner missing".into()));
        }
        state.in_play.take().unwrap()
    };
    state.piles.discard.push(moved);
    Ok(())
}
