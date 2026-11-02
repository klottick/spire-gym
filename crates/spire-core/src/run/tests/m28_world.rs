//! M28b world entry, act transitions, keys and Act 4 under `StandardA0` (chunk plan §4 M28b items
//! 1, 2, 17–19; design §6.6, §6.7, §7). Combats are won with a constructed lethal hit (supplied
//! frontier, not a strategy claim); routes avoid Event outcomes until M28c.
use super::super::*;
use crate::combat::{CombatAction, Outcome};
use crate::combat::rng::Partition;
use crate::content::{Character, EncounterId};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn rng(seed: u8) -> RunRng { RunRng::new([seed; 32], Partition::Test, 0) }
/// A standard Run past the short Neow (max-HP blessing), standing on the Act 1 map.
pub(super) fn on_map(character: Character, seed: u8) -> Run {
    let mut profile = CreationProfile::experienced(); profile.neow_eligible = false;
    let mut run = Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile, rng(seed), RoomScenario::StandardA0).unwrap();
    let id = rooms::public(&run).unwrap().choices[1].id;
    run.step(&GameAction::Choose(id)).unwrap();
    assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Map);
    run
}
fn alive(run: &Run) -> Vec<usize> {
    run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.iter().enumerate().filter(|(_, e)| e.hp > 0).map(|(i, _)| i).collect()
}
fn outcome(run: &Run) -> Option<Outcome> { run.state.combat.as_ref().unwrap().public_full().unwrap().outcome }
/// Constructed lethal frontier: hit every living enemy, end turns until the fight ends, settle.
pub(super) fn win(run: &mut Run) -> RunTransition {
    for _ in 0..12 {
        for slot in alive(run) { super::m9_boss::hit(run, slot, 2_000); if outcome(run).is_some() { break } }
        if outcome(run).is_some() { return run.settle_combat().unwrap() }
        run.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();
        if run.state.combat.is_none() { return RunTransition { reward: 0, terminal: run.state.terminal, committed_events: vec![] } }
    }
    panic!("constructed lethal frontier did not end the fight");
}
fn choose_kind(run: &mut Run, wanted: impl Fn(&RoomChoiceKind) -> bool) -> Option<RunTransition> {
    let id = rooms::public(run).unwrap().choices.iter().find(|c| c.status == RoomChoiceStatus::Offered && wanted(&c.kind)).map(|c| c.id)?;
    Some(run.step(&GameAction::Choose(id)).unwrap())
}
fn kind(run: &Run) -> RoomViewKind { rooms::public(run).unwrap().kind }

/// One non-combat, non-map screen step for the route driver: take gold, skip cards and relic
/// choices, rest, open chests, leave shops, walk the Act 3 ending.
fn advance_screen(run: &mut Run) {
    if run.state.pending.is_some() { run.step(&GameAction::Cancel).unwrap(); return }
    match kind(run) {
        RoomViewKind::Rewards => {
            if choose_kind(run, |k| matches!(k, RoomChoiceKind::Gold(_))).is_none() { choose_kind(run, |k| *k == RoomChoiceKind::Proceed).unwrap(); }
        }
        RoomViewKind::CardReward => { choose_kind(run, |k| *k == RoomChoiceKind::Skip).unwrap(); }
        RoomViewKind::Shop => { choose_kind(run, |k| *k == RoomChoiceKind::Leave).unwrap(); }
        RoomViewKind::Campfire => {
            if choose_kind(run, |k| matches!(k, RoomChoiceKind::Rest { .. })).is_none() { choose_kind(run, |_| true).unwrap(); }
        }
        RoomViewKind::Treasure => {
            if choose_kind(run, |k| *k == RoomChoiceKind::OpenChest).is_none() { choose_kind(run, |k| *k == RoomChoiceKind::Proceed).unwrap(); }
        }
        RoomViewKind::BossTreasure | RoomViewKind::BossRelicChoice => {
            if choose_kind(run, |k| matches!(k, RoomChoiceKind::Proceed | RoomChoiceKind::Skip)).is_none() { panic!("boss chest without skip") }
        }
        RoomViewKind::ProceedWithoutRelic => { choose_kind(run, |k| *k == RoomChoiceKind::ConfirmSkip).unwrap(); }
        RoomViewKind::Act3BossComplete | RoomViewKind::Act3Ending | RoomViewKind::HeartVictory | RoomViewKind::BossRelicChild => { choose_kind(run, |_| true).unwrap(); }
        other => panic!("unexpected standard screen {other:?}"),
    }
}
/// Move to the first legal node whose room is not an Event (the M28c interim leaves the Run unchanged).
fn move_avoiding_events(run: &mut Run) -> RunTransition {
    let moves = run.decision().unwrap().candidates.iter().filter_map(|c| match c.action { GameAction::Move(n) => Some(n), _ => None }).collect::<Vec<_>>();
    for target in moves {
        let before = run.clone();
        match run.step(&GameAction::Move(target)) {
            Ok(transition) => return transition,
            Err(RunError::Unsupported(rooms::standard::map::EVENT_ROOMS_INTERIM)) => {
                assert!(*run == before, "an interim event room leaves the whole Run unchanged");
            }
            Err(error) => panic!("standard move failed: {error:?}"),
        }
    }
    // Constructed route (chunk plan §4 M28b Done): when every next room is an Event, the first
    // candidate node becomes a Monster room.
    let target = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(n) => Some(n), _ => None }).unwrap();
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == target).unwrap().kind = MapNodeKind::Monster;
    run.step(&GameAction::Move(target)).unwrap()
}

