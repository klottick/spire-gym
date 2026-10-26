//! Thin finite M14 scenario adapter over the accepted M13 caller.

#[path = "../m13_support/mod.rs"]
#[allow(dead_code)]
pub mod m13_support;

#[allow(unused_imports)]
pub use self::m13_support::{
    rollout_m14, Caller, CallerOutcome, TraceStep, M14_ACTION_LIMIT_MAX,
};

use spire_core::{
    content::{Character, EncounterId},
    run::{CreationProfile, Run, RunConfig, RoomScenario},
};

pub const CALLER_POLICY_REVISION: &str = "m14-shared-public-caller-v5";
pub const FIXED_SEEDS: [u64; 2] = m13_support::FIXED_SEEDS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub act1_boss: EncounterId,
    pub act2_boss: EncounterId,
    pub act3_boss: EncounterId,
}

pub const FIXED_CASES: [(u64, Scenario); 6] = [
    (7, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::Champ, act3_boss: EncounterId::AwakenedOne }),
    (7, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::TimeEater }),
    (7, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Collector, act3_boss: EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::Collector, act3_boss: EncounterId::AwakenedOne }),
    (11, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Champ, act3_boss: EncounterId::TimeEater }),
];

mod evidence_suite;

/// Exactly the approved original six plus seeds12–23 parity triples.
pub fn evidence_cases() -> &'static [(u64, Scenario)] {
    &evidence_suite::EVIDENCE_CASES
}

#[cfg(test)]
mod evidence_suite_tests;

/// Persist the complete public caller transcript and the rollout result,
/// including operational errors. The result remains an error; this artifact
/// records it alongside every accepted transition and any failed action.
pub fn persist_attempt_trace(
    seed: u64,
    scenario: Scenario,
    result: &Result<CallerOutcome, String>,
    caller: &Caller,
    run: Option<&Run>,
) -> Result<(), String> {
    let Some(directory) = std::env::var_os("SPIRE_M14_TRACE_DIR") else {
        return Ok(());
    };
    use std::io::Write;
    let directory = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&directory).map_err(|error| format!("create trace directory: {error}"))?;
    let path = directory.join(format!(
        "seed-{seed}-{:?}-{:?}-{:?}.trace.txt",
        scenario.act1_boss, scenario.act2_boss, scenario.act3_boss
    ));
    let mut file = std::fs::File::create(path).map_err(|error| format!("create M14 trace: {error}"))?;
    let recorded_endpoint = caller.final_decision.as_ref().map(|decision| {
        (decision.act, decision.floor, decision.terminal, decision.room.as_ref().map(|room| room.kind))
    });
    let available_endpoint = match run {
        Some(run) => match run.decision() {
            Ok(decision) => format!("{:?}", (decision.act, decision.floor, decision.terminal, decision.room.as_ref().map(|room| room.kind))),
            Err(error) => format!("unavailable: {error:?}"),
        },
        None => "unavailable: no Run was created".into(),
    };
    writeln!(
        file,
        "seed={seed} caller_policy={} scenario={scenario:?} result={result:?} accepted_actions={} available_endpoint={available_endpoint} recorded_caller_endpoint={recorded_endpoint:?}\n{caller:#?}",
        CALLER_POLICY_REVISION,
        caller.accepted(),
    )
    .map_err(|error| format!("write M14 trace: {error}"))
}

pub fn endpoint_status(run: Option<&Run>, caller: &Caller) -> String {
    match run {
        Some(run) => match run.decision() {
            Ok(decision) => format!("act={} floor={} terminal={:?} room={:?}", decision.act, decision.floor, decision.terminal, decision.room.as_ref().map(|room| room.kind)),
            Err(error) => format!("unavailable: {error:?}; last caller decision={:?}", caller.final_decision.as_ref().map(|decision| (decision.act, decision.floor, decision.terminal, decision.room.as_ref().map(|room| room.kind)))),
        },
        None => format!("unavailable: no Run; last caller decision={:?}", caller.final_decision.as_ref().map(|decision| (decision.act, decision.floor, decision.terminal, decision.room.as_ref().map(|room| room.kind)))),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub seed: u64,
    pub scenarios: Vec<Scenario>,
    pub limit: usize,
    pub nonattacking: bool,
}

impl Scenario {
    pub const fn label(self) -> &'static str {
        match self.act1_boss {
            EncounterId::Guardian => "guardian",
            EncounterId::Hexaghost => "hexaghost",
            EncounterId::SlimeBoss => "slime",
            _ => "invalid",
        }
    }
}

