//! Qualified Act1 Acid/Spike Slime setup, move selection and payloads (A0 facts).
use super::*;

const BOSS_STICKY: &str = "sticky";
const BOSS_PREP: &str = "prep_slam";
const BOSS_SLAM: &str = "slam";
const BOSS_SPLIT: &str = "split";

pub(super) fn select_slime_boss(history: &[FullIntentId], memory: &EnemyMemory,
    stream: &StreamState) -> Result<MoveSelection, SelectError> {
    let moves = [BOSS_STICKY, BOSS_PREP, BOSS_SLAM, BOSS_SPLIT];
    if history.iter().any(|intent| !moves.contains(&intent.as_str())) {
        return Err(InputError::Unsupported("Slime Boss history/identity mismatch".into()).into());
    }
    let intent = match history.last().map(|intent| intent.as_str()) {
        None | Some(BOSS_SLAM) => BOSS_STICKY,
        Some(BOSS_STICKY) => BOSS_PREP,
        Some(BOSS_PREP) => BOSS_SLAM,
        Some(BOSS_SPLIT) => BOSS_SPLIT,
        _ => return Err(InputError::Invariant("Slime Boss history is empty or invalid".into()).into()),
    };
    if matches!(memory, EnemyMemory::SlimeBoss { split_triggered: true }) && intent != BOSS_SPLIT {
        return Err(InputError::Invariant("Slime Boss split latch/history mismatch".into()).into());
    }
    Ok(MoveSelection { intent: FullIntentId::from_name(match intent {
        BOSS_STICKY => BOSS_STICKY, BOSS_PREP => BOSS_PREP, BOSS_SLAM => BOSS_SLAM,
        _ => BOSS_SPLIT,
    }), stream: stream.clone(), memory: memory.clone() })
}

pub(super) fn select_acid(id: EnemyId, history: &[FullIntentId], memory: &EnemyMemory,
    stream: &StreamState) -> Result<MoveSelection, SelectError> {
    use FullIntentId::{AcidLick, AcidSpit, AcidTackle};
    if stream.id().domain() != rng::Domain::RunCombatEnemyAi {
        return Err(InputError::Unsupported("Acid Slime requires actor-owned AI stream".into()).into());
    }
    let moves: &[FullIntentId] = match id {
        EnemyId::AcidSlimeS => &[AcidTackle, AcidLick],
        EnemyId::AcidSlimeM | EnemyId::AcidSlimeL => &[AcidSpit, AcidTackle, AcidLick],
        _ => return Err(InputError::Invariant("Acid selector identity mismatch".into()).into()),
    };
    if history.iter().any(|mv| !moves.contains(mv)) {
        return Err(InputError::Unsupported("Acid Slime history/identity mismatch".into()).into());
    }
    let (intent, next) = if id == EnemyId::AcidSlimeS {
        match history.last() {
            Some(AcidTackle) => (AcidLick, stream.clone()),
            Some(AcidLick) => (AcidTackle, stream.clone()),
            None => {
                let (roll, next) = rng::uniform_below(stream, 2)?;
                (if roll == 0 { AcidTackle } else { AcidLick }, next)
            }
            _ => return Err(InputError::Unsupported("Acid Slime S history mismatch".into()).into()),
        }
    } else {
        select_acid_medium(history, stream)?
    };
    Ok(MoveSelection { intent, stream: next, memory: memory.clone() })
}

fn select_acid_medium(history: &[FullIntentId], stream: &StreamState)
    -> Result<(FullIntentId, StreamState), SelectError> {
    use FullIntentId::{AcidLick, AcidSpit, AcidTackle};
    let (roll, mut cursor) = rng::uniform_below(stream, 100)?;
    let conditional = match roll {
        0..=29 if history.ends_with(&[AcidSpit, AcidSpit]) => Some((1, 2)),
        30..=69 if history.last() == Some(&AcidTackle) => Some((6_710_887, 16_777_216)),
        70..=99 if history.ends_with(&[AcidLick, AcidLick]) => Some((6_710_887, 16_777_216)),
        _ => None,
    };
    let redirect = if let Some((numerator, denominator)) = conditional {
        let (ticket, next) = rng::uniform_below(&cursor, denominator)?;
        cursor = next;
        Some(ticket < numerator)
    } else { None };
    let intent = match (roll, redirect) {
        (0..=29, Some(true)) => AcidTackle,
        (0..=29, Some(false)) => AcidLick,
        (0..=29, None) => AcidSpit,
        (30..=69, Some(true)) => AcidSpit,
        (30..=69, Some(false)) => AcidLick,
        (30..=69, None) => AcidTackle,
        (70..=99, Some(true)) => AcidSpit,
        (70..=99, Some(false)) => AcidTackle,
        (70..=99, None) => AcidLick,
        _ => unreachable!("bounded Acid Slime roll"),
    };
    Ok((intent, cursor))
}

