//! Finite Shapes/Orb Walker rules; primary static qualification is in the M8 facts.
use super::*;
use super::super::super::{ActorLife,ActorIntentId};

pub(in crate::combat::enemies) fn select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    if !memory.valid_for(id){return Err(InputError::Invariant("Act3 memory/species mismatch".into()).into())}
    let mut cursor=stream.clone();
    let mut ticket=|bound|->Result<u64,SelectError>{let (n,next)=rng::uniform_below(&cursor,bound)?;cursor=next;Ok(n)};
    let last=history.last().map(|m|m.as_str());
    let name=match memory {
        EnemyMemory::OrbWalker=>{
            let claw=ticket(100)?<40;
            let repeated=history.len()>=2&&history[history.len()-2..].iter().all(|m|m.as_str()==if claw{"orb_claw"}else{"orb_laser"});
            if claw!=repeated{"orb_claw"}else{"orb_laser"}
        },
        EnemyMemory::Repulsor=>if ticket(100)?<20&&last!=Some("repulsor_attack"){"repulsor_attack"}else{"repulsor_daze"},
        EnemyMemory::Exploder{turns}=>if *turns<2{"exploder_attack"}else{"exploder_explode"},
        EnemyMemory::Spiker{buffs}=>if *buffs>=6||ticket(100)?<50&&last!=Some("spiker_attack"){"spiker_attack"}else{"spiker_buff"},
        _=>return Err(InputError::Invariant("Act3 simple memory unavailable".into()).into()),
    };
    Ok(MoveSelection{intent:FullIntentId::from_name(name),stream:cursor,memory:memory.clone()})
}

pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());
    let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("Act3 full intent absent".into()))};
    let work=match intent.as_str(){
        "orb_claw"|"repulsor_attack"|"exploder_attack"|"spiker_attack"=>vec![damage(actor,intent)],
        "orb_laser"=>vec![damage(actor.clone(),intent),effect(actor.clone(),Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Burn,destination:super::super::super::CardDestination::ShuffledDraw,upgrades:0}),effect(actor,Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Burn,destination:super::super::super::CardDestination::Discard,upgrades:0})],
        "repulsor_daze"=>(0..2).map(|_|effect(actor.clone(),Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Dazed,destination:super::super::super::CardDestination::ShuffledDraw,upgrades:0})).collect(),
        "spiker_buff"=>vec![effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("thorns"),amount:2})],
        "exploder_explode"=>vec![effect(actor.clone(),actor.clone(),EffectKind::ShapeSuicide),effect(actor.clone(),Actor::Player,EffectKind::Damage(DamagePayload::Thorns(30))),effect(actor.clone(),actor,EffectKind::ShapeDamageComplete)],
        _=>return Err(InputError::Unsupported("Act3 simple intent unavailable".into())),
    };Ok(work)
}

pub(in crate::combat::enemies) fn take_turn(enemy:&mut EnemyState)->Result<Vec<Effect>,InputError>{
    let mut work=move_work(enemy)?;
    match &mut enemy.full_memory {
        Some(EnemyMemory::Exploder{turns})=>{
            *turns=turns.checked_add(1).filter(|n|*n<=3).ok_or_else(||InputError::Invariant("Exploder turn counter exhausted".into()))?;
            if *turns<3{let actor=Actor::Enemy(enemy.instance_id.clone());work.push(effect(actor.clone(),actor,EffectKind::ReduceFullPower{id:FullPowerId("explosive"),amount:1}));}
        },
        Some(EnemyMemory::Spiker{buffs}) if enemy.intent_id.as_str()=="spiker_buff"=>{*buffs=buffs.checked_add(1).filter(|n|*n<=6).ok_or_else(||InputError::Invariant("Spiker buff counter exhausted".into()))?;},
        _=>{},
    }Ok(work)
}

pub(in crate::combat) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult){
    let Actor::Enemy(owner)=&result.target else{return};
    if !result.normal||matches!(&result.source,Source::None)||result.source==Source::Actor(Actor::Enemy(owner.clone())){return}
    let Some(enemy)=state.enemies.iter().find(|e|&e.instance_id==owner&&e.life==ActorLife::Active&&e.enemy_id==super::super::super::ActorEnemyId::Full(EnemyId::Spiker))else{return};
    let amount=enemy.full_powers.iter().find(|p|p.id.0=="thorns").map_or(0,|p|p.state.amount);if amount<=0{return}
    let actor=Actor::Enemy(owner.clone());
    state.pending.push_front(effect(actor.clone(),actor.clone(),EffectKind::ShapeDamageComplete));
    state.pending.push_front(effect(actor,Actor::Player,EffectKind::Damage(DamagePayload::Thorns(amount))));
}

pub(in crate::combat) fn complete_damage(state:&mut State,source:&Source){
    if state.player.hp>0&&act2::combat_victory_ready(state){act2::queue_victory(state,source);}
}
