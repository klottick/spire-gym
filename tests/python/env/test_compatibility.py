"""Applicable pinned upstream checks, not full mask-blind check_env acceptance."""

from copy import deepcopy
import inspect

import gymnasium as gym
from gymnasium.utils import env_checker as checks
from gymnasium.utils.passive_env_checker import (
    check_action_space, check_observation_space,
    env_reset_passive_checker, env_step_passive_checker,
)
import numpy as np
import pytest

from spire_lab.gymnasium import (
    CombatEnv, CombatVectorEnv, IroncladCombatEnv, RunEnv, RunVectorEnv,
    StandardCombatEnv,
)
from .test_integration import equal_tree, resource_catalog
from .test_standard_combat import catalog as standard_catalog, entry as standard_entry


def test_official_reset_space_signature_and_masked_passive_subset():
    env = CombatEnv()
    try:
        assert list(inspect.signature(env.reset).parameters) == ["seed", "options"]
        assert list(inspect.signature(env.step).parameters) == ["action"]
        for check in (checks.check_reset_seed_determinism, checks.check_reset_options,
                      checks.check_reset_return_info_deprecation, checks.check_seed_deprecation,
                      checks.check_reset_return_type):
            check(env)
        check_action_space(env.action_space)
        with pytest.warns(UserWarning, match="maximum and minimum values are equal"):
            check_observation_space(env.observation_space)
        checks.check_space_limit(env.action_space, "action")
        checks.check_space_limit(env.observation_space, "observation")
        obs, _ = env_reset_passive_checker(env, seed=0, options={})
        env.action_space.seed(15)
        for _ in range(100):
            action = env.action_space.sample(mask=obs["action_mask"])
            obs, _, term, trunc, _ = env_step_passive_checker(env, action)
            assert env.observation_space.contains(obs) and not trunc
            if term:
                break
        assert term
    finally:
        env.close()


def test_single_facade_passive_checkers_follow_legal_masks():
    gym66_catalog = {"profile_id": "ironclad-combat-v2", "entries": [{
        "id": "base", "encounter": "cultist", "hp": 80, "max_hp": 80,
        "deck": [{"card": "strike", "upgrades": 0}] * 10,
        "relics": [], "potions": [None, None, None], "relic_counters": {},
    }]}
    envs = [
        (RunEnv("ironclad", 0, profile="standard"), 5),
        (StandardCombatEnv(standard_catalog(standard_entry("base")), partition="test"), 6),
        (IroncladCombatEnv(gym66_catalog, partition="test"), 7),
    ]
    try:
        for env, seed in envs:
            first, _ = env.reset(seed=seed)
            second, _ = env.reset(seed=seed)
            equal_tree(first, second)
            assert all(not np.shares_memory(first[key], second[key]) for key in first)
            observation, _ = env_reset_passive_checker(env, seed=seed)
            assert env.observation_space.contains(observation)
            for _ in range(4):
                legal = np.flatnonzero(observation["action_mask"].reshape(-1))
                assert legal.size
                action = int(legal[0])
                observation, _, terminated, truncated, _ = env_step_passive_checker(env, action)
                assert env.observation_space.contains(observation)
                assert not truncated
                if terminated:
                    break
    finally:
        for env, _ in envs:
            env.close()


def test_supported_single_identity_transform_and_vector_mode_rejection():
    base = CombatEnv()
    wrapped = gym.wrappers.TransformObservation(
        CombatEnv(), func=lambda obs: obs, observation_space=base.observation_space)
    vec = CombatVectorEnv(3)
    try:
        equal_tree(base.reset(seed=2), wrapped.reset(seed=2))
        for _ in range(3):
            equal_tree(base.step(0), wrapped.step(0))
        with pytest.raises(ValueError, match="autoreset_mode.*NEXT_STEP or DISABLED"):
            gym.vector.VectorObservationWrapper(vec)
    finally:
        base.close()
        wrapped.close()
        vec.close()


def test_vector_statistics_same_step_returns_lengths_and_partial_reset_isolation():
    wrappers = [gym.wrappers.vector.RecordEpisodeStatistics(
        CombatVectorEnv(3, catalog=resource_catalog(), workers=w, record_replay=True))
        for w in (1, 2)]
    try:
        for wrapper in wrappers:
            wrapper.reset(seed=7)
        for _ in range(8):
            results = [w.step(np.ones(3, dtype=np.int64)) for w in wrappers]
            for result in results:
                assert not result[3].any()
                if result[2].any():
                    np.testing.assert_array_equal(result[4]["episode"]["r"], np.ones(3))
                    np.testing.assert_array_equal(result[4]["episode"]["l"], np.full(3, 8))
                    assert result[4]["_episode"].all()
                    # Wall-clock time is intentionally not deterministic metadata.
                    result[4]["episode"].pop("t")
            equal_tree(*results)
        for wrapper in wrappers:
            wrapper.step(np.ones(3, dtype=np.int64))
            before = deepcopy(wrapper.env.reproduction(0))
            options = {"reset_mask": np.array([False, True, False])}
            supplied = deepcopy(options)  # Upstream pops reset_mask; base does not.
            obs, info = wrapper.reset(options=supplied)
            assert "reset_mask" not in supplied
            assert options["reset_mask"].tolist() == [False, True, False]
            assert wrapper.episode_lengths.tolist() == [1, 0, 1]
            assert wrapper.episode_returns.tolist() == [0, 0, 0]
            assert info["_episode_id"].tolist() == [False, True, False]
            assert info["episode_steps"].tolist() == [0, 0, 0]  # Absent rows are masked zeros.
            equal_tree(before, wrapper.env.reproduction(0))
            assert wrapper.env.reproduction(1)["actions"].size == 0
            assert wrapper.observation_space.contains(obs)
    finally:
        for wrapper in wrappers:
            wrapper.close()


def test_run_vector_wrapper_mode_matrix_and_statistics_wrapper_support():
    for mode in ("disabled", "next_step", "same_step"):
        env = RunVectorEnv([{"character": "ironclad", "ascension": 0}],
                           autoreset_mode=mode)
        try:
            if mode == "same_step":
                with pytest.raises(ValueError, match="autoreset_mode.*NEXT_STEP or DISABLED"):
                    gym.vector.VectorObservationWrapper(env)
            else:
                wrapper = gym.vector.VectorObservationWrapper(env)
                assert wrapper.single_observation_space == env.single_observation_space
            stats = gym.wrappers.vector.RecordEpisodeStatistics(env)
            stats.close()
        finally:
            env.close()
