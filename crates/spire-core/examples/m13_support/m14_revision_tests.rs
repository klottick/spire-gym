//! Supplied public-state caller checks; no fixed seed/boss rollouts.
use super::*;
use spire_core::{
    combat::{enemies, CardInstanceId, Combat, PlayerState},
    items::CombatItemContext,
    run::{Candidate, ChoiceId, RoomScenario},
};

pub(super) fn supplied(cards: &[(u64, CardId, u32)], energy: i32, encounter: EncounterId) -> PublicDecision {
    let mut rng = RunRng::new([42; 32], Partition::Test, 0);
    let act = if encounter == EncounterId::GiantHead { 3 } else { 2 };
    let entry = enemies::setup(encounter, act, 0, &mut rng, enemies::SetupContext {
        player: PlayerState { hp: 70, max_hp: 70, block: 0, energy, max_energy: energy, draw_per_turn: 5, powers: vec![] },
        player_powers: vec![], character: Character::Silent,
        cards: cards.iter().map(|(instance, id, upgrades)| spire_core::combat::permanent_live_card(*id, CardInstanceId(*instance), *upgrades)).collect(),
        opening_draw: cards.len(), encounter_visit: 0,
    }).unwrap();
    let mut items = Some(CombatItemContext::new(vec![], vec![]));
    let combat = Combat::enter_with_item_context(entry, &mut items).unwrap();
    let legal = combat.legal_actions_full().unwrap();
    let mut profile = CreationProfile::experienced(); profile.neow_eligible = false;
    let mut decision = Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 }, profile,
        RunRng::new([42; 32], Partition::Test, 0), RoomScenario::M14SilentAct3,
    ).unwrap().decision().unwrap();
    decision.candidates = legal.into_iter().enumerate().map(|(i, action)| Candidate {
        id: ChoiceId(i as u64 + 1), action: GameAction::Combat(action), label: "supplied legal combat action", neow_offer: None, object: None,
    }).collect();
    decision.combat = Some(combat.public_full().unwrap());
    decision
}

fn choose(d: &PublicDecision) -> GameAction {
    action_for(d, None, &Caller::new_m14(8192).unwrap()).unwrap()
}

#[test]
fn m14_revision_unreserved_paid_defense_precedes_optional_setup() {
    // Wrong branch: the old nonurgent path spends on Fumes before legal block.
    for defense in [CardId::Backflip, CardId::Survivor, CardId::CloakAndDagger, CardId::DefendGreen] {
        let mut d = supplied(&[(1, CardId::NoxiousFumes, 0), (2, defense, 0)], 1, EncounterId::Slavers);
        let c = d.combat.as_mut().unwrap();
        c.player.hp = 70; c.player.max_hp = 70; c.player.block = 0;
        for enemy in &mut c.enemies { enemy.intent_damage = Some(3); }
        assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 2, target: None }), "uncovered nonurgent damage with {defense:?}");
    }
}

#[test]
fn m14_revision_covered_turn_preserves_setup_order() {
    let mut d = supplied(&[(1, CardId::NoxiousFumes, 0), (2, CardId::DefendGreen, 0)], 3, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.block = 9;
    for enemy in &mut c.enemies { enemy.intent_damage = Some(3); }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: None }));
}

#[test]
fn m14_revision_free_deflect_still_precedes_paid_defense() {
    let mut d = supplied(&[(1, CardId::NoxiousFumes, 0), (2, CardId::DefendGreen, 0), (3, CardId::Deflect, 0)], 3, EncounterId::Slavers);
    for enemy in &mut d.combat.as_mut().unwrap().enemies { enemy.intent_damage = Some(3); }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 3, target: None }));
}

#[test]
fn m14_revision_unaffordable_defense_is_never_invented() {
    let mut d = supplied(&[(1, CardId::Shiv, 0), (2, CardId::DefendGreen, 0)], 3, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.energy = 0;
    for enemy in &mut c.enemies { enemy.intent_damage = Some(3); }
    // Supplied current menu: only the free Shiv and EndTurn remain legal.
    d.candidates.retain(|candidate| !matches!(candidate.action, GameAction::Combat(CombatAction::PlayCard { instance: 2, .. })));
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }));
}

#[test]
fn m14_revision_saved_giant_head_discarded_defends_are_not_playable() {
    let mut d = supplied(&[
        (17, CardId::BladeDance, 0), (36, CardId::QuickSlash, 0),
        (29, CardId::DeadlyPoison, 0), (1, CardId::StrikeGreen, 1),
        (37, CardId::Catalyst, 0), (9, CardId::DefendGreen, 1), (10, CardId::DefendGreen, 1),
    ], 4, EncounterId::GiantHead);
    let c = d.combat.as_mut().unwrap();
    c.turn = 5; c.player.hp = 29; c.player.max_hp = 87; c.player.block = 0; c.player.energy = 4;
    c.enemies[0].hp = 387; c.enemies[0].max_hp = 625;
    c.enemies[0].intent = spire_core::combat::FullIntentId::from_name("head_time");
    c.enemies[0].intent_damage = Some(30);
    let mut discard = Vec::new();
    c.hand.retain(|card| { if matches!(card.instance.0, 9 | 10) { discard.push(card.clone()); false } else { true } });
    c.discard.extend(discard);
    d.candidates.retain(|candidate| !matches!(candidate.action, GameAction::Combat(CombatAction::PlayCard { instance: 9 | 10, .. })));
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 17, target: None }));
    assert_eq!(d.combat.as_ref().unwrap().discard.iter().filter(|card| card.id == CardId::DefendGreen).count(), 2);
}

