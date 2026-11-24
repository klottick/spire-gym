//! M34s-b b2: the canonical public root and the belief sampler (design §4.6, privacy oracle §4.10 P2, P4–P7 and the
//! negative controls). Every perturbation test first proves the true worlds differ.
use super::super::belief::{Leak, Unsupported, LEAK};
use super::super::*;
use crate::content::{CardId, Character, EncounterId, RelicId};
use crate::run::{AuthoredRunStart, CardInstance, CreationProfile, Run, RunConfig, RunRng, StandardCombatStart};

fn deck(cards: &[CardId]) -> Vec<CardInstance> {
    cards.iter().enumerate().map(|(i, id)| CardInstance { id: *id, instance: i as u64 + 1, upgrades: 0, misc: 0 }).collect()
}
/// An entered `standard-combat-v1` Run (the combat scope) at its opening decision.
fn standalone(character: Character, ascension: u8, encounter: EncounterId, cards: &[CardId], relics: Vec<RelicId>, seed: u8) -> Run {
    let start = StandardCombatStart { character, ascension, encounter, burning: None, bottled_cards: vec![], position: None,
        resources: AuthoredRunStart { hp: 70, max_hp: 80, gold: 50, deck: deck(cards), relics, potions: vec![None; if ascension >= 11 { 2 } else { 3 }], relic_counters: Default::default() } };
    let mut run = Run::prepare_standard_combat(start, RunRng::new([seed; 32], rng::Partition::Test, 0)).unwrap();
    run.enter_standard_combat().unwrap();
    run
}
const MIXED: &[CardId] = &[CardId::Strike, CardId::Strike, CardId::Strike, CardId::Defend, CardId::Defend, CardId::Bash, CardId::Anger, CardId::Armaments, CardId::Clothesline, CardId::Cleave];
/// A Standard Run walked into its first combat, then `steps` combat actions further (first legal candidate).
fn standard_run(seed: u8, steps: usize) -> Run {
    let mut run = Run::new_room_scenario(RunConfig { character: Character::Silent, ascension: 0 },
        CreationProfile { neow_eligible: false, ..CreationProfile::experienced() }, RunRng::new([seed; 32], rng::Partition::Test, 0), crate::run::rooms::RoomScenario::Standard).unwrap();
    for _ in 0..200 { if run.state().combat.is_some() { break } let a = run.decision().unwrap().candidates[0].action.clone(); run.step(&a).unwrap(); }
    for _ in 0..steps {
        let mut next = run.clone();
        let a = next.decision().unwrap().candidates[0].action.clone();
        next.step(&a).unwrap();
        if next.decision().unwrap().combat.as_ref().is_none_or(|c| c.outcome.is_some()) { break }
        run = next;
    }
    assert!(run.state().combat.is_some());
    run
}
fn combat(run: &Run) -> &Combat { run.state().combat.as_ref().unwrap() }
fn combat_mut(run: &mut Run) -> &mut Combat { run.state_mut_for_test().combat.as_mut().unwrap() }
fn root(run: &Run) -> Run { run.public_root_run().unwrap() }
fn root_with(run: &Run, l: Leak) -> Result<Run, Unsupported> {
    LEAK.with(|v| v.set(Some(l)));
    let r = run.public_root_run();
    LEAK.with(|v| v.set(None));
    r
}
fn sample(root: &Run, k: u64) -> Run { root.belief_sample(&crate::run::research_sample_key(&crate::run::research_root(11), &[1; 32], k)).unwrap() }

// ---------- hidden perturbations (each returns None when it cannot change this world) ----------
fn perturb_run_rng(run: &mut Run) -> Option<()> { run.state_mut_for_test().rng = RunRng::new([0x5A; 32], rng::Partition::Final, 9); Some(()) }
fn perturb_streams(run: &mut Run) -> Option<()> {
    let state = &mut combat_mut(run).state;
    let bump = |s: &rng::StreamState| rng::StreamState::new(s.id().clone(), [0x77; 32], s.word_index() + 3).unwrap();
    state.rng.shuffle = bump(&state.rng.shuffle);
    state.rng.enemy_ai = bump(&state.rng.enemy_ai);
    state.card_rng = state.card_rng.as_ref().map(bump);
    for e in &mut state.enemies { e.full_ai_rng = e.full_ai_rng.as_ref().map(bump); }
    Some(())
}
/// Another arrangement of the unknown draw cards that the public knowledge supports (a swap of two unknown cards).
fn perturb_draw_order(run: &mut Run) -> Option<()> {
    let c = combat_mut(run);
    if view::public_rules(&c.state).reveal_draw_order { return None }
    let k = c.draw_knowledge().ok()?;
    let len = c.state.piles.draw.len();
    let known = c.state.public.known_draw.len()..len - c.state.public.known_draw_bottom.len();
    for i in known.clone() { for j in known.clone() {
        if i >= j { continue }
        let (a, b) = (len - 1 - i, len - 1 - j);
        let mut trial = c.state.piles.draw.clone();
        trial.swap(a, b);
        let seq = trial.iter().rev().map(|card| knowledge::class_of(&c.state, card).unwrap()).collect::<Vec<_>>();
        let before = c.state.piles.draw.iter().rev().map(|card| knowledge::class_of(&c.state, card).unwrap()).collect::<Vec<_>>();
        if seq != before && k.supports(&seq) { c.state.piles.draw = trial; knowledge::rebind_draw(&mut c.state, Some(Ok(k))); return Some(()) }
    } }
    None
}
fn perturb_preroll(run: &mut Run) -> Option<()> {
    let c = combat_mut(run);
    let p = knowledge::preroll_candidates(&c.state).ok()?.into_iter().find(|p| p.values.len() > 1)?;
    let base = c.state.enemies.iter().find(|e| e.instance_id == p.enemy)?.full_damage_entries[&p.intent].base;
    let other = *p.values.iter().find(|v| **v != base)?;
    knowledge::set_preroll(&mut c.state, &p, other);
    effects::refresh_enemy_values(&mut c.state).unwrap();
    Some(())
}
type Perturb = fn(&mut Run) -> Option<()>;
const PERTURBATIONS: [(&str, Perturb); 4] = [("run rng", perturb_run_rng), ("streams", perturb_streams), ("draw order", perturb_draw_order), ("pre-roll", perturb_preroll)];

