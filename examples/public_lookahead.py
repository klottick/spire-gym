"""Small, non-strategic public-belief lookahead demonstration.

For every legal combat action, sample K public worlds and finish each with a
uniform random-legal policy. The printed means describe this baseline only.
"""

from __future__ import annotations

import argparse
import random

import numpy as np

from spire_lab.gymnasium import RunEnv, StandardCombatEnv


SEED = 1_942_013
MAX_STEPS_PER_BRANCH = 384


def entry_catalog():
    return {
        "catalog_schema_id": "spire-gym-standard-combat-catalog-v1",
        "profile_id": "standard-combat-v1",
        "entries": [{
            "id": "lookahead",
            "character": "ironclad",
            "ascension": 0,
            "encounter": "jaw_worm",
            "hp": 70,
            "max_hp": 80,
            "gold": 99,
            "deck": [{"card": "strike", "upgrades": 0}] * 5
                + [{"card": "defend", "upgrades": 0}] * 4
                + [{"card": "bash", "upgrades": 0}],
            "relics": ["burning_blood"],
            "potions": [None, None, None],
        }],
    }


def enter_run_combat(env):
    observation, _ = env.reset(seed=SEED)
    for _ in range(40):
        if observation["enemies_occupied"].any():
            return observation
        legal = np.flatnonzero(observation["action_mask"].reshape(-1))
        observation, _, terminated, _, _ = env.step(int(legal[0]))
        if terminated:
            break
    raise RuntimeError("the Standard route did not reach a combat decision")


def evaluate(label, env, observation, samples):
    legal = [int(i) for i in np.flatnonzero(observation["action_mask"].reshape(-1))]
    arena = env.public_search(
        SEED,
        max_live_branches=1,
        max_branches=len(legal) * samples,
        max_steps_per_branch=MAX_STEPS_PER_BRANCH,
        max_total_steps=len(legal) * samples * MAX_STEPS_PER_BRANCH,
    )
    print(f"{label}: {len(legal)} legal root actions, K={samples}")
    try:
        for action in legal:
            outcomes = []
            for sample in range(samples):
                branch = arena.branch(sample)
                try:
                    branch.decision()
                    mask, reward, terminated, truncated, _ = branch.step(action, view="mask")
                    policy = random.Random((SEED << 32) ^ (action << 16) ^ sample)
                    while not terminated and not truncated:
                        choices = np.flatnonzero(mask.reshape(-1))
                        mask, reward, terminated, truncated, _ = branch.step(
                            int(policy.choice(choices)), view="mask"
                        )
                    outcomes.append(float(reward))
                finally:
                    branch.close()
            print(f"  action={action:4d} outcome_mean={sum(outcomes) / len(outcomes):+.3f}")
        print(f"  counters={arena.counters()}")
    finally:
        arena.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("-k", "--samples", type=int, default=4)
    args = parser.parse_args()
    if args.samples < 1:
        parser.error("--samples must be positive")

    standalone = StandardCombatEnv(entry_catalog(), partition="test")
    run = RunEnv(
        "ironclad", 0, profile="standard", partition="test",
        creation={"neow": "short"},
    )
    try:
        standalone_observation, _ = standalone.reset(seed=SEED)
        evaluate("standalone Standard start", standalone, standalone_observation, args.samples)
        enter_run_combat(run)
        run_observation = run.combat_view()["obs"]
        evaluate("Standard Run combat", run, run_observation, args.samples)
    finally:
        standalone.close()
        run.close()


if __name__ == "__main__":
    main()
