//! M28d review-v1 fix round: the A15 band of every M28d body and Golden Shrine, the act gates of the
//! one-time rows, and the unupgraded transform (PM source check m28d/source-check-act-dependence-v1,
//! binding). Each value is pinned at the A14/A15 boundary. StandardA0 runs A0 today; these tests set
//! `RunState::ascension` on the diagnostic event Run before the event is built.
use super::super::*;
use super::m28_event_core::{before_event, count, leave, row, view};
use crate::content::{CardId, CardRarity, Character, EncounterId, EventId, RelicId};
use crate::run::{GameAction, RoomChoiceKind, RoomChoiceStatus, RoomViewKind, Run, RunTransition};
use rooms::standard::event_exec::Outcome;
use rooms::standard::{event_pool, events};

fn frame(run: &Run) -> &rooms::standard::event_frame::StandardEventFrame {
    run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap()
}

/// The event built at `ascension`; `setup` runs on the staged Run before the move.
fn entered(event: EventId, act: u8, ascension: u8, seed: u8, setup: impl FnOnce(&mut Run, NodeId)) -> Run {
    let (mut run, node) = before_event(Character::Ironclad, act, event, seed);
    run.state.ascension = ascension;
    setup(&mut run, node);
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(event));
    run
}

fn entered_character(character: Character, event: EventId, act: u8, ascension: u8, seed: u8,
    setup: impl FnOnce(&mut Run, NodeId)) -> Run {
    let (mut run, node) = before_event(character, act, event, seed);
    run.state.ascension = ascension;
    setup(&mut run, node);
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(event));
    run
}

fn max_hp(m: u32) -> impl FnOnce(&mut Run, NodeId) { move |run, _| { run.state.max_hp = m; run.state.hp = m; } }

fn choose(run: &mut Run, stage: u8, option: u8) -> RunTransition {
    let id = row(run, &RoomChoiceKind::EventOption { stage, option }).id;
    run.step(&GameAction::Choose(id)).unwrap()
}

/// The body's outcome list for (stage, option) on the live frame (bodies are pure).
fn outcomes(run: &Run, stage: u8, option: u8) -> Vec<Outcome> {
    let frame = frame(run);
    (events::body(frame.event).unwrap().choose)(&run.state, &frame.captured, stage, option).unwrap()
}

#[test]
fn m28_a15_golden_shrine_pray() {
    // GoldShrine constructor: Pray 100, or 50 at A15; Desecrate 275 at every ascension.
    for (ascension, pray) in [(14_u8, 100_u32), (15, 50)] {
        let mut run = entered(EventId::GoldenShrine, 1, ascension, 60, |_, _| {});
        let gold = run.state.gold;
        choose(&mut run, 0, 0);
        assert_eq!(run.state.gold, gold + pray, "A{ascension} Pray");
        leave(&mut run);
        let mut run = entered(EventId::GoldenShrine, 1, ascension, 61, |_, _| {});
        let gold = run.state.gold;
        choose(&mut run, 0, 1);
        assert_eq!(run.state.gold, gold + 275, "A{ascension} Desecrate");
    }
}

#[test]
fn m28_a15_match_and_keep_second_curse() {
    // GremlinMatchGame.initializeCards: at A15 a second random curse replaces the Uncommon colourless card.
    let colorless = crate::content::pools::cards(crate::content::pools::CardPool::Colourless { rarity: Some(CardRarity::Uncommon) }).unwrap();
    let curses = crate::content::pools::cards(crate::content::pools::CardPool::Curse).unwrap();
    for (ascension, curse_cards, colorless_cards) in [(14_u8, 2, 2), (15, 4, 0)] {
        for seed in 70..74_u8 {
            let run = entered(EventId::MatchAndKeep, 1, ascension, seed, |_, _| {});
            let ids = &frame(&run).captured.card_ids;
            assert_eq!(ids.len(), 12);
            assert_eq!(ids.iter().filter(|id| curses.contains(id)).count(), curse_cards, "A{ascension} curses: {ids:?}");
            assert_eq!(ids.iter().filter(|id| colorless.contains(id)).count(), colorless_cards, "A{ascension} colourless: {ids:?}");
            assert_eq!(ids.iter().filter(|id| **id == CardId::Bash).count(), 2, "starter pair unchanged");
            assert_eq!(view(&run).choices.len(), 12, "the board is unchanged");
        }
    }
}

