use spire_core::{combat::*, content::{CardId, EnemyId}, run::RunError};
#[path = "../examples/m1_support/mod.rs"]
mod support;
use support::*;

fn targeted(instance: u64) -> CombatAction {
    CombatAction::PlayCard { instance, target: Some(TargetId(1)) }
}

fn play(combat: &mut Combat, id: CardId) -> Result<(), RunError> {
    combat.step_full(&card_action(combat, id)?)
}

fn end_turn(combat: &mut Combat) -> Result<(), RunError> {
    assert!(combat.legal_actions_full()?.contains(&CombatAction::EndTurn));
    combat.step_full(&CombatAction::EndTurn)
}

#[test]
fn m1_bash_uses_shared_damage_modifiers_and_pays_once() {
    let mut entry = fixture(EnemyId::JawWorm, 7);
    entry.setup.player_powers.push(FullPowerEntry { id: FullPowerId("weak"),
        state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } });
    let mut combat = Combat::enter(entry).unwrap();
    combat.step_full(&targeted(3)).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.enemies[0].hp, 36, "base Bash8 × Weak0.75 =6 damage");
    assert_eq!(view.player.energy, 1);
    assert_eq!(view.discard.iter().filter(|c| c.id == CardId::Bash).count(), 1);
    assert!(view.enemies[0].powers.contains(&FullPowerView { id: FullPowerId("vulnerable"), amount: 2 }));
}

#[test]
fn m1_public_menu_is_legal_and_read_only() {
    let entry = fixture(EnemyId::JawWorm, 7);
    let mut combat = Combat::enter(entry.clone()).unwrap();
    let initial = combat.snapshot();
    let menu = combat.legal_actions_full().unwrap();
    assert_eq!(menu.len(), 5);
    for action in [targeted(1), targeted(3), targeted(4),
        CombatAction::PlayCard { instance: 2, target: None }, CombatAction::EndTurn] {
        assert!(menu.contains(&action), "missing {action:?}");
    }
    assert_eq!(combat.public_full().unwrap().player.energy, 3);
    assert_eq!(combat.legal_actions_full().unwrap(), menu);
    assert_eq!(combat.snapshot(), initial, "public/menu reads must not mutate state or RNG");

    // Reconstruct entry and accepted prefix; full snapshot restoration is not required.
    for prefix in [vec![], vec![targeted(3)]] {
        let mut at_prefix = Combat::enter(entry.clone()).unwrap();
        for action in &prefix { at_prefix.step_full(action).unwrap(); }
        for action in at_prefix.legal_actions_full().unwrap() {
            let mut independent = Combat::enter(entry.clone()).unwrap();
            for accepted in &prefix { independent.step_full(accepted).unwrap(); }
            independent.step_full(&action).unwrap();
        }
    }
    play(&mut combat, CardId::Bash).unwrap();
    let after_bash = combat.snapshot();
    let menu = combat.legal_actions_full().unwrap();
    assert_eq!(menu.len(), 3);
    assert!(!menu.contains(&targeted(4)), "unaffordable Clothesline is not legal");
    for action in [targeted(3), targeted(4),
        CombatAction::PlayCard { instance: 1, target: Some(TargetId(999)) },
        CombatAction::PlayCard { instance: 1, target: None },
        CombatAction::PlayCard { instance: 2, target: Some(TargetId(1)) }] {
        assert_eq!(combat.step_full(&action), Err(RunError::InvalidAction));
        assert_eq!(combat.snapshot(), after_bash, "rejected {action:?} changed state");
    }

    let mut two = entry.clone();
    let mut second = two.setup.enemies[0].clone();
    second.instance_id = EnemyHandle::parse("second").unwrap();
    second.hp = 6; // Supplied injured actor; its ordinary Strike kill must not end the fight.
    second.ai_rng = rng::make_stream(&[9; 32], rng::Partition::Test, "m1",
        &rng::StreamId::new(rng::Domain::RunCombatEnemyAi,
            vec!["m1-authored-combat".into(), "second".into()]).unwrap()).unwrap();
    two.setup.enemies.push(second);
    let mut combat = Combat::enter(two).unwrap();
    let second_target = CombatAction::PlayCard { instance: 1, target: Some(TargetId(2)) };
    assert!(combat.legal_actions_full().unwrap().contains(&second_target));
    combat.step_full(&second_target).unwrap();
    assert_eq!(combat.public_full().unwrap().outcome, None);
    assert!(combat.legal_actions_full().unwrap().contains(&targeted(3)));
    assert!(!combat.legal_actions_full().unwrap().iter().any(|action|
        matches!(action, CombatAction::PlayCard { target: Some(TargetId(2)), .. })));
    let before = combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance: 3, target: Some(TargetId(2)) }),
        Err(RunError::InvalidAction));
    assert_eq!(combat.snapshot(), before);

    // M2 intentionally extends the original M1 menu to real selection screens.
    let mut selecting = entry.clone();
    selecting.cards[1].id = CardId::Survivor;
    let mut selecting = Combat::enter(selecting).unwrap();
    play(&mut selecting, CardId::Survivor).unwrap();
    assert!(selecting.public_full().unwrap().selection.is_some());
    let before = selecting.snapshot();
    let menu = selecting.legal_actions_full().unwrap();
    assert!(!menu.is_empty());
    assert!(menu.iter().all(|action| matches!(action, CombatAction::SelectCard { .. })));
    assert_eq!(selecting.snapshot(), before);

    let mut unsupported = entry;
    unsupported.cards[0].id = CardId::Unknown("m1-unimplemented");
    let mut unsupported = Combat::enter(unsupported).unwrap();
    let before = unsupported.snapshot();
    assert!(matches!(unsupported.legal_actions_full(), Err(RunError::Unsupported(_))));
    assert_eq!(unsupported.snapshot(), before, "menu errors are also read-only");
    assert!(matches!(unsupported.step_full(&targeted(1)), Err(RunError::Faulted(_))));
    let faulted = unsupported.snapshot();
    assert!(matches!(unsupported.legal_actions_full(), Err(RunError::Faulted(_))));
    assert_eq!(unsupported.snapshot(), faulted, "fault menus must retain the original fault");
}

