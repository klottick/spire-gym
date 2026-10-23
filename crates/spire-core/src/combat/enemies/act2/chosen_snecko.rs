//! Actual Chosen/Snecko turns and Hex's source-bottom generated-card work.
use super::*;

pub(in crate::combat::enemies) fn select_move(history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,super::super::SelectError>{
    let mut cursor=stream.clone();let name=match memory {
        EnemyMemory::Chosen=>match history.last().map(|id|id.as_str()){
            None=>"chosen_poke",Some("chosen_poke") if history.len()==1=>"chosen_hex",
            Some("chosen_hex"|"chosen_poke"|"chosen_zap")=>{let (ticket,next)=rng::uniform_below(&cursor,2)?;cursor=next;if ticket==0{"chosen_debilitate"}else{"chosen_drain"}},
            Some("chosen_debilitate"|"chosen_drain")=>{let (ticket,next)=rng::uniform_below(&cursor,5)?;cursor=next;if ticket<3{"chosen_poke"}else{"chosen_zap"}},
            _=>return Err(InputError::Invariant("invalid Chosen history".into()).into()),
        },
        EnemyMemory::Snecko=>if history.is_empty(){"snecko_glare"}else{
            let (ticket,next)=rng::uniform_below(&cursor,100)?;cursor=next;
            if ticket<40||history.ends_with(&[FullIntentId::from_name("snecko_bite");2]){"snecko_tail_whip"}else{"snecko_bite"}
        },
        _=>return Err(InputError::Invariant("Chosen/Snecko memory mismatch".into()).into()),
    };Ok(MoveSelection{intent:FullIntentId::from_name(name),memory:memory.clone(),stream:cursor})
}

pub(in crate::combat::enemies) fn move_work(enemy:&EnemyState)->Result<Vec<Effect>,InputError>{
    let actor=Actor::Enemy(enemy.instance_id.clone());let ActorIntentId::Full(intent)=enemy.intent_id else{return Err(InputError::Invariant("Chosen/Snecko full intent missing".into()))};
    let power=|id,amount|effect(actor.clone(),Actor::Player,EffectKind::ApplyFullPower{id:FullPowerId(id),amount});
    Ok(match intent.as_str(){
        "chosen_poke"=>vec![damage(actor.clone(),intent),damage(actor,intent)],
        "chosen_hex"=>vec![power("hex",1)],
        "chosen_debilitate"=>vec![damage(actor.clone(),intent),power("vulnerable",2)],
        "chosen_drain"=>vec![power("weak",3),effect(actor.clone(),actor,EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:3})],
        "chosen_zap"|"snecko_bite"=>vec![damage(actor,intent)],
        "snecko_glare"=>vec![power("confused",1)],
        "snecko_tail_whip"=>vec![damage(actor.clone(),intent),power("vulnerable",2)],
        _=>return Err(InputError::Invariant("invalid Chosen/Snecko move".into())),
    })
}

pub(in crate::combat) fn card_played(state:&mut State,source:&Source)->Result<(),InputError>{
    let Source::Card(owner)=source else{return Ok(())};
    let Some(card)=state.in_play.as_ref().filter(|c|c.handle()==*owner).and_then(|c|c.full())else{return Ok(())};
    if crate::content::Catalog::standard().card(card.id).is_some_and(|c|c.card_type==crate::content::CardType::Attack){return Ok(())}
    let amount=state.player_full_powers.iter().find(|p|p.id.0=="hex").map_or(0,|p|p.state.amount);
    for _ in 0..amount {bottom(state,Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::CreateCard{id:crate::content::CardId::Dazed,destination:super::super::super::CardDestination::ShuffledDraw,upgrades:0}});}
    Ok(())
}
