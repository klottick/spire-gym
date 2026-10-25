"""M10 Run profile opt-in at the native and thin Gym boundaries."""

import json
from typing import Any

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import RunEnv

M9_RECORD_IDENTITY = (
    "spire-gym-run-replay-v1",
    "M9IroncladAct3",
    "spire-gym-native-m9-rules-v2",
    "spire-gym-m9-ironclad-a0-captured-v1",
)
M10_RECORD_IDENTITY = (
    "spire-gym-run-replay-v2",
    "M10IroncladHeart",
    "spire-gym-native-m10-rules-v1",
    "spire-gym-m10-ironclad-a0-captured-v1",
)


def _run(api: str, **kwargs: Any):
    options: dict[str, Any] = {"partition": "test", "record_replay": True}
    options.update(kwargs)
    if api == "native":
        return _native.NativeRunSession("ironclad", 0, **options)
    return RunEnv("ironclad", 0, **options)


def _reset(game, api: str):
    result = game.reset(seed=11)
    if api == "native":
        return result["obs"], result["info"], result["token"]
    observation, info = result
    return observation, info, None


def _record_identity(game):
    record = json.loads(game.record())
    return (
        record["schema_id"],
        record["profile_id"],
        record["rules_id"],
        record["content_id"],
    ), record


def _assert_public_info(info, schema_id):
    assert set(info) == {"schema_id", "terminal_reason"}
    assert info["schema_id"] == schema_id


@pytest.mark.parametrize("api", ["native", "gym"])
def test_run_profile_defaults_to_the_unchanged_m9_v1_identity(api):
    game = _run(api)
    try:
        spec = game.observation_spec()
        assert spec["schema_id"] == "spire-gym-run-observation-v1"
        _, info, _ = _reset(game, api)
        _assert_public_info(info, spec["schema_id"])
        identity, _ = _record_identity(game)
        assert identity == M9_RECORD_IDENTITY
    finally:
        game.close()


@pytest.mark.parametrize("api", ["native", "gym"])
def test_explicit_m10_profile_owns_v2_reset_spaces_and_record_identity(api):
    game = _run(api, profile="m10-ironclad-heart")
    try:
        spec = game.observation_spec()
        assert spec["schema_id"] == "spire-gym-run-observation-v2"
        assert spec["profile_id"] == "M10IroncladHeart"
        assert "confirm_selection" in spec["vocabularies"]["verb"]

        observation, info, token = _reset(game, api)
        _assert_public_info(info, spec["schema_id"])
        assert set(observation) == {tensor["key"] for tensor in spec["tensors"]}
        assert observation["player"][:3].tolist() == [80, 80, 99]
        assert observation["permanent_cards_occupied"].sum() == 10
        assert observation["context"][8:11].tolist() == [0, 0, 0]
        for tensor in spec["tensors"]:
            array = observation[tensor["key"]]
            assert array.dtype == np.dtype(tensor["dtype"])
            assert array.shape == tuple(tensor["shape"])
            assert array.flags.c_contiguous and array.dtype.isnative
            known_key = tensor.get("known")
            if known_key is not None:
                known = observation[known_key]
                assert known.dtype == np.dtype("int8")
                assert known.shape == array.shape
                assert np.isin(known, [0, 1]).all()
            if tensor["space"] == "MultiBinary":
                assert np.isin(array, [0, 1]).all()
        assert observation["action_mask"].any()
        if api == "gym":
            assert game.observation_space.contains(observation)
            assert game.action_space.n == spec["action_space"]["n"]

        # Act only on an index offered by this actual native M10 boundary.
        action = int(np.flatnonzero(observation["action_mask"])[0])
        if api == "native":
            stepped = game.step(action, token)
            next_observation = stepped["obs"]
            reward = stepped["reward"]
            terminated = stepped["terminated"]
            truncated = stepped["truncated"]
            next_info = stepped["info"]
        else:
            next_observation, reward, terminated, truncated, next_info = game.step(
                action
            )
        assert isinstance(reward, (int, float))
        assert not terminated and not truncated
        _assert_public_info(next_info, spec["schema_id"])

        identity, record = _record_identity(game)
        assert identity == M10_RECORD_IDENTITY
        assert bytes(record["observation_fingerprint"]).hex() == spec["fingerprint"]
        assert len(record["steps"]) == 1

        # Static replay dispatches the genuine M10 v2 record to the same native
        # owner and produces the same final public numeric boundary.
        replay = _native.NativeRunSession.replay(game.record())
        assert replay["accepted_count"] == 1
        assert replay["reported_fault"] is None
        assert replay["terminal_reason"] is None
        assert set(replay["obs"]) == set(next_observation)
        assert all(
            np.array_equal(next_observation[key], value)
            for key, value in replay["obs"].items()
        )
    finally:
        game.close()


@pytest.mark.parametrize("api", ["native", "gym"])
@pytest.mark.parametrize("profile", ["unknown-profile", None])
def test_run_rejects_unsupported_profile_before_creating_a_session(api, profile):
    with pytest.raises(ValueError):
        _run(api, profile=profile)


@pytest.mark.parametrize("api", ["native", "gym"])
@pytest.mark.parametrize(
    "unsupported",
    [
        {"keys": [True, True, True]},
        {"hp": 1},
        {"deck": []},
    ],
)
def test_m10_constructor_does_not_accept_injected_run_resources(api, unsupported):
    with pytest.raises(TypeError):
        _run(api, profile="m10-ironclad-heart", **unsupported)


@pytest.mark.parametrize("api", ["native", "gym"])
@pytest.mark.parametrize("seed", [True, np.bool_(False), np.int64(7), 1.5])
def test_m10_reset_requires_an_exact_non_bool_integer_and_is_atomic(api, seed):
    game = _run(api, profile="m10-ironclad-heart")
    try:
        _reset(game, api)
        before = game.record()
        with pytest.raises(ValueError):
            game.reset(seed=seed)
        assert game.record() == before
    finally:
        game.close()


@pytest.mark.parametrize("api", ["native", "gym"])
@pytest.mark.parametrize("action", [True, np.bool_(False), 1.5, np.array([0])])
def test_m10_action_inputs_reject_bool_and_non_scalars_without_advancing(api, action):
    game = _run(api, profile="m10-ironclad-heart")
    try:
        _, _, token = _reset(game, api)
        before = game.record()
        with pytest.raises(ValueError):
            if api == "native":
                game.step(action, token)
            else:
                game.step(action)
        assert game.record() == before
    finally:
        game.close()


@pytest.mark.parametrize("api", ["native", "gym"])
def test_m10_recording_disabled_remains_none(api):
    game = _run(api, record_replay=False, profile="m10-ironclad-heart")
    try:
        _reset(game, api)
        assert game.record() is None
    finally:
        game.close()


def test_m10_reset_has_no_profile_or_resource_override():
    native = _run("native", profile="m10-ironclad-heart")
    gym = _run("gym", profile="m10-ironclad-heart")
    try:
        native.reset(seed=11)
        native_before = native.record()
        with pytest.raises(TypeError):
            native.reset(seed=7, profile="m9-ironclad-act3")
        assert native.record() == native_before

        gym.reset(seed=11)
        gym_before = gym.record()
        for options in [
            {"profile": "m9-ironclad-act3"},
            {"keys": [True, True, True]},
            {"hp": 1},
            {"deck": []},
        ]:
            with pytest.raises(ValueError):
                gym.reset(options=options)
            assert gym.record() == gym_before
    finally:
        native.close()
        gym.close()
