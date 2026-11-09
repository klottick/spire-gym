//! M33 A20 Act 3 double boss (plan items 3, 5, 6; design §6.2–§6.5). Source: lane A §1–§7
//! (`TheBeyond.initializeBoss`, `ProceedButton:124-135`, `goToDoubleBoss`), PM floor ruling (second boss
//! floor + 1; Act 4 one higher after a double boss). Literals are worked by hand from the lane files:
//! the synthetic node `0xA300_0051` (design §6.3), floors 50/51 and Act 4 52–55/56. Combats are won with
//! a constructed lethal hit (supplied frontier, not a strategy claim).
use super::super::*;
use crate::combat::{CombatAction, Outcome};
use crate::combat::rng::Partition;
use crate::content::{Character, EncounterId};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
const ACT3_BOSSES: [(&str, EncounterId); 3] = [("AwakenedOne", EncounterId::AwakenedOne), ("TimeEater", EncounterId::TimeEater), ("DonuAndDeca", EncounterId::DonuAndDeca)];
pub(super) const SECOND: NodeId = NodeId(0xA300_0051);

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
pub(super) fn kind(run: &Run) -> RoomViewKind { rooms::public(run).unwrap().kind }

/// A Standard Run at `ascension` past the short Neow, then moved to a generated Act 3 map with global
/// floors (as `diagnostic_event_run` does). `start_act` builds the Act 3 boss order from the act-start
/// `seen_bosses` (all seen unless `seen` is given). Supplied 999/999 HP.
pub(super) fn act3_run(character: Character, ascension: u8, seed: u8, seen: Option<Vec<&'static str>>) -> Run {
    let profile = CreationProfile { neow_eligible: false, ..CreationProfile::experienced() };
    let mut run = Run::new_room_scenario(RunConfig { character, ascension }, profile, rng(seed), RoomScenario::Standard).unwrap();
    let neow = rooms::public(&run).unwrap().choices[1].id;
    run.step(&GameAction::Choose(neow)).unwrap();
    run.state.max_hp = 999; run.state.hp = 999;
    if let Some(seen) = seen { run.state.seen_bosses[2] = seen; }
    run.state.act = 3;
    rooms::standard::start_act(&mut run).unwrap();
    for node in &mut run.state.map.as_mut().unwrap().nodes { node.floor += 34; }
    run.state.floor = 34;
    run
}
pub(super) fn boss_node(run: &Run) -> MapNode { run.state.map.as_ref().unwrap().nodes.iter().find(|n| n.kind == MapNodeKind::Boss && n.id != SECOND).unwrap().clone() }
pub(super) fn order(run: &Run) -> Option<Vec<EncounterId>> { run.state.room_loop.as_ref().unwrap().act3_boss_order.clone() }
/// Drive to the Act 3 map with the boss node a legal move.
pub(super) fn to_boss_door(run: &mut Run) {
    let boss = boss_node(run).id;
    super::m28_world::drive(run, |r| r.state.act == 3 && r.state.pending.is_none() && kind(r) == RoomViewKind::Map
        && r.decision().unwrap().candidates.iter().any(|c| c.action == GameAction::Move(boss)));
    assert!(run.state.terminal.is_none(), "{:?}", run.state.terminal);
}
/// Constructed lethal hits until the fight is decided; returns the player's public HP before healing.
pub(super) fn lethal(run: &mut Run) -> i32 {
    let full = |run: &Run| run.state.combat.as_ref().unwrap().public_full().unwrap();
    for _ in 0..12 {
        let alive: Vec<usize> = full(run).enemies.iter().enumerate().filter(|(_, e)| e.hp > 0).map(|(i, _)| i).collect();
        for slot in alive { if full(run).outcome.is_none() { super::m9_boss::hit(run, slot, 2_000) } }
        if full(run).outcome.is_some() { break }
        run.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();
    }
    let view = full(run);
    assert_eq!(view.outcome, Some(Outcome::Victory));
    view.hp_before_healing.unwrap()
}
pub(super) fn proceed(run: &mut Run) -> RunTransition {
    let public = rooms::public(run).unwrap();
    assert_eq!(public.kind, RoomViewKind::Act3BossComplete);
    assert_eq!(public.choices.iter().map(|c| c.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Proceed]);
    let id = public.choices[0].id;
    run.step(&GameAction::Choose(id)).unwrap()
}
/// An A20 all-seen Run in its first Act 3 boss fight.
pub(super) fn in_first_boss(character: Character, seed: u8) -> Run {
    let mut run = act3_run(character, 20, seed, None);
    to_boss_door(&mut run);
    let boss = boss_node(&run).id;
    run.step(&GameAction::Move(boss)).unwrap();
    run
}
/// Past the first boss's settlement and the Proceed: in the second boss fight.
pub(super) fn in_second_boss(character: Character, seed: u8) -> Run {
    let mut run = in_first_boss(character, seed);
    lethal(&mut run);
    run.settle_combat().unwrap();
    proceed(&mut run);
    run
}

