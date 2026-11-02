//! M27 plan Task R2, lane R2, key m27_relics_play.
//! Complete-admission combat tests for play, damage and HP-loss relics.

use crate::combat::{Actor, CardDestination, CombatAction, DamagePayload, Effect, EffectKind, FullPowerId,
    Source, TargetId};
use crate::content::{CardId, CardType, Character, EncounterId, PotionId, RelicId};
use crate::items::{
    dispatch::{ItemPhase, PhaseCard, PhaseFacts},
    laws, CombatItemContext, RelicHookWork,
};
use crate::run::{Run, RunError};
use super::m27_support;
use std::collections::BTreeMap;

fn run_with(relic: RelicId, encounter: EncounterId) -> Run {
    m27_support::complete_run(Character::Ironclad, vec![relic], vec![None; 3],
        BTreeMap::new(), encounter)
}

/// Re-enter a real complete-admission Run with a small authored test deck. The first combat is
/// settled through Run, then the run-owned deck is replaced before the next complete combat.
fn run_with_deck(relic: RelicId, counters: BTreeMap<RelicId, i32>, _encounter: EncounterId,
    cards: &[CardId]) -> Run {
    let mut run = m27_support::complete_run(Character::Ironclad, vec![relic],
        vec![Some(PotionId::FirePotion); 3], counters, EncounterId::Cultist);
    for slot in 0..3 {
        if run.state().combat.is_none() { break; }
        run.choose_combat(CombatAction::UsePotion { slot, target: Some(TargetId(1)) })
            .unwrap_or_else(|error| panic!("settle fixture combat with Fire Potion {slot}: {error:?}"));
    }
    assert!(run.state().combat.is_none(), "three fixture Fire Potions settle the first combat");
    let deck = cards.iter().enumerate().map(|(index, id)| crate::run::CardInstance {
        instance: 100 + index as u64, id: *id, upgrades: 0, misc: 0,
    }).collect();
    run.state_mut_for_test().deck = deck;
    next_combat(&mut run, EncounterId::Cultist);
    let enemy_count = view(&run).enemies.len();
    let protect_from_attack_deck = cards.len() >= 10 && cards.iter().all(|id|
        crate::content::Catalog::standard().card(*id).is_some_and(|definition|definition.card_type==CardType::Attack));
    for slot in 0..enemy_count {
        if protect_from_attack_deck {
            crate::combat::tests::m27_state::set_enemy_block(
                run.state_mut_for_test().combat.as_mut().expect("live complete combat"), slot, 100);
        }
    }
    run
}

fn attack_deck() -> Vec<CardId> { vec![CardId::Strike; 12] }

fn view(run: &Run) -> crate::combat::CombatPublic {
    run.state().combat.as_ref().expect("live complete combat").public_full().unwrap()
}

fn item_context(run: &Run) -> &CombatItemContext {
    run.state().combat.as_ref().expect("live complete combat").item_context().unwrap()
}

fn item_local(run: &Run, relic: RelicId) -> i32 { item_context(run).local(relic) }
fn item_counter(run: &Run, relic: RelicId) -> i32 { item_context(run).counter(relic) }

fn power(run: &Run, id: &str) -> i32 {
    view(run).player.powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.amount)
}

fn create_card(run: &mut Run, id: CardId) {
    run.state_mut_for_test().combat.as_mut().expect("live complete combat")
        .enqueue_native_effects(vec![Effect {
            source: Source::Actor(Actor::Player),
            target: Some(Actor::Player),
            kind: EffectKind::CreateCard { id, destination: CardDestination::Hand, upgrades: 0 },
        }], None).unwrap_or_else(|error| panic!("create {id:?}: {error:?}"));
}

fn play_instance(run: &mut Run, instance: u64, target: Option<TargetId>) {
    run.choose_combat(CombatAction::PlayCard { instance, target })
        .unwrap_or_else(|error| panic!("play card instance {instance}: {error:?}; hand={:?}", view(run).hand));
}

fn ensure_card_in_hand(run: &mut Run, id: CardId) {
    for turn in 0..12 {
        if view(run).hand.iter().any(|card| card.id == id) { return; }
        run.choose_combat(CombatAction::EndTurn)
            .unwrap_or_else(|error| panic!("advance to {id:?} draw: {error:?}"));
        set_energy(run, 100);
        assert!(run.state().combat.is_some(), "combat ended before drawing {id:?} (cycle {turn})");
    }
    panic!("{id:?} was not drawn after 12 turns: {:?}", view(run).hand);
}

fn play_card(run: &mut Run, id: CardId, target: Option<TargetId>) {
    ensure_card_in_hand(run, id);
    let instance = view(run).hand.iter().find(|card| card.id == id).unwrap().instance.0;
    play_instance(run, instance, target);
}

fn play_new_card(run: &mut Run, id: CardId, target: Option<TargetId>) {
    play_card(run, id, target);
}

fn play_n(run: &mut Run, id: CardId, target: Option<TargetId>, count: usize) {
    for _ in 0..count { play_new_card(run, id, target); }
}

fn set_energy(run: &mut Run, energy: i32) {
    crate::combat::tests::m27_state::set_player_energy(
        run.state_mut_for_test().combat.as_mut().expect("live complete combat"), energy);
}

fn enqueue(run: &mut Run, effect: Effect) {
    run.state_mut_for_test().combat.as_mut().expect("live complete combat")
        .enqueue_native_effects(vec![effect], None).unwrap_or_else(|error| panic!("enqueue: {error:?}"));
}

