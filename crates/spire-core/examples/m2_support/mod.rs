//! Small authored setups/caller utilities; no simulation rules or run progression.
use spire_core::{combat::*, content::{CardId, EnemyId}};
#[path = "../m1_support/mod.rs"]
#[allow(dead_code)]
mod m1;
pub use m1::{card_action, DemoError};

pub fn fixture(enemy: EnemyId, seed: u64) -> CombatEntry {
    if enemy != EnemyId::SpikeSlimeM { return m1::fixture(enemy, seed); }
    let mut entry = m1::fixture(EnemyId::JawWorm, seed);
    let actor = &mut entry.setup.enemies[0];
    actor.enemy_id = EnemyId::SpikeSlimeM;
    actor.hp = 30; actor.max_hp = 30;
    actor.intent = FullIntentId::SpikeLick;
    entry
}

pub fn cards_entry(cards: &[CardId], seed: u64) -> CombatEntry {
    let mut entry = fixture(EnemyId::JawWorm, seed);
    entry.cards = cards.iter().enumerate().map(|(i, id)|
        permanent_live_card(*id, CardInstanceId(i as u64 + 1), 0)).collect();
    entry.opening_draw = cards.len();
    entry.setup.player.draw_per_turn = cards.len() as i32;
    entry
}

/// Caller-side budget only: choices and all game consequences stay in Combat.
pub struct Script { accepted: usize, limit: usize }

impl Script {
    pub fn new(limit: usize) -> Self { Self { accepted: 0, limit: limit.min(64) } }
    pub fn accepted(&self) -> usize { self.accepted }

    pub fn step(&mut self, combat: &mut Combat, action: CombatAction) -> Result<(), DemoError> {
        if self.accepted == self.limit { return Err(DemoError::BudgetExceeded { accepted: self.accepted }); }
        if !combat.legal_actions_full()?.contains(&action) {
            return Err(spire_core::run::RunError::InvalidAction.into());
        }
        combat.step_full(&action)?;
        self.accepted += 1;
        Ok(())
    }
}
