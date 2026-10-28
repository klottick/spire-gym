use spire_core::{
    combat::{enemies::{self, SetupContext}, rng::Partition, CardInstanceId, PlayerState},
    content::{CardId, Character, EncounterId, EnemyId, EventId, PotionId, RelicId},
    items::CombatItemContext,
    run::{
        ChoiceId, CreationProfile, PublicObject, RoomChoiceKind, RoomChoiceStatus,
        RoomChoiceView, Run, RunConfig, RunRng, RoomScenario, RoomViewKind,
    },
};
use spire_env::{
    encode_run_m13, m10_run_observation_spec, m12_run_observation_spec,
    m13_run_observation_spec, run_observation_spec, RunCapacities, RunSessionError,
    RUN_OBSERVATION_M10_SCHEMA_ID, RUN_OBSERVATION_M12_SCHEMA_ID, RUN_OBSERVATION_SCHEMA_ID,
};

const ACT2_ENCOUNTERS: [EncounterId; 12] = [
    EncounterId::SphericGuardian, EncounterId::ThreeByrds, EncounterId::CenturionAndHealer,
    EncounterId::Snecko, EncounterId::ChosenAndByrds, EncounterId::ThreeCultists,
    EncounterId::GremlinLeader, EncounterId::Slavers, EncounterId::BookOfStabbing,
    EncounterId::Champ, EncounterId::Collector, EncounterId::BronzeAutomaton,
];
const M13_ADDED_ACT2_ENEMIES: [EnemyId; 21] = [
    EnemyId::SphericGuardian, EnemyId::Byrd, EnemyId::Chosen, EnemyId::Snecko,
    EnemyId::Centurion, EnemyId::Mystic, EnemyId::GremlinLeader, EnemyId::GremlinMad,
    EnemyId::GremlinSneaky, EnemyId::GremlinFat, EnemyId::GremlinShield, EnemyId::GremlinWizard,
    EnemyId::SlaverBlue, EnemyId::Taskmaster, EnemyId::SlaverRed, EnemyId::BookOfStabbing,
    EnemyId::Champ, EnemyId::Collector, EnemyId::TorchHead, EnemyId::BronzeAutomaton,
    EnemyId::BronzeOrb,
];

