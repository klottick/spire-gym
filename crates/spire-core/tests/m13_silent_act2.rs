#[path = "../examples/m13_support/mod.rs"]
#[allow(dead_code)]
mod m13_support;

use m13_support::{Caller, CallerOutcome, Scenario, FIXED_CASES};
use spire_core::{
    combat::{CombatAction, Outcome},
    content::{CardId, Character, EncounterId, PotionId, RelicId},
    run::{CommittedEvent, GameAction, MapNodeKind, RoomChoiceKind, RoomScenario, RoomViewKind, RunTerminal},
};

// Catches bypassed Neow, post-construction target mutation, missing Act2
// progression, fabricated readiness, and a caller that cannot cover the matrix.
#[test]
fn m13_caller_transcripts_unchanged_by_m14_extension() {
    const ACCEPTED_FULL_DEBUG_SHA256: [&str; 6] = [
        "370ea52171b0c7958d6927cecae5327b82b151504448f51aeff8f758f7d902fa",
        "d715a1e13a6d08472918b9bd637b9ad308437ad1e5cc6cc7128fcfb87c887165",
        "c30e3d6336a9d0059cee1beeeb19c83f6c767d80cd13483a7e3f152b2005165e",
        "749e206bc53aace53973bea3a5f91305a31baf1d0e8f28265ea5f863b92ced47",
        "2c42be5086c090b30d5d379512c7c559d809f8055a58e51c4483db0e05d89a34",
        "700cb47702ab4ce25ef72d600288798933dac99d7da7e624bc7eb22d7ff7b09e",
    ];
    let mut completions = 0;
    // The separately approved coverage amendment aggregates real final-act
    // entries across all six unchanged runs. Early Death is reported, never
    // replaced by a fixture or counted through its generated map icon.
    let mut act2_coverage = Vec::new();
    let mut outcomes = Vec::new();
    for (case_index, (seed, scenario)) in FIXED_CASES.into_iter().enumerate() {
        let mut run = m13_support::make_run(seed, scenario).unwrap();
        let initial = run.decision().unwrap();
        assert_eq!(initial.content_profile, Some(RoomScenario::M13SilentAct2));
        assert_eq!((initial.act, initial.floor), (1, 0));
        assert_eq!((initial.player.character, initial.player.hp, initial.player.max_hp, initial.player.gold), (Character::Silent, 70, 70, 99));
        assert_eq!(initial.player.relics, [RelicId::RingOfTheSnake]);
        assert_eq!(initial.player.potions, [None; 3]);
        assert_eq!(initial.player.keys, [false; 3]);
        assert_eq!(initial.objects.len(), 12);
        for (id, count) in [(CardId::StrikeGreen, 5), (CardId::DefendGreen, 5), (CardId::Survivor, 1), (CardId::Neutralize, 1)] {
            assert_eq!(initial.objects.iter().filter(|card| card.id == id).count(), count);
        }
        assert_eq!(initial.room.as_ref().unwrap().kind, RoomViewKind::Neow);
        assert!(run.public_map().is_none());

        let mut caller = Caller::new(4096).unwrap();
        let outcome = m13_support::rollout(&mut run, &mut caller, scenario).unwrap();
        let full_debug = format!("seed={seed} scenario={scenario:?} outcome={outcome:?}\n{caller:#?}\n");
        assert_eq!(sha256_hex(full_debug.as_bytes()), ACCEPTED_FULL_DEBUG_SHA256[case_index],
            "accepted M13 full public transcript remains byte-identical for seed={seed} scenario={scenario:?}");
        if let Some(directory) = std::env::var_os("SPIRE_M13_TRACE_DIR") {
            use std::io::Write;
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            let path = directory.join(format!("seed-{seed}-{:?}-{:?}.trace.txt", scenario.act1_boss, scenario.act2_boss));
            let mut file = std::fs::File::create(path).unwrap();
            writeln!(file, "seed={seed} scenario={scenario:?} outcome={outcome:?}\n{caller:#?}").unwrap();
        }
        assert!(matches!(outcome, CallerOutcome::Act3Ready | CallerOutcome::Death));
        let first = &caller.trace[0];
        assert_eq!(first.decision, initial);
        assert!(first.map.is_none());
        assert!(first.decision.room.as_ref().unwrap().choices.iter().any(|row| row.kind == RoomChoiceKind::NeowLament && first.action == GameAction::Choose(row.id)));
        for (act, boss) in [(1, scenario.act1_boss), (2, scenario.act2_boss)] {
            let map = caller.trace.iter().find(|step| step.decision.act == act && step.decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Map)).and_then(|step| step.map.as_ref()).expect("actual generated successor map");
            assert_eq!(map.boss, Some(boss));
            let entries = caller.trace.iter().filter(|step| step.decision.act == act)
                .flat_map(|step| &step.transition.committed_events)
                .filter_map(|event| match event {
                    CommittedEvent::CombatEntered { visit, encounter } if *encounter == boss => Some(*visit),
                    _ => None,
                }).collect::<Vec<_>>();
            if act == 1 {
                assert!(!entries.is_empty(), "the unchanged Act1 target must actually be entered");
            }
            for visit in entries {
                assert!(caller.trace.iter().any(|step| step.transition.committed_events.iter().any(|event|
                    matches!(event, CommittedEvent::CombatFinished(record) if record.encounter == boss && record.visit == visit))),
                    "actual boss entry needs its same-visit matching CombatRecord");
                if act == 2 && !act2_coverage.contains(&boss) { act2_coverage.push(boss); }
            }
            if act == 2 {
                for event in caller.trace.iter().filter(|step| step.decision.act == 2)
                    .flat_map(|step| &step.transition.committed_events) {
                    if let CommittedEvent::CombatEntered { encounter, .. } = event {
                        if matches!(encounter, EncounterId::Champ | EncounterId::BronzeAutomaton | EncounterId::Collector) {
                            assert_eq!(*encounter, boss, "an actual Act2 boss entry must match this run's initial target");
                        }
                    }
                }
            }
        }
        for kind in [MapNodeKind::Monster, MapNodeKind::Elite] {
            assert!(caller.trace.iter().any(|step| step.decision.act == 2 && matches!((&step.action, &step.map), (GameAction::Move(id), Some(map)) if map.nodes.iter().any(|node| node.id == *id && node.kind == kind))), "Act2 must traverse ordinary and elite rooms");
        }
        let act2_start = caller.trace.iter().find(|step| step.decision.act == 2 && step.decision.floor == 17).expect("actual Act1 successor initialization");
        assert_eq!(act2_start.decision.player.hp, act2_start.decision.player.max_hp, "once-only actual act healing");
        let first_rewards = caller.trace.iter().find(|step| step.decision.act == 2 && step.decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Rewards)).expect("actual first Act2 settlement");
        assert!(first_rewards.decision.room.as_ref().unwrap().choices.iter().any(|row| row.kind == RoomChoiceKind::Potion(PotionId::FirePotion)), "Act2 potion order starts locally");
        let first_cards = caller.trace.iter().find(|step| step.decision.act == 2 && step.decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::CardReward)).expect("actual first Act2 card reward");
        assert_eq!(first_cards.decision.room.as_ref().unwrap().choices.iter().filter_map(|row| match row.kind { RoomChoiceKind::Card { id, .. } => Some(id), _ => None }).collect::<Vec<_>>(), [CardId::PoisonedStab, CardId::Backflip, CardId::BladeDance]);
        let mut visited_shops = std::collections::BTreeSet::new();
        for step in caller.trace.iter().filter(|step| step.decision.act == 2 && step.decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Shop)) {
            if !visited_shops.insert(step.decision.floor) { continue; }
            let expected = if visited_shops.len() == 1 { [CardId::QuickSlash, CardId::Reflex, CardId::Catalyst] } else { [CardId::Deflect, CardId::Tactician, CardId::Nightmare] };
            assert_eq!(step.decision.room.as_ref().unwrap().choices.iter().filter_map(|row| match row.kind { RoomChoiceKind::Card { id, .. } => Some(id), _ => None }).collect::<Vec<_>>(), expected, "actual Act2-local shop visit order");
        }
        let final_decision = caller.final_decision.as_ref().unwrap();
        match outcome {
            CallerOutcome::Act3Ready => {
                completions += 1;
                assert_eq!((final_decision.act, final_decision.floor), (3, 34));
                assert_eq!(final_decision.terminal, None);
                assert_eq!(final_decision.room.as_ref().unwrap().kind, RoomViewKind::Act3Ready);
                assert!(final_decision.candidates.is_empty());
                assert_eq!(final_decision.last_combat.as_ref().unwrap().encounter, scenario.act2_boss);
                assert_eq!(final_decision.last_combat.as_ref().unwrap().outcome, Outcome::Victory);
                for room_kind in [RoomViewKind::Rewards, RoomViewKind::CardReward, RoomViewKind::BossTreasure, RoomViewKind::BossRelicChoice] {
                    assert!(caller.trace.iter().any(|step| step.decision.act == 2 && step.decision.floor == if matches!(room_kind, RoomViewKind::Rewards | RoomViewKind::CardReward) { 33 } else { 34 } && step.decision.room.as_ref().is_some_and(|room| room.kind == room_kind)), "actual Act2 boss child {room_kind:?}");
                }
            }
            CallerOutcome::Death => assert_eq!(final_decision.terminal, Some(RunTerminal::Death)),
            CallerOutcome::BudgetExhausted => unreachable!(),
            CallerOutcome::Act3NoKeys | CallerOutcome::Act4Ready | CallerOutcome::EmptyDeck => unreachable!("M13 stops at Act3Ready"),
        }
        outcomes.push((seed, scenario, outcome, caller.accepted()));
        eprintln!("seed={seed} scenario={scenario:?} outcome={outcome:?} actions={}", caller.accepted());
        replay_transcript(seed, scenario, &caller);
    }
    assert_eq!(outcomes.len(), 6, "all six original fixed cases must be run and reported");
    let missing_species = [EncounterId::Champ, EncounterId::BronzeAutomaton, EncounterId::Collector]
        .into_iter().filter(|boss| !act2_coverage.contains(boss)).collect::<Vec<_>>();
    eprintln!("M13 amended coverage={act2_coverage:?}; Act3Ready completions={completions}; all six outcomes={outcomes:?}");
    assert!(missing_species.is_empty() && completions >= 1,
        "M13 amended delivery gate unmet: missing actual Act2 boss species={missing_species:?}; Act3Ready completions={completions} (at least one required); all six outcomes={outcomes:?}");
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

