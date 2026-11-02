//! A25 M4 remainder: relic offers, selection continuations, Dig and Neow boss swap.
use crate::content::{CardId, Character, EncounterId, PotionId, RelicId, RelicTier};
use crate::items::lifecycle::{RelicObtained, RelicSource, RewardSpec};
use crate::items::tests::m27_support::complete_run;
use crate::run::rooms::{RoomChoiceKind, RoomChoiceStatus, RoomChoiceView, RoomLoop, RoomView,
    RoomViewKind};
use crate::run::{CardInstance, ChoiceId, GameAction, MapNode, MapNodeKind, Run, RunError,
    RunMap, RunRngScope, RoomScenario, RunTerminal};
use std::collections::BTreeMap;

fn settled_run() -> Run {
    let mut run = complete_run(Character::Ironclad, vec![], vec![Some(PotionId::FirePotion); 3],
        BTreeMap::new(), EncounterId::Cultist);
    // Three Fire Potions settle the fixture combat without changing the authored deck.
    for slot in 0..3 {
        run.choose_combat(crate::combat::CombatAction::UsePotion {
            slot,
            target: Some(crate::combat::TargetId(1)),
        }).expect("Fire Potion settles the complete fixture");
    }
    assert!(run.state().combat.is_none() && run.state().active_combat.is_none());
    run
}

fn install_room(run: &mut Run, kind: RoomViewKind,
    rows: Vec<(RoomChoiceKind, Option<u32>, RoomChoiceStatus)>) -> Vec<ChoiceId> {
    let node = crate::run::NodeId(1);
    let map_kind = match kind {
        RoomViewKind::Shop => MapNodeKind::Shop,
        RoomViewKind::Campfire => MapNodeKind::Rest,
        RoomViewKind::BossRelicChoice | RoomViewKind::BossRelicChild => MapNodeKind::Boss,
        RoomViewKind::Event(_) => MapNodeKind::Event,
        _ => MapNodeKind::Monster,
    };
    run.state.act = 1;
    run.state.floor = 1;
    run.state.map = Some(RunMap {
        act: 1,
        current: Some(node),
        nodes: vec![MapNode {
            id: node, floor: 1, label: "A25 relic offer fixture", edges: vec![], x: 0, y: 0,
            kind: map_kind, hidden_kind: None, visited: true, revealed: true,
            encounter: None, burning: false,
        }],
        ..RunMap::default()
    });
    let ids = (0..rows.len()).map(|index| ChoiceId(100 + index as u64)).collect::<Vec<_>>();
    let choices = rows.into_iter().zip(ids.iter().copied()).map(|((kind, price, status), id)| {
        RoomChoiceView { id, kind, price, status }
    }).collect();
    run.next_choice = 100 + ids.len() as u64;
    let scenario = run.state.room_loop.as_ref().map_or(RoomScenario::M5Act1Captured, |room| room.scenario);
    run.state.room_loop = Some(RoomLoop {
        scenario, act1: None, act2: None, act3: None, act4: None, m10_elite_relic_cursor: 0,
        screen: RoomView { kind, choices }, purge_cost: 75, reward_parent: None, active_rewards: None,
        boss: None, transition: None, act2_boss_order: None, act3_boss_order: None,
        joust: None, joust_visits: 0, act3_event: None, act3_end: None, standard_event: None,
    });
    ids
}

fn row_id(run: &Run, kind: &RoomChoiceKind) -> ChoiceId {
    run.state.room_loop.as_ref().unwrap().screen.choices.iter()
        .find(|row| &row.kind == kind && row.status == RoomChoiceStatus::Offered)
        .unwrap_or_else(|| panic!("missing offered row {kind:?}" )).id
}

fn choose(run: &mut Run, kind: &RoomChoiceKind) -> crate::run::RunTransition {
    let id = row_id(run, kind);
    run.step(&GameAction::Choose(id)).unwrap_or_else(|error| panic!("choose {kind:?}: {error:?}"))
}