fn lose_player_hp(run: &mut Run, amount: i32) {
    enqueue(run, Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::HpLoss(amount)) });
}

fn player_damage(run: &mut Run, target: crate::combat::EnemyHandle, amount: i32) {
    enqueue(run, Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(target)),
        kind: EffectKind::Damage(DamagePayload::Fixed(amount)) });
}

fn apply_player_power(run: &mut Run, id: &'static str, amount: i32) {
    enqueue(run, Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId(id), amount } });
}

fn defeat_remaining_enemies(run: &mut Run) {
    while run.state().combat.is_some() {
        let public = view(run);
        let Some((slot, enemy)) = public.enemies.iter().enumerate().find(|(_, enemy)| enemy.hp > 0) else { break };
        crate::combat::tests::m27_state::set_enemy_block(
            run.state_mut_for_test().combat.as_mut().expect("live complete combat"), slot, 0);
        crate::combat::tests::m27_state::set_enemy_hp(
            run.state_mut_for_test().combat.as_mut().expect("live complete combat"), slot, 6);
        let attack = public.hand.iter().find(|card|
            crate::content::Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==CardType::Attack));
        let Some(attack) = attack else {
            run.choose_combat(CombatAction::EndTurn).unwrap_or_else(|error|panic!("find fixture Attack: {error:?}"));
            set_energy(run, 100);
            continue;
        };
        let _ = enemy;
        set_energy(run, 100);
        run.choose_combat(CombatAction::PlayCard { instance: attack.instance.0,
            target: Some(TargetId(slot as u64 + 1)) })
            .unwrap_or_else(|error| panic!("settle fixture combat with Attack: {error:?}"));
    }
}

fn next_combat(run: &mut Run, encounter: EncounterId) {
    m27_support::next_complete_combat(run, encounter)
        .unwrap_or_else(|error| panic!("next complete combat: {error:?}; progress={:?}, active={}, pending={}, last={:?}",
            run.state().combat_progress, run.state().active_combat.is_some(), run.state().pending.is_some(), run.state().last_combat));
}

fn phase_work(run: &Run, phase: ItemPhase, facts: &PhaseFacts<'_>) -> RelicHookWork {
    let mut context = item_context(run).clone();
    crate::items::dispatch::combat_phase(&mut context, phase, facts)
        .unwrap_or_else(|error| panic!("dispatch {phase:?}: {error:?}"))
}

fn phase_facts<'a>(enemies: &'a [crate::combat::EnemyHandle], card: Option<PhaseCard>) -> PhaseFacts<'a> {
    let mut facts = PhaseFacts::new(None, 40, 80, 0);
    facts.enemies = enemies;
    facts.live_enemies = enemies;
    facts.card = card;
    facts
}

fn enemy_handles(run: &Run) -> Vec<crate::combat::EnemyHandle> {
    view(run).enemies.into_iter().map(|enemy| enemy.instance_id).collect()
}

fn card(id: CardId, card_type: CardType) -> PhaseCard { PhaseCard { id, card_type, cost_paid: 1 } }

#[test]
fn m27_the_boot_four_damage_minimum() {
    let run = run_with(RelicId::TheBoot, EncounterId::GremlinNob);
    let target = Actor::Enemy(view(&run).enemies[0].instance_id.clone());
    let context = item_context(&run);
    for damage in 1..=4 {
        assert_eq!(laws::unblocked_outgoing_damage(context, &target, damage), 5,
            "positive unblocked Attack damage {damage} becomes 5");
    }
}

#[test]
fn m27_the_boot_boundary() {
    let run = run_with(RelicId::TheBoot, EncounterId::GremlinNob);
    let target = Actor::Enemy(view(&run).enemies[0].instance_id.clone());
    let context = item_context(&run);
    assert_eq!(laws::unblocked_outgoing_damage(context, &target, 0), 0);
    assert_eq!(laws::unblocked_outgoing_damage(context, &target, 4), 5);
}

#[test]
fn m27_centennial_puzzle_first_hp_loss_draw() {
    let mut run = run_with_deck(RelicId::CentennialPuzzle, BTreeMap::new(), EncounterId::GremlinNob,
        &attack_deck());
    assert_eq!(item_local(&run, RelicId::CentennialPuzzle), 0);
    let draw_before = view(&run).hand.len();
    lose_player_hp(&mut run, 1);
    assert_eq!(item_local(&run, RelicId::CentennialPuzzle), 1, "the first positive HP loss latches the relic");
    assert_eq!(view(&run).hand.len(), draw_before + 3, "the first positive HP loss draws three cards");
    let hand_after_first = view(&run).hand.len();
    lose_player_hp(&mut run, 1);
    assert_eq!(view(&run).hand.len(), hand_after_first, "subsequent HP loss does not draw again");
}

#[test]
fn m27_centennial_puzzle_boundary() {
    let run = run_with(RelicId::CentennialPuzzle, EncounterId::GremlinNob);
    let enemies = enemy_handles(&run);
    let mut facts = phase_facts(&enemies, None);
    facts.hp_lost = 0;
    let work = phase_work(&run, ItemPhase::HpLost, &facts);
    assert!(work.top.is_empty() && work.bottom.is_empty(), "zero actual HP loss leaves the latch unused");
    assert_eq!(item_local(&run, RelicId::CentennialPuzzle), 0);
}

