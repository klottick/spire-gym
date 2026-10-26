use spire_core::{combat::rng::Partition, content::Character, run::{RoomViewKind, RunTerminal}};
use spire_env::{
    encode_run_m12, replay_run, RunActionIndex, RunBossHistory, RunCapacities, RunEpisodeRecord,
    RunSession, RunSessionConfig, RunSessionError, RUN_CONTENT_ID_M12, RUN_RECORD_SCHEMA_ID_M12,
    RUN_RULES_ID_M12,
};

#[path = "../../spire-core/examples/m12_support/mod.rs"]
#[allow(dead_code)]
mod m12_support;

fn history(scenario: m12_support::Scenario) -> RunBossHistory {
    match scenario {
        m12_support::Scenario::Guardian | m12_support::Scenario::Death => RunBossHistory::Act1GuardianUnseen,
        m12_support::Scenario::Hexaghost => RunBossHistory::Act1HexaghostUnseen,
        m12_support::Scenario::SlimeBoss => RunBossHistory::Act1SlimeBossUnseen,
    }
}

fn config(scenario: m12_support::Scenario, record_replay: bool) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: history(scenario),
        capacities: RunCapacities::m10_default(),
        record_replay,
    }
}

#[test]
fn m12_rejects_wrong_character_ascension_and_typed_boss_history_before_run_creation() {
    for scenario in [m12_support::Scenario::Guardian, m12_support::Scenario::Hexaghost, m12_support::Scenario::SlimeBoss] {
        assert!(RunSession::new_m12(config(scenario, false)).is_ok());
    }
    let mut wrong = config(m12_support::Scenario::Guardian, false);
    wrong.character = Character::Ironclad;
    assert!(matches!(RunSession::new_m12(wrong), Err(RunSessionError::Input("M12 requires Silent A0"))));
    let mut wrong = config(m12_support::Scenario::Guardian, false);
    wrong.ascension = 1;
    assert!(matches!(RunSession::new_m12(wrong), Err(RunSessionError::Input("M12 requires Silent A0"))));
    let mut wrong = config(m12_support::Scenario::Guardian, false);
    wrong.boss_history = RunBossHistory::AwakenedUnseen;
    assert!(matches!(RunSession::new_m12(wrong), Err(RunSessionError::Input("boss history does not match Run profile"))));

    let mut old = config(m12_support::Scenario::Guardian, false);
    old.character = Character::Ironclad;
    old.boss_history = RunBossHistory::Act1GuardianUnseen;
    assert!(matches!(RunSession::new(old), Err(RunSessionError::Input("boss history does not match Run profile"))));
}

#[test]
fn m12_sidecar_probe_is_unsupported_without_freezing_and_metadata_is_profile_accurate() {
    let mut session = RunSession::new_m12(config(m12_support::Scenario::Guardian, true)).unwrap();
    let metadata = session.environment_spec().unwrap();
    assert_eq!(metadata["profile_id"], "M12SilentAct1");
    assert_eq!(metadata["schema_id"], "spire-gym-m12-silent-act1-observation-v1");
    assert_eq!(metadata["rules_id"], RUN_RULES_ID_M12);
    assert_eq!(metadata["content_id"], RUN_CONTENT_ID_M12);
    assert_eq!(metadata["combat_view_id"], serde_json::Value::Null);
    assert_eq!(metadata["cutoff"], "Act2Ready: nonterminal, zero actions, no completion reward");
    assert_eq!(metadata["finite_content"]["permanent_cards"].as_array().unwrap().len(), 19);
    assert_eq!(metadata["finite_content"]["generated_and_status_cards"].as_array().unwrap().len(), 4);
    assert_eq!(metadata["finite_content"]["relics"].as_array().unwrap().len(), 13);
    assert_eq!(metadata["finite_content"]["potions"].as_array().unwrap().len(), 4);
    assert_eq!(metadata["finite_content"]["playable_encounters"], serde_json::json!([
        "cultist", "jaw_worm", "two_louse", "large_slime", "three_louse", "lots_of_slimes",
        "gremlin_nob", "lagavulin", "three_sentries", "guardian", "hexaghost", "slime_boss"
    ]));
    assert_eq!(metadata["finite_content"]["map_only_boss_icons"], serde_json::json!(
        ["champ", "bronze_automaton", "collector"]));
    assert_eq!(metadata["finite_content"]["playable_enemies"].as_array().unwrap().len(), 16);

    let boundary = session.reset(Some(7)).unwrap();
    let before = (session.decision().unwrap(), session.record().unwrap());
    assert_eq!(session.combat_observation_spec(), Err(RunSessionError::UnsupportedBoundary("M12 does not provide the Ironclad combat policy sidecar")));
    assert_eq!(session.combat_view(), Err(RunSessionError::UnsupportedBoundary("M12 does not provide the Ironclad combat policy sidecar")));
    assert_eq!((session.decision().unwrap(), session.record().unwrap()), before);
    assert_eq!(session.decision().unwrap(), boundary);
    assert!(session.reproduction().unwrap().fault.is_none());
}

