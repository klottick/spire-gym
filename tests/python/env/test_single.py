from copy import deepcopy
from concurrent.futures import ThreadPoolExecutor
import subprocess
import sys
import threading

import gymnasium as gym
import numpy as np
import pytest

from spire_lab import _native
from .support import template


def test_single_is_real_env_with_native_spaces():
    from spire_lab.gymnasium import CombatEnv

    env = CombatEnv()
    obs, info = env.reset(seed=0)
    assert isinstance(env, gym.Env)
    assert isinstance(env.action_space, gym.spaces.Discrete)
    assert env.action_space.n == 32
    spec = _native.NativeCombatPool().observation_spec()
    assert list(obs) == [t['key'] for t in spec['tensors']]
    assert env.observation_space.contains(obs)
    for tensor in spec['tensors']:
        value = obs[tensor['key']]
        assert value.shape == tuple(tensor['shape'])
        assert value.dtype == np.dtype(tensor['dtype'])
        assert value.flags.c_contiguous and value.dtype.isnative
    assert obs['hand_features'][:5].tolist() == [
        [1, 6, 0, 0], [1, 0, 5, 0], [1, 6, 0, 0],
        [1, 0, 5, 0], [2, 8, 0, 2]]
    assert info == dict(episode_id='episode-0', scenario_id='starter-cultist-a0-v1',
                        episode_steps=0, outcome=0)
    obs, reward, terminated, truncated, info = env.step(np.int64(1))
    assert env.observation_space.contains(obs)
    assert type(reward) is float and type(terminated) is bool and type(truncated) is bool
    assert (reward, terminated, truncated) == (0., False, False)
    assert info['episode_steps'] == 1
    assert env.metadata['render_modes'] == [] and env.render_mode is None
    assert env.render() is None
    env.close()
    env.close()


def assert_same(left, right):
    if isinstance(left, dict):
        assert left.keys() == right.keys()
        for key in left:
            assert_same(left[key], right[key])
    elif isinstance(left, np.ndarray):
        np.testing.assert_array_equal(left, right)
    elif isinstance(left, (tuple, list)):
        assert len(left) == len(right)
        for a, b in zip(left, right):
            assert_same(a, b)
    else:
        assert left == right


@pytest.mark.parametrize('seed', [0, 2**300 + 7])
def test_single_reset_determinism_and_illegal_actions(seed):
    from spire_lab.gymnasium import CombatEnv

    a, b = CombatEnv(record_replay=True), CombatEnv(record_replay=True)
    assert_same(a.reset(seed=seed), b.reset(seed=seed))
    assert_same(a.np_random.bit_generator.state, b.np_random.bit_generator.state)
    assert a.np_random_seed == seed
    for action in [np.int8(1), np.uint64(0), 1]:
        assert_same(a.step(action), b.step(action))
    a.np_random.integers(0, 100, size=3)
    state = deepcopy(a.np_random.bit_generator.state)
    generator = a.np_random
    continued = a.reset()
    assert continued[1]['episode_id'] == 'episode-1'
    assert a.np_random is generator
    assert_same(state, a.np_random.bit_generator.state)
    a.close()
    b.close()


@pytest.mark.parametrize('action', [True, np.bool_(False), 1.0, np.float64(1),
                                   np.array(1), np.array([1]), -1, 31, 32,
                                   2**100, np.uint64(2**64-1), '1', None])
def test_single_invalid_action_is_atomic(action):
    from spire_lab.gymnasium import CombatEnv

    env = CombatEnv(record_replay=True)
    env.reset(seed=0)
    before = deepcopy(env._pool.diagnostics())
    record = env._pool.reproduction(0)
    state = deepcopy(env.np_random.bit_generator.state)
    with pytest.raises(_native.CombatInputError):
        env.step(action)
    assert_same(before, env._pool.diagnostics())
    assert_same(record, env._pool.reproduction(0))
    assert_same(state, env.np_random.bit_generator.state)
    assert env.step(1)[4]['episode_steps'] == 1
    env.close()


class MalformedIntegerIndex(np.int64):
    def __index__(self):
        raise TypeError('malformed integer index')


@pytest.mark.parametrize('action', [np.timedelta64(1, 'D'), np.timedelta64(0, 'ns'),
    np.timedelta64(-1, 's'), np.timedelta64('NaT', 'ns'), MalformedIntegerIndex(1)])
