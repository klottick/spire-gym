//! Map movement and room-entry checkpoints. Unwired room owners fault after
//! traversal commits; they never become a successful no-op room.
use crate::content::EncounterId;
use super::{CommittedEvent, MapNodeKind, NodeId, RunError, RunFaultKind, RunRngScope, RunState, RunTransition};

#[derive(Clone, Copy)]
struct Weighted { id: EncounterId, weight: u16 }
const ACT1_WEAK: &[Weighted] = &[
    Weighted { id: EncounterId::Cultist, weight: 2 }, Weighted { id: EncounterId::JawWorm, weight: 2 },
    Weighted { id: EncounterId::TwoLouse, weight: 2 }, Weighted { id: EncounterId::SmallSlimes, weight: 2 },
];
const ACT1_STRONG: &[Weighted] = &[
    Weighted { id: EncounterId::BlueSlaver, weight: 4 }, Weighted { id: EncounterId::RedSlaver, weight: 2 },
    Weighted { id: EncounterId::Looter, weight: 4 }, Weighted { id: EncounterId::LargeSlime, weight: 4 },
    Weighted { id: EncounterId::ThreeLouse, weight: 4 }, Weighted { id: EncounterId::TwoFungiBeasts, weight: 4 },
    Weighted { id: EncounterId::GremlinGang, weight: 2 }, Weighted { id: EncounterId::LotsOfSlimes, weight: 2 },
    Weighted { id: EncounterId::ExordiumThugs, weight: 3 }, Weighted { id: EncounterId::ExordiumWildlife, weight: 3 },
];
const ACT1_ELITE: &[Weighted] = &[
    Weighted { id: EncounterId::GremlinNob, weight: 1 }, Weighted { id: EncounterId::Lagavulin, weight: 1 },
    Weighted { id: EncounterId::ThreeSentries, weight: 1 },
];
const ACT1_BOSS: &[Weighted] = &[
    Weighted { id: EncounterId::Guardian, weight: 1 }, Weighted { id: EncounterId::Hexaghost, weight: 1 },
    Weighted { id: EncounterId::SlimeBoss, weight: 1 },
];
const ACT2_WEAK: &[Weighted] = &[
    Weighted { id: EncounterId::SphericGuardian, weight: 2 }, Weighted { id: EncounterId::Chosen, weight: 2 },
    Weighted { id: EncounterId::ShelledParasite, weight: 2 }, Weighted { id: EncounterId::ThreeByrds, weight: 2 },
    Weighted { id: EncounterId::TwoThieves, weight: 2 },
];
const ACT2_STRONG: &[Weighted] = &[
    Weighted { id: EncounterId::ChosenAndByrds, weight: 2 }, Weighted { id: EncounterId::SentryAndSphere, weight: 2 },
    Weighted { id: EncounterId::SnakePlant, weight: 6 }, Weighted { id: EncounterId::Snecko, weight: 4 },
    Weighted { id: EncounterId::CenturionAndHealer, weight: 6 }, Weighted { id: EncounterId::CultistAndChosen, weight: 3 },
    Weighted { id: EncounterId::ThreeCultists, weight: 3 }, Weighted { id: EncounterId::ShelledParasiteAndFungi, weight: 3 },
];
const ACT2_ELITE: &[Weighted] = &[
    Weighted { id: EncounterId::GremlinLeader, weight: 1 }, Weighted { id: EncounterId::Slavers, weight: 1 },
    Weighted { id: EncounterId::BookOfStabbing, weight: 1 },
];
const ACT2_BOSS: &[Weighted] = &[
    Weighted { id: EncounterId::Champ, weight: 1 }, Weighted { id: EncounterId::BronzeAutomaton, weight: 1 },
    Weighted { id: EncounterId::Collector, weight: 1 },
];
const ACT3_WEAK: &[Weighted] = &[
    Weighted { id: EncounterId::ThreeDarklings, weight: 2 }, Weighted { id: EncounterId::OrbWalker, weight: 2 },
    Weighted { id: EncounterId::ThreeShapes, weight: 2 },
];
const ACT3_STRONG: &[Weighted] = &[
    Weighted { id: EncounterId::SpireGrowth, weight: 1 }, Weighted { id: EncounterId::Transient, weight: 1 },
    Weighted { id: EncounterId::FourShapes, weight: 1 }, Weighted { id: EncounterId::Maw, weight: 1 },
    Weighted { id: EncounterId::SphereAndTwoShapes, weight: 1 }, Weighted { id: EncounterId::JawWormHorde, weight: 1 },
    Weighted { id: EncounterId::ThreeDarklings, weight: 1 }, Weighted { id: EncounterId::WrithingMass, weight: 1 },
];
const ACT3_ELITE: &[Weighted] = &[
    Weighted { id: EncounterId::GiantHead, weight: 2 }, Weighted { id: EncounterId::Nemesis, weight: 2 },
    Weighted { id: EncounterId::Reptomancer, weight: 2 },
];
const ACT3_BOSS: &[Weighted] = &[
    Weighted { id: EncounterId::AwakenedOne, weight: 1 }, Weighted { id: EncounterId::TimeEater, weight: 1 },
    Weighted { id: EncounterId::DonuAndDeca, weight: 1 },
];

