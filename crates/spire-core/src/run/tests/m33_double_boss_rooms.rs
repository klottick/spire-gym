//! M33 A20 second-room hooks and event-origin/preserved boss paths (plan item 5; review M2–M4).
//! Literals are worked from lane A §3–§5: Pantograph heals 25 at boss-battle start, Slaver's
//! Collar adds one energy against a BOSS-type group, Maw Bank grants 12 on an unspent floor entry.
use super::super::*;
use crate::content::{Character, EventId, PotionId, RelicId};
use super::m33_double_boss::{act3_run, boss_node, kind, lethal, order, proceed, to_boss_door, SECOND};

fn rng(seed: u8) -> crate::run::RunRng { crate::run::RunRng::new([seed; 32], crate::combat::rng::Partition::Test, 0) }

fn event_run(character: Character, event: EventId, seed: u8) -> (Run, NodeId) {
    let mut run = act3_run(character, 20, seed, None);
    let pool = crate::content::Catalog::standard().event(event).unwrap().pool;
    let list = vec![event];
    run.state.event_pools = Some(match pool {
        crate::content::EventPool::Ordinary => rooms::standard::event_pool::EventPools { ordinary: list, shrines: vec![], one_time: vec![], seen: vec![] },
        crate::content::EventPool::Shrine => rooms::standard::event_pool::EventPools { ordinary: vec![], shrines: list, one_time: vec![], seen: vec![] },
        crate::content::EventPool::OneTime => rooms::standard::event_pool::EventPools { ordinary: vec![], shrines: vec![], one_time: list, seen: vec![] },
        crate::content::EventPool::Start => panic!("Act 3 event cannot be in the start pool"),
    });
    let node = run.state.map.as_mut().unwrap().nodes.iter_mut().find(|node| node.y == 0).unwrap();
    node.kind = MapNodeKind::Event;
    let id = node.id;
    (run, id)
}

fn choose_event(run: &mut Run, option: u8) -> RunTransition {
    let row = rooms::public(run).unwrap().choices.into_iter().find(|row| row.kind == RoomChoiceKind::EventOption { stage: 0, option }).unwrap();
    run.step(&GameAction::Choose(row.id)).unwrap()
}

#[test]
fn m33_a20_second_room_hooks() {
    let mut run = act3_run(Character::Ironclad, 20, 0xA1, None);
    to_boss_door(&mut run);
    run.state.relics.extend([RelicId::Pantograph, RelicId::SlaversCollar, RelicId::MawBank, RelicId::WingBoots]);
    run.state.relic_counters.insert(RelicId::WingBoots, 3);
    // The move law's charge state (`world::commit_node`): an edgeless move to a higher row spends one (review M2).
    run.state.map.as_mut().unwrap().winged_moves = 3;
    let before_gold = run.state.gold;
    let boss = boss_node(&run).id;
    run.state.hp = 20;
    run.step(&GameAction::Move(boss)).unwrap();
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().player.hp, 45, "Pantograph heals 25 at the first boss battle start");
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().player.energy, 4, "Slaver's Collar adds one energy to the first boss fight");
    assert_eq!(run.state.gold, before_gold + 12, "Maw Bank grants 12 gold at the first unspent floor entry");
    let order = order(&run).unwrap();
    lethal(&mut run);
    run.settle_combat().unwrap();
    let boots_after_first = (run.state.relic_counters.get(&RelicId::WingBoots).copied(), run.state.map.as_ref().unwrap().winged_moves);
    assert_eq!(boots_after_first, (Some(3), 3), "the first boss is entered by its edge");
    run.state.hp = 20;
    let gold_before_second = run.state.gold;
    proceed(&mut run);
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, order[1]);
    let public = run.state.combat.as_ref().unwrap().public_full().unwrap();
    assert_eq!(public.player.hp, 45, "Pantograph heals 25 again at the second boss battle start");
    assert_eq!(public.player.energy, 4, "Slaver's Collar adds one energy again in the second boss fight");
    assert_eq!(run.state.gold, gold_before_second + 12, "Maw Bank grants 12 gold on the second room's floor entry");
    assert_eq!(run.state.map.as_ref().unwrap().current, Some(SECOND));
    lethal(&mut run);
    run.settle_combat().unwrap();
    assert_eq!((run.state.relic_counters.get(&RelicId::WingBoots).copied(), run.state.map.as_ref().unwrap().winged_moves), boots_after_first,
        "synthetic room entry does not spend a Wing Boots charge");
    assert!(run.state.fault.is_none());
}

