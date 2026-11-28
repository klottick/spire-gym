//! spire-3zm (env): a Runic Dome Shield and Spear root after the Spear died during its own Skewer. Every search surface
//! (`step`, `step_many`, `playout_many`) steps its sampled worlds through hand selections without a core fault, and a
//! redacted core fault carries only its closed class and static checkpoint, identical across sampled worlds.
use super::m34s_transplant::{standard_combat_session, standard_entry};
use crate::*;
use serde_json::json;
use spire_core::content::Character;
use spire_core::run::{diagnostics, CoreFaultClass, CORE_FAULT_KINDS};

const WIDE: SearchLimits = SearchLimits { max_live_branches: 64, max_branches: 1_000, max_steps_per_branch: 5_000, max_total_steps: 1_000_000 };

fn verb_at(o: &RunObservation, i: usize) -> Option<String> {
    let actions = o.group("actions").unwrap();
    let cols = *actions.shape.last().unwrap();
    usize::try_from(actions.values[i * cols]).ok().and_then(|v| standard_vocab_manifest()["families"]["verb"].as_array()?.get(v)?.as_str().map(str::to_owned))
}
fn first(o: &RunObservation, verb: &str) -> Option<i64> {
    o.action_mask.iter().enumerate().find(|(i, m)| **m == 1 && verb_at(o, *i).as_deref() == Some(verb)).map(|(i, _)| i as i64)
}
/// Encounter order: Spire Shield, Spire Spear.
const SPEAR: usize = 1;
/// A20 Ironclad, five True Grit+ and Runic Dome against Shield and Spear; Intangible and Thorns keep the player alive and
/// kill the Spear (3 HP, no Block, Thorns 40) on the first hit of its turn-2 Skewer. Returns the session at turn 3.
fn truncated_root(seed: u64) -> (CombatTrainingSession, CombatPolicyBoundary) {
    let mut e = standard_entry("s3zm", Character::Ironclad, 20, "shield_and_spear", None);
    e["deck"] = json!((0..5).map(|_| json!({"card": "true_grit", "upgrades": 1})).collect::<Vec<_>>());
    e["relics"].as_array_mut().unwrap().push(json!("runic_dome"));
    let mut s = standard_combat_session(vec![e]);
    let mut b = s.reset(Some(seed), None).unwrap();
    let run = s.run_mut_for_test().unwrap();
    diagnostics::supply_player_power(run, "intangible_player", 99);
    diagnostics::supply_player_power(run, "thorns", 5);
    b = s.step(b.token, RunActionIndex(first(&b.observation, "end_turn").unwrap())).unwrap().boundary;
    let run = s.run_mut_for_test().unwrap();
    diagnostics::supply_enemy_hp_and_block(run, SPEAR, 3, 0);
    let run = s.run_mut_for_test().unwrap();
    diagnostics::supply_player_power(run, "thorns", 40);
    let r = s.step(b.token, RunActionIndex(first(&b.observation, "end_turn").unwrap())).unwrap();
    assert!(!r.terminated);
    let spear_hp = s.run_for_test().unwrap().decision().unwrap().combat.unwrap().enemies[SPEAR].hp;
    assert!(spear_hp <= 0, "the Spear died during its Skewer ({spear_hp})");
    (s, r.boundary)
}

