//! M27 plan Task J1, lane J, key m27_relics_structural.
//! Complete-admission combat tests for the structural relics (Part 1): Toolbox, Nilry's Codex,
//! Gambling Chip, Strange Spoon, Unceasing Top, Snecko Eye, Velvet Choker, Runic Pyramid,
//! Runic Dome, Frozen Eye and Necronomicon. Spec-sheet literals: task0/spec-sheets-v2/j.md.

use crate::combat::{Actor, CardDestination, CombatAction, Effect, EffectKind, FullIntentId, FullPowerId,
    Source, TargetId};
use crate::content::{CardId, CardType, Character, EncounterId, PotionId, RelicId};
use crate::items::{dispatch::PhaseCard, laws};
use crate::run::Run;
use super::m27_support;
use std::collections::BTreeMap;

const T1: Option<TargetId> = Some(TargetId(1));

/// A real complete-admission Run whose first (fixture) combat is settled with three Fire
/// Potions and no relic; the authored deck, relics and potions are then installed in RunState
/// and the measured combat is entered through the shared Run owner.
fn run_with(relics: &[RelicId], cards: &[CardId], potions: Vec<Option<PotionId>>, encounter: EncounterId) -> Run {
    run_as(Character::Ironclad, relics, cards, potions, encounter)
}

fn run_as(character: Character, relics: &[RelicId], cards: &[CardId], potions: Vec<Option<PotionId>>,
    encounter: EncounterId) -> Run {
    let mut run = m27_support::complete_run(character, vec![],
        vec![Some(PotionId::FirePotion); 3], BTreeMap::new(), EncounterId::Cultist);
    for slot in 0..3 {
        if run.state().combat.is_none() { break; }
        run.choose_combat(CombatAction::UsePotion { slot, target: T1 })
            .unwrap_or_else(|error| panic!("fixture Fire Potion {slot}: {error:?}"));
    }
    assert!(run.state().combat.is_none(), "three fixture Fire Potions settle the first combat");
    let state = run.state_mut_for_test();
    state.deck = cards.iter().enumerate().map(|(index, id)| crate::run::CardInstance {
        instance: 100 + index as u64, id: *id, upgrades: 0, misc: 0 }).collect();
    state.relics = relics.to_vec();
    state.potions = potions;
    m27_support::next_complete_combat(&mut run, encounter).unwrap_or_else(|error| panic!("measured combat: {error:?}"));
    run
}

fn simple(relics: &[RelicId], cards: &[CardId]) -> Run { run_with(relics, cards, vec![None; 3], EncounterId::Cultist) }

fn view(run: &Run) -> crate::combat::CombatPublic {
    run.state().combat.as_ref().expect("live complete combat").public_full().unwrap()
}

fn local(run: &Run, relic: RelicId) -> i32 {
    run.state().combat.as_ref().unwrap().item_context().unwrap().local(relic)
}

fn set_energy(run: &mut Run, energy: i32) {
    crate::combat::tests::m27_state::set_player_energy(run.state_mut_for_test().combat.as_mut().unwrap(), energy);
}

fn enqueue(run: &mut Run, kind: EffectKind) {
    run.state_mut_for_test().combat.as_mut().unwrap()
        .enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player), kind }], None)
        .unwrap_or_else(|error| panic!("enqueue: {error:?}"));
}

fn create(run: &mut Run, id: CardId) {
    enqueue(run, EffectKind::CreateCard { id, destination: CardDestination::Hand, upgrades: 0 });
}

fn hand_instance(run: &Run, id: CardId) -> Option<u64> {
    view(run).hand.iter().find(|card| card.id == id).map(|card| card.instance.0)
}

/// End turns until every id is in hand (energy refilled to 100 each turn).
fn ensure(run: &mut Run, ids: &[CardId]) {
    for _ in 0..12 {
        set_energy(run, 100);
        if ids.iter().all(|id| hand_instance(run, *id).is_some()) { return; }
        run.choose_combat(CombatAction::EndTurn).unwrap();
        assert!(run.state().combat.is_some(), "combat ended while drawing {ids:?}");
    }
    panic!("{ids:?} not drawn together: {:?}", view(run).hand);
}

