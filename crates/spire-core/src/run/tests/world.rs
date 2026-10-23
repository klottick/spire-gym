use crate::{combat::rng::Partition, content::Character, run::*};

fn run(seed: u8) -> Run {
    Run::new(RunConfig { character: Character::Ironclad, ascension: 0 }, CreationProfile::experienced(),
        RunRng::new([seed; 32], Partition::Test, 0)).unwrap()
}

#[test]
fn seeded_valid_map() {
    let mut a = RunRng::new([7; 32], Partition::Test, 2);
    let mut b = RunRng::new([7; 32], Partition::Test, 2);
    let eligibility = map::MapEligibility::new(true, false);
    let left = map::generate(1, &mut a, eligibility).unwrap();
    let right = map::generate(1, &mut b, eligibility).unwrap();
    assert_eq!(left, right);
    assert_eq!(left.nodes.iter().filter(|n| n.y == 0).count(), 6);
    assert_eq!(left.nodes.iter().filter(|n| n.kind == MapNodeKind::Boss).count(), 1);
    assert!(left.nodes.iter().any(|n| n.kind == MapNodeKind::Elite));
    assert_eq!(left.nodes.iter().filter(|n| n.kind == MapNodeKind::Treasure).count(),
        left.nodes.iter().filter(|n| n.y == 8).count());
    assert!(left.nodes.iter().filter(|n| n.kind == MapNodeKind::Shop).all(|n| (1..=13).contains(&n.y)));
    assert!(left.nodes.iter().filter(|n| n.kind == MapNodeKind::Rest).all(|n| n.y == 14 || (5..=12).contains(&n.y)));
    assert!(left.nodes.iter().filter(|n| n.kind == MapNodeKind::Elite).all(|n| (5..=13).contains(&n.y)));
    for node in left.nodes.iter().filter(|n| n.kind == MapNodeKind::Elite) {
        assert!(can_reach_boss(&left, node.id));
    }
    for node in &left.nodes {
        for to in &node.edges {
            let child = left.nodes.iter().find(|n| n.id == *to).unwrap();
            assert_eq!(child.y, node.y + 1);
            if child.y <= 14 { assert!((child.x as i16 - node.x as i16).abs() <= 1); }
        }
    }
    let starts = left.nodes.iter().filter(|n| n.y == 0).map(|n| n.id).collect::<Vec<_>>();
    let mut reached = std::collections::HashSet::new();
    let mut pending = starts;
    while let Some(id) = pending.pop() {
        if !reached.insert(id) { continue; }
        pending.extend(left.nodes.iter().find(|n| n.id == id).unwrap().edges.iter().copied());
    }
    assert_eq!(reached.len(), left.nodes.len());
}

#[test]
fn maps_are_scoped_by_act_and_invalid_act_consumes_no_entropy() {
    let mut rng = RunRng::new([31; 32], Partition::Test, 0);
    for act in 1..=3 {
        let map = map::generate(act, &mut rng, map::MapEligibility::new(true, false)).unwrap();
        assert_eq!(map.act, act);
        assert!(map.nodes.iter().any(|node| node.kind == MapNodeKind::Elite));
        assert!(map.nodes.iter().any(|node| node.kind == MapNodeKind::Boss));
    }
    let before = rng.clone();
    assert!(map::generate(4, &mut rng, map::MapEligibility::new(true, false)).is_err());
    assert_eq!(rng, before);
}

