//! M34 Standard tool demo, not a strategy: a seeded random legal rollout, an exact fork (privileged oracle tool) that
//! reproduces the same future, and replay to a chosen recorded decision. Usage:
//! `cargo run -p spire-env --example standard_rollout -- [character] [ascension] [seed]`.
use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{RunActionIndex, RunBoundary, RunCapacities, RunSession, RunSessionError, StandardRunConfig};

/// A small seeded generator for the demo's action choice (never the game's RNG).
fn choose(state: &mut u64, boundary: &RunBoundary) -> usize {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let legal = boundary.observation.action_mask.iter().enumerate().filter_map(|(i, v)| (*v != 0).then_some(i)).collect::<Vec<_>>();
    legal[(*state >> 33) as usize % legal.len()]
}

fn rollout(session: &mut RunSession, mut boundary: RunBoundary, mut state: u64, limit: usize) -> Result<(RunBoundary, Vec<i32>), RunSessionError> {
    let mut rewards = Vec::new();
    while boundary.terminal.is_none() && rewards.len() < limit {
        let index = choose(&mut state, &boundary);
        let result = session.step(boundary.token, RunActionIndex(index as i64))?;
        rewards.push(result.reward);
        boundary = result.boundary;
    }
    Ok((boundary, rewards))
}

fn main() -> Result<(), RunSessionError> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let character = match args.first().map(String::as_str).unwrap_or("ironclad") {
        "ironclad" => Character::Ironclad, "silent" => Character::Silent, "defect" => Character::Defect, "watcher" => Character::Watcher,
        _ => return Err(RunSessionError::Input("character must be ironclad, silent, defect or watcher")),
    };
    let ascension = args.get(1).map_or(Ok(0), |a| a.parse()).map_err(|_| RunSessionError::Input("ascension must be 0..=20"))?;
    let seed = args.get(2).map_or(Ok(7), |a| a.parse()).map_err(|_| RunSessionError::Input("seed must be a u64"))?;
    let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
    config.record_replay = true;
    let mut session = RunSession::new_standard(config)?;
    let start = session.reset(Some(seed))?;

    // 1. Random legal prefix, then a privileged exact checkpoint.
    let (at_checkpoint, prefix) = rollout(&mut session, start, seed, 30)?;
    let checkpoint = session.exact_checkpoint()?;

    // 2. The live session and an exact fork given the same actions see the same (hidden) future.
    let (live_end, live_rewards) = rollout(&mut session, at_checkpoint.clone(), seed ^ 0x5eed, 3_000)?;
    let mut fork = RunSession::exact_fork(&checkpoint)?;
    let fork_start = fork.decision()?;
    let (fork_end, fork_rewards) = rollout(&mut fork, fork_start, seed ^ 0x5eed, 3_000)?;
    assert_eq!((&live_rewards, &live_end.observation), (&fork_rewards, &fork_end.observation), "an exact fork reproduces the same future");

    // 3. Replay to a chosen recorded decision (the checkpoint's decision) from the portable record.
    let record = session.record()?.expect("record_replay is enabled");
    let resumed = RunSession::from_record(&record, prefix.len(), RunCapacities::standard_default())?;
    assert_eq!(resumed.decision()?.observation, at_checkpoint.observation, "replay to a decision equals the live boundary");

    println!("character={character:?} ascension={ascension} seed={seed}");
    println!("prefix steps={} then {} steps to {:?}; episode return={}", prefix.len(), live_rewards.len(), live_end.terminal,
        prefix.iter().chain(&live_rewards).sum::<i32>());
    println!("exact fork reproduced the same {} rewards; from_record({}) matched the checkpoint decision", fork_rewards.len(), prefix.len());
    println!("(random legal actions only: no strategy and no win claim)");
    Ok(())
}
