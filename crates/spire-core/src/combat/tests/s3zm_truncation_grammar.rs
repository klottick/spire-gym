//! spire-3zm: a rule-2d truncation stand-in (an enemy reaching 0 HP during its own move under Runic Dome) must keep
//! every search world a valid engine state. The stand-in is the first damage move the species' history grammar admits
//! after the public prior history (one owner, `enemies::history_admits`, shared with `validate_runtime`); a world the
//! grammar still rejects is refused at the root with its own class, never a committed fault at a later selection.
use super::super::belief::{Leak, Unsupported, LEAK};
use super::super::*;
use super::m40c_hidden_intents::{add_player_power, combat, combat_mut, dome, end_turn_action, play, root};
use crate::content::{CardId, Character, EncounterId, EnemyId};
use crate::run::{AuthoredRunStart, CardInstance, Run, RunRng, StandardCombatStart};

/// A20 Ironclad against Shield and Spear under Runic Dome, holding True Grit+ (an exhaust selection) every turn; the
/// player is Intangible (survives) and has Thorns, so an attacking enemy at low HP dies during its own move.
fn shield_spear(seed: u64) -> Run {
    let card = |id, instance, upgrades| CardInstance { id, instance, upgrades, misc: 0 };
    // Five True Grit+ copies: with at most four Burns shuffled in, every five-card hand holds one plus a candidate.
    let deck = (1..=5).map(|i| card(CardId::TrueGrit, i, 1)).collect();
    let start = StandardCombatStart { character: Character::Ironclad, ascension: 20, encounter: EncounterId::ShieldAndSpear, burning: None, bottled_cards: vec![], position: None,
        resources: AuthoredRunStart { hp: 70, max_hp: 80, gold: 50, deck, relics: dome(), potions: vec![None; 2], relic_counters: Default::default() } };
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&seed.to_le_bytes());
    let mut run = Run::prepare_standard_combat(start, RunRng::new(key, rng::Partition::Test, 0)).unwrap();
    run.enter_standard_combat().unwrap();
    add_player_power(&mut run, "intangible_player", 99);
    add_player_power(&mut run, "thorns", 5);
    run
}
fn enemy(run: &Run, id: EnemyId) -> usize { combat(run).state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(id)).unwrap() }
fn end(run: &mut Run) { let a = end_turn_action(run).expect("end turn"); run.step(&a).expect("end turn steps"); }
fn names(run: &Run, index: usize) -> Vec<&'static str> {
    combat(run).state.enemies[index].move_history.iter().map(|h| match h { ActorIntentId::Full(id) => id.as_str(), _ => "legacy" }).collect()
}
fn record(run: &Run, index: usize) -> hidden_moves::EnemyMoves {
    combat(run).state.knowledge.moves.enemies[&combat(run).state.enemies[index].instance_id].clone()
}
/// Kill `victim` during its move at history slot `slot` (0-based): end turns until that move is its pending one, then
/// drop it to 3 HP without Block, raise Thorns and end the turn. Returns the Run at the next player decision.
fn truncated_at(victim: EnemyId, slot: usize, seed: u64) -> Run {
    let mut run = shield_spear(seed);
    let i = enemy(&run, victim);
    while combat(&run).state.enemies[i].move_history.len() < slot + 1 { end(&mut run); }
    // Shield's Fortify (30 Block to every enemy) can land in the same enemy turn before the victim acts: Thorns 40
    // still gets through on the first hit.
    let e = &mut combat_mut(&mut run).state.enemies[i];
    (e.hp, e.block) = (3, 0);
    combat_mut(&mut run).state.player_full_powers.iter_mut().filter(|p| p.id.0 == "thorns").for_each(|p| p.state.amount = 40);
    end(&mut run);
    assert!(combat(&run).state.enemies[i].hp <= 0, "{victim:?} died during its move");
    run
}
/// Every search world at this root is a valid engine state, and True Grit+ reaches its exhaust selection in each.
fn worlds_are_valid_and_select(run: &Run, what: &str) {
    let r = root(run).unwrap_or_else(|u| panic!("{what}: root refused: {u:?}"));
    let mut worlds = vec![r.clone()];
    for k in 0..8u64 { worlds.push(r.belief_sample(&crate::run::research_sample_key(&crate::run::research_root(7), &[3; 32], k)).unwrap()); }
    for (k, w) in worlds.iter_mut().enumerate() {
        combat(w).validate_runtime_for_test().unwrap_or_else(|e| panic!("{what} world {k}: invalid: {e:?}"));
        let grit = play(w, CardId::TrueGrit, None).unwrap_or_else(|| panic!("{what} world {k}: True Grit+ not playable"));
        w.step(&grit).unwrap_or_else(|e| panic!("{what} world {k}: True Grit+ faulted: {e:?}"));
        assert_eq!(combat(w).state.phase, Phase::Selection, "{what} world {k}: the exhaust selection");
    }
}