def test_single_invalid_scalar_dtype_is_typed_and_atomic(action):
    from spire_lab.gymnasium import CombatEnv

    env = CombatEnv(record_replay=True)
    env.reset(seed=0)
    env.np_random.integers(0, 100, size=3)
    before = env.diagnostics()
    record = env.reproduction()
    tokens = env._tokens.copy()
    state = deepcopy(env.np_random.bit_generator.state)
    with pytest.raises(_native.CombatInputError) as exc:
        env.step(action)
    assert exc.value.reason == 'dtype' and exc.value.slot == 0
    assert exc.value.faults == []
    assert_same(before, env.diagnostics())
    assert_same(record, env.reproduction())
    assert_same(tokens, env._tokens)
    assert_same(state, env.np_random.bit_generator.state)
    obs, reward, terminated, truncated, info = env.step(np.int64(1))
    assert obs['enemy'][0, 0] == 46 and obs['player'][3] == 2
    assert (reward, terminated, truncated) == (0., False, False)
    assert info['episode_steps'] == 1
    assert env.reproduction()['actions'].tolist() == [1]
    env.close()


def test_single_integer_action_uses_lossless_index_not_int_coercion():
    from spire_lab.gymnasium import CombatEnv

    class IntegerWithLossyCast(int):
        def __int__(self):
            return 0

    env = CombatEnv()
    env.reset(seed=0)
    obs, _, _, _, info = env.step(IntegerWithLossyCast(1))
    assert obs['enemy'][0, 0] == 46
    assert obs['player'][3] == 2
    assert info['episode_steps'] == 1
    env.close()


@pytest.mark.parametrize('kwargs', [dict(seed=True), dict(seed=-1), dict(seed=1.),
    dict(seed=np.int64(1)), dict(seed=[1]), dict(options=[]),
    dict(options={'unknown': 1}), dict(seed=99, options={'scenario_id': 'owned', 'extra': 1}),
    dict(options={'scenario_id': None}),
    dict(options={'scenario_id': 'missing'}), dict(options={'scenario_id': 1}),
    dict(options={'scenario_id': '\ud800'})])
def test_single_invalid_reset_is_atomic_and_does_not_mutate_options(kwargs):
    from spire_lab.gymnasium import CombatEnv

    env = CombatEnv(record_replay=True)
    env.reset(seed=0)
    saved = deepcopy(kwargs)
    before = env._pool.diagnostics()
    record = env._pool.reproduction(0)
    state = deepcopy(env.np_random.bit_generator.state)
    with pytest.raises(_native.CombatInputError):
        env.reset(**kwargs)
    assert kwargs == saved
    assert_same(before, env._pool.diagnostics())
    assert_same(record, env._pool.reproduction(0))
    assert_same(state, env.np_random.bit_generator.state)
    assert env.reset()[1]['episode_id'] == 'episode-1'
    env.close()


def test_single_lifecycle_and_native_entropy_seed():
    from spire_lab.gymnasium import CombatEnv

    with pytest.raises(_native.CombatInputError):
        CombatEnv(render_mode='human')
    env = CombatEnv(record_replay=True)
    with pytest.raises(_native.CombatStateError) as exc:
        env.step(0)
    assert exc.value.reason == 'uninitialized'
    with pytest.raises(_native.CombatInputError):
        env.reset(seed=-1)
    assert env._np_random is None
    env.reset()
    effective = int.from_bytes(env._pool.reproduction(0)['effective_seed_bytes'], 'big')
    assert env.np_random_seed == effective
    env.close()
    for operation in [lambda: env.step(0), lambda: env.reset(seed=0)]:
        with pytest.raises(_native.CombatStateError) as exc:
            operation()
        assert exc.value.reason == 'closed'


def test_single_compatibility_and_action_sampling_randomness_do_not_drive_gameplay():
    from spire_lab.gymnasium import CombatEnv

    a, b = CombatEnv(), CombatEnv()
    left, right = a.reset(seed=0), b.reset(seed=0)
    assert_same(left, right)
    a.np_random.integers(0, 100, size=100)
    a.action_space.seed(3)
    b.action_space.seed(97)
    states = [deepcopy(env.np_random.bit_generator.state) for env in (a, b)]
    for _ in range(100):
        # EndTurn exercises native draw/shuffle/AI randomness without Python RNG.
        left, right = a.step(0), b.step(np.uint8(0))
        assert_same(left, right)
        if left[2]:
            break
    assert left[2] and not left[3]
    for env, state in zip((a, b), states):
        assert_same(state, env.np_random.bit_generator.state)
        env.close()