#[test]
fn m28_a15_match_and_keep_still_five_attempts() {
    let mut run = entered(EventId::MatchAndKeep, 1, 15, 33, |_, _| {});
    let ids = frame(&run).captured.card_ids.clone();
    let (first, second) = (0..12).flat_map(|i| (0..12).map(move |j| (i, j))).find(|(i, j)| i != j && ids[*i] != ids[*j]).unwrap();
    for attempt in 0..5_u8 {
        for position in [first, second] {
            let id = view(&run).choices.into_iter().find(|c| matches!(c.kind, RoomChoiceKind::EventCard { stage, option, .. } if stage == attempt && usize::from(option) == position)).unwrap().id;
            run.step(&GameAction::Choose(id)).unwrap();
        }
    }
    assert_eq!(frame(&run).stage, u8::MAX, "five attempts at A15");
}

#[test]
fn m28_a15_wheel_of_change_hp_loss() {
    // GremlinWheelGame constructor: HP_LOSS T(.10M), or T(.15M) at A15. M=86: 8 / 12.
    for (ascension, loss) in [(14_u8, 8_u32), (15, 12)] {
        let mut found = false;
        for seed in 0..100_u8 {
            let mut run = entered(EventId::WheelOfChange, 1, ascension, seed, max_hp(86));
            choose(&mut run, 0, 0);
            if frame(&run).captured.values[2] != 5 { continue }
            choose(&mut run, 1, 5);
            assert_eq!(run.state.hp, 86 - loss, "A{ascension} Wheel HP loss");
            found = true;
            break;
        }
        assert!(found);
    }
}

#[test]
fn m28_a15_face_trader_touch() {
    // FaceTrader constructor: Touch 75 gold, or 50 at A15; damage max(1, M/10) at every ascension.
    for (ascension, gold_gain) in [(14_u8, 75_u32), (15, 50)] {
        let mut run = entered(EventId::FaceTrader, 1, ascension, 62, max_hp(80));
        let gold = run.state.gold;
        choose(&mut run, 0, 0);
        assert_eq!((run.state.gold, run.state.hp), (gold + gold_gain, 72), "A{ascension} Touch");
    }
}

#[test]
fn m28_a15_lab_potions() {
    // Lab.buttonEffect: three potions, or two at A15.
    for (ascension, potions) in [(14_u8, 3_usize), (15, 2)] {
        let mut run = entered(EventId::Lab, 1, ascension, 63, |_, _| {});
        choose(&mut run, 0, 0);
        assert_eq!(view(&run).choices.iter().filter(|c| matches!(c.kind, RoomChoiceKind::Potion(_))).count(), potions, "A{ascension} Lab");
    }
}

#[test]
fn m28_a15_dead_adventurer_thresholds() {
    // DeadAdventurer constructor/randomReward: fight thresholds 25/50/75, or 35/60/85 at A15.
    let floor7 = |run: &mut Run, node: NodeId| {
        run.state.floor = 6; // entry adds one: floor 7 (PM ruling m28f Q2)
        run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == node).unwrap().floor = 7;
    };
    for (ascension, thresholds) in [(14_u8, [25_u32, 50, 75]), (15, [35, 60, 85])] {
        let run = entered(EventId::DeadAdventurer, 1, ascension, 64, floor7);
        for (stage, threshold) in thresholds.into_iter().enumerate() {
            let list = outcomes(&run, stage as u8, 0);
            let [Outcome::Roll { bound: 100, arms }] = list.as_slice() else { panic!("Search is one roll: {list:?}") };
            assert_eq!(arms[0].0, threshold, "A{ascension} search {stage}");
            assert!(matches!(arms[0].1.as_slice(), [Outcome::Combat { elite: true, .. }]));
        }
    }
}