/// Item 3: A20 with every Act 3 boss seen stores two distinct bosses, the first on the map icon; a seed
/// sweep reaches all six ordered pairs (the source's shuffled three-boss list, its first two entries).
#[test]
fn m33_a20_all_seen_order_two_distinct() {
    let mut pairs = std::collections::BTreeSet::new();
    for seed in 0..48u8 {
        let run = act3_run(Character::Ironclad, 20, seed, None);
        let order = order(&run).unwrap_or_else(|| panic!("seed {seed}: no stored order"));
        assert_eq!(order.len(), 2, "seed {seed}");
        assert_ne!(order[0], order[1], "seed {seed}");
        assert!(order.iter().all(|boss| ACT3_BOSSES.iter().any(|(_, id)| id == boss)), "seed {seed}: {order:?}");
        assert_eq!(run.state.map.as_ref().unwrap().boss_icon, Some(order[0]), "seed {seed}");
        pairs.insert(format!("{:?}>{:?}", order[0], order[1]));
    }
    assert_eq!(pairs.len(), 6, "{pairs:?}");
}

/// Item 3 (lane A FINDING 1): with any Act 3 boss unseen the source list is one boss duplicated, so
/// one boss is fought: the first unseen in the priority CROW, DONUT, WIZARD.
#[test]
fn m33_a20_any_unseen_order_single() {
    let priority = [EncounterId::AwakenedOne, EncounterId::DonuAndDeca, EncounterId::TimeEater];
    let name = |id: EncounterId| ACT3_BOSSES.iter().find(|(_, boss)| *boss == id).unwrap().0;
    for mask in 0..7u8 {
        let seen: Vec<&'static str> = (0..3).filter(|bit| mask & (1 << bit) != 0).map(|bit| name(priority[bit])).collect();
        let expected = *priority.iter().find(|id| !seen.contains(&name(**id))).unwrap();
        let mut run = act3_run(Character::Silent, 20, 0x21, Some(seen.clone()));
        assert_eq!(run.state.map.as_ref().unwrap().boss_icon, Some(expected), "seen {seen:?}");
        assert!(order(&run).is_none_or(|order| order == vec![expected]), "seen {seen:?}: {:?}", order(&run));
        // The fight marks its boss seen; the order is not re-derived: one boss, then the ending.
        to_boss_door(&mut run);
        let boss = boss_node(&run).id;
        run.step(&GameAction::Move(boss)).unwrap();
        lethal(&mut run);
        run.settle_combat().unwrap();
        proceed(&mut run);
        assert_eq!(kind(&run), RoomViewKind::Act3Ending, "seen {seen:?}");
        assert!(run.state.map.as_ref().unwrap().nodes.iter().all(|n| n.id != SECOND));
    }
}

/// Item 3 t−1 (preservation): A19 all seen keeps one boss; its first draw equals A20's first entry
/// (the second draw's key is new).
#[test]
fn m33_a19_order_single_unchanged() {
    for seed in 0..12u8 {
        let a19 = act3_run(Character::Defect, 19, seed, None);
        let a20 = act3_run(Character::Defect, 20, seed, None);
        assert!(order(&a19).is_none(), "seed {seed}: {:?}", order(&a19));
        assert_eq!(a19.state.map.as_ref().unwrap().boss_icon, Some(order(&a20).unwrap()[0]), "seed {seed}");
    }
}

