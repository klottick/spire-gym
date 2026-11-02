//! Law entries of the accepted relics (Main). Accepted single-relic site checks stay inline
//! until Task M3 re-gates them; Self-Forming Clay and Red Skull's immediate HP-state work is
//! here because it visits more than one relic.
#![allow(dead_code)]
use crate::combat::{Actor, Effect, InputError};
use crate::content::{Character, RelicId};

use super::super::dispatch::PhaseCard;
use super::super::laws::{PlayRule, PublicViewRules};
use super::super::CombatItemContext;
pub(crate) fn outgoing_damage(_id: RelicId, _context: &CombatItemContext, _card: Option<PhaseCard>, _target: &Actor, _amount: &mut i32) -> bool { false }
pub(crate) fn incoming_damage(_id: RelicId, _context: &CombatItemContext, _attacker: Option<&Actor>, _amount: &mut i32) -> bool { false }
pub(crate) fn hp_loss(_id: RelicId, _context: &CombatItemContext, _amount: &mut i32) -> bool { false }
pub(crate) fn block_decay(id: RelicId, context: &CombatItemContext, current: i32, kept: &mut i32) -> bool {
    if id != RelicId::Calipers || !context.complete_items() { return false; }
    *kept = (*kept).max(current.saturating_sub(15).max(0));
    true
}
pub(crate) fn recharge(id: RelicId, context: &CombatItemContext, _character: Character, unspent: i32, energy: &mut i32) -> bool {
    if id != RelicId::IceCream || !context.complete_items() { return false; }
    *energy = energy.saturating_add(unspent.max(0)).min(999);
    true
}
pub(crate) fn master_energy(id: RelicId, context: &CombatItemContext, elite_or_boss: bool, bonus: &mut i32) -> bool {
    let adds_energy = match id {
        RelicId::Sozu | RelicId::MarkOfPain => true,
        RelicId::SlaversCollar => elite_or_boss,
        RelicId::BustedCrown | RelicId::CoffeeDripper | RelicId::CursedKey | RelicId::Ectoplasm
        | RelicId::FusionHammer | RelicId::PhilosophersStone | RelicId::RunicDome | RelicId::VelvetChoker =>
            context.complete_items(),
        _ => false,
    };
    if !adds_energy { return false; }
    *bonus += 1;
    true
}
pub(crate) fn play_allowed(_id: RelicId, _context: &CombatItemContext, _card: PhaseCard, _rule: &mut PlayRule) -> bool { false }
pub(crate) fn repeat_play(_id: RelicId, _context: &mut CombatItemContext, _card: PhaseCard, _extra: &mut u32) -> bool { false }
pub(crate) fn would_exhaust(_id: RelicId, _context: &mut CombatItemContext, _card: PhaseCard, _exhaust: &mut bool) -> bool { false }
pub(crate) fn power_attempt(_id: RelicId, _context: &CombatItemContext, _target: &Actor, _power: &str, _amount: i32, _allowed: &mut bool) -> bool { false }
pub(crate) fn would_die(id: RelicId, context: &mut CombatItemContext, max_hp: i32, revive: &mut Option<i32>) -> bool {
    if id!=RelicId::LizardTail||!context.complete_items()||!context.owns_relic(id)
        ||context.owns_relic(RelicId::MarkOfBloom)||context.counter(id)!=0{return false}
    context.set_counter(id,1);
    *revive=Some((max_hp/2).max(1));
    true
}
pub(crate) fn heal_amount(_id: RelicId, _context: &CombatItemContext, _amount: &mut i32) -> bool { false }
pub(crate) fn public_view(_id: RelicId, _context: &CombatItemContext, _rules: &mut PublicViewRules) -> bool { false }

pub(crate) fn unblocked_outgoing_damage(_id: RelicId, _context: &CombatItemContext, _target: &Actor, _amount: &mut i32) -> bool { false }
pub(crate) fn player_vulnerable_factor(_id: RelicId, _context: &CombatItemContext, _factor: &mut f32) -> bool { false }

/// Self-Forming Clay and Red Skull (`items/ironclad.rs`, bodies unchanged).
pub(crate) fn hp_changed(id: RelicId, context: &mut CombatItemContext, hp_lost: i32, hp: i32, max_hp: i32,
    work: &mut Vec<Effect>) -> Result<bool, InputError> {
    super::super::ironclad::hp_changed(id, context, hp_lost, hp, max_hp, work)
}
