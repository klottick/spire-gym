//! M34a event row parameter publication (census v1).
use super::super::*;
use super::m28_event_core::{before_event, row, view};
use crate::content::{Character, EventId};
use std::collections::{BTreeSet, HashSet, VecDeque};

fn params(run: &Run, stage: u8, option: u8) -> [Option<i32>; 3] {
    let id = view(run).choices.into_iter().find(|choice| matches!(choice.kind,
        RoomChoiceKind::EventOption { stage: s, option: o }
            | RoomChoiceKind::EventCard { stage: s, option: o, .. }
            | RoomChoiceKind::EventPotion { stage: s, option: o, .. }
            | RoomChoiceKind::EventRelic { stage: s, option: o, .. }
            if s == stage && o == option)).unwrap_or_else(|| panic!("no event row stage {stage} option {option}" )).id;
    run.decision().unwrap().room_params.as_ref().unwrap().rows.iter()
        .find(|(choice, _)| *choice == id).map_or([None; 3], |(_, values)| *values)
}

fn check_screen(run: &Run) {
    let RoomViewKind::Event(event) = view(run).kind else { return };
    for choice in view(run).choices {
        let Some((stage, option)) = (match choice.kind {
            RoomChoiceKind::EventOption { stage, option }
            | RoomChoiceKind::EventCard { stage, option, .. }
            | RoomChoiceKind::EventPotion { stage, option, .. }
            | RoomChoiceKind::EventRelic { stage, option, .. } => Some((stage, option)),
            _ => None,
        }) else { continue };
        let expected: BTreeSet<u8> = crate::run::rooms::standard::events::PARAM_SEMANTICS.iter()
            .filter(|m| m.event == event && m.stage == stage && m.option == option)
            .filter(|m| match (m.event,m.stage,m.option) {
                (EventId::WomanInBlue,0,3)=>run.state.ascension>=15,
                (EventId::MindBloom,0,2)=>run.state.room_loop.as_ref().and_then(|loop_state|loop_state.standard_event.as_ref())
                    .is_some_and(|frame|frame.captured.values.first()==Some(&1)),
                _=>true,
            })
            .map(|m| m.index).collect();
        let actual: BTreeSet<u8> = params_for_choice(run, choice.id).iter().enumerate()
            .filter_map(|(i, value)| value.map(|_| i as u8)).collect();
        assert_eq!(actual, expected, "{event:?} stage {stage} option {option}");
    }
}

fn params_for_choice(run: &Run, id: ChoiceId) -> [Option<i32>; 3] {
    run.decision().unwrap().room_params.as_ref().unwrap().rows.iter()
        .find(|(choice, _)| *choice == id).map_or([None; 3], |(_, values)| *values)
}