fn play(run: &mut Run, id: CardId, target: Option<TargetId>) {
    let instance = hand_instance(run, id).unwrap_or_else(|| panic!("{id:?} not in hand: {:?}", view(run).hand));
    run.choose_combat(CombatAction::PlayCard { instance, target })
        .unwrap_or_else(|error| panic!("play {id:?}: {error:?}"));
}

fn playable_count(run: &Run) -> usize {
    run.state().combat.as_ref().unwrap().legal_actions_full().unwrap().iter()
        .filter(|action| matches!(action, CombatAction::PlayCard { .. })).count()
}

fn enemy_power(run: &Run, id: &str) -> i32 {
    view(run).enemies[0].powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.amount)
}

fn player_power(run: &Run, id: &str) -> i32 {
    view(run).player.powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.amount)
}

fn win_with_potions(run: &mut Run) {
    for slot in 0..3 {
        if run.state().combat.is_none() { break; }
        run.choose_combat(CombatAction::UsePotion { slot, target: T1 })
            .unwrap_or_else(|error| panic!("Fire Potion {slot}: {error:?}"));
    }
    assert!(run.state().combat.is_none(), "Fire Potions settle the combat");
}

// ---- Velvet Choker -------------------------------------------------------------------------

#[test]
fn m27_velvet_choker_six_play_limit() {
    let mut run = simple(&[RelicId::VelvetChoker], &[CardId::ShrugItOff; 12]);
    set_energy(&mut run, 100);
    for play_index in 0..6 {
        assert!(playable_count(&run) > 0, "play {} is legal", play_index + 1);
        play(&mut run, CardId::ShrugItOff, None);
    }
    assert_eq!(local(&run, RelicId::VelvetChoker), 6);
    let instance = hand_instance(&run, CardId::ShrugItOff).expect("a card is still in hand");
    assert_eq!(playable_count(&run), 0, "the seventh play is absent from the action mask");
    let before = run.state().combat.as_ref().unwrap().snapshot();
    assert!(run.choose_combat(CombatAction::PlayCard { instance, target: None }).is_err(), "seventh play rejected");
    assert_eq!(run.state().combat.as_ref().unwrap().snapshot(), before, "rejection leaves combat unchanged");

    run.choose_combat(CombatAction::EndTurn).unwrap();
    set_energy(&mut run, 100);
    assert_eq!(local(&run, RelicId::VelvetChoker), 0, "next turn resets plays 6 -> 0");
    play(&mut run, CardId::ShrugItOff, None);
    assert_eq!(local(&run, RelicId::VelvetChoker), 1);
}

#[test]
fn m27_velvet_choker_boundary() {
    // Autoplay copies are plays too: Double Tap's Strike copy counts toward the six.
    let mut deck = vec![CardId::ShrugItOff; 8];
    deck.extend([CardId::DoubleTap, CardId::DoubleTap, CardId::Strike, CardId::Strike]);
    let mut run = simple(&[RelicId::VelvetChoker], &deck);
    ensure(&mut run, &[CardId::DoubleTap, CardId::Strike]);
    assert_eq!(local(&run, RelicId::VelvetChoker), 0);
    play(&mut run, CardId::DoubleTap, None);
    play(&mut run, CardId::Strike, T1);
    assert_eq!(local(&run, RelicId::VelvetChoker), 3, "Double Tap, Strike and its copy are three plays");
    for _ in 0..3 {
        let id = view(&run).hand.iter().map(|card| card.id).find(|id| *id != CardId::Strike).expect("a non-Strike card");
        play(&mut run, id, if id == CardId::Strike { T1 } else { None });
    }
    assert_eq!(playable_count(&run), 0, "six plays including the autoplay copy close the mask");
}

#[test]
fn m27_velvet_choker_two_combat_counter() {
    let mut run = run_with(&[RelicId::VelvetChoker], &[CardId::ShrugItOff; 12],
        vec![Some(PotionId::FirePotion); 3], EncounterId::Cultist);
    set_energy(&mut run, 100);
    for _ in 0..6 { play(&mut run, CardId::ShrugItOff, None); }
    assert_eq!(playable_count(&run), 0, "combat 1 rejects play 7");
    win_with_potions(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::JawWorm).unwrap();
    assert_eq!(local(&run, RelicId::VelvetChoker), 0, "combat 2 starts at 0");
    set_energy(&mut run, 100);
    for _ in 0..6 { play(&mut run, CardId::ShrugItOff, None); }
    assert_eq!(playable_count(&run), 0, "combat 2 rejects play 7 again");
}

