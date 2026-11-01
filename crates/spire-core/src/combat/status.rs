//! Held Status checkpoints: these are not card plays and never enter in_play.
use super::*;
pub(super) fn end_turn_work(state:&model::State)->Vec<Effect>{
    let mut work=super::enemies::act3::ordinary::player_end_work(state);
    for owned in &state.piles.hand{
        let Some(card)=owned.full() else{continue};
        if card.id==crate::content::CardId::Burn{
            work.push(Effect{source:Source::Card(owned.handle()),target:Some(Actor::Player),
                kind:EffectKind::Damage(DamagePayload::Thorns(if card.upgrades>0{4}else{2}))});
        }else{super::cards::curses::end_of_turn_in_hand(state,card,&mut work);}
    }
    work
}
