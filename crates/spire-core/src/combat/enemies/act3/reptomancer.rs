//! Reptomancer/Daggers share native actor streams, spawn custody and lifecycle.
use super::*;

pub(in crate::combat::enemies) fn supports(id:EnemyId)->bool{matches!(id,EnemyId::Reptomancer|EnemyId::Dagger)}
/// Array slots are not action order: x=-250,-220,parent,180,210.
pub(in crate::combat) fn physical_slot(tracked:u8)->u8{[4,1,3,0][tracked as usize]}
pub(in crate::combat) fn tracked_slot(physical:u8)->Option<u8>{[4,1,3,0].iter().position(|slot|*slot==physical).map(|n|n as u8)}
pub(in crate::combat::enemies) fn select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,context:Option<&act2::EnemyRollContext>)->Result<MoveSelection,SelectError>{
    if !memory.valid_for(id){return Err(InputError::Invariant("Reptomancer/Dagger memory mismatch".into()).into())}
    if let EnemyMemory::Dagger{first}=memory{return Ok(MoveSelection{intent:FullIntentId::from_name(if *first{"dagger_wound"}else{"dagger_explode"}),memory:EnemyMemory::Dagger{first:false},stream:stream.clone()})}
    let EnemyMemory::Reptomancer{first,..}=memory else{return Err(InputError::Invariant("Reptomancer memory absent".into()).into())};
    let mut next_memory=memory.clone();if let EnemyMemory::Reptomancer{first,..}=&mut next_memory{*first=false;}
    if *first{return Ok(MoveSelection{intent:FullIntentId::from_name("repto_spawn"),memory:next_memory,stream:stream.clone()})}
    let context=context.ok_or_else(||InputError::Unsupported("Reptomancer selection requires live formation".into()))?;
    let others=context.actors.iter().filter(|a|a.owner!=context.owner&&a.hp>0).count();let last=history.last().map(|m|m.as_str());let twice=history.len()>=2&&history[history.len()-2..].iter().all(|m|m.as_str()=="repto_spawn");
    let (mut ticket,mut cursor)=rng::uniform_below(stream,100)?;
    let name=loop{match ticket{
        0..=32 if last==Some("repto_snake")=>{let (n,next)=rng::uniform_below(&cursor,67)?;cursor=next;ticket=33+n;},
        0..=32=>break "repto_snake",
        33..=65=>break if twice||others>3{"repto_snake"}else{"repto_spawn"},
        _ if last==Some("repto_bite")=>{let (n,next)=rng::uniform_below(&cursor,66)?;cursor=next;ticket=n;},
        _=>break "repto_bite",
    }};
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:next_memory,stream:cursor})
}
pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());
    Ok(match (&enemy.enemy_id,enemy.intent_id.as_str()){
        (ActorEnemyId::Full(EnemyId::Dagger),"dagger_wound")=>vec![damage(actor.clone(),FullIntentId::from_name("dagger_wound")),effect(actor,Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Wound,destination:super::super::super::CardDestination::Discard,upgrades:0})],
        (ActorEnemyId::Full(EnemyId::Dagger),"dagger_explode")=>vec![damage(actor.clone(),FullIntentId::from_name("dagger_explode")),effect(actor.clone(),actor,EffectKind::Damage(super::super::super::DamagePayload::HpLoss(enemy.hp)))],
        (ActorEnemyId::Full(EnemyId::Reptomancer),"repto_snake")=>vec![damage(actor.clone(),FullIntentId::from_name("repto_snake")),damage(actor.clone(),FullIntentId::from_name("repto_snake")),effect(actor,Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:1})],
        (ActorEnemyId::Full(EnemyId::Reptomancer),"repto_bite")=>vec![damage(actor,FullIntentId::from_name("repto_bite"))],
        _=>return Err(InputError::Unsupported("Reptomancer/Dagger intent or summon unavailable".into())),
    })
}
pub(in crate::combat) fn take_turn(state:&mut State,owner:&EnemyHandle)->Result<Vec<Effect>,super::super::super::EffectError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Reptomancer turn owner absent".into()))?;
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::Reptomancer)&&enemy.intent_id.as_str()=="repto_spawn"{
        let Some(EnemyMemory::Reptomancer{slots,..})=&enemy.full_memory else{return Err(InputError::Invariant("Reptomancer slot owner absent".into()).into())};
        let free=(0..4).find(|slot|slots.get(slot).is_none_or(|handle|!state.enemies.iter().any(|e|&e.instance_id==handle&&e.hp>0&&e.life==super::super::super::ActorLife::Active)));
        return Ok(free.map_or_else(Vec::new,|slot|vec![Effect{source:Source::Actor(Actor::Enemy(owner.clone())),target:None,kind:EffectKind::SpawnEnemy(super::super::super::SpawnRequest{summoner:owner.clone(),species:EnemyId::Dagger,slot:physical_slot(slot)})}]))
    }
    Ok(move_work(enemy)?)
}

