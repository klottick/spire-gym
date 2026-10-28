use spire_core::{
    combat::rng::Partition,
    content::{CardId, Character, PotionId, RelicId},
    run::{CreationProfile, PublicObject, Run, RunConfig, RunRng, RoomScenario},
};
use spire_env::{
    encode_run_m12, m10_run_observation_spec, m12_run_observation_spec,
    policy_run_observation_spec, run_observation_spec, RunCapacities, RunSessionError,
    RUN_OBSERVATION_M10_SCHEMA_ID, RUN_OBSERVATION_M12_SCHEMA_ID,
    RUN_OBSERVATION_POLICY_SCHEMA_ID, RUN_OBSERVATION_SCHEMA_ID,
};

fn initial_m12_decision() -> spire_core::run::PublicDecision {
    let mut creation = CreationProfile::experienced();
    creation.neow_eligible = false;
    Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 },
        creation,
        RunRng::new([31; 32], Partition::Test, 0),
        RoomScenario::M12SilentAct1,
    ).unwrap().decision().unwrap()
}

fn group<'a>(observation: &'a spire_env::RunObservation, key: &str) -> &'a spire_env::RunFeatureGroup {
    observation.group(key).unwrap()
}

#[test]
fn m12_schema_publishes_only_its_captured_card_item_profile_and_selection_slice() {
    let caps = RunCapacities::m10_default();
    let descriptor = m12_run_observation_spec(&caps).unwrap().descriptor();
    assert_eq!(descriptor["schema_id"], RUN_OBSERVATION_M12_SCHEMA_ID);
    assert_eq!(descriptor["schema_id"], "spire-gym-m12-silent-act1-observation-v1");
    assert_eq!(descriptor["profile_id"], "M12SilentAct1");
    assert_eq!(descriptor["vocabularies"]["card"], serde_json::json!([
        "padding", "strike_g", "defend_g", "survivor", "neutralize", "poisoned_stab",
        "backflip", "blade_dance", "deadly_poison", "cloak_and_dagger", "dagger_throw",
        "noxious_fumes", "accuracy", "well_laid_plans", "quick_slash", "reflex",
        "catalyst", "deflect", "tactician", "nightmare", "shiv", "dazed", "burn", "slimed"
    ]));
    assert_eq!(descriptor["vocabularies"]["relic"], serde_json::json!([
        "padding", "ring_of_the_snake", "neows_lament", "lantern", "pear", "mango",
        "snecko_skull", "toy_ornithopter", "strawberry", "anchor", "blood_vial",
        "bag_of_preparation", "empty_cage", "slavers_collar"
    ]));
    assert_eq!(descriptor["vocabularies"]["potion"], serde_json::json!([
        "padding", "fire_potion", "strength_potion", "weak_potion", "dexterity_potion"
    ]));
    assert_eq!(descriptor["vocabularies"]["enemy"], serde_json::json!([
        "padding", "cultist", "jaw_worm", "gremlin_nob", "lagavulin", "sentry", "guardian",
        "hexaghost", "fuzzy_louse_normal", "fuzzy_louse_defensive", "acid_slime_s", "acid_slime_m", "acid_slime_l",
        "spike_slime_s", "spike_slime_m", "spike_slime_l", "slime_boss"
    ]));
    assert_eq!(descriptor["vocabularies"]["encounter"], serde_json::json!([
        "padding", "cultist", "jaw_worm", "two_louse", "large_slime", "three_louse", "lots_of_slimes",
        "gremlin_nob", "lagavulin", "three_sentries", "guardian", "hexaghost", "slime_boss",
        "champ", "bronze_automaton", "collector"
    ]));
    assert_eq!(descriptor["semantics"]["encounters"]["playable"], serde_json::json!([
        "cultist", "jaw_worm", "two_louse", "large_slime", "three_louse", "lots_of_slimes",
        "gremlin_nob", "lagavulin", "three_sentries", "guardian", "hexaghost", "slime_boss"
    ]));
    assert_eq!(descriptor["semantics"]["encounters"]["map_only_boss_icons"], serde_json::json!(
        ["champ", "bronze_automaton", "collector"]));
    for excluded in ["blue_slaver", "red_slaver", "looter", "two_fungi_beasts", "gremlin_gang", "exordium_thugs", "exordium_wildlife"] {
        assert!(!descriptor["vocabularies"]["encounter"].as_array().unwrap()
            .iter().any(|value| value == excluded), "not a playable M12 encounter: {excluded}");
    }
    for excluded in ["blue_slaver", "red_slaver", "taskmaster", "gremlin_mad", "gremlin_sneaky"] {
        assert!(!descriptor["vocabularies"]["enemy"].as_array().unwrap()
            .iter().any(|value| value == excluded), "not a playable M12 enemy: {excluded}");
    }
    assert_eq!(descriptor["vocabularies"]["selection_operation"], serde_json::json!([
        "padding", "move_hand", "move_discard", "move_exhaust", "move_draw_top", "upgrade",
        "topdeck", "return_from_exhaust", "copy_to_hand_1", "copy_to_hand_2", "copy_to_hand_3",
        "select_permanent", "nightmare_capture", "retain"
    ]));
    for power in ["poison", "noxious_fumes", "accuracy", "retain_cards"] {
        assert!(descriptor["vocabularies"]["power"].as_array().unwrap().iter().any(|value| value == power));
    }
    assert!(!descriptor["vocabularies"]["power"].as_array().unwrap().iter().any(|value| value == "nightmare"),
        "Nightmare is represented by its card ID and selection operation; no native Nightmare power is published");
    assert!(descriptor["vocabularies"]["card"].as_array().unwrap().iter().all(|value|
        !["iron_wave", "searing_blow", "corruption", "ascenders_bane"].iter().any(|foreign| value == foreign)));
    assert!(descriptor["vocabularies"]["relic"].as_array().unwrap().iter().all(|value|
        !["burning_blood", "black_blood", "brimstone", "magic_flower"].iter().any(|foreign| value == foreign)));
    assert!(!descriptor["tensors"].as_array().unwrap().iter().any(|tensor| tensor["key"] == "combat_state"));
}

