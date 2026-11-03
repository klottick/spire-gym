//! M28d Act 1 event bodies (plan-check-v1 Bodies table; spec batch1.md §§ Dead Adventurer–World of Goop).
//! Branch expectations are source literals. M28EventBody construction/choice draws use fixed seed ranges.
use super::super::*;
use super::m28_event_core::{at_event, base, before_event, count, leave, rng, row, view};
use crate::content::{CardColor, CardId, Catalog, Character, EncounterId, EventId, RelicId};
use rooms::standard::event_exec::RelicChoice;
use rooms::standard::event_frame;

fn event_at_floor(character: Character, event: EventId, seed: u8, floor: u32) -> Run {
    let (mut run, node) = before_event(character, 1, event, seed);
    // Dead Adventurer and Mushrooms use the ordinary pool's floor > 6 gate.
    run.state.floor = floor - 1; // entry adds one (PM ruling m28f Q2)
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|map_node| map_node.id == node).unwrap().floor = floor;
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(event));
    run
}

fn choose_at(run: &mut Run, stage: u8, option: u8) -> Result<RunTransition, RunError> {
    let id = row(run, &RoomChoiceKind::EventOption { stage, option }).id;
    run.step(&GameAction::Choose(id))
}

fn choose_event(run: &mut Run, option: u8) -> Result<RunTransition, RunError> {
    choose_at(run, 0, option)
}

fn select_first(run: &mut Run) {
    let id = run.decision().unwrap().candidates.iter().find_map(|candidate| match candidate.action {
        GameAction::Select(id) => Some(id),
        _ => None,
    }).expect("event selection has a card candidate");
    run.step(&GameAction::Select(id)).unwrap();
}

fn set_event_stage(run: &mut Run, stage: u8) {
    run.state.room_loop.as_mut().unwrap().standard_event.as_mut().unwrap().stage = stage;
    event_frame::publish(run).unwrap();
}

fn current_event_stage(run: &Run) -> u8 {
    run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().stage
}

fn event_captured_values(run: &Run) -> Vec<u32> {
    run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.values.clone()
}

fn set_deck(run: &mut Run, cards: &[(CardId, u32, i32)]) {
    let template = run.state.deck[0].clone();
    run.state.deck = cards.iter().enumerate().map(|(index, (id, upgrades, misc))| {
        let mut card = template.clone();
        card.id = *id;
        card.instance = 90_000 + index as u64;
        card.upgrades = *upgrades;
        card.misc = *misc;
        card
    }).collect();
    run.state.relic_cards.clear();
}

fn finish_rewards(run: &mut Run) {
    let id = row(run, &RoomChoiceKind::Proceed).id;
    run.step(&GameAction::Choose(id)).unwrap();
    assert_eq!(view(run).kind, RoomViewKind::Map);
}

fn gold_rows(run: &Run) -> Vec<u32> {
    view(run).choices.iter().filter_map(|choice| match &choice.kind {
        RoomChoiceKind::Gold(amount) => Some(*amount),
        _ => None,
    }).collect()
}

fn relic_rows(run: &Run) -> Vec<RelicId> {
    view(run).choices.iter().filter_map(|choice| match &choice.kind {
        RoomChoiceKind::Relic(id) => Some(*id),
        _ => None,
    }).collect()
}

fn count_upgrades(run: &Run) -> u32 {
    run.state.deck.iter().map(|card| card.upgrades).sum()
}