#[test]
fn m28_a15_golden_idol_costs() {
    // GoldenIdolEvent constructor: T(.25M) damage and max(1,T(.08M)) max-HP loss, or T(.35M) and
    // max(1,T(.10M)) at A15. M=80: (20, 6) / (28, 8).
    for (ascension, damage, max_loss) in [(14_u8, 20_u32, 6_u32), (15, 28, 8)] {
        let mut run = entered(EventId::GoldenIdol, 1, ascension, 65, max_hp(80));
        assert_eq!(frame(&run).captured.values, vec![damage, max_loss], "A{ascension}");
        choose(&mut run, 0, 0);
        choose(&mut run, 1, 1);
        assert_eq!(run.state.hp, 80 - damage, "A{ascension} Idol damage");
        let mut run = entered(EventId::GoldenIdol, 1, ascension, 66, max_hp(80));
        choose(&mut run, 0, 0);
        choose(&mut run, 1, 2);
        assert_eq!(run.state.max_hp, 80 - max_loss, "A{ascension} Idol max-HP loss");
    }
}

#[test]
fn m28_a15_scrap_ooze_first_damage() {
    // ScrapOoze constructor: the first Reach deals 3, or 5 at A15; each failure adds 1.
    for (ascension, first) in [(14_u8, 3_u32), (15, 5)] {
        let mut run = entered(EventId::ScrapOoze, 1, ascension, 67, max_hp(80));
        choose(&mut run, 0, 0);
        assert_eq!(run.state.hp, 80 - first, "A{ascension} first Reach");
        let list = outcomes(&run, 3, 0);
        assert_eq!(list[0], Outcome::Damage(first + 3), "A{ascension} fourth Reach");
    }
}

#[test]
fn m28_a15_shining_light_damage() {
    // ShiningLight constructor: J(.20M), or J(.30M) at A15. M=80: 16 / 24.
    for (ascension, damage) in [(14_u8, 16_u32), (15, 24)] {
        let mut run = entered(EventId::ShiningLight, 1, ascension, 68, max_hp(80));
        choose(&mut run, 0, 0);
        assert_eq!(run.state.hp, 80 - damage, "A{ascension} Shining Light");
    }
}

#[test]
fn m28_a15_cleric_purify_price() {
    // Cleric constructor: Purify 50, or 75 at A15; at A15 with 50–74 gold the row shows 75 and is disabled.
    for (ascension, price, status) in [(14_u8, 50_u32, RoomChoiceStatus::Offered), (15, 75, RoomChoiceStatus::Disabled)] {
        let mut run = entered(EventId::Cleric, 1, ascension, 69, |run, _| run.state.gold = 60);
        let purify = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 });
        assert_eq!((purify.price, purify.status), (Some(price), status), "A{ascension} Purify row");
        assert_eq!(row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 }).price, Some(35), "Heal is 35 at every ascension");
        if status == RoomChoiceStatus::Offered { continue }
        run.state.gold = 75;
        crate::run::rooms::standard::event_frame::publish(&mut run).unwrap();
        let deck = run.state.deck.len();
        choose(&mut run, 0, 1);
        let candidate = run.state.pending.as_ref().unwrap().selection.candidates[0].id;
        run.step(&GameAction::Select(candidate)).unwrap();
        assert_eq!((run.state.gold, run.state.deck.len()), (0, deck - 1), "A15 Purify pays 75");
    }
}

