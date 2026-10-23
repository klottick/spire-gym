//! Centurion/Mystic decisions consume live roster facts, not stored partner copies.
use super::*;

pub(super) fn select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,context:&EnemyRollContext)->Result<MoveSelection,super::super::SelectError>{
    let pair=match id {
        EnemyId::Centurion=>{let partner=context.actors.iter().any(|a|a.species==EnemyId::Mystic&&a.hp>0);("centurion_slash",if partner{"centurion_protect"}else{"centurion_fury"},65)},
        EnemyId::Mystic=>{
            let hurt=context.actors.iter().any(|a|a.hp>0&&a.max_hp-a.hp>=16);
            if hurt&&!history.ends_with(&[FullIntentId::from_name("mystic_heal");2]){return Ok(MoveSelection{intent:FullIntentId::from_name("mystic_heal"),memory:memory.clone(),stream:stream.clone()})}
            ("mystic_attack","mystic_buff",60)
        },
        _=>return Err(InputError::Invariant("partner selector species mismatch".into()).into()),
    };
    let (ticket,next)=rng::uniform_below(stream,100)?;
    let primary=FullIntentId::from_name(pair.0);let other=FullIntentId::from_name(pair.1);
    let mut intent=if ticket<pair.2{primary}else{other};
    if history.ends_with(&[intent;2]){intent=if intent==primary{other}else{primary};}
    Ok(MoveSelection{intent,memory:memory.clone(),stream:next})
}

pub(super) fn move_work(state:&State,owner:&EnemyHandle)->Result<Vec<Effect>,InputError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("partner actor absent".into()))?;
    let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("partner full intent absent".into()))};
    let actor=Actor::Enemy(owner.clone());
    Ok(match intent.as_str(){
        "centurion_slash"|"mystic_attack"=>{let mut work=vec![damage(actor.clone(),intent)];if intent.as_str()=="mystic_attack"{work.push(effect(actor,Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("frail"),amount:2}));}work},
        "centurion_protect"=>{
            let target=state.enemies.iter().find(|e|e.enemy_id==ActorEnemyId::Full(EnemyId::Mystic)&&e.life==super::super::super::ActorLife::Active&&e.hp>0).map_or(actor.clone(),|e|Actor::Enemy(e.instance_id.clone()));
            vec![effect(actor,target,EffectKind::GainBlock(15))]
        },
        "centurion_fury"=>(0..3).map(|_|damage(actor.clone(),intent)).collect(),
        "mystic_heal"|"mystic_buff"=>state.enemies.iter().filter(|e|e.life==super::super::super::ActorLife::Active&&e.hp>0).map(|e|effect(actor.clone(),Actor::Enemy(e.instance_id.clone()),if intent.as_str()=="mystic_heal"{EffectKind::Heal(16)}else{EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:2}})).collect(),
        _=>return Err(InputError::Invariant("partner intent mismatch".into())),
    })
}
