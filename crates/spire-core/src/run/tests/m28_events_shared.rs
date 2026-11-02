//! M28d shrine and one-time event bodies (plan-check §4; spec batch1): source-shaped branch tests.
use super::super::*;
use super::m28_event_core::{at_event, base, before_event, count, leave, row, view};
use crate::content::{CardId, CardRarity, Character, EventId, PotionId, RelicId};
use crate::run::{CardInstance, GameAction, RoomChoiceKind, RoomChoiceStatus, RoomViewKind, Run, RunError, RunTerminal};

fn event_option(run: &mut Run, stage: u8, option: u8) -> Result<RunTransition, RunError> {
    let id = row(run, &RoomChoiceKind::EventOption { stage, option }).id;
    run.step(&GameAction::Choose(id))
}

fn event_position(run: &mut Run, stage: u8, position: u8) -> RunTransition {
    let choice = view(run).choices.into_iter().find(|choice| matches!(choice.kind,
        RoomChoiceKind::EventCard { stage: s, option, .. } if s == stage && option == position))
        .unwrap_or_else(|| panic!("no Match and Keep position {position} at stage {stage}"));
    run.step(&GameAction::Choose(choice.id)).unwrap()
}

fn selected_instance(run: &mut Run, instance: u64) -> RunTransition {
    let pending = run.state.pending.as_ref().expect("event selection opened");
    let id = pending.selection.candidates.iter().find(|candidate| candidate.card.instance == instance)
        .unwrap_or_else(|| panic!("instance {instance} was not offered: {:?}", pending.selection.candidates)).id;
    run.step(&GameAction::Select(id)).unwrap()
}

fn first_selectable_instance(run: &Run) -> u64 {
    run.state.pending.as_ref().expect("event selection opened").selection.candidates[0].card.instance
}

fn keep_only(run: &mut Run, id: CardId) -> u64 {
    let instance = run.state.deck[0].instance;
    run.state.deck.clear();
    run.state.deck.push(CardInstance { id, instance, upgrades: 0, misc: 0 });
    instance
}

fn fill_deck(run: &mut Run, ids: &[CardId]) {
    let first = run.state.deck[0].instance;
    run.state.deck = ids.iter().enumerate().map(|(i, id)| CardInstance {
        id: *id, instance: first + i as u64, upgrades: 0, misc: 0,
    }).collect();
}

fn frame(run: &Run) -> &crate::run::rooms::standard::event_frame::StandardEventFrame {
    run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap()
}

#[test]
fn m28_match_and_keep_constructs_e11_board() {
    // Match and Keep's literal class-pool sizes and E11 starter identities: spec §Match and Keep,
    // PC §58; research :274, :305-312, :323-327; CD :63-72.
    let cases = [
        (Character::Ironclad, 20, 36, 16, CardId::Bash),
        (Character::Silent, 19, 33, 19, CardId::Neutralize),
        (Character::Defect, 18, 36, 17, CardId::Zap),
        (Character::Watcher, 19, 35, 17, CardId::Eruption),
    ];
    for (character, common, uncommon, rare, starter) in cases {
        let (mut run, node) = before_event(character, 1, EventId::MatchAndKeep, 31);
        run.state.relics.push(RelicId::PrismaticShard);
        run.step(&GameAction::Move(node)).unwrap();
        let ids = &frame(&run).captured.card_ids;
        assert_eq!(ids.len(), 12, "{character:?}");
        assert_eq!(ids.iter().filter(|id| **id == starter).count(), 2, "E11 {character:?}");
        let mut distinct = ids.clone(); distinct.sort(); distinct.dedup();
        assert_eq!(distinct.len(), 6, "six pairs {character:?}");
        for id in &distinct { assert_eq!(ids.iter().filter(|candidate| *candidate == id).count(), 2, "pair {id:?}"); }
        for (rarity, expected) in [(CardRarity::Common, common), (CardRarity::Uncommon, uncommon), (CardRarity::Rare, rare)] {
            let pool = crate::content::pools::cards(crate::content::pools::CardPool::Ordinary { character, rarity: Some(rarity), card_type: None }).unwrap();
            assert_eq!(pool.len(), expected, "{rarity:?} pool size {character:?}");
            assert_eq!(distinct.iter().filter(|id| pool.contains(id)).count(), 1, "one {rarity:?} {character:?}, no Shard widening");
        }
        let colorless = crate::content::pools::cards(crate::content::pools::CardPool::Colourless { rarity: Some(CardRarity::Uncommon) }).unwrap();
        let curses = crate::content::pools::cards(crate::content::pools::CardPool::Curse).unwrap();
        assert_eq!(colorless.len(), 20);
        assert_eq!(curses.len(), 10);
        // Pride has no CardId and is never deck-legal; the exact ordinary support excludes all four.
        for curse in [CardId::Clumsy, CardId::Decay, CardId::Doubt, CardId::Injury, CardId::Normality,
            CardId::Pain, CardId::Parasite, CardId::Regret, CardId::Shame, CardId::Writhe] {
            assert!(curses.contains(&curse), "ordinary curse support includes {curse:?}");
        }
        assert_eq!(distinct.iter().filter(|id| colorless.contains(id)).count(), 1);
        assert_eq!(distinct.iter().filter(|id| curses.contains(id)).count(), 1);
        assert!(ids.iter().all(|id| *id != CardId::AscendersBane && *id != CardId::CurseOfTheBell && *id != CardId::Necronomicurse));
        assert!(view(&run).choices.iter().all(|choice| matches!(choice.kind, RoomChoiceKind::EventCard { card: None, .. })));
    }
}

