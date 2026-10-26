#[path = "../examples/m14_support/mod.rs"]
#[allow(dead_code)]
mod m14_support;

use m14_support::{Caller, CallerOutcome, Scenario};
use spire_core::{
    combat::Outcome,
    content::{CardId, Character, EncounterId, RelicId},
    run::{
        CommittedEvent, GameAction, MapNodeKind, PublicDecision, PublicPlayer, RoomScenario,
        RoomView, RoomViewKind, RunTerminal,
    },
};

const FIXED_CASES: [(u64, Scenario); 6] = [
    (7, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::Champ, act3_boss: EncounterId::AwakenedOne }),
    (7, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::TimeEater }),
    (7, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Collector, act3_boss: EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::Collector, act3_boss: EncounterId::AwakenedOne }),
    (11, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Champ, act3_boss: EncounterId::TimeEater }),
];

#[test]
fn m14_frozen_v5_declared_suite_enters_real_generated_bosses() {
    assert_eq!(m14_support::FIXED_CASES, FIXED_CASES, "retain the six approved triples exactly");

    let mut route_errors = Vec::new();
    let mut operational_errors = Vec::new();
    let mut act2_coverage = Vec::new();
    let mut act3_boss_coverage = Vec::new();
    let mut act3_ordinary_coverage = Vec::new();
    let mut act3_elite_coverage = Vec::new();
    let mut no_key_completions = 0;
    let mut outcomes = Vec::new();

    let cases = m14_support::evidence_cases();
    assert_eq!(cases.len(), 42);
    assert_eq!(&cases[..6], &FIXED_CASES);
    let mut attempted_cases = 0;
    for &(seed, scenario) in cases {
        attempted_cases += 1;
        let mut caller = Caller::new_m14(8192).unwrap();
        let mut run = match m14_support::make_run(seed, scenario) {
            Ok(run) => run,
            Err(error) => {
                let attempt = Err(format!("M14 run creation: {error}"));
                if let Err(export_error) = m14_support::persist_attempt_trace(seed, scenario, &attempt, &caller, None) {
                    operational_errors.push(format!("seed={seed} scenario={scenario:?} trace export: {export_error}"));
                }
                operational_errors.push(format!("seed={seed} scenario={scenario:?}: {}", attempt.as_ref().unwrap_err()));
                outcomes.push(format!("seed={seed} scenario={scenario:?} result={attempt:?} actions=0"));
                eprintln!("M14 seed={seed} scenario={scenario:?} result={attempt:?} accepted_actions=0 endpoint=None");
                continue;
            }
        };

        if let Ok(initial) = run.decision() {
            require_eq(&mut route_errors, format!("seed={seed} initial content profile"), initial.content_profile, Some(RoomScenario::M14SilentAct3));
            require_eq(&mut route_errors, format!("seed={seed} initial act/floor"), (initial.act, initial.floor), (1, 0));
            require_eq(&mut route_errors, format!("seed={seed} initial player"),
                (initial.player.character, initial.player.hp, initial.player.max_hp, initial.player.gold),
                (Character::Silent, 70, 70, 99));
            require_eq(&mut route_errors, format!("seed={seed} initial relics"), initial.player.relics.clone(), vec![RelicId::RingOfTheSnake]);
            require_eq(&mut route_errors, format!("seed={seed} initial potions"), initial.player.potions.clone(), [None; 3].to_vec());
            require_eq(&mut route_errors, format!("seed={seed} initial keys"), initial.player.keys, [false; 3]);
            require_eq(&mut route_errors, format!("seed={seed} initial object count"), initial.objects.len(), 12);
            for (id, count) in [
                (CardId::StrikeGreen, 5),
                (CardId::DefendGreen, 5),
                (CardId::Survivor, 1),
                (CardId::Neutralize, 1),
            ] {
                require_eq(&mut route_errors, format!("seed={seed} initial {id:?} count"),
                    initial.objects.iter().filter(|card| card.id == id).count(), count);
            }
            require_eq(&mut route_errors, format!("seed={seed} initial room"), initial.room.as_ref().map(|room| room.kind), Some(RoomViewKind::Neow));
            require_eq(&mut route_errors, format!("seed={seed} initial map"), run.public_map(), None);
        } else if let Err(error) = run.decision() {
            route_errors.push(format!("seed={seed} initial decision before rollout: {error:?}"));
        }

        let attempt = m14_support::rollout_m14(&mut run, &mut caller, false);
        // Capture the result and full public history before deriving any
        // endpoint, stage, or coverage assertions from it.
        if let Err(error) = m14_support::persist_attempt_trace(seed, scenario, &attempt, &caller, Some(&run)) {
            operational_errors.push(format!("seed={seed} scenario={scenario:?} trace export: {error}"));
        }
        if let Err(error) = &attempt {
            operational_errors.push(format!("seed={seed} scenario={scenario:?}: {error}; accepted_actions={} endpoint={:?} failed_action={:?}",
                caller.accepted(),
                m14_support::endpoint_status(Some(&run), &caller),
                caller.failed_action_for_diagnostics(),
            ));
        }

        for (act, target) in [
            (1, scenario.act1_boss),
            (2, scenario.act2_boss),
            (3, scenario.act3_boss),
        ] {
            let map = caller.trace.iter()
                .find(|step| step.decision.act == act
                    && step.decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Map))
                .and_then(|step| step.map.as_ref());
            if let Some(map) = map {
                require_eq(&mut route_errors, format!("seed={seed} Act{act} observed generated map boss"), map.boss, Some(target));
            }
        }

        let mut boss_entered = [false; 4];
        let mut ordinary_move = [false; 4];
        let mut elite_move = [false; 4];
        let mut successor_observed = [false; 4];
        for step in &caller.trace {
            if let GameAction::Move(id) = &step.action {
                if let Some(node) = step.map.as_ref().and_then(|map| map.nodes.iter().find(|node| node.id == *id)) {
                    if (1..=3).contains(&step.decision.act) {
                        let act = step.decision.act as usize;
                        ordinary_move[act] |= node.kind == MapNodeKind::Monster;
                        elite_move[act] |= node.kind == MapNodeKind::Elite;
                    }
                }
            }
            for event in &step.transition.committed_events {
                let CommittedEvent::CombatEntered { visit, encounter } = event else { continue };
                let act = step.decision.act;
                let expected = target_for(scenario, act);
                if is_boss_for_act(*encounter, act) {
                    boss_entered[act as usize] = true;
                    require_eq(&mut route_errors, format!("seed={seed} Act{act} actual boss matches initial history"), *encounter, expected);
                    let recorded = has_matching_record(&caller.trace, *visit, *encounter);
                    require_true(&mut route_errors, recorded,
                        format!("seed={seed} Act{act} visit={visit} actual boss entry has same-visit CombatRecord"));
                    if recorded {
                        match act {
                            1 => (),
                            2 => remember_encounter(&mut act2_coverage, *encounter),
                            3 => remember_encounter(&mut act3_boss_coverage, *encounter),
                            _ => unreachable!("caller only evaluates Acts 1–3"),
                        }
                    }
                }
                if act == 3 && has_matching_record(&caller.trace, *visit, *encounter) {
                    if is_act3_ordinary(*encounter) { remember_encounter(&mut act3_ordinary_coverage, *encounter); }
                    if is_act3_elite(*encounter) { remember_encounter(&mut act3_elite_coverage, *encounter); }
                }
            }
        }
        for act in 1..=2 {
            successor_observed[act] = caller.trace.iter().any(|step| step.decision.act > act as u8)
                || (attempt.is_ok() && caller.final_decision.as_ref().is_some_and(|decision| decision.act > act as u8));
        }

        for act in 1..=3 {
            if !(boss_entered[act] || successor_observed[act]) {
                continue;
            }
            for kind in [MapNodeKind::Monster, MapNodeKind::Elite] {
                let moved = match kind {
                    MapNodeKind::Monster => ordinary_move[act],
                    MapNodeKind::Elite => elite_move[act],
                    _ => unreachable!(),
                };
                require_true(&mut route_errors, moved,
                    format!("seed={seed} Act{act} reached its boss/successor frontier with actual {kind:?} progression"));
            }
        }

        match &attempt {
            Ok(CallerOutcome::Act3NoKeys) => {
                no_key_completions += 1;
                if let Some(final_decision) = caller.final_decision.as_ref() {
                    require_eq(&mut route_errors, format!("seed={seed} Act3NoKeys endpoint act/floor"), (final_decision.act, final_decision.floor), (3, 50));
                    require_eq(&mut route_errors, format!("seed={seed} Act3NoKeys endpoint terminal"), final_decision.terminal, Some(RunTerminal::Act3NoKeys));
                    if let Some(record) = final_decision.last_combat.as_ref() {
                        require_eq(&mut route_errors, format!("seed={seed} Act3NoKeys last encounter"), record.encounter, scenario.act3_boss);
                        require_eq(&mut route_errors, format!("seed={seed} Act3NoKeys last combat outcome"), record.outcome, Outcome::Victory);
                    } else {
                        route_errors.push(format!("seed={seed} Act3NoKeys endpoint lacks its actual last combat record"));
                    }
                } else {
                    route_errors.push(format!("seed={seed} Act3NoKeys has no recorded endpoint"));
                }
                let stages = caller.trace.iter().filter_map(|step| {
                    if step.decision.room.as_ref().is_none_or(|room| room.kind != RoomViewKind::Act3Ending) {
                        return None;
                    }
                    let GameAction::Choose(id) = step.action else { return None };
                    step.decision.room.as_ref()?.choices.iter().find(|choice| choice.id == id)
                        .and_then(|choice| match choice.kind {
                            spire_core::run::RoomChoiceKind::Act3Continue { stage } => Some(stage),
                            _ => None,
                        })
                }).collect::<Vec<_>>();
                require_eq(&mut route_errors, format!("seed={seed} actual ending dialog stages"), stages, [1, 2, 3, 4].to_vec());
                let accepted_proceed = caller.trace.iter().any(|step| {
                    step.decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act3BossComplete)
                        && matches!(&step.action, GameAction::Choose(id) if step.decision.room.as_ref().is_some_and(|room| room.choices.iter().any(|choice| choice.id == *id && choice.kind == spire_core::run::RoomChoiceKind::Proceed)))
                });
                require_true(&mut route_errors, accepted_proceed, format!("seed={seed} actual boss settlement exposes and accepts offered Proceed"));
                let ended = caller.trace.iter().find(|step| step.transition.terminal == Some(RunTerminal::Act3NoKeys))
                    .map(|step| step.transition.reward);
                require_eq(&mut route_errors, format!("seed={seed} Act3NoKeys transition reward"), ended, Some(0));
            }
            Ok(CallerOutcome::Death) => require_true(&mut route_errors,
                caller.final_decision.as_ref().is_some_and(|decision| decision.terminal == Some(RunTerminal::Death)),
                format!("seed={seed} Death outcome retains actual Death endpoint")),
            Ok(CallerOutcome::EmptyDeck) => require_true(&mut route_errors,
                caller.final_decision.as_ref().is_some_and(|decision| decision.terminal == Some(RunTerminal::EmptyDeck)),
                format!("seed={seed} EmptyDeck outcome retains actual EmptyDeck endpoint")),
            Ok(CallerOutcome::BudgetExhausted) => {
                require_eq(&mut route_errors, format!("seed={seed} exhausted action budget"), caller.accepted(), 8192);
                require_true(&mut route_errors,
                    caller.final_decision.as_ref().is_some_and(|decision| decision.terminal.is_none()),
                    format!("seed={seed} budget endpoint remains nonterminal"));
            }
            Ok(CallerOutcome::Act3Ready) => route_errors.push(format!("seed={seed} M14 has no Act3Ready cutoff")),
            Ok(CallerOutcome::Act4Ready) => route_errors.push(format!("seed={seed} normal-start fixed triple unexpectedly reached supplied Act4Ready")),
            Err(_) => (), // The complete Err result and caller were already persisted above.
        }
        if let Ok(outcome) = &attempt {
            if !suite_outcome_is_valid_terminal(outcome) {
                route_errors.push(format!("seed={seed} suite result is not a valid normal terminal: {outcome:?}"));
            }
        }
        let recorded_encounters = caller.trace.iter().flat_map(|step| {
            step.transition.committed_events.iter().filter_map(|event| match event {
                CommittedEvent::CombatFinished(record) if caller.trace.iter().any(|entered| {
                    entered.transition.committed_events.iter().any(|event| matches!(event,
                        CommittedEvent::CombatEntered { visit, encounter }
                            if *visit == record.visit && *encounter == record.encounter))
                }) => Some((step.decision.act, record.visit, record.encounter, record.outcome)),
                _ => None,
            })
        }).collect::<Vec<_>>();
        eprintln!("M14 records seed={seed} scenario={scenario:?} entered_and_recorded={recorded_encounters:?}");
        outcomes.push(format!("seed={seed} scenario={scenario:?} result={attempt:?} accepted_actions={} endpoint={:?}",
            caller.accepted(),
            caller.final_decision.as_ref().map(|decision| (decision.act, decision.floor, decision.terminal)),
        ));
        eprintln!("M14 seed={seed} scenario={scenario:?} result={attempt:?} accepted_actions={} endpoint={:?}",
            caller.accepted(),
            m14_support::endpoint_status(Some(&run), &caller),
        );
        if attempt.is_ok() {
            if let Err(error) = replay_transcript(seed, scenario, &caller) {
                route_errors.push(format!("seed={seed} scenario={scenario:?} transcript replay: {error}"));
            }
        }
    }

    assert_eq!(attempted_cases, 42, "complete every declared row before the aggregate gate");
    eprintln!("M14 actual Act3 boss coverage={act3_boss_coverage:?}, ordinary={act3_ordinary_coverage:?}, elite={act3_elite_coverage:?}; normal Act3NoKeys={no_key_completions}; outcomes={outcomes:?}");
    route_errors.extend(coverage_contract_errors(&act2_coverage, &act3_boss_coverage,
        &act3_ordinary_coverage, &act3_elite_coverage, no_key_completions, &outcomes));
    assert!(operational_errors.is_empty(), "M14 operational errors after all 42 attempts: {operational_errors:#?}; outcomes={outcomes:#?}");
    assert!(route_errors.is_empty(), "M14 fixed-route contract failures after all 42 attempts: {route_errors:#?}; outcomes={outcomes:#?}");
}

