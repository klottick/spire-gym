//! Silent Poison card applications on the shared full-combat resolver.
use super::*;

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<super::super::PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    use crate::content::CardId;
    use super::super::PayloadTarget;

    if card.id == CardId::NoxiousFumes {
        if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
        return Ok(Some((1, vec![(Some(PayloadTarget::Player), EffectKind::ApplyFullPower {
            id: FullPowerId("noxious_fumes"), amount: if card.upgrades > 0 { 3 } else { 2 },
        })])));
    }

    let upgraded = card.upgrades > 0;
    let Some(target_id) = target else {
        return if matches!(card.id, CardId::DeadlyPoison | CardId::PoisonedStab | CardId::Catalyst) {
            Err(crate::run::RunError::InvalidAction)
        } else { Ok(None) };
    };
    let _enemy = state.enemies.get((target_id - 1) as usize)
        .ok_or(crate::run::RunError::InvalidAction)?;
    let result = match card.id {
        CardId::DeadlyPoison => (1, vec![(Some(PayloadTarget::SelectedEnemy),
            EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount: if upgraded { 7 } else { 5 } })]),
        CardId::PoisonedStab => {
            let damage = super::super::modified_damage(state, target_id, if upgraded { 8 } else { 6 })?;
            (1, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower {
                    id: FullPowerId("poison"), amount: if upgraded { 4 } else { 3 },
                }),
            ])
        }
        CardId::Catalyst => {
            (1, vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::Catalyst)])
        }
        _ => return Ok(None),
    };
    Ok(Some(result))
}

pub(super) fn apply_catalyst(state: &mut State, effect: &Effect) -> Result<(), super::super::EffectError> {
    let Some(Actor::Enemy(owner)) = effect.target.as_ref() else {
        return Err(InputError::Invariant("Catalyst action target absent".into()).into());
    };
    let upgraded = state.in_play.as_ref().and_then(|card| card.full())
        .is_some_and(|card| card.id == crate::content::CardId::Catalyst && card.upgrades > 0);
    let current = state.enemies.iter().find(|enemy| &enemy.instance_id == owner)
        .ok_or_else(|| InputError::Invariant("Catalyst target disappeared".into()))?
        .full_powers.iter().find(|power| power.id.0 == "poison")
        .map_or(0, |power| power.state.amount);
    if current <= 0 { return Ok(()); }
    let amount = if upgraded {
        current.checked_mul(2).ok_or_else(|| InputError::Unsupported(
            "Catalyst Poison application amount overflow".into()))?
    } else { current };
    let application = Effect { source: effect.source.clone(), target: effect.target.clone(),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount } };
    super::super::super::engine::validate_effect(&application, state)?;
    state.pending.push_front(application);
    Ok(())
}
