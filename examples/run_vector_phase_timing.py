"""Time the Python side of a Standard Run vector step (M37a diagnostic; no thresholds).

Usage: python examples/run_vector_phase_timing.py [--slots 64] [--ascension 20]
       [--workers 1] [--steps 2000] [--seed 7] [--loops example,facade,binding]
       [--drop-before-step] [--observation dense|compact] [--out PATH]

Three loops over the same mixed-character batch, each with its own batch:

- ``example``: what ``run_vector_rollout.py`` times: the per-slot Python action
  choice (``rng.choice`` over each legal mask) plus ``RunVectorEnv.step``.
- ``facade``: ``RunVectorEnv.step`` alone, with a vectorised legal-action choice
  outside the timer.
- ``binding``: ``_native.NativeRunBatch.step`` alone (native batch step plus the
  binding's packaging into NumPy arrays and info dicts), with no Gymnasium facade.
  Its ``step`` time includes releasing the previous call's arrays, which it also
  reports alone as ``free_previous_ms_per_call`` (M38a). With ``--drop-before-step``
  it releases them before the call instead of after it, so only one call's arrays
  are alive while the batch steps.

``facade - binding`` is the Gymnasium facade's per-call cost. ``binding`` minus the
native batch step from ``m37_phase_timing`` (same slots and workers, record off)
is the binding's packaging and transfer cost. Use the same host and build for
both. Prints JSON.
"""

from __future__ import annotations

import argparse
import json
import time

import numpy as np

from spire_lab import _native
from spire_lab.gymnasium import RunVectorEnv


CHARACTERS = ("ironclad", "silent", "defect", "watcher")


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    result.add_argument("--slots", type=int, default=64, help="number of slots; a multiple of four")
    result.add_argument("--ascension", type=int, default=20)
    result.add_argument("--workers", type=int, default=1)
    result.add_argument("--steps", type=int, default=2000, help="vector step calls per loop")
    result.add_argument("--seed", type=int, default=7)
    result.add_argument("--observation", choices=("dense", "compact"), default="dense")
    result.add_argument("--loops", default="example,facade,binding",
                        help="comma-separated loops to run, in this order (M38a)")
    result.add_argument("--drop-before-step", action="store_true",
                        help="binding loop: release the previous arrays before each step (M38a)")
    result.add_argument("--out", help="write the JSON here as well as to stdout")
    return result


def vector_choice(rng: np.random.Generator, mask: np.ndarray) -> np.ndarray:
    """One uniform legal action per row, without a Python loop over slots."""
    legal = mask.reshape(mask.shape[0], -1) != 0
    keys = np.where(legal, rng.random(legal.shape), -1.0)
    return keys.argmax(axis=1).astype(np.int64)


def summary(name: str, steps: int, slots: int, timers: dict[str, float]) -> dict:
    total = sum(timers.values())
    per_call = {key: 1e3 * value / steps for key, value in timers.items()}
    return {"loop": name, "calls": steps, "slot_steps": steps * slots, "ms_per_call": per_call,
            "slot_steps_per_second": steps * slots / total if total else 0.0,
            "slot_steps_per_second_timed_step_only": steps * slots / timers["step"] if timers["step"] else 0.0}


def example_loop(configs, args) -> dict:
    env = RunVectorEnv(configs, profile="standard", workers=args.workers, autoreset_mode="next_step", observation=args.observation)
    rng = np.random.default_rng(args.seed)
    timers = {"policy": 0.0, "step": 0.0}
    try:
        observation, _ = env.reset(seed=args.seed)
        waiting = np.zeros(args.slots, dtype=np.bool_)
        for _ in range(args.steps):
            start = time.perf_counter()
            actions = np.zeros(args.slots, dtype=np.int64)
            for slot in range(args.slots):
                if waiting[slot]:
                    continue
                legal = np.flatnonzero(observation["action_mask"][slot].reshape(-1))
                actions[slot] = int(rng.choice(legal))
            middle = time.perf_counter()
            observation, _, terminated, _, _ = env.step(actions)
            end = time.perf_counter()
            waiting = np.asarray(terminated, dtype=np.bool_)
            timers["policy"] += middle - start
            timers["step"] += end - middle
    finally:
        env.close()
    return summary("example", args.steps, args.slots, timers)


def facade_loop(configs, args) -> dict:
    env = RunVectorEnv(configs, profile="standard", workers=args.workers, autoreset_mode="next_step", observation=args.observation)
    rng = np.random.default_rng(args.seed)
    timers = {"policy": 0.0, "step": 0.0}
    try:
        observation, _ = env.reset(seed=args.seed)
        for _ in range(args.steps):
            start = time.perf_counter()
            actions = vector_choice(rng, observation["action_mask"])
            middle = time.perf_counter()
            observation, *_ = env.step(actions)
            timers["policy"] += middle - start
            timers["step"] += time.perf_counter() - middle
    finally:
        env.close()
    return summary("facade", args.steps, args.slots, timers)


def binding_loop(configs, args) -> dict:
    batch = _native.NativeRunBatch(configs, profile="standard", workers=args.workers, autoreset="next_step", observation=args.observation)
    rng = np.random.default_rng(args.seed)
    timers = {"policy": 0.0, "step": 0.0}
    free_previous = 0.0
    try:
        raw = batch.reset(seeds=args.seed)
        observation_bytes = sum(value.nbytes for value in raw["obs"].values())
        for _ in range(args.steps):
            start = time.perf_counter()
            actions = vector_choice(rng, raw["obs"]["action_mask"])
            tokens = raw["tokens"]
            previous, raw = raw, None
            middle = time.perf_counter()
            if args.drop_before_step:
                previous = None
            released = time.perf_counter()
            raw = batch.step(actions, tokens)
            stepped = time.perf_counter()
            del previous
            end = time.perf_counter()
            timers["policy"] += middle - start
            timers["step"] += end - middle
            free_previous += (released - middle) + (end - stepped)
    finally:
        batch.close()
    result = summary("binding", args.steps, args.slots, timers)
    result["free_previous_ms_per_call"] = 1e3 * free_previous / args.steps
    result["drop_before_step"] = args.drop_before_step
    result["stacked_observation_bytes_per_call"] = observation_bytes
    return result


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    if not 1 <= args.slots <= 256 or args.slots % 4:
        parser().error("--slots must be a positive multiple of four no greater than 256")
    if args.steps < 1 or args.workers < 1:
        parser().error("--steps and --workers must be positive")
    configs = [{"character": CHARACTERS[slot % 4], "ascension": args.ascension} for slot in range(args.slots)]
    runners = {"example": example_loop, "facade": facade_loop, "binding": binding_loop}
    names = args.loops.split(",")
    if not names or any(name not in runners for name in names):
        parser().error("--loops takes a comma-separated subset of example, facade and binding")
    loops = [runners[name](configs, args) for name in names]
    result = {"schema_id": "spire-gym-m37-python-phase-timing-v1", "native_file": _native.__file__,
              "build_info": _native.build_info(), "observation": args.observation, "slots": args.slots, "ascension": args.ascension,
              "workers": args.workers, "steps": args.steps, "seed": args.seed, "loops": loops}
    text = json.dumps(result, indent=2, sort_keys=True)
    print(text)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as handle:
            handle.write(text + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
