//! Run with --scenario all|victory|death|modifiers --seed <u64>.
mod m1_support;
use m1_support::*;
use spire_core::{combat::*, content::{CardId, EnemyId}};

const BUDGET: usize = 64;

fn observe(view: &CombatPublic, menu: &[CombatAction], action: Option<&CombatAction>) {
    println!("turn={} hp={} block={} energy={} player_powers={:?}",
        view.turn, view.player.hp, view.player.block, view.player.energy, view.player.powers);
    for enemy in &view.enemies {
        println!("  {} hp={} block={} intent={} raw_damage={:?} powers={:?}",
            enemy.enemy_id.as_str(), enemy.hp, enemy.block, enemy.intent.as_str(), enemy.intent_damage, enemy.powers);
    }
    println!("  hand={:?} draw_count={} menu={menu:?} action={action:?} outcome={:?}",
        view.hand.iter().map(|c| (c.instance.0, c.id.as_str())).collect::<Vec<_>>(), view.draw_count, view.outcome);
}

fn take(combat: &mut Combat, action: CombatAction, accepted: &mut usize) -> Result<(), DemoError> {
    if *accepted >= BUDGET { return Err(DemoError::BudgetExceeded { accepted: *accepted }); }
    let menu = combat.legal_actions_full()?;
    if !menu.contains(&action) { return Err(spire_core::run::RunError::InvalidAction.into()); }
    observe(&combat.public_full()?, &menu, Some(&action));
    combat.step_full(&action)?;
    *accepted += 1;
    observe(&combat.public_full()?, &combat.legal_actions_full()?, None);
    Ok(())
}

fn take_card(combat: &mut Combat, id: CardId, accepted: &mut usize) -> Result<(), DemoError> {
    take(combat, card_action(combat, id)?, accepted)
}

fn modifiers(seed: u64) -> Result<(), DemoError> {
    let mut accepted = 0;
    println!("modifiers: player Bash → Vulnerable → Strike");
    let mut combat = Combat::enter(fixture(EnemyId::JawWorm, seed))?;
    take_card(&mut combat, CardId::Bash, &mut accepted)?;
    take_card(&mut combat, CardId::Strike, &mut accepted)?;

    for weak in [false, true] {
        println!("modifiers: Jaw Chomp against Defend, player Clothesline Weak={weak}");
        let mut combat = Combat::enter(fixture(EnemyId::JawWorm, seed))?;
        if weak { take_card(&mut combat, CardId::Clothesline, &mut accepted)?; }
        take_card(&mut combat, CardId::Defend, &mut accepted)?;
        take(&mut combat, CombatAction::EndTurn, &mut accepted)?;
    }

    println!("modifiers: actual Green Louse SpitWeb → player Weak → Strike/Bash (including CurlUp Block)");
    let mut combat = Combat::enter(fixture(EnemyId::LouseGreen, seed))?;
    while !combat.public_full()?.player.powers.iter().any(|p| p.id.0 == "weak") {
        take(&mut combat, CombatAction::EndTurn, &mut accepted)?;
    }
    take_card(&mut combat, CardId::Strike, &mut accepted)?;
    take_card(&mut combat, CardId::Bash, &mut accepted)?;

    println!("modifiers: actual Nob Bellow/SkullBash → player Vulnerable → BullRush; no Enrage Skills");
    let mut combat = Combat::enter(fixture(EnemyId::GremlinNob, seed))?;
    while !combat.public_full()?.player.powers.iter().any(|p| p.id.0 == "vulnerable")
        || combat.public_full()?.enemies[0].intent != FullIntentId::GremlinNobBullRush {
        take(&mut combat, CombatAction::EndTurn, &mut accepted)?;
    }
    take(&mut combat, CombatAction::EndTurn, &mut accepted)?;
    println!("modifier checkpoints complete; accepted_actions={accepted} (not a completed run)");
    Ok(())
}

fn main() -> Result<(), String> {
    let mut scenario = "all".to_owned();
    let mut seed = 7_u64;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenario" => scenario = args.next().ok_or("--scenario requires a value")?,
            "--seed" => seed = args.next().ok_or("--seed requires a u64")?.parse()
                .map_err(|_| "--seed requires a u64")?,
            "--help" | "-h" => {
                println!("m1_combat --scenario all|victory|death|modifiers --seed <u64> (default7)");
                return Ok(());
            }
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    if !["all", "victory", "death", "modifiers"].contains(&scenario.as_str()) {
        return Err(format!("unknown scenario {scenario}"));
    }
    println!("M1 authored Ironclad A0 combat, seed={seed}; not a normal run or complete content pool");
    for (name, attack, hp) in [("victory", true, 80), ("death", false, 11)] {
        if scenario == "all" || scenario == name {
            println!("scenario={name}");
            let mut entry = fixture(EnemyId::JawWorm, seed);
            entry.setup.player.hp = hp;
            let mut combat = Combat::enter(entry).map_err(|e| format!("{e:?}"))?;
            let outcome = run_fight(&mut combat, attack, BUDGET, observe).map_err(|e| e.to_string())?;
            println!("scenario={name} actual_outcome={outcome:?}");
        }
    }
    if scenario == "all" || scenario == "modifiers" { modifiers(seed).map_err(|e| e.to_string())?; }
    Ok(())
}