fn domain(act: u8, kind: MapNodeKind, regular_count: usize) -> Option<&'static [Weighted]> {
    match (act, kind) {
        (1, MapNodeKind::Monster) => Some(if regular_count < 3 { ACT1_WEAK } else { ACT1_STRONG }),
        (1, MapNodeKind::Elite) => Some(ACT1_ELITE),
        (1, MapNodeKind::Boss) => Some(ACT1_BOSS),
        (2, MapNodeKind::Monster) => Some(if regular_count < 2 { ACT2_WEAK } else { ACT2_STRONG }),
        (2, MapNodeKind::Elite) => Some(ACT2_ELITE),
        (2, MapNodeKind::Boss) => Some(ACT2_BOSS),
        (3, MapNodeKind::Monster) => Some(if regular_count < 2 { ACT3_WEAK } else { ACT3_STRONG }),
        (3, MapNodeKind::Elite) => Some(ACT3_ELITE),
        (3, MapNodeKind::Boss) => Some(ACT3_BOSS),
        // Act 4 uses its separate fixed route, never this ordinary encounter scheduler.
        (4, MapNodeKind::Monster | MapNodeKind::Elite | MapNodeKind::Boss) => None,
        _ => None,
    }
}

/// The act's boss through the standard selector (M28 design §7).
pub(super) fn act_boss(state: &mut RunState) -> Result<EncounterId, RunError> {
    let choices = domain(state.act, MapNodeKind::Boss, 0).ok_or(RunError::Registry("no boss domain for act"))?;
    boss_selection(state, choices)
}

/// The Act 3 second boss's synthetic node (M33 design §6.3; source `goToDoubleBoss`'s
/// `MapRoomNode(-1, 15)` outside the map): appended after the map's boss node is won.
pub(in crate::run) const SECOND_BOSS_NODE: NodeId = NodeId(0xA300_0051);

/// The act's boss order, built once at act start from the act-start `seen_bosses` (M33 design §6.2;
/// source `TheBeyond.initializeBoss`, lane A §1). The first is `act_boss`. A standard A20 Act 3 with
/// every boss seen (the source's shuffled three-boss list) adds a second, drawn from the other two in
/// domain order under its own key; any unseen boss makes the source list one boss, duplicated.
pub(super) fn act_boss_order(state: &mut RunState) -> Result<Vec<EncounterId>, RunError> {
    let first = act_boss(state)?;
    if !(super::rooms::is_standard(state) && state.act == 3 && state.ascension >= 20) { return Ok(vec![first]) }
    let seen = state.seen_bosses.get(2).ok_or(RunError::Registry("act boss history unavailable"))?;
    if !ACT3_BOSS.iter().all(|boss| seen.contains(&source_boss_name(boss.id))) { return Ok(vec![first]) }
    let rest = ACT3_BOSS.iter().filter(|boss| boss.id != first).collect::<Vec<_>>();
    // Native draw, like the first (no source shuffle parity; RNG-001 class).
    let i = state.rng.bounded_scoped(RunRngScope::MapEncounter, &[3, MapNodeKind::Boss as u64, 0xB055, 1], rest.len() as u64);
    Ok(vec![first, rest[i].id])
}

