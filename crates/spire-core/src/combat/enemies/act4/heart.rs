//! Corrupt Heart setup and finite owner-local AI/payloads.
use super::super::*;
use super::super::super::{ActorLife, CardDestination};

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    let EnemyMemory::CorruptHeart{selection_count,first_attack,..}=memory else {
        return Err(InputError::Invariant("Corrupt Heart memory absent".into()).into());
    };
    if !memory.valid_for(EnemyId::CorruptHeart)||history.len()!=*selection_count as usize+1
        ||history.first().is_none_or(|id|id.as_str()!="heart_debilitate") {
        return Err(InputError::Invariant("Corrupt Heart selection cursor/history mismatch".into()).into());
    }
    let mut next_memory=memory.clone();
    let (name,next_stream)=match selection_count%3 {
        0=>{
            let (coin,next)=rng::uniform_below(stream,2)?;
            let first=coin==0;
            if let EnemyMemory::CorruptHeart{selection_count,first_attack,..}=&mut next_memory {
                *first_attack=Some(first);
                *selection_count=selection_count.checked_add(1).filter(|n|*n<=i32::MAX as u32)
                    .ok_or_else(||InputError::Unsupported("Corrupt Heart selection counter exhausted".into()))?;
            }
            (if first{"heart_blood_shots"}else{"heart_echo"},next)
        },
        1=>{
            let first=first_attack.ok_or_else(||InputError::Invariant("Corrupt Heart first attack missing".into()))?;
            if !matches!(history.last().map(|id|id.as_str()),Some("heart_blood_shots"|"heart_echo")) {
                return Err(InputError::Invariant("Corrupt Heart paired attack history absent".into()).into());
            }
            if let EnemyMemory::CorruptHeart{selection_count,..}=&mut next_memory {
                *selection_count=selection_count.checked_add(1).filter(|n|*n<=i32::MAX as u32)
                    .ok_or_else(||InputError::Unsupported("Corrupt Heart selection counter exhausted".into()))?;
            }
            (if first{"heart_echo"}else{"heart_blood_shots"},stream.clone())
        },
        2=>{
            if history.last().is_none_or(|id|!matches!(id.as_str(),"heart_blood_shots"|"heart_echo")) {
                return Err(InputError::Invariant("Corrupt Heart buff selection lacks paired attacks".into()).into());
            }
            if let EnemyMemory::CorruptHeart{selection_count,..}=&mut next_memory {
                *selection_count=selection_count.checked_add(1).filter(|n|*n<=i32::MAX as u32)
                    .ok_or_else(||InputError::Unsupported("Corrupt Heart selection counter exhausted".into()))?;
            }
            ("heart_buff",stream.clone())
        },
        _=>unreachable!(),
    };
    if !next_memory.valid_for(EnemyId::CorruptHeart){return Err(InputError::Invariant("Corrupt Heart selected memory invalid".into()).into())}
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:next_memory,stream:next_stream})
}

pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let ActorEnemyId::Full(EnemyId::CorruptHeart)=enemy.enemy_id else{return Err(InputError::Invariant("Heart payload owner identity mismatch".into()))};
    let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("Heart payload intent is not native".into()))};
    let actor=Actor::Enemy(enemy.instance_id.clone());
    Ok(match intent.as_str(){
        "heart_debilitate"=>{
            let mut work=vec![
                Effect{source:Source::Actor(actor.clone()),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:2}},
                Effect{source:Source::Actor(actor.clone()),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}},
                Effect{source:Source::Actor(actor.clone()),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("frail"),amount:2}},
            ];
            for id in [crate::content::CardId::Dazed,crate::content::CardId::Slimed,crate::content::CardId::Wound,crate::content::CardId::Burn,crate::content::CardId::Void]{
                work.push(Effect{source:Source::Actor(actor.clone()),target:Some(Actor::Player),kind:EffectKind::CreateCard{id,destination:CardDestination::ShuffledDraw,upgrades:0}});
            }
            work
        },
        "heart_blood_shots"=>(0..12).map(|_|damage(actor.clone(),intent)).collect(),
        "heart_echo"=>vec![damage(actor,intent)],
        "heart_buff"=>vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::CorruptHeartBuff}],
        _=>return Err(InputError::Unsupported("Corrupt Heart intent unavailable".into())),
    })
}