/// Drive a standard Run until `stop` holds or the Run ends; returns the visited room kinds.
pub(super) fn drive(run: &mut Run, mut stop: impl FnMut(&Run) -> bool) -> Vec<MapNodeKind> {
    let mut visited = Vec::new();
    for _ in 0..2_000 {
        if run.state.terminal.is_some() || stop(run) { return visited }
        if run.state.combat.is_some() { win(run); continue }
        if kind(run) == RoomViewKind::Map && run.state.pending.is_none() {
            move_avoiding_events(run);
            let node = run.state.map.as_ref().unwrap().current.unwrap();
            let node = run.state.map.as_ref().unwrap().nodes.iter().find(|n| n.id == node).unwrap();
            visited.push(node.hidden_kind.unwrap_or(node.kind));
            continue
        }
        advance_screen(run);
    }
    panic!("route driver exceeded its action budget");
}

/// Item 1: every non-event room kind enters through the standard owner with its producer.
#[test]
fn m28_world_enter_routes_every_kind() {
    for character in CHARACTERS {
        let mut run = on_map(character, 0x41);
        run.state.max_hp = 999; run.state.hp = 999;
        let mut seen = std::collections::BTreeSet::new();
        let visited = drive(&mut run, |r| r.state.act == 2);
        for kind in visited { seen.insert(kind.label()); }
        for wanted in ["Monster", "Elite", "Rest", "Boss"] { assert!(seen.contains(wanted), "{character:?} visited {seen:?}"); }
        assert_eq!(run.state.act, 2, "{character:?} reached Act 2 through the boss chest");
        assert_eq!(rooms::public(&run).unwrap().kind, RoomViewKind::Map);
    }
}

/// Item 2: the entry gate admits the encounter `world` chose at the current node, and nothing else.
#[test]
fn m28_standard_combat_gate_uses_the_current_node() {
    let mut run = on_map(Character::Ironclad, 0x42);
    move_avoiding_events(&mut run);
    let active = run.state.active_combat.clone().unwrap();
    let CombatOrigin::MapRoom { node, kind } = active.origin else { panic!("map-room origin") };
    assert_eq!(kind, MapNodeKind::Monster);
    let map_node = run.state.map.as_ref().unwrap().nodes.iter().find(|n| n.id == node).unwrap().clone();
    assert_eq!(map_node.encounter, Some(active.encounter));
    win(&mut run);
    // Another encounter at the same node is not admitted, including the legacy M3 Jaw Worm/Cultist pair.
    let mut probe = run.clone(); probe.state.room_loop.as_mut().unwrap().screen = RoomView { kind: RoomViewKind::Map, choices: vec![] };
    let other = if active.encounter == EncounterId::Cultist { EncounterId::JawWorm } else { EncounterId::Cultist };
    assert_eq!(probe.clone().enter_owned(other, CombatOrigin::MapRoom { node, kind }).err(), Some(RunError::Unsupported("encounter outside authored M3 profile")),
        "another Act 1 encounter is refused");
    assert_eq!(probe.enter_owned(EncounterId::Hexaghost, CombatOrigin::MapRoom { node, kind }).err(), Some(RunError::Unsupported("encounter outside authored M3 profile")));
}

