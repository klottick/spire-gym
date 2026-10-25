//! Turn scheduling preserves construction-time setup separately from queued hooks.
use super::*;
use super::model::State;
use super::rng::StreamState;

fn work(kind: EffectKind) -> Effect { Effect { source: Source::None, target: None, kind } }

fn legacy_round_hooks(state: &mut State) {
    let mut queued = Vec::new();
    round_hooks(&mut state.player.powers, Actor::Player, &mut queued);
    for enemy in state.enemies.iter_mut().filter(|e| e.hp > 0) {
        round_hooks(&mut enemy.powers, Actor::Enemy(enemy.instance_id.clone()), &mut queued);
    }
    state.pending.extend(queued);
}

fn round_hooks(powers: &mut [PowerEntry], actor: Actor, queued: &mut Vec<Effect>) {
    for power in powers {
        let kind = match power.id {
            PowerId::Ritual if power.state.skip_first => { power.state.skip_first = false; None }
            PowerId::Ritual => Some(EffectKind::ApplyPower { id: PowerId::Strength, amount: power.state.amount }),
            PowerId::Vulnerable if power.state.monster_source_grace => { power.state.monster_source_grace = false; None }
            PowerId::Vulnerable => Some(EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 }),
            PowerId::Strength => None,
        };
        if let Some(kind) = kind {
            queued.push(Effect { source: Source::Actor(actor.clone()), target: Some(actor.clone()), kind });
        }
    }
}

pub(super) fn schedule_next_player_turn(state: &mut State) -> Result<(), InputError> {
    let next = state.turn.checked_add(1).filter(|n| *n <= i32::MAX as u32)
        .ok_or_else(|| InputError::Unsupported("next turn outside supported numeric surface".into()))?;
    state.turn = next;
    super::piles::expire_turn_scoped_costs(state);
    state.phase = Phase::PlayerStart;
    let retain_player_block=state.full_runtime&&state.player_full_powers.iter().any(|power|power.id.0=="barricade"&&power.state.amount>0);
    if !retain_player_block{state.player.block = 0;}
    state.player.energy = state.player.max_energy;
    for enemy in &mut state.enemies { enemy.intent_id = enemy.selected_move.clone(); }
    super::effects::refresh_enemy_values(state)?;
    let reduction=i32::from(state.full_runtime&&state.player_full_powers.iter().any(|p|p.id.0=="draw_reduction"&&p.state.amount>0));
    if state.full_runtime {state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::TurnStartedPreDraw)});}
    state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::Draw((state.player.draw_per_turn-reduction).max(0)) });
    if state.full_runtime {
        state.pending.push_back(Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
            kind: EffectKind::Hook(Hook::TurnStarted) });
    }
    state.pending.push_back(work(EffectKind::Decision));
    Ok(())
}

// Both native selection and test-only forced exhaustion use this result boundary.
// No history/publication is fabricated when a selection fails partway through RNG.
pub(super) fn install_selection(state: &mut State, owner: Option<&EnemyHandle>,
    result: Result<(MoveId, StreamState), rules::SelectionError>) -> Result<(), EffectError>
{
    let (chosen, next) = result.map_err(|error| match error {
        rules::SelectionError::Invalid(error) => EffectError::Invalid(error),
        rules::SelectionError::Random(error) => EffectError::Random(error),
    })?;
    state.rng.enemy_ai = next;
    let enemy = match owner {
        Some(id) => state.enemies.iter_mut().find(|e| &e.instance_id == id)
            .ok_or_else(|| InputError::Invariant("move selection actor missing".into()))?,
        None => &mut state.enemies[0],
    };
    enemy.selected_move = ActorIntentId::Legacy(chosen);
    enemy.move_history.push(ActorIntentId::Legacy(chosen));
    Ok(())
}

pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    super::engine::validate_effect(effect, state)?;
    match effect.kind {
        EffectKind::EndPlayerTurn => {
            state.phase = Phase::EnemyTurn;
            state.pending.push_front(work(EffectKind::MonsterStart));
            if state.full_runtime {state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::PlayerEndTurn)});}
            if state.full_runtime&&state.player_full_powers.iter().any(|p|p.id.0=="no_draw"){
                state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(FullPowerId("no_draw"))});
            }
            if state.full_runtime&&state.player_full_powers.iter().any(|p|p.id.0=="entangled"){
                state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(FullPowerId("entangled"))});
            }
            if state.full_runtime {
                state.pending.push_front(Effect { source: Source::Actor(Actor::Player),
                    target: Some(Actor::Player),
                    kind: EffectKind::Character(super::characters::CharacterEffect::EndTurn) });
            }
            state.pending.push_front(work(EffectKind::DiscardHand));
            if state.full_runtime {for effect in super::status::end_turn_work(state).into_iter().rev(){state.pending.push_front(effect);}}
            if state.full_runtime {state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::PlayerPreEndTurnCards)});}
        }
        EffectKind::MonsterStart => {
            state.phase = Phase::EnemyTurn;
            if state.full_runtime {
                for enemy in state.enemies.iter_mut().filter(|enemy|
                    enemy.life == super::ActorLife::Active && enemy.hp > 0) {
                    if !enemy.full_powers.iter().any(|p|p.id.0=="barricade"&&p.state.amount>0){enemy.block=0;}
                    if enemy.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::Byrd){if let Some(power)=enemy.full_powers.iter_mut().find(|p|p.id.0=="flight"){power.state.amount=3;}}
                    if enemy.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::CorruptHeart){if let Some(power)=enemy.full_powers.iter_mut().find(|p|p.id.0=="invincible"){power.state.amount=300;}}
                }
                state.pending.push_back(work(EffectKind::EndRound));
                let mut scheduled=state.enemies.iter().filter(|e|e.life==super::ActorLife::HalfDead||(e.life==super::ActorLife::Active&&e.hp>0)).collect::<Vec<_>>();
                scheduled.sort_by_key(|e|state.actor_slots.get(&e.instance_id).copied().unwrap_or(0));
                let actors=scheduled.into_iter().map(|e|e.instance_id.clone()).collect::<Vec<_>>();
                for id in actors.into_iter().rev() {
                    state.pending.push_front(Effect { source: Source::None,
                        target: Some(Actor::Enemy(id)), kind: EffectKind::EnemyMove });
                }
            } else {
                state.enemies[0].block = 0;
                state.pending.push_front(work(EffectKind::EnemyMove));
            }
        }
        EffectKind::EnemyMove => {
            let enemy_id = if state.full_runtime {
                match effect.target.as_ref() { Some(Actor::Enemy(id)) => id.clone(), _ => return Err(InputError::Invariant("full enemy turn missing actor".into()).into()) }
            } else { state.enemies[0].instance_id.clone() };
            let enemy = state.enemies.iter().find(|e| e.instance_id == enemy_id)
                .ok_or_else(|| InputError::Invariant("enemy turn actor missing".into()))?;
            if state.full_runtime && enemy.life!=super::ActorLife::HalfDead && (enemy.life != super::ActorLife::Active || enemy.hp <= 0) {
                return Ok(());
            }
            let actor = Actor::Enemy(enemy.instance_id.clone());
            let observed_move = super::observe::enabled(state).then(|| enemy.intent_id.clone());
            if state.full_runtime {
                state.pending.push_front(Effect { source: Source::Actor(actor.clone()),
                    target: Some(Actor::Enemy(enemy.instance_id.clone())), kind: EffectKind::RollMove });
                super::enemies::act(state, &enemy_id)?;
                if let Some(move_id)=observed_move { super::observe::emit(state, &Source::Actor(actor.clone()), None,
                    || ResolvedEffectKind::EnemyMove { actor, move_id }); }
                return Ok(());
            }
            let payloads = match (&enemy.enemy_id, &enemy.intent_id) {
                (ActorEnemyId::Legacy(_), ActorIntentId::Legacy(MoveId::Incantation)) => vec![(actor.clone(), EffectKind::ApplyPower { id: PowerId::Ritual, amount: 3 })],
                (ActorEnemyId::Legacy(_), ActorIntentId::Legacy(id @ (MoveId::Attack | MoveId::Chomp))) => vec![(Actor::Player, EffectKind::Damage(DamagePayload::EnemyMove(*id)))],
                (ActorEnemyId::Legacy(_), ActorIntentId::Legacy(MoveId::Thrash)) => vec![(Actor::Player, EffectKind::Damage(DamagePayload::EnemyMove(MoveId::Thrash))),
                    (actor.clone(), EffectKind::GainBlock(5))],
                (ActorEnemyId::Legacy(_), ActorIntentId::Legacy(MoveId::Bellow)) => vec![(actor.clone(), EffectKind::ApplyPower { id: PowerId::Strength, amount: 3 }),
                    (actor.clone(), EffectKind::GainBlock(6))],
                (ActorEnemyId::Full(crate::content::EnemyId::Cultist), ActorIntentId::Full(id)) if id.as_str() == "incantation" =>
                    vec![(actor.clone(), EffectKind::ApplyFullPower { id: super::FullPowerId("ritual"), amount: 3 })],
                (ActorEnemyId::Full(crate::content::EnemyId::Cultist), ActorIntentId::Full(id)) if id.as_str() == "attack" =>
                    vec![(Actor::Player, EffectKind::Damage(DamagePayload::FullEnemyMove(*id)))],
                (ActorEnemyId::Full(crate::content::EnemyId::JawWorm), ActorIntentId::Full(id)) if id.as_str() == "chomp" =>
                    vec![(Actor::Player, EffectKind::Damage(DamagePayload::FullEnemyMove(*id)))],
                (ActorEnemyId::Full(crate::content::EnemyId::JawWorm), ActorIntentId::Full(id)) if id.as_str() == "thrash" =>
                    vec![(Actor::Player, EffectKind::Damage(DamagePayload::FullEnemyMove(*id))), (actor.clone(), EffectKind::GainBlock(5))],
                (ActorEnemyId::Full(crate::content::EnemyId::JawWorm), ActorIntentId::Full(id)) if id.as_str() == "bellow" =>
                    vec![(actor.clone(), EffectKind::ApplyFullPower { id: super::FullPowerId("strength"), amount: 3 }), (actor.clone(), EffectKind::GainBlock(6))],
                _ => return Err(InputError::Unsupported(format!("unhandled full enemy intent {}", enemy.intent_id.as_str())).into()),
            };
            if let Some(move_id)=observed_move { super::observe::emit(state, &Source::Actor(actor.clone()), None,
                || ResolvedEffectKind::EnemyMove { actor:actor.clone(), move_id }); }
            if !state.full_runtime {
                state.pending.push_front(work(EffectKind::EndRound));
                state.pending.push_front(Effect { source: Source::Actor(actor.clone()),
                    target: None, kind: EffectKind::RollMove });
            } else {
                state.pending.push_front(Effect { source: Source::Actor(actor.clone()),
                    target: Some(Actor::Enemy(enemy.instance_id.clone())), kind: EffectKind::RollMove });
            }
            for (target, kind) in payloads.into_iter().rev() {
                state.pending.push_front(Effect { source: Source::Actor(actor.clone()), target: Some(target), kind });
            }
        }
        EffectKind::RollMove => {
            let owner = if state.full_runtime {
                match effect.target.as_ref() { Some(Actor::Enemy(id)) => Some(id), _ => return Err(InputError::Invariant("full move roll missing actor".into()).into()) }
            } else { None };
            let enemy = if state.full_runtime {
                let id = owner.ok_or_else(|| InputError::Invariant("full move roll missing actor".into()))?;
                let enemy = state.enemies.iter().find(|enemy| &enemy.instance_id == id)
                    .ok_or_else(|| InputError::Invariant("full move roll actor missing".into()))?;
                if enemy.life!=super::ActorLife::HalfDead && (enemy.life != super::ActorLife::Active || enemy.hp <= 0) { return Ok(()); }
                enemy
            } else { &state.enemies[0] };
            if matches!(enemy.enemy_id,ActorEnemyId::Full(_)) {
                let owner=owner.ok_or_else(||InputError::Invariant("full move selection actor missing".into()))?;
                super::enemies::act2::roll_move(state,owner)?;
            } else {
                let enemy = if state.full_runtime {
                    let id = owner.ok_or_else(|| InputError::Invariant("full move roll missing actor".into()))?;
                    let enemy = state.enemies.iter().find(|enemy| &enemy.instance_id == id)
                        .ok_or_else(|| InputError::Invariant("full move roll actor missing".into()))?;
                    if enemy.life != super::ActorLife::Active || enemy.hp <= 0 { return Ok(()); }
                    enemy
                } else { &state.enemies[0] };
                let kind = enemy.enemy_id.legacy().ok_or_else(|| InputError::Invariant("legacy enemy identity missing".into()))?;
                let history = enemy.move_history.iter().map(ActorIntentId::legacy).collect::<Option<Vec<_>>>()
                    .ok_or_else(|| InputError::Invariant("mixed legacy enemy move history".into()))?;
                let result = rules::choose_enemy_move(kind, &history, &state.rng.enemy_ai);
                install_selection(state, owner, result)?;
            }
        }
        EffectKind::EndRound => {
            state.phase = Phase::RoundEnd;
            if state.full_runtime {
                state.pending.push_front(work(EffectKind::StartNextPlayer));
                state.pending.push_front(Effect { source: Source::Actor(Actor::Player),
                    target: Some(Actor::Player), kind: EffectKind::Hook(Hook::TurnEnded) });
            } else {
                // Preserve the shipped legacy turn-end power path and its ordering.
                legacy_round_hooks(state);
                schedule_next_player_turn(state)?;
            }
        }
        EffectKind::StartNextPlayer => {
            if !state.full_runtime { return Err(InputError::Invariant("legacy next-player continuation".into()).into()); }
            schedule_next_player_turn(state)?;
        }
        _ => return Err(InputError::Invariant("turn owner received non-turn work".into()).into()),
    }
    Ok(())
}

