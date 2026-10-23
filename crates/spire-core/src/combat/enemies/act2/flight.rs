//! Spheric Guardian's persistent shield and Byrd's queued Flight reactions.
use super::*;

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<super::super::MoveSelection,super::super::SelectError>{
    let mut cursor=stream.clone();
    let name=match memory {
        EnemyMemory::SphericGuardian=>match history.last().map(|m|m.as_str()){None=>"sphere_activate",Some("sphere_activate")=>"sphere_debuff",Some("sphere_debuff"|"sphere_harden")=>"sphere_slam",Some("sphere_slam")=>"sphere_harden",_=>return Err(InputError::Invariant("invalid Spheric Guardian history".into()).into())},
        EnemyMemory::Byrd{flying}=>{
            if history.is_empty(){let (ticket,next)=rng::uniform_below(&cursor,8)?;cursor=next;if ticket<3{"byrd_caw"}else{"byrd_peck"}}
            else if !flying {"byrd_headbutt"}
            else {
                let (bucket,next)=rng::uniform_below(&cursor,100)?;cursor=next;
                let last=history.last().unwrap().as_str();
                let numerator=match bucket {0..=49 if history.iter().rev().take(2).filter(|m|m.as_str()=="byrd_peck").count()==2=>Some(6_710_887),50..=69 if last=="byrd_swoop"=>Some(6_291_456),70..=99 if last=="byrd_caw"=>Some(4_793_251),_=>None};
                let coin=if let Some(n)=numerator{let (ticket,next)=rng::uniform_below(&cursor,16_777_216)?;cursor=next;Some(ticket<n)}else{None};
                match (bucket,coin){(0..=49,Some(true))=>"byrd_swoop",(0..=49,Some(false))=>"byrd_caw",(0..=49,None)=>"byrd_peck",(50..=69,Some(true))=>"byrd_caw",(50..=69,Some(false))=>"byrd_peck",(50..=69,None)=>"byrd_swoop",(_,Some(true))=>"byrd_swoop",(_,Some(false))=>"byrd_peck",_=>"byrd_caw"}
            }
        },
        _=>return Err(InputError::Invariant("ordinary Act2 memory mismatch".into()).into()),
    };
    Ok(super::super::MoveSelection{intent:FullIntentId::from_name(name),memory:memory.clone(),stream:cursor})
}

pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let intent=match enemy.intent_id{ActorIntentId::Full(id)=>id,_=>return Err(InputError::Invariant("Act2 actor requires full intent".into()))};
    let work=match intent.as_str(){
        "sphere_activate"=>vec![effect(actor.clone(),actor,EffectKind::GainBlock(25))],
        "sphere_debuff"=>vec![damage(actor.clone(),intent),effect(actor,Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("frail"),amount:5})],
        "sphere_slam"=>vec![damage(actor.clone(),intent),damage(actor,intent)],
        "sphere_harden"=>vec![damage(actor.clone(),intent),effect(actor.clone(),actor,EffectKind::GainBlock(15))],
        "byrd_peck"=>(0..5).map(|_|damage(actor.clone(),intent)).collect(),
        "byrd_swoop"|"byrd_headbutt"=>vec![damage(actor,intent)],
        "byrd_caw"=>vec![effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:1})],
        "byrd_stunned"=>vec![],
        "byrd_go_airborne"=>vec![effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("flight"),amount:3})],
        _=>return Err(InputError::Invariant("invalid Act2 ordinary intent".into())),
    };Ok(work)
}

pub(in crate::combat::enemies) fn take_turn(enemy:&mut EnemyState)->Result<Vec<Effect>,InputError>{
    let work=move_work(enemy)?;
    match enemy.intent_id.as_str(){
        "byrd_headbutt"=>{publish(enemy,"byrd_go_airborne");},
        "byrd_go_airborne"=>{if let Some(EnemyMemory::Byrd{flying})=enemy.full_memory.as_mut(){*flying=true;}},
        _=>{},
    }Ok(work)
}

fn publish(enemy:&mut EnemyState,name:&'static str){let intent=ActorIntentId::Full(FullIntentId::from_name(name));enemy.intent_id=intent.clone();enemy.selected_move=intent.clone();enemy.move_history.push(intent);enemy.intent_damage=None;}

pub(super) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult){
    let Actor::Enemy(owner)=&result.target else{return};
    if !result.normal||result.source==Source::None||result.hp_lost<=0||result.hp_after<=0{return}
    if state.enemies.iter().any(|e|&e.instance_id==owner&&e.full_powers.iter().any(|p|p.id.0=="flight"&&p.state.amount>0)){
        let actor=Actor::Enemy(owner.clone());bottom(state,effect(actor.clone(),actor,EffectKind::ReduceFullPower{id:FullPowerId("flight"),amount:1}));
    }
}

pub(in crate::combat) fn on_removed(state:&mut State,target:&Actor){
    if let Actor::Enemy(owner)=target {if state.enemies.iter().any(|e|&e.instance_id==owner&&e.enemy_id==ActorEnemyId::Full(EnemyId::Byrd)&&e.hp>0){bottom(state,effect(target.clone(),target.clone(),EffectKind::GroundByrd));}}
}

pub(in crate::combat) fn ground(state:&mut State,work:&Effect)->Result<(),InputError>{
    let Some(Actor::Enemy(owner))=&work.target else{return Err(InputError::Invariant("Byrd ground owner absent".into()))};
    let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Byrd ground actor absent".into()))?;
    if enemy.hp<=0{return Ok(())}
    if let Some(EnemyMemory::Byrd{flying})=enemy.full_memory.as_mut(){*flying=false;}else{return Err(InputError::Invariant("Byrd memory absent".into()))}
    publish(enemy,"byrd_stunned");super::super::super::effects::refresh_enemy_values(state)
}
