use spire_core::{combat::*, content::{CardId, Character, EnemyId, PotionId, RelicId},
    items::CombatItemContext, run::RunError};
#[path = "../examples/m2_support/mod.rs"]
#[allow(dead_code)]
mod support;
use support::{fixture, card_action};

fn entered(entry: CombatEntry, potions: Vec<Option<PotionId>>) -> Combat {
    with_items(entry, vec![], potions)
}

fn with_items(entry: CombatEntry, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> Combat {
    let mut items = Some(CombatItemContext::new(relics, potions));
    let combat = Combat::enter_with_item_context(entry, &mut items).unwrap();
    assert!(items.is_none());
    combat
}

fn cards_entry(cards: &[CardId]) -> CombatEntry {
    support::cards_entry(cards, 7)
}

fn play(combat: &mut Combat, id: CardId) {
    combat.step_full(&card_action(combat, id).unwrap()).unwrap();
}

fn use_potion(combat: &mut Combat, slot: usize) {
    let action = CombatAction::UsePotion { slot, target: None };
    assert!(combat.legal_actions_full().unwrap().contains(&action));
    combat.step_full(&action).unwrap();
}

fn power(view: &CombatPublic, id: &str) -> i32 {
    view.player.powers.iter().find(|p| p.id.0 == id).map_or(0, |p| p.amount)
}

#[test]
fn potion_menu_and_inventory_reads_are_owned_and_invalid_uses_are_atomic() {
    let entry = fixture(EnemyId::JawWorm, 7);
    let potions = vec![Some(PotionId::StrengthPotion), None, Some(PotionId::FirePotion)];
    let mut combat = entered(entry.clone(), potions.clone());
    let before = combat.snapshot();
    let menu = combat.legal_actions_full().unwrap();
    let strength = CombatAction::UsePotion { slot: 0, target: None };
    let fire = CombatAction::UsePotion { slot: 2, target: Some(TargetId(1)) };
    assert!(menu.contains(&strength) && menu.contains(&fire));
    let mut retained = combat.public_full().unwrap();
    assert_eq!(retained.inventory.as_ref().unwrap().potions, potions);
    retained.inventory.as_mut().unwrap().potions[0] = None;
    assert_eq!(combat.public_full().unwrap().inventory.unwrap().potions, potions);
    assert_eq!(combat.legal_actions_full().unwrap(), menu);
    assert_eq!(combat.snapshot(), before);
    for action in [CombatAction::UsePotion { slot: 1, target: None },
        CombatAction::UsePotion { slot: 99, target: None },
        CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) },
        CombatAction::UsePotion { slot: 2, target: None },
        CombatAction::UsePotion { slot: 2, target: Some(TargetId(999)) }] {
        assert!(!menu.contains(&action));
        assert_eq!(combat.step_full(&action), Err(RunError::InvalidAction));
        assert_eq!(combat.snapshot(), before);
    }
    for action in menu {
        let mut independent = entered(entry.clone(), potions.clone());
        independent.step_full(&action).unwrap();
    }
    combat.step_full(&strength).unwrap();
    assert_eq!(combat.public_full().unwrap().player.energy, 3);
    assert_eq!(combat.public_full().unwrap().inventory.unwrap().potions[0], None);
    let used = combat.snapshot();
    assert_eq!(combat.step_full(&strength), Err(RunError::InvalidAction));
    assert_eq!(combat.snapshot(), used);
    let strike = card_action(&combat, CardId::Strike).unwrap();
    combat.step_full(&strike).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 34);
}