#[test]
fn m27_centennial_puzzle_two_combat_counter() {
    let mut run = run_with_deck(RelicId::CentennialPuzzle, BTreeMap::new(), EncounterId::GremlinNob,
        &attack_deck());
    lose_player_hp(&mut run, 1);
    assert_eq!(item_local(&run, RelicId::CentennialPuzzle), 1);
    defeat_remaining_enemies(&mut run);
    next_combat(&mut run, EncounterId::Cultist);
    assert_eq!(item_local(&run, RelicId::CentennialPuzzle), 0, "combat-local latch resets on entry");
    lose_player_hp(&mut run, 1);
    assert_eq!(item_local(&run, RelicId::CentennialPuzzle), 1, "combat two gets its own first-HP-loss trigger");
}

#[test]
fn m27_nunchaku_tenth_attack_energy() {
    let cards = [vec![CardId::Strike; 9], vec![CardId::TwinStrike, CardId::Strike]].concat();
    let mut run = run_with_deck(RelicId::Nunchaku, BTreeMap::from([(RelicId::Nunchaku, 0)]),
        EncounterId::GremlinNob, &cards);
    set_energy(&mut run, 100);
    play_n(&mut run, CardId::Strike, Some(TargetId(1)), 9);
    assert_eq!(item_counter(&run, RelicId::Nunchaku), 9);
    ensure_card_in_hand(&mut run, CardId::TwinStrike);
    let before_tenth = view(&run).player.energy;
    play_card(&mut run, CardId::TwinStrike, Some(TargetId(1)));
    assert_eq!(item_counter(&run, RelicId::Nunchaku), 0);
    assert_eq!(view(&run).player.energy, before_tenth, "the 10th Attack pays one energy and gains one");
    ensure_card_in_hand(&mut run, CardId::Strike);
    let before_eleventh = view(&run).player.energy;
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(view(&run).player.energy, before_eleventh - 1, "the 11th Attack grants no energy");
}

#[test]
fn m27_nunchaku_boundary() {
    let mut run = run_with_deck(RelicId::Nunchaku, BTreeMap::new(), EncounterId::GremlinNob,
        &[CardId::Pummel]);
    set_energy(&mut run, 100);
    play_card(&mut run, CardId::Pummel, Some(TargetId(1)));
    assert_eq!(item_counter(&run, RelicId::Nunchaku), 1,
        "a multi-hit Attack counts as one card play");
}

#[test]
fn m27_nunchaku_two_combat_counter() {
    let mut run = run_with_deck(RelicId::Nunchaku, BTreeMap::new(), EncounterId::GremlinNob, &attack_deck());
    set_energy(&mut run, 100);
    play_n(&mut run, CardId::Strike, Some(TargetId(1)), 9);
    assert_eq!(item_counter(&run, RelicId::Nunchaku), 9);
    defeat_remaining_enemies(&mut run);
    assert_eq!(run.state().relic_counters.get(&RelicId::Nunchaku), Some(&9));
    next_combat(&mut run, EncounterId::Cultist);
    let energy = view(&run).player.energy;
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(item_counter(&run, RelicId::Nunchaku), 0);
    assert_eq!(view(&run).player.energy, energy, "combat two's first Attack completes the saved tenth");
    let energy = view(&run).player.energy;
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(view(&run).player.energy, energy - 1);
}

#[test]
fn m27_pen_nib_tenth_attack_doubles() {
    let mut run = run_with_deck(RelicId::PenNib, BTreeMap::from([(RelicId::PenNib, 9)]),
        EncounterId::GremlinNob, &attack_deck());
    crate::combat::tests::m27_state::set_enemy_block(run.state_mut_for_test().combat.as_mut().unwrap(), 0, 0);
    set_energy(&mut run, 999);
    assert_eq!(power(&run, "pen_nib"), 1, "counter 9 installs the visible Pen Nib power at battle start");
    let before = view(&run).enemies[0].hp;
    play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(before - view(&run).enemies[0].hp, 12, "the 10th 6-damage Attack deals 12");
    assert_eq!(item_counter(&run, RelicId::PenNib), 0);
    assert_eq!(power(&run, "pen_nib"), 0, "the 10th Attack removes the power");
    let before = view(&run).enemies[0].hp;
    play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(before - view(&run).enemies[0].hp, 6, "the 11th Attack is not doubled");
}

#[test]
fn m27_pen_nib_boundary() {
    let mut run = run_with_deck(RelicId::PenNib, BTreeMap::from([(RelicId::PenNib, 9)]),
        EncounterId::GremlinNob, &attack_deck());
    crate::combat::tests::m27_state::set_enemy_block(run.state_mut_for_test().combat.as_mut().unwrap(), 0, 0);
    set_energy(&mut run, 999);
    play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
    let before = view(&run).enemies[0].hp;
    play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(before - view(&run).enemies[0].hp, 6, "only the 10th Attack is doubled");
}

#[test]
fn m27_pen_nib_two_combat_counter() {
    let mut run = run_with_deck(RelicId::PenNib, BTreeMap::new(), EncounterId::GremlinNob, &attack_deck());
    set_energy(&mut run, 999);
    play_n(&mut run, CardId::Strike, Some(TargetId(1)), 9);
    assert_eq!(item_counter(&run, RelicId::PenNib), 9);
    assert_eq!(power(&run, "pen_nib"), 1, "the ninth Attack applies the visible power");
    defeat_remaining_enemies(&mut run);
    next_combat(&mut run, EncounterId::Cultist);
    set_energy(&mut run, 999);
    crate::combat::tests::m27_state::set_enemy_block(run.state_mut_for_test().combat.as_mut().unwrap(), 0, 0);
    assert_eq!(power(&run, "pen_nib"), 1, "battle start reinstalls one visible power at counter 9");
    let before = view(&run).enemies[0].hp;
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(before - view(&run).enemies[0].hp, 12, "battle start reinstalls the armed 10th Attack");
    assert_eq!(item_counter(&run, RelicId::PenNib), 0);
    assert_eq!(power(&run, "pen_nib"), 0);
    let before = view(&run).enemies[0].hp;
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(before - view(&run).enemies[0].hp, 6);
}