fn coverage_contract_errors(
    act2_coverage: &[EncounterId], act3_boss_coverage: &[EncounterId],
    act3_ordinary_coverage: &[EncounterId], act3_elite_coverage: &[EncounterId],
    _no_key_completions: usize, _outcomes: &[String],
) -> Vec<String> {
    let mut route_errors = Vec::new();
    let missing_act2 = [EncounterId::Champ, EncounterId::BronzeAutomaton, EncounterId::Collector]
        .into_iter().filter(|boss| !act2_coverage.contains(boss)).collect::<Vec<_>>();
    let missing_act3 = [EncounterId::AwakenedOne, EncounterId::TimeEater, EncounterId::DonuAndDeca]
        .into_iter().filter(|boss| !act3_boss_coverage.contains(boss)).collect::<Vec<_>>();
    if !missing_act2.is_empty() { route_errors.push(format!("predecessor Act2 boss entry/record coverage missing={missing_act2:?}")); }
    if !missing_act3.is_empty() { route_errors.push(format!("amended global Act3 boss entry/record coverage missing={missing_act3:?}")); }
    if act3_ordinary_coverage.is_empty() { route_errors.push("global real Act3 ordinary combat entry/record coverage is empty".into()); }
    if act3_elite_coverage.is_empty() { route_errors.push("global real Act3 elite combat entry/record coverage is empty".into()); }
    // Supplied-start approval replaces only the normal NoKeys gate.
    // Normal completion remains reported; its separate supplied proof is never
    // promoted into this normal-start entry/record coverage.
    route_errors
}