/// Item 5: the first A20 boss victory has no reward screen (no gold, card, potion, relic, chest or
/// key), no heal; victory hooks only (Ironclad Burning Blood +6). One Proceed.
#[test]
fn m33_a20_first_boss_settlement_no_rewards() {
    for (i, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = act3_run(character, 20, 0x40 + i as u8, None);
        to_boss_door(&mut run);
        run.state.hp = 900;
        // Before the room: relics and potions are in combat custody during the fight.
        let (gold, deck, potions, relics, keys, max_hp) = (run.state.gold, run.state.deck.clone(), run.state.potions.clone(), run.state.relics.clone(), run.state.keys, run.state.max_hp);
        let boss = boss_node(&run).id;
        run.step(&GameAction::Move(boss)).unwrap();
        let order = order(&run).unwrap();
        assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, order[0]);
        let before = lethal(&mut run);
        assert!(before > 0 && before < 990, "{character:?}: {before}");
        let transition = run.settle_combat().unwrap();
        assert_eq!(kind(&run), RoomViewKind::Act3BossComplete, "{character:?}");
        assert_eq!(rooms::public(&run).unwrap().choices.iter().map(|c| c.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Proceed], "{character:?}");
        let room = run.state.room_loop.as_ref().unwrap();
        assert!(room.active_rewards.is_none() && room.reward_parent.is_none() && room.boss.is_none(), "{character:?}");
        assert_eq!((run.state.gold, &run.state.deck, &run.state.potions, &run.state.relics, run.state.keys, run.state.max_hp), (gold, &deck, &potions, &relics, keys, max_hp), "{character:?}");
        let healed = if character == Character::Ironclad { 6 } else { 0 };
        assert_eq!(run.state.hp as i32, before + healed, "{character:?}");
        assert!(transition.terminal.is_none() && run.state.fault.is_none());
    }
}

/// Item 5: the Proceed enters the second boss as an ordinary room: floor + 1 (51), `FloorEntered{3,51}`,
/// origin the synthetic node, the other boss of the order, the map icon unchanged, +1 ledger credit.
#[test]
fn m33_a20_second_boss_entry() {
    for (i, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = in_first_boss(character, 0x50 + i as u8);
        let order = order(&run).unwrap();
        let boss = boss_node(&run);
        lethal(&mut run);
        run.settle_combat().unwrap();
        assert_eq!(run.state.floor, 50, "{character:?}");
        let rooms_before = run.state.history.rooms.len();
        let transition = proceed(&mut run);
        assert!(run.state.fault.is_none(), "{character:?}: {:?}", run.state.fault);
        let active = run.state.active_combat.as_ref().unwrap();
        assert_eq!(active.origin, CombatOrigin::MapRoom { node: SECOND, kind: MapNodeKind::Boss }, "{character:?}");
        assert_eq!(active.encounter, order[1], "{character:?}");
        assert_ne!(order[1], order[0]);
        assert_eq!(run.state.floor, 51, "{character:?}");
        assert!(transition.committed_events.contains(&CommittedEvent::FloorEntered { act: 3, floor: 51 }), "{character:?}");
        assert_eq!(transition.reward, 1, "{character:?}: the ledger's new Act 3 floor");
        let map = run.state.map.as_ref().unwrap();
        assert_eq!(map.boss_icon, Some(order[0]), "{character:?}");
        assert_eq!(map.current, Some(SECOND));
        let second = map.nodes.iter().find(|n| n.id == SECOND).unwrap();
        assert_eq!((second.kind, second.floor, second.x, second.y, second.visited, second.edges.is_empty(), second.encounter), (MapNodeKind::Boss, 51, boss.x, boss.y, true, true, Some(order[1])));
        assert_eq!(run.state.history.rooms.len(), rooms_before + 1);
        assert_eq!(run.state.history.rooms.last().map(|r| (r.act, r.floor)), Some((3, 51)));
        let names: Vec<&str> = order.iter().map(|id| ACT3_BOSSES.iter().find(|(_, boss)| boss == id).unwrap().0).collect();
        assert!(names.iter().all(|name| run.state.seen_bosses[2].contains(name)), "{character:?}");
        assert_eq!(run.state.room_loop.as_ref().unwrap().act3_boss_order.clone(), Some(vec![order[1]]), "{character:?}: the first room consumed the list head");
        assert!(run.state.room_loop.as_ref().unwrap().act3_end.is_none());
    }
}

