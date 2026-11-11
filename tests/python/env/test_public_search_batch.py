"""Python contract for batched public-information branch stepping."""

import numpy as np
import pytest
from concurrent.futures import ThreadPoolExecutor

from spire_lab import _native
from spire_lab.gymnasium import StandardCombatEnv
from .test_public_search import LIVE_SEED, RESEARCH_SEED, entry


def _action(obs):
    return int(np.flatnonzero(obs["action_mask"].reshape(-1))[0])


def test_step_many_full_and_mask_match_sequential_and_update_owned_tokens():
    env = StandardCombatEnv(entry(), partition="test")
    try:
        env.reset(seed=LIVE_SEED)
        batch_search = env.public_search(RESEARCH_SEED, workers=2)
        seq_search = env.public_search(RESEARCH_SEED, workers=1)
        batch_branches = [batch_search.branch(k) for k in (3, 7, 11)]
        seq_branches = [seq_search.branch(k) for k in (3, 7, 11)]
        initial = [branch.decision()[0] for branch in batch_branches]
        actions = [_action(obs) for obs in initial]
        old_tokens = [branch._token for branch in batch_branches]
        result = batch_search.step_many(batch_branches, actions)
        assert isinstance(result[0], dict)
        assert result[0]["action_mask"].shape[0] == len(actions)
        assert result[1].dtype == np.dtype(np.int32)
        assert result[2].dtype == result[3].dtype == np.dtype(np.bool_)
        assert result[4]["_fault"].dtype == np.dtype(np.bool_)
        assert not result[4]["_fault"].any()
        for branch, before in zip(batch_branches, old_tokens, strict=True):
            assert branch._token != before
        sequential = [branch.step(action, view="mask") for branch, action in
                      zip(seq_branches, actions, strict=True)]
        np.testing.assert_array_equal(result[0]["action_mask"], np.stack([row[0] for row in sequential]))
        np.testing.assert_array_equal(result[1], [row[1] for row in sequential])
        np.testing.assert_array_equal(result[2], [row[2] for row in sequential])
        np.testing.assert_array_equal(result[3], [row[3] for row in sequential])

        # Full view is also stacked, and mask mode has the documented compact key.
        full_actions = [_action(branch.decision()[0]) for branch in batch_branches]
        full_sequential = [branch.step(action, view="full")
                           for branch, action in zip(seq_branches, full_actions, strict=True)]
        full = batch_search.step_many(batch_branches, full_actions, view="full")
        assert set(full[0]) >= {"action_mask", "player"}
        for key, values in full[0].items():
            np.testing.assert_array_equal(values, np.stack([row[0][key] for row in full_sequential]))
        np.testing.assert_array_equal(full[1], [row[1] for row in full_sequential])
        np.testing.assert_array_equal(full[2], [row[2] for row in full_sequential])
        np.testing.assert_array_equal(full[3], [row[3] for row in full_sequential])
        mask_actions = [_action(branch.decision()[0]) for branch in batch_branches]
        masked = batch_search.step_many(batch_branches, mask_actions, view="mask")
        assert set(masked[0]) == {"action_mask"}
        batch_search.close()
        seq_search.close()
    finally:
        env.close()


def test_step_many_rejects_mixed_search_owners_before_mutation():
    env = StandardCombatEnv(entry(), partition="test")
    try:
        env.reset(seed=LIVE_SEED)
        first = env.public_search(RESEARCH_SEED)
        second = env.public_search(RESEARCH_SEED + 1)
        branch = first.branch(1)
        foreign = second.branch(2)
        with pytest.raises(_native.RunInputError):
            first.step_many([branch, foreign], [0, 0])
        first.close()
        second.close()
    finally:
        env.close()


def test_step_many_parallel_search_owners_match_sequential_results():
    env = StandardCombatEnv(entry(), partition="test")
    try:
        env.reset(seed=LIVE_SEED)
        searches = [env.public_search(RESEARCH_SEED, workers=2) for _ in range(4)]
        branch_groups = [[search.branch(k) for k in (17, 29)] for search in searches]
        action_groups = [[_action(branch.decision()[0]) for branch in group]
                         for group in branch_groups]
        def step(index):
            return searches[index].step_many(branch_groups[index], action_groups[index])
        with ThreadPoolExecutor(max_workers=2) as pool:
            concurrent = list(pool.map(step, (0, 1)))
        sequential = [step(index) for index in (2, 3)]
        for actual, expected in zip(concurrent, sequential, strict=True):
            assert actual[0].keys() == expected[0].keys()
            for key in actual[0]:
                np.testing.assert_array_equal(actual[0][key], expected[0][key])
            for left, right in zip(actual[1:4], expected[1:4], strict=True):
                np.testing.assert_array_equal(left, right)
            assert actual[4].keys() == expected[4].keys()
            for key in actual[4]:
                np.testing.assert_array_equal(actual[4][key], expected[4][key])
        for search in searches:
            search.close()
    finally:
        env.close()


def test_step_many_budget_preflight_is_atomic():
    env = StandardCombatEnv(entry(), partition="test")
    try:
        env.reset(seed=LIVE_SEED)
        search = env.public_search(RESEARCH_SEED, limits={
            "max_live_branches": 4, "max_branches": 8,
            "max_steps_per_branch": 16, "max_total_steps": 1,
        })
        branches = [search.branch(4), search.branch(5)]
        actions = [_action(branch.decision()[0]) for branch in branches]
        before = search.counters()
        with pytest.raises(_native.RunBudgetError):
            search.step_many(branches, actions)
        assert search.counters() == before
        search.close()
    finally:
        env.close()


def test_search_workers_cap_rejects_257_without_state_change():
    # M36 (spire-dmm M5): 1 <= workers <= 256, through the same native path as Rust.
    env = StandardCombatEnv(entry(), partition="test")
    try:
        obs, _ = env.reset(seed=LIVE_SEED)
        for bad in (0, 257):
            with pytest.raises(_native.RunInputError, match=r"search workers must be 1\.\.=256"):
                env.public_search(RESEARCH_SEED, workers=bad)
        # 256 is admitted (no pool exists until a parallel step_many), and the failed opens left the env usable.
        search = env.public_search(RESEARCH_SEED, workers=256)
        assert search.branch(1).decision()[0]["action_mask"].any()
        search.close()
        env.step(_action(obs))
    finally:
        env.close()
