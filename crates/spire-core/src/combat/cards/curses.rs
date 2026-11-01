//! M26 curse behaviour seams (plan Task M1; filled by Lane A, Task A1).
//! Main owns the call sites; until A1 every body returns "no effect".
use super::*;
use super::super::model::OwnedCard;

/// Normality: whether further card plays are blocked this turn. Read by
/// `validate_card_use`, which also drives the legal-action mask.
pub(in crate::combat) fn play_blocked(state:&State)->bool{
    state.piles.hand.iter().filter_map(OwnedCard::full).any(|card|card.id==crate::content::CardId::Normality)
        &&state.silent.played_this_turn.len()>=3
}

/// End-of-turn auto-use while held (Decay, Doubt, Shame, Regret), called per hand card in
/// hand order from the Burn owner (`status::end_turn_work`), interleaved with Burn by position.
pub(in crate::combat) fn end_of_turn_in_hand(state:&State,card:&LiveCard,work:&mut Vec<Effect>){
    use crate::content::CardId::*;
    let source=Source::Card(crate::combat::card_state::handle(card.instance));
    let kind=match card.id{
        Decay=>EffectKind::Damage(DamagePayload::Thorns(2)),
        Doubt=>EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:1},
        Shame=>EffectKind::ApplyFullPower{id:FullPowerId("frail"),amount:1},
        Regret=>EffectKind::Damage(DamagePayload::HpLoss(i32::try_from(state.piles.hand.len())
            .expect("native hand cap fits Regret's damage payload"))),
        _=>return,
    };
    work.push(Effect{source,target:Some(Actor::Player),kind});
}

/// Pain: reaction to another card's `CardPlayed` while held.
pub(in crate::combat) fn on_card_played(state:&State,played:&LiveCard,work:&mut Vec<Effect>){
    if played.id==crate::content::CardId::Pain{return}
    for pain in state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card|card.id==crate::content::CardId::Pain){
        work.push(Effect{source:Source::Card(crate::combat::card_state::handle(pain.instance)),
            target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))});
    }
}

/// Necronomicurse: reaction to its own exhaust.
pub(in crate::combat) fn on_exhaust(_state:&State,card:&LiveCard,work:&mut Vec<Effect>){
    if card.id==crate::content::CardId::Necronomicurse{
        work.push(Effect{source:Source::Card(crate::combat::card_state::handle(card.instance)),
            target:Some(Actor::Player),kind:EffectKind::CreateCard{
                id:crate::content::CardId::Necronomicurse,
                destination:crate::combat::command::CardDestination::Hand,upgrades:0,
            }});
    }
}

/// Validator acceptance for curse-sourced packets: the source is a card of kind `id`
/// held in the hand (Decay, Doubt, Shame, Regret, Pain) or, for Necronomicurse's
/// returned copy, the exhausted card itself. Weak, Frail and HP loss already pass the
/// shared bounds checks for any card source; THORNS and card creation need this owner.
pub(in crate::combat) fn sourced_from(state:&State,source:&Source,id:crate::content::CardId)->bool{
    let Source::Card(owner)=source else{return false};
    let pile=if id==crate::content::CardId::Necronomicurse{&state.piles.exhaust}else{&state.piles.hand};
    pile.iter().any(|card|card.handle()==*owner&&card.full().is_some_and(|live|live.id==id))
}
