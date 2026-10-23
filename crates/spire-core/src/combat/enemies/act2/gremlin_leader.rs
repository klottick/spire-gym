//! Live-count Leader AI and the five real small species it can construct.
use super::*;
use super::super::super::{ActorLife,EffectError,SpawnRequest};

pub(in crate::combat::enemies) const POOL:[EnemyId;8]=[EnemyId::GremlinMad,EnemyId::GremlinMad,EnemyId::GremlinSneaky,EnemyId::GremlinSneaky,EnemyId::GremlinFat,EnemyId::GremlinFat,EnemyId::GremlinShield,EnemyId::GremlinWizard];
pub(in crate::combat) fn is_small(id:EnemyId)->bool{matches!(id,EnemyId::GremlinMad|EnemyId::GremlinSneaky|EnemyId::GremlinFat|EnemyId::GremlinShield|EnemyId::GremlinWizard)}

pub(super) fn select_leader(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,context:&EnemyRollContext)->Result<MoveSelection,super::super::SelectError>{
    let count=context.actors.iter().filter(|a|a.owner!=context.owner&&a.hp>0).count();let last=history.last().map(|id|id.as_str());
    let (mut ticket,mut cursor)=rng::uniform_below(stream,100)?;
    let name=loop {
        let proposed=match count {0=>if ticket<75{"leader_rally"}else{"leader_stab"},1=>match ticket{0..=49=>"leader_rally",50..=79=>"leader_encourage",_=>"leader_stab"},_=>if ticket<66{"leader_encourage"}else{"leader_stab"}};
        if count!=1||last!=Some(proposed){break if last==Some(proposed){if proposed=="leader_stab"{if count==0{"leader_rally"}else{"leader_encourage"}}else{"leader_stab"}}else{proposed}}
        if proposed=="leader_encourage"{break "leader_stab"}
        let (draw,next)=if proposed=="leader_rally"{let (n,next)=rng::uniform_below(&cursor,50)?;(50+n,next)}else{rng::uniform_below(&cursor,81)?};ticket=draw;cursor=next;
    };
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:memory.clone(),stream:cursor})
}

pub(in crate::combat::enemies) fn select_small(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,super::super::SelectError>{
    let name=match memory{EnemyMemory::GremlinMad=>"gremlin_scratch",EnemyMemory::GremlinSneaky=>"gremlin_puncture",EnemyMemory::GremlinFat=>"gremlin_smash",EnemyMemory::GremlinShield=>if history.last().is_some_and(|id|id.as_str()=="gremlin_bash"){"gremlin_bash"}else{"gremlin_protect"},EnemyMemory::GremlinWizard{charge}=>if *charge==3{"gremlin_blast"}else{"gremlin_charge"},_=>return Err(InputError::Invariant("small Gremlin memory mismatch".into()).into())};
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:memory.clone(),stream:stream.clone()})
}

pub(super) fn take_turn(state:&mut State,owner:&EnemyHandle)->Result<Vec<Effect>,EffectError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Gremlin turn owner absent".into()))?;
    let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("Gremlin full intent absent".into()).into())};
    let actor=Actor::Enemy(owner.clone());
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::GremlinLeader){return Ok(match intent.as_str(){
        "leader_stab"=>(0..3).map(|_|damage(actor.clone(),intent)).collect(),
        "leader_encourage"=>{let mut work=vec![];for a in spawn::ordered_actors(state){let target=Actor::Enemy(a.instance_id.clone());work.push(effect(actor.clone(),target.clone(),EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3}));if a.instance_id!=*owner{work.push(effect(actor.clone(),target,EffectKind::GainBlock(6)));}}work},
        "leader_rally"=>vec![effect(actor.clone(),actor,EffectKind::RallyGremlins{remaining:2})],
        _=>return Err(InputError::Invariant("unhandled Gremlin Leader intent".into()).into()),
    })}
    if !matches!(enemy.enemy_id,ActorEnemyId::Full(id) if is_small(id)){return Err(InputError::Invariant("small Gremlin turn species".into()).into())}
    if intent.as_str()=="gremlin_smash"{return Ok(vec![damage(actor.clone(),intent),effect(actor,Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:1})])}
    let mut work=vec![];
    let next=match intent.as_str(){
        "gremlin_scratch"|"gremlin_puncture"|"gremlin_bash"=>{work.push(damage(actor.clone(),intent));intent},
        "gremlin_charge"|"gremlin_blast"=>{
            let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap();let Some(EnemyMemory::GremlinWizard{charge})=enemy.full_memory.as_mut()else{return Err(InputError::Invariant("Wizard charge owner absent".into()).into())};
            if intent.as_str()=="gremlin_blast"{*charge=0;work.push(damage(actor.clone(),intent));}else{*charge+=1;}
            FullIntentId::from_name(if *charge==3{"gremlin_blast"}else{"gremlin_charge"})
        },
        "gremlin_protect"=>{work.push(effect(actor.clone(),actor.clone(),EffectKind::ProtectGremlin));FullIntentId::from_name(if spawn::ordered_actors(state).len()>1{"gremlin_protect"}else{"gremlin_bash"})},
        _=>return Err(InputError::Invariant("unhandled small Gremlin intent".into()).into()),
    };
    state.pending.retain(|e|!(e.kind==EffectKind::RollMove&&e.target==Some(actor.clone())));
    work.push(effect(actor.clone(),actor,EffectKind::SetEnemyMove{intent:next}));Ok(work)
}