fn power(powers: &[FullPowerView], id: &str) -> i32 {
    powers.iter().find(|p| p.id.0 == id).map_or(0, |p| p.amount)
}

// Follow actual AI from its valid opener; never inject impossible move history.
fn advance_to(combat: &mut Combat, intent: FullIntentId) {
    for _ in 0..4 {
        if combat.public_full().unwrap().enemies[0].intent == intent { return; }
        end_turn(combat).unwrap();
    }
    panic!("fixture did not reach {intent:?} within four real rounds");
}

#[test]
fn m1_damage_block_and_debuffs_work_on_both_sides() {
    let mut bash = Combat::enter(fixture(EnemyId::JawWorm, 7)).unwrap();
    play(&mut bash, CardId::Bash).unwrap();
    assert_eq!(bash.public_full().unwrap().enemies[0].hp, 34);
    play(&mut bash, CardId::Strike).unwrap();
    let view = bash.public_full().unwrap();
    assert_eq!(view.enemies[0].hp, 25, "Strike6 × Vulnerable1.5 =9");
    assert_eq!(view.player.energy, 0, "two plays pay2+1 exactly once");

    // Same normal Chomp, with and without player-produced Weak; Block is separate.
    for (weak, raw, hp_after) in [(false, 11, 74), (true, 8, 77)] {
        let mut combat = Combat::enter(fixture(EnemyId::JawWorm, 7)).unwrap();
        if weak { play(&mut combat, CardId::Clothesline).unwrap(); }
        play(&mut combat, CardId::Defend).unwrap();
        let view = combat.public_full().unwrap();
        assert_eq!(view.enemies[0].intent_damage, Some(raw));
        assert_eq!(view.player.block, 5);
        end_turn(&mut combat).unwrap();
        assert_eq!(combat.public_full().unwrap().player.hp, hp_after);
    }

    let mut louse = Combat::enter(fixture(EnemyId::LouseGreen, 7)).unwrap();
    advance_to(&mut louse, FullIntentId::SpitWeb);
    end_turn(&mut louse).unwrap();
    play(&mut louse, CardId::Strike).unwrap();
    let view = louse.public_full().unwrap();
    assert_eq!(view.enemies[0].hp, 11, "Weak Strike is4");
    assert_eq!(view.enemies[0].block, 5, "real CurlUp applies after the hit");
    play(&mut louse, CardId::Bash).unwrap();
    let view = louse.public_full().unwrap();
    assert_eq!(view.enemies[0].hp, 10, "Weak Bash raw6 absorbs5 Block then loses1HP");
    assert_eq!(view.enemies[0].block, 0);
    assert_eq!(view.player.energy, 0);

    // Seed8 follows Bellow → SkullBash → BullRush → BullRush, so no
    // reapplication or Skill-triggered Strength can obscure the lifetime.
    let mut nob = Combat::enter(fixture(EnemyId::GremlinNob, 8)).unwrap();
    advance_to(&mut nob, FullIntentId::GremlinNobSkullBash);
    end_turn(&mut nob).unwrap();
    let view = nob.public_full().unwrap();
    assert_eq!(power(&view.enemies[0].powers, "strength"), 0, "no Skills were played into Enrage");
    assert_eq!(view.enemies[0].intent, FullIntentId::GremlinNobBullRush);
    assert_eq!(view.enemies[0].intent_damage, Some(21));
    end_turn(&mut nob).unwrap();
    assert_eq!(nob.public_full().unwrap().player.hp, view.player.hp - 21);
}

