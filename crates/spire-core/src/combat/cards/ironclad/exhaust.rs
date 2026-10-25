//! Ironclad exhaust cards and the finite callbacks on the existing exhaust hook.
use super::super::*;
use crate::content::CardId;

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<super::super::PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use CardId::*;
    let upgraded=card.upgrades>0;
    let result=match card.id{
        Corruption=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            (if upgraded{2}else{3},vec![(Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("corruption"),amount:1})])},
        DarkEmbrace=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            (if upgraded{1}else{2},vec![(Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("dark_embrace"),amount:1})])},
        FeelNoPain=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            (1,vec![(Some(super::super::PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("feel_no_pain"),amount:if upgraded{4}else{3}})])},
        Sentinel=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            (1,vec![(Some(super::super::PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_block(state,if upgraded{8}else{5})?))])},
        SecondWind=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            let block=super::super::modified_block(state,if upgraded{7}else{5})?;
            (1,vec![(None,EffectKind::ExhaustBatchStart{filter:super::super::model::ExhaustBatchFilter::NonAttack,random:false,finish:super::super::model::ExhaustBatchFinish::BlockPerExhaust(block)})])},
        SeverSoul=>{let Some(target_id)=target else{return Err(crate::run::RunError::InvalidAction)};
            let damage=super::super::modified_damage(state,target_id,if upgraded{22}else{16})?;
            let target=Actor::Enemy(state.enemies[(target_id-1) as usize].instance_id.clone());
            (2,vec![(None,EffectKind::ExhaustBatchStart{filter:super::super::model::ExhaustBatchFilter::NonAttack,random:false,
                finish:super::super::model::ExhaustBatchFinish::DamageAfterBatch{target,damage}})])},
        FiendFire=>{let Some(target_id)=target else{return Err(crate::run::RunError::InvalidAction)};
            let damage=super::super::modified_damage(state,target_id,if upgraded{10}else{7})?;
            let target=Actor::Enemy(state.enemies[(target_id-1) as usize].instance_id.clone());
            (2,vec![(None,EffectKind::ExhaustBatchStart{filter:super::super::model::ExhaustBatchFilter::AllHand,random:true,
                finish:super::super::model::ExhaustBatchFinish::DamagePerExhaust{target,damage}})])},
        _=>return Ok(None),
    };Ok(Some(result))
}

pub(in crate::combat::cards) fn sentinel_trigger(source:&Source,card:&LiveCard)->Effect{
    let energy=if card.upgrades>0{3}else{2};
    Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::GainEnergy(energy)}
}

pub(in crate::combat) fn per_use_disposition(state:&State,card:&LiveCard)->super::super::super::model::FinishDisposition{
    let corruption_active=corruption_skill_active(state);
    if corruption_active&&crate::content::Catalog::standard().card(card.id)
        .is_some_and(|definition|definition.card_type==crate::content::CardType::Skill){
        super::super::super::model::FinishDisposition::Exhaust
    }else{super::super::super::model::FinishDisposition::Natural}
}

pub(in crate::combat) fn corruption_skill_active(state:&State)->bool{
    state.player_full_powers.iter().any(|power|power.id.0=="corruption"&&power.state.amount>0)
}
