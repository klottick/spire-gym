mod m14_support;

use m14_support::{Caller, CallerOutcome, Scenario};
use spire_core::run::Run;

fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn execute() -> Result<(), String> {
    let options = m14_support::parse_args(std::env::args().skip(1))?;
    let mut errors = Vec::new();
    for scenario in options.scenarios {
        let mut caller = Caller::new_m14(options.limit)?;
        let mut run = None;
        let attempt = match m14_support::make_run(options.seed, scenario) {
            Ok(mut created) => {
                let result = m14_support::rollout_m14(&mut created, &mut caller, options.nonattacking);
                run = Some(created);
                result
            }
            Err(error) => Err(format!("M14 run creation: {error}")),
        };

        if let Err(error) = m14_support::persist_attempt_trace(options.seed, scenario, &attempt, &caller, run.as_ref()) {
            errors.push(format!("seed={} scenario={scenario:?} trace export: {error}", options.seed));
        }
        print_summary(options.seed, scenario, run.as_ref(), &caller, &attempt);
        if let Err(error) = &attempt {
            errors.push(format!("seed={} scenario={scenario:?}: {error}; accepted_actions={} endpoint={:?} failed_action={:?}",
                options.seed,
                caller.accepted(),
                m14_support::endpoint_status(run.as_ref(), &caller),
                caller.failed_action_for_diagnostics(),
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("one or more M14 cases failed:\n{}", errors.join("\n")))
    }
}

fn print_summary(
    seed: u64,
    scenario: Scenario,
    run: Option<&Run>,
    caller: &Caller,
    attempt: &Result<CallerOutcome, String>,
) {
    let live_decision = run.and_then(|run| run.decision().ok());
    let final_decision = if attempt.is_err() {
        live_decision.as_ref().or(caller.final_decision.as_ref())
    } else {
        caller.final_decision.as_ref().or(live_decision.as_ref())
    };
    let summary = m14_support::m13_support::summarize(caller);
    let (act, floor, hp, max_hp) = final_decision.map_or((0, 0, 0, 0), |decision| {
        (decision.act, decision.floor, decision.player.hp, decision.player.max_hp)
    });
    let boss = caller.final_map.as_ref().and_then(|map| map.boss);
    let outcome = match attempt {
        Ok(outcome) => outcome.to_string(),
        Err(error) => format!("Err({error:?})"),
    };
    println!(
        "seed={seed} caller_policy={} scenario={:?} requested_boss=({:?},{:?},{:?}) outcome={} accepted_actions={} endpoint=({act},{floor}) endpoint_detail={:?} hp={hp}/{max_hp} boss={boss:?} ordinary={} elite={} boss_moves={} reward_screens={} card_rewards={} chests={} boss_rewards={} boss_entries={:?} boss_records={:?} failed_action={:?}",
        m14_support::CALLER_POLICY_REVISION,
        scenario.label(),
        scenario.act1_boss,
        scenario.act2_boss,
        scenario.act3_boss,
        outcome,
        caller.accepted(),
        m14_support::endpoint_status(run, caller),
        summary.ordinary_moves,
        summary.elite_moves,
        summary.boss_moves,
        summary.reward_screens,
        summary.card_reward_screens,
        summary.chests_opened,
        summary.boss_rewards,
        summary.boss_combat_entries,
        summary.boss_combat_records,
        caller.failed_action_for_diagnostics(),
    );
}
