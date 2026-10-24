//! Ironclad card payload construction for the full native resolver.
use super::*;
pub(crate) mod rules;
mod direct;
mod choices;
mod state;
mod upgrade;
mod support;
pub(in crate::combat) mod exhaust;
mod powers;
pub(in crate::combat) mod reactive;
pub(in crate::combat) mod fatal;
pub(in crate::combat) mod growth;
pub(in crate::combat) mod autoplay;
mod copies;
pub use state::IroncladEffect;

pub(in crate::combat) fn apply(state:&mut State,effect:&Effect,work:&IroncladEffect)->Result<(),EffectError>{
    match work {
        IroncladEffect::ChooseTrueGritCard|IroncladEffect::ChooseHeadbuttCard|IroncladEffect::ChooseExhumeCard|IroncladEffect::ChooseDualWieldCard=>choices::apply(state,effect,work),
        IroncladEffect::DoubleCurrentBlock|IroncladEffect::SpotWeakness{..}|IroncladEffect::TrueGritRandomExhaust=>support::apply(state,effect,work),
        IroncladEffect::WhirlwindRound{..}|IroncladEffect::WhirlwindSpend|IroncladEffect::HavocTop{..}|IroncladEffect::HavocExtract{..}
        |IroncladEffect::DoubleTapRepeat{..}|IroncladEffect::HavocQueued{..}|IroncladEffect::FinishSuppressed=>autoplay::apply(state,effect,work),
        IroncladEffect::LimitBreakDispatch|IroncladEffect::DropkickDispatch{..}|IroncladEffect::RampageGrow{..}=>growth::apply(state,effect,work),
        IroncladEffect::ReaperResolve{..}=>fatal::apply(state,effect,work),
        IroncladEffect::InfernalBladeGenerate=>copies::apply(state,effect),
        _=>upgrade::apply(state,effect,work),
    }
}

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    if let Some(result)=exhaust::payload(state,card,target)?{return Ok(Some(result))}
    if let Some(result)=direct::payload(state,card,target)?{return Ok(Some(result))}
    if let Some(result)=fatal::payload(state,card,target)?{return Ok(Some(result))}
    if let Some(result)=support::payload(state,card,target)?{return Ok(Some(result))}
    if let Some(result)=powers::payload(card,target)?{return Ok(Some(result))}
    if let Some(result)=reactive::payload(state,card,target)?{return Ok(Some(result))}
    if let Some(result)=copies::payload(card,target)?{return Ok(Some(result))}
    use crate::content::CardId::*;
    let upgraded = card.upgrades > 0;
    let result = match card.id {
        InfernalBlade=>{return Err(crate::run::RunError::Unsupported("Infernal Blade payload dispatch was bypassed"))},
        Armaments=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            (1,vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(modified_block(state,5)?)),
                (None,EffectKind::Ironclad(IroncladEffect::UpgradeHand{all:upgraded}))])
        },
        SearingBlow=>{
            let Some(target)=target else{return Err(crate::run::RunError::InvalidAction)};
            let damage=modified_damage(state,target,rules::searing_damage(card))?;
            (2,vec![(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage)))])
        },
        Inflame=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}(1,vec![(Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:if upgraded{3}else{2}})])},
        Flex=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}(0,vec![(Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:if upgraded{4}else{2}}),(Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("lose_strength"),amount:if upgraded{4}else{2}})])},
        LimitBreak=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)};(1,vec![(Some(PayloadTarget::Player),EffectKind::Ironclad(IroncladEffect::LimitBreakDispatch))])},
        BloodForBlood=>{let Some(target)=target else{return Err(crate::run::RunError::InvalidAction)};(if upgraded{3}else{4},vec![(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(modified_damage(state,target,if upgraded{22}else{18})?)))])},
        Feed|Reaper=>return Err(crate::run::RunError::InvalidAction),
        Rampage=>{let Some(target)=target else{return Err(crate::run::RunError::InvalidAction)};let damage=modified_damage(state,target,card.local.base_damage.unwrap_or(8))?;(1,vec![(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage))),(None,EffectKind::Ironclad(IroncladEffect::RampageGrow{family:card.copy_family,amount:if upgraded{8}else{5}}))])},
        Dropkick=>{let Some(target)=target else{return Err(crate::run::RunError::InvalidAction)};let damage=modified_damage(state,target,if upgraded{8}else{5})?;(1,vec![(Some(PayloadTarget::Enemy(state.enemies[(target-1) as usize].instance_id.clone())),EffectKind::Ironclad(IroncladEffect::DropkickDispatch{damage}))])},
        Bludgeon=>{
            let Some(target_id)=target else{return Err(crate::run::RunError::InvalidAction)};
            let damage=modified_damage(state,target_id,if upgraded{42}else{32})?;
            (3,vec![(Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(damage)))])
        },
        Impervious=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            (2,vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(modified_block(state,if upgraded{40}else{30})?))])
        },
        Offering=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            (0,vec![(Some(PayloadTarget::Player),EffectKind::Damage(DamagePayload::HpLoss(6))),
                (Some(PayloadTarget::Player),EffectKind::GainEnergy(2)),(None,EffectKind::Draw(if upgraded{5}else{3}))])
        },
        Bloodletting => {
            if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
            (0, vec![
                (Some(PayloadTarget::Player), EffectKind::Damage(DamagePayload::HpLoss(3))),
                (Some(PayloadTarget::Player), EffectKind::GainEnergy(if upgraded { 3 } else { 2 })),
            ])
        }
        Havoc=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}(if upgraded{0}else{1},vec![(None,EffectKind::Ironclad(IroncladEffect::HavocTop{target:None}))])},
        DoubleTap=>{if target.is_some(){return Err(crate::run::RunError::InvalidAction)}(1,vec![(Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("double_tap"),amount:if upgraded{2}else{1}})])},
        Whirlwind=>{
            if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
            let mut hits=Vec::new();
            for enemy in state.enemies.iter().filter(|enemy|enemy.life==super::super::ActorLife::Active&&enemy.hp>0){
                hits.push((enemy.instance_id.clone(),modified_damage_for_enemy(state,enemy,if upgraded{8}else{5})?));
            }
            let energy_on_use=state.play_context.as_ref().filter(|context|context.card==card_handle(card))
                .map_or(state.player.energy,|context|if context.energy_on_use == -1 {state.player.energy}else{context.energy_on_use});
            if state.item_context.as_ref().is_some_and(|items|items.relics().iter().any(|relic|relic.as_str()=="chemical_x")){
                return Err(crate::run::RunError::Unsupported("Chemical X modifier is not represented for Whirlwind X payment".into()))
            }
            let rounds=u32::try_from(energy_on_use.max(0)).map_err(|_|crate::run::RunError::InvalidAction)?;
            (0,vec![(None,EffectKind::Ironclad(IroncladEffect::WhirlwindRound{remaining:rounds,hits,spend_current:rounds>0&&!card.local.free_play}))])
        },
        Disarm => {
            if target.is_none() { return Err(crate::run::RunError::InvalidAction); }
            (1, vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower {
                id: FullPowerId("strength"), amount: if upgraded { -3 } else { -2 } })])
        }
        BurningPact => {
            if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
            let candidates = state.piles.hand.iter().filter(|owned| owned.handle() != super::card_handle(card))
                .map(|owned| owned.handle()).collect::<Vec<_>>();
            let mut work = Vec::new();
            if candidates.len() == 1 {
                let instance = candidates[0].as_str().parse().map_err(|_| crate::run::RunError::InvalidAction)?;
                work.push((None, EffectKind::ExhaustHandCard { instance: CardInstanceId(instance) }));
            } else if candidates.len() > 1 {
                work.push((None, EffectKind::SelectCards { candidates, required: 1,
                    disposition: SelectionDisposition::Exhaust, empty: EmptySelection::Continue }));
            }
            work.push((None, EffectKind::Draw(if upgraded { 3 } else { 2 })));
            (1, work)
        }
        Bash => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 10 } else { 8 })?;
            let enemy = state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone();
            (2, vec![
                (Some(PayloadTarget::Enemy(enemy.clone())), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::Enemy(enemy)), EffectKind::ApplyFullPower {
                    id: FullPowerId("vulnerable"), amount: if upgraded { 3 } else { 2 } }),
            ])
        }
        Anger => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 8 } else { 6 })?;
            let source = super::card_handle(card);
            (0, vec![
                (Some(PayloadTarget::Enemy(state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(damage))),
                (None, EffectKind::CopyCard { source,
                    destination: crate::combat::GeneratedCardDestination::Discard }),
            ])
        }
        Clothesline => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 14 } else { 12 })?;
            let enemy = state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone();
            (2, vec![
                (Some(PayloadTarget::Enemy(enemy.clone())), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::Enemy(enemy)), EffectKind::ApplyFullPower {
                    id: FullPowerId("weak"), amount: if upgraded { 3 } else { 2 } }),
            ])
        }
        Cleave => {
            if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
            let mut effects = Vec::new();
            for enemy in state.enemies.iter().filter(|enemy| enemy.hp > 0) {
                let damage = modified_damage_for_enemy(state, enemy, if upgraded { 11 } else { 8 })?;
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            (1, effects)
        }
        PommelStrike => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 10 } else { 9 })?;
            (1, vec![
                (Some(PayloadTarget::Enemy(state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(damage))),
                (None, EffectKind::Draw(if upgraded { 2 } else { 1 })),
            ])
        }
        ShrugItOff => (1, vec![
            (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 11 } else { 8 })?)),
            (None, EffectKind::Draw(1)),
        ]),
        TwinStrike => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 7 } else { 5 })?;
            let enemy = state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone();
            (1, vec![
                (Some(PayloadTarget::Enemy(enemy.clone())), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::Enemy(enemy)), EffectKind::Damage(DamagePayload::Fixed(damage))),
            ])
        }
        HeavyBlade => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let strength = player_strength(state)?;
            let scaled = strength.checked_mul(if upgraded { 5 } else { 3 })
                .ok_or(crate::run::RunError::Unsupported("Heavy Blade Strength multiplication overflow"))?;
            let damage = modified_damage_with_strength(state, target_id, 14, scaled)?;
            (2, vec![(Some(PayloadTarget::Enemy(state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone())),
                EffectKind::Damage(DamagePayload::Fixed(damage)))])
        }
        PerfectedStrike => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let strike_count = state.piles.hand.iter().chain(&state.piles.draw).chain(&state.piles.discard)
                .filter_map(|owned| owned.full()).filter(|live| has_strike_tag(live.id)).count();
            let multiplier = if upgraded { 3_i32 } else { 2_i32 };
            let strike_count = i32::try_from(strike_count)
                .map_err(|_| crate::run::RunError::Unsupported("Perfected Strike count overflow"))?;
            let base = 6_i32.checked_add(strike_count.checked_mul(multiplier)
                .ok_or(crate::run::RunError::Unsupported("Perfected Strike damage overflow"))?)
                .ok_or(crate::run::RunError::Unsupported("Perfected Strike damage overflow"))?;
            let damage = modified_damage(state, target_id, base)?;
            (2, vec![(Some(PayloadTarget::Enemy(state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone())),
                EffectKind::Damage(DamagePayload::Fixed(damage)))])
        }
        Thunderclap => {
            if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
            let mut effects = Vec::new();
            for enemy in state.enemies.iter().filter(|enemy| enemy.hp > 0) {
                let damage = modified_damage_for_enemy(state, enemy, if upgraded { 7 } else { 4 })?;
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            for enemy in state.enemies.iter().filter(|enemy| enemy.hp > 0) {
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                    EffectKind::ApplyFullPower { id: FullPowerId("vulnerable"), amount: 1 }));
            }
            (1, effects)
        }
        _ => return Ok(None),
    };
    Ok(Some(result))
}

