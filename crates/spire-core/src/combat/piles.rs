//! Ordered pile membership and transactional discard recycling.
use super::*;
use super::model::{FinishDisposition,OwnedCard, State};

pub(super) fn expire_turn_scoped_costs(state:&mut State){
    for card in state.piles.hand.iter_mut().chain(&mut state.piles.draw).chain(&mut state.piles.discard).chain(&mut state.piles.exhaust).chain(state.in_play.iter_mut()){
        if let Some(card)=card.full_mut(){if card.local.infernal_blade_cost_this_turn{card.local.temporary_cost=0;card.local.infernal_blade_cost_this_turn=false;}}
    }
}

pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    super::engine::validate_effect(effect, state)?;
    match &effect.kind {
        EffectKind::Draw(requested) => {
            if state.full_runtime&&state.player_full_powers.iter().any(|power|power.id.0=="no_draw"&&power.state.amount>0){return Ok(())}
            if state.full_runtime {
                let room=10usize.checked_sub(state.piles.hand.len()).ok_or_else(||InputError::Invariant("hand exceeds cap".into()))?;
                let count=(*requested as usize).min(room).min(state.piles.draw.len()+state.piles.discard.len());
                if count==0{return Ok(())}
                if state.piles.draw.is_empty(){
                    state.pending.push_front(Effect{source:effect.source.clone(),target:effect.target.clone(),kind:EffectKind::Draw(count as i32)});
                    state.pending.push_front(Effect{source:effect.source.clone(),target:effect.target.clone(),kind:EffectKind::Recycle});
                }else{
                    if count>1{state.pending.push_front(Effect{source:effect.source.clone(),target:effect.target.clone(),kind:EffectKind::Draw(count as i32-1)});}
                    state.pending.push_front(Effect{source:effect.source.clone(),target:effect.target.clone(),kind:EffectKind::DrawOne});
                }return Ok(())
            }
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
            state.public.known_draw.clear();
            state.public.known_draw_bottom.clear();
        }
        EffectKind::DiscardHand => {
            if let Some(retain)=&state.temporary_retain {
                if retain.turn!=state.turn||retain.cards.iter().any(|handle|!state.piles.hand.iter().any(|card|
                    card.handle()==*handle&&card.full().is_some_and(|live|!live.local.ethereal))){
                    return Err(InputError::Invariant("temporary retain owner is not in the current end-turn hand".into()).into());
                }
            }
            while let Some(card) = state.piles.hand.iter().rev().find(|card|!state.temporary_retain.as_ref()
                .is_some_and(|retain|retain.cards.contains(&card.handle()))) {
                let id = card.handle();
                discard_card(state, &id, true)?;
            }
            state.temporary_retain=None;
        }
        EffectKind::DrawOne=>{
            let card=state.piles.draw.pop().ok_or_else(||InputError::Invariant("draw-one owner absent".into()))?;let handle=card.handle();
            if state.public.known_draw.first()==Some(&handle){state.public.known_draw.remove(0);}else{state.public.known_draw.clear();}
            if state.piles.draw.len()<state.public.known_draw_bottom.len(){
                let bottom_offset=state.piles.draw.len();
                if state.public.known_draw_bottom.get(bottom_offset)==Some(&handle){state.public.known_draw_bottom.remove(bottom_offset);}
                else{state.public.known_draw_bottom.clear();}
            }
            state.public.revealed_order.push(handle.clone());state.piles.hand.push(card);
            state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::AfterCardDrawn{card:handle}});
        },
        EffectKind::AfterCardDrawn{card}=>{
            after_draw(state,card)?;
            state.pending.push_front(Effect{source:effect.source.clone(),target:effect.target.clone(),kind:EffectKind::Hook(Hook::CardDrawn)});
        },
        EffectKind::CreateCard { id, destination, upgrades } => {
            let instance = next_generated_instance(state)?;
            let card = super::card_state::generated_with_combat_history(*id, instance, *upgrades,state.damage_events_this_combat)
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
            // Anger's real producer is stat-equivalent; the public `copy_as`
            // adapter remains available with its historical local-clone semantics.
            let copy = original.stat_equivalent_copy(instance);
            insert_generated(state, copy, *destination)?;
        }
        _ => return Err(InputError::Unsupported("not a pile effect".into()).into()),
    }
    Ok(())
}

