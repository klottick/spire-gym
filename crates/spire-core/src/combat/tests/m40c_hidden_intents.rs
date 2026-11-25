//! M40c (spire-61l stage 3): exact public roots under Runic Dome (design-v4). Every privacy test first proves that the
//! true worlds differ in hidden state, then that their public roots are equal (Run equality: `same_world` in core).
use super::super::belief::{Leak, Unsupported, LEAK};
use super::super::*;
use crate::content::{CardId, Character, EncounterId, EnemyId, RelicId};
use crate::run::{AuthoredRunStart, CardInstance, GameAction, Run, RunRng, StandardCombatStart};

pub(super) fn deck(cards: &[CardId]) -> Vec<CardInstance> {
    cards.iter().enumerate().map(|(i, id)| CardInstance { id: *id, instance: i as u64 + 1, upgrades: 0, misc: 0 }).collect()
}
/// An entered `standard-combat-v1` Run at its opening decision.
pub(super) fn standalone(character: Character, ascension: u8, encounter: EncounterId, cards: &[CardId], relics: Vec<RelicId>, seed: u64) -> Run {
    let start = StandardCombatStart { character, ascension, encounter, burning: None, bottled_cards: vec![], position: None,
        resources: AuthoredRunStart { hp: 70, max_hp: 80, gold: 50, deck: deck(cards), relics, potions: vec![None; if ascension >= 11 { 2 } else { 3 }], relic_counters: Default::default() } };
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&seed.to_le_bytes());
    let mut run = Run::prepare_standard_combat(start, RunRng::new(key, rng::Partition::Test, 0)).unwrap();
    run.enter_standard_combat().unwrap();
    run
}
pub(super) fn combat(run: &Run) -> &Combat { run.state().combat.as_ref().unwrap() }
pub(super) fn combat_mut(run: &mut Run) -> &mut Combat { run.state_mut_for_test().combat.as_mut().unwrap() }
pub(super) fn root(run: &Run) -> Result<Run, Unsupported> { run.public_root_run() }
const DEFENDS: &[CardId] = &[CardId::Defend];
pub(super) fn dome() -> Vec<RelicId> { vec![RelicId::BurningBlood, RelicId::RunicDome] }

/// The hidden part of every enemy: intent, selected move, history, memory (what a public root must not depend on).
pub(super) fn hidden(run: &Run) -> Vec<String> {
    combat(run).state.enemies.iter().map(|e| format!("{:?}{:?}{:?}{:?}", e.intent_id, e.selected_move, e.move_history, e.full_memory)).collect()
}
fn end_turn(run: &mut Run) -> Option<String> {
    let action = run.decision().ok()?.candidates.iter().find(|c| matches!(c.action, GameAction::Combat(CombatAction::EndTurn)))?.action.clone();
    let observed = run.step_observed(&action);
    observed.result.ok()?;
    if run.decision().ok()?.combat.as_ref().is_none_or(|c| c.outcome.is_some()) { return None }
    // The visible trace a watching player sees, without the executed-move identity event (design-v4 §8 T1).
    let trace = observed.effects.iter().filter(|e| !matches!(e.kind, ResolvedEffectKind::EnemyMove { .. }))
        .map(|e| format!("{:?}", e.kind)).collect::<Vec<_>>().join(";");
    Some(trace)
}
/// The public history after `depth` end turns: every decision view plus the visible trace.
fn public_history(run: &mut Run, depth: usize) -> Option<String> {
    let mut h = format!("{:?}", run.decision().ok()?.combat);
    for _ in 0..depth { h.push_str(&end_turn(run)?); h.push_str(&format!("{:?}", run.decision().ok()?.combat)); }
    Some(h)
}

#[test]
fn dome_opening_root_is_supported() {
    for ascension in [0, 20] {
        for encounter in [EncounterId::JawWorm, EncounterId::SphericGuardian, EncounterId::ThreeByrds, EncounterId::ThreeDarklings, EncounterId::CorruptHeart, EncounterId::Chosen] {
            let run = standalone(Character::Ironclad, ascension, encounter, DEFENDS, dome(), 3);
            assert!(view::public_rules(&combat(&run).state).hide_intents);
            assert!(root(&run).is_ok(), "A{ascension} {encounter:?}: {:?}", root(&run).err());
        }
    }
}

/// T1 (core): seeds with an identical public history (views and visible trace) have equal roots; pairs whose hidden
/// state differs are required at every depth.
#[test]
fn dome_natural_pairs_have_equal_roots() {
    let mut report = vec![];
    let mut per_depth = std::collections::BTreeMap::<usize, usize>::new();
    for encounter in [EncounterId::JawWorm, EncounterId::ThreeByrds, EncounterId::Chosen, EncounterId::ThreeDarklings, EncounterId::TwoLouse] {
        for depth in [0usize, 1, 3] {
            let mut groups: std::collections::BTreeMap<String, Vec<Run>> = Default::default();
            for seed in 0..400u64 {
                let mut run = standalone(Character::Ironclad, 20, encounter, DEFENDS, dome(), seed);
                if let Some(h) = public_history(&mut run, depth) { groups.entry(h).or_default().push(run); }
            }
            let mut pairs = 0;
            for runs in groups.values() {
                for b in &runs[1..] {
                    let a = &runs[0];
                    if hidden(a) == hidden(b) { continue }
                    match (root(a), root(b)) {
                        (Ok(ra), Ok(rb)) => assert!(ra == rb, "{encounter:?} depth {depth}: public-equivalent roots differ"),
                        // Only a shown pre-roll (an executed Bite or Nip) may refuse here: these decks read no intent.
                        (Err(x), Err(y)) => { assert_eq!(x, y, "{encounter:?} depth {depth}: refusal classes differ"); assert_eq!(x, Unsupported::HiddenPrerollShown, "{encounter:?} depth {depth}") }
                        (x, y) => panic!("{encounter:?} depth {depth}: one root refused: {:?} vs {:?}", x.err(), y.err()),
                    }
                    pairs += 1;
                }
            }
            report.push(format!("{encounter:?}@{depth}:{pairs}"));
            *per_depth.entry(depth).or_default() += pairs;
        }
    }
    for (depth, pairs) in &per_depth { assert!(*pairs > 0, "depth {depth}: no natural pair with differing hidden state ({report:?})"); }
    eprintln!("m40c natural pairs: {}", report.join(" "));
}

// ---------------------------------------------------------------- constructed pairs (T2, T2b, T4, T5, T6)

