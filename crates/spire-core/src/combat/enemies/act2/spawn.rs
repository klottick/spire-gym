//! Append-only summon custody. A fresh child never replaces or inherits its parent.
use super::*;
use super::super::super::{ActorLife,DamageEntry,SpawnRequest};
use std::collections::BTreeMap;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct SpawnProgress {
    pub request:SpawnRequest,pub ordinal:u64,pub prepared_child:Option<EnemyState>,pub rolled_hp:Option<i32>,
    pub constructor_rng:StreamState,pub latest_child_rng:StreamState,
}

pub(in crate::combat) fn initial_slots(actors:&[EnemyState])->BTreeMap<EnemyHandle,u8>{
    if actors.iter().any(|a|a.enemy_id==ActorEnemyId::Full(EnemyId::BronzeAutomaton)){
        let mut next=0;return actors.iter().map(|a|{let slot=if a.enemy_id==ActorEnemyId::Full(EnemyId::BronzeAutomaton){1}else{let slot=if next==0{0}else{next+1};next+=1;slot};(a.instance_id.clone(),slot)}).collect()
    }
    if actors.iter().any(|a|a.enemy_id==ActorEnemyId::Full(EnemyId::Collector)){
        let mut next=0;return actors.iter().map(|a|{let slot=if a.enemy_id==ActorEnemyId::Full(EnemyId::Collector){2}else{let slot=if next<2{next}else{next+1};next+=1;slot};(a.instance_id.clone(),slot)}).collect()
    }
    let leader=actors.iter().any(|a|a.enemy_id==ActorEnemyId::Full(EnemyId::GremlinLeader));
    let mut small=0;
    actors.iter().enumerate().map(|(i,a)|{let slot=if leader{if a.enemy_id==ActorEnemyId::Full(EnemyId::GremlinLeader){3}else{let slot=[1,2,0].get(small).copied().unwrap_or(9);small+=1;slot}}else{i as u8};(a.instance_id.clone(),slot)}).collect()
}

pub(in crate::combat) fn ordered_actors(state:&State)->Vec<&EnemyState>{
    let mut actors=state.enemies.iter().filter(|a|a.life==ActorLife::Active&&a.hp>0).collect::<Vec<_>>();
    actors.sort_by_key(|a|state.actor_slots.get(&a.instance_id).copied());actors
}

pub(in crate::combat) fn validate_request(state:&State,request:&SpawnRequest)->Result<(),InputError>{
    let policy=state.enemies.iter().any(|e|e.instance_id==request.summoner&&e.life==ActorLife::Active&&e.hp>0&&match e.enemy_id{
        ActorEnemyId::Full(EnemyId::GremlinLeader)=>request.slot<3&&gremlin_leader::is_small(request.species),
        ActorEnemyId::Full(EnemyId::Collector)=>request.slot<2&&request.species==EnemyId::TorchHead,
        ActorEnemyId::Full(EnemyId::BronzeAutomaton)=>matches!(request.slot,0|2)&&request.species==EnemyId::BronzeOrb,_=>false,
    });
    if !state.full_runtime||!policy{return Err(InputError::Invariant("unqualified summon producer/request".into()))}
    if ordered_actors(state).iter().any(|e|state.actor_slots.get(&e.instance_id)==Some(&request.slot)){
        return Err(InputError::Invariant("summon logical slot occupied".into()))
    }Ok(())
}

fn child_stream(parent:&StreamState,child:&EnemyHandle,slot:u8,role:&str)->Result<StreamState,RngError>{
    if parent.id().domain()!=rng::Domain::RunCombatEnemyAi||parent.id().scope().len()!=2{return Err(RngError::Invalid("summoner stream domain/scope".into()))}
    let message=format!("spire-lab/rng/v1/summon-child\0{}\0{}\0{}\0{}\0{}",parent.id().scope()[0],parent.id().scope()[1],child.as_str(),slot,role);
    let key=rng::hmac_sha256(parent.key(),message.as_bytes());
    let actor=if role=="ai"{child.as_str().to_owned()}else{format!("{}/constructor",child.as_str())};
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec![parent.id().scope()[0].clone(),actor])?;
    StreamState::new(id,key,0)
}

