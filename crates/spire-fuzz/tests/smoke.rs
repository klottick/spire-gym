use std::{collections::BTreeMap, time::Duration};
#[cfg(not(feature = "diagnostics"))]
use std::fs;
use serde_json::{Value, json};
use spire_env::{RunFeatureGroup, RunObservation};
use spire_fuzz::{Decision, Driver, DriverError, EpisodeOptions, IllegalProbe, Policy, run_episode, write_failure};
#[cfg(not(feature = "diagnostics"))]
use spire_fuzz::{read_repro, run_repro};
#[cfg(not(feature = "diagnostics"))]
use spire_fuzz::{CatalogCombat, FullRun, RunSummary, RUN_PROFILES, POLICIES};

#[cfg(not(feature = "diagnostics"))]
fn smoke_summary() -> RunSummary {
    let mut summary = RunSummary::new("smoke".into());
    let combat_options = EpisodeOptions { max_steps: 200, all_legal_every: 0, determinism: false, replay: false,
        shrink_budget: Duration::ZERO, scripted_actions: None, continue_first: false };
    let run_options = EpisodeOptions { max_steps: 6, replay: true, ..combat_options.clone() };
    let catalog = CatalogCombat::generated(7).unwrap();
    assert!(catalog.entry_count() >= POLICIES.len(), "generated catalog needs one entry per policy episode");
    for (index, policy) in POLICIES.into_iter().enumerate() {
        let mut driver = CatalogCombat::generated(7).unwrap();
        driver.set_entry_cursor(index);
        let options = EpisodeOptions { replay: index == 0, ..combat_options.clone() };
        summary.add(run_episode(Box::new(driver), 7 + index as u64, policy, options));
    }
    for (index, profile) in RUN_PROFILES.iter().copied().enumerate() {
        let policy = POLICIES[index % POLICIES.len()];
        let driver = FullRun::new(profile).unwrap();
        summary.add(run_episode(Box::new(driver), 11 + index as u64, policy, run_options.clone()));
    }
    summary
}

#[test]
#[cfg(not(feature = "diagnostics"))]
fn feature_free_native_smoke_is_deterministic() {
    let first = smoke_summary();
    let second = smoke_summary();
    assert_eq!(first.fingerprint_stability_scope, "stable_within_run_and_identical_across_determinism_rerun");
    assert_smoke_coverage(&first);
    assert_smoke_coverage(&second);
    assert_smoke_failures_are_engine_repros(&first, "first");
    assert_smoke_failures_are_engine_repros(&second, "second");
    assert_eq!(first.episodes, 6 + RUN_PROFILES.len() as u64);
    assert_eq!(first.stable_hash(), second.stable_hash());
}

#[test]
fn standard_combat_specs_smoke_without_engine_or_harness_failures() {
    use spire_core::content::Character;
    use spire_fuzz::{generated_standard_catalog_json, CatalogCombat, RunSummary, run_episode};
    let specs = [
        (Character::Ironclad, 0), (Character::Ironclad, 20),
        (Character::Silent, 0), (Character::Silent, 20),
        (Character::Defect, 0), (Character::Defect, 20),
        (Character::Watcher, 0), (Character::Watcher, 20),
    ];
    let pass = || {
        let mut summary = RunSummary::new("standard-combat-smoke".into());
        let options = EpisodeOptions {
            max_steps: 120, all_legal_every: 0, determinism: true, replay: true,
            shrink_budget: Duration::ZERO, scripted_actions: None, continue_first: false,
        };
        for (character, ascension) in specs {
            for seed in 1..=8 {
                let bytes = generated_standard_catalog_json(seed, character, ascension);
                let mut driver = CatalogCombat::standard_from_json(&bytes, character, ascension).unwrap_or_else(|error| {
                    let catalog: Value = serde_json::from_slice(&bytes).unwrap();
                    for entry in catalog["entries"].as_array().unwrap() {
                        let mut single = catalog.clone();
                        single["entries"] = json!([entry]);
                        let one = serde_json::to_vec(&single).unwrap();
                        if let Err(entry_error) = CatalogCombat::standard_from_json(&one, character, ascension) {
                            panic!("seed {seed} {character:?} A{ascension} entry {} rejected: {entry_error:?}; data={entry}; full catalog: {error:?}", entry["id"]);
                        }
                    }
                    panic!("seed {seed} {character:?} A{ascension} catalog rejected despite individually valid entries: {error:?}");
                });
                driver.set_entry_cursor(seed as usize);
                summary.add(run_episode(Box::new(driver), seed, Policy::First, options.clone()));
            }
        }
        summary
    };
    let first = pass();
    let second = pass();
    for (spec, totals) in &first.outcomes_by_driver {
        eprintln!("{spec}: episodes={} victories={} deaths={} escapes={} capped={}",
            totals.episodes, totals.wins, totals.deaths, totals.escapes, totals.capped);
    }
    assert_eq!(first.episodes, 64);
    assert_eq!(first.episodes_by_driver.len(), 8);
    assert!(first.episodes_by_driver.values().all(|episodes| *episodes == 8));
    assert!(first.failures.is_empty(), "Standard combat smoke failures: {:?}",
        first.failures.iter().map(|failure| (&failure.driver, &failure.kind, &failure.message)).collect::<Vec<_>>());
    assert_eq!(first.stable_hash(), second.stable_hash());
}

