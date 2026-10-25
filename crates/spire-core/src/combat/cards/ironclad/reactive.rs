//! Ironclad draw/attack/HP-loss/Block reactive card payloads and shared callbacks.
use super::*;
use crate::{combat::{EffectError,model::DamageResult},content::{CardId,CardType,Catalog}};

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<super::super::PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use CardId::*;
    if !matches!(card.id,Evolve|FireBreathing|FlameBarrier|Juggernaut|Rage|Rupture){return Ok(None)}
    if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    let up=card.upgrades>0;
    let (cost,effects)=match card.id{
        Evolve=>(1,vec![(Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("evolve"),amount:if up{2}else{1}})]),
        FireBreathing=>(1,vec![(Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("fire_breathing"),amount:if up{10}else{6}})]),
        FlameBarrier=>(2,vec![(Some(super::super::PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,if up{16}else{12})?)),(Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("flame_barrier"),amount:if up{6}else{4}})]),
        Juggernaut=>(2,vec![(Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("juggernaut"),amount:if up{7}else{5}})]),
        Rage=>(0,vec![(Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("rage"),amount:if up{5}else{3}})]),
        Rupture=>(1,vec![(Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("rupture"),amount:if up{2}else{1}})]),
        _=>unreachable!(),
    };
    let _=state;
    Ok(Some((cost,effects)))
}

fn amount(state:&State,id:&str)->i32{state.player_full_powers.iter().filter(|power|power.id.0==id).map(|power|power.state.amount).sum()}

/// Run for one actual moved card. Callback-created actions go behind all work
/// already queued by the active DrawCardAction, matching addToBot ordering.
pub(in crate::combat) fn after_draw(state:&mut State,card:&LiveCard,owner:&CardHandle)->Result<(),EffectError>{
    if !state.full_runtime||state.player.hp<=0{return Ok(())}
    let typ=Catalog::standard().card(card.id).map(|definition|definition.card_type);
    let no_draw=state.player_full_powers.iter().any(|power|power.id.0=="no_draw"&&power.state.amount>0);
    let mut callbacks=Vec::new();
    for power in &state.player_full_powers {
        let reactive=match power.id.0 {
            "evolve" if typ==Some(CardType::Status)&&power.state.amount>0&&!no_draw=>
                Some(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(power.state.amount)}),
            "fire_breathing" if matches!(typ,Some(CardType::Status|CardType::Curse))&&power.state.amount>0=>None,
            _=>None,
        };
        if let Some(effect)=reactive{callbacks.push(effect);}
        if power.id.0=="fire_breathing"&&power.state.amount>0&&matches!(typ,Some(CardType::Status|CardType::Curse)){
        let targets=state.enemies.iter().filter(|enemy|enemy.life==super::super::super::ActorLife::Active&&enemy.hp>0).map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
            for target in targets{callbacks.push(Effect{source:Source::None,target:Some(Actor::Enemy(target)),kind:EffectKind::Damage(DamagePayload::Thorns(power.state.amount))});}
        }
    }
    for effect in callbacks{append_before_decision(state,effect);}
    let _=owner;
    Ok(())
}

fn append_before_decision(state:&mut State,effect:Effect){
    let index=state.pending.iter().position(|pending|pending.kind==EffectKind::Decision).unwrap_or(state.pending.len());
    state.pending.insert(index,effect);
}

pub(in crate::combat) fn card_played(state:&mut State,source:&Source)->Result<(),EffectError>{
    let Source::Card(handle)=source else{return Ok(())};
    let card=state.in_play.as_ref().filter(|owned|owned.handle()==*handle).and_then(crate::combat::model::OwnedCard::full);
    if card.is_some_and(|card|Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==CardType::Attack)){
        let block=amount(state,"rage");
        if block>0{
            // onUseCard follows the card's `use` enqueueing; place its work
            // after this card's payload, immediately before the finish owner.
            let insert=state.pending.iter().position(|effect|effect.kind==EffectKind::FinishCard&&effect.source==Source::Card(handle.clone())).unwrap_or(state.pending.len());
            state.pending.insert(insert,Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(block)});
        }
    }
    Ok(())
}

pub(in crate::combat) fn after_damage(state:&mut State,result:&DamageResult)->Result<(),EffectError>{
    if result.target==Actor::Player&&result.normal{
        if let Source::Actor(Actor::Enemy(attacker))=&result.source{
            let retaliation=amount(state,"flame_barrier");
            if retaliation>0&&state.enemies.iter().any(|enemy|&enemy.instance_id==attacker&&enemy.life==super::super::super::ActorLife::Active&&enemy.hp>0){
                state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(attacker.clone())),kind:EffectKind::Damage(DamagePayload::Thorns(retaliation))});
            }
        }
    }
    if result.target==Actor::Player&&result.hp_lost>0&&matches!(result.source,Source::Card(_)|Source::Potion{..}|Source::Actor(Actor::Player)){
        let strength=amount(state,"rupture");
        if strength>0{state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:strength}});}
    }
    Ok(())
}

pub(in crate::combat) fn after_block(state:&mut State,target:&Actor,notified:i32,_before:i32,_after:i32)->Result<(),EffectError>{
    if !state.full_runtime||target!=&Actor::Player||notified<=0{return Ok(())}
    let damage=amount(state,"juggernaut");if damage<=0{return Ok(())}
    append_before_decision(state,Effect{source:Source::Actor(Actor::Player),target:None,kind:EffectKind::JuggernautRandomDamage(damage)});
    Ok(())
}

pub(in crate::combat) fn turn_ended(state:&mut State,work:&mut Vec<Effect>){
    if state.player_full_powers.iter().any(|power|power.id.0=="rage"){
        work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(FullPowerId("rage"))});
    }
}

pub(in crate::combat) fn turn_started(state:&mut State,work:&mut Vec<Effect>){
    if state.player_full_powers.iter().any(|power|power.id.0=="flame_barrier"){
        work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(FullPowerId("flame_barrier"))});
    }
}
