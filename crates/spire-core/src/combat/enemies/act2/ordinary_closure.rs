//! A0 Shelled Parasite and Snake Plant producers on the shared combat resolver.
use super::*;

const SHELLED_DOUBLE_STRIKE: &str = "shelled_parasite_double_strike";
const SHELLED_LIFE_SUCK: &str = "shelled_parasite_life_suck";
const SHELLED_FELL: &str = "shelled_parasite_fell";
const SHELLED_STUNNED: &str = "shelled_parasite_stunned";
const SNAKE_CHOMP_CHOMP: &str = "snake_plant_chomp_chomp";
const SNAKE_ENFEEBLING_SPORES: &str = "snake_plant_enfeebling_spores";
const PLATED_ARMOR: &str = "plated_armor";
const MALLEABLE: &str = "malleable";

pub(in crate::combat) fn supports(id: EnemyId) -> bool {
    matches!(id, EnemyId::ShelledParasite | EnemyId::SnakePlant)
}

pub(in crate::combat) fn valid_intent(id: EnemyId, name: &str) -> bool {
    matches!(
        (id, name),
        (EnemyId::ShelledParasite, SHELLED_DOUBLE_STRIKE | SHELLED_LIFE_SUCK | SHELLED_FELL | SHELLED_STUNNED)
            | (EnemyId::SnakePlant, SNAKE_CHOMP_CHOMP | SNAKE_ENFEEBLING_SPORES)
    )
}

pub(in crate::combat) fn select_move(
    id: EnemyId,
    history: &[FullIntentId],
    memory: &EnemyMemory,
    stream: &StreamState,
) -> Result<MoveSelection, SelectError> {
    let mut cursor = stream.clone();
    let (name, memory) = match (id, memory) {
        (EnemyId::ShelledParasite, EnemyMemory::ShelledParasite { first }) => {
            if *first {
                let (ticket, next) = rng::uniform_below(&cursor, 2)?;
                cursor = next;
                let name = if ticket == 0 { SHELLED_DOUBLE_STRIKE } else { SHELLED_LIFE_SUCK };
                (name, EnemyMemory::ShelledParasite { first: false })
            } else {
                let (ticket, next) = rng::uniform_below(&cursor, 100)?;
                cursor = next;
                let name = if ticket < 20 {
                    if history.last().is_some_and(|move_id| move_id.as_str() == SHELLED_FELL) {
                        let (reroll, next) = rng::uniform_below(&cursor, 80)?;
                        cursor = next;
                        choose_shelled_without_fell(history, reroll + 20)
                    } else {
                        SHELLED_FELL
                    }
                } else {
                    choose_shelled_without_fell(history, ticket)
                };
                (name, memory.clone())
            }
        }
        (EnemyId::SnakePlant, EnemyMemory::SnakePlant) => {
            let (ticket, next) = rng::uniform_below(&cursor, 100)?;
            cursor = next;
            let repeats_chomp = history.ends_with(&[FullIntentId::from_name(SNAKE_CHOMP_CHOMP); 2]);
            let last_was_spores = history.last().is_some_and(|move_id| move_id.as_str() == SNAKE_ENFEEBLING_SPORES);
            let name = if ticket < 65 {
                if repeats_chomp { SNAKE_ENFEEBLING_SPORES } else { SNAKE_CHOMP_CHOMP }
            } else if last_was_spores {
                SNAKE_CHOMP_CHOMP
            } else {
                SNAKE_ENFEEBLING_SPORES
            };
            (name, memory.clone())
        }
        _ => return Err(InputError::Invariant("ordinary Act2 closure memory mismatch".into()).into()),
    };
    Ok(MoveSelection { intent: FullIntentId::from_name(name), memory, stream: cursor })
}

fn choose_shelled_without_fell(history: &[FullIntentId], ticket: u64) -> &'static str {
    let repeated_double_strike = history.ends_with(&[FullIntentId::from_name(SHELLED_DOUBLE_STRIKE); 2]);
    let repeated_life_suck = history.ends_with(&[FullIntentId::from_name(SHELLED_LIFE_SUCK); 2]);
    if ticket < 60 {
        if repeated_double_strike { SHELLED_LIFE_SUCK } else { SHELLED_DOUBLE_STRIKE }
    } else if repeated_life_suck {
        SHELLED_DOUBLE_STRIKE
    } else {
        SHELLED_LIFE_SUCK
    }
}

