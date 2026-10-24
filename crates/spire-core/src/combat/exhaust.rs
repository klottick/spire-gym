//! Character-neutral exhaust notification routing.
use super::{Actor,Effect,EffectError,EffectKind,InputError,Source};
use super::model::{ExhaustBatch,ExhaustBatchFilter,ExhaustBatchFinish,OwnedCard,State};

/// Resolve the finite exhaust listeners for the already-moved card owner.
/// Any future item/relic listener belongs before powers; card-content triggers
/// remain last. No callback registry or character gate is involved.
pub(super) fn dispatch(state:&State,source:&Source)->Result<(Vec<Effect>,Vec<Effect>),EffectError>{
    if !state.full_runtime{return Ok((Vec::new(),Vec::new()))}
    let Source::Card(owner)=source else{return Err(InputError::Invariant("CardExhausted requires card source".into()).into())};
    let card=state.piles.exhaust.iter().find(|owned|owned.handle()==*owner).and_then(OwnedCard::full)
        .ok_or_else(||InputError::Invariant("exhaust callback source is not owned by exhaust pile".into()))?;

    // Relic listeners may own top-queued children; power and card callback
    // children retain their established placement behind the batch continuation.
    let enemies=state.enemies.iter().filter(|enemy|enemy.life==super::ActorLife::Active&&enemy.hp>0).map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
    let top=state.item_context.as_ref().map_or_else(Vec::new,|context|crate::items::relic_exhaust_effects(context,source,&enemies));
    let mut work=Vec::new();
    work.extend(super::powers::on_card_exhausted(state,source));
    work.extend(super::cards::on_card_exhausted(state,source,card)?);
    Ok((top,work))
}

/// Begin a content-requested batch against the hand as it exists at resolution
/// time. Captured handles keep subsequent callback draws outside this batch.
pub(super) fn begin_batch(state:&mut State,effect:&Effect,filter:ExhaustBatchFilter,random:bool,finish:ExhaustBatchFinish)->Result<(),EffectError>{
    let Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("exhaust batch requires a card owner".into()).into())};
    if state.in_play.as_ref().map(OwnedCard::handle).as_ref()!=Some(owner){return Err(InputError::Invariant("exhaust batch parent is not in play".into()).into())}
    let catalog=crate::content::Catalog::standard();
    let mut candidates=Vec::new();
    for owned in &state.piles.hand{
        let Some(card)=owned.full() else{continue};
        let include=match filter{
            ExhaustBatchFilter::AllHand=>true,
            ExhaustBatchFilter::NonAttack=>catalog.card(card.id)
                .ok_or_else(||InputError::Unsupported(format!("exhaust batch card type unavailable: {}",card.id.as_str())))?
                .card_type!=crate::content::CardType::Attack,
        };
        if include{candidates.push(owned.handle())}
    }
    // Both source actions collect in hand order and repeatedly add each
    // ExhaustSpecificCardAction to the stack-like front; execution is reverse.
    if !random { candidates.reverse(); }
    let remaining=u8::try_from(candidates.len()).map_err(|_|InputError::Invariant("exhaust candidate count exceeds hand bound".into()))?;
    let candidates=if random{Vec::new()}else{candidates};
    advance_batch(state,effect.source.clone(),ExhaustBatch{owner:owner.clone(),filter,candidates,random,remaining,committed:0,finish})
}

/// Resolve a single captured owner at a time through the shared exhaust
/// transfer path. Its callbacks are queued ahead of this continuation.
pub(super) fn continue_batch(state:&mut State,effect:&Effect,batch:&ExhaustBatch)->Result<(),EffectError>{
    advance_batch(state,effect.source.clone(),batch.clone())
}

fn advance_batch(state:&mut State,source:Source,mut batch:ExhaustBatch)->Result<(),EffectError>{
    if !state.full_runtime||!matches!(&source,Source::Card(owner) if owner==&batch.owner)
        ||state.in_play.as_ref().map(OwnedCard::handle).as_ref()!=Some(&batch.owner){
        return Err(InputError::Invariant("exhaust batch continuation lost its parent".into()).into())
    }
    // Persist the exact pre-selection frontier before touching the fallible
    // scoped RNG. On success this staged copy is replaced by the next cursor;
    // on failure it remains in diagnostic state beside the retained RNG cursor.
    state.pending.push_front(Effect{source:source.clone(),target:None,kind:EffectKind::ExhaustBatchContinue(batch.clone())});
    let available=if batch.random {
        let catalog=crate::content::Catalog::standard();
        state.piles.hand.iter().enumerate().filter_map(|(position,owned)|{
            let card=owned.full()?;
            let eligible=match batch.filter{ExhaustBatchFilter::AllHand=>true,ExhaustBatchFilter::NonAttack=>
                catalog.card(card.id).is_some_and(|definition|definition.card_type!=crate::content::CardType::Attack)};
            eligible.then(|| (position,owned.handle()))
        }).collect::<Vec<_>>()
    }else{
        batch.candidates.iter().filter_map(|handle|
            state.piles.hand.iter().position(|owned|owned.handle()==*handle).map(|position|(position,handle.clone()))).collect::<Vec<_>>()
    };
    if batch.remaining>0&&!available.is_empty(){
        let chosen=if batch.random&&available.len()>1{
            let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("exhaust batch card RNG absent".into()))?;
            let (index,next)=super::rng::uniform_below(stream,available.len() as u128)?;
            state.card_rng=Some(next);index as usize
        }else{0};
        let _=state.pending.pop_front();
        let (hand_index,handle)=available[chosen].clone();
        if !batch.random {batch.candidates.retain(|candidate|candidate!=&handle);}
        let next_committed=batch.committed.checked_add(1).ok_or_else(||InputError::Unsupported("exhaust batch committed counter exhausted".into()))?;
        let owned=state.piles.hand.remove(hand_index);
        if owned.handle()!=handle{return Err(InputError::Invariant("exhaust batch detached a different hand owner".into()).into())}
        batch.committed=next_committed;
        batch.remaining=batch.remaining.checked_sub(1).ok_or_else(||InputError::Invariant("exhaust batch remaining underflow".into()))?;
        state.pending.push_front(Effect{source:source.clone(),target:None,kind:EffectKind::ExhaustBatchContinue(batch)});
        super::piles::exhaust_owned_card(state,owned)?;
        return Ok(())
    }
    let _=state.pending.pop_front();
    let count=usize::try_from(batch.committed).map_err(|_|InputError::Unsupported("exhaust batch count exceeds platform".into()))?;
    let mut after=Vec::new();
    match batch.finish{
        ExhaustBatchFinish::BlockPerExhaust(block)=>after.extend((0..count).map(|_|Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::GainBlock(block)})),
        ExhaustBatchFinish::DamagePerExhaust{target,damage}=>after.extend((0..count).map(|_|Effect{source:source.clone(),target:Some(target.clone()),kind:EffectKind::Damage(super::DamagePayload::Fixed(damage))})),
        ExhaustBatchFinish::DamageAfterBatch{target,damage}=>after.push(Effect{source:source.clone(),target:Some(target),kind:EffectKind::Damage(super::DamagePayload::Fixed(damage))}),
    }
    for next in after.into_iter().rev(){state.pending.push_front(next)}
    Ok(())
}
