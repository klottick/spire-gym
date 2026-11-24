//! M34s-b final review I1: P1/P3 families with generated identical copies. A world and its label-swapped twin (two
//! combat-local copies of one card class exchange places, so the hidden instance order differs while every public fact
//! is equal) must publish identical Standard views, equal public roots and digests, and identical branches for the same
//! research seed, at a normal decision and at the order-hiding grid itself. Each family first proves the true worlds
//! differ (the canonical worlds differ, and the same live grid pick leads to different next draws).
use super::m34s_transplant::{standard_combat_session, standard_entry};
use crate::*;
use spire_core::{combat::{CardClass, CardDestination, CombatAction, Combat, EffectKind}, content::{CardId, Character}, run::{diagnostics, GameAction, Run}};

fn caps() -> RunCapacities { RunCapacities::standard_default() }
fn view(run: &Run) -> RunObservation { crate::combat_training::standard_combat_publication(run, &caps()).unwrap().0 }
fn combat(run: &mut Run) -> &mut Combat { diagnostics::supplied_combat(run).unwrap() }
fn start(character: Character, encounter: &str, seed: u64) -> Run {
    let mut s = standard_combat_session(vec![standard_entry("e", character, 0, encounter, None)]);
    s.reset(Some(seed), None).unwrap();
    s.run_for_test().unwrap().clone()
}
fn action_index(run: &Run, want: impl Fn(&GameAction) -> bool) -> Option<usize> { run.decision().unwrap().candidates.iter().position(|c| want(&c.action)) }
fn step(run: &mut Run, index: usize) { let a = run.decision().unwrap().candidates[index].action.clone(); run.step(&a).unwrap(); }
fn play(run: &mut Run, instance: u64) {
    let i = action_index(run, |a| matches!(a, GameAction::Combat(CombatAction::PlayCard { instance: x, .. }) if *x == instance)).expect("grid card playable");
    step(run, i);
}
/// The live pick of the first grid candidate of card `id` (grid order).
fn pick_first(run: &mut Run, id: CardId) {
    let d = run.decision().unwrap();
    let chosen = d.combat.as_ref().unwrap().selection.as_ref().unwrap().candidates.iter().find(|c| c.id == id).unwrap().instance.0;
    let i = action_index(run, |a| matches!(a, GameAction::Combat(CombatAction::SelectCard { instance }) if *instance == chosen)).unwrap();
    step(run, i);
}

struct Rng(u64);
impl Rng { fn next(&mut self) -> u64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; self.0 } }
/// A research-seeded random legal policy over `max` steps; a refused hidden-order pick tries the next legal index.
/// Returns the trace (mask, reward, terminated, truncated) and the number of refusals.
fn rollout(search: &mut PublicSearch, b: BranchId, seed: u64, max: usize) -> (Vec<(Vec<i8>, i32, bool, bool)>, usize) {
    let mut rng = Rng(seed | 1);
    let d = search.decision(b).unwrap();
    let (mut mask, mut token, mut out, mut refused) = (d.observation.action_mask.clone(), d.token, vec![], 0);
    for _ in 0..max {
        let legal = mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect::<Vec<_>>();
        let first = (rng.next() % legal.len() as u64) as usize;
        let mut taken = None;
        for k in 0..legal.len() {
            match search.step(b, token, RunActionIndex(legal[(first + k) % legal.len()] as i64), BranchView::Full) {
                Ok(r) => { taken = Some(r); break }
                Err(RunSessionError::UnsupportedBoundary(HIDDEN_ORDER_PICK)) => refused += 1,
                Err(e) => panic!("branch step: {e:?}"),
            }
        }
        let r = taken.expect("some legal action is supported");
        out.push((r.action_mask.clone(), r.reward, r.terminated, r.truncated));
        if r.terminated || r.truncated { break }
        (mask, token) = (r.action_mask, r.token);
    }
    (out, refused)
}