#[test]
fn m12_every_advertised_state_has_finite_codec_support() {
    let caps = RunCapacities::m10_default();
    let mut decision = initial_m12_decision();
    for (row, id) in spire_core::run::M12_SILENT_ACT1_CARDS.iter().copied().enumerate() {
        for upgrades in [0, 1] {
            decision.objects = vec![PublicObject { misc:0, instance: row as u64 + 1, id, upgrades }];
            let observation = encode_run_m12(&decision, None, &caps).unwrap();
            let encoded = group(&observation, "permanent_cards");
            assert_eq!(encoded.values[0], (row + 1) as i64, "{id:?} numeric ID");
            assert_eq!(encoded.values[1], upgrades as i64, "{id:?} upgrade count");
            assert_eq!(&encoded.known[..2], &[1, 1]);
        }
    }
    decision.objects = vec![PublicObject { misc:0, instance: 1, id: CardId::Shiv, upgrades: 0 }];
    assert!(encode_run_m12(&decision, None, &caps).is_ok(), "generated Shiv is in the transport vocabulary");
    for foreign in [CardId::IronWave, CardId::SearingBlow, CardId::AscendersBane] {
        decision.objects[0].id = foreign;
        assert_eq!(encode_run_m12(&decision, None, &caps), Err(RunSessionError::Input("unsupported M12 CardId")));
    }
}

#[test]
fn m12_encoder_covers_the_finite_relic_and_potion_rows_and_fails_closed() {
    let caps = RunCapacities::m10_default();
    let mut decision = initial_m12_decision();
    for (row, relic) in spire_core::run::M12_SILENT_ACT1_RELICS.iter().copied().enumerate() {
        decision.player.relics = vec![relic];
        let observation = encode_run_m12(&decision, None, &caps).unwrap();
        assert_eq!(group(&observation, "relics").values[0], (row + 1) as i64, "{relic:?}");
    }
    decision.player.relics = vec![RelicId::Brimstone];
    assert_eq!(encode_run_m12(&decision, None, &caps), Err(RunSessionError::Input("unsupported M12 RelicId")));
    decision.player.relics.clear();

    for (row, potion) in spire_core::run::M12_SILENT_ACT1_POTIONS.iter().copied().enumerate() {
        decision.player.potions = vec![Some(potion)];
        let observation = encode_run_m12(&decision, None, &caps).unwrap();
        assert_eq!(group(&observation, "potions").values[0], (row + 1) as i64, "{potion:?}");
    }
    decision.player.potions = vec![Some(PotionId::BloodPotion)];
    assert_eq!(encode_run_m12(&decision, None, &caps), Err(RunSessionError::Input("unsupported M12 PotionId")));
}

