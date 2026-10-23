use super::super::*;
use crate::content::{CardId, Character, EnemyId};
use super::full_runtime::full_stream;

pub(super) fn entry() -> CombatEntry {
    CombatEntry {
        setup: FullCombatSetup {
            player: PlayerState { hp: 40, max_hp: 50, block: 0, energy: 3,
                max_energy: 3, draw_per_turn: 3, powers: vec![] },
            player_powers: vec![], character: Character::Ironclad,
            shuffle_rng: full_stream(rng::Domain::RunCombatShuffle, vec!["m2-rules"]),
            card_rng: full_stream(rng::Domain::RunCombatCardRandom, vec!["m2-rules"]),
            enemies: vec![FullEnemySetup {
                instance_id: EnemyHandle::parse("enemy-1").unwrap(), enemy_id: EnemyId::JawWorm,
                ascension: 0, profile: enemies::EnemyProfile::Standard, hp: 42, max_hp: 42,
                block: 0, powers: vec![], intent: FullIntentId::Chomp,
                ai_rng: full_stream(rng::Domain::RunCombatEnemyAi, vec!["m2-rules", "enemy-1"]),
            }], enemy_damage_bases: Default::default(),
        }, opening_draw: 3,
        cards: [CardId::Strike, CardId::Defend, CardId::Bash].into_iter().enumerate()
            .map(|(i, id)| permanent_live_card(id, CardInstanceId(i as u64 + 1), 0)).collect(),
    }
}

fn apply(combat: &mut Combat, target: Actor, id: &'static str, amount: i32) {
    combat.enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Player), target: Some(target),
        kind: EffectKind::ApplyFullPower { id: FullPowerId(id), amount } }], None).unwrap();
}

fn amount(combat: &Combat, target: &Actor, id: &str) -> i32 {
    let public = combat.public_full().unwrap();
    let powers = match target {
        Actor::Player => public.player.powers,
        Actor::Enemy(owner) => public.enemies.into_iter().find(|e| e.instance_id == *owner).unwrap().powers,
    };
    powers.iter().filter(|p| p.id.0 == id).map(|p| p.amount).sum()
}

#[test]
fn signed_stats_stack_refresh_cross_zero_and_clamp_without_duplicate_owners() {
    let enemy = Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap());
    let mut combat = Combat::enter(entry()).unwrap();
    apply(&mut combat, Actor::Player, "strength", 2);
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 34, "Strike6 + Strength2");
    apply(&mut combat, enemy.clone(), "strength", -2);
    assert_eq!(combat.public_full().unwrap().enemies[0].intent_damage, Some(9));
    apply(&mut combat, Actor::Player, "strength", -2);
    assert!(!combat.public_full().unwrap().player.powers.iter().any(|p| p.id.0 == "strength"));
    apply(&mut combat, Actor::Player, "strength", 1002);
    assert_eq!(amount(&combat, &Actor::Player, "strength"), 999);
    apply(&mut combat, Actor::Player, "strength", -1002);
    assert_eq!(amount(&combat, &Actor::Player, "strength"), -3);
    apply(&mut combat, Actor::Player, "strength", -1002);
    assert_eq!(amount(&combat, &Actor::Player, "strength"), -999);
    assert!(engine::validate_runtime(&combat.state).is_ok());

    let mut legacy = Combat::enter(entry()).unwrap();
    legacy.state.player.powers.push(PowerEntry { id: PowerId::Strength,
        state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } });
    apply(&mut legacy, Actor::Player, "strength", -3);
    assert_eq!(amount(&legacy, &Actor::Player, "strength"), -1);
    assert_eq!(legacy.public_full().unwrap().player.powers.len(), 1);
    assert!(engine::validate_runtime(&legacy.state).is_ok());
}