#[test]
fn selection_menu_exposes_only_captured_choices_and_resumes_parent_once() {
    let mut entry = fixture(EnemyId::JawWorm, 7);
    entry.cards[1].id = CardId::Survivor; // Existing off-color handler as an adapter fixture only.
    let mut combat = entered(entry, vec![Some(PotionId::BlockPotion)]);
    combat.step_full(&card_action(&combat, CardId::Survivor).unwrap()).unwrap();
    let view = combat.public_full().unwrap();
    let selection = view.selection.unwrap();
    assert_eq!(selection.required, 1);
    assert_eq!(selection.disposition, SelectionDisposition::Discard);
    assert!(selection.selected.is_empty());
    let before = combat.snapshot();
    let menu = combat.legal_actions_full().unwrap();
    assert_eq!(menu.len(), selection.candidates.len());
    assert!(menu.iter().all(|a| matches!(a, CombatAction::SelectCard { .. })));
    assert_eq!(combat.snapshot(), before);
    for action in [CombatAction::EndTurn,
        CombatAction::UsePotion { slot: 0, target: None },
        CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) },
        CombatAction::SelectCard { instance: 999 }] {
        assert_eq!(combat.step_full(&action), Err(RunError::InvalidAction));
        assert_eq!(combat.snapshot(), before);
    }
    let choice = selection.candidates.iter().find(|c| c.id == CardId::Clothesline).unwrap().instance.0;
    combat.step_full(&CombatAction::SelectCard { instance: choice }).unwrap();
    let view = combat.public_full().unwrap();
    assert!(view.selection.is_none());
    assert_eq!(view.player.energy, 2);
    assert_eq!(view.discard.len(), 2);
    assert_eq!(view.hand.len(), 2);
    let after = combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance: choice }), Err(RunError::InvalidAction));
    assert_eq!(combat.snapshot(), after);
}

#[test]
fn dead_target_and_terminal_potion_uses_do_not_spend_remaining_inventory() {
    let mut entry = fixture(EnemyId::JawWorm, 7);
    entry.setup.enemies[0].hp = 6;
    let mut second = entry.setup.enemies[0].clone();
    second.hp = 42;
    second.instance_id = EnemyHandle::parse("second").unwrap();
    second.ai_rng = rng::make_stream(&[9; 32], rng::Partition::Test, "m2",
        &rng::StreamId::new(rng::Domain::RunCombatEnemyAi,
            vec!["m1-authored-combat".into(), "second".into()]).unwrap()).unwrap();
    entry.setup.enemies.push(second);
    let mut combat = entered(entry, vec![Some(PotionId::FirePotion)]);
    combat.step_full(&card_action(&combat, CardId::Strike).unwrap()).unwrap();
    let dead = CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) };
    let before = combat.snapshot();
    assert!(!combat.legal_actions_full().unwrap().contains(&dead));
    assert_eq!(combat.step_full(&dead), Err(RunError::InvalidAction));
    assert_eq!(combat.snapshot(), before);

    let mut entry = fixture(EnemyId::JawWorm, 7);
    entry.setup.enemies[0].hp = 6;
    let mut combat = entered(entry, vec![Some(PotionId::FirePotion), Some(PotionId::BlockPotion)]);
    combat.step_full(&CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().outcome, Some(Outcome::Victory));
    assert!(combat.legal_actions_full().unwrap().is_empty());
    let before = combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::UsePotion { slot: 1, target: None }), Err(RunError::InvalidAction));
    assert_eq!(combat.snapshot(), before);
    assert_eq!(combat.public_full().unwrap().inventory.unwrap().potions[1], Some(PotionId::BlockPotion));
}

fn startup_fault(seed: u64) -> (CombatEntryFailure, Option<CombatItemContext>) {
    let mut entry = fixture(EnemyId::JawWorm, seed);
    entry.setup.character = Character::Watcher;
    entry.cards[0].instance = CardInstanceId(u64::MAX);
    entry.cards[0].origin = CombatCardRef::Permanent(CardInstanceId(u64::MAX));
    let mut inventory = Some(CombatItemContext::new(vec![RelicId::PureWater], vec![Some(PotionId::FirePotion)]));
    (Combat::enter_with_item_context(entry, &mut inventory).unwrap_err(), inventory)
}

