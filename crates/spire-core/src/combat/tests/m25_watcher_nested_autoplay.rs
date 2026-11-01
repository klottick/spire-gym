//! Nested queued-card continuations retain one source owner until all work drains.
use super::super::*;
use super::full_runtime::entered_combat;
use crate::{
    content::{CardId, Character, EncounterId, PotionId},
    items::CombatItemContext,
    run::RunRng,
};
use model::OwnedCard;

fn watcher_fight(deck: &[(CardId, u32)]) -> Combat {
    let mut entry = enemies::setup(
        EncounterId::CorruptHeart,
        4,
        0,
        &mut RunRng::new([0xa1; 32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 9, max_energy: 9, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![],
            character: Character::Watcher,
            cards: vec![permanent_live_card(CardId::StrikePurple, CardInstanceId(90), 0)],
            opening_draw: 1,
            encounter_visit: 0,
        },
    ).expect("qualified Heart setup");
    entry.setup.player.energy = 9;
    entry.setup.player.max_energy = 9;
    entry.cards = deck.iter().enumerate().map(|(i, (id, upgrades))|
        m25_watcher_live_card(*id, CardInstanceId(i as u64 + 1), *upgrades).unwrap()).collect();
    entry.opening_draw = deck.len();
    let mut inventory = Some(CombatItemContext::new(vec![], vec![None; 3]).with_m25_profile());
    let combat = Combat::enter_with_item_context(entry, &mut inventory).unwrap();
    assert!(inventory.is_none());
    combat
}

fn ironclad_with_duplication_potion(deck: &[CardId]) -> Combat {
    let mut entry = enemies::setup(
        EncounterId::Cultist,
        1,
        0,
        &mut RunRng::new([0xa2; 32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 9, max_energy: 9, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![],
            character: Character::Ironclad,
            cards: vec![permanent_live_card(CardId::Strike, CardInstanceId(90), 0)],
            opening_draw: 1,
            encounter_visit: 0,
        },
    ).expect("qualified Cultist setup");
    entry.setup.player.energy = 9;
    entry.setup.player.max_energy = 9;
    entry.cards = deck.iter().enumerate().map(|(i, id)| permanent_live_card(*id, CardInstanceId(i as u64 + 1), 0)).collect();
    entry.opening_draw = deck.len();
    let mut inventory = Some(CombatItemContext::new(vec![], vec![Some(PotionId::DuplicationPotion), None, None]).with_m15_profile());
    let combat = Combat::enter_with_item_context(entry, &mut inventory).unwrap();
    assert!(inventory.is_none());
    combat
}

fn move_card(combat: &mut Combat, instance: u64, destination: SelectionSourceZone) {
    let handle = card_state::handle(CardInstanceId(instance));
    let take = |pile: &mut Vec<OwnedCard>| pile.iter().position(|card| card.handle() == handle).map(|index| pile.remove(index));
    let card = take(&mut combat.state.piles.hand)
        .or_else(|| take(&mut combat.state.piles.draw))
        .or_else(|| take(&mut combat.state.piles.discard))
        .or_else(|| take(&mut combat.state.piles.exhaust))
        .expect("fixture card ownership");
    match destination {
        SelectionSourceZone::Hand => combat.state.piles.hand.push(card),
        SelectionSourceZone::Draw => combat.state.piles.draw.push(card),
        SelectionSourceZone::Discard => combat.state.piles.discard.push(card),
        _ => unreachable!("only visible combat piles are arranged"),
    }
    piles::refresh_hand_values(&mut combat.state);
}

fn add_card(combat: &mut Combat, id: CardId, instance: u64, destination: SelectionSourceZone) {
    let owned = OwnedCard::Full(permanent_live_card(id, CardInstanceId(instance), 0));
    assert!(combat.state.card_ids_issued.insert(owned.handle()));
    match destination {
        SelectionSourceZone::Hand => combat.state.piles.hand.push(owned),
        SelectionSourceZone::Draw => combat.state.piles.draw.push(owned),
        _ => unreachable!("fixture additions use hand or draw"),
    }
    piles::refresh_hand_values(&mut combat.state);
}

fn play(combat: &mut Combat, instance: u64, target: Option<u64>) {
    combat.step_full(&CombatAction::PlayCard { instance, target: target.map(TargetId) })
        .unwrap_or_else(|error| panic!("legal play {instance} failed: {error:?}; fault={:?}", combat.fault_record()));
}

fn card_played_count(combat: &Combat) -> usize {
    combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardPlayed).count()
}

#[test]
fn m25_nested_omniscience_chain_completes() {
    // OMNISCIENCE plays each chosen card twice free and exhausts the chosen originals
    // (implementation-notes.json:76-78).
    let mut combat = watcher_fight(&[
        (CardId::Omniscience, 0), (CardId::Omniscience, 0),
        (CardId::StrikePurple, 0), (CardId::DefendPurple, 0),
    ]);
    for instance in 2..=4 { move_card(&mut combat, instance, SelectionSourceZone::Draw); }
    let enemy_hp = combat.state.enemies[0].hp;

    play(&mut combat, 1, None);
    combat.step_full(&CombatAction::SelectCard { instance: 2 }).expect("first Omniscience selects its Omniscience child");
    combat.step_full(&CombatAction::SelectCard { instance: 3 }).expect("nested Omniscience selects Strike");
    combat.step_full(&CombatAction::SelectCard { instance: 4 }).expect("the second source continuation selects Defend");

    assert!(combat.fault_record().is_none(), "nested Omniscience fault: {:?}", combat.fault_record());
    assert_eq!(enemy_hp - combat.state.enemies[0].hp, 12, "the selected Strike is played twice");
    assert_eq!(combat.state.player.block, 8, "the repeated Omniscience selects and plays Defend twice");
    assert!(combat.state.piles.exhaust.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(1))));
    assert!(combat.state.piles.exhaust.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(2))));
    assert_eq!(card_played_count(&combat), 7, "parent, nested Omniscience, and both chosen-card pairs each play");
    assert!(combat.state.play_frames.is_empty());
    engine::validate_runtime(&combat.state).unwrap();
}

