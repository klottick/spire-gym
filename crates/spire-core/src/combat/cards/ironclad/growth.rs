//! Finite mutable growth mechanics for the Ironclad growth cards.
use super::super::{EffectError,InputError,Effect,EffectKind,Actor,FullPowerId,DamagePayload};
use super::super::model::{OwnedCard,State};
use super::state::IroncladEffect;

fn grow(owned:&mut OwnedCard,family:u64,amount:i32)->Result<(),EffectError>{
    if let Some(card)=owned.full_mut().filter(|card|card.copy_family==family){
        let base=card.local.base_damage.unwrap_or(8);
        card.local.base_damage=Some(base.checked_add(amount).ok_or_else(||InputError::Unsupported("Rampage damage growth overflow".into()))?.max(0));
    }
    Ok(())
}

fn grow_family(state:&mut State,family:u64,amount:i32)->Result<(),EffectError>{
    // The source modifies every same-UUID battle instance as one queued action.
    // Preflight the complete owned traversal so a retained overflow fault does
    // not leave earlier owners partially incremented.
    for owned in state.piles.hand.iter().chain(&state.piles.draw).chain(&state.piles.discard).chain(&state.piles.exhaust)
        .chain(state.in_play.iter()).chain(state.play_frames.iter().filter_map(|frame|frame.parent.as_ref()))
        .chain(state.queued_autoplay.values()) {
        if let Some(card)=owned.full().filter(|card|card.copy_family==family){
            card.local.base_damage.unwrap_or(8).checked_add(amount).map(|value|value.max(0))
                .ok_or_else(||InputError::Unsupported("Rampage damage growth overflow".into()))?;
        }
    }
    for card in state.piles.hand.iter_mut().chain(&mut state.piles.draw).chain(&mut state.piles.discard).chain(&mut state.piles.exhaust){grow(card,family,amount)?;}
    if let Some(card)=state.in_play.as_mut(){grow(card,family,amount)?;}
    for frame in &mut state.play_frames{
        if let Some(card)=frame.parent.as_mut(){grow(card,family,amount)?;}
    }
    // Double Tap's queued child is already an accepted/started repeat owner,
    // unlike an unrelated autoplay card waiting in the draw pile.
    for child in state.queued_autoplay.values_mut(){grow(child,family,amount)?;}
    Ok(())
}

pub(in crate::combat) fn apply(state:&mut State,effect:&Effect,work:&IroncladEffect)->Result<(),EffectError>{
    match work {
        IroncladEffect::LimitBreakDispatch=>{
            let strength=super::player_strength_power(state).map_err(|error|InputError::Invariant(format!("Limit Break dispatch Strength unavailable: {error:?}")))?;
            if let Some(strength)=strength{state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:strength}});}
        }
        IroncladEffect::DropkickDispatch{damage}=>{
            let Some(Actor::Enemy(target))=effect.target.as_ref() else{return Err(InputError::Invariant("Dropkick dispatch target absent".into()).into())};
            let enemy=state.enemies.iter().find(|enemy|&enemy.instance_id==target)
                .ok_or_else(||InputError::Invariant("Dropkick captured target disappeared".into()))?;
            let vulnerable=enemy.life==super::super::ActorLife::Active&&enemy.hp>0
                &&(enemy.full_powers.iter().any(|p|p.id.0=="vulnerable"&&p.state.amount>0)
                    ||enemy.powers.iter().any(|p|p.id==super::super::PowerId::Vulnerable&&p.state.amount>0));
            let mut work=vec![Effect{source:effect.source.clone(),target:Some(Actor::Enemy(target.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(*damage))}];
            if vulnerable {work.push(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::GainEnergy(1)});work.push(Effect{source:effect.source.clone(),target:None,kind:EffectKind::Draw(1)});}
            for next in work.into_iter().rev(){state.pending.push_front(next);}
        }
        IroncladEffect::RampageGrow{family,amount}=>grow_family(state,*family,*amount)?,
        _=>return Err(InputError::Invariant("growth dispatcher received unrelated Ironclad work".into()).into()),
    }
    Ok(())
}