pub(super) fn action(run: &Run, want: impl Fn(&GameAction) -> bool) -> Option<GameAction> {
    run.decision().ok()?.candidates.iter().map(|c| c.action.clone()).find(|a| want(a))
}
pub(super) fn end_turn_action(run: &Run) -> Option<GameAction> { action(run, |a| matches!(a, GameAction::Combat(CombatAction::EndTurn))) }
/// Play a hand card of kind `id` at enemy `target` (0-based) or untargeted.
pub(super) fn play(run: &Run, id: CardId, target: Option<usize>) -> Option<GameAction> {
    let hand = &combat(run).state.piles.hand;
    action(run, |a| match a {
        GameAction::Combat(CombatAction::PlayCard { instance, target: t }) => hand.iter().any(|c| c.full().is_some_and(|f| f.instance.0 == *instance && f.id == id))
            && t.map(|t| t.0 as usize) == target.map(|i| i + 1),
        _ => false,
    })
}
fn living_target(run: &Run) -> Option<usize> { combat(run).state.enemies.iter().position(|e| e.life == ActorLife::Active && e.hp > 0) }
/// Attack the first living enemy with `card` when possible, else end the turn.
pub(super) fn attack(card: CardId) -> impl Fn(&Run) -> Option<GameAction> + Copy {
    move |run: &Run| living_target(run).and_then(|t| play(run, card, Some(t))).or_else(|| end_turn_action(run))
}
pub(super) fn passive(run: &Run) -> Option<GameAction> { end_turn_action(run) }

/// Perturb every enemy's AI stream key (hidden: future rolls differ, the public history so far does not).
pub(super) fn perturb_ai(run: &mut Run) {
    for e in &mut combat_mut(run).state.enemies {
        if let Some(s) = e.full_ai_rng.as_ref() { e.full_ai_rng = Some(rng::StreamState::new(s.id().clone(), [0x5A; 32], s.word_index()).unwrap()); }
    }
}
#[derive(Debug, Default)]
pub(super) struct Stats { pub supported_differ: usize, pub refused: std::collections::BTreeMap<String, usize>, pub decisions: usize, pub phase_two: usize }
fn view_of(run: &Run) -> String { format!("{:?}", run.decision().ok().and_then(|d| d.combat)) }
/// Check one public-equivalent pair at a decision: equal roots (and valid canonical worlds) or one refusal class.
pub(super) fn check_pair(a: &Run, b: &Run, stats: &mut Stats, what: &str) {
    stats.decisions += 1;
    // The records are public knowledge: equal for public-equivalent worlds whether or not a root exists.
    assert!(combat(a).state.knowledge.moves == combat(b).state.knowledge.moves, "{what}: public-equivalent worlds hold different hidden-move records");
    match (root(a), root(b)) {
        (Ok(ra), Ok(rb)) => {
            assert!(ra == rb, "{what}: public-equivalent roots differ");
            combat(&ra).validate_runtime_for_test().unwrap_or_else(|e| panic!("{what}: canonical root invalid: {e:?}"));
            let s = ra.belief_sample(&crate::run::research_sample_key(&crate::run::research_root(5), &[2; 32], 1)).unwrap();
            combat(&s).validate_runtime_for_test().unwrap_or_else(|e| panic!("{what}: sample invalid: {e:?}"));
            if hidden(a) != hidden(b) { stats.supported_differ += 1 }
            if combat(a).state.enemies.iter().any(|e| matches!(e.full_memory, Some(enemies::EnemyMemory::AwakenedOne { phase: enemies::AwakenedPhase::Second, .. })) && e.hp > 0) { stats.phase_two += 1 }
        }
        (Err(x), Err(y)) => { assert_eq!(x, y, "{what}: refusal classes differ"); *stats.refused.entry(x.reason().to_owned()).or_default() += 1 }
        (x, y) => panic!("{what}: one root refused: {:?} vs {:?}", x.err(), y.err()),
    }
}
/// Step a pair with `policy` (chosen on world a, applied to both) for up to `steps` decisions while it stays
/// public-equivalent (equal views and visible trace), checking every decision.
pub(super) fn run_pair(a: Run, perturb: impl FnOnce(&mut Run), policy: impl Fn(&Run) -> Option<GameAction>, steps: usize, what: &str) -> Stats {
    run_pair_with(a, perturb, policy, steps, what, true)
}
/// `keep_moves`: compare executed-move events too (they are public history); the truncation pairs drop them because an
/// unidentifiable execution is exactly what they construct.
pub(super) fn run_pair_with(mut a: Run, perturb: impl FnOnce(&mut Run), policy: impl Fn(&Run) -> Option<GameAction>, steps: usize, what: &str, keep_moves: bool) -> Stats {
    let mut b = a.clone();
    perturb(&mut b);
    let mut stats = Stats::default();
    for _ in 0..steps {
        if view_of(&a) != view_of(&b) { break }
        check_pair(&a, &b, &mut stats, what);
        let Some(act) = policy(&a) else { break };
        let (oa, ob) = (a.step_observed(&act), b.step_observed(&act));
        if oa.result.is_err() || ob.result.is_err() { break }
        // Executed moves are public history (design §1), so the constructed-pair trace keeps them.
        let trace = |o: &Observed<_>| o.effects.iter().filter(|e| keep_moves || !matches!(e.kind, ResolvedEffectKind::EnemyMove { .. })).map(|e| format!("{:?}", e.kind)).collect::<Vec<_>>();
        if trace(&oa) != trace(&ob) { break }
        if a.decision().ok().and_then(|d| d.combat).is_none_or(|c| c.outcome.is_some()) { break }
    }
    stats
}
fn strikes(n: usize) -> Vec<CardId> { vec![CardId::Strike; n] }
pub(super) fn add_player_power(run: &mut Run, id: &'static str, amount: i32) {
    combat_mut(run).state.player_full_powers.push(FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } });
}
fn set_enemy_hp(run: &mut Run, index: usize, hp: i32) { let e = &mut combat_mut(run).state.enemies[index]; e.hp = hp.min(e.max_hp); }
fn set_nip(run: &mut Run, value: i32) {
    for e in &mut combat_mut(run).state.enemies {
        if let Some(d) = e.full_damage_entries.get_mut(&FullIntentId::from_name("darkling_nip")) { d.base = value }
        if let Some(enemies::EnemyMemory::Darkling { nip, .. }) = e.full_memory.as_mut() { *nip = value }
    }
    effects::refresh_enemy_values(&mut combat_mut(run).state).unwrap();
}

/// Scenario table: (name, encounter, character, deck, relics, setup, policy, steps); every pair is checked at each
/// decision. `seeds` worlds each; the assertions on the accumulated stats live in the tests below.
pub(super) fn scenario(name: &str, encounter: EncounterId, character: Character, cards: &[CardId], relics: Vec<RelicId>,
    setup: impl Fn(&mut Run), policy: impl Fn(&Run) -> Option<GameAction> + Copy, steps: usize, seeds: u64) -> Stats {
    let mut total = Stats::default();
    for seed in 0..seeds {
        let mut run = standalone(character, 20, encounter, cards, relics.clone(), seed);
        setup(&mut run);
        let s = run_pair(run, perturb_ai, policy, steps, &format!("{name} seed {seed}"));
        total.supported_differ += s.supported_differ;
        total.decisions += s.decisions;
        total.phase_two += s.phase_two;
        for (k, v) in s.refused { *total.refused.entry(k).or_default() += v }
    }
    eprintln!("m40c scenario {name}: {total:?}");
    total
}
fn ironclad_dome() -> Vec<RelicId> { dome() }

