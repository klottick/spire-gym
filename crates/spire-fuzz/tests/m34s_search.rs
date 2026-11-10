#![cfg(feature = "diagnostics")]

use spire_fuzz::{run_episode, run_search_mode, search_selection, Driver, EpisodeOptions, Policy, RunSummary, SearchDriver, SearchFuzzStats};

#[test]
fn search_selection_covers_characters_ascensions_and_standalone_encounters() {
    let mut characters = std::collections::BTreeSet::new();
    let mut ascensions = std::collections::BTreeSet::new();
    let mut encounters = std::collections::BTreeSet::new();
    for seed in 0..512 {
        let selection = search_selection(seed);
        let repeat = search_selection(seed);
        assert_eq!(selection, repeat, "selection must replay from its seed");
        characters.insert(selection.character.as_str());
        ascensions.insert(selection.ascension);
        encounters.insert(selection.standalone_encounter);
    }
    assert_eq!(characters, spire_core::content::Character::ALL.into_iter().map(|character| character.as_str()).collect());
    assert_eq!(ascensions, [0, 20].into_iter().collect());
    assert_eq!(encounters, ["two_louse", "three_darklings", "gremlin_nob"].into_iter().collect());
}

#[test]
fn public_search_random_branches_preserve_live_sessions_and_public_beliefs() {
    let mut total = SearchFuzzStats::default();
    for seed in 1..=8 {
        let first = run_search_mode(seed).unwrap_or_else(|error| panic!("seed {seed}: {error}"));
        let replay = run_search_mode(seed).unwrap_or_else(|error| panic!("repeat seed {seed}: {error}"));
        assert_eq!(first, replay, "search fuzz is deterministic for seed {seed}");
        assert!(first.searches > 0, "no search in seed {seed}: {first:?}");
        assert!(first.branches >= first.searches * 3, "missing branch coverage: {first:?}");
        assert!(first.steps >= first.searches * 2, "missing Full/Mask steps: {first:?}");
        assert!(first.run_combat_decisions <= 40);
        assert!(first.standalone_combat_decisions <= 40);
        assert!(first.run_combats_seen <= 2);
        total.searches += first.searches;
        total.branches += first.branches;
        total.steps += first.steps;
        total.perturbations_applied += first.perturbations_applied;
        for (reason, count) in first.unsupported_roots { *total.unsupported_roots.entry(reason).or_default() += count; }
    }
    assert!(total.perturbations_applied > 0, "no hidden perturbation applied: {total:?}");
    println!("m34s_search_fuzz searches={} branches={} steps={} perturbations_applied={} unsupported_roots={:?}",
        total.searches, total.branches, total.steps, total.perturbations_applied, total.unsupported_roots);
}

#[test]
fn search_driver_writes_seed_stats_to_the_regular_summary() {
    let seed = 41;
    let driver = SearchDriver::new();
    assert_eq!(driver.class(), "search");
    assert_eq!(driver.describe()["driver"], "search");
    let options = EpisodeOptions { replay: false, ..EpisodeOptions::default() };
    let result = run_episode(Box::new(driver), seed, Policy::First, options);
    assert!(result.failure.is_none(), "unexpected search driver failure: {:?}", result.failure);
    let mut summary = RunSummary::new("test-revision".into());
    summary.add(result);
    let serialized = serde_json::to_value(&summary).unwrap();
    let stats = &serialized["search_fuzz_by_seed"][seed.to_string()];
    assert!(stats["searches"].as_u64().unwrap() > 0);
    assert!(stats["branches"].as_u64().unwrap() > 0);
    assert!(stats["steps"].as_u64().unwrap() > 0);
    assert!(stats["perturbations_applied"].as_u64().unwrap() > 0);
    assert!(stats["unsupported_roots"].is_object());
    assert_eq!(stats["standalone_encounter"], search_selection(seed).standalone_encounter);
}