/// The reported shape: the Spear dies during Skewer (slot 1, `1 % 3 == 1`); the stand-in is Skewer, the only move the
/// Spear grammar admits there, and every world is valid.
#[test]
fn spear_skewer_truncation_keeps_worlds_valid() {
    for seed in 0..3 {
        let run = truncated_at(EnemyId::SpireSpear, 1, seed);
        let i = enemy(&run, EnemyId::SpireSpear);
        assert_eq!(names(&run, i)[1], "spear_skewer", "precondition: the live move was Skewer");
        let r = record(&run, i);
        assert!(r.ambiguous.contains(&1), "rule 2d marked the truncated slot: {r:?}");
        assert_eq!(r.inert.get(&1).map(|s| s.as_str()), Some("spear_skewer"), "grammar-admitted stand-in");
        worlds_are_valid_and_select(&run, &format!("spear seed {seed}"));
    }
}

/// The Shield dies during Smash (slot 2): the stand-in is Smash, and the living Spear's worlds stay valid.
#[test]
fn shield_smash_truncation_keeps_worlds_valid() {
    for seed in 0..3 {
        let run = truncated_at(EnemyId::SpireShield, 2, seed);
        let i = enemy(&run, EnemyId::SpireShield);
        assert_eq!(names(&run, i)[2], "shield_smash", "precondition: the live move was Smash");
        let r = record(&run, i);
        assert!(r.ambiguous.contains(&2), "rule 2d marked the truncated slot: {r:?}");
        assert_eq!(r.inert.get(&2).map(|s| s.as_str()), Some("shield_smash"), "grammar-admitted stand-in");
        worlds_are_valid_and_select(&run, &format!("shield seed {seed}"));
    }
}

/// Species without a history grammar keep the first damage-base key (byte-identical records and digests).
#[test]
fn species_without_a_grammar_keep_the_first_key() {
    for id in [EnemyId::Darkling, EnemyId::AwakenedOne, EnemyId::Byrd, EnemyId::JawWorm, EnemyId::Maw] {
        let first = enemies::damage_bases_at_ascension(id, 20).unwrap().keys().next().copied();
        for prior in [vec![], first.into_iter().collect::<Vec<_>>()] {
            assert!(prior.iter().all(|_| enemies::history_admits(id, &prior)));
            assert_eq!(hidden_moves::stand_in(id, 20, &prior), first, "{id:?}");
        }
    }
}

/// Every grammar species: after every admitted prefix, the stand-in (when there is one) keeps the history admitted;
/// with no admitted damage move there is none (the record goes opaque).
#[test]
fn stand_ins_are_admitted_after_every_valid_prefix() {
    for id in [EnemyId::SpireShield, EnemyId::SpireSpear, EnemyId::CorruptHeart] {
        let moves = match id {
            EnemyId::SpireShield => vec!["shield_bash", "shield_fortify", "shield_smash"],
            EnemyId::SpireSpear => vec!["spear_burn_strike", "spear_skewer", "spear_strength"],
            _ => vec!["heart_debilitate", "heart_blood_shots", "heart_echo", "heart_buff"],
        }.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();
        let mut frontier = vec![vec![]];
        let (mut checked, mut some) = (0, 0);
        for _ in 0..9 {
            let mut next = vec![];
            for prefix in &frontier {
                if let Some(s) = hidden_moves::stand_in(id, 20, prefix) {
                    let mut h = prefix.clone(); h.push(s);
                    assert!(enemies::history_admits(id, &h), "{id:?} {prefix:?} + {s:?}");
                    some += 1;
                } else {
                    let bases = enemies::damage_bases_at_ascension(id, 20).unwrap();
                    assert!(bases.keys().all(|m| { let mut h = prefix.clone(); h.push(*m); !enemies::history_admits(id, &h) }), "{id:?} {prefix:?}: an admitted move was skipped");
                }
                checked += 1;
                for m in &moves { let mut h = prefix.clone(); h.push(*m); if enemies::history_admits(id, &h) { next.push(h) } }
            }
            frontier = next;
        }
        assert!(checked > 9 && some > 0, "{id:?}: {checked} prefixes, {some} stand-ins");
    }
}

/// One owner: a history the shared grammar rejects fails `validate_runtime`, its admitted twin passes.
#[test]
fn validator_uses_the_shared_grammar() {
    let mut run = shield_spear(0);
    end(&mut run);
    let i = enemy(&run, EnemyId::SpireSpear);
    assert!(combat(&run).validate_runtime_for_test().is_ok());
    let history = |run: &Run| combat(run).state.enemies[i].move_history.iter().map(|h| match h { ActorIntentId::Full(id) => *id, _ => unreachable!() }).collect::<Vec<_>>();
    assert!(enemies::history_admits(EnemyId::SpireSpear, &history(&run)));
    let mut bad = run.clone();
    let e = &mut combat_mut(&mut bad).state.enemies[i];
    let burn = ActorIntentId::Full(FullIntentId::from_name("spear_burn_strike"));
    e.move_history = vec![burn.clone(), burn.clone()]; e.intent_id = burn.clone(); e.selected_move = burn;
    assert!(!enemies::history_admits(EnemyId::SpireSpear, &history(&bad)));
    assert!(combat(&bad).validate_runtime_for_test().is_err());
}

