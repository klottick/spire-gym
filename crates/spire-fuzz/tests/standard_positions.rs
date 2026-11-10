use std::time::Duration;

use spire_core::content::Character;
use spire_fuzz::{standard_position_spec, run_episode, Driver, EpisodeOptions, Policy, RunSummary, StandardPositionCombat};

const SPECS: [(Character, u8); 8] = [
    (Character::Ironclad, 0), (Character::Ironclad, 20),
    (Character::Silent, 0), (Character::Silent, 20),
    (Character::Defect, 0), (Character::Defect, 20),
    (Character::Watcher, 0), (Character::Watcher, 20),
];

fn options() -> EpisodeOptions {
    EpisodeOptions { max_steps: 40, all_legal_every: 0, determinism: true, replay: true,
        shrink_budget: Duration::ZERO, scripted_actions: None, continue_first: false }
}

fn run_spec(character: Character, ascension: u8, episodes: u64, seed_base: u64) -> RunSummary {
    let mut summary = RunSummary::new("m34s-c-position-fuzz".into());
    let mut rejected = 0u64;
    for offset in 0..episodes {
        let seed = seed_base + offset;
        let result = run_episode(Box::new(StandardPositionCombat::new(character, ascension)), seed, Policy::Uniform, options());
        rejected += result.labels.get("rejected_candidates").and_then(|value| value.parse::<u64>().ok()).unwrap_or(0);
        summary.add(result);
    }
    let total_candidates = episodes + rejected;
    let rejection_rate = rejected as f64 / total_candidates as f64;
    assert!(rejection_rate < 0.5, "{} rejection rate {rejected}/{total_candidates} = {rejection_rate:.3}", standard_position_spec(character, ascension));
    assert!(summary.failures.is_empty(), "{} failure groups: {}", standard_position_spec(character, ascension), serde_json::to_string(&summary.failures).unwrap());
    println!("{} episodes={} steps={} wins={} deaths={} escapes={} capped={} rejected={} candidates={} rejection_rate={:.4} failures={}",
        standard_position_spec(character, ascension), summary.episodes, summary.steps, summary.wins, summary.deaths, summary.escapes,
        summary.capped, rejected, total_candidates, rejection_rate, summary.failures.len());
    summary
}

#[test]
fn standard_position_specs_run_twenty_random_legal_episodes_each() {
    for (index, (character, ascension)) in SPECS.into_iter().enumerate() {
        let summary = run_spec(character, ascension, 20, 0xC500_0000 + (index as u64) * 10_000);
        assert_eq!(summary.episodes, 20);
    }
}

#[test]
fn same_seed_builds_the_same_standard_position() {
    for (index, (character, ascension)) in SPECS.into_iter().enumerate() {
        let seed = 0xC5D0_0000 + index as u64;
        let mut first = StandardPositionCombat::new(character, ascension);
        let first_boundary = first.reset(seed).unwrap();
        let first_config = first.describe();
        let mut second = StandardPositionCombat::new(character, ascension);
        let second_boundary = second.reset(seed).unwrap();
        assert_eq!(second.describe(), first_config, "{} catalog", standard_position_spec(character, ascension));
        assert_eq!(second_boundary.observation, first_boundary.observation, "{} observation", standard_position_spec(character, ascension));
    }
}

#[test]
#[ignore = "500 episode evidence lane; invoked explicitly by the c5 brief"]
fn standard_position_specs_run_five_hundred_random_legal_episodes_each() {
    for (index, (character, ascension)) in SPECS.into_iter().enumerate() {
        let summary = run_spec(character, ascension, 500, 0xC550_0000 + (index as u64) * 1_000_000);
        assert_eq!(summary.episodes, 500);
    }
}
