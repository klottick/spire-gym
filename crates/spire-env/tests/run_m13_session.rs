use spire_core::{
    combat::rng::Partition,
    content::{Character, EncounterId},
    run::RoomViewKind,
};
use spire_env::{
    encode_run_m13, replay_run, RunAct1BossTarget, RunAct2BossTarget, RunActionIndex,
    RunBossHistory, RunCapacities, RunEpisodeRecord, RunSession, RunSessionConfig,
    RunSessionError, RUN_CONTENT_ID_M13, RUN_RECORD_SCHEMA_ID_M13, RUN_RULES_ID_M13,
};

#[path = "../../spire-core/examples/m13_support/mod.rs"]
#[allow(dead_code)]
mod m13_support;

const CASES: [(u64, EncounterId, EncounterId); 6] = [
    (7, EncounterId::Guardian, EncounterId::Champ),
    (7, EncounterId::Hexaghost, EncounterId::BronzeAutomaton),
    (7, EncounterId::SlimeBoss, EncounterId::Collector),
    (11, EncounterId::Guardian, EncounterId::BronzeAutomaton),
    (11, EncounterId::Hexaghost, EncounterId::Collector),
    (11, EncounterId::SlimeBoss, EncounterId::Champ),
];

fn act1_target(boss: EncounterId) -> RunAct1BossTarget {
    match boss {
        EncounterId::Guardian => RunAct1BossTarget::Guardian,
        EncounterId::Hexaghost => RunAct1BossTarget::Hexaghost,
        EncounterId::SlimeBoss => RunAct1BossTarget::SlimeBoss,
        _ => panic!("not an Act1 target: {boss:?}"),
    }
}
fn act2_target(boss: EncounterId) -> RunAct2BossTarget {
    match boss {
        EncounterId::Champ => RunAct2BossTarget::Champ,
        EncounterId::BronzeAutomaton => RunAct2BossTarget::BronzeAutomaton,
        EncounterId::Collector => RunAct2BossTarget::Collector,
        _ => panic!("not an Act2 target: {boss:?}"),
    }
}
fn history(act1: EncounterId, act2: EncounterId) -> RunBossHistory {
    RunBossHistory::M13SilentAct2 {
        act1: act1_target(act1),
        act2: act2_target(act2),
    }
}
fn config(act1: EncounterId, act2: EncounterId, record_replay: bool) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: history(act1, act2),
        capacities: RunCapacities::m10_default(),
        record_replay,
    }
}
fn scenario(act1_boss: EncounterId, act2_boss: EncounterId) -> m13_support::Scenario {
    m13_support::Scenario { act1_boss, act2_boss }
}

#[test]
fn m13_typed_boss_history_accepts_all_nine_pairs_and_rejects_profile_mismatch() {
    let act1 = [
        RunAct1BossTarget::Guardian,
        RunAct1BossTarget::Hexaghost,
        RunAct1BossTarget::SlimeBoss,
    ];
    let act2 = [
        RunAct2BossTarget::Champ,
        RunAct2BossTarget::BronzeAutomaton,
        RunAct2BossTarget::Collector,
    ];
    for first in act1 {
        for second in act2 {
            assert!(RunSession::new_m13(RunSessionConfig {
                character: Character::Silent,
                ascension: 0,
                partition: Partition::Test,
                boss_history: RunBossHistory::M13SilentAct2 { act1: first, act2: second },
                capacities: RunCapacities::m10_default(),
                record_replay: false,
            }).is_ok(), "typed pair {first:?}/{second:?}");
        }
    }

    let mut m12 = config(EncounterId::Guardian, EncounterId::Champ, false);
    assert_eq!(
        RunSession::new_m12(m12.clone()).err(),
        Some(RunSessionError::Input("boss history does not match Run profile"))
    );
    m12.boss_history = RunBossHistory::Act1GuardianUnseen;
    assert_eq!(
        RunSession::new_m13(m12).err(),
        Some(RunSessionError::Input("boss history does not match Run profile"))
    );

    let mut wrong = config(EncounterId::Guardian, EncounterId::Champ, false);
    wrong.character = Character::Ironclad;
    assert_eq!(
        RunSession::new_m13(wrong).err(),
        Some(RunSessionError::Input("M13 requires Silent A0"))
    );
    let mut wrong = config(EncounterId::Guardian, EncounterId::Champ, false);
    wrong.ascension = 1;
    assert_eq!(
        RunSession::new_m13(wrong).err(),
        Some(RunSessionError::Input("M13 requires Silent A0"))
    );
}