#[test]
fn m27_blue_candle_curse_play_exhaust() {
    let mut run = run_with_deck(RelicId::BlueCandle, BTreeMap::new(), EncounterId::GremlinNob,
        &[CardId::Clumsy]);
    set_energy(&mut run, 999);
    crate::combat::tests::m27_state::set_player_hp(
        run.state_mut_for_test().combat.as_mut().unwrap(), 3);
    play_card(&mut run, CardId::Clumsy, None);
    assert_eq!(view(&run).player.hp, 2, "playing the Curse costs 1 HP");
    assert!(view(&run).exhaust.iter().any(|card| card.id == CardId::Clumsy),
        "Blue Candle exhausts the Curse after use");
}

#[test]
fn m27_blue_candle_boundary() {
    let run = run_with(RelicId::BlueCandle, EncounterId::GremlinNob);
    let context = item_context(&run);
    assert_eq!(laws::play_allowed(context, card(CardId::Clumsy, CardType::Curse)).allow, Some(true));
    assert_eq!(laws::play_allowed(context, card(CardId::Dazed, CardType::Status)).allow, None,
        "Blue Candle does not authorize a non-Curse Status");
}

#[test]
fn m27_gremlin_horn_nonfinal_kill_payoff() {
    let mut run = run_with(RelicId::GremlinHorn, EncounterId::ThreeCultists);
    let before_energy = view(&run).player.energy;
    let enemy = view(&run).enemies[0].instance_id.clone();
    player_damage(&mut run, enemy, 30_000);
    assert_eq!(view(&run).player.energy, before_energy + 1, "a non-final enemy death grants 1 Energy");
    let enemies = enemy_handles(&run);
    let mut facts = phase_facts(&enemies, None);
    facts.dead_enemy = Some((&enemies[0], None, 2));
    let work = phase_work(&run, ItemPhase::EnemyDied, &facts);
    assert!(work.bottom.iter().any(|effect| effect.kind == EffectKind::GainEnergy(1)));
    assert!(work.bottom.iter().any(|effect| effect.kind == EffectKind::Draw(1)));
}

#[test]
fn m27_gremlin_horn_boundary() {
    let run = run_with(RelicId::GremlinHorn, EncounterId::GremlinNob);
    let enemies = enemy_handles(&run);
    let mut facts = phase_facts(&enemies, None);
    facts.dead_enemy = Some((&enemies[0], None, 0));
    let work = phase_work(&run, ItemPhase::EnemyDied, &facts);
    assert!(work.bottom.is_empty(), "the final group death grants no energy or draw");
}

#[test]
fn m27_ink_bottle_tenth_card_draw() {
    let mut run = run_with_deck(RelicId::InkBottle, BTreeMap::from([(RelicId::InkBottle, 0)]),
        EncounterId::GremlinNob, &attack_deck());
    set_energy(&mut run, 999);
    play_n(&mut run, CardId::Strike, Some(TargetId(1)), 9);
    assert_eq!(item_counter(&run, RelicId::InkBottle), 9);
    let hand_before = view(&run).hand.clone();
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(item_counter(&run, RelicId::InkBottle), 0);
    assert_eq!(view(&run).hand.len(), hand_before.len(), "the 10th card use draws 1 after replacing the played card");
    assert!(view(&run).hand.iter().any(|card| !hand_before.iter().any(|old| old.instance == card.instance)),
        "a new hand instance replaces the played card");
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(item_counter(&run, RelicId::InkBottle), 1);
    assert!(view(&run).hand.is_empty(), "the 11th card use draws 0");
}

#[test]
fn m27_ink_bottle_boundary() {
    let mut run = run_with_deck(RelicId::InkBottle, BTreeMap::from([(RelicId::InkBottle, 9)]),
        EncounterId::GremlinNob, &[CardId::Pummel]);
    set_energy(&mut run, 999);
    play_card(&mut run, CardId::Pummel, Some(TargetId(1)));
    assert_eq!(item_counter(&run, RelicId::InkBottle), 0,
        "one multi-hit card use advances the persistent counter once");
}

#[test]
fn m27_ink_bottle_two_combat_counter() {
    let mut run = run_with_deck(RelicId::InkBottle, BTreeMap::new(), EncounterId::GremlinNob, &attack_deck());
    set_energy(&mut run, 999);
    play_n(&mut run, CardId::Strike, Some(TargetId(1)), 9);
    assert_eq!(item_counter(&run, RelicId::InkBottle), 9);
    defeat_remaining_enemies(&mut run);
    assert_eq!(run.state().relic_counters.get(&RelicId::InkBottle), Some(&9));
    next_combat(&mut run, EncounterId::Cultist);
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    assert_eq!(item_counter(&run, RelicId::InkBottle), 0, "the next combat completes the saved 10th use");
}

