//! Native handlers for the card families implemented in this module.
use super::*;
use super::model::State;
mod ironclad;
mod shared;
mod silent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefectContinuation {
    GoForTheEyesWeak { amount: i32 },
    BarrageStart { damage: i32 },
    BarrageNext { damage: i32, remaining: u8 },
    CompileDriverDraw,
}

pub(super) fn apply_defect_continuation(state: &mut State, effect: &Effect,
    continuation: &DefectContinuation) -> Result<(), crate::combat::EffectError> {
    let target = effect.target.as_ref();
    let source = effect.source.clone();
    match continuation {
        DefectContinuation::GoForTheEyesWeak { amount } => {
            let Some(Actor::Enemy(id)) = target else { return Err(crate::combat::InputError::Invariant(
                "Go for the Eyes continuation target missing".into()).into()) };
            let enemy = state.enemies.iter().find(|enemy| &enemy.instance_id == id)
                .ok_or_else(|| crate::combat::InputError::Invariant("Go for the Eyes target disappeared".into()))?;
            let intent_base = match &enemy.intent_id {
                ActorIntentId::Legacy(intent) => enemy.damage_entries.get(intent).map(|entry| entry.base),
                ActorIntentId::Full(intent) => enemy.full_damage_entries.get(intent).map(|entry| entry.base),
            };
            if intent_base.is_some_and(|damage| damage >= 0) {
                state.pending.push_front(Effect { source, target: Some(Actor::Enemy(id.clone())),
                    kind: EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: *amount } });
            }
        }
        DefectContinuation::BarrageStart { damage } => {
            let Some(Actor::Enemy(id)) = target else { return Err(crate::combat::InputError::Invariant(
                "Barrage continuation target missing".into()).into()) };
            let count = state.character.orb_slots.iter().flatten().count();
            if count > 0 && state.enemies.iter().any(|enemy|
                &enemy.instance_id == id && enemy.life == ActorLife::Active && enemy.hp > 0) {
                if count > 1 {
                    state.pending.push_front(Effect { source: source.clone(), target: Some(Actor::Enemy(id.clone())),
                        kind: EffectKind::CardContinuation(DefectContinuation::BarrageNext {
                            damage: *damage, remaining: (count - 1) as u8 }) });
                }
                state.pending.push_front(Effect { source, target: Some(Actor::Enemy(id.clone())),
                    kind: EffectKind::Damage(DamagePayload::Fixed(*damage)) });
            }
        }
        DefectContinuation::BarrageNext { damage, remaining } => {
            let Some(Actor::Enemy(id)) = target else { return Err(crate::combat::InputError::Invariant(
                "Barrage continuation target missing".into()).into()) };
            if state.enemies.iter().any(|enemy|
                &enemy.instance_id == id && enemy.life == ActorLife::Active && enemy.hp > 0) {
                if *remaining > 1 {
                    state.pending.push_front(Effect { source: source.clone(), target: Some(Actor::Enemy(id.clone())),
                        kind: EffectKind::CardContinuation(DefectContinuation::BarrageNext {
                            damage: *damage, remaining: remaining - 1 }) });
                }
                state.pending.push_front(Effect { source, target: Some(Actor::Enemy(id.clone())),
                    kind: EffectKind::Damage(DamagePayload::Fixed(*damage)) });
            }
        }
        DefectContinuation::CompileDriverDraw => {
            let mut kinds = Vec::new();
            for orb in state.character.orb_slots.iter().flatten() {
                if !kinds.contains(orb) { kinds.push(*orb); }
            }
            let kinds = kinds.len() as i32;
            if kinds > 0 {
                state.pending.push_front(Effect { source, target: None, kind: EffectKind::Draw(kinds) });
            }
        }
    }
    Ok(())
}

pub(super) struct PreparedCard {
    pub hand_index: usize,
    pub instance: CardInstanceId,
    pub remaining_energy: i32,
    pub target: Option<Actor>,
    pub effects: Vec<Effect>,
}