fn initial_m13_decision() -> spire_core::run::PublicDecision {
    let mut creation = CreationProfile::experienced();
    creation.neow_eligible = false;
    Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 },
        creation, RunRng::new([31; 32], Partition::Test, 0), RoomScenario::M13SilentAct2,
    ).unwrap().decision().unwrap()
}
fn group<'a>(observation: &'a spire_env::RunObservation, key: &str) -> &'a spire_env::RunFeatureGroup {
    observation.group(key).unwrap()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn m13_schema_publishes_only_its_captured_act12_content_and_act3_cutoff() {
    let caps = RunCapacities::m10_default();
    let descriptor = m13_run_observation_spec(&caps).unwrap().descriptor();
    assert_eq!(descriptor["schema_id"], "spire-gym-m13-silent-act2-observation-v1");
    assert_eq!(descriptor["profile_id"], "M13SilentAct2");
    assert_eq!(descriptor["vocabularies"]["card"], serde_json::json!([
        "padding", "strike_g", "defend_g", "survivor", "neutralize", "poisoned_stab",
        "backflip", "blade_dance", "deadly_poison", "cloak_and_dagger", "dagger_throw",
        "noxious_fumes", "accuracy", "well_laid_plans", "quick_slash", "reflex",
        "catalyst", "deflect", "tactician", "nightmare", "shiv", "dazed", "burn",
        "slimed", "wound"
    ]));
    assert_eq!(descriptor["vocabularies"]["relic"], serde_json::json!([
        "padding", "ring_of_the_snake", "neows_lament", "lantern", "pear", "mango",
        "snecko_skull", "toy_ornithopter", "strawberry", "anchor", "blood_vial",
        "bag_of_preparation", "empty_cage", "slavers_collar"
    ]));
    assert_eq!(descriptor["vocabularies"]["potion"], serde_json::json!([
        "padding", "fire_potion", "strength_potion", "weak_potion", "dexterity_potion"
    ]));
    assert_eq!(descriptor["vocabularies"]["event"], serde_json::json!([
        "padding", "neow", "upgrade_shrine", "purifier", "ancient_writing", "joust"
    ]));
    for encounter in ACT2_ENCOUNTERS {
        assert!(
            descriptor["vocabularies"]["encounter"].as_array().unwrap()
                .iter().any(|value| value == encounter.as_str()),
            "Act2 encounter row: {encounter:?}"
        );
    }
    for enemy in M13_ADDED_ACT2_ENEMIES {
        assert!(
            descriptor["vocabularies"]["enemy"].as_array().unwrap()
                .iter().any(|value| value == enemy.as_str()),
            "Act2 enemy row: {enemy:?}"
        );
    }
    assert_eq!(
        descriptor["semantics"]["encounters"]["playable"],
        serde_json::json!([
            "cultist", "jaw_worm", "two_louse", "large_slime", "three_louse", "lots_of_slimes",
            "gremlin_nob", "lagavulin", "three_sentries", "guardian", "hexaghost", "slime_boss",
            "spheric_guardian", "three_byrds", "centurion_and_healer", "snecko",
            "chosen_and_byrds", "three_cultists", "gremlin_leader", "slavers",
            "book_of_stabbing", "champ", "collector", "bronze_automaton"
        ])
    );
    assert_eq!(
        descriptor["semantics"]["encounters"]["map_only_boss_icons"],
        serde_json::json!(["awakened_one", "time_eater", "donu_and_deca"])
    );
    assert!(descriptor["vocabularies"]["room"].as_array().unwrap().iter().any(|v| v == "act3_ready"));
    assert!(!descriptor["vocabularies"]["choice"].as_array().unwrap().iter().any(|v| v == "act3_continue"));
    for power in [
        "artifact", "barricade", "flight", "hex", "confused", "minion", "angry",
        "entangled", "painful_stabs", "stasis", "noxious_fumes", "accuracy", "retain_cards",
    ] {
        assert!(
            descriptor["vocabularies"]["power"].as_array().unwrap().iter().any(|v| v == power),
            "public power row {power}"
        );
    }
    for group_name in [
        "context", "map_nodes", "permanent_cards", "combat_cards", "enemies", "powers",
        "relics", "potions", "offers", "selection", "selection_counts", "actions",
    ] {
        assert!(
            descriptor["tensors"].as_array().unwrap().iter().any(|tensor| tensor["key"] == group_name),
            "shared observation group {group_name}"
        );
    }

    let m9 = run_observation_spec(&caps).unwrap();
    let m10 = m10_run_observation_spec(&caps).unwrap();
    let m12 = m12_run_observation_spec(&caps).unwrap();
    assert_eq!(m9.descriptor()["schema_id"], RUN_OBSERVATION_SCHEMA_ID);
    assert_eq!(m9.descriptor()["schema_id"], "spire-gym-run-observation-v1");
    assert_eq!(
        hex(&m9.fingerprint()),
        "0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999"
    );
    assert_eq!(m10.descriptor()["schema_id"], RUN_OBSERVATION_M10_SCHEMA_ID);
    assert_eq!(m10.descriptor()["schema_id"], "spire-gym-run-observation-v2");
    assert_eq!(
        hex(&m10.fingerprint()),
        "7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db"
    );
    assert_eq!(m12.descriptor()["schema_id"], RUN_OBSERVATION_M12_SCHEMA_ID);
    assert_eq!(m12.descriptor()["schema_id"], "spire-gym-m12-silent-act1-observation-v1");
    assert_eq!(
        hex(&m12.fingerprint()),
        "366c572af8141146995e849acc737a33484623790e2b3d661e290fadfb6c9fac"
    );
}

