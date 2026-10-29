//! Action-time green counts, draws and source-specific discard transactions.
use super::*;
use super::state::SilentEffect;
use crate::content::{CardId, CardType, Catalog};
use crate::combat::{model::OwnedCard, SelectionDisposition};

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use CardId::*;
    if !matches!(card.id,Acrobatics|Prepared|SneakyStrike|AllOutAttack|CalculatedGamble|Concentrate|EscapePlan|Expertise|Finisher|Flechettes|GrandFinale|Unload){return Ok(None)}
    let up=card.upgrades>0;
    let selected=matches!(card.id,SneakyStrike|Finisher|Flechettes|Unload);
    if selected!=target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    if card.id==GrandFinale&&!state.piles.draw.is_empty(){return Err(crate::run::RunError::InvalidAction)}
    let mut effects=Vec::new();
    let work=|kind| (None,EffectKind::Silent(kind));
    let cost=match card.id {
        Acrobatics|Prepared=>{
            let (draw,discard)=if card.id==Acrobatics{(if up{4}else{3},1)}else{(if up{2}else{1},if up{2}else{1})};
            effects.push((None,EffectKind::Draw(draw)));
            effects.push(work(SilentEffect::Discard{count:discard,random:false}));
            i32::from(card.id==Acrobatics)
        }
        Concentrate=>{
            effects.push(work(SilentEffect::Discard{count:if up{2}else{3},random:false}));
            effects.push((Some(PayloadTarget::Player),EffectKind::GainEnergy(2)));0
        }
        CalculatedGamble=>{effects.push(work(SilentEffect::Gamble));0}
        Expertise=>{effects.push(work(SilentEffect::Expertise{size:if up{7}else{6}}));1}
        EscapePlan=>{
            effects.push((None,EffectKind::Draw(1)));
            effects.push(work(SilentEffect::EscapePlanBlock{block:modified_block(state,if up{5}else{3})?}));0
        }
        Finisher|Flechettes=>{
            let damage=super::super::modified_damage(state,target.unwrap(),if card.id==Finisher{if up{8}else{6}}else{if up{6}else{4}})?;
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Silent(if card.id==Finisher{SilentEffect::AttackCountHits{damage}}else{SilentEffect::SkillCountHits{damage}})));1
        }
        SneakyStrike|Unload=>{
            let base=if card.id==SneakyStrike{if up{16}else{12}}else{if up{18}else{14}};
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_damage(state,target.unwrap(),base)?))));
            effects.push(work(if card.id==SneakyStrike{SilentEffect::SneakyEnergy}else{SilentEffect::Unload}));
            if card.id==SneakyStrike{2}else{1}
        }
        AllOutAttack|GrandFinale=>{
            let base=if card.id==AllOutAttack{if up{14}else{10}}else{if up{60}else{50}};
            for (index,enemy) in state.enemies.iter().enumerate().filter(|(_,e)|e.hp>0){
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_damage(state,index as u64+1,base)?))));
            }
            if card.id==AllOutAttack{effects.push(work(SilentEffect::Discard{count:1,random:true}));1}else{0}
        }
        _=>unreachable!(),
    };
    Ok(Some((cost,effects)))
}