/// Louse roots whose Bite base is still ambiguous (the louse has not shown Bite).
fn louse_root(ascension: u8) -> Run { ambiguous_root(ascension, EncounterId::TwoLouse) }
fn ambiguous_root(ascension: u8, encounter: EncounterId) -> Run {
    for seed in 0..=255u8 {
        let run = standalone(Character::Ironclad, ascension, encounter, MIXED, vec![RelicId::BurningBlood], seed);
        if knowledge::preroll_candidates(&combat(&run).state).unwrap().iter().any(|p| p.values.len() > 1) { return run }
    }
    panic!("no {encounter:?} seed with an ambiguous pre-roll")
}
/// A selection decision: Armaments' hand selection.
fn selection_root() -> Run {
    for seed in 0..=255u8 {
        let mut run = standalone(Character::Ironclad, 0, EncounterId::JawWorm, &[CardId::Armaments, CardId::Strike, CardId::Strike, CardId::Defend, CardId::Bash, CardId::Strike, CardId::Defend], vec![], seed);
        let d = run.decision().unwrap();
        let Some(play) = d.candidates.iter().find(|c| matches!(&c.action, crate::run::GameAction::Combat(CombatAction::PlayCard { instance, .. })
            if combat(&run).state.piles.hand.iter().any(|h| h.full().unwrap().instance.0 == *instance && h.full().unwrap().id == CardId::Armaments))) else { continue };
        let action = play.action.clone();
        run.step(&action).unwrap();
        if combat(&run).state.phase == Phase::Selection && combat(&run).state.piles.draw.len() > 1 { return run }
    }
    panic!("no Armaments selection")
}
fn roots() -> Vec<(&'static str, Run)> {
    vec![("standard run opening", standard_run(3, 0)), ("standard run later", standard_run(4, 6)),
        ("standalone", standalone(Character::Ironclad, 20, EncounterId::JawWorm, MIXED, vec![RelicId::BurningBlood], 9)),
        ("louse", louse_root(0)), ("louse A20", louse_root(20)), ("darklings", ambiguous_root(0, EncounterId::ThreeDarklings)), ("selection", selection_root())]
}

#[test]
fn canonical_root_is_invariant_under_hidden_perturbation() {
    let mut applied = std::collections::BTreeMap::new();
    for (name, run) in roots() {
        let base = root(&run);
        for (what, perturb) in PERTURBATIONS {
            let mut other = run.clone();
            if perturb(&mut other).is_none() { continue }
            assert!(other != run, "{name}/{what}: the true worlds differ");
            assert!(root(&other) == base, "{name}/{what}: the root is invariant");
            *applied.entry(what).or_insert(0) += 1;
        }
    }
    for (what, _) in PERTURBATIONS { assert!(applied.get(what).copied().unwrap_or(0) >= 2, "{what} applied: {applied:?}"); }
}

#[test]
fn selection_root_payload_perturbation_is_equal_or_unsupported() {
    let run = selection_root();
    assert_eq!(combat(&run).state.phase, Phase::Selection);
    let base = root(&run);
    for (what, perturb) in PERTURBATIONS {
        let mut other = run.clone();
        if perturb(&mut other).is_some() { assert!(root(&other) == base, "{what}"); }
    }
    // A disallowed payload queued behind the selection gives Unsupported.
    let mut other = run.clone();
    let enemy = combat(&run).state.enemies[0].instance_id.clone();
    combat_mut(&mut other).state.pending.push_back(Effect { source: Source::None, target: None,
        kind: EffectKind::Ironclad(cards::IroncladEffect::HavocTop { target: Some(enemy) }) });
    assert!(matches!(other.public_root_run(), Err(Unsupported::DisallowedContinuation(_))));
}

#[test]
fn canonical_root_keeps_every_public_leaf() {
    for (name, run) in roots() {
        let r = root(&run);
        let (a, b) = (run.decision().unwrap(), r.decision().unwrap());
        assert_eq!((a.combat, a.player, a.objects), (b.combat, b.player, b.objects), "{name}: public view kept");
        assert_eq!(a.candidates.iter().map(|c| &c.action).collect::<Vec<_>>(), b.candidates.iter().map(|c| &c.action).collect::<Vec<_>>(), "{name}");
        assert_eq!(combat(&run).draw_knowledge(), combat(&r).draw_knowledge(), "{name}: knowledge kept");
        assert_eq!(combat(&run).preroll_sets(), combat(&r).preroll_sets(), "{name}: pre-roll sets kept");
        assert!(combat(&r).state.rng.shuffle.word_index() == 0, "{name}: cursors zero");
    }
}

#[test]
fn sample_is_a_function_of_root_seed_and_index() {
    for (name, run) in roots() {
        let r = root(&run);
        assert!(sample(&r, 0) == sample(&r, 0), "{name}: deterministic");
        assert!((1..8).any(|k| sample(&r, k) != sample(&r, 0)), "{name}: indices differ");
        // R1 depends on the key: the draw order varies across samples wherever more than one order is possible.
        if combat(&r).draw_knowledge().unwrap().components().iter().any(|c| c.segments.iter().any(|s| matches!(s, knowledge::Segment::Bag(e) if e.len() > 1))) {
            let order = |k| combat(&sample(&r, k)).state.piles.draw.iter().map(|c| c.handle()).collect::<Vec<_>>();
            assert!((1..16).any(|k| order(k) != order(0)), "{name}: R1 varies with the key");
        }
        // Public state is kept by every sample.
        let s = sample(&r, 5);
        assert_eq!(r.decision().unwrap().combat.map(|c| c.hand), s.decision().unwrap().combat.map(|c| c.hand), "{name}");
        assert_eq!(combat(&r).draw_knowledge(), combat(&s).draw_knowledge(), "{name}");
        assert!(s.public_root_run().unwrap() == r, "{name}: a sample's root is its root");
    }
}

#[test]
fn sample_is_not_concentrated_on_the_true_next_card() {
    let run = standalone(Character::Ironclad, 0, EncounterId::JawWorm, MIXED, vec![], 21);
    let truth = combat(&run).state.piles.draw.last().unwrap().handle();
    let r = root(&run);
    let hits = (0..200).filter(|k| combat(&sample(&r, *k)).state.piles.draw.last().unwrap().handle() == truth).count();
    // Five unknown distinct permanent cards: the posterior on the top is 1/5.
    assert!(combat(&run).state.piles.draw.len() == 5 && (20..=70).contains(&hits), "hits {hits}");
}

#[test]
fn pre_rolls_are_uniform_on_the_consistent_set() {
    let run = louse_root(0);
    let r = root(&run);
    let p = knowledge::preroll_candidates(&combat(&r).state).unwrap().into_iter().find(|p| p.values.len() > 1).unwrap();
    let mut counts = std::collections::BTreeMap::new();
    for k in 0..300 {
        let s = sample(&r, k);
        let base = combat(&s).state.enemies.iter().find(|e| e.instance_id == p.enemy).unwrap().full_damage_entries[&p.intent].base;
        assert!(p.values.contains(&base));
        *counts.entry(base).or_insert(0) += 1;
    }
    let expected = 300 / p.values.len();
    assert_eq!(counts.len(), p.values.len());
    assert!(counts.values().all(|n| n * 2 > expected && *n < expected * 2), "{counts:?}");
}

/// P4: the sample's streams are the research derivation and never the root's or the true world's keys.
#[test]
fn sample_keys_follow_the_research_derivation_and_differ_from_truth() {
    let run = louse_root(20);
    let r = root(&run);
    let key = crate::run::research_sample_key(&crate::run::research_root(11), &[1; 32], 3);
    let s = r.belief_sample(&key).unwrap();
    let research = RunRng::new(rng::hmac_sha256(&key, b"r2-run-root"), rng::Partition::Test, 0);
    let active = s.state().active_combat.clone().unwrap();
    let expected = research.derive_combat_stream(rng::Domain::RunCombatShuffle, s.state().act, active.encounter.as_str(), active.visit, None, "shuffle").unwrap();
    assert_eq!(combat(&s).state.rng.shuffle, expected);
    for (truth, world) in [(&combat(&run).state, &combat(&s).state), (&combat(&r).state, &combat(&s).state)] {
        assert_ne!(truth.rng.shuffle.key(), world.rng.shuffle.key());
        assert_ne!(truth.card_rng.as_ref().map(|c| *c.key()), world.card_rng.as_ref().map(|c| *c.key()));
        for (a, b) in truth.enemies.iter().zip(&world.enemies) { assert_ne!(a.full_ai_rng.as_ref().map(|x| *x.key()), b.full_ai_rng.as_ref().map(|x| *x.key())); }
    }
    assert_ne!(s.state().rng.root_for_test(), run.state().rng.root_for_test());
}

