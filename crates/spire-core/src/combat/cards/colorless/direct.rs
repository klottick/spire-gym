//! Direct Colorless card rules; resource and effect ownership stays shared.
use super::*;

pub(super) fn requires_draw_history(id:crate::content::CardId)->bool{
    use crate::content::CardId::*;
    matches!(id,Blind|DarkShackles|DeepBreath|DramaticEntrance|Finesse|FlashOfSteel
        |GoodInstincts|Impatience|MindBlast|Panacea|PanicButton|SwiftStrike|Trip|MasterOfStrategy|HandOfGreed)
}

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use crate::content::CardId::*;
    if !matches!(card.id,Blind|DarkShackles|DeepBreath|DramaticEntrance|Finesse|FlashOfSteel
        |GoodInstincts|Impatience|MindBlast|Panacea|PanicButton|SwiftStrike|Trip|MasterOfStrategy|HandOfGreed){
        return Ok(None)
    }
    let up=card.upgrades>0;
    let selected=match card.id{
        DarkShackles|FlashOfSteel|MindBlast|SwiftStrike|HandOfGreed=>true,
        Blind|Trip=>!up,
        _=>false,
    };
    if selected!=target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    let mut effects=Vec::new();
    let cost=match card.id{
        Blind=>{
            if up {
                for enemy in state.enemies.iter().filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0){
                    effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                        EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}));
                }
            }else{
                effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}));
            }
            0
        }
        DarkShackles=>{
            let target_id=target.unwrap();
            let enemy=state.enemies.get((target_id-1) as usize).filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0)
                .ok_or(crate::run::RunError::InvalidAction)?;
            let amount=if up{15}else{9};
            if !enemy.full_powers.iter().any(|power|power.id.0=="artifact"&&power.state.amount>0){
                effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{
                    id:FullPowerId("shackled"),amount}));
            }
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{
                id:FullPowerId("strength"),amount:-amount}));
            0
        }
        DeepBreath=>{
            if !state.piles.discard.is_empty(){
                effects.push((None,EffectKind::Recycle));
                effects.push((None,EffectKind::ShuffleDraw));
            }
            effects.push((None,EffectKind::Draw(if up{2}else{1})));
            0
        }
        DramaticEntrance=>{
            for (index,enemy) in state.enemies.iter().enumerate().filter(|(_,enemy)|enemy.life==ActorLife::Active&&enemy.hp>0){
                let damage=super::super::modified_damage(state,index as u64+1,if up{12}else{8})?;
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            0
        }
        Finesse=>{
            effects.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_block(state,if up{4}else{2})?)));
            effects.push((None,EffectKind::Draw(1)));
            0
        }
        FlashOfSteel=>{
            let damage=super::super::modified_damage(state,target.unwrap(),if up{6}else{3})?;
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage))));
            effects.push((None,EffectKind::Draw(1)));
            0
        }
        GoodInstincts=>{
            effects.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_block(state,if up{9}else{6})?)));
            0
        }
        Impatience=>{
            effects.push((Some(PayloadTarget::Player),EffectKind::Colorless(ColorlessEffect::ImpatienceResolve{draw:if up{3}else{2}})));
            0
        }
        MindBlast=>{
            let base=card.local.base_damage.ok_or(crate::run::RunError::Unsupported("Mind Blast draw count was not captured on hand entry"))?;
            let damage=super::super::modified_damage(state,target.unwrap(),base)?;
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage))));
            if up{1}else{2}
        }
        Panacea=>{
            effects.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("artifact"),amount:if up{2}else{1}}));
            0
        }
        PanicButton=>{
            effects.push((Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_block(state,if up{40}else{30})?)));
            effects.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("no_block"),amount:2}));
            0
        }
        SwiftStrike=>{
            let damage=super::super::modified_damage(state,target.unwrap(),if up{10}else{7})?;
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage))));
            0
        }
        Trip=>{
            if up {
                for enemy in state.enemies.iter().filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0){
                    effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                        EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:2}));
                }
            }else{
                effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:2}));
            }
            0
        }
        HandOfGreed=>{
            let damage=super::super::modified_damage(state,target.unwrap(),if up{25}else{20})?;
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage))));
            2
        }
        MasterOfStrategy=>{
            effects.push((None,EffectKind::Draw(if up{4}else{3})));
            0
        }
        _=>unreachable!(),
    };
    Ok(Some((cost,effects)))
}

pub(super) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    use crate::content::CardId::*;
    let up=card.upgrades>0;
    let (damage,block,magic)=match card.id{
        Blind=>(None,None,Some(2)),
        DarkShackles=>(None,None,Some(if up{15}else{9})),
        DeepBreath=>(None,None,Some(if up{2}else{1})),
        DramaticEntrance=>(Some(if up{12}else{8}),None,None),
        Finesse=>(None,Some(if up{4}else{2}),Some(1)),
        FlashOfSteel=>(Some(if up{6}else{3}),None,Some(1)),
        GoodInstincts=>(None,Some(if up{9}else{6}),None),
        Impatience=>(None,None,Some(if up{3}else{2})),
        MindBlast=>(card.local.base_damage,None,None),
        Panacea=>(None,None,Some(if up{2}else{1})),
        PanicButton=>(None,Some(if up{40}else{30}),Some(2)),
        SwiftStrike=>(Some(if up{10}else{7}),None,None),
        Trip=>(None,None,Some(2)),
        MasterOfStrategy=>(None,None,Some(if up{4}else{3})),
        HandOfGreed=>(Some(if up{25}else{20}),None,Some(if up{25}else{20})),
        _=>return None,
    };
    Some(CardPublicValues{base_damage:damage,base_block:block,magic_number:magic,damage_bonus:None,
        cost_kind:CardCostKind::Fixed,free_play:card.local.free_play,
        exhaust_on_finish:card.local.exhaust_on_finish,ethereal:card.local.ethereal,
        innate:card.local.innate,temporary_cost:card.local.temporary_cost})
}