// ---- Necronomicon --------------------------------------------------------------------------

#[test]
fn m27_necronomicon_first_attack_repeat() {
    let mut run = simple(&[RelicId::Necronomicon], &[CardId::Clothesline; 12]);
    set_energy(&mut run, 100);
    play(&mut run, CardId::Clothesline, T1);
    assert_eq!(enemy_power(&run, "weak"), 4, "paid cost 2 plays twice (Weak 2 + 2)");
    assert_eq!(view(&run).player.energy, 98, "energy is paid once");
    play(&mut run, CardId::Clothesline, T1);
    assert_eq!(enemy_power(&run, "weak"), 6, "no second repeat within the same turn");

    // Another combat-local latch check: X paid as 2 qualifies and the repeat is free.
    let mut run = simple(&[RelicId::Necronomicon], &[CardId::Whirlwind; 12]);
    let hp = view(&run).enemies[0].hp;
    set_energy(&mut run, 2);
    play(&mut run, CardId::Whirlwind, None);
    assert_eq!(hp - view(&run).enemies[0].hp, 20, "X=2 Whirlwind hits 2x5 twice");
    assert_eq!(view(&run).player.energy, 0);
}

#[test]
fn m27_necronomicon_boundary() {
    let mut deck = vec![CardId::Strike; 6];
    deck.extend([CardId::Clothesline; 6]);
    let mut run = simple(&[RelicId::Necronomicon], &deck);
    ensure(&mut run, &[CardId::Strike, CardId::Clothesline]);
    let (hp, weak) = (view(&run).enemies[0].hp, enemy_power(&run, "weak"));
    play(&mut run, CardId::Strike, T1);
    assert_eq!(hp - view(&run).enemies[0].hp, 6, "paid cost 1 plays once");
    assert_eq!(local(&run, RelicId::Necronomicon), 0, "a non-qualifying Attack leaves the latch armed");
    play(&mut run, CardId::Clothesline, T1);
    assert_eq!(enemy_power(&run, "weak") - weak, 4, "the first qualifying Attack still repeats");
}