def test_single_valid_options_descriptor_and_privileged_record_ownership():
    from spire_lab.gymnasium import CombatEnv

    env = CombatEnv(record_replay=True)
    options = {'scenario_id': 'starter-jaw-worm-a0-v1'}
    obs, info = env.reset(seed=0, options=options)
    assert options == {'scenario_id': 'starter-jaw-worm-a0-v1'}
    assert info['scenario_id'] == options['scenario_id']
    spec = env.observation_spec()
    assert spec['fingerprint'] == '8fd66d212259d2bdd27139988a772e1a6c223f8d65524f38c74174a0d0843357'
    spec['tensors'][0]['shape'][0] = 999
    assert env.observation_spec()['tensors'][0]['shape'] == [7]
    env.step(0)
    record = env.reproduction()
    record['actions'][:] = 99
    assert env.reproduction()['actions'].tolist() == [0]
    env.reset()
    records = env.take_completed_records()
    assert len(records) == 1 and records[0]['status'] == 'abandoned'
    assert records[0]['actions'].tolist() == [0]
    assert env.take_completed_records() == []
    assert env.diagnostics()['status'] == 'active'
    env.close()
    plain = CombatEnv()
    plain.reset(seed=0)
    assert plain.reproduction() is None and plain.take_completed_records() == []
    plain.close()


def test_single_standard_statistics_wrapper_with_masked_actions():
    from spire_lab.gymnasium import CombatEnv

    env = gym.wrappers.RecordEpisodeStatistics(CombatEnv(catalog=[template(player_hp=1)]))
    obs, _ = env.reset(seed=0)
    env.action_space.seed(123)
    for _ in range(100):
        obs, _, terminated, truncated, info = env.step(env.action_space.sample(mask=obs['action_mask']))
        if terminated or truncated:
            break
    assert terminated and not truncated
    assert info['episode']['r'] == -1.
    assert info['episode']['l'] == info['episode_steps']
    env.close()


@pytest.mark.parametrize('win', [True, False])
def test_single_terminal_ownership(win):
    from spire_lab.gymnasium import CombatEnv

    env = CombatEnv(catalog=[template(enemy_hp=48, player_hp=70, energy_per_turn=8,
                             hand=[dict(kind='strike', upgraded=False) for _ in range(8)]) if win
                             else template(player_hp=1)], record_replay=True)
    initial, initial_info = env.reset(seed=0, options={'scenario_id': 'owned'})
    saved = deepcopy((initial, initial_info))
    if win:
        for _ in range(8):
            result = env.step(1)
    else:
        result = env.step(0)
        assert result[1:4] == (0., False, False)
        result = env.step(0)
    obs, reward, terminated, truncated, info = result
    assert (reward, terminated, truncated) == ((1., True, False) if win else (-1., True, False))
    assert info == dict(episode_id='episode-0', scenario_id='owned',
                        episode_steps=8 if win else 2, outcome=1 if win else 2,
                        hp_before_healing=70 if win else 0,
                        hp_after_cleanup=76 if win else 0)
    assert not obs['action_mask'].any()
    assert env.observation_space.contains(obs)
    if win:
        assert obs['in_play_counts'][0, 0] == 1
        assert obs['enemy'][0, 0] == 0
    with pytest.raises(_native.CombatStateError) as exc:
        env.step(0)
    assert exc.value.reason == 'terminal'
    assert_same(saved, (initial, initial_info))
    terminal_saved = deepcopy(result)
    initial['player'][:] = -99
    initial_info['episode_id'] = 'changed'
    replacement, _ = env.reset(seed=0)
    assert not np.shares_memory(obs['player'], replacement['player'])
    replacement['player'][:] = -55
    env.close()
    assert_same(terminal_saved, result)
    assert initial['player'][0] == -99


class UnprintableExportError(LookupError):
    def __str__(self):
        raise RuntimeError('cannot format exception')


@pytest.mark.parametrize('failure', [LookupError('focused packaging failure'),
                                   LookupError('\ud800'), UnprintableExportError()])