/// The P1/P3 checks for a world and its twin at their current decision; returns the refusals seen.
fn same_public_outputs(label: &str, a: &Run, b: &Run) -> usize {
    assert_ne!(a, b, "{label}: the true worlds differ");
    assert_eq!(view(a), view(b), "{label}: identical Standard views");
    let (ra, rb) = (PublicRoot::of(a, &caps()).unwrap(), PublicRoot::of(b, &caps()).unwrap());
    assert_eq!(ra.digest(), rb.digest(), "{label}: equal root digests");
    assert_eq!(ra, rb, "{label}: equal public roots");
    let (mut sa, mut sb) = (PublicSearch::new(ra, SearchConfig { research_seed: 11, limits: WIDE }).unwrap(), PublicSearch::new(rb, SearchConfig { research_seed: 11, limits: WIDE }).unwrap());
    let mut refusals = 0;
    for k in 0..6 {
        let (ba, bb) = (sa.branch(Some(k)).unwrap(), sb.branch(Some(k)).unwrap());
        assert_eq!(sa.decision(ba).unwrap().observation, sb.decision(bb).unwrap().observation, "{label}: sample {k} starts equal");
        let (ta, na) = rollout(&mut sa, ba, 100 + k, 60);
        let (tb, nb) = rollout(&mut sb, bb, 100 + k, 60);
        assert_eq!((ta, na), (tb, nb), "{label}: sample {k}: identical branches");
        refusals += na;
    }
    refusals
}
const WIDE: SearchLimits = SearchLimits { max_live_branches: 64, max_branches: 1_000, max_steps_per_branch: 5_000, max_total_steps: 1_000_000 };

/// Builds a family's world (supplied copies and the grid card in hand) and returns (world, the grid card, the copy pair
/// to swap, the card id the live pick takes).
type Family = fn(u64) -> (Run, u64, (u64, u64), CardId);
/// Defect Seek: a Wound topdecked (published) and a Wound shuffled in (hidden), swapped.
fn seek_known_and_hidden(seed: u64) -> (Run, u64, (u64, u64), CardId) {
    let mut run = start(Character::Defect, "jaw_worm", seed);
    let c = combat(&mut run);
    let hidden = c.diagnostic_create_card(CardId::Wound, 0, CardDestination::ShuffledDraw).unwrap();
    let known = c.diagnostic_create_card(CardId::Wound, 0, CardDestination::Draw).unwrap();
    let seek = c.diagnostic_create_card(CardId::Seek, 0, CardDestination::Hand).unwrap();
    (run, seek, (known, hidden), CardId::Wound)
}
/// Watcher Secret Technique: an Insight topdecked and one shuffled in (Insight is a skill), swapped.
fn secret_technique_insights(seed: u64) -> (Run, u64, (u64, u64), CardId) {
    let mut run = start(Character::Watcher, "jaw_worm", seed);
    let c = combat(&mut run);
    let hidden = c.diagnostic_create_card(CardId::Insight, 0, CardDestination::ShuffledDraw).unwrap();
    let known = c.diagnostic_create_card(CardId::Insight, 0, CardDestination::Draw).unwrap();
    let secret = c.diagnostic_create_card(CardId::SecretTechnique, 0, CardDestination::Hand).unwrap();
    (run, secret, (known, hidden), CardId::Insight)
}
/// Cross-zone: a Wound in hand and a Wound topdecked (Seek grid), swapped across hand and draw.
fn hand_and_draw_wounds(seed: u64) -> (Run, u64, (u64, u64), CardId) {
    let mut run = start(Character::Defect, "cultist", seed);
    let c = combat(&mut run);
    let in_hand = c.diagnostic_create_card(CardId::Wound, 0, CardDestination::Hand).unwrap();
    let known = c.diagnostic_create_card(CardId::Wound, 0, CardDestination::Draw).unwrap();
    c.diagnostic_create_card(CardId::Wound, 0, CardDestination::ShuffledDraw).unwrap();
    let seek = c.diagnostic_create_card(CardId::Seek, 0, CardDestination::Hand).unwrap();
    (run, seek, (in_hand, known), CardId::Wound)
}
/// Ironclad Secret Weapon: a Shiv topdecked and one shuffled in (Shiv is an attack), swapped.
fn secret_weapon_shivs(seed: u64) -> (Run, u64, (u64, u64), CardId) {
    let mut run = start(Character::Ironclad, "jaw_worm", seed);
    let c = combat(&mut run);
    let hidden = c.diagnostic_create_card(CardId::Shiv, 0, CardDestination::ShuffledDraw).unwrap();
    let known = c.diagnostic_create_card(CardId::Shiv, 0, CardDestination::Draw).unwrap();
    let secret = c.diagnostic_create_card(CardId::SecretWeapon, 0, CardDestination::Hand).unwrap();
    (run, secret, (known, hidden), CardId::Shiv)
}
const FAMILIES: [(&str, Family); 4] = [("seek_known_and_hidden", seek_known_and_hidden), ("secret_technique_insights", secret_technique_insights),
    ("hand_and_draw_wounds", hand_and_draw_wounds), ("secret_weapon_shivs", secret_weapon_shivs)];

