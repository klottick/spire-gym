//! Native handlers for special and producer-created shared cards.
use super::*;

pub(super) fn payload(card: &LiveCard)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    use crate::content::CardId::*;
    let result = match card.id {
        Miracle => (0, vec![(Some(PayloadTarget::Player), EffectKind::GainEnergy(if card.upgrades > 0 { 2 } else { 1 }))]),
        Slimed => (1, Vec::new()),
        _ => return Ok(None),
    };
    Ok(Some(result))
}
