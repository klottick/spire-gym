use super::*;
use spire_core::{
    combat::{enemies, CardInstanceId, Combat, PlayerState},
    items::CombatItemContext,
    run::{Candidate, ChoiceId},
};

// These supplied public-state fixtures test action selection only. They never
// roll a fixed seed/boss scenario or claim a gameplay completion.
fn public_combat(cards: &[(CardId, u32)], potions: Vec<Option<PotionId>>) -> (PublicDecision, spire_core::combat::CombatPublic) {
    let mut rng = RunRng::new([42; 32], Partition::Test, 0);
    let setup = enemies::setup(EncounterId::Slavers, 2, 0, &mut rng, enemies::SetupContext {
        player: PlayerState { hp: 70, max_hp: 70, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
        player_powers: vec![], character: Character::Silent,
        cards: cards.iter().enumerate().map(|(i, (id, upgrades))|
            spire_core::combat::permanent_live_card(*id, CardInstanceId(i as u64 + 1), *upgrades)
        ).collect(), opening_draw: cards.len(), encounter_visit: 0,
    }).unwrap();
    let mut inventory = Some(CombatItemContext::new(vec![], potions));
    let combat = Combat::enter_with_item_context(setup, &mut inventory).unwrap();
    let legal = combat.legal_actions_full().unwrap();
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    let mut decision = Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 }, profile,
        RunRng::new([42; 32], Partition::Test, 0), spire_core::run::RoomScenario::M13SilentAct2,
    ).unwrap().decision().unwrap();
    decision.candidates = legal.into_iter().enumerate().map(|(i, action)| Candidate {
        id: ChoiceId(i as u64 + 1), action: GameAction::Combat(action), label: "supplied public legal action", neow_offer: None, object: None,
    }).collect();
    let public = combat.public_full().unwrap();
    (decision, public)
}

#[test]
fn policy_revision_v2_survival_precedes_scaling_at_low_health() {
    let (decision, mut combat) = public_combat(&[(CardId::NoxiousFumes, 0), (CardId::Survivor, 0), (CardId::DefendGreen, 1)], vec![]);
    combat.player.hp = 12;
    for (enemy, damage) in combat.enemies.iter_mut().zip([7, 7, 13]) { enemy.intent_damage = Some(damage); }
    let action = combat_action(&decision, &combat).unwrap();
    assert!(matches!(action, GameAction::Combat(CombatAction::PlayCard { instance: 2 | 3, .. })), "block precedes setup under immediate lethal pressure: {action:?}");
}

#[test]
fn policy_revision_v2_free_defense_preserves_energy_before_paid_setup() {
    let (decision, mut combat) = public_combat(&[(CardId::NoxiousFumes, 0), (CardId::Deflect, 0), (CardId::Survivor, 0)], vec![]);
    combat.player.hp = 12;
    let action = combat_action(&decision, &combat).unwrap();
    assert_eq!(action, GameAction::Combat(CombatAction::PlayCard { instance: 2, target: None }));
}

#[test]
fn policy_revision_v2_weak_potion_targets_greatest_unblocked_threat() {
    let (decision, mut combat) = public_combat(&[(CardId::DefendGreen, 0)], vec![Some(PotionId::WeakPotion)]);
    combat.player.hp = 12;
    combat.enemies[0].hp = 10;
    combat.enemies[1].hp = 45;
    combat.enemies[2].hp = 50;
    for (enemy, damage) in combat.enemies.iter_mut().zip([7, 7, 13]) { enemy.intent_damage = Some(damage); }
    assert_eq!(combat_action(&decision, &combat).unwrap(), GameAction::Combat(CombatAction::UsePotion { slot: 0, target: Some(TargetId(3)) }));
}

#[test]
fn policy_revision_v2_fire_potion_prefers_finishable_attacking_target() {
    let (decision, mut combat) = public_combat(&[(CardId::DefendGreen, 0)], vec![Some(PotionId::FirePotion)]);
    combat.player.hp = 12;
    for (enemy, (hp, damage)) in combat.enemies.iter_mut().zip([(5, 0), (45, 0), (18, 13)]) {
        enemy.hp = hp; enemy.block = 0; enemy.intent_damage = Some(damage);
    }
    assert_eq!(combat_action(&decision, &combat).unwrap(), GameAction::Combat(CombatAction::UsePotion { slot: 0, target: Some(TargetId(3)) }));
}

#[test]
fn policy_revision_v2_strength_is_saved_when_no_attack_is_offered() {
    let (decision, mut combat) = public_combat(&[(CardId::NoxiousFumes, 0), (CardId::DefendGreen, 0)], vec![Some(PotionId::StrengthPotion)]);
    for enemy in &mut combat.enemies { enemy.intent_damage = Some(0); }
    assert_eq!(combat_action(&decision, &combat).unwrap(), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: None }));
}

#[test]
fn policy_revision_v2_urgent_attacks_prioritize_visible_threat_over_lowest_hp() {
    let (decision, mut combat) = public_combat(&[(CardId::StrikeGreen, 0)], vec![]);
    combat.player.hp = 12;
    for (enemy, (hp, damage)) in combat.enemies.iter_mut().zip([(30, 5), (50, 0), (35, 13)]) {
        enemy.hp = hp; enemy.block = 0; enemy.intent_damage = Some(damage);
    }
    assert_eq!(combat_action(&decision, &combat).unwrap(), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: Some(TargetId(3)) }));
}

#[test]
fn policy_revision_v2_safe_blocked_turn_still_allows_existing_scaling() {
    let (decision, mut combat) = public_combat(&[(CardId::NoxiousFumes, 0), (CardId::DefendGreen, 0)], vec![]);
    for enemy in &mut combat.enemies { enemy.intent_damage = Some(0); }
    assert_eq!(combat_action(&decision, &combat).unwrap(), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: None }));
}