#[test]
fn m12_readiness_cutoff_is_healthy_resettable_and_replayable() {
    for seed in [7, 11] {
      for scenario in [m12_support::Scenario::Guardian, m12_support::Scenario::Hexaghost, m12_support::Scenario::SlimeBoss] {
        let mut source = m12_support::make_run(seed, scenario).unwrap();
        let mut caller = m12_support::Caller::new(4096).unwrap();
        assert_eq!(m12_support::rollout(&mut source, &mut caller, scenario).unwrap(), m12_support::CallerOutcome::Act2Ready);
        assert!(caller.trace[0].map.is_none(), "the canonical M12 session and caller begin at Neow before map generation");
        assert_eq!(caller.final_decision.as_ref().unwrap().room.as_ref().map(|room| room.kind), Some(RoomViewKind::Act2Ready));
        assert_eq!(caller.final_decision.as_ref().unwrap().terminal, None);
        assert!(caller.final_decision.as_ref().unwrap().candidates.is_empty());
        assert_eq!(caller.trace.last().unwrap().transition.terminal, None);
        assert_eq!(caller.trace.last().unwrap().transition.reward, 0, "Act2Ready adds no completion reward");

        let mut session = RunSession::new_m12(config(scenario, true)).unwrap();
        let mut boundary = session.reset(Some(seed)).unwrap();
        assert_eq!(boundary.observation, encode_run_m12(&caller.trace[0].decision, caller.trace[0].map.as_ref(), &RunCapacities::m10_default()).unwrap());
        for (step_index, expected) in caller.trace.iter().enumerate() {
            let action_index = expected.decision.candidates.iter().position(|candidate| candidate.action == expected.action)
                .unwrap_or_else(|| panic!("source action not offered at {scenario:?}/{step_index}"));
            let step = session.step(boundary.token, RunActionIndex(action_index as i64))
                .unwrap_or_else(|error| panic!("M12 caller replay failed at {scenario:?}/{step_index}: {error:?}"));
            assert_eq!(step.reward, expected.transition.reward, "reward at {scenario:?}/{step_index}");
            assert_eq!(step.boundary.terminal, expected.transition.terminal, "terminal at {scenario:?}/{step_index}");
            assert!(!step.terminated && !step.truncated, "M12 has manual reset and nonterminal cutoff");
            assert_eq!(step.boundary.observation,
                encode_run_m12(&caller.trace.get(step_index + 1).map(|next| &next.decision).unwrap_or(caller.final_decision.as_ref().unwrap()),
                    caller.trace.get(step_index + 1).and_then(|next| next.map.as_ref()).or(caller.final_map.as_ref()),
                    &RunCapacities::m10_default()).unwrap(), "observation at {scenario:?}/{step_index}");
            boundary = step.boundary;
        }
        let final_decision = session.decision().unwrap();
        assert_eq!(final_decision.observation, encode_run_m12(caller.final_decision.as_ref().unwrap(), caller.final_map.as_ref(), &RunCapacities::m10_default()).unwrap());
        assert_eq!(final_decision.terminal, None);
        assert!(final_decision.observation.action_mask.iter().all(|value| *value == 0));

        let record = session.record().unwrap().unwrap();
        assert_eq!(record.schema_id, RUN_RECORD_SCHEMA_ID_M12);
        assert_eq!(record.schema_id, "spire-gym-m12-silent-act1-replay-v1");
        assert_eq!(record.profile_id, "M12SilentAct1");
        assert_eq!(record.rules_id, RUN_RULES_ID_M12);
        assert_eq!(record.content_id, RUN_CONTENT_ID_M12);
        assert_eq!(record.config.boss_history, history(scenario));
        assert_eq!(record.terminal, None);
        let decoded = RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
        assert_eq!(decoded, record);
        let replay = replay_run(&decoded, RunCapacities::m10_default()).unwrap();
        assert_eq!(replay.accepted_count, caller.trace.len());
        assert_eq!(replay.final_decision, caller.final_decision.clone().unwrap());
        assert_eq!(replay.final_map, caller.final_map);
        assert_eq!(replay.rewards, caller.trace.iter().map(|step| step.transition.reward).collect::<Vec<_>>());
        assert_eq!(replay.terminals, caller.trace.iter().map(|step| step.transition.terminal).collect::<Vec<_>>());
        assert_eq!(replay.reported_fault, None);

        let before = (session.decision().unwrap(), session.record().unwrap());
        assert_eq!(session.step(boundary.token, RunActionIndex(0)), Err(RunSessionError::UnsupportedBoundary("M12 stops at Act2Ready")));
        assert_eq!((session.decision().unwrap(), session.record().unwrap()), before, "cutoff action preflight does not mutate or freeze");
        assert!(session.reproduction().unwrap().fault.is_none());
        let reset = session.reset(None).unwrap();
        assert_eq!(session.record().unwrap().unwrap().seed, seed);
        assert_eq!(session.record().unwrap().unwrap().ordinal, 1);
        assert!(reset.token != boundary.token);
        assert_eq!(session.decision().unwrap(), reset);
        session.close().unwrap();
        assert_eq!(session.decision(), Err(RunSessionError::Closed));
      }
    }
}