#[test]
fn m27_necronomicon_two_combat_counter() {
    let mut run = run_with(&[RelicId::Necronomicon], &[CardId::Clothesline; 12],
        vec![Some(PotionId::FirePotion); 3], EncounterId::Cultist);
    set_energy(&mut run, 100);
    play(&mut run, CardId::Clothesline, T1);
    assert_eq!(local(&run, RelicId::Necronomicon), 1, "used false -> true");
    play(&mut run, CardId::Clothesline, T1);
    assert_eq!(enemy_power(&run, "weak"), 6, "further qualifying attacks repeat 0 times");
    win_with_potions(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(local(&run, RelicId::Necronomicon), 0, "combat 2 resets true -> false");
    set_energy(&mut run, 100);
    play(&mut run, CardId::Clothesline, T1);
    assert_eq!(enemy_power(&run, "weak"), 4, "combat 2's first qualifying Attack totals 2 plays");
}

#[test]
fn m27_necronomicon_with_duplicators() {
    // Review Focus 5 / F9: Double Tap once and Necronomicon once, energy paid once.
    let mut deck = vec![CardId::DoubleTap; 4];
    deck.extend([CardId::Clothesline; 8]);
    let mut run = simple(&[RelicId::Necronomicon], &deck);
    ensure(&mut run, &[CardId::DoubleTap, CardId::Clothesline]);
    let weak = enemy_power(&run, "weak");
    let energy = view(&run).player.energy;
    play(&mut run, CardId::DoubleTap, None);
    let hp = view(&run).enemies[0].hp;
    play(&mut run, CardId::Clothesline, T1);
    assert_eq!(enemy_power(&run, "weak") - weak, 6, "three plays on the same target");
    assert_eq!(hp - view(&run).enemies[0].hp, 36, "3 x 12 damage");
    assert_eq!(energy - view(&run).player.energy, 3, "Double Tap 1 + Clothesline 2, repeats free");
    assert_eq!(player_power(&run, "double_tap"), 0);
    // A second paid-2 Attack the same turn: neither duplicator repeats it.
    if hand_instance(&run, CardId::Clothesline).is_some() {
        play(&mut run, CardId::Clothesline, T1);
        assert_eq!(enemy_power(&run, "weak") - weak, 8);
    }
}

#[test]
fn m27_necronomicon_repeat_law_is_one_latch() {
    let run = simple(&[RelicId::Necronomicon], &[CardId::Defend; 12]);
    let mut context = run.state().combat.as_ref().unwrap().item_context().unwrap().clone();
    let attack = |cost_paid| PhaseCard { id: CardId::Clothesline, card_type: CardType::Attack, cost_paid };
    assert_eq!(laws::repeat_play(&mut context, PhaseCard { id: CardId::Defend, card_type: CardType::Skill, cost_paid: 2 }), 0);
    assert_eq!(laws::repeat_play(&mut context, attack(1)), 0);
    assert_eq!(laws::repeat_play(&mut context, attack(2)), 1);
    assert_eq!(laws::repeat_play(&mut context, attack(3)), 0, "used this turn");
}

// ---- Unceasing Top -------------------------------------------------------------------------

#[test]
fn m27_unceasing_top_empty_hand_draw() {
    let mut run = simple(&[RelicId::UnceasingTop], &[CardId::Defend; 12]);
    set_energy(&mut run, 100);
    assert_eq!(view(&run).hand.len(), 5);
    for _ in 0..5 { play(&mut run, CardId::Defend, None); }
    assert_eq!(view(&run).hand.len(), 1, "an emptied hand during the turn draws exactly 1");
    assert_eq!(view(&run).draw_count, 6);
    play(&mut run, CardId::Defend, None);
    assert_eq!(view(&run).hand.len(), 1, "each later emptying draws 1 again");
}

#[test]
fn m27_unceasing_top_boundary() {
    // No Draw blocks the refresh; a nonempty hand never draws.
    let mut run = simple(&[RelicId::UnceasingTop], &[CardId::Defend; 12]);
    set_energy(&mut run, 100);
    for _ in 0..4 { play(&mut run, CardId::Defend, None); }
    assert_eq!(view(&run).hand.len(), 1, "nonempty hand draws 0");
    assert_eq!(view(&run).draw_count, 7);
    enqueue(&mut run, EffectKind::ApplyFullPower { id: FullPowerId("no_draw"), amount: 1 });
    play(&mut run, CardId::Defend, None);
    assert_eq!(view(&run).hand.len(), 0, "No Draw: empty hand draws 0");
    assert_eq!(view(&run).draw_count, 7);
}

// ---- Runic Dome / Frozen Eye ---------------------------------------------------------------

#[test]
fn m27_runic_dome_hide_enemy_intents() {
    let dome = simple(&[RelicId::RunicDome], &[CardId::Defend; 12]);
    let plain = simple(&[], &[CardId::Defend; 12]);
    for enemy in view(&dome).enemies {
        assert_eq!(enemy.intent, FullIntentId::Unknown("hidden"));
        assert_eq!(enemy.intent_damage, None);
    }
    assert!(view(&plain).enemies.iter().all(|enemy| enemy.intent != FullIntentId::Unknown("hidden")));
}

#[test]
fn m27_runic_dome_boundary() {
    // Hidden display only: the actual enemy action and RNG are unchanged.
    let mut dome = simple(&[RelicId::RunicDome], &[CardId::Defend; 12]);
    let mut plain = simple(&[], &[CardId::Defend; 12]);
    for _ in 0..3 {
        dome.choose_combat(CombatAction::EndTurn).unwrap();
        plain.choose_combat(CombatAction::EndTurn).unwrap();
        let (d, p) = (view(&dome), view(&plain));
        assert_eq!(d.player.hp, p.player.hp);
        assert_eq!(d.enemies.iter().map(|e| (e.hp, e.powers.clone())).collect::<Vec<_>>(),
            p.enemies.iter().map(|e| (e.hp, e.powers.clone())).collect::<Vec<_>>());
    }
}

#[test]
fn m27_frozen_eye_draw_order_visible() {
    let mut deck = vec![CardId::Defend; 6];
    deck.extend([CardId::Strike; 6]);
    let mut run = simple(&[RelicId::FrozenEye], &deck);
    let public = view(&run);
    assert_eq!(public.known_draw.len(), public.draw_count, "the whole current draw pile, no more");
    assert!(public.known_draw.iter().enumerate().all(|(i, card)| card.from_top && card.offset == i));
    let next: Vec<_> = public.known_draw.iter().take(5).map(|card| card.card.instance).collect();
    run.choose_combat(CombatAction::EndTurn).unwrap();
    let drawn: Vec<_> = view(&run).hand.iter().map(|card| card.instance).collect();
    assert_eq!(drawn, next, "the exposed top-first order is the actual draw order");
}

#[test]
fn m27_frozen_eye_boundary() {
    let plain = simple(&[], &[CardId::Defend; 12]);
    assert!(view(&plain).known_draw.is_empty(), "without Frozen Eye the order is not public");
    let mut eye = simple(&[RelicId::FrozenEye], &[CardId::Defend; 12]);
    set_energy(&mut eye, 100);
    play(&mut eye, CardId::Defend, None);
    let public = view(&eye);
    assert_eq!(public.discard.len(), 1);
    assert_eq!(public.known_draw.len(), public.draw_count, "discard / future shuffle order is not exposed");
}

// ---- Snecko Eye ----------------------------------------------------------------------------

#[test]
fn m27_snecko_eye_confused_on_draw() {
    let mut run = simple(&[RelicId::SneckoEye], &[CardId::Defend; 20]);
    assert_eq!(player_power(&run, "confused"), 1, "start each combat Confused");
    create(&mut run, CardId::Miracle);
    let miracle = view(&run).hand.into_iter().find(|card| card.id == CardId::Miracle).unwrap();
    assert_eq!(miracle.effective_cost, Some(0), "a directly added card keeps its cost");
    run.choose_combat(CombatAction::EndTurn).unwrap();
    let costs: Vec<_> = view(&run).hand.iter().filter(|card| card.id == CardId::Defend)
        .map(|card| card.effective_cost).collect();
    assert!(costs.iter().all(|cost| cost.is_some_and(|cost| (0..=3).contains(&cost))), "{costs:?}");
}

#[test]
fn m27_snecko_eye_boundary() {
    // +2 master hand size (catalog "draw 2 additional cards"): opening and turn draws are 7.
    let mut run = simple(&[RelicId::SneckoEye], &[CardId::Defend; 20]);
    assert_eq!(view(&run).hand.len(), 7, "opening hand 5 + 2");
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).hand.len(), 7, "turn draw 5 + 2");
}

