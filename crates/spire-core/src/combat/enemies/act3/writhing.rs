//! Writhing reactions and permanent obtain requests share the native resolver.
use super::*;
use super::super::super::{ActorLife,Hook};

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    let EnemyMemory::WrithingMass{first,parasite_used,..}=memory else{return Err(InputError::Invariant("Writhing memory absent".into()).into())};
    let mut cursor=stream.clone();let (mut n,next)=rng::uniform_below(&cursor,100)?;cursor=next;let last=history.last().map(|m|m.as_str());
    let name=if *first{match n{0..=32=>"writhing_multi",33..=65=>"writhing_block",_=>"writhing_debuff"}}else{loop{
        match n{
            0..=9 if last!=Some("writhing_multi")=>break "writhing_multi",
            0..=9=>{let (v,next)=rng::uniform_below(&cursor,90)?;cursor=next;n=10+v;},
            10..=19 if !parasite_used&&last!=Some("writhing_parasite")=>break "writhing_parasite",
            10..=19=>{
                // Original nextFloat has 24 equally spaced values; strict <0.1f
                // accepts 1677722 values, not a rounded one-in-ten coin.
                let (coin,next)=rng::uniform_below(&cursor,1<<24)?;cursor=next;if coin<1_677_722{break "writhing_multi"}
                let (v,next)=rng::uniform_below(&cursor,80)?;cursor=next;n=20+v;
            },
            20..=39 if last!=Some("writhing_debuff")=>break "writhing_debuff",
            20..=39=>{
                let (coin,next)=rng::uniform_below(&cursor,1<<24)?;cursor=next;
                let (v,next)=if coin<6_710_887{rng::uniform_below(&cursor,20)?}else{let (v,next)=rng::uniform_below(&cursor,60)?;(40+v,next)};cursor=next;n=v;
            },
            40..=69 if last!=Some("writhing_block")=>break "writhing_block",
            40..=69=>{let (v,next)=rng::uniform_below(&cursor,70)?;cursor=next;n=v;},
            _ if last!=Some("writhing_big")=>break "writhing_big",
            _=>{let (v,next)=rng::uniform_below(&cursor,40)?;cursor=next;n=v;},
        }
    }};
    let mut memory=memory.clone();if let EnemyMemory::WrithingMass{first,..}=&mut memory{*first=false;}
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory,stream:cursor})
}

pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let intent=FullIntentId::from_name(enemy.intent_id.as_str());
    let mut work=match intent.as_str(){
        "writhing_multi"=>vec![damage(actor.clone(),intent);3],
        "writhing_big"=>vec![damage(actor.clone(),intent)],
        "writhing_block"=>vec![damage(actor.clone(),intent),effect(actor.clone(),actor.clone(),EffectKind::GainBlock(enemy.full_damage_entries.get(&intent).ok_or_else(||InputError::Invariant("Writhing Block base absent".into()))?.base))],
        "writhing_debuff"=>vec![damage(actor.clone(),intent),effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}),effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:2})],
        "writhing_parasite"=>vec![effect(actor.clone(),Actor::Player,EffectKind::ObtainParasite)],
        _=>return Err(InputError::Unsupported("Writhing intent unavailable".into())),
    };
    work.push(effect(actor.clone(),actor,EffectKind::ResetMalleable));Ok(work)
}

pub(in crate::combat::enemies) fn take_turn(enemy:&mut EnemyState)->Result<Vec<Effect>,InputError>{
    let work=move_work(enemy)?;
    if enemy.intent_id.as_str()=="writhing_parasite"{
        let Some(EnemyMemory::WrithingMass{parasite_used,parasite_pending,..})=&mut enemy.full_memory else{return Err(InputError::Invariant("Parasite execution owner absent".into()))};
        if *parasite_used{return Err(InputError::Invariant("Parasite already used".into()))}*parasite_used=true;*parasite_pending=true;
    }Ok(work)
}

pub(in crate::combat) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult)->Result<(),InputError>{
    let Actor::Enemy(owner)=&result.target else{return Ok(())};
    if result.hp_lost<=0||result.hp_after<=0||matches!(result.source,Source::None){return Ok(())}
    let Some(e)=state.enemies.iter_mut().find(|e|&e.instance_id==owner&&e.life==ActorLife::Active&&e.enemy_id==ActorEnemyId::Full(EnemyId::WrithingMass))else{return Ok(())};
    let actor=Actor::Enemy(owner.clone());let reactive=!result.hp_loss&&!result.thorns&&e.full_powers.iter().any(|p|p.id.0=="reactive");
    let block=if result.normal{if let Some(p)=e.full_powers.iter_mut().find(|p|p.id.0=="malleable"){
        let before=p.state.amount;p.state.amount=before.checked_add(1).filter(|n|*n<=999).ok_or_else(||InputError::Unsupported("Malleable amount exceeds native bounds".into()))?;Some(before)
    }else{None}}else{None};
    if reactive{act2::bottom(state,effect(actor.clone(),actor.clone(),EffectKind::RollMove));}
    if let Some(n)=block{act2::bottom(state,effect(actor.clone(),actor,EffectKind::GainBlock(n)));}Ok(())
}

pub(in crate::combat) fn obtain(state:&mut State,source:&Source)->Result<(),InputError>{
    let ordinal=state.next_permanent_gain;let next=ordinal.checked_add(1).ok_or_else(||InputError::Unsupported("permanent gain ordinal exhausted".into()))?;
    state.permanent_gains.push(super::super::super::model::PermanentCardGain{ordinal,id:crate::content::CardId::Parasite,upgrades:0});state.next_permanent_gain=next;
    if let Source::Actor(Actor::Enemy(owner))=source{if let Some(e)=state.enemies.iter_mut().find(|e|&e.instance_id==owner){if let Some(EnemyMemory::WrithingMass{parasite_pending,..})=&mut e.full_memory{*parasite_pending=false;}}}
    state.pending.push_front(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::CardAcquired)});Ok(())
}

pub(in crate::combat) fn reset_malleable(state:&mut State,owner:&EnemyHandle){
    if let Some(e)=state.enemies.iter_mut().find(|e|&e.instance_id==owner){if let Some(p)=e.full_powers.iter_mut().find(|p|p.id.0=="malleable"){p.state.amount=4;}}
}
