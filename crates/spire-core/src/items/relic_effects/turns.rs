//! Lane R1: turn, start-of-combat and end-of-turn relic bodies (plan Task R1).
#![allow(dead_code)]
use crate::combat::{Actor, CardDestination, DamagePayload, Effect, EffectKind, FullPowerId,
    GenerationCost, GenerationPool, GenerationWork, InputError, Outcome, RelicEffect, Source};
use crate::content::{CardType, Character, RelicId};
use crate::run::{RunError, RunState};

use super::super::dispatch::{ItemPhase, PhaseCard, PhaseFacts, RunItemEvent};
use super::super::laws::{PlayRule, PublicViewRules};
use super::super::lifecycle::EquipWork;
use super::super::{CombatItemContext, RelicHookWork};

/// The relics this lane owns (design Appendix A, Lane R1); each is handled here only.
pub(crate) const RELICS: &[RelicId] = &[
    RelicId::Akabeko, RelicId::ArtOfWar, RelicId::HappyFlower, RelicId::PreservedInsect, RelicId::HornCleat,
    RelicId::MeatOnTheBone, RelicId::MercuryHourglass, RelicId::Pantograph, RelicId::Sundial,
    RelicId::CaptainsWheel, RelicId::DuVuDoll, RelicId::FossilizedHelix, RelicId::IncenseBurner,
    RelicId::Pocketwatch, RelicId::StoneCalendar, RelicId::ThreadAndNeedle, RelicId::TheAbacus,
    RelicId::ClockworkSouvenir, RelicId::SlingOfCourage, RelicId::CultistHeadpiece, RelicId::Enchiridion,
    RelicId::FaceOfCleric, RelicId::GremlinVisage, RelicId::MutagenicStrength, RelicId::WarpedTongs,
];

fn bottom(work: &mut RelicHookWork, id: RelicId, target: Actor, kind: EffectKind) {
    work.bottom.push(Effect { source: Source::Relic(id), target: Some(target), kind });
}

/// atBattleStart addToTop work (source-check-turnstart-hooks-v1); `dispatch::source_top` lists these relics.
fn top(work: &mut RelicHookWork, id: RelicId, target: Actor, kind: EffectKind) {
    work.top.push(Effect { source: Source::Relic(id), target: Some(target), kind });
}

fn bottom_relic(work: &mut RelicHookWork, id: RelicId, kind: EffectKind) {
    work.bottom.push(Effect { source: Source::Relic(id), target: None, kind });
}

fn damage_all(work: &mut RelicHookWork, id: RelicId, enemies: &[crate::combat::EnemyHandle], amount: i32) {
    for enemy in enemies {
        bottom(work, id, Actor::Enemy(enemy.clone()), EffectKind::Damage(DamagePayload::Thorns(amount)));
    }
}