macro_rules! third_attack_relic_tests {
    ($first:ident, $boundary:ident, $two_combat:ident, $relic:expr, $power_id:literal, $is_block:expr, $amount:expr) => {
        #[test]
        fn $first() {
            let mut run = run_with_deck($relic, BTreeMap::new(), EncounterId::GremlinNob, &attack_deck());
            set_energy(&mut run, 999);
            enqueue(&mut run, Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
                kind: EffectKind::Draw(5) });
            play_n(&mut run, CardId::Strike, Some(TargetId(1)), 3);
            let first = if $is_block { view(&run).player.block } else { power(&run, $power_id) };
            assert_eq!(first, $amount, "the third Attack triggers the relic once");
            play_n(&mut run, CardId::Strike, Some(TargetId(1)), 3);
            let second = if $is_block { view(&run).player.block } else { power(&run, $power_id) };
            assert_eq!(second, 2 * $amount, "the sixth Attack triggers a second time");
        }

        #[test]
        fn $boundary() {
            let mut run = run_with_deck($relic, BTreeMap::new(), EncounterId::GremlinNob, &attack_deck());
            set_energy(&mut run, 999);
            for _ in 0..3 {
                play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
                run.choose_combat(CombatAction::EndTurn).unwrap();
            }
            let result = if $is_block { view(&run).player.block } else { power(&run, $power_id) };
            assert_eq!(result, 0, "Attacks in separate turns do not share the local counter");
        }

        #[test]
        fn $two_combat() {
            let mut run = run_with_deck($relic, BTreeMap::new(), EncounterId::GremlinNob, &attack_deck());
            set_energy(&mut run, 999);
            play_n(&mut run, CardId::Strike, Some(TargetId(1)), 2);
            assert_eq!(item_local(&run, $relic), 2);
            defeat_remaining_enemies(&mut run);
            next_combat(&mut run, EncounterId::Cultist);
            assert_eq!(item_local(&run, $relic), 0, "combat-local Attack count resets");
            set_energy(&mut run, 999);
            play_n(&mut run, CardId::Strike, Some(TargetId(1)), 3);
            let result = if $is_block { view(&run).player.block } else { power(&run, $power_id) };
            assert_eq!(result, $amount, "the third Attack in combat two triggers");
        }
    };
}

third_attack_relic_tests!(m27_kunai_third_attack_dexterity, m27_kunai_boundary,
    m27_kunai_two_combat_counter, RelicId::Kunai, "dexterity", false, 1);

#[test]
fn m27_letter_opener_third_skill_thorns() {
    let skills = [vec![CardId::Defend; 6], vec![CardId::Strike]].concat();
    let mut run = run_with_deck(RelicId::LetterOpener, BTreeMap::new(), EncounterId::GremlinNob, &skills);
    set_energy(&mut run, 999);
    let before = view(&run).enemies[0].hp;
    play_n(&mut run, CardId::Defend, None, 3);
    assert_eq!(before - view(&run).enemies[0].hp, 5, "the third Skill deals all-enemy Thorns 5");
    assert_eq!(item_local(&run, RelicId::LetterOpener), 0, "the third Skill resets the local count");
}

#[test]
fn m27_letter_opener_boundary() {
    let skills = [vec![CardId::Defend; 6], vec![CardId::Strike]].concat();
    let mut run = run_with_deck(RelicId::LetterOpener, BTreeMap::new(), EncounterId::GremlinNob, &skills);
    set_energy(&mut run, 999);
    apply_player_power(&mut run, "strength", 9);
    let before = view(&run).enemies[0].hp;
    play_n(&mut run, CardId::Defend, None, 3);
    assert_eq!(before - view(&run).enemies[0].hp, 5, "Thorns damage is non-Attack and ignores Strength");
}

#[test]
fn m27_letter_opener_two_combat_counter() {
    let skills = [vec![CardId::Defend; 6], vec![CardId::Strike]].concat();
    let mut run = run_with_deck(RelicId::LetterOpener, BTreeMap::new(), EncounterId::GremlinNob, &skills);
    set_energy(&mut run, 999);
    play_n(&mut run, CardId::Defend, None, 2);
    assert_eq!(item_local(&run, RelicId::LetterOpener), 2);
    defeat_remaining_enemies(&mut run);
    next_combat(&mut run, EncounterId::Cultist);
    assert_eq!(item_local(&run, RelicId::LetterOpener), 0);
    set_energy(&mut run, 999);
    let before = view(&run).enemies[0].hp;
    play_n(&mut run, CardId::Defend, None, 3);
    assert_eq!(before - view(&run).enemies[0].hp, 5);
}

#[test]
fn m27_mummified_hand_power_cost_zero() {
    let mut run = run_with_deck(RelicId::MummifiedHand, BTreeMap::new(), EncounterId::GremlinNob,
        &[CardId::Strike, CardId::Inflame]);
    set_energy(&mut run, 999);
    play_card(&mut run, CardId::Inflame, None);
    assert_eq!(view(&run).hand.iter().find(|card| card.id == CardId::Strike).unwrap().effective_cost,
        Some(0), "Power use discounts the sole eligible hand card immediately");
}

#[test]
fn m27_mummified_hand_boundary() {
    let mut run = run_with_deck(RelicId::MummifiedHand, BTreeMap::new(), EncounterId::GremlinNob,
        &[CardId::Wound, CardId::Inflame]);
    set_energy(&mut run, 999);
    play_card(&mut run, CardId::Inflame, None);
    assert!(view(&run).hand.iter().any(|card| card.id == CardId::Wound && card.effective_cost.is_none()),
        "a Power does not change a Status card when no eligible card is held");
}

