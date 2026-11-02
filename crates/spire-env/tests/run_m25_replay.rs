use serde_json::{json, Value};
use spire_core::{combat::{rng::Partition, CombatAction}, content::Character, run::GameAction};
use spire_env::*;

fn config() -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Watcher,
        ascension: 0,
        partition: Partition::Test,
        boss_history: RunBossHistory::M25WatcherHeart {
            act1: RunAct1BossTarget::Guardian,
            act2: RunAct2BossTarget::Champ,
            act3: RunAct3BossTarget::AwakenedOne,
        },
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn valid_record() -> RunEpisodeRecord {
    let mut session = RunSession::new_m25(config()).unwrap();
    session.reset(Some(7)).unwrap();
    session.record().unwrap().unwrap().clone()
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn m25_replay_preflight() {
    let record = valid_record();
    assert_eq!(hex(record.observation_fingerprint), "09355f51be8b906ecfe1bfd3465f400107ec387d4d5ebe1964f80a6810e77550", "record identity binds the accepted M25 descriptor");
    let replay = replay_run(&record, config().capacities).unwrap();
    assert_eq!(replay.accepted_count, 0);
    assert_eq!(replay.final_decision.content_profile, Some(spire_core::run::RoomScenario::M25WatcherHeart));

    let mutate: [(&str, Box<dyn Fn(&mut RunEpisodeRecord)>); 12] = [
        ("schema", Box::new(|value| value.schema_id = "wrong-schema".into())),
        ("profile", Box::new(|value| value.profile_id = "M24WatcherAct3".into())),
        ("rules", Box::new(|value| value.rules_id = "wrong-rules".into())),
        ("content", Box::new(|value| value.content_id = "wrong-content".into())),
        ("seed-adapter", Box::new(|value| value.seed_adapter_id = "wrong-seed".into())),
        ("rng", Box::new(|value| value.rng_id = "wrong-rng".into())),
        ("reward", Box::new(|value| value.reward_id = "wrong-reward".into())),
        ("fingerprint-byte", Box::new(|value| value.observation_fingerprint[0] ^= 1)),
        ("m24-fingerprint", Box::new(|value| value.observation_fingerprint = m24_run_observation_spec(&RunCapacities::m10_default()).unwrap().fingerprint())),
        ("history", Box::new(|value| value.config.boss_history = RunBossHistory::M24WatcherAct3 { act1: RunAct1BossTarget::Guardian, act2: RunAct2BossTarget::Champ, act3: RunAct3BossTarget::AwakenedOne })),
        ("character", Box::new(|value| value.config.character = Character::Silent)),
        ("ascension", Box::new(|value| value.config.ascension = 1)),
    ];
    for (name, change) in mutate {
        let mut invalid = record.clone();
        change(&mut invalid);
        assert!(matches!(replay_run(&invalid, config().capacities), Err(RunSessionError::IncompatibleReplay(_))), "identity mutation {name}");
        assert!(invalid.to_json().is_err(), "invalid identity {name} exports");
    }

    // A M25-specific offers capacity is checked while importing the record,
    // before its observation fingerprint can be accepted or a Run initialized.
    let valid_json = record.to_json().unwrap();
    let mut invalid: Value = serde_json::from_slice(&valid_json).unwrap();
    invalid["config"]["capacities"]["offers"] = json!(0);
    assert_eq!(
        RunEpisodeRecord::from_json(&serde_json::to_vec(&invalid).unwrap()),
        Err(RunSessionError::IncompatibleReplay("invalid Run replay value")),
        "M25 offer capacity must fail in profile validation before descriptor identity or startup",
    );
}

#[test]
fn m25_strict_replay_decodes_card_offer_actions() {
    let mut wire: Value = serde_json::from_slice(&valid_record().to_json().unwrap()).unwrap();
    wire["steps"] = json!([
        {"action": {"verb": "combat.choose_card_offer", "index": 0}, "reward": 0, "terminal": null},
        {"action": {"verb": "combat.skip_card_offer"}, "reward": 0, "terminal": null}
    ]);

    let bytes = serde_json::to_vec(&wire).unwrap();
    let record = RunEpisodeRecord::from_json(&bytes).expect("strict M25 replay accepts card offer actions");
    assert_eq!(
        record.steps.iter().map(|step| step.action.clone()).collect::<Vec<_>>(),
        vec![
            GameAction::Combat(CombatAction::ChooseCardOffer { index: 0 }),
            GameAction::Combat(CombatAction::SkipCardOffer),
        ],
    );

    let exported = record.to_json().unwrap();
    assert_eq!(RunEpisodeRecord::from_json(&exported).unwrap(), record);
}