/// Combat phase work of this lane's relics. Returns whether `id` is this lane's relic.
pub(crate) fn phase(id: RelicId, context: &mut CombatItemContext, phase: ItemPhase,
    facts: &PhaseFacts<'_>, work: &mut RelicHookWork) -> Result<bool, InputError> {
    if !RELICS.contains(&id) { return Ok(false); }

    match id {
        RelicId::Akabeko => match phase {
            ItemPhase::CombatStart => top(work, id, Actor::Player,
                EffectKind::ApplyFullPower { id: FullPowerId("vigor"), amount: 8 }),
            ItemPhase::CardUsed if facts.card.is_some_and(|card| card.card_type == CardType::Attack) =>
                bottom(work, id, Actor::Player, EffectKind::RemoveFullPower(FullPowerId("vigor"))),
            _ => (),
        },
        RelicId::ArtOfWar => match phase {
            ItemPhase::CombatStart => context.set_local(id, -1),
            ItemPhase::CardUsed if facts.card.is_some_and(|card| card.card_type == CardType::Attack) =>
                context.set_local(id, 1),
            ItemPhase::TurnStartPreDraw if facts.target == Some(&Actor::Player) => {
                let prior = context.local(id);
                if prior == -1 {
                    // The opening player turn has no preceding qualifying interval.
                    context.set_local(id, 0);
                } else {
                    if prior == 0 { bottom(work, id, Actor::Player, EffectKind::GainEnergy(1)); }
                    context.set_local(id, 0);
                }
            }
            _ => (),
        },
        RelicId::HappyFlower if phase == ItemPhase::TurnStartPreDraw && facts.target == Some(&Actor::Player) => {
            let next = if context.counter(id) == -1 { 1 } else { context.counter(id) + 1 };
            if next >= 3 {
                context.set_counter(id, 0);
                bottom(work, id, Actor::Player, EffectKind::GainEnergy(1));
            } else {
                context.set_counter(id, next);
            }
        }
        // Its HP cut is immediate in the source; top is the nearest queued position.
        RelicId::PreservedInsect if phase == ItemPhase::CombatStart && facts.elite => {
            work.top.push(Effect { source: Source::Relic(id), target: None, kind: EffectKind::Relic(RelicEffect::CapEnemyHp { percent: 75 }) });
        }
        RelicId::HornCleat => match phase {
            ItemPhase::CombatStart => context.set_local(id, 0),
            ItemPhase::TurnStartPreDraw if facts.target == Some(&Actor::Player) => {
                let turn = context.local(id) + 1;
                context.set_local(id, turn);
                if turn == 2 { bottom(work, id, Actor::Player, EffectKind::GainBlock(14)); }
            }
            _ => (),
        },
        RelicId::MeatOnTheBone => (),
        RelicId::MercuryHourglass if phase == ItemPhase::TurnStartPreDraw && facts.target == Some(&Actor::Player) => {
            damage_all(work, id, facts.live_enemies, 3);
        }
        RelicId::Pantograph if phase == ItemPhase::CombatStart && facts.boss => {
            top(work, id, Actor::Player, EffectKind::Heal(25));
        }
        RelicId::Sundial if phase == ItemPhase::DrawPileShuffled => {
            let callbacks = context.counter(id) + 1;
            if callbacks == 3 {
                context.set_counter(id, 0);
                bottom(work, id, Actor::Player, EffectKind::GainEnergy(2));
            } else {
                context.set_counter(id, callbacks);
            }
        }
        RelicId::CaptainsWheel => match phase {
            ItemPhase::CombatStart => context.set_local(id, 0),
            ItemPhase::TurnStartPreDraw if facts.target == Some(&Actor::Player) => {
                let turn = context.local(id) + 1;
                context.set_local(id, turn);
                if turn == 3 { bottom(work, id, Actor::Player, EffectKind::GainBlock(18)); }
            }
            _ => (),
        },
        RelicId::DuVuDoll if phase == ItemPhase::CombatStart => {
            let curses = facts.deck_curses.min(i32::MAX as usize) as i32;
            if curses > 0 {
                top(work, id, Actor::Player, EffectKind::ApplyFullPower {
                    id: FullPowerId("strength"), amount: curses,
                });
            }
        }
        RelicId::FossilizedHelix if phase == ItemPhase::CombatStart => {
            bottom(work, id, Actor::Player, EffectKind::ApplyFullPower {
                id: FullPowerId("buffer"), amount: 1,
            });
        }
        RelicId::IncenseBurner if phase == ItemPhase::TurnStartPreDraw && facts.target == Some(&Actor::Player) => {
            let next = if context.counter(id) == -1 { 1 } else { context.counter(id) + 1 };
            if next >= 6 {
                context.set_counter(id, 0);
                bottom(work, id, Actor::Player, EffectKind::ApplyFullPower {
                    id: FullPowerId("intangible_player"), amount: 1,
                });
            } else {
                context.set_counter(id, next);
            }
        }
        RelicId::Pocketwatch => match phase {
            ItemPhase::CardPlayed => context.set_local(id, context.local(id) + 1),
            // atTurnStartPostDraw: AfterOpeningDraw on turn one, AfterDraw on later turns.
            // Pocketwatch intentionally has no opening-turn bonus.
            ItemPhase::AfterDraw => {
                if context.local(id) <= 3 { bottom(work, id, Actor::Player, EffectKind::Draw(3)); }
                context.set_local(id, 0);
            }
            _ => (),
        },
        RelicId::StoneCalendar => match phase {
            ItemPhase::TurnStartPreDraw if facts.target == Some(&Actor::Player) =>
                context.set_local(id, context.local(id) + 1),
            ItemPhase::EndTurn if context.local(id) == 7 => damage_all(work, id, facts.live_enemies, 52),
            _ => (),
        },
        RelicId::ThreadAndNeedle if phase == ItemPhase::CombatStart => {
            top(work, id, Actor::Player, EffectKind::ApplyFullPower {
                id: FullPowerId("plated_armor"), amount: 4,
            });
        }
        RelicId::TheAbacus if phase == ItemPhase::DrawPileShuffled => {
            bottom(work, id, Actor::Player, EffectKind::GainBlock(6));
        }
        RelicId::ClockworkSouvenir if phase == ItemPhase::CombatStart => {
            top(work, id, Actor::Player, EffectKind::ApplyFullPower {
                id: FullPowerId("artifact"), amount: 1,
            });
        }
        RelicId::SlingOfCourage if phase == ItemPhase::CombatStart && facts.elite => {
            top(work, id, Actor::Player, EffectKind::ApplyFullPower {
                id: FullPowerId("strength"), amount: 2,
            });
        }
        RelicId::CultistHeadpiece => {
            // Its “Caw” is cosmetic; the combat model has no speech/audio output surface.
        }
        RelicId::Enchiridion if phase == ItemPhase::CombatStartPreDraw && context.is_pre_draw_visit(facts.hook) => {
            bottom(work, id, Actor::Player, EffectKind::Generate(GenerationWork {
                pool: GenerationPool::Power,
                count: 1,
                unique: false,
                destination: CardDestination::Hand,
                cost: GenerationCost::TurnZero,
                upgraded: false,
                offer: None,
                captured: false,
                cards: vec![],
                next: 0,
            }));
        }
        RelicId::FaceOfCleric => (),
        RelicId::GremlinVisage if phase == ItemPhase::CombatStart => {
            bottom(work, id, Actor::Player, EffectKind::ApplyFullPower {
                id: FullPowerId("weak"), amount: 1,
            });
        }
        // Catalog MUTAGENIC_STRENGTH (F15): 3 Strength, then a matching Strength Down (the shared
        // `lose_strength` debuff, as Flex), so Artifact can block it and Orange Pellets can remove it.
        RelicId::MutagenicStrength if phase == ItemPhase::CombatStart => {
            top(work, id, Actor::Player, EffectKind::ApplyFullPower {
                id: FullPowerId("strength"), amount: 3,
            });
            top(work, id, Actor::Player, EffectKind::ApplyFullPower {
                id: FullPowerId("lose_strength"), amount: 3,
            });
        }
        RelicId::WarpedTongs => match phase {
            ItemPhase::AfterOpeningDraw | ItemPhase::AfterDraw => bottom_relic(
                work, id, EffectKind::Relic(RelicEffect::UpgradeRandomHandCard)),
            _ => (),
        },
        _ => (),
    }
    Ok(true)
}