#[test]
fn m28_match_and_keep_match_obtains_fresh_egg_preview_copy() {
    // A matching pair obtains one fresh copy; the shared obtain owner applies Molten Egg: spec §Match and Keep; ER :274.
    let (mut run, node) = before_event(Character::Ironclad, 1, EventId::MatchAndKeep, 32);
    run.state.relics.push(RelicId::MoltenEgg);
    run.step(&GameAction::Move(node)).unwrap();
    {
        let captured = &mut run.state.room_loop.as_mut().unwrap().standard_event.as_mut().unwrap().captured;
        captured.card_ids[0] = CardId::IronWave;
        captured.card_ids[1] = CardId::IronWave;
    }
    let first = 0_u8;
    let second = 1_u8;
    let id = CardId::IronWave;
    let old_count = count(&run, id);
    event_position(&mut run, 0, first);
    assert_eq!(frame(&run).stage, 0, "first selection stays on the same attempt");
    assert_eq!(frame(&run).captured.values[1], u32::from(first) + 1);
    assert_eq!(frame(&run).captured.values[2], 1 << first);
    event_position(&mut run, 0, second);
    assert_eq!(count(&run, id), old_count + 1, "one copy only");
    assert!(run.state.deck.iter().any(|card| card.id == id && card.upgrades == 1), "Molten Egg applies on fresh obtain");
    assert_eq!(frame(&run).stage, 1);
    assert_ne!(frame(&run).captured.values[0] & (1 << first), 0);
    assert_ne!(frame(&run).captured.values[0] & (1 << second), 0);

    let (mut negated, node) = before_event(Character::Ironclad, 1, EventId::MatchAndKeep, 132);
    negated.state.relics.push(RelicId::Omamori);
    negated.state.relic_counters.insert(RelicId::Omamori, 2);
    negated.step(&GameAction::Move(node)).unwrap();
    {
        let captured = &mut negated.state.room_loop.as_mut().unwrap().standard_event.as_mut().unwrap().captured;
        captured.card_ids[0] = CardId::Clumsy;
        captured.card_ids[1] = CardId::Clumsy;
    }
    let clumsy = count(&negated, CardId::Clumsy);
    event_position(&mut negated, 0, 0);
    event_position(&mut negated, 0, 1);
    assert_eq!(count(&negated, CardId::Clumsy), clumsy, "Omamori prevents the matched curse copy");
    assert_eq!(negated.state.relic_counters.get(&RelicId::Omamori), Some(&1));
}

#[test]
fn m28_match_and_keep_mismatch_reveals_and_stops_after_five_pairs() {
    // Mismatches count, remain remembered under D2, and attempt five reaches DONE: spec §Match and Keep; PC §71-72; ER :274, :464.
    let mut run = at_event(Character::Silent, 1, EventId::MatchAndKeep, 33);
    let ids = frame(&run).captured.card_ids.clone();
    let (first, second) = (0..12).flat_map(|i| (0..12).map(move |j| (i, j)))
        .find(|(i, j)| i != j && ids[*i] != ids[*j]).unwrap();
    for attempt in 0..5_u8 {
        event_position(&mut run, attempt, first as u8);
        assert_eq!(frame(&run).stage, attempt);
        event_position(&mut run, attempt, second as u8);
        if attempt < 4 {
            assert_eq!(frame(&run).stage, attempt + 1);
            for (position, expected) in [(first, ids[first]), (second, ids[second])] {
                assert!(view(&run).choices.iter().any(|choice| matches!(choice.kind,
                    RoomChoiceKind::EventCard { option, card: Some(shown), .. } if usize::from(option) == position && shown == expected)));
            }
        } else {
            assert_eq!(frame(&run).stage, u8::MAX);
            assert_eq!(view(&run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Leave]);
        }
    }
}

#[test]
fn m28_transmogrifier_transforms_one_selected_card() {
    // Transform removes the selected input and obtains one fresh same-pool result: spec §Transmogrifier; ER :276, :314-321.
    let mut run = at_event(Character::Defect, 1, EventId::Transmogrifier, 34);
    let before_len = run.state.deck.len();
    let input = run.state.deck[0].clone();
    event_option(&mut run, 0, 0).unwrap();
    selected_instance(&mut run, input.instance);
    assert_eq!(run.state.deck.len(), before_len);
    assert!(!run.state.deck.iter().any(|card| card.instance == input.instance));
    assert!(run.state.deck.iter().any(|card| card.id != input.id));
    leave(&mut run);
}