#[test]
fn hidden_intent_root_is_unsupported() {
    let run = standalone(Character::Ironclad, 0, EncounterId::JawWorm, MIXED, vec![RelicId::RunicDome], 1);
    assert_eq!(run.public_root_run().err(), Some(Unsupported::HiddenIntents));
}

#[test]
fn disallowed_continuation_root_is_unsupported() {
    let run = standalone(Character::Ironclad, 0, EncounterId::TwoLouse, MIXED, vec![], 2);
    let enemy = combat(&run).state.enemies[0].instance_id.clone();
    let child = combat(&run).state.piles.hand[0].handle();
    let denied: Vec<EffectKind> = vec![
        EffectKind::Ironclad(cards::IroncladEffect::HavocTop { target: Some(enemy.clone()) }),
        EffectKind::Ironclad(cards::IroncladEffect::HavocExtract { target: Some(enemy.clone()) }),
        EffectKind::Ironclad(cards::IroncladEffect::HavocQueued { child: child.clone(), target: Some(enemy.clone()), energy_on_use: 0, cancelled: false }),
        EffectKind::Colorless(cards::ColorlessEffect::MayhemPlayTopCard { target: enemy.clone() }),
        EffectKind::Colorless(cards::ColorlessEffect::MayhemAutoplay { child: child.clone(), target: Some(Actor::Enemy(enemy.clone())), energy_on_use: 0, cancelled: false }),
        EffectKind::Potion(PotionEffect::DistilledChaosTopCard { target: Some(enemy.clone()), energy_on_use: 0 }),
        EffectKind::Potion(PotionEffect::DistilledChaosQueuedChild { child: child.clone(), target: Some(enemy.clone()), energy_on_use: 0, cancelled: false }),
        EffectKind::Silent(cards::SilentEffect::FlaskBounce { target: Some(enemy.clone()), remaining: 2 }),
        EffectKind::Defect(cards::DefectEffect::Direct(cards::defect::DirectEffect::Chaos {
            requested: 1, orbs: vec![characters::OrbKind::Lightning],
        })),
        EffectKind::Alchemize { proposal: Some(crate::content::PotionId::FirePotion) },
    ];
    for kind in denied {
        let mut other = run.clone();
        combat_mut(&mut other).state.pending.push_back(Effect { source: Source::None, target: None, kind: kind.clone() });
        let error = other.public_root_run().err();
        assert!(matches!(error, Some(Unsupported::DisallowedContinuation(_))), "{kind:?}");
        assert!(!error.unwrap().reason().chars().any(|c| c.is_ascii_digit()), "P7: the reason names a class");
    }
    let mut other = run.clone();
    combat_mut(&mut other).state.replacement_progress = Some(enemies::ReplacementProgress { parent: enemy.clone(), children: vec![], inherited_hp: 1,
        next_child: 0, prepared: vec![], latest_child_rng: None });
    assert!(matches!(other.public_root_run(), Err(Unsupported::DisallowedContinuation(_))));
    // Fail-closed rows (verify-v1 note 3): captured, unoffered generated cards (an offered generation is public and allowed),
    // and enemy spawn work in progress.
    let work = |offer| generation::GenerationWork { pool: generation::GenerationPool::Attack, count: 1, unique: false,
        destination: command::CardDestination::Hand, cost: generation::GenerationCost::Ordinary, upgraded: false, offer, captured: true,
        cards: vec![CardId::Strike], next: 0 };
    let mut generated = run.clone();
    combat_mut(&mut generated).state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Generate(work(None)) });
    assert!(matches!(generated.public_root_run(), Err(Unsupported::DisallowedContinuation(_))));
    let mut offered = run.clone();
    combat_mut(&mut offered).state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Generate(work(Some((false, 1)))) });
    assert!(offered.public_root_run().is_ok(), "an offered generation is public");
    let mut spawning = run.clone();
    let stream = combat(&run).state.rng.shuffle.clone();
    combat_mut(&mut spawning).state.spawn_progress = Some(enemies::act2::spawn::SpawnProgress { request: SpawnRequest { summoner: enemy.clone(), slot: 2,
        species: crate::content::EnemyId::Dagger }, ordinal: 1, prepared_child: None, rolled_hp: None, constructor_rng: stream.clone(), latest_child_rng: stream });
    assert!(matches!(spawning.public_root_run(), Err(Unsupported::DisallowedContinuation(_))));
    // A nested child play holding an enemy target (the autoplay paths draw it at random).
    let mut framed = run.clone();
    let child_card = combat(&run).state.piles.hand[0].clone();
    combat_mut(&mut framed).state.play_frames.push(model::PlayFrame { parent_owner: model::PlayFrameParentOwner::Card(child_card.handle()),
        child: child_card.handle(), target: Some(Actor::Enemy(enemy.clone())), resume_phase: Phase::Decision, captured_energy: 0, paid: 0,
        disposition: model::FinishDisposition::Natural });
    assert!(matches!(framed.public_root_run(), Err(Unsupported::DisallowedContinuation(_))));
    // A thief's rolled branch held before its move roll.
    let thieves = standalone(Character::Ironclad, 0, EncounterId::Looter, MIXED, vec![], 2);
    let mut held = thieves.clone();
    let Some(enemies::EnemyMemory::Thief(memory)) = combat_mut(&mut held).state.enemies[0].full_memory.as_mut() else { panic!("thief memory") };
    memory.pending_next = Some(FullIntentId::from_name("smoke_bomb"));
    assert!(thieves.public_root_run().is_ok());
    assert!(matches!(held.public_root_run(), Err(Unsupported::DisallowedContinuation(_))));
}

#[test]
fn unrepresentable_knowledge_root_is_unsupported() {
    let mut run = standalone(Character::Ironclad, 0, EncounterId::JawWorm, MIXED, vec![], 3);
    combat_mut(&mut run).state.knowledge.draw = Err(knowledge::Unrepresentable);
    assert_eq!(run.public_root_run().err(), Some(Unsupported::UnrepresentableKnowledge));
}

#[test]
fn knowledge_over_the_byte_cap_makes_the_root_unsupported() {
    // Final review I2: the review's ~14 MB knowledge (40 known Wounds on top, 3 Dazed shuffled in) is over
    // `KNOWLEDGE_BYTES_CAP`, so the root is Unsupported instead of every branch retaining it.
    let mut run = standalone(Character::Ironclad, 0, EncounterId::JawWorm, MIXED, vec![], 3);
    let create = |run: &mut Run, id: CardId, destination: CardDestination| piles::apply_effect(&mut combat_mut(run).state, &Effect {
        source: Source::Actor(Actor::Player), target: Some(Actor::Player), kind: EffectKind::CreateCard { id, destination, upgrades: 0 } }).unwrap();
    for _ in 0..40 { create(&mut run, CardId::Wound, CardDestination::Draw); }
    create(&mut run, CardId::Dazed, CardDestination::ShuffledDraw);
    let bytes = combat_mut(&mut run).draw_knowledge().unwrap().heap_bytes();
    assert!(bytes <= knowledge::KNOWLEDGE_BYTES_CAP && run.public_root_run().is_ok(), "{bytes}");
    for _ in 0..2 { create(&mut run, CardId::Dazed, CardDestination::ShuffledDraw); }
    assert_eq!(combat_mut(&mut run).draw_knowledge(), Err(knowledge::Unrepresentable));
    assert_eq!(run.public_root_run().err(), Some(Unsupported::UnrepresentableKnowledge));
}