/// Backstop: with the v1 grammar-blind stand-in (test-only leak) the root is refused with its own class, never handed
/// out as a world that faults at its next selection.
#[test]
fn grammar_blind_stand_in_is_refused_at_the_root() {
    LEAK.with(|v| v.set(Some(Leak::GrammarBlindStandIn)));
    let run = std::panic::catch_unwind(|| truncated_at(EnemyId::SpireSpear, 1, 0));
    let refused = run.as_ref().ok().map(|run| root(run).err());
    LEAK.with(|v| v.set(None));
    let run = run.unwrap();
    assert_eq!(record(&run, enemy(&run, EnemyId::SpireSpear)).inert.get(&1).map(|s| s.as_str()), Some("spear_burn_strike"), "the leak is the v1 stand-in");
    assert_eq!(refused, Some(Some(Unsupported::HiddenWorldInvalid)));
    assert_eq!(Unsupported::HiddenWorldInvalid.reason(), "a redrawn hidden-move world is not a valid combat state");
}
/// Negative control: the grammar-blind stand-in turns the Skewer test red.
#[test]
fn leak_grammar_blind_stand_in_fails_skewer() {
    LEAK.with(|v| v.set(Some(Leak::GrammarBlindStandIn)));
    let r = std::panic::catch_unwind(spear_skewer_truncation_keeps_worlds_valid);
    LEAK.with(|v| v.set(None));
    assert!(r.is_err(), "the Skewer test stayed green under GrammarBlindStandIn");
}

/// Every `RunError` shape maps into the published `CORE_FAULT_KINDS` vocabulary, one kind per variant (the engine-bound
/// variants share `bound`); a retained fault keeps its static checkpoint and nothing else (no message, handle or action).
#[test]
fn fault_class_maps_every_variant_into_the_published_vocabulary() {
    use crate::run::{CoreFaultClass, RunError, RunFault, RunFaultKind, CORE_FAULT_KINDS};
    let bound = EngineBound::new("test_counter", 1, 2);
    let combat = |kind: FaultKind| RunFaultKind::Combat(Box::new(CombatFault {
        record: FaultRecord { kind, message: "private message 34".into(), resolved_effects: 3 }, current: None, pending: Default::default(),
        attempted_action: Some(CombatAction::PlayCard { instance: 34, target: None }), selection: None, replacement_progress: None, spawn_progress: None }));
    let faulted = |kind: RunFaultKind| RunError::Faulted(RunFault { kind, checkpoint: "combat.runtime", committed_events: vec![] });
    let cases = [
        (RunError::InvalidAscension(25), "invalid_ascension", None), (RunError::InvalidAction, "invalid_action", None),
        (RunError::Unsupported("private"), "unsupported", None), (RunError::Registry("private"), "registry", None),
        (RunError::Terminal, "terminal", None), (RunError::Bound(bound), "bound", None),
        (faulted(RunFaultKind::UnsupportedBoundary("private")), "run_unsupported_boundary", Some("combat.runtime")),
        (faulted(RunFaultKind::Invariant("private")), "run_invariant", Some("combat.runtime")),
        (faulted(RunFaultKind::Bound(bound)), "bound", Some("combat.runtime")),
        (faulted(combat(FaultKind::Unsupported)), "combat_unsupported", Some("combat.runtime")),
        (faulted(combat(FaultKind::Invariant)), "combat_invariant", Some("combat.runtime")),
        (faulted(combat(FaultKind::EffectGuard)), "combat_effect_guard", Some("combat.runtime")),
        (faulted(combat(FaultKind::RngExhausted)), "combat_rng_exhausted", Some("combat.runtime")),
        (faulted(combat(FaultKind::Bound(bound))), "bound", Some("combat.runtime")),
    ];
    let mut kinds = std::collections::BTreeSet::new();
    for (error, kind, checkpoint) in cases {
        let class = error.fault_class();
        assert_eq!(class, CoreFaultClass { kind, checkpoint }, "{error:?}");
        assert!(CORE_FAULT_KINDS.contains(&class.kind), "{kind} is published");
        assert!(!format!("{class:?}").contains("private") && !format!("{class:?}").contains("34"), "no payload in {class:?}");
        kinds.insert(class.kind);
    }
    kinds.insert(CoreFaultClass::PANIC.kind);
    assert_eq!(kinds.into_iter().collect::<Vec<_>>(), { let mut all = CORE_FAULT_KINDS.to_vec(); all.sort(); all }, "every published kind is reachable");
}
