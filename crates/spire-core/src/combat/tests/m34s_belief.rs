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
        if seq != before && k.supports(&seq) { c.state.piles.draw = trial; knowledge::resync(&mut c.state); return Some(()) }
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
    combat_mut(&mut framed).state.play_frames.push(model::PlayFrame { parent: None, parent_owner: model::PlayFrameParentOwner::Card(child_card.handle()),
        parent_context: None, child: child_card.handle(), target: Some(Actor::Enemy(enemy.clone())), resume_phase: Phase::Decision, captured_energy: 0, paid: 0,
        disposition: model::FinishDisposition::Natural, parent_finish_checkpoint_done: false });
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
        thief_gold: _ /*P*/, colorless, defect, watcher, knowledge: _ /*P: public knowledge; its physical bookkeeping is resynchronised*/,
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