// Catches fake success on limits/loss and invalid-action partial mutation.
#[test]
fn m13_natural_death_budget_and_invalid_requests_are_distinct() {
    for seed in [7, 11] {
        let scenario = m13_support::cases_for_seed(seed).unwrap()[0];
        let mut run = m13_support::make_run(seed, scenario).unwrap();
        let mut caller = Caller::new(4096).unwrap();
        caller.nonattacking = true;
        assert_eq!(m13_support::rollout(&mut run, &mut caller, scenario).unwrap(), CallerOutcome::Death);
        assert_eq!(caller.final_decision.as_ref().unwrap().terminal, Some(RunTerminal::Death));
        assert!(caller.trace.iter().any(|step| step.action == GameAction::Combat(CombatAction::EndTurn)));
        assert!(caller.trace.iter().any(|step| step.transition.terminal == Some(RunTerminal::Death)));
        replay_transcript(seed, scenario, &caller);
        for budget in [0, 1] {
            let mut run = m13_support::make_run(seed, scenario).unwrap();
            let mut caller = Caller::new(budget).unwrap();
            assert_eq!(m13_support::rollout(&mut run, &mut caller, scenario).unwrap(), CallerOutcome::BudgetExhausted);
            assert_eq!(caller.accepted(), budget);
            assert_eq!(run.decision().unwrap().terminal, None);
            assert_eq!(run.public_map().is_some(), budget == 1);
        }
    }
    assert!(Caller::new(4097).is_err());
    assert!(m13_support::cases_for_seed(13).is_err());
    assert!(m13_support::parse_args(["--seed".into(), "13".into()]).is_err());
    assert!(m13_support::parse_args(["--scenario".into(), "foreign".into()]).is_err());
    assert!(m13_support::parse_args(["--budget".into(), "4097".into()]).is_err());
    let (seed, scenario) = FIXED_CASES[0];
    let mut run = m13_support::make_run(seed, scenario).unwrap();
    let before = run.clone();
    assert!(run.step(&GameAction::Move(spire_core::run::NodeId(u64::MAX))).is_err());
    assert_eq!(run, before);
    let mut one = Caller::new(1).unwrap();
    assert_eq!(m13_support::rollout(&mut run, &mut one, scenario).unwrap(), CallerOutcome::BudgetExhausted);
    let before = run.clone();
    let decision = run.decision().unwrap();
    let map = run.public_map();
    assert!(map.is_some());
    assert!(run.step(&GameAction::Move(spire_core::run::NodeId(u64::MAX))).is_err());
    assert_eq!(run, before);
    assert_eq!(run.decision().unwrap(), decision);
    assert_eq!(run.public_map(), map);
}

fn replay_transcript(seed: u64, scenario: Scenario, caller: &Caller) {
    let mut replay = m13_support::make_run(seed, scenario).unwrap();
    for (index, step) in caller.trace.iter().enumerate() {
        assert_eq!(replay.decision().unwrap(), step.decision, "decision {index}");
        assert_eq!(replay.public_map(), step.map, "map {index}");
        assert_eq!(replay.step(&step.action).unwrap(), step.transition, "transition {index}");
    }
    assert_eq!(replay.decision().unwrap(), *caller.final_decision.as_ref().unwrap());
    assert_eq!(replay.public_map(), caller.final_map);
}