#[test]
fn t2_overrides_retirements_and_chains_keep_equal_roots() {
    for (name, encounter, cards) in [
        ("byrd ground", EncounterId::ThreeByrds, strikes(10)),
        ("shelled armor break", EncounterId::ShelledParasite, strikes(10)),
        ("slime split", EncounterId::LargeSlime, strikes(10)),
        ("louse kill", EncounterId::TwoLouse, strikes(10)),
        ("fungi kill", EncounterId::TwoFungiBeasts, strikes(10)),
        ("looter escape", EncounterId::Looter, vec![CardId::Defend]),
        ("writhing chain", EncounterId::WrithingMass, strikes(10)),
        ("darkling down", EncounterId::ThreeDarklings, strikes(10)),
    ] {
        let s = scenario(name, encounter, Character::Ironclad, &cards, ironclad_dome(), |_| {}, attack(CardId::Strike), 60, 12);
        // Darklings nip early, and a shown pre-roll refuses for the actor's life (design-v4 §6): both worlds agree.
        if name == "darkling down" { assert!(s.refused.get("a hidden pre-rolled value was shown by damage").copied().unwrap_or(0) > 0, "{s:?}"); continue }
        assert!(s.supported_differ > 0, "{name}: no supported pair with differing hidden state: {s:?}");
    }
}

#[test]
fn t2_awakened_rebirth_phase_two_is_supported() {
    // Down the Awakened One by the player's attacks, let it be reborn, then form the pair in phase 2.
    let setup = |r: &mut Run| {
        let i = combat(r).state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::AwakenedOne)).unwrap();
        set_enemy_hp(r, i, 12);
        for _ in 0..40 {
            if matches!(combat(r).state.enemies[i].full_memory, Some(enemies::EnemyMemory::AwakenedOne { phase: enemies::AwakenedPhase::Second, .. })) && combat(r).state.enemies[i].hp > 0 { break }
            let e = &combat(r).state.enemies[i];
            let a = if e.life == ActorLife::Active && e.hp > 0 && matches!(e.full_memory, Some(enemies::EnemyMemory::AwakenedOne { phase: enemies::AwakenedPhase::First, .. })) { play(r, CardId::Strike, Some(i)).or_else(|| end_turn_action(r)) } else { end_turn_action(r) };
            r.step(&a.unwrap()).unwrap();
        }
    };
    let s = scenario("awakened rebirth", EncounterId::AwakenedOne, Character::Ironclad, &strikes(10), ironclad_dome(), setup, passive, 30, 8);
    assert!(s.supported_differ > 0 && s.phase_two > 0, "phase-2 Awakened roots are supported: {s:?}");
}

/// T4 (C-MEM): equal root memory across worlds whose roll-written memory differs.
#[test]
fn t4_move_dependent_memory_is_replayed() {
    for encounter in [EncounterId::BookOfStabbing, EncounterId::Champ, EncounterId::BronzeAutomaton, EncounterId::Nemesis, EncounterId::TimeEater, EncounterId::CorruptHeart] {
        // The Automaton's Bronze Orbs exist only after its first move: form the pair once they are on the board.
        let setup = |r: &mut Run| if encounter == EncounterId::BronzeAutomaton { for _ in 0..1 { let a = end_turn_action(r).unwrap(); r.step(&a).unwrap(); } };
        let s = scenario(&format!("c-mem {encounter:?}"), encounter, Character::Ironclad, &[CardId::Defend], ironclad_dome(), setup, passive, 12, 10);
        assert!(s.supported_differ > 0, "{encounter:?}: {s:?}");
    }
}

/// A Darkling pair whose hidden Nip differs (before any Nip executes) has equal roots.
#[test]
fn t2_darkling_nip_pair_has_equal_roots() {
    let mut checked = 0;
    for seed in 0..6u64 {
        let a = standalone(Character::Ironclad, 20, EncounterId::ThreeDarklings, DEFENDS, dome(), seed);
        let mut b = a.clone();
        set_nip(&mut b, 13);
        let mut a2 = a.clone();
        set_nip(&mut a2, 9);
        assert!(hidden(&a2) != hidden(&b));
        let mut stats = Stats::default();
        check_pair(&a2, &b, &mut stats, "nip pair");
        checked += stats.decisions;
        assert!(root(&a2).is_ok());
    }
    assert!(checked > 0);
}

/// Same-shape ambiguity (rule 2c) and truncation (rule 2d): pairs refuse with one class, then become supported.
#[test]
fn t2_ambiguous_executions_refuse_then_recover() {
    for (name, encounter, power, amount) in [("growth intangible", EncounterId::SpireGrowth, "intangible_player", 2), ("maw intangible", EncounterId::Maw, "intangible_player", 3),
        ("growth buffer", EncounterId::SpireGrowth, "buffer", 3), ("growth intangible long", EncounterId::SpireGrowth, "intangible_player", 9)] {
        let s = scenario(name, encounter, Character::Ironclad, &[CardId::Defend], ironclad_dome(), |r| add_player_power(r, power, amount), passive, 30, 10);
        assert!(s.refused.get("an executed enemy move was not identifiable").copied().unwrap_or(0) > 0, "{name}: {s:?}");
        if !name.ends_with("long") { assert!(s.supported_differ > 0, "{name}: {s:?}"); }
    }
}

#[test]
fn t2b_truncated_executions_are_not_identifiable() {
    // Darkling at A20 (Chomp 2 x 9) with Nip = 9, killed by retaliation on its first hit: Nip vs truncated Chomp.
    let s = scenario("darkling truncation", EncounterId::ThreeDarklings, Character::Ironclad, &[CardId::Defend], ironclad_dome(),
        |r| { set_nip(r, 9); add_player_power(r, "thorns", 30); for i in 0..3 { set_enemy_hp(r, i, 5); } }, passive, 12, 12);
    assert!(s.refused.get("a hidden pre-rolled value was shown by damage").copied().unwrap_or(0) > 0, "{s:?}");
    // Awakened One under Intangible plus Thorns: Soul Strike vs Slash, downed mid-move; phase 2 is supported again.
    let s = scenario("awakened truncation", EncounterId::AwakenedOne, Character::Ironclad, &[CardId::Defend], ironclad_dome(),
        |r| { add_player_power(r, "intangible_player", 99); add_player_power(r, "thorns", 30); let i = combat(r).state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(EnemyId::AwakenedOne)).unwrap(); set_enemy_hp(r, i, 20); }, passive, 30, 8);
    assert!(s.supported_differ > 0, "{s:?}");
    // Byrd under Intangible plus Thorns, killed mid-move.
    let s = scenario("byrd truncation", EncounterId::ThreeByrds, Character::Ironclad, &[CardId::Defend], ironclad_dome(),
        |r| { add_player_power(r, "intangible_player", 99); add_player_power(r, "thorns", 3); for i in 0..3 { set_enemy_hp(r, i, 3); } }, passive, 20, 8);
    assert!(s.decisions > 0, "{s:?}");
}