/// Validate a full-card play and construct its ordered native effects without
/// mutating combat state. The engine owns the single commit and resolver drain.
pub(super) fn prepare(combat: &Combat, action: &CombatAction) -> Result<PreparedCard, crate::run::RunError> {
    let CombatAction::PlayCard { instance, target } = action else {
        return Err(crate::run::RunError::InvalidAction);
    };
    let handle = CardHandle::parse(&instance.to_string())
        .map_err(|_| crate::run::RunError::InvalidAction)?;
    if matches!(target, Some(TargetId(0))) { return Err(crate::run::RunError::InvalidAction); }
    let (hand_index, owned) = combat.state.piles.hand.iter().enumerate()
        .find(|(_, card)| card.handle() == handle)
        .ok_or(crate::run::RunError::InvalidAction)?;
    let card = owned.full().ok_or(crate::run::RunError::InvalidAction)?;
    if matches!(card.id, crate::content::CardId::Defend | crate::content::CardId::DefendGreen
        | crate::content::CardId::DefendBlue | crate::content::CardId::DefendPurple
        | crate::content::CardId::Survivor | crate::content::CardId::Zap
        | crate::content::CardId::Dualcast | crate::content::CardId::Vigilance)
        && target.is_some() {
        return Err(crate::run::RunError::InvalidAction);
    }
    if matches!(card.id, crate::content::CardId::Cleave | crate::content::CardId::Thunderclap
        | crate::content::CardId::ShrugItOff | crate::content::CardId::Miracle | crate::content::CardId::Slimed
        | crate::content::CardId::Backflip | crate::content::CardId::Deflect
        | crate::content::CardId::CloakAndDagger | crate::content::CardId::Consecrate)
        && target.is_some() {
        return Err(crate::run::RunError::InvalidAction);
    }
    if matches!(card.id, crate::content::CardId::Coolheaded | crate::content::CardId::Leap
        | crate::content::CardId::EmptyBody | crate::content::CardId::EmptyMind)
        && target.is_some() {
        return Err(crate::run::RunError::InvalidAction);
    }
    if matches!(card.id, crate::content::CardId::SweepingBeam) && target.is_some() {
        return Err(crate::run::RunError::InvalidAction);
    }
    let (cost, payloads) = payload(&combat.state, card, target.map(|id| id.0))?;
    let payment = super::card_state::payment(card, combat.state.player.energy, cost)
        .map_err(|_| crate::run::RunError::InvalidAction)?;
    let actor = match target {
        Some(TargetId(id)) if *id > 0 => Some(combat.state.enemies.get((*id - 1) as usize)
            .filter(|enemy| enemy.hp > 0).map(|enemy| Actor::Enemy(enemy.instance_id.clone()))
            .ok_or(crate::run::RunError::InvalidAction)?),
        Some(_) => return Err(crate::run::RunError::InvalidAction),
        None => None,
    };
    let source = Source::Card(handle);
    let effects = payloads.into_iter().map(|(target, kind)| Effect {
        source: source.clone(), target: target.map(|target| match target {
            PayloadTarget::Player => Actor::Player,
            PayloadTarget::SelectedEnemy => actor.clone().expect("enemy target validated"),
            PayloadTarget::Enemy(id) => Actor::Enemy(id),
        }), kind,
    }).collect();
    Ok(PreparedCard { hand_index, instance: card.instance,
        remaining_energy: payment.remaining_energy, target: actor, effects })
}

/// Public card adapter. Native play validation and commit remain in
/// `Combat::step_full`; that path delegates here to `prepare` exactly once.
pub fn resolve(combat: &mut Combat, action: &CombatAction) -> Result<(), crate::run::RunError> {
    combat.step_full(action)
}

#[derive(Clone)]
enum PayloadTarget { Player, SelectedEnemy, Enemy(EnemyHandle) }

fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<(i32, Vec<(Option<PayloadTarget>, EffectKind)>), crate::run::RunError>
{
    if let Some(result) = ironclad::payload(state, card, target)? { return Ok(result); }
    if let Some(result) = shared::payload(card)? { return Ok(result); }
    if let Some(result) = silent::payload(state, card, target)? { return Ok(result); }
    use crate::content::CardId::*;
    let upgraded = card.upgrades > 0;
    let (cost, effects) = match card.id {
        IronWave => {
            if target.is_none() { return Err(crate::run::RunError::InvalidAction); }
            let block = modified_block(state, if upgraded { 7 } else { 5 })?;
            let damage = modified_damage(state, target.unwrap(), if upgraded { 7 } else { 5 })?;
            (1, vec![ (Some(PayloadTarget::Player), EffectKind::GainBlock(block)),
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))) ])
        }
        Strike | StrikeGreen | StrikeBlue | StrikePurple => {
            if target.is_none() { return Err(crate::run::RunError::InvalidAction); }
            let damage = modified_damage(state, target.unwrap(), if upgraded { 9 } else { 6 })?;
            (1, vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage)))])
        }
        Defend | DefendGreen | DefendBlue | DefendPurple => {
            (1, vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 8 } else { 5 })?))])
        }
        Neutralize => {
            if target.is_none() { return Err(crate::run::RunError::InvalidAction); }
            let damage = modified_damage(state, target.unwrap(), if upgraded { 4 } else { 3 })?;
            (0, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: if upgraded { 2 } else { 1 } }),
            ])
        }
        Survivor => {
            let candidates = state.piles.hand.iter().filter(|owned| owned.handle() != card_handle(card))
                .map(|owned| owned.handle()).collect::<Vec<_>>();
            (1, vec![
                (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 11 } else { 8 })?)),
                (None, EffectKind::SelectCards { candidates, required: 1,
                    disposition: super::SelectionDisposition::Discard,
                    empty: super::EmptySelection::Continue }),
            ])
        }
        Zap => (if upgraded { 0 } else { 1 }, vec![
            (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Channel(super::characters::OrbKind::Lightning)))
        ]),
        BallLightning | ColdSnap => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, match card.id {
                BallLightning => if upgraded { 10 } else { 7 },
                ColdSnap => if upgraded { 9 } else { 6 },
                _ => unreachable!(),
            })?;
            let orb = if card.id == BallLightning {
                super::characters::OrbKind::Lightning
            } else { super::characters::OrbKind::Frost };
            (1, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Channel(orb))),
            ])
        }
        BeamCell | GoForTheEyes => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 4 } else { 3 })?;
            let mut effects = vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(
                DamagePayload::Fixed(damage)))];
            if card.id == BeamCell {
                effects.push((Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower {
                    id: FullPowerId("vulnerable"), amount: if upgraded { 2 } else { 1 } }));
            } else {
                effects.push((Some(PayloadTarget::SelectedEnemy), EffectKind::CardContinuation(
                    DefectContinuation::GoForTheEyesWeak { amount: if upgraded { 2 } else { 1 } })));
            }
            (0, effects)
        }
        Barrage => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 6 } else { 4 })?;
            (1, vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::CardContinuation(
                DefectContinuation::BarrageStart { damage }))])
        }
        CompileDriver => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 10 } else { 7 })?;
            (1, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (None, EffectKind::CardContinuation(DefectContinuation::CompileDriverDraw)),
            ])
        }
        SweepingBeam => {
            let mut effects = Vec::new();
            for (index, enemy) in state.enemies.iter().enumerate()
                .filter(|(_, enemy)| enemy.life == ActorLife::Active && enemy.hp > 0) {
                let damage = modified_damage(state, index as u64 + 1, if upgraded { 9 } else { 6 })?;
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            effects.push((None, EffectKind::Draw(1)));
            (1, effects)
        }
        Coolheaded => {
            (1, vec![
                (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Channel(super::characters::OrbKind::Frost))),
                (None, EffectKind::Draw(if upgraded { 2 } else { 1 })),
            ])
        }
        Leap => (1, vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 12 } else { 9 })?))]),
        Dualcast => (if upgraded { 0 } else { 1 }, vec![
            (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::EvokeFront { remove: false })),
            (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::EvokeFront { remove: true })),
        ]),
        Eruption => {
            if target.is_none() { return Err(crate::run::RunError::InvalidAction); }
            let damage = modified_damage(state, target.unwrap(), 9)?;
            (if upgraded { 1 } else { 2 }, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Stance(super::characters::Stance::Wrath))),
            ])
        }
        EmptyFist => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 14 } else { 9 })?;
            (1, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::Player), EffectKind::Character(
                    super::characters::CharacterEffect::Stance(super::characters::Stance::Neutral))),
            ])
        }
        EmptyBody => (1, vec![
            (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 10 } else { 7 })?)),
            (Some(PayloadTarget::Player), EffectKind::Character(
                super::characters::CharacterEffect::Stance(super::characters::Stance::Neutral))),
        ]),
        EmptyMind => (1, vec![
            (None, EffectKind::Draw(if upgraded { 3 } else { 2 })),
            (Some(PayloadTarget::Player), EffectKind::Character(
                super::characters::CharacterEffect::Stance(super::characters::Stance::Neutral))),
        ]),
        Consecrate => {
            if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
            let mut effects = Vec::new();
            for (index, enemy) in state.enemies.iter().enumerate().filter(|(_, enemy)| enemy.hp > 0) {
                let damage = modified_damage(state, index as u64 + 1, if upgraded { 8 } else { 5 })?;
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            (0, effects)
        }
        Vigilance => (2, vec![
            (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 12 } else { 8 })?)),
            (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Stance(super::characters::Stance::Calm))),
        ]),
        _ => return Err(crate::run::RunError::Unsupported("card handler not implemented")),
    };
    Ok((cost, effects))
}

