use spire_core::{
    combat::{enemies::{self, SetupContext}, rng::Partition, CardInstanceId, Combat, CombatAction, FullPowerId, FullPowerView, PlayerState, SelectionDisposition, SelectionOperation, TargetId},
    content::{CardId, Character, EncounterId, EnemyId, EventId, PotionId, RelicId},
    items::CombatItemContext,
    run::{
        Candidate, ChoiceId, CreationProfile, GameAction, PublicObject, RoomChoiceKind,
        RoomChoiceStatus, RoomChoiceView, RoomScenario, RoomViewKind, Run, RunConfig, RunRng,
        M14_SILENT_ACT3_CARDS, M14_SILENT_ACT3_POTIONS, M14_SILENT_ACT3_RELICS,
    },
};
use spire_env::{
    encode_run_m14, m10_run_observation_spec, m12_run_observation_spec, m13_run_observation_spec,
    m14_run_observation_spec, run_observation_spec, RunCapacities, RunSessionError,
    RUN_OBSERVATION_M10_SCHEMA_ID, RUN_OBSERVATION_M12_SCHEMA_ID, RUN_OBSERVATION_M13_SCHEMA_ID,
    RUN_OBSERVATION_M14_SCHEMA_ID, RUN_OBSERVATION_SCHEMA_ID,
};

const ACT3_ENCOUNTERS: [EncounterId; 16] = [
    EncounterId::OrbWalker, EncounterId::ThreeShapes, EncounterId::SpireGrowth,
    EncounterId::Transient, EncounterId::FourShapes, EncounterId::Maw,
    EncounterId::SphereAndTwoShapes, EncounterId::JawWormHorde, EncounterId::ThreeDarklings,
    EncounterId::WrithingMass, EncounterId::GiantHead, EncounterId::Nemesis,
    EncounterId::Reptomancer, EncounterId::AwakenedOne, EncounterId::TimeEater,
    EncounterId::DonuAndDeca,
];
const ACT3_SPECIES: [&str; 20] = [
    "spheric_guardian", "jaw_worm", "cultist", "orb_walker", "repulsor", "exploder",
    "spiker", "darkling", "spire_growth", "transient", "maw", "writhing_mass",
    "giant_head", "nemesis", "reptomancer", "dagger", "awakened_one", "time_eater",
    "deca", "donu",
];
const NEW_ACT3_SPECIES: [&str; 17] = [
    "orb_walker", "repulsor", "exploder", "spiker", "darkling", "spire_growth",
    "transient", "maw", "writhing_mass", "giant_head", "nemesis", "reptomancer",
    "dagger", "awakened_one", "time_eater", "deca", "donu",
];

fn initial_m14_decision() -> spire_core::run::PublicDecision {
    let mut creation = CreationProfile::experienced();
    creation.neow_eligible = false;
    Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 },
        creation,
        RunRng::new([31; 32], Partition::Test, 0),
        RoomScenario::M14SilentAct3,
    ).unwrap().decision().unwrap()
}

fn group<'a>(observation: &'a spire_env::RunObservation, key: &str) -> &'a spire_env::RunFeatureGroup {
    observation.group(key).unwrap()
}

fn row(vocab: &serde_json::Value, category: &str, name: &str) -> i64 {
    vocab[category].as_array().unwrap().iter().position(|value| value == name)
        .unwrap_or_else(|| panic!("missing {category} vocabulary row {name}")) as i64
}

fn cap_error(
    decision: spire_core::run::PublicDecision,
    map: Option<&spire_core::run::map::PublicMap>,
    set: impl FnOnce(&mut RunCapacities),
    expected_group: &'static str,
) {
    let mut caps = RunCapacities::m10_default();
    set(&mut caps);
    assert!(matches!(
        encode_run_m14(&decision, map, &caps),
        Err(RunSessionError::Capacity { group, .. }) if group == expected_group
    ), "expected fail-closed capacity for {expected_group}; decision={:?}", decision.room);
}

