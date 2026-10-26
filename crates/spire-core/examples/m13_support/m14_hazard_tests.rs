//! Supplied public-state checks for M14's current hazard pressure estimate.
//!
//! These exercise the shared dispatcher with a public combat view and legal
//! actions. They do not run a seeded encounter or claim an outcome.
use super::*;
use super::m14_revision_tests::supplied;

fn choose(decision: &PublicDecision) -> GameAction {
    action_for(decision, None, &Caller::new_m14(M14_ACTION_LIMIT_MAX).unwrap()).unwrap()
}

fn decision(cards: &[(u64, CardId, u32)], block: i32, enemy_damage: i32) -> PublicDecision {
    let mut decision = supplied(cards, 3, EncounterId::Slavers);
    let combat = decision.combat.as_mut().unwrap();
    combat.player.block = block;
    for enemy in &mut combat.enemies {
        enemy.intent_damage = Some(0);
    }
    if let Some(enemy) = combat.enemies.first_mut() {
        enemy.intent_damage = Some(enemy_damage);
    }
    decision
}

fn add_player_power(decision: &mut PublicDecision, id: &'static str, amount: i32) {
    decision
        .combat
        .as_mut()
        .unwrap()
        .player
        .powers
        .push(spire_core::combat::FullPowerView {
            id: spire_core::combat::FullPowerId(id),
            amount,
        });
}

fn play(instance: u64, target: Option<TargetId>) -> GameAction {
    GameAction::Combat(CombatAction::PlayCard { instance, target })
}

#[test]
fn m14_revision_hazard_held_burn_adds_pressure_when_enemy_intent_is_zero() {
    let decision = decision(
        &[
            (1, CardId::Burn, 0),
            (2, CardId::DefendGreen, 0),
            (3, CardId::Shiv, 0),
        ],
        0,
        0,
    );

    assert_eq!(choose(&decision), play(2, None));
}

#[test]
fn m14_revision_hazard_burn_uses_its_printed_two_or_four_damage() {
    for (upgrades, expected) in [(0, play(3, Some(TargetId(1)))), (1, play(2, None))] {
        let decision = decision(
            &[
                (1, CardId::Burn, upgrades),
                (2, CardId::DefendGreen, 0),
                (3, CardId::Shiv, 0),
            ],
            2,
            0,
        );

        assert_eq!(choose(&decision), expected, "Burn upgrades={upgrades}");
    }
}

#[test]
fn m14_revision_hazard_positive_constricted_adds_current_player_damage() {
    let mut decision = decision(
        &[(1, CardId::DefendGreen, 0), (2, CardId::Shiv, 0)],
        0,
        0,
    );
    add_player_power(&mut decision, "constricted", 10);

    assert_eq!(choose(&decision), play(1, None));
}

#[test]
fn m14_revision_hazard_adds_to_enemy_damage_already_covered_by_block() {
    let decision = decision(
        &[
            (1, CardId::Burn, 0),
            (2, CardId::DefendGreen, 0),
            (3, CardId::Shiv, 0),
        ],
        3,
        3,
    );

    assert_eq!(choose(&decision), play(2, None));
}

#[test]
fn m14_revision_hazard_covered_combined_threat_preserves_offense() {
    let mut decision = decision(
        &[(1, CardId::DefendGreen, 0), (2, CardId::Shiv, 0)],
        10,
        0,
    );
    add_player_power(&mut decision, "constricted", 10);

    assert_eq!(choose(&decision), play(2, Some(TargetId(1))));
}