pub(in crate::combat) fn rally(state:&mut State,owner:&EnemyHandle,remaining:u8)->Result<(),EffectError>{
    let live=spawn::ordered_actors(state);let free=[1,2,0].into_iter().find(|slot|!live.iter().any(|a|state.actor_slots.get(&a.instance_id)==Some(slot)));
    let Some(slot)=free else{return Ok(())};
    let summoner=state.enemies.iter_mut().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Rally owner absent".into()))?;
    let (ticket,next)=rng::uniform_below(summoner.full_ai_rng.as_ref().ok_or_else(||InputError::Invariant("Rally stream absent".into()))?,8)?;summoner.full_ai_rng=Some(next);
    let actor=Actor::Enemy(owner.clone());if remaining>1{state.pending.push_front(effect(actor.clone(),actor,EffectKind::RallyGremlins{remaining:remaining-1}));}
    state.pending.push_front(Effect{source:Source::Actor(Actor::Enemy(owner.clone())),target:None,kind:EffectKind::SpawnEnemy(SpawnRequest{summoner:owner.clone(),slot,species:POOL[ticket as usize]})});Ok(())
}

pub(in crate::combat) fn protect(state:&mut State,owner:&EnemyHandle)->Result<(),EffectError>{
    let targets=spawn::ordered_actors(state).into_iter().filter(|e|&e.instance_id!=owner&&e.intent_id.as_str()!="gremlin_escape").map(|e|e.instance_id.clone()).collect::<Vec<_>>();
    let target=if targets.is_empty(){owner.clone()}else{
        let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap();let (index,next)=rng::uniform_below(enemy.full_ai_rng.as_ref().unwrap(),targets.len() as u128)?;enemy.full_ai_rng=Some(next);targets[index as usize].clone()
    };
    state.pending.push_front(effect(Actor::Enemy(owner.clone()),Actor::Enemy(target),EffectKind::GainBlock(7)));Ok(())
}

pub(in crate::combat) fn set_move(state:&mut State,owner:&EnemyHandle,intent:FullIntentId)->Result<(),InputError>{
    let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("SetMove owner absent".into()))?;
    enemy.intent_id=ActorIntentId::Full(intent);enemy.selected_move=ActorIntentId::Full(intent);enemy.move_history.push(ActorIntentId::Full(intent));super::super::super::effects::refresh_enemy_values(state)
}

pub(super) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult){
    let Actor::Enemy(owner)=&result.target else{return};
    let Some(enemy)=state.enemies.iter().find(|e|&e.instance_id==owner)else{return};
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::GremlinLeader)&&result.hp_after==0&&result.hp_lost>0{
        let others=spawn::ordered_actors(state).into_iter().map(|e|e.instance_id.clone()).collect::<Vec<_>>();
        for target in others{bottom(state,effect(Actor::Enemy(owner.clone()),Actor::Enemy(target),EffectKind::EscapeEnemy));}
    }else if enemy.enemy_id==ActorEnemyId::Full(EnemyId::GremlinMad)&&result.normal&&result.hp_lost>0&&result.hp_after>0&&result.source!=Source::None{
        if let Some(angry)=enemy.full_powers.iter().find(|p|p.id.0=="angry"){state.pending.push_front(effect(Actor::Enemy(owner.clone()),Actor::Enemy(owner.clone()),EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:angry.state.amount}));}
    }
}

pub(in crate::combat) fn escape(state:&mut State,owner:&EnemyHandle){
    if let Some(enemy)=state.enemies.iter_mut().find(|e|&e.instance_id==owner&&e.hp>0&&e.life==ActorLife::Active){enemy.life=ActorLife::Escaped;}
    if state.player.hp>0&&combat_victory_ready(state){
        state.pending.push_front(Effect{source:Source::None,target:None,kind:EffectKind::Terminal(super::super::super::Outcome::Victory)});
        state.pending.push_front(Effect{source:Source::None,target:None,kind:EffectKind::Hook(super::super::super::Hook::CombatSettled)});
    }
}