pub(crate) fn outgoing_damage(_id: RelicId, _context: &CombatItemContext, _card: Option<PhaseCard>, _target: &Actor, _amount: &mut i32) -> bool { false }
pub(crate) fn incoming_damage(_id: RelicId, _context: &CombatItemContext, _attacker: Option<&Actor>, _amount: &mut i32) -> bool { false }
pub(crate) fn hp_loss(_id: RelicId, _context: &CombatItemContext, _amount: &mut i32) -> bool { false }
pub(crate) fn block_decay(_id: RelicId, _context: &CombatItemContext, _current: i32, _kept: &mut i32) -> bool { false }
pub(crate) fn recharge(_id: RelicId, _context: &CombatItemContext, _character: Character, _unspent: i32, _energy: &mut i32) -> bool { false }
pub(crate) fn master_energy(_id: RelicId, _context: &CombatItemContext, _elite_or_boss: bool, _bonus: &mut i32) -> bool { false }
pub(crate) fn play_allowed(_id: RelicId, _context: &CombatItemContext, _card: PhaseCard, _rule: &mut PlayRule) -> bool { false }
pub(crate) fn repeat_play(_id: RelicId, _context: &mut CombatItemContext, _card: PhaseCard, _extra: &mut u32) -> bool { false }
pub(crate) fn would_exhaust(_id: RelicId, _context: &mut CombatItemContext, _card: PhaseCard, _exhaust: &mut bool) -> bool { false }
pub(crate) fn power_attempt(_id: RelicId, _context: &CombatItemContext, _target: &Actor, _power: &str, _amount: i32, _allowed: &mut bool) -> bool { false }
pub(crate) fn would_die(_id: RelicId, _context: &mut CombatItemContext, _max_hp: i32, _revive: &mut Option<i32>) -> bool { false }
pub(crate) fn heal_amount(_id: RelicId, _context: &CombatItemContext, _amount: &mut i32) -> bool { false }
pub(crate) fn public_view(_id: RelicId, _context: &CombatItemContext, _rules: &mut PublicViewRules) -> bool { false }
pub(crate) fn unblocked_outgoing_damage(_id: RelicId, _context: &CombatItemContext, _target: &Actor, _amount: &mut i32) -> bool { false }
pub(crate) fn player_vulnerable_factor(_id: RelicId, _context: &CombatItemContext, _factor: &mut f32) -> bool { false }