#[test]
fn m28_cursed_tome_take_a15_plus() {
    // CursedTome constructor and Take: captured HP loss is 10 at A14, 15 at A15; the three
    // separate page choices still cost 1, 2, and 3 HP.
    for (ascension, take, total) in [(14_u8, 10_u32, 16_u32), (15, 15, 21)] {
        let mut run = entered(EventId::CursedTome, 2, ascension, 80 + ascension, |run, _| {
            run.state.hp = 80;
            run.state.max_hp = 80;
            run.state.relics.extend([RelicId::Necronomicon, RelicId::Enchiridion, RelicId::NilrysCodex]);
        });
        let old_relics = run.state.relics.len();
        choose(&mut run, 0, 0);
        choose(&mut run, 1, 0);
        choose(&mut run, 2, 0);
        choose(&mut run, 3, 0);
        assert_eq!(frame(&run).captured.values[0], take);
        choose(&mut run, 4, 0);
        assert_eq!(run.state.hp, 80 - total, "A{ascension}");
        assert_eq!(run.state.relics.len(), old_relics, "Circlet remains unclaimed on the reward screen");
        assert_eq!(view(&run).kind, RoomViewKind::Rewards);
        let circlet = row(&run, &RoomChoiceKind::Relic(RelicId::Circlet)).id;
        run.step(&GameAction::Choose(circlet)).unwrap();
        assert!(run.state.relics.contains(&RelicId::Circlet));
        assert_eq!(run.state.relics.len(), old_relics + 1);
        let proceed = row(&run, &RoomChoiceKind::Proceed).id;
        run.step(&GameAction::Choose(proceed)).unwrap();
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

#[test]
fn m28_forgotten_altar_sacrifice_a15_plus() {
    // ForgottenAltar: MathUtils.round(.25 × 80) is 20; MathUtils.round(.35 × 80) is 28.
    for (ascension, damage) in [(14_u8, 20_u32), (15, 28)] {
        let mut run = entered(EventId::ForgottenAltar, 2, ascension, 90 + ascension, max_hp(80));
        choose(&mut run, 0, 1);
        assert_eq!((run.state.max_hp, run.state.hp), (85, 85 - damage), "A{ascension}");
        assert_eq!(frame(&run).stage, u8::MAX);
        leave(&mut run);
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

#[test]
fn m28_ghosts_accept_a15_plus() {
    for (ascension, count) in [(14_u8, 5_usize), (15, 3)] {
        let mut run = entered(EventId::Ghosts, 2, ascension, 100 + ascension, max_hp(80));
        choose(&mut run, 0, 0);
        assert_eq!((run.state.max_hp, run.state.hp), (40, 40), "A{ascension}: captured ceil(.5 × 80)");
        assert_eq!(count_cards(&run, CardId::Apparition), count);
        assert_eq!(frame(&run).stage, u8::MAX);
        leave(&mut run);
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

#[test]
fn m28_the_nest_smash_and_grab_a15_plus() {
    for (ascension, reward) in [(14_u8, 99_u32), (15, 50)] {
        let mut run = entered(EventId::Nest, 2, ascension, 110 + ascension, |_, _| {});
        let gold = run.state.gold;
        choose(&mut run, 0, 0);
        assert_eq!(run.state.gold, gold + reward, "A{ascension}");
        assert_eq!(frame(&run).stage, u8::MAX);
        leave(&mut run);
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

#[test]
fn m28_the_library_sleep_a15_plus() {
    for (ascension, heal) in [(14_u8, 26_u32), (15, 16)] {
        let mut run = entered(EventId::Library, 2, ascension, 120 + ascension, |run, _| {
            run.state.max_hp = 80;
            run.state.hp = 1;
        });
        choose(&mut run, 0, 1);
        assert_eq!((run.state.hp, run.state.max_hp), (1 + heal, 80), "A{ascension}");
        assert_eq!(frame(&run).stage, u8::MAX);
        leave(&mut run);
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

#[test]
fn m28_the_mausoleum_open_a15_plus() {
    for seed in 0..8_u8 {
        let mut run = entered(EventId::Mausoleum, 2, 15, seed, |_, _| {});
        let old_writhe = count_cards(&run, CardId::Writhe);
        let old_relics = run.state.relics.len();
        choose(&mut run, 0, 0);
        assert_eq!(count_cards(&run, CardId::Writhe), old_writhe + 1, "seed {seed}");
        assert_eq!(run.state.relics.len(), old_relics + 1, "seed {seed}");
        assert_eq!(frame(&run).stage, u8::MAX);
        leave(&mut run);
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

#[test]
fn m28_woman_in_blue_leave_a15_plus() {
    for (ascension, hp) in [(14_u8, 80_u32), (15, 76)] {
        let mut run = entered(EventId::WomanInBlue, 2, ascension, 130 + ascension, max_hp(80));
        let gold = run.state.gold;
        choose(&mut run, 0, 3);
        assert_eq!((run.state.hp, run.state.max_hp, run.state.gold), (hp, 80, gold), "A{ascension}");
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

fn count_cards(run: &Run, id: CardId) -> usize { run.state.deck.iter().filter(|card| card.id == id).count() }

#[test]
fn m28_a15_ssssserpent_gold() {
    // Sssserpent constructor: 175 gold, or 150 at A15, with Doubt either way.
    for (ascension, gold_gain) in [(14_u8, 175_u32), (15, 150)] {
        let mut run = entered(EventId::Ssssserpent, 1, ascension, 70, |_, _| {});
        let gold = run.state.gold;
        choose(&mut run, 0, 0);
        assert_eq!((run.state.gold, count(&run, CardId::Doubt)), (gold + gold_gain, 1), "A{ascension} Agree");
    }
}

#[test]
fn m28_a15_world_of_goop_leave_loss() {
    // GoopPuddle constructor: Leave loses 20–50, or 35–75 at A15, capped at the gold held.
    for (ascension, low, high) in [(14_u8, 20_u32, 50_u32), (15, 35, 75)] {
        let losses: Vec<u32> = (0..60_u8).map(|seed| {
            let run = entered(EventId::WorldOfGoop, 1, ascension, seed, |run, _| run.state.gold = 999);
            frame(&run).captured.values[0]
        }).collect();
        assert!(losses.iter().all(|loss| (low..=high).contains(loss)), "A{ascension}: {losses:?}");
        assert!(losses.iter().any(|loss| *loss > high - 10) && losses.iter().any(|loss| *loss < low + 10), "A{ascension} spans its range: {losses:?}");
    }
    let run = entered(EventId::WorldOfGoop, 1, 15, 0, |run, _| run.state.gold = 30);
    assert_eq!(frame(&run).captured.values[0], 30, "capped at the gold held");
}

#[test]
fn m28_a15_transforms_stay_unupgraded() {
    // AbstractDungeon.transformCard(.., autoUpgrade=false) for Transmogrifier and Living Wall at A15.
    for (event, option) in [(EventId::Transmogrifier, 0_u8), (EventId::LivingWall, 1)] {
        for seed in 71..76_u8 {
            let mut run = entered(event, 1, 15, seed, |_, _| {});
            let before: Vec<u64> = run.state.deck.iter().map(|c| c.instance).collect();
            choose(&mut run, 0, option);
            let candidate = run.state.pending.as_ref().unwrap().selection.candidates[0].id;
            run.step(&GameAction::Select(candidate)).unwrap();
            let fresh: Vec<_> = run.state.deck.iter().filter(|c| !before.contains(&c.instance)).collect();
            assert_eq!(fresh.len(), 1, "{event:?} one transform");
            assert_eq!(fresh[0].upgrades, 0, "{event:?} A15 transform is not upgraded: {:?}", fresh[0]);
        }
    }
}

#[test]
fn m28_a15_and_act_gates_of_the_one_time_rows() {
    // Selector gates (AbstractDungeon.getShrine / getEvent): Note For Yourself below A15, Face Trader
    // Acts 1–2, N'loth Act 2 with two relics; Dead Adventurer and Mushrooms floor > 6 at every ascension.
    let (mut run, node) = before_event(Character::Ironclad, 1, EventId::Lab, 72);
    for (ascension, note) in [(0_u8, true), (14, true), (15, false), (20, false)] {
        run.state.ascension = ascension;
        assert_eq!(event_pool::eligible(&run.state, EventId::NoteForYourself, node), note, "Note at A{ascension}");
    }
    run.state.ascension = 0;
    run.state.relics = vec![RelicId::BurningBlood, RelicId::Anchor];
    for (act, face, nloth) in [(1_u8, true, false), (2, true, true), (3, false, false)] {
        run.state.act = act;
        assert_eq!(event_pool::eligible(&run.state, EventId::FaceTrader, node), face, "Face Trader in Act {act}");
        assert_eq!(event_pool::eligible(&run.state, EventId::Nloth, node), nloth, "N'loth in Act {act}");
    }
    run.state.act = 2;
    run.state.relics.truncate(1);
    assert!(!event_pool::eligible(&run.state, EventId::Nloth, node), "N'loth needs two relics");
    run.state.act = 1;
    for ascension in [0_u8, 15] {
        run.state.ascension = ascension;
        for (floor, eligible) in [(6_u32, false), (7, true)] {
            run.state.floor = floor;
            for event in [EventId::DeadAdventurer, EventId::Mushrooms] {
                assert_eq!(event_pool::eligible(&run.state, event, node), eligible, "{event:?} floor {floor} A{ascension}");
            }
        }
    }
}

#[test]
fn m28_a15_m28f_moai_head_jump() {
    for (character, max_hp, amount) in [
        (Character::Ironclad, 80, 14),
        (Character::Silent, 70, 13),
        (Character::Defect, 75, 14),
        (Character::Watcher, 72, 13),
    ] {
        let mut run = entered_character(character, EventId::MoaiHead, 3, 15, 140, |run, _| {
            run.state.max_hp = max_hp;
            run.state.hp = max_hp / 2;
        });
        choose(&mut run, 0, 0);
        assert_eq!((run.state.max_hp, run.state.hp), (max_hp - amount, max_hp - amount), "{character:?}");
    }
}

#[test]
fn m28_a15_m28f_winding_halls_madness() {
    for (character, max_hp, amount) in [
        (Character::Ironclad, 80, 14),
        (Character::Silent, 70, 13),
        (Character::Defect, 75, 14),
        (Character::Watcher, 72, 13),
    ] {
        let mut run = entered_character(character, EventId::WindingHalls, 3, 15, 141, |run, _| {
            run.state.max_hp = max_hp;
            run.state.hp = max_hp;
        });
        let madness = count(&run, CardId::Madness);
        choose(&mut run, 0, 0);
        assert_eq!(run.state.hp, max_hp - amount, "{character:?}");
        assert_eq!(count(&run, CardId::Madness), madness + 2, "{character:?}");
    }
}

#[test]
fn m28_a15_m28f_winding_halls_rest() {
    for (character, max_hp, amount) in [
        (Character::Ironclad, 80, 16),
        (Character::Silent, 70, 14),
        (Character::Defect, 75, 15),
        (Character::Watcher, 72, 14),
    ] {
        let mut run = entered_character(character, EventId::WindingHalls, 3, 15, 142, |run, _| {
            run.state.max_hp = max_hp;
            run.state.hp = 1;
        });
        let writhe = count(&run, CardId::Writhe);
        choose(&mut run, 0, 1);
        assert_eq!(run.state.hp, 1 + amount, "{character:?}");
        assert_eq!(count(&run, CardId::Writhe), writhe + 1, "{character:?}");
    }
}

#[test]
fn m28_a15_m28f_winding_halls_maze() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = entered_character(character, EventId::WindingHalls, 3, 15, 143, |run, _| {
            let max_hp = match run.state.character {
                Character::Ironclad => 80,
                Character::Silent => 70,
                Character::Defect => 75,
                Character::Watcher => 72,
            };
            run.state.max_hp = max_hp;
            run.state.hp = max_hp;
        });
        choose(&mut run, 0, 2);
        assert_eq!(run.state.max_hp, run.state.hp);
        assert_eq!(run.state.max_hp + 4, match character {
            Character::Ironclad => 80,
            Character::Silent => 70,
            Character::Defect => 75,
            Character::Watcher => 72,
        });
    }
}

#[test]
fn m28_a15_m28f_designer_costs_and_rows() {
    for (option, price) in [(0_u8, 50_u32), (1, 50), (2, 75), (3, 75), (4, 110), (5, 5)] {
        let mut found = false;
        for seed in 0..=u8::MAX {
            let mut run = entered(EventId::Designer, 2, 15, seed, |run, _| {
                run.state.gold = 500;
                run.state.deck.iter_mut().for_each(|card| card.upgrades = 0);
            });
            run.state.gold = 49;
            crate::run::rooms::standard::event_frame::publish(&mut run).unwrap();
            if let Some(choice) = view(&run).choices.iter().find(|choice|
                choice.kind == RoomChoiceKind::EventOption { stage: 0, option })
            {
                // A row price is a gold price (`public` gates it on gold); Punch's HP cost shows none.
                assert_eq!(choice.price, (option != 5).then_some(price), "option {option}, A15 row price");
                if option != 5 {
                    assert_eq!(choice.status, RoomChoiceStatus::Disabled, "option {option} at 49 gold");
                } else {
                    assert_eq!(choice.status, RoomChoiceStatus::Offered, "Punch is not gold-gated");
                }
                found = true;
                break;
            }
        }
        assert!(found, "Designer constructor did not expose variant {option}");
    }
}

#[test]
fn m28_a15_m28f_mysterious_sphere_unsupported_and_leave() {
    let mut run = entered(EventId::MysteriousSphere, 3, 1, 144, |_, _| {});
    let fight = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 });
    assert_eq!(fight.status, RoomChoiceStatus::Disabled, "typed setup gate above A0");
    let before = run.clone();
    assert_eq!(run.step(&GameAction::Choose(fight.id)), Err(crate::run::RunError::InvalidAction));
    assert!(run == before, "disabled Fight is atomic");
    let before = run.clone();
    let unsupported = crate::run::rooms::standard::event_frame::choose_constructed(&mut run, vec![
        Outcome::Combat {
            encounter: EncounterId::TwoOrbWalkers,
            rewards: None,
            next_stage: 0,
            elite: false,
        },
    ]);
    assert!(matches!(unsupported, Err(crate::run::RunError::Unsupported(_))), "{unsupported:?}");
    assert!(run == before, "typed Unsupported is atomic");
    choose(&mut run, 0, 1);
    assert_eq!(view(&run).kind, RoomViewKind::Map, "Leave remains available above A0");
}

#[test]
fn m28_a15_m28f_secret_portal_enter_unsupported_and_leave() {
    // Review-fixes M1: no Act 3 boss has enemy facts above A0, so Enter is the typed setup gate.
    for ascension in [1_u8, 15] {
        let mut run = entered(EventId::SecretPortal, 3, ascension, 22, |_, _| {});
        let enter = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 });
        assert_eq!(enter.status, RoomChoiceStatus::Disabled, "A{ascension}: typed setup gate");
        let before = run.clone();
        assert_eq!(run.step(&GameAction::Choose(enter.id)), Err(crate::run::RunError::InvalidAction));
        assert!(run == before, "A{ascension}: disabled Enter is atomic");
        let unsupported = crate::run::rooms::standard::event_frame::choose_constructed(&mut run, vec![Outcome::EnterBoss]);
        assert!(matches!(unsupported, Err(crate::run::RunError::Unsupported(_))), "{unsupported:?}");
        assert!(run == before, "A{ascension}: typed Unsupported is atomic");
        assert!(run.state.fault.is_none());
        choose(&mut run, 0, 1);
        assert_eq!(view(&run).kind, RoomViewKind::Map, "A{ascension}: Leave remains available");
    }
}
