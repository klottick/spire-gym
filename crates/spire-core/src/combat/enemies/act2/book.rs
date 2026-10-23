//! Book's selected Multi count and source-bottom, inflictor-owned Wound reaction.
use super::*;
use super::super::super::model::DamageResult;

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,super::super::SelectError>{
    let EnemyMemory::BookOfStabbing{stab_count}=memory else{return Err(InputError::Invariant("Book memory absent".into()).into())};
    let (ticket,next)=rng::uniform_below(stream,100)?;let multi=if ticket<15{history.last().is_some_and(|m|m.as_str()=="book_big")}else{!history.ends_with(&[FullIntentId::from_name("book_multi");2])};
    let count=if multi{stab_count.checked_add(1).filter(|n|*n<=i32::MAX as u32).ok_or_else(||InputError::Unsupported("Book stab counter exhausted".into()))?}else{*stab_count};
    Ok(MoveSelection{intent:FullIntentId::from_name(if multi{"book_multi"}else{"book_big"}),memory:EnemyMemory::BookOfStabbing{stab_count:count},stream:next})
}

pub(super) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("Book full intent absent".into()))};
    Ok(match intent.as_str(){
        "book_multi"=>{let Some(EnemyMemory::BookOfStabbing{stab_count})=enemy.full_memory else{return Err(InputError::Invariant("Book stab count absent".into()))};vec![effect(actor.clone(),actor,EffectKind::BookStab{remaining:stab_count})]},
        "book_big"=>vec![damage(actor,intent)],_=>return Err(InputError::Invariant("Book intent mismatch".into())),
    })
}

pub(in crate::combat) fn stab(state:&mut State,owner:&EnemyHandle,remaining:u32){
    let actor=Actor::Enemy(owner.clone());if remaining>1{state.pending.push_front(effect(actor.clone(),actor.clone(),EffectKind::BookStab{remaining:remaining-1}));}
    state.pending.push_front(damage(actor,FullIntentId::from_name("book_multi")));
}

pub(super) fn after_damage(state:&mut State,result:&DamageResult){
    if result.target!=Actor::Player||result.hp_lost<=0||!(result.normal||result.hp_loss){return}
    let Source::Actor(Actor::Enemy(owner))=&result.source else{return};
    if state.enemies.iter().any(|e|&e.instance_id==owner&&e.full_powers.iter().any(|p|p.id.0=="painful_stabs"&&p.state.amount>0)){
        bottom(state,effect(Actor::Enemy(owner.clone()),Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Wound,destination:super::super::super::CardDestination::Discard,upgrades:0}));
    }
}