/// Item 17: Act 1 and 2 boss victory → rewards → boss chest → next act, with a fresh map and ramps,
/// the boss from live `seen_bosses`, full heal, and global floors.
#[test]
fn m28_act_transitions() {
    for character in CHARACTERS {
        let mut run = on_map(character, 0x43);
        run.state.max_hp = 999; run.state.hp = 999;
        let act1_boss = run.state.map.as_ref().unwrap().boss_icon.unwrap();
        drive(&mut run, |r| r.state.act == 2);
        let map = run.state.map.as_ref().unwrap();
        assert_eq!((map.act, run.state.floor, run.state.hp), (2, 17, run.state.max_hp), "{character:?}");
        assert!(map.nodes.iter().all(|n| n.floor >= 18), "{character:?} Act 2 floors are global");
        assert_eq!(map.question_ramps, crate::run::map::generate(2, &mut rng(1), crate::run::map::MapEligibility::from_run(&run.state)).unwrap().question_ramps, "ramps reset");
        assert!(matches!(map.boss_icon, Some(EncounterId::Champ | EncounterId::BronzeAutomaton | EncounterId::Collector)));
        let name = match act1_boss { EncounterId::Guardian => "Guardian", EncounterId::Hexaghost => "Hexaghost", _ => "SlimeBoss" };
        assert!(run.state.seen_bosses[0].contains(&name), "{character:?} the Act 1 map boss {act1_boss:?} was fought and recorded");
        assert!(run.state.history.events.iter().any(|e| matches!(e, CommittedEvent::CombatEntered { encounter, .. } if *encounter == act1_boss)));
        assert!(run.state.history.events.contains(&CommittedEvent::ActAdvanced { from: 1, to: 2 }));
        drive(&mut run, |r| r.state.act == 3);
        assert_eq!((run.state.act, run.state.floor), (3, 34), "{character:?}");
        assert!(matches!(run.state.map.as_ref().unwrap().boss_icon, Some(EncounterId::AwakenedOne | EncounterId::TimeEater | EncounterId::DonuAndDeca)));
        assert!(run.state.history.events.contains(&CommittedEvent::ActAdvanced { from: 2, to: 3 }));
    }
}

/// Items 18–19 and the chunk's Done line: a supplied standard Run with three keys reaches and beats
/// the Heart through non-event rooms; Act 4's floors 51–53 use the standard producers.
#[test]
fn m28_keys_and_act4_entry() {
    for character in CHARACTERS {
        let mut run = on_map(character, 0x44);
        run.state.max_hp = 999; run.state.hp = 999;
        drive(&mut run, |r| r.state.act == 3);
        run.state.keys = [true; 3];
        drive(&mut run, |r| r.state.act == 4);
        assert_eq!((run.state.act, rooms::public(&run).unwrap().kind), (4, RoomViewKind::Map), "{character:?} entered Act 4");
        // Floor 51: standard campfire (no Recall: Ruby owned).
        move_avoiding_events(&mut run);
        assert!(rooms::public(&run).unwrap().choices.iter().all(|c| c.kind != RoomChoiceKind::Recall), "{character:?}");
        drive(&mut run, |r| r.state.floor == 52 && kind(r) == RoomViewKind::Shop);
        // Floor 52: the standard shop (five class cards, two colourless, three relics, three potions).
        let shop = rooms::public(&run).unwrap();
        assert_eq!(shop.choices.iter().filter(|c| matches!(c.kind, RoomChoiceKind::Card { .. })).count(), 7, "{character:?}");
        assert_eq!(shop.choices.iter().filter(|c| matches!(c.kind, RoomChoiceKind::Relic(_))).count(), 3, "{character:?}");
        assert_eq!(shop.choices.iter().filter(|c| matches!(c.kind, RoomChoiceKind::Potion(_))).count(), 3, "{character:?}");
        drive(&mut run, |r| r.state.floor == 53 && kind(r) == RoomViewKind::Rewards);
        // Floor 53: the standard elite reward (elite gold 25–35 before bonuses, one card offer).
        let rewards = rooms::public(&run).unwrap();
        assert!(rewards.choices.iter().any(|c| matches!(c.kind, RoomChoiceKind::Relic(_))), "{character:?} elite relic row");
        assert!(rewards.choices.iter().any(|c| c.kind == RoomChoiceKind::CardReward), "{character:?} card offer");
        drive(&mut run, |_| false);
        assert_eq!(run.state.terminal, Some(RunTerminal::HeartVictory), "{character:?}");
    }
    // Without the three keys the Act 3 boss ends the Run at the accepted ending.
    let mut run = on_map(Character::Silent, 0x45);
    run.state.max_hp = 999; run.state.hp = 999;
    drive(&mut run, |_| false);
    assert_eq!(run.state.terminal, Some(RunTerminal::Act3NoKeys));
}

