mod m20_support;

use m20_support::{Caller, M20CallerOutcome, Scenario};
use spire_core::run::Run;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() == ["--contract"] {
        print!("{}", m20_support::contract_text());
        return;
    }
    if let Err(error) = execute(args) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn execute(args: Vec<String>) -> Result<(), String> {
    let options = m20_support::parse_args(args)?;
    let (directory, freeze_sha256) = m20_support::required_execution_environment()?;
    m20_support::preflight_attempt_files(&directory, &options.scenarios, options.seed)?;
    println!("source_freeze_sha256={freeze_sha256}");

    let mut errors = Vec::new();
    for scenario in options.scenarios {
        let mut caller = Caller::new_m20(options.limit)?;
        let mut run: Option<Run> = None;
        let (attempt, provenance) = match m20_support::make_run(options.seed, scenario) {
            Ok(mut created) => {
                // Bind the initial source snapshot before the shared rollout
                // can accept any action or change Run state. If the public
                // initial snapshot itself fails, retain that as this case's
                // error and continue to the next selected case.
                let (result, provenance) = match m20_support::normal_provenance(&created) {
                    Ok(provenance) => (
                        m20_support::shared::rollout_m20(
                            &mut created,
                            &mut caller,
                            m20_support::policy,
                        ),
                        Some(provenance),
                    ),
                    Err(error) => (
                        Err(format!("M20 initial public snapshot: {error}")),
                        None,
                    ),
                };
                run = Some(created);
                (result, provenance)
            }
            Err(error) => (Err(format!("M20 run creation: {error}")), None),
        };

        if let Some(provenance) = provenance {
            if let Err(error) = m20_support::persist_attempt(
                &directory,
                options.seed,
                scenario,
                options.limit,
                &attempt,
                &caller,
                run.as_ref(),
                &provenance,
            ) {
                errors.push(format!("seed={} scenario={scenario:?} receipt export: {error}", options.seed));
            }
        } else if let Err(error)=&attempt {
            if let Err(export)=m20_support::persist_initialization_error(&directory,options.seed,scenario,options.limit,error){
                errors.push(format!("seed={} scenario={scenario:?} initialization-error export: {export}",options.seed));
            }
        }
        print_summary(options.seed, scenario, run.as_ref(), options.limit, &caller, &attempt);
        match &attempt {
            Err(error) => errors.push(format!(
                "seed={} scenario={scenario:?}: {error}; accepted_actions={} endpoint={} failed_action={:?}",
                options.seed,
                caller.accepted(),
                m20_support::endpoint_summary(run.as_ref(), &caller),
                caller.failed_action_for_diagnostics(),
            )),
            Ok(M20CallerOutcome::BudgetExhausted) => errors.push(format!(
                "seed={} scenario={scenario:?}: caller action budget exhausted at {}",
                options.seed, options.limit,
            )),
            Ok(_) => (),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("one or more M20 attempts were incomplete:\n{}", errors.join("\n")))
    }
}

fn print_summary(
    seed: u64,
    scenario: Scenario,
    run: Option<&Run>,
    budget: usize,
    caller: &Caller,
    attempt: &Result<M20CallerOutcome, String>,
) {
    let outcome = match attempt {
        Ok(outcome) => outcome.to_string(),
        Err(error) => format!("Err({error:?})"),
    };
    println!(
        "seed={seed} caller_policy={} scenario=({:?},{:?},{:?}) outcome={} accepted_actions={} budget={} endpoint={} failed_action={:?}",
        m20_support::CALLER_POLICY_REVISION,
        scenario.act1_boss,
        scenario.act2_boss,
        scenario.act3_boss,
        outcome,
        caller.accepted(),
        budget,
        m20_support::endpoint_summary(run, caller),
        caller.failed_action_for_diagnostics(),
    );
}