fn compare_m13_session_with_six_fixed_core_transcripts() -> bool {
    let mut reached_act3_ready = false;
    for (seed, act1, act2) in CASES {
        let selected = scenario(act1, act2);
        let mut source = m13_support::make_run(seed, selected).unwrap();
        let mut caller = m13_support::Caller::new(4096).unwrap();
        let outcome = m13_support::rollout(&mut source, &mut caller, selected).unwrap();
        let final_decision = caller.final_decision.as_ref().unwrap();
        let final_map = caller.final_map.as_ref();
        eprintln!("fixed route {seed}/{act1:?}/{act2:?}: {outcome:?}");
        match outcome {
            m13_support::CallerOutcome::Act3Ready => {
                assert_eq!(final_decision.terminal, None);
                assert_eq!(final_decision.room.as_ref().map(|room| room.kind), Some(RoomViewKind::Act3Ready));
            }
            m13_support::CallerOutcome::Death => {
                assert_eq!(final_decision.terminal, Some(spire_core::run::RunTerminal::Death));
            }
            m13_support::CallerOutcome::BudgetExhausted => {
                panic!("fixed route exhausted the caller budget at {seed}/{act1:?}/{act2:?}");
            }
            m13_support::CallerOutcome::Act3NoKeys
            | m13_support::CallerOutcome::Act4Ready
            | m13_support::CallerOutcome::EmptyDeck => {
                unreachable!("M14-only caller outcomes cannot occur in an M13 rollout");
            }
        }
        let mut session = RunSession::new_m13(config(act1, act2, true)).unwrap();
        let mut boundary = session.reset(Some(seed)).unwrap();
        assert_eq!(
            boundary.observation,
            encode_run_m13(
                &caller.trace[0].decision,
                caller.trace[0].map.as_ref(),
                &RunCapacities::m10_default()
            ).unwrap(),
            "initial observation at {seed}/{act1:?}/{act2:?}"
        );

        for (step_index, expected) in caller.trace.iter().enumerate() {
            let action_index = expected.decision.candidates.iter()
                .position(|candidate| candidate.action == expected.action)
                .unwrap_or_else(|| panic!("source action absent at {seed}/{act1:?}/{act2:?}/{step_index}"));
            let step = session.step(boundary.token, RunActionIndex(action_index as i64))
                .unwrap_or_else(|error| panic!("M13 session rejected core action at {seed}/{act1:?}/{act2:?}/{step_index}: {error:?}"));
            assert_eq!(step.reward, expected.transition.reward, "reward at {seed}/{act1:?}/{act2:?}/{step_index}");
            assert_eq!(step.boundary.terminal, expected.transition.terminal, "terminal at {seed}/{act1:?}/{act2:?}/{step_index}");
            assert_eq!(step.terminated, expected.transition.terminal.is_some(), "terminal flag at {seed}/{act1:?}/{act2:?}/{step_index}");
            assert!(!step.truncated, "manual reset profile has no caller time limit");
            let next_decision = caller.trace.get(step_index + 1)
                .map(|next| &next.decision)
                .unwrap_or(final_decision);
            let next_map = caller.trace.get(step_index + 1)
                .and_then(|next| next.map.as_ref())
                .or(final_map);
            assert_eq!(
                step.boundary.observation,
                encode_run_m13(next_decision, next_map, &RunCapacities::m10_default()).unwrap(),
                "observation at {seed}/{act1:?}/{act2:?}/{step_index}"
            );
            boundary = step.boundary;
        }

        let final_boundary = session.decision().unwrap();
        assert_eq!(
            final_boundary.observation,
            encode_run_m13(final_decision, final_map, &RunCapacities::m10_default()).unwrap()
        );
        assert_eq!(final_boundary.terminal, final_decision.terminal);
        assert_eq!(session.reproduction().unwrap().latest_map, caller.final_map);
        let record = session.record().unwrap().unwrap();
        assert_eq!(record.schema_id, RUN_RECORD_SCHEMA_ID_M13);
        assert_eq!(record.profile_id, "M13SilentAct2");
        assert_eq!(record.rules_id, RUN_RULES_ID_M13);
        assert_eq!(record.content_id, RUN_CONTENT_ID_M13);
        assert_eq!(record.config.boss_history, history(act1, act2));
        let decoded = RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
        assert_eq!(decoded, record);
        let replay = replay_run(&decoded, RunCapacities::m10_default()).unwrap();
        assert_eq!(replay.accepted_count, caller.trace.len());
        assert_eq!(replay.final_decision, *final_decision);
        assert_eq!(replay.final_map, caller.final_map);
        assert_eq!(
            replay.rewards,
            caller.trace.iter().map(|step| step.transition.reward).collect::<Vec<_>>()
        );
        assert_eq!(
            replay.terminals,
            caller.trace.iter().map(|step| step.transition.terminal).collect::<Vec<_>>()
        );
        if outcome == m13_support::CallerOutcome::Act3Ready {
            reached_act3_ready = true;
        }
    }
    reached_act3_ready
}

