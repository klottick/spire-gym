mod m15_support;

use m15_support::{Caller, M15CallerOutcome, Scenario};
use spire_core::run::Run;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() == ["--contract"] {
        print_contract();
        return;
    }
    if let Err(error) = execute(args) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn print_contract() {
    println!("caller_policy={}", m15_support::CALLER_POLICY_REVISION);
    println!("action_limit_max={}", m15_support::M15_ACTION_LIMIT_MAX);
    println!("case_count={}", m15_support::FIXED_CASES.len());
    println!("receipt_schema=m15-silent-heart-caller-attempt-v1");
    println!("trace_schema=m15-silent-heart-caller-trace-v1");
    println!("trace_dir_env=SPIRE_M15_TRACE_DIR");
}

fn execute(args: Vec<String>) -> Result<(), String> {
    let options = m15_support::parse_args(args)?;
    let directory = std::env::var_os("SPIRE_M15_TRACE_DIR")
        .map(std::path::PathBuf::from)
        .ok_or("set SPIRE_M15_TRACE_DIR to a new retained directory before running M15 attempts")?;
    m15_support::preflight_attempt_files(&directory, &options.scenarios, options.seed)?;

    let mut errors = Vec::new();
    for scenario in options.scenarios {
        let mut caller = Caller::new_m15(options.limit)?;
        let mut run: Option<Run> = None;
        let attempt = match m15_support::make_run(options.seed, scenario) {
            Ok(mut created) => {
                let result = m15_support::rollout_m15(&mut created, &mut caller, m15_support::key_action);
                run = Some(created);
                result
            }
            Err(error) => Err(format!("M15 run creation: {error}")),
        };

        if let Err(error) = m15_support::persist_attempt(
            &directory, options.seed, scenario, options.limit, &attempt, &caller, run.as_ref(),
        ) {
            errors.push(format!("seed={} scenario={scenario:?} trace export: {error}", options.seed));
        }
        print_summary(options.seed, scenario, run.as_ref(), options.limit, &caller, &attempt);
        match &attempt {
            Err(error) => errors.push(format!(
                "seed={} scenario={scenario:?}: {error}; accepted_actions={} endpoint={} failed_action={:?}",
                options.seed,
                caller.accepted(),
                m15_support::endpoint_summary(run.as_ref(), &caller),
                caller.failed_action_for_diagnostics(),
            )),
            Ok(M15CallerOutcome::BudgetExhausted) => errors.push(format!(
                "seed={} scenario={scenario:?}: caller action budget exhausted at {}",
                options.seed, options.limit,
            )),
            Ok(_) => (),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("one or more M15 attempts were incomplete:\n{}", errors.join("\n")))
    }
}

fn print_summary(
    seed: u64,
    scenario: Scenario,
    run: Option<&Run>,
    budget: usize,
    caller: &Caller,
    attempt: &Result<M15CallerOutcome, String>,
) {
    let outcome = match attempt {
        Ok(outcome) => outcome.to_string(),
        Err(error) => format!("Err({error:?})"),
    };
    println!(
        "seed={seed} caller_policy={} scenario=({:?},{:?},{:?}) outcome={} accepted_actions={} budget={} endpoint={} failed_action={:?}",
        m15_support::CALLER_POLICY_REVISION,
        scenario.act1_boss,
        scenario.act2_boss,
        scenario.act3_boss,
        outcome,
        caller.accepted(),
        budget,
        m15_support::endpoint_summary(run, caller),
        caller.failed_action_for_diagnostics(),
    );
}
