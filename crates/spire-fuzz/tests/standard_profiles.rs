use spire_core::content::Character;
use spire_fuzz::RUN_PROFILES;

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
