//! A0 elite-local state; all payloads use the authoritative shared resolver.
use super::*;

pub(super) fn select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    let intent=match (id,memory) {
        (EnemyId::Sentry,EnemyMemory::Sentry{initial_beam})=>match history.last().map(|i|i.as_str()){
            None=>if *initial_beam{"beam"}else{"bolt"},Some("bolt")=>"beam",Some("beam")=>"bolt",
            _=>return Err(InputError::Invariant("invalid Sentry history".into()).into()),
        },
        (EnemyId::Lagavulin,EnemyMemory::Lagavulin{awake:false,..})=>"sleep",
        (EnemyId::Lagavulin,EnemyMemory::Lagavulin{attacks,..})=>{
            if history.is_empty()||*attacks>=2||history.ends_with(&[FullIntentId::Attack,FullIntentId::Attack]){"siphon"}else{"attack"}
        },
        _=>return Err(InputError::Invariant("elite identity/memory mismatch".into()).into()),
    };Ok(MoveSelection{intent:FullIntentId::from_name(intent),stream:stream.clone(),memory:memory.clone()})
}

pub(super) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());
    let work=match (&enemy.enemy_id,enemy.intent_id.as_str()){
        (ActorEnemyId::Full(EnemyId::Sentry),"bolt")=>(0..2).map(|_|effect(actor.clone(),Actor::Player,EffectKind::CreateCard{id:crate::content::CardId::Dazed,destination:super::super::CardDestination::Discard,upgrades:0})).collect(),
        (ActorEnemyId::Full(EnemyId::Sentry),"beam")=>vec![damage(actor,FullIntentId::from_name("beam"))],
        (ActorEnemyId::Full(EnemyId::Lagavulin),"sleep"|"stunned")=>vec![],
        (ActorEnemyId::Full(EnemyId::Lagavulin),"attack")=>vec![damage(actor,FullIntentId::Attack)],
        (ActorEnemyId::Full(EnemyId::Lagavulin),"siphon")=>vec![effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("dexterity"),amount:-1}),effect(actor,Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:-1})],
        _=>return Err(InputError::Invariant("invalid elite intent".into())),
    };Ok(work)
}

fn opening(owner:&EnemyHandle)->Effect {let actor=Actor::Enemy(owner.clone());effect(actor.clone(),actor,EffectKind::ReduceFullPower{id:FullPowerId("metallicize"),amount:8})}

pub(super) fn take_turn(enemy:&mut EnemyState)->Result<Vec<Effect>,InputError>{
    let mut work=move_work(enemy)?;
    let Some(EnemyMemory::Lagavulin{sleep_turns,awake,attacks})=enemy.full_memory.as_mut() else{return Err(InputError::Invariant("Lagavulin memory missing".into()))};
    match enemy.intent_id.as_str(){
        "sleep"=>{*sleep_turns+=1;if *sleep_turns==3{*awake=true;work.push(opening(&enemy.instance_id));}},
        "attack"=>*attacks=attacks.saturating_add(1).min(2),"siphon"=>*attacks=0,_=>{},
    }Ok(work)
}

pub(super) fn on_hit(state:&mut State,target:&EnemyHandle,hp_damage:i32){
    let Some(enemy)=state.enemies.iter_mut().find(|e|&e.instance_id==target) else{return};
    if hp_damage<=0||enemy.hp<=0{return}
    if let Some(EnemyMemory::Lagavulin{awake, ..})=enemy.full_memory.as_mut(){
        if !*awake {
            *awake=true;let intent=super::super::ActorIntentId::Full(FullIntentId::from_name("stunned"));
            enemy.intent_id=intent.clone();enemy.selected_move=intent.clone();enemy.move_history.push(intent);enemy.intent_damage=None;
            let checkpoint=state.pending.iter().position(|e|e.kind==EffectKind::Decision).unwrap_or(state.pending.len());
            state.pending.insert(checkpoint,opening(target));
        }
    }
}
