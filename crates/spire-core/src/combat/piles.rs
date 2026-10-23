//! Ordered pile membership and transactional discard recycling.
use super::*;
use super::model::{OwnedCard, State};

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
                    state.public.revealed_order.push(card.handle());
                    state.piles.hand.push(card);
                    if state.full_runtime {
                        state.pending.push_front(Effect { source: effect.source.clone(), target: effect.target.clone(),
                            kind: EffectKind::Hook(Hook::CardDrawn) });
                    }
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
                let id = card.handle();
                discard_card(state, &id, true)?;
            }
        }
        EffectKind::CreateCard { id, destination, upgrades } => {
            let instance = next_generated_instance(state)?;
            let card = super::card_state::generated(*id, instance, *upgrades)
                .ok_or_else(|| InputError::Unsupported("generated card constructor unavailable".into()))?;
            insert_generated(state, card, *destination)?;
        }
        EffectKind::ExhaustHandCard { instance } => {
            let index = state.piles.hand.iter().position(|card| card.full().is_some_and(|c| c.instance == *instance))
                .ok_or_else(|| InputError::Invariant("forced exhaust card missing".into()))?;
            let card = state.piles.hand.remove(index);
            exhaust_owned_card(state, card)?;
        }
        EffectKind::CopyCard { source, destination } => {
            let original = match (&state.in_play, &effect.source) {
                (Some(OwnedCard::Full(card)), Source::Card(owner))
                    if owner == source && owner == &super::card_state::handle(card.instance) => card.clone(),
                _ => return Err(InputError::Invariant("copy source is not current in-play owner".into()).into()),
            };
            let instance = next_generated_instance(state)?;
            let copy = original.copy_as(instance);
            insert_generated(state, copy, *destination)?;
        }
        _ => return Err(InputError::Unsupported("not a pile effect".into()).into()),
    }
    Ok(())
}

fn next_generated_instance(state: &State) -> Result<super::CardInstanceId, InputError> {
    if !state.full_runtime || state.card_ids_issued.len() >= 999 {
        return Err(InputError::Unsupported("generated card count/runtime boundary".into()));
    }
    let mut maximum = 0_u64;
    for handle in &state.card_ids_issued {
        let issued = handle.as_str().parse::<u64>()
            .map_err(|_| InputError::Invariant("issued full card id is not unsigned numeric".into()))?;
        maximum = maximum.max(issued);
    }
    let next = maximum.checked_add(1)
        .ok_or_else(|| InputError::Unsupported("generated card identity exhausted".into()))?;
    let handle = CardHandle::parse(&next.to_string())?;
    if state.card_ids_issued.contains(&handle) {
        return Err(InputError::Invariant("generated card identity collision".into()));
    }
    Ok(super::CardInstanceId(next))
}

fn insert_generated(state: &mut State, card: super::card_state::LiveCard,
    requested: super::command::CardDestination) -> Result<(), InputError> {
    let instance = card.instance;
    let handle = super::card_state::handle(instance);
    if !state.full_runtime || state.card_ids_issued.len() >= 999
        || state.card_ids_issued.contains(&handle) {
        return Err(InputError::Invariant("generated card issuance precondition".into()));
    }
    let destination = if requested == super::command::CardDestination::Hand
        && state.piles.hand.len() >= 10 {
        super::command::CardDestination::Discard
    } else { requested };
    let owned = OwnedCard::Full(card);
    state.card_ids_issued.insert(handle);
    match destination {
        super::command::CardDestination::Hand => state.piles.hand.push(owned),
        // Draw reads from the Vec tail (`pop`), so tail insertion is top-of-draw.
        super::command::CardDestination::Draw => state.piles.draw.push(owned),
        super::command::CardDestination::Discard => state.piles.discard.push(owned),
    }
    Ok(())
}

/// Callers validate before detaching. Ownership is committed before notifying;
/// multiple synchronous selected exhaustions keep notification order.
pub(super) fn exhaust_owned_card(state: &mut State, card: OwnedCard) -> Result<(), InputError> {
    let handle = card.handle();
    state.piles.exhaust.push(card);
    if state.full_runtime {
        let after_exhaust_hooks = state.pending.iter().take_while(|work|
            work.kind == EffectKind::Hook(Hook::CardExhausted)).count();
        state.pending.insert(after_exhaust_hooks, Effect { source: Source::Card(handle), target: None,
            kind: EffectKind::Hook(Hook::CardExhausted) });
    }
    Ok(())
}

pub(super) fn discard_card(state: &mut State, card: &CardHandle, end_turn: bool) -> Result<(), InputError> {
    let exhaust = !end_turn && state.in_play.as_ref().and_then(OwnedCard::full)
        .map_or(false, |live| live.local.exhaust_on_finish);
    let moved = if end_turn {
        let index = state.piles.hand.iter().position(|c| c.handle() == *card)
            .ok_or_else(|| InputError::Invariant("discard hand owner missing".into()))?;
        state.piles.hand.remove(index)
    } else {
        if state.in_play.as_ref().map(OwnedCard::handle).as_ref() != Some(card) {
            return Err(InputError::Invariant("discard in-play owner missing".into()));
        }
        state.in_play.take().unwrap()
    };
    if exhaust { exhaust_owned_card(state, moved)?; } else { state.piles.discard.push(moved); }
    if state.full_runtime && !exhaust {
        state.pending.push_front(Effect { source: Source::Card(card.clone()), target: None,
            kind: EffectKind::Hook(Hook::CardDiscarded) });
    }
    Ok(())
}
