//! A0 Masked Bandits; these event enemies use fixed native move loops.
use super::*;

pub(in crate::combat::enemies) fn select_move(
    history: &[FullIntentId],
    memory: &EnemyMemory,
    stream: &StreamState,
) -> Result<MoveSelection, super::super::SelectError> {
    if !memory.valid_for(match memory {
        EnemyMemory::BanditPointy => EnemyId::BanditPointy,
        EnemyMemory::BanditLeader => EnemyId::BanditLeader,
        EnemyMemory::BanditBear => EnemyId::BanditBear,
        _ => return Err(InputError::Invariant("Bandit memory mismatch".into()).into()),
    }) {
        return Err(InputError::Invariant("Bandit memory mismatch".into()).into());
    }
    let name = match memory {
        EnemyMemory::BanditPointy => "pointy_special",
        EnemyMemory::BanditLeader if history.is_empty() => "romeo_mock",
        EnemyMemory::BanditLeader if history.len() % 2 == 1 => "romeo_agonizing_slash",
        EnemyMemory::BanditLeader => "romeo_cross_slash",
        EnemyMemory::BanditBear if history.is_empty() => "bear_hug",
        EnemyMemory::BanditBear if history.len() % 2 == 1 => "bear_lunge",
        EnemyMemory::BanditBear => "bear_maul",
        _ => return Err(InputError::Invariant("Bandit memory mismatch".into()).into()),
    };
    Ok(MoveSelection {
        intent: FullIntentId::from_name(name),
        memory: memory.clone(),
        stream: stream.clone(),
    })
}

pub(super) fn take_turn(enemy: &mut EnemyState, ascension: u8) -> Result<Vec<Effect>, InputError> {
    let actor = Actor::Enemy(enemy.instance_id.clone());
    let ActorIntentId::Full(intent) = enemy.intent_id else {
        return Err(InputError::Invariant("Bandit full intent absent".into()));
    };
    let work = match (enemy.enemy_id.clone(), intent.as_str()) {
        (ActorEnemyId::Full(EnemyId::BanditPointy), "pointy_special") => vec![
            damage(actor.clone(), intent),
            damage(actor, intent),
        ],
        (ActorEnemyId::Full(EnemyId::BanditLeader), "romeo_mock") => vec![],
        (ActorEnemyId::Full(EnemyId::BanditLeader), "romeo_agonizing_slash") => vec![
            damage(actor.clone(), intent),
            effect(actor, Actor::Player, EffectKind::ApplyFullPower {
                id: FullPowerId("weak"),
                amount: if ascension >= 17 { 3 } else { 2 },
            }),
        ],
        (ActorEnemyId::Full(EnemyId::BanditLeader), "romeo_cross_slash") => {
            vec![damage(actor, intent)]
        }
        (ActorEnemyId::Full(EnemyId::BanditBear), "bear_hug") => vec![
            effect(actor, Actor::Player, EffectKind::ApplyFullPower {
                id: FullPowerId("dexterity"),
                amount: if ascension >= 17 { -4 } else { -2 },
            }),
        ],
        (ActorEnemyId::Full(EnemyId::BanditBear), "bear_lunge") => vec![
            damage(actor.clone(), intent),
            effect(actor.clone(), actor, EffectKind::GainBlock(9)),
        ],
        (ActorEnemyId::Full(EnemyId::BanditBear), "bear_maul") => vec![
            damage(actor, intent),
        ],
        _ => return Err(InputError::Invariant("Bandit intent mismatch".into())),
    };
    Ok(work)
}