// Extracted old harness acceptance; the budget-negative test pins the approved
// suite requirement that exhausted budgets must also fail the aggregate gate.
fn suite_outcome_is_valid_terminal(outcome: &CallerOutcome) -> bool {
    matches!(outcome, CallerOutcome::Act3NoKeys | CallerOutcome::Death | CallerOutcome::EmptyDeck)
}

#[test]
fn m14_suite_oracle_rejects_budget_and_nonterminal_outcomes() {
    assert!(suite_outcome_is_valid_terminal(&CallerOutcome::Death));
    assert!(suite_outcome_is_valid_terminal(&CallerOutcome::EmptyDeck));
    assert!(suite_outcome_is_valid_terminal(&CallerOutcome::Act3NoKeys));
    for outcome in [CallerOutcome::BudgetExhausted, CallerOutcome::Act3Ready, CallerOutcome::Act4Ready] {
        assert!(!suite_outcome_is_valid_terminal(&outcome), "must fail suite: {outcome:?}");
    }
}

#[test]
fn m14_suite_oracle_rejects_every_missing_act3_boss_alternative() {
    let bosses = [EncounterId::AwakenedOne, EncounterId::TimeEater, EncounterId::DonuAndDeca];
    for missing in bosses {
        let present = bosses.into_iter().filter(|boss| *boss != missing).collect::<Vec<_>>();
        let errors = coverage_contract_errors(
            &[EncounterId::Champ, EncounterId::BronzeAutomaton, EncounterId::Collector],
            &present, &[EncounterId::OrbWalker], &[EncounterId::GiantHead], 1, &[],
        );
        assert!(errors.iter().any(|error| error.contains("Act3 boss") && error.contains(&format!("{missing:?}"))));
    }
}