#[test]
fn every_published_event_param_has_one_meaning() {
    use crate::run::rooms::standard::events::{body, PARAM_SEMANTICS};
    let mut keys = Vec::new();
    for meaning in PARAM_SEMANTICS {
        let key = (meaning.event, meaning.stage, meaning.option, meaning.index);
        assert!(!keys.contains(&key), "duplicate {meaning:?}");
        keys.push(key);
        assert!(body(meaning.event).is_some(), "meaning without a body: {meaning:?}");
    }
    // 4200 in Colosseum's opening prose is intentionally excluded; every remaining census field is listed.
    // Final review I1/m1: Face Trader Touch (damage, gold) and Cursed Tome Stop (3 HP) are printed on their rows.
    assert_eq!(PARAM_SEMANTICS.len(), 92);

    // The complete standard catalog also guards the no-body boundary.
    for definition in crate::content::Catalog::standard().events() {
        if body(definition.id).is_none() {
            assert!(!PARAM_SEMANTICS.iter().any(|m| m.event == definition.id));
        }
    }

    let events: Vec<EventId> = crate::content::Catalog::standard().events().iter()
        .filter(|definition| body(definition.id).is_some())
        .map(|definition| definition.id).collect();
    for event in events {
        let definition = crate::content::Catalog::standard().event(event).unwrap();
        let act = (1..=3).find(|act| definition.acts & (1 << act) != 0).unwrap();
        for ascension in [0u8, 15] {
            if event == EventId::NoteForYourself && ascension >= 15 { continue; }
            for seed in 0..3u8 {
                let run = if event == EventId::Colosseum {
                    // This event's eligibility gate requires an event node below map row 8.
                    let mut run = super::m28_events_act2::colosseum(Character::Ironclad, seed);
                    run.state.ascension = ascension;
                    run
                } else {
                    let (mut run, node) = before_event(Character::Ironclad, act, event, seed);
                    run.state.ascension = ascension;
                    run.state.floor = run.state.floor.max(20);
                    let floor = run.state.floor + 1;
                    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|candidate| candidate.id == node).unwrap().floor = floor;
                    run.state.gold = 300;
                    if event == EventId::Nloth && run.state.relics.len() < 2 {
                        run.state.relics.push(crate::content::RelicId::Anchor);
                    }
                    if event == EventId::FountainOfCleansing { run.state.deck[0].id = crate::content::CardId::Regret; }
                    if event == EventId::MoaiHead { run.state.hp = run.state.max_hp / 2; }
                    run.step(&GameAction::Move(node)).unwrap_or_else(|error| panic!("{event:?} A{ascension} seed {seed}: {error:?}"));
                    run
                };
                let mut queue = VecDeque::from([(run, 0u8)]);
                while let Some((current, depth)) = queue.pop_front() {
                    check_screen(&current);
                    if depth >= 12 { continue; }
                    let current_view = view(&current);
                    let current_stage = current.state.room_loop.as_ref().and_then(|loop_state| loop_state.standard_event.as_ref()).map(|frame| frame.stage);
                    let RoomViewKind::Event(current_event) = current_view.kind else { continue };
                    for choice in current_view.choices {
                        if !matches!(choice.kind, RoomChoiceKind::EventOption { .. } | RoomChoiceKind::EventCard { .. } | RoomChoiceKind::EventPotion { .. } | RoomChoiceKind::EventRelic { .. }) { continue; }
                        let mut next = current.clone();
                        if next.step(&GameAction::Choose(choice.id)).is_ok()
                            && matches!(view(&next).kind, RoomViewKind::Event(e) if e == current_event)
                            && next.state.room_loop.as_ref().and_then(|loop_state| loop_state.standard_event.as_ref()).map(|frame| frame.stage) != current_stage
                        {
                            queue.push_back((next, depth + 1));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn we_meet_again_publishes_captured_gold_gift() {
    for seed in 0..12u8 {
        let (mut run, node) = before_event(Character::Ironclad, 1, EventId::WeMeetAgain, seed);
        run.state.gold = 99;
        run.step(&GameAction::Move(node)).unwrap();
        let shown = params(&run, 0, 1)[0].expect("captured gift is public") as u32;
        assert!((50..=99).contains(&shown));
        let before = run.state.gold;
        let id = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 }).id;
        run.step(&GameAction::Choose(id)).unwrap();
        assert_eq!(before - run.state.gold, shown);
    }
}

#[test]
fn event_params_equal_applied_values() {
    use crate::run::rooms::standard::events::PARAM_SEMANTICS;

    fn is_quantity(meaning: &str) -> bool {
        matches!(meaning,
            "gold_gained" | "gold_lost" | "gold_cost" | "gold_gift_amount"
            | "damage_taken" | "hp_lost" | "max_hp_gained" | "max_hp_lost" | "heal_amount"
            | "upgrade_count" | "transform_count" | "apparition_count" | "bite_count"
            | "madness_count" | "card_count" | "potion_count")
    }

    fn quantity_event(event: EventId, ascension: u8, seed: u8) -> Run {
        let definition = crate::content::Catalog::standard().event(event).unwrap();
        let act = (1..=3).find(|act| definition.acts & (1 << act) != 0).unwrap();
        let (mut run, node) = before_event(Character::Ironclad, act, event, seed);
        run.state.ascension = ascension;
        run.state.max_hp = 500;
        run.state.hp = 250;
        run.state.gold = 1_000;
        run.state.relics.retain(|relic| *relic == crate::content::RelicId::BurningBlood);
        let mut potion_slots = crate::run::potion_base_slots(ascension);
        if event == EventId::WomanInBlue && ascension >= 11 {
            run.state.relics.push(crate::content::RelicId::PotionBelt);
            potion_slots += 2;
        }
        run.state.potions = vec![None; potion_slots];
        if event == EventId::Vampires { run.state.relics.push(crate::content::RelicId::BloodVial); }
        if event == EventId::MoaiHead { run.state.relics.push(crate::content::RelicId::GoldenIdol); }
        run.state.floor = run.state.floor.max(20);
        let floor = run.state.floor + 1;
        run.state.map.as_mut().unwrap().nodes.iter_mut().find(|candidate| candidate.id == node).unwrap().floor = floor;
        run.step(&GameAction::Move(node)).unwrap_or_else(|error| panic!("{event:?} A{ascension} seed {seed}: {error:?}"));
        run
    }

    fn publish_stage(run: &mut Run, stage: u8) {
        crate::run::rooms::standard::event_frame::set_stage(run, stage).unwrap();
        crate::run::rooms::standard::event_frame::publish(run).unwrap();
    }

    fn total_upgrades(run: &Run) -> u32 { run.state.deck.iter().map(|card| card.upgrades).sum() }
    fn card_count(run: &Run, id: crate::content::CardId) -> usize { run.state.deck.iter().filter(|card| card.id == id).count() }

    fn finish_transform_selection(run: &mut Run, expected: usize) -> usize {
        let pending = run.state.pending.as_ref().expect("quantity opens card selection");
        assert_eq!(pending.selection.required, expected, "selection count is the published quantity");
        let before: HashSet<u64> = run.state.deck.iter().map(|card| card.instance).collect();
        for _ in 0..expected {
            let id = run.state.pending.as_ref().unwrap().selection.candidates[0].id;
            run.step(&GameAction::Select(id)).unwrap();
        }
        let replaced = run.state.deck.iter().filter(|card| !before.contains(&card.instance)).count();
        assert_eq!(replaced, expected, "each selected card was transformed into a fresh instance");
        replaced
    }

    fn assert_applied_row(mut run: Run, meaning: crate::run::rooms::standard::events::ParamMeaning) -> u32 {
        publish_stage(&mut run, meaning.stage);
        let shown = params(&run, meaning.stage, meaning.option)[usize::from(meaning.index)]
            .unwrap_or_else(|| panic!("missing {:?} on {meaning:?}", run.state.ascension)) as u32;
        let event_view = view(&run);
        let choice = event_view.choices.iter().find(|choice| matches!(choice.kind,
            RoomChoiceKind::EventOption { stage, option }
                | RoomChoiceKind::EventCard { stage, option, .. }
                | RoomChoiceKind::EventPotion { stage, option, .. }
                | RoomChoiceKind::EventRelic { stage, option, .. }
                if stage == meaning.stage && option == meaning.option))
            .unwrap_or_else(|| panic!("missing option row for {meaning:?}"));
        assert_eq!(choice.status, RoomChoiceStatus::Offered, "quantity option is enabled: {meaning:?}");
        let before = run.clone();
        let mut after = before.clone();
        after.step(&GameAction::Choose(choice.id)).unwrap_or_else(|error| panic!("{meaning:?}: {error:?}"));
        let actual = match meaning.meaning {
            "gold_gained" => after.state.gold - before.state.gold,
            "gold_lost" | "gold_cost" | "gold_gift_amount" => before.state.gold - after.state.gold,
            // GainMaxHp also heals by the same amount before the damage outcome runs.
            "damage_taken" => before.state.hp + after.state.max_hp.saturating_sub(before.state.max_hp) - after.state.hp,
            "hp_lost" => before.state.hp - after.state.hp,
            "max_hp_gained" => after.state.max_hp - before.state.max_hp,
            "max_hp_lost" => before.state.max_hp - after.state.max_hp,
            "heal_amount" => after.state.hp - before.state.hp,
            "upgrade_count" => total_upgrades(&after) - total_upgrades(&before),
            "apparition_count" => (card_count(&after, crate::content::CardId::Apparition) - card_count(&before, crate::content::CardId::Apparition)) as u32,
            "bite_count" => (card_count(&after, crate::content::CardId::Bite) - card_count(&before, crate::content::CardId::Bite)) as u32,
            "madness_count" => (card_count(&after, crate::content::CardId::Madness) - card_count(&before, crate::content::CardId::Madness)) as u32,
            "transform_count" => finish_transform_selection(&mut after, shown as usize) as u32,
            "card_count" if meaning.event == EventId::Designer && meaning.option == 1 => total_upgrades(&after) - total_upgrades(&before),
            "card_count" if meaning.event == EventId::Designer && meaning.option == 3 => finish_transform_selection(&mut after, shown as usize) as u32,
            "card_count" => view(&after).choices.iter().filter(|choice| choice.kind == RoomChoiceKind::CardReward).count() as u32,
            "potion_count" => {
                let ids: Vec<_> = view(&after).choices.iter().filter_map(|choice|
                    matches!(choice.kind, RoomChoiceKind::Potion(_)).then_some(choice.id)).collect();
                assert_eq!(ids.len(), shown as usize, "the event opened the advertised potion rows");
                for id in ids { after.step(&GameAction::Choose(id)).unwrap(); }
                (after.state.potions.iter().filter(|potion| potion.is_some()).count()
                    - before.state.potions.iter().filter(|potion| potion.is_some()).count()) as u32
            }
            _ => panic!("not a quantity param: {meaning:?}"),
        };
        assert_eq!(actual, shown, "{meaning:?} A{}", before.state.ascension);
        actual
    }

    fn assert_wheel_result(ascension: u8) {
        for seed in 0..=u8::MAX {
            let mut run = quantity_event(EventId::WheelOfChange, ascension, seed);
            let spin = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 }).id;
            run.step(&GameAction::Choose(spin)).unwrap();
            if view(&run).choices.iter().any(|choice| choice.kind == RoomChoiceKind::EventOption { stage: 1, option: 5 }) {
                let meaning = PARAM_SEMANTICS.iter().find(|meaning| meaning.event == EventId::WheelOfChange).unwrap();
                assert_applied_row(run, *meaning);
                return;
            }
        }
        panic!("no Wheel of Change damage result found across all test seeds at A{ascension}");
    }

    fn assert_colosseum_gold(ascension: u8) {
        let mut run = super::m28_events_act2::colosseum(Character::Ironclad, 26);
        run.state.ascension = ascension;
        run.state.gold = 1_000;
        run.state.potions = vec![None; crate::run::potion_base_slots(ascension)];
        let first = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 }).id;
        run.step(&GameAction::Choose(first)).unwrap();
        super::m28_world::win(&mut run);
        let meaning = PARAM_SEMANTICS.iter().find(|meaning| meaning.event == EventId::Colosseum).unwrap();
        let shown = params(&run, meaning.stage, meaning.option)[usize::from(meaning.index)].unwrap() as u32;
        assert_eq!(shown, 100);
        let second = row(&run, &RoomChoiceKind::EventOption { stage: 1, option: 1 }).id;
        run.step(&GameAction::Choose(second)).unwrap();
        super::m28_world::win(&mut run);
        let reward = row(&run, &RoomChoiceKind::Gold(shown));
        let before_gold = run.state.gold;
        run.step(&GameAction::Choose(reward.id)).unwrap();
        assert_eq!(run.state.gold - before_gold, shown, "Colosseum's advertised reward is applied after the fight");
    }

    let meanings: Vec<_> = PARAM_SEMANTICS.iter().copied().filter(|meaning| is_quantity(meaning.meaning)).collect();
    let expected: HashSet<_> = meanings.iter().map(|meaning| (meaning.event, meaning.stage, meaning.option, meaning.index)).collect();
    assert_eq!(meanings.len(), 76, "the E1/E2 census (v2 corrections: Face Trader Touch, Cursed Tome Stop) has 76 applied quantity params");
    assert_eq!(meanings.len(), expected.len(), "each quantity parameter key is unique");
    let mut covered = HashSet::new();

    for ascension in [0, 15] {
        for meaning in &meanings {
            if meaning.event == EventId::WomanInBlue && meaning.option == 3 && ascension < 15 { continue; }
            if meaning.event == EventId::Colosseum {
                assert_colosseum_gold(ascension);
                covered.insert((meaning.event, meaning.stage, meaning.option, meaning.index));
                continue;
            }
            if meaning.event == EventId::WheelOfChange {
                assert_wheel_result(ascension);
                covered.insert((meaning.event, meaning.stage, meaning.option, meaning.index));
                continue;
            }
            let mut checked = false;
            for seed in 0..=u8::MAX {
                let run = quantity_event(meaning.event, ascension, seed);
                let mut stage_probe = run.clone();
                publish_stage(&mut stage_probe, meaning.stage);
                let probe_view = view(&stage_probe);
                if !probe_view.choices.iter().any(|choice| matches!(choice.kind,
                    RoomChoiceKind::EventOption { stage, option }
                        | RoomChoiceKind::EventCard { stage, option, .. }
                        | RoomChoiceKind::EventPotion { stage, option, .. }
                        | RoomChoiceKind::EventRelic { stage, option, .. }
                        if stage == meaning.stage && option == meaning.option && choice.status == RoomChoiceStatus::Offered)) { continue; }
                if params(&stage_probe, meaning.stage, meaning.option)[usize::from(meaning.index)].is_none() { continue; }
                assert_applied_row(run, *meaning);
                checked = true;
                break;
            }
            assert!(checked, "could not expose and apply {meaning:?} at A{ascension}");
            covered.insert((meaning.event, meaning.stage, meaning.option, meaning.index));
        }
    }
    assert_eq!(covered, expected, "every quantity param with applied_matches=true has a state or reward-screen delta witness at both ascension endpoints");
}

