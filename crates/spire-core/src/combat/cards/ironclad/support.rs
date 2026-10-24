//! Finite Ironclad support-card payloads and their two action-time conditions.
use super::*;
use crate::{combat::{command::CardDestination,EffectError,InputError},content::CardId};

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<super::super::PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use CardId::*;
    if !matches!(card.id,Entrench|GhostlyArmor|Intimidate|PowerThrough|RecklessCharge|SeeingRed|Shockwave|SpotWeakness|TrueGrit|Headbutt|Warcry|BattleTrance|Exhume|DualWield){return Ok(None)}
    let upgraded=card.upgrades>0;
    let targeted=matches!(card.id,RecklessCharge|SpotWeakness|Headbutt);
    if targeted&&target.is_none()||!targeted&&target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    let mut effects=Vec::new();
    let cost=match card.id{
        Entrench=>{effects.push((Some(super::super::PayloadTarget::Player),EffectKind::Ironclad(IroncladEffect::DoubleCurrentBlock)));if upgraded{1}else{2}},
        GhostlyArmor=>{effects.push((Some(super::super::PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,if upgraded{13}else{10})?)));1},
        Intimidate=>{for enemy in state.enemies.iter().filter(|enemy|enemy.life==super::super::super::ActorLife::Active&&enemy.hp>0){
            effects.push((Some(super::super::PayloadTarget::Enemy(enemy.instance_id.clone())),EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:if upgraded{2}else{1}}));
        }0},
        PowerThrough=>{
            for _ in 0..2{effects.push((Some(super::super::PayloadTarget::Player),EffectKind::CreateCard{id:Wound,destination:CardDestination::Hand,upgrades:0}));}
            effects.push((Some(super::super::PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,if upgraded{20}else{15})?)));1
        },
        RecklessCharge=>{let damage=super::modified_damage(state,target.unwrap(),if upgraded{10}else{7})?;
            effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage))));
            effects.push((Some(super::super::PayloadTarget::Player),EffectKind::CreateCard{id:Dazed,destination:CardDestination::ShuffledDraw,upgrades:0}));0
        },
        SeeingRed=>{effects.push((Some(super::super::PayloadTarget::Player),EffectKind::GainEnergy(2)));if upgraded{0}else{1}},
        Shockwave=>{for enemy in state.enemies.iter().filter(|enemy|enemy.life==super::super::super::ActorLife::Active&&enemy.hp>0){
            for id in ["weak","vulnerable"]{effects.push((Some(super::super::PayloadTarget::Enemy(enemy.instance_id.clone())),EffectKind::ApplyFullPower{id:FullPowerId(id),amount:if upgraded{5}else{3}}));}
        }2},
        SpotWeakness=>{effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Ironclad(IroncladEffect::SpotWeakness{amount:if upgraded{4}else{3}})));1},
        TrueGrit=>{effects.push((Some(super::super::PayloadTarget::Player),EffectKind::GainBlock(super::modified_block(state,if upgraded{9}else{7})?)));
            if upgraded{effects.push((None,EffectKind::Ironclad(IroncladEffect::ChooseTrueGritCard)));}
            else{effects.push((None,EffectKind::Ironclad(IroncladEffect::TrueGritRandomExhaust)));}1},
        Headbutt=>{effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::modified_damage(state,target.unwrap(),if upgraded{12}else{9})?))));
            effects.push((None,EffectKind::Ironclad(IroncladEffect::ChooseHeadbuttCard)));1},
        Warcry=>{effects.push((Some(super::super::PayloadTarget::Player),EffectKind::Draw(if upgraded{2}else{1})));
            effects.push((None,EffectKind::SelectHandCardsAfterDraw{required:1,disposition:super::SelectionDisposition::DrawTop,empty:super::EmptySelection::Continue}));0},
        BattleTrance=>{effects.push((Some(super::super::PayloadTarget::Player),EffectKind::Draw(if upgraded{4}else{3})));
            effects.push((Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("no_draw"),amount:1}));0},
        Exhume=>{effects.push((None,EffectKind::Ironclad(IroncladEffect::ChooseExhumeCard)));if upgraded{0}else{1}},
        DualWield=>{effects.push((None,EffectKind::Ironclad(IroncladEffect::ChooseDualWieldCard)));1},
        _=>unreachable!(),
    };
    Ok(Some((cost,effects)))
}

pub(super) fn apply(state:&mut State,effect:&Effect,work:&IroncladEffect)->Result<(),EffectError>{
    let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("Ironclad support parent is not a card".into()).into())};
    if !state.full_runtime||state.in_play.as_ref().map(|card|card.handle()).as_ref()!=Some(parent){
        return Err(InputError::Invariant("Ironclad support action lost its in-play parent".into()).into())
    }
    match work{
        IroncladEffect::DoubleCurrentBlock=>{
            let block=state.player.block;
            state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::GainBlock(block)});
        },
        IroncladEffect::SpotWeakness{amount}=>{
            let Some(Actor::Enemy(target))=effect.target.as_ref() else{return Err(InputError::Invariant("Spot Weakness target is not an enemy".into()).into())};
            let enemy=state.enemies.iter().find(|enemy|&enemy.instance_id==target)
                .ok_or_else(||InputError::Invariant("Spot Weakness target disappeared".into()))?;
            let intent_base=match &enemy.intent_id{
                ActorIntentId::Legacy(intent)=>enemy.damage_entries.get(intent).map(|entry|entry.base),
                ActorIntentId::Full(intent)=>enemy.full_damage_entries.get(intent).map(|entry|entry.base),
            };
            if intent_base.is_some_and(|base|base>=0){
                state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:*amount}});
            }
        },
        IroncladEffect::TrueGritRandomExhaust=>{
            let candidates=state.piles.hand.iter().filter(|card|card.handle()!=*parent).map(|card|card.handle()).collect::<Vec<_>>();
            if candidates.len()==1{state.pending.push_front(Effect{source:effect.source.clone(),target:None,kind:EffectKind::ExhaustHandCard{instance:crate::combat::CardInstanceId(candidates[0].as_str().parse().map_err(|_|InputError::Invariant("True Grit candidate handle invalid".into()))?)}});}
            else if candidates.len()>1{let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("True Grit card stream absent".into()))?;
                let(index,next)=super::super::super::rng::uniform_below(stream,candidates.len() as u128)?;state.card_rng=Some(next);
                let id=candidates[index as usize].as_str().parse().map_err(|_|InputError::Invariant("True Grit candidate handle invalid".into()))?;
                state.pending.push_front(Effect{source:effect.source.clone(),target:None,kind:EffectKind::ExhaustHandCard{instance:crate::combat::CardInstanceId(id)}});}
        },
        _=>return Err(InputError::Invariant("non-support Ironclad action reached support dispatcher".into()).into()),
    }
    Ok(())
}
