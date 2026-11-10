"""M34 Standard tool demo, not a learner or strategy.

A seeded random legal rollout, an exact fork (a privileged oracle tool, reached
through ``env.unwrapped``) that reproduces the same future, and replay to a
chosen recorded decision.
"""

import argparse
import sys

import numpy as np

from spire_lab.gymnasium import RunEnv


def rollout(game, observation, info, rng, limit):
    """Random legal actions from (observation, info) until a terminal or ``limit`` steps."""
    rewards = []
    terminated = info.get("terminal_reason") is not None
    while not terminated and len(rewards) < limit:
        legal = np.flatnonzero(observation["action_mask"])
        observation, reward, terminated, _, info = game.step(int(rng.choice(legal)))
        rewards.append(reward)
    return observation, rewards, info


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--character", default="ironclad")
    parser.add_argument("--ascension", type=int, default=0)
    parser.add_argument("--seed", type=int, default=7)
    args = parser.parse_args(argv)
    game = RunEnv(
        args.character,
        args.ascension,
        profile="standard",
        partition="test",
        record_replay=True,
    )
    observation, info = game.reset(seed=args.seed)
    print("creation:", info["creation"])

    at_checkpoint, prefix, prefix_info = rollout(
        game, observation, info, np.random.default_rng(args.seed), 30
    )
    checkpoint = game.unwrapped.exact_checkpoint()

    live_end, live, live_info = rollout(
        game, at_checkpoint, prefix_info, np.random.default_rng(args.seed + 1), 3000
    )
    fork = game.unwrapped.exact_fork(checkpoint)
    fork_start, fork_info = fork.decision()
    fork_end, forked, _ = rollout(
        fork, fork_start, fork_info, np.random.default_rng(args.seed + 1), 3000
    )
    assert live == forked, "an exact fork reproduces the same future"
    assert all(np.array_equal(live_end[k], fork_end[k]) for k in live_end)

    resumed = RunEnv.from_record(game.record(), len(prefix))
    current, _ = resumed.decision()
    assert all(np.array_equal(current[k], at_checkpoint[k]) for k in current)

    print(
        f"prefix {len(prefix)} steps, then {len(live)} steps to "
        f"{live_info.get('terminal_reason')}; episode return {live_info.get('episode_return')}"
    )
    print(f"exact fork reproduced the same {len(forked)} rewards; from_record({len(prefix)}) matched")
    print("(random legal actions only: no strategy and no win claim)")
    for env in (game, fork, resumed):
        env.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