#[test]
fn m14_suite_oracle_requires_real_ordinary_elite_but_reports_normal_completion() {
    for (ordinary, elite, completions) in [(false, true, 1), (true, false, 1), (true, true, 0), (true, true, 1)] {
        let errors = coverage_contract_errors(
            &[EncounterId::Champ, EncounterId::BronzeAutomaton, EncounterId::Collector],
            &[EncounterId::AwakenedOne, EncounterId::TimeEater, EncounterId::DonuAndDeca],
            if ordinary { &[EncounterId::OrbWalker] } else { &[] },
            if elite { &[EncounterId::GiantHead] } else { &[] }, completions, &[],
        );
        assert_eq!(errors.is_empty(), ordinary && elite);
    }
}

#[test]
fn m14_death_budget_invalid_and_cutoff_are_distinct() {
    assert_eq!(m14_support::parse_args(Vec::<String>::new()).unwrap().limit, 8192);
    assert!(m14_support::parse_args(["--budget".into(), "8193".into()]).is_err());
    assert!(Caller::new(4097).is_err());
    assert!(Caller::new_m14(8193).is_err());

    let (seed, scenario) = FIXED_CASES[0];
    for budget in [0, 1] {
        let mut run = m14_support::make_run(seed, scenario).unwrap();
        let mut caller = Caller::new_m14(budget).unwrap();
        assert_eq!(m14_support::rollout_m14(&mut run, &mut caller, false).unwrap(), CallerOutcome::BudgetExhausted);
        assert_eq!(caller.accepted(), budget);
        assert_eq!(run.decision().unwrap().terminal, None);
        assert_eq!(run.public_map().is_some(), budget == 1);
    }

    let mut run = m14_support::make_run(seed, scenario).unwrap();
    let mut caller = Caller::new_m14(8192).unwrap();
    assert_eq!(m14_support::rollout_m14(&mut run, &mut caller, true).unwrap(), CallerOutcome::Death);
    assert_eq!(caller.final_decision.as_ref().unwrap().terminal, Some(RunTerminal::Death));
    assert!(caller.trace.iter().any(|step| step.action == GameAction::Combat(spire_core::combat::CombatAction::EndTurn)));
    assert!(caller.trace.iter().any(|step| step.transition.terminal == Some(RunTerminal::Death)));

    let mut m13_run = m14_support::m13_support::make_run(seed, m14_support::m13_support::Scenario {
        act1_boss: scenario.act1_boss,
        act2_boss: scenario.act2_boss,
    }).unwrap();
    let before = m13_run.clone();
    let mut wrong_profile = Caller::new_m14(0).unwrap();
    assert!(m14_support::rollout_m14(&mut m13_run, &mut wrong_profile, false).is_err());
    assert_eq!(m13_run, before, "M14 caller rejects a predecessor profile before stepping it");
    assert!(wrong_profile.trace.is_empty());

    let mut invalid = m14_support::make_run(seed, scenario).unwrap();
    let before = invalid.clone();
    assert!(invalid.step(&GameAction::Move(spire_core::run::NodeId(u64::MAX))).is_err());
    assert_eq!(invalid, before);
    let mut one = Caller::new_m14(1).unwrap();
    assert_eq!(m14_support::rollout_m14(&mut invalid, &mut one, false).unwrap(), CallerOutcome::BudgetExhausted);
    let before = invalid.clone();
    let decision = invalid.decision().unwrap();
    let map = invalid.public_map();
    assert!(invalid.step(&GameAction::Move(spire_core::run::NodeId(u64::MAX))).is_err());
    assert_eq!(invalid, before);
    assert_eq!(invalid.decision().unwrap(), decision);
    assert_eq!(invalid.public_map(), map);
}

