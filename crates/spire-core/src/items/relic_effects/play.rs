//! Lane R2 (M27): card-play, damage and HP-loss relic bodies (plan Task R2).
#![allow(dead_code)]
use crate::combat::{Actor, CardDestination, DamagePayload, Effect, EffectKind, FullPowerId, GenerationCost,
    GenerationPool, GenerationWork, InputError, RelicEffect, Source};
use crate::content::{self, Character, CardType, RelicId};
use crate::run::{RunError, RunState};

use super::super::dispatch::{ItemPhase, PhaseCard, PhaseFacts, RunItemEvent};
use super::super::laws::{PlayRule, PublicViewRules};
use super::super::lifecycle::EquipWork;
use super::super::{CombatItemContext, RelicHookWork};

/// The relics this lane owns (design Appendix A, Lane R2); each is handled here only.
pub(crate) const RELICS: &[RelicId] = &[
    RelicId::TheBoot, RelicId::CentennialPuzzle, RelicId::Nunchaku, RelicId::PenNib, RelicId::BlueCandle,
    RelicId::GremlinHorn, RelicId::InkBottle, RelicId::Kunai, RelicId::LetterOpener, RelicId::MummifiedHand,
    RelicId::OrnamentalFan, RelicId::Shuriken, RelicId::StrikeDummy, RelicId::BirdFacedUrn,
    RelicId::DeadBranch, RelicId::Ginger, RelicId::Torii, RelicId::TungstenRod, RelicId::Turnip,
    RelicId::HandDrill, RelicId::MedicalKit, RelicId::OrangePellets, RelicId::OddMushroom,
];

fn owns(id: RelicId) -> bool { RELICS.contains(&id) }

fn effect(id: RelicId, target: Option<Actor>, kind: EffectKind) -> Effect {
    Effect { source: Source::Relic(id), target, kind }
}

fn queue_bottom(id: RelicId, target: Actor, kind: EffectKind, work: &mut RelicHookWork) {
    work.bottom.push(effect(id, Some(target), kind));
}