#[test]
fn m13_every_advertised_content_row_encodes_and_foreign_ids_fail_closed() {
    let caps = RunCapacities::m10_default();
    let mut decision = initial_m13_decision();

    for (row, id) in spire_core::run::M12_SILENT_ACT1_CARDS.iter().copied()
        .chain([CardId::Shiv, CardId::Dazed, CardId::Burn, CardId::Slimed, CardId::Wound])
        .enumerate()
    {
        decision.objects = vec![PublicObject { misc:0, instance: 1, id, upgrades: 0 }];
        let observation = encode_run_m13(&decision, None, &caps).unwrap();
        assert_eq!(group(&observation, "permanent_cards").values[0], row as i64 + 1, "{id:?}");
    }
    decision.objects = vec![PublicObject { misc:0, instance: 1, id: CardId::IronWave, upgrades: 0 }];
    assert!(matches!(encode_run_m13(&decision, None, &caps), Err(RunSessionError::Input(_))));
    decision.objects.clear();

    for (row, id) in spire_core::run::M12_SILENT_ACT1_RELICS.iter().copied().enumerate() {
        decision.player.relics = vec![id];
        let observation = encode_run_m13(&decision, None, &caps).unwrap();
        assert_eq!(group(&observation, "relics").values[0], row as i64 + 1, "{id:?}");
    }
    decision.player.relics = vec![RelicId::Brimstone];
    assert!(matches!(encode_run_m13(&decision, None, &caps), Err(RunSessionError::Input(_))));
    decision.player.relics.clear();

    for (row, id) in spire_core::run::M12_SILENT_ACT1_POTIONS.iter().copied().enumerate() {
        decision.player.potions = vec![Some(id)];
        let observation = encode_run_m13(&decision, None, &caps).unwrap();
        assert_eq!(group(&observation, "potions").values[0], row as i64 + 1, "{id:?}");
    }
    decision.player.potions = vec![Some(PotionId::BloodPotion)];
    assert!(matches!(encode_run_m13(&decision, None, &caps), Err(RunSessionError::Input(_))));
    decision.player.potions.clear();

    for (event, name) in [
        (EventId::Neow, "neow"),
        (EventId::UpgradeShrine, "upgrade_shrine"),
        (EventId::Purifier, "purifier"),
        (EventId::AncientWriting, "ancient_writing"),
        (EventId::Joust, "joust"),
    ] {
        decision.room.as_mut().unwrap().kind = RoomViewKind::Event(event);
        assert!(encode_run_m13(&decision, None, &caps).is_ok(), "event row {name}");
    }
    decision.room.as_mut().unwrap().kind = RoomViewKind::Event(EventId::Falling);
    assert!(matches!(encode_run_m13(&decision, None, &caps), Err(RunSessionError::Input(_))));
    decision.room.as_mut().unwrap().kind = RoomViewKind::Neow;

    let mut map = spire_core::run::map::PublicMap {
        boss: None, act: 2, nodes: Vec::new(), current: None, winged_moves: 0,
    };
    for encounter in ACT2_ENCOUNTERS {
        map.boss = Some(encounter);
        assert!(encode_run_m13(&decision, Some(&map), &caps).is_ok(), "encounter row {encounter:?}");
    }
    // Codec-only boundary fixture; the session transcript test owns natural readiness.
    let mut act3_ready = initial_m13_decision();
    act3_ready.act = 3;
    act3_ready.floor = 34;
    let room = act3_ready.room.as_mut().unwrap();
    room.kind = RoomViewKind::Act3Ready;
    room.choices.clear();
    act3_ready.candidates.clear();
    map.act = 3;
    for (encounter, numeric_id) in [
        (EncounterId::AwakenedOne, 25),
        (EncounterId::TimeEater, 26),
        (EncounterId::DonuAndDeca, 27),
    ] {
        map.boss = Some(encounter);
        let observation = encode_run_m13(&act3_ready, Some(&map), &caps).unwrap();
        assert_eq!(group(&observation, "context").values[11], numeric_id, "{encounter:?}");
    }
    map.boss = Some(EncounterId::Unknown("foreign_encounter"));
    assert!(matches!(encode_run_m13(&decision, Some(&map), &caps), Err(RunSessionError::Input(_))));

    let mut act3_choice = initial_m13_decision();
    act3_choice.room.as_mut().unwrap().choices.push(RoomChoiceView {
        id: ChoiceId(999),
        kind: RoomChoiceKind::Act3Continue { stage: 1 },
        price: None,
        status: RoomChoiceStatus::Offered,
    });
    assert!(matches!(encode_run_m13(&act3_choice, None, &caps), Err(RunSessionError::Input(_))));

    decision.objects = (1..=10)
        .map(|instance| PublicObject { misc:0, instance, id: CardId::StrikeGreen, upgrades: 0 })
        .collect();
    let setup_context = || SetupContext {
        player: PlayerState {
            hp: 70, max_hp: 70, block: 0, energy: 3, max_energy: 3,
            draw_per_turn: 5, powers: vec![],
        },
        player_powers: vec![],
        character: Character::Silent,
        cards: (1..=10)
            .map(|id| spire_core::combat::permanent_live_card(CardId::StrikeGreen, CardInstanceId(id), 0))
            .collect(),
        opening_draw: 5,
        encounter_visit: 0,
    };
    let mut rng = RunRng::new([37; 32], Partition::Test, 0);
    let setup = enemies::setup(EncounterId::SphericGuardian, 2, 0, &mut rng, setup_context()).unwrap();
    let mut inventory = Some(CombatItemContext::new(Vec::new(), vec![None; 4]));
    let combat = spire_core::combat::Combat::enter_with_item_context(setup, &mut inventory)
        .unwrap().public_full().unwrap();
    decision.combat = Some(combat);
    decision.room.as_mut().unwrap().kind = RoomViewKind::Combat;
    decision.candidates.clear();
    decision.offers.clear();
    for enemy in M13_ADDED_ACT2_ENEMIES {
        decision.combat.as_mut().unwrap().enemies[0].enemy_id = enemy;
        let encoded = encode_run_m13(&decision, None, &caps);
        assert!(encoded.is_ok(), "enemy row {enemy:?}: {encoded:?}");
    }
    decision.combat.as_mut().unwrap().enemies[0].enemy_id = EnemyId::Unknown("foreign_enemy");
    assert!(matches!(encode_run_m13(&decision, None, &caps), Err(RunSessionError::Input(_))));

    let setup = enemies::setup(EncounterId::ThreeCultists, 2, 0, &mut rng, setup_context()).unwrap();
    let mut inventory = Some(CombatItemContext::new(Vec::new(), vec![None; 4]));
    let combat = spire_core::combat::Combat::enter_with_item_context(setup, &mut inventory)
        .unwrap().public_full().unwrap();
    assert_eq!(combat.enemies.len(), 3);
    assert!(combat.enemies.iter().all(|enemy| enemy.enemy_id == EnemyId::Cultist));
    decision.combat = Some(combat);
    let observation = encode_run_m13(&decision, None, &caps).unwrap();
    let rows = group(&observation, "enemies");
    let columns = *rows.shape.last().unwrap();
    assert!((0..3).all(|row| rows.values[row * columns] == 1), "existing Cultist remains an encoded Act2 setup enemy");
}

