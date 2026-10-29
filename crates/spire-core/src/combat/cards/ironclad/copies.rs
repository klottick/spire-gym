//! Native generators whose result is sampled only when the effect executes.
use super::*;
use crate::combat::{EffectError,InputError,CardDestination,card_state,model::OwnedCard};

pub(super) fn payload(card:&LiveCard,target:Option<u64>)->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    if card.id!=crate::content::CardId::InfernalBlade{return Ok(None)}
    if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    Ok(Some((if card.upgrades>0{0}else{1},vec![(None,EffectKind::Ironclad(IroncladEffect::InfernalBladeGenerate))])))
}

pub(super) fn apply(state:&mut State,effect:&Effect)->Result<(),EffectError>{
    let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("Infernal Blade has no card source".into()).into())};
    let Some(OwnedCard::Full(in_play))=state.in_play.as_ref() else{return Err(InputError::Invariant("Infernal Blade source is not current in-play card".into()).into())};
    if state.in_play.as_ref().map(OwnedCard::handle).as_ref()!=Some(parent)
        ||in_play.id!=crate::content::CardId::InfernalBlade
        ||effect.target.is_some()
        ||!state.full_runtime
    {return Err(InputError::Invariant("Infernal Blade generation owner/target/runtime mismatch".into()).into())}
    let candidates=crate::run::rooms::m10_content::infernal_candidates();
    let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Infernal Blade card-random stream absent".into()))?;
    let (index,next)=super::super::super::rng::uniform_below(stream,candidates.len() as u128)?;
    state.card_rng=Some(next);
    let id=candidates[index as usize];
    let instance=super::super::super::piles::next_generated_instance(state)?;
    let mut generated=card_state::generated_with_history(id,instance,0,super::super::super::card_state::CardConstructionHistory::from_state(state))
        .ok_or_else(||InputError::Invariant("Infernal Blade selected unsupported generated card".into()))?;
    generated.origin=crate::combat::CombatCardRef::CombatLocal(instance.0);
    generated.copy_provenance=card_state::CopyProvenance::Generated;
    generated.copy_family=instance.0;
    // setCostForTurn(0) leaves Whirlwind's X marker/cost intact.
    if !generated.local.x_cost{
        let base=super::super::base_cost(state,&generated).ok_or_else(||InputError::Unsupported("Infernal Blade generated attack cost unavailable".into()))?;
        generated.local.temporary_cost=-base;
        generated.local.infernal_blade_cost_this_turn=true;
    }
    super::super::super::piles::insert_generated(state,generated,CardDestination::Hand)
}