#[test]
fn m27_equip_selection_from_reward_and_shop() {
    let mut reward = settled_run();
    let ids = install_room(&mut reward, RoomViewKind::Rewards, vec![
        (RoomChoiceKind::Relic(RelicId::EmptyCage), None, RoomChoiceStatus::Offered),
        (RoomChoiceKind::Proceed, None, RoomChoiceStatus::Offered),
    ]);
    let first_token = reward.next_choice;
    reward.step(&GameAction::Choose(ids[0])).expect("claim reward Empty Cage");
    let pending = reward.state.pending.as_ref().expect("reward claim opens equip selection");
    assert_eq!(pending.selection.required, 2);
    assert_eq!(pending.selection.candidates.len(), 6);
    assert!(pending.selection.candidates.iter().all(|candidate| candidate.id.0 >= first_token));
    assert!(pending.selection.candidates.iter().all(|candidate|
        !reward.state.deck.iter().any(|card| card.instance == candidate.id.0)));
    let first = pending.selection.candidates[0].id;
    let second = pending.selection.candidates[1].id;
    reward.step(&GameAction::Select(first)).expect("select first published token");
    reward.step(&GameAction::Select(second)).expect("complete reward equip selection");
    assert!(reward.state.pending.is_none());
    assert_eq!(reward.state.room_loop.as_ref().unwrap().screen.kind, RoomViewKind::Rewards);
    assert_eq!(reward.state.last_selection.as_ref().unwrap().len(), 2);

    let mut shop = settled_run();
    shop.state.gold = 200;
    let shop_rows = install_room(&mut shop, RoomViewKind::Shop, vec![
        (RoomChoiceKind::Relic(RelicId::EmptyCage), Some(150), RoomChoiceStatus::Offered),
        (RoomChoiceKind::Leave, None, RoomChoiceStatus::Offered),
    ]);
    let first_token = shop.next_choice;
    shop.step(&GameAction::Choose(shop_rows[0])).expect("buy Empty Cage");
    let pending = shop.state.pending.as_ref().expect("shop purchase opens equip selection");
    assert_eq!(pending.selection.required, 2);
    assert!(pending.selection.candidates.iter().all(|candidate| candidate.id.0 >= first_token));

    let mut small = settled_run();
    small.state.deck.truncate(2);
    let small_rows = install_room(&mut small, RoomViewKind::Rewards, vec![
        (RoomChoiceKind::Relic(RelicId::EmptyCage), None, RoomChoiceStatus::Offered),
        (RoomChoiceKind::Proceed, None, RoomChoiceStatus::Offered),
    ]);
    let transition = small.step(&GameAction::Choose(small_rows[0])).expect("auto-apply eligible <= count");
    assert_eq!(transition.terminal, Some(RunTerminal::EmptyDeck));
    assert!(small.state.pending.is_none());
    assert!(small.state.deck.is_empty());

    let mut empty = settled_run();
    empty.state.deck = vec![CardId::CurseOfTheBell, CardId::AscendersBane].into_iter().enumerate()
        .map(|(index, id)| CardInstance { instance: index as u64 + 1, id, upgrades: 0, misc: 0 }).collect();
    let empty_rows = install_room(&mut empty, RoomViewKind::Rewards, vec![
        (RoomChoiceKind::Relic(RelicId::EmptyCage), None, RoomChoiceStatus::Offered),
        (RoomChoiceKind::Proceed, None, RoomChoiceStatus::Offered),
    ]);
    let before = empty.state.deck.clone();
    empty.step(&GameAction::Choose(empty_rows[0])).expect("empty equip group opens nothing");
    assert!(empty.state.pending.is_none());
    assert_eq!(empty.state.deck, before);
}