pub(super) fn next_generated_instance(state: &State) -> Result<super::CardInstanceId, InputError> {
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

pub(super) fn insert_generated(state: &mut State, mut card: super::card_state::LiveCard,
    requested: super::command::CardDestination) -> Result<(), EffectError> {
    if card.id == crate::content::CardId::Shiv && card.local.base_damage.is_none() {
        let accuracy = state.player_full_powers.iter()
            .find(|power| power.id.0 == "accuracy" && power.state.amount > 0)
            .map_or(0, |power| power.state.amount);
        if accuracy > 0 {
            let base: i32 = if card.upgrades > 0 { 6 } else { 4 };
            card.local.base_damage = Some(base.checked_add(accuracy)
                .ok_or_else(|| InputError::Unsupported("Accuracy Shiv damage overflow".into()))?);
        }
    }
    let instance = card.instance;
    let handle = super::card_state::handle(instance);
    if !state.full_runtime || state.card_ids_issued.len() >= 999
        || state.card_ids_issued.contains(&handle) {
        return Err(InputError::Invariant("generated card issuance precondition".into()).into());
    }
    let destination = if requested == super::command::CardDestination::Hand
        && state.piles.hand.len() >= 10 {
        super::command::CardDestination::Discard
    } else { requested };
    let position=if destination==super::command::CardDestination::ShuffledDraw {
        if state.piles.draw.is_empty(){Some(0)}else{
            let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("generated insertion stream absent".into()))?;
            let (position,next)=super::rng::uniform_below(stream,state.piles.draw.len() as u128)?;state.card_rng=Some(next);Some(position as usize)
        }
    }else{None};
    let owned = OwnedCard::Full(card);
    state.card_ids_issued.insert(handle.clone());
    let empty_draw=state.piles.draw.is_empty();
    match destination {
        super::command::CardDestination::Hand => state.piles.hand.push(owned),
        // Draw reads from the Vec tail (`pop`), so tail insertion is top-of-draw.
        super::command::CardDestination::Draw => state.piles.draw.push(owned),
        super::command::CardDestination::BottomDraw=>state.piles.draw.insert(0,owned),
        super::command::CardDestination::Discard => state.piles.discard.push(owned),
        super::command::CardDestination::ShuffledDraw=>state.piles.draw.insert(position.unwrap(),owned),
    }
    if destination==super::command::CardDestination::ShuffledDraw{
        if empty_draw{state.public.known_draw.clear();state.public.known_draw.push(handle.clone());state.public.known_draw_bottom=vec![handle.clone()];}
        else{state.public.known_draw.truncate(1);state.public.known_draw_bottom.clear();}
    }
    if destination==super::command::CardDestination::Draw{
        state.public.known_draw.insert(0,handle.clone());
        if empty_draw{state.public.known_draw_bottom=vec![handle.clone()];}
    }
    if destination==super::command::CardDestination::BottomDraw{
        state.public.known_draw_bottom.insert(0,handle.clone());
        if empty_draw{state.public.known_draw.insert(0,handle);}
    }
    if destination==super::command::CardDestination::Hand {
        if let Some(amount)=state.player_full_powers.iter()
            .find(|power|power.id.0=="accuracy"&&power.state.amount>0)
            .map(|power|power.state.amount) {
            // Actual generated hand construction calls the copy/draw-discard
            // listener synchronously; this local behavior is not a card draw.
            super::cards::rewrite_accuracy_hand_shivs(state,amount);
        }
    }
    Ok(())
}

pub(super) fn stat_copy_to_hand(state:&mut State,source:&super::card_state::LiveCard,copies:u8)->Result<(),EffectError>{
    for _ in 0..copies{
        let instance=next_generated_instance(state)?;
        let copy=source.stat_equivalent_copy(instance);
        insert_generated(state,copy,super::command::CardDestination::Hand)?;
    }
    Ok(())
}

