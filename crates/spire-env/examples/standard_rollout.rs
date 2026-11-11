//! M34 Standard tool demo, not a strategy: a seeded random or first-legal rollout, an exact fork (privileged oracle
//! tool) that reproduces the same future, and replay to a chosen recorded decision. Usage:
//! `cargo run -p spire-env --example standard_rollout -- [character] [ascension] [seed] [--policy random|first_legal]`.
use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{RunActionIndex, RunBoundary, RunCapacities, RunSession, RunSessionError, StandardRunConfig};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Policy { Random, FirstLegal }

/// A small seeded generator for the demo's random action choice (never the game's RNG).
fn choose(state: &mut u64, boundary: &RunBoundary, policy: Policy) -> usize {
    let first = boundary.observation.action_mask.iter().position(|v| *v != 0)
        .expect("a live boundary has a legal action");
    if policy == Policy::FirstLegal { return first; }
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let legal = boundary.observation.action_mask.iter().enumerate().filter_map(|(i, v)| (*v != 0).then_some(i)).collect::<Vec<_>>();
    legal[(*state >> 33) as usize % legal.len()]
}

fn rollout(session: &mut RunSession, mut boundary: RunBoundary, mut state: u64, limit: usize, policy: Policy) -> Result<(RunBoundary, Vec<i32>), RunSessionError> {
    let mut rewards = Vec::new();
    while boundary.terminal.is_none() && rewards.len() < limit {
        let index = choose(&mut state, &boundary, policy);
        let result = session.step(boundary.token, RunActionIndex(index as i64))?;
        rewards.push(result.reward);
        boundary = result.boundary;
    }
    Ok((boundary, rewards))
}

fn main() -> Result<(), RunSessionError> {
    let mut positional = Vec::new();
    let mut policy = Policy::Random;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--policy" {
            policy = match args.next().as_deref() {
                Some("random") => Policy::Random,
                Some("first_legal") => Policy::FirstLegal,
                _ => return Err(RunSessionError::Input("--policy must be random or first_legal")),
            };
        } else if arg.starts_with("--") {
            return Err(RunSessionError::Input("usage: standard_rollout [character] [ascension] [seed] [--policy random|first_legal]"));
        } else {
            positional.push(arg);
        }
    }
    let character = match positional.first().map(String::as_str).unwrap_or("ironclad") {
        "ironclad" => Character::Ironclad, "silent" => Character::Silent, "defect" => Character::Defect, "watcher" => Character::Watcher,
        _ => return Err(RunSessionError::Input("character must be ironclad, silent, defect or watcher")),
    };
    let ascension = positional.get(1).map_or(Ok(0), |a| a.parse()).map_err(|_| RunSessionError::Input("ascension must be 0..=20"))?;
    let seed = positional.get(2).map_or(Ok(7), |a| a.parse()).map_err(|_| RunSessionError::Input("seed must be a u64"))?;
    let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
    config.record_replay = true;
    let mut session = RunSession::new_standard(config)?;
    let start = session.reset(Some(seed))?;

    // 1. Apply the caller policy for a prefix, then take a privileged exact checkpoint.
    let (at_checkpoint, prefix) = rollout(&mut session, start, seed, 30, policy)?;
    let checkpoint = session.exact_checkpoint()?;

    // 2. The live session and an exact fork given the same actions see the same (hidden) future.
    let (live_end, live_rewards) = rollout(&mut session, at_checkpoint.clone(), seed ^ 0x5eed, 3_000, policy)?;
    let mut fork = RunSession::exact_fork(&checkpoint)?;
    let fork_start = fork.decision()?;
    let (fork_end, fork_rewards) = rollout(&mut fork, fork_start, seed ^ 0x5eed, 3_000, policy)?;
    assert_eq!((&live_rewards, &live_end.observation), (&fork_rewards, &fork_end.observation), "an exact fork reproduces the same future");

    // 3. Replay to a chosen recorded decision (the checkpoint's decision) from the portable record.
    let record = session.record()?.expect("record_replay is enabled");
    let resumed = RunSession::from_record(&record, prefix.len(), RunCapacities::standard_default())?;
    assert_eq!(resumed.decision()?.observation, at_checkpoint.observation, "replay to a decision equals the live boundary");

    println!("character={character:?} ascension={ascension} seed={seed} policy={}", if policy == Policy::Random { "random" } else { "first_legal" });
    println!("prefix steps={} then {} steps to {:?}; episode return={}", prefix.len(), live_rewards.len(), live_end.terminal,
        prefix.iter().chain(&live_rewards).sum::<i32>());
    println!("exact fork reproduced the same {} rewards; from_record({}) matched the checkpoint decision", fork_rewards.len(), prefix.len());
    println!("(random legal actions only: no strategy and no win claim)");
    Ok(())
}