third_attack_relic_tests!(m27_ornamental_fan_third_attack_block, m27_ornamental_fan_boundary,
    m27_ornamental_fan_two_combat_counter, RelicId::OrnamentalFan, "", true, 4);

third_attack_relic_tests!(m27_shuriken_third_attack_strength, m27_shuriken_boundary,
    m27_shuriken_two_combat_counter, RelicId::Shuriken, "strength", false, 1);

#[test]
fn m27_strike_dummy_strike_tag_bonus() {
    let run = run_with(RelicId::StrikeDummy, EncounterId::GremlinNob);
    let context = item_context(&run);
    let target = Actor::Enemy(view(&run).enemies[0].instance_id.clone());
    assert_eq!(laws::outgoing_damage(context, Some(card(CardId::Strike, CardType::Attack)), &target, 6), 9);
    assert_eq!(laws::outgoing_damage(context, Some(card(CardId::Bash, CardType::Attack)), &target, 6), 6);
}

#[test]
fn m27_strike_dummy_boundary() {
    let run = run_with(RelicId::StrikeDummy, EncounterId::GremlinNob);
    let context = item_context(&run);
    let target = Actor::Enemy(view(&run).enemies[0].instance_id.clone());
    assert_eq!(laws::outgoing_damage(context, Some(card(CardId::Bash, CardType::Attack)), &target, 6), 6,
        "Attack type and card name alone do not replace the Strike tag");
}

#[test]
fn m27_bird_faced_urn_power_use_heal() {
    let mut run = run_with_deck(RelicId::BirdFacedUrn, BTreeMap::new(), EncounterId::GremlinNob,
        &[CardId::Inflame, CardId::Strike, CardId::Defend]);
    set_energy(&mut run, 999);
    crate::combat::tests::m27_state::set_player_hp(
        run.state_mut_for_test().combat.as_mut().unwrap(), 40);
    play_new_card(&mut run, CardId::Inflame, None);
    assert_eq!(view(&run).player.hp, 42, "a Power use queues a top-side heal of 2");
    let hp = view(&run).player.hp;
    play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
    play_new_card(&mut run, CardId::Defend, None);
    assert_eq!(view(&run).player.hp, hp, "Attack and Skill uses do not heal");
}

#[test]
fn m27_bird_faced_urn_boundary() {
    let mut run = run_with_deck(RelicId::BirdFacedUrn, BTreeMap::new(), EncounterId::GremlinNob,
        &[CardId::Strike, CardId::Defend]);
    set_energy(&mut run, 999);
    crate::combat::tests::m27_state::set_player_hp(
        run.state_mut_for_test().combat.as_mut().unwrap(), 40);
    play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
    play_new_card(&mut run, CardId::Defend, None);
    assert_eq!(view(&run).player.hp, 40, "only a Power use triggers Bird-Faced Urn");
}

#[test]
fn m27_dead_branch_actual_exhaust_card() {
    let mut run = run_with_deck(RelicId::DeadBranch, BTreeMap::new(), EncounterId::GremlinNob,
        &[CardId::Pummel]);
    set_energy(&mut run, 999);
    crate::combat::tests::m27_state::set_enemy_hp(
        run.state_mut_for_test().combat.as_mut().unwrap(), 0, 200);
    let hand_before = view(&run).hand.iter().map(|card|card.instance).collect::<Vec<_>>();
    play_card(&mut run, CardId::Pummel, Some(TargetId(1)));
    assert_eq!(view(&run).exhaust.iter().filter(|card| card.id == CardId::Pummel).count(), 1);
    assert!(view(&run).enemies[0].hp > 0, "the exhaust listener runs while an enemy remains");
    assert_eq!(view(&run).hand.len(), hand_before.len(),
        "one generated card replaces the only played card in this one-card fixture");
    assert!(view(&run).hand.iter().any(|card|!hand_before.contains(&card.instance)),
        "a real exhaust while an enemy remains adds one class-pool card to hand");
}

#[test]
fn m27_dead_branch_boundary() {
    let run = run_with(RelicId::DeadBranch, EncounterId::GremlinNob);
    let enemies = enemy_handles(&run);
    let mut facts = phase_facts(&[], Some(card(CardId::Pummel, CardType::Attack)));
    facts.enemies = &enemies;
    let work = phase_work(&run, ItemPhase::Exhaust, &facts);
    assert!(work.top.is_empty() && work.bottom.is_empty(),
        "a final-group exhaust has no eligible generated-card work");
}

#[test]
fn m27_ginger_weak_prevention() {
    let mut run = run_with(RelicId::Ginger, EncounterId::GremlinNob);
    assert!(!laws::power_attempt(item_context(&run), &Actor::Player, "weak", 1));
    apply_player_power(&mut run, "artifact", 1);
    apply_player_power(&mut run, "weak", 1);
    assert_eq!(power(&run, "weak"), 0);
    assert_eq!(power(&run, "artifact"), 1, "Ginger blocks Weak before Artifact");
}

#[test]
fn m27_ginger_boundary() {
    let mut run = run_with(RelicId::Ginger, EncounterId::GremlinNob);
    apply_player_power(&mut run, "artifact", 1);
    apply_player_power(&mut run, "weak", 1);
    assert_eq!((power(&run, "weak"), power(&run, "artifact")), (0, 1));
}

