//! Qualified direct Ironclad card payloads; execution remains in the shared resolver.
use super::*;
use crate::{content::{CardId,CardType,Catalog},combat::{command::CardDestination,model::OwnedCard}};

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<super::super::PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use CardId::*;
    let upgraded=card.upgrades>0;
    let targeted=matches!(card.id,BodySlam|Carnage|Clash|Hemokinesis|Pummel|Uppercut|WildStrike);
    if !matches!(card.id,BodySlam|Carnage|Clash|Hemokinesis|Pummel|SwordBoomerang|Uppercut|WildStrike|Immolate){return Ok(None)}
    if targeted&&target.is_none(){return Err(crate::run::RunError::InvalidAction)}
    if matches!(card.id,Immolate|SwordBoomerang)&&target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    if card.id==Clash&&state.piles.hand.iter().filter_map(OwnedCard::full).any(|held|
        Catalog::standard().card(held.id).is_none_or(|definition|definition.card_type!=CardType::Attack)){
        return Err(crate::run::RunError::InvalidAction)
    }
    let mut effects=Vec::new();
    let cost=match card.id{
        BodySlam=>{let target=target.unwrap();let damage=super::super::modified_damage(state,target,state.player.block)?;
            effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage))));if upgraded{0}else{1}},
        Carnage=>{effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_damage(state,target.unwrap(),if upgraded{28}else{20})?))));2},
        Clash=>{effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_damage(state,target.unwrap(),if upgraded{18}else{14})?))));0},
        Hemokinesis=>{effects.push((Some(super::super::PayloadTarget::Player),EffectKind::Damage(DamagePayload::HpLoss(2))));effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_damage(state,target.unwrap(),if upgraded{20}else{15})?))));1},
        Pummel=>{let damage=super::super::modified_damage(state,target.unwrap(),2)?;for _ in 0..if upgraded{5}else{4}{effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage))))}1},
        SwordBoomerang=>{for _ in 0..if upgraded{4}else{3}{effects.push((None,EffectKind::Ironclad(IroncladEffect::SwordBoomerangHit{damage:3})));}1},
        Uppercut=>{effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_damage(state,target.unwrap(),13)?))));
            for id in ["weak","vulnerable"]{effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{id:FullPowerId(id),amount:if upgraded{2}else{1}}))}2},
        Immolate=>{
            for enemy in state.enemies.iter().filter(|e|e.hp>0){
                let damage=super::modified_damage_for_enemy(state,enemy,if upgraded{28}else{21})?;
                effects.push((Some(super::super::PayloadTarget::Enemy(enemy.instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            effects.push((Some(super::super::PayloadTarget::Player),EffectKind::CreateCard{id:Burn,destination:CardDestination::Discard,upgrades:0}));2
        },
        WildStrike=>{effects.push((Some(super::super::PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_damage(state,target.unwrap(),if upgraded{17}else{12})?))));effects.push((Some(super::super::PayloadTarget::Player),EffectKind::CreateCard{id:Wound,destination:CardDestination::ShuffledDraw,upgrades:0}));1},
        _=>unreachable!(),
    };
    Ok(Some((cost,effects)))
}
