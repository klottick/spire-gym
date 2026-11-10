use spire_core::{combat::rng::Partition, content::{CardId, Character}, run::{RewardItemKind, RunTerminal}};
use spire_env::{RunActionIndex, RunSession, RunSessionError, StandardCreation, StandardNeow, StandardRunConfig};

fn standard(character: Character, ascension: u8) -> StandardRunConfig {
    StandardRunConfig::new(character, ascension, Partition::Test)
}

fn first_legal(boundary: &spire_env::RunBoundary) -> usize {
    boundary.observation.action_mask.iter().position(|&value| value != 0).expect("at least one legal action")
}

fn take_step(session: &mut RunSession, boundary: spire_env::RunBoundary, index: usize) -> spire_env::RunStepResult {
    session.step(boundary.token, RunActionIndex(index as i64)).unwrap()
}

fn context(boundary: &spire_env::RunBoundary) -> &spire_env::RunFeatureGroup {
    boundary.observation.group("context").unwrap()
}

#[test]
fn standard_constructs_every_character_and_ascension() {
    for character in Character::ALL {
        for ascension in [0, 10, 19, 20] {
            let mut session = RunSession::new_standard(standard(character, ascension)).unwrap();
            let boundary = session.reset(Some(1200 + u64::from(ascension))).unwrap();
            let ctx = context(&boundary);
            assert_eq!(ctx.known[1], 1, "ascension must be known for {character:?} A{ascension}");
            assert_eq!(ctx.values[1], i64::from(ascension));
            let char_id = match character { Character::Ironclad => 1, Character::Silent => 2, Character::Defect => 3, Character::Watcher => 4 };
            assert_eq!(ctx.values[0], char_id);
        }
    }
}

#[test]
fn standard_rejects_invalid_creation() {
    assert!(matches!(RunSession::new_standard(standard(Character::Ironclad, 21)), Err(RunSessionError::Input(_))));
    for note in [(CardId::IronWave, 5), (CardId::AscendersBane, 0)] {
        let mut config = standard(Character::Ironclad, 0);
        config.creation.stored_note = note;
        assert!(matches!(RunSession::new_standard(config), Err(RunSessionError::Input(_))), "accepted {note:?}");
    }

    let mut wrong_act = StandardCreation::experienced().to_json();
    wrong_act["seen_bosses"][1] = serde_json::json!(["Guardian"]);
    assert!(matches!(StandardCreation::from_json(&wrong_act), Err(RunSessionError::Input(_))));
    let mut unknown_boss = StandardCreation::experienced().to_json();
    unknown_boss["seen_bosses"][0] = serde_json::json!(["TheUnknownBoss"]);
    assert!(matches!(StandardCreation::from_json(&unknown_boss), Err(RunSessionError::Input(_))));

    let config = spire_env::RunSessionConfig { character: Character::Ironclad, ascension: 0, partition: Partition::Test,
        boss_history: spire_env::RunBossHistory::Standard(StandardCreation::experienced()),
        capacities: spire_env::RunCapacities::m10_default(), record_replay: false };
    assert!(matches!(RunSession::new(config), Err(RunSessionError::Input(_))), "captured profile accepted Standard creation");
}

#[test]
fn standard_default_creation_is_experienced() {
    assert_eq!(standard(Character::Watcher, 20).creation, StandardCreation::experienced());
}

fn trajectory(session: &mut RunSession, seed: u64, noisy_reads: bool) -> Vec<(spire_env::RunObservation, i32, bool)> {
    let mut boundary = session.reset(Some(seed)).unwrap();
    let mut out = Vec::new();
    for _ in 0..64 {
        if noisy_reads {
            let _ = session.environment_spec().unwrap();
            let _ = session.observation_spec().unwrap();
            let _ = session.reproduction().unwrap();
            let _ = session.record().unwrap();
        }
        let index = first_legal(&boundary);
        let result = take_step(session, boundary, index);
        out.push((result.boundary.observation.clone(), result.reward, result.terminated));
        boundary = result.boundary;
        if boundary.terminal.is_some() { break; }
    }
    out
}

#[test]
fn standard_same_seed_same_trajectory() {
    let mut a = RunSession::new_standard(standard(Character::Silent, 0)).unwrap();
    let mut b = RunSession::new_standard(standard(Character::Silent, 0)).unwrap();
    assert_eq!(trajectory(&mut a, 99, false), trajectory(&mut b, 99, false));
}

#[test]
fn standard_reset_none_advances_ordinal() {
    let mut config = standard(Character::Defect, 0);
    config.record_replay = true;
    let mut session = RunSession::new_standard(config).unwrap();
    session.reset(Some(183)).unwrap();
    let first = session.record().unwrap().unwrap();
    session.reset(None).unwrap();
    let second = session.record().unwrap().unwrap();
    assert_eq!((first.seed, first.ordinal), (183, 0));
    assert_eq!((second.seed, second.ordinal), (183, 1));
}