#[test]
fn m1_debuff_lifetimes_follow_application_source() {
    for (card, id) in [(CardId::Clothesline, "weak"), (CardId::Bash, "vulnerable")] {
        let mut combat = Combat::enter(fixture(EnemyId::JawWorm, 7)).unwrap();
        play(&mut combat, card).unwrap();
        assert_eq!(power(&combat.public_full().unwrap().enemies[0].powers, id), 2);
        for remaining in [1, 0] {
            end_turn(&mut combat).unwrap();
            assert_eq!(power(&combat.public_full().unwrap().enemies[0].powers, id), remaining);
        }
    }
    for (enemy, seed, intent, id) in [
        (EnemyId::LouseGreen, 7, FullIntentId::SpitWeb, "weak"),
        (EnemyId::GremlinNob, 8, FullIntentId::GremlinNobSkullBash, "vulnerable"),
    ] {
        let mut combat = Combat::enter(fixture(enemy, seed)).unwrap();
        advance_to(&mut combat, intent);
        end_turn(&mut combat).unwrap();
        assert_eq!(power(&combat.public_full().unwrap().player.powers, id), 2,
            "enemy-applied {id} must survive its application round");
        for remaining in [1, 0] {
            assert_ne!(combat.public_full().unwrap().enemies[0].intent, intent,
                "fixture must not reapply while checking expiry");
            end_turn(&mut combat).unwrap();
            assert_eq!(power(&combat.public_full().unwrap().player.powers, id), remaining);
        }
    }
}

#[test]
fn m1_victory_death_and_replay_are_real() {
    for (attack, hp, outcome) in [(true, 80, Outcome::Victory), (false, 11, Outcome::Death)] {
        let mut entry = fixture(EnemyId::JawWorm, 7);
        entry.setup.player.hp = hp;
        let mut combat = Combat::enter(entry.clone()).unwrap();
        let mut history = Vec::new();
        let actual = run_fight(&mut combat, attack, 64, |view, menu, action| {
            if let Some(action) = action { assert!(menu.contains(action)); }
            history.push((view.clone(), action.cloned()));
        }).unwrap();
        assert_eq!(actual, outcome);
        assert!(!history.is_empty());
        let final_view = combat.public_full().unwrap();
        assert_eq!(final_view.outcome, Some(outcome));
        assert!(combat.legal_actions_full().unwrap().is_empty());
        if attack { assert_eq!(final_view.enemies[0].hp, 0); }
        else { assert_eq!(final_view.player.hp, 0); }
        let final_snapshot = combat.snapshot();
        for action in [CombatAction::EndTurn, targeted(1)] {
            assert_eq!(combat.step_full(&action), Err(RunError::InvalidAction));
            assert_eq!(combat.snapshot(), final_snapshot);
        }
        let mut replay = Combat::enter(entry).unwrap();
        for (checkpoint, action) in history {
            assert_eq!(replay.public_full().unwrap(), checkpoint);
            if let Some(action) = action { replay.step_full(&action).unwrap(); }
        }
    }

    let mut unfinished = Combat::enter(fixture(EnemyId::JawWorm, 7)).unwrap();
    let initial = unfinished.snapshot();
    assert!(matches!(run_fight(&mut unfinished, true, 0, |_, _, _| {}),
        Err(DemoError::BudgetExceeded { accepted: 0 })));
    assert_eq!(unfinished.public_full().unwrap().outcome, None);
    assert_eq!(unfinished.snapshot(), initial, "zero budget never steps or fabricates death");
    assert!(matches!(run_fight(&mut unfinished, false, 1, |_, _, _| {}),
        Err(DemoError::BudgetExceeded { accepted: 1 })));
    assert_eq!(unfinished.public_full().unwrap().player.hp, 69);
    assert_eq!(unfinished.public_full().unwrap().outcome, None);
}
