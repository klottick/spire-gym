"""Matched logical jobs across manual, serial and worker-backed facades."""

from copy import deepcopy

import numpy as np
import pytest

from spire_lab.gymnasium import CombatEnv, CombatVectorEnv
from .support import template


def equal_tree(left, right):
    if isinstance(left, dict):
        assert left.keys() == right.keys()
        for key in left:
            equal_tree(left[key], right[key])
    elif isinstance(left, (tuple, list)):
        assert len(left) == len(right)
        for a, b in zip(left, right):
            equal_tree(a, b)
    elif isinstance(left, np.ndarray):
        assert left.dtype == right.dtype and left.shape == right.shape
        if left.dtype == object:
            for a, b in zip(left.flat, right.flat):
                equal_tree(a, b)
        else:
            np.testing.assert_array_equal(left, right)
    else:
        assert left == right


def resource_catalog():
    return [template(id="resources", player_hp=70, enemy_hp=48, energy_per_turn=8,
                     hand=[dict(kind="strike", upgraded=False)] * 8,
                     context=dict(act=1, next_encounter="jaw_worm"))]


@pytest.mark.parametrize("workers", [1, 2, 4])
def test_masked_contract_end_to_end_and_wrapper_subset(workers):
    catalog = resource_catalog() + [template(id="death", player_hp=1)]
    vec = CombatVectorEnv(3, workers=workers, catalog=catalog)
    serial = CombatVectorEnv(3, catalog=catalog)
    singles = [CombatEnv(catalog=catalog) for _ in range(3)]
    try:
        # Repeat explicit reseeding, including mixed episode lengths and endings.
        for _ in range(2):
            options = {"scenario_ids": ["death", "resources", "death"]}
            obs, info = vec.reset(seed=255, options=options)
            equal_tree((obs, info), serial.reset(seed=255, options=options))
            for i, env in enumerate(singles):
                single, single_info = env.reset(seed=255+i, options={
                    "scenario_id": options["scenario_ids"][i]})
                equal_tree(single, {k: v[i] for k, v in obs.items()})
                for key, value in single_info.items():
                    assert info[key][i] == value
            victories = deaths = 0
            for _ in range(32):
                assert vec.observation_space.contains(obs)
                actions = np.zeros(3, dtype=np.int64)
                actions[1] = int(obs["action_mask"][1, 1] != 0)
                assert all(obs["action_mask"][i, a] for i, a in enumerate(actions))
                result = vec.step(actions)
                equal_tree(result, serial.step(actions))
                obs, rewards, terms, truncs, info = result
                assert vec.observation_space.contains(obs)
                assert not truncs.any()
                for i, env in enumerate(singles):
                    single, reward, term, trunc, single_info = env.step(actions[i])
                    assert env.observation_space.contains(single)
                    assert (reward, term, trunc) == (rewards[i], terms[i], truncs[i])
                    if term:
                        victories += reward == 1
                        deaths += reward == -1
                        equal_tree(single, info["final_obs"][i])
                        assert not single["action_mask"].any()
                        for key, value in single_info.items():
                            assert info["final_info"][key][i] == value
                        if reward == 1:
                            assert single_info["hp_before_healing"] == 70
                            assert single_info["hp_after_cleanup"] == 76
                            assert single["in_play_counts"][0, 0] == 1
                            # Full final resource/context row equals manual, not replacement.
                            assert single["context"].tolist() == [1, 0, 1, 2]
                            assert single["context_mask"].tolist() == [1, 1, 1, 1]
                        single, single_info = env.reset()
                    equal_tree(single, {k: v[i] for k, v in obs.items()})
                    for key, value in single_info.items():
                        assert info[key][i] == value
            assert victories and deaths
    finally:
        vec.close()
        serial.close()
        for env in singles:
            env.close()


def test_ordinary_transition_and_retained_current_final_after_reset_and_close():
    plain = CombatVectorEnv(3)
    try:
        plain.reset(seed=0)
        obs, reward, terminated, truncated, info = plain.step(np.zeros(3, dtype=np.int64))
        assert plain.observation_space.contains(obs)
        np.testing.assert_array_equal(truncated, np.zeros(3, dtype=np.bool_))
        assert not terminated.any() and "final_obs" not in info
    finally:
        plain.close()

    vec = CombatVectorEnv(3, catalog=resource_catalog(), workers=2)
    vec.reset(seed=0)
    for _ in range(7):
        vec.step(np.ones(3, dtype=np.int64))
    retained = vec.step(np.array([1, 0, 1], dtype=np.int64))
    saved = deepcopy(retained)
    assert retained[4]["_final_obs"].tolist() == [True, False, True]
    try:
        vec.step(np.zeros(3, dtype=np.int64))
        equal_tree(retained, saved)
        replacement, _ = vec.reset(seed=17)
        equal_tree(retained, saved)
        for key in replacement:
            for i in (0, 2):
                assert not np.shares_memory(replacement[key], retained[4]["final_obs"][i][key])
            assert not np.shares_memory(replacement[key], retained[0][key])
        replacement["player"][:] = -99
        equal_tree(retained, saved)
    finally:
        vec.close()
    equal_tree(retained, saved)
