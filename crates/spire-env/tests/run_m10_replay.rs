use spire_core::{
    combat::{self, CombatAction, rng::Partition},
    content::{CardId, Character, EncounterId, PotionId, RelicId},
    items::CombatItemContext,
    run::{GameAction, RunRng},
};
use spire_env::{
    RunCapacities, RunEpisodeRecord, RunRecordedStep, RunSession, RunSessionConfig, RunSessionError,
    M9BossHistory, replay_run, m10_run_observation_spec, run_observation_spec,
};

fn config() -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Ironclad,
        ascension: 0,
        partition: Partition::Test,
        boss_history: M9BossHistory::AwakenedUnseen,
        capacities: RunCapacities::m9_default(),
        record_replay: true,
    }
}

fn m9_prefix() -> RunEpisodeRecord {
    let mut session = RunSession::new(config()).unwrap();
    session.reset(Some(7)).unwrap();
    session.record().unwrap().unwrap()
}

/// This is a bounded supplied combat fixture, not a normal-start gameplay trace:
/// the native Heart combat consumes a real Elixir and publishes its optional
/// selector's legal menu. The confirmation below comes from that menu.
fn actual_optional_elixir_confirmation() -> GameAction {
    let cards = (1..=5)
        .map(|id| combat::permanent_live_card(CardId::Strike, combat::CardInstanceId(id), 0))
        .collect::<Vec<_>>();
    let setup = combat::enemies::setup(
        EncounterId::CorruptHeart,
        4,
        0,
        &mut RunRng::new([94; 32], Partition::Test, 0),
        combat::enemies::SetupContext {
            player: combat::PlayerState {
                hp: 80,
                max_hp: 80,
                block: 0,
                energy: 3,
                max_energy: 3,
                draw_per_turn: 5,
                powers: vec![],
            },
            player_powers: vec![],
            character: Character::Ironclad,
            cards,
            opening_draw: 5,
            encounter_visit: 0,
        },
    )
    .unwrap();
    let mut items = Some(CombatItemContext::new(
        vec![RelicId::BurningBlood],
        vec![Some(PotionId::Elixir), None, None],
    ));
    let mut combat = combat::Combat::enter_with_item_context(setup, &mut items).unwrap();
    combat
        .step_full(&CombatAction::UsePotion { slot: 0, target: None })
        .unwrap();
    let selection = combat.public_full().unwrap().selection.unwrap();
    assert_eq!(selection.minimum, 0);
    assert!(selection.maximum > 0);
    let action = combat
        .legal_actions_full()
        .unwrap()
        .into_iter()
        .find(|action| *action == CombatAction::ConfirmSelection)
        .expect("native optional Elixir menu offers confirmation");
    GameAction::Combat(action)
}

fn m10_header(record: &mut RunEpisodeRecord) {
    record.schema_id = "spire-gym-run-replay-v2".into();
    record.profile_id = "M10IroncladHeart".into();
    record.rules_id = "spire-gym-native-m10-rules-v1".into();
    record.content_id = "spire-gym-m10-ironclad-a0-captured-v1".into();
    record.config.capacities = RunCapacities::m10_default();
    record.observation_fingerprint = m10_run_observation_spec(&record.config.capacities)
        .unwrap()
        .fingerprint();
}