fn act3_native_combat(
    encounter: EncounterId,
    decision: &spire_core::run::PublicDecision,
) -> Combat {
    let cards = if decision.objects.is_empty() {
        M14_SILENT_ACT3_CARDS.iter().take(10).enumerate().map(|(index, id)|
            spire_core::combat::permanent_live_card(*id, CardInstanceId(index as u64 + 1), 0)
        ).collect()
    } else {
        decision.objects.iter().map(|card|
            spire_core::combat::permanent_live_card(card.id, CardInstanceId(card.instance), card.upgrades)
        ).collect()
    };
    let mut rng = RunRng::new([37; 32], Partition::Test, 0);
    let setup = enemies::setup(encounter, 3, 0, &mut rng, SetupContext {
        player: PlayerState {
            hp: 70, max_hp: 70, block: 0, energy: 3, max_energy: 3,
            draw_per_turn: 5, powers: vec![],
        },
        player_powers: vec![],
        character: Character::Silent,
        cards,
        opening_draw: 5,
        encounter_visit: 0,
    }).unwrap();
    let mut inventory = Some(CombatItemContext::new(
        vec![RelicId::RingOfTheSnake],
        vec![None; 4],
    ));
    Combat::enter_with_item_context(setup, &mut inventory).unwrap()
}

fn act3_combat(
    encounter: EncounterId,
    decision: &spire_core::run::PublicDecision,
) -> spire_core::combat::CombatPublic {
    act3_native_combat(encounter, decision).public_full().unwrap()
}

