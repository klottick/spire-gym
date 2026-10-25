//! Feed's synchronous Fatal checkpoint and Reaper's ordered source action.
use super::*;
use crate::content::CardId;
use crate::combat::{EffectError,InputError,model::{DamageResult,OwnedCard}};

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<super::super::PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    let up=card.upgrades>0;
    match card.id{
        CardId::Feed=>{
            let Some(target)=target else{return Err(crate::run::RunError::InvalidAction)};
            state.enemies.get((target-1) as usize).filter(|enemy|enemy.hp>0&&enemy.life==super::super::ActorLife::Active)
                .ok_or(crate::run::RunError::InvalidAction)?;
            let damage=super::modified_damage(state,target,if up{12}else{10})?;
            Ok(Some((1,vec![(Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage)))])))
        },
        CardId::Reaper=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            let mut targets=Vec::new();
            for enemy in state.enemies.iter().filter(|enemy|enemy.hp>0&&enemy.life==super::super::ActorLife::Active){
                let damage=super::modified_damage_for_enemy(state,enemy,if up{5}else{4})?;
                targets.push((enemy.instance_id.clone(),damage));
            }
            Ok(Some((2,vec![(None,EffectKind::Ironclad(IroncladEffect::ReaperResolve{targets,next:0,healed:0}))])))
        },
        _=>Ok(None),
    }
}

/// Called after the shared damage commit and lifecycle callbacks, before
/// retaliation is put into execution. Only an actual owned Feed can earn it.
pub(in crate::combat) fn after_damage(state:&mut State,result:&DamageResult)->Result<(),EffectError>{
    if !state.full_runtime||!result.normal||result.hp_after>0{return Ok(())}
    let Source::Card(source)=&result.source else{return Ok(())};
    if !state.in_play.as_ref().is_some_and(|owned|owned.handle()==*source&&owned.full().is_some_and(|card|card.id==CardId::Feed)){return Ok(())}
    let Actor::Enemy(target)=&result.target else{return Ok(())};
    let Some(enemy)=state.enemies.iter().find(|enemy|&enemy.instance_id==target)else{return Ok(())};
    if enemy.hp>0||enemy.life!=super::super::ActorLife::Active
        ||enemy.full_powers.iter().any(|power|power.id.0=="minion"){return Ok(())}
    let amount=if state.in_play.as_ref().and_then(OwnedCard::full).is_some_and(|card|card.upgrades>0){4}else{3};
    let max=state.player.max_hp.checked_add(amount).filter(|max|*max<=999)
        .ok_or_else(||InputError::Unsupported("Feed maximum HP ceiling exceeded".into()))?;
    state.player.max_hp=max;
    super::super::super::effects::ordinary_heal_now(state,result.source.clone(),amount)?;
    Ok(())
}

pub(super) fn apply(state:&mut State,effect:&Effect,work:&IroncladEffect)->Result<(),EffectError>{
    let IroncladEffect::ReaperResolve{targets,next,healed}=work else{return Err(InputError::Invariant("non-Reaper work reached fatal dispatcher".into()).into())};
    let Source::Card(source)=&effect.source else{return Err(InputError::Invariant("Reaper action lost its card owner".into()).into())};
    if !state.in_play.as_ref().is_some_and(|owned|owned.handle()==*source&&owned.full().is_some_and(|card|card.id==CardId::Reaper)){
        return Err(InputError::Invariant("Reaper action is not owned by the current card".into()).into())
    }
    let mut total=*healed;
    for index in *next..targets.len(){
        let (owner,damage)=&targets[index];
        // Retain the exact next target and prior actual HP loss before entering
        // the fallible shared damage resolver. A retained fault cannot retry it.
        retain_progress(state,targets,index,total)?;
        let Some(enemy)=state.enemies.iter().find(|enemy|&enemy.instance_id==owner&&enemy.life==super::super::ActorLife::Active&&enemy.hp>0)else{
            retain_progress(state,targets,index+1,total)?;continue
        };
        if enemy.life!=super::super::ActorLife::Active||enemy.hp<=0{retain_progress(state,targets,index+1,total)?;continue}
        let damage_effect=Effect{source:effect.source.clone(),target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(*damage))};
        let (callback,receipt)=super::super::super::effects::apply_effect_with_receipt(state,&damage_effect);
        let result=match (callback,receipt){
            (Ok(()),Some(result))=>result,
            (Ok(()),None)=>return Err(InputError::Invariant("Reaper shared hit omitted its damage result".into()).into()),
            (Err(error),Some(result))=>{
                total=total.checked_add(result.hp_lost).ok_or_else(||InputError::Unsupported("Reaper earned-heal accumulator exhausted".into()))?;
                // The current hit has committed. Advance past it before
                // retaining the callback fault so a retry cannot damage twice.
                retain_progress(state,targets,index+1,total)?;
                return Err(error);
            }
            (Err(error),None)=>return Err(error),
        };
        let removed=result.hp_lost;
        total=total.checked_add(removed).ok_or_else(||InputError::Unsupported("Reaper earned-heal accumulator exhausted".into()))?;
        retain_progress(state,targets,index+1,total)?;
    }
    if total>0{
        let heal=Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Heal(total)};
        let index=state.pending.iter().position(|queued|matches!(queued.kind,EffectKind::Hook(super::super::Hook::CombatSettled)|EffectKind::Terminal(_)|EffectKind::Decision))
            .unwrap_or(state.pending.len());
        state.pending.insert(index,heal);
    }
    Ok(())
}

fn retain_progress(state:&mut State,targets:&[(super::super::EnemyHandle,i32)],next:usize,healed:i32)->Result<(),EffectError>{
    let Some(current)=state.current.as_mut()else{return Err(InputError::Invariant("Reaper progress has no retained current effect".into()).into())};
    current.kind=EffectKind::Ironclad(IroncladEffect::ReaperResolve{targets:targets.to_vec(),next,healed});
    Ok(())
}