#[test]
fn m28_transmogrifier_softlock_001_empty_grid() {
    // Accepted SOFTLOCK-001 behavior: no P gives [NextStage(DONE)] rather than an empty selector: spec §Transmogrifier; PC §59,77-78; ER :276.
    let mut run = at_event(Character::Ironclad, 1, EventId::Transmogrifier, 35);
    fill_deck(&mut run, &[CardId::AscendersBane, CardId::CurseOfTheBell, CardId::Necronomicurse]);
    let before = run.state.deck.clone();
    event_option(&mut run, 0, 0).unwrap();
    assert_eq!(run.state.deck, before);
    assert_eq!(view(&run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Leave]);
}

#[test]
fn m28_transmogrifier_leave() {
    let mut run = at_event(Character::Watcher, 1, EventId::Transmogrifier, 36);
    let before = run.state.deck.clone();
    event_option(&mut run, 0, 1).unwrap();
    assert_eq!(run.state.deck, before);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
}

fn wheel_spin(seed: u8, character: Character, hp: u32) -> (Run, u32, u32, usize) {
    let (mut run, node) = before_event(character, 1, EventId::WheelOfChange, seed);
    let literal_max_hp = base(character);
    run.state.max_hp = literal_max_hp;
    run.state.hp = literal_max_hp;
    run.step(&GameAction::Move(node)).unwrap();
    let max_hp = run.state.max_hp;
    let gold = run.state.gold;
    let deck_len = run.state.deck.len();
    run.state.hp = hp;
    assert_eq!(view(&run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::EventOption { stage: 0, option: 0 }], "Spin is mandatory");
    event_option(&mut run, 0, 0).unwrap();
    assert_eq!(frame(&run).stage, 1, "Spin publishes the reveal stage");
    let result = frame(&run).captured.values[2] as u8;
    assert_eq!(view(&run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::EventOption { stage: 1, option: result }]);
    assert_eq!(run.state.hp, hp, "Spin only reveals the HP effect");
    (run, max_hp, gold, deck_len)
}

fn find_wheel_arm(index: u8, character: Character, hp: u32) -> (Run, u32, u32, usize) {
    for seed in 0..100_u8 {
        let (run, max_hp, gold, deck_len) = wheel_spin(seed, character, hp);
        if frame(&run).captured.values[2] == u32::from(index) { return (run, max_hp, gold, deck_len); }
    }
    panic!("Wheel index {index} not reached in fixed seed range for {character:?}")
}