#[test]
fn m14_schema_is_finite_and_additive_to_accepted_run_profiles() {
    let caps = RunCapacities::m10_default();
    let descriptor = m14_run_observation_spec(&caps).unwrap().descriptor();
    assert_eq!(descriptor["schema_id"], RUN_OBSERVATION_M14_SCHEMA_ID);
    assert_eq!(descriptor["schema_id"], "spire-gym-m14-silent-act3-observation-v1");
    assert_eq!(descriptor["profile_id"], "M14SilentAct3");
    let vocab = &descriptor["vocabularies"];

    assert_eq!(vocab["card"].as_array().unwrap().len(), 27, "padding + 20 permanent + 6 generated/status IDs");
    for id in M14_SILENT_ACT3_CARDS {
        assert!(vocab["card"].as_array().unwrap().iter().any(|row| row == id.as_str()), "permanent {id:?}");
    }
    for id in ["shiv", "dazed", "burn", "slimed", "wound", "void"] {
        assert!(vocab["card"].as_array().unwrap().iter().any(|row| row == id), "generated/status {id}");
    }
    assert!(!vocab["card"].as_array().unwrap().iter().any(|row| row == "sludge"));
    assert_eq!(vocab["relic"].as_array().unwrap().len(), 19);
    for id in M14_SILENT_ACT3_RELICS {
        assert!(vocab["relic"].as_array().unwrap().iter().any(|row| row == id.as_str()), "relic {id:?}");
    }
    assert_eq!(vocab["potion"], serde_json::json!([
        "padding", "fire_potion", "strength_potion", "weak_potion", "dexterity_potion"
    ]));
    assert_eq!(M14_SILENT_ACT3_POTIONS.len(), 4);
    assert_eq!(vocab["event"], serde_json::json!([
        "padding", "neow", "upgrade_shrine", "purifier", "ancient_writing", "joust",
        "falling", "tomb_of_lord_red_mask"
    ]));
    assert_eq!(vocab["terminal"], serde_json::json!(["ongoing", "death", "empty_deck", "act3_no_keys"]));
    assert_eq!(vocab["room"].as_array().unwrap().last().unwrap(), "act4_ready");
    assert!(!vocab["room"].as_array().unwrap().iter().any(|row| row == "heart_victory" || row == "true_victory"));
    assert!(vocab["choice"].as_array().unwrap().iter().any(|row| row == "act3_continue"));

    let declared_act3 = descriptor["semantics"]["encounters"]["act3"].as_array().unwrap();
    assert_eq!(declared_act3.len(), 16);
    for encounter in ACT3_ENCOUNTERS {
        assert!(declared_act3.iter().any(|row| row == encounter.as_str()), "Act3 encounter {encounter:?}");
        assert!(vocab["encounter"].as_array().unwrap().iter().any(|row| row == encounter.as_str()));
    }
    let act3_species = descriptor["semantics"]["act3_enemy_species"]["species"].as_array().unwrap();
    assert_eq!(act3_species.len(), 20);
    assert_eq!(act3_species.iter().map(|row| row.as_str().unwrap()).collect::<Vec<_>>(), ACT3_SPECIES);
    let additions = descriptor["semantics"]["act3_enemy_species"]["new_to_m13"].as_array().unwrap();
    assert_eq!(additions.iter().map(|row| row.as_str().unwrap()).collect::<Vec<_>>(), NEW_ACT3_SPECIES);
    let cumulative = descriptor["semantics"]["act3_enemy_species"]["cumulative_through_act3"].as_array().unwrap();
    assert_eq!(cumulative.len(), 54);
    let mut unique = cumulative.iter().map(|row| row.as_str().unwrap()).collect::<Vec<_>>();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), 54, "cumulative species are an explicit deduplicated union");
    for species in ACT3_SPECIES {
        assert!(cumulative.iter().any(|row| row == species), "Act3 setup species {species}");
        assert!(vocab["enemy"].as_array().unwrap().iter().any(|row| row == species));
    }
    assert!(descriptor["semantics"]["encounters"]["map_only_boss_icons"].as_array().unwrap().is_empty());
    assert_eq!(descriptor["semantics"]["sludge"], "awa_sludge creates Void at BottomDraw; there is no Sludge CardId");
    for hidden in ["draw_queue", "rng", "boss_history", "private_queue", "combat_state"] {
        assert!(descriptor.to_string().find(hidden).is_none(), "schema must not expose {hidden}");
    }

    let m9 = run_observation_spec(&caps).unwrap();
    let m10 = m10_run_observation_spec(&caps).unwrap();
    let m12 = m12_run_observation_spec(&caps).unwrap();
    let m13 = m13_run_observation_spec(&caps).unwrap();
    assert_eq!(m9.descriptor()["schema_id"], RUN_OBSERVATION_SCHEMA_ID);
    assert_eq!(m9.descriptor()["schema_id"], "spire-gym-run-observation-v1");
    assert_eq!(m9.fingerprint().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999");
    assert_eq!(m10.descriptor()["schema_id"], RUN_OBSERVATION_M10_SCHEMA_ID);
    assert_eq!(m10.fingerprint().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db");
    assert_eq!(m12.descriptor()["schema_id"], RUN_OBSERVATION_M12_SCHEMA_ID);
    assert_eq!(m12.fingerprint().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "366c572af8141146995e849acc737a33484623790e2b3d661e290fadfb6c9fac");
    assert_eq!(m13.descriptor()["schema_id"], RUN_OBSERVATION_M13_SCHEMA_ID);
    assert_eq!(m13.fingerprint().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "3f1e21d50106b8ca71d7cd67a2de7bfa4074455dbbb459629d8e2ac24cf39cf8");
    assert_eq!(m13, m13_run_observation_spec(&caps).unwrap(), "M14 extension leaves the accepted M13 schema bytes and fingerprint stable");
}

#[test]
fn m14_every_finite_row_maps_to_its_descriptor_name_and_foreign_rows_reject() {
    let caps = RunCapacities::m10_default();
    let descriptor = m14_run_observation_spec(&caps).unwrap().descriptor();
    let vocab = &descriptor["vocabularies"];
    let mut decision = initial_m14_decision();
    for id in M14_SILENT_ACT3_CARDS.iter().copied().chain([
        CardId::Shiv, CardId::Dazed, CardId::Burn, CardId::Slimed, CardId::Wound, CardId::Void,
    ]) {
        decision.objects = vec![PublicObject { instance: 1, id, upgrades: 0 }];
        let encoded = encode_run_m14(&decision, None, &caps).unwrap();
        assert_eq!(group(&encoded, "permanent_cards").values[0], row(vocab, "card", id.as_str()), "{id:?}");
    }
    decision.objects = vec![PublicObject { instance: 1, id: CardId::IronWave, upgrades: 0 }];
    assert!(matches!(encode_run_m14(&decision, None, &caps), Err(RunSessionError::Input(_))));
    decision.objects.clear();

    for id in M14_SILENT_ACT3_RELICS.iter().copied() {
        decision.player.relics = vec![id];
        let encoded = encode_run_m14(&decision, None, &caps).unwrap();
        assert_eq!(group(&encoded, "relics").values[0], row(vocab, "relic", id.as_str()), "{id:?}");
    }
    decision.player.relics = vec![RelicId::Brimstone];
    assert!(matches!(encode_run_m14(&decision, None, &caps), Err(RunSessionError::Input(_))));
    decision.player.relics.clear();

    for id in M14_SILENT_ACT3_POTIONS.iter().copied() {
        decision.player.potions = vec![Some(id)];
        let encoded = encode_run_m14(&decision, None, &caps).unwrap();
        assert_eq!(group(&encoded, "potions").values[0], row(vocab, "potion", id.as_str()), "{id:?}");
    }
    decision.player.potions = vec![Some(PotionId::BloodPotion)];
    assert!(matches!(encode_run_m14(&decision, None, &caps), Err(RunSessionError::Input(_))));
    decision.player.potions.clear();

    for (id, name) in [
        (EventId::Neow, "neow"),
        (EventId::UpgradeShrine, "upgrade_shrine"),
        (EventId::Purifier, "purifier"),
        (EventId::AncientWriting, "ancient_writing"),
        (EventId::Joust, "joust"),
        (EventId::Falling, "falling"),
        (EventId::TombOfLordRedMask, "tomb_of_lord_red_mask"),
    ] {
        decision.room.as_mut().unwrap().kind = RoomViewKind::Event(id);
        let encoded = encode_run_m14(&decision, None, &caps).unwrap();
        assert_eq!(group(&encoded, "context").values[5], row(vocab, "event", name), "{id:?}");
    }
    decision.room.as_mut().unwrap().kind = RoomViewKind::Event(EventId::Unknown("foreign_event"));
    assert!(matches!(encode_run_m14(&decision, None, &caps), Err(RunSessionError::Input(_))));
    decision.room.as_mut().unwrap().kind = RoomViewKind::Neow;

    for id in ACT3_ENCOUNTERS {
        let map = spire_core::run::map::PublicMap {
            boss: Some(id), act: 3, nodes: vec![], current: None, winged_moves: 0,
        };
        let encoded = encode_run_m14(&decision, Some(&map), &caps).unwrap();
        assert_eq!(group(&encoded, "context").values[11], row(vocab, "encounter", id.as_str()), "{id:?}");
    }
    let map = spire_core::run::map::PublicMap {
        boss: Some(EncounterId::Unknown("foreign_encounter")), act: 3,
        nodes: vec![], current: None, winged_moves: 0,
    };
    assert!(matches!(encode_run_m14(&decision, Some(&map), &caps), Err(RunSessionError::Input(_))));

    decision.objects = M14_SILENT_ACT3_CARDS.iter().take(10).enumerate().map(|(index, id)|
        PublicObject { instance: index as u64 + 1, id: *id, upgrades: 0 }
    ).collect();
    decision.combat = Some(act3_combat(EncounterId::ThreeShapes, &decision));
    decision.room.as_mut().unwrap().kind = RoomViewKind::Combat;
    decision.candidates.clear();
    decision.offers.clear();
    let descriptor_enemy = vocab["enemy"].as_array().unwrap();
    for encounter in ACT3_ENCOUNTERS {
        let public = act3_combat(encounter, &decision);
        let encoded = encode_run_m14(&spire_core::run::PublicDecision { combat: Some(public.clone()), ..decision.clone() }, None, &caps)
            .unwrap_or_else(|error| panic!("actual Act3 setup {encounter:?} must encode: {error:?}"));
        let enemies = group(&encoded, "enemies");
        let columns = *enemies.shape.last().unwrap();
        for (index, actual) in public.enemies.iter().enumerate() {
            let expected = descriptor_enemy.iter().position(|value| value == actual.enemy_id.as_str()).unwrap() as i64;
            assert_eq!(enemies.values[index * columns], expected, "{encounter:?} actor {:?}", actual.enemy_id);
            let intent = row(vocab, "intent", actual.intent.as_str());
            assert_eq!(enemies.values[index * columns + 4], intent, "{encounter:?} intent {:?}", actual.intent);
        }
        let powers = group(&encoded, "powers");
        for (index, power) in public.player.powers.iter().chain(public.enemies.iter().flat_map(|enemy| &enemy.powers)).enumerate() {
            assert_eq!(powers.values[index * 4 + 2], row(vocab, "power", power.id.0), "{encounter:?} power {}", power.id.0);
        }
    }
    let mut foreign = decision.clone();
    foreign.combat.as_mut().unwrap().enemies[0].enemy_id = EnemyId::Unknown("foreign_enemy");
    assert!(matches!(encode_run_m14(&foreign, None, &caps), Err(RunSessionError::Input(_))));

    let mut ending = initial_m14_decision();
    ending.room.as_mut().unwrap().kind = RoomViewKind::Act3Ending;
    ending.room.as_mut().unwrap().choices.clear();
    ending.candidates.clear();
    ending.room.as_mut().unwrap().choices.push(RoomChoiceView {
        id: ChoiceId(999), kind: RoomChoiceKind::Act3Continue { stage: 1 }, price: None,
        status: RoomChoiceStatus::Offered,
    });
    let encoded = encode_run_m14(&ending, None, &caps).unwrap();
    assert_eq!(group(&encoded, "offers").values[0], row(vocab, "choice", "act3_continue"));
    ending.room.as_mut().unwrap().kind = RoomViewKind::HeartVictory;
    assert!(matches!(encode_run_m14(&ending, None, &caps), Err(RunSessionError::UnsupportedBoundary(_))));
    ending.room.as_mut().unwrap().kind = RoomViewKind::Act3Ending;
    ending.terminal = Some(spire_core::run::RunTerminal::HeartVictory);
    assert!(matches!(encode_run_m14(&ending, None, &caps), Err(RunSessionError::UnsupportedBoundary(_))));
}

#[test]
fn m14_actual_act3_discard_selection_uses_finite_shared_rows() {
    let caps = RunCapacities::m10_default();
    let vocab = m14_run_observation_spec(&caps).unwrap().descriptor()["vocabularies"].clone();
    let cards = [
        CardId::DaggerThrow,
        CardId::Reflex,
        CardId::StrikeGreen,
        CardId::DefendGreen,
        CardId::Neutralize,
    ];
    let mut decision = initial_m14_decision();
    decision.objects = cards.iter().enumerate().map(|(index, id)| PublicObject {
        instance: index as u64 + 1,
        id: *id,
        upgrades: 0,
    }).collect();
    let mut combat = act3_native_combat(EncounterId::ThreeShapes, &decision);
    let dagger = combat.public_full().unwrap().hand.iter()
        .find(|card| card.id == CardId::DaggerThrow).unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: dagger, target: Some(TargetId(1)) }).unwrap();
    let public = combat.public_full().unwrap();
    let selection = public.selection.as_ref().unwrap();
    assert_eq!(selection.operation, SelectionOperation::Move(SelectionDisposition::Discard));

    decision.room = None;
    decision.offers.clear();
    decision.combat = Some(public.clone());
    decision.candidates = combat.legal_actions_full().unwrap().into_iter().enumerate().map(|(index, action)| Candidate {
        id: ChoiceId(900 + index as u64),
        action: GameAction::Combat(action),
        label: "actual Act3 Dagger Throw selection",
        neow_offer: None,
        object: None,
    }).collect();
    let encoded = encode_run_m14(&decision, None, &caps).unwrap();
    assert_eq!(group(&encoded, "selection_counts").values[5], row(&vocab, "selection_operation", "move_discard"));
    let combat_rows = group(&encoded, "combat_cards").values.chunks(18).collect::<Vec<_>>();
    let selection_rows = group(&encoded, "selection").values.chunks(4).collect::<Vec<_>>();
    for candidate in &selection.candidates {
        let permanent = candidate.instance.0 as i64 - 1;
        let combat_row = combat_rows.iter().position(|row| row[5] == permanent).unwrap() as i64;
        let selected_row = selection_rows.iter().find(|row| row[1] == combat_row).unwrap();
        assert_eq!(&selected_row[0..3], &[4, combat_row, 1]);
        assert_eq!(selected_row[3], 0);
    }
}

