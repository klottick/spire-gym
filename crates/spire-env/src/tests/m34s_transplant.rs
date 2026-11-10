//! M34s-b b1: the combat-scope transplant (design §4.6.3). `Run::combat_scope` copies a positive list of Run-side combat
//! inputs plus the live combat (no canonicalisation); the transplant must step identically to the original combat.
use super::m34s_standard_combat_view::{current_node_kind, event_seeking};
use super::standard_traversal::{legal_indices, needs_supplied_wound, short_creation, simple_action, standard_session, CHARACTERS};
use serde_json::{json, Value};
use spire_core::{combat::rng::Partition, content::{Catalog, Character}};
use crate::*;
use spire_core::run::{diagnostics::supplied_wound_enemies, GameAction, Run};

/// Step `twin` with `action` and require the same public combat state and candidates as `original` afterwards.
fn assert_twins(original: &Run, twin: &Run, at: &str) {
    let (a, b) = (original.decision().unwrap(), twin.decision().unwrap());
    assert_eq!(a.combat, b.combat, "{at}: combat");
    assert_eq!(a.player, b.player, "{at}: player");
    assert_eq!((&a.objects, a.act, a.floor), (&b.objects, b.act, b.floor), "{at}: permanent deck, act, floor");
    assert_eq!(a.candidates.iter().map(|c| &c.action).collect::<Vec<_>>(), b.candidates.iter().map(|c| &c.action).collect::<Vec<_>>(), "{at}: candidates");
    assert_eq!(original.combat_identity(), twin.combat_identity(), "{at}: identity");
}

/// Runs `session` along its supplied route; at the first and the third decision of every combat a transplant starts and
/// follows the original's actions (and supplied hits) to the combat's end.
fn check_route(session: &mut RunSession, limit: usize, mut policy: impl FnMut(&RunObservation) -> usize) -> (usize, usize, usize) {
    let mut boundary = session.decision().unwrap();
    let mut twins: Vec<Run> = vec![];
    let (mut combat_step, mut transplants, mut ends, mut event_combats) = (0, 0, 0, 0);
    for _ in 0..limit {
        if boundary.terminal.is_some() { break }
        let run = session.run_for_test().unwrap().clone();
        let in_combat = run.decision().unwrap().combat.is_some();
        if !in_combat { ends += twins.len(); twins.clear(); combat_step = 0; } else {
            if combat_step == 0 && current_node_kind(&boundary.observation).as_deref() == Some("event") { event_combats += 1; }
            if combat_step == 0 || combat_step == 2 { twins.push(run.combat_scope().unwrap()); transplants += 1; }
            for twin in &twins { assert_twins(&run, twin, "boundary"); }
            combat_step += 1;
        }
        if needs_supplied_wound(&boundary.observation) {
            boundary = session.apply_supplied(supplied_wound_enemies).unwrap();
            for twin in &mut twins { supplied_wound_enemies(twin).unwrap(); }
        }
        let index = policy(&boundary.observation);
        assert!(legal_indices(&boundary.observation).contains(&index));
        let action = session.run_for_test().unwrap().decision().unwrap().candidates[index].action.clone();
        boundary = session.step(boundary.token, RunActionIndex(index as i64)).unwrap().boundary;
        let after = session.run_for_test().unwrap();
        for twin in &mut twins {
            let GameAction::Combat(_) = &action else { panic!("noncombat action inside a combat") };
            twin.step(&action).unwrap();
            if after.decision().unwrap().combat.is_some() { assert_twins(after, twin, "after step"); } else {
                // The original settled; the transplant stops at the combat's own terminal (no settlement).
                let t = twin.combat_scope_terminal().expect("transplant reached the combat terminal");
                let record = after.combat_checkpoint().expect("original settled");
                assert_eq!((t.outcome, t.hp_before_healing, t.hp_after_cleanup), (record.record.outcome, record.hp_before_healing, record.hp_after_cleanup));
            }
        }
    }
    (transplants, ends, event_combats)
}

