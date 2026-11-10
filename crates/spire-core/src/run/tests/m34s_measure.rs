//! M34s-a a6 measurement (ignored; release): public draw-knowledge cost and component counts on supplied Standard
//! routes, 4 characters × {A0, A20}. A measurement, not a gate.
use crate::combat::{CombatAction, Phase};
use crate::combat::rng::Partition;
use crate::content::Character;
use crate::run::{diagnostics::{standard_supplied_run_with, supplied_wound_enemies}, CreationProfile, GameAction, RunRng};
use std::time::Instant;

#[test]
#[ignore = "measurement; run with --release --ignored --nocapture"]
fn m34s_knowledge_cost_on_supplied_routes() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        for ascension in [0u8, 20] {
            let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
            let mut run = standard_supplied_run_with(character, ascension, profile, 900, true, RunRng::new([0xB0 + ascension; 32], Partition::Test, 0)).unwrap();
            let (mut times, mut components, mut unrepresentable, mut steps, mut wounded_turn) = (vec![], vec![], 0usize, 0usize, None);
            while run.decision().is_ok_and(|d| d.terminal.is_none() && !d.candidates.is_empty()) && steps < 4_000 {
                let d = run.decision().unwrap();
                if let Some(combat) = run.state.combat.as_ref() {
                    let start = Instant::now();
                    let k = combat.draw_knowledge();
                    times.push(start.elapsed().as_nanos() as u64);
                    match k { Ok(k) => components.push(k.components().len()), Err(_) => unrepresentable += 1 }
                    let view = combat.public_full().unwrap();
                    let turn = view.turn;
                    if view.phase == Phase::Decision && wounded_turn != Some((run.state.combat_progress.entered, turn)) && view.enemies.iter().any(|e| e.hp > 1 && !e.half_dead) {
                        wounded_turn = Some((run.state.combat_progress.entered, turn));
                        let _ = supplied_wound_enemies(&mut run);
                        continue;
                    }
                }
                let play = d.candidates.iter().find(|c| matches!(c.action, GameAction::Combat(CombatAction::PlayCard { .. })));
                let action = play.unwrap_or(&d.candidates[0]).action.clone();
                if run.step(&action).is_err() { break }
                steps += 1;
            }
            times.sort_unstable(); components.sort_unstable();
            let pct = |v: &[u64], p: usize| v.get(v.len().saturating_sub(1) * p / 100).copied().unwrap_or(0);
            let max_components = components.last().copied().unwrap_or(0);
            let over_one = components.iter().filter(|c| **c > 1).count();
            println!("m34s_measure {character:?} A{ascension}: steps {steps}, terminal {:?}, combat decisions {}, draw_knowledge ns median {} p99 {} max {}, components max {max_components} (>1 at {over_one}), unrepresentable {unrepresentable}",
                run.decision().ok().and_then(|d| d.terminal), times.len(), pct(&times, 50), pct(&times, 99), times.last().copied().unwrap_or(0));
        }
    }
}

/// Review M7 (ignored; release): knowledge maintenance on the step path — every `knowledge::note` and pre-roll observation —
/// against the whole step, on Gym66-style Ironclad combats, the M10 Ironclad Run and the supplied Standard routes.
#[test]
#[ignore = "measurement; run with --release --ignored --nocapture"]
fn m34s_knowledge_maintenance_cost_per_step() {
    use crate::combat::knowledge::MAINTENANCE;
    use crate::run::{CombatTrainingEntry, RoomScenario, Run, RunConfig, TRAINING_CARDS, TRAINING_ENCOUNTERS_V2};
    fn drive(run: &mut Run, cap: usize, wound: bool, done: impl Fn(&Run) -> bool) -> (usize, u128, u64, u128) {
        MAINTENANCE.with(|m| m.set((0, 0)));
        let (mut steps, mut step_ns, mut wounded) = (0usize, 0u128, None);
        while steps < cap && !done(run) && run.decision().is_ok_and(|d| d.terminal.is_none() && !d.candidates.is_empty()) {
            let d = run.decision().unwrap();
            if wound { if let Some(combat) = run.state.combat.as_ref() {
                let view = combat.public_full().unwrap();
                let key = (run.state.combat_progress.entered, view.turn);
                if view.phase == Phase::Decision && wounded != Some(key) && view.enemies.iter().any(|e| e.hp > 1 && !e.half_dead) {
                    wounded = Some(key); let _ = supplied_wound_enemies(run); continue;
                }
            } }
            let play = d.candidates.iter().find(|c| matches!(c.action, GameAction::Combat(CombatAction::PlayCard { .. })));
            let action = play.unwrap_or(&d.candidates[0]).action.clone();
            let start = Instant::now();
            if run.step(&action).is_err() { break }
            step_ns += start.elapsed().as_nanos(); steps += 1;
        }
        let (calls, nanos) = MAINTENANCE.with(|m| m.get());
        (steps, step_ns, calls, nanos)
    }
    let report = |label: &str, (steps, step_ns, calls, nanos): (usize, u128, u64, u128)| println!(
        "m34s_maintenance {label}: steps {steps}, step mean {} ns, maintenance calls/step {:.2}, maintenance mean {} ns/step ({:.1}% of the step)",
        step_ns / steps.max(1) as u128, calls as f64 / steps.max(1) as f64, nanos / steps.max(1) as u128, 100.0 * nanos as f64 / step_ns.max(1) as f64);
    // Gym66-style Ironclad combats (ironclad-combat-v2 resources and entry).
    let mut total = (0, 0, 0, 0);
    for (index, (encounter, _, _)) in TRAINING_ENCOUNTERS_V2.iter().enumerate().take(24) {
        let deck = TRAINING_CARDS.iter().cycle().skip(index).take(14).map(|c| (*c, 0)).collect();
        let entry = CombatTrainingEntry { encounter: *encounter, hp: 80, max_hp: 80, deck, relics: vec![], potions: vec![None; 3], neows_lament_remaining: None };
        let mut run = Run::prepare_combat_training_v2_with_gold(entry, 0, RunRng::new([index as u8; 32], Partition::Test, 0)).unwrap();
        if run.enter_training_combat(*encounter).is_err() { continue }
        let r = drive(&mut run, 2_000, false, |run| run.combat_checkpoint().is_some());
        total = (total.0 + r.0, total.1 + r.1, total.2 + r.2, total.3 + r.3);
    }
    report("Gym66 Ironclad v2 (24 encounters)", total);
    for seed in [1u8, 2] {
        let mut run = Run::new_room_scenario(RunConfig { character: Character::Ironclad, ascension: 0 }, CreationProfile { neow_eligible: false, ..CreationProfile::experienced() },
            RunRng::new([seed; 32], Partition::Test, 0), RoomScenario::M10IroncladHeart).unwrap();
        report(&format!("M10 Ironclad Run seed {seed}"), drive(&mut run, 3_000, false, |_| false));
    }
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
        let mut run = standard_supplied_run_with(character, 20, profile, 900, true, RunRng::new([0xB8; 32], Partition::Test, 0)).unwrap();
        report(&format!("Standard {character:?} A20 supplied"), drive(&mut run, 4_000, true, |_| false));
    }
}
