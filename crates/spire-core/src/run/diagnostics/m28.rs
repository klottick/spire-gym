//! M28 diagnostics: standard A0 Run constructors and supplied-state builders (design §13).
use crate::content::Character;
use crate::run::{CreationProfile, Run, RunConfig, RunError, RunRng, RoomScenario};

/// A standard A0 Run at Neow for `character`, full Neow when `full_neow` (the public constructor
/// `Run::new_room_scenario(.., RoomScenario::Standard)` with `CreationProfile::experienced()`).
pub fn standard_run(character: Character, full_neow: bool, rng: RunRng) -> Result<Run, RunError> {
    let profile = CreationProfile { neow_eligible: full_neow, ..CreationProfile::experienced() };
    Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile, rng, RoomScenario::Standard)
}

pub use crate::run::rooms::standard::shop::ShopSupply;

/// A standard A0 Run standing in an Act 1 shop with the supplied state (see `ShopSupply`); the
/// only route to a standard shop until M28b lands the map rooms.
pub fn standard_shop_run(character: Character, supply: ShopSupply, rng: RunRng) -> Result<Run, RunError> {
    crate::run::rooms::standard::shop::diagnostic_shop(character, supply, rng)
}

/// A standard A0 Run on `act`'s map whose first-row Event room constructs `event` (its event lists
/// hold only that event); returns the Run and that node (M28c item 16).
pub fn standard_event_run(character: Character, act: u8, event: crate::content::EventId, rng: RunRng) -> Result<(Run, crate::run::NodeId), RunError> {
    crate::run::rooms::standard::diagnostic_event_run(character, act, event, rng)
}

/// A standard A0 Run at Neow with supplied current and maximum HP `max_hp` and, when `keys`, all
/// three keys (plan Task X1 supplied traversal, design §14 grade Supplied). Nothing else differs
/// from `standard_run`: the Run then plays the normal acts, transitions and rooms.
pub fn standard_supplied_run(character: Character, full_neow: bool, max_hp: u32, keys: bool, rng: RunRng) -> Result<Run, RunError> {
    let mut run = standard_run(character, full_neow, rng)?;
    run.state.max_hp = max_hp;
    run.state.hp = max_hp;
    if keys { run.state.keys = [true; 3]; }
    Ok(run)
}