#[test]
fn duplicate_copy_families_publish_no_instance_order() {
    let (mut refusals, mut diverged) = (0, 0);
    for (name, family) in FAMILIES {
        for seed in 1..=4u64 {
            let (a, grid_card, (x, y), id) = family(seed);
            let mut b = a.clone();
            assert!(combat(&mut b).diagnostic_swap_identical_copies(x, y), "{name}: identical copies");
            let label = format!("{name} seed {seed}");
            refusals += same_public_outputs(&format!("{label} at the decision"), &a, &b);
            let (mut ga, mut gb) = (a.clone(), b.clone());
            play(&mut ga, grid_card);
            play(&mut gb, grid_card);
            refusals += same_public_outputs(&format!("{label} at the grid"), &ga, &gb);
            // The live pick of the first candidate: the true worlds now differ in what lies on the draw pile.
            pick_first(&mut ga, id);
            pick_first(&mut gb, id);
            assert_eq!(view(&ga), view(&gb), "{label}: identical views after the pick");
            let draws = |run: &mut Run| combat(run).diagnostic_draw_classes();
            diverged += usize::from(draws(&mut ga) != draws(&mut gb));
        }
    }
    assert!(refusals > 0, "branches met and refused hidden-order picks");
    assert!(diverged > 0, "the hidden order matters: the same live pick leaves different draw piles");
}

/// Ruling (c): two Burn copies, one upgraded at a public discard position after their labels were swapped, are
/// shuffled together; the Seek grid shows the same order in both worlds.
#[test]
fn different_class_copies_show_one_grid_order() {
    for seed in 1..=4u64 {
        let mut a = start(Character::Defect, "jaw_worm", seed);
        let c = combat(&mut a);
        let first = c.diagnostic_create_card(CardId::Burn, 0, CardDestination::Discard).unwrap();
        let second = c.diagnostic_create_card(CardId::Burn, 0, CardDestination::Discard).unwrap();
        let seek = c.diagnostic_create_card(CardId::Seek, 0, CardDestination::Hand).unwrap();
        let mut b = a.clone();
        assert!(combat(&mut b).diagnostic_swap_identical_copies(first, second));
        for run in [&mut a, &mut b] {
            let c = combat(run);
            c.diagnostic_upgrade_discard(c.diagnostic_discard_len() - 1).unwrap();
            c.diagnostic_apply(EffectKind::Recycle).unwrap();
        }
        let label = format!("upgraded Burn copy seed {seed}");
        same_public_outputs(&format!("{label} at the decision"), &a, &b);
        play(&mut a, seek);
        play(&mut b, seek);
        same_public_outputs(&format!("{label} at the grid"), &a, &b);
    }
}

