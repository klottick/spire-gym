//! Silent Poison card applications on the shared full-combat resolver.
use super::*;

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<super::super::PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    if let Some(result)=remaining_payload(state,card,target)?{return Ok(Some(result));}
    use crate::content::CardId;
    use super::super::PayloadTarget;

    if card.id == CardId::NoxiousFumes {
        if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
        return Ok(Some((1, vec![(Some(PayloadTarget::Player), EffectKind::ApplyFullPower {
            id: FullPowerId("noxious_fumes"), amount: if card.upgrades > 0 { 3 } else { 2 },
        })])));
    }

    let upgraded = card.upgrades > 0;
    let Some(target_id) = target else {
        return if matches!(card.id, CardId::DeadlyPoison | CardId::PoisonedStab | CardId::Catalyst) {
            Err(crate::run::RunError::InvalidAction)
        } else { Ok(None) };
    };
    let _enemy = state.enemies.get((target_id - 1) as usize)
        .ok_or(crate::run::RunError::InvalidAction)?;
    let result = match card.id {
        CardId::DeadlyPoison => (1, vec![(Some(PayloadTarget::SelectedEnemy),
            EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount: if upgraded { 7 } else { 5 } })]),
        CardId::PoisonedStab => {
            let damage = super::super::modified_damage(state, target_id, if upgraded { 8 } else { 6 })?;
            (1, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower {
                    id: FullPowerId("poison"), amount: if upgraded { 4 } else { 3 },
                }),
            ])
        }
        CardId::Catalyst => {
            (1, vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::Catalyst)])
        }
        _ => return Ok(None),
    };
    Ok(Some(result))
}

fn remaining_payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<super::super::PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use crate::content::CardId::*;
    use super::super::PayloadTarget;
    if !matches!(card.id,Bane|BouncingFlask|PiercingWail|CripplingCloud|HeelHook|Terror|CorpseExplosion){return Ok(None)}
    let up=card.upgrades>0;
    if matches!(card.id,Bane|HeelHook|Terror|CorpseExplosion)!=target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    let mut effects=Vec::new();
    let cost=match card.id{
        Bane=>{
            let damage=super::super::modified_damage(state,target.unwrap(),if up{10}else{7})?;
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage))));
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Silent(super::SilentEffect::BaneHit{damage})));1
        }
        BouncingFlask=>{effects.push((None,EffectKind::Silent(super::SilentEffect::FlaskStart{bounces:if up{4}else{3}})));2}
        PiercingWail=>{
            let amount=if up{8}else{6};
            for enemy in state.enemies.iter(){effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:-amount}));}
            for enemy in state.enemies.iter().filter(|enemy|!enemy.full_powers.iter().any(|p|p.id.0=="artifact"&&p.state.amount>0)){
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),EffectKind::ApplyFullPower{id:FullPowerId("shackled"),amount}));
            }1
        }
        CripplingCloud=>{
            for enemy in state.enemies.iter().filter(|e|e.life==ActorLife::Active&&e.hp>0){
                for (id,amount) in [("poison",if up{7}else{4}),("weak",2)]{
                    effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),EffectKind::ApplyFullPower{id:FullPowerId(id),amount}));
                }
            }2
        }
        HeelHook=>{
            let target=target.unwrap();let enemy=state.enemies.get((target-1) as usize).ok_or(crate::run::RunError::InvalidAction)?;
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_damage(state,target,if up{8}else{5})?))));
            if enemy.full_powers.iter().any(|p|p.id.0=="weak"&&p.state.amount>0){
                effects.push((Some(PayloadTarget::Player),EffectKind::GainEnergy(1)));effects.push((None,EffectKind::Draw(1)));
            }1
        }
        Terror=>{effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:99}));if up{0}else{1}}
        CorpseExplosion=>{
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{id:FullPowerId("poison"),amount:if up{9}else{6}}));
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{id:FullPowerId("corpse_explosion"),amount:1}));2
        }
        _=>unreachable!(),
    };
    Ok(Some((cost,effects)))
}

fn sample_target(state:&mut State)->Result<Option<EnemyHandle>,EffectError>{
    let living=state.enemies.iter().filter(|e|e.life==ActorLife::Active&&e.hp>0).map(|e|e.instance_id.clone()).collect::<Vec<_>>();
    if living.is_empty(){return Ok(None)}
    let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Flask stream absent".into()))?;
    let (index,next)=crate::combat::rng::uniform_below(stream,living.len() as u128)?;
    state.card_rng=Some(next);Ok(Some(living[index as usize].clone()))
}

