//! Held Status checkpoints: these are not card plays and never enter in_play.
use super::*;
pub(super) fn end_turn_work(state:&model::State)->Vec<Effect>{
    state.piles.hand.iter().filter_map(|owned|{
        let card=owned.full()?;
        (card.id==crate::content::CardId::Burn).then(||Effect{source:Source::Card(owned.handle()),target:Some(Actor::Player),
            kind:EffectKind::Damage(DamagePayload::Thorns(if card.upgrades>0{4}else{2}))})
    }).collect()
}