pub fn cases_for_seed(seed: u64) -> Result<Vec<Scenario>, String> {
    let cases = evidence_cases().iter().filter(|(case_seed, _)| *case_seed == seed)
        .map(|(_, scenario)| *scenario).collect::<Vec<_>>();
    if cases.is_empty() {
        return Err("seed must be 7, 11, or 12..=23 from the declared M14 suite".into());
    }
    Ok(cases)
}

/// Installs every exact canonical boss-history string before creating the Run.
/// Only the 42 predeclared seed/triple inputs are accepted by this adapter.
pub fn make_run(seed: u64, scenario: Scenario) -> Result<Run, String> {
    if !cases_for_seed(seed)?.contains(&scenario) {
        return Err("scenario is outside the 42 declared seed/boss triples".into());
    }

    let act1_act2 = m13_support::Scenario {
        act1_boss: scenario.act1_boss,
        act2_boss: scenario.act2_boss,
    };
    let act1_seen = m13_support::act1_seen_bosses(act1_act2);
    let act2_seen = m13_support::act2_seen_bosses(act1_act2);
    let act3_seen = act3_seen_bosses(scenario.act3_boss)?;
    if act1_seen.len() != 2 || act2_seen.len() != 2 || act3_seen.len() != 2 {
        return Err("M14 boss history must exclude exactly the two canonical bosses per act".into());
    }

    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    profile.seen_bosses[0] = act1_seen;
    profile.seen_bosses[1] = act2_seen;
    profile.seen_bosses[2] = act3_seen;
    Run::new_room_scenario(
        RunConfig { character: Character::Silent, ascension: 0 },
        profile,
        m13_support::rng(seed),
        RoomScenario::M14SilentAct3,
    )
    .map_err(|error| format!("M14 Silent A0 creation: {error:?}"))
}

fn act3_seen_bosses(target: EncounterId) -> Result<Vec<&'static str>, String> {
    match target {
        EncounterId::AwakenedOne => Ok(vec!["TimeEater", "DonuAndDeca"]),
        EncounterId::TimeEater => Ok(vec!["AwakenedOne", "DonuAndDeca"]),
        EncounterId::DonuAndDeca => Ok(vec!["AwakenedOne", "TimeEater"]),
        _ => Err(format!("invalid M14 Act3 boss history target: {target:?}")),
    }
}

pub fn parse_args<I>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = String>,
{
    let mut seed = 7;
    let mut scenario_label = "all".to_owned();
    let mut nonattacking = false;
    let mut limit = M14_ACTION_LIMIT_MAX;
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--seed" => seed = args.next().ok_or("missing --seed value")?.parse().map_err(|_| "seed must be integer")?,
            "--budget" => limit = args.next().ok_or("missing --budget value")?.parse().map_err(|_| "budget must be integer")?,
            "--scenario" => scenario_label = args.next().ok_or("missing --scenario value")?,
            _ => return Err(format!("unknown argument {flag}")),
        }
    }
    let all = cases_for_seed(seed)?;
    let scenarios = match scenario_label.as_str() {
        "all" => all,
        "death" => { nonattacking = true; vec![all[0]] },
        "guardian" | "hexaghost" | "slime" => all.into_iter()
            .filter(|scenario| scenario.label() == scenario_label).collect(),
        _ => return Err("scenario must be all|guardian|hexaghost|slime|death".into()),
    };
    if limit > M14_ACTION_LIMIT_MAX {
        return Err(format!("budget must be in 0..={M14_ACTION_LIMIT_MAX}"));
    }
    Ok(Options { seed, scenarios, limit, nonattacking })
}
