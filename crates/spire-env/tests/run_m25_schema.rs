use serde_json::{json, Value};
use spire_core::{
    content::{watcher, CardColor, CardId, Catalog},
    run::m25_content,
};
use spire_env::*;

fn caps() -> RunCapacities {
    RunCapacities::m10_default()
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn names<T: AsRef<str>>(values: impl IntoIterator<Item = T>) -> Vec<Value> {
    std::iter::once(json!("padding"))
        .chain(values.into_iter().map(|value| json!(value.as_ref())))
        .collect()
}

fn has(vocabularies: &Value, key: &str, value: &str) -> bool {
    vocabularies[key]
        .as_array()
        .is_some_and(|rows| rows.iter().any(|row| row.as_str() == Some(value)))
}

#[test]
fn m25_descriptor_vocabulary_and_old_profiles() {
    let spec = m25_run_observation_spec(&caps()).unwrap();
    let d = spec.descriptor();
    let vocab = &d["vocabularies"];
    let m24 = m24_run_observation_spec(&caps()).unwrap();
    let m22 = m22_run_observation_spec(&caps()).unwrap();
    let m15 = m15_run_observation_spec(&caps()).unwrap();
    let m20 = m20_run_observation_spec(&caps()).unwrap();
    println!("BASE_M24_DESCRIPTOR_SHA256 {}", hex(m24.fingerprint()));
    println!("M25_DEFAULT_DESCRIPTOR_SHA256 {}", hex(spec.fingerprint()));
    assert_eq!(hex(spec.fingerprint()), "d1197df6ed3d786ffe0413ec88beeb33db44873c8032f6c67d926a109af93e78");
    let combat_state = d["tensors"].as_array().unwrap().iter()
        .find(|tensor| tensor["key"] == "combat_state").unwrap();
    assert_eq!(combat_state["fields"], json!([
        "facing", "surrounded", "heart_damage_remaining", "heart_cap", "previous_card_type"
    ]));
    assert_eq!(combat_state["units"][4], "category");
    assert_eq!(combat_state["columns"][4]["vocabulary"], "card_type");
    assert_eq!((combat_state["low"][4].as_i64(), combat_state["high"][4].as_i64()), (Some(0), Some(5)));

    // Existing public schemas are frozen; values are copied from the accepted
    // predecessor oracle at tests/m24_watcher_act3.rs:12-21 and its base probe.
    let policy_caps = combat_policy_capacities();
    let frozen = [
        ("m9", run_observation_spec(&RunCapacities::m9_default()).unwrap().fingerprint(), "0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999"),
        ("m10", m10_run_observation_spec(&caps()).unwrap().fingerprint(), "7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db"),
        ("m12", m12_run_observation_spec(&caps()).unwrap().fingerprint(), "366c572af8141146995e849acc737a33484623790e2b3d661e290fadfb6c9fac"),
        ("m13", m13_run_observation_spec(&caps()).unwrap().fingerprint(), "3f1e21d50106b8ca71d7cd67a2de7bfa4074455dbbb459629d8e2ac24cf39cf8"),
        ("m14", m14_run_observation_spec(&caps()).unwrap().fingerprint(), "50a685721e5db56ed7a5332160f3747087f96ce738da020415b1427159377aee"),
        ("m15", m15.fingerprint(), "254fb75b404fc558070468c6397324e68f962f23b33aca819b54fecf142ea11a"),
        ("m16", defect_combat_observation_spec(&policy_caps).unwrap().fingerprint(), "2b53f59fac1899e231300914b0b88aabc4e26add3492584de479b60e5fafce0a"),
        ("m17", m17_run_observation_spec(&caps()).unwrap().fingerprint(), "2176f835eca1384b2070f3d7d001158ef1fead23be5d6d7eea138b57f6bff999"),
        ("m18", m18_run_observation_spec(&caps()).unwrap().fingerprint(), "e097a8202f7cf7433f0ce57a39807bb74d370fba75a9aa84af65d726269564bb"),
        ("m19", m19_run_observation_spec(&caps()).unwrap().fingerprint(), "149092e11432e182920b5f07e0728dac45e880296815d9b84fba9b6180c2817b"),
        ("m20", m20.fingerprint(), "8c03211b4a3c589cfb55ba1078ddd447a444f3f53d8e9bfdab9ab85ab80e512d"),
        ("m21", watcher_combat_observation_spec(&policy_caps).unwrap().fingerprint(), "e7e49e88d50c947fde064a764153a5846ba3dc6c8298116f03e8126383b51e34"),
        ("m22", m22.fingerprint(), "5d1c77658141b2dfefbe95e17b85899ad6b77fb370e402bcbbe771146821e087"),
        ("m23", m23_run_observation_spec(&caps()).unwrap().fingerprint(), "5d5524ff7f60575dea528d42808f7a0a5e67c41bfd0e3b36f7f83b663c9f9d7e"),
        ("m24", m24.fingerprint(), "453b433051801b5cfbfd40bedbd70a8d2d53590e1ebffb4969c8cb17bc5b7564"),
        ("policy", policy_run_observation_spec(&caps()).unwrap().fingerprint(), "30905db1b7a520482d3ea8aa2e63ddb7f78b7c081de89d87268151f29cf4c7f8"),
        ("combat-v1", combat_policy_observation_spec().unwrap().fingerprint(), "02009686282c592f971eece3d5853eb54f2a482dde9dae1d0cf380ecdaf8fd78"),
        ("combat-v2", combat_policy_observation_spec_v2().unwrap().fingerprint(), "02433813e1297244060d9852f342c6eb4c9c4d62f3dde9763ff82191867eceec"),
    ];
    for (name, fingerprint, expected) in frozen {
        assert_eq!(hex(fingerprint), expected, "accepted predecessor fingerprint {name}");
    }

    let mut card_names = m25_content::M25_WATCHER_HEART_CARDS
        .iter()
        .chain(m25_content::GENERATED_ONLY)
        .chain(m25_content::STATUSES)
        .map(|id| id.as_str().to_owned())
        .collect::<Vec<_>>();
    for id in watcher::foreign_influence_support() {
        if !card_names.iter().any(|name| name == id.as_str()) {
            card_names.push(id.as_str().to_owned());
        }
    }
    assert_eq!(card_names.len(), 171);
    assert_eq!(card_names.len() - m25_content::M25_WATCHER_HEART_CARDS.len() - m25_content::GENERATED_ONLY.len() - m25_content::STATUSES.len(), 82);
    assert_eq!(vocab["card"].as_array().unwrap(), &names(card_names.iter()));
    assert_eq!(m25_content::M25_WATCHER_HEART_CARDS.len(), 76);
    assert_eq!(m25_content::GENERATED_ONLY.len(), 8);
    assert_eq!(m25_content::CHOICES.len(), 3);
    assert_eq!(vocab["wish_choice"].as_array().unwrap(), &names(m25_content::CHOICES.iter().map(|id| id.as_str())));
    assert_eq!(m25_content::STATUSES.len(), 5);
    assert_eq!(watcher::foreign_influence_support().len(), 106);
    assert_eq!(watcher::M25_WATCHER_POWER_IDS.len(), 12);
    assert_eq!(vocab["relic"].as_array().unwrap(), &names(m25_content::M25_WATCHER_HEART_RELICS.iter().map(|id| id.as_str())));
    assert_eq!(vocab["relic"].as_array().unwrap().len(), 24);
    assert_eq!(vocab["potion"].as_array().unwrap(), &names(m25_content::M25_WATCHER_HEART_POTIONS.iter().map(|id| id.as_str())));
    assert_eq!(vocab["potion"].as_array().unwrap().len(), 8);
    assert!(!has(vocab, "power", "focus"), "Hyperbeam's Focus loss is a player stat, not a FullPowerId row");
    let offer_tensor = d["tensors"].as_array().unwrap().iter()
        .find(|tensor| tensor["key"] == "combat_card_offers").unwrap();
    assert_eq!(offer_tensor["fields"][0], json!("card_kind_or_wish_choice"));
    assert_eq!(offer_tensor["columns"][0]["low"], json!(-3));
    assert_eq!(offer_tensor["columns"][0]["high"], json!(vocab["card"].as_array().unwrap().len() - 1));

    let finite = &d["semantics"]["finite_content"];
    assert_eq!(finite["choice_cards"], json!(m25_content::CHOICES.iter().map(|id| id.as_str()).collect::<Vec<_>>()));
    assert_eq!(finite["playable_encounters"].as_array().unwrap().len(), 42);
    assert_eq!(finite["playable_enemies"].as_array().unwrap().len(), 57);
    assert_eq!(vocab["encounter"].as_array().unwrap().len(), 43);
    assert_eq!(vocab["enemy"].as_array().unwrap().len(), 58);
    assert_eq!(finite["relic_taxonomy"].as_array().unwrap().len(), 23);
    assert_eq!(
        &finite["relic_taxonomy"].as_array().unwrap()[..18],
        &m22.descriptor()["semantics"]["finite_content"]["relic_taxonomy"].as_array().unwrap()[..18]
    );
    assert_eq!(
        &finite["relic_taxonomy"].as_array().unwrap()[..19],
        &m24.descriptor()["semantics"]["finite_content"]["relic_taxonomy"].as_array().unwrap()[..]
    );
    let mut expected_taxonomy = m24.descriptor()["semantics"]["finite_content"]["relic_taxonomy"]
        .as_array().unwrap().clone();
    expected_taxonomy.extend(m25_content::RELIC_TAXONOMY_ADDITIONS.iter().map(|(id, tier)| json!({"id":id.as_str(),"tier":tier})));
    assert_eq!(finite["relic_taxonomy"].as_array().unwrap(), &expected_taxonomy);

    for name in m15.descriptor()["semantics"]["act4"]["intents"].as_array().unwrap().iter().filter_map(Value::as_str) {
        assert!(has(vocab, "intent", name), "missing accepted Act4 intent {name}");
    }
    for name in m15.descriptor()["semantics"]["act4"]["powers"].as_array().unwrap().iter().filter_map(Value::as_str) {
        assert!(has(vocab, "power", name), "missing accepted Act4 power {name}");
    }
    for name in watcher::M25_WATCHER_POWER_IDS {
        assert!(has(vocab, "power", name), "missing owner power {name}");
    }
    for name in ["lose_dexterity", "plated_armor"] {
        assert!(has(vocab, "power", name), "missing M25 reused power {name}");
        assert!(m20.descriptor()["vocabularies"]["power"].as_array().unwrap().contains(&json!(name)),
            "M25 reused power {name} must have accepted M20 descriptor membership");
    }
    let c5_off_colour = watcher::foreign_influence_support().iter().copied()
        .filter(|id| Catalog::standard().card(*id).is_some_and(|row| row.color != CardColor::Purple))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(c5_off_colour.len(), 82, "HC1 C5's exhaustive playable table is the 82 off-colour output set");
    let c5_power_producers = [
        ("artifact", CardId::CoreSurge),
        ("choked", CardId::Choke),
        ("draw_card_next_turn", CardId::Predator),
        ("energized", CardId::FlyingKnee),
        ("lock_on", CardId::Bullseye),
        ("poison", CardId::PoisonedStab),
        ("rebound", CardId::Rebound),
        ("vulnerable", CardId::Uppercut),
        ("weak", CardId::GoForTheEyes),
    ];
    for (power, producer) in c5_power_producers {
        assert!(c5_off_colour.contains(&producer), "C5 power row {power} producer {producer:?} must be one of the 82 playable off-colour outputs");
        assert!(has(vocab, "power", power), "missing Foreign Influence output power {power}");
        assert!(m15.descriptor()["vocabularies"]["power"].as_array().unwrap().contains(&json!(power))
            || m20.descriptor()["vocabularies"]["power"].as_array().unwrap().contains(&json!(power)),
            "Foreign Influence power {power} must have accepted M15/M20 descriptor membership");
    }
    assert!(c5_off_colour.contains(&CardId::Hyperbeam), "Hyperbeam's Focus loss is in the 82-output playability set");
    let player_fields = d["tensors"].as_array().unwrap().iter()
        .find(|tensor| tensor["key"] == "player").unwrap()["fields"].as_array().unwrap();
    assert!(player_fields.iter().any(|field| field.as_str() == Some("focus")),
        "Hyperbeam's Focus stat uses the public player field, not an appended power ID");
    let c5_selection_producers = [
        ("topdeck", CardId::Headbutt),
        ("move_discard", CardId::DaggerThrow),
    ];
    for (operation, producer) in c5_selection_producers {
        assert!(c5_off_colour.contains(&producer), "C5 selection row {operation} producer {producer:?} must be one of the 82 playable off-colour outputs");
        assert!(has(vocab, "selection_operation", operation), "missing Foreign Influence output selection operation {operation}");
        assert!(m15.descriptor()["vocabularies"]["selection_operation"].as_array().unwrap().contains(&json!(operation)),
            "Foreign Influence selection operation {operation} must have accepted M15 descriptor membership");
    }
    assert!(has(vocab, "selection_operation", "omniscience"), "missing HC0 Omniscience selection operation");
    for id in ["shield_and_spear", "corrupt_heart"] {
        assert!(has(vocab, "encounter", id), "missing Act4 encounter {id}");
    }
    for id in ["spire_shield", "spire_spear", "corrupt_heart"] {
        assert!(has(vocab, "enemy", id), "missing Act4 enemy {id}");
    }
    for id in ["heart_victory", "true_victory"] {
        assert!(has(vocab, "room", id), "missing terminal room {id}");
    }
    assert!(has(vocab, "terminal", "heart_victory"));
    for class_only in ["buffer", "creative_ai"] {
        assert!(vocab["power"].as_array().unwrap().iter().all(|row| row.as_str() != Some(class_only)), "class-only power {class_only} is not inherited");
    }
    assert_ne!(spec.fingerprint(), m24.fingerprint());
    assert_eq!(hex(m24.fingerprint()), "453b433051801b5cfbfd40bedbd70a8d2d53590e1ebffb4969c8cb17bc5b7564");
    assert_eq!(RUN_OBSERVATION_M25_SCHEMA_ID, "spire-gym-m25-watcher-heart-observation-v1");
    assert_eq!(d["profile_id"], "M25WatcherHeart");
    let _ = m20;
}