fn source_boss_name(id: EncounterId) -> &'static str {
    match id {
        EncounterId::Guardian=>"Guardian", EncounterId::Hexaghost=>"Hexaghost", EncounterId::SlimeBoss=>"SlimeBoss",
        EncounterId::Champ=>"Champ", EncounterId::BronzeAutomaton=>"Automaton", EncounterId::Collector=>"Collector",
        EncounterId::AwakenedOne=>"AwakenedOne", EncounterId::DonuAndDeca=>"DonuAndDeca", EncounterId::TimeEater=>"TimeEater",
        _=>"",
    }
}

/// G1 (M33 design §6.6, lane B §6): the source's unseen priority. Acts 1–2 equal their domain order;
/// Act 3's (`TheBeyond.initializeBoss`: CROW, DONUT, WIZARD) does not.
fn unseen_priority(act: u8, id: EncounterId) -> usize {
    match (act, id) {
        (3, EncounterId::AwakenedOne) => 0, (3, EncounterId::DonuAndDeca) => 1, (3, EncounterId::TimeEater) => 2,
        _ => 0,
    }
}

fn boss_selection(state: &mut RunState, choices: &[Weighted]) -> Result<EncounterId, RunError> {
    if choices.is_empty() { return Err(RunError::Registry("empty boss encounter domain")); }
    let seen = state.seen_bosses.get((state.act.saturating_sub(1)) as usize)
        .ok_or(RunError::Registry("act boss history unavailable"))?;
    let mut unseen = choices.iter().filter(|choice| !seen.iter().any(|name| *name == source_boss_name(choice.id))).collect::<Vec<_>>();
    unseen.sort_by_key(|choice| unseen_priority(state.act, choice.id));
    if let Some(first) = unseen.first() { return Ok(first.id); }
    // The all-seen initialized shuffle is not reproduced; native independent
    // semantic selection keeps the complete eligible domain without claiming
    // source shuffle parity.
    let i = state.rng.bounded_scoped(RunRngScope::MapEncounter, &[state.act as u64, MapNodeKind::Boss as u64, 0xB055], choices.len() as u64);
    Ok(choices[i].id)
}

