"""Real native-batched facade contract, ownership and atomicity tests."""

from copy import deepcopy
import importlib

import gymnasium as gym
from gymnasium.vector.utils import batch_space
import numpy as np
import pytest

from spire_lab import _native
from .support import template


def vector_class():
    return importlib.import_module("spire_lab.gymnasium.vector").CombatVectorEnv


def victory_catalog():
    return [template(id="victory", player_hp=70, enemy_hp=48, energy_per_turn=8,
                     hand=[dict(kind="strike", upgraded=False)] * 8)]


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
        if left.dtype == object:
            assert left.shape == right.shape
            for a, b in zip(left.flat, right.flat):
                equal_tree(a, b)
        else:
            np.testing.assert_array_equal(left, right)
    else:
        assert left == right


def test_vector_contract_and_same_step_final_layout():
    cls = vector_class()
    from spire_lab.gymnasium import CombatEnv, CombatVectorEnv
    assert cls is CombatVectorEnv
    vec = cls(3)
    obs, info = vec.reset(seed=255)
    assert isinstance(vec, gym.vector.VectorEnv)
    assert vec.np_random_seed == (255, 256, 257)
    assert obs["player"].shape == (3, 7)
    assert vec.metadata["autoreset_mode"] is gym.vector.AutoresetMode.SAME_STEP
    assert vec.metadata["render_modes"] == []
    assert vec.action_space == batch_space(vec.single_action_space, 3)
    assert vec.observation_space == batch_space(vec.single_observation_space, 3)
    assert isinstance(vec.action_space, gym.spaces.MultiDiscrete)
    assert vec.observation_space.contains(obs)
    assert "tokens" not in info and "effective_seed_bytes" not in info
    vec.close()

    for ended in ([False, False, False], [True, False, True], [True, True, True]):
        vec = cls(3, catalog=victory_catalog())
        manuals = [CombatEnv(catalog=victory_catalog()) for _ in range(3)]
        vec.reset(seed=255)
        for i, env in enumerate(manuals):
            env.reset(seed=255 + i)
        for _ in range(7):
            vec.step(np.ones(3, dtype=np.int64))
            for env in manuals:
                env.step(1)
        actions = np.array(ended, dtype=np.int64)
        obs, rewards, terms, truncs, info = vec.step(actions)
        np.testing.assert_array_equal(terms, ended)
        np.testing.assert_array_equal(rewards, np.array(ended, dtype=np.float32))
        assert rewards.dtype == np.float32 and terms.dtype == truncs.dtype == np.bool_
        assert not truncs.any()
        assert vec.observation_space.contains(obs)
        if not any(ended):
            assert not {"final_obs", "_final_obs", "final_info", "_final_info"} & info.keys()
        else:
            finals = info["final_obs"]
            assert finals.shape == (3,) and finals.dtype == object
            np.testing.assert_array_equal(info["_final_obs"], ended)
            np.testing.assert_array_equal(info["_final_info"], ended)
            for key in ("episode_id", "scenario_id", "episode_steps", "outcome",
                        "hp_before_healing", "hp_after_cleanup"):
                np.testing.assert_array_equal(info["final_info"]["_" + key], ended)
        for i, env in enumerate(manuals):
            final, reward, term, trunc, final_info = env.step(int(actions[i]))
            assert (reward, term, trunc) == (rewards[i], terms[i], truncs[i])
            if ended[i]:
                equal_tree(info["final_obs"][i], final)
                for key, value in final_info.items():
                    assert info["final_info"][key][i] == value
                assert info["episode_id"][i] == "episode-1"
                assert info["episode_steps"][i] == 0
                assert info["final_info"]["hp_before_healing"][i] == 70
                assert info["final_info"]["hp_after_cleanup"][i] == 76
                assert not info["final_obs"][i]["action_mask"].any()
            else:
                equal_tree({k: v[i] for k, v in obs.items()}, final)
                if any(ended):
                    assert info["final_obs"][i] is None
            env.close()
        vec.close()


