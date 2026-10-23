//! Independent Deca/Donu turns use the existing actor queue and live group.
use super::*;

pub(in crate::combat::enemies) fn supports(id:EnemyId)->bool{matches!(id,EnemyId::Deca|EnemyId::Donu)}

pub(in crate::combat::enemies) fn select_move(id:EnemyId,_history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    if !memory.valid_for(id)||!supports(id){return Err(InputError::Invariant("Act3 partner memory/species mismatch".into()).into())}
    let name=match memory{
        EnemyMemory::Deca{attacking:true}=>"deca_beam",EnemyMemory::Deca{attacking:false}=>"deca_square",
        EnemyMemory::Donu{attacking:true}=>"donu_beam",EnemyMemory::Donu{attacking:false}=>"donu_circle",_=>unreachable!(),
    };
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:memory.clone(),stream:stream.clone()})
}

pub(in crate::combat::enemies) fn move_work(state:&mut State,owner:&EnemyHandle)->Result<Vec<Effect>,InputError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Act3 pair owner absent".into()))?;
    let ActorEnemyId::Full(id)=enemy.enemy_id else{return Err(InputError::Invariant("Act3 pair full owner absent".into()))};
    let name=enemy.intent_id.as_str();if !valid_intent(id,name){return Err(InputError::Unsupported("Act3 pair intent/species mismatch".into()))}
    let actor=Actor::Enemy(owner.clone());
    let work=match name{
        "deca_beam"|"donu_beam"=>{
            let mut work=vec![damage(actor.clone(),FullIntentId::from_name(name));2];
            if name=="deca_beam"{for _ in 0..2{work.push(effect(actor.clone(),Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Dazed,destination:super::super::super::CardDestination::Discard,upgrades:0}));}}work
        },
        "deca_square"|"donu_circle"=>state.enemies.iter().map(|e|effect(actor.clone(),Actor::Enemy(e.instance_id.clone()),if name=="deca_square"{EffectKind::GainBlock(16)}else{EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3}})).collect(),
        _=>unreachable!(),
    };
    let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap();match enemy.full_memory.as_mut(){
        Some(EnemyMemory::Deca{attacking})=>*attacking=name!="deca_beam",
        Some(EnemyMemory::Donu{attacking})=>*attacking=name!="donu_beam",
        _=>return Err(InputError::Invariant("Act3 pair cycle memory absent".into())),
    }Ok(work)
}

pub(in crate::combat) fn valid_intent(id:EnemyId,name:&str)->bool{matches!((id,name),(EnemyId::Deca,"deca_beam"|"deca_square")|(EnemyId::Donu,"donu_beam"|"donu_circle"))}