pub(in crate::combat) fn capture_targets(state:&mut State,effects:&mut [Effect])->Result<(),EffectError>{
    for effect in effects{
        if let EffectKind::Silent(super::SilentEffect::FlaskStart{bounces})=effect.kind{
            let target=sample_target(state)?;
            effect.kind=EffectKind::Silent(super::SilentEffect::FlaskBounce{target,remaining:bounces});
        }
    }Ok(())
}

pub(in crate::combat) fn apply_work(state:&mut State,effect:&Effect,work:&super::SilentEffect)->Result<(),EffectError>{
    match work{
        super::SilentEffect::BaneHit{damage}=>{
            if let Some(Actor::Enemy(owner))=&effect.target{
                if state.player.hp>0&&state.enemies.iter().any(|e|e.instance_id==*owner&&e.hp>0&&e.full_powers.iter().any(|p|p.id.0=="poison"&&p.state.amount>0)){
                    state.pending.push_front(Effect{source:effect.source.clone(),target:effect.target.clone(),kind:EffectKind::Damage(DamagePayload::Fixed(*damage))});
                }
            }
        }
        super::SilentEffect::FlaskStart{..}=>return Err(InputError::Invariant("Flask use target was not captured".into()).into()),
        super::SilentEffect::FlaskBounce{target,remaining}=>{
            let Some(target)=target else{return Ok(())};
            if !state.enemies.iter().any(|e|e.instance_id==*target&&e.life==ActorLife::Active&&e.hp>0){return Ok(())}
            let next=if *remaining>1{sample_target(state)?}else{None};
            if *remaining>1{state.pending.push_front(Effect{source:effect.source.clone(),target:None,kind:EffectKind::Silent(super::SilentEffect::FlaskBounce{target:next,remaining:remaining-1})});}
            state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Enemy(target.clone())),kind:EffectKind::ApplyFullPower{id:FullPowerId("poison"),amount:3}});
        }
        _=>return Err(InputError::Invariant("not a green poison continuation".into()).into()),
    }Ok(())
}

/// Capture at the actual death/half-death callback, before phase cleanse.
pub(in crate::combat) fn death_work(state:&State,target:Option<&Actor>)->Result<Vec<Effect>,InputError>{
    let Some(Actor::Enemy(owner))=target else{return Ok(Vec::new())};
    let Some(enemy)=state.enemies.iter().find(|e|e.instance_id==*owner&&e.hp<=0)else{return Ok(Vec::new())};
    let stacks=enemy.full_powers.iter().find(|p|p.id.0=="corpse_explosion").map_or(0,|p|p.state.amount);
    if stacks<=0||crate::combat::enemies::act2::combat_victory_ready(state){return Ok(Vec::new())}
    let amount=enemy.max_hp.checked_mul(stacks).ok_or_else(||InputError::Unsupported("Corpse Explosion damage exhausted".into()))?;
    Ok(state.enemies.iter().filter(|e|e.life==ActorLife::Active&&e.hp>0).map(|target|Effect{
        source:Source::None,target:Some(Actor::Enemy(target.instance_id.clone())),
        kind:EffectKind::Damage(DamagePayload::CorpseExplosion{owner:owner.clone(),amount}),
    }).collect())
}

pub(super) fn apply_catalyst(state: &mut State, effect: &Effect) -> Result<(), super::super::EffectError> {
    let Some(Actor::Enemy(owner)) = effect.target.as_ref() else {
        return Err(InputError::Invariant("Catalyst action target absent".into()).into());
    };
    let upgraded = state.in_play.as_ref().and_then(|card| card.full())
        .is_some_and(|card| card.id == crate::content::CardId::Catalyst && card.upgrades > 0);
    let current = state.enemies.iter().find(|enemy| &enemy.instance_id == owner)
        .ok_or_else(|| InputError::Invariant("Catalyst target disappeared".into()))?
        .full_powers.iter().find(|power| power.id.0 == "poison")
        .map_or(0, |power| power.state.amount);
    if current <= 0 { return Ok(()); }
    let amount = if upgraded {
        current.checked_mul(2).ok_or_else(|| InputError::Unsupported(
            "Catalyst Poison application amount overflow".into()))?
    } else { current };
    let application = Effect { source: effect.source.clone(), target: effect.target.clone(),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount } };
    super::super::super::engine::validate_effect(&application, state)?;
    state.pending.push_front(application);
    Ok(())
}