#[test]
fn partition_changes_rng() {
    let mut train_config = standard(Character::Ironclad, 0);
    train_config.partition = Partition::Train;
    let mut test_config = standard(Character::Ironclad, 0);
    test_config.partition = Partition::Test;
    let mut train = RunSession::new_standard(train_config).unwrap();
    let mut test = RunSession::new_standard(test_config).unwrap();
    assert_ne!(train.reset(Some(607)).unwrap().observation, test.reset(Some(607)).unwrap().observation);
}

#[test]
fn reads_do_not_consume_rng() {
    let mut a_config = standard(Character::Watcher, 0);
    a_config.record_replay = true;
    let mut a = RunSession::new_standard(a_config.clone()).unwrap();
    let mut b = RunSession::new_standard(a_config.clone()).unwrap();
    assert_eq!(trajectory(&mut a, 772, false), trajectory(&mut b, 772, true));
}

#[test]
fn creation_is_independent_across_resets() {
    let creation = StandardCreation { neow: StandardNeow::Short, seen_bosses: [[false; 3]; 3], stored_note: (CardId::IronWave, 0) };
    let mut config = standard(Character::Defect, 10);
    config.creation = creation;
    let mut session = RunSession::new_standard(config).unwrap();
    let spec = session.environment_spec().unwrap();
    let one = session.reset(Some(414)).unwrap();
    let two = session.reset(Some(414)).unwrap();
    let mut fresh = RunSession::new_standard({ let mut c = standard(Character::Defect, 10); c.creation = creation; c }).unwrap();
    let fresh_boundary = fresh.reset(Some(414)).unwrap();
    assert_eq!(one.observation, two.observation);
    assert_eq!(one.observation, fresh_boundary.observation);
    let _ = take_step(&mut session, two, first_legal(&one));
    assert_eq!(session.environment_spec().unwrap(), spec);
}

#[test]
fn standard_environment_spec_publishes_creation_without_seed_or_rng() {
    let mut config = standard(Character::Silent, 19);
    config.creation.neow = StandardNeow::Short;
    let session = RunSession::new_standard(config.clone()).unwrap();
    let spec = session.environment_spec().unwrap();
    assert_eq!(spec["character"], "silent");
    assert_eq!(spec["ascension"], 19);
    assert_eq!(spec["creation"], config.creation.to_json());
    fn keys(value: &serde_json::Value, found: &mut Vec<String>) {
        match value { serde_json::Value::Object(map) => for (key, value) in map { found.push(key.clone()); keys(value, found); },
            serde_json::Value::Array(rows) => for value in rows { keys(value, found); }, _ => {} }
    }
    let mut found = Vec::new(); keys(&spec, &mut found);
    for private in ["seed", "ordinal", "cursor", "rng_state"] { assert!(!found.iter().any(|key| key == private)); }
}

#[test]
fn captured_environment_spec_still_excludes_note() {
    let config = spire_env::RunSessionConfig { character: Character::Ironclad, ascension: 0, partition: Partition::Test,
        boss_history: spire_env::RunBossHistory::AwakenedUnseen, capacities: spire_env::RunCapacities::m10_default(), record_replay: false };
    let session = RunSession::new_m25(config);
    assert!(session.is_err(), "M25 needs Watcher");
    let config = spire_env::RunSessionConfig { character: Character::Watcher, ascension: 0, partition: Partition::Test,
        boss_history: spire_env::RunBossHistory::M25WatcherHeart { act1: spire_env::RunAct1BossTarget::Guardian,
            act2: spire_env::RunAct2BossTarget::Champ, act3: spire_env::RunAct3BossTarget::AwakenedOne },
        capacities: spire_env::RunCapacities::m10_default(), record_replay: false };
    let session = RunSession::new_m25(config).unwrap();
    let spec = session.environment_spec().unwrap();
    assert!(spec.get("creation").is_none());
    assert!(spec.get("stored_note").is_none());
}