pub(in crate::combat) fn apply(state:&mut State,effect:&Effect,work:&SilentEffect)->Result<(),EffectError>{
    super::state::validate_work(state,effect,work)?;
    let child=|kind|Effect{source:effect.source.clone(),target:effect.target.clone(),kind};
    match work {
        SilentEffect::BaneHit{..}|SilentEffect::FlaskStart{..}|SilentEffect::FlaskBounce{..}=>super::poison::apply_work(state,effect,work)?,
        SilentEffect::ToolsDiscard{count,turn}=>{
            let candidates=state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>();
            if candidates.len()<=*count {
                for _ in 0..candidates.len(){let card=state.piles.hand.remove(0);crate::combat::piles::discard_selected_card(state,card,true)?;}
            }else{crate::combat::effects::begin_selection_range(state,crate::combat::SelectionParent::Power{turn:*turn,id:FullPowerId("tools_of_the_trade")},crate::combat::SelectionSourceZone::Hand,candidates,*count,*count,SelectionDisposition::Discard,crate::combat::EmptySelection::Continue)?;}
        }
        SilentEffect::Setup|SilentEffect::BulletTime|SilentEffect::GlassKnifeAfterHits
        |SilentEffect::EndlessAgonyCopy{..}=>super::costs::apply(state,effect,work)?,
        SilentEffect::StormOfSteel{..}=>super::x_repeat::apply(state,effect,work)?,
        SilentEffect::BurstRepeat{..}=>crate::combat::cards::ironclad::autoplay::apply_burst_repeat(state,effect,work)?,
        SilentEffect::DrawContinue(count)=>crate::combat::piles::draw_action(state,effect,*count,false)?,
        SilentEffect::Discard{count,random}=>{
            let size=state.piles.hand.len();
            if size==0{return Ok(())}
            if !random&&size>*count{
                let Source::Card(parent)=&effect.source else{unreachable!()};
                crate::combat::effects::begin_selection(state,parent.clone(),state.piles.hand.iter().map(OwnedCard::handle).collect(),*count,SelectionDisposition::Discard,crate::combat::EmptySelection::Continue)?;
            }else{
                let mut remaining=(*count).min(size);
                while remaining>0 {
                    let index=if *random&&state.piles.hand.len()>1{
                        let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("discard random stream absent".into()))?;
                        let (index,next)=crate::combat::rng::uniform_below(stream,state.piles.hand.len() as u128)?;
                        state.card_rng=Some(next);index as usize
                    }else{0};
                    let card=state.piles.hand.remove(index);
                    crate::combat::piles::discard_selected_card(state,card,true)?;
                    remaining-=1;
                }
            }
        }
        SilentEffect::DiscardSpecific{card}=>{
            if let Some(index)=state.piles.hand.iter().position(|c|c.handle()==*card){
                let card=state.piles.hand.remove(index);
                crate::combat::piles::discard_selected_card_ordered(state,card,true,true)?;
            }
        }
        SilentEffect::Unload=>{
            let cards=state.piles.hand.iter().filter(|c|c.full().is_some_and(|c|Catalog::standard().card(c.id).is_some_and(|d|d.card_type!=CardType::Attack)))
                .map(OwnedCard::handle).collect::<Vec<_>>();
            for card in cards{state.pending.push_front(child(EffectKind::Silent(SilentEffect::DiscardSpecific{card})));}
        }
        SilentEffect::Gamble=>{
            let count=state.piles.hand.len();
            if count>0{
                state.pending.push_front(child(EffectKind::Draw(count as i32)));
                state.pending.push_front(child(EffectKind::Silent(SilentEffect::Discard{count,random:true})));
            }
        }
        SilentEffect::Expertise{size}=>{
            if state.piles.hand.len()<*size{state.pending.push_front(child(EffectKind::Draw((*size-state.piles.hand.len()) as i32)));}
        }
        SilentEffect::EscapePlanBlock{block}=>{
            if state.silent.drawn_cards.iter().any(|id|Catalog::standard().card(id.id).is_some_and(|c|c.card_type==CardType::Skill)){
                state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::GainBlock(*block)});
            }
        }
        SilentEffect::AttackCountHits{damage}|SilentEffect::SkillCountHits{damage}=>{
            let count=if matches!(work,SilentEffect::AttackCountHits{..}) {
                state.silent.played_this_turn.iter().filter(|id|Catalog::standard().card(**id).is_some_and(|c|c.card_type==CardType::Attack)).count().saturating_sub(1)
            }else{state.piles.hand.iter().filter_map(OwnedCard::full).filter(|c|Catalog::standard().card(c.id).is_some_and(|d|d.card_type==CardType::Skill)).count()};
            if let Some(Actor::Enemy(owner))=&effect.target{
                if state.enemies.iter().any(|e|e.instance_id==*owner&&e.hp>0){
                    for _ in 0..count{state.pending.push_front(child(EffectKind::Damage(DamagePayload::Fixed(*damage))));}
                }
            }
        }
        SilentEffect::SneakyEnergy=>{
            if state.silent.discarded_this_turn>0{state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::GainEnergy(2)});}
        }
    }
    Ok(())
}