// ---------- M34s-b recheck R1: a hidden uniform pick (Stasis, Violence) of one of several identical copies ----------
type Custody = [Vec<(u64, Option<CardClass>)>; 2];
fn classes(c: &Custody) -> [Vec<Option<CardClass>>; 2] { c.clone().map(|zone| zone.into_iter().map(|(_, class)| class).collect()) }
/// A P1 pair for a hidden pick: copies of `world` that differ only in their hidden card-random state, each run through
/// `act` (the pick and the play up to the next decision). Returns the first two outcomes that show the same classes in
/// hand and in Stasis custody, in order (an equal public history) but hold different copies (the hidden pick differs).
fn hidden_pick_pair(world: &Run, act: impl Fn(&mut Run)) -> Option<(Run, Run)> {
    let mut seen: Vec<(Run, Custody)> = vec![];
    for draws in 0..400 {
        let mut w = world.clone();
        combat(&mut w).diagnostic_advance_card_rng(draws).unwrap();
        act(&mut w);
        let custody = combat(&mut w).diagnostic_hand_and_stasis();
        if let Some((other, _)) = seen.iter().find(|(_, c)| classes(c) == classes(&custody) && *c != custody) { return Some((other.clone(), w)) }
        seen.push((w, custody));
    }
    None
}
fn end_turn(run: &mut Run) { step(run, action_index(run, |a| matches!(a, GameAction::Combat(CombatAction::EndTurn))).expect("end turn")); }
/// Ironclad Violence: a Shiv topdecked (published) and a Shiv shuffled in (hidden); Violence takes 3 random Attacks.
fn violence_shivs(seed: u64) -> (Run, Box<dyn Fn(&mut Run)>) {
    let mut run = start(Character::Ironclad, "jaw_worm", seed);
    let c = combat(&mut run);
    c.diagnostic_create_card(CardId::Shiv, 0, CardDestination::ShuffledDraw).unwrap();
    c.diagnostic_create_card(CardId::Shiv, 0, CardDestination::Draw).unwrap();
    let violence = c.diagnostic_create_card(CardId::Violence, 0, CardDestination::Hand).unwrap();
    (run, Box::new(move |r: &mut Run| play(r, violence)))
}
/// Ironclad against Bronze Automaton (a doubled, all-Basic starter deck, so every card shares Stasis's rarity tier; no
/// random-target effect draws the card stream, unlike Defect's Lightning): after the orbs spawn, a Wound bottom-decked (published) and a Wound shuffled in (hidden); ending the turn lets the orbs cast
/// Stasis before the next draw.
fn stasis_wounds(seed: u64) -> (Run, Box<dyn Fn(&mut Run)>) {
    let mut entry = super::m34s_transplant::standard_entry("e", Character::Ironclad, 0, "bronze_automaton", None);
    let deck = entry["deck"].as_array().unwrap().clone();
    entry["deck"].as_array_mut().unwrap().extend(deck);
    let mut s = standard_combat_session(vec![entry]);
    s.reset(Some(seed), None).unwrap();
    let mut run = s.run_for_test().unwrap().clone();
    end_turn(&mut run);
    let c = combat(&mut run);
    c.diagnostic_create_card(CardId::Wound, 0, CardDestination::ShuffledDraw).unwrap();
    c.diagnostic_create_card(CardId::Wound, 0, CardDestination::BottomDraw).unwrap();
    (run, Box::new(end_turn))
}
type HiddenPickFamily = fn(u64) -> (Run, Box<dyn Fn(&mut Run)>);

/// P1 pairs (recheck R1): two worlds with an equal public history whose hidden RNG took different identical copies (one
/// of them at a published draw position) have the same root support, identical Standard views, equal roots and digests,
/// and identical branches. Without the forgetting, the view shows the published copy in one world only and that world's
/// root is unsupported (red: the R1 mutants and `logs/r1-env-red-*.log`).
fn hidden_pick_family(name: &str, family: HiddenPickFamily) {
    let mut pairs = 0;
    for seed in 1..=4u64 {
        let (world, act) = family(seed);
        let Some((a, b)) = hidden_pick_pair(&world, act) else { continue };
        let label = format!("{name} seed {seed}");
        let support = |run: &Run| PublicRoot::of(run, &caps()).map(|_| ()).map_err(|e| format!("{e:?}"));
        assert_eq!((view(&a) == view(&b), support(&a) == support(&b)), (true, true),
            "{label}: (equal views, equal root support) whichever copy the hidden pick took");
        assert_eq!(support(&a), Ok(()), "{label}: supported root");
        same_public_outputs(&format!("{label} after the hidden pick"), &a, &b);
        pairs += 1;
    }
    assert!(pairs >= 2, "{name}: at least two seeds give a hidden-pick pair ({pairs})");
}
#[test]
fn violence_pick_publishes_no_copy_choice() { hidden_pick_family("violence_shivs", violence_shivs) }
#[test]
fn stasis_pick_publishes_no_copy_choice() { hidden_pick_family("stasis_wounds", stasis_wounds) }