#[test]
fn reward_events_itemise_and_sum() {
    let mut session = RunSession::new_standard({ let mut c = standard(Character::Ironclad, 0); c.record_replay = true; c }).unwrap();
    let mut boundary = session.reset(Some(906)).unwrap();
    let mut rng = 0x9e37_79b9_u64;
    for _ in 0..3000 {
        let legal = boundary.observation.action_mask.iter().enumerate().filter_map(|(i, &v)| (v != 0).then_some(i)).collect::<Vec<_>>();
        assert!(!legal.is_empty(), "nonterminal Standard boundary without legal actions");
        rng ^= rng << 13; rng ^= rng >> 7; rng ^= rng << 17;
        let index = legal[(rng as usize) % legal.len()];
        let result = take_step(&mut session, boundary, index);
        assert_eq!(result.reward_events.iter().map(|item| item.amount).sum::<i32>(), result.reward);
        let next_context = context(&result.boundary);
        for item in &result.reward_events {
            match item.kind {
                RewardItemKind::NewFloor { act, floor } => {
                    assert_eq!(item.amount, 1);
                    assert_eq!((next_context.values[2], next_context.values[3]), (i64::from(act), i64::from(floor)));
                }
                RewardItemKind::NewAct4Floor { floor } => {
                    assert_eq!(item.amount, 10);
                    assert_eq!((next_context.values[2], next_context.values[3]), (4, i64::from(floor)));
                }
                RewardItemKind::FirstKey(key) => {
                    assert_eq!(item.amount, 5);
                    let column = match key { spire_core::run::Key::Ruby => 8, spire_core::run::Key::Sapphire => 9, spire_core::run::Key::Emerald => 10 };
                    assert_eq!(next_context.values[column], 1);
                }
                RewardItemKind::HeartVictory => { assert_eq!(item.amount, 100); assert_eq!(next_context.values[6], 4); }
                RewardItemKind::Death => { assert_eq!(item.amount, -1); assert_eq!(next_context.values[6], 1); }
                RewardItemKind::EmptyDeck => { assert_eq!(item.amount, -1); assert_eq!(next_context.values[6], 2); }
            }
        }
        assert!(!result.truncated);
        boundary = result.boundary;
        if boundary.terminal.is_some() { break; }
    }
}

#[test]
fn standard_terminal_reasons_are_distinct() {
    let terminal_kinds = [RunTerminal::HeartVictory, RunTerminal::Death, RunTerminal::EmptyDeck, RunTerminal::Act3NoKeys];
    for i in 0..terminal_kinds.len() { for j in i + 1..terminal_kinds.len() { assert_ne!(terminal_kinds[i], terminal_kinds[j]); } }
    let session = RunSession::new_standard(standard(Character::Ironclad, 0)).unwrap();
    let vocabulary = session.observation_spec().unwrap().descriptor()["vocabularies"]["terminal"].as_array().unwrap().clone();
    assert_eq!(vocabulary.iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>(),
        ["ongoing", "death", "empty_deck", "act3_no_keys", "heart_victory"]);
}

#[test]
fn standard_fault_freezes_without_retry() {
    let mut config = standard(Character::Ironclad, 0);
    config.record_replay = true;
    config.capacities.actions = 1;
    let mut session = RunSession::new_standard(config).unwrap();
    let error = session.reset(Some(81)).unwrap_err();
    assert!(matches!(error, RunSessionError::Capacity { .. }));
    assert_eq!(session.reset(Some(82)).unwrap_err(), error);
    assert_eq!(session.step(spire_env::RunDecisionToken::from([0, 0]), RunActionIndex(0)).unwrap_err(), error);
    let record = session.record().unwrap().unwrap();
    let fault = record.fault.unwrap();
    assert_eq!((fault.committed, fault.gameplay_recorded), (false, false));
    assert!(matches!(fault.error.kind, spire_env::RunRecordErrorKind::Capacity));
}


/// Every published value lies inside its descriptor column domain (Gymnasium `Box.contains`), including padding,
/// over seeded random-legal rollouts for every character at A0 and A20 (main, M34b lane P finding).
#[test]
fn standard_observations_lie_in_descriptor_domains() {
    let spec = spire_env::standard_run_observation_spec(&spire_env::RunCapacities::standard_default()).unwrap().descriptor();
    let tensors = spec["tensors"].as_array().unwrap();
    let check = |obs: &spire_env::RunObservation, at: &str| {
        for group in &obs.groups {
            let tensor = tensors.iter().find(|t| t["key"] == group.key).unwrap_or_else(|| panic!("{} undeclared", group.key));
            let (low, high) = (tensor["low"].as_array().unwrap(), tensor["high"].as_array().unwrap());
            let cols = *group.shape.last().unwrap();
            for (i, value) in group.values.iter().enumerate() {
                let (lo, hi) = (low[i % cols].as_i64().unwrap(), high[i % cols].as_i64().unwrap());
                assert!((lo..=hi).contains(value), "{at}: {}[{}] column {} = {value} outside {lo}..={hi}", group.key, i / cols, tensor["fields"][i % cols]);
            }
            assert!(group.known.iter().all(|k| (0..=1).contains(k)) && group.occupied.iter().flatten().all(|o| (0..=1).contains(o)));
        }
    };
    for character in Character::ALL {
        for ascension in [0u8, 20] {
            let mut session = RunSession::new_standard(standard(character, ascension)).unwrap();
            let mut boundary = session.reset(Some(4100 + u64::from(ascension))).unwrap();
            let mut x = 0x9E37_79B9_7F4A_7C15u64 ^ u64::from(ascension);
            for step in 0..3000 {
                check(&boundary.observation, &format!("{character:?} A{ascension} step {step}"));
                if boundary.terminal.is_some() { break }
                let legal = boundary.observation.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect::<Vec<_>>();
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                boundary = take_step(&mut session, boundary, legal[(x >> 33) as usize % legal.len()]).boundary;
            }
        }
    }
}
