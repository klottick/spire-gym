//! M28f2 (design §13, PM ruling Q1): `RoomScenario::Standard` is a public native constructor in a
//! default build, for every character at A0; M28 adds no spire-env or Python profile. (Amended: it
//! pinned the diagnostics-only interim of PM ruling 2026-11-02, which lasted until M28f.)
#![cfg(not(feature = "m28-diagnostics"))]
use spire_core::content::Character;
use spire_core::run::{CreationProfile, RoomScenario, RoomViewKind, Run, RunConfig, RunError, RunRng};

#[test]
fn m28_standard_a0_is_public_in_a_default_build() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let rng = || RunRng::new([0x28; 32], spire_core::combat::rng::Partition::Test, 0);
        let run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, CreationProfile::experienced(), rng(), RoomScenario::Standard).unwrap();
        let decision = run.decision().unwrap();
        assert_eq!(decision.content_profile, Some(RoomScenario::Standard), "{character:?}");
        assert_eq!(decision.room.map(|room| room.kind), Some(RoomViewKind::Neow), "{character:?}");
        // Amended (M29a1, plan erratum v3 n2): A1–A5 were diagnostics-only until M29f. Amended again at the
        // M29 close (erratum n2): A1 is public in a default build. Amended at M30 (plan item 1(b), review
        // n1): the cap is A10, so A11 is the rejected level. Amended at M31 (plan item 1(a)): the cap is A15,
        // so A16 is the rejected level. Amended at M32 (plan item 1): the cap is A19, so A20 is the rejected level.
        assert!(
            Run::new_room_scenario(RunConfig { character, ascension: 1 }, CreationProfile::experienced(), rng(), RoomScenario::Standard).is_ok(),
            "{character:?}",
        );
        assert_eq!(
            Run::new_room_scenario(RunConfig { character, ascension: 20 }, CreationProfile::experienced(), rng(), RoomScenario::Standard).err(),
            Some(RunError::Unsupported("Standard supports A0–A19")),
            "{character:?}",
        );
    }
}