#[cfg(not(feature = "diagnostics"))]
fn assert_smoke_coverage(summary: &RunSummary) {
    assert_eq!(summary.episodes_by_driver.get("catalog"), Some(&(POLICIES.len() as u64)));
    assert_eq!(summary.episodes_by_driver.get("run"), Some(&(RUN_PROFILES.len() as u64)));
    assert!(summary.steps > summary.episodes, "smoke should cover multiple decisions per episode");
    for profile in RUN_PROFILES {
        assert_eq!(summary.episodes_by_profile.get(profile.id), Some(&1), "missing profile {}", profile.id);
        let outcome = summary.outcomes_by_profile.get(profile.id).expect("Run profile outcome");
        let faulted_before_combat = summary.failures.iter().any(|failure| {
            failure.driver == "run" && failure.driver_config["profile"] == profile.id
        });
        assert!(outcome.combat_episodes > 0 || faulted_before_combat,
            "Run profile {} did not reach combat and has no recorded engine failure", profile.id);
    }
    for character in ["ironclad", "silent", "defect", "watcher"] {
        assert!(summary.episodes_by_character.get(character).is_some_and(|episodes| *episodes > 0),
            "missing character {character}");
    }
    for policy in POLICIES { assert!(summary.episodes_by_policy.get(policy.name()).is_some_and(|episodes| *episodes > 0), "missing policy {}", policy.name()); }
    assert_eq!(summary.episodes_by_entry.len(), POLICIES.len(), "each catalog episode should use a distinct entry");
    assert!(summary.episodes_by_entry.values().all(|episodes| *episodes == 1), "catalog entry was reused: {:?}", summary.episodes_by_entry);
}

#[cfg(not(feature = "diagnostics"))]
fn assert_smoke_failures_are_engine_repros(summary: &RunSummary, run: &str) {
    let output = std::env::temp_dir().join(format!("spire-fuzz-smoke-default-{}-{run}", std::process::id()));
    let _ = fs::remove_dir_all(&output);
    for failure in &summary.failures {
        assert!(!matches!(failure.kind.as_str(), "determinism" | "replay" | "panic"),
            "harness integrity failure must be zero: {}", failure_details(summary));
        assert!(matches!(failure.kind.as_str(), "fault" | "validator" | "legal_rejected" | "illegal_accepted"
            | "illegal_corrupts" | "encode" | "stuck" | "livelock"),
            "unclassified smoke failure {}: {}", failure.kind, failure.message);
        assert!(matches!(failure.driver.as_str(), "catalog" | "run"));
        assert!(failure.driver_config.is_object());
        assert_eq!(failure.feature_set, "default");
        assert!(!failure.message.trim().is_empty());
        assert!(failure.step <= failure.actions.len(), "failure step exceeds its saved action prefix");
        let path = write_failure(&output, failure).expect("write smoke repro");
        assert!(path.is_file(), "missing written repro {}", path.display());
        assert!(path.with_file_name("beads.md").is_file(), "missing repro report");
        let repro = read_repro(&path).expect("parse written repro");
        assert_eq!(repro.expected_kind, failure.kind);
        assert_eq!(repro.driver, failure.driver);
        run_repro(&repro).unwrap_or_else(|error| panic!("smoke repro failed to re-detect {} seed={}: {error}", failure.kind, failure.seed));
    }
    let _ = fs::remove_dir_all(output);
}

