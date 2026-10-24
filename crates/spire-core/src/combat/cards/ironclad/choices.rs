//! Action-time hand/discard/exhaust selections for finite Ironclad card effects.
use super::*;
use crate::combat::{selection::{SelectionOperation,SelectionSourceZone},model::OwnedCard};

pub(super) fn apply(state:&mut State,effect:&Effect,choice:&IroncladEffect)->Result<(),EffectError>{
    let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("Ironclad choice has no card parent".into()).into())};
    if state.in_play.as_ref().map(OwnedCard::handle).as_ref()!=Some(parent){
        return Err(InputError::Invariant("Ironclad choice lost its in-play parent".into()).into())
    }
    match choice{
        IroncladEffect::ChooseTrueGritCard=>{
            let candidates=state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>();
            if !candidates.is_empty(){
                super::super::super::effects::begin_selection(state,parent.clone(),candidates,1,
                    SelectionDisposition::Exhaust,EmptySelection::Continue)?;
            }
        }
        IroncladEffect::ChooseHeadbuttCard=>{
            if state.outcome.is_some()||super::super::super::enemies::act2::combat_victory_ready(state){return Ok(())}
            let candidates=state.piles.discard.iter().map(OwnedCard::handle).collect::<Vec<_>>();
            match candidates.len(){
                0=>(),1=>move_forced(state,&candidates[0],SelectionSourceZone::Discard,SelectionDisposition::DrawTop)?,
                _=>{
                    super::super::super::effects::begin_selection(state,parent.clone(),candidates,1,
                        SelectionDisposition::DrawTop,EmptySelection::Continue)?;
                    state.pending_selection.as_mut().unwrap().operation=SelectionOperation::Topdeck;
                }
            }
        }
        IroncladEffect::ChooseExhumeCard=>{
            if state.piles.hand.len()>=10{return Ok(())}
            let candidates=state.piles.exhaust.iter().filter(|owned|
                owned.full().is_none_or(|card|card.id!=crate::content::CardId::Exhume))
                .map(OwnedCard::handle).collect::<Vec<_>>();
            match candidates.len(){
                0=>(),1=>move_forced(state,&candidates[0],SelectionSourceZone::Exhaust,SelectionDisposition::Hand)?,
                _=>{
                    super::super::super::effects::begin_selection(state,parent.clone(),candidates,1,
                        SelectionDisposition::Hand,EmptySelection::Continue)?;
                    state.pending_selection.as_mut().unwrap().operation=SelectionOperation::ReturnFromExhaust;
                }
            }
        }
        IroncladEffect::ChooseDualWieldCard=>{
            if state.player_full_powers.iter().any(|power|power.id.0=="master_reality"&&power.state.amount>0){
                return Err(InputError::Unsupported("Master Reality card-creation modifier is not implemented for Dual Wield".into()).into())
            }
            let copies=state.in_play.as_ref().and_then(OwnedCard::full).map(|card|if card.upgrades>0{2}else{1})
                .ok_or_else(||InputError::Invariant("Dual Wield parent missing".into()))?;
            let candidates=state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|
                crate::content::Catalog::standard().card(card.id).is_some_and(|definition|
                    matches!(definition.card_type,crate::content::CardType::Attack|crate::content::CardType::Power)))
                .map(|card|super::super::card_handle(card)).collect::<Vec<_>>();
            match candidates.as_slice(){
                []=>(),
                [only]=>{
                    let source=state.piles.hand.iter().find(|card|card.handle()==*only).and_then(OwnedCard::full).unwrap().clone();
                    super::super::super::piles::stat_copy_to_hand(state,&source,copies)?;
                }
                _=>{
                    super::super::super::effects::begin_selection(state,parent.clone(),candidates,1,
                        SelectionDisposition::Hand,EmptySelection::Continue)?;
                    state.pending_selection.as_mut().unwrap().operation=SelectionOperation::CopyToHand{copies:copies+1};
                }
            }
        }
        _=>return Err(InputError::Invariant("non-choice Ironclad effect reached choices dispatcher".into()).into()),
    }
    Ok(())
}

fn move_forced(state:&mut State,id:&crate::combat::CardHandle,zone:SelectionSourceZone,
    disposition:SelectionDisposition)->Result<(),EffectError>{
    let pile=match zone{SelectionSourceZone::Hand=>&mut state.piles.hand,SelectionSourceZone::Draw=>&mut state.piles.draw,
        SelectionSourceZone::Discard=>&mut state.piles.discard,SelectionSourceZone::Exhaust=>&mut state.piles.exhaust};
    let index=pile.iter().position(|card|card.handle()==*id)
        .ok_or_else(||InputError::Invariant("forced selection source disappeared".into()))?;
    let card=pile.remove(index);
    match disposition{
        SelectionDisposition::Hand=>state.piles.hand.push(card),
        SelectionDisposition::DrawTop=>{let empty=state.piles.draw.is_empty();state.piles.draw.push(card);state.public.known_draw.insert(0,id.clone());if empty{state.public.known_draw_bottom=vec![id.clone()];}},
        SelectionDisposition::Discard=>state.piles.discard.push(card),
        SelectionDisposition::Exhaust=>state.piles.exhaust.push(card),
    }
    Ok(())
}