#[test]
fn m33_a20_potions_between_bosses() {
    let mut run = act3_run(Character::Silent, 20, 0xA2, None);
    to_boss_door(&mut run);
    run.state.potions.resize(crate::run::potion_base_slots(20), None);
    crate::items::obtain_run_potion(&mut run.state, PotionId::FruitJuice).unwrap();
    crate::items::obtain_run_potion(&mut run.state, PotionId::FirePotion).unwrap();
    run.state.max_hp = 200;
    run.state.hp = 200;
    let boss = boss_node(&run).id;
    run.step(&GameAction::Move(boss)).unwrap();
    lethal(&mut run);
    run.settle_combat().unwrap();
    let old_max = run.state.max_hp;
    run.step(&GameAction::UsePotion { slot: 0, target: None }).unwrap();
    run.step(&GameAction::DiscardPotion { slot: 1 }).unwrap();
    assert_eq!(run.state.max_hp, old_max + 5, "Fruit Juice permanently adds 5 max HP");
    assert_eq!(run.state.potions[0], None);
    assert_eq!(run.state.potions[1], None);
    let proceed = rooms::public(&run).unwrap().choices.iter().find(|row| row.kind == RoomChoiceKind::Proceed).unwrap().id;
    run.step(&GameAction::Choose(proceed)).unwrap();
    assert_eq!(run.state.map.as_ref().unwrap().current, Some(SECOND));
    assert_eq!(run.state.max_hp, old_max + 5);
    assert_eq!(run.state.combat.as_ref().unwrap().item_context().unwrap().potions(), &[None, None]);
    assert!(run.state.fault.is_none());
}

#[test]
fn m33_a20_mind_bloom_war_keeps_order() {
    let (mut run, node) = event_run(Character::Defect, EventId::MindBloom, 0xA3);
    let original_order = order(&run).unwrap();
    run.step(&GameAction::Move(node)).unwrap();
    choose_event(&mut run, 0); // Mind Bloom: "I am War" starts its Act 1 boss as an event combat.
    assert!(matches!(run.state.active_combat.as_ref().unwrap().origin, CombatOrigin::Event { event: EventId::MindBloom, .. }));
    super::m28_world::win(&mut run);
    assert_eq!(kind(&run), RoomViewKind::Rewards, "event combat returns to the ordinary event reward screen");
    assert_eq!(order(&run), Some(original_order.clone()));
    assert!(run.state.room_loop.as_ref().unwrap().act3_end.is_none(), "event origin does not enter the second-boss frame");
    super::m28_world::drive(&mut run, |candidate| candidate.state.act == 3 && candidate.state.pending.is_none()
        && kind(candidate) == RoomViewKind::Map
        && candidate.decision().unwrap().candidates.iter().any(|c| c.action == GameAction::Move(boss_node(candidate).id)));
    to_boss_door(&mut run);
    let boss = boss_node(&run).id;
    run.step(&GameAction::Move(boss)).unwrap();
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, original_order[0]);
    lethal(&mut run);
    run.settle_combat().unwrap();
    proceed(&mut run);
    assert_eq!(run.state.map.as_ref().unwrap().current, Some(SECOND));
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, original_order[1]);
}