#[test]
fn m14_completion_classification_distinguishes_terminal_and_supplied_cutoff() {
    let decision = |room_kind, terminal| PublicDecision {
        content_profile: Some(RoomScenario::M14SilentAct3),
        room: Some(RoomView { kind: room_kind, choices: Vec::new() }),
        reward_total: 0,
        act: 3,
        floor: 50,
        player: PublicPlayer {
            relic_counters: Vec::new(),
            character: Character::Silent,
            hp: 70,
            max_hp: 70,
            gold: 0,
            relics: Vec::new(),
            potions: vec![None; 3],
            keys: [false; 3],
        },
        combat: None,
        combat_progress: Default::default(),
        last_combat: None,
        objects: Vec::new(),
        candidates: Vec::new(),
        offers: Vec::new(),
        selection: None,
        terminal,
    };
    assert_eq!(
        m14_support::m13_support::m14_stop_outcome(&decision(RoomViewKind::Act3Ending, Some(RunTerminal::Act3NoKeys))),
        Some(CallerOutcome::Act3NoKeys),
    );
    assert_eq!(
        m14_support::m13_support::m14_stop_outcome(&decision(RoomViewKind::Combat, Some(RunTerminal::Death))),
        Some(CallerOutcome::Death),
    );
    assert_eq!(
        m14_support::m13_support::m14_stop_outcome(&decision(RoomViewKind::Event(spire_core::content::EventId::Falling), Some(RunTerminal::EmptyDeck))),
        Some(CallerOutcome::EmptyDeck),
    );
    assert_eq!(
        m14_support::m13_support::m14_stop_outcome(&decision(RoomViewKind::Act4Ready, None)),
        Some(CallerOutcome::Act4Ready),
    );
    assert_eq!(
        m14_support::m13_support::m14_stop_outcome(&decision(RoomViewKind::Map, None)),
        None,
    );
}