#[test]
fn m28_wheel_of_change_reveal_stage_and_potion_window() {
    // Spin reveals one result row; UsePotion remains legal until the forced press: PM Wheel ruling, D1; event frame choice window.
    let mut found = None;
    for seed in 0..100_u8 {
        let (mut run, node) = before_event(Character::Ironclad, 1, EventId::WheelOfChange, seed);
        run.state.max_hp = 80;
        run.state.hp = 40;
        run.state.potions[0] = Some(PotionId::FruitJuice);
        run.step(&GameAction::Move(node)).unwrap();
        event_option(&mut run, 0, 0).unwrap();
        if frame(&run).captured.values[2] == 5 { found = Some(run); break; }
    }
    let mut run = found.expect("fixed seed range reaches HP-loss arm");
    assert_eq!(frame(&run).stage, 1);
    assert_eq!(run.state.hp, 40, "no loss at reveal");
    assert_eq!(view(&run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::EventOption { stage: 1, option: 5 }]);
    assert!(run.decision().unwrap().candidates.iter().any(|candidate| candidate.action == GameAction::UsePotion { slot: 0, target: None }));
    run.step(&GameAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!((run.state.hp, run.state.max_hp), (45, 85), "Fruit Juice acts before the reveal is confirmed");
    event_option(&mut run, 1, 5).unwrap();
    assert_eq!((run.state.hp, run.state.max_hp), (37, 85), "HP loss is T(.1 × confirmation M 85)=8");
    leave(&mut run);
}

/// Main (M28d integration): with the reveal window, M can change before the press; Heal(M) and HP_LOSS
/// T(.1M) read M at confirmation ("Others apply on result confirmation": ER :278; spec heal row "at
/// confirmation"). M=95 → Fruit Juice → 100: confirmation loss T(10.0)=10, entry loss would be T(9.5)=9.
#[test]
fn m28_wheel_of_change_reads_max_hp_at_confirmation() {
    let mut found = None;
    for seed in 0..100_u8 {
        let (mut run, node) = before_event(Character::Ironclad, 1, EventId::WheelOfChange, seed);
        run.state.max_hp = 95;
        run.state.hp = 40;
        run.state.potions[0] = Some(PotionId::FruitJuice);
        run.step(&GameAction::Move(node)).unwrap();
        event_option(&mut run, 0, 0).unwrap();
        if frame(&run).captured.values[2] == 5 { found = Some(run); break; }
    }
    let mut run = found.expect("fixed seed range reaches HP-loss arm");
    run.step(&GameAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!((run.state.hp, run.state.max_hp), (45, 100));
    event_option(&mut run, 1, 5).unwrap();
    assert_eq!((run.state.hp, run.state.max_hp), (35, 100), "HP loss is T(.1 × confirmation M)=10");
    leave(&mut run);
}

#[test]
fn m28_wheel_of_change_gold() {
    // Index 0 commits +100 at reveal; forced press must not award it again: spec §Wheel of Change, D1; ER :278, :427.
    let (mut run, _, gold, _) = find_wheel_arm(0, Character::Ironclad, 40);
    assert_eq!(run.state.gold, gold + 100, "gold commits at reveal");
    event_option(&mut run, 1, 0).unwrap();
    assert_eq!(run.state.gold, gold + 100, "press does not award gold twice");
    leave(&mut run);
}

#[test]
fn m28_wheel_of_change_relic_reward() {
    // Skippable screenless RandomTier row; the spec's "Reward Proceed → event result → Leave → map" is collapsed by D1 to map: spec §Wheel of Change; PC X4/D1; ER :278.
    let (mut run, _, _, _) = find_wheel_arm(1, Character::Silent, 40);
    event_option(&mut run, 1, 1).unwrap();
    let screen = view(&run);
    assert_eq!(screen.kind, RoomViewKind::Rewards);
    assert!(screen.choices.iter().any(|choice| matches!(choice.kind, RoomChoiceKind::Relic(_))));
    assert!(screen.choices.iter().any(|choice| choice.kind == RoomChoiceKind::Proceed));
    let proceed = row(&run, &RoomChoiceKind::Proceed).id;
    run.step(&GameAction::Choose(proceed)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map, "D1 collapses the result page after reward Proceed");
}

#[test]
fn m28_wheel_of_change_heal() {
    // Heal(M) for each character, captured at construction: spec §Wheel of Change; ER :278, :428; CD :95.
    for (character, expected) in [(Character::Ironclad, 80), (Character::Silent, 70), (Character::Defect, 75), (Character::Watcher, 72)] {
        let (mut run, max_hp, _, _) = find_wheel_arm(2, character, 1);
        assert_eq!(run.state.hp, 1, "no heal at reveal");
        event_option(&mut run, 1, 2).unwrap();
        assert_eq!((max_hp, run.state.hp), (expected, expected), "{character:?}");
        leave(&mut run);
    }
}

#[test]
fn m28_wheel_of_change_decay() {
    // Index 3 obtains fixed Decay only on the reveal press: spec §Wheel of Change; ER :278, :323-327.
    let (mut run, _, _, _) = find_wheel_arm(3, Character::Defect, 40);
    assert_eq!(count(&run, CardId::Decay), 0);
    event_option(&mut run, 1, 3).unwrap();
    assert_eq!(count(&run, CardId::Decay), 1);
    leave(&mut run);
}

#[test]
fn m28_wheel_of_change_remove() {
    // Index 4 removes one P after the press; empty P skips removal and still completes: spec §Wheel of Change; ER :278.
    let (mut run, _, _, deck_len) = find_wheel_arm(4, Character::Watcher, 40);
    event_option(&mut run, 1, 4).unwrap();
    let instance = first_selectable_instance(&run);
    selected_instance(&mut run, instance);
    assert_eq!(run.state.deck.len(), deck_len - 1);
    leave(&mut run);

    let mut empty = None;
    for seed in 0..100_u8 {
        let (candidate, _, _, _) = wheel_spin(seed, Character::Ironclad, 40);
        if frame(&candidate).captured.values[2] == 4 { empty = Some(candidate); break; }
    }
    let mut empty = empty.expect("fixed seed range reaches empty-grid arm");
    fill_deck(&mut empty, &[CardId::AscendersBane, CardId::CurseOfTheBell, CardId::Necronomicurse]);
    let before = empty.state.deck.clone();
    event_option(&mut empty, 1, 4).unwrap();
    assert_eq!(empty.state.deck, before);
    leave(&mut empty);
}

#[test]
fn m28_wheel_of_change_hp_loss() {
    // HP_LOSS T(.1M): four literal A0 values; loss occurs only on press: spec §Wheel of Change; ER :278, :430; CD :95.
    for (character, max_hp, loss) in [(Character::Ironclad, 80, 8), (Character::Silent, 70, 7), (Character::Defect, 75, 7), (Character::Watcher, 72, 7)] {
        let (mut run, actual_max, _, _) = find_wheel_arm(5, character, max_hp);
        assert_eq!(actual_max, max_hp);
        assert_eq!(run.state.hp, max_hp, "no HP loss at reveal");
        event_option(&mut run, 1, 5).unwrap();
        assert_eq!(run.state.hp, max_hp - loss, "{character:?}");
        leave(&mut run);
    }
}

#[test]
fn m28_wheel_of_change_each_index_for_four_characters() {
    // The six result indices and their four character literals are fixed by spec §Wheel of Change, D1; ER :278; CD :95.
    for (character, max_hp, loss) in [(Character::Ironclad, 80, 8), (Character::Silent, 70, 7), (Character::Defect, 75, 7), (Character::Watcher, 72, 7)] {
        for index in 0..6_u8 {
            let hp = if index == 2 { 1 } else { max_hp };
            let (mut run, _, gold, deck_len) = find_wheel_arm(index, character, hp);
            if index == 0 { assert_eq!(run.state.gold, gold + 100); } else { assert_eq!(run.state.gold, gold); }
            event_option(&mut run, 1, index).unwrap();
            match index {
                0 => { assert_eq!(run.state.gold, gold + 100); leave(&mut run); }
                1 => {
                    assert_eq!(view(&run).kind, RoomViewKind::Rewards);
                    let proceed = row(&run, &RoomChoiceKind::Proceed).id;
                    run.step(&GameAction::Choose(proceed)).unwrap();
                    assert_eq!(view(&run).kind, RoomViewKind::Map, "D1 result-page collapse");
                }
                2 => { assert_eq!(run.state.hp, max_hp, "{character:?}"); leave(&mut run); }
                3 => { assert_eq!(count(&run, CardId::Decay), 1, "{character:?}"); leave(&mut run); }
                4 => {
                    let instance = first_selectable_instance(&run);
                    selected_instance(&mut run, instance);
                    assert_eq!(run.state.deck.len(), deck_len - 1, "{character:?}");
                    leave(&mut run);
                }
                5 => { assert_eq!(run.state.hp, max_hp - loss, "{character:?}"); leave(&mut run); }
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn m28_accursed_blacksmith_forge() {
    // Forge selects one U1 and upgrades that instance: spec §Accursed Blacksmith; ER :284.
    let mut run = at_event(Character::Ironclad, 1, EventId::AccursedBlacksmith, 38);
    let bash = run.state.deck.iter().find(|card| card.id == CardId::Bash).unwrap().clone();
    event_option(&mut run, 0, 0).unwrap();
    selected_instance(&mut run, bash.instance);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == bash.instance).unwrap().upgrades, 1);
    leave(&mut run);
}

#[test]
fn m28_accursed_blacksmith_rummage() {
    // Pain precedes direct Warped Tongs; no Circlet substitution: spec §Accursed Blacksmith; ER :284, :390-391.
    let mut run = at_event(Character::Silent, 1, EventId::AccursedBlacksmith, 39);
    let pain = count(&run, CardId::Pain);
    event_option(&mut run, 0, 1).unwrap();
    assert_eq!(count(&run, CardId::Pain), pain + 1);
    assert!(run.state.relics.contains(&RelicId::WarpedTongs));
    assert!(!run.state.relics.contains(&RelicId::Circlet));
    leave(&mut run);
}

#[test]
fn m28_accursed_blacksmith_forge_disabled_and_leave() {
    let mut run = at_event(Character::Defect, 1, EventId::AccursedBlacksmith, 40);
    fill_deck(&mut run, &[CardId::AscendersBane, CardId::CurseOfTheBell, CardId::Necronomicurse]);
    let forge = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 });
    assert_eq!(forge.status, RoomChoiceStatus::Disabled);
    let before = run.clone();
    assert_eq!(run.step(&GameAction::Choose(forge.id)), Err(RunError::InvalidAction));
    assert_eq!(run, before, "disabled Forge rejects atomically");
    event_option(&mut run, 0, 2).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
}

fn bonfire_with(card: CardId, seed: u8) -> (Run, u64, u32, u32) {
    let mut run = at_event(Character::Ironclad, 1, EventId::BonfireSpirits, seed);
    let instance = keep_only(&mut run, card);
    run.state.hp = 1;
    let max_hp = run.state.max_hp;
    (run, instance, max_hp, 1)
}

fn bonfire_offer(run: &mut Run, instance: u64) -> RunTransition {
    event_option(run, 0, 0).unwrap();
    selected_instance(run, instance)
}

#[test]
fn m28_bonfire_spirits_basic() {
    // Basic gives no reward before the selected instance is removed: spec §Bonfire Spirits; ER :285, :431.
    let (mut run, instance, max_hp, _) = bonfire_with(CardId::Strike, 41);
    let result = bonfire_offer(&mut run, instance);
    assert_eq!(result.terminal, Some(RunTerminal::EmptyDeck));
    assert_eq!((run.state.max_hp, run.state.hp), (max_hp, 1));
}

#[test]
fn m28_bonfire_spirits_common_special() {
    // Common and Special both request Heal(5): spec §Bonfire Spirits; ER :285, :428.
    for card in [CardId::Anger, CardId::Bite] {
        let (mut run, instance, max_hp, _) = bonfire_with(card, 42);
        let result = bonfire_offer(&mut run, instance);
        assert_eq!(result.terminal, Some(RunTerminal::EmptyDeck));
        assert_eq!((run.state.max_hp, run.state.hp), (max_hp, 6), "{card:?}");
    }
}

#[test]
fn m28_bonfire_spirits_uncommon() {
    // Uncommon requests Heal(M), by character literal: spec §Bonfire Spirits; ER :285, :428; CD :95.
    for (character, expected) in [(Character::Ironclad, 80), (Character::Silent, 70), (Character::Defect, 75), (Character::Watcher, 72)] {
        let mut run = at_event(character, 1, EventId::BonfireSpirits, 43);
        let instance = keep_only(&mut run, CardId::BattleTrance);
        run.state.max_hp = expected;
        run.state.hp = 1;
        let result = bonfire_offer(&mut run, instance);
        assert_eq!(result.terminal, Some(RunTerminal::EmptyDeck));
        assert_eq!((run.state.max_hp, run.state.hp), (expected, expected), "{character:?}");
    }
}

#[test]
fn m28_bonfire_spirits_rare() {
    // Rare requests +10 max HP then Heal(new M), then removal: spec §Bonfire Spirits; ER :285, :428-431; CD :95.
    for (character, start, expected) in [(Character::Ironclad, 80, 90), (Character::Silent, 70, 80), (Character::Defect, 75, 85), (Character::Watcher, 72, 82)] {
        let mut run = at_event(character, 1, EventId::BonfireSpirits, 44);
        let instance = keep_only(&mut run, CardId::Feed);
        run.state.max_hp = start;
        run.state.hp = 1;
        let result = bonfire_offer(&mut run, instance);
        assert_eq!(result.terminal, Some(RunTerminal::EmptyDeck));
        assert_eq!((run.state.max_hp, run.state.hp), (expected, expected), "{character:?}, start M {start}");
    }
}

#[test]
fn m28_bonfire_spirits_curse_parasite_and_empty_deck() {
    // Curse grants Spirit Poop (or Circlet if owned), and Parasite removal applies its -3 M hook: spec §Bonfire Spirits; ER :285, :390-391, :431.
    let mut run = at_event(Character::Silent, 1, EventId::BonfireSpirits, 45);
    let parasite = keep_only(&mut run, CardId::Parasite);
    let max_hp = run.state.max_hp;
    let result = bonfire_offer(&mut run, parasite);
    assert_eq!(result.terminal, Some(RunTerminal::EmptyDeck));
    assert_eq!(run.state.max_hp, max_hp - 3);
    assert!(run.state.relics.contains(&RelicId::SpiritPoop));
    let mut fallback = at_event(Character::Ironclad, 1, EventId::BonfireSpirits, 46);
    let pain = keep_only(&mut fallback, CardId::Pain);
    fallback.state.relics.push(RelicId::SpiritPoop);
    bonfire_offer(&mut fallback, pain);
    assert!(fallback.state.relics.contains(&RelicId::Circlet));
}

#[test]
fn m28_face_trader_touch() {
    // Touch grants 75 then NORMAL Damage(max(1,M/10)), with captured M: spec §Face Trader; ER :286, :427, :430; CD :95.
    for (character, max_hp, damage) in [(Character::Ironclad, 80, 8), (Character::Silent, 70, 7), (Character::Defect, 75, 7), (Character::Watcher, 72, 7)] {
        let (mut run, node) = before_event(character, 1, EventId::FaceTrader, 47);
        run.state.max_hp = max_hp; run.state.hp = max_hp;
        run.step(&GameAction::Move(node)).unwrap();
        run.state.max_hp += 10; run.state.hp = max_hp + 10;
        let gold = run.state.gold;
        event_option(&mut run, 0, 0).unwrap();
        assert_eq!((run.state.gold, run.state.hp), (gold + 75, max_hp + 10 - damage), "{character:?}");
        leave(&mut run);
    }
}

#[test]
fn m28_face_trader_trade_and_empty_face_pool() {
    // Trade draws a missing face at the choice (`getRandomFace` in FaceTrader.buttonEffect; PM ruling on
    // the review-v1 source check); if all five are owned it grants Circlet: spec §Face Trader; ER :286, :389-391.
    let faces = [RelicId::CultistHeadpiece, RelicId::FaceOfCleric, RelicId::GremlinVisage, RelicId::NlothsHungryFace, RelicId::SsserpentHead];
    let mut run = at_event(Character::Defect, 2, EventId::FaceTrader, 48);
    assert!(frame(&run).captured.relics.is_empty(), "no construction face draw");
    let owned = run.state.relics.clone();
    event_option(&mut run, 0, 1).unwrap();
    let gained: Vec<_> = run.state.relics.iter().filter(|id| !owned.contains(id)).copied().collect();
    assert_eq!(gained.len(), 1);
    assert!(faces.contains(&gained[0]));
    leave(&mut run);
    // Ownership is read at the choice: four faces gained after construction leave only the fifth.
    let mut late = at_event(Character::Silent, 1, EventId::FaceTrader, 51);
    late.state.relics.extend(&faces[..4]);
    event_option(&mut late, 0, 1).unwrap();
    assert!(late.state.relics.contains(&RelicId::SsserpentHead));
    leave(&mut late);
    let (mut all_owned, node) = before_event(Character::Watcher, 1, EventId::FaceTrader, 49);
    all_owned.state.relics.extend(faces);
    all_owned.step(&GameAction::Move(node)).unwrap();
    event_option(&mut all_owned, 0, 1).unwrap();
    assert!(all_owned.state.relics.contains(&RelicId::Circlet));
    leave(&mut all_owned);
}

#[test]
fn m28_face_trader_leave() {
    let mut run = at_event(Character::Ironclad, 1, EventId::FaceTrader, 50);
    let before = run.state.clone();
    event_option(&mut run, 0, 2).unwrap();
    assert_eq!(run.state.gold, before.gold);
    assert_eq!(run.state.hp, before.hp);
    assert_eq!(run.state.relics, before.relics);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
}

fn fountain_at_event(character: Character, act: u8, seed: u8) -> Run {
    let (mut run, node) = before_event(character, act, EventId::FountainOfCleansing, seed);
    let instance = run.state.deck.iter().map(|card| card.instance).max().unwrap() + 1;
    run.state.deck.push(CardInstance { id: CardId::Clumsy, instance, upgrades: 0, misc: 0 });
    run.step(&GameAction::Move(node)).unwrap();
    run
}

#[test]
fn m28_fountain_of_cleansing_removes_qualifying_curses_backwards() {
    // Keep the three protected curses and bottled Flame/Lightning; local Fountain logic still removes Bottled Tornado: spec §Fountain; ER :245-247, :287, :431.
    let mut run = fountain_at_event(Character::Defect, 1, 51);
    let ids = [CardId::Strike, CardId::Clumsy, CardId::Pain, CardId::AscendersBane, CardId::CurseOfTheBell,
        CardId::Necronomicurse, CardId::Doubt, CardId::Shame, CardId::Writhe];
    fill_deck(&mut run, &ids);
    run.state.relic_cards.clear();
    let first = run.state.deck[0].instance;
    run.state.relic_cards.insert(RelicId::BottledFlame, first + 1);
    run.state.relic_cards.insert(RelicId::BottledLightning, first + 2);
    run.state.relic_cards.insert(RelicId::BottledTornado, first + 6);
    event_option(&mut run, 0, 0).unwrap();
    let remaining: Vec<_> = run.state.deck.iter().map(|card| card.id).collect();
    assert_eq!(remaining, vec![CardId::Strike, CardId::Clumsy, CardId::Pain, CardId::AscendersBane,
        CardId::CurseOfTheBell, CardId::Necronomicurse]);
    leave(&mut run);
}

#[test]
fn m28_fountain_of_cleansing_leave() {
    let mut run = fountain_at_event(Character::Watcher, 2, 52);
    let before = run.state.deck.clone();
    event_option(&mut run, 0, 1).unwrap();
    assert_eq!(run.state.deck, before);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
}

#[test]
fn m28_lab_three_flat_class_potions() {
    // Lab creates three successive flat-ID class-potion offers; Proceed goes directly to map: spec §Lab; ER :288, :329-343, :435-437; CD potion table.
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = at_event(character, 1, EventId::Lab, 53);
        event_option(&mut run, 0, 0).unwrap();
        let screen = view(&run);
        assert_eq!(screen.kind, RoomViewKind::Rewards);
        let expected = crate::items::potion_generation::candidates(character, false).unwrap();
        let potions: Vec<_> = screen.choices.iter().filter_map(|choice| match choice.kind { RoomChoiceKind::Potion(id) => Some(id), _ => None }).collect();
        assert_eq!(potions.len(), 3);
        assert!(potions.iter().all(|id| expected.contains(id)), "{character:?}: {potions:?}");
        let proceed = row(&run, &RoomChoiceKind::Proceed).id;
        run.step(&GameAction::Choose(proceed)).unwrap();
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

#[test]
fn m28_note_for_yourself_take_default_iron_wave_all_characters() {
    // Default stored profile is (IronWave,0) for all four characters; the retrieved card may itself be chosen: spec §Note For Yourself; ER :289, :431-434; CD :74-85.
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = at_event(character, 1, EventId::NoteForYourself, 54);
        assert_eq!(run.state.stored_note, (CardId::IronWave, 0), "default {character:?}");
        let old: Vec<_> = run.state.deck.iter().map(|card| card.instance).collect();
        event_option(&mut run, 0, 0).unwrap();
        let stored_instance = run.state.pending.as_ref().unwrap().selection.candidates.iter()
            .find(|candidate| candidate.card.id == CardId::IronWave && !old.contains(&candidate.card.instance)).unwrap().card.instance;
        selected_instance(&mut run, stored_instance);
        assert_eq!(run.state.stored_note, (CardId::IronWave, 0));
        assert!(!run.state.deck.iter().any(|card| card.instance == stored_instance));
        leave(&mut run);
    }
}

#[test]
fn m28_note_for_yourself_stored_curse_bypasses_omamori() {
    // Direct stored-card obtain bypasses Omamori but keeps the outgoing chosen pair: spec §Note For Yourself; PC X6; ER :289, :431-434.
    let mut run = at_event(Character::Silent, 1, EventId::NoteForYourself, 55);
    run.state.stored_note = (CardId::Clumsy, 0);
    run.state.relics.push(RelicId::Omamori);
    run.state.relic_counters.insert(RelicId::Omamori, 2);
    event_option(&mut run, 0, 0).unwrap();
    let stored = run.state.pending.as_ref().unwrap().selection.candidates.iter().find(|candidate| candidate.card.id == CardId::Clumsy).unwrap().card.instance;
    selected_instance(&mut run, stored);
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&2));
    assert_eq!(run.state.stored_note, (CardId::Clumsy, 0));
    assert!(!run.state.deck.iter().any(|card| card.instance == stored));
    leave(&mut run);
}

