//! M27 one relic dispatcher (plan Task M2, design §6.1–§6.2, H0 seam map §5).
use super::*;
use crate::combat::tests::m15_silent_items::{enter, entry};
use crate::combat::FullIntentId;
use crate::content::{CardId, EnemyId, RelicId};

fn enemy_power_order(relics: Vec<RelicId>) -> Vec<&'static str> {
    let combat = enter(entry(&[CardId::StrikeGreen; 5], 1, 2, EnemyId::Cultist, FullIntentId::Incantation, &[], &[]), relics, vec![None; 3]);
    combat.public_full().unwrap().enemies[0].powers.iter().map(|power| power.id.0)
        .filter(|id| matches!(*id, "poison" | "vulnerable" | "weak")).collect()
}

/// G4: Twisted Funnel (Silent module) and Bag of Marbles (generic module) co-fire at battle
/// start; the order law makes acquisition order decide (before M27 Vulnerable always came first).
#[test]
fn m27_dispatch_cross_path_order_follows_acquisition() {
    assert_eq!(enemy_power_order(vec![RelicId::TwistedFunnel, RelicId::BagOfMarbles]), ["poison", "vulnerable"]);
    assert_eq!(enemy_power_order(vec![RelicId::BagOfMarbles, RelicId::TwistedFunnel]), ["vulnerable", "poison"]);
    assert_eq!(enemy_power_order(vec![RelicId::TwistedFunnel, RelicId::RedMask]), ["poison", "weak"]);
    assert_eq!(enemy_power_order(vec![RelicId::RedMask, RelicId::TwistedFunnel]), ["weak", "poison"]);
}

/// Guard (already true before M27): one loop visits Bag of Marbles and Red Mask in order.
#[test]
fn m27_dispatch_visit_order_is_acquisition_order() {
    assert_eq!(enemy_power_order(vec![RelicId::BagOfMarbles, RelicId::RedMask]), ["vulnerable", "weak"]);
    assert_eq!(enemy_power_order(vec![RelicId::RedMask, RelicId::BagOfMarbles]), ["weak", "vulnerable"]);
}

/// Design §6.1 rule 3 on the dispatcher's ordering function: groups of source-top relics are
/// reversed among themselves in place; other work keeps visit order.
#[test]
fn m27_dispatch_top_work_order() {
    // Visit order: A (top, 1 item), B (front, 1 item), C (top, 2 items).
    let mut top = vec!["a", "b", "c1", "c2"];
    dispatch::order_top_groups(&mut top, &[(0, 1, true), (1, 1, false), (2, 2, true)]);
    assert_eq!(top, ["c1", "c2", "b", "a"]);
    // One source-top group, or none, changes nothing.
    let mut top = vec!["a", "b", "c"];
    dispatch::order_top_groups(&mut top, &[(0, 1, true), (1, 1, false), (2, 1, false)]);
    assert_eq!(top, ["a", "b", "c"]);
    // Empty groups are ignored.
    let mut top = vec!["a", "c"];
    dispatch::order_top_groups(&mut top, &[(0, 1, true), (1, 0, true), (1, 1, true)]);
    assert_eq!(top, ["c", "a"]);
}

/// Non-test source files that may iterate owned relics, with their exact count of iteration
/// sites: the dispatcher, the laws, the Run laws and the owner-policy (O) rows of H0 —
/// validators, acquisition and replacement, and room offer producers.
const VISIT_ALLOW_LIST: &[(&str, usize)] = &[
    // The loops: combat phases (one site counted by two patterns), Run events and the
    // on-demand hook consumer query (`phase_consumed`); the laws.
    ("items/dispatch.rs", 4),
    ("items/laws.rs", 1),
    ("items/run_laws.rs", 5),
    // O: Run reward owner offers additional elite relic rows.
    ("run/rooms/rewards.rs", 4),
    // O: the inventory validators (`validate_combat_inventory`, `validate_complete_inventory`).
    ("items/mod.rs", 10),
    // O: the starter-slot replacement owners (`replace_cracked_core`, `replace_pure_water`).
    ("items/relics.rs", 3),
    // O: Chemical X fail-closed guards; they match by name so an unrepresented
    // `Unknown("chemical_x")` also fails closed (accepted M10 autoplay tests).
    ("combat/cards/ironclad.rs", 1),
    ("combat/cards/silent/x_repeat.rs", 1),
    // O: the public inventory copy and the M22 diagnostic checkpoint.
    ("combat/engine.rs", 1),
    ("combat/m22_diagnostics.rs", 1),
    // O: supplied-start validation, Tomb of Lord Red Mask, M10 training validation.
    ("run/combat.rs", 2),
    ("run/rooms/events/act3.rs", 1),
    ("run/training.rs", 1),
];