#[test]
fn knowing_skull_publishes_growing_hp_costs() {
    let (mut run, node) = before_event(Character::Ironclad, 2, EventId::KnowingSkull, 3);
    run.state.hp = 60;
    run.step(&GameAction::Move(node)).unwrap();
    for expected_cost in [6u32, 7] {
        assert_eq!(params(&run, 0, 1)[0], Some(expected_cost as i32));
        let before_hp = run.state.hp;
        let before_gold = run.state.gold;
        let id = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 }).id;
        run.step(&GameAction::Choose(id)).unwrap();
        assert_eq!(before_hp - run.state.hp, expected_cost);
        assert_eq!(run.state.gold - before_gold, 90);
    }
    assert_eq!(params(&run, 0, 1)[0], Some(8));
}

#[test]
fn chance_and_ascension_params_match_independent_formulas() {
    use crate::run::rooms::standard::{event_frame::EventCaptures, events::body};
    let (run, _) = before_event(Character::Ironclad, 1, EventId::BigFish, 21);
    let captures = EventCaptures::default();
    for ascension in [0u8, 15] {
        let mut state = run.state.clone();
        state.ascension = ascension;
        let high = ascension >= 15;
        let dead = body(EventId::DeadAdventurer).unwrap().params.unwrap();
        for stage in 0..=2u8 {
            assert_eq!(dead(&state, &captures, stage, 0)[0], Some((if high { 35 } else { 25 }) + 25 * i32::from(stage)));
        }
        let scrap = body(EventId::ScrapOoze).unwrap().params.unwrap();
        for stage in 0..=8u8 {
            let shown = scrap(&state, &captures, stage, 0);
            assert_eq!(shown[0], Some((if high { 5 } else { 3 }) + i32::from(stage)));
            assert_eq!(shown[1], Some(25 + 10 * i32::from(stage)), "Scrap Ooze A{ascension} stage {stage}");
        }
        let mausoleum = body(EventId::Mausoleum).unwrap().params.unwrap();
        assert_eq!(mausoleum(&state, &captures, 0, 0)[0], Some(if high { 100 } else { 50 }));
        let wing = body(EventId::WingStatue).unwrap().params.unwrap();
        assert_eq!(wing(&state, &captures, 0, 1), [Some(50), Some(80), None], "gold reward bounds remain formula checks");
        assert_eq!(wing(&state, &captures, 0, 2), [Some(10), None, None], "the destroy threshold remains a formula check");
        let woman = body(EventId::WomanInBlue).unwrap().params.unwrap();
        assert_eq!(woman(&state, &captures, 0, 3)[0], if high { Some(((state.max_hp as f32)*0.05_f32).ceil() as i32) } else { None });
        let wheel = body(EventId::WheelOfChange).unwrap().params.unwrap();
        assert_eq!(wheel(&state, &captures, 1, 5)[0], Some(((state.max_hp as f32)*if high { 0.15_f32 } else { 0.10_f32 }) as i32));
    }
}