#[test]
fn m27_equip_offers_through_reward_owner() {
    let mut run = settled_run();
    let parent = install_room(&mut run, RoomViewKind::Shop, vec![
        (RoomChoiceKind::Relic(RelicId::Strawberry), Some(150), RoomChoiceStatus::Claimed),
        (RoomChoiceKind::Leave, None, RoomChoiceStatus::Offered),
    ]);
    crate::items::obtain_relic(&mut run.state, RelicId::Strawberry, RelicSource::Reward).unwrap();
    run.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
        vec![RelicId::Anchor], vec![RelicId::BagOfPreparation], vec![RelicId::DeadBranch], vec![], vec![]));
    let spec = RewardSpec {
        card_rewards: 2,
        relic_tiers: vec![RelicTier::Common, RelicTier::Uncommon, RelicTier::Rare],
        potions: vec![PotionId::FirePotion],
        gold: 50,
    };
    crate::items::lifecycle::after_obtain(&mut run, parent[0], RelicId::Strawberry,
        RelicObtained { selection: None, offers: vec![spec] }).expect("equip opens reward owner");
    let screen = &run.state.room_loop.as_ref().unwrap().screen;
    assert_eq!(screen.kind, RoomViewKind::Rewards);
    assert_eq!(screen.choices.iter().map(|row| row.kind.clone()).collect::<Vec<_>>(), vec![
        RoomChoiceKind::CardReward, RoomChoiceKind::CardReward,
        RoomChoiceKind::Relic(RelicId::Anchor), RoomChoiceKind::Relic(RelicId::BagOfPreparation),
        RoomChoiceKind::Relic(RelicId::DeadBranch), RoomChoiceKind::Potion(PotionId::FirePotion),
        RoomChoiceKind::Gold(50), RoomChoiceKind::Proceed,
    ]);

    let first_parent = row_id(&run, &RoomChoiceKind::CardReward);
    run.step(&GameAction::Choose(first_parent)).expect("first card reward opens its child");
    let first_child = run.state.room_loop.as_ref().unwrap().screen.clone();
    assert_eq!(first_child.choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Card { .. })).count(), 3);
    choose(&mut run, &RoomChoiceKind::Skip);
    let second_parent = row_id(&run, &RoomChoiceKind::CardReward);
    run.step(&GameAction::Choose(second_parent)).expect("second card reward opens its child");
    let second_child = run.state.room_loop.as_ref().unwrap().screen.clone();
    assert_ne!(first_child, second_child, "each reward row owns distinct child choice IDs");
    choose(&mut run, &RoomChoiceKind::Skip);

    choose(&mut run, &RoomChoiceKind::Relic(RelicId::Anchor));
    assert!(run.state.relics.contains(&RelicId::Anchor));
    assert!(!run.state.relic_pools.as_ref().unwrap().common.contains(&RelicId::Anchor));
    choose(&mut run, &RoomChoiceKind::Proceed);
    assert_eq!(run.state.room_loop.as_ref().unwrap().screen.kind, RoomViewKind::Shop);
}

#[test]
fn m27_shovel_dig_offers_one_relic() {
    let mut run = settled_run();
    run.state.relics.push(RelicId::Shovel);
    run.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
        vec![RelicId::Anchor], vec![RelicId::BagOfPreparation], vec![RelicId::DeadBranch], vec![], vec![]));
    let ids = install_room(&mut run, RoomViewKind::Campfire, vec![
        (RoomChoiceKind::Dig, None, RoomChoiceStatus::Offered),
    ]);
    run.state.rng.set_counter_for_test(RunRngScope::M27RelicTier, &[1, 1, 1, ids[0].0, 5], 0);
    run.step(&GameAction::Choose(ids[0])).expect("Dig rolls and opens one relic offer");
    let screen = &run.state.room_loop.as_ref().unwrap().screen;
    assert_eq!(screen.kind, RoomViewKind::Rewards);
    assert_eq!(screen.choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).count(), 1);
    assert_eq!(screen.choices.iter().find(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).unwrap().kind,
        RoomChoiceKind::Relic(RelicId::Anchor));
    choose(&mut run, &RoomChoiceKind::Relic(RelicId::Anchor));
    choose(&mut run, &RoomChoiceKind::Proceed);
    assert_eq!(run.state.room_loop.as_ref().unwrap().screen.kind, RoomViewKind::Map,
        "proceeding from Dig finishes the campfire");
}

#[test]
fn m27_neow_boss_swap_removes_starter_first() {
    let mut run = settled_run();
    run.state.relics = vec![RelicId::BurningBlood];
    run.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
        vec![], vec![], vec![], vec![], vec![RelicId::BlackBlood, RelicId::SneckoEye]));
    assert_eq!(crate::items::lifecycle::neow_boss_swap(&mut run.state, &[27]), Ok(RelicId::SneckoEye));
    assert_eq!(run.state.relics, vec![RelicId::SneckoEye]);
    assert!(!run.state.relic_pools.as_ref().unwrap().boss.contains(&RelicId::BlackBlood));

    let mut combat = complete_run(Character::Ironclad, vec![RelicId::BurningBlood],
        vec![Some(PotionId::FirePotion); 3], BTreeMap::new(), EncounterId::Cultist);
    combat.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
        vec![], vec![], vec![], vec![], vec![RelicId::BlackBlood, RelicId::SneckoEye]));
    let before = format!("{:?}", combat.state);
    assert_eq!(crate::items::lifecycle::neow_boss_swap(&mut combat.state, &[27]), Err(RunError::InvalidAction));
    assert_eq!(format!("{:?}", combat.state), before, "in-combat swap rejection is atomic");
}