/// Actual-draw callback owns the just-moved identity before the remaining draw.
pub(super) fn after_draw(state:&mut State,owner:&CardHandle)->Result<(),EffectError>{
    let card=state.piles.hand.iter().find(|c|c.handle()==*owner).and_then(OwnedCard::full).cloned().ok_or_else(||InputError::Invariant("draw callback card absent".into()))?;
    // Void's actual-draw callback queues before installed onCardDraw powers.
    if card.id==crate::content::CardId::Void{
        let effect=Effect{source:Source::Card(owner.clone()),target:Some(Actor::Player),kind:EffectKind::LoseEnergy(1)};
        let before_decision=state.pending.iter().position(|pending|pending.kind==EffectKind::Decision).unwrap_or(state.pending.len());
        state.pending.insert(before_decision,effect);
    }
    // Confusion mutates the real drawn object synchronously during its power
    // callback; the reactive powers below only enqueue their later actions.
    if state.player_full_powers.iter().any(|p|p.id.0=="confused"&&p.state.amount>0)&&!card.local.x_cost&&super::cards::base_cost(state,&card).is_some(){
        let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Confused stream absent".into()))?;
        let (cost,next)=super::rng::uniform_below(stream,4)?;
        let local=&mut state.piles.hand.iter_mut().find(|c|c.handle()==*owner).unwrap().full_mut().unwrap().local;
        local.randomized_cost=Some(cost as i32);local.temporary_cost=0;local.free_play=false;state.card_rng=Some(next);
    }
    super::cards::ironclad::reactive::after_draw(state,&card,owner)
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

/// One character-neutral finish path for a detached, still-owned card.
/// `Natural` honors the live card's intrinsic purge/exhaust flags on card use;
/// end-turn movement passes `honor_card_disposition=false` and supplies only
/// the intrinsic Ethereal property as an override.
pub(super) fn finish_owned_card(state:&mut State,card:OwnedCard,override_disposition:FinishDisposition,
    honor_card_disposition:bool)->Result<(),InputError>{
    let handle=card.handle();
    let card_disposition=card.full().map(|live|(live.local.purge,live.local.exhaust_on_finish));
    let disposition=match override_disposition{
        FinishDisposition::Exhaust=>FinishDisposition::Exhaust,
        FinishDisposition::Purge=>FinishDisposition::Purge,
        FinishDisposition::Natural if honor_card_disposition&&card_disposition.is_some_and(|data|data.0)=>FinishDisposition::Purge,
        FinishDisposition::Natural if honor_card_disposition&&card_disposition.is_some_and(|data|data.1)=>FinishDisposition::Exhaust,
        FinishDisposition::Natural=>FinishDisposition::Natural,
    };
    match disposition{
        FinishDisposition::Exhaust=>exhaust_owned_card(state,card)?,
        FinishDisposition::Purge=>state.inactive_cards.push(card),
        FinishDisposition::Natural=>{
            state.piles.discard.push(card);
            if state.full_runtime{state.pending.push_front(Effect{source:Source::Card(handle),target:None,kind:EffectKind::Hook(Hook::CardDiscarded)});}
        }
    }
    Ok(())
}

/// Commit a selection's discard movement. Only a hand-sourced Dagger Throw or
/// Survivor selection is eligible for the manual Silent reactions; end-turn and
/// natural-finish paths remain on their existing owners.
pub(super) fn discard_selected_card(state: &mut State, card: OwnedCard, manual_hand_discard: bool)
    -> Result<(), InputError> {
    let handle = card.handle();
    let live = card.full().cloned();
    state.piles.discard.push(card);
    if state.full_runtime && manual_hand_discard {
        let listener = if let Some(live) = live.as_ref() {
            super::cards::manual_discard(state, live)?
        } else {
            false
        };
        if listener {
            let notification = Effect { source: Source::Card(handle), target: None,
                kind: EffectKind::Hook(Hook::CardDiscarded) };
            super::engine::validate_effect(&notification, state)?;
            state.pending.push_front(notification);
        }
    }
    Ok(())
}

pub(super) fn discard_card(state: &mut State, card: &CardHandle, end_turn: bool) -> Result<(), InputError> {
    let (moved,disposition,honor_card_disposition) = if end_turn {
        let index = state.piles.hand.iter().position(|c| c.handle() == *card)
            .ok_or_else(|| InputError::Invariant("discard hand owner missing".into()))?;
        let ethereal=state.piles.hand[index].full().is_some_and(|live|live.local.ethereal);
        (state.piles.hand.remove(index),if ethereal{FinishDisposition::Exhaust}else{FinishDisposition::Natural},false)
    } else {
        if state.in_play.as_ref().map(OwnedCard::handle).as_ref() != Some(card) {
            return Err(InputError::Invariant("discard in-play owner missing".into()));
        }
        (state.in_play.take().unwrap(),FinishDisposition::Natural,true)
    };
    finish_owned_card(state,moved,disposition,honor_card_disposition)
}