#[test]
fn transplant_steps_identically_to_the_original_combat() {
    let mut summary = vec![];
    for (character_index, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0, 20] {
            let seed = 0xB0 + character_index as u8 + if ascension == 20 { 8 } else { 0 };
            let (mut session, _) = standard_session(character, ascension, seed, short_creation());
            let (transplants, ends, _) = check_route(&mut session, 12_000, simple_action);
            assert_eq!(session.decision().unwrap().terminal, Some(spire_core::run::RunTerminal::HeartVictory), "{character:?} A{ascension}");
            assert!(ends > 20, "{character:?} A{ascension}: {ends} transplanted combats ended");
            summary.push(format!("{character:?} A{ascension}: {transplants} transplants"));
        }
    }
    // Event combats (the event-seeking routes of the a3 sidecar test).
    let mut event_combats = 0;
    for (character_index, character) in CHARACTERS.into_iter().enumerate() {
        let (mut session, _) = standard_session(character, 0, 0xE0 + character_index as u8, short_creation());
        let (transplants, _, events) = check_route(&mut session, 12_000, event_seeking);
        summary.push(format!("{character:?} A0 event-seeking: {transplants} transplants, {events} event combats"));
        event_combats += events;
    }
    assert!(event_combats > 0, "an event combat was transplanted");
    // Standalone starts: 4 characters x {A0, A20} x an ordinary, a burning elite and a boss encounter.
    for character in CHARACTERS {
        for ascension in [0, 20] {
            for (encounter, burning) in [("jaw_worm", None), ("gremlin_nob", Some("strength")), ("hexaghost", None)] {
                let mut session = standard_combat_session(vec![standard_entry("e", character, ascension, encounter, burning)]);
                let mut boundary = session.reset(Some(7), None).unwrap();
                let mut twins = vec![session.run_for_test().unwrap().combat_scope().unwrap()];
                for step in 0.. {
                    let run = session.run_for_test().unwrap();
                    if step == 2 { twins.push(run.combat_scope().unwrap()); }
                    let legal = legal_indices(&boundary.observation);
                    let index = legal[step % legal.len()];
                    let action = run.decision().unwrap().candidates[index].action.clone();
                    let result = session.step(boundary.token, RunActionIndex(index as i64)).unwrap();
                    let after = session.run_for_test().unwrap();
                    for twin in &mut twins { twin.step(&action).unwrap(); assert_twins(after, twin, "standalone"); assert_eq!(after.combat_scope_terminal(), twin.combat_scope_terminal()); }
                    if result.terminated { break }
                    boundary = result.boundary;
                }
            }
        }
    }
    eprintln!("transplant_steps_identically_to_the_original_combat: {}", summary.join("; "));
}

/// One `standard-combat-v1` catalog entry with starter resources (the a4 test pattern).
pub(super) fn standard_entry(id: &str, character: Character, ascension: u8, encounter: &str, burning: Option<&str>) -> Value {
    let deck = Catalog::standard().starter_deck(character).unwrap().iter().map(|c| json!({"card": c.as_str(), "upgrades": 0})).collect::<Vec<_>>();
    let relic = match character { Character::Ironclad => "burning_blood", Character::Silent => "ring_of_the_snake", Character::Defect => "cracked_core", Character::Watcher => "pure_water" };
    let mut entry = json!({"id": id, "character": character.as_str(), "ascension": ascension, "encounter": encounter, "hp": 70, "max_hp": 80, "gold": 99,
        "deck": deck, "relics": [relic], "potions": vec![Value::Null; if ascension >= 11 { 2 } else { 3 }]});
    if let Some(buff) = burning { entry["burning"] = json!(buff); }
    entry
}
pub(super) fn standard_combat_session(entries: Vec<Value>) -> CombatTrainingSession {
    let bytes = serde_json::to_vec(&json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": entries})).unwrap();
    CombatTrainingSession::new_standard(CombatTrainingCatalog::standard_from_json(&bytes).unwrap(), Partition::Test).unwrap()
}

/// A combat-scope terminal never settles: no reward, no Run terminal, no settlement effects.
#[test]
fn combat_scope_terminal_does_not_settle() {
    let (mut session, _) = standard_session(spire_core::content::Character::Ironclad, 0, 0xB0, short_creation());
    let mut boundary = session.decision().unwrap();
    while session.run_for_test().unwrap().decision().unwrap().combat.is_none() {
        boundary = session.step(boundary.token, RunActionIndex(simple_action(&boundary.observation) as i64)).unwrap().boundary;
    }
    let mut twin = session.run_for_test().unwrap().combat_scope().unwrap();
    supplied_wound_enemies(&mut twin).unwrap();
    let deck = twin.decision().unwrap().objects;
    while twin.combat_scope_terminal().is_none() {
        let d = twin.decision().unwrap();
        let action = d.candidates.iter().find(|c| matches!(c.action, GameAction::Combat(spire_core::combat::CombatAction::PlayCard { .. }))).unwrap_or(&d.candidates[0]).action.clone();
        let t = twin.step(&action).unwrap();
        assert_eq!((t.reward, t.terminal, t.committed_events.len()), (0, None, 0));
    }
    let d = twin.decision().unwrap();
    assert!(d.candidates.is_empty() && d.terminal.is_none() && d.combat.is_some());
    assert_eq!((d.combat_progress.settled, d.objects), (0, deck));
    assert!(twin.combat_checkpoint().is_none());
}
