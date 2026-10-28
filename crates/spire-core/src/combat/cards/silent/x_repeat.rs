//! X-cost, actual-use repeat, and captured-hand Silent card work.
use super::*;
use super::state::SilentEffect;
use crate::combat::model::OwnedCard;
use crate::content::CardId;

fn captured_x(state: &State, card: &LiveCard) -> Result<i32, crate::run::RunError> {
    super::super::captured_x(state,card)
}

fn chemical_x_unavailable(state: &State) -> bool {
    state.item_context.as_ref().is_some_and(|items|
        items.relics().iter().any(|relic| relic.as_str() == "chemical_x"))
}

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    use CardId::*;
    if !matches!(card.id, Skewer | Doppelganger | Malaise | StormOfSteel) {
        return Ok(None);
    }
    let targeted = matches!(card.id, Skewer | Malaise);
    if targeted != target.is_some() { return Err(crate::run::RunError::InvalidAction); }
    let upgraded = card.upgrades > 0;
    let mut effects = Vec::new();
    let cost = match card.id {
        Skewer => {
            if chemical_x_unavailable(state) {
                return Err(crate::run::RunError::Unsupported("Chemical X modifier is not represented for Skewer X payment"));
            }
            let amount = captured_x(state, card)?;
            let damage = super::super::modified_damage(state, target.unwrap(), if upgraded { 10 } else { 7 })?;
            for _ in 0..amount {
                effects.push((Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            0
        }
        Doppelganger => {
            if chemical_x_unavailable(state) {
                return Err(crate::run::RunError::Unsupported("Chemical X modifier is not represented for Doppelganger X payment"));
            }
            let amount = captured_x(state, card)?.checked_add(i32::from(upgraded))
                .ok_or(crate::run::RunError::Unsupported("Doppelganger X amount overflow"))?;
            if amount > 999 { return Err(crate::run::RunError::Unsupported("Doppelganger amount outside supported power surface")); }
            if amount > 0 {
                effects.push((Some(PayloadTarget::Player), EffectKind::ApplyFullPower {
                    id: FullPowerId("draw_card_next_turn"), amount,
                }));
                effects.push((Some(PayloadTarget::Player), EffectKind::ApplyFullPower {
                    id: FullPowerId("energized"), amount,
                }));
            }
            0
        }
        Malaise => {
            if chemical_x_unavailable(state) {
                return Err(crate::run::RunError::Unsupported("Chemical X modifier is not represented for Malaise X payment"));
            }
            let amount = captured_x(state, card)?.checked_add(i32::from(upgraded))
                .ok_or(crate::run::RunError::Unsupported("Malaise X amount overflow"))?;
            if amount > 999 { return Err(crate::run::RunError::Unsupported("Malaise amount outside supported power surface")); }
            if amount > 0 {
                effects.push((Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower {
                    id: FullPowerId("strength"), amount: -amount,
                }));
                effects.push((Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower {
                    id: FullPowerId("weak"), amount,
                }));
            }
            0
        }
        StormOfSteel => {
            let owner = crate::combat::card_state::handle(card.instance);
            let cards = state.piles.hand.iter().filter(|owned| owned.handle() != owner)
                .map(OwnedCard::handle).collect::<Vec<_>>();
            effects.push((None, EffectKind::Silent(SilentEffect::StormOfSteel {
                cards, shiv_upgrades: u32::from(upgraded),
            })));
            super::costs::starting_cost(card.id, card.upgrades).unwrap()
        }
        _ => unreachable!(),
    };
    Ok(Some((cost, effects)))
}

pub(in crate::combat) fn apply(state: &mut State, effect: &Effect, work: &SilentEffect)
    -> Result<(), EffectError>
{
    let SilentEffect::StormOfSteel { cards, shiv_upgrades } = work else {
        return Err(InputError::Invariant("non-Storm work reached X/repeat dispatcher".into()).into());
    };
    for handle in cards {
        if let Some(index) = state.piles.hand.iter().position(|owned| owned.handle() == *handle) {
            let card = state.piles.hand.remove(index);
            crate::combat::piles::discard_selected_card(state, card, true)?;
        }
    }

    // Discard listeners can queue front work or append before Decision. Place
    // the captured Shiv batch after that reaction work and before the barrier.
    let insertion = state.pending.iter().position(|pending| pending.kind == EffectKind::Decision)
        .unwrap_or(state.pending.len());
    let shivs = cards.iter().map(|_| Effect {
        source: effect.source.clone(),
        target: Some(Actor::Player),
        kind: EffectKind::CreateCard {
            id: CardId::Shiv,
            destination: crate::combat::CardDestination::Hand,
            upgrades: *shiv_upgrades,
        },
    }).collect::<Vec<_>>();
    for (offset, shiv) in shivs.into_iter().enumerate() {
        state.pending.insert(insertion + offset, shiv);
    }
    Ok(())
}
