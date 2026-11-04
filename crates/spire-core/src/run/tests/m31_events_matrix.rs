//! M31 A14/A15 Run and event matrix (plan items 5–7; design §§6.7–6.8). Expected values are hand-worked
//! from the cited source check, with event rows built through the existing public or diagnostic routes.
use super::super::*;
use crate::combat::rng::Partition;
use crate::content::{CardId, Character, EventId};

fn rng(seed: u64) -> RunRng { RunRng::new([seed as u8; 32], Partition::Test, seed) }
fn profile(full: bool) -> CreationProfile { CreationProfile { neow_eligible: full, ..CreationProfile::experienced() } }
fn room(run: &Run) -> RoomView { rooms::public(run).unwrap() }
fn room_kind(run: &Run) -> RoomViewKind { room(run).kind }

fn standard(character: Character, ascension: u8, full: bool, seed: u64) -> Run {
    Run::new_room_scenario(RunConfig { character, ascension }, profile(full), rng(seed), RoomScenario::Standard)
        .unwrap_or_else(|error| panic!("{character:?} A{ascension}: {error:?}"))
}

fn force_neow(run: &mut Run, category: u8, reward: NeowReward, drawback: Option<NeowDrawback>) -> ChoiceId {
    let candidate = run.candidates.iter_mut().find(|candidate| {
        candidate.neow_offer.as_ref().is_some_and(|offer| offer.category == category)
    }).expect("full Neow category");
    candidate.neow_offer = Some(NeowOffer { category, reward, drawback });
    let id = candidate.id;
    run.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|row| row.id == id).unwrap().kind =
        RoomChoiceKind::NeowBlessing { category, reward, drawback };
    run.step(&GameAction::Choose(id)).unwrap();
    id
}

fn event_frame(run: &Run) -> &rooms::standard::event_frame::StandardEventFrame {
    run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap()
}

fn event_row(run: &Run, stage: u8, option: u8) -> ChoiceId {
    room(run).choices.into_iter().find(|row| row.kind == RoomChoiceKind::EventOption { stage, option })
        .unwrap_or_else(|| panic!("missing EventOption {stage}/{option}: {:?}", room(run))).id
}

fn choose_event(run: &mut Run, stage: u8, option: u8) {
    let id = event_row(run, stage, option);
    run.step(&GameAction::Choose(id)).unwrap();
}

fn card_count(run: &Run, id: CardId) -> usize { run.state.deck.iter().filter(|card| card.id == id).count() }

/// A constructed event fixture that sets Ascension before the event is selected and built.
fn event_run(character: Character, event: EventId, act: u8, ascension: u8, seed: u64,
    setup: impl FnOnce(&mut Run, NodeId)) -> Run {
    let (mut run, node) = super::m28_event_core::before_event(character, act, event, seed as u8);
    run.state.ascension = ascension;
    setup(&mut run, node);
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(room_kind(&run), RoomViewKind::Event(event));
    run
}

fn public_on_map(character: Character, ascension: u8, seed: u64) -> Run {
    let mut run = standard(character, ascension, false, seed);
    let choice = room(&run).choices.into_iter().find(|row| matches!(row.kind, RoomChoiceKind::NeowMaxHp { .. })).unwrap().id;
    run.step(&GameAction::Choose(choice)).unwrap();
    assert_eq!(room_kind(&run), RoomViewKind::Map);
    run
}

fn public_act_map(character: Character, ascension: u8, seed: u64, act: u8) -> Run {
    let mut run = public_on_map(character, ascension, seed);
    run.state.hp = 999;
    run.state.max_hp = 999;
    run.state.gold = 999;
    if act > 1 {
        super::m28_world::drive(&mut run, |candidate| candidate.state.act == act && room_kind(candidate) == RoomViewKind::Map);
    }
    assert_eq!((run.state.act, room_kind(&run)), (act, RoomViewKind::Map));
    run
}

fn force_public_event(run: &mut Run, event: EventId, setup: impl FnOnce(&mut Run)) {
    setup(run);
    let pools = run.state.event_pools.as_mut().expect("Standard event pools");
    pools.ordinary = vec![event];
    pools.shrines.clear();
    pools.one_time.clear();
    let target = run.decision().unwrap().candidates.iter().find_map(|candidate| match candidate.action {
        GameAction::Move(id) => Some(id), _ => None,
    }).unwrap();
    let node = run.state.map.as_mut().unwrap().nodes.iter_mut().find(|node| node.id == target).unwrap();
    node.kind = MapNodeKind::Event;
    node.hidden_kind = None;
    node.encounter = None;
    run.step(&GameAction::Move(target)).unwrap();
    assert_eq!(room_kind(run), RoomViewKind::Event(event));
}