#[test]
fn m27_torii_two_to_five_damage() {
    let mut run = run_with(RelicId::Torii, EncounterId::GremlinNob);
    let enemy = view(&run).enemies[0].instance_id.clone();
    let hp = view(&run).player.hp;
    enqueue(&mut run, Effect { source: Source::Actor(Actor::Enemy(enemy.clone())), target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::Fixed(3)) });
    assert_eq!(view(&run).player.hp, hp - 1);
    let hp = view(&run).player.hp;
    enqueue(&mut run, Effect { source: Source::Actor(Actor::Enemy(enemy)), target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::Fixed(6)) });
    assert_eq!(view(&run).player.hp, hp - 6);
}

#[test]
fn m27_torii_boundary() {
    let mut run = run_with(RelicId::Torii, EncounterId::GremlinNob);
    let enemy = view(&run).enemies[0].instance_id.clone();
    let hp = view(&run).player.hp;
    enqueue(&mut run, Effect { source: Source::Actor(Actor::Enemy(enemy)), target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::Fixed(6)) });
    assert_eq!(view(&run).player.hp, hp - 6, "6 incoming Attack damage is outside Torii's 2–5 range");
}

#[test]
fn m27_tungsten_rod_positive_loss_minus_one() {
    let mut run = run_with(RelicId::TungstenRod, EncounterId::GremlinNob);
    lose_player_hp(&mut run, 3);
    assert_eq!(view(&run).player.hp, 78, "3 HP loss becomes 2");
    lose_player_hp(&mut run, 1);
    assert_eq!(view(&run).player.hp, 78, "1 HP loss becomes 0");
    lose_player_hp(&mut run, 0);
    assert_eq!(view(&run).player.hp, 78);
}

#[test]
fn m27_tungsten_rod_boundary() {
    let mut run = run_with(RelicId::TungstenRod, EncounterId::GremlinNob);
    lose_player_hp(&mut run, 1);
    lose_player_hp(&mut run, 0);
    assert_eq!(view(&run).player.hp, 80, "HP loss never goes below zero");
}

#[test]
fn m27_turnip_frail_prevention() {
    let mut run = run_with(RelicId::Turnip, EncounterId::GremlinNob);
    assert!(!laws::power_attempt(item_context(&run), &Actor::Player, "frail", 1));
    apply_player_power(&mut run, "artifact", 1);
    apply_player_power(&mut run, "frail", 1);
    assert_eq!(power(&run, "frail"), 0);
    assert_eq!(power(&run, "artifact"), 1, "Turnip blocks Frail before Artifact");
}

#[test]
fn m27_turnip_boundary() {
    let mut run = run_with(RelicId::Turnip, EncounterId::GremlinNob);
    apply_player_power(&mut run, "artifact", 1);
    apply_player_power(&mut run, "frail", 1);
    assert_eq!((power(&run, "frail"), power(&run, "artifact")), (0, 1));
}

#[test]
fn m27_hand_drill_block_break_vulnerable() {
    let mut run = run_with(RelicId::HandDrill, EncounterId::GremlinNob);
    crate::combat::tests::m27_state::set_enemy_block(
        run.state_mut_for_test().combat.as_mut().unwrap(), 0, 5);
    let enemy = view(&run).enemies[0].instance_id.clone();
    player_damage(&mut run, enemy, 5);
    assert_eq!(view(&run).enemies[0].block, 0);
    let vulnerable = view(&run).enemies[0].powers.iter().find(|power| power.id.0 == "vulnerable")
        .map_or(0, |power| power.amount);
    assert_eq!(vulnerable, 2, "exact monster Block depletion applies Vulnerable 2");
}

#[test]
fn m27_hand_drill_boundary() {
    let mut run = run_with(RelicId::HandDrill, EncounterId::GremlinNob);
    crate::combat::tests::m27_state::set_enemy_block(
        run.state_mut_for_test().combat.as_mut().unwrap(), 0, 5);
    let enemy = view(&run).enemies[0].instance_id.clone();
    player_damage(&mut run, enemy, 1);
    assert_eq!(view(&run).enemies[0].block, 4);
    assert_eq!(view(&run).enemies[0].powers.iter().find(|power| power.id.0 == "vulnerable")
        .map_or(0, |power| power.amount), 0, "a hit that leaves Block does not trigger");
}

#[test]
fn m27_medical_kit_status_play_exhaust() {
    let mut run = run_with_deck(RelicId::MedicalKit, BTreeMap::new(), EncounterId::GremlinNob,
        &[CardId::Wound, CardId::Clumsy]);
    set_energy(&mut run, 999);
    play_card(&mut run, CardId::Wound, None);
    assert!(view(&run).exhaust.iter().any(|card| card.id == CardId::Wound),
        "Medical Kit exhausts an otherwise-unplayable Status after use");
    ensure_card_in_hand(&mut run, CardId::Clumsy);
    assert!(matches!(run.choose_combat(CombatAction::PlayCard {
        instance: view(&run).hand.iter().find(|card| card.id == CardId::Clumsy).unwrap().instance.0,
        target: None,
    }), Err(RunError::InvalidAction)), "Medical Kit does not make Curses playable");
}

#[test]
fn m27_medical_kit_boundary() {
    let run = run_with(RelicId::MedicalKit, EncounterId::GremlinNob);
    let context = item_context(&run);
    assert_eq!(laws::play_allowed(context, card(CardId::Wound, CardType::Status)).allow, Some(true));
    assert_eq!(laws::play_allowed(context, card(CardId::Clumsy, CardType::Curse)).allow, None,
        "Medical Kit does not authorize a Curse");
}