/// Run event work of this lane's relics.
pub(crate) fn run_event(id: RelicId, state: &mut RunState, event: &mut RunItemEvent) -> Result<bool, RunError> {
    if !matches!(id, RelicId::MeatOnTheBone | RelicId::FaceOfCleric)
        || !matches!(event, RunItemEvent::CombatSettled { .. })
    {
        return Ok(false);
    }

    match (id, event) {
        (RelicId::MeatOnTheBone, RunItemEvent::CombatSettled { outcome })
            if *outcome == Outcome::Victory && state.hp > 0 && state.max_hp > 0 && state.hp <= state.max_hp / 2 =>
        {
            let amount = crate::items::victory_heal_amount(state, 12).max(0) as u32;
            state.hp = state.hp.saturating_add(amount).min(state.max_hp);
        }
        (RelicId::FaceOfCleric, RunItemEvent::CombatSettled { outcome })
            if *outcome == Outcome::Victory && state.hp > 0 =>
        {
            let max_hp = state.max_hp.checked_add(1).filter(|max_hp| *max_hp <= 999)
                .ok_or(RunError::Unsupported("maximum HP overflow"))?;
            let heal = crate::items::victory_heal_amount(state, 1).max(0) as u32;
            state.max_hp = max_hp;
            state.hp = state.hp.saturating_add(heal).min(max_hp);
        }
        _ => (),
    }
    Ok(true)
}

/// Obtain-time (equip) work of this lane's relics.
pub(crate) fn equip(id: RelicId, state: &mut RunState, _work: &mut EquipWork) -> Result<bool, RunError> {
    match id {
        RelicId::HappyFlower | RelicId::Sundial | RelicId::IncenseBurner => {
            state.relic_counters.entry(id).or_insert(0);
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Completion of a selection this lane's relic opened.
pub(crate) fn selection_done(_id: RelicId, _state: &mut RunState, _chosen: &[u64]) -> Result<bool, RunError> { Ok(false) }