#[test]
fn m10_replay_v2_roundtrips_native_optional_confirm_selection() {
    let mut record = m9_prefix();
    m10_header(&mut record);
    let action = actual_optional_elixir_confirmation();
    record.steps.push(RunRecordedStep {
        action: action.clone(),
        reward: 0,
        terminal: None,
    });

    let bytes = record.to_json().expect("M10 v2 encodes actual optional confirmation");
    let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(wire["schema_id"], "spire-gym-run-replay-v2");
    assert_eq!(wire["profile_id"], "M10IroncladHeart");
    assert_eq!(wire["rules_id"], "spire-gym-native-m10-rules-v1");
    assert_eq!(wire["content_id"], "spire-gym-m10-ironclad-a0-captured-v1");
    assert_eq!(wire["seed_adapter_id"], "spire-gym-run-u64-le-v1");
    assert_eq!(wire["reward_id"], "spire-gym-floor-key-terminal-reward-v1");
    assert_eq!(wire["steps"][0]["action"]["verb"], "combat.confirm_selection");
    let decoded = RunEpisodeRecord::from_json(&bytes).unwrap();
    assert_eq!(decoded, record);

    // The action is genuinely legal in its supplied Elixir selector, but this
    // isolated wire fixture is not a replayable Run prefix: replay still checks
    // the action against the recorded normal-start Run's current menu.
    assert_eq!(
        replay_run(&decoded, RunCapacities::m10_default()),
        Err(RunSessionError::IncompatibleReplay("Run action not currently offered")),
    );

    // The same new action cannot be hidden under the unchanged M9/v1 identity.
    let mut mislabeled = decoded.clone();
    mislabeled.schema_id = "spire-gym-run-replay-v1".into();
    mislabeled.profile_id = "M9IroncladAct3".into();
    mislabeled.rules_id = "spire-gym-native-m9-rules-v2".into();
    mislabeled.content_id = "spire-gym-m9-ironclad-a0-captured-v1".into();
    mislabeled.config.capacities = RunCapacities::m9_default();
    mislabeled.observation_fingerprint = run_observation_spec(&RunCapacities::m9_default()).unwrap().fingerprint();
    assert_eq!(
        mislabeled.to_json(),
        Err(RunSessionError::UnsupportedBoundary("ConfirmSelection is not representable in replay v1")),
    );
    assert_eq!(
        replay_run(&mislabeled, RunCapacities::m9_default()),
        Err(RunSessionError::IncompatibleReplay("ConfirmSelection is not representable in replay v1")),
    );
    let mut m9_wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    m9_wire["schema_id"] = "spire-gym-run-replay-v1".into();
    m9_wire["profile_id"] = "M9IroncladAct3".into();
    m9_wire["rules_id"] = "spire-gym-native-m9-rules-v2".into();
    m9_wire["content_id"] = "spire-gym-m9-ironclad-a0-captured-v1".into();
    m9_wire["observation_fingerprint"] = serde_json::to_value(run_observation_spec(&RunCapacities::m9_default()).unwrap().fingerprint()).unwrap();
    assert_eq!(
        RunEpisodeRecord::from_json(&serde_json::to_vec(&m9_wire).unwrap()),
        Err(RunSessionError::IncompatibleReplay("ConfirmSelection is not representable in replay v1")),
    );
}

#[test]
fn m10_identity_and_fingerprint_preflight_precede_run_creation() {
    let mut record = m9_prefix();
    m10_header(&mut record);
    record.steps.push(RunRecordedStep {
        action: GameAction::Move(spire_core::run::NodeId(u64::MAX)),
        reward: 0,
        terminal: None,
    });
    let mut bad_identity = record.clone();
    bad_identity.content_id.push('x');
    assert_eq!(
        replay_run(&bad_identity, RunCapacities::m10_default()),
        Err(RunSessionError::IncompatibleReplay("Run identity mismatch")),
    );
    assert_eq!(bad_identity.to_json(), Err(RunSessionError::IncompatibleReplay("Run identity mismatch")));
    let mut bad_fingerprint = record.clone();
    bad_fingerprint.observation_fingerprint[0] ^= 1;
    assert_eq!(
        replay_run(&bad_fingerprint, RunCapacities::m10_default()),
        Err(RunSessionError::IncompatibleReplay("Run observation fingerprint mismatch")),
    );
    assert_eq!(bad_fingerprint.to_json(), Err(RunSessionError::IncompatibleReplay("Run observation fingerprint mismatch")));
    let mut bad_schema = record.clone();
    bad_schema.schema_id = "spire-gym-run-replay-v999".into();
    assert!(matches!(
        replay_run(&bad_schema, RunCapacities::m10_default()),
        Err(RunSessionError::IncompatibleReplay(_))
    ));
    let mut bad_profile = record.clone();
    bad_profile.profile_id = "M9IroncladAct3".into();
    assert_eq!(
        replay_run(&bad_profile, RunCapacities::m10_default()),
        Err(RunSessionError::IncompatibleReplay("Run identity mismatch")),
    );
    let mut bad_rules = record.clone();
    bad_rules.rules_id.push('x');
    assert_eq!(
        replay_run(&bad_rules, RunCapacities::m10_default()),
        Err(RunSessionError::IncompatibleReplay("Run identity mismatch")),
    );
    let mut bad_content = record.clone();
    bad_content.content_id.push('x');
    assert_eq!(
        replay_run(&bad_content, RunCapacities::m10_default()),
        Err(RunSessionError::IncompatibleReplay("Run identity mismatch")),
    );
}