#[test]
fn public_map_redacts_unrevealed_question_encounter_and_history() {
    let mut left = run(29);
    let mut right = run(29);
    let mut left_rng = RunRng::new([29; 32], Partition::Test, 0);
    let mut right_rng = RunRng::new([29; 32], Partition::Test, 0);
    left.state_mut_for_test().map = Some(map::generate(1, &mut left_rng, map::MapEligibility::new(true, false)).unwrap());
    right.state_mut_for_test().map = Some(map::generate(1, &mut right_rng, map::MapEligibility::new(true, false)).unwrap());
    let question = right.state().map.as_ref().unwrap().nodes.iter().find(|n| n.kind == MapNodeKind::Question).unwrap().id;
    let private_map = right.state_mut_for_test().map.as_mut().unwrap();
    let node = private_map.nodes.iter_mut().find(|n| n.id == question).unwrap();
    node.hidden_kind = Some(MapNodeKind::Event);
    node.encounter = Some(crate::content::EncounterId::Cultist);
    private_map.encounter_history.push(crate::content::EncounterId::Cultist);
    private_map.regular_encounter_history.push(crate::content::EncounterId::Cultist);
    private_map.question_roll_ordinal = 17;
    private_map.question_ramps = [0; 3];

    let left_view = left.public_map().unwrap();
    let right_view = right.public_map().unwrap();
    assert_eq!(left_view, right_view);
    assert_eq!(right_view.nodes.iter().find(|n| n.id == question).unwrap().kind, MapNodeKind::Question);
    assert_eq!(right_view.nodes.iter().find(|n| n.id == question).unwrap().resolved_room, None);
    assert!(right_view.nodes.iter().all(|n| n.edges.iter().all(|id| right_view.nodes.iter().any(|child| child.id == *id))));
}

#[test]
fn map_debug_ignores_private_state_but_tracks_revealed_room_facts() {
    let mut rng = RunRng::new([43; 32], Partition::Test, 0);
    let mut baseline = map::generate(1, &mut rng, map::MapEligibility::new(true, false)).unwrap();
    let mut private_variant = baseline.clone();
    let question_id = private_variant.nodes.iter().find(|node| node.kind == MapNodeKind::Question).unwrap().id;
    let private_question = private_variant.nodes.iter_mut().find(|node| node.id == question_id).unwrap();
    private_question.hidden_kind = Some(MapNodeKind::Event);
    private_question.encounter = Some(crate::content::EncounterId::Cultist);
    private_variant.encounter_history.push(crate::content::EncounterId::Cultist);
    private_variant.regular_encounter_history.push(crate::content::EncounterId::Cultist);
    private_variant.draft_ordinal = private_variant.draft_ordinal.saturating_add(1);
    private_variant.question_ramps = [1, 2, 3];
    private_variant.question_roll_ordinal = private_variant.question_roll_ordinal.saturating_add(1);

    assert_eq!(format!("{baseline:?}"), format!("{private_variant:?}"));

    let baseline_question = baseline.nodes.iter_mut().find(|node| node.id == question_id).unwrap();
    baseline_question.visited = true;
    baseline_question.revealed = true;
    baseline_question.hidden_kind = Some(MapNodeKind::Event);
    assert_ne!(format!("{baseline:?}"), format!("{private_variant:?}"));
}

#[test]
fn public_map_reveals_question_result_at_room_entry() {
    let mut game = run(30);
    let mut rng = RunRng::new([30; 32], Partition::Test, 0);
    let mut raw = map::generate(1, &mut rng, map::MapEligibility::new(true, false)).unwrap();
    let id = raw.nodes.iter().find(|n| n.kind == MapNodeKind::Question).unwrap().id;
    let parent_id = raw.nodes.iter().find(|n| n.edges.contains(&id)).unwrap().id;
    let parent_floor = raw.nodes.iter().find(|n| n.id == parent_id).unwrap().floor;
    raw.current = Some(parent_id);
    raw.nodes.iter_mut().find(|n| n.id == parent_id).unwrap().visited = true;
    game.state_mut_for_test().floor = parent_floor;
    game.state_mut_for_test().map = Some(raw);
    let before = game.public_map().unwrap();
    let before_node = before.nodes.iter().find(|node| node.id == id).unwrap();
    assert_eq!(before_node.kind, MapNodeKind::Question);
    assert_eq!(before_node.resolved_room, None);
    assert!(matches!(world::enter(game.state_mut_for_test(), id), Err(RunError::Faulted(_))));
    let resolved = game.state().map.as_ref().unwrap().nodes.iter().find(|node| node.id == id).unwrap().hidden_kind.unwrap();
    let after = game.public_map().unwrap();
    let after_node = after.nodes.iter().find(|node| node.id == id).unwrap();
    assert_eq!(after_node.kind, MapNodeKind::Question);
    assert_eq!(after_node.resolved_room, Some(resolved));
}

