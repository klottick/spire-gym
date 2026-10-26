//! Current published blockable threat used by the finite M14 caller.
//!
//! This is a lower bound recomputed from the public combat view at each
//! decision. It does not predict later enemy actions, count hidden hits, or
//! call combat rules.
use spire_core::{combat::CombatPublic, content::CardId};

pub(super) fn incoming(combat: &CombatPublic) -> i32 {
    let mut damage = 0_i32;

    for (row, enemy) in combat.enemies.iter().enumerate() {
        if super::targetable(combat, row) {
            if let Some(intent_damage) = enemy.intent_damage {
                damage = damage.saturating_add(intent_damage.max(0));
            }
        }
    }

    for power in &combat.player.powers {
        if power.id.0 == "constricted" && power.amount > 0 {
            damage = damage.saturating_add(power.amount);
        }
    }

    for card in &combat.hand {
        if card.id == CardId::Burn {
            damage = damage.saturating_add(if card.upgrades > 0 { 4 } else { 2 });
        }
    }

    damage
}