/// Item 5: the second victory has no reward either and reaches the ordinary Act 3 ending; without the
/// keys the Run ends there (`Act3NoKeys`).
#[test]
fn m33_a20_second_boss_settlement_reaches_ending() {
    for (i, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = in_second_boss(character, 0x60 + i as u8);
        run.state.keys = [false; 3];
        let (gold, deck) = (run.state.gold, run.state.deck.clone());
        lethal(&mut run);
        run.settle_combat().unwrap();
        assert_eq!(kind(&run), RoomViewKind::Act3BossComplete, "{character:?}");
        assert_eq!((run.state.gold, &run.state.deck), (gold, &deck));
        proceed(&mut run);
        assert_eq!(kind(&run), RoomViewKind::Act3Ending, "{character:?}");
        assert!(run.state.combat.is_none());
        super::m28_world::drive(&mut run, |_| false);
        assert_eq!(run.state.terminal, Some(RunTerminal::Act3NoKeys), "{character:?}");
        assert!(run.state.fault.is_none());
    }
}

/// Item 5: a death in the second fight is the ordinary `Death` terminal.
#[test]
fn m33_a20_death_at_second_boss() {
    let mut run = in_first_boss(Character::Watcher, 0x70);
    lethal(&mut run);
    run.settle_combat().unwrap();
    run.state.hp = 1;
    proceed(&mut run);
    assert_eq!(run.state.active_combat.as_ref().unwrap().origin, CombatOrigin::MapRoom { node: SECOND, kind: MapNodeKind::Boss });
    // A death settles inside the step that decides it.
    for _ in 0..12 {
        if run.state.terminal.is_some() { break }
        run.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();
    }
    assert_eq!(run.state.terminal, Some(RunTerminal::Death));
    assert!(run.state.combat.is_none() && run.state.fault.is_none());
}

/// Item 5 t−1 (preservation): A19 fights one Act 3 boss; its Proceed opens the ending dialogs.
#[test]
fn m33_a19_single_boss_ending_unchanged() {
    for (i, character) in CHARACTERS.into_iter().enumerate() {
        let mut run = act3_run(character, 19, 0x50 + i as u8, None);
        to_boss_door(&mut run);
        let boss = boss_node(&run).id;
        run.step(&GameAction::Move(boss)).unwrap();
        lethal(&mut run);
        run.settle_combat().unwrap();
        proceed(&mut run);
        assert_eq!(kind(&run), RoomViewKind::Act3Ending, "{character:?}");
        assert!(run.state.map.as_ref().unwrap().nodes.iter().all(|n| n.id != SECOND));
        assert_eq!(run.state.floor, 50);
    }
}

/// Act 4 floors after the ending with three keys; returns (node floors, Act 4 history floors, terminal).
fn act4_floors(mut run: Run) -> (Vec<u32>, Vec<u32>, Option<RunTerminal>) {
    run.state.keys = [true; 3];
    super::m28_world::drive(&mut run, |r| r.state.act == 4);
    assert!(run.state.fault.is_none(), "{:?}", run.state.fault);
    let nodes = run.state.map.as_ref().unwrap().nodes.iter().map(|n| n.floor).collect();
    super::m28_world::drive(&mut run, |_| false);
    assert!(run.state.fault.is_none(), "{:?}", run.state.fault);
    let rooms = run.state.history.rooms.iter().filter(|r| r.act == 4).map(|r| r.floor).collect();
    (nodes, rooms, run.state.terminal)
}