#[test]
fn artifact_blocks_applications_not_buffs_reductions_or_existing_debuffs() {
    let enemy = Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap());
    let mut combat = Combat::enter(entry()).unwrap();
    apply(&mut combat, enemy.clone(), "weak", 2);
    apply(&mut combat, enemy.clone(), "artifact", 1);
    apply(&mut combat, enemy.clone(), "strength", 2);
    assert_eq!(amount(&combat, &enemy, "artifact"), 1, "buff doesn't spend Artifact");
    apply(&mut combat, enemy.clone(), "weak", 3);
    assert_eq!(amount(&combat, &enemy, "weak"), 2, "prevent the whole attempted stack, don't cleanse old Weak");
    assert_eq!(amount(&combat, &enemy, "artifact"), 0);
    apply(&mut combat, enemy.clone(), "artifact", 1);
    apply(&mut combat, enemy.clone(), "strength", -2);
    assert_eq!(amount(&combat, &enemy, "strength"), 2, "negative stat application is a debuff");
    apply(&mut combat, enemy.clone(), "artifact", 1);
    combat.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(enemy.clone()),
        kind: EffectKind::ReduceFullPower { id: FullPowerId("strength"), amount: 2 } }], None).unwrap();
    assert_eq!(amount(&combat, &enemy, "strength"), 0);
    assert_eq!(amount(&combat, &enemy, "artifact"), 1, "ordinary reduction isn't an application");
    apply(&mut combat, enemy.clone(), "vulnerable", 2);
    apply(&mut combat, enemy.clone(), "frail", 1);
    assert_eq!(amount(&combat, &enemy, "vulnerable"), 0);
    assert_eq!(amount(&combat, &enemy, "frail"), 1, "second application sees charge consumed");
}

#[test]
fn card_block_applies_signed_dexterity_and_frail_once_but_direct_block_does_not() {
    for (dex, frail, expected) in [(0, false, 5), (2, false, 7), (0, true, 3),
        (2, true, 5), (-2, true, 2), (-9, false, 0)] {
        let mut fixture = entry();
        for (id, value) in [("dexterity", dex), ("frail", i32::from(frail))] {
            if value != 0 { fixture.setup.player_powers.push(FullPowerEntry { id: FullPowerId(id),
                state: PowerState { amount: value, skip_first: false, monster_source_grace: false } }); }
        }
        let mut combat = Combat::enter(fixture).unwrap();
        combat.step_full(&CombatAction::PlayCard { instance: 2, target: None }).unwrap();
        assert_eq!(combat.public_full().unwrap().player.block, expected, "Dex{dex} Frail{frail}");
    }
    let mut combat = Combat::enter(entry()).unwrap();
    apply(&mut combat, Actor::Player, "dexterity", 2);
    apply(&mut combat, Actor::Player, "frail", 2);
    combat.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::GainBlock(12) }], None).unwrap();
    assert_eq!(combat.public_full().unwrap().player.block, 12);
    combat.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::GainBlock(i32::MAX) }], None).unwrap();
    assert_eq!(combat.public_full().unwrap().player.block, 999, "wide addition retains the actual Block cap");
}

#[test]
fn card_block_producers_share_the_calculation_without_scaling_damage_or_direct_gains() {
    for (card, target, block) in [(CardId::Defend, None, 7),
        (CardId::IronWave, Some(TargetId(1)), 7), (CardId::ShrugItOff, None, 10)] {
        let mut setup = entry();
        setup.cards[1].id = card;
        let mut combat = Combat::enter(setup).unwrap();
        apply(&mut combat, Actor::Player, "dexterity", 2);
        combat.step_full(&CombatAction::PlayCard { instance: 2, target }).unwrap();
        assert_eq!(combat.public_full().unwrap().player.block, block, "{card:?}");
        if card == CardId::IronWave { assert_eq!(combat.public_full().unwrap().enemies[0].hp, 37); }
    }
}

#[test]
fn monster_source_frail_survives_application_round_then_expires() {
    let mut combat = Combat::enter(entry()).unwrap();
    combat.enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
        target: Some(Actor::Player), kind: EffectKind::ApplyFullPower { id: FullPowerId("frail"), amount: 1 } }], None).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: 2, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().player.block, 3);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(amount(&combat, &Actor::Player, "frail"), 1);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(amount(&combat, &Actor::Player, "frail"), 0);
}