fn choose_encounter(state: &mut RunState, kind: MapNodeKind) -> Result<EncounterId, RunError> {
    let (regular_count, history, elite_history) = {
        let map = state.map.as_ref().expect("map checked");
        (map.regular_encounter_history.len(), map.regular_encounter_history.clone(), map.elite_encounter_history.clone())
    };
    let choices = domain(state.act, kind, regular_count).ok_or(RunError::Registry("complete qualified encounter domain not registered for this act/room"))?;
    if choices.is_empty() { return Err(RunError::Registry("empty encounter scheduling domain")); }
    // The boss room fights the act's boss already chosen for the map icon (design §7); a second
    // selection could differ once every boss of the act has been seen.
    if kind == MapNodeKind::Boss { return match state.map.as_ref().and_then(|map| map.boss_icon) { Some(boss) => Ok(boss), None => boss_selection(state, choices) }; }
    let (history, recent_count) = if kind == MapNodeKind::Elite { (&elite_history, 1) } else { (&history, 2) };
    let recent = history.iter().rev().take(recent_count).copied().collect::<Vec<_>>();
    let first_strong = kind == MapNodeKind::Monster && match state.act { 1=>regular_count==3, 2|3=>regular_count==2, _=>false };
    let special_exclusions: &[EncounterId] = if first_strong {
        let last_weak = history.last().copied();
        match (state.act, last_weak) {
            (1, Some(EncounterId::TwoLouse))=>&[EncounterId::ThreeLouse],
            (1, Some(EncounterId::SmallSlimes))=>&[EncounterId::LargeSlime, EncounterId::LotsOfSlimes],
            (2, Some(EncounterId::SphericGuardian))=>&[EncounterId::SentryAndSphere],
            (2, Some(EncounterId::ThreeByrds))=>&[EncounterId::ChosenAndByrds],
            (2, Some(EncounterId::Chosen))=>&[EncounterId::ChosenAndByrds, EncounterId::CultistAndChosen],
            (3, Some(EncounterId::ThreeDarklings))=>&[EncounterId::ThreeDarklings],
            (3, Some(EncounterId::ThreeShapes))=>&[EncounterId::FourShapes],
            (3, Some(EncounterId::OrbWalker))=>&[EncounterId::OrbWalker],
            _=>&[],
        }
    } else { &[] };
    let excluded = if first_strong { special_exclusions } else { &recent };
    let eligible = choices.iter().filter(|choice| !excluded.contains(&choice.id)).copied().collect::<Vec<_>>();
    if eligible.is_empty() { return Err(RunError::Registry("history exclusions exhausted encounter domain")); }
    let candidates = eligible.as_slice();
    let total = candidates.iter().map(|choice| choice.weight as f32).sum::<f32>();
    if total <= 0.0 { return Err(RunError::Registry("encounter weight total is not positive")); }
    let mut ordered = candidates.to_vec();
    ordered.sort_by_key(|choice| choice.weight);
    let bits = state.rng.draw_scoped(RunRngScope::MapEncounter,
        &[state.act as u64, kind as u64, history.len() as u64, recent_count as u64]);
    let roll = ((bits >> 40) as f32) / 16_777_216.0;
    let mut cumulative = 0.0f32;
    for choice in ordered {
        cumulative += choice.weight as f32 / total;
        if roll < cumulative { return Ok(choice.id); }
    }
    Err(RunError::Registry("encounter normalized-weight selection had no interval"))
}

fn resolve_question(state: &mut RunState, leaving_shop: bool, standard: bool) -> Result<MapNodeKind, RunError> {
    if state.map.as_ref().ok_or(RunError::InvalidAction)?.question_roll_ordinal == u64::MAX { return Err(RunError::Registry("question-room visit ordinal exhausted")); }
    let ordinal = state.map.as_ref().expect("map checked").question_roll_ordinal;
    let draw = state.rng.draw_scoped(RunRngScope::MapQuestionRoom, &[state.act as u64, ordinal]);
    // M28 standard Run (design §6.2): Tiny Chest counts this roll and Juzu converts a monster, through
    // M27's `run_laws::unknown_room`, called exactly once per roll after the room float is consumed
    // (event-neow research :140, :195-199).
    let rules = if standard { crate::items::run_laws::unknown_room(state) } else { Default::default() };
    let map = state.map.as_mut().expect("map checked");
    let cell = (((draw >> 40) as u64 * 100) / (1u64 << 24)) as u32;
    let [monster_bits, shop_bits, treasure_bits] = map.question_ramps;
    let f = |bits: u32| f32::from_bits(bits);
    let m = (f(monster_bits) * 100.0) as u32;
    let s = if leaving_shop { 0 } else { (f(shop_bits) * 100.0) as u32 };
    let t = (f(treasure_bits) * 100.0) as u32;
    let mut result = MapNodeKind::Event;
    if cell < m.min(100) { result = MapNodeKind::Monster; }
    if cell >= m.min(99) && cell < (m + s).min(100) { result = MapNodeKind::Shop; }
    if cell >= (m + s).min(99) && cell < (m + s + t).min(100) { result = MapNodeKind::Treasure; }
    // Tiny Chest's forced treasure ramps as a treasure result; Juzu's converted monster ramps as a
    // monster result (event-neow research :195-199).
    if rules.force_treasure { result = MapNodeKind::Treasure; }
    let add = |bits: u32, increment: f32| (f32::from_bits(bits) + increment).to_bits();
    let next_monster = if result == MapNodeKind::Monster { 0x3DCCCCCD } else { add(monster_bits, 0.1) };
    let next_shop = if result == MapNodeKind::Shop { 0x3CF5C28F } else { add(shop_bits, 0.03) };
    let next_treasure = if result == MapNodeKind::Treasure { 0x3CA3D70A } else { add(treasure_bits, 0.02) };
    map.question_ramps = [next_monster, next_shop, next_treasure];
    map.question_roll_ordinal += 1;
    if result == MapNodeKind::Monster && rules.no_monsters { result = MapNodeKind::Event; }
    Ok(result)
}