#[test]
fn fault_or_terminal_root_is_unsupported() {
    let run = standalone(Character::Ironclad, 0, EncounterId::JawWorm, MIXED, vec![], 4);
    let mut faulted = run.clone();
    combat_mut(&mut faulted).state.fault = Some(FaultRecord { kind: FaultKind::Invariant, message: "test".into(), resolved_effects: 0 });
    assert_eq!(faulted.public_root_run().err(), Some(Unsupported::Faulted));
    let mut ended = run.clone();
    combat_mut(&mut ended).state.outcome = Some(Outcome::Victory);
    assert_eq!(ended.public_root_run().err(), Some(Unsupported::Terminal));
    // Not in combat; captured profiles maintain no public knowledge.
    let idle = Run::new(RunConfig { character: Character::Ironclad, ascension: 0 }, CreationProfile::experienced(), RunRng::new([1; 32], rng::Partition::Test, 0)).unwrap();
    assert_eq!(idle.public_root_run().err(), Some(Unsupported::NotAtDecision));
    let mut gym66 = run.clone();
    combat_mut(&mut gym66).state.knowledge.maintained = false;
    assert_eq!(gym66.public_root_run().err(), Some(Unsupported::NoPublicKnowledge));
}

/// P7: an Unsupported reason names a class, never a value (no digit, no handle, no key).
#[test]
fn unsupported_reasons_name_no_values() {
    let all = [Unsupported::NoPublicKnowledge, Unsupported::Terminal, Unsupported::Faulted, Unsupported::NotAtDecision, Unsupported::HiddenIntents,
        Unsupported::DisallowedContinuation("x"), Unsupported::UnrepresentableKnowledge, Unsupported::UnrepresentablePreroll];
    for u in all { assert!(!u.reason().chars().any(|c| c.is_ascii_digit()), "{u:?}"); assert!(!format!("{u:?}").chars().any(|c| c.is_ascii_digit())); }
}

// ---------- negative controls (expected failures of the privacy tests) ----------
fn p2_fails_with(l: Leak, what: &str) {
    let mut failed = 0;
    for (_, run) in roots() {
        for (name, perturb) in PERTURBATIONS {
            if name != what { continue }
            let mut other = run.clone();
            if perturb(&mut other).is_none() { continue }
            if root_with(&run, l).ok() != root_with(&other, l).ok() { failed += 1; }
        }
    }
    assert!(failed > 0, "the leaky canonicaliser {l:?} passed P2 on {what}");
}
#[test]
fn leaky_root_without_rng_zeroing_fails_p2() { p2_fails_with(Leak::KeepStreams, "streams"); }
#[test]
fn leaky_root_keeping_draw_order_fails_p2_and_p3() {
    p2_fails_with(Leak::KeepDrawOrder, "draw order");
    // P3: under the leak a sample keeps the true order, so equal-root worlds that differ in order give different branches.
    let run = standalone(Character::Ironclad, 0, EncounterId::JawWorm, MIXED, vec![], 21);
    let mut other = run.clone();
    perturb_draw_order(&mut other).unwrap();
    LEAK.with(|v| v.set(Some(Leak::KeepDrawOrder)));
    let key = crate::run::research_sample_key(&crate::run::research_root(1), &[0; 32], 0);
    let (a, b) = (run.public_root_run().unwrap().belief_sample(&key).unwrap(), other.public_root_run().unwrap().belief_sample(&key).unwrap());
    LEAK.with(|v| v.set(None));
    assert!(a != b, "the leak shows in the branch");
}
#[test]
fn leaky_root_keeping_an_ambiguous_pre_roll_fails_p2() { p2_fails_with(Leak::KeepPreroll, "pre-roll"); }
#[test]
fn leaky_root_copying_a_captured_target_fails_p2() {
    let run = standalone(Character::Ironclad, 0, EncounterId::TwoLouse, MIXED, vec![], 2);
    let targets = combat(&run).state.enemies.iter().map(|e| e.instance_id.clone()).collect::<Vec<_>>();
    let world = |t: &EnemyHandle| { let mut w = run.clone(); combat_mut(&mut w).state.pending.push_back(Effect { source: Source::None, target: None,
        kind: EffectKind::Ironclad(cards::IroncladEffect::HavocTop { target: Some(t.clone()) }) }); w };
    let (a, b) = (world(&targets[0]), world(&targets[1]));
    assert!(a.public_root_run().is_err() && b.public_root_run().is_err(), "the allow-list denies both");
    assert!(root_with(&a, Leak::AllowAll).unwrap() != root_with(&b, Leak::AllowAll).unwrap(), "without it the captured target leaks");
}