pub(in crate::combat) fn move_work(enemy: &EnemyState) -> Result<Vec<Effect>, InputError> {
    let ActorEnemyId::Full(id) = enemy.enemy_id else {
        return Err(InputError::Unsupported("ordinary Act2 closure requires a full actor".into()));
    };
    if !valid_intent(id, enemy.intent_id.as_str()) {
        return Err(InputError::Unsupported("ordinary Act2 closure intent/species mismatch".into()));
    }
    let ActorIntentId::Full(intent) = enemy.intent_id else {
        return Err(InputError::Invariant("ordinary Act2 closure requires a full intent".into()));
    };
    let actor = Actor::Enemy(enemy.instance_id.clone());
    let work = match (id, intent.as_str()) {
        (EnemyId::ShelledParasite, SHELLED_DOUBLE_STRIKE) => {
            vec![damage(actor.clone(), intent), damage(actor, intent)]
        }
        (EnemyId::ShelledParasite, SHELLED_LIFE_SUCK | SHELLED_FELL) => {
            let mut work = vec![damage(actor.clone(), intent)];
            if intent.as_str() == SHELLED_FELL {
                work.push(effect(actor, Actor::Player, EffectKind::ApplyFullPower {
                    id: FullPowerId("frail"), amount: 2,
                }));
            }
            work
        }
        (EnemyId::ShelledParasite, SHELLED_STUNNED) => Vec::new(),
        (EnemyId::SnakePlant, SNAKE_CHOMP_CHOMP) => (0..3).map(|_| damage(actor.clone(), intent)).collect(),
        (EnemyId::SnakePlant, SNAKE_ENFEEBLING_SPORES) => vec![
            effect(actor.clone(), Actor::Player, EffectKind::ApplyFullPower { id: FullPowerId("frail"), amount: 2 }),
            effect(actor, Actor::Player, EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 2 }),
        ],
        _ => return Err(InputError::Unsupported("ordinary Act2 closure move unavailable".into())),
    };
    if id == EnemyId::SnakePlant {
        let owner = Actor::Enemy(enemy.instance_id.clone());
        let mut work = work;
        work.push(effect(owner.clone(), owner, EffectKind::ResetMalleable));
        Ok(work)
    } else {
        Ok(work)
    }
}

/// Resolve the actual selected turn where the actor needs a source-specific
/// state update before the shared scheduler's later RollMove continuation.
pub(in crate::combat) fn take_turn(
    state: &mut State,
    owner: &EnemyHandle,
) -> Result<Vec<Effect>, InputError> {
    let enemy = state.enemies.iter().find(|enemy| &enemy.instance_id == owner)
        .ok_or_else(|| InputError::Invariant("ordinary Act2 closure turn owner absent".into()))?
        .clone();
    let ActorEnemyId::Full(id) = enemy.enemy_id else {
        return Err(InputError::Unsupported("ordinary Act2 closure requires a full actor".into()));
    };
    if !supports(id) || !valid_intent(id, enemy.intent_id.as_str()) {
        return Err(InputError::Unsupported("ordinary Act2 closure turn intent/species mismatch".into()));
    }
    if id == EnemyId::ShelledParasite && enemy.intent_id.as_str() == SHELLED_STUNNED {
        // Native takeTurn case 4 sets Fell before queueing RollMoveAction. The
        // shared setter records it as history, so that the subsequent getMove
        // suppresses a second consecutive Fell proposal correctly.
        super::gremlin_leader::set_move(state, owner, FullIntentId::from_name(SHELLED_FELL))?;
        return Ok(Vec::new());
    }
    move_work(&enemy)
}

/// Return the enemy-owned Plated Armor callback work for the pre-end-turn-card stage.
pub(in crate::combat) fn plated_armor_end_turn_work(enemy: &EnemyState) -> Option<Effect> {
    let amount = enemy.full_powers.iter().find(|power| power.id.0 == PLATED_ARMOR)?.state.amount;
    if amount <= 0 || enemy.life != super::super::super::ActorLife::Active || enemy.hp <= 0 {
        return None;
    }
    let actor = Actor::Enemy(enemy.instance_id.clone());
    Some(effect(actor.clone(), actor, EffectKind::GainBlock(amount)))
}