#[cfg(not(feature = "diagnostics"))]
fn failure_details(summary: &RunSummary) -> String {
    summary.failures.iter().map(|failure| format!("{} seed={} policy={} step={} {}: {}", failure.driver,
        failure.seed, failure.policy.name(), failure.step, failure.kind, failure.message)).collect::<Vec<_>>().join("\n")
}

struct FakeDriver { seed: u64, steps: usize }
impl FakeDriver { fn new() -> Self { Self { seed: 0, steps: 0 } } }
impl Driver for FakeDriver {
    fn class(&self) -> &'static str { "fake" }
    fn describe(&self) -> Value { json!({"driver":"fake","injected":"validator-after-two-steps"}) }
    fn fresh(&self) -> Box<dyn Driver> { Box::new(Self::new()) }
    fn reset(&mut self, seed: u64) -> Result<Decision, DriverError> {
        self.seed = seed; self.steps = 0; Ok(fake_decision(self.seed, self.steps))
    }
    fn decision(&self) -> Result<Decision, DriverError> { Ok(fake_decision(self.seed, self.steps)) }
    fn step(&mut self, index: usize) -> Result<spire_fuzz::StepOut, DriverError> {
        if index != 0 { return Err(DriverError::fault("fake bad index")); }
        self.steps += 1;
        Ok(spire_fuzz::StepOut { decision: fake_decision(self.seed, self.steps), reward: 0, terminal: None })
    }
    fn preflight(&self, index: usize) -> Result<(), DriverError> {
        if index == 0 { Ok(()) } else { Err(DriverError::new("legal_rejected", "fake advertised index rejected")) }
    }
    fn probe_illegal(&mut self, _probe: IllegalProbe) -> Result<(), DriverError> { Err(DriverError::new("probe_rejected", "fake rejected invalid index")) }
    fn descriptor_stable(&self) -> Result<(), DriverError> { Ok(()) }
    fn diagnostic(&self) -> Value { json!({"steps":self.steps,"seed":self.seed}) }
    fn labels(&self) -> BTreeMap<String, String> { BTreeMap::from([("profile".into(), "fake".into())]) }
    fn validate(&self) -> Result<(), DriverError> {
        if self.steps >= 2 { Err(DriverError::new("validator", "synthetic invariant: counter reached two")) } else { Ok(()) }
    }
}

fn fake_decision(seed: u64, step: usize) -> Decision {
    Decision { observation: RunObservation { groups: vec![RunFeatureGroup { key: "actions", shape: vec![1, 8],
        values: vec![10, 0, 0, 0, 0, 0, 0, 0], known: vec![1; 8], occupied: Some(vec![1]) }], action_mask: vec![1] },
        token: [seed, step as u64], terminal: None }
}

#[test]
fn synthetic_failure_is_detected_shrunk_and_written_with_a_fake_driver() {
    let options = EpisodeOptions { max_steps: 8, all_legal_every: 0, determinism: false, replay: false,
        shrink_budget: Duration::from_secs(1), scripted_actions: Some(vec![0; 8]), continue_first: true };
    let result = run_episode(Box::new(FakeDriver::new()), 7, Policy::First, options);
    let failure = result.failure.expect("synthetic invariant should be detected");
    assert_eq!(failure.kind, "validator");
    assert!(failure.actions.len() < 2, "actions were not shrunk: {:?}", failure.actions);
    let out = std::env::temp_dir().join(format!("spire-fuzz-smoke-{}", std::process::id()));
    let path = write_failure(&out, &failure).unwrap();
    assert!(path.ends_with("repro.json"));
    assert!(path.with_file_name("beads.md").exists());
    let case: spire_fuzz::ReproCase = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(case.expected_kind, "validator");
    let _ = std::fs::remove_dir_all(out);
}