fn visit_sites(text: &str) -> usize {
    let mut count = 0;
    let mut in_test = false;
    for line in text.lines() {
        if line.trim_start().starts_with("#[cfg(test)]") { in_test = true; continue; }
        if in_test { in_test = line.trim_start().starts_with("#[") || line.trim().is_empty(); if line.contains("fn ") || line.contains("mod ") { in_test = false; continue; } }
        let code = line.split("//").next().unwrap();
        count += ["relics().iter()", "relics.iter()", "relics().to_vec()", "relics.clone() {", "in &context.relics", "in context.relics()"]
            .iter().map(|pattern| code.matches(pattern).count()).sum::<usize>();
    }
    count
}

#[test]
fn m27_single_relic_visit_loop() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let rel = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
            if path.is_dir() { if !rel.ends_with("tests") && rel != "run/diagnostics" && rel != "content" { stack.push(path); } continue; }
            if !rel.ends_with(".rs") || rel.ends_with("tests.rs") || rel.contains("_tests") || rel.starts_with("items/m1") { continue; }
            let sites = visit_sites(&std::fs::read_to_string(&path).unwrap());
            if sites > 0 { found.push((rel, sites)); }
        }
    }
    found.sort();
    let mut expected: Vec<_> = VISIT_ALLOW_LIST.iter().map(|(file, n)| (file.to_string(), *n)).collect();
    expected.sort();
    assert_eq!(found, expected, "relic iteration outside the dispatcher, the laws and the H0 O rows");
}

#[test]
fn m27_each_relic_has_one_handler() {
    use crate::items::relic_effects::{play, run, structural, turns, ACCEPTED_RELICS, MAIN_RELICS};
    let owners: [(&str, &[RelicId]); 6] = [("accepted", ACCEPTED_RELICS), ("main", MAIN_RELICS), ("R1", turns::RELICS),
        ("R2", play::RELICS), ("R3", run::RELICS), ("J", structural::RELICS)];
    assert_eq!(owners.map(|(_, list)| list.len()), [62, 5, 25, 23, 43, 21]);
    let all: std::collections::BTreeSet<_> = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher]
        .iter().flat_map(|c| crate::content::complete::relic_union(*c)).collect();
    for id in &all {
        let holders: Vec<_> = owners.iter().filter(|(_, list)| list.contains(id)).map(|(name, _)| *name).collect();
        assert_eq!(holders.len(), 1, "{id:?} owners {holders:?}");
    }
    assert_eq!(owners.iter().map(|(_, list)| list.len()).sum::<usize>(), all.len());
}

/// The dispatcher changes nothing when no relic is owned: a combat with an empty inventory
/// still reaches its first decision with no relic work.
#[test]
fn m27_seams_inert_without_items() {
    let with_none = enter(entry(&[CardId::StrikeGreen; 5], 1, 2, EnemyId::Cultist, FullIntentId::Incantation, &[], &[]), vec![], vec![None; 3]);
    let view = with_none.public_full().unwrap();
    assert!(view.enemies.iter().all(|enemy| enemy.powers.is_empty()));
    let mut context = CombatItemContext::new(vec![], vec![None; 3]);
    for hook in [Hook::CombatStarted, Hook::TurnStartedPreDraw, Hook::TurnStarted, Hook::CardAfterUse, Hook::PlayerEndTurn, Hook::CombatSettled] {
        let work = relic_phase_effects(&mut context, &dispatch::PhaseFacts::new(Some(hook), 80, 80, 0)).unwrap();
        assert!(work.is_empty(), "{hook:?}");
    }
    assert_eq!(context, CombatItemContext::new(vec![], vec![None; 3]));
}
