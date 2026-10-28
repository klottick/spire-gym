//! Native direct payloads and narrowly owned continuations for Colorless cards.
use super::*;
use super::super::model::OwnedCard;

mod direct;
mod choices;
mod powers;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorlessEffect {
    ImpatienceResolve { draw: i32 },
    MayhemPlayTopCard,
    MayhemAutoplay { child:super::super::CardHandle,target:Option<super::super::Actor> },
    BombInstall { damage:i32 },
    BombTick { id:u64 },
    PanacheDamage { damage:i32 },
    ApotheosisUpgrade,
    EnlightenmentResolve,
    ForethoughtSelect,
    MadnessResolve,
    PuritySelect,
    SecretRetrieve { card_type: crate::content::CardType },
    ThinkingAheadSelect,
    ViolenceMove { count: u8 },
}

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    if let Some(payload)=powers::payload(state,card,target)?{return Ok(Some(payload))}
    if let Some(payload)=direct::payload(state,card,target)?{return Ok(Some(payload))}
    choices::payload(state,card,target)
}

pub(super) fn public_values(card:&LiveCard)->Option<CardPublicValues>{powers::public_values(card).or_else(||direct::public_values(card)).or_else(||choices::public_values(card))}

pub(super) fn upgrade_local(card:&mut LiveCard)->Result<(),crate::run::RunError>{
    let cost=match card.id {crate::content::CardId::Apotheosis|crate::content::CardId::MindBlast=>Some((2,1)),
        crate::content::CardId::Madness=>Some((1,0)),crate::content::CardId::Mayhem=>Some((2,1)),_=>None};
    if let Some((old,next))=cost {super::super::card_state::upgrade_base_cost(card,old,next)?;}
    if matches!(card.id,crate::content::CardId::SecretTechnique|crate::content::CardId::SecretWeapon|crate::content::CardId::ThinkingAhead){
        card.local.exhaust_on_finish=false;
    }
    Ok(())
}

pub(super) fn starting_cost(id:crate::content::CardId,upgrades:u32)->Option<i32>{powers::starting_cost(id,upgrades).or_else(||choices::starting_cost(id,upgrades))}

pub(super) fn requires_draw_history(id:crate::content::CardId)->bool{direct::requires_draw_history(id)||choices::requires_draw_history(id)}

pub(super) fn capture_draw_value(card:&mut LiveCard,draw_count:usize){
    if card.id==crate::content::CardId::MindBlast {
        card.local.base_damage=Some(i32::try_from(draw_count).expect("combat draw pile bounded by card registry"));
    }
}

pub(super) fn validate_work(state:&State,effect:&Effect,work:&ColorlessEffect)->Result<(),InputError>{
    if matches!(work,ColorlessEffect::MayhemPlayTopCard|ColorlessEffect::MayhemAutoplay{..}|ColorlessEffect::BombInstall{..}|ColorlessEffect::BombTick{..}|ColorlessEffect::PanacheDamage{..}){return powers::validate_work(state,effect,work)}
    if !state.full_runtime{return Err(InputError::Invariant("Colorless work requires native combat".into()))}
    let Source::Card(source)=&effect.source else{return Err(InputError::Invariant("Colorless work card source absent".into()))};
    match work {
        ColorlessEffect::ImpatienceResolve{draw}=>{
            let card=state.in_play.as_ref().filter(|card|card.handle()==*source).and_then(OwnedCard::full)
                .ok_or_else(||InputError::Invariant("Impatience continuation parent absent".into()))?;
            if card.id!=crate::content::CardId::Impatience||effect.target!=Some(Actor::Player)
                ||*draw!=if card.upgrades>0{3}else{2}{
                return Err(InputError::Invariant("Impatience continuation owner/count".into()))
            }
        }
        _=>return choices::validate_work(state,effect,work),
    }
    Ok(())
}

pub(super) fn apply(state:&mut State,effect:&Effect,work:&ColorlessEffect)->Result<(),EffectError>{
    if matches!(work,ColorlessEffect::MayhemPlayTopCard|ColorlessEffect::MayhemAutoplay{..}|ColorlessEffect::BombInstall{..}|ColorlessEffect::BombTick{..}|ColorlessEffect::PanacheDamage{..}){return powers::apply(state,effect,work)}
    match work {
        ColorlessEffect::ImpatienceResolve{draw}=>{
            let has_attack=state.piles.hand.iter().filter_map(OwnedCard::full).any(|card|
                crate::content::Catalog::standard().card(card.id)
                    .is_some_and(|definition|definition.card_type==crate::content::CardType::Attack));
            if !has_attack {
                state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Draw(*draw)});
            }
            Ok(())
        }
        _=>choices::apply(state,effect,work),
    }
}

pub(super) fn power_applied(state:&mut State,actor:&Actor,id:FullPowerId,application:bool,previous:i32){
    powers::power_applied(state,actor,id,application,previous)
}

pub(super) fn power_removed(state:&mut State,actor:&Actor,id:FullPowerId){powers::power_removed(state,actor,id)}

pub(super) fn card_played(state:&mut State,source:&Source)->Result<(),EffectError>{powers::card_played(state,source)}

pub(super) fn power_hook(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{
    powers::power_hook(state,hook,target,work)
}

pub(super) fn validate_power_application(state:&State,effect:&Effect,id:FullPowerId,amount:i32)->Result<(),InputError>{
    powers::validate_power_application(state,effect,id,amount)
}

pub(super) fn validate_state(state:&State)->Result<(),InputError>{powers::validate_state(state)}
