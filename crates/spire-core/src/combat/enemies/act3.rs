//! Act3 producers use existing actor memory and the shared effect queue.
use super::*;
pub(in crate::combat) mod shapes;
pub(in crate::combat) mod darklings;
pub(in crate::combat) mod ordinary;
pub(in crate::combat) mod writhing;
pub(in crate::combat) mod elites;
pub(in crate::combat) mod reptomancer;
pub(in crate::combat) mod awakened;
pub(in crate::combat) mod time_eater;
pub(in crate::combat) mod partners;

pub(in crate::combat) fn supports(id:EnemyId)->bool{matches!(id,EnemyId::OrbWalker|EnemyId::Repulsor|EnemyId::Exploder|EnemyId::Spiker|EnemyId::Darkling|EnemyId::WrithingMass|EnemyId::AwakenedOne|EnemyId::TimeEater)||ordinary::supports(id)||elites::supports(id)||reptomancer::supports(id)||partners::supports(id)}
pub(crate) fn select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,context:&act2::EnemyRollContext)->Result<MoveSelection,SelectError>{
    if partners::supports(id){return partners::select_move(id,history,memory,stream)}
    if id==EnemyId::TimeEater{return time_eater::select_move(history,memory,stream,context)}
    if id==EnemyId::AwakenedOne{return awakened::select_move(history,memory,stream)}
    if reptomancer::supports(id){return reptomancer::select_move(id,history,memory,stream,Some(context))}
    if elites::supports(id){elites::select_move(history,memory,stream)}else if id==EnemyId::WrithingMass{writhing::select_move(history,memory,stream)}else if id==EnemyId::Darkling{darklings::select_move(history,memory,stream,context.actors.iter().any(|a|a.owner==context.owner&&a.hp==0))}else if ordinary::supports(id){ordinary::select_move(id,history,memory,stream,None)}else{shapes::select_move(id,history,memory,stream)}
}
pub(in crate::combat) fn take_turn(state:&mut State,owner:&EnemyHandle)->Result<Vec<Effect>,super::super::EffectError>{
    if matches!(state.enemies.iter().find(|e|&e.instance_id==owner).map(|e|&e.enemy_id),Some(ActorEnemyId::Full(id)) if partners::supports(*id)){return Ok(partners::move_work(state,owner)?)}
    if matches!(state.enemies.iter().find(|e|&e.instance_id==owner).map(|e|&e.enemy_id),Some(ActorEnemyId::Full(id)) if reptomancer::supports(*id)){return reptomancer::take_turn(state,owner)}
    let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Act3 turn owner absent".into()))?;
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::TimeEater){return Ok(time_eater::move_work(enemy)?)}
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::AwakenedOne){return Ok(awakened::move_work(enemy)?)}
    Ok(if matches!(enemy.enemy_id,ActorEnemyId::Full(id) if elites::supports(id)){elites::move_work(enemy)?}else if enemy.enemy_id==ActorEnemyId::Full(EnemyId::WrithingMass){writhing::take_turn(enemy)?}else if enemy.enemy_id==ActorEnemyId::Full(EnemyId::Darkling){darklings::move_work(enemy)?}else if matches!(enemy.enemy_id,ActorEnemyId::Full(id) if ordinary::supports(id)){ordinary::take_turn(enemy)?}else{shapes::take_turn(enemy)?})
}
