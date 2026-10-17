//! Exhaustive routing to the finite synchronous payload owners.
use super::*;
use super::model::State;

fn amount(powers: &[PowerEntry], id: PowerId) -> i32 {
    powers.iter().find(|p| p.id == id).map_or(0, |p| p.state.amount)
}

pub(super) fn refresh_enemy_values(state: &mut State) -> Result<(), InputError> {
    let vulnerable = amount(&state.player.powers, PowerId::Vulnerable) > 0;
    for enemy in &mut state.enemies {
        let strength = amount(&enemy.powers, PowerId::Strength);
        for entry in enemy.damage_entries.values_mut() {
            entry.output = rules::normal_damage(entry.base, strength, vulnerable)?;
        }
        enemy.intent_damage = enemy.damage_entries.get(&enemy.intent_id).map(|d| d.output);
    }
    Ok(())
}

fn powers_mut<'a>(state: &'a mut State, target: &Actor) -> &'a mut Vec<PowerEntry> {
    // validate_effect checks the sole enemy identity before entering any owner.
    match target { Actor::Player => &mut state.player.powers, Actor::Enemy(_) => &mut state.enemies[0].powers }
}

pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    super::engine::validate_effect(effect, state)?;
    match &effect.kind {
        EffectKind::Damage(payload) => {
            let damage = match payload {
                DamagePayload::Fixed(n) => *n,
                DamagePayload::EnemyMove(id) => state.enemies[0].damage_entries[id].output,
            };
            let target = effect.target.as_ref().unwrap();
            let (hp, block) = match target {
                Actor::Player => (&mut state.player.hp, &mut state.player.block),
                Actor::Enemy(_) => {
                    let enemy = &mut state.enemies[0];
                    (&mut enemy.hp, &mut enemy.block)
                }
            };
            let (next_hp, next_block) = rules::hp_and_block_after_damage(*hp, *block, damage)?;
            *hp = next_hp;
            *block = next_block;
            if next_hp == 0 {
                let outcome = match target { Actor::Player => Outcome::Death, Actor::Enemy(_) => Outcome::Victory };
                state.pending.push_front(Effect { source: effect.source.clone(), target: None,
                    kind: EffectKind::Terminal(outcome) });
            }
        }
        EffectKind::GainBlock(n) => {
            let block = match effect.target.as_ref().unwrap() {
                Actor::Player => &mut state.player.block,
                Actor::Enemy(_) => &mut state.enemies[0].block,
            };
            *block = rules::block_after_add(*block, *n)?;
        }
        EffectKind::ApplyPower { id, amount: delta } => {
            let grace = matches!(effect.source, Source::Actor(Actor::Enemy(_)));
            let powers = powers_mut(state, effect.target.as_ref().unwrap());
            let previous = amount(powers, *id);
            let total = i64::from(previous).checked_add(i64::from(*delta))
                .ok_or_else(|| InputError::Invariant("power sum overflow".into()))?;
            let next = if *id == PowerId::Strength { total.min(999) } else { total };
            let next = i32::try_from(next).map_err(|_| InputError::Invariant("power amount overflow".into()))?;
            if let Some(entry) = powers.iter_mut().find(|p| p.id == *id) {
                entry.state.amount = next;
            } else {
                powers.push(PowerEntry { id: *id, state: PowerState { amount: next,
                    skip_first: *id == PowerId::Ritual,
                    monster_source_grace: *id == PowerId::Vulnerable && grace } });
            }
            refresh_enemy_values(state)?;
        }
        EffectKind::ReducePower { id, amount: delta } => {
            let powers = powers_mut(state, effect.target.as_ref().unwrap());
            if let Some(entry) = powers.iter_mut().find(|p| p.id == *id) {
                entry.state.amount = entry.state.amount.checked_sub(*delta)
                    .ok_or_else(|| InputError::Invariant("power reduction overflow".into()))?.max(0);
                if entry.state.amount == 0 {
                    state.pending.push_front(Effect { source: effect.source.clone(), target: effect.target.clone(),
                        kind: EffectKind::RemovePower(*id) });
                }
            }
            refresh_enemy_values(state)?;
        }
        EffectKind::RemovePower(id) => {
            powers_mut(state, effect.target.as_ref().unwrap()).retain(|p| p.id != *id);
            refresh_enemy_values(state)?;
        }
        EffectKind::Decision => {
            if !state.pending.is_empty() || state.in_play.is_some() || state.player.hp <= 0
                || state.enemies[0].hp <= 0 || state.outcome.is_some() {
                return Err(InputError::Invariant("decision before quiescence".into()).into());
            }
            refresh_enemy_values(state)?;
            state.phase = Phase::Decision;
        }
        EffectKind::FinishCard => {
            let Source::Card(card) = &effect.source else {
                return Err(InputError::Invariant("FinishCard requires card source".into()).into());
            };
            super::piles::discard_card(state, card, false)?;
        }
        EffectKind::Terminal(outcome) => {
            state.hp_before_healing = Some(state.player.hp);
            if *outcome == Outcome::Victory {
                state.player.hp = i32::try_from((i64::from(state.player.hp) + 6)
                    .min(i64::from(state.player.max_hp)))
                    .map_err(|_| InputError::Invariant("terminal HP overflow".into()))?;
            }
            state.pending.clear();
            // Do not run FinishCard or reset diagnostic piles/block. dispatch_one
            // releases current after this synchronous terminal payload completes.
            state.phase = Phase::Terminal;
            state.outcome = Some(*outcome);
            state.hp_after_cleanup = Some(state.player.hp);
        }
        EffectKind::Draw(_) | EffectKind::Recycle | EffectKind::DiscardHand => {
            super::piles::apply_effect(state, effect)?;
        }
        EffectKind::EndPlayerTurn |
        EffectKind::MonsterStart | EffectKind::EnemyMove | EffectKind::RollMove | EffectKind::EndRound => {
            super::turns::apply_effect(state, effect)?;
        }
    }
    Ok(())
}
