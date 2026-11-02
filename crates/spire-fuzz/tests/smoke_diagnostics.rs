#![cfg(feature = "diagnostics")]

use std::{fs, time::Duration};
use spire_core::{content::{CardId, Character, EncounterId}, run::{AuthoredRunStart, CardInstance}};
use spire_fuzz::{CatalogCombat, CompleteCombat, CompleteItemsRun, EpisodeOptions, FullRun, POLICIES, RUN_PROFILES,
    RunSummary, COMPLETE_ITEMS_RUN_PROFILES, complete_cases_for_smoke, read_repro, run_episode, run_repro,
    write_failure, Driver};

fn diagnostics_smoke() -> RunSummary {
    let mut summary = RunSummary::new("diagnostics-smoke".into());
    let options = EpisodeOptions { max_steps: 8, all_legal_every: 4, determinism: false, replay: true,
        shrink_budget: Duration::ZERO, scripted_actions: None, continue_first: false };
    let catalog = CatalogCombat::generated(31).unwrap();
    assert!(catalog.entry_count() >= POLICIES.len(), "generated catalog needs one entry per policy episode");
    for (index, policy) in POLICIES.into_iter().enumerate() {
        let mut driver = CatalogCombat::generated(31).unwrap();
        driver.set_entry_cursor(index);
        summary.add(run_episode(Box::new(driver), 31 + index as u64, policy, options.clone()));
    }
    for (index, (character, encounter)) in complete_cases_for_smoke().into_iter().enumerate() {
        let policy = POLICIES[index % POLICIES.len()];
        let driver = CompleteCombat::new(character, encounter);
        summary.add(run_episode(Box::new(driver), 71 + index as u64, policy, options.clone()));
    }
    for (index, profile) in RUN_PROFILES.iter().copied().enumerate() {
        let policy = POLICIES[index % POLICIES.len()];
        let driver = FullRun::new(profile).unwrap();
        summary.add(run_episode(Box::new(driver), 113 + index as u64, policy, options.clone()));
    }
    for (index, profile) in COMPLETE_ITEMS_RUN_PROFILES.iter().copied().enumerate() {
        let policy = POLICIES[index % POLICIES.len()];
        let driver = CompleteItemsRun::new(profile);
        summary.add(run_episode(Box::new(driver), 211 + index as u64, policy, options.clone()));
    }
    summary
}

#[test]
fn all_diagnostic_drivers_policies_characters_acts_and_profiles_are_deterministic() {
    let first = diagnostics_smoke();
    let second = diagnostics_smoke();
    assert_eq!(first.fingerprint_stability_scope, "stable_within_run_and_identical_across_determinism_rerun");
    assert_diagnostics_coverage(&first);
    assert_diagnostics_coverage(&second);
    assert_smoke_failures_are_engine_repros(&first, "first");
    assert_smoke_failures_are_engine_repros(&second, "second");
    assert_eq!(first.episodes, 37);
    assert_eq!(first.driver_descriptions["items"]["starts_at"], "Act4 map");
    assert_eq!(first.driver_descriptions["items"]["neow"], false);
    assert_eq!(first.driver_descriptions["items"]["encode"], "n/a");
    assert_eq!(first.driver_descriptions["complete"]["encode"], "harness projection (spire-fuzz-complete-public-v1)");
    assert_eq!(first.driver_descriptions["complete"]["fingerprint"], "n/a");
    for profile in RUN_PROFILES { assert!(first.episodes_by_profile.contains_key(profile.id), "missing {}", profile.id); }
    assert_eq!(first.stable_hash(), second.stable_hash());
}

