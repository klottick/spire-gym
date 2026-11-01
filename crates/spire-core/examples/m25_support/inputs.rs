//! Fixed canonical M25 inputs and parse-only validation.
#[path = "../m24_support/inputs.rs"]
mod m24_inputs;

use spire_core::{
    combat::rng::Partition,
    content::{Character, EncounterId},
    run::{RoomScenario, Run, RunConfig, RunRng},
};

pub const ACTION_LIMIT_MAX: usize = 32768;
pub const SEEDS: [u64; 2] = [7, 11];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    AwakenedOne,
    TimeEater,
    DonuAndDeca,
}

pub const SCENARIOS: [Scenario; 3] = [
    Scenario::AwakenedOne,
    Scenario::TimeEater,
    Scenario::DonuAndDeca,
];

impl Scenario {
    pub fn name(self) -> &'static str {
        match self {
            Self::AwakenedOne => "awakened-one",
            Self::TimeEater => "time-eater",
            Self::DonuAndDeca => "donu-and-deca",
        }
    }

    pub fn act3(self) -> EncounterId {
        match self {
            Self::AwakenedOne => EncounterId::AwakenedOne,
            Self::TimeEater => EncounterId::TimeEater,
            Self::DonuAndDeca => EncounterId::DonuAndDeca,
        }
    }

    fn m24(self) -> m24_inputs::Scenario {
        match self {
            Self::AwakenedOne => m24_inputs::Scenario::AwakenedOne,
            Self::TimeEater => m24_inputs::Scenario::TimeEater,
            Self::DonuAndDeca => m24_inputs::Scenario::DonuAndDeca,
        }
    }
}

/// Design §12.3 triples, in `(Act1, Act2, Act3)` order. The death control is
/// only the seed-7 Guardian/Champ/AwakenedOne case.
pub fn triple(
    seed: u64,
    scenario: Scenario,
    death: bool,
) -> Result<(EncounterId, EncounterId, EncounterId), String> {
    let (act1, act2) = m24_inputs::triple(seed, scenario.m24(), death)?;
    Ok((act1, act2, scenario.act3()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub seed: u64,
    pub budget: usize,
    pub scenarios: Vec<Scenario>,
    pub nonattacking: bool,
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut options = Options {
        seed: 7,
        budget: ACTION_LIMIT_MAX,
        scenarios: SCENARIOS.to_vec(),
        nonattacking: false,
    };
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--seed" => options.seed = value.parse().map_err(|_| "invalid seed")?,
            "--budget" => options.budget = value.parse().map_err(|_| "invalid budget")?,
            "--scenario" => {
                options.nonattacking = value == "death";
                options.scenarios = match value.as_str() {
                    "all" => SCENARIOS.to_vec(),
                    "awakened-one" | "death" => vec![Scenario::AwakenedOne],
                    "time-eater" => vec![Scenario::TimeEater],
                    "donu-and-deca" => vec![Scenario::DonuAndDeca],
                    _ => return Err("unsupported M25 scenario".into()),
                };
            }
            _ => return Err(format!("unsupported flag {flag}")),
        }
    }

    if !SEEDS.contains(&options.seed) {
        return Err("M25 seed must be 7 or 11".into());
    }
    if options.budget > ACTION_LIMIT_MAX {
        return Err("M25 budget must be 0..=32768".into());
    }
    for scenario in &options.scenarios {
        triple(options.seed, *scenario, options.nonattacking)?;
    }
    Ok(options)
}

/// Construct only after the caller has accepted the complete input contract.
pub fn make_run(seed: u64, scenario: Scenario, death: bool) -> Result<Run, String> {
    triple(seed, scenario, death)?;
    let profile = m24_inputs::creation(seed, scenario.m24(), death)?;
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    let run = Run::new_room_scenario(
        RunConfig {
            character: Character::Watcher,
            ascension: 0,
        },
        profile,
        RunRng::new(root, Partition::Test, 0),
        RoomScenario::M25WatcherHeart,
    )
    .map_err(|error| format!("{error:?}"))?;

    Ok(run)
}