#[test]
fn burning_marker_requires_act4_and_unowned_emerald_without_ineligible_draw() {
    let seed = [31; 32];
    let mut eligible_rng = RunRng::new(seed, Partition::Test, 3);
    let eligible = map::generate(1, &mut eligible_rng, map::MapEligibility::new(true, false)).unwrap();
    let markers = eligible.nodes.iter().filter(|node| node.burning).collect::<Vec<_>>();
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].kind, MapNodeKind::Elite);

    for (act4_enabled, emerald_owned) in [(false, false), (true, true)] {
        let mut rng = RunRng::new(seed, Partition::Test, 3);
        let map = map::generate(1, &mut rng, map::MapEligibility::new(act4_enabled, emerald_owned)).unwrap();
        assert!(map.nodes.iter().all(|node| !node.burning));
        let marker_context = [1, map.draft_ordinal, map.nodes.iter().filter(|node| node.kind == MapNodeKind::Elite).count() as u64];
        let actual = rng.draw_scoped(RunRngScope::MapBurningElite, &marker_context);
        let mut reference = RunRng::new(seed, Partition::Test, 3);
        let expected = reference.draw_scoped(RunRngScope::MapBurningElite, &marker_context);
        assert_eq!(actual, expected, "ineligible maps must not consume marker entropy");
        let mut normalized = map.clone();
        let mut eligible_normalized = eligible.clone();
        for node in &mut normalized.nodes { node.burning = false; }
        for node in &mut eligible_normalized.nodes { node.burning = false; }
        assert_eq!(normalized, eligible_normalized, "eligibility changes must not perturb other map scopes");
    }
}

#[test]
fn map_eligibility_reads_persistent_act4_and_emerald_state() {
    let mut game = run(32);
    assert_eq!(map::MapEligibility::from_run(game.state()), map::MapEligibility::new(true, false));
    let state = game.state_mut_for_test();
    state.act4_enabled = false;
    state.keys[2] = true;
    assert_eq!(map::MapEligibility::from_run(game.state()), map::MapEligibility::new(false, true));
}

#[test]
fn illegal_edge_does_not_advance() {
    let mut state = run(1).state().clone();
    state.map = Some(map::generate(1, &mut state.rng, map::MapEligibility::new(true, false)).unwrap());
    let unreachable = state.map.as_ref().unwrap().nodes.iter().find(|n| n.y == 1).unwrap().id;
    let before = (state.floor, state.history.clone(), state.rng.clone(), state.map.clone());
    assert_eq!(world::enter(&mut state, unreachable), Err(RunError::InvalidAction));
    assert_eq!((state.floor, state.history, state.rng, state.map), before);
}

#[test]
fn hidden_encounter_stays_hidden() {
    let mut rng = RunRng::new([4; 32], Partition::Test, 0);
    let mut map = map::generate(1, &mut rng, map::MapEligibility::new(true, false)).unwrap();
    let question = map.nodes.iter().find(|n| n.kind == MapNodeKind::Question).unwrap();
    assert!(question.encounter.is_none());
    assert_eq!(question.kind, MapNodeKind::Question);
    assert_eq!(question.hidden_kind, None);
    assert!(!question.revealed);
    let mut state = run(4).state().clone();
    let question_id = question.id;
    let parent_id = map.nodes.iter().find(|n| n.edges.contains(&question_id)).unwrap().id;
    let parent_floor = map.nodes.iter().find(|n| n.id == parent_id).unwrap().floor;
    map.current = Some(parent_id);
    map.nodes.iter_mut().find(|n| n.id == parent_id).unwrap().visited = true;
    state.floor = parent_floor;
    state.map = Some(map);
    let err = world::enter(&mut state, question_id).unwrap_err();
    assert!(matches!(err, RunError::Faulted(_)));
    let entered = state.map.as_ref().unwrap().nodes.iter().find(|n| n.id == question_id).unwrap();
    assert_eq!(entered.kind, MapNodeKind::Question);
    assert!(matches!(entered.hidden_kind, Some(MapNodeKind::Monster | MapNodeKind::Shop | MapNodeKind::Treasure | MapNodeKind::Event)));
    assert!(entered.revealed);
}