#[test]
fn m27_orange_pellets_complete_card_trio() {
    let cards = [vec![CardId::Strike; 4], vec![CardId::Defend; 2], vec![CardId::Inflame]].concat();
    let mut run = run_with_deck(RelicId::OrangePellets, BTreeMap::new(), EncounterId::GremlinNob, &cards);
    set_energy(&mut run, 999);
    apply_player_power(&mut run, "weak", 1);
    play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
    play_new_card(&mut run, CardId::Defend, None);
    play_new_card(&mut run, CardId::Inflame, None);
    assert_eq!(power(&run, "weak"), 0, "Attack, Skill and Power remove player debuffs");
    play_n(&mut run, CardId::Strike, Some(TargetId(1)), 2);
    play_new_card(&mut run, CardId::Defend, None);
    assert_eq!(power(&run, "weak"), 0, "a second complete trio can be processed after its reset");
}

#[test]
fn m27_orange_pellets_boundary() {
    let mut run = run_with_deck(RelicId::OrangePellets, BTreeMap::new(), EncounterId::GremlinNob,
        &[CardId::Strike, CardId::Strike, CardId::Defend]);
    set_energy(&mut run, 999);
    apply_player_power(&mut run, "weak", 1);
    play_n(&mut run, CardId::Strike, Some(TargetId(1)), 2);
    play_new_card(&mut run, CardId::Defend, None);
    assert_eq!(power(&run, "weak"), 1, "duplicate Attack types do not complete the trio");
}

#[test]
fn m27_orange_pellets_two_combat_counter() {
    let cards = [vec![CardId::Strike; 3], vec![CardId::Defend; 2], vec![CardId::Inflame]].concat();
    let mut run = run_with_deck(RelicId::OrangePellets, BTreeMap::new(), EncounterId::GremlinNob, &cards);
    set_energy(&mut run, 999);
    play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
    play_new_card(&mut run, CardId::Defend, None);
    assert_eq!(item_local(&run, RelicId::OrangePellets), 3,
        "Attack and Skill flags persist only inside this combat");
    defeat_remaining_enemies(&mut run);
    next_combat(&mut run, EncounterId::Cultist);
    assert_eq!(item_local(&run, RelicId::OrangePellets), 0);
    set_energy(&mut run, 999);
    apply_player_power(&mut run, "weak", 1);
    play_new_card(&mut run, CardId::Strike, Some(TargetId(1)));
    play_new_card(&mut run, CardId::Defend, None);
    play_new_card(&mut run, CardId::Inflame, None);
    assert_eq!(power(&run, "weak"), 0);
    assert_eq!(item_local(&run, RelicId::OrangePellets), 0, "the completed trio resets immediately");
}

#[test]
fn m27_odd_mushroom_vulnerable_damage() {
    let run = run_with(RelicId::OddMushroom, EncounterId::GremlinNob);
    let context = item_context(&run);
    let factor = laws::player_vulnerable_factor(context);
    assert_eq!(factor, 1.25);
    assert_eq!((4.0_f32 * factor).ceil() as i32, 5, "4 NORMAL Attack damage becomes 5");
    let hp = view(&run).player.hp;
    let mut run = run;
    lose_player_hp(&mut run, 4);
    assert_eq!(view(&run).player.hp, hp - 4, "direct HP_LOSS is unchanged");
}

#[test]
fn m27_odd_mushroom_boundary() {
    let run = run_with(RelicId::OddMushroom, EncounterId::GremlinNob);
    let context = item_context(&run);
    assert_eq!(laws::player_vulnerable_factor(context), 1.25);
    let mut run = run;
    lose_player_hp(&mut run, 4);
    assert_eq!(view(&run).player.hp, 76, "the Vulnerable factor does not affect direct HP_LOSS");
}

#[test]
fn m27_review_m1_dead_branch_precedes_dark_embrace_at_full_hand() {
    let deck = vec![CardId::Pummel; 11];
    let mut run = run_with_deck(RelicId::DeadBranch, BTreeMap::new(), EncounterId::Cultist, &deck);
    run.state_mut_for_test().combat.as_mut().unwrap().enqueue_native_effects(vec![Effect {
        source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("dark_embrace"), amount: 1 },
    }], None).unwrap();
    run.state_mut_for_test().combat.as_mut().unwrap().enqueue_native_effects(vec![Effect {
        source: Source::None, target: Some(Actor::Player), kind: EffectKind::Draw(5),
    }], None).unwrap();
    let before = view(&run);
    assert_eq!(before.hand.len(), 10, "the fixture exhausts one card from a ten-card hand");
    assert!(before.draw_count > 0, "Dark Embrace has a card available to draw");
    let remaining = before.hand.iter().skip(1).map(|card| card.instance).collect::<Vec<_>>();
    let draw_count = before.draw_count as usize;
    set_energy(&mut run, 999);
    play_card(&mut run, CardId::Pummel, Some(TargetId(1)));
    let after = view(&run);
    assert_eq!(after.draw_count as usize, draw_count,
        "Dark Embrace finds the full hand after Dead Branch and leaves the draw pile unchanged");
    assert_eq!(after.hand.len(), 10,
        "Dead Branch puts its generated card into hand before Dark Embrace's draw");
    assert!(after.hand.iter().any(|card| !remaining.contains(&card.instance)),
        "the Dead Branch generated card is in hand");
}