/// P6 (design §4.10): every field of the combat state and every enemy-memory variant is classified here, by exhaustive
/// destructuring with no rest pattern, so a new field or variant does not compile until it is classified. Classes (b0
/// census lane 1): P public or public history, PD public-derived, H-pre hidden pre-roll (R3), H-fut randomness
/// material (R2), D the draw order (R1), C a continuation (allow-list), B bookkeeping that is a function of public history.
#[test]
fn census_classifies_every_field_and_variant() {
    let run = standalone(Character::Ironclad, 0, EncounterId::JawWorm, MIXED, vec![], 5);
    let model::State { content_id: _ /*P*/, phase: _ /*P*/, turn: _ /*P*/, player, enemies, piles: _ /*P, draw order D*/,
        card_ids_issued: _ /*B*/, rng: _ /*H-fut*/, pending: _ /*C*/, current: _ /*C*/, in_play: _ /*P*/, resolved_effects: _ /*B*/,
        public: _ /*P*/, outcome: _ /*P*/, fault: _ /*C: Unsupported*/, hp_before_healing: _ /*P*/, hp_after_cleanup: _ /*P*/,
        full_runtime: _ /*P*/, ascension: _ /*P*/, pending_selection: _ /*P*/, temporary_retain: _ /*P*/, nightmare_templates: _ /*P*/,
        next_nightmare_installation_id: _ /*B*/, hook_events: _ /*B*/, attempted_action: _ /*C: Unsupported with a fault*/,
        fault_selection: _ /*C: Unsupported with a fault*/, player_full_powers: _ /*P*/, character, card_rng: _ /*H-fut*/,
        item_context, replacement_progress: _ /*C: denied when Some*/, actor_slots: _ /*P*/,
        facing: _ /*P*/, spawn_ordinals: _ /*B*/, spawn_progress: _ /*C: denied when Some*/, stasis: _ /*P*/, permanent_gains: _ /*P*/,
        next_permanent_gain: _ /*B*/, played_powers: _ /*P*/, played_card_types: _ /*P*/, inactive_cards: _ /*P*/,
        play_frames: _ /*C: enemy target denied*/, play_context: _ /*C: with the frames*/, queued_autoplay: _ /*P*/,
        pending_time_warp: _ /*P*/, extra_turn_pending: _ /*P*/, damage_events_this_combat: _ /*P*/, silent, combust: _ /*P*/,
        thief_gold: _ /*P*/, colorless, defect, watcher, public_target_plays: _ /*B: a function of public play history*/, knowledge: _ /*P: public knowledge; its physical bookkeeping is resynchronised*/,
    } = &combat(&run).state;
    let model::PlayerState { hp: _ /*P*/, max_hp: _ /*P*/, block: _ /*P*/, energy: _ /*P*/, max_energy: _ /*P*/,
        draw_per_turn: _ /*P*/, powers: _ /*P*/ } = player;
    let characters::CharacterState { stance: _ /*P*/, mantra_gained: _ /*P*/, orb_slots: _ /*P*/, focus: _ /*P*/ } = character;
    let cards::SilentState { history_enabled: _ /*P*/, smoked: _ /*P*/, source_character: _ /*P*/,
        retain_block_next_start: _ /*P*/, played_this_turn: _ /*P*/, discarded_this_turn: _ /*P*/,
        drawn_cards: _ /*P*/, powers_played_this_combat: _ /*P*/, card_offer: _ /*P*/ } = silent;
    let model::ColorlessState { panache_remaining: _ /*P*/, bombs: _ /*P*/, next_bomb_id: _ /*B*/ } = colorless;
    let cards::DefectState { echo_doubled_this_turn: _ /*P*/, frost_channeled: _ /*P*/, lightning_channeled: _ /*P*/,
        rebound_just_applied: _ /*P*/, permanent_mutations: _ /*P*/ } = defect;
    let cards::watcher::WatcherState { stance_choice: _ /*P*/, resolving_stance_choice: _ /*P*/,
        permanent_upgrades: _ /*P*/, deva_energy_gain: _ /*P*/ } = watcher;
    assert!(item_context.is_some(), "the native fixture has its item context");
    for enemy in enemies {
        let model::EnemyState { instance_id: _ /*B*/, enemy_id: _ /*P*/, hp: _ /*P*/, max_hp: _ /*P*/, block: _ /*P*/, powers: _ /*P*/,
            full_powers: _ /*P*/, intent_id: _ /*P*/, move_history: _ /*P*/, selected_move: _ /*P*/, damage_entries: _ /*PD*/,
            full_damage_entries: _ /*PD; Louse Bite and Darkling Nip bases H-pre*/, full_ai_rng: _ /*H-fut*/, full_memory,
            intent_damage: _ /*PD*/, life: _ /*P*/ } = enemy;
        use enemies::EnemyMemory as M;
        if let Some(memory) = full_memory { match memory {
            M::Darkling { .. } => {} // nip: H-pre (R3, both stores); center, first: P
            M::Thief(_) => {} // pending_next: H-pre until its move roll (denied when Some); the rest P
            M::Cultist | M::JawWorm{..} | M::GremlinNob | M::Lagavulin{..} | M::Sentry{..} | M::Guardian{..} | M::Hexaghost{..} | M::SphericGuardian
            | M::Byrd{..} | M::Chosen | M::Snecko | M::Centurion | M::Mystic | M::GremlinLeader | M::GremlinMad | M::GremlinSneaky | M::GremlinFat
            | M::GremlinShield | M::GremlinWizard{..} | M::ShelledParasite{..} | M::SnakePlant | M::SlaverBlue | M::FungiBeast | M::Taskmaster
            | M::SlaverRed{..} | M::BanditPointy | M::BanditLeader | M::BanditBear | M::BookOfStabbing{..} | M::Champ{..} | M::Collector{..}
            | M::TorchHead | M::BronzeAutomaton{..} | M::BronzeOrb{..} | M::OrbWalker | M::Repulsor | M::Exploder{..} | M::Spiker{..}
            | M::SpireGrowth | M::Transient{..} | M::Maw{..} | M::WrithingMass{..} | M::GiantHead{..} | M::Nemesis{..} | M::Reptomancer{..}
            | M::Dagger{..} | M::AwakenedOne{..} | M::TimeEater{..} | M::Deca{..} | M::Donu{..} | M::SpireShield{..} | M::SpireSpear{..}
            | M::LouseRed{..} | M::LouseGreen{..} | M::AcidSlime{..} | M::SpikeSlime{..} | M::SlimeBoss{..} | M::CorruptHeart{..} => {} // P: public history
        } }
    }
    for card in combat(&run).state.piles.hand.iter().filter_map(|c| c.full()) {
        let card_state::LiveCard { instance: _ /*B: public-history allocation*/, id: _, origin: _, upgrades: _, misc: _, local, copy_family: _ /*B*/, copy_provenance: _ /*B*/ } = card;
        let card_state::CombatCardData { m21: _, self_retain: _, retain: _, base_damage: _, base_block: _, magic_number: _, temporary_upgrades: _,
            randomized_cost: _, mutable_combat_cost: _, turn_cost: _, combat_cost_reduction: _, temporary_cost: _, infernal_blade_cost_this_turn: _,
            free_to_play_once: _, free_play: _, x_cost: _, exhaust_on_finish: _, ethereal: _, innate: _, purge: _, rebound_on_finish: _ } = local; // P/PD
    }
    let model::PublicKnowledge { revealed_order: _, known_draw: _, known_draw_bottom: _ } = &combat(&run).state.public; // P
}

