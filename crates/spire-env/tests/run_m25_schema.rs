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
    assert_eq!(hex(spec.fingerprint()), "a1e637bf29fc28620a9c2e487cfd950462d8680967eb68bb9620e3a38bc5ca67");
    let combat_state = d["tensors"].as_array().unwrap().iter()
        .find(|tensor| tensor["key"] == "combat_state").unwrap();
    assert_eq!(combat_state["fields"], json!([
        "facing", "surrounded", "heart_damage_remaining", "heart_cap", "previous_card_type", "phase", "selection_resume_phase"
    ]));
    assert_eq!(combat_state["units"][4], "category");
    assert_eq!(combat_state["columns"][4]["vocabulary"], "card_type");
    assert_eq!((combat_state["low"][4].as_i64(), combat_state["high"][4].as_i64()), (Some(0), Some(5)));

    // Existing public schemas are frozen; values are copied from the accepted
    // predecessor oracle at tests/m24_watcher_act3.rs:12-21 and its base probe.
    let policy_caps = combat_policy_capacities();
    let frozen = [
        ("m9", run_observation_spec(&RunCapacities::m9_default()).unwrap().fingerprint(), "270163f700c7ca19556898d14c22ca8d641b0ef6f5c757d80d0ca7983989981c"),
        ("m10", m10_run_observation_spec(&caps()).unwrap().fingerprint(), "dccc7624c45898bc33e3fa682b620fd78ce70fe558d6db7624a888a7ed068c21"),
        ("m12", m12_run_observation_spec(&caps()).unwrap().fingerprint(), "ae0b995eade57ef1dccc39bd9cad6064d8bd33a1cfbc37a312e4b76ac01ceb20"),
        ("m13", m13_run_observation_spec(&caps()).unwrap().fingerprint(), "cd7f78f7bc8ae8566f727f431f8a8d840303cdcb556173b31bf169cea8c34ecf"),
        ("m14", m14_run_observation_spec(&caps()).unwrap().fingerprint(), "72e75d8357ebbbba2b905f381262a99a4385c670d9fa20fa8cd008f4b90fb827"),
        ("m15", m15.fingerprint(), "ab9fd38b68e7bd276526e136caf58fbbf6f0ef5916eb67fcba20fd91b54004f1"),
        ("m16", defect_combat_observation_spec(&policy_caps).unwrap().fingerprint(), "feeffa64c463aee521944f9a85e53446289226fd18518a45c5ebb72581236ef2"),
        ("m17", m17_run_observation_spec(&caps()).unwrap().fingerprint(), "eeb45fcedd185710aeb0e5cb58ca619cc5927861cebda39b1612ff1500887a92"),
        ("m18", m18_run_observation_spec(&caps()).unwrap().fingerprint(), "6a4bb8526357f0264ae0cad8ccaad8015a0a3d360a678d236d94afe21a4ac967"),
        ("m19", m19_run_observation_spec(&caps()).unwrap().fingerprint(), "b2aa0b85a83a51171524a7487ab7cb3ed450e00811bd7f00c472e358a0220b15"),
        ("m20", m20.fingerprint(), "3f11018081cfe0a2e9e69c87d57a57bd4043df2496c6d2c539dd74b0bf20fb56"),
        ("m21", watcher_combat_observation_spec(&policy_caps).unwrap().fingerprint(), "46fef9f50c0e3212dcd0b3d244cdae663c633713e40f578976c06303f267d771"),
        ("m22", m22.fingerprint(), "ead825f19203b3da9df19c39e79c7fb5d5f2efaa6d35c2971587fed7e0af1d9a"),
        ("m23", m23_run_observation_spec(&caps()).unwrap().fingerprint(), "13dbacc4553ad50bef32e5e23ee05d0c6257aad55e0b5b2a134332784fed7613"),
        ("m24", m24.fingerprint(), "3044b3d2b3ce6edc4343d393cdf1d4eee7733a94752fcd5aac9dd1a021da0332"),
        ("policy", policy_run_observation_spec(&caps()).unwrap().fingerprint(), "28d92d89c437eea7fdc260240585bca466dee1fd74b9b3eb38513d2b34b0be64"),
        ("combat-v1", combat_policy_observation_spec().unwrap().fingerprint(), "4c73f7de04d7b2875f473f8034c1cbacf984ea2d52d0a9428289c30cf1dff463"),
        ("combat-v2", combat_policy_observation_spec_v2().unwrap().fingerprint(), "5ba729fd1b10a067fb8fc5d777d78b47fc9526ec0af0030d6868a32b52a7824b"),
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
    // spire-85d: powers/intents are the shared engine-registry tables (Defect `focus` included).
    assert_eq!(vocab["power"].as_array().unwrap().len(), spire_core::combat::public_ids::PUBLIC_POWERS.len() + 1);
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

    for name in ["shield_bash", "spear_skewer", "heart_debilitate", "heart_buff"] {
        assert!(has(vocab, "intent", name), "missing Act4 intent {name}");
    }
    for name in ["beat_of_death", "invincible", "surrounded", "back_attack"] {
        assert!(has(vocab, "power", name), "missing Act4 power {name}");
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
    // spire-85d: class-only powers are deliberately shared registry rows; an engine-publishable power never faults.
    for shared in ["buffer", "creative_ai"] {
        assert!(has(vocab, "power", shared), "shared registry power {shared}");
    }
    assert_ne!(spec.fingerprint(), m24.fingerprint());
    assert_eq!(hex(m24.fingerprint()), "3044b3d2b3ce6edc4343d393cdf1d4eee7733a94752fcd5aac9dd1a021da0332");
    assert_eq!(RUN_OBSERVATION_M25_SCHEMA_ID, "spire-gym-m25-watcher-heart-observation-v1");
    assert_eq!(d["profile_id"], "M25WatcherHeart");
    let _ = m20;
}