#[test]
fn m25_burst_havoc_keeps_both_source_continuations() {
    let mut combat = entered_combat(12, 12, |card| {
        *card = permanent_live_card(CardId::Burst, card.instance, 0);
    });
    combat.state.character = characters::CharacterState::for_character(Character::Silent);
    combat.state.enemies[0].hp = 500;
    combat.state.enemies[0].max_hp = 500;
    add_card(&mut combat, CardId::Havoc, 100, SelectionSourceZone::Hand);
    add_card(&mut combat, CardId::Strike, 101, SelectionSourceZone::Draw);
    add_card(&mut combat, CardId::Strike, 102, SelectionSourceZone::Draw);

    play(&mut combat, 3, None);
    let enemy_hp = combat.state.enemies[0].hp;
    play(&mut combat, 100, None);

    assert!(combat.fault_record().is_none(), "Burst+Havoc fault: {:?}", combat.fault_record());
    assert_eq!(enemy_hp - combat.state.enemies[0].hp, 12, "both Havoc uses play their actual top-card Strike");
    assert_eq!(combat.state.silent.played_this_turn.iter().filter(|id| **id == CardId::Havoc).count(), 2);
    assert_eq!(card_played_count(&combat), 5);
    assert!(combat.state.play_frames.is_empty());
    engine::validate_runtime(&combat.state).unwrap();
}

#[test]
fn m25_double_tap_duplication_keep_both_repeat_owners() {
    let mut combat = ironclad_with_duplication_potion(&[CardId::DoubleTap, CardId::Strike]);
    move_card(&mut combat, 1, SelectionSourceZone::Hand);
    move_card(&mut combat, 2, SelectionSourceZone::Hand);
    play(&mut combat, 1, None);
    combat.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    let enemy_hp = combat.state.enemies[0].hp;

    play(&mut combat, 2, Some(1));

    assert!(combat.fault_record().is_none(), "Double Tap+Duplication Potion fault: {:?}", combat.fault_record());
    assert_eq!(enemy_hp - combat.state.enemies[0].hp, 18, "original Strike, Double Tap repeat, and potion repeat each deal six");
    assert_eq!(card_played_count(&combat), 4, "Double Tap and three Strike instances each fire CardPlayed");
    assert!(combat.state.play_frames.is_empty());
    engine::validate_runtime(&combat.state).unwrap();
}

#[test]
fn m25_duplication_potion_havoc_keeps_both_source_continuations() {
    let mut combat = ironclad_with_duplication_potion(&[CardId::Havoc, CardId::Strike, CardId::Strike]);
    move_card(&mut combat, 1, SelectionSourceZone::Hand);
    move_card(&mut combat, 2, SelectionSourceZone::Draw);
    move_card(&mut combat, 3, SelectionSourceZone::Draw);
    combat.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    let enemy_hp = combat.state.enemies[0].hp;

    play(&mut combat, 1, None);

    assert!(combat.fault_record().is_none(), "Duplication Potion+Havoc fault: {:?}", combat.fault_record());
    assert_eq!(enemy_hp - combat.state.enemies[0].hp, 12, "both Havoc uses play their actual top-card Strike");
    assert_eq!(card_played_count(&combat), 4, "two Havoc uses each play one Strike child");
    assert!(combat.state.play_frames.is_empty());
    engine::validate_runtime(&combat.state).unwrap();
}