/// Combat phase work of this lane's supported relics. Returns whether `id` is this lane's relic.
pub(crate) fn phase(id: RelicId, context: &mut CombatItemContext, phase: ItemPhase,
    facts: &PhaseFacts<'_>, work: &mut RelicHookWork) -> Result<bool, InputError> {
    if !owns(id) { return Ok(false); }
    match (id, phase) {
        (RelicId::CentennialPuzzle, ItemPhase::CombatStart | ItemPhase::CombatStartPreDraw) => {
            context.set_local(id, 0);
        }
        (RelicId::CentennialPuzzle, ItemPhase::HpLost) if facts.hp_lost > 0 && context.local(id) == 0 => {
            context.set_local(id, 1);
            queue_bottom(id, Actor::Player, EffectKind::Draw(3), work);
        }
        (RelicId::Nunchaku, ItemPhase::CardUsed) if facts.card.is_some_and(|card|card.card_type==CardType::Attack) => {
            let next = context.counter(id).rem_euclid(10) + 1;
            if next == 10 {
                context.set_counter(id, 0);
                queue_bottom(id, Actor::Player, EffectKind::GainEnergy(1), work);
            } else {
                context.set_counter(id, next);
            }
        }
        (RelicId::PenNib, ItemPhase::CombatStartPreDraw) if context.counter(id) == 9 => {
            queue_bottom(id, Actor::Player, EffectKind::ApplyFullPower { id: FullPowerId("pen_nib"), amount: 1 }, work);
        }
        (RelicId::PenNib, ItemPhase::CardUsed) if facts.card.is_some_and(|card|card.card_type==CardType::Attack) => {
            let current = context.counter(id).clamp(0, 9);
            if current == 9 {
                context.set_counter(id, 0);
            } else {
                context.set_counter(id, current + 1);
                if current == 8 {
                    queue_bottom(id, Actor::Player,
                        EffectKind::ApplyFullPower { id: FullPowerId("pen_nib"), amount: 1 }, work);
                }
            }
        }
        (RelicId::Kunai | RelicId::OrnamentalFan | RelicId::Shuriken | RelicId::LetterOpener,
            ItemPhase::TurnStartPreDraw | ItemPhase::TurnStart) => context.set_local(id, 0),
        (RelicId::GremlinHorn, ItemPhase::EnemyDied) if facts.dead_enemy.is_some_and(|(_, _, live)|live > 0) => {
            queue_bottom(id, Actor::Player, EffectKind::GainEnergy(1), work);
            queue_bottom(id, Actor::Player, EffectKind::Draw(1), work);
        }
        (RelicId::InkBottle, ItemPhase::CardUsed) if facts.card.is_some() => {
            let next = context.counter(id).rem_euclid(10) + 1;
            if next == 10 {
                context.set_counter(id, 0);
                queue_bottom(id, Actor::Player, EffectKind::Draw(1), work);
            } else {
                context.set_counter(id, next);
            }
        }
        (RelicId::Kunai | RelicId::OrnamentalFan | RelicId::Shuriken, ItemPhase::CardUsed)
            if facts.card.is_some_and(|card|card.card_type==CardType::Attack) => {
            let next = context.local(id) + 1;
            if next == 3 {
                context.set_local(id, 0);
                match id {
                    RelicId::Kunai => queue_bottom(id, Actor::Player,
                        EffectKind::ApplyFullPower { id: FullPowerId("dexterity"), amount: 1 }, work),
                    RelicId::OrnamentalFan => queue_bottom(id, Actor::Player, EffectKind::GainBlock(4), work),
                    RelicId::Shuriken => queue_bottom(id, Actor::Player,
                        EffectKind::ApplyFullPower { id: FullPowerId("strength"), amount: 1 }, work),
                    _ => unreachable!(),
                }
            } else {
                context.set_local(id, next);
            }
        }
        (RelicId::LetterOpener, ItemPhase::CardUsed)
            if facts.card.is_some_and(|card|card.card_type==CardType::Skill) => {
            let next = context.local(id) + 1;
            if next == 3 {
                context.set_local(id, 0);
                for enemy in facts.live_enemies {
                    work.bottom.push(effect(id, Some(Actor::Enemy(enemy.clone())),
                        EffectKind::Damage(DamagePayload::Thorns(5))));
                }
            } else {
                context.set_local(id, next);
            }
        }
        (RelicId::BirdFacedUrn, ItemPhase::CardUsed)
            if facts.card.is_some_and(|card|card.card_type==CardType::Power) => {
            work.top.push(effect(id, Some(Actor::Player), EffectKind::Heal(2)));
        }
        (RelicId::BlueCandle, ItemPhase::CardUsed)
            if facts.card.is_some_and(|card|card.card_type==CardType::Curse) => {
            queue_bottom(id, Actor::Player, EffectKind::Damage(DamagePayload::HpLoss(1)), work);
        }
        (RelicId::MummifiedHand, ItemPhase::CardUsed)
            if facts.card.is_some_and(|card|card.card_type==CardType::Power) => {
            work.bottom.push(effect(id, None,
                EffectKind::Relic(RelicEffect::SetRandomHandCardCostZeroThisTurn)));
        }
        (RelicId::DeadBranch, ItemPhase::Exhaust)
            if facts.card.is_some() && !facts.live_enemies.is_empty() => {
            queue_bottom(id, Actor::Player, EffectKind::Generate(GenerationWork {
                pool: GenerationPool::Any,
                count: 1,
                unique: false,
                destination: CardDestination::Hand,
                cost: GenerationCost::Ordinary,
                upgraded: false,
                offer: None,
                captured: false,
                cards: vec![],
                next: 0,
            }), work);
        }
        (RelicId::OrangePellets, ItemPhase::TurnStartPreDraw | ItemPhase::TurnStart) => {
            context.set_local(id, 0);
        }
        (RelicId::OrangePellets, ItemPhase::CardUsed) if let Some(card) = facts.card => {
            let flag = match card.card_type {
                CardType::Attack => 1,
                CardType::Skill => 2,
                CardType::Power => 4,
                CardType::Curse | CardType::Status => 0,
            };
            if flag != 0 {
                let flags = context.local(id) | flag;
                if flags == 7 {
                    context.set_local(id, 0);
                    work.bottom.push(effect(id, None,
                        EffectKind::Relic(RelicEffect::RemovePlayerDebuffs)));
                } else {
                    context.set_local(id, flags);
                }
            }
        }
        (RelicId::HandDrill, ItemPhase::BlockBroken) => {
            if let Some(Actor::Enemy(enemy)) = facts.target {
                work.bottom.push(effect(id, Some(Actor::Enemy(enemy.clone())),
                    EffectKind::ApplyFullPower { id: FullPowerId("vulnerable"), amount: 2 }));
            }
        }
        // Medical Kit's allowed Status is exhausted by the shared play-finish owner.
        (RelicId::MedicalKit, _) => {}
        _ => {}
    }
    Ok(true)
}

