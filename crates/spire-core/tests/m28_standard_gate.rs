//! M28f2 (design §13, PM ruling Q1): `RoomScenario::StandardA0` is a public native constructor in a
//! default build, for every character at A0; M28 adds no spire-env or Python profile. (Amended: it
//! pinned the diagnostics-only interim of PM ruling 2026-11-02, which lasted until M28f.)
#![cfg(not(feature = "m28-diagnostics"))]
use spire_core::content::Character;
use spire_core::run::{CreationProfile, RoomScenario, RoomViewKind, Run, RunConfig, RunError, RunRng};

#[test]
fn m28_standard_a0_is_public_in_a_default_build() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let rng = || RunRng::new([0x28; 32], spire_core::combat::rng::Partition::Test, 0);
        let run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, CreationProfile::experienced(), rng(), RoomScenario::StandardA0).unwrap();
        let decision = run.decision().unwrap();
        assert_eq!(decision.content_profile, Some(RoomScenario::StandardA0), "{character:?}");
        assert_eq!(decision.room.map(|room| room.kind), Some(RoomViewKind::Neow), "{character:?}");
        assert_eq!(
            Run::new_room_scenario(RunConfig { character, ascension: 1 }, CreationProfile::experienced(), rng(), RoomScenario::StandardA0).err(),
            Some(RunError::Unsupported("StandardA0 is A0 only")),
            "{character:?}",
        );
    }
}