pub(in crate::combat) fn validate_slots(state:&State)->Result<(),InputError>{
    for parent in &state.enemies{if let Some(EnemyMemory::Reptomancer{slots,..})=&parent.full_memory{for (slot,handle) in slots{
        if *slot>=4{return Err(InputError::Invariant("Reptomancer tracked position out of bounds".into()))}
        let physical=physical_slot(*slot);
        let live_or_historical=state.enemies.iter().any(|e|&e.instance_id==handle&&e.enemy_id==ActorEnemyId::Full(EnemyId::Dagger)&&state.actor_slots.get(handle)==Some(&physical));
        let prepared=state.spawn_progress.as_ref().is_some_and(|p|p.request.summoner==parent.instance_id&&p.request.slot==physical&&p.prepared_child.as_ref().is_some_and(|e|&e.instance_id==handle&&e.enemy_id==ActorEnemyId::Full(EnemyId::Dagger)));
        if !live_or_historical&&!prepared{return Err(InputError::Invariant("Reptomancer tracked child custody mismatch".into()))}
    }}}Ok(())
}

pub(in crate::combat) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult){
    let Actor::Enemy(owner)=&result.target else{return};
    if result.hp_lost<=0||result.hp_after!=0||!state.enemies.iter().any(|e|&e.instance_id==owner&&e.enemy_id==ActorEnemyId::Full(EnemyId::Reptomancer)){return}
    let targets=act2::spawn::ordered_actors(state).into_iter().map(|e|e.instance_id.clone()).collect::<Vec<_>>();
    state.pending.retain(|e|!((matches!(e.kind,EffectKind::EnemyMove|EffectKind::RollMove)&&matches!(&e.target,Some(Actor::Enemy(id)) if id==owner||targets.contains(id)))
        ||(matches!(e.kind,EffectKind::Damage(super::super::super::DamagePayload::FullEnemyMove(_)))&&matches!(&e.source,Source::Actor(Actor::Enemy(id)) if id==owner||targets.contains(id)))));
    // Repeated addToTop in group order makes the rightmost living actor die first.
    for target in targets{state.pending.push_front(effect(Actor::Enemy(owner.clone()),Actor::Enemy(target),EffectKind::ReptomancerSuicide));}
}

pub(in crate::combat) fn cleanup_suicide(state:&mut State,owner:&EnemyHandle){
    let Some(enemy)=state.enemies.iter_mut().find(|e|&e.instance_id==owner&&e.hp>0&&e.life==super::super::super::ActorLife::Active)else{return};enemy.hp=0;
    if state.player.hp>0&&act2::combat_victory_ready(state){
        // Finish already accepted gameplay before terminal cleanup; never erase
        // prepared child insertion or earned player healing.
        act2::bottom(state,Effect{source:Source::None,target:None,kind:EffectKind::Hook(super::super::super::Hook::CombatSettled)});
        act2::bottom(state,Effect{source:Source::None,target:None,kind:EffectKind::Terminal(super::super::super::Outcome::Victory)});
    }
    state.pending.push_front(Effect{source:Source::None,target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::Hook(super::super::super::Hook::Death)});
}