fn movement_fixture(seed: u8, winged: u8) -> (RunState, NodeId, NodeId) {
    let mut state = run(seed).state().clone();
    let mut map = map::generate(1, &mut state.rng, map::MapEligibility::new(true, false)).unwrap();
    let (start, target) = map.nodes.iter().filter(|n| n.y == 0).find_map(|start| {
        map.nodes.iter().find(|n| n.y == 1 && !start.edges.contains(&n.id)).map(|target| (start.id, target.id))
    }).expect("a six-route map has a next-row node not connected to every start");
    map.act = 1;
    map.current = Some(start);
    map.winged_moves = winged;
    map.nodes.iter_mut().find(|n| n.id == start).unwrap().visited = true;
    state.map = Some(map);
    (state, start, target)
}

#[test]
fn winged_move_spends_one_charge_on_unconnected_next_row() {
    let (mut state, _, target) = movement_fixture(5, 1);
    let result = world::enter(&mut state, target);
    assert!(matches!(result, Err(RunError::Faulted(_))));
    let map = state.map.as_ref().unwrap();
    assert_eq!(map.current, Some(target));
    assert_eq!(map.winged_moves, 0);
}

#[test]
fn winged_move_rejects_row_skip_without_mutation() {
    let (mut state, start, _) = movement_fixture(6, 1);
    let target = state.map.as_ref().unwrap().nodes.iter().find(|n| n.y == 2).unwrap().id;
    let before = (state.floor, state.history.clone(), state.rng.clone(), state.map.clone());
    assert_eq!(world::enter(&mut state, target), Err(RunError::InvalidAction));
    assert_eq!(state.map.as_ref().unwrap().current, Some(start));
    assert_eq!((state.floor, state.history, state.rng, state.map), before);
}

#[test]
fn normal_edge_does_not_spend_winged_charge() {
    let (mut state, start, _) = movement_fixture(7, 1);
    let target = state.map.as_ref().unwrap().nodes.iter().find(|n| n.id == start).unwrap().edges[0];
    let result = world::enter(&mut state, target);
    assert!(matches!(result, Err(RunError::Faulted(_))));
    assert_eq!(state.map.as_ref().unwrap().winged_moves, 1);
}

#[test]
fn exhausted_winged_moves_reject_unconnected_next_row_without_mutation() {
    let (mut state, _, target) = movement_fixture(8, 0);
    let before = (state.floor, state.history.clone(), state.rng.clone(), state.map.clone());
    assert_eq!(world::enter(&mut state, target), Err(RunError::InvalidAction));
    assert_eq!((state.floor, state.history, state.rng, state.map), before);
}

fn can_reach_boss(map: &ActMap, from: NodeId) -> bool {
    let mut pending = vec![from];
    let mut seen = std::collections::HashSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) { continue; }
        let Some(node) = map.nodes.iter().find(|node| node.id == id) else { continue };
        if node.kind == MapNodeKind::Boss { return true; }
        pending.extend(node.edges.iter().copied());
    }
    false
}

#[test]
fn floor_advances_once() {
    let mut state = run(2).state().clone();
    state.map = Some(map::generate(1, &mut state.rng, map::MapEligibility::new(true, false)).unwrap());
    let start = state.map.as_ref().unwrap().nodes.iter().find(|n| n.y == 0).unwrap().id;
    let err = world::enter(&mut state, start).unwrap_err();
    assert!(matches!(err, RunError::Faulted(_)));
    let events = state.history.events.iter().filter(|e| matches!(e, CommittedEvent::FloorEntered { .. })).count();
    assert_eq!(events, 1);
    assert_eq!(state.history.rooms.len(), 1);
    let after = state.history.clone();
    assert!(world::enter(&mut state, start).is_err());
    assert_eq!(state.history, after);
}
