mod m13_support;

use m13_support::{Caller, CallerOutcome};
use spire_core::run::{GameAction, Run};

fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn execute() -> Result<(), String> {
    let options = m13_support::parse_args(std::env::args().skip(1))?;
    for scenario in options.scenarios {
        let mut run = m13_support::make_run(options.seed, scenario)?;
        let mut caller = Caller::new(options.limit)?;
        caller.nonattacking = options.nonattacking;
        let outcome = m13_support::rollout(&mut run, &mut caller, scenario)?;
        print_summary(options.seed, scenario, &run, &caller, outcome);
    }
    Ok(())
}

fn print_summary(
    seed: u64,
    scenario: m13_support::Scenario,
    run: &Run,
    caller: &Caller,
    outcome: CallerOutcome,
) {
    let fallback_decision = caller.final_decision.is_none().then(|| run.decision().ok()).flatten();
    let final_decision = caller.final_decision.as_ref().or(fallback_decision.as_ref());
    let summary = m13_support::summarize(caller);
    let (act, floor, hp, max_hp) = final_decision.map_or((0, 0, 0, 0), |decision| {
        (decision.act, decision.floor, decision.player.hp, decision.player.max_hp)
    });
    let boss = caller.final_map.as_ref().and_then(|map| map.boss);
    println!(
        "seed={seed} caller_policy={} scenario={} requested_boss={:?} outcome={} accepted={} act={act} floor={floor} hp={hp}/{max_hp} boss={boss:?} ordinary={} elite={} boss_moves={} reward_screens={} card_rewards={} chests={} boss_rewards={} boss_entries={:?} boss_records={:?}",
        m13_support::CALLER_POLICY_REVISION, scenario.label(), (scenario.act1_boss, scenario.act2_boss), outcome, caller.accepted(), summary.ordinary_moves, summary.elite_moves,
        summary.boss_moves, summary.reward_screens, summary.card_reward_screens,
        summary.chests_opened, summary.boss_rewards, summary.boss_combat_entries,
        summary.boss_combat_records,
    );
    for step in &caller.trace {
        let room = step.decision.room.as_ref().map(|room| room.kind);
        let summarize_action = !matches!(step.action, GameAction::Combat(_))
            || !step.transition.committed_events.is_empty();
        if summarize_action {
            println!(
                "  floor={} room={room:?} hp={}/{} action={:?} reward={} events={:?}",
                step.decision.floor,
                step.decision.player.hp,
                step.decision.player.max_hp,
                step.action,
                step.transition.reward,
                step.transition.committed_events,
            );
        }
    }
}