#[test]
fn m14_revision_existing_weak_potion_target_is_preserved() {
    let mut d = supplied(&[(1, CardId::DefendGreen, 0)], 3, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.hp = 12;
    c.inventory.as_mut().unwrap().potions = vec![Some(PotionId::WeakPotion)];
    for (enemy, damage) in c.enemies.iter_mut().zip([7, 7, 13]) { enemy.intent_damage = Some(damage); }
    d.candidates.push(Candidate { id: ChoiceId(100), action: GameAction::Combat(CombatAction::UsePotion { slot: 0, target: Some(TargetId(3)) }), label: "supplied legal Weak potion", neow_offer: None, object: None });
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::UsePotion { slot: 0, target: Some(TargetId(3)) }));
}

#[test]
fn m14_revision_summary_reports_act3_events_without_changing_m13() {
    let d = supplied(&[(1, CardId::DefendGreen, 0)], 3, EncounterId::GiantHead);
    let step = TraceStep {
        decision: d, map: None, action: GameAction::Combat(CombatAction::EndTurn),
        transition: RunTransition { reward: -1, terminal: Some(RunTerminal::Death), committed_events: vec![
            CommittedEvent::CombatEntered { visit: 20, encounter: EncounterId::TimeEater },
            CommittedEvent::CombatFinished(spire_core::run::CombatRecord { visit: 20, encounter: EncounterId::TimeEater, outcome: Outcome::Death }),
        ] },
    };
    let mut m14 = Caller::new_m14(8192).unwrap(); m14.trace.push(step.clone());
    assert_eq!(summarize(&m14).boss_combat_entries, [EncounterId::TimeEater]);
    assert_eq!(summarize(&m14).boss_combat_records, [(EncounterId::TimeEater, Outcome::Death)]);
    let mut m13 = Caller::new(4096).unwrap(); m13.trace.push(step);
    assert!(summarize(&m13).boss_combat_entries.is_empty());
    assert!(summarize(&m13).boss_combat_records.is_empty());
}

#[test]
fn m14_revision_v3_fumes_sets_up_when_current_defense_energy_is_reserved() {
    let mut d = supplied(&[(1, CardId::NoxiousFumes, 0), (2, CardId::DefendGreen, 0)], 2, EncounterId::Slavers);
    for enemy in &mut d.combat.as_mut().unwrap().enemies { enemy.intent_damage = Some(3); }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: None }));
}

#[test]
fn m14_revision_v3_zero_cost_weak_precedes_paid_defense_when_not_urgent() {
    let mut d = supplied(&[(1, CardId::Neutralize, 0), (2, CardId::DefendGreen, 0)], 3, EncounterId::Slavers);
    for enemy in &mut d.combat.as_mut().unwrap().enemies { enemy.intent_damage = Some(3); }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }));
}

#[test]
fn m14_revision_v3_catalyst_uses_the_largest_current_poison_target() {
    let mut d = supplied(&[(1, CardId::Catalyst, 0), (2, CardId::BladeDance, 0)], 3, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.block = 100;
    c.enemies[0].powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("poison"), amount: 2 });
    c.enemies[1].powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("poison"), amount: 8 });
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: Some(TargetId(2)) }));
}

#[test]
fn m14_revision_v3_unpoisoned_catalyst_waits_for_a_current_poison_source() {
    let mut d = supplied(&[(1, CardId::Catalyst, 0), (2, CardId::DeadlyPoison, 0)], 3, EncounterId::Slavers);
    d.combat.as_mut().unwrap().player.block = 100;
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 2, target: Some(TargetId(1)) }));
}

#[test]
fn m14_revision_v3_lethal_lower_bound_still_takes_defense_before_setup() {
    let mut d = supplied(&[(1, CardId::NoxiousFumes, 0), (2, CardId::DefendGreen, 0)], 2, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.hp = 9;
    for enemy in &mut c.enemies { enemy.intent_damage = Some(3); }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 2, target: None }));
}

#[test]
fn m14_revision_v3_zero_poison_catalyst_is_not_spent_without_a_source() {
    let mut d = supplied(&[(1, CardId::Catalyst, 0)], 3, EncounterId::Slavers);
    d.combat.as_mut().unwrap().player.block = 100;
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::EndTurn));
}