/// Dead Adventurer: search risk is 25/50/75 on successful-search counts 0/1/2; a safe search
/// consumes the front token and the third safe search completes the event. [ER:258; PC:48; W rev 68427]
#[test]
fn m28_dead_adventurer_search() {
    let mut risk = [[false; 2]; 3]; // [successful searches][ambush, safe]
    let mut tokens = [false; 3]; // GOLD, NOTHING, RELIC
    let mut saw_three_safe = false;
    for seed in 0..=u8::MAX {
        let mut run = event_at_floor(Character::Ironclad, EventId::DeadAdventurer, seed, 7);
        for successful_searches in 0..3u8 {
            assert_eq!(current_event_stage(&run), successful_searches);
            let gold = run.state.gold;
            let relic_count = run.state.relics.len();
            let deck = run.state.deck.clone();
            choose_at(&mut run, successful_searches, 0).unwrap();
            if let Some(combat) = &run.state.active_combat {
                let expected = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.encounter.unwrap();
                assert_eq!(combat.encounter, expected, "ambush uses the construction-time encounter");
                let remaining=&run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.values[usize::from(successful_searches)..];
                let rewards=event_frame::combat_rewards(&run.state).unwrap();
                let gold_range=if remaining.contains(&0){55..=65}else{25..=35};
                assert!(gold_range.contains(&rewards.gold), "ambush gold retains only unconsumed GOLD: {rewards:?}");
                assert_eq!(rewards.gold_range,None,"the 25–35 range is drawn at fight entry");
                // M28e (PM check m28e/source-check-open-b-v1 item 12): the RELIC row is rolled and taken at fight entry.
                assert_eq!(rewards.relics.len(),usize::from(remaining.contains(&2)),"ambush relic row remains only while RELIC is unconsumed");
                assert!(rewards.relics.iter().all(|r|matches!(r,RelicChoice::Specific(_))),"the relic row is taken at fight entry: {rewards:?}");
                risk[successful_searches as usize][0] = true;
                break;
            }
            risk[successful_searches as usize][1] = true;
            let gained_gold = run.state.gold - gold;
            let gained_relics = run.state.relics.len() - relic_count;
            match (gained_gold, gained_relics) {
                (30, 0) => tokens[0] = true,
                (0, 0) if run.state.deck == deck => tokens[1] = true,
                (0, 1) => tokens[2] = true,
                other => panic!("unexpected safe-search token effect {other:?}"),
            }
            if view(&run).kind != RoomViewKind::Event(EventId::DeadAdventurer) {
                // A random relic can open its own obtain continuation; other seeds still witness
                // later search stages without assuming that continuation's completion here.
                break;
            }
            if successful_searches == 2 {
                assert_eq!(view(&run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Leave]);
                saw_three_safe = true;
            } else {
                assert_eq!(current_event_stage(&run), successful_searches + 1);
                assert!(view(&run).choices.iter().any(|choice| choice.kind == RoomChoiceKind::EventOption { stage: successful_searches + 1, option: 0 }));
            }
        }
    }
    assert!(risk.iter().all(|arms| arms == &[true, true]), "both Roll arms at all three thresholds: {risk:?}");
    assert!(tokens.iter().all(|seen| *seen), "each shuffled token effect was reached: {tokens:?}");
    assert!(saw_three_safe, "three safe searches end with Leave only");
}

#[test]
fn m28_dead_adventurer_leave() {
    let mut run = event_at_floor(Character::Ironclad, EventId::DeadAdventurer, 11, 7);
    let before = (run.state.gold, run.state.hp, run.state.relics.clone(), run.state.deck.clone());
    choose_event(&mut run, 1).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.gold, run.state.hp, run.state.relics, run.state.deck), before);
}

/// Event combat reward return: all three construction-time enemy choices reach the ambush branch;
/// unconsumed GOLD and RELIC are merged with 25–35 fight gold and ordinary EventRoom rewards. [ER:258; DES:372-401]
#[test]
fn m28_event_combat_dead_adventurer_returns() {
    for enemy in [EncounterId::ThreeSentries, EncounterId::GremlinNob, EncounterId::LagavulinEvent] {
        let mut found = None;
        for seed in 0..=u8::MAX {
            let mut run = event_at_floor(Character::Ironclad, EventId::DeadAdventurer, seed, 7);
            let captured = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.encounter.unwrap();
            if captured != enemy { continue }
            if choose_event(&mut run, 0).is_ok() && run.state.active_combat.as_ref().is_some_and(|combat| combat.encounter == enemy) {
                assert!(matches!(&run.state.active_combat.as_ref().unwrap().origin,
                    CombatOrigin::Event { event: EventId::DeadAdventurer, stage: 0, .. }));
                assert!(event_frame::combat_elite(&run.state), "event combat sets eliteTrigger");
                super::m28_world::win(&mut run);
                if view(&run).kind == RoomViewKind::Rewards
                    && view(&run).choices.iter().any(|choice| matches!(&choice.kind, RoomChoiceKind::Potion(_)))
                {
                    found = Some(run);
                    break;
                }
            }
        }
        let mut run = found.unwrap_or_else(|| panic!("no fixed seed reached the {enemy:?} ambush"));
        assert_eq!(view(&run).kind, RoomViewKind::Rewards);
        let gold = gold_rows(&run);
        assert_eq!(gold.len(), 1, "fixed +30 token gold joins the 25–35 combat range");
        assert!((55..=65).contains(&gold[0]), "one merged gold row: {gold:?}");
        assert_eq!(relic_rows(&run).len(), 1, "unconsumed RELIC is a reward-row relic");
        assert_eq!(view(&run).choices.iter().filter(|choice| matches!(&choice.kind, RoomChoiceKind::Potion(_))).count(), 1);
        assert_eq!(view(&run).choices.iter().filter(|choice| choice.kind == RoomChoiceKind::CardReward).count(), 1);
        assert!(!view(&run).choices.iter().any(|choice| matches!(&choice.kind, RoomChoiceKind::Emerald)));
        finish_rewards(&mut run);
    }
}

