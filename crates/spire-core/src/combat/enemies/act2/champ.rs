//! Champ's phase latch and counters belong to the actual move-selection checkpoint.
use super::*;

pub(super) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState,context:&EnemyRollContext)->Result<MoveSelection,super::super::SelectError>{
    let EnemyMemory::Champ{turns,forges,angered}=memory else{return Err(InputError::Invariant("Champ memory absent".into()).into())};
    let owner=context.actors.iter().find(|a|a.owner==context.owner).ok_or_else(||InputError::Invariant("Champ live owner absent".into()))?;
    let mut turns=turns.checked_add(1).filter(|n|*n<=i32::MAX as u32).ok_or_else(||InputError::Unsupported("Champ turn counter exhausted".into()))?;
    let mut forges=*forges;let mut angered=*angered;let mut cursor=stream.clone();let last=history.last().map(|m|m.as_str());
    let name=if !angered&&owner.hp<owner.max_hp/2{angered=true;"champ_anger"}
        else if angered&&!history.iter().rev().take(2).any(|m|m.as_str()=="champ_execute"){"champ_execute"}
        else if !angered&&turns==4{turns=0;"champ_taunt"}
        else {
            let (ticket,next)=rng::uniform_below(&cursor,100)?;cursor=next;
            if ticket<=15&&last!=Some("champ_stance")&&forges<2{forges+=1;"champ_stance"}
            else if ticket<=30&&!matches!(last,Some("champ_stance"|"champ_gloat")){"champ_gloat"}
            else if ticket<=55&&last!=Some("champ_slap"){"champ_slap"}
            else if last==Some("champ_slash"){"champ_slap"}else{"champ_slash"}
        };
    Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:EnemyMemory::Champ{turns,forges,angered},stream:cursor})
}

pub(super) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("Champ full intent absent".into()))};
    let debuff=|id|effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId(id),amount:2});
    let own=|kind|effect(actor.clone(),actor.clone(),kind);
    Ok(match intent.as_str(){
        "champ_slash"=>vec![damage(actor,intent)],
        "champ_stance"=>vec![own(EffectKind::GainBlock(15)),own(EffectKind::ApplyFullPower{id:FullPowerId("metallicize"),amount:5})],
        "champ_execute"=>vec![damage(actor.clone(),intent),damage(actor,intent)],
        "champ_slap"=>vec![damage(actor.clone(),intent),debuff("frail"),debuff("vulnerable")],
        "champ_gloat"=>vec![own(EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:2})],
        "champ_taunt"=>vec![debuff("weak"),debuff("vulnerable")],
        "champ_anger"=>vec![own(EffectKind::ChampCleanse),own(EffectKind::RemoveFullPower(FullPowerId("shackled"))),own(EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:6})],
        _=>return Err(InputError::Invariant("Champ intent mismatch".into())),
    })
}

pub(in crate::combat) fn cleanse(state:&mut State,owner:&EnemyHandle)->Result<(),InputError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Champ cleanse owner absent".into()))?;
    // Classify the complete finite input before committing any removal. Unknown
    // positive IDs are not evidence of BUFF type, and must remain explicit faults.
    let mut removals=vec![];
    for power in &enemy.full_powers {
        let harmful=match power.id.0 {
            "weak"|"vulnerable"|"frail"|"hex"|"confused"|"entangled"|"poison"=>true,
            "strength"|"dexterity"=>power.state.amount<=0,
            "metallicize"|"artifact"|"ritual"|"enrage"|"regeneration"|"barricade"|"angry"|"flight"|"minion"|"painful_stabs"|"curl_up"|"split"|"mode_shift"|"sharp_hide"=>false,
            "shackled"=>false, // Its separately queued literal removal is unconditional, independent of category.
            id=>return Err(InputError::Unsupported(format!("Champ cleanse power category unavailable: {id}"))),
        };if harmful{removals.push(power.id)}
    }
    let actor=Actor::Enemy(owner.clone());for id in removals.into_iter().rev(){state.pending.push_front(effect(actor.clone(),actor.clone(),EffectKind::RemoveFullPower(id)));}Ok(())
}