def test_vector_seed_reset_mask_atomicity():
    vec = vector_class()(3, record_replay=True)
    obs, _ = vec.reset(seed=255)
    original = deepcopy(obs)
    tokens = vec._tokens.copy()
    generators = vec.np_random
    states = [deepcopy(g.bit_generator.state) for g in generators]
    mask = np.array([False, True, False])
    seeds = [999, 2**300 + 255, None]
    options = {"reset_mask": mask, "scenario_ids": [None, None, None]}
    saved_options, saved_seeds = deepcopy(options), deepcopy(seeds)
    obs, info = vec.reset(seed=seeds, options=options)
    equal_tree(options, saved_options)
    equal_tree(seeds, saved_seeds)
    assert vec.np_random_seed == (255, 2**300 + 255, 257)
    for i in [0, 2]:
        assert vec.np_random[i] is generators[i]
        equal_tree(vec.np_random[i].bit_generator.state, states[i])
        np.testing.assert_array_equal(vec._tokens[i], tokens[i])
        equal_tree({k: v[i] for k, v in obs.items()}, {k: v[i] for k, v in original.items()})
    for key in ("episode_id", "scenario_id", "episode_steps", "outcome"):
        np.testing.assert_array_equal(info["_" + key], mask)
    assert info["episode_id"][1] == "episode-0"
    before = vec.np_random
    vec.reset(seed=[None, None, None])
    assert all(a is b for a, b in zip(before, vec.np_random))
    vec.reset(seed=2**300 + 255)
    assert vec.np_random_seed == (2**300 + 255, 2**300 + 256, 2**300 + 257)
    obs, _ = vec.reset(seed=[7, 7, 7])
    for i in [1, 2]:
        equal_tree({k: v[0] for k, v in obs.items()}, {k: v[i] for k, v in obs.items()})
    assert len({id(g) for g in vec.np_random}) == 3
    vec.close()


@pytest.mark.parametrize("seed,options", [
    (1, {"reset_mask": []}), (1, {"reset_mask": np.array([], dtype=bool)}),
    (1, {"reset_mask": np.zeros(3, dtype=bool)}),
    (1, {"reset_mask": np.ones(3, dtype=np.int8)}),
    (1, {"reset_mask": np.ones((3, 1), dtype=bool)}),
    (1, {"unknown": None}), (1, []), (1, {"scenario_ids": [None]}),
    (1, {"scenario_ids": None}),
    (1, {"scenario_ids": (None, None, None)}),
    (1, {"reset_mask": np.array([True, False, False]), "scenario_ids": [None, None, "missing"]}),
    (1, {"reset_mask": np.array([True, False, False]), "scenario_ids": [None, None, "\ud800"]}),
    (1, {"reset_mask": np.array([True, False, False]), "scenario_ids": [None, None, 5]}),
    ([8, None, -1], {"reset_mask": np.array([True, False, False])}),
    ([8, None, True], {"reset_mask": np.array([True, False, False])}),
    ([8, None, np.int64(3)], {"reset_mask": np.array([True, False, False])}),
    ([1], None), ((1, 2, 3), None), (True, None), (np.int64(3), None),
])
def test_vector_rejected_reset_leaves_entire_pool_unchanged(seed, options):
    vec = vector_class()(3, record_replay=True)
    vec.reset(seed=255)
    diagnostic, tokens = vec.diagnostics(), vec._tokens.copy()
    records = [vec.reproduction(i) for i in range(3)]
    generators = vec.np_random
    states = [deepcopy(g.bit_generator.state) for g in generators]
    with pytest.raises(_native.CombatInputError):
        vec.reset(seed=seed, options=options)
    equal_tree(vec.diagnostics(), diagnostic)
    equal_tree(vec._tokens, tokens)
    equal_tree([vec.reproduction(i) for i in range(3)], records)
    assert all(a is b for a, b in zip(vec.np_random, generators))
    equal_tree([g.bit_generator.state for g in vec.np_random], states)
    assert vec.take_completed_records() == []
    vec.step(np.ones(3, dtype=np.int64))
    vec.close()


