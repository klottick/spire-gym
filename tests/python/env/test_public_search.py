"""Python contract for public-belief search over Standard combat decisions."""

import pickle

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import PublicBranch, PublicSearch, RunEnv, StandardCombatEnv


LIMITS = dict(
    max_live_branches=4,
    max_branches=16,
    max_steps_per_branch=128,
    max_total_steps=1024,
)
RESEARCH_SEED = 9_182_736_451_091
SAMPLE_INDEX = 73_910_273
LIVE_SEED = 8_118_273_645_109
HIDDEN_ORDER_PICK = "a pick among identical draw-pile copies depends on hidden order"


def entry(*, dome=False):
    return {
        "catalog_schema_id": "spire-gym-standard-combat-catalog-v1",
        "profile_id": "standard-combat-v1",
        "entries": [{
            "id": "base",
            "character": "ironclad",
            "ascension": 0,
            "encounter": "jaw_worm",
            "hp": 70,
            "max_hp": 80,
            "gold": 99,
            "deck": [{"card": "strike", "upgrades": 0}] * 5
                + [{"card": "defend", "upgrades": 0}] * 4
                + [{"card": "bash", "upgrades": 0}],
            "relics": ["burning_blood"] + (["runic_dome"] if dome else []),
            "potions": [None, None, None],
        }],
    }


def assert_observation_equal(left, right):
    assert left.keys() == right.keys()
    for key in left:
        np.testing.assert_array_equal(left[key], right[key])


def root_search(env, seed=RESEARCH_SEED):
    return env.public_search(seed, **LIMITS)


def search_step(branch, legal_actions, *, view):
    for action in legal_actions:
        try:
            return branch.step(action, view=view), action
        except _native.RunUnsupportedBoundaryError as error:
            if str(error) != HIDDEN_ORDER_PICK:
                raise
    pytest.fail("all legal branch actions were refused for hidden draw order")


def test_limits_mapping_defaults_and_none_match_the_declared_hard_caps():
    env = StandardCombatEnv(entry(), partition="test")
    try:
        env.reset(seed=LIVE_SEED)
        defaults = env.public_search(RESEARCH_SEED, limits=None)
        explicit = env.public_search(RESEARCH_SEED, limits={"max_live_branches": 2})
        legacy = env.public_search(RESEARCH_SEED, **LIMITS)
        assert defaults.limits() == defaults.spec()["hard_caps"]
        assert explicit.limits() == {**defaults.spec()["hard_caps"], "max_live_branches": 2}
        assert legacy.limits() == LIMITS
        defaults.close()
        explicit.close()
        legacy.close()
    finally:
        env.close()


def test_fresh_and_fork_mask_steps_use_creation_tokens_without_full_encoding(monkeypatch):
    env = StandardCombatEnv(entry(), partition="test")
    try:
        observation, _ = env.reset(seed=LIVE_SEED)
        search = root_search(env)
        branch = search.branch(SAMPLE_INDEX)
        fork = branch.fork()
        action = int(np.flatnonzero(observation["action_mask"].reshape(-1))[0])
        calls = 0
        original = PublicBranch.decision

        def counted(self):
            nonlocal calls
            calls += 1
            return original(self)

        monkeypatch.setattr(PublicBranch, "decision", counted)
        legal = np.flatnonzero(observation["action_mask"].reshape(-1)).tolist()
        (branch_mask, *_), _ = search_step(branch, legal, view="mask")
        (fork_mask, *_), _ = search_step(fork, legal, view="mask")
        assert branch_mask.shape == fork_mask.shape
        assert calls == 0
        assert branch._token == (branch._branch_id, 1)
        assert fork._token == (fork._branch_id, 1)
        search.close()
    finally:
        env.close()


def test_root_digest_is_public_and_independent_of_other_sessions():
    catalog = entry()
    first = StandardCombatEnv(catalog, partition="test")
    second = StandardCombatEnv(catalog, partition="test")
    unrelated = StandardCombatEnv(catalog, partition="test")
    try:
        first.reset(seed=LIVE_SEED)
        second.reset(seed=LIVE_SEED)
        unrelated_observation, _ = unrelated.reset(seed=29)
        left = root_search(first)
        right = root_search(second)
        assert left.spec()["root_digest"] == right.spec()["root_digest"]
        digest = left.spec()["root_digest"]
        unrelated.step(int(np.flatnonzero(unrelated_observation["action_mask"].reshape(-1))[0]))
        assert root_search(first).spec()["root_digest"] == digest
    finally:
        first.close()
        second.close()
        unrelated.close()