// Source: AbstractDungeon.java:2674–2687; character max-HP loss getters cited in source check §E.
// Grade: Run
#[test]
fn m31_a14_start_hp_standard() {
    let rows = [
        (Character::Ironclad, (72, 80), (68, 75)),
        (Character::Silent, (63, 70), (59, 66)),
        (Character::Defect, (68, 75), (64, 71)),
        (Character::Watcher, (65, 72), (61, 68)),
    ];
    for (character, a13, a14) in rows {
        for (ascension, expected) in [(13_u8, a13), (14, a14)] {
            let run = standard(character, ascension, false, 0x41);
            assert_eq!(room_kind(&run), RoomViewKind::Neow, "public Standard A{ascension} creation");
            assert_eq!((run.state.hp, run.state.max_hp), expected, "{character:?} A{ascension}");
        }
    }
}

// Source: AbstractDungeon.java:2674–2687; NeowReward.java:288–300; source check §E.
// Grade: Run
#[test]
fn m31_a14_neow_max_hp_rewards() {
    // Rows are (character, A14 start HP/max, b, MaxHp result, MaxHpLoss result), worked from b=floor(M×.1f).
    let table = [
        (Character::Ironclad, (68, 75), 7, (75, 82), (68, 68)),
        (Character::Silent, (59, 66), 6, (65, 72), (59, 60)),
        (Character::Defect, (64, 71), 7, (71, 78), (64, 64)),
        (Character::Watcher, (61, 68), 6, (67, 74), (61, 62)),
    ];
    for (character, start, b, max_hp_reward, max_hp_loss) in table {
        let mut reward = standard(character, 14, true, 0x42);
        force_neow(&mut reward, 1, NeowReward::MaxHp, None);
        assert_eq!((reward.state.hp, reward.state.max_hp), max_hp_reward, "{character:?} b={b} MaxHp");

        let mut loss = standard(character, 14, true, 0x42);
        force_neow(&mut loss, 2, NeowReward::Gold250, Some(NeowDrawback::MaxHpLoss));
        assert_eq!((loss.state.hp, loss.state.max_hp), max_hp_loss, "{character:?} start {start:?}, b={b} MaxHpLoss");
    }
}

// Source: Designer.java:60–76, 240–267; source check §F.
// Grade: Run
#[test]
fn m31_a15_designer_punch() {
    for (ascension, loss) in [(14_u8, 3_u32), (15, 5)] {
        let mut run = event_run(Character::Ironclad, EventId::Designer, 2, ascension, 0x43 + u64::from(ascension), |run, _| {
            run.state.gold = 500;
            run.state.hp = 60;
            run.state.max_hp = 80;
        });
        choose_event(&mut run, 0, 5);
        assert_eq!(run.state.hp, 60 - loss, "A{ascension} Designer Punch HP_LOSS");
    }
}

// Source: MoaiHead.java:42–51; WindingHalls.java:46–68; TheMausoleum.java:44–46; Designer.java:60–76.
// Grade: Run
#[test]
fn m31_a14_arms_of_a15_only_tests() {
    let hp_rows = [
        (Character::Ironclad, 80_u32, 10_u32, 20_u32),
        (Character::Silent, 70, 9, 18),
        (Character::Defect, 75, 9, 19),
        (Character::Watcher, 72, 9, 18),
    ];
    for (character, max_hp, damage, heal) in hp_rows {
        let mut moai = event_run(character, EventId::MoaiHead, 3, 14, 0x44, |run, _| {
            run.state.max_hp = max_hp;
            run.state.hp = max_hp / 2;
        });
        choose_event(&mut moai, 0, 0);
        assert_eq!((moai.state.max_hp, moai.state.hp), (max_hp - damage, max_hp - damage), "{character:?} Moai A14 round(12.5% M)");

        let mut madness = event_run(character, EventId::WindingHalls, 3, 14, 0x45, |run, _| {
            run.state.max_hp = max_hp;
            run.state.hp = max_hp;
        });
        let old_madness = card_count(&madness, CardId::Madness);
        choose_event(&mut madness, 0, 0);
        assert_eq!(madness.state.hp, max_hp - damage, "{character:?} Winding Halls Embrace A14");
        assert_eq!(card_count(&madness, CardId::Madness), old_madness + 2);

        let mut rest = event_run(character, EventId::WindingHalls, 3, 14, 0x46, |run, _| {
            run.state.max_hp = max_hp;
            run.state.hp = 1;
        });
        let old_writhe = card_count(&rest, CardId::Writhe);
        choose_event(&mut rest, 0, 1);
        assert_eq!(rest.state.hp, 1 + heal, "{character:?} Winding Halls Focus A14 round(25% M)");
        assert_eq!(card_count(&rest, CardId::Writhe), old_writhe + 1);

        let mut maze = event_run(character, EventId::WindingHalls, 3, 14, 0x47, |run, _| {
            run.state.max_hp = max_hp;
            run.state.hp = max_hp;
        });
        choose_event(&mut maze, 0, 2);
        assert_eq!(maze.state.max_hp, max_hp - 4, "{character:?} Winding Halls Retrace keeps round(5% M)");
    }

    // Mausoleum keeps its 50% Writhe roll through A14; A15 alone forces the outcome true.
    let (mut saw_writhe, mut saw_clean) = (false, false);
    for seed in 0..64_u64 {
        let mut run = event_run(Character::Ironclad, EventId::Mausoleum, 2, 14, seed, |_, _| {});
        let old_writhe = card_count(&run, CardId::Writhe);
        choose_event(&mut run, 0, 0);
        match card_count(&run, CardId::Writhe) - old_writhe {
            0 => saw_clean = true,
            1 => saw_writhe = true,
            other => panic!("Mausoleum added {other} Writhe cards"),
        }
    }
    assert!(saw_writhe && saw_clean, "A14 Mausoleum exercises both arms of its 50% gate");

    // Designer A14 still uses its A0 prices (40/60/90), with Punch offered without a gold price.
    for (option, price) in [(0_u8, Some(40)), (1, Some(40)), (2, Some(60)), (3, Some(60)), (4, Some(90)), (5, None)] {
        let mut found = false;
        for seed in 0..=u8::MAX {
            let mut run = event_run(Character::Ironclad, EventId::Designer, 2, 14, u64::from(seed), |run, _| run.state.gold = 500);
            run.state.gold = 39;
            crate::run::rooms::standard::event_frame::publish(&mut run).unwrap();
            if let Some(row) = room(&run).choices.iter().find(|row| row.kind == RoomChoiceKind::EventOption { stage: 0, option }) {
                assert_eq!(row.price, price, "A14 Designer option {option}");
                assert_eq!(row.status, if price.is_some() { RoomChoiceStatus::Disabled } else { RoomChoiceStatus::Offered },
                    "A14 Designer option {option} status at 39 gold");
                found = true;
                break;
            }
        }
        assert!(found, "Designer A14 constructor did not expose variant {option}");
    }
}

