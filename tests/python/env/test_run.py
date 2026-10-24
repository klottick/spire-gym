"""Real rebuilt Run bindings/Gym transport; no strategy-strength requirement."""

import json

import numpy as np
import pytest

from spire_lab import _native
from spire_lab import gymnasium as adapters


def native(**kwargs):
    return _native.NativeRunSession("ironclad", 0, partition="test", **kwargs)


def env(**kwargs):
    return adapters.RunEnv("ironclad", 0, partition="test", **kwargs)


def test_run_native_descriptor_drives_fixed_owned_numeric_spaces():
    game = env()
    spec = game.observation_spec()
    assert spec["schema_id"] == "spire-gym-run-observation-v1"
    assert spec["schema_id"] != adapters.CombatEnv().observation_spec()["schema_id"]
    observation, info = game.reset(seed=11)
    assert game.observation_space.contains(observation)
    assert game.action_space.n == 2048
    assert observation["player"][:3].tolist() == [80, 80, 99]
    assert observation["permanent_cards_occupied"].sum() == 10
    assert observation["context"][8:11].tolist() == [0, 0, 0]
    assert observation["action_mask"].sum() == 2
    assert info == {"schema_id": spec["schema_id"], "terminal_reason": None}
    for tensor in spec["tensors"]:
        array = observation[tensor["key"]]
        assert array.dtype == np.dtype(tensor["dtype"])
        assert array.shape == tuple(tensor["shape"])
        assert array.flags.c_contiguous and array.dtype.isnative
    original = {key: value.copy() for key, value in observation.items()}
    game.step(np.int64(0))
    assert all(
        np.array_equal(observation[key], value) for key, value in original.items()
    )
    assert "seed" not in info and "token" not in info and "boss_history" not in info
    game.close()
    assert all(
        np.array_equal(observation[key], value) for key, value in original.items()
    )


@pytest.mark.parametrize("seed", [True, -1, 2**64, np.int64(7), 1.5])
def test_run_seed_validation_is_atomic(seed):
    game = native(record_replay=True)
    game.reset(seed=11)
    before = game.record()
    with pytest.raises(ValueError):
        game.reset(seed=seed)
    assert game.record() == before


def test_run_native_tokens_and_invalid_indices_do_not_advance():
    game = native(record_replay=True)
    start = game.reset(seed=11)
    other = native().reset(seed=11)
    before = game.record()
    for index, token in [
        (-1, start["token"]),
        (2, start["token"]),
        (0, other["token"]),
    ]:
        with pytest.raises((ValueError, RuntimeError)):
            game.step(index, token)
        assert game.record() == before
    next_step = game.step(0, start["token"])
    with pytest.raises(RuntimeError):
        game.step(0, start["token"])
    assert next_step["reward"] == 0
    assert not next_step["terminated"] and not next_step["truncated"]
    assert len(json.loads(game.record())["steps"]) == 1


@pytest.mark.parametrize(
    "action", [True, np.bool_(False), 1.5, np.array([0]), -1, 2**63]
)
def test_run_gym_rejects_non_integer_or_unoffered_actions_without_mutation(action):
    game = env(record_replay=True)
    game.reset(seed=11)
    before = game.record()
    with pytest.raises(ValueError):
        game.step(action)
    assert game.record() == before


def test_run_constructor_and_reset_options_cannot_inject_resources():
    cls = adapters.RunEnv
    with pytest.raises(TypeError):
        cls()  # ty: ignore[missing-argument] - intentional runtime rejection
    for character, ascension in [("silent", 0), ("ironclad", 1), ("ironclad", True)]:
        with pytest.raises(ValueError):
            cls(character, ascension)
    with pytest.raises(TypeError):
        cls("ironclad", 0, deck=[])  # ty: ignore[unknown-argument] - no injection API
    game = env(record_replay=True)
    game.reset(seed=11)
    before = game.record()
    for options in [{"hp": 999}, {"keys": [True] * 3}, {"scenario_id": "boss"}, False]:
        with pytest.raises(ValueError):
            game.reset(options=options)
        assert game.record() == before


def test_run_natural_death_is_not_truncated_and_native_record_replays():
    game = env(record_replay=True)
    observation, _ = game.reset(seed=11)
    spec = game.observation_spec()
    end_turn = spec["vocabularies"]["verb"].index("end_turn")
    total = 0
    for _ in range(256):
        legal = np.flatnonzero(observation["action_mask"])
        turns = [int(i) for i in legal if observation["actions"][i, 0] == end_turn]
        action = turns[0] if turns else int(legal[0])
        observation, reward, terminated, truncated, info = game.step(action)
        assert game.observation_space.contains(observation)
        assert not truncated
        total += reward
        if terminated:
            break
    else:
        pytest.fail("nonattacking diagnostic did not reach an actual native outcome")
    assert info["terminal_reason"] == "death" and reward == -1
    assert not observation["action_mask"].any()
    assert observation["player"][0] == 0
    with pytest.raises(RuntimeError):
        game.step(0)
    record = game.record()
    assert record is not None
    replay = _native.NativeRunSession.replay(record)
    assert replay["terminal_reason"] == "death"
    assert replay["reported_fault"] is None
    assert sum(replay["rewards"]) == total
    assert all(
        np.array_equal(observation[key], value) for key, value in replay["obs"].items()
    )


def test_run_reset_reseeds_or_continues_native_ordinal():
    game = env(record_replay=True)
    game.reset()
    assert (
        json.loads(game.record())["seed"],
        json.loads(game.record())["ordinal"],
    ) == (0, 0)
    game.reset()
    assert (
        json.loads(game.record())["seed"],
        json.loads(game.record())["ordinal"],
    ) == (0, 1)
    first, _ = game.reset(seed=7)
    second, _ = game.reset(seed=7)
    assert all(np.array_equal(first[key], value) for key, value in second.items())
    assert game.np_random_seed == 7