pub(crate) fn outgoing_damage(id: RelicId, _context: &CombatItemContext, card: Option<PhaseCard>,
    _target: &Actor, amount: &mut i32) -> bool {
    match id {
        RelicId::StrikeDummy if card.is_some_and(|card|content::is_strike(card.id)) => {
            *amount = amount.saturating_add(3);
            true
        }
        _ => false,
    }
}

pub(crate) fn hp_loss(id: RelicId, _context: &CombatItemContext, amount: &mut i32) -> bool {
    if id == RelicId::TungstenRod && *amount > 0 {
        *amount -= 1;
        true
    } else { false }
}

pub(crate) fn block_decay(_id: RelicId, _context: &CombatItemContext, _current: i32, _kept: &mut i32) -> bool { false }
pub(crate) fn recharge(_id: RelicId, _context: &CombatItemContext, _character: Character, _unspent: i32, _energy: &mut i32) -> bool { false }
pub(crate) fn master_energy(_id: RelicId, _context: &CombatItemContext, _elite_or_boss: bool, _bonus: &mut i32) -> bool { false }

pub(crate) fn play_allowed(id: RelicId, _context: &CombatItemContext, card: PhaseCard, rule: &mut PlayRule) -> bool {
    match (id, card.card_type) {
        (RelicId::BlueCandle, CardType::Curse) | (RelicId::MedicalKit, CardType::Status) => {
            rule.allow = Some(true);
            true
        }
        _ => false,
    }
}

pub(crate) fn repeat_play(_id: RelicId, _context: &mut CombatItemContext, _card: PhaseCard, _extra: &mut u32) -> bool { false }
pub(crate) fn would_exhaust(_id: RelicId, _context: &mut CombatItemContext, _card: PhaseCard, _exhaust: &mut bool) -> bool { false }

pub(crate) fn power_attempt(id: RelicId, _context: &CombatItemContext, target: &Actor,
    power: &str, amount: i32, allowed: &mut bool) -> bool {
    if *target == Actor::Player && amount > 0
        && (id == RelicId::Ginger && power == "weak" || id == RelicId::Turnip && power == "frail") {
        *allowed = false;
        true
    } else { false }
}

pub(crate) fn would_die(_id: RelicId, _context: &mut CombatItemContext, _max_hp: i32, _revive: &mut Option<i32>) -> bool { false }
pub(crate) fn heal_amount(_id: RelicId, _context: &CombatItemContext, _amount: &mut i32) -> bool { false }
pub(crate) fn public_view(_id: RelicId, _context: &CombatItemContext, _rules: &mut PublicViewRules) -> bool { false }

pub(crate) fn unblocked_outgoing_damage(id: RelicId, _context: &CombatItemContext, _target: &Actor,
    amount: &mut i32) -> bool {
    if id == RelicId::TheBoot && (1..=4).contains(amount) {
        *amount = 5;
        true
    } else { false }
}

pub(crate) fn player_vulnerable_factor(id: RelicId, _context: &CombatItemContext, factor: &mut f32) -> bool {
    if id == RelicId::OddMushroom {
        *factor = 1.25;
        true
    } else { false }
}

pub(crate) fn incoming_damage(id: RelicId, _context: &CombatItemContext, _attacker: Option<&Actor>,
    amount: &mut i32) -> bool {
    if id == RelicId::Torii && (2..=5).contains(amount) {
        *amount = 1;
        true
    } else { false }
}

/// Run event work of this lane's relics.
pub(crate) fn run_event(_id: RelicId, _state: &mut RunState, _event: &mut RunItemEvent) -> Result<bool, RunError> { Ok(false) }
/// Obtain-time (equip) work of this lane's relics.
pub(crate) fn equip(_id: RelicId, _state: &mut RunState, _work: &mut EquipWork) -> Result<bool, RunError> { Ok(false) }
/// Completion of a selection this lane's relic opened.
pub(crate) fn selection_done(_id: RelicId, _state: &mut RunState, _chosen: &[u64]) -> Result<bool, RunError> { Ok(false) }
