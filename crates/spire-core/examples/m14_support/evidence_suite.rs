//! One predeclared normal evidence table for the frozen M14 v5 caller.
//!
//! The original six rows come directly from the adapter's fixed table. Seeds
//! 12 through 23 reuse its two ordered triple patterns, alternating by parity.
const fn build_evidence_cases() -> [(u64, super::Scenario); 42] {
    let mut cases = [super::FIXED_CASES[0]; 42];

    let mut index = 0;
    while index < super::FIXED_CASES.len() {
        cases[index] = super::FIXED_CASES[index];
        index += 1;
    }

    let mut seed = 12;
    let mut output = super::FIXED_CASES.len();
    while seed <= 23 {
        let pattern_start = if seed % 2 == 0 { 0 } else { 3 };
        let mut pattern_index = 0;
        while pattern_index < 3 {
            cases[output] = (
                seed,
                super::FIXED_CASES[pattern_start + pattern_index].1,
            );
            output += 1;
            pattern_index += 1;
        }
        seed += 1;
    }

    cases
}

/// Exact normal-start input rows for the approved frozen-v5 evidence suite.
pub const EVIDENCE_CASES: [(u64, super::Scenario); 42] = build_evidence_cases();
