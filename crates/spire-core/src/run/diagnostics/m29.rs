//! M29 diagnostics (plan M29a1 item 10): standard Runs and maps at any admitted ascension (A0–A19 since
//! M32). A1–A5 standard Runs were diagnostics-only until M29f (plan §1 Exposure) and are public since
//! the M29 close; A6–A10 are public since M30, A11–A15 since M31 and A16–A19 since M32. These builders
//! go through the public constructor (the supplied variant stays diagnostics-only).
use crate::content::Character;
use crate::run::{CreationProfile, Run, RunConfig, RunError, RunRng, RoomScenario};

/// A standard Run at `ascension` (A0–A19) at Neow, full Neow when `full_neow`.
pub fn standard_run_at(character: Character, ascension: u8, full_neow: bool, rng: RunRng) -> Result<Run, RunError> {
    let profile = CreationProfile { neow_eligible: full_neow, ..CreationProfile::experienced() };
    Run::new_room_scenario(RunConfig { character, ascension }, profile, rng, RoomScenario::Standard)
}

/// `standard_supplied_run` at `ascension`: supplied current and maximum HP `max_hp` and, when
/// `keys`, all three keys (grade Supplied); the Run then plays its normal acts and rooms.
pub fn standard_supplied_run_at(character: Character, ascension: u8, full_neow: bool, max_hp: u32, keys: bool, rng: RunRng) -> Result<Run, RunError> {
    let mut run = standard_run_at(character, ascension, full_neow, rng)?;
    run.state.max_hp = max_hp;
    run.state.hp = max_hp;
    if keys { run.state.keys = [true; 3]; }
    Ok(run)
}

/// The ordinary generated map of `act` (1–3) at `ascension`, Act 4 enabled and no Emerald key.
pub fn standard_map_at(act: u8, ascension: u8, rng: &mut RunRng) -> Result<crate::run::ActMap, RunError> {
    crate::run::map::generate(act, rng, crate::run::map::MapEligibility::new(true, false).with_ascension(ascension))
}
