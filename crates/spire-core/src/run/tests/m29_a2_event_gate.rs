//! M29a2: an event fight above its encounter's playable ceiling is rejected up front with the
//! M32 reason (not the M29 interim), leaving the whole Run unchanged. Witness: the Colosseum,
//! whose first fight (Colosseum Slavers: Slaver Red) has ceiling 16 (inventory §3a).
use super::super::*;
use crate::content::Character;

#[test]
fn m29_event_fight_above_ceiling_names_m32() {
    fn row8(character: Character, seed: u8) -> (Run, crate::run::NodeId) {
        let (mut run, _) = super::m28_event_core::before_event(character, 2, crate::content::EventId::Colosseum, seed);
        let map = run.state.map.as_mut().unwrap();
        let from = map.nodes.iter().find(|n| n.y == 7 && !n.edges.is_empty()).unwrap().clone();
        let node = from.edges[0];
        map.nodes.iter_mut().find(|n| n.id == node).unwrap().kind = MapNodeKind::Event;
        map.current = Some(from.id);
        run.state.floor = from.floor;
        (run, node)
    }
    for (ascension, reason) in [(16, "event combat enemy setup unavailable at this ascension"),
        (17, crate::combat::enemies::ascension::ABOVE_CEILING)] {
        let (mut run, target) = row8(Character::Silent, 9);
        run.state.event_pools = Some(rooms::standard::event_pool::EventPools {
            ordinary: vec![crate::content::EventId::Colosseum], shrines: vec![], one_time: vec![], seen: vec![] });
        run.state.ascension = ascension;
        let before = run.clone();
        assert_eq!(run.step(&GameAction::Move(target)), Err(RunError::Unsupported(reason)), "A{ascension}");
        assert!(run == before, "A{ascension}: whole Run unchanged");
    }
}