// ---------- M40a: held nested targets (spire-61l stage 1) ----------
/// The first legal action matching `want`.
fn act(run: &mut Run, what: &str, want: impl Fn(&crate::run::GameAction) -> bool) {
    let a = run.decision().unwrap().candidates.iter().map(|c| c.action.clone()).find(|a| want(a)).unwrap_or_else(|| panic!("no legal action: {what}"));
    run.step(&a).unwrap_or_else(|e| panic!("{what}: {a:?}: {e:?}"));
}
fn play_card(run: &mut Run, what: &str, instance: u64) {
    act(run, what, |a| matches!(a, crate::run::GameAction::Combat(CombatAction::PlayCard { instance: i, .. }) if *i == instance));
}
fn hand_holds(run: &Run, instances: &[u64]) -> bool {
    let hand = &combat(run).state.piles.hand;
    instances.iter().all(|i| hand.iter().any(|c| c.full().is_some_and(|f| f.instance.0 == *i)))
}
/// Put `count` copies of `id` (fresh instances from 900) into the discard pile, as combat history would.
fn push_discard(run: &mut Run, id: CardId, count: u64) {
    let state = &mut combat_mut(run).state;
    for k in 0..count {
        let instance = 900 + k;
        state.card_ids_issued.insert(card_state::handle(CardInstanceId(instance)));
        state.piles.discard.push(model::OwnedCard::Full(card_state::permanent(id, CardInstanceId(instance), 0)));
    }
}
/// A standalone combat whose opening hand holds `instances` (deck instances are 1-based positions of `cards`).
fn hand_with(character: Character, encounter: EncounterId, cards: &[CardId], relics: Vec<RelicId>, instances: &[u64]) -> Run {
    for seed in 0..=255u8 {
        let run = standalone(character, 20, encounter, cards, relics.clone(), seed);
        if hand_holds(&run, instances) { return run }
    }
    panic!("no seed holds {instances:?} in the opening hand")
}
fn dt_deck() -> Vec<CardId> {
    let mut d = vec![CardId::DoubleTap, CardId::Headbutt];
    d.extend([CardId::Strike; 8]);
    d
}
/// Double Tap active, Headbutt played at an enemy (first selection), one discard card chosen: the repeat's own selection is
/// open inside a play frame that holds the player's enemy target.
fn double_tap_headbutt_repeat_selection() -> Run {
    let mut run = hand_with(Character::Ironclad, EncounterId::JawWorm, &dt_deck(), vec![], &[1, 2]);
    push_discard(&mut run, CardId::Strike, 2);
    play_card(&mut run, "Double Tap", 1);
    play_card(&mut run, "Headbutt", 2);
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the original Headbutt's selection");
    assert!(combat(&run).state.play_frames.is_empty());
    act(&mut run, "first pick", |a| matches!(a, crate::run::GameAction::Combat(CombatAction::SelectCard { .. })));
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the repeat's selection");
    assert_eq!(combat(&run).state.play_frames.len(), 1);
    assert!(matches!(combat(&run).state.play_frames[0].target, Some(Actor::Enemy(_))), "the frame holds the player's enemy target");
    run
}
#[test]
fn m40a_double_tap_repeat_frame_with_the_players_target_is_a_public_root() {
    let run = double_tap_headbutt_repeat_selection();
    let base = root(&run);
    for (what, perturb) in PERTURBATIONS {
        let mut other = run.clone();
        if perturb(&mut other).is_none() { continue }
        assert!(other != run, "{what}: the true worlds differ");
        assert!(root(&other) == base, "{what}: the root is invariant");
    }
    let s = sample(&base, 3);
    assert!(s.public_root_run().unwrap() == base, "a sample's root is its root");
}
fn standalone_with(character: Character, encounter: EncounterId, cards: &[CardId], potions: Vec<Option<crate::content::PotionId>>, seed: u8) -> Run {
    let start = StandardCombatStart { character, ascension: 20, encounter, burning: None, bottled_cards: vec![], position: None,
        resources: AuthoredRunStart { hp: 70, max_hp: 80, gold: 50, deck: deck(cards), relics: vec![], potions, relic_counters: Default::default() } };
    let mut run = Run::prepare_standard_combat(start, RunRng::new([seed; 32], rng::Partition::Test, 0)).unwrap();
    run.enter_standard_combat().unwrap();
    run
}
fn assert_refused(run: &Run, what: &str) {
    assert!(matches!(run.public_root_run(), Err(Unsupported::DisallowedContinuation(_))), "{what}: {:?}", run.public_root_run().map(|_| ()));
}
fn assert_equal_roots_under_perturbation(run: &Run, what: &str) {
    let base = root(run);
    let mut applied = 0;
    for (name, perturb) in PERTURBATIONS {
        let mut other = run.clone();
        if perturb(&mut other).is_none() { continue }
        assert!(other != *run, "{what}/{name}: the true worlds differ");
        assert!(root(&other) == base, "{what}/{name}: the root is invariant");
        applied += 1;
    }
    assert!(applied >= 2, "{what}: perturbations applied {applied}");
}

#[test]
fn m40a_duplication_repeat_of_an_enemy_targeted_selection_attack_is_a_public_root() {
    use crate::content::PotionId;
    let deck = [CardId::DaggerThrow, CardId::Strike, CardId::Strike, CardId::Defend, CardId::Defend, CardId::Strike, CardId::Defend, CardId::Strike, CardId::Defend, CardId::Strike];
    let mut run = (0..=255u8).map(|seed| standalone_with(Character::Silent, EncounterId::JawWorm, &deck, vec![Some(PotionId::DuplicationPotion), None], seed))
        .find(|run| hand_holds(run, &[1])).expect("Dagger Throw in the opening hand");
    act(&mut run, "Duplication potion", |a| matches!(a, crate::run::GameAction::Combat(CombatAction::UsePotion { slot: 0, .. })));
    play_card(&mut run, "Dagger Throw", 1);
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the original's discard selection");
    assert!(combat(&run).state.play_frames.is_empty());
    act(&mut run, "first pick", |a| matches!(a, crate::run::GameAction::Combat(CombatAction::SelectCard { .. })));
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the repeat's discard selection");
    assert!(matches!(combat(&run).state.play_frames.last().map(|f| &f.target), Some(Some(Actor::Enemy(_)))));
    assert_equal_roots_under_perturbation(&run, "Duplication + Dagger Throw");
}

/// Havoc autoplays the top card at a random enemy; Double Tap then queues a repeat of that child. Every boundary on the
/// way holds the random target, in the frame or in the queued repeat, and stays refused.
#[test]
fn m40a_havoc_child_and_its_double_tap_repeat_stay_refused() {
    let cards = [CardId::Havoc, CardId::DoubleTap, CardId::Headbutt, CardId::Strike, CardId::Strike, CardId::Strike, CardId::Strike, CardId::Strike, CardId::Strike, CardId::Strike];
    let mut run = (0..=255u8).map(|seed| standalone(Character::Ironclad, 20, EncounterId::TwoLouse, &cards, vec![], seed))
        .find(|run| hand_holds(run, &[1, 2]) && combat(run).state.piles.draw.last().is_some_and(|c| c.full().is_some_and(|f| f.id == CardId::Headbutt)))
        .expect("Havoc and Double Tap in hand, Headbutt on top");
    push_discard(&mut run, CardId::Strike, 3);
    play_card(&mut run, "Double Tap", 2);
    play_card(&mut run, "Havoc", 1);
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the Havoc child's own selection");
    assert!(matches!(combat(&run).state.play_frames.last().map(|f| &f.target), Some(Some(Actor::Enemy(_)))));
    assert_refused(&run, "(i) first selection: Havoc frame and a pending Double Tap repeat");
    act(&mut run, "first pick", |a| matches!(a, crate::run::GameAction::Combat(CombatAction::SelectCard { .. })));
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the repeat's selection");
    assert_eq!(combat(&run).state.play_frames.len(), 1);
    assert!(!combat(&run).state.public_target_plays.contains(&combat(&run).state.play_frames[0].child), "random lineage is not recorded");
    assert_refused(&run, "(iii) second selection: the repeat of a random-target child");
}