/// Apply the source-ordered callbacks after each committed shared damage result.
pub(in crate::combat) fn after_damage(
    state: &mut State,
    result: &super::super::super::model::DamageResult,
) -> Result<(), InputError> {
    if let Actor::Enemy(owner) = &result.target {
        let loses_plated_armor = result.hp_lost > 0 && !result.hp_loss && !result.thorns
            && source_is_nonnull_and_external(&result.source, &result.target)
            && state.enemies.iter().find(|enemy| &enemy.instance_id == owner)
                .is_some_and(|enemy| enemy.full_powers.iter()
                    .any(|power| power.id.0 == PLATED_ARMOR && power.state.amount > 0));
        if loses_plated_armor {
            let actor = Actor::Enemy(owner.clone());
            super::bottom(state, effect(actor.clone(), actor, EffectKind::ReduceFullPower {
                id: FullPowerId(PLATED_ARMOR), amount: 1,
            }));
        }

        let malleable_block = {
            let enemy = state.enemies.iter_mut().find(|enemy| &enemy.instance_id == owner)
                .ok_or_else(|| InputError::Invariant("ordinary Act2 closure damage owner absent".into()))?;
            if enemy.enemy_id == ActorEnemyId::Full(EnemyId::SnakePlant)
                && result.normal && result.source != Source::None && result.hp_lost > 0 && result.hp_after > 0
                && enemy.life == super::super::super::ActorLife::Active
            {
                if let Some(power) = enemy.full_powers.iter_mut().find(|power| power.id.0 == MALLEABLE) {
                    let block = power.state.amount;
                    power.state.amount = block.checked_add(1).filter(|next| *next <= 999)
                        .ok_or_else(|| InputError::Unsupported("Malleable amount exceeds native bounds".into()))?;
                    Some(block)
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(block) = malleable_block {
            let actor = Actor::Enemy(owner.clone());
            super::bottom(state, effect(actor.clone(), actor, EffectKind::GainBlock(block)));
        }
    }

    Ok(())
}

/// Queue VampireDamageAction's follow-up after the shared Damage path has run
/// all damage, retaliation, power, death, and hook callbacks. The original
/// action uses addToTop for HealAction, so this continuation must be last in
/// that producer path and push its child to the shared queue front.
pub(in crate::combat) fn vampire_damage_complete(
    state: &mut State,
    result: &super::super::super::model::DamageResult,
    payload: &DamagePayload,
) {
    if result.target != Actor::Player || !result.normal
        || !matches!(payload, DamagePayload::FullEnemyMove(intent) if intent.as_str() == SHELLED_LIFE_SUCK)
    { return; }
    let Source::Actor(Actor::Enemy(owner)) = &result.source else { return; };
    let Some(enemy) = state.enemies.iter().find(|enemy| &enemy.instance_id == owner
        && enemy.enemy_id == ActorEnemyId::Full(EnemyId::ShelledParasite)) else { return; };

    // AbstractPlayer.damage stores lastDamageTaken as min(post-Block damage,
    // currentHealth), before later onLoseHp callbacks. Rebuild that value from
    // the shared result's attempted/blocked pair and pre-hit HP; hp_lost alone
    // can differ on overkill or when a later callback changes committed loss.
    let damage_after_block = result.attempted.saturating_sub(result.blocked).max(0);
    let hp_before = result.hp_after.saturating_add(result.hp_lost).max(0);
    let last_damage_taken = damage_after_block.min(hp_before);
    if last_damage_taken > 0 {
        let actor = Actor::Enemy(enemy.instance_id.clone());
        state.pending.push_front(effect(actor.clone(), actor, EffectKind::Heal(last_damage_taken)));
    }
}

fn source_is_nonnull_and_external(source: &Source, target: &Actor) -> bool {
    match source {
        Source::None => false,
        Source::Actor(actor) => actor != target,
        Source::Card(_) | Source::Potion { .. } => true,
    }
}

pub(in crate::combat) fn reset_malleable(state: &mut State, owner: &EnemyHandle) {
    if let Some(enemy) = state.enemies.iter_mut().find(|enemy| &enemy.instance_id == owner
        && enemy.enemy_id == ActorEnemyId::Full(EnemyId::SnakePlant))
    {
        if let Some(power) = enemy.full_powers.iter_mut().find(|power| power.id.0 == MALLEABLE) {
            power.state.amount = 3;
        }
    }
}

/// Apply Shelled Parasite's queued ARMOR_BREAK state transition. The shared
/// RemoveFullPower path must enqueue its dedicated native-bottom continuation;
/// this is called only when that continuation executes, after the source's
/// queued ChangeStateAction boundary.
pub(in crate::combat) fn armor_break(state: &mut State, owner: &EnemyHandle) -> Result<(), InputError> {
    let enemy = state.enemies.iter().find(|enemy| &enemy.instance_id == owner)
        .ok_or_else(|| InputError::Invariant("Shelled Parasite armor-break owner absent".into()))?;
    if enemy.enemy_id != ActorEnemyId::Full(EnemyId::ShelledParasite) {
        return Err(InputError::Invariant("Shelled Parasite armor-break species mismatch".into()));
    }
    super::gremlin_leader::set_move(state, owner, FullIntentId::from_name(SHELLED_STUNNED))
}
