//! Actual Ironclad power-card application payloads.
use super::super::{EffectKind,FullPowerId};
use crate::combat::LiveCard;
use crate::{content::CardId,run::RunError};

pub(super) fn payload(card:&LiveCard,target:Option<u64>)->Result<Option<(i32,Vec<(Option<super::super::PayloadTarget>,EffectKind)>)>,RunError>{
    use super::super::PayloadTarget::{Player};
    if target.is_some(){return if matches!(card.id,CardId::Barricade|CardId::Berserk|CardId::Brutality|CardId::DemonForm|CardId::Metallicize|CardId::Combust){Err(RunError::InvalidAction)}else{Ok(None)}}
    let upgraded=card.upgrades>0;
    let (cost,effects)=match card.id{
        CardId::Barricade=>(if upgraded{2}else{3},vec![(Some(Player),EffectKind::ApplyFullPower{id:FullPowerId("barricade"),amount:1})]),
        CardId::Berserk=>{
            (0,vec![(Some(Player),EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:if upgraded{1}else{2}}),
                (Some(Player),EffectKind::ApplyFullPower{id:FullPowerId("berserk"),amount:1})])
        },
        CardId::Brutality=>(0,vec![(Some(Player),EffectKind::ApplyFullPower{id:FullPowerId("brutality"),amount:1})]),
        CardId::DemonForm=>(3,vec![(Some(Player),EffectKind::ApplyFullPower{id:FullPowerId("demon_form"),amount:if upgraded{3}else{2}})]),
        CardId::Metallicize=>(1,vec![(Some(Player),EffectKind::ApplyFullPower{id:FullPowerId("metallicize"),amount:if upgraded{4}else{3}})]),
        CardId::Combust=>(1,vec![(Some(Player),EffectKind::ApplyFullPower{id:FullPowerId("combust"),amount:if upgraded{7}else{5}})]),
        _=>return Ok(None),
    };
    Ok(Some((cost,effects)))
}
