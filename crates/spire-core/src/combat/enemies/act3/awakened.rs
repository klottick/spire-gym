//! Same-actor boss revival and source-specific final Cultist escape.
use super::*;
use super::super::super::{ActorLife,Hook};

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    let EnemyMemory::AwakenedOne{phase,first}=memory else{return Err(InputError::Invariant("Awakened phase absent".into()).into())};
    let mut cursor=stream.clone();let last=history.last().map(|m|m.as_str());let twice=|name|history.len()>=2&&history[history.len()-2..].iter().all(|m|m.as_str()==name);
    let name=if *phase==AwakenedPhase::Rebirth{"awa_rebirth"}else if *first{if *phase==AwakenedPhase::First{"awa_slash"}else{"awa_echo"}}else{
        let (n,next)=rng::uniform_below(&cursor,100)?;cursor=next;
        if *phase==AwakenedPhase::First{if n<25&&last!=Some("awa_soul")||n>=25&&twice("awa_slash"){"awa_soul"}else{"awa_slash"}}
        else if n<50{if twice("awa_sludge"){"awa_tackle"}else{"awa_sludge"}}else if twice("awa_tackle"){"awa_sludge"}else{"awa_tackle"}
    };
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:EnemyMemory::AwakenedOne{phase:*phase,first:false},stream:cursor})
}

pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let name=enemy.intent_id.as_str();let own=|kind|effect(actor.clone(),actor.clone(),kind);
    Ok(match name{
        "awa_slash"|"awa_echo"=>vec![damage(actor.clone(),FullIntentId::from_name(name))],
        "awa_soul"=>vec![damage(actor.clone(),FullIntentId::from_name(name));4],
        "awa_tackle"=>vec![damage(actor.clone(),FullIntentId::from_name(name));3],
        "awa_sludge"=>vec![damage(actor.clone(),FullIntentId::from_name(name)),effect(actor.clone(),Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Void,destination:super::super::super::CardDestination::BottomDraw,upgrades:0})],
        "awa_rebirth"=>vec![own(EffectKind::AwakenedRebirth)],
        _=>return Err(InputError::Unsupported("Awakened intent unavailable".into())),
    })
}

pub(in crate::combat) fn down(state:&mut State,owner:&EnemyHandle,source:&Source)->bool{
    let Some(enemy)=state.enemies.iter_mut().find(|e|&e.instance_id==owner&&e.enemy_id==ActorEnemyId::Full(EnemyId::AwakenedOne)&&e.hp==0&&e.life==ActorLife::Active&&matches!(e.full_memory,Some(EnemyMemory::AwakenedOne{phase:AwakenedPhase::First,..})))else{return false};
    enemy.life=ActorLife::HalfDead;enemy.full_memory=Some(EnemyMemory::AwakenedOne{phase:AwakenedPhase::Rebirth,first:true});let i=ActorIntentId::Full(FullIntentId::from_name("awa_rebirth"));enemy.intent_id=i.clone();enemy.selected_move=i.clone();enemy.move_history.push(i);enemy.intent_damage=None;
    let actor=Actor::Enemy(owner.clone());state.pending.push_front(effect(actor.clone(),actor.clone(),EffectKind::AwakenedCollapse));state.pending.push_front(Effect{source:source.clone(),target:Some(actor),kind:EffectKind::Hook(Hook::HalfDeath)});true
}

pub(in crate::combat) fn apply(state:&mut State,effect:&Effect)->Result<(),InputError>{
    let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("Awakened lifecycle owner absent".into()))};
    match effect.kind{
        EffectKind::AwakenedCollapse=>{
            let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).unwrap();let mut removals=vec![];
            for p in &enemy.full_powers{let remove=if matches!(p.id.0,"curiosity"|"unawakened"|"shackled"){true}else{super::super::super::powers::cleanse_debuff(p).ok_or_else(||InputError::Unsupported(format!("Awakened cleanse power category unavailable: {}",p.id.0)))?};if remove{removals.push(p.id);}}
            let actor=Actor::Enemy(owner.clone());for id in removals.into_iter().rev(){state.pending.push_front(super::super::effect(actor.clone(),actor.clone(),EffectKind::RemoveFullPower(id)));}
        },
        EffectKind::AwakenedRebirth=>{
            let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap();enemy.max_hp=300;enemy.life=ActorLife::Active;let actor=Actor::Enemy(owner.clone());state.pending.push_front(super::super::effect(actor.clone(),actor.clone(),EffectKind::AwakenedCanLose));state.pending.push_front(super::super::effect(actor.clone(),actor,EffectKind::RevivalHeal(300)));
        },
        EffectKind::AwakenedCanLose=>{state.enemies.iter_mut().find(|e|&e.instance_id==owner).unwrap().full_memory=Some(EnemyMemory::AwakenedOne{phase:AwakenedPhase::Second,first:true});},
        _=>return Err(InputError::Invariant("Awakened lifecycle stage unavailable".into())),
    }Ok(())
}

pub(in crate::combat) fn card_played(state:&mut State,source:&Source)->Result<(),InputError>{
    let Source::Card(owner)=source else{return Ok(())};
    if !state.in_play.as_ref().and_then(|c|(c.handle()==*owner).then(||c.full()).flatten()).is_some_and(|c|crate::content::Catalog::standard().card(c.id).is_some_and(|d|d.card_type==crate::content::CardType::Power)){return Ok(())}
    let mut insertion=state.pending.iter().position(|e|e.kind==EffectKind::FinishCard&&e.source==*source).ok_or_else(||InputError::Invariant("Power after-use continuation absent".into()))?;
    for enemy in state.enemies.iter().filter(|e|e.life==ActorLife::Active&&e.hp>0){if let Some(power)=enemy.full_powers.iter().find(|p|p.id.0=="curiosity"&&p.state.amount>0){let actor=Actor::Enemy(enemy.instance_id.clone());state.pending.insert(insertion,effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:power.state.amount}));insertion+=1;}}
    Ok(())
}

pub(in crate::combat) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult){
    let Actor::Enemy(owner)=&result.target else{return};
    if result.hp_after!=0||result.hp_lost<=0||!state.enemies.iter().any(|e|&e.instance_id==owner&&e.enemy_id==ActorEnemyId::Full(EnemyId::AwakenedOne)&&matches!(e.full_memory,Some(EnemyMemory::AwakenedOne{phase:AwakenedPhase::Second,..}))){return}
    let targets=state.enemies.iter().filter(|e|e.enemy_id==ActorEnemyId::Full(EnemyId::Cultist)&&e.hp>0&&e.life==ActorLife::Active).map(|e|e.instance_id.clone()).collect::<Vec<_>>();
    state.pending.retain(|e|!((matches!(e.kind,EffectKind::EnemyMove|EffectKind::RollMove)&&matches!(&e.target,Some(Actor::Enemy(id)) if targets.contains(id)))||(matches!(e.kind,EffectKind::Damage(super::super::super::DamagePayload::FullEnemyMove(_)))&&matches!(&e.source,Source::Actor(Actor::Enemy(id)) if targets.contains(id)))));
    for target in targets{act2::bottom(state,effect(Actor::Enemy(owner.clone()),Actor::Enemy(target),EffectKind::EscapeEnemy));}
}