#[test]
fn m28_golden_idol_take() {
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenIdol, 12);
    choose_event(&mut run, 0).unwrap();
    assert!(run.state.relics.contains(&RelicId::GoldenIdol));
    assert_eq!(view(&run).kind, RoomViewKind::Event(EventId::GoldenIdol));
    assert_eq!(view(&run).choices.len(), 3, "Take opens mandatory Injury, damage, or max-HP payment");
    assert!(view(&run).choices.iter().all(|choice| !matches!(choice.kind, RoomChoiceKind::Leave)));
    choose_at(&mut run, 1, 0).unwrap();
    leave(&mut run);

    let (mut run, node) = before_event(Character::Silent, 1, EventId::GoldenIdol, 13);
    run.state.relics.push(RelicId::GoldenIdol);
    run.step(&GameAction::Move(node)).unwrap();
    let circlets = run.state.relics.iter().filter(|id| **id == RelicId::Circlet).count();
    choose_event(&mut run, 0).unwrap();
    assert_eq!(run.state.relics.iter().filter(|id| **id == RelicId::Circlet).count(), circlets + 1,
        "owned Golden Idol is replaced by fixed Circlet");
}

#[test]
fn m28_golden_idol_injury() {
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenIdol, 14);
    choose_event(&mut run, 0).unwrap();
    choose_at(&mut run, 1, 0).unwrap();
    assert_eq!(count(&run, CardId::Injury), 1, "fixed Injury, not a random curse");
    leave(&mut run);
}

#[test]
fn m28_golden_idol_damage() {
    // T(.25M), captured before Take: Ironclad 20, Silent 17, Defect 18, Watcher 18. [CD:95,104; spec §Golden Idol]
    for (character, damage) in [(Character::Ironclad, 20), (Character::Silent, 17), (Character::Defect, 18), (Character::Watcher, 18)] {
        let m = base(character);
        let (mut run, node) = before_event(character, 1, EventId::GoldenIdol, 15);
        run.state.max_hp = m;
        run.state.hp = m;
        run.step(&GameAction::Move(node)).unwrap();
        run.state.max_hp += 20;
        choose_event(&mut run, 0).unwrap();
        choose_at(&mut run, 1, 1).unwrap();
        assert_eq!(run.state.hp, m - damage, "{character:?}");
        assert_eq!(run.state.max_hp, m + 20, "damage payment leaves max HP unchanged");
        leave(&mut run);
    }
}

#[test]
fn m28_golden_idol_max_hp_loss() {
    // max(1,T(.08M)): Ironclad 6, Silent 5, Defect 6, Watcher 5. [CD:95,105; spec §Golden Idol]
    for (character, loss) in [(Character::Ironclad, 6), (Character::Silent, 5), (Character::Defect, 6), (Character::Watcher, 5)] {
        let m = base(character);
        let (mut run, node) = before_event(character, 1, EventId::GoldenIdol, 16);
        run.state.max_hp = m;
        run.state.hp = m;
        run.step(&GameAction::Move(node)).unwrap();
        run.state.max_hp += 20;
        run.state.hp = run.state.max_hp;
        choose_event(&mut run, 0).unwrap();
        choose_at(&mut run, 1, 2).unwrap();
        assert_eq!((run.state.max_hp, run.state.hp), (m + 20 - loss, m + 20 - loss), "{character:?}");
        leave(&mut run);
    }
}

#[test]
fn m28_golden_idol_leave() {
    let mut run = at_event(Character::Defect, 1, EventId::GoldenIdol, 17);
    let before = (run.state.gold, run.state.hp, run.state.max_hp, run.state.relics.clone(), run.state.deck.clone());
    choose_event(&mut run, 1).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.gold, run.state.hp, run.state.max_hp, run.state.relics, run.state.deck), before);
}

#[test]
fn m28_mushrooms_fight() {
    let (mut run, node) = before_event(Character::Silent, 1, EventId::Mushrooms, 18);
    run.state.floor = 6; // entry adds one: floor 7 (PM ruling m28f Q2)
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|map_node| map_node.id == node).unwrap().floor = 7;
    run.state.relics.push(RelicId::OddMushroom);
    run.step(&GameAction::Move(node)).unwrap();
    choose_event(&mut run, 0).unwrap();
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, EncounterId::ThreeFungiBeasts);
    assert!(!event_frame::combat_elite(&run.state));
    let rewards = event_frame::combat_rewards(&run.state).unwrap();
    assert!(rewards.gold_range.is_none(), "the inclusive range is drawn at combat entry");
    assert!((20..=30).contains(&rewards.gold), "fight-start gold draw: {}", rewards.gold);
    assert_eq!(rewards.relics, vec![RelicChoice::Specific(RelicId::Circlet)]);
}