pub(super) fn select_spike(id: EnemyId, history: &[FullIntentId], memory: &EnemyMemory,
    stream: &StreamState) -> Result<MoveSelection, SelectError> {
    use FullIntentId::{SpikeFlameTackle, SpikeLick, SpikeTackle};
    if stream.id().domain() != rng::Domain::RunCombatEnemyAi {
        return Err(InputError::Unsupported("Spike Slime requires actor-owned AI stream".into()).into());
    }
    let moves: &[FullIntentId] = match id {
        EnemyId::SpikeSlimeS => &[SpikeTackle],
        EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL => &[SpikeFlameTackle, SpikeLick],
        _ => return Err(InputError::Invariant("Spike selector identity mismatch".into()).into()),
    };
    if history.iter().any(|mv| !moves.contains(mv)) {
        return Err(InputError::Unsupported("Spike Slime history/identity mismatch".into()).into());
    }
    let (intent, next) = match id {
        EnemyId::SpikeSlimeS => (SpikeTackle, stream.clone()),
        EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL if history.ends_with(&[SpikeFlameTackle, SpikeFlameTackle]) =>
            (SpikeLick, stream.clone()),
        EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL if history.ends_with(&[SpikeLick, SpikeLick]) =>
            (SpikeFlameTackle, stream.clone()),
        EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL => {
            let (roll, next) = rng::uniform_below(stream, 100)?;
            (if roll < 30 { SpikeFlameTackle } else { SpikeLick }, next)
        }
        _ => unreachable!("identity checked"),
    };
    Ok(MoveSelection { intent, stream: next, memory: memory.clone() })
}

pub(super) fn move_work(enemy: &EnemyState, id: EnemyId, intent: FullIntentId)
    -> Result<Vec<Effect>, InputError> {
    let actor = Actor::Enemy(enemy.instance_id.clone());
    let work = match (id, intent) {
        (EnemyId::SlimeBoss, intent) if intent.as_str() == BOSS_STICKY => (0..3).map(|_| Effect {
            source: Source::Actor(actor.clone()), target: Some(Actor::Player), kind: EffectKind::CreateCard {
                id: crate::content::CardId::Slimed,
                destination: crate::combat::GeneratedCardDestination::Discard, upgrades: 0,
            }
        }).collect(),
        (EnemyId::SlimeBoss, intent) if intent.as_str() == BOSS_PREP => vec![],
        (EnemyId::SlimeBoss, intent) if intent.as_str() == BOSS_SLAM => vec![super::damage(actor, intent)],
        (EnemyId::SlimeBoss, intent) if intent.as_str() == BOSS_SPLIT => vec![Effect {
            source: Source::Actor(actor), target: None,
            kind: EffectKind::ReplaceEnemy { parent: enemy.instance_id.clone(),
                children: vec![EnemyId::SpikeSlimeL, EnemyId::AcidSlimeL] },
        }],
        (EnemyId::AcidSlimeS, FullIntentId::AcidTackle)
        | (EnemyId::AcidSlimeM, FullIntentId::AcidTackle)
        | (EnemyId::AcidSlimeL, FullIntentId::AcidTackle)
        | (EnemyId::SpikeSlimeS, FullIntentId::SpikeTackle) => vec![super::damage(actor, intent)],
        (EnemyId::AcidSlimeM | EnemyId::AcidSlimeL, FullIntentId::AcidSpit) => {
            let count = if id == EnemyId::AcidSlimeM { 1 } else { 2 };
            let mut effects = vec![super::damage(actor.clone(), intent)];
            effects.extend((0..count).map(|_| Effect { source: Source::Actor(actor.clone()),
                target: Some(Actor::Player), kind: EffectKind::CreateCard {
                    id: crate::content::CardId::Slimed,
                    destination: crate::combat::GeneratedCardDestination::Discard, upgrades: 0,
                } }));
            effects
        },
        (EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL, FullIntentId::SpikeFlameTackle) => {
            let count = if id == EnemyId::SpikeSlimeM { 1 } else { 2 };
            let mut effects = vec![super::damage(actor.clone(), intent)];
            effects.extend((0..count).map(|_| Effect { source: Source::Actor(actor.clone()),
                target: Some(Actor::Player), kind: EffectKind::CreateCard {
                    id: crate::content::CardId::Slimed,
                    destination: crate::combat::GeneratedCardDestination::Discard, upgrades: 0,
                } }));
            effects
        },
        (EnemyId::AcidSlimeS | EnemyId::AcidSlimeM, FullIntentId::AcidLick) => vec![
            super::effect(actor, Actor::Player, EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 1 })],
        (EnemyId::AcidSlimeL, FullIntentId::AcidLick) => vec![
            super::effect(actor, Actor::Player, EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 2 })],
        (EnemyId::SpikeSlimeM, FullIntentId::SpikeLick) => vec![
            super::effect(actor, Actor::Player, EffectKind::ApplyFullPower { id: FullPowerId("frail"), amount: 1 })],
        (EnemyId::SpikeSlimeL, FullIntentId::SpikeLick) => vec![
            super::effect(actor, Actor::Player, EffectKind::ApplyFullPower { id: FullPowerId("frail"), amount: 2 })],
        (EnemyId::AcidSlimeL, FullIntentId::AcidSplit) => super::split_work(enemy, EnemyId::AcidSlimeM),
        (EnemyId::SpikeSlimeL, FullIntentId::SpikeSplit) => super::split_work(enemy, EnemyId::SpikeSlimeM),
        _ => return Err(InputError::Unsupported(format!("unhandled slime intent {}", intent.as_str()))),
    };
    Ok(work)
}
