//! Darkling temporary downing is neither Slime replacement nor final actor death.
use super::*;
use super::super::super::{ActorLife,ActorIntentId};

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,half_dead:bool)->Result<MoveSelection,SelectError>{
    let EnemyMemory::Darkling{center,nip,first}=memory else{return Err(InputError::Invariant("Darkling memory missing".into()).into())};
    if half_dead{return Ok(MoveSelection{intent:FullIntentId::from_name("darkling_reincarnate"),memory:memory.clone(),stream:stream.clone()})}
    let mut cursor=stream.clone();let (mut n,next)=rng::uniform_below(&cursor,100)?;cursor=next;
    let last=history.last().map(|m|m.as_str());
    let name=if *first{if n<50{"darkling_harden"}else{"darkling_nip"}}else{loop{
        if n<40 {
            if !center&&last!=Some("darkling_chomp"){break "darkling_chomp"}
            let (v,next)=rng::uniform_below(&cursor,60)?;cursor=next;n=40+v;
        }else if n<70 {break if last==Some("darkling_harden"){"darkling_nip"}else{"darkling_harden"}}
        else if history.len()<2||!history[history.len()-2..].iter().all(|m|m.as_str()=="darkling_nip"){break "darkling_nip"}
        else {let (v,next)=rng::uniform_below(&cursor,100)?;cursor=next;n=v;}
    }};
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:EnemyMemory::Darkling{center:*center,nip:*nip,first:false},stream:cursor})
}

pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());
    Ok(match enemy.intent_id.as_str(){
        "darkling_nip"=>vec![damage(actor,FullIntentId::from_name("darkling_nip"))],
        "darkling_chomp"=>vec![damage(actor.clone(),FullIntentId::from_name("darkling_chomp")),damage(actor,FullIntentId::from_name("darkling_chomp"))],
        "darkling_harden"=>vec![effect(actor.clone(),actor,EffectKind::GainBlock(12))],
        "darkling_regrowing"=>vec![],
        "darkling_reincarnate"=>vec![effect(actor.clone(),actor.clone(),EffectKind::RevivalHeal(enemy.max_hp/2)),effect(actor.clone(),actor.clone(),EffectKind::DarklingRevive),effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("life_link"),amount:1})],
        _=>return Err(InputError::Unsupported("Darkling intent unavailable".into())),
    })
}

fn is_darkling(enemy:&EnemyState)->bool{enemy.enemy_id==ActorEnemyId::Full(EnemyId::Darkling)}

/// Returns true when this lethal payload belongs to a revival lifecycle. Source
/// callbacks survive, but generic corpse death/Victory cannot steal its ownership.
pub(in crate::combat) fn down(state:&mut State,owner:&EnemyHandle,source:&Source)->bool{
    let Some(enemy)=state.enemies.iter_mut().find(|e|&e.instance_id==owner&&is_darkling(e)&&e.hp==0&&e.life==ActorLife::Active)else{return false};
    enemy.life=ActorLife::HalfDead;
    let intent=ActorIntentId::Full(FullIntentId::from_name("darkling_regrowing"));enemy.intent_id=intent.clone();enemy.selected_move=intent.clone();enemy.move_history.push(intent);enemy.intent_damage=None;
    let target=Actor::Enemy(owner.clone());
    if state.enemies.iter().filter(|e|is_darkling(e)).all(|e|e.life==ActorLife::HalfDead){state.pending.push_front(effect(target.clone(),target.clone(),EffectKind::DarklingsDefeat));}
    state.pending.push_front(effect(target.clone(),target.clone(),EffectKind::DarklingCollapse));
    state.pending.push_front(Effect{source:source.clone(),target:Some(target),kind:EffectKind::Hook(super::super::super::Hook::HalfDeath)});
    true
}

pub(in crate::combat) fn apply(state:&mut State,effect:&Effect)->Result<(),InputError>{
    let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("revival target absent".into()))};
    match effect.kind {
        EffectKind::DarklingCollapse=>{let e=state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap();e.powers.clear();e.full_powers.clear();},
        EffectKind::RevivalHeal(amount)=>{let e=state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap();e.hp=(e.hp+amount).min(e.max_hp);},
        EffectKind::DarklingRevive=>{state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap().life=ActorLife::Active;},
        EffectKind::DarklingsDefeat=>{
            let ids=state.enemies.iter_mut().filter(|e|is_darkling(e)&&e.life==ActorLife::HalfDead).map(|e|{e.life=ActorLife::Active;e.instance_id.clone()}).collect::<Vec<_>>();
            if act2::combat_victory_ready(state)&&state.player.hp>0{act2::queue_victory(state,&effect.source);}
            for id in ids.into_iter().rev(){state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Enemy(id)),kind:EffectKind::Hook(super::super::super::Hook::Death)});}
        },
        _=>return Err(InputError::Invariant("Darkling lifecycle payload mismatch".into())),
    }Ok(())
}
