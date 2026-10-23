//! Captured Divider base and actor-owned Inferno/Sear gates on the shared queue.
use super::*;
use super::super::{CardDestination,EffectError};
use crate::content::CardId;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum HexaghostEffect { UpgradeBurns,InsertBurns }
const CYCLE:[&str;7]=["sear","tackle","sear","inflame","tackle","sear","inferno"];

pub(super) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    let EnemyMemory::Hexaghost{cycle,..}=memory else{return Err(InputError::Invariant("Hexaghost memory missing".into()).into())};
    let name=match history.last().map(|i|i.as_str()){None=>"activate",Some("activate")=>"divider",_=>*CYCLE.get(*cycle as usize).ok_or_else(||InputError::Invariant("Hexaghost cycle outside bounds".into()))?};
    Ok(MoveSelection{intent:FullIntentId::from_name(name),stream:stream.clone(),memory:memory.clone()})
}

pub(super) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let intent=match enemy.intent_id{ActorIntentId::Full(i)=>i,_=>return Err(InputError::Invariant("Hexaghost intent missing".into()))};
    let upgraded=matches!(enemy.full_memory,Some(EnemyMemory::Hexaghost{sear_upgraded:true,..}));
    let work=match intent.as_str(){
        "activate"=>vec![],"divider"=>(0..6).map(|_|damage(actor.clone(),intent)).collect(),
        "sear"=>vec![damage(actor.clone(),intent),effect(actor,Actor::Player,EffectKind::CreateCard{id:CardId::Burn,destination:CardDestination::Discard,upgrades:u32::from(upgraded)})],
        "tackle"=>(0..2).map(|_|damage(actor.clone(),intent)).collect(),
        "inflame"=>vec![effect(actor.clone(),actor.clone(),EffectKind::GainBlock(12)),effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:2})],
        "inferno"=>{let mut work=(0..6).map(|_|damage(actor.clone(),intent)).collect::<Vec<_>>();work.push(effect(actor.clone(),actor,EffectKind::HexaghostBurn(HexaghostEffect::UpgradeBurns)));work},
        _=>return Err(InputError::Invariant("invalid Hexaghost intent".into())),
    };Ok(work)
}

pub(super) fn take_turn(enemy:&mut EnemyState,player_hp:i32)->Result<Vec<Effect>,InputError>{
    let work=move_work(enemy)?;
    let Some(EnemyMemory::Hexaghost{divider_base,cycle,sear_upgraded})=enemy.full_memory.as_mut()else{return Err(InputError::Invariant("Hexaghost memory missing".into()))};
    match enemy.intent_id.as_str(){
        "activate"=>{if divider_base.is_some(){return Err(InputError::Invariant("Hexaghost cannot recapture Divider".into()))}let base=player_hp/12+1;*divider_base=Some(base);enemy.full_damage_entries.get_mut(&FullIntentId::from_name("divider")).ok_or_else(||InputError::Invariant("Divider damage missing".into()))?.base=base;},
        "divider"=>{},
        name=>{if name=="inferno"{*sear_upgraded=true;}*cycle=(*cycle+1)%7;},
    }Ok(work)
}

pub(in crate::combat) fn apply(state:&mut State,work:&Effect,gate:HexaghostEffect)->Result<(),EffectError>{
    match gate {
        HexaghostEffect::UpgradeBurns=>{
            for owned in state.piles.draw.iter_mut().chain(&mut state.piles.discard){if let Some(card)=owned.full_mut(){if card.id==CardId::Burn&&card.upgrades==0{card.upgrades=1;}}}
            state.pending.push_front(Effect{source:work.source.clone(),target:work.target.clone(),kind:EffectKind::HexaghostBurn(HexaghostEffect::InsertBurns)});
        },
        HexaghostEffect::InsertBurns=>{for _ in 0..3{state.pending.push_front(Effect{source:work.source.clone(),target:Some(Actor::Player),kind:EffectKind::CreateCard{id:CardId::Burn,destination:CardDestination::Discard,upgrades:1}});}},
    }Ok(())
}
