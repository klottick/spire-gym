#[path = "../examples/m12_support/mod.rs"]
#[allow(dead_code)]
mod m12_support;

use m12_support::{Caller, CallerOutcome, Scenario};
use spire_core::{
    combat::Outcome,
    content::{CardId, Character, RelicId},
    run::{
        CommittedEvent, GameAction, RoomChoiceKind, RoomChoiceStatus, RoomViewKind, Run,
        RunTerminal, RoomScenario,
    },
};

const SEEDS: [u64; 2] = [7, 11];
const BOSS_SCENARIOS: [Scenario; 3] = [Scenario::Guardian, Scenario::Hexaghost, Scenario::SlimeBoss];

fn assert_canonical_normal_start(run: &Run) {
    let decision = run.decision().expect("normal short-Neow decision is public");
    assert_eq!(decision.content_profile, Some(RoomScenario::M12SilentAct1));
    assert_eq!((decision.act, decision.floor), (1, 0));
    assert_eq!(
        (
            decision.player.character,
            decision.player.hp,
            decision.player.max_hp,
            decision.player.gold,
        ),
        (Character::Silent, 70, 70, 99),
    );
    assert_eq!(decision.player.relics, [RelicId::RingOfTheSnake]);
    assert_eq!(decision.player.potions, [None; 3]);
    assert_eq!(decision.player.keys, [false; 3]);
    assert_eq!(decision.objects.len(), 12);
    assert_eq!(decision.objects.iter().filter(|card| card.id == CardId::StrikeGreen).count(), 5);
    assert_eq!(decision.objects.iter().filter(|card| card.id == CardId::DefendGreen).count(), 5);
    assert_eq!(decision.objects.iter().filter(|card| card.id == CardId::Survivor).count(), 1);
    assert_eq!(decision.objects.iter().filter(|card| card.id == CardId::Neutralize).count(), 1);
    assert_eq!(decision.room.as_ref().map(|room| room.kind), Some(RoomViewKind::Neow));
    assert!(run.public_map().is_none(), "the generated map must follow an actual Neow action");
}

#[test]
fn m12_normal_start_has_canonical_resources_and_real_neow() {
    for scenario in BOSS_SCENARIOS {
        let run = m12_support::make_run(7, scenario).expect("supported Silent A0 scenario");
        assert_canonical_normal_start(&run);
        assert_eq!(
            m12_support::act1_seen_bosses(scenario),
            match scenario {
                Scenario::Guardian => vec!["Hexaghost", "SlimeBoss"],
                Scenario::Hexaghost => vec!["Guardian", "SlimeBoss"],
                Scenario::SlimeBoss => vec!["Guardian", "Hexaghost"],
                Scenario::Death => unreachable!(),
            },
        );
    }
}

#[test]
fn m12_initial_boss_history_drives_generated_map_and_actual_encounter() {
    for seed in SEEDS {
        for scenario in BOSS_SCENARIOS {
            let boss = scenario.encounter().expect("normal scenario has an Act 1 boss");
            let mut run = m12_support::make_run(seed, scenario).unwrap();
            assert_canonical_normal_start(&run);
            let initial = run.decision().unwrap();
            assert!(initial.room.as_ref().unwrap().choices.iter().any(|choice| {
                choice.kind == RoomChoiceKind::NeowLament && choice.status == RoomChoiceStatus::Offered
            }));

            let mut caller = Caller::new(4096).unwrap();
            let outcome = m12_support::rollout(&mut run, &mut caller, scenario).unwrap();
            assert!(matches!(outcome, CallerOutcome::Act2Ready | CallerOutcome::Death));
            assert!(caller.accepted() > 0);
            assert_eq!(caller.accepted(), caller.trace.len());

            let first = caller.trace.first().unwrap();
            assert_eq!(first.decision, initial);
            assert!(first.map.is_none());
            assert!(matches!(first.action, GameAction::Choose(_)));
            assert!(first.decision.room.as_ref().unwrap().choices.iter().any(|choice| {
                choice.kind == RoomChoiceKind::NeowLament && first.action == GameAction::Choose(choice.id)
            }));

            let generated_map = caller.trace.iter().find_map(|step| {
                (step.decision.room.as_ref().map(|room| room.kind) == Some(RoomViewKind::Map))
                    .then(|| step.map.as_ref())
                    .flatten()
            }).expect("map actions use the public generated Act 1 map");
            assert_eq!(generated_map.boss, Some(boss));

            let boss_entry = caller.trace.iter().any(|step| {
                let (GameAction::Move(id), Some(map)) = (&step.action, &step.map) else { return false };
                let is_boss_node = map.nodes.iter().any(|node| node.id == *id && node.kind == spire_core::run::MapNodeKind::Boss);
                is_boss_node && step.transition.committed_events.iter().any(|event| {
                    matches!(event, CommittedEvent::CombatEntered { encounter, .. } if *encounter == boss)
                })
            });
            assert!(boss_entry, "seed {seed} scenario {scenario:?} did not enter actual {boss:?}");

            let final_decision = caller.final_decision.as_ref().expect("normal run has a public final decision");
            assert!(final_decision.last_combat.as_ref().is_some_and(|record| record.encounter == boss));
            match outcome {
                CallerOutcome::Act2Ready => {
                    assert_eq!(final_decision.room.as_ref().map(|room| room.kind), Some(RoomViewKind::Act2Ready));
                    assert_eq!(final_decision.terminal, None);
                    assert!(final_decision.candidates.is_empty());
                    assert_eq!(final_decision.last_combat.as_ref().unwrap().outcome, Outcome::Victory);
                }
                CallerOutcome::Death => {
                    assert_eq!(final_decision.terminal, Some(RunTerminal::Death));
                }
                CallerOutcome::BudgetExhausted => unreachable!("a required boss run exceeded its fixed budget"),
            }

            let summary = m12_support::summarize(&caller);
            assert!(summary.ordinary_moves > 0, "the run used ordinary map rooms");
            assert!(summary.elite_moves > 0, "the public route policy should select an actual elite");
            assert!(summary.boss_moves > 0, "the actual route reached its boss node");
            assert!(summary.reward_screens > 0, "a real victory should expose a captured reward screen");
            if outcome == CallerOutcome::Act2Ready {
                assert!(summary.chests_opened > 0);
                assert!(summary.boss_rewards > 0);
            }

            replay_transcript(seed, scenario, &caller);
        }
    }
}