#[test]
fn entry_rejection_keeps_inventory_and_post_transfer_fault_keeps_the_combat_privately() {
    for (relics, potions) in [(vec![RelicId::Unknown("unwired")], vec![]),
        (vec![], vec![Some(PotionId::Unknown("unwired"))])] {
        let mut inventory = Some(CombatItemContext::new(relics, potions));
        let before = inventory.clone();
        assert!(matches!(Combat::enter_with_item_context(fixture(EnemyId::JawWorm, 7), &mut inventory),
            Err(CombatEntryFailure::Rejected(RunError::Unsupported(_)))));
        assert_eq!(inventory, before);
    }
    let (failure, inventory) = startup_fault(7);
    assert!(inventory.is_none(), "accepted entry transferred its one inventory owner");
    let (other, _) = startup_fault(8);
    assert_eq!(format!("{failure:?}"), format!("{other:?}"), "entry error rendering must not expose retained Combat/hidden keys");
    let CombatEntryFailure::Faulted { mut combat, fault } = failure else { panic!("retained fault expected") };
    let before = combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) }),
        Err(RunError::Faulted(fault.clone())));
    assert_eq!(combat.legal_actions_full(), Err(RunError::Faulted(fault)));
    assert_eq!(combat.snapshot(), before);
}

#[test]
fn bloodletting_and_disarm_use_real_cost_ordering_signed_stats_and_destinations() {
    for (upgrades, energy, incoming) in [(0, 5, 9), (1, 6, 8)] {
        let mut entry = cards_entry(&[CardId::Bloodletting, CardId::Disarm]);
        for card in &mut entry.cards { card.upgrades = upgrades; }
        let mut combat = Combat::enter(entry).unwrap();
        play(&mut combat, CardId::Bloodletting);
        let view = combat.public_full().unwrap();
        assert_eq!((view.player.hp, view.player.energy), (77, energy));
        assert_eq!(view.discard.len(), 1);
        play(&mut combat, CardId::Disarm);
        let view = combat.public_full().unwrap();
        assert_eq!(view.enemies[0].intent_damage, Some(incoming));
        assert_eq!(view.player.energy, energy - 1);
        assert_eq!(view.exhaust.iter().filter(|c| c.id == CardId::Disarm).count(), 1);
    }
    let mut entry = cards_entry(&[CardId::Disarm]);
    entry.setup.enemies[0].powers.push(FullPowerEntry { id: FullPowerId("artifact"),
        state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
    let mut combat = Combat::enter(entry).unwrap();
    play(&mut combat, CardId::Disarm);
    let view = combat.public_full().unwrap();
    assert_eq!(view.enemies[0].intent_damage, Some(11));
    assert!(!view.enemies[0].powers.iter().any(|p| matches!(p.id.0, "artifact" | "strength")));
    assert_eq!(view.exhaust.len(), 1);

    let mut entry = cards_entry(&[CardId::Defend, CardId::Bloodletting]);
    entry.setup.player.hp = 3;
    let mut combat = Combat::enter(entry).unwrap();
    play(&mut combat, CardId::Defend);
    play(&mut combat, CardId::Bloodletting);
    let view = combat.public_full().unwrap();
    assert_eq!((view.player.hp, view.player.block, view.player.energy, view.outcome), (0, 5, 2, Some(Outcome::Death)));
    assert!(combat.legal_actions_full().unwrap().is_empty());
}

#[test]
fn burning_pact_selects_an_exact_copy_exhausts_then_recycles_without_repaying_parent() {
    let mut combat = Combat::enter(cards_entry(&[CardId::Strike, CardId::Strike, CardId::Defend,
        CardId::Defend, CardId::BurningPact, CardId::PommelStrike])).unwrap();
    for action in [CombatAction::PlayCard { instance: 3, target: None },
        CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) },
        CombatAction::PlayCard { instance: 5, target: None }] {
        assert!(combat.legal_actions_full().unwrap().contains(&action));
        combat.step_full(&action).unwrap();
    }
    assert_eq!(combat.public_full().unwrap().player.energy, 0);
    let action = CombatAction::SelectCard { instance: 4 };
    assert!(combat.legal_actions_full().unwrap().contains(&action));
    combat.step_full(&action).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!((view.hand.len(), view.exhaust.len(), view.discard.len(), view.player.energy), (4, 1, 1, 0));
    assert_eq!(view.exhaust[0].instance, CardInstanceId(4));
    assert_eq!(view.discard[0].instance, CardInstanceId(5));
    assert!(view.hand.iter().any(|c| c.instance == CardInstanceId(3)), "the played Defend sibling was recycled, not exhausted");
    assert!(view.hand.iter().any(|c| c.instance == CardInstanceId(1)));
    let before = combat.snapshot();
    assert_eq!(combat.step_full(&action), Err(RunError::InvalidAction));
    assert_eq!(combat.snapshot(), before);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.hand.len(), 5);
    assert!(!view.hand.iter().any(|c| c.instance == CardInstanceId(4)));
}