#[test]
fn m28_note_for_yourself_ignore() {
    // Ignore leaves both the profile note and deck unchanged: spec §Note For Yourself; ER :289.
    let mut run = at_event(Character::Ironclad, 1, EventId::NoteForYourself, 56);
    let (deck, note) = (run.state.deck.clone(), run.state.stored_note);
    event_option(&mut run, 0, 1).unwrap();
    assert_eq!((&run.state.deck, run.state.stored_note), (&deck, note));
    assert_eq!(view(&run).kind, RoomViewKind::Map);
}

#[test]
fn m28_all_shared_bodies_are_registered() {
    // Dispatch list is kept at each owning file; Main's Golden Shrine remains in events::mod.
    for event in [EventId::MatchAndKeep, EventId::Transmogrifier, EventId::WheelOfChange,
        EventId::AccursedBlacksmith, EventId::BonfireSpirits, EventId::FaceTrader,
        EventId::FountainOfCleansing, EventId::Lab, EventId::NoteForYourself] {
        let run = if event == EventId::FountainOfCleansing {
            fountain_at_event(Character::Ironclad, 1, 57)
        } else {
            at_event(Character::Ironclad, 1, event, 57)
        };
        assert_eq!(view(&run).kind, RoomViewKind::Event(event));
    }
}

