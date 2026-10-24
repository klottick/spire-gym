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

fn boss_selection(state: &mut RunState, choices: &[Weighted]) -> Result<EncounterId, RunError> {
    if choices.is_empty() { return Err(RunError::Registry("empty boss encounter domain")); }
    let seen = state.seen_bosses.get((state.act.saturating_sub(1)) as usize)
        .ok_or(RunError::Registry("act boss history unavailable"))?;
    let source_name = |id: EncounterId| match id {
        EncounterId::Guardian=>"Guardian", EncounterId::Hexaghost=>"Hexaghost", EncounterId::SlimeBoss=>"SlimeBoss",
        EncounterId::Champ=>"Champ", EncounterId::BronzeAutomaton=>"Automaton", EncounterId::Collector=>"Collector",
        EncounterId::AwakenedOne=>"AwakenedOne", EncounterId::DonuAndDeca=>"DonuAndDeca", EncounterId::TimeEater=>"TimeEater",
        _=>"",
    };
    let unseen = choices.iter().filter(|choice| !seen.iter().any(|name| *name == source_name(choice.id))).collect::<Vec<_>>();
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
    if kind == MapNodeKind::Boss { return boss_selection(state, choices); }
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

fn resolve_question(state: &mut RunState, leaving_shop: bool) -> Result<MapNodeKind, RunError> {
    let map = state.map.as_mut().ok_or(RunError::InvalidAction)?;
    if map.question_roll_ordinal == u64::MAX { return Err(RunError::Registry("question-room visit ordinal exhausted")); }
    let ordinal = map.question_roll_ordinal;
    let draw = state.rng.draw_scoped(RunRngScope::MapQuestionRoom, &[state.act as u64, ordinal]);
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
    let add = |bits: u32, increment: f32| (f32::from_bits(bits) + increment).to_bits();
    let next_monster = if result == MapNodeKind::Monster { 0x3DCCCCCD } else { add(monster_bits, 0.1) };
    let next_shop = if result == MapNodeKind::Shop { 0x3CF5C28F } else { add(shop_bits, 0.03) };
    let next_treasure = if result == MapNodeKind::Treasure { 0x3CA3D70A } else { add(treasure_bits, 0.02) };
    map.question_ramps = [next_monster, next_shop, next_treasure];
    map.question_roll_ordinal += 1;
    Ok(result)
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
    let target_node=preflight_move(state,target)?;

    super::reward::preflight(state,&[CommittedEvent::FloorEntered{act:state.act,floor:target_node.floor}])?;
    let mut kind = target_node.kind;
    let resolving_question = kind == MapNodeKind::Question;
    if resolving_question {
        let leaving_shop = state.room == "Shop";
        kind = resolve_question(state, leaving_shop)?;
    }
    let encounter = if matches!(kind, MapNodeKind::Monster | MapNodeKind::Elite | MapNodeKind::Boss) {
        Some(choose_encounter(state, kind)?)
    } else { None };

    let map = state.map.as_mut().expect("map checked");
    if map.current.is_some() && map.winged_moves > 0 {
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
    state.floor = node.floor;
    state.act = map.act;
    state.room = kind.label();
    let event = CommittedEvent::FloorEntered { act: map.act, floor: node.floor };
    state.history.rooms.push(super::VisitedRoom { act: map.act, floor: node.floor, label: kind.label() });
    super::reward::commit(state,&[event])?;

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