/// E6 (M28 design §5.2): a standard event room whose selection law finds no event becomes a
/// monster room of the act, chosen with its normal history; the node and room label say so.
pub(super) fn standard_event_fallback(state: &mut RunState, node: NodeId) -> Result<EncounterId, RunError> {
    let encounter = choose_encounter(state, MapNodeKind::Monster)?;
    let map = state.map.as_mut().ok_or(RunError::InvalidAction)?;
    let target = map.nodes.iter_mut().find(|n| n.id == node).ok_or(RunError::InvalidAction)?;
    target.encounter = Some(encounter);
    target.hidden_kind = Some(MapNodeKind::Monster);
    map.encounter_history.push(encounter);
    map.regular_encounter_history.push(encounter);
    state.room = MapNodeKind::Monster.label();
    if let Some(room) = state.history.rooms.last_mut() { room.label = MapNodeKind::Monster.label(); }
    Ok(encounter)
}

/// Enter a connected node. Illegal moves are rejected without mutation or RNG
/// consumption. Legal entry commits traversal and one floor event before the
/// room owner is invoked; unsupported constructors retain that progress as fault.
pub(super) fn preflight_move(state:&RunState,target:NodeId)->Result<super::MapNode,RunError>{
    if let Some(fault) = &state.fault { return Err(RunError::Faulted(fault.clone())); }
    if state.terminal.is_some() { return Err(RunError::Terminal); }
    let map = state.map.as_ref().ok_or(RunError::InvalidAction)?;
    if map.act != state.act { return Err(RunError::Registry("run/map act mismatch")); }
    let target_node = map.nodes.iter().find(|n| n.id == target).cloned().ok_or(RunError::InvalidAction)?;
    if target_node.visited { return Err(RunError::InvalidAction); }
    let legal = match map.current {
        Some(current) => {
            let Some(current_node) = map.nodes.iter().find(|n| n.id == current) else { return Err(RunError::InvalidAction) };
            current_node.edges.contains(&target)
                || map.winged_moves > 0 && target_node.y == current_node.y + 1
        }
        None => target_node.y == 0,
    };
    if !legal { return Err(RunError::InvalidAction); }
    Ok(target_node)
}

pub fn enter(state: &mut RunState, target: NodeId) -> Result<RunTransition, RunError> {
    let (kind, _, _, _) = commit_move(state, target, false)?;
    let fault_boundary = match kind {
        MapNodeKind::Monster | MapNodeKind::Elite | MapNodeKind::Boss => "Task6 encounter constructor/combat bridge",
        MapNodeKind::Question | MapNodeKind::Event => "event-room owner",
        MapNodeKind::Shop => "shop-room owner",
        MapNodeKind::Rest => "rest-room owner",
        MapNodeKind::Treasure => "treasure-room owner",
    };
    state.fault = Some(super::RunFault { kind: RunFaultKind::UnsupportedBoundary(fault_boundary), checkpoint: "run.world.room_entry", committed_events: state.history.events.clone() });
    Err(RunError::Faulted(state.fault.clone().unwrap()))
}