/// Rule 2d is edge-triggered: a Darkling revival and an Awakened rebirth (moves executed at hp 0) leave no mark.
#[test]
fn revival_and_rebirth_leave_no_truncation_mark() {
    for (encounter, hp) in [(EncounterId::ThreeDarklings, 6), (EncounterId::AwakenedOne, 6)] {
        let mut run = standalone(Character::Ironclad, 20, encounter, &strikes(10), dome(), 4);
        let idx = combat(&run).state.enemies.iter().position(|e| matches!(e.enemy_id, ActorEnemyId::Full(EnemyId::AwakenedOne | EnemyId::Darkling))).unwrap();
        set_enemy_hp(&mut run, idx, hp);
        let before = hidden_moves::TRUNCATIONS.with(|t| t.borrow().values().sum::<u64>());
        let mut revived = false;
        for _ in 0..120 {
            // Down only enemy 0 (by the player, never mid-move), then wait for its revival.
            let first = &combat(&run).state.enemies[idx];
            let a = if first.life == ActorLife::Active && first.hp > 0 && !revived && !matches!(first.intent_id.as_str(), "awa_rebirth") { play(&run, CardId::Strike, Some(idx)).or_else(|| end_turn_action(&run)) } else { end_turn_action(&run) };
            let Some(a) = a else { break };
            if run.step(&a).is_err() || run.decision().ok().and_then(|d| d.combat).is_none_or(|c| c.outcome.is_some()) { break }
            let first = &combat(&run).state.enemies[idx];
            revived |= first.move_history.iter().any(|m| matches!(m.as_str(), "darkling_reincarnate" | "awa_rebirth")) && first.hp > 0;
            if revived { break }
        }
        let after = hidden_moves::TRUNCATIONS.with(|t| t.borrow().values().sum::<u64>());
        assert!(revived, "{encounter:?}: no revival reached");
        assert_eq!(before, after, "{encounter:?}: a revival was marked as a truncation");
        assert!(combat(&run).state.knowledge.moves.enemies.values().all(|r| r.ambiguous.is_empty()), "{encounter:?}");
    }
}

// ---------------------------------------------------------------- T6 refusals and lifecycle

fn read_record(run: &Run, index: usize) -> hidden_moves::EnemyMoves {
    combat(run).state.knowledge.moves.enemies.get(&combat(run).state.enemies[index].instance_id).cloned().unwrap_or_default()
}
fn step(run: &mut Run, pick: impl FnOnce(&Run) -> Option<GameAction>, what: &str) { let a = pick(run).unwrap_or_else(|| panic!("{what}: no action")); run.step(&a).unwrap_or_else(|e| panic!("{what}: {e:?}")); }

#[test]
fn t6_card_reads_refuse_until_the_move_executes() {
    // Spot Weakness on a random species: refused until the read roll executes; on a deterministic species: supported.
    let mut run = standalone(Character::Ironclad, 20, EncounterId::JawWorm, &[CardId::SpotWeakness], vec![RelicId::RunicDome], 1);
    assert!(root(&run).is_ok());
    step(&mut run, |r| play(r, CardId::SpotWeakness, Some(0)), "spot weakness");
    assert!(read_record(&run, 0).read);
    assert_eq!(root(&run).err(), Some(Unsupported::HiddenIntentRead));
    step(&mut run, |r| end_turn_action(r), "end turn");
    assert!(root(&run).is_ok(), "after execution: {:?}", root(&run).err());
    let mut run = standalone(Character::Ironclad, 20, EncounterId::Cultist, &[CardId::SpotWeakness], vec![RelicId::RunicDome], 1);
    step(&mut run, |r| play(r, CardId::SpotWeakness, Some(0)), "spot weakness on a cultist");
    assert!(root(&run).is_ok(), "a deterministic species' read leaks nothing: {:?}", root(&run).err());
    // Go for the Eyes (Defect) from hand.
    let mut run = standalone(Character::Defect, 20, EncounterId::JawWorm, &[CardId::GoForTheEyes], vec![RelicId::RunicDome], 1);
    step(&mut run, |r| play(r, CardId::GoForTheEyes, Some(0)), "go for the eyes");
    assert_eq!(root(&run).err(), Some(Unsupported::HiddenIntentRead));
}

#[test]
fn t6_fear_no_evil_is_marked_on_every_play_path() {
    // From hand: the play step returns at a decision with the read already marked (no boundary between read and mark).
    let mut run = standalone(Character::Watcher, 20, EncounterId::JawWorm, &[CardId::FearNoEvil], vec![RelicId::RunicDome], 2);
    step(&mut run, |r| play(r, CardId::FearNoEvil, Some(0)), "fear no evil");
    assert!(read_record(&run, 0).read, "hand play");
    assert_eq!(root(&run).err(), Some(Unsupported::HiddenIntentRead));
    // Duplication Potion repeat and Distilled Chaos autoplay (no hand play first).
    for potion in [crate::content::PotionId::DuplicationPotion, crate::content::PotionId::DistilledChaos] {
        let start = StandardCombatStart { character: Character::Watcher, ascension: 20, encounter: EncounterId::JawWorm, burning: None, bottled_cards: vec![], position: None,
            resources: AuthoredRunStart { hp: 70, max_hp: 80, gold: 50, deck: deck(&[CardId::FearNoEvil; 6]), relics: vec![RelicId::RunicDome], potions: vec![Some(potion), None], relic_counters: Default::default() } };
        let mut run = Run::prepare_standard_combat(start, RunRng::new([3; 32], rng::Partition::Test, 0)).unwrap();
        run.enter_standard_combat().unwrap();
        step(&mut run, |r| action(r, |a| matches!(a, GameAction::Combat(CombatAction::UsePotion { slot: 0, .. }))), "potion");
        if potion == crate::content::PotionId::DuplicationPotion {
            assert!(!read_record(&run, 0).read);
            step(&mut run, |r| play(r, CardId::FearNoEvil, Some(0)), "duplicated fear no evil");
        }
        assert!(read_record(&run, 0).read, "{potion:?}");
        assert_eq!(root(&run).err(), Some(Unsupported::HiddenIntentRead), "{potion:?}");
    }
}

#[test]
fn t6_a_read_survives_a_reactive_re_roll() {
    let mut run = standalone(Character::Defect, 20, EncounterId::WrithingMass, &[CardId::GoForTheEyes; 5], vec![RelicId::RunicDome], 1);
    let before = read_record(&run, 0).pending.unwrap().rolls.len();
    step(&mut run, |r| play(r, CardId::GoForTheEyes, Some(0)), "go for the eyes on writhing mass");
    let r = read_record(&run, 0);
    assert!(r.pending.as_ref().unwrap().rolls.len() > before, "the hit re-rolled the Mass");
    assert!(r.read);
    assert_eq!(root(&run).err(), Some(Unsupported::HiddenIntentRead));
}

#[test]
fn t6_a_shown_pre_roll_refuses() {
    for seed in 0..20u64 {
        let mut run = standalone(Character::Ironclad, 20, EncounterId::TwoLouse, DEFENDS, dome(), seed);
        for _ in 0..6 {
            if combat(&run).state.knowledge.moves.enemies.values().any(|r| r.shown) {
                assert_eq!(root(&run).err(), Some(Unsupported::HiddenPrerollShown));
                return
            }
            let Some(a) = end_turn_action(&run) else { break };
            run.step(&a).unwrap();
        }
    }
    panic!("no Bite executed");
}

