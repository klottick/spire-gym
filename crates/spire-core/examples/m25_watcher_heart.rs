//! Fixed M25 canonical cases and controls over the shared caller loop.
mod m25_support;

use m25_support::*;
use std::path::PathBuf;

fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn execute() -> Result<(), String> {
    // Reject every malformed value before constructing the first Run.
    let options = parse_args(std::env::args().skip(1))?;
    let directory = std::env::var_os(TRACE_DIR_ENV)
        .map(PathBuf::from)
        .ok_or("M25_TRACE_DIR is required for durable receipts")?;
    let freeze = std::env::var(FREEZE_SHA256_ENV)
        .map_err(|_| "M25_FREEZE_SHA256 is required")?;
    if !valid_sha256(&freeze) {
        return Err("M25_FREEZE_SHA256 must be 64 hexadecimal characters".into());
    }
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;

    let label = if options.nonattacking {
        "death-control"
    } else if options.budget < ACTION_LIMIT_MAX {
        "budget-control"
    } else {
        "normal"
    };
    for scenario in &options.scenarios {
        let path = attempt_path(
            &directory,
            options.seed,
            *scenario,
            label,
            options.budget,
        );
        match std::fs::symlink_metadata(&path) {
            Ok(_) => return Err(format!("refusing existing M25 attempt {}", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(format!("inspect {}: {error}", path.display())),
        }
    }

    let mut failures = Vec::new();
    for scenario in options.scenarios {
        let (act1, act2, _) = triple(options.seed, scenario, options.nonattacking)?;
        let path = attempt_path(
            &directory,
            options.seed,
            scenario,
            label,
            options.budget,
        );
        let mut caller = Caller::new_m25(options.budget)?;
        caller.nonattacking = options.nonattacking;
        let mut created = make_run(options.seed, scenario, options.nonattacking);
        let (attempt, initial) = match created.as_mut() {
            Ok(run) => match initial_value(run) {
                Ok(initial) => (
                    shared::rollout_m25(run, &mut caller, caller_policy),
                    initial,
                ),
                Err(error) => (Err(format!("M25 initial public snapshot: {error}")), serde_json::Value::Null),
            },
            Err(error) => (Err(format!("M25 Run creation: {error}")), serde_json::Value::Null),
        };
        let run = created.as_ref().ok();
        persist(
            &path,
            options.seed,
            case(act1, act2, scenario),
            options.budget,
            label,
            &freeze,
            &initial,
            &caller,
            run,
            &attempt,
        )?;
        println!(
            "seed={} act1={} act2={} act3={} result={attempt:?} accepted_actions={} endpoint={:?}",
            options.seed,
            act1.as_str(),
            act2.as_str(),
            scenario.name(),
            caller.accepted(),
            run.and_then(|run| run.decision().ok().map(|decision| (decision.act, decision.floor, decision.terminal))),
        );
        match &attempt {
            Err(error) => failures.push(format!(
                "seed={} scenario={scenario:?}: {error}; accepted_actions={} failed_action={:?}",
                options.seed,
                caller.accepted(),
                caller.failed_action_for_diagnostics(),
            )),
            Ok(M25CallerOutcome::BudgetExhausted) => failures.push(format!(
                "seed={} scenario={scenario:?}: caller budget exhausted at {}",
                options.seed, options.budget,
            )),
            Ok(_) => (),
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
