"""Show two synthetic tactical starts, one reachable recipe, and paired baselines.

These examples demonstrate repeatable setup and evaluation mechanics. Their
outcomes do not establish a strategy strength claim.
"""

from __future__ import annotations

import hashlib
import json
import random

import numpy as np

from spire_lab.gymnasium import RunEnv, StandardCombatEnv


SEED = 84_211
MAX_STEPS = 160


def entry(entry_id, character, encounter, deck, relic):
    return {
        "id": entry_id,
        "character": character,
        "ascension": 0,
        "encounter": encounter,
        "hp": 70,
        "max_hp": 80,
        "gold": 99,
        "deck": [{"card": card, "upgrades": upgrades, "misc": 0} for card, upgrades in deck],
        "relics": [relic],
        "potions": [None, None, None],
    }


def card(card_id, upgrades=0, generated=False):
    return {"card": card_id, "upgrades": upgrades, "misc": 0, "generated": generated}


def lessons():
    ironclad_deck = [("strike", 0)] * 4 + [("defend", 0)] * 3 + [
        ("bash", 0), ("true_grit", 1), ("burning_pact", 0), ("second_wind", 0), ("shrug_it_off", 0)
    ]
    l1 = entry("l1-ironclad-nob", "ironclad", "gremlin_nob", ironclad_deck, "burning_blood")
    l1["position"] = {
        "player": {"powers": {"feel_no_pain": 3, "dark_embrace": 1}},
        "piles": {
            "hand": [card("true_grit", 1), card("burning_pact"), card("second_wind"), card("shrug_it_off"), card("bash")],
            "draw": [card("strike")] * 3 + [card("defend")] * 3,
            "discard": [card("strike")],
        },
    }

    silent_deck = [("strike_g", 0)] * 3 + [("defend_g", 0)] * 2 + [
        ("neutralize", 0), ("survivor", 0), ("deadly_poison", 0), ("bouncing_flask", 0), ("catalyst", 1), ("footwork", 0)
    ]
    l2 = entry("l2-silent-poison", "silent", "jaw_worm", silent_deck, "ring_of_the_snake")
    l2["position"] = {
        "enemies": [{"slot": 0, "powers": {"poison": 4}}],
        "piles": {
            "hand": [card("deadly_poison"), card("bouncing_flask"), card("catalyst", 1), card("neutralize"), card("footwork")],
            "draw": [card("strike_g")] * 3 + [card("defend_g")] * 2 + [card("survivor")],
        },
    }
    return l1, l2


def catalog(entries):
    return {"catalog_schema_id": "spire-gym-standard-combat-catalog-v1", "profile_id": "standard-combat-v1", "entries": entries}


def copy_observation(observation):
    return {key: value.copy() for key, value in observation.items()}


def assert_same_observation(left, right):
    assert left.keys() == right.keys()
    for key in left:
        np.testing.assert_array_equal(left[key], right[key])


def make_recipe():
    run = RunEnv(
        "ironclad", 0, profile="standard", partition="test",
        creation={"neow": "short"}, record_replay=True,
    )
    try:
        observation, _ = run.reset(seed=SEED + 1)
        for _ in range(80):
            if run.combat_view()["obs"]["enemies_occupied"].any():
                break
            legal = np.flatnonzero(observation["action_mask"].reshape(-1))
            observation, _, terminated, truncated, _ = run.step(int(legal[0]))
            if terminated or truncated:
                raise RuntimeError("Standard route ended before its first combat")
        else:
            raise RuntimeError("Standard route did not reach its first combat")
        record = run.record()
        if record is None:
            raise RuntimeError("record_replay did not produce a Standard Run record")
        steps = len(json.loads(record)["steps"])
        return {"id": "reachable-first-combat", "recipe": {
            "record_sha256": hashlib.sha256(record).hexdigest(), "steps": steps,
        }}, record
    finally:
        run.close()


def demonstrate_lesson(env, entry_id):
    observation, info = env.reset(seed=SEED, options={"entry_id": entry_id})
    legal = np.flatnonzero(observation["action_mask"].reshape(-1))
    next_observation, reward, terminated, truncated, _ = env.step(int(legal[0]))
    print(f"{entry_id}: start_kind={info['start_kind']} first_action={int(legal[0])} reward={reward:+.1f} terminal={terminated or truncated}")
    return next_observation


def run_policy(env, policy_name, seed):
    observation, info = env.reset(seed=seed, options={"entry_id": "paired-nob"})
    initial = copy_observation(observation)
    trajectory = [initial]
    actions = []
    terminal_info = {}
    rng = random.Random(seed ^ 0x51A7)
    terminated = truncated = False
    for _ in range(MAX_STEPS):
        legal = [int(action) for action in np.flatnonzero(observation["action_mask"].reshape(-1))]
        if not legal:
            break
        action = legal[0] if policy_name == "first_legal" else rng.choice(legal)
        actions.append(action)
        observation, reward, terminated, truncated, terminal_info = env.step(action)
        trajectory.append(copy_observation(observation))
        if terminated or truncated:
            break
    return initial, actions, trajectory, copy_observation(observation), reward, terminated, truncated, terminal_info, info


def replay_actions(env, seed, actions):
    observation, _ = env.reset(seed=seed, options={"entry_id": "paired-nob"})
    trajectory = [copy_observation(observation)]
    for action in actions:
        observation, _, terminated, truncated, _ = env.step(action)
        trajectory.append(copy_observation(observation))
        if terminated or truncated:
            break
    return trajectory


def main():
    l1, l2 = lessons()
    recipe, record = make_recipe()
    paired = entry(
        "paired-nob", "ironclad", "gremlin_nob",
        [("strike", 0)] * 5 + [("defend", 0)] * 4 + [("bash", 0)], "burning_blood",
    )
    entries = catalog([l1, l2, recipe, paired])
    env = StandardCombatEnv(entries, records=[record], partition="test")
    try:
        demonstrate_lesson(env, l1["id"])
        demonstrate_lesson(env, l2["id"])
        recipe_obs, recipe_info = env.reset(seed=SEED, options={"entry_id": recipe["id"]})
        print(f"{recipe['id']}: start_kind={recipe_info['start_kind']} enemies={int(recipe_obs['enemies_occupied'].sum())}")

        first = run_policy(env, "first_legal", SEED)
        random_result = run_policy(env, "seeded_random", SEED)
        assert_same_observation(first[0], random_result[0])

        replay = StandardCombatEnv(entries, records=[record], partition="test")
        try:
            replay_trajectory = replay_actions(replay, SEED, first[1])
        finally:
            replay.close()
        assert len(replay_trajectory) == len(first[1]) + 1
        assert_same_observation(first[0], replay_trajectory[0])
        assert len(first[2]) == len(replay_trajectory)
        for expected, actual in zip(first[2], replay_trajectory):
            assert_same_observation(expected, actual)

        for label, result in (("first_legal", first), ("seeded_random", random_result)):
            _, actions, _, _, reward, terminated, truncated, info, reset_info = result
            outcome = info.get("terminal_reason", "step_limit" if not terminated and not truncated else "truncated")
            print(f"{label}: start_kind={reset_info['start_kind']} actions={len(actions)} outcome={outcome} reward={float(reward):+.1f}")
    finally:
        env.close()


if __name__ == "__main__":
    main()
