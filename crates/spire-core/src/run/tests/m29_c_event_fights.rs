//! M29c: event fights of the landed species are accepted at A1–A5 through the event executor's
//! up-front gate (Dead Adventurer's three draws, the Colosseum's two fights, Mind Bloom's three Act 1
//! bosses), enter at the Run's ascension and settle.
use super::super::*;
use super::m28_event_core::{at_event, before_event, row, view};
use crate::content::{Character, EncounterId, EnemyId, EventId};
use crate::run::{GameAction, RoomChoiceKind, RoomViewKind};
use rooms::standard::event_exec::Outcome;
use rooms::standard::event_frame::{self, DONE};

fn enemies(run: &Run) -> Vec<(EnemyId, i32)> {
    run.state.combat.as_ref().expect("event combat is active").public_full().unwrap()
        .enemies.iter().map(|enemy| (enemy.enemy_id, enemy.max_hp)).collect()
}

fn assert_hp(run: &Run, ascension: u8, context: &str) {
    for (id, max_hp) in enemies(run) {
        let (lo, hi) = crate::combat::enemies::hp_range(id, ascension);
        assert!((lo..=hi).contains(&max_hp), "{context} A{ascension}: {id:?} HP {max_hp}");
    }
}

#[test]
fn m29_event_fights_landed_species_accepted_a1_to_a5() {
    use EncounterId::*;
    for (encounter, species) in [
        (ThreeSentries, vec![EnemyId::Sentry; 3]), (GremlinNob, vec![EnemyId::GremlinNob]), (LagavulinEvent, vec![EnemyId::Lagavulin]),
        (ColosseumSlavers, vec![EnemyId::SlaverBlue, EnemyId::SlaverRed]), (ColosseumNobs, vec![EnemyId::Taskmaster, EnemyId::GremlinNob]),
        (Guardian, vec![EnemyId::Guardian]), (Hexaghost, vec![EnemyId::Hexaghost]), (SlimeBoss, vec![EnemyId::SlimeBoss]),
    ] {
        for ascension in 1..=5u8 {
            let mut run = at_event(Character::Ironclad, 2, EventId::GoldenShrine, 40 + ascension);
            run.state.ascension = ascension;
            event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter, rewards: None, next_stage: DONE, elite: false }])
                .unwrap_or_else(|e| panic!("{encounter:?} A{ascension}: {e:?}"));
            assert_eq!(enemies(&run).into_iter().map(|(id, _)| id).collect::<Vec<_>>(), species, "{encounter:?}");
            assert_hp(&run, ascension, &format!("{encounter:?}"));
            super::m28_world::win(&mut run);
            assert!(run.state.combat.is_none() && run.state.fault.is_none(), "{encounter:?} A{ascension} settles");
        }
    }
}

#[test]
fn m29_colosseum_both_fights_a1_to_a5() {
    for ascension in 1..=5u8 {
        let character = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher][usize::from(ascension) % 4];
        let (mut run, _) = before_event(character, 2, EventId::Colosseum, 60 + ascension);
        let map = run.state.map.as_mut().unwrap();
        let from = map.nodes.iter().find(|n| n.y == 7 && !n.edges.is_empty()).unwrap().clone();
        let node = from.edges[0];
        map.nodes.iter_mut().find(|n| n.id == node).unwrap().kind = MapNodeKind::Event;
        map.current = Some(from.id);
        run.state.floor = from.floor;
        run.state.ascension = ascension;
        run.step(&GameAction::Move(node)).unwrap_or_else(|e| panic!("A{ascension}: {e:?}"));
        assert_eq!(view(&run).kind, RoomViewKind::Event(EventId::Colosseum));
        super::m28_events_act2::option_row(&mut run, 0, 0);
        assert_eq!(enemies(&run).into_iter().map(|(id, _)| id).collect::<Vec<_>>(), vec![EnemyId::SlaverBlue, EnemyId::SlaverRed]);
        assert_hp(&run, ascension, "Colosseum Slavers");
        super::m28_world::win(&mut run);
        assert_eq!(view(&run).kind, RoomViewKind::Event(EventId::Colosseum), "A{ascension}: reopens at stage 1");
        super::m28_events_act2::option_row(&mut run, 1, 1);
        assert_eq!(enemies(&run).into_iter().map(|(id, _)| id).collect::<Vec<_>>(), vec![EnemyId::Taskmaster, EnemyId::GremlinNob]);
        assert_hp(&run, ascension, "Colosseum Nobs");
        super::m28_world::win(&mut run);
        assert!(run.state.combat.is_none() && run.state.fault.is_none(), "A{ascension}");
    }
}

#[test]
fn m29_mind_bloom_act1_bosses_a1_to_a5() {
    // "Species, not room": I am War fights the drawn Act 1 boss in Act 3 at the Run's ascension.
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..30u8 {
        let ascension = 1 + seed % 5;
        let (mut run, node) = before_event(Character::Silent, 3, EventId::MindBloom, seed);
        run.state.ascension = ascension;
        run.step(&GameAction::Move(node)).unwrap_or_else(|e| panic!("seed {seed} A{ascension}: {e:?}"));
        let id = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 }).id;
        run.step(&GameAction::Choose(id)).unwrap_or_else(|e| panic!("seed {seed} A{ascension}: {e:?}"));
        let boss = enemies(&run)[0].0;
        assert!(matches!(boss, EnemyId::Guardian | EnemyId::Hexaghost | EnemyId::SlimeBoss), "{boss:?}");
        assert_hp(&run, ascension, "Mind Bloom");
        seen.insert(format!("{boss:?}"));
    }
    assert_eq!(seen.len(), 3, "all three Act 1 bosses drawn: {seen:?}");
}

#[test]
fn m29_dead_adventurer_fights_a1_to_a5() {
    // Dead Adventurer (floor 7+): the ambush draws (Three Sentries, Gremlin Nob, awake Lagavulin) pass the
    // up-front gate, and a Search that ambushes enters the drawn fight at the Run's ascension.
    let mut fought = std::collections::BTreeSet::new();
    for seed in 0..80u8 {
        let ascension = 1 + seed % 5;
        let (mut run, node) = before_event(Character::Defect, 1, EventId::DeadAdventurer, seed);
        run.state.floor = 6;
        run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == node).unwrap().floor = 7;
        run.state.ascension = ascension;
        run.step(&GameAction::Move(node)).unwrap_or_else(|e| panic!("seed {seed} A{ascension}: {e:?}"));
        assert_eq!(view(&run).kind, RoomViewKind::Event(EventId::DeadAdventurer), "seed {seed} A{ascension}");
        for stage in 0..3u8 {
            let id = row(&run, &RoomChoiceKind::EventOption { stage, option: 0 }).id;
            run.step(&GameAction::Choose(id)).unwrap_or_else(|e| panic!("seed {seed} A{ascension} stage {stage}: {e:?}"));
            if run.state.combat.is_some() {
                assert_hp(&run, ascension, "Dead Adventurer");
                fought.insert(format!("{:?}", enemies(&run)[0].0));
                break;
            }
            if view(&run).kind != RoomViewKind::Event(EventId::DeadAdventurer) { break; }
        }
    }
    assert_eq!(fought.len(), 3, "all three ambushes fought: {fought:?}");
}
