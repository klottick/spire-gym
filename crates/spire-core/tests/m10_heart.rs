#[path = "../examples/m10_support/mod.rs"]
mod support;

use spire_core::{
    content::{CardId, EnemyId, RelicId},
    run::{GameAction, RoomScenario, RoomViewKind, RunTerminal},
};

fn histories() -> [support::M9BossHistory; 3] {
    [
        support::M9BossHistory::AwakenedUnseen,
        support::M9BossHistory::TimeEaterUnseen,
        support::M9BossHistory::DonuDecaUnseen,
    ]
}

fn replay(seed: u64, history: support::M9BossHistory, report: &support::M10Report) {
    let mut run = support::start(seed, history).expect("same public M10 profile and root");
    for (index, step) in report.trace.iter().enumerate() {
        let before = run.decision().expect("public decision before accepted step");
        assert!(
            before.candidates.iter().any(|candidate| candidate.action == step.action),
            "trace action {index} was not currently offered"
        );
        let transition = run.step(&step.action).expect("accepted semantic action replays");
        let after = run.decision().expect("public decision after accepted step");
        assert_eq!(transition.reward, step.reward, "reward at public boundary {index}");
        assert_eq!(transition.terminal, after.terminal, "terminal at boundary {index}");
        assert_eq!(after, step.decision, "public decision at boundary {index}");
        assert_eq!(run.public_map(), step.map, "public map at boundary {index}");
    }
    assert_eq!(run.decision().unwrap(), report.final_decision);
    assert_eq!(run.public_map(), report.final_map);
}

#[test]
fn m10_normal_start_keeps_standard_resources_and_real_boss_history_context() {
    for history in histories() {
        let decision = support::start(11, history).unwrap().decision().unwrap();
        assert_eq!(decision.content_profile, Some(RoomScenario::M10IroncladHeart));
        assert_eq!(
            (
                decision.act,
                decision.floor,
                decision.player.hp,
                decision.player.max_hp,
                decision.player.gold,
            ),
            (1, 0, 80, 80, 99)
        );
        assert_eq!(decision.player.relics, vec![RelicId::BurningBlood]);
        assert_eq!(decision.player.keys, [false; 3]);
        assert_eq!(decision.player.potions, vec![None; 3]);
        assert_eq!(decision.objects.len(), 10);
        assert_eq!(
            decision.objects.iter().filter(|card| card.id == CardId::Strike && card.upgrades == 0).count(),
            5
        );
        assert_eq!(
            decision.objects.iter().filter(|card| card.id == CardId::Defend && card.upgrades == 0).count(),
            4
        );
        assert_eq!(
            decision.objects.iter().filter(|card| card.id == CardId::Bash && card.upgrades == 0).count(),
            1
        );
        assert_eq!((decision.reward_total, decision.terminal), (0, None));
        assert!(decision.combat.is_none());
        assert!(!decision.candidates.is_empty());
    }
}

#[test]
fn m10_seed7_and_seed11_histories_are_real_outcomes_with_complete_public_replay() {
    for seed in [11, 7] {
        for history in histories() {
            let report = support::rollout(seed, history, 8192).unwrap();
            let decision = &report.final_decision;
            assert_ne!(report.completion, support::M10Completion::Unfinished);
            assert!(decision.candidates.is_empty());
            match report.completion {
                support::M10Completion::Death => {
                    assert_eq!(decision.terminal, Some(RunTerminal::Death));
                    assert_eq!(decision.player.hp, 0);
                    assert_eq!(report.trace.last().unwrap().reward, -1);
                    assert_eq!(report.trace.last().unwrap().decision.terminal, Some(RunTerminal::Death));
                }
                support::M10Completion::EmptyDeck => {
                    assert_eq!(decision.terminal, Some(RunTerminal::EmptyDeck));
                    assert!(decision.objects.is_empty(), "EmptyDeck is classified from the settled permanent-deck census");
                    assert_eq!(report.trace.last().unwrap().reward, -1, "EmptyDeck settlement is a loss without implying HP loss");
                }
                support::M10Completion::Act3NoKeys => {
                    assert_eq!(decision.terminal, Some(RunTerminal::Act3NoKeys));
                    assert_eq!((decision.act, decision.floor), (3, 50));
                    assert!(!report.trace.iter().any(|step| step.decision.act == 4));
                }
                support::M10Completion::HeartVictory => {
                    assert_eq!(decision.terminal, Some(RunTerminal::HeartVictory));
                    assert!(report.trace.iter().any(|step| {
                        step.decision.act == 4
                            && step.decision.floor == 54
                            && step.decision.combat.as_ref().is_some_and(|combat| {
                                combat.enemies.iter().any(|enemy| enemy.enemy_id == EnemyId::CorruptHeart)
                            })
                    }));
                    assert_eq!(decision.room.as_ref().map(|room| room.kind), Some(RoomViewKind::TrueVictory));
                }
                support::M10Completion::Unfinished | support::M10Completion::Act4Ready => {
                    panic!("8192 accepted public actions must not be reported as a completed episode")
                }
            }
            println!(
                "M10 normal start seed{seed} {history:?}: {:?}, actions{}, act{} floor{} HP{}/{} reward{}",
                report.completion,
                report.trace.len(),
                decision.act,
                decision.floor,
                decision.player.hp,
                decision.player.max_hp,
                decision.reward_total
            );
            replay(seed, history, &report);
        }
    }
}

#[test]
fn m10_nonattacking_caller_records_a_real_death_not_a_budget_result() {
    let report = support::nonattacking(11, support::M9BossHistory::AwakenedUnseen, 8192).unwrap();
    assert_eq!(report.completion, support::M10Completion::Death);
    assert_eq!(report.final_decision.terminal, Some(RunTerminal::Death));
    assert_eq!(report.final_decision.player.hp, 0);
    assert!(report.trace.iter().filter_map(|step| match &step.action {
        GameAction::Combat(action) => Some(action),
        _ => None,
    }).all(|action| *action == spire_core::combat::CombatAction::EndTurn));
    replay(11, support::M9BossHistory::AwakenedUnseen, &report);
}

#[test]
fn m10_small_budgets_are_unfinished_and_invalid_budgets_or_arguments_are_errors() {
    for budget in [0, 1] {
        let report = support::rollout(11, support::M9BossHistory::TimeEaterUnseen, budget).unwrap();
        assert_eq!(report.completion, support::M10Completion::Unfinished);
        assert_eq!(report.trace.len(), budget);
        assert_eq!(report.final_decision.terminal, None);
    }
    assert!(support::rollout(11, support::M9BossHistory::TimeEaterUnseen, 8193).is_err());
    for args in [
        vec!["--seed".to_owned(), "-1".to_owned()],
        vec!["--budget".to_owned(), "8193".to_owned()],
        vec!["--scenario".to_owned(), "victory".to_owned()],
        vec!["--unknown".to_owned(), "1".to_owned()],
        vec!["--seed".to_owned()],
    ] {
        assert!(support::parse_args(args).is_err());
    }
}