#[test]
fn m13_session_matches_six_fixed_core_transcripts_and_replay_identity() {
    let _ = compare_m13_session_with_six_fixed_core_transcripts();
}

#[test]
fn m13_session_matches_six_fixed_core_transcripts_and_act3ready_cutoff() {
    assert!(
        compare_m13_session_with_six_fixed_core_transcripts(),
        "at least one fixed normal-start boss victory reaches Act3Ready"
    );
}

#[test]
fn m13_act3ready_is_healthy_resettable_zero_action_boundary() {
    let successful = CASES.into_iter().find_map(|(seed, act1, act2)| {
        let selected = scenario(act1, act2);
        let mut source = m13_support::make_run(seed, selected).ok()?;
        let mut caller = m13_support::Caller::new(4096).ok()?;
        (m13_support::rollout(&mut source, &mut caller, selected).ok()?
            == m13_support::CallerOutcome::Act3Ready)
            .then_some((seed, act1, act2, caller))
    }).expect("the fixed M13 matrix includes a normal Act3Ready completion");
    let (seed, act1, act2, caller) = successful;
    let mut session = RunSession::new_m13(config(act1, act2, true)).unwrap();
    let mut boundary = session.reset(Some(seed)).unwrap();
    for expected in &caller.trace {
        let index = expected.decision.candidates.iter()
            .position(|candidate| candidate.action == expected.action).unwrap();
        boundary = session.step(boundary.token, RunActionIndex(index as i64)).unwrap().boundary;
    }

    let final_boundary = session.decision().unwrap();
    assert_eq!(final_boundary.observation.group("context").unwrap().values[2], 3);
    assert_eq!(final_boundary.observation.group("context").unwrap().values[3], 34);
    assert_eq!(final_boundary.terminal, None);
    assert!(final_boundary.observation.action_mask.iter().all(|value| *value == 0));
    assert_eq!(caller.final_decision.as_ref().unwrap().terminal, None);
    assert_eq!(
        caller.final_decision.as_ref().unwrap().room.as_ref().map(|room| room.kind),
        Some(RoomViewKind::Act3Ready)
    );
    let before = (session.decision().unwrap(), session.record().unwrap(), session.reproduction().unwrap());
    assert_eq!(
        session.step(final_boundary.token, RunActionIndex(0)),
        Err(RunSessionError::UnsupportedBoundary("M13 stops at Act3Ready"))
    );
    assert_eq!(
        (session.decision().unwrap(), session.record().unwrap(), session.reproduction().unwrap()),
        before,
        "the rejected Act3 action does not mutate cutoff state or replay"
    );
    assert_eq!(session.reproduction().unwrap().fault, None);
    let reset = session.reset(None).unwrap();
    assert_ne!(reset.token, final_boundary.token);
    assert_eq!(session.record().unwrap().unwrap().seed, seed);
    assert_eq!(session.record().unwrap().unwrap().ordinal, 1);
    assert_eq!(session.decision().unwrap(), reset);
    assert_eq!(reset.terminal, None);
    assert_ne!(reset.observation.group("context").unwrap().values[2], 3);
}