#[test]
fn m14_encoder_rejects_act4_gameplay_but_keeps_the_supplied_act4ready_cutoff() {
    let caps = RunCapacities::m10_default();
    let mut decision = initial_m14_decision();

    // An otherwise finite Silent decision must not cross into Act4 Map gameplay.
    decision.act = 4;
    decision.room.as_mut().unwrap().kind = RoomViewKind::Map;
    assert!(matches!(
        encode_run_m14(&decision, None, &caps),
        Err(RunSessionError::UnsupportedBoundary("M14 cannot encode Act4 gameplay"))
    ), "Act4 Map is outside the M14 endpoint even when its public resources are finite");

    // Reject map metadata on the same boundary too, even with no boss or enemies.
    decision.act = 3;
    decision.room.as_mut().unwrap().kind = RoomViewKind::Map;
    let foreign_act4_map = spire_core::run::map::PublicMap {
        boss: None,
        act: 4,
        nodes: vec![],
        current: None,
        winged_moves: 0,
    };
    assert!(matches!(
        encode_run_m14(&decision, Some(&foreign_act4_map), &caps),
        Err(RunSessionError::UnsupportedBoundary("M14 cannot encode Act4 gameplay"))
    ), "Act4 map metadata cannot pass by omitting enemy rows");

    // The supplied healthy diagnostic remains within the act3 endpoint.
    decision.act = 3;
    decision.floor = 50;
    let room = decision.room.as_mut().unwrap();
    room.kind = RoomViewKind::Act4Ready;
    room.choices.clear();
    decision.candidates.clear();
    let observation = encode_run_m14(&decision, None, &caps).unwrap();
    let vocab = m14_run_observation_spec(&caps).unwrap().descriptor();
    assert_eq!(group(&observation, "context").values[2], 3);
    assert_eq!(group(&observation, "context").values[3], 50);
    assert_eq!(group(&observation, "context").values[4], row(&vocab["vocabularies"], "room", "act4_ready"));
    assert!(observation.action_mask.iter().all(|value| *value == 0));
}

