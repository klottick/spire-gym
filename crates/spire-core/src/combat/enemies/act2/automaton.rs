//! A0 Automaton cycle and once-selected Orb Stasis on the single native queue.
use super::*;
use super::super::super::{EffectError,SpawnRequest,ActorLife};

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,super::super::SelectError>{
    let mut cursor=stream.clone();let (name,memory)=match memory {
        EnemyMemory::BronzeAutomaton{turns}=>{
            let mut next=*turns;let last=history.last().map(|m|m.as_str());
            let name=if history.is_empty(){"automaton_spawn"}else if next==4{next=0;"automaton_beam"}else if last==Some("automaton_beam"){"automaton_stun"}else{next+=1;if matches!(last,Some("automaton_spawn"|"automaton_stun"|"automaton_boost")){"automaton_flail"}else{"automaton_boost"}};
            (name,EnemyMemory::BronzeAutomaton{turns:next})
        },
        EnemyMemory::BronzeOrb{used_stasis}=>{
            let (ticket,next)=rng::uniform_below(&cursor,100)?;cursor=next;
            let name=if !used_stasis&&ticket>=25{"orb_stasis"}else if ticket>=70&&!history.ends_with(&[FullIntentId::from_name("orb_support");2]){"orb_support"}else if !history.ends_with(&[FullIntentId::from_name("orb_beam");2]){"orb_beam"}else{"orb_support"};
            (name,EnemyMemory::BronzeOrb{used_stasis:*used_stasis||name=="orb_stasis"})
        },_=>return Err(InputError::Invariant("Automaton/Orb memory mismatch".into()).into()),
    };Ok(MoveSelection{intent:FullIntentId::from_name(name),memory,stream:cursor})
}

pub(in crate::combat) fn move_work(state:&State,owner:&EnemyHandle)->Result<Vec<Effect>,EffectError>{
    let enemy=state.enemies.iter().find(|e|&e.instance_id==owner).ok_or_else(||InputError::Invariant("Automaton turn owner absent".into()))?;
    let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("Automaton full intent absent".into()).into())};let actor=Actor::Enemy(owner.clone());
    Ok(match intent.as_str(){
        "automaton_spawn"=>[0,2].into_iter().map(|slot|Effect{source:Source::Actor(actor.clone()),target:None,kind:EffectKind::SpawnEnemy(SpawnRequest{summoner:owner.clone(),slot,species:EnemyId::BronzeOrb})}).collect(),
        "automaton_flail"=>vec![damage(actor.clone(),intent),damage(actor,intent)],
        "automaton_boost"=>vec![effect(actor.clone(),actor.clone(),EffectKind::GainBlock(9)),effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3})],
        "automaton_beam"|"orb_beam"=>vec![damage(actor,intent)],"automaton_stun"=>vec![],
        "orb_support"=>{let boss=state.enemies.iter().find(|e|e.enemy_id==ActorEnemyId::Full(EnemyId::BronzeAutomaton)&&e.life==ActorLife::Active&&e.hp>0).ok_or_else(||InputError::Unsupported("Orb Support has no live Automaton".into()))?;vec![effect(actor,Actor::Enemy(boss.instance_id.clone()),EffectKind::GainBlock(12))]},
        "orb_stasis"=>vec![effect(actor.clone(),actor,EffectKind::CaptureStasis{orb:owner.clone()})],
        _=>return Err(InputError::Invariant("Automaton/Orb intent mismatch".into()).into()),
    })
}

pub(super) fn after_damage(state:&mut State,result:&super::super::super::model::DamageResult){
    if let Actor::Enemy(owner)=&result.target {if result.hp_after==0&&result.hp_lost>0&&state.enemies.iter().any(|e|&e.instance_id==owner&&e.enemy_id==ActorEnemyId::Full(EnemyId::BronzeAutomaton)){scripted_cleanup(state,owner,EffectKind::AutomatonSuicide);}}
}