fn assert_diagnostics_coverage(summary: &RunSummary) {
    assert_eq!(summary.episodes_by_driver.get("catalog"), Some(&6));
    assert_eq!(summary.episodes_by_driver.get("complete"), Some(&12));
    assert_eq!(summary.episodes_by_driver.get("run"), Some(&(RUN_PROFILES.len() as u64)));
    assert_eq!(summary.episodes_by_driver.get("items"), Some(&(COMPLETE_ITEMS_RUN_PROFILES.len() as u64)));
    assert!(summary.steps > summary.episodes, "diagnostics smoke should cover multiple decisions per episode");
    assert!(summary.outcomes_by_driver.values().map(|outcome| outcome.all_legal_checks_after_step_zero).sum::<u64>() > 0,
        "all-legal clone checks must run beyond step zero");
    assert_eq!(summary.episodes_by_entry.len(), POLICIES.len(), "each catalog episode should use a distinct entry");
    assert!(summary.episodes_by_entry.values().all(|episodes| *episodes == 1), "catalog entry was reused: {:?}", summary.episodes_by_entry);
    assert_eq!(summary.episodes_by_character.len(), 4);
    for character in ["ironclad", "silent", "defect", "watcher"] {
        assert!(summary.episodes_by_character.get(character).is_some_and(|episodes| *episodes > 0), "missing character {character}");
    }
    for policy in POLICIES { assert!(summary.episodes_by_policy.get(policy.name()).is_some_and(|episodes| *episodes > 0), "missing policy {}", policy.name()); }
    for profile in RUN_PROFILES {
        assert_eq!(summary.episodes_by_profile.get(profile.id), Some(&1), "missing profile {}", profile.id);
        let outcome = summary.outcomes_by_profile.get(profile.id).expect("Run profile outcome");
        let faulted_before_combat = summary.failures.iter().any(|failure| {
            failure.driver == "run" && failure.driver_config["profile"] == profile.id
        });
        assert!(outcome.combat_episodes > 0 || faulted_before_combat,
            "Run profile {} did not reach combat and has no recorded engine failure", profile.id);
    }
    for profile in COMPLETE_ITEMS_RUN_PROFILES {
        assert_eq!(summary.episodes_by_profile.get(profile.id), Some(&1), "missing profile {}", profile.id);
    }
}

fn assert_smoke_failures_are_engine_repros(summary: &RunSummary, run: &str) {
    let output = std::env::temp_dir().join(format!("spire-fuzz-smoke-diagnostics-{}-{run}", std::process::id()));
    let _ = fs::remove_dir_all(&output);
    for failure in &summary.failures {
        assert_engine_failure_is_reproduced(failure, &output, &failure_details(summary));
    }
    let _ = fs::remove_dir_all(output);
}

fn assert_engine_failure_is_reproduced(failure: &spire_fuzz::FuzzFailure, output: &std::path::Path, context: &str) {
    assert!(!matches!(failure.kind.as_str(), "determinism" | "replay" | "panic"),
        "harness integrity failure must be zero: {context}");
    assert!(matches!(failure.kind.as_str(), "fault" | "validator" | "legal_rejected" | "illegal_accepted"
        | "illegal_corrupts" | "encode" | "stuck" | "livelock"),
        "unclassified smoke failure {}: {}", failure.kind, failure.message);
    assert!(matches!(failure.driver.as_str(), "catalog" | "complete" | "run" | "items"));
    assert!(failure.driver_config.is_object());
    assert_eq!(failure.feature_set, "diagnostics");
    assert!(!failure.message.trim().is_empty());
    assert!(failure.step <= failure.actions.len(), "failure step exceeds its saved action prefix");
    let path = write_failure(output, failure).expect("write smoke repro");
    assert!(path.is_file(), "missing written repro {}", path.display());
    assert!(path.with_file_name("beads.md").is_file(), "missing repro report");
    let repro = read_repro(&path).expect("parse written repro");
    assert_eq!(repro.expected_kind, failure.kind);
    assert_eq!(repro.driver, failure.driver);
    run_repro(&repro).unwrap_or_else(|error| panic!("smoke repro failed to re-detect {} seed={}: {error}; {context}", failure.kind, failure.seed));
}

