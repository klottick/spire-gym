//! M37b differential checks (spire-4yo.2.1 and .2.2): the shared card-payload projection and `Run::step_offered`
//! against the pre-M37b paths, over random Standard runs of the four characters at A0 and A20.
//!
//! Each run is played twice in lockstep with the same seeded action choices: the new path (shared projection,
//! `step_offered`) and the reference path (the verbatim pre-M37b clone-per-payload projection, `step`). Every public
//! decision, transition and the final Run must be equal. At every combat decision of the new run, every (hand card,
//! target) payload from one shared projection is also compared with the reference payload and with the one-off
//! wrapper, and the projection must be restored to the combat state afterwards. Runs get 900 HP (so random play goes
//! past Act 1) and, in the relic variant (granted after Neow), those of Wrist Blade (Silent), Akabeko (Vigor), Pen Nib, Paper Phrog
//! (Ironclad) and Strike Dummy that the character can own, so the
//! Vigor, Wrist Blade and item damage-card projections are all exercised (asserted through coverage counters).
use super::*;
use crate::content::{Character, RelicId};
use crate::run::{CreationProfile, GameAction, RoomScenario, Run, RunConfig, RunRng};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
const RELICS: [RelicId; 5] = [RelicId::WristBlade, RelicId::Akabeko, RelicId::PenNib, RelicId::PaperPhrog, RelicId::StrikeDummy];

#[derive(Debug, Default)]
struct Coverage { decisions: u64, combat_decisions: u64, pairs: u64, ok_pairs: u64, vigor: u64, wrist: u64, max_hand: usize, max_act: u8 }

fn with_reference<R>(f: impl FnOnce() -> R) -> R {
    REFERENCE_PAYLOADS.with(|flag| flag.set(true));
    let result = f();
    REFERENCE_PAYLOADS.with(|flag| flag.set(false));
    result
}

/// Compare every (hand card, target) payload of `combat` from one shared projection with the reference and the
/// one-off wrapper, then check the projection is back to the combat state.
fn compare_payloads(combat: &Combat, coverage: &mut Coverage) {
    let state = &combat.state;
    let targets = std::iter::once(None).chain((1..=state.enemies.len() as u64).map(Some)).collect::<Vec<_>>();
    let mut projection = Projection::new(state);
    coverage.max_hand = coverage.max_hand.max(state.piles.hand.len());
    for owned in &state.piles.hand {
        let Some(card) = owned.full() else { continue };
        let attack = crate::content::Catalog::standard().card(card.id).is_some_and(|d| d.card_type == crate::content::CardType::Attack);
        for target in &targets {
            let reference = reference_payload_with_wrist_blade(state, card, *target);
            let shared = projected_payload(&mut projection, card, *target);
            let single = payload_with_wrist_blade(state, card, *target);
            assert_eq!(shared, reference, "shared projection payload differs for {:?} target {target:?}", card.id);
            assert_eq!(single, reference, "one-off projection payload differs for {:?} target {target:?}", card.id);
            coverage.pairs += 1;
            if let Ok((cost, _)) = &reference {
                coverage.ok_pairs += 1;
                if attack && watcher::cards_active(state) && state.player_full_powers.iter().any(|p| p.id.0 == "vigor" && p.state.amount > 0) { coverage.vigor += 1 }
                if attack && !card.local.x_cost && state.item_context.as_ref().is_some_and(|i| i.owns_relic(RelicId::WristBlade))
                    && super::super::card_state::effective_cost(card, *cost).ok() == Some(0) { coverage.wrist += 1 }
            }
        }
    }
    if let Some(scratch) = &projection.scratch { assert!(scratch == state, "projection not restored to the combat state") }
}