/// The ambiguity window refuses exactly while a pending roll reads the entry: the two decisions after an ambiguous
/// Spire Growth execution refuse, the third does not (Maw: one).
#[test]
fn t6_ambiguity_window_is_exact() {
    let mut seen = 0;
    for (encounter, window) in [(EncounterId::SpireGrowth, 2usize), (EncounterId::Maw, 1)] {
        for seed in 0..10u64 {
            for amount in [1, 2] {
                let mut run = standalone(Character::Ironclad, 20, encounter, DEFENDS, dome(), seed);
                for _ in 0..2 { let a = end_turn_action(&run).unwrap(); run.step(&a).unwrap(); }
                add_player_power(&mut run, "intangible_player", amount);
                let mut pattern = vec![];
                let mut marked_at = None;
                for d in 0..6 {
                    if marked_at.is_none() && !read_record(&run, 0).ambiguous.is_empty() { marked_at = Some(d) }
                    pattern.push(root(&run).err() == Some(Unsupported::HiddenMoveAmbiguous));
                    let Some(a) = end_turn_action(&run) else { break };
                    if run.step(&a).is_err() || run.decision().ok().and_then(|d| d.combat).is_none_or(|c| c.outcome.is_some()) { break }
                }
                let Some(m) = marked_at else { continue };
                if read_record(&run, 0).ambiguous.len() != 1 || pattern.len() < m + window + 1 { continue }
                let expect = (0..=window).map(|i| i < window).collect::<Vec<_>>();
                assert_eq!(pattern[m..=m + window], expect[..], "{encounter:?} seed {seed} amount {amount}: {pattern:?}");
                seen += 1;
            }
        }
    }
    assert!(seen > 0);
}

/// An unclassified change to a C-MEM species' memory while its roll is pending makes the record opaque.
#[test]
fn t6_unclassified_change_is_opaque() {
    let mut run = standalone(Character::Ironclad, 20, EncounterId::BookOfStabbing, DEFENDS, dome(), 1);
    let state = &mut combat_mut(&mut run).state;
    let before = hidden_moves::snapshot(state).unwrap();
    if let Some(enemies::EnemyMemory::BookOfStabbing { stab_count }) = state.enemies[0].full_memory.as_mut() { *stab_count += 1 }
    hidden_moves::observe(state, &Effect { source: Source::None, target: None, kind: EffectKind::Decision }, before);
    assert!(read_record(&run, 0).opaque);
    assert_eq!(root(&run).err(), Some(Unsupported::HiddenIntents));
}

// ---------------------------------------------------------------- T9 census, digest

#[test]
fn t9_classification_tables_are_consistent() {
    use EnemyId::*;
    let all = [Cultist, JawWorm, GremlinNob, Lagavulin, Sentry, Guardian, Hexaghost, LouseRed, LouseGreen, AcidSlimeS, AcidSlimeM, AcidSlimeL, SpikeSlimeS, SpikeSlimeM, SpikeSlimeL, SlimeBoss, SphericGuardian, Byrd, Chosen, Snecko, Centurion, Mystic, GremlinLeader, GremlinMad, GremlinSneaky, GremlinFat, GremlinShield, GremlinWizard, SlaverBlue, Taskmaster, SlaverRed, BookOfStabbing, Champ, Collector, TorchHead, BronzeAutomaton, BronzeOrb, OrbWalker, Repulsor, Exploder, Spiker, Darkling, SpireGrowth, Transient, Maw, WrithingMass, GiantHead, Nemesis, Reptomancer, Dagger, AwakenedOne, TimeEater, Deca, Donu, SpireShield, SpireSpear, CorruptHeart, FungiBeast, Looter, Mugger, ShelledParasite, SnakePlant, BanditPointy, BanditLeader, BanditBear];
    for id in all {
        assert!(!(hidden_moves::override_species(id) && hidden_moves::replay_memory(id)), "{id:?}: an override species must not be C-MEM");
        assert!(!hidden_moves::replay_memory(id) || hidden_moves::random_law(id), "{id:?}");
        assert_eq!(hidden_moves::read_window(id) > 0, hidden_moves::random_law(id), "{id:?}");
    }
    let shapes = all.iter().filter_map(|id| hidden_moves::same_shape(*id).map(|s| (*id, s))).collect::<Vec<_>>();
    assert_eq!(shapes.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [SpireGrowth, Maw]);
    assert_eq!(all.iter().filter(|id| hidden_moves::chain_species(**id)).collect::<Vec<_>>(), [&WrithingMass]);
    assert_eq!(all.iter().filter(|id| hidden_moves::preroll_move(**id).is_some()).count(), 3);
    assert_eq!((hidden_moves::read_window(Maw), hidden_moves::read_window(SpireGrowth)), (1, 2));
}

/// Equal views with different public records give different knowledge bytes (the digest covers the records).
#[test]
fn digest_covers_the_records() {
    let mut a = standalone(Character::Ironclad, 20, EncounterId::SphericGuardian, DEFENDS, dome(), 1);
    step(&mut a, |r| end_turn_action(r), "end turn");
    let mut b = a.clone();
    let other = FullIntentId::from_name("sphere_harden");
    let h = combat(&b).state.enemies[0].instance_id.clone();
    let state = &mut combat_mut(&mut b).state;
    state.enemies[0].move_history[0] = ActorIntentId::Full(other);
    state.knowledge.moves.enemies.get_mut(&h).unwrap().pending.as_mut().unwrap().history[0] = other;
    let (ra, rb) = (root(&a).unwrap(), root(&b).unwrap());
    assert_eq!(format!("{:?}", ra.decision().unwrap().combat), format!("{:?}", rb.decision().unwrap().combat), "equal views");
    assert_ne!(ra.public_knowledge_bytes(), rb.public_knowledge_bytes(), "the records differ, so the digests must");
}

// ---------------------------------------------------------------- T3 exact law

fn outcome(run: &Run) -> String {
    combat(run).state.enemies.iter().map(|e| format!("{}:{:?}", e.intent_id.as_str(), e.full_memory)).collect::<Vec<_>>().join("|")
}
fn law(paths: Vec<(String, f64)>) -> std::collections::BTreeMap<String, f64> {
    let mut m = std::collections::BTreeMap::new();
    for (k, p) in paths { *m.entry(k).or_insert(0.0) += p }
    m
}
fn assert_same_law(a: &std::collections::BTreeMap<String, f64>, b: &std::collections::BTreeMap<String, f64>, what: &str) {
    assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>(), "{what}: supports differ");
    for (k, p) in a { assert!((p - b[k]).abs() < 1e-12, "{what}: {k}: {p} vs {}", b[k]); }
}