/// Ruby by Recall, Sapphire from a chest and Emerald from a burning elite (constructed key states).
#[test]
fn m28_standard_key_sources() {
    // Recall at a standard campfire when Act 4 is enabled and Ruby is not owned.
    let mut run = on_map(Character::Defect, 0x46);
    run.state.max_hp = 999; run.state.hp = 999;
    drive(&mut run, |r| kind(r) == RoomViewKind::Campfire);
    let options = rooms::public(&run).unwrap().choices.iter().map(|c| c.kind.clone()).collect::<Vec<_>>();
    assert_eq!(options.last(), Some(&RoomChoiceKind::Recall), "Recall comes last");
    choose_kind(&mut run, |k| *k == RoomChoiceKind::Recall).unwrap();
    assert_eq!(run.state.keys, [true, false, false]);
    // Emerald: a constructed burning mark on the first elite the route fights; its reward offers an
    // unlinked Emerald row while unowned, and claiming it sets the key.
    let mut run = on_map(Character::Ironclad, 0x47);
    run.state.max_hp = 999; run.state.hp = 999;
    let current = |r: &Run| r.state.map.as_ref().unwrap().current.and_then(|id| r.state.map.as_ref().unwrap().nodes.iter().find(|n| n.id == id).cloned());
    drive(&mut run, |r| r.state.combat.is_some() && current(r).is_some_and(|n| n.kind == MapNodeKind::Elite));
    let elite = current(&run).unwrap().id;
    for node in &mut run.state.map.as_mut().unwrap().nodes { node.burning = node.id == elite; }
    win(&mut run);
    let rows = rooms::public(&run).unwrap().choices;
    let relic = rows.iter().position(|c| matches!(c.kind, RoomChoiceKind::Relic(_))).unwrap();
    let emerald = rows.iter().position(|c| c.kind == RoomChoiceKind::Emerald).unwrap();
    assert!(emerald > relic);
    choose_kind(&mut run, |k| *k == RoomChoiceKind::Emerald).unwrap();
    assert!(run.state.keys[2]);
    // Unlinked: the relic row is still offered.
    assert_eq!(rooms::public(&run).unwrap().choices[relic].status, RoomChoiceStatus::Offered);
}

/// Design §6.6 usability scan (G8, constructed): with Coffee Dripper, Fusion Hammer and Ruby owned a
/// standard campfire has no usable option, so entering it completes the room at once (no stuck screen).
#[test]
fn m28_campfire_without_usable_option_completes() {
    for character in CHARACTERS {
        let mut run = on_map(character, 0x48);
        run.state.max_hp = 999; run.state.hp = 999;
        run.state.relics.extend([crate::content::RelicId::CoffeeDripper, crate::content::RelicId::FusionHammer]);
        run.state.keys[0] = true;
        let at_rest = |r: &Run| r.state.map.as_ref().unwrap().current
            .and_then(|id| r.state.map.as_ref().unwrap().nodes.iter().find(|n| n.id == id)).is_some_and(|n| n.kind == MapNodeKind::Rest);
        drive(&mut run, at_rest);
        assert!(at_rest(&run), "{character:?} reached a campfire");
        assert_eq!(kind(&run), RoomViewKind::Map, "{character:?} an unusable campfire completes");
        assert!(!run.decision().unwrap().candidates.is_empty(), "{character:?} the Run is not stuck");
    }
}

/// G9 (fuzz `items` driver, Act 4 shop "Choose … InvalidAction"): under Sozu a complete-admission
/// shop's potion rows are not advertised, since the purchase is rejected (economy research :190-197).
#[test]
fn m28_sozu_shop_potion_rows_not_advertised() {
    use crate::run::rooms::standard::shop::{diagnostic_shop, ShopSupply};
    for character in CHARACTERS {
        let run = diagnostic_shop(character, ShopSupply { gold: 999, deck: None, relics: vec![crate::content::RelicId::Sozu], potions: None, purge_cost: 75 }, rng(0x49)).unwrap();
        let view = rooms::public(&run).unwrap();
        let potions = view.choices.iter().filter(|c| matches!(c.kind, RoomChoiceKind::Potion(_))).collect::<Vec<_>>();
        assert_eq!(potions.len(), 3, "{character:?}");
        assert!(potions.iter().all(|c| c.status == RoomChoiceStatus::Disabled), "{character:?}");
        let advertised = run.decision().unwrap().candidates;
        assert!(potions.iter().all(|row| !advertised.iter().any(|c| c.action == GameAction::Choose(row.id))), "{character:?}");
        // Every advertised shop choice is accepted.
        for candidate in advertised.iter().filter(|c| matches!(c.action, GameAction::Choose(_))) {
            assert!(run.clone().step(&candidate.action).is_ok(), "{character:?} advertised {:?} rejected", candidate.action);
        }
    }
}