#[test]
fn m28_mushrooms_eat() {
    // T(.25M) at choice time: Ironclad 20, Silent 17, Defect 18, Watcher 18. [CD:95,106; spec §Hypnotizing Colored Mushrooms]
    for (character, heal) in [(Character::Ironclad, 20), (Character::Silent, 17), (Character::Defect, 18), (Character::Watcher, 18)] {
        let m = base(character);
        let (mut run, node) = before_event(character, 1, EventId::Mushrooms, 19);
        run.state.floor = 6; // entry adds one: floor 7 (PM ruling m28f Q2)
        run.state.map.as_mut().unwrap().nodes.iter_mut().find(|map_node| map_node.id == node).unwrap().floor = 7;
        run.state.max_hp = m;
        run.state.hp = 1;
        run.step(&GameAction::Move(node)).unwrap();
        choose_event(&mut run, 1).unwrap();
        assert_eq!((run.state.hp, count(&run, CardId::Parasite)), (1 + heal, 1), "{character:?}");
        leave(&mut run);
    }
    // Changing M after construction must affect the Eat branch's choice-time heal.
    let mut run = event_at_floor(Character::Ironclad, EventId::Mushrooms, 20, 7);
    run.state.hp = 1;
    run.state.max_hp = 100;
    choose_event(&mut run, 1).unwrap();
    assert_eq!(run.state.hp, 26);
}

#[test]
fn m28_event_combat_mushrooms_returns() {
    let (mut run, node) = before_event(Character::Defect, 1, EventId::Mushrooms, 21);
    run.state.floor = 6; // entry adds one: floor 7 (PM ruling m28f Q2)
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|map_node| map_node.id == node).unwrap().floor = 7;
    run.state.relics.push(RelicId::OddMushroom);
    run.step(&GameAction::Move(node)).unwrap();
    choose_event(&mut run, 0).unwrap();
    super::m28_world::win(&mut run);
    assert_eq!(view(&run).kind, RoomViewKind::Rewards);
    let gold = gold_rows(&run);
    assert_eq!(gold.len(), 1);
    assert!((20..=30).contains(&gold[0]));
    assert_eq!(relic_rows(&run), vec![RelicId::Circlet]);
    assert_eq!(view(&run).choices.iter().filter(|choice| matches!(&choice.kind, RoomChoiceKind::Potion(_))).count(), 1);
    assert_eq!(view(&run).choices.iter().filter(|choice| choice.kind == RoomChoiceKind::CardReward).count(), 1);
    assert!(!view(&run).choices.iter().any(|choice| matches!(&choice.kind, RoomChoiceKind::Emerald)));
    finish_rewards(&mut run);
}

#[test]
fn m28_living_wall_forget() {
    let mut run = at_event(Character::Ironclad, 1, EventId::LivingWall, 22);
    let deck_size = run.state.deck.len();
    choose_event(&mut run, 0).unwrap();
    select_first(&mut run);
    assert_eq!(run.state.deck.len(), deck_size - 1);
    leave(&mut run);

    let mut run = at_event(Character::Silent, 1, EventId::LivingWall, 23);
    set_deck(&mut run, &[(CardId::AscendersBane, 0, 0)]);
    let before = run.state.deck.clone();
    choose_event(&mut run, 0).unwrap();
    assert!(run.state.pending.is_none(), "empty P skips the removal grid");
    assert_eq!(run.state.deck, before);
    leave(&mut run);
}

#[test]
fn m28_living_wall_change() {
    // The accepted sheet records 72/71/69/71, but the checked-in Blue ordinary_size rows total
    // 18+36+17=71. Pin the live pool owner here and flag the spec discrepancy for reconciliation.
    let cases = [
        (Character::Ironclad, CardId::Strike, CardColor::Red, 72),
        (Character::Silent, CardId::StrikeGreen, CardColor::Green, 71),
        (Character::Defect, CardId::StrikeBlue, CardColor::Blue, 71),
        (Character::Watcher, CardId::StrikePurple, CardColor::Purple, 71),
    ];
    for (character, input, color, pool_size) in cases {
        let mut run = at_event(character, 1, EventId::LivingWall, 24);
        set_deck(&mut run, &[(input, 0, 0)]);
        assert_eq!(crate::content::pools::cards(crate::content::pools::CardPool::Ordinary {
            character, rarity: None, card_type: None,
        }).unwrap().len(), pool_size, "{character:?} source pool literal");
        choose_event(&mut run, 1).unwrap();
        select_first(&mut run);
        assert_eq!(run.state.deck.len(), 1, "transform replaces one physical card");
        let output = run.state.deck[0].id;
        assert_ne!(output, input, "transform excludes the input identity");
        assert_eq!(Catalog::standard().card(output).unwrap().color, color, "{character:?} class transform support");
        leave(&mut run);
    }

    let mut run = at_event(Character::Ironclad, 1, EventId::LivingWall, 25);
    set_deck(&mut run, &[(CardId::AscendersBane, 0, 0)]);
    let before = run.state.deck.clone();
    choose_event(&mut run, 1).unwrap();
    assert!(run.state.pending.is_none(), "empty P skips the transform grid");
    assert_eq!(run.state.deck, before);
    leave(&mut run);
}