#[test]
fn m14_shared_row_axes_fail_closed_at_capacity() {
    let decision = initial_m14_decision();
    let map_nodes = spire_core::run::map::PublicMap {
        boss: None, act: 1, current: None, winged_moves: 0,
        nodes: vec![
            spire_core::run::map::PublicMapNode { id: spire_core::run::NodeId(1), floor: 1, x: 0, y: 0,
                kind: spire_core::run::MapNodeKind::Monster, resolved_room: None, edges: vec![], visited: false, revealed: true, burning: None },
            spire_core::run::map::PublicMapNode { id: spire_core::run::NodeId(2), floor: 2, x: 1, y: 0,
                kind: spire_core::run::MapNodeKind::Monster, resolved_room: None, edges: vec![], visited: false, revealed: true, burning: None },
        ],
    };
    cap_error(decision.clone(), Some(&map_nodes), |caps| caps.map_nodes = 1, "map_nodes");

    let mut map_edges = map_nodes.clone();
    map_edges.nodes[0].edges = vec![spire_core::run::NodeId(2), spire_core::run::NodeId(2)];
    cap_error(decision.clone(), Some(&map_edges), |caps| caps.map_edges = 1, "map_edges");

    let mut permanent = decision.clone();
    permanent.objects.push(PublicObject { instance: 999, id: CardId::StrikeGreen, upgrades: 0 });
    cap_error(permanent, None, |caps| caps.permanent_cards = 1, "permanent_cards");

    let mut combat = decision.clone();
    combat.combat = Some(act3_combat(EncounterId::ThreeShapes, &decision));
    combat.room.as_mut().unwrap().kind = RoomViewKind::Combat;
    combat.candidates.clear();
    combat.offers.clear();
    cap_error(combat.clone(), None, |caps| caps.combat_cards = 1, "combat_cards");
    cap_error(combat.clone(), None, |caps| caps.enemies = 1, "enemies");
    combat.combat.as_mut().unwrap().player.powers = vec![
        FullPowerView { id: FullPowerId("strength"), amount: 1 },
        FullPowerView { id: FullPowerId("dexterity"), amount: 1 },
    ];
    cap_error(combat, None, |caps| caps.powers = 1, "powers");

    let mut relics = decision.clone();
    relics.player.relics = vec![RelicId::RingOfTheSnake, RelicId::Lantern];
    cap_error(relics, None, |caps| caps.relics = 1, "relics");
    let mut potions = decision.clone();
    potions.player.potions = vec![Some(PotionId::FirePotion), Some(PotionId::WeakPotion)];
    cap_error(potions, None, |caps| caps.potions = 1, "potions");
    cap_error(decision.clone(), None, |caps| caps.offers = 1, "offers");

    let mut selection = decision.clone();
    selection.selection = Some(spire_core::run::PublicSelection {
        required: 2, selected_count: 0,
        eligible: vec![
            PublicObject { instance: 11, id: CardId::StrikeGreen, upgrades: 0 },
            PublicObject { instance: 12, id: CardId::DefendGreen, upgrades: 0 },
        ],
    });
    cap_error(selection, None, |caps| caps.selection = 1, "selection");

    let mut actions = decision;
    actions.candidates = vec![
        Candidate { id: ChoiceId(801), action: GameAction::Confirm, label: "test", neow_offer: None, object: None },
        Candidate { id: ChoiceId(802), action: GameAction::Cancel, label: "test", neow_offer: None, object: None },
    ];
    cap_error(actions, None, |caps| caps.actions = 1, "actions");
}