// ---- Runic Pyramid -------------------------------------------------------------------------

#[test]
fn m27_runic_pyramid_retain_hand() {
    let mut run = simple(&[RelicId::RunicPyramid], &[CardId::Defend; 12]);
    let kept: Vec<_> = view(&run).hand.iter().map(|card| card.instance).collect();
    create(&mut run, CardId::Dazed);
    run.choose_combat(CombatAction::EndTurn).unwrap();
    let public = view(&run);
    assert!(kept.iter().all(|instance| public.hand.iter().any(|card| card.instance == *instance)),
        "ordinary hand cards are retained: {:?}", public.hand);
    assert!(public.exhaust.iter().any(|card| card.id == CardId::Dazed), "Ethereal still exhausts");
    assert!(public.discard.is_empty(), "no end-of-turn hand discard");
}

#[test]
fn m27_runic_pyramid_boundary() {
    let mut run = simple(&[], &[CardId::Defend; 12]);
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).discard.len(), 5, "without Pyramid the hand is discarded");
}

// ---- Strange Spoon -------------------------------------------------------------------------

#[test]
fn m27_strange_spoon_law_consumes_one_boolean() {
    let run = simple(&[RelicId::StrangeSpoon], &[CardId::Defend; 12]);
    let mut context = run.state().combat.as_ref().unwrap().item_context().unwrap().clone();
    let skill = PhaseCard { id: CardId::Impervious, card_type: CardType::Skill, cost_paid: 2 };
    context.set_local(RelicId::StrangeSpoon, crate::items::relic_effects::structural::SPOON_SAVE);
    assert!(!laws::would_exhaust(&mut context, skill), "a saving roll keeps the card");
    assert_eq!(context.local(RelicId::StrangeSpoon), 0, "the Boolean is consumed");
    assert!(laws::would_exhaust(&mut context, skill), "no roll, no save");
    context.set_local(RelicId::StrangeSpoon, crate::items::relic_effects::structural::SPOON_SAVE);
    assert!(laws::would_exhaust(&mut context, PhaseCard { card_type: CardType::Power, ..skill }), "Power excluded");
}

