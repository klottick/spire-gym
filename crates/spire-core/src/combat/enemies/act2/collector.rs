//! Collector's owned slots and actual queued, callback-delivering suicide cleanup.
use super::*;
use super::super::super::{ActorLife,EffectError,SpawnRequest};

pub(in crate::combat) fn validate_slots(state:&State)->Result<(),InputError>{
    for actor in &state.enemies {
        if let Some(EnemyMemory::Collector{slots,..})=&actor.full_memory {
            for (slot,child) in slots {
                if !state.enemies.iter().any(|e|&e.instance_id==child&&e.enemy_id==ActorEnemyId::Full(EnemyId::TorchHead))||state.actor_slots.get(child)!=Some(slot){return Err(InputError::Invariant("Collector slot custody/identity mismatch".into()))}
            }
        }
    }Ok(())
}

pub(super) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,context:&EnemyRollContext)->Result<MoveSelection,super::super::SelectError>{
    let EnemyMemory::Collector{turns_taken,ult_used,initial_spawn,slots}=memory else{return Err(InputError::Invariant("Collector memory absent".into()).into())};
    let mut cursor=stream.clone();let name=if *initial_spawn{"collector_spawn"}else if *turns_taken>=3&&!*ult_used{"collector_ultimate"}else{
        let (ticket,next)=rng::uniform_below(&cursor,100)?;cursor=next;let last=history.last().map(|m|m.as_str());
        let missing=slots.values().any(|owner|!context.actors.iter().any(|a|&a.owner==owner&&a.hp>0));
        if ticket<=25&&missing&&last!=Some("collector_revive"){"collector_revive"}
        else if ticket<=70&&!history.ends_with(&[FullIntentId::from_name("collector_rake");2]){"collector_rake"}
        else if last!=Some("collector_buff"){"collector_buff"}else{"collector_rake"}
    };Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:memory.clone(),stream:cursor})
}

pub(super) fn take_turn(state:&mut State,owner:&EnemyHandle)->Result<Vec<Effect>,EffectError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Collector turn owner absent".into()))?;
    let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("Collector full intent absent".into()).into())};
    let actor=Actor::Enemy(owner.clone());
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::TorchHead){
        if intent.as_str()!="torch_attack"{return Err(InputError::Invariant("TorchHead intent mismatch".into()).into())}
        state.pending.retain(|e|!(e.kind==EffectKind::RollMove&&e.target==Some(actor.clone())));
        return Ok(vec![damage(actor.clone(),intent),effect(actor.clone(),actor,EffectKind::SetEnemyMove{intent})])
    }
    let Some(EnemyMemory::Collector{turns_taken,slots,..})=enemy.full_memory.as_ref()else{return Err(InputError::Invariant("Collector state absent".into()).into())};
    let next=turns_taken.checked_add(1).filter(|n|*n<=i32::MAX as u32).ok_or_else(||InputError::Unsupported("Collector turn counter exhausted".into()))?;
    let request=|slot|Effect{source:Source::Actor(actor.clone()),target:None,kind:EffectKind::SpawnEnemy(SpawnRequest{summoner:owner.clone(),slot,species:EnemyId::TorchHead})};
    let work=match intent.as_str(){
        "collector_spawn"=>vec![request(0),request(1)],
        "collector_revive"=>slots.iter().filter(|(_,child)|!state.enemies.iter().any(|e|&e.instance_id==*child&&e.life==ActorLife::Active&&e.hp>0)).map(|(slot,_)|request(*slot)).collect(),
        "collector_rake"=>vec![damage(actor.clone(),intent)],
        "collector_buff"=>{let mut work=vec![effect(actor.clone(),actor.clone(),EffectKind::GainBlock(15))];for enemy in spawn::ordered_actors(state){work.push(effect(actor.clone(),Actor::Enemy(enemy.instance_id.clone()),EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3}));}work},
        "collector_ultimate"=>["weak","vulnerable","frail"].into_iter().map(|id|effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId(id),amount:3})).collect(),
        _=>return Err(InputError::Invariant("Collector intent mismatch".into()).into()),
    };
    let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap();let Some(EnemyMemory::Collector{turns_taken,initial_spawn,ult_used,..})=enemy.full_memory.as_mut()else{unreachable!()};
    *turns_taken=next;if intent.as_str()=="collector_spawn"{*initial_spawn=false;}if intent.as_str()=="collector_ultimate"{*ult_used=true;}Ok(work)
}

pub(super) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult){
    let Actor::Enemy(owner)=&result.target else{return};
    if result.hp_after!=0||result.hp_lost<=0||!state.enemies.iter().any(|e|&e.instance_id==owner&&e.enemy_id==ActorEnemyId::Full(EnemyId::Collector)){return}
    scripted_cleanup(state,owner,EffectKind::CollectorSuicide);
}