#[test]
fn truncated_spear_root_steps_selections_on_every_surface() {
    for seed in 0..2 {
        let (s, b) = truncated_root(seed);
        let grit = first(&b.observation, "play_card").expect("True Grit+ in hand");
        let root = s.public_root().expect("a supported root");
        // Scalar `step` and `step_many` with True Grit+ (it stops at its exhaust selection) in eight sampled worlds.
        let mut search = PublicSearch::new(root.clone(), SearchConfig { research_seed: 9, limits: WIDE }).unwrap();
        let ids = (0..8).map(|k| search.branch(Some(k)).unwrap()).collect::<Vec<_>>();
        for id in &ids[..4] {
            let d = search.decision(*id).unwrap();
            let r = search.step(*id, d.token, RunActionIndex(grit), BranchView::Full).unwrap_or_else(|e| panic!("seed {seed}: step faulted: {e:?}"));
            let o = r.observation.unwrap();
            assert!(first(&o, "combat_select_card").is_some(), "seed {seed}: the exhaust selection");
        }
        let reqs = ids[4..].iter().map(|id| BranchStepRequest { branch: *id, token: search.decision(*id).unwrap().token, index: RunActionIndex(grit) }).collect::<Vec<_>>();
        for (k, o) in search.step_many(&reqs, BranchView::Mask).unwrap().iter().enumerate() {
            assert!(matches!(o, BranchStepOutcome::Stepped(_)), "seed {seed} step_many {k}: {o:?}");
        }
        // `playout_many` (plays every card, so every True Grit+ selection) on fresh sampled worlds, the reported surface.
        let mut search = PublicSearch::new(root, SearchConfig { research_seed: 11, limits: WIDE }).unwrap();
        let ids = (0..8).map(|k| search.branch(Some(k)).unwrap()).collect::<Vec<_>>();
        let reqs = ids.iter().map(|id| BranchPlayoutRequest { branch: *id, token: search.decision(*id).unwrap().token }).collect::<Vec<_>>();
        let config = PlayoutConfig { rule: PlayoutRule::default(), max_steps: Some(60), playout_seed: 0 };
        for (k, o) in search.playout_many(&reqs, &config).unwrap().iter().enumerate() {
            assert!(matches!(o, BranchPlayoutOutcome::Finished(_)), "seed {seed} playout {k}: {o:?}");
        }
    }
}

/// A redacted core fault exposes only its closed class and static checkpoint: from the fixed vocabulary, identical in
/// every sampled world, never the core message or an attempted action, and the message stays `BRANCH_CORE_FAULT`.
#[test]
fn redacted_core_fault_class_is_closed_and_world_independent() {
    let (s, b) = truncated_root(0);
    let grit = first(&b.observation, "play_card").unwrap();
    let mut search = PublicSearch::new(s.public_root().unwrap(), SearchConfig { research_seed: 13, limits: WIDE }).unwrap();
    let ids = (0..6).map(|k| search.branch(Some(k)).unwrap()).collect::<Vec<_>>();
    let mut classes = vec![];
    for id in &ids[..3] {
        let d = search.decision(*id).unwrap();
        diagnostics::supplied_fault(search.branch_run_for_test(*id));
        let e = search.step(*id, d.token, RunActionIndex(grit), BranchView::Full).unwrap_err();
        classes.push(e.clone());
        assert_eq!(RunRecordError::from_error(&e).message, BRANCH_CORE_FAULT);
    }
    let reqs = ids[3..].iter().map(|id| { let t = search.decision(*id).unwrap().token; diagnostics::supplied_fault(search.branch_run_for_test(*id)); BranchStepRequest { branch: *id, token: t, index: RunActionIndex(grit) } }).collect::<Vec<_>>();
    for o in search.step_many(&reqs, BranchView::Mask).unwrap() {
        let BranchStepOutcome::Fault { error, stage, committed } = o else { panic!("expected a fault: {o:?}") };
        assert_eq!((stage, committed), ("step", true));
        classes.push(error);
    }
    let want = RunSessionError::SearchCoreFault(CoreFaultClass { kind: "run_invariant", checkpoint: Some("m34s.supplied-fault") });
    for c in &classes {
        assert_eq!(c, &want, "identical across sampled worlds");
        let RunSessionError::SearchCoreFault(class) = c else { unreachable!() };
        assert!(CORE_FAULT_KINDS.contains(&class.kind));
        assert!(!format!("{c:?}").contains("supplied diagnostic fault") && !format!("{c:?}").contains("instance"), "no payload text");
    }
}

/// The class vocabulary is closed, unique and value-free; every `RunError` shape maps into it.
#[test]
fn core_fault_kinds_are_a_closed_value_free_vocabulary() {
    use spire_core::run::RunError;
    let set = CORE_FAULT_KINDS.iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(set.len(), CORE_FAULT_KINDS.len());
    assert!(CORE_FAULT_KINDS.iter().all(|k| k.chars().all(|c| c.is_ascii_lowercase() || c == '_')));
    for e in [RunError::InvalidAscension(25), RunError::InvalidAction, RunError::Unsupported("x"), RunError::Registry("x"), RunError::Terminal] {
        let class = e.fault_class();
        assert!(CORE_FAULT_KINDS.contains(&class.kind) && class.checkpoint.is_none(), "{e:?}");
    }
    assert!(CORE_FAULT_KINDS.contains(&CoreFaultClass::PANIC.kind));
}
