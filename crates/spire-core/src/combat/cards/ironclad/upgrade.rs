//! Armaments' hand-only child; no permanent deck or second pile owner.
use super::*;
use crate::combat::{EffectError,InputError,SelectionOperation};

pub(super) fn apply(state:&mut State,effect:&Effect,work:&IroncladEffect)->Result<(),EffectError> {
    let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("upgrade parent is not a card".into()).into())};
    if !state.full_runtime || state.in_play.as_ref().map(|c|c.handle()).as_ref()!=Some(parent){
        return Err(InputError::Invariant("upgrade parent is not current in-play owner".into()).into())
    }
    match work {
        IroncladEffect::UpgradeHand{all}=>{
            let candidates=state.piles.hand.iter().filter_map(|c|c.full())
                .filter(|c|rules::can_upgrade(c.id,c.upgrades)).map(|c|super::super::card_handle(c)).collect::<Vec<_>>();
            if *all || candidates.len()==1 {
                // Each committed upgrade has its own retained current effect;
                // an error cannot erase earlier upgraded instances.
                for handle in candidates.into_iter().rev(){
                    let instance=handle.as_str().parse().map_err(|_|InputError::Invariant("upgrade identity not numeric".into()))?;
                    state.pending.push_front(Effect{source:effect.source.clone(),target:None,
                        kind:EffectKind::Ironclad(IroncladEffect::UpgradeCard{instance:CardInstanceId(instance)})});
                }
            }else if candidates.len()>1{
                crate::combat::effects::begin_selection(state,parent.clone(),candidates,1,SelectionDisposition::Hand,EmptySelection::Continue)?;
                state.pending_selection.as_mut().unwrap().operation=SelectionOperation::Upgrade;
            }
        },
        IroncladEffect::UpgradeCard{instance}=>{
            let card=state.piles.hand.iter().filter_map(|c|c.full()).find(|c|c.instance==*instance).cloned()
                .ok_or_else(||InputError::Invariant("upgrade candidate left its captured hand".into()))?;
            let upgraded=crate::combat::card_state::upgrade_live_card(state,&card).map_err(|error|InputError::Unsupported(format!("owned card upgrade failed: {error:?}")))?;
            *state.piles.hand.iter_mut().filter_map(|c|c.full_mut()).find(|c|c.instance==*instance).unwrap()=upgraded;
        },
        IroncladEffect::SwordBoomerangHit{damage}=>{
            let candidates=state.enemies.iter().filter(|enemy|enemy.life==super::super::super::ActorLife::Active&&enemy.hp>0)
                .map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
            if candidates.is_empty(){return Ok(())}
            let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Sword Boomerang card-random stream absent".into()))?;
            let (index,next)=super::super::super::rng::uniform_below(stream,candidates.len() as u128)?;
            state.card_rng=Some(next);
            let target=candidates[index as usize].clone();
            let enemy=state.enemies.iter().find(|enemy|enemy.instance_id==target).unwrap();
            let amount=super::modified_damage_for_enemy(state,enemy,*damage)
                .map_err(|error|InputError::Invariant(format!("Sword Boomerang damage calculation failed: {error:?}")))?;
            state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Enemy(target)),kind:EffectKind::Damage(DamagePayload::Fixed(amount))});
        },
        IroncladEffect::DoubleCurrentBlock|IroncladEffect::SpotWeakness{..}|IroncladEffect::TrueGritRandomExhaust
        |IroncladEffect::ChooseTrueGritCard|IroncladEffect::ChooseHeadbuttCard|IroncladEffect::ChooseExhumeCard|IroncladEffect::ChooseDualWieldCard=>return Err(InputError::Invariant("support effect reached upgrade dispatcher".into()).into()),
        IroncladEffect::WhirlwindRound{..}|IroncladEffect::WhirlwindSpend|IroncladEffect::HavocTop{..}|IroncladEffect::HavocExtract{..}
        |IroncladEffect::DoubleTapRepeat{..}|IroncladEffect::HavocQueued{..}|IroncladEffect::FinishSuppressed=>return Err(InputError::Invariant("autoplay effect reached upgrade dispatcher".into()).into()),
        IroncladEffect::LimitBreakDispatch|IroncladEffect::DropkickDispatch{..}|IroncladEffect::RampageGrow{..}|IroncladEffect::ReaperResolve{..}|IroncladEffect::InfernalBladeGenerate=>return Err(InputError::Invariant("growth/fatal/copy effect reached upgrade dispatcher".into()).into()),
    }
    Ok(())
}