fn legacy_intent(enemy: EnemyKind, intent: super::FullIntentId) -> Option<MoveId> {
    let id = match intent.as_str() { "incantation" => MoveId::Incantation, "attack" => MoveId::Attack,
        "chomp" => MoveId::Chomp, "thrash" => MoveId::Thrash, "bellow" => MoveId::Bellow, _ => return None };
    if super::engine::supports_move_for_turn(enemy, id) { Some(id) } else { None }
}

fn install_full_selection(state: &mut State, owner: &EnemyHandle,
    result: Result<(MoveId, StreamState), rules::SelectionError>) -> Result<(), EffectError> {
    let (chosen, next) = result.map_err(|error| match error {
        rules::SelectionError::Invalid(error) => EffectError::Invalid(error),
        rules::SelectionError::Random(error) => EffectError::Random(error),
    })?;
    let enemy = state.enemies.iter_mut().find(|e| &e.instance_id == owner)
        .ok_or_else(|| InputError::Invariant("full move selection actor missing".into()))?;
    *enemy.full_ai_rng.as_mut().ok_or_else(|| InputError::Invariant("full AI stream missing".into()))? = next;
    let intent = super::FullIntentId::from_name(chosen.as_str());
    enemy.selected_move = ActorIntentId::Full(intent);
    enemy.move_history.push(ActorIntentId::Full(intent));
    Ok(())
}