/// Final review I1: the Touch row prints "Lose {max HP / 10, min 1} HP, gain {75, or 50 at A15} Gold" (FaceTrader.java:48).
#[test]
fn face_trader_touch_publishes_damage_and_gold() {
    let definition = crate::content::Catalog::standard().event(EventId::FaceTrader).unwrap();
    let act = (1..=3).find(|act| definition.acts & (1 << act) != 0).unwrap();
    for (ascension, max_hp, damage, gold) in [(0u8, 80u32, 8i32, 75i32), (15, 80, 8, 50), (0, 9, 1, 75), (20, 123, 12, 50)] {
        let (mut run, node) = before_event(Character::Ironclad, act, EventId::FaceTrader, 4);
        run.state.ascension = ascension;
        run.state.max_hp = max_hp;
        run.state.hp = max_hp;
        run.step(&GameAction::Move(node)).unwrap();
        assert_eq!(params(&run, 0, 0), [Some(damage), Some(gold), None], "A{ascension} max HP {max_hp}");
        let (hp, before_gold) = (run.state.hp, run.state.gold);
        run.step(&GameAction::Choose(row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 }).id)).unwrap();
        assert_eq!((hp - run.state.hp, run.state.gold - before_gold), (damage as u32, gold as u32), "Touch applies what it shows");
    }
}

/// Final review m1: the final page's Stop row prints "[Stop] Lose 3 HP." (localization Cursed Tome OPTIONS[4]).
#[test]
fn cursed_tome_stop_publishes_hp_loss() {
    let (mut run, node) = before_event(Character::Ironclad, 2, EventId::CursedTome, 5);
    run.step(&GameAction::Move(node)).unwrap();
    crate::run::rooms::standard::event_frame::set_stage(&mut run, 4).unwrap();
    crate::run::rooms::standard::event_frame::publish(&mut run).unwrap();
    assert_eq!(params(&run, 4, 1), [Some(3), None, None]);
    let hp = run.state.hp;
    run.step(&GameAction::Choose(row(&run, &RoomChoiceKind::EventOption { stage: 4, option: 1 }).id)).unwrap();
    assert_eq!(hp - run.state.hp, 3);
}