#[test]
fn burning_pact_empty_and_singleton_branches_draw_without_a_fake_choice() {
    for singleton in [false, true] {
        let mut cards = vec![CardId::Strike, CardId::Defend, CardId::BurningPact];
        if singleton { cards.push(CardId::PommelStrike); }
        let mut combat = Combat::enter(cards_entry(&cards)).unwrap();
        play(&mut combat, CardId::Defend);
        play(&mut combat, CardId::Strike);
        play(&mut combat, CardId::BurningPact);
        let view = combat.public_full().unwrap();
        assert!(view.selection.is_none());
        assert_eq!((view.hand.len(), view.discard.len(), view.exhaust.len(), view.player.energy),
            (2, 1, usize::from(singleton), 0));
    }
}

#[test]
fn burning_pact_upgrade_requests_one_more_real_draw_from_the_same_recycled_supply() {
    for (upgrades, hand, remaining_draw) in [(0, 4, 1), (1, 5, 0)] {
        let mut entry = cards_entry(&[CardId::Strike, CardId::Defend, CardId::Strike,
            CardId::Defend, CardId::BurningPact, CardId::Bloodletting, CardId::PommelStrike]);
        entry.cards[4].upgrades = upgrades;
        let mut combat = Combat::enter(entry).unwrap();
        for action in [CombatAction::PlayCard { instance: 6, target: None },
            CombatAction::PlayCard { instance: 2, target: None },
            CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) },
            CombatAction::PlayCard { instance: 5, target: None },
            CombatAction::SelectCard { instance: 4 }] {
            assert!(combat.legal_actions_full().unwrap().contains(&action));
            combat.step_full(&action).unwrap();
        }
        let view = combat.public_full().unwrap();
        assert_eq!((view.hand.len(), view.draw_count, view.discard.len(), view.exhaust.len(), view.player.energy),
            (hand, remaining_draw, 1, 1, 2), "recycle moves the whole discard to draw before drawing2/3");
    }
}

#[test]
fn finite_stat_artifact_and_energy_potions_drive_real_consumers_with_bark() {
    for (bark, dexterity, artifact, energy) in [(false, 2, 1, 5), (true, 4, 2, 7)] {
        let relics = if bark { vec![RelicId::SacredBark] } else { vec![] };
        let mut combat = with_items(fixture(EnemyId::JawWorm, 7), relics,
            vec![Some(PotionId::DexterityPotion), Some(PotionId::AncientPotion), Some(PotionId::EnergyPotion)]);
        use_potion(&mut combat, 0);
        use_potion(&mut combat, 1);
        use_potion(&mut combat, 2);
        let view = combat.public_full().unwrap();
        assert_eq!((power(&view, "dexterity"), power(&view, "artifact")), (dexterity, artifact));
        assert_eq!((view.player.energy, view.player.max_energy), (energy, 3));
        play(&mut combat, CardId::Defend);
        assert_eq!(combat.public_full().unwrap().player.block, 5 + dexterity);
    }
    let mut entry = fixture(EnemyId::JawWorm, 7);
    entry.setup.player.max_energy = 998;
    let mut combat = entered(entry, vec![Some(PotionId::EnergyPotion)]);
    let before = combat.snapshot();
    let action = CombatAction::UsePotion { slot: 0, target: None };
    assert!(!combat.legal_actions_full().unwrap().contains(&action));
    assert_eq!(combat.step_full(&action), Err(RunError::InvalidAction));
    assert_eq!(combat.snapshot(), before);
}