#[test]
fn m12_profile_and_room_reject_foreign_boundaries_before_encoding() {
    let caps = RunCapacities::m10_default();
    let mut decision = initial_m12_decision();
    decision.content_profile = Some(RoomScenario::M10IroncladHeart);
    assert_eq!(encode_run_m12(&decision, None, &caps), Err(RunSessionError::Input("M12 v1 encoder requires the M12 Silent Act1 profile")));
    decision = initial_m12_decision();
    decision.room.as_mut().unwrap().kind = spire_core::run::RoomViewKind::Act3Ready;
    assert!(matches!(encode_run_m12(&decision, None, &caps), Err(RunSessionError::UnsupportedBoundary(_))));
}

#[test]
fn m12_schema_names_the_real_next_act_map_boss_icon() {
    let caps = RunCapacities::m10_default();
    let descriptor = m12_run_observation_spec(&caps).unwrap().descriptor();
    for boss in ["champ", "bronze_automaton", "collector"] {
        assert!(descriptor["vocabularies"]["encounter"].as_array().unwrap()
            .iter().any(|value| value == boss), "M12 initialized-map boss icon {boss}");
    }

    let decision = initial_m12_decision();
    for (index, boss) in [
        spire_core::content::EncounterId::Champ,
        spire_core::content::EncounterId::BronzeAutomaton,
        spire_core::content::EncounterId::Collector,
    ].into_iter().enumerate() {
        let map = spire_core::run::map::PublicMap {
            boss: Some(boss), act: 2, nodes: Vec::new(), current: None, winged_moves: 0,
        };
        let observation = encode_run_m12(&decision, Some(&map), &caps).unwrap();
        let context = group(&observation, "context");
        assert_eq!(context.values[11], index as i64 + 13);
    }
    let map = spire_core::run::map::PublicMap {
        boss: Some(spire_core::content::EncounterId::CorruptHeart),
        act: 2, nodes: Vec::new(), current: None, winged_moves: 0,
    };
    assert_eq!(encode_run_m12(&decision, Some(&map), &caps), Err(RunSessionError::Input("unsupported M12 EncounterId")));
}

#[test]
fn new_m12_codec_leaves_m9_m10_and_66_policy_schema_ids_and_fingerprints_stable() {
    let caps = RunCapacities::m10_default();
    let v1 = run_observation_spec(&caps).unwrap();
    assert_eq!(v1.descriptor()["schema_id"], RUN_OBSERVATION_SCHEMA_ID);
    assert_eq!(v1.fingerprint().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999");
    let v2 = m10_run_observation_spec(&caps).unwrap();
    assert_eq!(v2.descriptor()["schema_id"], RUN_OBSERVATION_M10_SCHEMA_ID);
    let v3 = policy_run_observation_spec(&caps).unwrap();
    assert_eq!(v3.descriptor()["schema_id"], RUN_OBSERVATION_POLICY_SCHEMA_ID);
    assert_eq!(v2, m10_run_observation_spec(&caps).unwrap());
    assert_eq!(v3, policy_run_observation_spec(&caps).unwrap());
}

#[test]
fn m12_rejects_foreign_terminal_values_instead_of_emitting_out_of_descriptor_range(){
    for terminal in [spire_core::run::RunTerminal::Act3NoKeys,spire_core::run::RunTerminal::HeartVictory]{
        let mut decision=initial_m12_decision();decision.terminal=Some(terminal);decision.candidates.clear();
        assert!(matches!(encode_run_m12(&decision,None,&RunCapacities::m10_default()),Err(RunSessionError::UnsupportedBoundary(_))),"foreign terminal {terminal:?} must not exceed M12 terminal vocabulary");
    }
}