fn modified_damage_for_enemy(state: &State, enemy: &super::super::model::EnemyState, base: i32)
    -> Result<i32, crate::run::RunError>
{
    let strength = state.player.powers.iter().find(|p| p.id == PowerId::Strength)
        .map_or(0, |p| p.state.amount)
        + state.player_full_powers.iter().filter(|p| p.id.0 == "strength").map(|p| p.state.amount).sum::<i32>();
    let weak = state.player_full_powers.iter().any(|p| p.id.0 == "weak" && p.state.amount > 0);
    super::normal_damage(state, base, strength, weak, enemy)
}

fn player_strength(state: &State) -> Result<i32, crate::run::RunError> {
    let legacy = state.player.powers.iter().find(|p| p.id == PowerId::Strength).map_or(0, |p| p.state.amount);
    let full = state.player_full_powers.iter().filter(|p| p.id.0 == "strength").try_fold(0_i32,
        |sum, p| sum.checked_add(p.state.amount))
        .ok_or(crate::run::RunError::Unsupported("player Strength sum overflow"))?;
    legacy.checked_add(full).ok_or(crate::run::RunError::Unsupported("player Strength sum overflow"))
}

fn player_strength_power(state:&State)->Result<Option<i32>,crate::run::RunError>{
    let present=state.player.powers.iter().any(|power|power.id==PowerId::Strength)
        ||state.player_full_powers.iter().any(|power|power.id.0=="strength");
    if present{player_strength(state).map(Some)}else{Ok(None)}
}

fn modified_damage_with_strength(state: &State, target: u64, base: i32, strength: i32)
    -> Result<i32, crate::run::RunError>
{
    let enemy = state.enemies.get((target - 1) as usize)
        .filter(|enemy| enemy.hp > 0).ok_or(crate::run::RunError::InvalidAction)?;
    let weak = state.player_full_powers.iter().any(|p| p.id.0 == "weak" && p.state.amount > 0);
    super::normal_damage(state, base, strength, weak, enemy)
}

fn has_strike_tag(id: crate::content::CardId) -> bool {
    use crate::content::CardId::*;
    matches!(id, Strike | StrikeGreen | StrikeBlue | StrikePurple | PerfectedStrike
        | PommelStrike | TwinStrike | Thunderclap)
}