def test_vector_partial_scalar_actual_slot_seed_and_scenario_override():
    catalog = [template(id="first"), template(id="second", player_hp=50)]
    vec = vector_class()(3, catalog=catalog, record_replay=True)
    vec.reset(seed=10, options={"scenario_ids": ["first"] * 3})
    before = [vec.reproduction(i) for i in range(3)]
    mask = np.array([False, False, True])[::-1][::-1]
    mask.flags.writeable = False
    options = {"reset_mask": mask, "scenario_ids": ["second", "second", "second"]}
    obs, info = vec.reset(seed=2**300 + 255, options=options)
    assert vec.np_random_seed == (10, 11, 2**300 + 257)
    for i in (0, 1):
        equal_tree(vec.reproduction(i), before[i])
    assert info["scenario_id"][2] == "second"
    assert vec.reproduction(2)["scenario_id"] == "second"
    assert vec.reproduction(2)["ordinal"] == 0
    assert len(vec.take_completed_records()) == 1
    assert vec.observation_space.contains(obs)
    vec.close()


@pytest.mark.parametrize("workers", [1, 2, 4])
def test_vector_mixed_death_victory_manual_jobs_and_worker_equivalence(workers):
    from spire_lab.gymnasium import CombatEnv
    catalog = victory_catalog() + [template(id="death", player_hp=1)]
    scenarios = ["death", "victory", "death"]
    vec = vector_class()(3, workers=workers, catalog=catalog, record_replay=True)
    serial = vector_class()(3, workers=1, catalog=catalog, record_replay=True)
    manuals = [CombatEnv(catalog=catalog, record_replay=True) for _ in range(3)]
    observations, _ = vec.reset(seed=255, options={"scenario_ids": scenarios})
    serial.reset(seed=255, options={"scenario_ids": scenarios})
    for i, env in enumerate(manuals):
        env.reset(seed=255 + i, options={"scenario_id": scenarios[i]})
    deaths = victories = 0
    for _ in range(24):
        # Only play Strike while a current hand/energy offers it; otherwise end
        # turn. All action decisions here are test policy, never production rules.
        actions = np.zeros(3, dtype=np.int64)
        if observations["action_mask"][1, 1]:
            actions[1] = 1
        actual = vec.step(actions)
        equal_tree(actual, serial.step(actions))
        current, rewards, terms, truncs, info = actual
        observations = current
        for i, env in enumerate(manuals):
            obs, reward, term, trunc, end_info = env.step(int(actions[i]))
            assert (reward, term, trunc) == (rewards[i], terms[i], truncs[i])
            if term:
                deaths += reward == -1
                victories += reward == 1
                equal_tree(info["final_obs"][i], obs)
                for key, value in end_info.items():
                    assert info["final_info"][key][i] == value
                obs, end_info = env.reset()
            equal_tree({k: v[i] for k, v in current.items()}, obs)
            for key, value in end_info.items():
                assert info[key][i] == value
        equal_tree(vec.take_completed_records(), serial.take_completed_records())
    assert deaths > 0 and victories > 0
    for env in manuals:
        env.close()
    vec.close()
    serial.close()


def test_vector_supported_statistics_and_info_wrappers():
    from gymnasium.wrappers.vector import DictInfoToList, RecordEpisodeStatistics
    vec = RecordEpisodeStatistics(vector_class()(3, catalog=victory_catalog()))
    vec.reset(seed=255)
    for _ in range(8):
        result = vec.step(np.ones(3, dtype=np.int64))
    np.testing.assert_array_equal(result[4]["episode"]["r"], [1, 1, 1])
    np.testing.assert_array_equal(result[4]["episode"]["l"], [8, 8, 8])
    vec.close()
    vec = DictInfoToList(vector_class()(3, catalog=victory_catalog()))
    vec.reset(seed=255)
    for _ in range(8):
        result = vec.step(np.ones(3, dtype=np.int64))
    for info in result[4]:
        assert info["episode_id"] == "episode-1"
        assert info["final_info"]["episode_id"] == "episode-0"
        assert isinstance(info["final_obs"], dict)
    vec.close()


