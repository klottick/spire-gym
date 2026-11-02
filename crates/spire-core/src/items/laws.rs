//! Single-relic modifier laws (M27 design §6.2): one function per law, each called from
//! exactly one owner site and visiting owned relics in acquisition order. For each relic
//! the law asks the handler chain; a lane fills its relics' bodies in its lane file.
use crate::combat::{Actor, Effect, InputError};
use crate::content::{Character, RelicId};

use super::dispatch::PhaseCard;
use super::CombatItemContext;

#[cfg(test)]
thread_local! { static LAW_LOG: std::cell::RefCell<Vec<&'static str>> = std::cell::RefCell::new(Vec::new()); }

#[cfg(test)]
fn record_law(name: &'static str) { LAW_LOG.with(|log| log.borrow_mut().push(name)); }

#[cfg(test)]
pub(crate) fn take_law_log() -> Vec<&'static str> {
    LAW_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}

/// What the core public combat view hides or reveals (binding F12).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PublicViewRules { pub hide_intents: bool, pub reveal_draw_order: bool }

/// A legality override for one card play: `None` keeps the card's own rule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PlayRule { pub allow: Option<bool> }

macro_rules! chain {
    ($law:ident, $id:expr, $($arg:expr),*) => {
        super::relic_effects::accepted_law::$law($id, $($arg),*)
            || super::relic_effects::turns::$law($id, $($arg),*)
            || super::relic_effects::play::$law($id, $($arg),*)
            || super::relic_effects::run::$law($id, $($arg),*)
            || super::relic_effects::structural::$law($id, $($arg),*)
    };
}

macro_rules! chain_four {
    ($law:ident, $id:expr, $($arg:expr),*) => {
        super::relic_effects::accepted_law::$law($id, $($arg),*)
            || super::relic_effects::play::$law($id, $($arg),*)
            || super::relic_effects::run::$law($id, $($arg),*)
            || super::relic_effects::structural::$law($id, $($arg),*)
    };
}

fn owned(context: &CombatItemContext) -> Vec<RelicId> { context.relics().to_vec() }

/// Outgoing attack damage to `target` (Pen Nib, Strike Dummy, The Boot).
pub(crate) fn outgoing_damage(context: &CombatItemContext, card: Option<PhaseCard>, target: &Actor, mut amount: i32) -> i32 {
    #[cfg(test)] record_law("outgoing_damage");
    for id in owned(context) { chain!(outgoing_damage, id, context, card, target, &mut amount); }
    amount
}
/// NORMAL enemy attack damage to the player after Block (Torii, Odd Mushroom).
pub(crate) fn incoming_damage(context: &CombatItemContext, attacker: Option<&Actor>, mut amount: i32) -> i32 {
    #[cfg(test)] record_law("incoming_damage");
    for id in owned(context) { chain!(incoming_damage, id, context, attacker, &mut amount); }
    amount
}
/// NORMAL player damage to an enemy after Block (The Boot).
pub(crate) fn unblocked_outgoing_damage(context: &CombatItemContext, target: &Actor, mut amount: i32) -> i32 {
    #[cfg(test)] record_law("unblocked_outgoing_damage");
    for id in owned(context) { chain_four!(unblocked_outgoing_damage, id, context, target, &mut amount); }
    amount
}
/// Player HP loss after Block (Tungsten Rod).
pub(crate) fn hp_loss(context: &CombatItemContext, mut amount: i32) -> i32 {
    #[cfg(test)] record_law("hp_loss");
    for id in owned(context) { chain!(hp_loss, id, context, &mut amount); }
    amount
}
/// Block kept at turn start (Calipers); `block` is the Block the ordinary rule keeps.
pub(crate) fn block_decay(context: &CombatItemContext, current: i32, mut kept: i32) -> i32 {
    #[cfg(test)] record_law("block_decay");
    for id in owned(context) { chain!(block_decay, id, context, current, &mut kept); }
    kept
}
/// The player's NORMAL damage received multiplier (Odd Mushroom), defaulting to today's 1.5.
pub(crate) fn player_vulnerable_factor(context: &CombatItemContext) -> f32 {
    #[cfg(test)] record_law("player_vulnerable_factor");
    let mut factor = 1.5_f32;
    for id in owned(context) { chain_four!(player_vulnerable_factor, id, context, &mut factor); }
    factor
}
/// Energy after recharge (Ice Cream); `energy` starts as the ordinary recharge result.
pub(crate) fn recharge(context: &CombatItemContext, character: Character, unspent: i32, mut energy: i32) -> i32 {
    for id in owned(context) { chain!(recharge, id, context, character, unspent, &mut energy); }
    energy
}
/// Master-energy bonus at combat preparation (G1: Sozu and the boss energy relics).
pub(crate) fn master_energy(context: &CombatItemContext, elite_or_boss: bool) -> i32 {
    let mut bonus = 0;
    for id in owned(context) { chain!(master_energy, id, context, elite_or_boss, &mut bonus); }
    bonus
}
/// Card play legality override (Velvet Choker, Blue Candle, Medical Kit).
pub(crate) fn play_allowed(context: &CombatItemContext, card: PhaseCard) -> PlayRule {
    #[cfg(test)] record_law("play_allowed");
    let mut rule = PlayRule::default();
    for id in owned(context) { chain!(play_allowed, id, context, card, &mut rule); }
    rule
}
/// Extra plays of a card (Necronomicon).
pub(crate) fn repeat_play(context: &mut CombatItemContext, card: PhaseCard) -> u32 {
    #[cfg(test)] record_law("repeat_play");
    let mut extra = 0;
    for id in owned(context) { chain!(repeat_play, id, context, card, &mut extra); }
    extra
}
/// Whether a played card that would exhaust still exhausts (Strange Spoon).
pub(crate) fn would_exhaust(context: &mut CombatItemContext, card: PhaseCard) -> bool {
    #[cfg(test)] record_law("would_exhaust");
    let mut exhaust = true;
    for id in owned(context) { chain!(would_exhaust, id, context, card, &mut exhaust); }
    exhaust
}
/// Whether a power application to `target` proceeds (Ginger, Turnip).
pub(crate) fn power_attempt(context: &CombatItemContext, target: &Actor, power: &str, amount: i32) -> bool {
    let mut allowed = true;
    for id in owned(context) { chain!(power_attempt, id, context, target, power, amount, &mut allowed); }
    allowed
}
/// HP to revive to when the player would die (Lizard Tail after Fairy, design §9.3).
pub(crate) fn would_die(context: &mut CombatItemContext, max_hp: i32) -> Option<i32> {
    #[cfg(test)] record_law("would_die");
    let mut revive = None;
    for id in owned(context) { chain!(would_die, id, context, max_hp, &mut revive); }
    revive
}
/// A heal amount after relic modifiers (the one heal law, design §9.1).
pub(crate) fn heal_amount(context: &CombatItemContext, mut amount: i32) -> i32 {
    #[cfg(test)] record_law("heal_amount");
    for id in owned(context) { chain!(heal_amount, id, context, &mut amount); }
    amount
}
/// The core public view rules (Runic Dome, Frozen Eye).
pub(crate) fn public_view(context: &CombatItemContext) -> PublicViewRules {
    #[cfg(test)] record_law("public_view");
    let mut rules = PublicViewRules::default();
    for id in owned(context) { chain!(public_view, id, context, &mut rules); }
    rules
}
/// Immediate HP-state callbacks (Self-Forming Clay's pending Block, Red Skull's threshold).
/// `hp_lost` is 0 for a heal or maximum-HP change.
pub(crate) fn hp_changed(context: &mut CombatItemContext, hp_lost: i32, hp: i32, max_hp: i32) -> Result<Vec<Effect>, InputError> {
    let mut work = Vec::new();
    for id in owned(context) {
        super::relic_effects::accepted_law::hp_changed(id, context, hp_lost, hp, max_hp, &mut work)?;
    }
    Ok(work)
}