// ---------- M40a: the branch rule for a pick among identical, exchangeable draw-pile copies ----------
/// Defect Seek with two Wounds that were both shuffled in (one bag: the copies are exchangeable under public history).
fn seek_two_hidden_wounds(seed: u64, upgrades: u32) -> (Run, u64, (u64, u64)) {
    let mut run = start(Character::Defect, "jaw_worm", seed);
    let c = combat(&mut run);
    let x = c.diagnostic_create_card(CardId::Wound, 0, CardDestination::ShuffledDraw).unwrap();
    let y = c.diagnostic_create_card(CardId::Wound, 0, CardDestination::ShuffledDraw).unwrap();
    let seek = c.diagnostic_create_card(CardId::Seek, upgrades, CardDestination::Hand).unwrap();
    (run, seek, (x, y))
}
/// The indices of the branch's legal `SelectCard` actions that name a Wound.
fn wound_picks(search: &mut PublicSearch, b: BranchId) -> Vec<usize> {
    let run = search.branch_run_for_test(b);
    let d = run.decision().unwrap();
    let wounds = d.combat.as_ref().unwrap().selection.as_ref().unwrap().candidates.iter().filter(|c| c.id == CardId::Wound).map(|c| c.instance.0).collect::<Vec<_>>();
    d.candidates.iter().enumerate().filter(|(_, c)| matches!(&c.action, GameAction::Combat(CombatAction::SelectCard { instance }) if wounds.contains(instance))).map(|(i, _)| i).collect()
}
#[test]
fn single_pick_among_exchangeable_identical_copies_is_applied_and_label_free() {
    for seed in 1..=4u64 {
        let (a, seek, (x, y)) = seek_two_hidden_wounds(seed, 0);
        let mut b = a.clone();
        assert!(combat(&mut b).diagnostic_swap_identical_copies(x, y) || combat(&mut b).diagnostic_swap_identical_copies(y, x), "identical copies");
        let (mut ga, mut gb) = (a.clone(), b.clone());
        play(&mut ga, seek);
        play(&mut gb, seek);
        let label = format!("two hidden Wounds seed {seed}");
        assert_eq!(same_public_outputs(&format!("{label} at the grid"), &ga, &gb), 0, "{label}: no pick is refused");
        // Every identical candidate is the same action: the next public view and the whole branch agree, in both twins.
        let mut outcomes = vec![];
        for world in [&ga, &gb] {
            let mut search = PublicSearch::new(PublicRoot::of(world, &caps()).unwrap(), SearchConfig { research_seed: 5, limits: WIDE }).unwrap();
            for k in 0..4u64 {
                let probe = search.branch(Some(k)).unwrap();
                let picks = wound_picks(&mut search, probe);
                assert_eq!(picks.len(), 2, "{label}: two identical Wound candidates");
                let views = picks.iter().map(|i| {
                    let b = search.branch(Some(k)).unwrap();
                    let d = search.decision(b).unwrap();
                    let step = search.step(b, d.token, RunActionIndex(*i as i64), BranchView::Full).expect("the pick is applied");
                    (step.observation, step.action_mask, step.info)
                }).collect::<Vec<_>>();
                assert_eq!(views[0], views[1], "{label} sample {k}: either identical candidate gives one outcome");
                outcomes.push(views[0].clone());
            }
        }
        assert_eq!(outcomes[..4], outcomes[4..], "{label}: label-swapped twins give the same branch outcomes");
    }
}
#[test]
fn replay_and_fork_reproduce_the_applied_pick() {
    let (a, seek, _) = seek_two_hidden_wounds(2, 0);
    let mut g = a.clone();
    play(&mut g, seek);
    let mut search = PublicSearch::new(PublicRoot::of(&g, &caps()).unwrap(), SearchConfig { research_seed: 9, limits: WIDE }).unwrap();
    let b = search.branch(Some(3)).unwrap();
    let pick = wound_picks(&mut search, b)[0];
    let d = search.decision(b).unwrap();
    let before_fork = search.fork(b).unwrap();
    let stepped = search.step(b, d.token, RunActionIndex(pick as i64), BranchView::Full).unwrap();
    let forked = {
        let d = search.decision(before_fork).unwrap();
        search.step(before_fork, d.token, RunActionIndex(pick as i64), BranchView::Full).unwrap()
    };
    assert_eq!((&stepped.observation, &stepped.action_mask), (&forked.observation, &forked.action_mask), "a fork applies the same rule");
    let live = search.branch_run_for_test(b).clone();
    assert_eq!(search.replay_for_test(b).unwrap(), live, "replay reproduces the applied pick");
}
#[test]
fn a_two_pick_grid_still_refuses_with_no_effect() {
    let (a, seek, _) = seek_two_hidden_wounds(1, 1);
    let mut g = a.clone();
    play(&mut g, seek);
    let mut search = PublicSearch::new(PublicRoot::of(&g, &caps()).unwrap(), SearchConfig { research_seed: 5, limits: WIDE }).unwrap();
    let b = search.branch(Some(0)).unwrap();
    let pick = wound_picks(&mut search, b)[0];
    let d = search.decision(b).unwrap();
    assert_eq!(search.step(b, d.token, RunActionIndex(pick as i64), BranchView::Full).unwrap_err(), RunSessionError::UnsupportedBoundary(HIDDEN_ORDER_PICK));
    assert_eq!(search.decision(b).unwrap().token, d.token, "no effect: the branch is live at the same boundary");
}
/// The applied pick removes a uniformly random identical copy (by pile position), not the lowest label: across samples the
/// removed Wound is the upper of the two about half of the time. (Red: a label-indexed pick always removes the same one.)
#[test]
fn applied_pick_removes_a_uniformly_random_identical_copy() {
    let (a, seek, _) = seek_two_hidden_wounds(3, 0);
    let mut g = a.clone();
    play(&mut g, seek);
    let mut search = PublicSearch::new(PublicRoot::of(&g, &caps()).unwrap(), SearchConfig { research_seed: 21, limits: WIDE }).unwrap();
    let (mut upper, mut lower) = (0, 0);
    for k in 0..96u64 {
        let b = search.branch(Some(k)).unwrap();
        let before = combat(search.branch_run_for_test(b)).diagnostic_draw_classes();
        let pick = wound_picks(&mut search, b)[0];
        let d = search.decision(b).unwrap();
        search.step(b, d.token, RunActionIndex(pick as i64), BranchView::Mask).unwrap();
        let after = combat(search.branch_run_for_test(b)).diagnostic_draw_classes();
        let removed = (0..before.len()).filter(|i| { let mut t = before.clone(); t.remove(*i); t == after }).collect::<Vec<_>>();
        let wound_class = before.iter().flatten().find(|c| before.iter().filter(|x| *x == &Some(**c)).count() == 2).copied().expect("two identical Wounds");
        let wound_depths = (0..before.len()).filter(|i| before[*i] == Some(wound_class)).collect::<Vec<_>>();
        let hits = removed.iter().filter(|i| wound_depths.contains(i)).collect::<Vec<_>>();
        search.close(b).unwrap();
        if hits.len() != 1 { continue } // identical neighbours: the removal is not distinguishable
        if *hits[0] == wound_depths[0] { upper += 1 } else { lower += 1 }
    }
    assert!(upper >= 20 && lower >= 20, "removal by rank is uniform: upper {upper}, lower {lower}");
}