/// Review-v1 I1 (reviewer probe `review_probe_match_first_flip_visible`): the first card of an attempt
/// stays face up while the second is chosen (D2 "each card once it has been revealed"). Its position is
/// published as a Disabled row carrying the identity; choosing it again is rejected atomically.
#[test]
fn m28_match_and_keep_first_flip_visible_before_second_pick() {
    let mut run = at_event(Character::Silent, 1, EventId::MatchAndKeep, 33);
    let ids = frame(&run).captured.card_ids.clone();
    event_position(&mut run, 0, 0);
    let shown: Vec<_> = view(&run).choices.iter().filter_map(|c| match c.kind { RoomChoiceKind::EventCard { option, card, .. } => Some((option, card, c.status)), _ => None }).collect();
    assert_eq!(shown.len(), 12, "the flipped position keeps its row");
    assert!(shown.contains(&(0, Some(ids[0]), RoomChoiceStatus::Disabled)), "first flipped card visible and not selectable: {shown:?}");
    assert!(shown.iter().filter(|(o, ..)| *o != 0).all(|(_, card, status)| card.is_none() && *status == RoomChoiceStatus::Offered));
    let flipped = view(&run).choices.into_iter().find(|c| matches!(c.kind, RoomChoiceKind::EventCard { option: 0, .. })).unwrap();
    let before = run.clone();
    assert_eq!(run.step(&GameAction::Choose(flipped.id)), Err(RunError::InvalidAction));
    assert!(run == before, "re-choosing the flipped card leaves the Run unchanged");
}

/// Review-v1 C1: `GremlinWheelGame` constructor `setGold()` keys on the dungeon (PM static check):
/// Exordium 100, TheCity 200, TheBeyond 300, granted at the reveal; the press does not award it again.
#[test]
fn m28_wheel_of_change_gold_by_act() {
    for (act, amount) in [(1_u8, 100_u32), (2, 200), (3, 300)] {
        let mut done = false;
        for seed in 0..100_u8 {
            let (mut run, node) = before_event(Character::Ironclad, act, EventId::WheelOfChange, seed);
            run.step(&GameAction::Move(node)).unwrap();
            let gold = run.state.gold;
            event_option(&mut run, 0, 0).unwrap();
            if frame(&run).captured.values[2] != 0 { continue }
            assert_eq!(run.state.gold, gold + amount, "act {act} Wheel gold commits at reveal");
            event_option(&mut run, 1, 0).unwrap();
            assert_eq!(run.state.gold, gold + amount, "act {act} press does not award gold twice");
            leave(&mut run);
            done = true;
            break;
        }
        assert!(done, "fixed seed range reaches the act {act} gold arm");
    }
}