fn lockstep(character: Character, ascension: u8, seed: u64, relics: bool, steps: usize, coverage: &mut Coverage) {
    let mut root = [0u8; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    root[8] = ascension;
    let mut run = Run::new_room_scenario(RunConfig { character, ascension }, CreationProfile::experienced(),
        RunRng::new(root, crate::combat::rng::Partition::Test, 0), RoomScenario::Standard).unwrap();
    run.state_mut_for_test().max_hp = 900;
    run.state_mut_for_test().hp = 900;
    run.state_mut_for_test().keys = [true; 3];
    let mut new = run.clone();
    let mut reference = run;
    let mut choice = seed ^ 0x9E37_79B9_7F4A_7C15 ^ u64::from(ascension) << 32;
    let mut granted = !relics;
    for step in 0..steps {
        // The relic variant grants its relics (to both runs alike) at the first decision outside Neow and combat.
        if !granted && new.state().combat.is_none() && new.state().room != "Neow" {
            let union = crate::content::complete::relic_union(character);
            let missing = RELICS.into_iter().filter(|relic| union.contains(relic) && !new.state().relics.contains(relic)).collect::<Vec<_>>();
            for relic in missing {
                let source = || crate::items::lifecycle::RelicSource::Reward;
                let obtained = crate::items::lifecycle::obtain_relic(new.state_mut_for_test(), relic, source()).map(|_| ());
                assert_eq!(obtained, crate::items::lifecycle::obtain_relic(reference.state_mut_for_test(), relic, source()).map(|_| ()));
                obtained.unwrap_or_else(|error| panic!("granting {relic:?}: {error:?}"));
            }
            granted = true;
        }
        let decision = new.decision();
        assert_eq!(decision, with_reference(|| reference.decision()), "decision {step} differs");
        let Ok(decision) = decision else { break };
        coverage.decisions += 1;
        coverage.max_act = coverage.max_act.max(decision.act);
        if let Some(combat) = &new.state().combat {
            coverage.combat_decisions += 1;
            compare_payloads(combat, coverage);
            let legal = combat.legal_actions_full();
            assert_eq!(legal, with_reference(|| combat.legal_actions_full()), "legal actions differ at decision {step}");
            assert_eq!(legal.unwrap().into_iter().map(GameAction::Combat).collect::<Vec<_>>(),
                decision.candidates.iter().map(|c| c.action.clone()).collect::<Vec<_>>(), "published combat candidates");
        }
        if decision.terminal.is_some() || decision.candidates.is_empty() { break }
        choice ^= choice << 13; choice ^= choice >> 7; choice ^= choice << 17;
        let action = decision.candidates[(choice % decision.candidates.len() as u64) as usize].action.clone();
        // F2 alone: `step_offered` and `step` on the same Run (both with the shared projection).
        let mut checked = new.clone();
        let checked_transition = checked.step(&action);
        let transition = new.step_offered(&action);
        assert_eq!(transition, checked_transition, "step_offered transition differs at decision {step}");
        assert!(new == checked, "step_offered Run differs at decision {step}");
        // F1 and F2 against the reference path.
        assert_eq!(transition, with_reference(|| reference.step(&action)), "transition differs at decision {step}");
        if transition.is_err() { break }
    }
    assert!(new == reference, "final Run differs ({character:?} A{ascension} seed {seed} relics {relics})");
}

fn sweep(seeds: std::ops::Range<u64>, steps: usize) -> Coverage {
    let mut coverage = Coverage::default();
    for character in CHARACTERS {
        for ascension in [0, 20] {
            for seed in seeds.clone() {
                for relics in [false, true] { lockstep(character, ascension, seed, relics, steps, &mut coverage) }
            }
        }
    }
    coverage
}

#[test]
fn m37b_shared_projection_and_step_offered_match_the_reference_paths() {
    let coverage = sweep(0..2, 700);
    eprintln!("m37b projection coverage: {coverage:?}");
    assert!(coverage.vigor > 0 && coverage.wrist > 0 && coverage.ok_pairs > 1000 && coverage.max_act >= 2, "{coverage:?}");
}

/// The broad corpus for runner B: `cargo test --release -p spire-core m37b_broad -- --ignored --nocapture`.
#[test]
#[ignore = "broad M37b differential corpus; run explicitly"]
fn m37b_broad_shared_projection_and_step_offered_match_the_reference_paths() {
    let coverage = sweep(0..12, 3000);
    eprintln!("m37b broad projection coverage: {coverage:?}");
    assert!(coverage.vigor > 0 && coverage.wrist > 0 && coverage.max_act >= 3, "{coverage:?}");
}
