//! Silent/common card payloads using the shared native resolver.
use super::*;

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    use crate::content::CardId::*;
    let upgraded = card.upgrades > 0;
    let result = match card.id {
        Backflip => (1, vec![
            (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 8 } else { 5 })?)),
            (None, EffectKind::Draw(2)),
        ]),
        Deflect => (0, vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 7 } else { 4 })?))]),
        Shiv => {
            let target_id = target.ok_or(crate::run::RunError::InvalidAction)?;
            let damage = super::modified_damage(state, target_id, if upgraded { 6 } else { 4 })?;
            (0, vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage)))])
        }
        QuickSlash | DaggerThrow => {
            let target_id = target.ok_or(crate::run::RunError::InvalidAction)?;
            let base = match (card.id, upgraded) {
                (QuickSlash, false) => 8, (QuickSlash, true) => 12,
                (DaggerThrow, false) => 9, (DaggerThrow, true) => 12,
                _ => unreachable!(),
            };
            let damage = super::modified_damage(state, target_id, base)?;
            let enemy = state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone();
            let mut effects = vec![(Some(PayloadTarget::Enemy(enemy)), EffectKind::Damage(DamagePayload::Fixed(damage)))];
            if card.id == QuickSlash {
                effects.push((None, EffectKind::Draw(1)));
            } else {
                effects.push((None, EffectKind::Draw(1)));
                effects.push((None, EffectKind::SelectHandCardsAfterDraw { required: 1,
                    disposition: super::super::SelectionDisposition::Discard,
                    empty: super::super::EmptySelection::Continue }));
            }
            (1, effects)
        }
        CloakAndDagger => {
            let mut effects = vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, 6)?)),
                (Some(PayloadTarget::Player), EffectKind::CreateCard { id: crate::content::CardId::Shiv,
                    destination: crate::combat::CardDestination::Hand, upgrades: 0 })];
            if upgraded {
                effects.push((Some(PayloadTarget::Player), EffectKind::CreateCard { id: crate::content::CardId::Shiv,
                    destination: crate::combat::CardDestination::Hand, upgrades: 0 }));
            }
            (1, effects)
        }
        _ => return Ok(None),
    };
    Ok(Some(result))
}
