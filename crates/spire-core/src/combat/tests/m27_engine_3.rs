//! M27 Main-2 shared owners for engine additions batch 3.
use crate::combat::{self, Actor, CardDestination, Combat, CombatAction, Effect, EffectKind,
    FullPowerEntry, FullPowerId, Hook, Source, TargetId};
use crate::content::{CardId, Character, EncounterId, PotionId, RelicId};
use crate::items::tests::m27_support;
use crate::run::Run;
use std::collections::BTreeMap;

const TARGET: Option<TargetId> = Some(TargetId(1));

fn run_with(relics: &[RelicId], cards: &[CardId]) -> Run {
    let mut run = m27_support::complete_run(Character::Ironclad, vec![],
        vec![Some(PotionId::FirePotion); 3], BTreeMap::new(), EncounterId::Cultist);
    for slot in 0..3 {
        if run.state().combat.is_none() { break; }
        run.choose_combat(CombatAction::UsePotion { slot, target: TARGET }).unwrap();
    }
    assert!(run.state().combat.is_none());
    let state = run.state_mut_for_test();
    state.deck = cards.iter().enumerate().map(|(index, id)| crate::run::CardInstance {
        instance: 100 + index as u64, id: *id, upgrades: 0, misc: 0 }).collect();
    state.relics = relics.to_vec();
    state.potions = vec![None; 3];
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    run
}

fn full_combat(run: &mut Run) -> &mut Combat {
    run.state_mut_for_test().combat.as_mut().expect("complete Run has active combat")
}

fn public(run: &Run) -> crate::combat::CombatPublic {
    run.state().combat.as_ref().unwrap().public_full().unwrap()
}

fn set_energy(run: &mut Run, energy: i32) {
    crate::combat::tests::m27_state::set_player_energy(full_combat(run), energy);
}

fn create(run: &mut Run, id: CardId) {
    full_combat(run).enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::CreateCard { id, destination: CardDestination::Hand, upgrades: 0 } }], None).unwrap();
}

fn full_power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: combat::PowerState { amount, skip_first: false,
        monster_source_grace: false } }
}

#[test]
fn m27_engine_3_strange_spoon_rolls_once_in_played_cleanup() {
    let mut run = run_with(&[RelicId::StrangeSpoon], &[CardId::Impervious; 12]);
    let instance = public(&run).hand.iter().find(|card| card.id == CardId::Impervious).unwrap().instance.0;
    let (bit, expected_next) = {
        let state = &full_combat(&mut run).state;
        let stream = state.card_rng.as_ref().unwrap();
        combat::rng::uniform_below(stream, 2).unwrap()
    };
    set_energy(&mut run, 3);
    run.choose_combat(CombatAction::PlayCard { instance, target: None }).unwrap();
    let state = &full_combat(&mut run).state;
    assert_eq!(state.card_rng, Some(expected_next), "eligible cleanup consumes one card-RNG draw");
    let pile = if bit == 0 { &state.piles.discard } else { &state.piles.exhaust };
    assert!(pile.iter().any(|card| card.handle().as_str() == instance.to_string()), "the roll selects its pile");
    assert_eq!(state.hook_events.iter().filter(|event| event.hook == Hook::CardExhausted).count(),
        usize::from(bit != 0), "a saved card skips exhaust callbacks");
}

