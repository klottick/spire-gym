//! Shared public key objectives; the caller supplies its explicit finite profile.
/// Prefer only offered key objectives visible in the current public decision.
pub fn key_action(
    decision: &spire_core::run::PublicDecision,
    map: Option<&spire_core::run::map::PublicMap>,
    profile: spire_core::run::RoomScenario,
) -> Option<spire_core::run::GameAction> {
    use spire_core::run::{RoomChoiceKind, RoomViewKind};

    if decision.content_profile != Some(profile)
        || decision.combat.is_some()
        || decision.selection.is_some()
    {
        return None;
    }
    let room = decision.room.as_ref()?;

    if !decision.player.keys[0] {
        if let Some(action) = room_choice_action(decision, |kind| kind == &RoomChoiceKind::Recall) {
            return Some(action);
        }
    }
    if !decision.player.keys[1] {
        if let Some(action) = room_choice_action(decision, |kind| kind == &RoomChoiceKind::Sapphire) {
            return Some(action);
        }
        if matches!(room.kind, RoomViewKind::Treasure | RoomViewKind::BossTreasure) {
            if let Some(action) = room_choice_action(decision, |kind| kind == &RoomChoiceKind::OpenChest) {
                return Some(action);
            }
        }
    }
    if !decision.player.keys[2] {
        if let Some(action) = room_choice_action(decision, |kind| kind == &RoomChoiceKind::Emerald) {
            return Some(action);
        }
    }

    match room.kind {
        RoomViewKind::Map if !decision.player.keys[2] => map.and_then(|map| emerald_route_action(decision, map)),
        RoomViewKind::Act3Ending => room_choice_action(decision, |kind| kind == &RoomChoiceKind::Act3Continue { stage: 4 }),
        RoomViewKind::HeartVictory => room_choice_action(decision, |kind| kind == &RoomChoiceKind::Proceed),
        _ => None,
    }
}

fn room_choice_action(
    decision: &spire_core::run::PublicDecision,
    wanted: impl Fn(&spire_core::run::RoomChoiceKind) -> bool,
) -> Option<spire_core::run::GameAction> {
    use spire_core::run::{GameAction, RoomChoiceStatus};

    let room = decision.room.as_ref()?;
    let choice = room.choices.iter().find(|choice| choice.status == RoomChoiceStatus::Offered && wanted(&choice.kind))?;
    let action = GameAction::Choose(choice.id);
    decision.candidates.iter().any(|candidate| candidate.id == choice.id && candidate.action == action).then_some(action)
}

fn emerald_route_action(
    decision: &spire_core::run::PublicDecision,
    map: &spire_core::run::map::PublicMap,
) -> Option<spire_core::run::GameAction> {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};
    use spire_core::run::{GameAction, MapNodeKind, NodeId};

    if !(1..=3).contains(&decision.act) || decision.act != map.act {
        return None;
    }
    let nodes = map.nodes.iter().map(|node| (node.id, node)).collect::<BTreeMap<_, _>>();
    let mut best: Option<(usize, NodeId)> = None;
    for candidate in &decision.candidates {
        let GameAction::Move(start) = &candidate.action else { continue };
        let start = *start;
        let Some(start_node) = nodes.get(&start) else { continue };
        if start_node.visited {
            continue;
        }
        let mut queue = VecDeque::from([(start, 0usize)]);
        let mut seen = BTreeSet::from([start]);
        while let Some((at, distance)) = queue.pop_front() {
            let Some(node) = nodes.get(&at) else { continue };
            if node.kind == MapNodeKind::Elite
                && !node.visited
                && node.burning == Some(true)
                && reaches_boss(at, &nodes)
            {
                let route = (distance, start);
                if best.map_or(true, |current| route < current) {
                    best = Some(route);
                }
                break;
            }
            for next in &node.edges {
                if nodes.contains_key(next) && seen.insert(*next) {
                    queue.push_back((*next, distance + 1));
                }
            }
        }
    }
    best.map(|(_, node)| GameAction::Move(node))
}

fn reaches_boss(
    start: spire_core::run::NodeId,
    nodes: &std::collections::BTreeMap<spire_core::run::NodeId, &spire_core::run::map::PublicMapNode>,
) -> bool {
    use std::collections::{BTreeSet, VecDeque};
    use spire_core::run::MapNodeKind;

    let mut queue = VecDeque::from([start]);
    let mut seen = BTreeSet::from([start]);
    while let Some(at) = queue.pop_front() {
        let Some(node) = nodes.get(&at) else { continue };
        for next in &node.edges {
            let Some(next_node) = nodes.get(next) else { continue };
            if next_node.kind == MapNodeKind::Boss {
                return true;
            }
            if seen.insert(*next) {
                queue.push_back(*next);
            }
        }
    }
    false
}