// Catches a missing actual Act2 setup/summon/status/intent row or a numeric
// encoder ID that points at the wrong descriptor name. This is a codec
// diagnostic with supplied resources, never a normal-start completion.
#[test]
fn m13_actual_act2_setups_and_emitted_public_kinds_encode() {
    use spire_core::combat::{Combat, CombatAction, FullPowerId, FullPowerView};
    let caps = RunCapacities::m10_default();
    let descriptor = m13_run_observation_spec(&caps).unwrap().descriptor();
    let vocabulary = &descriptor["vocabularies"];
    let enemies = vocabulary["enemy"].as_array().unwrap();
    assert_eq!(enemies.len(), 38, "padding plus 16 inherited and 21 added identities");
    let mut act2_ids = M13_ADDED_ACT2_ENEMIES.to_vec();
    act2_ids.push(EnemyId::Cultist);
    act2_ids.sort_by_key(|id| id.as_str());
    act2_ids.dedup();
    assert_eq!(act2_ids.len(), 22, "actual Act2 union includes the retained Cultist");
    let index = |category: &str, name: &str| -> i64 {
        vocabulary[category].as_array().unwrap().iter()
            .position(|value| value == name).unwrap_or_else(|| panic!("missing {category} {name}")) as i64
    };
    let mut decision = initial_m13_decision();
    decision.act = 2;
    decision.floor = 18;
    decision.room.as_mut().unwrap().kind = RoomViewKind::Combat;
    decision.candidates.clear();
    decision.offers.clear();
    for encounter in ACT2_ENCOUNTERS {
        let cards = decision.objects.iter().map(|card|
            spire_core::combat::permanent_live_card(card.id, CardInstanceId(card.instance), card.upgrades)
        ).collect();
        let mut rng = RunRng::new([37; 32], Partition::Test, 0);
        let setup = enemies::setup(encounter, 2, 0, &mut rng, SetupContext {
            player: PlayerState { hp: 70, max_hp: 70, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![], character: Character::Silent, cards,
            opening_draw: 5, encounter_visit: 0,
        }).unwrap();
        let mut inventory = Some(CombatItemContext::new(vec![RelicId::RingOfTheSnake], vec![None; 3]));
        let mut combat = Combat::enter_with_item_context(setup, &mut inventory).unwrap();
        for _ in 0..3 {
            let public = combat.public_full().unwrap();
            assert!(public.enemies.iter().all(|enemy| act2_ids.contains(&enemy.enemy_id)));
            decision.combat = Some(public.clone());
            let encoded = encode_run_m13(&decision, None, &caps).unwrap_or_else(|error| panic!("{encounter:?}: {error:?}"));
            let enemy_rows = encoded.group("enemies").unwrap();
            let columns = *enemy_rows.shape.last().unwrap();
            for (row, enemy) in public.enemies.iter().enumerate() {
                assert_eq!(enemy_rows.values[row * columns], index("enemy", enemy.enemy_id.as_str()));
                assert_eq!(enemy_rows.values[row * columns + 4], index("intent", enemy.intent.as_str()));
            }
            let powers = encoded.group("powers").unwrap();
            for (row, power) in public.player.powers.iter().chain(public.enemies.iter().flat_map(|enemy| &enemy.powers)).enumerate() {
                assert_eq!(powers.values[row * 4 + 2], index("power", power.id.0));
                assert_eq!(powers.values[row * 4 + 3], i64::from(power.amount));
            }
            if public.outcome.is_some() { break; }
            combat.step_full(&CombatAction::EndTurn).unwrap();
        }
    }
    for name in ["artifact", "barricade", "flight", "hex", "confused", "minion", "angry", "entangled", "painful_stabs", "stasis", "noxious_fumes", "accuracy", "retain_cards"] {
        let combat = decision.combat.as_mut().unwrap();
        combat.player.powers = vec![FullPowerView { id: FullPowerId(name), amount: 2 }];
        let encoded = encode_run_m13(&decision, None, &caps).unwrap();
        assert_eq!(encoded.group("powers").unwrap().values[2], index("power", name));
    }
    decision.combat.as_mut().unwrap().player.powers[0].id = FullPowerId("foreign_power");
    assert!(matches!(encode_run_m13(&decision, None, &caps), Err(RunSessionError::Input(_))));
}