/// A queued repeat holding an enemy target is a public root only when its child is recorded as player-chosen; this is the
/// boundary with `play_frames` empty (a Melange scry after the parent's frame ended).
#[test]
fn m40a_pending_repeat_with_an_enemy_target_is_classified_by_provenance() {
    let run = selection_root();
    let state = &combat(&run).state;
    let enemy = state.enemies[0].instance_id.clone();
    assert!(state.play_frames.is_empty());
    let child = card_state::handle(CardInstanceId(777));
    let kinds = |child: &CardHandle, target: &Option<Actor>| vec![
        EffectKind::Ironclad(cards::IroncladEffect::DoubleTapRepeat { child: child.clone(), target: target.clone(), energy_on_use: 0, cancelled: false }),
        EffectKind::Silent(cards::SilentEffect::BurstRepeat { child: child.clone(), target: target.clone(), energy_on_use: 0, cancelled: false }),
        EffectKind::Potion(PotionEffect::DuplicationRepeat { child: child.clone(), target: target.clone(), energy_on_use: 0, cancelled: false }),
        EffectKind::Defect(cards::DefectEffect::Repeat { child: child.clone(), target: target.clone(), energy_on_use: 0, cancelled: false }),
    ];
    for kind in kinds(&child, &Some(Actor::Enemy(enemy.clone()))) {
        let mut other = run.clone();
        combat_mut(&mut other).state.pending.push_back(Effect { source: Source::None, target: None, kind: kind.clone() });
        assert_refused(&other, &format!("{kind:?} unrecorded"));
        combat_mut(&mut other).state.public_target_plays.insert(child.clone());
        assert!(other.public_root_run().is_ok(), "{kind:?} recorded: {:?}", other.public_root_run().map(|_| ()));
    }
    for kind in kinds(&child, &None).into_iter().chain(kinds(&child, &Some(Actor::Player))) {
        let mut other = run.clone();
        combat_mut(&mut other).state.pending.push_back(Effect { source: Source::None, target: None, kind });
        assert!(other.public_root_run().is_ok(), "a repeat with no enemy target is public");
    }
}

/// Watcher's Just Lucky scries before it deals damage, so the repeat's Scry selection is open while the frame holds the
/// player's enemy target.
#[test]
fn m40a_duplicated_just_lucky_scry_inside_the_repeat_frame_is_a_public_root() {
    use crate::content::PotionId;
    let deck = [CardId::JustLucky, CardId::Strike, CardId::Strike, CardId::Defend, CardId::Defend, CardId::Strike, CardId::Defend, CardId::Strike, CardId::Defend, CardId::Strike];
    let mut run = (0..=255u8).map(|seed| standalone_with(Character::Watcher, EncounterId::TwoLouse, &deck, vec![Some(PotionId::DuplicationPotion), None], seed))
        .find(|run| hand_holds(run, &[1])).expect("Just Lucky in the opening hand");
    act(&mut run, "Duplication potion", |a| matches!(a, crate::run::GameAction::Combat(CombatAction::UsePotion { slot: 0, .. })));
    play_card(&mut run, "Just Lucky", 1);
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the original's Scry");
    assert!(combat(&run).state.play_frames.is_empty());
    assert!(run.public_root_run().is_ok(), "a hand play's Scry is a root (no frame)");
    act(&mut run, "confirm the Scry", |a| matches!(a, crate::run::GameAction::Combat(CombatAction::ConfirmSelection)));
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the repeat's Scry");
    assert!(matches!(combat(&run).state.play_frames.last().map(|f| &f.target), Some(Some(Actor::Enemy(_)))));
    assert_equal_roots_under_perturbation(&run, "Duplication + Just Lucky");
}

