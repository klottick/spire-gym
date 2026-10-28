//! Paired direct green rules; payment, hits and disposition use shared owners.
use super::*;

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    use crate::content::CardId::*;
    let up = card.upgrades > 0;
    if !matches!(card.id, DaggerSpray|Slice|SuckerPunch|Backstab|Dash|LegSweep|RiddleWithHoles|Adrenaline|DieDieDie) {
        return Ok(None);
    }
    let selected = matches!(card.id, Slice|SuckerPunch|Backstab|Dash|LegSweep|RiddleWithHoles);
    if selected != target.is_some() { return Err(crate::run::RunError::InvalidAction); }
    let mut effects = Vec::new();
    let cost = match card.id {
        Adrenaline => {
            effects.push((Some(PayloadTarget::Player), EffectKind::GainEnergy(if up {2} else {1})));
            effects.push((None, EffectKind::Draw(2)));
            0
        }
        LegSweep => {
            effects.push((Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower {id:FullPowerId("weak"),amount:if up{3}else{2}}));
            effects.push((Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state,if up{14}else{11})?)));
            2
        }
        DaggerSpray | DieDieDie => {
            let (damage,hits) = if card.id==DaggerSpray {(if up{6}else{4},2)} else {(if up{17}else{13},1)};
            for _ in 0..hits {
                for (index,enemy) in state.enemies.iter().enumerate().filter(|(_,enemy)|enemy.hp>0) {
                    effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())), EffectKind::Damage(DamagePayload::Fixed(
                        super::super::modified_damage(state,index as u64+1,damage)?))));
                }
            }
            1
        }
        _ => {
            let (damage,hits,cost) = match card.id {
                Slice => (if up{9}else{6},1,0),
                SuckerPunch => (if up{9}else{7},1,1),
                Backstab => (if up{15}else{11},1,0),
                Dash => (if up{13}else{10},1,2),
                RiddleWithHoles => (if up{4}else{3},5,2),
                _ => unreachable!(),
            };
            if card.id==Dash {
                effects.push((Some(PayloadTarget::Player),EffectKind::GainBlock(modified_block(state,if up{13}else{10})?)));
            }
            let damage=super::super::modified_damage(state,target.unwrap(),damage)?;
            for _ in 0..hits { effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage)))); }
            if card.id==SuckerPunch {
                effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:if up{2}else{1}}));
            }
            cost
        }
    };
    Ok(Some((cost,effects)))
}