/// Item 6: after a double boss the Act 4 route is one higher (M10's collapsed mapping + 1, PM ruling):
/// Rest 52, Shop 53, Shield and Spear 54, Heart 55, True Victory 56.
#[test]
fn m33_a20_act4_route_floors() {
    let mut run = in_second_boss(Character::Ironclad, 0x80);
    lethal(&mut run);
    run.settle_combat().unwrap();
    assert_eq!(act4_floors(run), (vec![52, 53, 54, 55], vec![52, 53, 54, 55, 56], Some(RunTerminal::HeartVictory)));
}

/// Item 6 t−1 (preservation): A19 keeps Rest 51 … Heart 54, True Victory 55.
#[test]
fn m33_a19_act4_route_floors_unchanged() {
    let mut run = act3_run(Character::Ironclad, 19, 0x80, None);
    to_boss_door(&mut run);
    let boss = boss_node(&run).id;
    run.step(&GameAction::Move(boss)).unwrap();
    lethal(&mut run);
    run.settle_combat().unwrap();
    assert_eq!(act4_floors(run), (vec![51, 52, 53, 54], vec![51, 52, 53, 54, 55], Some(RunTerminal::HeartVictory)));
}

/// Item 5: edits between the fights fault as an invariant, never a silent pass.
#[test]
fn m33_a20_double_boss_tamper_faults() {
    let after_first = || {
        let mut run = in_first_boss(Character::Ironclad, 0x90);
        lethal(&mut run);
        run.settle_combat().unwrap();
        run
    };
    let invariant = |run: &mut Run, context: &str| {
        let id = rooms::public(run).unwrap().choices[0].id;
        let error = run.step(&GameAction::Choose(id)).err();
        assert!(matches!(error, Some(RunError::Faulted(ref fault)) if matches!(fault.kind, RunFaultKind::Invariant(_))), "{context}: {error:?}");
        assert!(run.state.combat.is_none(), "{context}");
    };
    let mut run = after_first();
    run.state.last_combat.as_mut().unwrap().visit += 1;
    invariant(&mut run, "changed last combat");
    let mut run = after_first();
    let first = order(&run).unwrap()[0];
    run.state.room_loop.as_mut().unwrap().act3_boss_order = Some(vec![first]);
    invariant(&mut run, "order cut to one");
    let mut run = after_first();
    run.state.room_loop.as_mut().unwrap().act3_boss_order.as_mut().unwrap().swap(0, 1);
    invariant(&mut run, "order swapped");
    // The synthetic node removed during the second fight: its settlement faults.
    let mut run = in_second_boss(Character::Ironclad, 0x91);
    lethal(&mut run);
    run.state.map.as_mut().unwrap().nodes.retain(|n| n.id != SECOND);
    let error = run.settle_combat().err();
    assert!(matches!(error, Some(RunError::Faulted(ref fault)) if matches!(fault.kind, RunFaultKind::Invariant(_))), "removed node: {error:?}");
}

/// Final review m6: the second boss's victory sources the ending only with the first boss won at the map's
/// boss node: a missing first `CombatFinished` Victory or an edited first boss node faults at settlement.
#[test]
fn m33_a20_second_victory_requires_the_first() {
    let settle_faults = |run: &mut Run, context: &str| {
        lethal(run);
        let error = run.settle_combat().err();
        assert!(matches!(error, Some(RunError::Faulted(ref fault)) if matches!(fault.kind, RunFaultKind::Invariant(_))), "{context}: {error:?}");
    };
    let mut run = in_second_boss(Character::Defect, 0x92);
    let first = order(&run).map(|_| run.state.map.as_ref().unwrap().boss_icon.unwrap()).unwrap();
    run.state.history.events.retain(|event| !matches!(event, CommittedEvent::CombatFinished(record) if record.encounter == first));
    settle_faults(&mut run, "first victory removed from history");
    let mut run = in_second_boss(Character::Defect, 0x93);
    let boss = boss_node(&run).id;
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == boss).unwrap().encounter = None;
    settle_faults(&mut run, "first boss node edited");
    // Control: untouched, the same Run reaches the ending.
    let mut run = in_second_boss(Character::Defect, 0x93);
    lethal(&mut run);
    run.settle_combat().unwrap();
    proceed(&mut run);
    assert_eq!(kind(&run), RoomViewKind::Act3Ending);
}