#[test]
fn hp_loss_bypasses_block_and_modifiers_and_lethal_loss_prunes_later_energy() {
    for (hp, expected, energy, outcome) in [(20, 17, 4, None), (3, 0, 2, Some(Outcome::Death))] {
        let mut setup = entry();
        setup.setup.player.hp = hp;
        let mut combat = Combat::enter(setup).unwrap();
        combat.step_full(&CombatAction::PlayCard { instance: 2, target: None }).unwrap();
        for (id, amount) in [("strength", 2), ("weak", 2), ("vulnerable", 2)] {
            apply(&mut combat, Actor::Player, id, amount);
        }
        let effect = |kind| Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player), kind };
        combat.enqueue_native_effects(vec![effect(EffectKind::Damage(DamagePayload::HpLoss(3))),
            effect(EffectKind::GainEnergy(2))], None).unwrap();
        let view = combat.public_full().unwrap();
        assert_eq!((view.player.hp, view.player.block, view.player.energy, view.outcome), (expected, 5, energy, outcome));
        if outcome.is_some() { assert!(combat.legal_actions_full().unwrap().is_empty()); }
    }
    let mut combat = Combat::enter(entry()).unwrap();
    let enemy = Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap());
    apply(&mut combat, Actor::Player, "strength", 2);
    apply(&mut combat, Actor::Player, "weak", 2);
    apply(&mut combat, enemy.clone(), "vulnerable", 2);
    combat.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(enemy.clone()), kind: EffectKind::GainBlock(7) },
        Effect { source: Source::Actor(Actor::Player), target: Some(enemy), kind: EffectKind::Damage(DamagePayload::Thorns(20)) }], None).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 29, "THORNS20 absorbs Block7, loses13HP, no NORMAL modifiers");
}

#[test]
fn accepted_potion_fault_retains_consumed_slot_action_and_owned_partial_work_without_retry() {
    let mut items = Some(crate::items::CombatItemContext::new(vec![], vec![Some(crate::content::PotionId::SwiftPotion)]));
    let mut combat = Combat::enter_with_item_context(entry(), &mut items).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: 2, target: None }).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }).unwrap();
    // Valid exhausted sentinel at a ready boundary. A two-card recycle must
    // draw entropy only after accepted potion use, not during menu preparation.
    let shuffle = &combat.state.rng.shuffle;
    combat.state.rng.shuffle = rng::StreamState::new(shuffle.id().clone(), *shuffle.key(), 1_u128 << 64).unwrap();
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let action = CombatAction::UsePotion { slot: 0, target: None };
    assert!(combat.legal_actions_full().unwrap().contains(&action));
    let error = combat.step_full(&action).unwrap_err();
    assert_eq!(combat.item_context().unwrap().potions(), &[None]);
    let crate::run::RunError::Faulted(fault) = &error else { panic!("retained fault expected") };
    let crate::run::RunFaultKind::Combat(packet) = &fault.kind else { panic!("native packet expected") };
    assert_eq!(packet.attempted_action, Some(action.clone()));
    assert_eq!(packet.current.as_ref().unwrap().kind, EffectKind::Recycle);
    let before = combat.snapshot();
    assert_eq!(combat.step_full(&action), Err(error));
    assert_eq!(combat.snapshot(), before);
}

#[test]
fn selected_forced_and_finish_exhaustion_notify_once_for_the_actual_owned_identity() {
    for cards in [vec![CardId::Strike, CardId::Defend, CardId::BurningPact],
        vec![CardId::Defend, CardId::BurningPact], vec![CardId::Disarm]] {
        let mut setup = entry();
        setup.opening_draw = cards.len();
        setup.cards = cards.iter().enumerate().map(|(i, id)| permanent_live_card(*id, CardInstanceId(i as u64 + 1), 0)).collect();
        let mut combat = Combat::enter(setup).unwrap();
        let parent = cards.len() as u64;
        combat.step_full(&CombatAction::PlayCard { instance: parent,
            target: if cards[0] == CardId::Disarm { Some(TargetId(1)) } else { None } }).unwrap();
        if combat.public_full().unwrap().selection.is_some() {
            combat.step_full(&CombatAction::SelectCard { instance: 1 }).unwrap();
        }
        let exhausted = Source::Card(CardHandle::parse("1").unwrap());
        assert_eq!(combat.state.hook_events.iter().filter(|e| e.hook == Hook::CardExhausted && e.source == exhausted).count(), 1);
        assert!(!combat.state.hook_events.iter().any(|e| e.hook == Hook::CardDiscarded && e.source == exhausted));
        assert_eq!(combat.public_full().unwrap().exhaust[0].instance, CardInstanceId(1));
    }
}