/// The law of each hidden roll the belief replays equals the law of the gameplay roll path, enumerated exactly (the
/// reference is the real end turn on a clone of the live world, not the replay's selection call).
#[test]
fn t3_replay_law_equals_the_gameplay_roll_law() {
    let key = crate::run::research_sample_key(&crate::run::research_root(9), &[4; 32], 0);
    let mut checked = 0;
    for encounter in [EncounterId::Chosen, EncounterId::Snecko, EncounterId::BookOfStabbing, EncounterId::Champ, EncounterId::Nemesis,
        EncounterId::GiantHead, EncounterId::SpireGrowth, EncounterId::Maw, EncounterId::CorruptHeart, EncounterId::ShieldAndSpear, EncounterId::Looter] {
        let mut run = standalone(Character::Ironclad, 20, encounter, DEFENDS, dome(), 7);
        for turn in 0..5 {
            let Some(game) = rng::script::enumerate(|| { let mut r = run.clone(); let a = end_turn_action(&r).unwrap(); r.step(&a).unwrap(); outcome(&r) }, 50_000) else { break };
            let game = law(game);
            let a = end_turn_action(&run).unwrap();
            run.step(&a).unwrap();
            if run.decision().ok().and_then(|d| d.combat).is_none_or(|c| c.outcome.is_some()) { break }
            let Ok(r) = root(&run) else { continue };
            let replay = law(rng::script::enumerate(|| outcome(&r.belief_sample(&key).unwrap()), 50_000).expect("bounded replay"));
            assert_same_law(&game, &replay, &format!("{encounter:?} turn {turn}"));
            checked += 1;
        }
    }
    assert!(checked >= 20, "{checked} laws compared");
}

/// The exact law of one Writhing Mass roll (`act3/writhing.rs:5-32`), computed independently on bucket masses.
fn writhing_law(last: Option<&str>, parasite_used: bool, first: bool) -> std::collections::BTreeMap<&'static str, f64> {
    let mut out = std::collections::BTreeMap::new();
    if first { out.insert("writhing_multi", 0.33); out.insert("writhing_block", 0.33); out.insert("writhing_debuff", 0.34); return out }
    let mut add = |k: &'static str, p: f64| *out.entry(k).or_insert(0.0) += p;
    let (c1, c2) = (1_677_722f64 / 16_777_216f64, 6_710_887f64 / 16_777_216f64);
    let mut m = [0.10, 0.10, 0.20, 0.30, 0.30];
    for _ in 0..10_000 {
        let mut n = [0.0f64; 5];
        if last != Some("writhing_multi") { add("writhing_multi", m[0]) } else { n[1] += m[0] * 10. / 90.; n[2] += m[0] * 20. / 90.; n[3] += m[0] * 30. / 90.; n[4] += m[0] * 30. / 90. }
        if !parasite_used && last != Some("writhing_parasite") { add("writhing_parasite", m[1]) } else {
            add("writhing_multi", m[1] * c1); let r = m[1] * (1. - c1); n[2] += r * 20. / 80.; n[3] += r * 30. / 80.; n[4] += r * 30. / 80.
        }
        if last != Some("writhing_debuff") { add("writhing_debuff", m[2]) } else {
            n[0] += m[2] * c2 * 0.5; n[1] += m[2] * c2 * 0.5; let r = m[2] * (1. - c2); n[3] += r * 0.5; n[4] += r * 0.5
        }
        if last != Some("writhing_block") { add("writhing_block", m[3]) } else { n[0] += m[3] * 10. / 70.; n[1] += m[3] * 10. / 70.; n[2] += m[3] * 20. / 70.; n[3] += m[3] * 30. / 70. }
        if last != Some("writhing_big") { add("writhing_big", m[4]) } else { n[0] += m[4] * 0.25; n[1] += m[4] * 0.25; n[2] += m[4] * 0.5 }
        m = n;
        if m.iter().sum::<f64>() < 1e-15 { break }
    }
    out
}

