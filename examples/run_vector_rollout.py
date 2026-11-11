"""Roll out mixed-character Standard Runs through RunVectorEnv.

Usage: python examples/run_vector_rollout.py [--slots 4] [--ascension 20]
       [--policy random|first_legal] [--steps 100] [--workers 1] [--seed 7]

The caller policies are legal-action baselines, not strategy or win claims.
"""

from __future__ import annotations

import argparse
import time

import numpy as np

from spire_lab.gymnasium import RunVectorEnv


CHARACTERS = ("ironclad", "silent", "defect", "watcher")


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--slots", type=int, default=4, help="number of slots; must be a multiple of four")
    result.add_argument("--ascension", type=int, default=20)
    result.add_argument("--policy", choices=("random", "first_legal"), default="random")
    result.add_argument("--steps", type=int, default=100, help="number of vector step calls")
    result.add_argument("--workers", type=int, default=1)
    result.add_argument("--seed", type=int, default=7)
    return result


def info_at(info, key: str, slot: int, default=None):
    values = info.get(key)
    present = info.get("_" + key)
    if values is None or (present is not None and not bool(present[slot])):
        return default
    return values[slot]


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    if not 1 <= args.slots <= 256 or args.slots % 4:
        parser().error("--slots must be a positive multiple of four no greater than 256")
    if not 0 <= args.ascension <= 20:
        parser().error("--ascension must be in 0..20")
    if args.steps < 0:
        parser().error("--steps must be nonnegative")
    if args.workers < 1:
        parser().error("--workers must be positive")

    characters = [CHARACTERS[slot % len(CHARACTERS)] for slot in range(args.slots)]
    configs = [{"character": character, "ascension": args.ascension} for character in characters]
    env = RunVectorEnv(configs, profile="standard", workers=args.workers, autoreset_mode="next_step")
    rng = np.random.default_rng(args.seed)
    completed = np.zeros(args.slots, dtype=np.int64)
    lengths = np.zeros(args.slots, dtype=np.int64)
    returns = np.zeros(args.slots, dtype=np.float64)
    waiting_for_reset = np.zeros(args.slots, dtype=np.bool_)
    last_reason = ["-"] * args.slots
    last_return = [None] * args.slots
    last_length = [None] * args.slots
    slot_steps = 0
    try:
        observation, _ = env.reset(seed=args.seed)
        started = time.perf_counter()
        for _ in range(args.steps):
            actions = np.zeros(args.slots, dtype=np.int64)
            active = ~waiting_for_reset
            for slot in range(args.slots):
                if not active[slot]:
                    continue
                legal = np.flatnonzero(observation["action_mask"][slot].reshape(-1))
                if legal.size == 0:
                    raise RuntimeError(f"slot {slot} has no legal actions outside NEXT_STEP autoreset")
                actions[slot] = int(legal[0]) if args.policy == "first_legal" else int(rng.choice(legal))
            slot_steps += int(active.sum())
            observation, rewards, terminated, truncated, info = env.step(actions)
            if truncated.any():
                raise RuntimeError("Standard Run vector unexpectedly truncated")

            for slot in range(args.slots):
                if waiting_for_reset[slot]:
                    lengths[slot] = 0
                    returns[slot] = 0
                    waiting_for_reset[slot] = False
                    continue
                lengths[slot] += 1
                returns[slot] += float(rewards[slot])
                if terminated[slot]:
                    completed[slot] += 1
                    last_reason[slot] = str(info_at(info, "terminal_reason", slot, "terminal"))
                    reported_return = info_at(info, "episode_return", slot)
                    last_return[slot] = float(reported_return) if reported_return is not None else float(returns[slot])
                    last_length[slot] = int(lengths[slot])
                    waiting_for_reset[slot] = True
        elapsed = time.perf_counter() - started
        rate = slot_steps / elapsed if elapsed > 0 else 0.0
        print(f"episodes={int(completed.sum())} steps={args.steps} slot_steps_per_s={rate:.2f}")
        print("slot\tcharacter\tcompleted_episodes\tlast_terminal_reason\tlast_return\tlast_length")
        for slot, character in enumerate(characters):
            ret = "-" if last_return[slot] is None else f"{last_return[slot]:g}"
            length = "-" if last_length[slot] is None else str(last_length[slot])
            print(f"{slot}\t{character}\t{completed[slot]}\t{last_reason[slot]}\t{ret}\t{length}")
    finally:
        env.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
