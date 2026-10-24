//! Time Warp requests the existing end-turn scheduler after the owned card.
use super::*;
use super::super::super::ActorLife;

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,context:&act2::EnemyRollContext)->Result<MoveSelection,SelectError>{
    let EnemyMemory::TimeEater{haste_used}=memory else{return Err(InputError::Invariant("Time Eater memory absent".into()).into())};
    let actor=context.actors.iter().find(|a|a.owner==context.owner).ok_or_else(||InputError::Invariant("Time Eater selection owner absent".into()))?;
    if !*haste_used&&actor.hp<actor.max_hp/2{return Ok(MoveSelection{intent:FullIntentId::from_name("time_haste"),memory:EnemyMemory::TimeEater{haste_used:true},stream:stream.clone()})}
    let (mut n,mut cursor)=rng::uniform_below(stream,100)?;let last=history.last().map(|m|m.as_str());let twice=history.len()>=2&&history[history.len()-2..].iter().all(|m|m.as_str()=="time_reverb");
    let name=loop{if n<45{
        if !twice{break "time_reverb"}let(v,next)=rng::uniform_below(&cursor,50)?;cursor=next;n=50+v;
    }else if n<80{
        if last!=Some("time_slam"){break "time_slam"}let(v,next)=rng::uniform_below(&cursor,1<<24)?;cursor=next;break if v<11_072_963{"time_reverb"}else{"time_ripple"}
    }else{
        if last!=Some("time_ripple"){break "time_ripple"}let(v,next)=rng::uniform_below(&cursor,75)?;cursor=next;n=v;
    }};
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:memory.clone(),stream:cursor})
}

pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let own=|kind|effect(actor.clone(),actor.clone(),kind);
    Ok(match enemy.intent_id.as_str(){
        "time_reverb"=>vec![damage(actor.clone(),FullIntentId::from_name("time_reverb"));3],
        "time_slam"=>vec![damage(actor.clone(),FullIntentId::from_name("time_slam")),effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("draw_reduction"),amount:1})],
        "time_ripple"=>vec![own(EffectKind::GainBlock(20)),effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:1}),effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:1})],
        "time_haste"=>vec![own(EffectKind::TimeEaterCleanse),own(EffectKind::RemoveFullPower(FullPowerId("shackled"))),own(EffectKind::Heal((enemy.max_hp/2-enemy.hp).max(0)))],
        _=>return Err(InputError::Unsupported("Time Eater intent unavailable".into())),
    })
}

pub(in crate::combat) fn after_card(state:&mut State)->Result<(),InputError>{
    // A 12th-card checkpoint inside an autoplay child is an owned barrier,
    // not permission to drop its suspended parent. Parent FinishCard work is
    // already accepted and must unwind before the ordinary forced end turn.
    if state.pending_time_warp.is_some(){return Ok(())}
    let owners=state.enemies.iter().enumerate().filter(|(_,e)|e.enemy_id==ActorEnemyId::Full(EnemyId::TimeEater)&&e.life==ActorLife::Active&&e.hp>0&&e.full_powers.iter().any(|p|p.id.0=="time_warp")).map(|(i,_)|i).collect::<Vec<_>>();
    if owners.len()>1{return Err(InputError::Unsupported("multiple Time Warp owners unavailable".into()))}
    let Some(index)=owners.first().copied() else{return Ok(())};
    let power=state.enemies[index].full_powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap();
    if !(0..12).contains(&power.state.amount)||state.pending_time_warp.is_some(){return Err(InputError::Invariant("Time Warp counter/request frontier".into()))}
    if power.state.amount<11{power.state.amount+=1;return Ok(())}
    if !state.pending.iter().any(|e|e.kind==EffectKind::Decision){return Err(InputError::Invariant("Time Warp card decision continuation absent".into()))}
    power.state.amount=0;let owner=state.enemies[index].instance_id.clone();state.pending_time_warp=Some(owner.clone());
    for queued in &mut state.pending{if let EffectKind::Ironclad(work)=&mut queued.kind{match work{
        super::super::super::cards::IroncladEffect::HavocQueued{cancelled,..}
        |super::super::super::cards::IroncladEffect::DoubleTapRepeat{cancelled,..}=>*cancelled=true,
        _=>(),
    }}}
    let actors=state.enemies.iter().map(|e|Actor::Enemy(e.instance_id.clone())).collect::<Vec<_>>();
    for target in actors.into_iter().rev(){state.pending.push_front(effect(Actor::Enemy(owner.clone()),target,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:2}));}
    let at=state.pending.iter().position(|e|e.kind==EffectKind::Decision).unwrap();state.pending.insert(at,Effect{source:Source::Actor(Actor::Enemy(owner)),target:None,kind:EffectKind::TimeWarpEndTurn});Ok(())
}

pub(in crate::combat) fn force_end(state:&mut State,effect:&Effect)->Result<(),InputError>{
    if state.in_play.is_some()||state.pending_selection.is_some()||state.pending.len()!=1||state.pending.front().is_none_or(|e|e.kind!=EffectKind::Decision){return Err(InputError::Invariant("Time Warp reached before owned card work completed".into()))}
    state.pending.pop_front();state.pending_time_warp=None;state.pending.push_front(Effect{source:effect.source.clone(),target:None,kind:EffectKind::EndPlayerTurn});Ok(())
}

pub(in crate::combat) fn cleanse(state:&mut State,owner:&EnemyHandle)->Result<(),InputError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Haste cleanse owner absent".into()))?;let mut removals=vec![];
    for p in &enemy.full_powers{if super::super::super::powers::cleanse_debuff(p).ok_or_else(||InputError::Unsupported(format!("Haste cleanse power category unavailable: {}",p.id.0)))?{removals.push(p.id);}}
    let actor=Actor::Enemy(owner.clone());for id in removals.into_iter().rev(){state.pending.push_front(super::super::effect(actor.clone(),actor.clone(),EffectKind::RemoveFullPower(id)));}Ok(())
}
