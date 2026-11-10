"""M34c Python surface: exact checkpoints (privileged) and replay to a decision."""

import copy
import pickle

import gymnasium as gym
import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import RunEnv


def pick(observation, step):
    legal = np.flatnonzero(observation["action_mask"])
    return int(legal[(step * 7 + 3) % len(legal)])


def advance(game, observation, steps, salt=0):
    trace = []
    for step in range(steps):
        observation, reward, terminated, truncated, info = game.step(pick(observation, step + salt))
        trace.append((observation, reward, terminated, truncated, info["episode_return"]))
        if terminated:
            break
    return observation, trace


def same_trace(a, b):
    assert len(a) == len(b) and a
    for (oa, *ra), (ob, *rb) in zip(a, b):
        assert ra == rb
        assert oa.keys() == ob.keys()
        for key in oa:
            np.testing.assert_array_equal(oa[key], ob[key])


def same_observation(a, b):
    assert a.keys() == b.keys()
    for key in a:
        np.testing.assert_array_equal(a[key], b[key])


def standard(character="silent", ascension=0):
    return RunEnv(character, ascension, profile="standard", partition="test", record_replay=True)


@pytest.mark.parametrize("character,ascension", [("ironclad", 0), ("watcher", 20)])
def test_restore_is_deterministic(character, ascension):
    game = standard(character, ascension)
    observation, _ = game.reset(seed=41)
    observation, _ = advance(game, observation, 10)
    checkpoint = game.unwrapped.exact_checkpoint()
    _, first = advance(game, observation, 60, salt=5)
    restored, info = game.unwrapped.exact_restore(checkpoint)
    same_observation(restored, observation)
    assert info["episode_return"] == first[0][4] - first[0][1]
    _, second = advance(game, restored, 60, salt=5)
    same_trace(first, second)


def test_restore_makes_old_tokens_stale():
    session = _native.NativeRunSession("defect", 0, profile="standard", partition="test")
    boundary = session.reset(5)
    checkpoint = session.exact_checkpoint()
    index = pick(boundary["obs"], 0)
    later = session.step(index, boundary["token"])
    restored = session.exact_restore(checkpoint)
    for stale in (boundary["token"], later["token"]):
        with pytest.raises(_native.RunStateError):
            session.step(pick(restored["obs"], 0), stale)
    assert session.decision()["token"] == restored["token"]
    session.step(pick(restored["obs"], 0), restored["token"])


def test_fork_is_isolated_from_source_and_siblings():
    game = standard("watcher")
    observation, _ = game.reset(seed=63)
    observation, _ = advance(game, observation, 8)
    checkpoint = game.unwrapped.exact_checkpoint()
    a = game.unwrapped.exact_fork(checkpoint)
    b = game.unwrapped.exact_fork(checkpoint)
    a_start, _ = a.decision()
    _, a_trace = advance(a, a_start, 25, salt=1)
    advance(game, observation, 25, salt=2)
    b_start, _ = b.decision()
    same_observation(b_start, observation)
    _, b_trace = advance(b, b_start, 25, salt=1)
    same_trace(a_trace, b_trace)
    game.close()
    c = game.unwrapped.exact_fork(checkpoint)
    same_observation(c.decision()[0], observation)


def test_checkpoint_is_opaque_unpicklable_and_identity_equal():
    session = _native.NativeRunSession("ironclad", 0, profile="standard", partition="test")
    session.reset(123456789)
    first = session.exact_checkpoint()
    second = session.exact_checkpoint()
    assert first == first and first != second
    assert len({first, second}) == 2
    text = repr(first)
    assert "redacted" in text and "123456789" not in text
    assert not [name for name in dir(first) if not name.startswith("__")]
    for attempt in (pickle.dumps, copy.copy, copy.deepcopy):
        with pytest.raises(TypeError):
            attempt(first)


def test_exact_tools_are_reached_through_unwrapped_only():
    wrapped = gym.wrappers.RecordEpisodeStatistics(standard())
    wrapped.reset(seed=3)
    for name in ("exact_checkpoint", "exact_restore", "exact_fork"):
        assert not hasattr(gym.Env, name)
        with pytest.raises(AttributeError):
            getattr(wrapped, name)
    checkpoint = wrapped.unwrapped.exact_checkpoint()
    wrapped.unwrapped.exact_restore(checkpoint)


def test_restore_rejects_other_configuration():
    session = _native.NativeRunSession("ironclad", 0, profile="standard", partition="test")
    session.reset(1)
    checkpoint = session.exact_checkpoint()
    other = _native.NativeRunSession("silent", 0, profile="standard", partition="test")
    other.reset(1)
    with pytest.raises(_native.RunInputError):
        other.exact_restore(checkpoint)
    captured = _native.NativeRunSession("ironclad", 0, profile="m10-ironclad-heart", partition="test")
    captured.reset(1)
    with pytest.raises(_native.RunInputError):
        captured.exact_restore(checkpoint)
    session.close()
    with pytest.raises(_native.RunStateError):
        session.exact_restore(checkpoint)
    with pytest.raises(_native.RunStateError):
        session.exact_checkpoint()


def test_from_record_matches_live_observations():
    game = standard("silent")
    observation, _ = game.reset(seed=404)
    seen = [observation]
    for step in range(60):
        observation, _, terminated, _, _ = game.step(pick(observation, step))
        seen.append(observation)
        if terminated:
            break
    record = game.record()
    count = len(seen) - 1
    for k in (0, 1, count // 2, count):
        replayed = RunEnv.from_record(record, k)
        current, info = replayed.decision()
        same_observation(current, seen[k])
        if k < count:
            nxt, *_ = replayed.step(pick(current, k))
            same_observation(nxt, seen[k + 1])
    with pytest.raises(_native.RunInputError):
        RunEnv.from_record(record, count + 1)
    with pytest.raises(_native.RunInputError):
        RunEnv.from_record(record, True)
