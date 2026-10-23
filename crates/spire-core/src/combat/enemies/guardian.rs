//! Guardian's finite queued phase changes. There is no second damage path.
use super::*;
use super::super::{ActorLife,EffectError};
use crate::content::CardType;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum GuardianEffect { Defensive,Offensive,Reset,LoseBlock }

/// Queue at the gameplay bottom, ahead of scheduler-only decision/round sentinels.
fn bottom(state:&mut State,effect:Effect){
    let i=state.pending.iter().position(|e|matches!(e.kind,EffectKind::Decision|EffectKind::EndRound|EffectKind::StartNextPlayer)).unwrap_or(state.pending.len());
    state.pending.insert(i,effect);
}
fn change(owner:&EnemyHandle,kind:GuardianEffect)->Effect{let actor=Actor::Enemy(owner.clone());effect(actor.clone(),actor,EffectKind::GuardianChange(kind))}
fn publish(enemy:&mut EnemyState,name:&'static str){
    let intent=ActorIntentId::Full(FullIntentId::from_name(name));enemy.intent_id=intent.clone();enemy.selected_move=intent.clone();enemy.move_history.push(intent);enemy.intent_damage=None;
}

pub(super) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    let intent=match history.last().map(|m|m.as_str()){
        None|Some("whirlwind")=>"charging_up",Some("charging_up")=>"fierce_bash",Some("fierce_bash")=>"vent_steam",Some("vent_steam")|Some("twin_slam")=>"whirlwind",Some("defensive_mode")=>"roll_attack",Some("roll_attack")=>"twin_slam",
        _=>return Err(InputError::Invariant("invalid Guardian move history".into()).into()),
    };Ok(MoveSelection{intent:FullIntentId::from_name(intent),memory:memory.clone(),stream:stream.clone()})
}

pub(super) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let intent=match enemy.intent_id{ActorIntentId::Full(i)=>i,_=>return Err(InputError::Invariant("Guardian full intent missing".into()))};
    let work=match intent.as_str(){
        "charging_up"=>vec![effect(actor.clone(),actor,EffectKind::GainBlock(9))],
        "fierce_bash"|"roll_attack"=>vec![damage(actor,intent)],
        "whirlwind"=>(0..4).map(|_|damage(actor.clone(),intent)).collect(),
        "vent_steam"=>vec![effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}),effect(actor,Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:2})],
        "defensive_mode"=>vec![effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("sharp_hide"),amount:3})],
        "twin_slam"=>vec![change(&enemy.instance_id,GuardianEffect::Offensive),damage(actor.clone(),intent),damage(actor.clone(),intent),effect(actor.clone(),actor,EffectKind::RemoveFullPower(FullPowerId("sharp_hide")))],
        _=>return Err(InputError::Invariant("invalid Guardian intent".into())),
    };Ok(work)
}

pub(super) fn on_hit(state:&mut State,target:&EnemyHandle,hp_loss:i32){
    let Some(enemy)=state.enemies.iter_mut().find(|e|&e.instance_id==target)else{return};
    if enemy.life!=ActorLife::Active||enemy.hp<=0||hp_loss<=0{return}
    let Some(EnemyMemory::Guardian{quota,loss,open:true,latched})=enemy.full_memory.as_mut()else{return};
    if *latched{return}
    let accumulated=i64::from(*loss)+i64::from(hp_loss);
    if accumulated>=i64::from(*quota){*loss=0;*latched=true;bottom(state,change(target,GuardianEffect::Defensive));}
    else{*loss=accumulated as i32;if let Some(p)=enemy.full_powers.iter_mut().find(|p|p.id.0=="mode_shift"){p.state.amount=*quota-*loss;}}
}

pub(in crate::combat) fn apply(state:&mut State,work:&Effect,kind:GuardianEffect)->Result<(),EffectError>{
    let Some(Actor::Enemy(owner))=work.target.as_ref()else{return Err(InputError::Invariant("Guardian change owner missing".into()).into())};
    let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Guardian owner absent".into()))?;
    if enemy.life!=ActorLife::Active||enemy.hp<=0{return Ok(())}
    let Some(EnemyMemory::Guardian{quota,loss,open,latched})=enemy.full_memory.as_mut()else{return Err(InputError::Invariant("Guardian memory absent".into()).into())};
    let actor=Actor::Enemy(owner.clone());
    match kind {
        GuardianEffect::Defensive=>{
            if !*open||!*latched{return Err(InputError::Invariant("Guardian transition without open latch".into()).into())}
            *quota=quota.checked_add(10).ok_or_else(||InputError::Unsupported("Guardian quota overflow".into()))?;*open=false;*loss=0;
            publish(enemy,"defensive_mode");
            bottom(state,effect(actor.clone(),actor.clone(),EffectKind::RemoveFullPower(FullPowerId("mode_shift"))));bottom(state,effect(actor.clone(),actor,EffectKind::GainBlock(20)));
        },
        GuardianEffect::Offensive=>bottom(state,change(owner,GuardianEffect::Reset)),
        GuardianEffect::Reset=>{*loss=0;*open=true;*latched=false;let amount=*quota;
            bottom(state,effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("mode_shift"),amount}));bottom(state,change(owner,GuardianEffect::LoseBlock));},
        GuardianEffect::LoseBlock=>enemy.block=0,
    }Ok(())
}

pub(in crate::combat) fn card_played(state:&mut State,source:&Source)->Result<(),InputError>{
    let Source::Card(owner)=source else{return Ok(())};let Some(card)=state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full())else{return Ok(())};
    if !crate::content::Catalog::standard().card(card.id).is_some_and(|c|c.card_type==CardType::Attack){return Ok(())}
    let work=state.enemies.iter().filter(|e|e.life==ActorLife::Active&&e.hp>0).filter_map(|e|{
        let power=e.full_powers.iter().find(|p|p.id.0=="sharp_hide")?;
        Some(effect(Actor::Enemy(e.instance_id.clone()),Actor::Player,EffectKind::Damage(DamagePayload::Thorns(power.state.amount))))
    }).collect::<Vec<_>>();
    for work in work{bottom(state,work)}Ok(())
}
