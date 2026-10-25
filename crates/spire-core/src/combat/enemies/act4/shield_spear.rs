use super::super::*;
use crate::content::EnemyId;

pub(in crate::combat) fn face_toward(state:&mut State,target:&EnemyHandle){
    if !state.player_full_powers.iter().any(|p|p.id.0=="surrounded"&&p.state.amount>0){return}
    let Some(slot)=state.actor_slots.get(target).copied() else{return};
    if state.enemies.iter().any(|e|&e.instance_id==target&&matches!(e.enemy_id,ActorEnemyId::Full(EnemyId::SpireShield|EnemyId::SpireSpear))&&e.life==super::super::super::ActorLife::Active&&e.hp>0){
        state.facing=match slot{0=>Some(super::super::super::PlayerFacing::Left),1=>Some(super::super::super::PlayerFacing::Right),_=>state.facing};
    }
}

pub(in crate::combat) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult){
    let Actor::Enemy(dead)=&result.target else{return};
    if result.hp_after!=0||!state.enemies.iter().any(|e|e.instance_id.as_str()==dead.as_str()&&matches!(e.enemy_id,ActorEnemyId::Full(EnemyId::SpireShield|EnemyId::SpireSpear))){return}
    if !state.player_full_powers.iter().any(|p|p.id.0=="surrounded"&&p.state.amount>0){return}
    let Some(survivor)=state.enemies.iter().find(|e|e.instance_id.as_str()!=dead.as_str()&&e.life==super::super::super::ActorLife::Active&&e.hp>0&&matches!(e.enemy_id,ActorEnemyId::Full(EnemyId::SpireShield|EnemyId::SpireSpear))).map(|e|e.instance_id.clone())else{return};
    let Some(slot)=state.actor_slots.get(&survivor).copied()else{return};
    state.facing=match slot{0=>Some(super::super::super::PlayerFacing::Left),1=>Some(super::super::super::PlayerFacing::Right),_=>state.facing};
    state.pending.push_front(Effect{source:Source::None,target:Some(Actor::Enemy(survivor)),kind:EffectKind::RemoveFullPower(FullPowerId("back_attack"))});
    state.pending.push_front(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(FullPowerId("surrounded"))});
}

pub(super) fn select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    if !memory.valid_for(id)||stream.id().domain()!=rng::Domain::RunCombatEnemyAi{return Err(InputError::Invariant("Shield/Spear selection identity mismatch".into()).into())}
    let (selections,name,next_stream)=match (id,memory){
        (EnemyId::SpireShield,EnemyMemory::SpireShield{selections})=>{
            let (name,next)=match selections%3 {
                0=>{let (n,next)=rng::uniform_below(stream,2)?;(if n==0{"shield_bash"}else{"shield_fortify"},next)},
                1=>{let other=match history.last().map(|x|x.as_str()){Some("shield_bash")=>"shield_fortify",Some("shield_fortify")=>"shield_bash",_=>return Err(InputError::Invariant("Shield paired selection history mismatch".into()).into())};(other,stream.clone())},
                _=>("shield_smash",stream.clone()),
            };
            (selections.checked_add(1).ok_or_else(||InputError::Invariant("Shield selection counter overflow".into()))?,name,next)
        },
        (EnemyId::SpireSpear,EnemyMemory::SpireSpear{selections})=>{
            let (name,next)=match selections%3 {
                0 if history.last().map(|x|x.as_str())!=Some("spear_burn_strike")=>("spear_burn_strike",stream.clone()),
                0=>("spear_strength",stream.clone()),
                1=>("spear_skewer",stream.clone()),
                _=>{let (n,next)=rng::uniform_below(stream,2)?;(if n==0{"spear_burn_strike"}else{"spear_strength"},next)},
            };
            (selections.checked_add(1).ok_or_else(||InputError::Invariant("Spear selection counter overflow".into()))?,name,next)
        },
        _=>return Err(InputError::Invariant("Shield/Spear memory mismatch".into()).into()),
    };
    let memory=match id{EnemyId::SpireShield=>EnemyMemory::SpireShield{selections},_=>EnemyMemory::SpireSpear{selections}};
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory,stream:next_stream})
}

