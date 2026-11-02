//! PM ruling 2026-11-02: `RoomScenario::StandardA0` is diagnostics-only until M28f.
#![cfg(not(feature = "m28-diagnostics"))]
use spire_core::content::Character;
use spire_core::run::{CreationProfile, RoomScenario, Run, RunConfig, RunError, RunRng};

#[test]
fn m28_standard_a0_is_diagnostics_only() {
    let rng = RunRng::new([0x28; 32], spire_core::combat::rng::Partition::Test, 0);
    assert_eq!(
        Run::new_room_scenario(RunConfig { character: Character::Ironclad, ascension: 0 }, CreationProfile::experienced(), rng, RoomScenario::StandardA0).err(),
        Some(RunError::Unsupported("StandardA0 is diagnostics-only until M28f")),
    );
}
