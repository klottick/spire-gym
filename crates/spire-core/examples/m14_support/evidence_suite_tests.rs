//! Supplied case-table checks for the frozen M14 v5 evidence suite.
//!
//! These inspect only the declared public inputs; they construct no Run and
//! execute no episode.
use super::*;

const EXPECTED_ORIGINAL_SIX: [(u64, Scenario); 6] = [
    (7, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::Champ, act3_boss: EncounterId::AwakenedOne }),
    (7, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::TimeEater }),
    (7, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Collector, act3_boss: EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::DonuAndDeca }),
    (11, Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::Collector, act3_boss: EncounterId::AwakenedOne }),
    (11, Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Champ, act3_boss: EncounterId::TimeEater }),
];

const EVEN_SEED_PATTERN: [Scenario; 3] = [
    EXPECTED_ORIGINAL_SIX[0].1,
    EXPECTED_ORIGINAL_SIX[1].1,
    EXPECTED_ORIGINAL_SIX[2].1,
];

const ODD_SEED_PATTERN: [Scenario; 3] = [
    EXPECTED_ORIGINAL_SIX[3].1,
    EXPECTED_ORIGINAL_SIX[4].1,
    EXPECTED_ORIGINAL_SIX[5].1,
];

const EXPECTED_SEEDS: [u64; 14] = [
    7, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
];

fn expected_pattern(seed: u64) -> &'static [Scenario; 3] {
    if seed == 7 || (seed >= 12 && seed % 2 == 0) {
        &EVEN_SEED_PATTERN
    } else {
        &ODD_SEED_PATTERN
    }
}

#[test]
fn m14_evidence_suite_has_42_unique_rows_and_preserves_the_original_six_prefix() {
    let cases = evidence_cases();

    assert_eq!(cases.len(), 42);
    assert_eq!(&cases[..EXPECTED_ORIGINAL_SIX.len()], &EXPECTED_ORIGINAL_SIX);
    for (index, case) in cases.iter().enumerate() {
        assert!(!cases[..index].contains(case), "duplicate case row {case:?}");
    }
    assert!(cases.windows(2).all(|pair| pair[0].0 <= pair[1].0));
    if let Some(path) = std::env::var_os("SPIRE_M14_CASE_TABLE_PATH") {
        let rows = cases.iter().enumerate().map(|(index, (seed, scenario))| {
            serde_json::json!({
                "row": index + 1, "seed": seed,
                "act1": format!("{:?}", scenario.act1_boss),
                "act2": format!("{:?}", scenario.act2_boss),
                "act3": format!("{:?}", scenario.act3_boss),
                "action_budget": M14_ACTION_LIMIT_MAX,
            })
        }).collect::<Vec<_>>();
        let file = std::fs::OpenOptions::new().write(true).create_new(true)
            .open(path).expect("new retained case-table receipt");
        serde_json::to_writer_pretty(file, &rows).expect("write compiled case-table receipt");
    }
}

#[test]
fn m14_evidence_suite_has_exact_seeds_three_ordered_triples_and_alternating_patterns() {
    let cases = evidence_cases();
    let mut actual_seeds = Vec::new();
    for (seed, _) in cases {
        if actual_seeds.last() != Some(seed) {
            actual_seeds.push(*seed);
        }
    }
    assert_eq!(actual_seeds.as_slice(), EXPECTED_SEEDS.as_slice());

    for seed in EXPECTED_SEEDS {
        let table_triples = cases
            .iter()
            .filter_map(|(case_seed, scenario)| (*case_seed == seed).then_some(*scenario))
            .collect::<Vec<_>>();
        assert_eq!(table_triples.len(), 3, "seed {seed}");
        assert_eq!(table_triples.as_slice(), expected_pattern(seed).as_slice(), "table order for seed {seed}");

        let supplied_triples = cases_for_seed(seed).unwrap_or_else(|error| {
            panic!("declared evidence seed {seed} rejected: {error}")
        });
        assert_eq!(supplied_triples, table_triples, "cases_for_seed({seed}) must match the table");
    }

    assert!(cases_for_seed(24).is_err(), "the declared suite ends at seed 23");
}

#[test]
fn m14_evidence_suite_keeps_the_8192_action_limit() {
    assert_eq!(M14_ACTION_LIMIT_MAX, 8192);
}
