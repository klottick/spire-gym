//! Source-qualified ordinary Act3 actors on the shared native queue.
use super::*;

pub(in crate::combat::enemies) fn supports(id:EnemyId)->bool{matches!(id,EnemyId::SpireGrowth|EnemyId::Transient|EnemyId::Maw)}
pub(in crate::combat::enemies) fn select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,constricted:Option<bool>)->Result<MoveSelection,SelectError>{
    let mut cursor=stream.clone();let last=history.last().map(|m|m.as_str());let mut memory=memory.clone();
    let name=match &mut memory {
        EnemyMemory::SpireGrowth=>{
            let constricted=constricted.ok_or_else(||InputError::Unsupported("Growth selection needs actual player power context".into()))?;
            let (n,next)=rng::uniform_below(&cursor,100)?;cursor=next;
            let twice=|name|history.len()>=2&&history[history.len()-2..].iter().all(|m|m.as_str()==name);
            if n<50&&!twice("growth_quick"){"growth_quick"}else if !constricted&&last!=Some("growth_constrict"){"growth_constrict"}else if !twice("growth_smash"){"growth_smash"}else{"growth_quick"}
        },
        EnemyMemory::Transient{..}=>"transient_attack",
        EnemyMemory::Maw{turns,roared}=>{
            *turns=turns.checked_add(1).filter(|n|*n<i32::MAX as u32).ok_or_else(||InputError::Unsupported("Maw selected-turn counter exhausted".into()))?;
            if !*roared{"maw_roar"}else{
                let (n,next)=rng::uniform_below(&cursor,100)?;cursor=next;
                if n<50&&last!=Some("maw_nom"){"maw_nom"}else if matches!(last,Some("maw_nom"|"maw_slam")){"maw_drool"}else{"maw_slam"}
            }
        },
        _=>return Err(InputError::Invariant("Act3 ordinary memory absent".into()).into()),
    };
    if !memory.valid_for(id){return Err(InputError::Invariant("Act3 ordinary memory/species mismatch".into()).into())}
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory,stream:cursor})
}

pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());
    let own=|kind|effect(actor.clone(),actor.clone(),kind);
    let intent=FullIntentId::from_name(match enemy.intent_id{super::super::super::ActorIntentId::Full(i)=>i.as_str(),_=>return Err(InputError::Invariant("full ordinary intent absent".into()))});
    Ok(match intent.as_str(){
        "growth_quick"|"growth_smash"|"maw_slam"=>vec![damage(actor,intent)],
        "growth_constrict"=>vec![effect(actor,Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("constricted"),amount:10})],
        "transient_attack"=>{
            let Some(EnemyMemory::Transient{turns,..})=enemy.full_memory else{return Err(InputError::Invariant("Transient counter absent".into()))};
            vec![damage(actor.clone(),intent),own(if turns==4{EffectKind::TransientFade}else{EffectKind::ReduceFullPower{id:FullPowerId("fading"),amount:1}}),own(EffectKind::TransientRecover)]
        },
        "maw_roar"=>["weak","frail"].into_iter().map(|id|effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId(id),amount:3})).collect(),
        "maw_drool"=>vec![own(EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3})],
        "maw_nom"=>{let Some(EnemyMemory::Maw{turns,..})=enemy.full_memory else{return Err(InputError::Invariant("Maw turn counter absent".into()))};vec![own(EffectKind::MawNom{remaining:(turns+1)/2})]},
        _=>return Err(InputError::Unsupported("Act3 ordinary intent unavailable".into())),
    })
}

pub(in crate::combat::enemies) fn take_turn(enemy:&mut EnemyState)->Result<Vec<Effect>,InputError>{
    let work=move_work(enemy)?;
    match &mut enemy.full_memory{
        Some(EnemyMemory::Transient{turns,..})=>*turns=turns.checked_add(1).filter(|t|*t<=5).ok_or_else(||InputError::Invariant("expired Transient turn".into()))?,
        Some(EnemyMemory::Maw{roared,..}) if enemy.intent_id.as_str()=="maw_roar"=>*roared=true,_=>{},
    }Ok(work)
}

pub(in crate::combat) fn refresh_selected(state:&mut State,owner:&EnemyHandle){
    let Some(e)=state.enemies.iter_mut().find(|e|&e.instance_id==owner)else{return};
    if let Some(EnemyMemory::Transient{turns,base})=&mut e.full_memory{
        *base=30+10*i32::from(*turns);if let Some(entry)=e.full_damage_entries.get_mut(&FullIntentId::from_name("transient_attack")){entry.base=*base;}
    }
}

pub(in crate::combat) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult){
    let Actor::Enemy(owner)=&result.target else{return};
    let Some(e)=state.enemies.iter().find(|e|&e.instance_id==owner&&e.hp>0&&e.enemy_id==ActorEnemyId::Full(EnemyId::Transient)&&e.full_powers.iter().any(|p|p.id.0=="shifting"))else{return};
    // onAttacked receives post-Block damage before the original HP clamp, not
    // attempted damage. Living ordinary cases equal actual HP lost.
    let amount=result.attempted-result.blocked;if amount<=0{return}
    let artifact=e.full_powers.iter().any(|p|p.id.0=="artifact"&&p.state.amount>0);let actor=Actor::Enemy(owner.clone());
    let mut work=vec![];if !artifact{work.push(effect(actor.clone(),actor.clone(),EffectKind::ApplyFullPower{id:FullPowerId("shackled"),amount}));}
    work.push(effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:-amount}));
    for e in work.into_iter().rev(){state.pending.push_front(e);}
}

pub(in crate::combat) fn player_end_work(state:&State)->Vec<Effect>{
    let Some(power)=state.player_full_powers.iter().find(|p|p.id.0=="constricted"&&p.state.amount>0)else{return vec![]};
    // The admitted Growth encounter has exactly one inflictor. No arbitrary
    // mixed-source or debug-applied Constricted support is inferred.
    let Some(owner)=state.enemies.iter().find(|e|e.enemy_id==ActorEnemyId::Full(EnemyId::SpireGrowth))else{return vec![]};
    vec![effect(Actor::Enemy(owner.instance_id.clone()),Actor::Player,EffectKind::Damage(DamagePayload::Thorns(power.state.amount)))]
}

pub(in crate::combat) fn apply(state:&mut State,effect:&Effect)->Result<(),InputError>{
    let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("ordinary continuation owner absent".into()))};
    let actor=Actor::Enemy(owner.clone());
    match effect.kind{
        EffectKind::TransientFade=>act2::suicide(state,owner),
        EffectKind::TransientRecover=>{
            let amount=state.enemies.iter().find(|e|&e.instance_id==owner).and_then(|e|e.full_powers.iter().find(|p|p.id.0=="shackled")).map_or(0,|p|p.state.amount);
            if amount>0{state.pending.push_front(super::super::effect(actor.clone(),actor.clone(),EffectKind::RemoveFullPower(FullPowerId("shackled"))));state.pending.push_front(super::super::effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount}));}
        },
        EffectKind::MawNom{remaining}=>{
            if remaining>1{state.pending.push_front(super::super::effect(actor.clone(),actor.clone(),EffectKind::MawNom{remaining:remaining-1}));}
            state.pending.push_front(damage(actor,FullIntentId::from_name("maw_nom")));
        },
        _=>return Err(InputError::Invariant("ordinary continuation mismatch".into())),
    }Ok(())
}