// Source: AbstractDungeon.java:1439–1466; event_pool.rs:35–59.
// Grade: Run
#[test]
fn m31_a15_note_for_yourself_pool_gate() {
    for (ascension, expected) in [(14_u8, true), (15, false)] {
        let run = public_on_map(Character::Ironclad, ascension, 0x48);
        let pools = run.state.event_pools.as_ref().expect("public Standard event pools");
        let node = run.state.map.as_ref().unwrap().nodes[0].id;
        let eligible_one_time: Vec<_> = pools.one_time.iter().copied().filter(|event| {
            rooms::standard::event_pool::eligible(&run.state, *event, node)
        }).collect();
        assert_eq!(eligible_one_time.contains(&EventId::NoteForYourself), expected, "public Standard A{ascension}");
        assert_eq!(rooms::standard::event_pool::eligible(&run.state, EventId::NoteForYourself, node), expected);
    }
}

// Source: GoldenIdolEvent.java:54–75; Nest.java:46–50; WindingHalls.java:46–68; task0-asst/a15-reach-v1.md.
// Grade: Run
#[test]
fn m31_a15_event_from_public_route() {
    let mut golden = public_act_map(Character::Ironclad, 15, 0x49, 1);
    force_public_event(&mut golden, EventId::GoldenIdol, |run| {
        run.state.max_hp = 80;
        run.state.hp = 80;
    });
    assert_eq!(golden.state.ascension, 15, "created at A15 on the public Standard route");
    assert_eq!(event_frame(&golden).captured.values, vec![28, 8], "A15 Golden Idol captures 35% damage and 10% max-HP loss at M=80");
    choose_event(&mut golden, 0, 0);
    choose_event(&mut golden, 1, 1);
    assert_eq!(golden.state.hp, 52, "Golden Idol Touch applies A15's 28 HP loss");

    let mut nest = public_act_map(Character::Ironclad, 15, 0x4A, 2);
    force_public_event(&mut nest, EventId::Nest, |_| {});
    assert_eq!(nest.state.ascension, 15);
    let gold = nest.state.gold;
    choose_event(&mut nest, 0, 0);
    assert_eq!(nest.state.gold, gold + 50, "A15 Nest Smash and Grab gives 50 gold");

    let mut halls = public_act_map(Character::Ironclad, 15, 0x4B, 3);
    force_public_event(&mut halls, EventId::WindingHalls, |run| {
        run.state.max_hp = 80;
        run.state.hp = 80;
    });
    assert_eq!(halls.state.ascension, 15);
    choose_event(&mut halls, 0, 0);
    assert_eq!(halls.state.hp, 66, "A15 Winding Halls Embrace applies round(18% of 80)=14 HP loss");
    assert_eq!(card_count(&halls, CardId::Madness), 2);
}