def test_vector_first_partial_reset_and_lifecycle():
    vec = vector_class()(3)
    with pytest.raises(_native.CombatStateError):
        vec.step(np.zeros(3, dtype=np.int64))
    with pytest.raises(_native.CombatStateError):
        vec.reset(seed=255, options={"reset_mask": np.array([True, False, False])})
    assert all(s["episode_id"] is None for s in vec.diagnostics()["slots"])
    vec.reset(seed=None)
    assert all(type(s) is int and s >= 0 for s in vec.np_random_seed)
    generators = vec.np_random
    vec.reset()
    assert all(a is b for a, b in zip(generators, vec.np_random))
    assert vec.render() is None
    vec.close()
    vec.close()
    assert vec.closed and vec.render() is None
    for operation in (lambda: vec.reset(), lambda: vec.step(np.zeros(3, dtype=np.int64))):
        with pytest.raises(_native.CombatStateError) as closed:
            operation()
        assert closed.value.reason == "closed"


@pytest.mark.parametrize("actions", [
    [1, 1, 1], np.ones(3, dtype=np.int32), np.ones(3, dtype=np.uint64),
    np.ones(3, dtype=float), np.ones(3, dtype=bool), np.ones(3, dtype=">i8"),
    np.ones((3, 1), dtype=np.int64), np.ones(2, dtype=np.int64),
    np.array([1, 1, 31], dtype=np.int64), np.array([1, 1, -1], dtype=np.int64),
])
def test_vector_ownership_batch_rejection_and_fault(actions):
    vec = vector_class()(3, record_replay=True)
    vec.reset(seed=255)
    before, tokens = vec.diagnostics(), vec._tokens.copy()
    with pytest.raises(_native.CombatInputError):
        vec.step(actions)
    equal_tree(vec.diagnostics(), before)
    equal_tree(vec._tokens, tokens)
    assert all(vec.reproduction(i)["actions"].size == 0 for i in range(3))
    obs, _, _, _, info = vec.step(np.ones(6, dtype=np.int64)[::2])
    assert info["episode_steps"].tolist() == [1, 1, 1]
    vec.close()


@pytest.mark.parametrize("workers", [1, 2, 4])
def test_vector_workers_owned_buffers_and_b1_equivalence(workers):
    cls = vector_class()
    vec = cls(3, workers=workers, catalog=victory_catalog())
    reference = cls(3, workers=1, catalog=victory_catalog())
    single = cls(1, catalog=victory_catalog())
    retained = vec.reset(seed=[7, 7, 7])
    reference.reset(seed=[7, 7, 7])
    single.reset(seed=7)
    saved = deepcopy(retained)
    action = np.ones(3, dtype=np.int64)[::-1]
    action.flags.writeable = False
    for _ in range(8):
        for generator in vec.np_random:
            generator.integers(0, 1000, size=13)
        vec.action_space.seed(999)
        actual = vec.step(action)
        equal_tree(actual, reference.step(action))
        one = single.step(np.ones(1, dtype=np.int64))
        for key in actual[0]:
            np.testing.assert_array_equal(actual[0][key][0], one[0][key][0])
        for j in (1, 2, 3):
            assert actual[j][0] == one[j][0]
    equal_tree(retained, saved)
    saved_final = deepcopy(actual)
    current, *_, info = actual
    for key in current:
        assert not np.shares_memory(current[key], info["final_obs"][0][key])
        assert not np.shares_memory(info["final_obs"][0][key], info["final_obs"][1][key])
    vec.step(action)
    reference.step(action)
    equal_tree(actual, saved_final)
    for values in current.values():
        values.fill(0)
    for values in info["final_obs"][0].values():
        values.fill(0)
    for values in info["final_info"].values():
        values.fill(0)
    for values in info.values():
        if isinstance(values, np.ndarray):
            values.fill(None if values.dtype == object else 0)
    descriptor = vec.observation_spec()
    descriptor["tensors"][0]["shape"][0] = 999
    equal_tree(vec.observation_spec(), reference.observation_spec())
    equal_tree(vec.step(action), reference.step(action))
    vec.close()
    reference.close()
    single.close()


@pytest.mark.parametrize("kwargs", [
    {"num_envs": 0}, {"num_envs": True}, {"num_envs": 1.0},
    {"num_envs": 3, "workers": 0}, {"num_envs": 3, "workers": True},
    {"num_envs": 3, "partition": "unknown"}, {"num_envs": 3, "render_mode": "rgb_array"},
    {"num_envs": 3, "record_replay": 1},
])
def test_vector_constructor_strict_native_configuration(kwargs):
    with pytest.raises(_native.CombatInputError):
        vector_class()(**kwargs)


