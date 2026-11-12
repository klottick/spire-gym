"""Short fixed Python speed check for build records (M38b, spire-qu9.3; no thresholds).

Usage: python examples/speed_check.py [--steps 300] [--observation dense|compact] [--out PATH]

Times ``RunVectorEnv.step`` over 64 Standard A20 slots (16 per character), NEXT_STEP
autoreset and ``record_replay`` off, at 1 and 16 workers, twice each. Actions are a
seeded uniform legal choice made outside the timer, so the trajectories are fixed.
Prints ``spire-gym-speed-check-python-v1`` JSON with ``build_info()``, every run, and the
best slot steps/s per worker count. It records numbers and never fails on speed; compare
like with like (same host and build). The native counterpart is
``a20_checks --speed-check``.
"""

from __future__ import annotations

import argparse
import json
import time

import numpy as np

from spire_lab import _native
from spire_lab.gymnasium import RunVectorEnv


CHARACTERS = ("ironclad", "silent", "defect", "watcher")
SLOTS = 64
SEED = 7


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    result.add_argument("--steps", type=int, default=300, help="vector step calls per run")
    result.add_argument("--observation", choices=("dense", "compact"), default="dense")
    result.add_argument("--out", help="write the JSON here as well as to stdout")
    return result


def run(workers: int, steps: int, repeat: int, observation: str) -> dict:
    configs = [{"character": CHARACTERS[slot // (SLOTS // 4)], "ascension": 20} for slot in range(SLOTS)]
    env = RunVectorEnv(configs, profile="standard", workers=workers, autoreset_mode="next_step", observation=observation)
    rng = np.random.default_rng(SEED)
    timed = 0.0
    try:
        observation, _ = env.reset(seed=SEED)
        for _ in range(steps):
            legal = observation["action_mask"].reshape(SLOTS, -1) != 0
            actions = np.where(legal, rng.random(legal.shape), -1.0).argmax(axis=1).astype(np.int64)
            start = time.perf_counter()
            observation, _, _, _, _ = env.step(actions)
            timed += time.perf_counter() - start
    finally:
        env.close()
    return {"workers": workers, "repeat": repeat, "calls": steps, "slot_steps": steps * SLOTS,
            "step_seconds": timed, "ms_per_call": 1e3 * timed / steps,
            "slot_steps_per_second": steps * SLOTS / timed if timed else 0.0}


def main() -> None:
    args = parser().parse_args()
    if args.steps <= 0:
        raise SystemExit("--steps must be positive")
    runs = [run(workers, args.steps, repeat, args.observation) for workers in (1, 16) for repeat in range(2)]
    best = {f"workers_{w}": max(r["slot_steps_per_second"] for r in runs if r["workers"] == w) for w in (1, 16)}
    result = {"schema_id": "spire-gym-speed-check-python-v1", "native_file": _native.__file__,
              "build_info": _native.build_info(), "observation": args.observation, "slots": SLOTS, "ascension": 20, "record_replay": False,
              "autoreset": "next_step", "seed": SEED, "runs": runs, "best": best}
    text = json.dumps(result, indent=2, sort_keys=True)
    print(text)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as handle:
            handle.write(text + "\n")


if __name__ == "__main__":
    main()