fn effect(source:Actor,target:Option<Actor>,kind:EffectKind)->Effect{Effect{source:Source::Actor(source),target,kind}}
fn damage(owner:&EnemyHandle,name:&'static str)->Effect{effect(Actor::Enemy(owner.clone()),Some(Actor::Player),EffectKind::Damage(DamagePayload::FullEnemyMove(FullIntentId::from_name(name))))}

pub(in crate::combat::enemies) fn move_work(state:&mut State,owner:&EnemyHandle)->Result<Vec<Effect>,super::super::super::EffectError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Shield/Spear owner absent".into()))?;
    let ActorEnemyId::Full(id)=enemy.enemy_id else{return Err(InputError::Invariant("Shield/Spear owner is not full".into()).into())};
    let name=enemy.intent_id.as_str();let actor=Actor::Enemy(owner.clone());
    let work=match (id,name){
        (EnemyId::SpireShield,"shield_bash")=>{
            if state.character.orb_slots.iter().all(Option::is_none){
                vec![damage(owner,name),effect(actor.clone(),Some(Actor::Player),EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:-1})]
            }else{
                vec![effect(actor.clone(),Some(actor.clone()),EffectKind::ShieldBashFocus)]
            }
        },
        (EnemyId::SpireShield,"shield_fortify")=>state.enemies.iter().map(|e|effect(actor.clone(),Some(Actor::Enemy(e.instance_id.clone())),EffectKind::GainBlock(30))).collect(),
        (EnemyId::SpireShield,"shield_smash")=>vec![damage(owner,name),effect(actor.clone(),Some(actor),EffectKind::GainBlock(enemy.full_damage_entries[&FullIntentId::from_name(name)].output))],
        (EnemyId::SpireSpear,"spear_burn_strike")=>vec![damage(owner,name),damage(owner,name),effect(actor.clone(),Some(Actor::Player),EffectKind::CreateCard{id:crate::content::CardId::Burn,destination:super::super::super::CardDestination::Discard,upgrades:0}),effect(actor.clone(),Some(Actor::Player),EffectKind::CreateCard{id:crate::content::CardId::Burn,destination:super::super::super::CardDestination::Discard,upgrades:0})],
        (EnemyId::SpireSpear,"spear_skewer")=>vec![damage(owner,name),damage(owner,name),damage(owner,name)],
        (EnemyId::SpireSpear,"spear_strength")=>state.enemies.iter().map(|e|effect(actor.clone(),Some(Actor::Enemy(e.instance_id.clone())),EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:2})).collect(),
        _=>return Err(InputError::Unsupported("Shield/Spear intent mismatch".into()).into()),
    };Ok(work)
}

pub(in crate::combat) fn apply_shield_bash_focus(state:&mut State,payload:&Effect)->Result<(),super::super::super::EffectError>{
    let Some(Actor::Enemy(owner))=payload.target.as_ref() else{return Err(InputError::Invariant("Shield Bash continuation owner absent".into()).into())};
    let index=state.enemies.iter().position(|enemy|&enemy.instance_id==owner&&enemy.enemy_id==ActorEnemyId::Full(EnemyId::SpireShield)&&enemy.intent_id.as_str()=="shield_bash")
        .ok_or_else(||InputError::Invariant("Shield Bash continuation actor/intent mismatch".into()))?;
    let stream=state.enemies[index].full_ai_rng.as_ref().ok_or_else(||InputError::Invariant("Shield Bash AI stream missing".into()))?.clone();
    // Publish the captured hit before the fallible coin, but do not dispatch it
    // until after the producer returns. A failed draw therefore retains this
    // attack behind the current producer without applying it or retrying it.
    state.pending.push_front(damage(owner,"shield_bash"));
    let(focus,next)=rng::bernoulli(&stream,1,2)?;
    state.enemies[index].full_ai_rng=Some(next);
    let source=payload.source.clone();
    state.pending.insert(1,if focus{Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Character(super::super::super::characters::CharacterEffect::Focus(-1))}}
        else{Effect{source,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:-1}}});
    Ok(())
}

pub(in crate::combat) fn valid_intent(id:EnemyId,name:&str)->bool{matches!((id,name),
    (EnemyId::SpireShield,"shield_bash"|"shield_fortify"|"shield_smash")|
    (EnemyId::SpireSpear,"spear_burn_strike"|"spear_skewer"|"spear_strength"))}

pub(in crate::combat) fn valid_history(id:EnemyId,history:&[FullIntentId])->bool{
    match id {
        EnemyId::SpireShield=>history.iter().enumerate().all(|(index,intent)|match index%3{
            0=>matches!(intent.as_str(),"shield_bash"|"shield_fortify"),
            1=>matches!((history[index-1].as_str(),intent.as_str()),("shield_bash","shield_fortify")|("shield_fortify","shield_bash")),
            _=>intent.as_str()=="shield_smash",
        }),
        EnemyId::SpireSpear=>history.iter().enumerate().all(|(index,intent)|match index%3{
            0 if index==0=>intent.as_str()=="spear_burn_strike",
            0=>intent.as_str()==if history[index-1].as_str()=="spear_burn_strike"{"spear_strength"}else{"spear_burn_strike"},
            1=>intent.as_str()=="spear_skewer",
            _=>matches!(intent.as_str(),"spear_burn_strike"|"spear_strength"),
        }),
        _=>false,
    }
}