pub(in crate::combat::enemies) fn selected(state:&mut State,owner:&EnemyHandle,intent:FullIntentId){
    let Some(enemy)=state.enemies.iter_mut().find(|enemy|&enemy.instance_id==owner&&enemy.enemy_id==ActorEnemyId::Full(EnemyId::CorruptHeart)) else{return};
    let damage=match intent{FullIntentId::HeartBloodShots=>2,FullIntentId::HeartEcho=>40,_=>0};
    if let Some(entry)=enemy.full_damage_entries.get_mut(&intent){entry.base=damage;entry.output=damage;}
}

pub(in crate::combat) fn apply_buff(state:&mut State,effect:&Effect)->Result<(),InputError>{
    let Some(Actor::Enemy(owner))=effect.target.as_ref() else{return Err(InputError::Invariant("Heart buff target absent".into()))};
    let enemy=state.enemies.iter_mut().find(|enemy|&enemy.instance_id==owner&&enemy.enemy_id==ActorEnemyId::Full(EnemyId::CorruptHeart)&&enemy.life==ActorLife::Active&&enemy.hp>0)
        .ok_or_else(||InputError::Invariant("Heart buff owner is not living".into()))?;
    if enemy.intent_id.as_str()!="heart_buff" {return Err(InputError::Invariant("Heart buff is not the selected move".into()))}
    let Some(EnemyMemory::CorruptHeart{selection_count,buff_count,last_buff_selection,..})=enemy.full_memory.as_mut() else{return Err(InputError::Invariant("Heart buff counter absent".into()))};
    let selection=selection_count.checked_sub(1).ok_or_else(||InputError::Invariant("Heart buff has no selected move cursor".into()))?;
    if *last_buff_selection==Some(selection){return Err(InputError::Invariant("Heart selected buff already executed".into()))}
    let ordinal=*buff_count;
    *buff_count=buff_count.checked_add(1).filter(|n|*n<=i32::MAX as u32)
        .ok_or_else(||InputError::Unsupported("Corrupt Heart buff counter exhausted".into()))?;
    *last_buff_selection=Some(selection);
    let actor=Actor::Enemy(owner.clone());
    let mut work=Vec::new();
    let negative=enemy.full_powers.iter().find(|power|power.id.0=="strength").map_or(0,|power|power.state.amount.min(0));
    let compensation=negative.checked_neg().and_then(|amount|amount.checked_add(2))
        .ok_or_else(||InputError::Unsupported("Heart Strength compensation overflow".into()))?;
    work.push(Effect{source:effect.source.clone(),target:Some(actor.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:compensation}});
    match ordinal{
        0=>work.push(Effect{source:effect.source.clone(),target:Some(actor.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("artifact"),amount:2}}),
        1=>work.push(Effect{source:effect.source.clone(),target:Some(actor.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("beat_of_death"),amount:1}}),
        2=>work.push(Effect{source:effect.source.clone(),target:Some(actor.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("painful_stabs"),amount:1}}),
        _=>(),
    }
    if ordinal>=3{
        let amount=if ordinal==3{10}else{50};
        work.push(Effect{source:effect.source.clone(),target:Some(actor.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount}});
    }
    for child in work.into_iter().rev(){super::super::super::engine::validate_effect(&child,state).map_err(|error|InputError::Invariant(format!("Heart buff child failed preflight: {error:?}")))?;state.pending.push_front(child)}
    Ok(())
}

pub(in crate::combat::enemies) fn prebattle_work(_enemy:&EnemyState)->Vec<Effect>{Vec::new()}
