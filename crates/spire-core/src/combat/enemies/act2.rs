//! Finite Act2 producers on the existing shared resolver.
use super::*;
pub(in crate::combat) mod flight;
pub(in crate::combat) mod chosen_snecko;
mod partners;
pub(in crate::combat) mod spawn;
pub(in crate::combat) mod gremlin_leader;
pub(in crate::combat) mod slavers;
pub(in crate::combat) mod book;
pub(in crate::combat) mod champ;
pub(in crate::combat) mod collector;
pub(in crate::combat) mod automaton;
pub(in crate::combat) mod stasis;

/// Source group readiness follows actual death/replacement/escape work. A Minion
/// tag alone does not fabricate a death or remove a living actor from the fight.
pub(in crate::combat) fn combat_victory_ready(state:&State)->bool{
    state.enemies.iter().all(|e|e.life!=super::super::ActorLife::Active||e.hp==0)
}
pub(in crate::combat) fn queue_victory(state:&mut State,source:&Source){
    if !state.stasis.is_empty(){stasis::victory_after_returns(state,source);return}
    state.pending.push_front(Effect{source:source.clone(),target:None,kind:EffectKind::Terminal(super::super::Outcome::Victory)});
    state.pending.push_front(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(super::super::Hook::CombatSettled)});
}
pub(in crate::combat) fn scripted_cleanup(state:&mut State,owner:&EnemyHandle,kind:EffectKind){
    let targets=spawn::ordered_actors(state).into_iter().map(|e|e.instance_id.clone()).collect::<Vec<_>>();
    state.pending.retain(|e|!((matches!(e.kind,EffectKind::EnemyMove|EffectKind::RollMove)&&matches!(&e.target,Some(Actor::Enemy(id)) if targets.contains(id)))
        ||(matches!(e.kind,EffectKind::Damage(DamagePayload::FullEnemyMove(_)))&&matches!(&e.source,Source::Actor(Actor::Enemy(id)) if targets.contains(id)))));
    for target in targets{bottom(state,effect(Actor::Enemy(owner.clone()),Actor::Enemy(target),kind.clone()));}
}
pub(in crate::combat) fn suicide(state:&mut State,target:&EnemyHandle){
    let Some(enemy)=state.enemies.iter_mut().find(|e|&e.instance_id==target&&e.life==super::super::ActorLife::Active&&e.hp>0)else{return};enemy.hp=0;
    if combat_victory_ready(state)&&state.player.hp>0{queue_victory(state,&Source::None);}
    state.pending.push_front(Effect{source:Source::None,target:Some(Actor::Enemy(target.clone())),kind:EffectKind::Hook(super::super::Hook::Death)});
}

#[derive(Clone,Debug)]
pub(crate) struct ActorSummary {pub owner:EnemyHandle,pub species:EnemyId,pub hp:i32,pub max_hp:i32,pub slot:u8}
#[derive(Clone,Debug)]
pub(crate) struct EnemyRollContext {pub owner:EnemyHandle,pub turn:u32,pub actors:Vec<ActorSummary>}

pub(crate) fn select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,context:&EnemyRollContext)->Result<MoveSelection,super::SelectError>{
    if stream.id().domain()!=rng::Domain::RunCombatEnemyAi||context.turn==0||!memory.valid_for(id)||!context.actors.iter().any(|a|a.owner==context.owner&&a.species==id){return Err(InputError::Invariant("live enemy selection context mismatch".into()).into())}
    let mut slots=std::collections::BTreeSet::new();
    for actor in context.actors.iter().filter(|a|a.hp>0){if actor.hp>actor.max_hp||!slots.insert(actor.slot){return Err(InputError::Invariant("live selection formation mismatch".into()).into())}}
    if id==EnemyId::Collector{collector::select_move(history,memory,stream,context)}else if id==EnemyId::Champ{champ::select_move(history,memory,stream,context)}else if id==EnemyId::GremlinLeader{gremlin_leader::select_leader(history,memory,stream,context)}else if matches!(id,EnemyId::Centurion|EnemyId::Mystic){partners::select_move(id,history,memory,stream,context)}else{super::select_move(id,history,memory,stream)}
}

