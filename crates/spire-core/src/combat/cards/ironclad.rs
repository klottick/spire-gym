//! Ironclad card payload construction for the full native resolver.
use super::*;

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    use crate::content::CardId::*;
    let upgraded = card.upgrades > 0;
    let result = match card.id {
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
    let vulnerable = enemy.powers.iter().any(|p| p.id == PowerId::Vulnerable && p.state.amount > 0)
        || enemy.full_powers.iter().any(|p| p.id.0 == "vulnerable" && p.state.amount > 0);
    let weak = state.player_full_powers.iter().any(|p| p.id.0 == "weak" && p.state.amount > 0);
    super::normal_damage(state, base, strength, weak, vulnerable)
}

fn player_strength(state: &State) -> Result<i32, crate::run::RunError> {
    let legacy = state.player.powers.iter().find(|p| p.id == PowerId::Strength).map_or(0, |p| p.state.amount);
    let full = state.player_full_powers.iter().filter(|p| p.id.0 == "strength").try_fold(0_i32,
        |sum, p| sum.checked_add(p.state.amount))
        .ok_or(crate::run::RunError::Unsupported("player Strength sum overflow"))?;
    legacy.checked_add(full).ok_or(crate::run::RunError::Unsupported("player Strength sum overflow"))
}

fn modified_damage_with_strength(state: &State, target: u64, base: i32, strength: i32)
    -> Result<i32, crate::run::RunError>
{
    let enemy = state.enemies.get((target - 1) as usize)
        .filter(|enemy| enemy.hp > 0).ok_or(crate::run::RunError::InvalidAction)?;
    let vulnerable = enemy.powers.iter().any(|p| p.id == PowerId::Vulnerable && p.state.amount > 0)
        || enemy.full_powers.iter().any(|p| p.id.0 == "vulnerable" && p.state.amount > 0);
    let weak = state.player_full_powers.iter().any(|p| p.id.0 == "weak" && p.state.amount > 0);
    super::normal_damage(state, base, strength, weak, vulnerable)
}

fn has_strike_tag(id: crate::content::CardId) -> bool {
    use crate::content::CardId::*;
    matches!(id, Strike | StrikeGreen | StrikeBlue | StrikePurple | PerfectedStrike
        | PommelStrike | TwinStrike | Thunderclap)
}