#[test]
fn m28_living_wall_grow() {
    let mut run = at_event(Character::Watcher, 1, EventId::LivingWall, 26);
    let upgrades = count_upgrades(&run);
    choose_event(&mut run, 2).unwrap();
    select_first(&mut run);
    assert_eq!(count_upgrades(&run), upgrades + 1, "Grow upgrades one selected U1 instance");
    leave(&mut run);

    let (mut run, node) = before_event(Character::Ironclad, 1, EventId::LivingWall, 27);
    for card in &mut run.state.deck { card.upgrades = 1; }
    run.step(&GameAction::Move(node)).unwrap();
    let grow = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 2 });
    assert_eq!(grow.status, RoomChoiceStatus::Disabled, "Grow is disabled when U1 is empty");
    let before = run.clone();
    assert_eq!(run.step(&GameAction::Choose(grow.id)), Err(RunError::InvalidAction));
    assert!(run == before, "rejected disabled Grow leaves the Run unchanged");

    let mut run = at_event(Character::Defect, 1, EventId::LivingWall, 28);
    set_deck(&mut run, &[(CardId::StrikeBlue, 0, 0)]);
    run.state.relic_cards.insert(RelicId::BottledFlame, run.state.deck[0].instance);
    let before = run.state.deck.clone();
    choose_event(&mut run, 2).unwrap();
    assert!(run.state.pending.is_none(), "empty P skips Grow's grid even with U1 present");
    assert_eq!(run.state.deck, before);
    leave(&mut run);
}

#[test]
fn m28_scrap_ooze_reach() {
    // BC comparator: success iff r >= 74−10n; damage is 3+n; n=7 still has four failures,
    // n=8 is guaranteed success. [ER:262 BC; PC:52]
    let mut arms = [[false; 2]; 3]; // n=0,7,8; [failure, success]
    for n in [0u8, 7, 8] {
        for seed in 0..=u8::MAX {
            let mut run = at_event(Character::Ironclad, 1, EventId::ScrapOoze, seed);
            run.state.max_hp = 200;
            run.state.hp = 200;
            set_event_stage(&mut run, n);
            // The choice-time Roll has a fixed root seed, independent of construction tags.
            run.state.rng = rng(seed);
            let relics = run.state.relics.len();
            let max_hp = run.state.max_hp;
            choose_at(&mut run, n, 0).unwrap();
            // A random-tier relic can raise max HP (and heal by the same amount) as its obtain hook runs.
            assert_eq!(run.state.hp, 200 - (3 + u32::from(n)) + (run.state.max_hp - max_hp), "n={n}, seed={seed}");
            if n == 8 {
                assert_eq!(run.state.relics.len(), relics + 1, "the n=8 success interval is all 100 inputs");
                arms[2][1] = true;
                continue;
            }
            let stage = current_event_stage(&run);
            if stage == n + 1 {
                arms[if n == 0 { 0 } else { 1 }][0] = true;
            } else {
                assert_eq!(run.state.relics.len(), relics + 1, "success obtains one screenless random-tier relic");
                arms[if n == 0 { 0 } else { 1 }][1] = true;
            }
        }
    }
    assert!(arms[0] == [true, true], "n=0 reaches failure and success: {arms:?}");
    assert!(arms[1] == [true, true], "n=7 reaches both sides of the 4/100 boundary: {arms:?}");
    assert!(arms[2][1], "n=8 always succeeds");
}

#[test]
fn m28_scrap_ooze_leave() {
    let mut run = at_event(Character::Silent, 1, EventId::ScrapOoze, 29);
    let before = (run.state.hp, run.state.gold, run.state.relics.clone(), run.state.deck.clone());
    choose_event(&mut run, 1).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.gold, run.state.relics, run.state.deck), before);
}

