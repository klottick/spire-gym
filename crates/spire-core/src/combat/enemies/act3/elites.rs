//! Source-qualified Act3 elite producers on the shared native resolver.
use super::*;

pub(in crate::combat::enemies) fn supports(id:EnemyId)->bool{matches!(id,EnemyId::GiantHead|EnemyId::Nemesis)}
pub(in crate::combat) fn valid_intent(id:EnemyId,name:&str)->bool{matches!((id,name),
    (EnemyId::GiantHead,"head_count"|"head_time"|"head_glare")|
    (EnemyId::Nemesis,"nemesis_tri"|"nemesis_scythe"|"nemesis_burn"))}
pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    if matches!(memory,EnemyMemory::Nemesis{..}){return select_nemesis(history,memory,stream)}
    let EnemyMemory::GiantHead{count}=memory else{return Err(InputError::Invariant("Head memory absent".into()).into())};
    let (ticket,next)=rng::uniform_below(stream,100)?;let mut count=*count;
    let name=if count>1{
        count-=1;let twice=|name|history.len()>=2&&history[history.len()-2..].iter().all(|m|m.as_str()==name);
        if ticket<50{if twice("head_glare"){"head_count"}else{"head_glare"}}else if twice("head_count"){"head_glare"}else{"head_count"}
    }else{count=(count-1).max(-6);"head_time"};
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:EnemyMemory::GiantHead{count},stream:next})
}
fn select_nemesis(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    let EnemyMemory::Nemesis{first,cooldown}=memory else{return Err(InputError::Invariant("Nemesis memory absent".into()).into())};
    let (n,mut cursor)=rng::uniform_below(stream,100)?;let mut cooldown=cooldown.saturating_sub(1);
    let last=history.last().map(|m|m.as_str());let twice=history.len()>=2&&history[history.len()-2..].iter().all(|m|m.as_str()=="nemesis_tri");
    let mut coin=||->Result<bool,SelectError>{let (n,next)=rng::uniform_below(&cursor,2)?;cursor=next;Ok(n==0)};
    let name=if *first{if n<50{"nemesis_tri"}else{"nemesis_burn"}}
    else if n<30{
        if last!=Some("nemesis_scythe")&&cooldown==0{"nemesis_scythe"}
        else if coin()?{if twice{"nemesis_burn"}else{"nemesis_tri"}}
        else if last!=Some("nemesis_burn"){"nemesis_burn"}else{"nemesis_tri"}
    }else if n<65{
        if !twice{"nemesis_tri"}else if coin()?&&cooldown==0{"nemesis_scythe"}else{"nemesis_burn"}
    }else if last!=Some("nemesis_burn"){"nemesis_burn"}
    else if coin()?&&cooldown==0{"nemesis_scythe"}else{"nemesis_tri"};
    if name=="nemesis_scythe"{cooldown=2;}
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:EnemyMemory::Nemesis{first:false,cooldown},stream:cursor})
}
pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    if !matches!(enemy.enemy_id,ActorEnemyId::Full(id) if valid_intent(id,enemy.intent_id.as_str())){
        return Err(InputError::Unsupported("elite intent/species mismatch".into()))
    }
    let actor=Actor::Enemy(enemy.instance_id.clone());
    let mut work=match enemy.intent_id.as_str(){
        "nemesis_tri"=>vec![damage(actor.clone(),FullIntentId::from_name("nemesis_tri"));3],
        "nemesis_scythe"=>vec![damage(actor.clone(),FullIntentId::from_name("nemesis_scythe"))],
        "nemesis_burn"=>(0..3).map(|_|effect(actor.clone(),Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Burn,destination:super::super::super::CardDestination::Discard,upgrades:0})).collect(),
        "head_count"|"head_time"=>vec![damage(actor.clone(),FullIntentId::from_name(enemy.intent_id.as_str()))],
        "head_glare"=>vec![effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:1})],
        _=>return Err(InputError::Unsupported("Act3 elite intent unavailable".into())),
    };
    if enemy.enemy_id==ActorEnemyId::Full(EnemyId::Nemesis)&&!enemy.full_powers.iter().any(|p|p.id.0=="intangible"){work.push(effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("intangible"),amount:1}));}
    Ok(work)
}
pub(in crate::combat) fn refresh_actor(enemy:&mut EnemyState){
    if let Some(EnemyMemory::GiantHead{count})=enemy.full_memory{
        if let Some(entry)=enemy.full_damage_entries.get_mut(&FullIntentId::from_name("head_time")){entry.base=30-5*i32::from(count.min(0));}
    }
}
pub(in crate::combat) fn after_card(state:&mut State)->Result<(),InputError>{
    for enemy in state.enemies.iter_mut().filter(|e|e.hp>0&&e.life==super::super::super::ActorLife::Active){
        if let Some(power)=enemy.full_powers.iter_mut().find(|p|p.id.0=="slow"){
            power.state.amount=power.state.amount.checked_add(1).filter(|n|*n<=999).ok_or_else(||InputError::Unsupported("Slow amount exceeds native bounds".into()))?;
        }
    }Ok(())
}
