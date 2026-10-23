//! A0 Slaver species; Entangle's consumed flag changes at takeTurn, not selection.
use super::*;

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,super::super::SelectError>{
    let mut cursor=stream.clone();let name=match memory {
        EnemyMemory::Taskmaster=>"taskmaster_whip",
        EnemyMemory::SlaverBlue=>{
            let (ticket,next)=rng::uniform_below(&cursor,100)?;cursor=next;
            let proposed=if ticket<40{"slaver_rake"}else{"slaver_stab"};
            if history.ends_with(&[FullIntentId::from_name(proposed);2]){if proposed=="slaver_rake"{"slaver_stab"}else{"slaver_rake"}}else{proposed}
        },
        EnemyMemory::SlaverRed{used_entangle}=>{
            if history.is_empty(){"slaver_stab"}else{
                let (ticket,next)=rng::uniform_below(&cursor,100)?;cursor=next;
                if !used_entangle {if ticket>=75{"slaver_entangle"}else if history.ends_with(&[FullIntentId::from_name("slaver_scrape");2]){"slaver_stab"}else{"slaver_scrape"}}
                else if ticket>=55&&!history.ends_with(&[FullIntentId::from_name("slaver_stab");2]){"slaver_stab"}
                else if !history.ends_with(&[FullIntentId::from_name("slaver_scrape");2]){"slaver_scrape"}else{"slaver_stab"}
            }
        },
        _=>return Err(InputError::Invariant("Slaver memory mismatch".into()).into()),
    };Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:memory.clone(),stream:cursor})
}

pub(super) fn take_turn(enemy:&mut EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("Slaver full intent absent".into()))};
    Ok(match intent.as_str(){
        "slaver_stab"=>vec![damage(actor,intent)],
        "slaver_rake"|"slaver_scrape"=>vec![damage(actor.clone(),intent),effect(actor,Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId(if intent.as_str()=="slaver_rake"{"weak"}else{"vulnerable"}),amount:1})],
        "taskmaster_whip"=>vec![damage(actor.clone(),intent),effect(actor,Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Wound,destination:super::super::super::CardDestination::Discard,upgrades:0})],
        "slaver_entangle"=>{
            let Some(EnemyMemory::SlaverRed{used_entangle})=enemy.full_memory.as_mut()else{return Err(InputError::Invariant("Red Slaver Entangle memory absent".into()))};*used_entangle=true;
            vec![effect(actor,Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("entangled"),amount:1})]
        },
        _=>return Err(InputError::Invariant("Slaver intent mismatch".into())),
    })
}