/// M39b compact branch views: the applied pick is independent of the published layout. Either identical candidate gives one
/// compact outcome through `step` and through `step_many`, and the compact outcome widens to the dense one.
#[test]
fn applied_pick_is_layout_independent_in_compact_branches() {
    let (a, seek, _) = seek_two_hidden_wounds(2, 0);
    let mut g = a.clone();
    play(&mut g, seek);
    let root = PublicRoot::of(&g, &caps()).unwrap();
    let mut config = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
    config.capacities.combat_cards = 1024;
    config.capacities.selection = 1024;
    config.capacities.actions = 256;
    let mut compact = PublicSearch::new_with_combat_layout(root.clone(), SearchConfig { research_seed: 5, limits: WIDE }, config).unwrap();
    let mut dense = PublicSearch::new(root, SearchConfig { research_seed: 5, limits: WIDE }).unwrap();
    for k in 0..4u64 {
        let probe = compact.branch(Some(k)).unwrap();
        let picks = wound_picks(&mut compact, probe);
        assert_eq!(picks.len(), 2, "two identical Wound candidates");
        let mut outcomes = vec![];
        for &i in &picks {
            let b = compact.branch(Some(k)).unwrap();
            let d = compact.decision(b).unwrap();
            let stepped = compact.step(b, d.token, RunActionIndex(i as i64), BranchView::Full).expect("the pick is applied in a compact branch");
            let c = compact.branch(Some(k)).unwrap();
            let d = compact.decision(c).unwrap();
            let many = compact.step_many(&[BranchStepRequest { branch: c, token: d.token, index: RunActionIndex(i as i64) }], BranchView::Full).unwrap();
            let BranchStepOutcome::Stepped(many) = &many[0] else { panic!("step_many faulted") };
            assert_eq!((&stepped.observation, &stepped.action_mask, &stepped.info), (&many.observation, &many.action_mask, &many.info), "step_many equals step");
            outcomes.push((stepped.observation.clone(), stepped.action_mask.clone()));
            compact.close(b).unwrap();
            compact.close(c).unwrap();
        }
        assert_eq!(outcomes[0], outcomes[1], "sample {k}: either identical candidate gives one compact outcome");
        compact.close(probe).unwrap();
        // The dense branch of the same sample ends in the same legal mask width and terminal flags.
        let d = dense.branch(Some(k)).unwrap();
        let dd = dense.decision(d).unwrap();
        let dstep = dense.step(d, dd.token, RunActionIndex(picks[0] as i64), BranchView::Mask).expect("dense pick applied");
        assert_eq!(dstep.action_mask.iter().filter(|m| **m == 1).count(), outcomes[0].1.iter().filter(|m| **m == 1).count(), "same legal actions in both layouts");
        dense.close(d).unwrap();
    }
}