/// The M28 standard Run's map move (design §7): traversal, floor and history commit and the
/// resolved room kind and encounter. The caller (`rooms::standard::map`) stages the Run and
/// invokes the room producer.
pub(super) fn enter_standard(state: &mut RunState, target: NodeId) -> Result<(MapNodeKind, Option<EncounterId>, CommittedEvent, i32), RunError> {
    commit_move(state, target, true)
}

/// Secret Portal (PM check m28f/source-check-events-v1): the event's `nextRoom` is a `MonsterRoomBoss`
/// node outside the map's edges; `nextRoomTransition` advances the floor by one. Returns the act's boss
/// node and the shared commit's results.
/// The floor a room entry reaches (PM ruling m28f Q2, `AbstractDungeon.nextRoomTransition`: `floorNum++`
/// on every room entry, never reset by an act): the Run's floor + 1. Every map path reaches the node's
/// floor; only a standard Run after Secret Portal (Act 3 on) is below it. The captured profiles have no
/// portal, so their node floor is the same value; they keep it (their fixtures place the Run directly).
pub(in crate::run) fn room_floor(state: &RunState, node_floor: u32) -> Result<u32, RunError> {
    if !super::rooms::is_standard(state) { return Ok(node_floor) }
    let floor = state.floor.checked_add(1).ok_or(RunError::InvalidAction)?;
    debug_assert!(floor == node_floor || state.act >= 3 && floor < node_floor,
        "room floor {floor} differs from node floor {node_floor}");
    Ok(floor)
}

pub(super) fn enter_portal_boss(state: &mut RunState) -> Result<(NodeId, MapNodeKind, Option<EncounterId>, CommittedEvent, i32), RunError> {
    if let Some(fault) = &state.fault { return Err(RunError::Faulted(fault.clone())); }
    if state.terminal.is_some() { return Err(RunError::Terminal); }
    let map = state.map.as_ref().ok_or(RunError::InvalidAction)?;
    if map.act != state.act { return Err(RunError::Registry("run/map act mismatch")); }
    let boss = map.nodes.iter().find(|n| n.kind == MapNodeKind::Boss && !n.visited).cloned().ok_or(RunError::InvalidAction)?;
    let (kind, encounter, event, credit) = commit_node(state, boss.clone(), true, true, None)?;
    Ok((boss.id, kind, encounter, event, credit))
}

/// The A20 Act 3 second boss room (M33 design §6.3; source `ProceedButton.goToDoubleBoss`): the
/// order's head was fought, so it is consumed and the next boss is entered as an ordinary room
/// (`nextRoomTransition`: floor + 1, floor and room hooks). The synthetic node copies the boss node's
/// `x`/`y` and has no edges; it is committed like Secret Portal's entry (no Winged Boots charge). The
/// map's boss icon is not changed (`goToDoubleBoss` sets `bossKey` without `setBoss`).
pub(super) fn enter_second_boss(state: &mut RunState) -> Result<(NodeId, MapNodeKind, Option<EncounterId>, CommittedEvent, i32), RunError> {
    if let Some(fault) = &state.fault { return Err(RunError::Faulted(fault.clone())); }
    if state.terminal.is_some() { return Err(RunError::Terminal); }
    let map = state.map.as_ref().ok_or(RunError::InvalidAction)?;
    if map.act != state.act || state.act != 3 || map.nodes.iter().any(|n| n.id == SECOND_BOSS_NODE) { return Err(RunError::InvalidAction); }
    let boss = map.nodes.iter().find(|n| Some(n.id) == map.current && n.kind == MapNodeKind::Boss && n.visited).cloned().ok_or(RunError::InvalidAction)?;
    let order = state.room_loop.as_mut().and_then(|room| room.act3_boss_order.as_mut()).filter(|order| order.len() == 2).ok_or(RunError::InvalidAction)?;
    if boss.encounter != Some(order[0]) { return Err(RunError::InvalidAction); }
    order.remove(0);
    let second = order[0];
    let node = super::MapNode { id: SECOND_BOSS_NODE, floor: boss.floor.checked_add(1).ok_or(RunError::InvalidAction)?, label: boss.label,
        edges: vec![], x: boss.x, y: boss.y, kind: MapNodeKind::Boss, hidden_kind: None, visited: false, revealed: true, encounter: None, burning: false };
    state.map.as_mut().expect("map checked").nodes.push(node.clone());
    let (kind, encounter, event, credit) = commit_node(state, node, true, true, Some(second))?;
    Ok((SECOND_BOSS_NODE, kind, encounter, event, credit))
}