#[test]
fn m12_real_death_and_replay_remain_terminal_and_distinct_from_cutoff() {
    let scenario = m12_support::Scenario::Death;
  for seed in [7, 11] {
    let mut source = m12_support::make_run(seed, scenario).unwrap();
    let mut caller = m12_support::Caller::new(4096).unwrap();
    assert_eq!(m12_support::rollout(&mut source, &mut caller, scenario).unwrap(), m12_support::CallerOutcome::Death);
    assert_eq!(caller.final_decision.as_ref().unwrap().terminal, Some(RunTerminal::Death));

    let mut session = RunSession::new_m12(config(scenario, true)).unwrap();
    let mut boundary = session.reset(Some(seed)).unwrap();
    for expected in &caller.trace {
        let index = expected.decision.candidates.iter().position(|candidate| candidate.action == expected.action).unwrap();
        let result = session.step(boundary.token, RunActionIndex(index as i64)).unwrap();
        boundary = result.boundary;
    }
    assert_eq!(boundary.terminal, Some(RunTerminal::Death));
    assert!(session.step(boundary.token, RunActionIndex(0)).is_err());
    let record = RunEpisodeRecord::from_json(&session.record().unwrap().unwrap().to_json().unwrap()).unwrap();
    assert_eq!(record.terminal, Some(RunTerminal::Death));
    assert_eq!(replay_run(&record, RunCapacities::m10_default()).unwrap().final_decision, caller.final_decision.unwrap());
  }
}