/// Secret Portal at A20 all seen: both bosses, then the ordinary ending. Without keys the Run ends there
/// (`Act3NoKeys`); with them (`heart`) the Act 4 route follows through the Heart's Proceed.
fn secret_portal_path(heart: bool) {
    let (mut run, node) = event_run(Character::Ironclad, EventId::SecretPortal, 0xA4);
    let order = order(&run).unwrap();
    let event_entry_floor = run.state.floor + 1;
    run.step(&GameAction::Move(node)).unwrap();
    choose_event(&mut run, 0);
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, order[0]);
    assert_eq!(run.state.floor, event_entry_floor + 1, "portal boss follows the event room by one floor");
    lethal(&mut run);
    run.settle_combat().unwrap();
    proceed(&mut run);
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, order[1]);
    assert_eq!(run.state.map.as_ref().unwrap().current, Some(SECOND));
    assert_eq!(run.state.floor, event_entry_floor + 2);
    assert!(run.state.floor < run.state.map.as_ref().unwrap().nodes.iter().find(|n| n.id == SECOND).unwrap().floor);
    lethal(&mut run);
    run.settle_combat().unwrap();
    proceed(&mut run);
    assert_eq!(kind(&run), RoomViewKind::Act3Ending);
    if !heart {
        run.state.keys = [false; 3];
        super::m28_world::drive(&mut run, |_| false);
        assert_eq!(run.state.terminal, Some(RunTerminal::Act3NoKeys));
        assert!(run.state.fault.is_none());
        return
    }
    run.state.keys = [true; 3];
    super::m28_world::drive(&mut run, |candidate| candidate.state.act == 4 && candidate.state.active_combat.as_ref().is_some_and(|c| c.encounter == crate::content::EncounterId::CorruptHeart));
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, crate::content::EncounterId::CorruptHeart);
    super::m28_world::win(&mut run);
    assert_eq!(kind(&run), RoomViewKind::HeartVictory);
    assert_eq!(run.state.floor, event_entry_floor + 6, "Secret Portal's Act 4 rooms continue one floor at a time");
    let heart_proceed = rooms::public(&run).unwrap().choices.iter().find(|row| row.kind == RoomChoiceKind::Proceed).unwrap().id;
    run.step(&GameAction::Choose(heart_proceed)).unwrap();
    assert_eq!(run.state.terminal, Some(RunTerminal::HeartVictory));
    assert_eq!(run.state.room, "TrueVictoryRoom");
    assert_eq!(run.state.floor, event_entry_floor + 7);
    let act4_floors = run.state.history.rooms.iter().filter(|entry| entry.act == 4).map(|entry| entry.floor).collect::<Vec<_>>();
    assert_eq!(act4_floors.windows(2).map(|pair| pair[1] - pair[0]).collect::<Vec<_>>(), vec![1, 1, 1, 1]);
}

#[test]
fn m33_a20_secret_portal_double_boss() { secret_portal_path(false); }

#[test]
fn m33_a20_heart_path_after_portal() { secret_portal_path(true); }

#[test]
fn m33_non_standard_a20_act3_typed() {
    fn fixture(ascension: u8) -> Run {
        let mut profile = CreationProfile::experienced();
        profile.neow_eligible = false;
        profile.seen_bosses[0] = vec!["Hexaghost", "SlimeBoss"];
        profile.seen_bosses[1] = vec!["Automaton", "Collector"];
        profile.seen_bosses[2] = vec!["AwakenedOne", "DonuAndDeca"];
        let mut run = Run::new_room_scenario(RunConfig { character: Character::Silent, ascension: 0 }, profile,
            rng(ascension.wrapping_add(3)), RoomScenario::M14SilentAct3).unwrap();
        let choice = rooms::public(&run).unwrap().choices.into_iter().find(|c| c.kind == RoomChoiceKind::NeowLament).unwrap();
        run.step(&GameAction::Choose(choice.id)).unwrap();
        run.state.relics.retain(|id| *id != RelicId::NeowsLament);
        run.state.relic_counters.remove(&RelicId::NeowsLament);
        let mut rng = run.state.rng.clone();
        let mut map = crate::run::map::generate(3, &mut rng, crate::run::map::MapEligibility::from_run(&run.state)).unwrap();
        for node in &mut map.nodes { node.floor += 34; }
        let boss = map.nodes.iter().find(|n| n.floor == 50 && n.kind == MapNodeKind::Boss).unwrap().clone();
        let predecessor = map.nodes.iter_mut().find(|n| n.floor == 49 && n.edges.contains(&boss.id)).unwrap();
        predecessor.visited = true;
        let predecessor_id = predecessor.id;
        map.current = Some(predecessor_id);
        map.boss_icon = Some(crate::content::EncounterId::AwakenedOne);
        run.state.act = 3; run.state.floor = 49; run.state.map = Some(map); run.state.rng = rng;
        let room = run.state.room_loop.as_mut().unwrap(); room.act1 = None; room.act2 = None; room.act3 = Some(Default::default());
        room.act3_boss_order = Some(vec![crate::content::EncounterId::AwakenedOne]);
        run.state.room_loop.as_mut().unwrap().screen = RoomView { kind: RoomViewKind::Map, choices: vec![] };
        run.step(&GameAction::Move(boss.id)).unwrap();
        run.state.ascension = ascension;
        run
    }
    for ascension in [0, 20] {
        let mut run = fixture(ascension);
        lethal(&mut run);
        if ascension == 0 {
            run.settle_combat().unwrap();
            assert!(run.state.fault.is_none());
        } else {
            let error = run.settle_combat().unwrap_err();
            assert!(matches!(error, RunError::Faulted(ref fault)
                if fault.kind == RunFaultKind::UnsupportedBoundary("A20 Act 3 double boss is Standard-only")
                    && fault.checkpoint == rooms::act3_end::fault_checkpoint(&run.state)), "{error:?}");
        }
    }
}