fn commit_move(state: &mut RunState, target: NodeId, standard: bool) -> Result<(MapNodeKind, Option<EncounterId>, CommittedEvent, i32), RunError> {
    let target_node=preflight_move(state,target)?;
    commit_node(state, target_node, standard, false, None)
}

/// The shared move commit; `portal` is Secret Portal's entry (no edge, no Winged Boots charge);
/// `fixed` is an encounter already chosen (the A20 second boss) instead of the scheduler's.
fn commit_node(state: &mut RunState, target_node: super::MapNode, standard: bool, portal: bool, fixed: Option<EncounterId>) -> Result<(MapNodeKind, Option<EncounterId>, CommittedEvent, i32), RunError> {
    let target = target_node.id;
    let entered_floor = room_floor(state, target_node.floor)?;
    super::reward::preflight(state,&[CommittedEvent::FloorEntered{act:state.act,floor:entered_floor}])?;
    let mut kind = target_node.kind;
    let resolving_question = kind == MapNodeKind::Question;
    if resolving_question {
        let leaving_shop = state.room == "Shop";
        kind = resolve_question(state, leaving_shop, standard)?;
    }
    let encounter = if fixed.is_some() { fixed } else if matches!(kind, MapNodeKind::Monster | MapNodeKind::Elite | MapNodeKind::Boss) {
        Some(choose_encounter(state, kind)?)
    } else { None };

    let map = state.map.as_mut().expect("map checked");
    if !portal && map.current.is_some() && map.winged_moves > 0 {
        let current = map.current.and_then(|id| map.nodes.iter().find(|n| n.id == id)).map(|n| n.y).unwrap_or(0);
        if target_node.y > current && !map.nodes.iter().find(|n| n.id == map.current.unwrap()).is_some_and(|n| n.edges.contains(&target)) {
            map.winged_moves -= 1;
        }
    }
    map.current = Some(target);
    let node = map.nodes.iter_mut().find(|n| n.id == target).expect("validated target");
    node.visited = true;
    node.revealed = true;
    if resolving_question { node.hidden_kind = Some(kind); }
    if let Some(encounter) = encounter {
        node.encounter = Some(encounter);
        map.encounter_history.push(encounter);
        if kind == MapNodeKind::Monster { map.regular_encounter_history.push(encounter); }
        if kind == MapNodeKind::Elite { map.elite_encounter_history.push(encounter); }
    }
    let floor=entered_floor;
    state.floor = floor;
    state.act = map.act;
    let map_act=map.act;
    if state.item_admission==crate::run::ItemAdmission::Complete {
        let mut item_event=crate::items::dispatch::RunItemEvent::FloorEntered{floor};
        // M27 seam: floor entered
        crate::items::dispatch::run_event(state,&mut item_event)?;
    }
    state.room = kind.label();
    let event = CommittedEvent::FloorEntered { act: map_act, floor };
    state.history.rooms.push(super::VisitedRoom { act: map_act, floor, label: kind.label() });
    let credit = super::reward::commit(state,std::slice::from_ref(&event))?;
    Ok((kind, encounter, event, credit))
}
