//! A held live card is a zone owner, not a newly issued card or hidden second pile.
use super::*;
use super::super::super::{EffectError,LiveCard};

pub(in crate::combat) fn validate_custody(state:&State)->Result<(),InputError>{
    for (owner,card) in &state.stasis {
        if !state.full_runtime||card.full().is_none()||!state.enemies.iter().any(|e|&e.instance_id==owner&&e.enemy_id==ActorEnemyId::Full(EnemyId::BronzeOrb)){return Err(InputError::Invariant("Stasis actor/card custody mismatch".into()))}
    }
    for enemy in state.enemies.iter().filter(|e|e.enemy_id==ActorEnemyId::Full(EnemyId::BronzeOrb)&&e.hp>0){
        if enemy.full_powers.iter().any(|p|p.id.0=="stasis")!=state.stasis.contains_key(&enemy.instance_id){return Err(InputError::Invariant("live Stasis power/card mismatch".into()))}
    }Ok(())
}

pub(in crate::combat) fn capture(state:&mut State,owner:&EnemyHandle)->Result<(),EffectError>{
    let pile=if state.piles.draw.is_empty(){&state.piles.discard}else{&state.piles.draw};if pile.is_empty(){return Ok(())}
    let mut candidates=Vec::new();for (index,owned) in pile.iter().enumerate(){
        let card=owned.full().ok_or_else(||InputError::Invariant("legacy card in Stasis pile".into()))?;
        let row=crate::content::Catalog::standard().card(card.id).ok_or_else(||InputError::Unsupported("Stasis card rarity unavailable".into()))?;
        let rank=match row.rarity{crate::content::CardRarity::Rare=>3,crate::content::CardRarity::Uncommon=>2,crate::content::CardRarity::Common=>1,_=>0};candidates.push((index,rank,card.id.as_str(),card.instance));
    }
    let rank=candidates.iter().map(|c|c.1).max().unwrap();candidates.retain(|c|c.1==rank);candidates.sort_by_key(|c|(c.2,c.3));
    let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Stasis card stream absent".into()))?;let (index,next)=rng::uniform_below(stream,candidates.len() as u128)?;
    let index=candidates[index as usize].0;let draw=!state.piles.draw.is_empty();state.card_rng=Some(next);
    let card=if draw{state.piles.draw.remove(index)}else{state.piles.discard.remove(index)};state.stasis.insert(owner.clone(),card);
    state.pending.push_front(effect(Actor::Enemy(owner.clone()),Actor::Enemy(owner.clone()),EffectKind::ApplyFullPower{id:FullPowerId("stasis"),amount:1}));Ok(())
}

fn returned(card:&mut LiveCard){
    // Same-instance stat-equivalent copy keeps explicit costs/free-play/identity;
    // exhaust is supplied by the known card constructor/upgrade, not cloned blindly.
    card.local.exhaust_on_finish=super::super::super::card_state::generated(card.id,card.instance,card.upgrades)
        .unwrap_or_else(||super::super::super::card_state::permanent(card.id,card.instance,card.upgrades)).local.exhaust_on_finish;
}

pub(in crate::combat) fn release(state:&mut State,owner:&EnemyHandle)->Result<(),InputError>{
    if !state.stasis.get(owner).is_some_and(|card|card.full().is_some()){return Err(InputError::Invariant("Stasis return is not full card".into()))}
    let mut card=state.stasis.remove(owner).ok_or_else(||InputError::Invariant("Stasis release owner absent".into()))?;
    returned(card.full_mut().unwrap());
    if state.piles.hand.len()<10{state.piles.hand.push(card)}else{state.piles.discard.push(card)}Ok(())
}

fn barrier(state:&State)->usize{state.pending.iter().position(|e|matches!(e.kind,EffectKind::Hook(super::super::super::Hook::CombatSettled)|EffectKind::Terminal(_)|EffectKind::EndRound|EffectKind::StartNextPlayer|EffectKind::Decision)).unwrap_or(state.pending.len())}
pub(in crate::combat) fn on_death(state:&mut State,target:Option<&Actor>){
    if let Some(Actor::Enemy(owner))=target {if state.stasis.contains_key(owner){let work=effect(Actor::Enemy(owner.clone()),Actor::Enemy(owner.clone()),EffectKind::ReleaseStasis{orb:owner.clone()});let index=barrier(state);state.pending.insert(index,work);}}
}
pub(super) fn victory_after_returns(state:&mut State,source:&Source){
    let index=barrier(state);state.pending.insert(index,Effect{source:source.clone(),target:None,kind:EffectKind::Hook(super::super::super::Hook::CombatSettled)});
    state.pending.insert(index+1,Effect{source:source.clone(),target:None,kind:EffectKind::Terminal(super::super::super::Outcome::Victory)});
}
