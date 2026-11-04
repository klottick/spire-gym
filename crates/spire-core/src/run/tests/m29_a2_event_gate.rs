//! M29a2: an event fight above its encounter's playable ceiling was rejected up front with the
//! M32 reason, leaving the whole Run unchanged. Witness: the Colosseum, whose first fight (Colosseum
//! Slavers: Slaver Red) had ceiling 16 (inventory §3a). M32 removes every ceiling (plan item 3).
use super::super::*;
use crate::content::Character;

/// Formerly `m29_event_fight_above_ceiling_names_m32` (M32 plan item 3, design §6.2): no ceiling
/// remains, so the Colosseum event enters at A16 and A17 alike.
#[test]
fn m29_event_fight_admits_a17() {
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
    // M29c: the Colosseum's species landed, so A16 enters the event. M32: A17 enters too.
    for ascension in [16, 17] {
        let (mut run, target) = row8(Character::Silent, 9);
        run.state.event_pools = Some(rooms::standard::event_pool::EventPools {
            ordinary: vec![crate::content::EventId::Colosseum], shrines: vec![], one_time: vec![], seen: vec![] });
        run.state.ascension = ascension;
        assert!(run.step(&GameAction::Move(target)).is_ok(), "A{ascension}");
    }
}
