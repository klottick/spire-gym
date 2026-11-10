use spire_core::{combat::enemies::A0_ENCOUNTER_SETUPS, content::Character};
use spire_env::CombatTrainingCatalog;
use spire_fuzz::{generated_standard_catalog_json, CatalogCombat, Driver, RUN_PROFILES};
use serde_json::Value;

#[test]
fn standard_fuzz_profiles_cover_four_characters_at_a0_a10_and_a20() {
    let mut ids = RUN_PROFILES
        .iter()
        .filter(|profile| profile.id.starts_with("standard-"))
        .map(|profile| profile.id)
        .collect::<Vec<_>>();
    ids.sort_unstable();

    let expected = [
        "standard-defect-a0", "standard-defect-a10", "standard-defect-a20",
        "standard-ironclad-a0", "standard-ironclad-a10", "standard-ironclad-a20",
        "standard-silent-a0", "standard-silent-a10", "standard-silent-a20",
        "standard-watcher-a0", "standard-watcher-a10", "standard-watcher-a20",
    ];
    assert_eq!(ids, expected);

    let expected_profiles = [
        ("standard-ironclad-a0", Character::Ironclad, 0),
        ("standard-ironclad-a10", Character::Ironclad, 10),
        ("standard-ironclad-a20", Character::Ironclad, 20),
        ("standard-silent-a0", Character::Silent, 0),
        ("standard-silent-a10", Character::Silent, 10),
        ("standard-silent-a20", Character::Silent, 20),
        ("standard-defect-a0", Character::Defect, 0),
        ("standard-defect-a10", Character::Defect, 10),
        ("standard-defect-a20", Character::Defect, 20),
        ("standard-watcher-a0", Character::Watcher, 0),
        ("standard-watcher-a10", Character::Watcher, 10),
        ("standard-watcher-a20", Character::Watcher, 20),
    ];
    for (id, character, ascension) in expected_profiles {
        let profile = RUN_PROFILES.iter().find(|profile| profile.id == id).unwrap();
        assert_eq!(profile.character, character, "{id}");
        assert_eq!(profile.ascension, ascension, "{id}");
        let creation = profile.creation.expect("Standard creation spec");
        assert_eq!(creation.to_json()["neow"], "full", "{id}");
    }
}

#[test]
fn standard_combat_specs_exist_and_generated_catalogs_parse_for_50_seeds() {
    let characters = Character::ALL;
    let expected = [
        ("ironclad", Character::Ironclad, 0), ("ironclad", Character::Ironclad, 20),
        ("silent", Character::Silent, 0), ("silent", Character::Silent, 20),
        ("defect", Character::Defect, 0), ("defect", Character::Defect, 20),
        ("watcher", Character::Watcher, 0), ("watcher", Character::Watcher, 20),
    ];
    let mut seen = Vec::new();
    for (name, character, ascension) in expected {
        let driver = CatalogCombat::generated_standard(17, character, ascension).unwrap();
        assert_eq!(driver.class(), format!("standard-combat-{name}-a{ascension}"));
        assert_eq!(driver.entry_count(), A0_ENCOUNTER_SETUPS.len());
        let mut driver = driver;
        driver.reset(17).unwrap();
        let labels = driver.labels();
        assert_eq!(labels.get("character").map(String::as_str), Some(name));
        assert_eq!(labels.get("ascension").map(String::as_str), Some(if ascension == 0 { "0" } else { "20" }));
        seen.push(driver.class());
    }
    seen.sort_unstable();
    assert_eq!(seen, [
        "standard-combat-defect-a0", "standard-combat-defect-a20",
        "standard-combat-ironclad-a0", "standard-combat-ironclad-a20",
        "standard-combat-silent-a0", "standard-combat-silent-a20",
        "standard-combat-watcher-a0", "standard-combat-watcher-a20",
    ]);
    assert_eq!(characters.len(), 4);

    for seed in 0..50 {
        for (_, character, ascension) in expected {
            let bytes = generated_standard_catalog_json(seed, character, ascension);
            let first: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(bytes, generated_standard_catalog_json(seed, character, ascension), "seed {seed} {character:?} A{ascension}");
            assert_eq!(first["entries"].as_array().unwrap().len(), A0_ENCOUNTER_SETUPS.len());
            CombatTrainingCatalog::standard_from_json(&bytes).unwrap_or_else(|error| {
                panic!("seed {seed} {character:?} A{ascension} catalog rejected: {error:?}")
            });
        }
    }
}