def test_first_branch_view_matches_standalone_and_run_decisions():
    standalone = StandardCombatEnv(entry(), partition="test")
    try:
        current, _ = standalone.reset(seed=LIVE_SEED)
        search = root_search(standalone)
        branch = search.branch(SAMPLE_INDEX)
        first, _ = branch.decision()
        assert_observation_equal(first, current)
        search.close()
    finally:
        standalone.close()

    run = RunEnv(
        "ironclad", 0, profile="standard", partition="test",
        creation={"neow": "short"},
    )
    try:
        observation, _ = run.reset(seed=LIVE_SEED)
        for _ in range(40):
            if observation["enemies_occupied"].any():
                break
            legal = np.flatnonzero(observation["action_mask"].reshape(-1))
            observation, _, terminated, _, _ = run.step(int(legal[0]))
            assert not terminated
        else:
            pytest.fail("Standard route did not reach a combat decision")
        search = root_search(run)
        branch = search.branch(SAMPLE_INDEX)
        first, _ = branch.decision()
        assert_observation_equal(first, run.combat_view()["obs"])
        search.close()
    finally:
        run.close()


def test_budgets_stale_tokens_closed_branches_and_dome_errors():
    env = StandardCombatEnv(entry(), partition="test")
    try:
        env.reset(seed=LIVE_SEED)
        search = env.public_search(
            RESEARCH_SEED,
            max_live_branches=1,
            max_branches=LIMITS["max_branches"],
            max_steps_per_branch=LIMITS["max_steps_per_branch"],
            max_total_steps=LIMITS["max_total_steps"],
        )
        branch = search.branch()
        with pytest.raises(_native.RunBudgetError):
            search.branch()
        assert issubclass(_native.RunBudgetError, _native.RunStateError)

        native = env._session.public_search(RESEARCH_SEED, **LIMITS)
        native_branch = native.branch()
        decision = native.decision(native_branch)
        token = decision["token"]
        legal = int(np.flatnonzero(decision["obs"]["action_mask"].reshape(-1))[0])
        native.step(native_branch, token, legal, "full")
        with pytest.raises(_native.RunStateError, match="stale"):
            native.step(native_branch, token, legal, "full")

        branch.close()
        with pytest.raises(_native.RunStateError):
            branch.decision()
        native.close()
        search.close()
    finally:
        env.close()

    dome = StandardCombatEnv(entry(dome=True), partition="test")
    try:
        dome.reset(seed=4)
        with pytest.raises(_native.RunUnsupportedBoundaryError):
            root_search(dome)
    finally:
        dome.close()


def test_search_is_deterministic_owned_redacted_and_supports_mask_steps():
    env = StandardCombatEnv(entry(), partition="test")
    try:
        env.reset(seed=LIVE_SEED)
        left, right = root_search(env), root_search(env)
        assert isinstance(left, PublicSearch)
        a, b = left.branch(SAMPLE_INDEX), right.branch(SAMPLE_INDEX)
        assert isinstance(a, PublicBranch)
        first, info = a.decision()
        repeated, other_info = b.decision()
        assert_observation_equal(first, repeated)
        assert info == other_info
        # Fresh, writable C-contiguous arrays (the shared observation helper adds no copy; final review I3);
        # isolation from the branch is checked by mutating the mask below.
        assert all(
            array.flags.writeable and array.flags.c_contiguous for array in first.values()
        )
        pristine = first["action_mask"].copy()
        first["action_mask"][:] = 0
        again, _ = a.decision()
        np.testing.assert_array_equal(again["action_mask"], pristine)

        legal = int(np.flatnonzero(pristine.reshape(-1))[0])
        fork = a.fork()
        fork_observation, _ = fork.decision()
        assert_observation_equal(again, fork_observation)
        assert fork.actions() == []
        nested = fork.public_search(
            RESEARCH_SEED + 1,
            max_live_branches=LIMITS["max_live_branches"],
            max_branches=LIMITS["max_branches"],
            max_steps_per_branch=LIMITS["max_steps_per_branch"],
            max_total_steps=LIMITS["max_total_steps"],
        )
        nested_branch = nested.branch()
        nested_observation, _ = nested_branch.decision()
        assert_observation_equal(nested_observation, fork_observation)
        nested.close()

        legal_actions = np.flatnonzero(pristine.reshape(-1)).tolist()
        (full_obs, _, _, _, _), accepted_action = search_step(a, legal_actions, view="full")
        assert a.actions() == [accepted_action]
        (mask, _, _, _, _), _ = search_step(b, legal_actions, view="mask")
        np.testing.assert_array_equal(mask, full_obs["action_mask"])

        spec, counters = left.spec(), left.counters()
        rendered = repr(left) + repr(a) + repr(info) + repr(spec) + repr(counters)
        for secret in (str(RESEARCH_SEED), str(SAMPLE_INDEX), str(LIVE_SEED)):
            assert secret not in rendered
        with pytest.raises(_native.RunInputError) as error:
            left._native.actions(LIVE_SEED)
        assert all(secret not in str(error.value) for secret in (str(RESEARCH_SEED), str(SAMPLE_INDEX), str(LIVE_SEED)))
        with pytest.raises(TypeError):
            pickle.dumps(left)
        with pytest.raises(TypeError):
            pickle.dumps(a)
        assert "sample index" not in repr(a).lower()
        left.close()
        right.close()
    finally:
        env.close()