#[test]
fn m28_shining_light_enter() {
    // J(.20M): Ironclad 16, Silent 14, Defect 15, Watcher 14. [CD:95,107; spec §Shining Light]
    for (character, damage) in [(Character::Ironclad, 16), (Character::Silent, 14), (Character::Defect, 15), (Character::Watcher, 14)] {
        let m = base(character);
        let (mut run, node) = before_event(character, 1, EventId::ShiningLight, 30);
        run.state.max_hp = m;
        run.state.hp = m;
        run.step(&GameAction::Move(node)).unwrap();
        let upgrades = count_upgrades(&run);
        choose_event(&mut run, 0).unwrap();
        assert_eq!(run.state.hp, m - damage, "owned NORMAL damage for {character:?}");
        assert_eq!(count_upgrades(&run), upgrades + 2, "upgrade two distinct U1 instances");
        leave(&mut run);
    }

    let (mut run, node) = before_event(Character::Ironclad, 1, EventId::ShiningLight, 31);
    for card in &mut run.state.deck { card.upgrades = 1; }
    run.step(&GameAction::Move(node)).unwrap();
    let enter = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 });
    assert_eq!(enter.status, RoomChoiceStatus::Disabled);
    let before = run.clone();
    assert_eq!(run.step(&GameAction::Choose(enter.id)), Err(RunError::InvalidAction));
    assert!(run == before, "disabled Enter leaves the Run unchanged");
}

#[test]
fn m28_shining_light_leave() {
    let mut run = at_event(Character::Silent, 1, EventId::ShiningLight, 32);
    let before = (run.state.hp, run.state.gold, run.state.deck.clone());
    choose_event(&mut run, 1).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.gold, run.state.deck), before);
}

#[test]
fn m28_cleric_heal() {
    // Captured T(.25M): Ironclad 20, Silent 17, Defect 18, Watcher 18. [CD:95,108; spec §The Cleric]
    for (character, heal) in [(Character::Ironclad, 20), (Character::Silent, 17), (Character::Defect, 18), (Character::Watcher, 18)] {
        let m = base(character);
        let (mut run, node) = before_event(character, 1, EventId::Cleric, 33);
        run.state.gold = 35;
        run.state.max_hp = m;
        run.state.hp = 1;
        run.step(&GameAction::Move(node)).unwrap();
        run.state.max_hp += 20;
        choose_event(&mut run, 0).unwrap();
        assert_eq!((run.state.gold, run.state.hp), (0, 1 + heal), "{character:?}");
        leave(&mut run);
    }
}

#[test]
fn m28_cleric_purify() {
    let (mut run, node) = before_event(Character::Ironclad, 1, EventId::Cleric, 34);
    run.state.gold = 50;
    run.step(&GameAction::Move(node)).unwrap();
    let deck_size = run.state.deck.len();
    choose_event(&mut run, 1).unwrap();
    assert_eq!(run.state.gold, 0, "pay 50 before opening the purge grid");
    select_first(&mut run);
    assert_eq!(run.state.deck.len(), deck_size - 1);
    leave(&mut run);

    let (mut run, node) = before_event(Character::Silent, 1, EventId::Cleric, 35);
    run.state.gold = 50;
    set_deck(&mut run, &[(CardId::AscendersBane, 0, 0)]);
    run.step(&GameAction::Move(node)).unwrap();
    let deck = run.state.deck.clone();
    choose_event(&mut run, 1).unwrap();
    assert_eq!(run.state.gold, 50, "empty P returns without charging");
    assert!(run.state.pending.is_none(), "empty P does not open a selection");
    assert_eq!(run.state.deck, deck);
    leave(&mut run);

    let (mut run, node) = before_event(Character::Defect, 1, EventId::Cleric, 36);
    run.state.gold = 50;
    run.step(&GameAction::Move(node)).unwrap();
    run.state.gold = 49;
    let purify = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 });
    assert_eq!(purify.status, RoomChoiceStatus::Disabled, "50-gold removal is disabled at 49");
    let before = run.clone();
    assert_eq!(run.step(&GameAction::Choose(purify.id)), Err(RunError::InvalidAction));
    assert!(run == before, "rejected disabled Purify leaves the Run unchanged");
}

#[test]
fn m28_cleric_leave() {
    let (mut run, node) = before_event(Character::Watcher, 1, EventId::Cleric, 37);
    run.state.gold = 35;
    run.step(&GameAction::Move(node)).unwrap();
    let before = (run.state.gold, run.state.hp, run.state.deck.clone());
    choose_event(&mut run, 2).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.gold, run.state.hp, run.state.deck), before);
}

#[test]
fn m28_sssserpent_agree() {
    let mut run = at_event(Character::Ironclad, 1, EventId::Ssssserpent, 38);
    let gold = run.state.gold;
    choose_event(&mut run, 0).unwrap();
    assert_eq!((run.state.gold, count(&run, CardId::Doubt)), (gold + 175, 1));
    leave(&mut run);

    let (mut run, node) = before_event(Character::Silent, 1, EventId::Ssssserpent, 39);
    run.state.relics.push(RelicId::Omamori);
    run.state.relic_counters.insert(RelicId::Omamori, 1);
    run.step(&GameAction::Move(node)).unwrap();
    let gold = run.state.gold;
    choose_event(&mut run, 0).unwrap();
    assert_eq!((run.state.gold, count(&run, CardId::Doubt)), (gold + 175, 0), "Omamori prevents Doubt without cancelling gold");
}

