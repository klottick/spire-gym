//! Turn scheduling preserves construction-time setup separately from queued hooks.
use super::*;
use super::model::State;
use super::rng::StreamState;

fn work(kind: EffectKind) -> Effect { Effect { source: Source::None, target: None, kind } }

pub(super) fn schedule_next_player_turn(state: &mut State) -> Result<(), InputError> {
    let next = state.turn.checked_add(1).filter(|n| *n <= i32::MAX as u32)
        .ok_or_else(|| InputError::Unsupported("next turn outside supported numeric surface".into()))?;
    state.turn = next;
    state.phase = Phase::PlayerStart;
    state.player.block = 0;
    state.player.energy = state.player.max_energy;
    for enemy in &mut state.enemies { enemy.intent_id = enemy.selected_move; }
    super::effects::refresh_enemy_values(state)?;
    state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::Draw(state.player.draw_per_turn) });
    state.pending.push_back(work(EffectKind::Decision));
    Ok(())
}

// Both native selection and test-only forced exhaustion use this result boundary.
// No history/publication is fabricated when a selection fails partway through RNG.
pub(super) fn install_selection(state: &mut State,
    result: Result<(MoveId, StreamState), rules::SelectionError>) -> Result<(), EffectError>
{
    let (chosen, next) = result.map_err(|error| match error {
        rules::SelectionError::Invalid(error) => EffectError::Invalid(error),
        rules::SelectionError::Random(error) => EffectError::Random(error),
    })?;
    state.rng.enemy_ai = next;
    let enemy = &mut state.enemies[0];
    enemy.selected_move = chosen;
    enemy.move_history.push(chosen);
    Ok(())
}

fn round_hooks(powers: &mut [PowerEntry], actor: Actor, pending: &mut std::collections::VecDeque<Effect>) {
    for power in powers {
        let kind = match power.id {
            PowerId::Ritual if power.state.skip_first => {
                power.state.skip_first = false;
                None
            }
            PowerId::Ritual => Some(EffectKind::ApplyPower { id: PowerId::Strength, amount: power.state.amount }),
            PowerId::Vulnerable if power.state.monster_source_grace => {
                power.state.monster_source_grace = false;
                None
            }
            PowerId::Vulnerable => Some(EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 }),
            PowerId::Strength => None,
        };
        if let Some(kind) = kind {
            pending.push_back(Effect { source: Source::Actor(actor.clone()), target: Some(actor.clone()), kind });
        }
    }
}

pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    super::engine::validate_effect(effect, state)?;
    match effect.kind {
        EffectKind::EndPlayerTurn => {
            state.phase = Phase::EnemyTurn;
            state.pending.push_front(work(EffectKind::MonsterStart));
            state.pending.push_front(work(EffectKind::DiscardHand));
        }
        EffectKind::MonsterStart => {
            state.phase = Phase::EnemyTurn;
            state.enemies[0].block = 0;
            state.pending.push_front(work(EffectKind::EnemyMove));
        }
        EffectKind::EnemyMove => {
            let enemy = &state.enemies[0];
            let actor = Actor::Enemy(enemy.instance_id.clone());
            let payloads = match enemy.intent_id {
                MoveId::Incantation => vec![(actor.clone(), EffectKind::ApplyPower { id: PowerId::Ritual, amount: 3 })],
                id @ (MoveId::Attack | MoveId::Chomp) => vec![(Actor::Player, EffectKind::Damage(DamagePayload::EnemyMove(id)))],
                MoveId::Thrash => vec![(Actor::Player, EffectKind::Damage(DamagePayload::EnemyMove(MoveId::Thrash))),
                    (actor.clone(), EffectKind::GainBlock(5))],
                MoveId::Bellow => vec![(actor.clone(), EffectKind::ApplyPower { id: PowerId::Strength, amount: 3 }),
                    (actor.clone(), EffectKind::GainBlock(6))],
            };
            state.pending.push_front(work(EffectKind::EndRound));
            state.pending.push_front(Effect { source: Source::Actor(actor.clone()),
                target: None, kind: EffectKind::RollMove });
            for (target, kind) in payloads.into_iter().rev() {
                state.pending.push_front(Effect { source: Source::Actor(actor.clone()), target: Some(target), kind });
            }
        }
        EffectKind::RollMove => {
            let enemy = &state.enemies[0];
            let result = rules::choose_enemy_move(enemy.enemy_id, &enemy.move_history, &state.rng.enemy_ai);
            install_selection(state, result)?;
        }
        EffectKind::EndRound => {
            state.phase = Phase::RoundEnd;
            round_hooks(&mut state.player.powers, Actor::Player, &mut state.pending);
            for enemy in &mut state.enemies {
                round_hooks(&mut enemy.powers, Actor::Enemy(enemy.instance_id.clone()), &mut state.pending);
            }
            schedule_next_player_turn(state)?;
        }
        _ => return Err(InputError::Invariant("turn owner received non-turn work".into()).into()),
    }
    Ok(())
}
