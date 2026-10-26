//! Source-specific reactions for explicit selected hand discards.
use crate::combat::{Actor, Effect, EffectKind, InputError, LiveCard, Source};
use crate::combat::model::State;
use crate::content::CardId;

/// Queue the only Task 3 listeners from the already committed selected card.
/// Reflex appends behind existing parent work and before its Decision barrier;
/// Tactician is top-queued ahead of the remaining parent work.
pub(super) fn manual_discard(state: &mut State, card: &LiveCard) -> Result<bool, InputError> {
    if !state.full_runtime {
        return Ok(false);
    }
    if let Some(amount) = state.player_full_powers.iter()
        .find(|power| power.id.0 == "accuracy" && power.state.amount > 0)
        .map(|power| power.state.amount) {
        super::shiv::rewrite_hand(state, amount);
    }
    let (kind, append) = match card.id {
        CardId::Reflex => (EffectKind::Draw(if card.upgrades > 0 { 3 } else { 2 }), true),
        CardId::Tactician => (EffectKind::GainEnergy(if card.upgrades > 0 { 2 } else { 1 }), false),
        _ => return Ok(false),
    };
    let effect = Effect {
        source: Source::Card(super::super::super::card_state::handle(card.instance)),
        target: Some(Actor::Player),
        kind,
    };
    crate::combat::engine::validate_effect(&effect, state)?;
    if append {
        let decision = state.pending.iter().position(|queued| queued.kind == EffectKind::Decision)
            .unwrap_or(state.pending.len());
        state.pending.insert(decision, effect);
    } else {
        state.pending.push_front(effect);
    }
    Ok(true)
}