#[test]
fn complete_items_run_starts_each_character_at_the_act4_map_without_an_encoder() {
    for (index, profile) in COMPLETE_ITEMS_RUN_PROFILES.iter().copied().enumerate() {
        let mut driver = CompleteItemsRun::new(profile);
        let decision = driver.reset(307 + index as u64).expect("M27 complete-items start");
        let description = driver.describe();
        assert_eq!(description["starts_at"], "Act4 map");
        assert_eq!(description["neow"], false);
        assert_eq!(description["encode"], "n/a");
        assert_eq!(description["character"], profile.character.as_str());
        let inner = driver.inner_run().expect("read-only inner Run");
        let public = inner.decision().expect("M27 public decision");
        assert_eq!(public.act, 4, "CompleteItemsRun begins in Act 4");
        assert_eq!(public.room.as_ref().map(|room| room.kind), Some(spire_core::run::RoomViewKind::Map));
        assert!(decision.observation.groups.is_empty(), "no matching observation encoder is declared");
        assert!(!decision.observation.action_mask.is_empty(), "Act4 map should publish actions");
        let supply = &description["supply"];
        let relics = supply["relics"].as_array().expect("generated relic supply");
        let potions = supply["potions"].as_array().expect("generated potion supply");
        let relic_names = relics.iter().filter_map(|name| name.as_str()).collect::<std::collections::BTreeSet<_>>();
        assert_eq!(relic_names.len(), relics.len(), "unique relics");
        let relic_ids = spire_core::content::complete::relic_union(profile.character);
        assert!(relics.iter().all(|name| name.as_str().is_some_and(|name| relic_ids.iter().any(|id| id.as_str() == name))));
        let potion_ids = spire_core::content::complete::potion_union(profile.character);
        assert!(potions.iter().filter_map(|name| name.as_str()).all(|name| potion_ids.iter().any(|id| id.as_str() == name)));
        let has_potion_belt = relics.iter().any(|name| name.as_str() == Some("potion_belt"));
        assert_eq!(potions.len(), if has_potion_belt { 5 } else { 3 }, "complete potion capacity");
        assert!(supply["counters"].as_object().expect("generated counters").keys().all(|id| relics.iter().any(|relic| relic.as_str() == Some(id))));
        driver.validate().expect("complete items Run validates");
        driver.descriptor_stable().expect("driver description is stable");
        for index in decision.observation.action_mask.iter().enumerate().filter_map(|(index, legal)| (*legal != 0).then_some(index)) {
            driver.preflight(index).expect("advertised action passes preflight");
        }
        let mut replay = driver.fresh();
        let replay_decision = replay.reset(307 + index as u64).expect("same M27 complete-items start");
        assert_eq!(decision, replay_decision, "deterministic Act4 start");
        assert_eq!(description, replay.describe(), "deterministic catalog-derived supply");
    }
}

#[test]
fn a_settled_complete_combat_is_not_reported_as_capped() {
    let deck = [CardId::Strike; 5].into_iter().chain([CardId::Defend; 4]).chain([CardId::Bash])
        .enumerate().map(|(index, id)| CardInstance { id, instance: index as u64 + 1, upgrades: 0, misc: 0 }).collect();
    let start = AuthoredRunStart { hp: 80, max_hp: 80, gold: 0, deck, relics: vec![], potions: vec![None; 3], relic_counters: Default::default() };
    let options = EpisodeOptions { max_steps: 2_000, all_legal_every: 0, determinism: false, replay: false,
        shrink_budget: Duration::ZERO, scripted_actions: None, continue_first: false };
    let result = run_episode(Box::new(CompleteCombat::from_repro(Character::Ironclad, EncounterId::Cultist, start)),
        9, POLICIES[1], options);
    if let Some(failure) = &result.failure {
        let output = std::env::temp_dir().join(format!("spire-fuzz-settled-combat-{}", std::process::id()));
        let _ = fs::remove_dir_all(&output);
        assert_engine_failure_is_reproduced(failure, &output, "CompleteCombat settled-combat case");
        let _ = fs::remove_dir_all(output);
        return;
    }
    assert_eq!(result.capped, 0, "settled combat must be a terminal episode");
    assert_eq!(result.wins + result.deaths, 1, "the settled combat outcome must be counted");
}

#[test]
fn choose_none_escapes_a_repeated_public_selection_boundary() {
    let options = EpisodeOptions { max_steps: 1_000, all_legal_every: 0, determinism: false, replay: false,
        shrink_budget: Duration::ZERO, scripted_actions: None, continue_first: false };
    for (profile_id, seed) in [
        ("CompleteItemsDefect", 188_960_708_781_607_937),
        ("CompleteItemsIronclad", 93_543_106_880_423_687),
        ("CompleteItemsSilent", 513_095_039_450_759_383),
        ("CompleteItemsWatcher", 1_528_711_875_150_632_422),
    ] {
        let profile = COMPLETE_ITEMS_RUN_PROFILES.iter().find(|profile| profile.id == profile_id).unwrap();
        let result = run_episode(Box::new(CompleteItemsRun::new(*profile)), seed, spire_fuzz::Policy::ChooseNone, options.clone());
        if let Some(failure) = &result.failure {
            let output = std::env::temp_dir().join(format!("spire-fuzz-choose-none-{}-{}", std::process::id(), profile_id));
            let _ = fs::remove_dir_all(&output);
            assert_engine_failure_is_reproduced(failure, &output, profile_id);
            let _ = fs::remove_dir_all(output);
        }
        assert_eq!(result.capped, 0, "ChooseNone must leave repeated selection boundaries: {profile_id} seed={seed}");
    }
}

fn failure_details(summary: &RunSummary) -> String {
    summary.failures.iter().map(|failure| format!("{} seed={} policy={} step={} {}: {}", failure.driver,
        failure.seed, failure.policy.name(), failure.step, failure.kind, failure.message)).collect::<Vec<_>>().join("\n")
}