pub(super) fn normal_damage(state: &State, base: i32, strength: i32, weak: bool, vulnerable: bool)
    -> Result<i32, crate::run::RunError>
{
    if !(0..=3003).contains(&base) || !(-9990..=9990).contains(&strength) {
        return Err(crate::run::RunError::Unsupported("normal damage outside supported numeric surface"));
    }
    let combined_base = base.checked_add(strength)
        .ok_or(crate::run::RunError::Unsupported("full card damage arithmetic overflow"))?;
    if combined_base > 10004 {
        return Err(crate::run::RunError::Unsupported("full card damage exceeds supported numeric surface"));
    }
    rules::full_normal_damage_modifiers(combined_base.max(0), 0, weak,
        state.character.stance == super::characters::Stance::Wrath, vulnerable)
        .map_err(|_| crate::run::RunError::Unsupported("invalid card damage"))
}

fn modified_damage(state: &State, target: u64, base: i32) -> Result<i32, crate::run::RunError> {
    let enemy = state.enemies.get((target - 1) as usize)
        .filter(|enemy| enemy.hp > 0).ok_or(crate::run::RunError::InvalidAction)?;
    let strength = state.player.powers.iter().find(|p| p.id == PowerId::Strength)
        .map_or(0, |p| p.state.amount)
        + state.player_full_powers.iter().filter(|p| p.id.0 == "strength").map(|p| p.state.amount).sum::<i32>();
    let vulnerable = enemy.powers.iter().any(|p| p.id == PowerId::Vulnerable && p.state.amount > 0)
        || enemy.full_powers.iter().any(|p| p.id.0 == "vulnerable" && p.state.amount > 0);
    let weak = state.player_full_powers.iter().any(|p| p.id.0 == "weak" && p.state.amount > 0);
    normal_damage(state, base, strength, weak, vulnerable)
}

fn modified_block(state: &State, base: i32) -> Result<i32, crate::run::RunError> {
    let dexterity = state.player_full_powers.iter().find(|p| p.id.0 == "dexterity").map_or(0, |p| p.state.amount);
    let frail = state.player_full_powers.iter().any(|p| p.id.0 == "frail" && p.state.amount > 0);
    rules::full_card_block(base, dexterity, frail)
        .map_err(|_| crate::run::RunError::Unsupported("invalid calculated card Block"))
}

fn card_handle(card: &LiveCard) -> CardHandle {
    super::card_state::handle(card.instance)
}