/// The whole Writhing Mass chain (setup roll plus Reactive re-rolls) is redrawn from its exact Markov law.
#[test]
fn t3_writhing_chain_follows_its_closed_form_law() {
    let mut run = standalone(Character::Ironclad, 20, EncounterId::WrithingMass, &strikes(10), dome(), 3);
    for _ in 0..3 { if let Some(a) = play(&run, CardId::Strike, Some(0)) { run.step(&a).unwrap(); } }
    let chain = read_record(&run, 0).pending.unwrap();
    assert!(chain.rolls.len() >= 3, "{} rolls", chain.rolls.len());
    let r = root(&run).unwrap();
    let start = chain.history.len();
    let k = chain.rolls.len();
    // Exact law of the chain tuple.
    let mut exact: std::collections::BTreeMap<Vec<&'static str>, f64> = [(vec![], 1.0)].into_iter().collect();
    for i in 0..k {
        let mut next = std::collections::BTreeMap::new();
        for (tuple, p) in &exact {
            for (m, q) in writhing_law(tuple.last().copied(), false, i == 0 && start == 0) { let mut t = tuple.clone(); t.push(m); *next.entry(t).or_insert(0.0) += p * q; }
        }
        exact = next;
    }
    let n = 4000u64;
    let mut seen: std::collections::BTreeMap<Vec<String>, u64> = Default::default();
    for s in 0..n {
        let key = crate::run::research_sample_key(&crate::run::research_root(13), &[6; 32], s);
        let w = r.belief_sample(&key).unwrap();
        let h = &combat(&w).state.enemies[0].move_history;
        *seen.entry(h[start..start + k].iter().map(|m| m.as_str().to_owned()).collect()).or_default() += 1;
    }
    let mut chi = 0.0;
    let mut cells = 0;
    for (t, p) in &exact {
        let e = p * n as f64;
        let o = seen.get(&t.iter().map(|s| s.to_string()).collect::<Vec<_>>()).copied().unwrap_or(0) as f64;
        if e > 0.0 { chi += (o - e) * (o - e) / e; cells += 1; }
    }
    for t in seen.keys() { assert!(exact.contains_key(&t.iter().map(|s| -> &'static str { Box::leak(s.clone().into_boxed_str()) }).collect::<Vec<_>>()), "impossible chain {t:?}"); }
    let df = (cells - 1) as f64;
    assert!(chi < df + 5.0 * (2.0 * df).sqrt() + 10.0, "chi-square {chi} over {cells} cells");
    // Not concentrated on the true chain.
    let truth = combat(&run).state.enemies[0].move_history[start..start + k].iter().map(|m| m.as_str().to_owned()).collect::<Vec<_>>();
    assert!(seen.get(&truth).copied().unwrap_or(0) < n * 9 / 10);
}

// ---------------------------------------------------------------- negative controls (design-v4 §8)

/// Run `test` with leak `l` forced on; it must fail.
fn red_under(l: Leak, test: fn(), what: &str) {
    LEAK.with(|v| v.set(Some(l)));
    let r = std::panic::catch_unwind(test);
    LEAK.with(|v| v.set(None));
    assert!(r.is_err(), "{what} stayed green under {l:?}");
}
#[test] fn leak_keep_hidden_move_fails_t1() { red_under(Leak::KeepHiddenMove, dome_natural_pairs_have_equal_roots, "T1") }
#[test] fn leak_record_post_roll_memory_fails_t4() { red_under(Leak::RecordPostRollMemory, t4_move_dependent_memory_is_replayed, "T4") }
#[test] fn leak_replay_live_stream_fails_t1() { red_under(Leak::ReplayLiveStream, dome_natural_pairs_have_equal_roots, "T1") }
#[test] fn leak_digest_without_moves_fails_digest() { red_under(Leak::DigestWithoutMoves, digest_covers_the_records, "digest") }
#[test] fn leak_no_monitor_fails_opaque() { red_under(Leak::NoMonitor, t6_unclassified_change_is_opaque, "T6 opaque") }
#[test] fn leak_raw_chain_history_fails_t2() { red_under(Leak::RawChainHistory, t2_overrides_retirements_and_chains_keep_equal_roots, "T2 overrides") }
#[test] fn leak_keep_retired_residue_fails_t2() { red_under(Leak::KeepRetiredResidue, t2_overrides_retirements_and_chains_keep_equal_roots, "T2 retirement") }
#[test] fn leak_skip_mark_spot_fails_t6() { red_under(Leak::SkipMarkSpot, t6_card_reads_refuse_until_the_move_executes, "T6 Spot Weakness") }
#[test] fn leak_skip_mark_eyes_fails_t6() { red_under(Leak::SkipMarkEyes, t6_card_reads_refuse_until_the_move_executes, "T6 Go for the Eyes") }
#[test] fn leak_skip_mark_fne_fails_t6() { red_under(Leak::SkipMarkFne, t6_fear_no_evil_is_marked_on_every_play_path, "T6 Fear No Evil") }
#[test] fn leak_live_memory_keeps_nip_fails_t2() { red_under(Leak::LiveMemoryKeepsNip, t2_darkling_nip_pair_has_equal_roots, "T2 Nip pair") }
#[test] fn leak_clear_read_on_reroll_fails_t6() { red_under(Leak::ClearReadOnReroll, t6_a_read_survives_a_reactive_re_roll, "T6 re-roll") }
#[test] fn leak_skip_preroll_shown_fails_t6() { red_under(Leak::SkipPrerollShown, t6_a_shown_pre_roll_refuses, "T6 pre-roll") }
#[test] fn leak_accept_ambiguous_fails_t2() { red_under(Leak::AcceptAmbiguous, t2_ambiguous_executions_refuse_then_recover, "T2 ambiguity") }
#[test] fn leak_early_ambiguity_clear_fails_t6() { red_under(Leak::EarlyAmbiguityClear, t6_ambiguity_window_is_exact, "T6 window") }
#[test] fn leak_ambiguity_stand_in_from_later_fails_t2() { red_under(Leak::AmbiguityStandInFromLater, t2_ambiguous_executions_refuse_then_recover, "T2 Maw stand-in") }
#[test] fn leak_sticky_retire_fails_t2() { red_under(Leak::StickyRetire, t2_awakened_rebirth_phase_two_is_supported, "T2 Awakened phase 2") }
#[test] fn leak_ignore_truncation_fails_t2b() { red_under(Leak::IgnoreTruncation, t2b_truncation_pairs_agree, "T2b truncation") }

/// A public-equivalent pair whose pending (hidden) moves differ: world b's current roll replaced by `other`.
fn with_pending_move(run: &Run, index: usize, other: &'static str) -> Run {
    let mut b = run.clone();
    let e = &mut combat_mut(&mut b).state.enemies[index];
    let id = ActorIntentId::Full(FullIntentId::from_name(other));
    e.intent_id = id.clone(); e.selected_move = id.clone(); *e.move_history.last_mut().unwrap() = id;
    effects::refresh_enemy_values(&mut combat_mut(&mut b).state).unwrap();
    b
}
fn force_pending(run: &mut Run, index: usize, m: &'static str) { *run = with_pending_move(run, index, m) }
/// T2b (rule 2d): one visible hit that is a Nip in one world and a truncated multi-hit move in the other.
#[test]
fn t2b_truncation_pairs_agree() {
    // (encounter, enemy species, the one-hit move, the multi-hit move, player powers, enemy hp)
    let cases: [(EncounterId, EnemyId, &'static str, &'static str, &[(&'static str, i32)], i32); 3] = [
        (EncounterId::ThreeDarklings, EnemyId::Darkling, "darkling_nip", "darkling_chomp", &[("thorns", 30)], 5),
        (EncounterId::AwakenedOne, EnemyId::AwakenedOne, "awa_slash", "awa_soul", &[("thorns", 30), ("intangible_player", 99)], 20),
        (EncounterId::ThreeByrds, EnemyId::Byrd, "byrd_swoop", "byrd_peck", &[("thorns", 30), ("intangible_player", 99)], 3),
    ];
    for (encounter, species, one, many, powers, hp) in cases {
        let mut run = standalone(Character::Ironclad, 20, encounter, DEFENDS, dome(), 2);
        let i = combat(&run).state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(species)).unwrap();
        if species == EnemyId::Darkling { set_nip(&mut run, 9); }
        if species == EnemyId::Byrd { for _ in 0..1 { let a = end_turn_action(&run).unwrap(); run.step(&a).unwrap(); } }
        for (p, n) in powers { add_player_power(&mut run, p, *n); }
        for j in 0..combat(&run).state.enemies.len() { if combat(&run).state.enemies[j].enemy_id == ActorEnemyId::Full(species) { set_enemy_hp(&mut run, j, hp); } }
        force_pending(&mut run, i, one);
        let b = with_pending_move(&run, i, many);
        assert!(hidden(&run) != hidden(&b));
        let s = run_pair_with(run, |w| *w = b, passive, 12, &format!("{species:?} {one} vs truncated {many}"), false);
        assert!(s.decisions >= 2, "{species:?}: the pair diverged publicly: {s:?}");
        eprintln!("m40c truncation pair {species:?}: {s:?}");
    }
}

/// Fuzz mask 0x08 redraws hidden moves into a public-equivalent world: the root is unchanged.
#[test]
fn fuzz_hidden_move_perturbation_keeps_the_root() {
    let mut applied = 0;
    for encounter in [EncounterId::JawWorm, EncounterId::Chosen, EncounterId::WrithingMass, EncounterId::ThreeByrds, EncounterId::CorruptHeart] {
        for seed in 0..4u64 {
            let mut run = standalone(Character::Ironclad, 20, encounter, &strikes(5), dome(), seed);
            for _ in 0..3 {
                let mut b = run.clone();
                let bits = combat_mut(&mut b).fuzz_perturb_hidden(seed as u8 + 3);
                if bits & 8 != 0 {
                    assert!(hidden(&run) != hidden(&b));
                    assert!(root(&run).ok() == root(&b).ok(), "{encounter:?}: mask 0x08 changed the root");
                    applied += 1;
                }
                let Some(a) = attack(CardId::Strike)(&run) else { break };
                if run.step(&a).is_err() || run.decision().ok().and_then(|d| d.combat).is_none_or(|c| c.outcome.is_some()) { break }
            }
        }
    }
    assert!(applied > 10, "{applied}");
}

// ---------------------------------------------------------------- actors dead before their queued move (final review)

fn add_enemy_power(run: &mut Run, index: usize, id: &'static str, amount: i32) {
    combat_mut(run).state.enemies[index].full_powers.push(FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } });
}
/// A pair stepped to a decision where enemy `victim` has a different hidden pending roll in each world, then Poison
/// kills it at the start of the enemy turn, before its queued move (which the engine skips). The dead actor's roll was
/// never executed: records, roots and canonical worlds must stay equal, and its pending chain must be kept.
fn poisoned_before_move(encounter: EncounterId, victim: usize, warmup: usize, seeds: u64) -> usize {
    let mut pairs = 0;
    for seed in 0..seeds {
        let mut a = standalone(Character::Silent, 20, encounter, DEFENDS, vec![RelicId::RunicDome], seed);
        for _ in 0..warmup { let x = end_turn_action(&a).unwrap(); a.step(&x).unwrap(); }
        let mut b = a.clone();
        perturb_ai(&mut b);
        // One end turn: every enemy rolls its next (hidden) move, differently in the two worlds.
        for w in [&mut a, &mut b] { let x = end_turn_action(w).unwrap(); w.step(&x).unwrap(); }
        if view_of(&a) != view_of(&b) || victim >= combat(&a).state.enemies.len() { continue }
        if combat(&a).state.enemies[victim].intent_id == combat(&b).state.enemies[victim].intent_id { continue }
        for w in [&mut a, &mut b] { set_enemy_hp(w, victim, 2); add_enemy_power(w, victim, "poison", 9); }
        let had_pending = read_record(&a, victim).pending.is_some();
        let shown_before = read_record(&a, victim).shown;
        let (oa, ob) = (a.step_observed(&end_turn_action(&a).unwrap()), b.step_observed(&end_turn_action(&b).unwrap()));
        if oa.result.is_err() || ob.result.is_err() || a.decision().ok().and_then(|d| d.combat).is_none_or(|c| c.outcome.is_some()) { continue }
        // Public equivalence through the step: equal views and equal traces, executed moves included.
        let trace = |o: &Observed<Result<crate::run::RunTransition, crate::run::RunError>>| o.effects.iter().map(|e| format!("{:?}", e.kind)).collect::<Vec<_>>();
        if trace(&oa) != trace(&ob) || view_of(&a) != view_of(&b) { continue }
        let mover = |o: &Observed<Result<crate::run::RunTransition, crate::run::RunError>>| o.effects.iter().any(|e| matches!(&e.kind, ResolvedEffectKind::EnemyMove { actor: Actor::Enemy(h), .. } if *h == combat(&a).state.enemies[victim].instance_id));
        assert!(!mover(&oa) && !mover(&ob), "the poisoned actor did not move");
        assert!(combat(&a).state.enemies[victim].hp <= 0, "Poison killed it before its move");
        assert_eq!(read_record(&a, victim).pending.is_some(), had_pending, "an unexecuted roll keeps its pending chain");
        assert_eq!(read_record(&a, victim).shown, shown_before, "an unexecuted Bite is not shown");
        let mut stats = Stats::default();
        check_pair(&a, &b, &mut stats, &format!("{encounter:?} seed {seed}: dead before its move"));
        pairs += 1;
    }
    pairs
}
#[test]
fn t2_poisoned_before_their_move_keep_equal_roots() {
    assert!(poisoned_before_move(EncounterId::TwoLouse, 0, 1, 24) > 0, "louse");
    assert!(poisoned_before_move(EncounterId::BronzeAutomaton, 1, 1, 24) > 0, "bronze orb (C-MEM)");
}
/// Monitor-level pairs for solo encounters (a death would end the combat): Maw (same-shape), Writhing Mass after a
/// Reactive re-roll (chain), Book of Stabbing (C-MEM). The actor is dead when its `EnemyMove` dispatches.
#[test]
fn t2_dead_actor_move_dispatch_is_not_an_execution() {
    for (encounter, others) in [(EncounterId::Maw, &["maw_slam", "maw_nom", "maw_drool"][..]), (EncounterId::WrithingMass, &["writhing_big", "writhing_multi", "writhing_block"][..]), (EncounterId::BookOfStabbing, &["book_big", "book_multi"][..])] {
        let mut a = standalone(Character::Ironclad, 20, encounter, &strikes(5), dome(), 3);
        // Maw under Intangible: its executed slam or nom is genuinely unidentifiable (rule 2c), so the record carries an
        // ambiguous position that the dead dispatch must leave untouched.
        if encounter == EncounterId::Maw { add_player_power(&mut a, "intangible_player", 9); }
        for _ in 0..2 { let x = end_turn_action(&a).unwrap(); a.step(&x).unwrap(); }
        if encounter == EncounterId::WrithingMass { for _ in 0..2 { if let Some(x) = play(&a, CardId::Strike, Some(0)) { a.step(&x).unwrap(); } } }
        match encounter {
            EncounterId::Maw => assert!(!read_record(&a, 0).ambiguous.is_empty(), "Maw: the setup produced no ambiguous execution"),
            EncounterId::WrithingMass => assert!(read_record(&a, 0).pending.as_ref().is_some_and(|c| c.rolls.len() > 1), "Writhing Mass: the setup produced no re-roll chain"),
            _ => {}
        }
        let live = combat(&a).state.enemies[0].intent_id.as_str();
        let other = others.iter().find(|m| **m != live).unwrap_or_else(|| panic!("{encounter:?}: no other move"));
        let mut b = with_pending_move(&a, 0, other);
        assert!(hidden(&a) != hidden(&b));
        let marks_before = (read_record(&a, 0).ambiguous, read_record(&a, 0).inert, read_record(&a, 0).shown);
        for w in [&mut a, &mut b] {
            let handle = combat(w).state.enemies[0].instance_id.clone();
            let state = &mut combat_mut(w).state;
            state.enemies[0].hp = 0;
            let before = hidden_moves::snapshot(state).unwrap();
            hidden_moves::observe(state, &Effect { source: Source::None, target: Some(Actor::Enemy(handle)), kind: EffectKind::EnemyMove }, before);
            assert!(state.knowledge.moves.executing.is_none(), "{encounter:?}: no execution mark for a dead actor");
        }
        let (ra, rb) = (read_record(&a, 0), read_record(&b, 0));
        assert!(ra.pending.is_some() && (ra.ambiguous.clone(), ra.inert.clone(), ra.shown) == marks_before, "{encounter:?}: the dead dispatch changed the record: {ra:?}");
        assert!(ra == rb, "{encounter:?}: records differ");
        assert!(root(&a).unwrap() == root(&b).unwrap(), "{encounter:?}: the dead actor's hidden roll reached the root");
    }
}
/// A dead enemy without a record (a hand-built entry) is refused, not left uncanonicalised.
#[test]
fn a_dead_enemy_without_a_record_is_refused() {
    let mut run = standalone(Character::Ironclad, 20, EncounterId::TwoLouse, DEFENDS, dome(), 1);
    let handle = combat(&run).state.enemies[0].instance_id.clone();
    let state = &mut combat_mut(&mut run).state;
    state.enemies[0].hp = 0;
    state.knowledge.moves.enemies.remove(&handle);
    assert_eq!(root(&run).err(), Some(Unsupported::HiddenIntents));
}
#[test] fn leak_dead_move_executes_fails_t2() { red_under(Leak::DeadMoveExecutes, t2_dead_actor_move_dispatch_is_not_an_execution, "T2 dead actor (monitor)") }
#[test] fn leak_dead_move_executes_fails_t2_poison() { red_under(Leak::DeadMoveExecutes, t2_poisoned_before_their_move_keep_equal_roots, "T2 poisoned before its move") }