#[test]
fn real_spike_frail_and_louse_weak_respect_card_scaling_artifact_and_duration() {
    for (dexterity, artifact, block) in [(false, false, 3), (true, false, 5), (false, true, 5)] {
        let mut entry = fixture(EnemyId::JawWorm, 7);
        let enemy = &mut entry.setup.enemies[0];
        enemy.enemy_id = EnemyId::SpikeSlimeM;
        enemy.hp = 30; enemy.max_hp = 30; enemy.intent = FullIntentId::SpikeLick;
        let potions = vec![if dexterity { Some(PotionId::DexterityPotion) } else { None },
            if artifact { Some(PotionId::AncientPotion) } else { None }];
        let mut combat = entered(entry, potions);
        if dexterity { use_potion(&mut combat, 0); }
        if artifact { use_potion(&mut combat, 1); }
        combat.step_full(&CombatAction::EndTurn).unwrap();
        let view = combat.public_full().unwrap();
        assert_eq!(power(&view, "frail"), i32::from(!artifact));
        assert_eq!(power(&view, "artifact"), 0);
        play(&mut combat, CardId::Defend);
        assert_eq!(combat.public_full().unwrap().player.block, block);
        assert_eq!(view.enemies[0].intent, FullIntentId::SpikeFlameTackle, "expiry fixture must not reapply Frail");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(power(&combat.public_full().unwrap(), "frail"), 0);
    }
    let mut entry = fixture(EnemyId::LouseGreen, 7);
    entry.setup.player_powers.push(FullPowerEntry { id: FullPowerId("weak"),
        state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } });
    let mut combat = entered(entry, vec![Some(PotionId::AncientPotion)]);
    use_potion(&mut combat, 0);
    assert_eq!(power(&combat.public_full().unwrap(), "weak"), 2, "Artifact is not cleansing");
    for _ in 0..3 {
        if combat.public_full().unwrap().enemies[0].intent == FullIntentId::SpitWeb { break; }
        combat.step_full(&CombatAction::EndTurn).unwrap();
    }
    assert_eq!(combat.public_full().unwrap().enemies[0].intent, FullIntentId::SpitWeb);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!((power(&view, "artifact"), power(&view, "weak")), (0, 0));
}

#[test]
fn ordinary_draw_caps_and_short_supply_do_not_create_or_duplicate_cards() {
    for (count, opening, expected) in [(10, 9, 10), (3, 3, 3)] {
        let mut entry = cards_entry(&vec![CardId::Strike; count]);
        entry.opening_draw = opening;
        let mut combat = entered(entry, vec![Some(PotionId::SwiftPotion)]);
        use_potion(&mut combat, 0);
        let view = combat.public_full().unwrap();
        assert_eq!((view.hand.len(), view.draw_count), (expected, 0));
        let ids = view.hand.iter().map(|c| c.instance).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), expected);
    }
}

#[test]
fn caller_budget_errors_never_advance_extra_actions_or_fabricate_death() {
    let mut combat = Combat::enter(fixture(EnemyId::JawWorm, 7)).unwrap();
    let initial = combat.snapshot();
    assert!(matches!(support::Script::new(0).step(&mut combat, CombatAction::EndTurn),
        Err(support::DemoError::BudgetExceeded { accepted: 0 })));
    assert_eq!(combat.snapshot(), initial);
    let mut script = support::Script::new(1);
    script.step(&mut combat, CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, 69);
    let after = combat.snapshot();
    assert!(matches!(script.step(&mut combat, CombatAction::EndTurn),
        Err(support::DemoError::BudgetExceeded { accepted: 1 })));
    assert_eq!(combat.snapshot(), after);
    assert_eq!(combat.public_full().unwrap().outcome, None);
}
