mod m10_support;

use m10_support::{M10Completion, M9BossHistory};
use spire_core::content::EnemyId;

fn execute() -> Result<(), String> {
    let options = m10_support::parse_args(std::env::args().skip(1))?;
    let histories = [
        M9BossHistory::AwakenedUnseen,
        M9BossHistory::TimeEaterUnseen,
        M9BossHistory::DonuDecaUnseen,
    ];
    let mut reports = Vec::new();
    if options.scenario != "death" {
        for history in histories {
            reports.push((
                format!("{history:?}"),
                m10_support::rollout(options.seed, history, options.budget)?,
            ));
        }
    }
    if options.scenario != "complete" {
        reports.push((
            "nonattacking".into(),
            m10_support::nonattacking(
                options.seed,
                M9BossHistory::AwakenedUnseen,
                options.budget,
            )?,
        ));
    }

    let mut unfinished = false;
    for (label, report) in reports {
        let decision = &report.final_decision;
        let observed_no_keys = report.trace.iter().any(|step| {
            step.decision.terminal == Some(spire_core::run::RunTerminal::Act3NoKeys)
        });
        let observed_shield_spear = report.trace.iter().any(|step| {
            step.decision.act == 4
                && step.decision.floor == 53
                && step.decision.combat.as_ref().is_some_and(|combat| {
                    combat.enemies.iter().any(|enemy| {
                        matches!(enemy.enemy_id, EnemyId::SpireShield | EnemyId::SpireSpear)
                    })
                })
        });
        let observed_heart = report.trace.iter().any(|step| {
            step.decision.act == 4
                && step.decision.floor == 54
                && step.decision.combat.as_ref().is_some_and(|combat| {
                    combat.enemies.iter().any(|enemy| enemy.enemy_id == EnemyId::CorruptHeart)
                })
        });
        let outcome = match report.completion {
            M10Completion::Death => "valid gameplay Death (not a tool error or Heart victory)",
            M10Completion::EmptyDeck => "EmptyDeck loss",
            M10Completion::Act3NoKeys => "actual Act3 no-key ending (not Heart victory)",
            M10Completion::HeartVictory => "actual Heart victory and final Proceed",
            M10Completion::Act4Ready => "unsupported pre-Act4 boundary (not completion)",
            M10Completion::Unfinished => "unfinished caller budget (not a loss or truncation)",
        };
        println!(
            "seed{} {label}: {outcome}; actions{} act{} floor{} HP{}/{} reward{} terminal{:?}",
            options.seed,
            report.trace.len(),
            decision.act,
            decision.floor,
            decision.player.hp,
            decision.player.max_hp,
            decision.reward_total,
            decision.terminal
        );
        println!(
            "  normal-start observations: no-key-ending={observed_no_keys}, Shield/Spear={observed_shield_spear}, Heart={observed_heart}, HeartVictory={}",
            report.completion == M10Completion::HeartVictory
        );
        unfinished |= matches!(report.completion, M10Completion::Unfinished | M10Completion::Act4Ready);
    }
    println!(
        "Supplied native key-source/fourth-choice/Act4 diagnostics are separate from these normal-start episodes; they are not normal-start victory evidence."
    );
    if unfinished {
        Err("unfinished budget or unsupported boundary; not a gameplay outcome".into())
    } else {
        Ok(())
    }
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("M10 Heart: {error}");
        std::process::exit(1);
    }
}