#[test]
fn m40b_retired_family_live_and_branch_enter_the_same_draw_grid() {
    let mut live = start(Character::Ironclad, "jaw_worm", 22);
    let c = combat(&mut live);
    let tap = c.diagnostic_create_card(CardId::DoubleTap, 0, CardDestination::Hand).unwrap();
    let cleave = c.diagnostic_create_card(CardId::Cleave, 0, CardDestination::Hand).unwrap();
    let other = c.diagnostic_create_card(CardId::Cleave, 0, CardDestination::Discard).unwrap();
    let grid = c.diagnostic_create_card(CardId::SecretWeapon, 0, CardDestination::Hand).unwrap();
    play(&mut live, tap);
    play(&mut live, cleave);
    combat(&mut live).diagnostic_apply(EffectKind::Recycle).unwrap();

    let c = combat(&mut live);
    let pairs = c.diagnostic_identical_copies();
    assert!(pairs.contains(&(cleave, other)) || pairs.contains(&(other, cleave)));
    let root = PublicRoot::of(&live, &caps()).expect("retired repeat leaves its original drawable");
    let mut search = PublicSearch::new(root, SearchConfig { research_seed: 31, limits: WIDE }).unwrap();
    let branch = search.branch(Some(0)).unwrap();
    let decision = search.decision(branch).unwrap();
    let pick = action_index(&live, |a| matches!(a, GameAction::Combat(CombatAction::PlayCard { instance, .. }) if *instance == grid)).unwrap();
    step(&mut live, pick);
    search.step(branch, decision.token, RunActionIndex(pick as i64), BranchView::Full).unwrap();
    assert!(live.decision().unwrap().combat.unwrap().selection.is_some());
    let live_law = combat(&mut live).draw_knowledge().unwrap();
    let branch_run = search.branch_run_for_test(branch);
    assert_eq!(combat(branch_run).draw_knowledge(), Ok(live_law));
    assert!(combat(&mut live).identical_pick_rank(cleave, &[4; 32]).is_some());
    assert!(combat(branch_run).identical_pick_rank(cleave, &[4; 32]).is_some());
    let a = PublicRoot::of(&live, &caps()).unwrap();
    let b = search.public_root(branch).unwrap();
    assert_eq!(a, b);
    assert!(a.same_world(&b));
}

#[test]
fn m40b_rekey_draw_classes_are_present_in_the_root_digest_input() {
    let mut live = start(Character::Defect, "jaw_worm", 22);
    let c = combat(&mut live);
    let claw = c.diagnostic_create_card(CardId::Claw, 0, CardDestination::Hand).unwrap();
    c.diagnostic_create_card(CardId::Claw, 0, CardDestination::Discard).unwrap();
    c.diagnostic_apply(EffectKind::Recycle).unwrap();
    play(&mut live, claw);
    assert!(live.public_knowledge_bytes().is_some());
    let root = PublicRoot::of(&live, &caps()).expect("root keeps synchronized classes");
    let mut search = PublicSearch::new(root.clone(), SearchConfig { research_seed: 31, limits: WIDE }).unwrap();
    let branch = search.branch(Some(0)).unwrap();
    assert_eq!(search.public_root(branch).unwrap(), root);
}