#[test]
fn m12_nonattacking_caller_records_real_death_and_replays_it() {
    for seed in SEEDS {
        let scenario = Scenario::Death;
        let mut run = m12_support::make_run(seed, scenario).unwrap();
        assert_canonical_normal_start(&run);
        let mut caller = Caller::new(4096).unwrap();
        assert_eq!(m12_support::rollout(&mut run, &mut caller, scenario).unwrap(), CallerOutcome::Death);
        let final_decision = caller.final_decision.as_ref().unwrap();
        assert_eq!(final_decision.terminal, Some(RunTerminal::Death));
        assert!(caller.trace.iter().any(|step| step.action == GameAction::Combat(spire_core::combat::CombatAction::EndTurn)));
        assert!(caller.trace.iter().any(|step| step.transition.terminal == Some(RunTerminal::Death)));
        replay_transcript(seed, scenario, &caller);
    }
}

#[test]
fn m12_small_budgets_and_invalid_actions_remain_distinct() {
    let mut zero_run = m12_support::make_run(7, Scenario::Guardian).unwrap();
    let mut zero = Caller::new(0).unwrap();
    assert_eq!(m12_support::rollout(&mut zero_run, &mut zero, Scenario::Guardian).unwrap(), CallerOutcome::BudgetExhausted);
    assert_eq!(zero.accepted(), 0);
    assert_eq!(zero_run.decision().unwrap().terminal, None);
    assert!(zero_run.public_map().is_none());

    let mut one_run = m12_support::make_run(7, Scenario::Guardian).unwrap();
    let mut one = Caller::new(1).unwrap();
    assert_eq!(m12_support::rollout(&mut one_run, &mut one, Scenario::Guardian).unwrap(), CallerOutcome::BudgetExhausted);
    assert_eq!(one.accepted(), 1);
    assert_eq!(one.trace.len(), 1);
    assert!(one.trace[0].map.is_none());
    assert!(one_run.public_map().is_some(), "the one accepted action was the real Neow choice");
    assert_eq!(one_run.decision().unwrap().terminal, None);

    assert!(Caller::new(4096).is_ok());
    assert!(Caller::new(4097).is_err());
    assert!(m12_support::parse_args(["--seed".into(), "13".into()]).is_err());
    assert!(m12_support::parse_args(["--scenario".into(), "unknown".into()]).is_err());
    assert!(m12_support::parse_args(["--budget".into(), "4097".into()]).is_err());
    assert!(m12_support::make_run(13, Scenario::Guardian).is_err());
    assert_eq!(m12_support::parse_args(["--budget".into(), "0".into()]).unwrap().limit, 0);

    let mut invalid_run = m12_support::make_run(7, Scenario::Guardian).unwrap();
    let before = invalid_run.clone();
    assert!(invalid_run.step(&GameAction::Confirm).is_err());
    assert_eq!(invalid_run, before, "an invalid action neither advances Neow nor creates the map");
}

fn replay_transcript(seed: u64, scenario: Scenario, caller: &Caller) {
    let mut replay = m12_support::make_run(seed, scenario).unwrap();
    for (index, step) in caller.trace.iter().enumerate() {
        assert_eq!(replay.decision().unwrap(), step.decision, "public decision at action {index}");
        assert_eq!(replay.public_map(), step.map, "public map at action {index}");
        let transition = replay.step(&step.action).unwrap_or_else(|error| {
            panic!("recorded action {index} failed on replay: {error:?}")
        });
        assert_eq!(transition, step.transition, "transition at action {index}");
    }
    assert_eq!(replay.decision().unwrap(), caller.final_decision.clone().unwrap());
    assert_eq!(replay.public_map(), caller.final_map);
}