fn require_true(errors: &mut Vec<String>, condition: bool, context: String) {
    if !condition {
        errors.push(context);
    }
}

fn require_eq<T>(errors: &mut Vec<String>, context: String, actual: T, expected: T)
where
    T: std::fmt::Debug + PartialEq,
{
    if actual != expected {
        errors.push(format!("{context}: expected {expected:?}, got {actual:?}"));
    }
}

fn target_for(scenario: Scenario, act: u8) -> EncounterId {
    match act {
        1 => scenario.act1_boss,
        2 => scenario.act2_boss,
        3 => scenario.act3_boss,
        _ => unreachable!("caller only evaluates Acts 1–3"),
    }
}

fn is_boss_for_act(encounter: EncounterId, act: u8) -> bool {
    match act {
        1 => matches!(encounter, EncounterId::Guardian | EncounterId::Hexaghost | EncounterId::SlimeBoss),
        2 => matches!(encounter, EncounterId::Champ | EncounterId::BronzeAutomaton | EncounterId::Collector),
        3 => matches!(encounter, EncounterId::AwakenedOne | EncounterId::TimeEater | EncounterId::DonuAndDeca),
        _ => false,
    }
}

fn is_act3_ordinary(encounter: EncounterId) -> bool {
    matches!(encounter,
        EncounterId::OrbWalker | EncounterId::ThreeShapes | EncounterId::SpireGrowth
        | EncounterId::Transient | EncounterId::FourShapes | EncounterId::Maw
        | EncounterId::SphereAndTwoShapes | EncounterId::JawWormHorde
        | EncounterId::ThreeDarklings | EncounterId::WrithingMass)
}

fn is_act3_elite(encounter: EncounterId) -> bool {
    matches!(encounter, EncounterId::GiantHead | EncounterId::Nemesis | EncounterId::Reptomancer)
}

fn remember_encounter(coverage: &mut Vec<EncounterId>, encounter: EncounterId) {
    if !coverage.contains(&encounter) {
        coverage.push(encounter);
    }
}

fn has_matching_record(trace: &[m14_support::TraceStep], visit: u64, encounter: EncounterId) -> bool {
    trace.iter().any(|step| step.transition.committed_events.iter().any(|event| {
        matches!(event, CommittedEvent::CombatFinished(record)
            if record.visit == visit && record.encounter == encounter)
    }))
}

fn replay_transcript(seed: u64, scenario: Scenario, caller: &Caller) -> Result<(), String> {
    let mut replay = m14_support::make_run(seed, scenario)?;
    for (index, step) in caller.trace.iter().enumerate() {
        let decision = replay.decision().map_err(|error| format!("decision {index}: {error:?}"))?;
        if decision != step.decision {
            return Err(format!("decision {index} mismatch"));
        }
        if replay.public_map() != step.map {
            return Err(format!("map {index} mismatch"));
        }
        let transition = replay.step(&step.action).map_err(|error| format!("transition {index}: {error:?}"))?;
        if transition != step.transition {
            return Err(format!("transition {index} mismatch"));
        }
    }
    let final_decision = caller.final_decision.as_ref().ok_or("caller has no recorded endpoint")?;
    let replay_decision = replay.decision().map_err(|error| format!("final decision: {error:?}"))?;
    if replay_decision != *final_decision {
        return Err("final decision mismatch".into());
    }
    if replay.public_map() != caller.final_map {
        return Err("final map mismatch".into());
    }
    Ok(())
}