#[test]
fn m28_sssserpent_leave() {
    let mut run = at_event(Character::Defect, 1, EventId::Ssssserpent, 40);
    let before = (run.state.gold, run.state.deck.clone());
    choose_event(&mut run, 1).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.gold, run.state.deck), before);
}

#[test]
fn m28_wing_statue_pray() {
    let mut run = at_event(Character::Ironclad, 1, EventId::WingStatue, 41);
    set_deck(&mut run, &[(CardId::RitualDagger, 0, 12), (CardId::AscendersBane, 0, 0)]);
    run.state.hp = 40;
    run.state.relics.push(RelicId::Torii);
    let before_deck = run.state.deck.clone();
    choose_event(&mut run, 0).unwrap();
    assert_eq!(run.state.hp, 33, "player-owned NORMAL damage 7 is unchanged by Torii's 2–5 rule");
    select_first(&mut run);
    assert_eq!(run.state.deck.len(), 1);
    assert_eq!(run.state.deck[0].id, CardId::AscendersBane);
    assert_eq!(before_deck.len(), 2);
    leave(&mut run);
}

#[test]
fn m28_wing_statue_softlock_001_empty_grid() {
    let mut run = at_event(Character::Silent, 1, EventId::WingStatue, 42);
    set_deck(&mut run, &[(CardId::AscendersBane, 0, 0)]);
    run.state.hp = 30;
    let before_deck = run.state.deck.clone();
    choose_event(&mut run, 0).unwrap();
    assert_eq!(run.state.hp, 23);
    assert_eq!(run.state.deck, before_deck);
    assert!(run.state.pending.is_none(), "SOFTLOCK-001 escape skips the empty grid");
    leave(&mut run);
}

#[test]
fn m28_wing_statue_destroy() {
    // Bash+ has printed base damage 10; Ritual Dagger reads its live misc-backed base damage. [ER:266; PC:56]
    assert_eq!(crate::combat::static_card_face(CardId::Bash, 1).unwrap().base_damage, Some(10));
    for deck in [
        vec![(CardId::Bash, 1, 0), (CardId::AscendersBane, 0, 0)],
        vec![(CardId::RitualDagger, 0, 10), (CardId::AscendersBane, 0, 0)],
    ] {
        let (mut run, node) = before_event(Character::Ironclad, 1, EventId::WingStatue, 43);
        set_deck(&mut run, &deck);
        run.step(&GameAction::Move(node)).unwrap();
        // Review-v1 source check (PM ruling): GoldenWing.buttonEffect draws the 50–80 at Destroy;
        // construction captures only the eligibility flag.
        assert_eq!(event_captured_values(&run), vec![1], "no construction gold draw");
        let destroy = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 });
        assert_eq!(destroy.status, RoomChoiceStatus::Offered, "baseDamage ≥ 10 enables Destroy");
        // Eligibility is fixed at construction.
        run.state.deck[0].id = CardId::Strike;
        let current_gold = run.state.gold + 20;
        run.state.gold = current_gold;
        choose_event(&mut run, 1).unwrap();
        assert!((current_gold + 50..=current_gold + 80).contains(&run.state.gold), "choice-time 50–80 gold: {}", run.state.gold - current_gold);
        leave(&mut run);
    }

    let (mut run, node) = before_event(Character::Ironclad, 1, EventId::WingStatue, 44);
    run.step(&GameAction::Move(node)).unwrap();
    let destroy = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 });
    assert_eq!(destroy.status, RoomChoiceStatus::Disabled, "starter attacks have base damage below 10");
    let before = run.clone();
    assert_eq!(run.step(&GameAction::Choose(destroy.id)), Err(RunError::InvalidAction));
    assert!(run == before, "rejected disabled Destroy leaves the Run unchanged");
}

#[test]
fn m28_wing_statue_leave() {
    let mut run = at_event(Character::Watcher, 1, EventId::WingStatue, 45);
    let before = (run.state.gold, run.state.hp, run.state.deck.clone());
    choose_event(&mut run, 2).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.gold, run.state.hp, run.state.deck), before);
}

#[test]
fn m28_world_of_goop_recover() {
    let mut run = at_event(Character::Defect, 1, EventId::WorldOfGoop, 46);
    run.state.hp = 20;
    run.state.gold = 4;
    choose_event(&mut run, 0).unwrap();
    assert_eq!((run.state.hp, run.state.gold), (9, 79), "owned damage 11 then gain 75 gold");
    leave(&mut run);
}