@pytest.mark.parametrize('operation', ['reset', 'step'])
def test_single_packaging_failure_faults_before_guard_release(monkeypatch, operation, failure):
    from spire_lab.gymnasium import CombatEnv, _common

    env = CombatEnv(record_replay=True)
    if operation == 'step':
        env.reset(seed=0)
    original = _common.package_row
    def fail(*args):
        with pytest.raises(_native.CombatStateError) as exc:
            env.reset(seed=99)
        assert exc.value.reason == 'busy'
        raise failure
    monkeypatch.setattr(_common, 'package_row', fail)
    with pytest.raises(type(failure)) as caught:
        env.reset(seed=0) if operation == 'reset' else env.step(1)
    assert caught.value is failure
    monkeypatch.setattr(_common, 'package_row', original)
    diagnostic = env._pool.diagnostics()
    assert diagnostic['status'] == 'faulted'
    assert diagnostic['faults'][0]['stage'] == 'export'
    assert 'facade export' in diagnostic['faults'][0]['message']
    for call in [lambda: env.step(1), lambda: env.reset(seed=0)]:
        with pytest.raises(_native.CombatStateError) as exc:
            call()
        assert exc.value.reason == 'faulted'
    record = env._pool.reproduction(0)
    assert record['actions'].tolist() == ([1] if operation == 'step' else [])
    assert record['status'] == 'faulted'
    env.close()


def test_single_guard_spans_packaging_in_other_thread(monkeypatch):
    from spire_lab.gymnasium import CombatEnv, _common

    env = CombatEnv()
    env.reset(seed=0)
    entered, release = threading.Event(), threading.Event()
    original = _common.package_row
    def pause(*args):
        entered.set()
        assert release.wait(5)
        return original(*args)
    monkeypatch.setattr(_common, 'package_row', pause)
    with ThreadPoolExecutor(1) as executor:
        future = executor.submit(env.step, 1)
        try:
            assert entered.wait(5)
            for call in [lambda: env.step(0), lambda: env.reset(), env.close]:
                with pytest.raises(_native.CombatStateError) as exc:
                    call()
                assert exc.value.reason == 'busy'
        finally:
            release.set()
        assert future.result()[4]['episode_steps'] == 1
    env.close()


def test_single_upstream_reset_space_and_passive_masked_checks():
    from gymnasium.utils import env_checker as checks
    from gymnasium.utils.passive_env_checker import (check_action_space,
        check_observation_space, env_reset_passive_checker, env_step_passive_checker)
    from spire_lab.gymnasium import CombatEnv

    env = CombatEnv()
    for check in [checks.check_reset_seed_determinism, checks.check_reset_options,
                  checks.check_reset_return_info_deprecation, checks.check_seed_deprecation,
                  checks.check_reset_return_type]:
        check(env)
    check_action_space(env.action_space)
    # The native context descriptor intentionally fixes Ascension to zero.
    with pytest.warns(UserWarning, match='maximum and minimum values are equal'):
        check_observation_space(env.observation_space)
    checks.check_space_limit(env.action_space, 'action')
    checks.check_space_limit(env.observation_space, 'observation')
    obs, _ = env_reset_passive_checker(env, seed=0, options={})
    env.action_space.seed(123)
    for _ in range(100):
        action = env.action_space.sample(mask=obs['action_mask'])
        obs, reward, terminated, truncated, _ = env_step_passive_checker(env, action)
        assert env.observation_space.contains(obs)
        if terminated or truncated:
            break
    assert terminated and not truncated
    env.close()


def test_single_optional_boundary_and_spaces_are_descriptor_derived():
    from spire_lab.gymnasium.spaces import spaces_from_spec

    subprocess.run([sys.executable, '-B', '-c', '''
import sys
import importlib.abc
import spire_lab
from spire_lab import _native
_native.NativeCombatPool().observation_spec()
assert 'gymnasium' not in sys.modules
assert 'spire_lab.gymnasium' not in sys.modules
class MissingGymnasium(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname == 'gymnasium':
            raise ModuleNotFoundError('optional gymnasium is absent', name=fullname)
sys.meta_path.insert(0, MissingGymnasium())
try:
    import spire_lab.gymnasium
except ModuleNotFoundError as error:
    assert error.name == 'gymnasium'
else:
    raise AssertionError('optional dependency must not have a fallback')
'''], check=True)
    spec = _native.NativeCombatPool().observation_spec()
    # Alter the input descriptor to catch a duplicated Python feature table.
    spec['tensors'] = [dict(spec['tensors'][0], key='renamed', shape=[2], low=-3, high=7),
                       dict(spec['tensors'][3], shape=[2, 3])]
    spec['action_space']['n'] = 17
    obs_space, action_space = spaces_from_spec(spec)
    assert list(obs_space.spaces) == ['renamed', 'hand_upgraded']
    assert obs_space['renamed'].shape == (2,)
    assert obs_space['renamed'].low.tolist() == [-3, -3]
    assert obs_space['renamed'].high.tolist() == [7, 7]
    assert isinstance(obs_space['hand_upgraded'], gym.spaces.MultiBinary)
    assert obs_space['hand_upgraded'].shape == (2, 3)
    assert action_space.n == 17
