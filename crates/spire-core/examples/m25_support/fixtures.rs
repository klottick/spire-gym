//! Fixed M25 supplied case order and budgets; no loadout search or outcome tuning.
use super::M25CallerOutcome;

pub const TRAVERSAL_BUDGET: usize = 32768;
pub const ACT4_BUDGET: usize = 8192;
pub const FIXTURE_BUDGET: usize = 128;

/// The strong Act4 case runs only if all three primary cases completed without
/// an operational error and none reached HeartVictory.
pub fn should_run_strong(
    primary: &[Result<M25CallerOutcome, String>],
) -> Result<bool, String> {
    for (index, result) in primary.iter().enumerate() {
        if let Err(error) = result {
            return Err(format!(
                "primary Act4 case {} has an operational error; diagnose before strong case: {error}",
                index + 1
            ));
        }
    }
    Ok(!primary
        .iter()
        .any(|result| matches!(result, Ok(M25CallerOutcome::HeartVictory))))
}

/// Freeze v1, whose once-only supplied series stopped at fault F1 (Act4 coverage B).
pub const V1_FREEZE_SHA256: &str = "b0e4c5ffe6d021fb0d59fde36daed7294fe54dc2782c27581f4605c4639fa72c";
/// The completed v1 primary Act4 receipts the F1 corrective series reuses (file, SHA-256), in the
/// primary order A, C around the re-executed coverage B.
pub const V1_RETAINED_PRIMARY: [(&str, &str); 2] = [
    ("seed-7-supplied-act4-coverage-a-budget-8192.attempt.json", "ebd55ccc79af62a34718a21b3f2aeed69257701833bb31d3a24627066547cc36"),
    ("seed-7-supplied-act4-coverage-c-budget-8192.attempt.json", "1b86608b8ecff67ff216ffd48e6fc75fc730ea8d09cf5680b7ca2f1372dcc6dd"),
];

/// Freeze v2 (the F1 corrective), whose coverage-B receipt the G1 corrective reuses.
pub const V2_FREEZE_SHA256: &str = "879fbb1be3fb396802c315916052494768265df93ba21d5461ace735f03f344e";
/// The completed v2 coverage-B receipt (file, SHA-256) reused by the G1 corrective.
pub const V2_RETAINED_B: (&str, &str) =
    ("seed-7-supplied-act4-coverage-b-budget-8192.attempt.json", "35a35ec6b54df3e25a589507200d462ca797a81ec859a44715ff7d37d6147ab5");

/// Which supplied series one invocation runs. `Full` is the frozen v1 order; `F1Corrective` runs
/// only coverage B, the strong-case rule over A (v1), B and C (v1), and the eight fixtures;
/// `G1Corrective` runs only coverage C and evaluates the strong-case rule over A (v1), B (v2) and
/// C for information, without executing the strong case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Series {
    Full,
    F1Corrective { v1_supplied: std::path::PathBuf },
    G1Corrective { v1_supplied: std::path::PathBuf, v2_supplied: std::path::PathBuf },
}

/// Reject anything but no arguments or the exact corrective form, before any Run is built.
pub fn parse_series(args: &[String]) -> Result<Series, String> {
    match args {
        [] => Ok(Series::Full),
        [series, name, flag, dir] if series == "--series" && name == "f1-corrective" && flag == "--v1-supplied" => {
            Ok(Series::F1Corrective { v1_supplied: dir.into() })
        }
        [series, name, flag1, dir1, flag2, dir2]
            if series == "--series" && name == "g1-corrective" && flag1 == "--v1-supplied" && flag2 == "--v2-supplied" =>
        {
            Ok(Series::G1Corrective { v1_supplied: dir1.into(), v2_supplied: dir2.into() })
        }
        _ => Err("usage: m25_supplied_diagnostics [--series f1-corrective --v1-supplied DIR | --series g1-corrective --v1-supplied DIR --v2-supplied DIR]".into()),
    }
}

pub fn outcome_from_name(name: &str) -> Option<M25CallerOutcome> {
    use M25CallerOutcome::*;
    [HeartVictory, Act3NoKeys, Death, EmptyDeck, BudgetExhausted]
        .into_iter()
        .find(|outcome| outcome.to_string() == name)
}

/// One retained v1 receipt as the strong-case rule saw it: its exact bytes, its freeze and its
/// recorded outcome or operational error.
pub fn retained_outcome(
    bytes: &[u8],
    expected_sha256: &str,
) -> Result<Result<M25CallerOutcome, String>, String> {
    retained_outcome_from(bytes, expected_sha256, V1_FREEZE_SHA256)
}

/// As `retained_outcome`, for a receipt of the named freeze.
pub fn retained_outcome_from(
    bytes: &[u8],
    expected_sha256: &str,
    freeze_sha256: &str,
) -> Result<Result<M25CallerOutcome, String>, String> {
    use sha2::{Digest, Sha256};
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual != expected_sha256 {
        return Err(format!("retained receipt SHA-256 {actual} is not {expected_sha256}"));
    }
    let receipt: serde_json::Value = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
    if receipt["schema_id"] != super::RECEIPT_SCHEMA_ID || receipt["source_freeze_sha256"] != freeze_sha256 {
        return Err("retained receipt is not an M25 attempt of the expected freeze".into());
    }
    match (receipt["outcome"].as_str(), receipt["error"].as_str()) {
        (Some(name), None) => outcome_from_name(name)
            .map(Ok)
            .ok_or_else(|| format!("unknown retained outcome {name}")),
        (None, Some(error)) => Ok(Err(error.to_owned())),
        _ => Err("retained receipt has neither one outcome nor one error".into()),
    }
}