#[test]
fn m27_strange_spoon_eligible_exhaust_save() {
    // Ruling STRANGE-SPOON-OMNISCIENCE: a saved card goes to the discard pile (spec-sheet
    // "draw pile" wording contradicts data/relics/relics.json:8596; source law followed).
    let mut run = simple(&[RelicId::StrangeSpoon], &[CardId::Impervious; 12]);
    let (mut saved, mut exhausted, mut plays) = (0, 0, 0);
    for _ in 0..4 {
        set_energy(&mut run, 100);
        while hand_instance(&run, CardId::Impervious).is_some() {
            let (d, e) = (view(&run).discard.len(), view(&run).exhaust.len());
            play(&mut run, CardId::Impervious, None);
            plays += 1;
            saved += view(&run).discard.len() - d;
            exhausted += view(&run).exhaust.len() - e;
        }
        run.choose_combat(CombatAction::EndTurn).unwrap();
    }
    assert!(plays >= 12);
    assert_eq!(saved + exhausted, plays, "each played Impervious goes to exactly one pile");
    assert!(saved > 0 && exhausted > 0, "the 50% roll saves some and exhausts some: {saved}/{exhausted}");
}

#[test]
fn m27_strange_spoon_boundary() {
    // A non-played exhaust (Ethereal at end of turn) consumes no Spoon Boolean.
    let mut run = simple(&[RelicId::StrangeSpoon], &[CardId::Defend; 12]);
    let _ = laws::take_law_log();
    create(&mut run, CardId::Dazed);
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert!(view(&run).exhaust.iter().any(|card| card.id == CardId::Dazed), "Ethereal exhausts");
    assert!(!laws::take_law_log().contains(&"would_exhaust"), "no Spoon law outside played cleanup");
}

// ---- Selection relics: Toolbox, Nilry's Codex, Gambling Chip -------------------------------

fn colorless(id: CardId) -> bool {
    crate::content::Catalog::standard().card(id).is_some_and(|card| card.color == crate::content::CardColor::Colorless)
}

