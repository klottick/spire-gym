//! Exhaustive routing to the finite synchronous payload owners.
use super::*;
use super::model::{OwnedCard, PendingSelection, State};

fn amount(powers: &[PowerEntry], id: PowerId) -> i32 {
    powers.iter().find(|p| p.id == id).map_or(0, |p| p.state.amount)
}

/// Canonical output used by both refresh and ready-check validation. Keeping
/// the power projection here prevents full-only power state from being applied
/// during refresh but omitted by the invariant.
pub(super) fn refreshed_damage(base: i32, enemy_powers: &[PowerEntry],
    enemy_full_powers: &[super::FullPowerEntry], player_powers: &[PowerEntry],
    player_full_powers: &[super::FullPowerEntry], wrath: bool, full_runtime: bool) -> Result<i32, InputError>
{
    let strength = amount(enemy_powers, PowerId::Strength)
        + enemy_full_powers.iter().find(|p| p.id.0 == "strength").map_or(0, |p| p.state.amount);
    let vulnerable = amount(player_powers, PowerId::Vulnerable) > 0
        || player_full_powers.iter().any(|p| p.id.0 == "vulnerable" && p.state.amount > 0);
    let weak = enemy_full_powers.iter().any(|p| p.id.0 == "weak" && p.state.amount > 0);
    if full_runtime { rules::full_normal_damage_modifiers(base, strength, weak, wrath, vulnerable) }
    else { rules::normal_damage_modifiers(base, strength, weak, wrath, vulnerable) }
}

pub(super) fn refresh_enemy_values(state: &mut State) -> Result<(), InputError> {
    let wrath = state.character.stance == super::characters::Stance::Wrath;
    let player_powers = &state.player.powers;
    let player_full_powers = &state.player_full_powers;
    for enemy in &mut state.enemies {
        for entry in enemy.damage_entries.values_mut().chain(enemy.full_damage_entries.values_mut()) {
            entry.output = refreshed_damage(entry.base, &enemy.powers, &enemy.full_powers,
                player_powers, player_full_powers, wrath, state.full_runtime)?;
        }
        enemy.intent_damage = match &enemy.intent_id {
            ActorIntentId::Legacy(id) => enemy.damage_entries.get(id).map(|d| d.output),
            ActorIntentId::Full(id) => enemy.full_damage_entries.get(id).map(|d| d.output),
        };
    }
    Ok(())
}

fn powers_mut<'a>(state: &'a mut State, target: &Actor) -> &'a mut Vec<PowerEntry> {
    match target {
        Actor::Player => &mut state.player.powers,
        Actor::Enemy(id) => &mut state.enemies.iter_mut().find(|e| &e.instance_id == id).expect("validated actor").powers,
    }
}

pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    super::engine::validate_effect(effect, state)?;
    if state.full_runtime && matches!(effect.kind, EffectKind::ApplyPower { .. }
        | EffectKind::ReducePower { .. } | EffectKind::RemovePower(_)
        | EffectKind::ApplyFullPower { .. } | EffectKind::ReduceFullPower { .. } | EffectKind::RemoveFullPower(_)) {
        return super::powers::apply_effect(state, effect).map_err(Into::into);
    }
    match &effect.kind {
        EffectKind::Damage(payload) => {
            // The validated target is needed while calculating typed payloads as
            // well as for the shared Block/HP commit below.
            let target = effect.target.as_ref().unwrap();
            let damage = match payload {
                DamagePayload::Fixed(n) | DamagePayload::Orb(n) | DamagePayload::Thorns(n) | DamagePayload::HpLoss(n) => *n,
                DamagePayload::EnemyMove(id) => {
                    let source = match &effect.source { Source::Actor(Actor::Enemy(id)) => id, _ => unreachable!("validated enemy source") };
                    state.enemies.iter().find(|e| &e.instance_id == source).expect("validated enemy").damage_entries[id].output
                },
                DamagePayload::FullEnemyMove(id) => {
                    let source = match &effect.source { Source::Actor(Actor::Enemy(id)) => id, _ => unreachable!("validated full enemy source") };
                    state.enemies.iter().find(|e| &e.instance_id == source).expect("validated full enemy").full_damage_entries[id].output
                }
            };
            let full_runtime = state.full_runtime;
            let (hp, block) = match target {
                Actor::Player => (&mut state.player.hp, &mut state.player.block),
                Actor::Enemy(id) => {
                    let enemy = state.enemies.iter_mut().find(|e| &e.instance_id == id).expect("validated actor");
                    (&mut enemy.hp, &mut enemy.block)
                }
            };
            let hp_before = *hp;
            let (next_hp, next_block) = if matches!(payload, DamagePayload::HpLoss(_)) {
                ((i64::from(hp_before) - i64::from(damage)).max(0) as i32, *block)
            } else if full_runtime {
                rules::full_hp_and_block_after_damage(hp_before, *block, damage)?
            } else {
                rules::hp_and_block_after_damage(hp_before, *block, damage)?
            };
            *hp = next_hp;
            *block = next_block;
            if let Actor::Enemy(id) = target {
                // Only these native payload kinds are NORMAL for Curl Up.
                // Orb and typed THORNS damage retain their distinct exclusions.
                let normal_damage = matches!(payload,
                    DamagePayload::Fixed(_) | DamagePayload::EnemyMove(_) | DamagePayload::FullEnemyMove(_));
                super::enemies::on_hit(state, id, &effect.source, normal_damage, hp_before, hp_before - next_hp);
            }
            if next_hp == 0 {
                let outcome = match target {
                    Actor::Player => Some(Outcome::Death),
                    Actor::Enemy(_) if state.enemies.iter().all(|e| e.hp == 0) => Some(Outcome::Victory),
                    Actor::Enemy(_) => None,
                };
                if let Some(outcome) = outcome {
                    state.pending.push_front(Effect { source: effect.source.clone(), target: None,
                        kind: EffectKind::Terminal(outcome) });
                    if state.full_runtime {
                        state.pending.push_front(Effect { source: effect.source.clone(), target: None,
                            kind: EffectKind::Hook(Hook::CombatSettled) });
                    }
                }
                if state.full_runtime {
                    state.pending.push_front(Effect { source: effect.source.clone(), target: effect.target.clone(),
                        kind: EffectKind::Hook(Hook::Death) });
                }
            }
            if state.full_runtime {
                state.pending.push_front(Effect { source: effect.source.clone(), target: effect.target.clone(),
                    kind: EffectKind::Hook(Hook::Damage) });
            }
        }
        EffectKind::GainBlock(n) => {
            let full_runtime = state.full_runtime;
            let block = match effect.target.as_ref().unwrap() {
                Actor::Player => &mut state.player.block,
                Actor::Enemy(id) => &mut state.enemies.iter_mut().find(|e| &e.instance_id == id).expect("validated actor").block,
            };
            *block = if full_runtime {
                rules::full_block_after_add(*block, *n)?
            } else {
                rules::block_after_add(*block, *n)?
            };
        }
        EffectKind::Heal(n) => {
            if state.player.hp > 0 {
                let amount = if state.full_runtime {
                    crate::items::ordinary_heal_amount_in_room(
                        state.item_context.as_ref().map_or(&[], |context| context.relics()), *n,
                        state.item_context.as_ref().map_or(
                            crate::items::NativeHealRoom::default(), |context| context.heal_room()))
                } else { *n };
                state.player.hp = i32::try_from((i64::from(state.player.hp) + i64::from(amount))
                    .min(i64::from(state.player.max_hp)))
                    .map_err(|_| InputError::Invariant("healing overflow".into()))?;
            }
        }
        EffectKind::NeowsLament=>{
            for enemy in state.enemies.iter_mut().filter(|e|e.hp>0&&e.life==super::ActorLife::Active){enemy.hp=1;}
        }
        EffectKind::IncreaseMaxHpThenHeal(n) => {
            // Commit the max-HP child first, then schedule the ordinary Heal
            // as the next queue item with the same effect owner.
            let max_hp = state.player.max_hp.checked_add(*n)
                .ok_or_else(|| InputError::Unsupported("player maximum HP overflow".into()))?;
            state.player.max_hp = max_hp;
            state.pending.push_front(Effect { source: effect.source.clone(),
                target: Some(Actor::Player), kind: EffectKind::Heal(*n) });
        }
        EffectKind::GainEnergy(n) => {
            state.player.energy = state.player.energy.checked_add(*n)
                .filter(|energy| *energy <= 999)
                .ok_or_else(|| InputError::Unsupported("player energy overflow".into()))?;
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
        EffectKind::ApplyFullPower { .. } | EffectKind::ReduceFullPower { .. } | EffectKind::RemoveFullPower(_) =>
            return Err(InputError::Invariant("full power outside full runtime".into()).into()),
        EffectKind::Character(action) => super::characters::apply(state, &effect.source, *action)?,
        EffectKind::CardContinuation(continuation) =>
            super::cards::apply_defect_continuation(state, effect, continuation)?,
        EffectKind::ReplaceEnemy { parent, children } =>
            super::enemies::apply_replacement(state, parent, children)?,
        EffectKind::Decision => {
            if !state.pending.is_empty() || state.in_play.is_some() || state.player.hp <= 0
                || state.enemies.iter().all(|e| e.hp <= 0) || state.outcome.is_some() {
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
            if *outcome == Outcome::Victory && !state.full_runtime {
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
        EffectKind::SelectCard { candidates, empty } => {
            if candidates.is_empty() {
                match empty { EmptySelection::Continue => (), }
            } else {
                let Source::Card(parent) = &effect.source else {
                    return Err(InputError::Invariant("selection parent is not a card".into()).into());
                };
                begin_selection(state, parent.clone(), candidates.clone(), 1,
                    super::selection::SelectionDisposition::Hand, *empty)?;
            }
        }
        EffectKind::SelectHandCardsAfterDraw { required, disposition, empty } => {
            let Source::Card(parent) = &effect.source else {
                return Err(InputError::Invariant("selection parent is not a card".into()).into());
            };
            let candidates = state.piles.hand.iter().map(|card| card.handle()).collect::<Vec<_>>();
            if candidates.is_empty() {
                match empty { EmptySelection::Continue => return Ok(()), }
            }
            if *required == 0 || *required > candidates.len() {
                return Err(InputError::Invariant("invalid captured hand selection count".into()).into());
            }
            begin_selection(state, parent.clone(), candidates, *required, *disposition, *empty)?;
        }
        EffectKind::SelectCards { candidates, required, disposition, empty } => {
            let Source::Card(parent) = &effect.source else {
                return Err(InputError::Invariant("selection parent is not a card".into()).into());
            };
            if *required == 0 {
                if candidates.is_empty() { return Ok(()); }
                return Err(InputError::Invariant("zero-count selection has candidates".into()).into());
            }
            if candidates.is_empty() {
                match empty {
                    EmptySelection::Continue => return Ok(()),
                }
            }
            if *required > candidates.len() { return Err(InputError::Invariant("selection count exceeds candidates".into()).into()); }
            begin_selection(state, parent.clone(), candidates.clone(), *required, *disposition, *empty)?;
        }
        EffectKind::Hook(hook) => {
            if state.full_runtime {
                super::hooks::dispatch(state, *hook, &effect.source)?;
                let hp = state.player.hp;
                let work = if let Some(context) = state.item_context.as_mut() {
                    crate::items::relic_hook_effects(context, *hook, hp)
                } else {
                    crate::items::relic_hook_effects(
                        &mut crate::items::CombatItemContext::default(), *hook, hp)
                };
                for effect in work.bottom {
                    super::engine::validate_effect(&effect, state)?;
                    state.pending.push_back(effect);
                }
                for effect in work.top.into_iter().rev() {
                    super::engine::validate_effect(&effect, state)?;
                    state.pending.push_front(effect);
                }
                state.hook_events.push(HookEvent { hook: *hook,
                    source: effect.source.clone(), target: effect.target.clone() });
            }
        }
        EffectKind::Draw(_) | EffectKind::Recycle | EffectKind::DiscardHand
        | EffectKind::CreateCard { .. } | EffectKind::CopyCard { .. } | EffectKind::ExhaustHandCard { .. } => {
            super::piles::apply_effect(state, effect)?;
        }
        EffectKind::EndPlayerTurn |
        EffectKind::MonsterStart | EffectKind::EnemyMove | EffectKind::RollMove | EffectKind::EndRound => {
            super::turns::apply_effect(state, effect)?;
        }
        EffectKind::StartNextPlayer => super::turns::apply_effect(state, effect)?,
    }
    Ok(())
}

fn begin_selection(state: &mut State, parent: super::model::CardHandle,
    candidates: Vec<super::model::CardHandle>, required: usize,
    disposition: super::selection::SelectionDisposition,
    empty: super::selection::EmptySelection) -> Result<(), EffectError> {
    let mut seen = std::collections::BTreeSet::new();
    if candidates.iter().any(|id| !seen.insert(id.clone()))
        || candidates.iter().any(|id| !state.piles.all(None).any(|card| card.handle() == *id)) {
        return Err(InputError::Invariant("selection candidates are duplicate or unowned".into()).into());
    }
    if state.in_play.as_ref().map(OwnedCard::handle).as_ref() != Some(&parent) {
        return Err(InputError::Invariant("selection parent is not the in-play card".into()).into());
    }
    if required == 0 || required > candidates.len() {
        return Err(InputError::Invariant("invalid required selection count".into()).into());
    }
    state.pending_selection = Some(PendingSelection { parent, candidates, selected: Vec::new(),
        required, disposition, empty });
    state.phase = Phase::Selection;
    Ok(())
}
