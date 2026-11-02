//! M28 diagnostics: the standard A0 Run constructor (diagnostics-only until M28f).
use crate::content::Character;
use crate::run::{CreationProfile, Run, RunConfig, RunError, RunRng, RoomScenario};

/// A standard A0 Run at Neow for `character`, full Neow when `full_neow`.
pub fn standard_run(character: Character, full_neow: bool, rng: RunRng) -> Result<Run, RunError> {
    let profile = CreationProfile { neow_eligible: full_neow, ..CreationProfile::experienced() };
    Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile, rng, RoomScenario::StandardA0)
}

pub use crate::run::rooms::standard::shop::ShopSupply;

/// A standard A0 Run standing in an Act 1 shop with the supplied state (see `ShopSupply`); the
/// only route to a standard shop until M28b lands the map rooms.
pub fn standard_shop_run(character: Character, supply: ShopSupply, rng: RunRng) -> Result<Run, RunError> {
    crate::run::rooms::standard::shop::diagnostic_shop(character, supply, rng)
}