pub(in crate::combat) fn apply(state:&mut State,request:&SpawnRequest)->Result<(),super::super::super::EffectError>{
    validate_request(state,request)?;
    if state.spawn_progress.is_some(){return Err(InputError::Invariant("overlapping summon custody".into()).into())}
    let ordinal=state.spawn_ordinals.get(&request.summoner).copied().unwrap_or(0);
    let next=ordinal.checked_add(1).ok_or_else(||InputError::Unsupported("summon ordinal exhausted".into()))?;
    let handle=EnemyHandle::parse(&format!("{}/spawn/{ordinal}",request.summoner.as_str()))?;
    if state.enemies.iter().any(|e|e.instance_id==handle){return Err(InputError::Invariant("summon identity collision".into()).into())}
    let parent=state.enemies.iter().find(|e|e.instance_id==request.summoner).and_then(|e|e.full_ai_rng.as_ref()).ok_or_else(||InputError::Invariant("summoner AI owner absent".into()))?;
    let constructor_rng=child_stream(parent,&handle,request.slot,"constructor")?;
    let latest_child_rng=child_stream(parent,&handle,request.slot,"ai")?;
    state.spawn_progress=Some(SpawnProgress{request:request.clone(),ordinal,prepared_child:None,rolled_hp:None,constructor_rng,latest_child_rng});
    state.spawn_ordinals.insert(request.summoner.clone(),next);
    state.pending.push_front(effect(Actor::Enemy(request.summoner.clone()),Actor::Enemy(request.summoner.clone()),EffectKind::CompleteSpawn));Ok(())
}

pub(in crate::combat) fn complete(state:&mut State)->Result<(),super::super::super::EffectError>{
    let progress=state.spawn_progress.as_ref().ok_or_else(||InputError::Invariant("summon continuation absent".into()))?;
    validate_request(state,&progress.request)?;
    let (lo,hi)=super::super::hp_range(progress.request.species,0);
    let (ticket,next)=rng::uniform_below(&progress.constructor_rng,(hi-lo+1) as u128)?;
    let hp=lo+ticket as i32;
    let progress=state.spawn_progress.as_mut().unwrap();progress.constructor_rng=next;progress.rolled_hp=Some(hp);
    let memory=EnemyMemory::for_enemy(progress.request.species)?;
    let selected=super::super::select_move(progress.request.species,&[],&memory,&progress.latest_child_rng).map_err(|e|match e{super::super::SelectError::Invalid(e)=>super::super::super::EffectError::Invalid(e),super::super::SelectError::Random(e)=>super::super::super::EffectError::Random(e)})?;
    progress.latest_child_rng=selected.stream.clone();
    let full_damage_entries=super::super::damage_bases(progress.request.species)?.into_iter().map(|(id,base)|(id,DamageEntry{base,output:base})).collect::<BTreeMap<_,_>>();
    let intent_damage=full_damage_entries.get(&selected.intent).map(|d|d.output);
    // Both explicit producer policies above request Minion; neither species nor
    // the generic act of summoning implies this role for an unlisted producer.
    let mut full_powers=vec![power("minion",1)];if progress.request.species==EnemyId::GremlinMad{full_powers.push(power("angry",1));}
    let child=EnemyState{instance_id:EnemyHandle::parse(&format!("{}/spawn/{}",progress.request.summoner.as_str(),progress.ordinal))?,enemy_id:ActorEnemyId::Full(progress.request.species),hp,max_hp:hp,block:0,powers:vec![],full_powers,
        intent_id:ActorIntentId::Full(selected.intent),selected_move:ActorIntentId::Full(selected.intent),move_history:vec![ActorIntentId::Full(selected.intent)],damage_entries:Default::default(),full_damage_entries,
        full_ai_rng:Some(selected.stream),full_memory:Some(selected.memory),intent_damage,life:ActorLife::Active};
    progress.prepared_child=Some(child.clone());
    let slot=progress.request.slot;
    let summoner=progress.request.summoner.clone();
    if state.enemies.len().checked_add(1).and_then(|n|u64::try_from(n).ok()).is_none()||ordered_actors(state).len()>=9{return Err(InputError::Unsupported("summon actor storage exhausted".into()).into())}
    let mut candidate=state.clone();candidate.actor_slots.insert(child.instance_id.clone(),slot);
    if let Some(EnemyMemory::Collector{slots,..})=candidate.enemies.iter_mut().find(|e|e.instance_id==summoner).and_then(|e|e.full_memory.as_mut()){slots.insert(slot,child.instance_id.clone());}
    candidate.enemies.push(child);super::super::super::effects::refresh_enemy_values(&mut candidate)?;
    state.enemies=candidate.enemies;state.actor_slots=candidate.actor_slots;state.spawn_progress=None;Ok(())
}

fn power(id:&'static str,amount:i32)->super::super::super::FullPowerEntry{super::super::super::FullPowerEntry{id:FullPowerId(id),state:super::super::super::PowerState{amount,skip_first:false,monster_source_grace:false}}}