def test_vector_guard_held_through_attached_packaging(monkeypatch):
    from concurrent.futures import ThreadPoolExecutor
    from threading import Event
    module = importlib.import_module("spire_lab.gymnasium.vector")
    vec = module.CombatVectorEnv(3)
    vec.reset(seed=255)
    entered, release = Event(), Event()
    original = module._package_info

    def blocked(*args, **kwargs):
        entered.set()
        assert release.wait(timeout=10)
        return original(*args, **kwargs)

    monkeypatch.setattr(module, "_package_info", blocked)
    with ThreadPoolExecutor(max_workers=1) as executor:
        future = executor.submit(vec.step, np.ones(3, dtype=np.int64))
        try:
            assert entered.wait(timeout=10)
            for call in (lambda: vec.reset(), lambda: vec.step(np.ones(3, dtype=np.int64)),
                         vec.close, vec.render, lambda: vec.np_random,
                         vec._pool.diagnostics, lambda: vec._pool.reproduction(0),
                         vec._pool.take_completed_records, vec._pool.close,
                         lambda: vec._pool.reset([0])):
                with pytest.raises(_native.CombatStateError) as busy:
                    call()
                assert busy.value.reason == "busy"
        finally:
            release.set()
        result = future.result(timeout=10)
    np.testing.assert_array_equal(result[4]["episode_steps"], [1, 1, 1])
    assert vec.diagnostics()["status"] == "active"
    vec.close()


@pytest.mark.parametrize("phase", ["reset", "final"])
def test_vector_packaging_failure_faults_before_facade_release(monkeypatch, phase):
    module = importlib.import_module("spire_lab.gymnasium.vector")
    vec = module.CombatVectorEnv(1, catalog=[template(
        encounter="jaw_worm", enemy_hp=42, player_hp=1)], record_replay=True)
    vec.reset(seed=7)
    # Deliver episode-0 successfully but leave its record in the native queue.
    delivered = vec.step(np.zeros(1, dtype=np.int64))
    assert delivered[2].tolist() == [True]
    assert delivered[4]["final_info"]["episode_id"].tolist() == ["episode-0"]
    if phase == "reset":
        vec.step(np.ones(1, dtype=np.int64))

    def broken(*args, **kwargs):
        with pytest.raises(_native.CombatStateError) as busy:
            vec.reset()
        assert busy.value.reason == "busy"
        with pytest.raises(_native.CombatStateError) as busy:
            vec.close()
        assert busy.value.reason == "busy"
        raise MemoryError("injected packaging failure")

    monkeypatch.setattr(module, "_package_info" if phase == "reset" else "_package_finals", broken)
    with pytest.raises(MemoryError, match="injected"):
        if phase == "reset":
            vec.reset()
        else:
            vec.step(np.zeros(1, dtype=np.int64))
    assert vec.diagnostics()["status"] == "faulted"
    replacement = vec.reproduction(0)
    assert replacement["episode_id"] == "episode-2"
    assert replacement["status"] == "faulted"
    assert replacement["fault"]["stage"] == "export"
    assert replacement["actions"].tolist() == []
    assert replacement["attempted_action"] is None
    earlier, retired = vec.take_completed_records()
    assert earlier["episode_id"] == "episode-0"
    assert earlier["status"] == "completed" and earlier["fault"] is None
    assert earlier["actions"].tolist() == [0]
    assert earlier["attempted_action"] is None
    assert retired["episode_id"] == "episode-1"
    assert retired["status"] == "faulted"
    assert retired["fault"]["stage"] == "export"
    assert retired["actions"].tolist() == ([0] if phase == "final" else [1])
    assert retired["attempted_action"] is None
    assert vec.take_completed_records() == []
    for call in (lambda: vec.reset(), lambda: vec.step(np.zeros(1, dtype=np.int64))):
        with pytest.raises(_native.CombatStateError) as faulted:
            call()
        assert faulted.value.reason == "faulted"
    vec.close()