/// The shared-repeat sources (Echo Form, Necronomicon) reach a selection inside their repeat frame through the same
/// `allocate_repeat` provenance write. Headbutt stands in for a selection attack: Echo Form is given as a power (the
/// character gate does not matter to the resolver) and Necronomicon as the relic with the card priced at 2.
fn shared_repeat_headbutt_selection(echo: bool) -> Run {
    let relics = if echo { vec![] } else { vec![RelicId::Necronomicon] };
    let mut run = hand_with(Character::Ironclad, EncounterId::JawWorm, &dt_deck(), relics, &[2]);
    push_discard(&mut run, CardId::Strike, 2);
    {
        let state = &mut combat_mut(&mut run).state;
        state.player.energy = 3;
        if echo {
            state.player_full_powers.push(model::FullPowerEntry { id: model::FullPowerId("echo_form"),
                state: model::PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
        } else {
            let card = state.piles.hand.iter_mut().filter_map(|c| c.full_mut()).find(|c| c.instance.0 == 2).unwrap();
            card.local.mutable_combat_cost = Some(2);
        }
    }
    play_card(&mut run, "Headbutt", 2);
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the original's selection");
    act(&mut run, "first pick", |a| matches!(a, crate::run::GameAction::Combat(CombatAction::SelectCard { .. })));
    assert_eq!(combat(&run).state.phase, Phase::Selection, "the repeat's selection");
    assert_eq!(combat(&run).state.play_frames.len(), 1);
    assert!(matches!(combat(&run).state.play_frames[0].target, Some(Actor::Enemy(_))));
    run
}
#[test]
fn m40a_echo_form_repeat_frame_with_the_players_target_is_a_public_root() {
    assert_equal_roots_under_perturbation(&shared_repeat_headbutt_selection(true), "Echo Form + Headbutt");
}
#[test]
fn m40a_necronomicon_repeat_frame_with_the_players_target_is_a_public_root() {
    assert_equal_roots_under_perturbation(&shared_repeat_headbutt_selection(false), "Necronomicon + Headbutt");
}

#[test]
fn m40b_retired_family_is_removed_from_canonical_and_sampled_worlds() {
    use crate::content::CardId::*;
    use card_state::CopyProvenance;
    let mut live = hand_with(Character::Ironclad, EncounterId::JawWorm,
        &[DoubleTap, Defend, Defend, Defend, Defend], vec![], &[1]);
    let a = combat_mut(&mut live).diagnostic_create_card(Cleave, 0, CardDestination::Hand).unwrap();
    let b = combat_mut(&mut live).diagnostic_create_card(Cleave, 0, CardDestination::Discard).unwrap();
    play_card(&mut live, "Double Tap", 1);
    play_card(&mut live, "generated Cleave", a);
    combat_mut(&mut live).diagnostic_apply(EffectKind::Recycle).unwrap();

    let retired = |run: &Run| {
        let card = combat(run).state.inactive_cards[0].full().unwrap();
        (card.instance.0, card.copy_family, card.copy_provenance)
    };
    let (child, family, provenance) = retired(&live);
    assert_eq!((family, provenance), (a, CopyProvenance::SameInstanceFamily));
    let mut twin = live.clone();
    combat_mut(&mut twin).state.inactive_cards[0].full_mut().unwrap().copy_family = b;
    assert_ne!(retired(&live), retired(&twin));
    assert_eq!(live.decision().unwrap().combat, twin.decision().unwrap().combat);
    let canonical = root(&live);
    assert!(perturb_draw_order(&mut twin).is_some());
    assert!(canonical == root(&twin));
    assert!(root_with(&live, Leak::KeepDrawOrder).unwrap() != root_with(&twin, Leak::KeepDrawOrder).unwrap());
    let normalized = (child, child, CopyProvenance::StatEquivalent);
    assert_eq!(retired(&canonical), normalized);
    for k in 0..8 {
        let world = sample(&canonical, k);
        assert!(world == sample(&root(&twin), k));
        assert_eq!(retired(&world), normalized);
        assert!(combat(&live).draw_knowledge().unwrap().supports(&combat(&world).true_draw_classes()));
    }
    assert_eq!(retired(&live), (child, a, CopyProvenance::SameInstanceFamily));
}

fn m40b_rekey_fixture() -> Run {
    use crate::content::CardId::*;
    let mut run = hand_with(Character::Defect, EncounterId::JawWorm,
        &[Claw, DefendBlue, DefendBlue, DefendBlue, DefendBlue], vec![], &[1]);
    let c = combat_mut(&mut run);
    c.diagnostic_create_card(Claw, 0, CardDestination::Discard).unwrap();
    c.diagnostic_create_card(Claw, 0, CardDestination::Discard).unwrap();
    c.diagnostic_create_card(StrikeBlue, 0, CardDestination::Discard).unwrap();
    c.diagnostic_apply(EffectKind::Recycle).unwrap();
    run
}

#[test]
fn m40b_rekey_posterior_survives_canonicalization_and_sampling() {
    let mut live = m40b_rekey_fixture();
    let before = combat(&live).draw_knowledge().unwrap();
    let mut twin = live.clone();
    assert!(perturb_draw_order(&mut twin).is_some());
    play_card(&mut live, "Claw", 1);
    play_card(&mut twin, "Claw", 1);
    let after = combat(&live).draw_knowledge().unwrap();
    assert_ne!(before, after);
    assert_eq!(after, combat(&twin).draw_knowledge().unwrap());
    let strike = *after.classes().iter().find(|c| c.id() == CardId::StrikeBlue).unwrap();
    let claw = *after.classes().iter().find(|c| c.id() == CardId::Claw).unwrap();
    let expected = (0..3).map(|position| {
        let mut draw = vec![claw; 3];
        draw[position] = strike;
        (draw, (1, 3))
    }).collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(belief::exact::sampler_distribution(&after), expected);
    let canonical = root(&live);
    assert_eq!(combat(&canonical).draw_knowledge(), Ok(after.clone()));
    assert!(canonical == root(&twin));
    for k in 0..8 {
        let world = sample(&canonical, k);
        assert!(after.supports(&combat(&world).true_draw_classes()));
    }
}

#[test]
fn m40b_rekey_perturbations_preserve_the_synchronized_law() {
    let mut live = m40b_rekey_fixture();
    play_card(&mut live, "Claw", 1);
    let law = combat(&live).draw_knowledge().unwrap();
    let truth = combat(&live).true_draw_classes();
    let mut swap = live.clone();
    assert!(perturb_draw_order(&mut swap).is_some());
    assert_ne!(combat(&swap).true_draw_classes(), truth);
    assert_eq!(combat(&swap).draw_knowledge(), Ok(law.clone()));
    let mut fuzz = live.clone();
    assert_ne!(combat_mut(&mut fuzz).fuzz_perturb_hidden(3) & 2, 0);
    assert_ne!(combat(&fuzz).true_draw_classes(), truth);
    assert_eq!(combat(&fuzz).draw_knowledge(), Ok(law.clone()));
    let mut labels = live.clone();
    let c = combat_mut(&mut labels);
    let (a, b) = c.diagnostic_identical_copies()[0];
    assert!(c.diagnostic_swap_identical_copies(a, b));
    assert_eq!(c.draw_knowledge(), Ok(law));
    assert!(root(&swap) == root(&fuzz));
    assert!(root(&labels) == root(&live));
    assert!(root_with(&live, Leak::KeepDrawOrder).unwrap() != root_with(&swap, Leak::KeepDrawOrder).unwrap());
    assert!(root_with(&live, Leak::KeepStreams).unwrap() != root_with(&fuzz, Leak::KeepStreams).unwrap());
}

#[test]
fn m40b_anchors_accept_agreeing_top_and_bottom() {
    let mut live = standalone(Character::Watcher, 0, EncounterId::JawWorm,
        &[CardId::DefendPurple; 5], vec![], 2);
    let c = combat_mut(&mut live);
    c.diagnostic_create_card(CardId::Insight, 0, CardDestination::Draw).unwrap();
    assert_eq!(c.state.public.known_draw, c.state.public.known_draw_bottom);
    let truth = c.true_draw_classes();
    assert_eq!(truth.len(), 1);
    let mut twin = live.clone();
    perturb_streams(&mut twin).unwrap();
    let canonical = root(&live);
    assert!(canonical == root(&twin));
    assert_eq!(combat(&sample(&canonical, 4)).true_draw_classes(), truth);
}

#[test]
fn m40b_retired_family_keeps_live_and_family_sensitive_relatives_guarded() {
    use crate::content::CardId::*;
    for card in [Cleave, Rampage] {
        let mut live = hand_with(Character::Ironclad, EncounterId::JawWorm,
            &[DoubleTap, card, Defend, Defend, Defend], vec![], &[1, 2]);
        play_card(&mut live, "Double Tap", 1);
        play_card(&mut live, "permanent attack", 2);
        combat_mut(&mut live).diagnostic_apply(EffectKind::Recycle).unwrap();
        assert_eq!(combat(&live).state.inactive_cards.len(), 1);
        if card == Rampage {
            assert_eq!(live.public_root_run().err(), Some(Unsupported::UnrepresentableKnowledge));
        } else {
            assert!(live.public_root_run().is_ok());
            let state = &mut combat_mut(&mut live).state;
            state.piles.hand.push(state.inactive_cards.pop().unwrap());
            assert_eq!(live.public_root_run().err(), Some(Unsupported::UnrepresentableKnowledge));
        }
    }
}

#[test]
fn m40b_rekey_preserves_force_field_and_corruption_cost_changes() {
    use crate::content::CardId::*;
    for (character, played, affected, filler) in [
        (Character::Defect, Defragment, ForceField, DefendBlue),
        (Character::Ironclad, Corruption, ShrugItOff, Defend),
    ] {
        let mut live = hand_with(character, EncounterId::JawWorm,
            &[played, filler, filler, filler, filler], vec![], &[1]);
        let c = combat_mut(&mut live);
        c.diagnostic_create_card(affected, 0, CardDestination::Discard).unwrap();
        c.diagnostic_create_card(Wound, 0, CardDestination::Discard).unwrap();
        c.diagnostic_apply(EffectKind::Recycle).unwrap();
        let before = c.draw_knowledge().unwrap();
        play_card(&mut live, "public power", 1);
        let after = combat(&live).draw_knowledge().unwrap();
        assert_ne!(before, after);
        assert_eq!(combat(&root(&live)).draw_knowledge(), Ok(after));
    }
}

#[test]
fn m40b_anchors_refuse_conflicting_handles_even_when_classes_match() {
    let mut live = standalone(Character::Watcher, 0, EncounterId::JawWorm,
        &[CardId::DefendPurple; 5], vec![], 2);
    let c = combat_mut(&mut live);
    c.diagnostic_create_card(CardId::Wound, 0, CardDestination::Draw).unwrap();
    c.diagnostic_create_card(CardId::Wound, 0, CardDestination::Draw).unwrap();
    let good = c.state.public.known_draw_bottom.clone();
    c.state.public.known_draw_bottom = vec![c.state.public.known_draw[0].clone()];
    assert!(c.draw_knowledge().is_ok(), "equal classes cannot detect a corrupt anchor");
    assert_eq!(live.public_root_run().err(), Some(Unsupported::UnrepresentableKnowledge));

    let c = combat_mut(&mut live);
    c.state.public.known_draw_bottom = good;
    c.state.public.known_draw_bottom.push(c.state.public.known_draw[1].clone());
    c.state.public.known_draw_bottom.push(c.state.public.known_draw[0].clone());
    assert_eq!(live.public_root_run().err(), Some(Unsupported::UnrepresentableKnowledge));
}