pub(in crate::combat) fn roll_move(state:&mut State,owner:&EnemyHandle)->Result<(),super::super::EffectError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("live roll owner absent".into()))?;
    let ActorEnemyId::Full(id)=enemy.enemy_id else{return Err(InputError::Invariant("live roll owner not full".into()).into())};
    let history=enemy.move_history.iter().map(|id|match id{ActorIntentId::Full(id)=>Ok(*id),_=>Err(InputError::Invariant("mixed full intent history".into()))}).collect::<Result<Vec<_>,_>>()?;
    let actors=state.enemies.iter().filter_map(|e|match e.enemy_id{
        ActorEnemyId::Full(species) if e.life==super::super::ActorLife::Active=>Some((e,species)),_=>None,
    }).map(|(e,species)|Ok(ActorSummary{owner:e.instance_id.clone(),species,hp:e.hp,max_hp:e.max_hp,slot:*state.actor_slots.get(&e.instance_id).ok_or_else(||InputError::Invariant("logical actor slot absent".into()))?})).collect::<Result<Vec<_>,InputError>>()?;
    let context=EnemyRollContext{owner:owner.clone(),turn:state.turn,actors};
    let memory=enemy.full_memory.as_ref().ok_or_else(||InputError::Invariant("live roll memory absent".into()))?;
    let stream=enemy.full_ai_rng.as_ref().ok_or_else(||InputError::Invariant("live roll stream absent".into()))?;
    let selected=select_move(id,&history,memory,stream,&context).map_err(|e|match e{super::SelectError::Invalid(e)=>super::super::EffectError::Invalid(e),super::SelectError::Random(e)=>super::super::EffectError::Random(e)})?;
    let actor=state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap();
    actor.intent_id=ActorIntentId::Full(selected.intent);actor.selected_move=ActorIntentId::Full(selected.intent);actor.move_history.push(ActorIntentId::Full(selected.intent));actor.full_memory=Some(selected.memory);actor.full_ai_rng=Some(selected.stream);
    super::super::effects::refresh_enemy_values(state)?;Ok(())
}

pub(in crate::combat) fn take_turn(state:&mut State,owner:&EnemyHandle)->Result<Vec<Effect>,super::super::EffectError>{
    if let Some(enemy)=state.enemies.iter_mut().find(|e|&e.instance_id==owner){
        match enemy.enemy_id {
            ActorEnemyId::Full(EnemyId::SlaverBlue|EnemyId::Taskmaster|EnemyId::SlaverRed)=>return Ok(slavers::take_turn(enemy)?),
            ActorEnemyId::Full(EnemyId::BookOfStabbing)=>return Ok(book::move_work(enemy)?),
            ActorEnemyId::Full(EnemyId::Champ)=>return Ok(champ::move_work(enemy)?),_=>{},
        }
    }
    if state.enemies.iter().any(|e|&e.instance_id==owner&&matches!(e.enemy_id,ActorEnemyId::Full(EnemyId::Collector|EnemyId::TorchHead))){return collector::take_turn(state,owner)}
    if state.enemies.iter().any(|e|&e.instance_id==owner&&matches!(e.enemy_id,ActorEnemyId::Full(EnemyId::Centurion|EnemyId::Mystic))){Ok(partners::move_work(state,owner)?)}else{gremlin_leader::take_turn(state,owner)}
}

/// Source-bottom work precedes only native scheduler sentinels, never gameplay payloads.
pub(in crate::combat) fn bottom(state:&mut State,work:Effect){
    let index=state.pending.iter().position(|e|matches!(e.kind,EffectKind::Decision|EffectKind::EndRound|EffectKind::StartNextPlayer)).unwrap_or(state.pending.len());
    state.pending.insert(index,work);
}

pub(in crate::combat) fn modify_incoming_damage(state:&State,target:&Actor,payload:&DamagePayload,amount:i32)->Result<i32,InputError>{
    let normal=matches!(payload,DamagePayload::Fixed(_)|DamagePayload::EnemyMove(_)|DamagePayload::FullEnemyMove(_));
    let flight=match target{Actor::Enemy(owner)=>state.enemies.iter().any(|e|&e.instance_id==owner&&e.full_powers.iter().any(|p|p.id.0=="flight"&&p.state.amount>0)),_=>false};
    Ok(if normal&&flight{amount/2}else{amount})
}

pub(in crate::combat) fn after_damage(state:&mut State,result:&super::super::model::DamageResult)->Result<(),super::super::EffectError>{
    flight::after_damage(state,result);gremlin_leader::after_damage(state,result);book::after_damage(state,result);collector::after_damage(state,result);automaton::after_damage(state,result);Ok(())
}
