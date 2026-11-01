//! Fixed M25 caller, canonical inputs and loss-preserving attempt receipts.
#[path = "../m24_support/mod.rs"]
pub mod m24;
pub mod fixtures;
pub mod inputs;
pub mod policy;

pub use inputs::{
    make_run, parse_args, triple, Options, Scenario, ACTION_LIMIT_MAX, SCENARIOS, SEEDS,
};
pub use m24::{initial_value, semantic_actions, sha, write_new};
pub use m24::shared::{self, Caller, M25CallerOutcome};
pub use policy::{choose, PublicCallerProgress};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::run::{GameAction, PublicDecision, Run, RunError, map::PublicMap};
use std::path::{Path, PathBuf};

pub const CALLER_POLICY_REVISION: &str = policy::CALLER_POLICY_REVISION;
pub const RECEIPT_SCHEMA_ID: &str = "m25-watcher-heart-caller-attempt-v1";
pub const TRACE_DIR_ENV: &str = "M25_TRACE_DIR";
pub const FREEZE_SHA256_ENV: &str = "M25_FREEZE_SHA256";
pub const SETTLED: &str = "fixture combat settled";

/// Accepted public progress for the decision's current act.
pub fn caller_policy(
    decision: &PublicDecision,
    map: Option<&PublicMap>,
    caller: &Caller,
) -> Result<GameAction, String> {
    let mut progress = policy::act_progress(
        caller
            .trace
            .iter()
            .map(|step| (&step.decision, &step.action, &step.transition)),
        decision.act,
    );
    // Act4 is a fixed Rest → Shop → Elite → Heart route with no ordinary
    // encounter node. Mark that route constraint as satisfied for M22's shared
    // path scorer, which otherwise requires an ordinary encounter in Acts 1–3.
    if decision.act == 4 {
        progress.ordinary_reached = true;
    }
    policy::choose(decision, map, &progress)
}

/// Supplied interaction fixtures stop after their combat settles.
pub fn fixture_policy(
    decision: &PublicDecision,
    map: Option<&PublicMap>,
    caller: &Caller,
) -> Result<GameAction, String> {
    if decision.combat.is_none() {
        return Err(SETTLED.into());
    }
    caller_policy(decision, map, caller)
}

pub fn attempt_path(
    directory: &Path,
    seed: u64,
    scenario: Scenario,
    label: &str,
    budget: usize,
) -> PathBuf {
    directory.join(format!(
        "seed-{seed}-{}-{label}-budget-{budget}.attempt.json",
        scenario.name()
    ))
}

pub fn case(
    act1: spire_core::content::EncounterId,
    act2: spire_core::content::EncounterId,
    scenario: Scenario,
) -> Value {
    json!({
        "act1_boss": act1.as_str(),
        "act2_boss": act2.as_str(),
        "act3_boss": scenario.act3().as_str(),
    })
}

/// Exact argv and the SHA-256 of the executable when an attempt is run.
pub fn invocation() -> Value {
    let executable = std::env::current_exe().ok();
    let hash = executable
        .as_ref()
        .and_then(|path| std::fs::read(path).ok())
        .map(|bytes| format!("{:x}", Sha256::digest(&bytes)));
    json!({
        "argv": std::env::args().collect::<Vec<_>>(),
        "executable": executable.map(|path| path.display().to_string()),
        "executable_sha256": hash,
    })
}

/// Preserve the common M23 receipt shape with the M25 schema and outcome.
pub fn persist(
    path: &Path,
    seed: u64,
    scenario: Value,
    budget: usize,
    label: &str,
    freeze_sha256: &str,
    initial: &Value,
    caller: &Caller,
    run: Option<&Run>,
    result: &Result<M25CallerOutcome, String>,
) -> Result<(), String> {
    let text_result = result
        .as_ref()
        .map(ToString::to_string)
        .map_err(Clone::clone);
    let mut receipt = m24::m23::receipt(
        RECEIPT_SCHEMA_ID,
        CALLER_POLICY_REVISION,
        seed,
        scenario,
        budget,
        label,
        freeze_sha256,
        initial,
        caller,
        run,
        &text_result,
    );
    receipt["invocation"] = invocation();
    write_new(path, &receipt)
}

pub fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Bootstrap one supplied combat Move outside the frozen caller's budget and
/// action count, retaining the accepted transition beside the initial snapshot.
#[cfg(feature = "m25-diagnostics")]
pub fn bootstrap_supplied(run: &mut Run, initial: &mut Value) -> Result<(), String> {
    use spire_core::run::diagnostics;

    initial["diagnostic_checkpoint_before_bootstrap"] = json!(format!(
        "{:?}",
        diagnostics::m22_diagnostic_checkpoint(run).map_err(|error| format!("{error:?}"))?
    ));
    let action = run
        .decision()
        .map_err(|error| format!("{error:?}"))?
        .candidates
        .into_iter()
        .find(|candidate| matches!(candidate.action, GameAction::Move(_)))
        .ok_or("missing fixed supplied entry")?
        .action;
    let entry = run.step_observed(&action);
    let successful = entry.result.is_ok();
    let accepted = successful || matches!(&entry.result, Err(RunError::Faulted(_)));
    initial["supplied_entry"] = json!({
        "scope": "supplied encounter bootstrap; excluded from policy budget/count",
        "accepted": accepted,
        "successful": successful,
        "action": semantic_actions::action_value(&action),
        "post_diagnostic_checkpoint_debug": format!("{:?}", diagnostics::m22_diagnostic_checkpoint(run)),
        "transition_debug": format!("{:?}", entry.result),
        "effects_debug": format!("{:?}", entry.effects),
        "boundaries_debug": format!("{:?}", entry.boundaries),
        "post_public_debug": format!("{:?}", run.decision()),
        "map_debug": format!("{:?}", run.public_map()),
    });
    entry
        .result
        .map(|_| ())
        .map_err(|error| format!("fixed supplied entry: {error:?}"))
}
