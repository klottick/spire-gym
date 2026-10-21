use crate::*;
use spire_core::combat::{CardKind, EnemyKind};

pub fn template() -> ScenarioTemplate {
    ScenarioTemplate {
        id: "authored".into(), encounter: EnemyKind::Cultist,
        player_hp: 80, player_max_hp: 80, energy_per_turn: 3,
        draw_per_turn: 5, enemy_hp: 52,
        hand: vec![CardSpec { kind: CardKind::Strike, upgraded: false }],
        draw: vec![], context: PublicContext { act: None, next_encounter: None },
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