#[test]
fn m27_engine_3_toolbox_and_codex_keep_their_offer_destinations() {
    let mut toolbox = run_with(&[RelicId::Toolbox], &[CardId::Defend; 12]);
    let chosen = public(&toolbox).card_offer.expect("Toolbox opening offer").cards[0];
    let old_ids = full_combat(&mut toolbox).state.card_ids_issued.clone();
    toolbox.choose_combat(CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    let state = &full_combat(&mut toolbox).state;
    let added = state.card_ids_issued.difference(&old_ids).cloned().collect::<Vec<_>>();
    assert_eq!(added.len(), 1);
    assert!(state.piles.hand.iter().any(|card| card.handle() == added[0] && card.full().unwrap().id == chosen),
        "Toolbox delivers its chosen colorless card to hand");

    let mut codex = run_with(&[RelicId::NilrysCodex], &[CardId::Defend; 12]);
    let old_ids = full_combat(&mut codex).state.card_ids_issued.clone();
    codex.choose_combat(CombatAction::EndTurn).unwrap();
    let chosen = public(&codex).card_offer.expect("Nilry's Codex end-turn offer").cards[0];
    codex.choose_combat(CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    let state = &full_combat(&mut codex).state;
    let added = state.card_ids_issued.difference(&old_ids).cloned().collect::<Vec<_>>();
    assert_eq!(added.len(), 1);
    assert!(state.piles.draw.iter().chain(&state.piles.hand).any(|card|
        card.handle() == added[0] && card.full().unwrap().id == chosen),
        "Nilry's Codex delivers its choice through the shuffled draw pile");
    assert!(!state.piles.discard.iter().any(|card| card.handle() == added[0]));
}

#[test]
fn m27_engine_3_gambling_chip_discards_manually_then_draws() {
    let mut run = run_with(&[RelicId::GamblingChip], &[CardId::Defend; 12]);
    let state = &full_combat(&mut run).state;
    let selection = state.pending_selection.as_ref().expect("opening discard selection");
    assert_eq!(selection.parent, combat::selection::SelectionParent::Relic(RelicId::GamblingChip));
    assert_eq!((selection.minimum, selection.maximum), (0, 5));
    let instance = public(&run).hand[0].instance.0;
    run.choose_combat(CombatAction::SelectCard { instance }).unwrap();
    run.choose_combat(CombatAction::ConfirmSelection).unwrap();
    let state = &full_combat(&mut run).state;
    assert_eq!(state.piles.hand.len(), 5);
    assert_eq!(state.piles.discard.len(), 1);
    // Main-2 integration: the manual-discard hook is raised only when an owned relic consumes it
    // (Tough Bandages / Tingsha / Hovering Kite); with Gambling Chip alone it stays absent (inert).
    assert!(!state.hook_events.iter().any(|event| event.hook == Hook::CardManuallyDiscarded),
        "no manual-discard consumer is owned, so no callback is raised");
    assert_eq!(state.item_context.as_ref().unwrap().local(RelicId::GamblingChip), 1);
}

#[test]
fn m27_engine_3_snecko_eye_adds_two_opening_and_recurring_draws() {
    let run = run_with(&[RelicId::SneckoEye], &[CardId::Defend; 12]);
    let state = &run.state().combat.as_ref().unwrap().state;
    assert_eq!(state.player.draw_per_turn, 7);
    assert_eq!(state.piles.hand.len(), 7, "the first opening hand uses the same +2 bonus");
}

#[test]
fn m27_engine_3_runic_pyramid_retains_non_ethereal_hand() {
    let mut run = run_with(&[RelicId::RunicPyramid], &[CardId::Defend; 12]);
    let kept = public(&run).hand.iter().map(|card| card.instance).collect::<Vec<_>>();
    create(&mut run, CardId::Dazed);
    run.choose_combat(CombatAction::EndTurn).unwrap();
    let public = public(&run);
    assert!(kept.iter().all(|instance| public.hand.iter().any(|card| card.instance == *instance)));
    assert!(public.exhaust.iter().any(|card| card.id == CardId::Dazed));
    assert!(public.discard.is_empty());
}

#[test]
fn m27_engine_3_necronomicon_counts_x_and_queues_after_power_duplicates() {
    fn whirlwind_damage(relics: &[RelicId]) -> i32 {
        let mut run = run_with(relics, &[CardId::Whirlwind; 12]);
        set_energy(&mut run, 3);
        let before = public(&run).enemies[0].hp;
        let instance = public(&run).hand.iter().find(|card| card.id == CardId::Whirlwind).unwrap().instance.0;
        run.choose_combat(CombatAction::PlayCard { instance, target: None }).unwrap();
        before - public(&run).enemies[0].hp
    }
    assert_eq!(whirlwind_damage(&[RelicId::Necronomicon]), 2 * whirlwind_damage(&[]),
        "X pays through energy_on_use and meets the free repeat threshold");

    let mut run = run_with(&[RelicId::Necronomicon], &[CardId::Bash; 12]);
    let combat = full_combat(&mut run);
    let state = &mut combat.state;
    let index = state.piles.hand.iter().position(|card|card.full().is_some_and(|live|live.id==CardId::Bash)).unwrap();
    let card = state.piles.hand.remove(index);
    let owner = card.handle();
    let target = Actor::Enemy(state.enemies[0].instance_id.clone());
    state.in_play = Some(card);
    state.phase = combat::Phase::PlayerAction;
    state.play_context = Some(combat::model::PlayContext { card: owner.clone(), target: Some(target.clone()),
        energy_on_use: 2, paid: 2, disposition: combat::model::FinishDisposition::Natural });
    state.player_full_powers.extend([full_power("double_tap", 2), full_power("echo_form", 1)]);
    state.pending.clear();
    state.pending.push_back(Effect { source: Source::Card(owner.clone()), target: None, kind: EffectKind::FinishCard });
    let hook = Effect { source: Source::Card(owner), target: Some(target), kind: EffectKind::Hook(Hook::CardPlayed) };
    combat::effects::apply_effect(state, &hook).unwrap();
    let finish = state.pending.iter().position(|effect|effect.kind==EffectKind::FinishCard).unwrap();
    let queued = state.pending.iter().skip(finish+1).map(|effect|effect.kind.clone()).collect::<Vec<_>>();
    assert_eq!(queued.len(), 3);
    let EffectKind::Defect(combat::cards::DefectEffect::Repeat { child: echo, .. }) = &queued[0] else { panic!("Echo copy queues first") };
    assert!(matches!(queued[1], EffectKind::Ironclad(combat::cards::IroncladEffect::DoubleTapRepeat { .. })));
    let EffectKind::Defect(combat::cards::DefectEffect::Repeat { child: relic, .. }) = &queued[2] else { panic!("Necronomicon repeat queues after powers") };
    assert!(echo.as_str().parse::<u64>().unwrap() > relic.as_str().parse::<u64>().unwrap(),
        "later power insertion is at the front of the post-finish queue");
}
