//! M34c late-game exactness (final review M4): an exact checkpoint taken at the first Act 3 boss of a supplied Standard
//! route is exact for the rest of the Run. A fork, and a restore into the source session, given the same actions see
//! the same observations, rewards, reward items, terminals and record, to the Run's terminal (on some routes in Act 4).
use super::standard_traversal::{context, current_node_is_boss, drive, short_creation, simple_action, standard_session, CHARACTERS};
use crate::{RunActionIndex, RunBoundary, RunObservation, RunSession};
use spire_core::run::{RewardItem, RunTerminal};

type Step = (RunObservation, i32, Vec<RewardItem>, Option<RunTerminal>);

fn replay_actions(session: &mut RunSession, mut boundary: RunBoundary, actions: &[usize]) -> Vec<Step> {
    actions.iter().map(|&index| {
        let result = session.step(boundary.token, RunActionIndex(index as i64)).unwrap();
        let step = (result.boundary.observation.clone(), result.reward, result.reward_events, result.boundary.terminal);
        boundary = result.boundary;
        step
    }).collect()
}

#[test]
fn exact_fork_and_restore_are_exact_from_the_act3_boss_to_the_end() {
    let mut ends = Vec::new();
    for (index, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0, 20] {
            let seed = 0xB0 + index as u8 + if ascension == 20 { 8 } else { 0 };
            let (mut session, _) = standard_session(character, ascension, seed, short_creation());
            let at_act3_boss = |obs: &RunObservation| context(obs).0 == 3 && current_node_is_boss(obs);
            drive(&mut session, 12_000, at_act3_boss, simple_action).unwrap_or_else(|f| panic!("{character:?} A{ascension}: {f:?}"));
            let start = session.decision().unwrap();
            assert!(at_act3_boss(&start.observation), "{character:?} A{ascension} did not reach the Act 3 boss");
            let checkpoint = session.exact_checkpoint().unwrap();
            // Live: the deliberately simple policy (no supplied wounds after the checkpoint) to a terminal.
            let (mut boundary, mut actions, mut live) = (start.clone(), Vec::new(), Vec::new());
            while boundary.terminal.is_none() && actions.len() < 1_500 {
                let action = simple_action(&boundary.observation);
                live.extend(replay_actions(&mut session, boundary, &[action]));
                actions.push(action);
                boundary = session.decision().unwrap();
            }
            assert!(boundary.terminal.is_some(), "{character:?} A{ascension}: no terminal within 1,500 steps");
            let live_record = session.record().unwrap().unwrap();
            let mut fork = RunSession::exact_fork(&checkpoint).unwrap();
            let fork_start = fork.decision().unwrap();
            assert_eq!(fork_start.observation, start.observation);
            assert_eq!(replay_actions(&mut fork, fork_start, &actions), live, "{character:?} A{ascension}: fork diverged");
            assert_eq!(fork.record().unwrap().unwrap(), live_record);
            let restored = session.exact_restore(&checkpoint).unwrap();
            assert_eq!(restored.observation, start.observation);
            assert_eq!(replay_actions(&mut session, restored, &actions), live, "{character:?} A{ascension}: restore diverged");
            assert_eq!(session.record().unwrap().unwrap(), live_record);
            ends.push((character, ascension, context(&boundary.observation).0, boundary.terminal));
        }
    }
    // The exact spans cover the rest of Act 3 to a terminal and, on some routes, Act 4.
    assert!(ends.iter().any(|end| end.2 == 4), "no route crossed into Act 4: {ends:?}");
    eprintln!("standard_exact_late ends (character, ascension, act, terminal): {ends:?}");
}