#[test]
fn m28_world_of_goop_leave() {
    let mut seen_49 = [false; 30];
    let mut seen_50 = [false; 31];
    for seed in 0..=u8::MAX {
        for entry_gold in [19, 20, 49, 50] {
            let (mut run, node) = before_event(Character::Silent, 1, EventId::WorldOfGoop, seed);
            run.state.gold = entry_gold;
            run.step(&GameAction::Move(node)).unwrap();
            let loss = event_captured_values(&run)[0];
            match entry_gold {
                19 => assert_eq!(loss, 19, "G=19 caps every 20–50 draw"),
                20 => assert_eq!(loss, 20, "G=20 caps every 20–50 draw"),
                49 => { assert!((20..=49).contains(&loss)); seen_49[(loss - 20) as usize] = true; }
                50 => { assert!((20..=50).contains(&loss)); seen_50[(loss - 20) as usize] = true; }
                _ => unreachable!(),
            }
            assert_eq!(view(&run).choices.iter().filter(|choice| matches!(choice.kind, RoomChoiceKind::EventOption { .. })).count(), 2,
                "Gather and paid Leave are the only options");
            let current_gold = entry_gold + 100;
            run.state.gold = current_gold;
            choose_event(&mut run, 1).unwrap();
            assert_eq!(run.state.gold, current_gold - loss, "L is fixed at construction for G={entry_gold}");
            leave(&mut run);
        }
    }
    assert!(seen_49[0] && seen_49[29], "G=49 reaches losses 20 and 49");
    assert!(seen_50[0] && seen_50[30], "G=50 reaches losses 20 and 50");
}

/// Review-v1 M2: Shining Light's player-owned J(.20M) is in Torii's 2–5 band for M 8–27, and the shared
/// out-of-combat law applies Torii to it (ER :427-435): M=25 → J(5)=5 → 1; M=28 → J(5.6)=6 is unchanged.
#[test]
fn m28_shining_light_torii_small_max_hp() {
    for (m, loss) in [(25_u32, 1_u32), (27, 1), (28, 6)] {
        let (mut run, node) = before_event(Character::Ironclad, 1, EventId::ShiningLight, 30);
        run.state.relics.push(RelicId::Torii);
        run.state.max_hp = m;
        run.state.hp = m;
        run.step(&GameAction::Move(node)).unwrap();
        choose_event(&mut run, 0).unwrap();
        assert_eq!(run.state.hp, m - loss, "Torii at M={m}");
        leave(&mut run);
    }
}

/// Review-v1 M3: Dead Adventurer's `eliteTrigger` reaches elite-only combat relics through the shared
/// combat-start seam: Preserved Insect caps every enemy at T(.75 max HP) and Sling of Courage gives
/// 2 Strength. The non-elite Mushrooms fight gets neither.
#[test]
fn m28_event_combat_dead_adventurer_elite_relics() {
    fn combat(run: &Run) -> crate::combat::CombatPublic { run.state.combat.as_ref().unwrap().public_full().unwrap() }
    fn strength(run: &Run) -> i32 {
        combat(run).player.powers.iter().find(|power| power.id.0 == "strength").map_or(0, |power| power.amount)
    }
    for enemy in [EncounterId::ThreeSentries, EncounterId::GremlinNob, EncounterId::LagavulinEvent] {
        let mut found = false;
        for seed in 0..=u8::MAX {
            let mut run = event_at_floor(Character::Ironclad, EventId::DeadAdventurer, seed, 7);
            if run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.encounter != Some(enemy) { continue }
            run.state.relics.extend([RelicId::PreservedInsect, RelicId::SlingOfCourage]);
            if choose_event(&mut run, 0).is_err() || run.state.active_combat.is_none() { continue }
            assert_eq!(strength(&run), 2, "Sling of Courage in the {enemy:?} ambush");
            for e in combat(&run).enemies { assert_eq!(e.hp, e.max_hp * 3 / 4, "Preserved Insect in the {enemy:?} ambush"); }
            found = true;
            break;
        }
        assert!(found, "no fixed seed reached the {enemy:?} ambush");
    }
    let (mut run, node) = before_event(Character::Silent, 1, EventId::Mushrooms, 18);
    run.state.floor = 6; // entry adds one: floor 7 (PM ruling m28f Q2)
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|map_node| map_node.id == node).unwrap().floor = 7;
    run.state.relics.extend([RelicId::PreservedInsect, RelicId::SlingOfCourage]);
    run.step(&GameAction::Move(node)).unwrap();
    choose_event(&mut run, 0).unwrap();
    assert_eq!(strength(&run), 0, "the Mushrooms fight is not elite");
    for e in combat(&run).enemies { assert_eq!(e.hp, e.max_hp, "Preserved Insect is elite-only"); }
}