#[test]
fn m14_revision_v3_small_uncovered_lower_bound_can_set_up_without_defense_in_hand() {
    let mut d = supplied(&[(1, CardId::NoxiousFumes, 0), (2, CardId::BladeDance, 0)], 2, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.hp = 20;
    for enemy in &mut c.enemies { enemy.intent_damage = Some(2); }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: None }));
}

#[test]
fn m14_revision_v4_poison_source_precedes_funded_catalyst() {
    let mut d = supplied(&[(1, CardId::Catalyst, 0), (2, CardId::DeadlyPoison, 0)], 3, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.block = 100;
    c.enemies[1].powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("poison"), amount: 8 });
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 2, target: Some(TargetId(2)) }));
}

#[test]
fn m14_revision_v4_unfunded_poison_source_does_not_delay_catalyst() {
    let mut d = supplied(&[(1, CardId::Catalyst, 0), (2, CardId::DeadlyPoison, 0)], 1, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.block = 100;
    c.enemies[1].powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("poison"), amount: 8 });
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: Some(TargetId(2)) }));
}

#[test]
fn m14_revision_v4_attack_prefers_a_public_target_without_intangible() {
    let mut d = supplied(&[(1, CardId::StrikeGreen, 0)], 3, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.block = 100;
    c.enemies[0].hp = 10;
    c.enemies[0].powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("intangible"), amount: 1 });
    c.enemies[1].hp = 30; c.enemies[2].hp = 50;
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: Some(TargetId(2)) }));
}

#[test]
fn m14_revision_v4_uncovered_lethal_retaliation_prefers_poison_over_attack() {
    let mut d = supplied(&[(1, CardId::Shiv, 0), (2, CardId::DeadlyPoison, 0)], 3, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.player.hp = 3;
    for enemy in &mut c.enemies { enemy.intent_damage = Some(0); }
    c.enemies[0].powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("thorns"), amount: 3 });
    for row in [1, 2] { c.enemies[row].hp = 0; c.targets[row].1 = false; }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 2, target: Some(TargetId(1)) }));
}

#[test]
fn m14_revision_v5_active_poison_scaling_prefers_stronger_printed_block() {
    let mut d = supplied(&[(1, CardId::Backflip, 0), (2, CardId::DefendGreen, 1)], 3, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap();
    c.player.powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("noxious_fumes"), amount: 3 });
    for enemy in &mut c.enemies { enemy.intent_damage = Some(3); }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 2, target: None }));
}

#[test]
fn m14_revision_v5_without_passive_scaling_keeps_draw_block_priority() {
    let mut d = supplied(&[(1, CardId::Backflip, 0), (2, CardId::DefendGreen, 1)], 3, EncounterId::Slavers);
    for enemy in &mut d.combat.as_mut().unwrap().enemies { enemy.intent_damage = Some(3); }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: None }));
}

#[test]
fn m14_revision_v5_published_shifting_prefers_attack_to_reduce_current_threat() {
    let mut d = supplied(&[(1, CardId::BladeDance, 0), (2, CardId::DefendGreen, 0)], 1, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap();
    c.enemies[0].powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("shifting"), amount: 1 });
    c.enemies[0].intent_damage = Some(30);
    for row in [1, 2] { c.enemies[row].hp = 0; c.targets[row].1 = false; }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: None }));
}

#[test]
fn m14_revision_v5_no_shifting_preserves_current_defense_priority() {
    let mut d = supplied(&[(1, CardId::BladeDance, 0), (2, CardId::DefendGreen, 0)], 1, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.enemies[0].intent_damage = Some(30);
    for row in [1, 2] { c.enemies[row].hp = 0; c.targets[row].1 = false; }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 2, target: None }));
}

#[test]
fn m14_revision_v5_shifting_current_threat_precedes_optional_setup() {
    let mut d = supplied(&[(1, CardId::NoxiousFumes, 0), (2, CardId::BladeDance, 0), (3, CardId::DefendGreen, 0)], 2, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.enemies[0].intent_damage = Some(30);
    c.enemies[0].powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("shifting"), amount: 1 });
    for row in [1, 2] { c.enemies[row].hp = 0; c.targets[row].1 = false; }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 2, target: None }));
}

#[test]
fn m14_revision_v5_shifting_prefers_attack_draw_before_poisoned_stab() {
    let mut d = supplied(&[(1, CardId::DaggerThrow, 0), (2, CardId::PoisonedStab, 0)], 2, EncounterId::Slavers);
    let c = d.combat.as_mut().unwrap(); c.enemies[0].intent_damage = Some(30);
    c.enemies[0].powers.push(spire_core::combat::FullPowerView { id: spire_core::combat::FullPowerId("shifting"), amount: 1 });
    for row in [1, 2] { c.enemies[row].hp = 0; c.targets[row].1 = false; }
    assert_eq!(choose(&d), GameAction::Combat(CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }));
}