#[test]
fn m27_toolbox_three_colorless_choose() {
    let mut run = simple(&[RelicId::Toolbox], &[CardId::Defend; 12]);
    let offer = view(&run).card_offer.expect("Toolbox offer before the opening draw");
    assert_eq!(offer.cards.len(), 3);
    assert!(!offer.optional);
    assert!(offer.cards.iter().all(|id| colorless(*id)), "three colourless ids: {:?}", offer.cards);
    assert!(view(&run).hand.is_empty(), "offer resolves before the opening draw");
    let chosen = offer.cards[0];
    run.choose_combat(CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    let public = view(&run);
    assert_eq!(public.hand.iter().filter(|card| card.id == chosen).count(), 1, "exactly the chosen card");
    assert_eq!(public.hand.len(), 6, "chosen card + opening hand of 5");
}

#[test]
fn m27_toolbox_boundary() {
    let mut run = simple(&[RelicId::Toolbox], &[CardId::Defend; 12]);
    let offer = view(&run).card_offer.expect("Toolbox offer");
    let mut unique = offer.cards.clone();
    unique.sort_by_key(|id| id.as_str());
    unique.dedup();
    assert_eq!(unique.len(), 3, "three distinct ids");
    run.choose_combat(CombatAction::ChooseCardOffer { index: 1 }).unwrap();
    assert!(view(&run).discard.is_empty() && view(&run).exhaust.is_empty(), "nothing auto-plays");
}

#[test]
fn m27_nilrys_codex_choose_one_of_three() {
    let mut run = simple(&[RelicId::NilrysCodex], &[CardId::Defend; 12]);
    let draw = view(&run).draw_count;
    run.choose_combat(CombatAction::EndTurn).unwrap();
    let offer = view(&run).card_offer.expect("end-turn Codex offer");
    assert_eq!(offer.cards.len(), 3);
    assert!(offer.optional, "Codex is skippable");
    let chosen = offer.cards[0];
    run.choose_combat(CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    let public = view(&run);
    assert!(!public.discard.iter().any(|card| card.id == chosen), "not discarded with the hand");
    assert_eq!(public.draw_count + public.hand.len(), draw + 1, "one card inserted into the draw pile");
}

#[test]
fn m27_nilrys_codex_boundary() {
    let mut run = simple(&[RelicId::NilrysCodex], &[CardId::Defend; 12]);
    let draw = view(&run).draw_count;
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert!(view(&run).card_offer.is_some());
    run.choose_combat(CombatAction::SkipCardOffer).unwrap();
    let public = view(&run);
    assert_eq!(public.draw_count + public.hand.len(), draw, "skipping inserts nothing");
}

#[test]
fn m27_gambling_chip_opening_discard_redraw() {
    let mut deck = vec![CardId::Defend; 6];
    deck.extend([CardId::Strike; 6]);
    let mut run = simple(&[RelicId::GamblingChip], &deck);
    let selection = view(&run).selection.expect("Gambling Chip selection after the opening draw");
    assert_eq!((selection.minimum, selection.maximum), (0, 5));
    let hand: Vec<_> = view(&run).hand.iter().map(|card| card.instance.0).collect();
    for instance in &hand[..2] { run.choose_combat(CombatAction::SelectCard { instance: *instance }).unwrap(); }
    run.choose_combat(CombatAction::ConfirmSelection).unwrap();
    let public = view(&run);
    assert_eq!(public.discard.len(), 2, "two discarded");
    assert_eq!(public.hand.len(), 5, "two redrawn");
    assert_eq!(local(&run, RelicId::GamblingChip), 1);
}

#[test]
fn m27_gambling_chip_boundary() {
    let mut run = simple(&[RelicId::GamblingChip], &[CardId::Defend; 12]);
    assert!(view(&run).selection.is_some(), "selection after the opening draw");
    run.choose_combat(CombatAction::ConfirmSelection).unwrap();
    let public = view(&run);
    assert_eq!((public.hand.len(), public.discard.len()), (5, 0), "empty choice redraws 0");
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert!(view(&run).selection.is_none(), "no second selection on turn 2");
}

/// Integration-4 follow-up: with a manual-discard consumer owned, the Gambling Chip discards
/// raise the callback (Tough Bandages: gain 3 Block per discarded card).
#[test]
fn m27_gambling_chip_discards_trigger_tough_bandages() {
    let mut deck = vec![CardId::DefendGreen; 6];
    deck.extend([CardId::StrikeGreen; 6]);
    let mut run = run_as(Character::Silent, &[RelicId::GamblingChip, RelicId::ToughBandages], &deck,
        vec![None; 3], EncounterId::Cultist);
    assert!(view(&run).selection.is_some(), "Gambling Chip selection after the opening draw");
    let block = run.state().combat.as_ref().unwrap().public_full().unwrap().player.block;
    let hand: Vec<_> = view(&run).hand.iter().map(|card| card.instance.0).collect();
    for instance in &hand[..2] { run.choose_combat(CombatAction::SelectCard { instance: *instance }).unwrap(); }
    run.choose_combat(CombatAction::ConfirmSelection).unwrap();
    let public = view(&run);
    assert_eq!((public.hand.len(), public.discard.len()), (5, 2));
    assert_eq!(public.player.block, block + 6, "Tough Bandages: 3 Block per discarded card");
}

#[test]
fn m27_gambling_chip_two_combat_counter() {
    let mut run = run_with(&[RelicId::GamblingChip], &[CardId::Defend; 12],
        vec![Some(PotionId::FirePotion); 3], EncounterId::Cultist);
    assert_eq!(local(&run, RelicId::GamblingChip), 1, "activated false -> true at first postdraw");
    assert!(view(&run).selection.is_some(), "combat 1 selection");
    run.choose_combat(CombatAction::ConfirmSelection).unwrap();
    win_with_potions(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert!(view(&run).selection.is_some(), "combat 2 rearms and allows one selection");
}
