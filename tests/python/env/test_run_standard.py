"""Python transport contract for the native Standard Run profile."""

import random

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import RunEnv


@pytest.mark.parametrize("character", ["ironclad", "silent", "defect", "watcher"])
@pytest.mark.parametrize("ascension", [0, 20])
def test_standard_constructs_every_character_and_endpoint_ascension(character, ascension):
    game = RunEnv(character, ascension, profile="standard", partition="test")
    try:
        spec = game.observation_spec()
        assert spec["schema_id"] == "spire-gym-standard-run-observation-v1"
        assert spec["profile_id"] == "Standard"
        observation, info = game.reset(seed=321)
        assert info["schema_id"] == spec["schema_id"]
        assert info["creation"]["neow"] == "full"
        assert info["creation"]["stored_note"] == {"card": "iron_wave", "upgrades": 0}
        assert game.observation_space.contains(observation)
        assert game.action_space.n == spec["action_space"]["n"]
        assert set(observation) == {tensor["key"] for tensor in spec["tensors"]}
        for tensor in spec["tensors"]:
            value = observation[tensor["key"]]
            assert value.dtype == np.dtype(tensor["dtype"])
            assert value.shape == tuple(tensor["shape"])
            assert value.flags.c_contiguous and value.dtype.isnative
    finally:
        game.close()


@pytest.mark.parametrize(
    "creation",
    [
        {"unknown": True},
        {"neow": "free relic"},
        {"seen_bosses": [["Guardian"], ["Guardian"], []]},
        {"note": {"card": "not_a_real_card", "upgrades": 0}},
        {"note": {"card": "iron_wave", "upgrades": 5}},
        {"note": {"card": "iron_wave", "upgrades": 0, "extra": 1}},
    ],
)
def test_standard_rejects_invalid_creation(creation):
    with pytest.raises(_native.RunInputError):
        _native.NativeRunSession(
            "ironclad", 0, profile="standard", creation=creation
        )


def test_standard_rejects_captured_creation_and_boss_history():
    with pytest.raises(_native.RunInputError):
        _native.NativeRunSession("ironclad", 0, creation={})
    with pytest.raises(_native.RunInputError):
        _native.NativeRunSession(
            "ironclad", 0, profile="standard", boss_history="awakened-unseen"
        )


@pytest.mark.parametrize("character", ["ironclad", "silent", "defect", "watcher"])
@pytest.mark.parametrize("ascension", [0, 20])
def test_standard_random_legal_rollout_info_arrays_and_record_replay(character, ascension):
    game = RunEnv(
        character, ascension, profile="standard", partition="test", record_replay=True,
        creation={"neow": "short", "note": {"card": "strike_g", "upgrades": 0}},
    )
    try:
        seed = 704 + ascension + ["ironclad", "silent", "defect", "watcher"].index(character) * 37
        observation, reset_info = game.reset(seed=seed)
        assert reset_info["creation"]["neow"] == "short"
        before = {key: value.copy() for key, value in observation.items()}
        rng = random.Random(seed)
        episode_return = 0
        for _ in range(300):
            legal = np.flatnonzero(observation["action_mask"])
            assert len(legal) > 0
            observation, reward, terminated, truncated, info = game.step(
                int(rng.choice(legal))
            )
            assert not truncated
            assert isinstance(info["reward_events"], list)
            assert sum(item["amount"] for item in info["reward_events"]) == reward
            episode_return += reward
            assert info["episode_return"] == episode_return
            assert int(observation["context"][7]) == episode_return
            assert info["heart_victory"] is (info["terminal_reason"] == "heart_victory")
            assert all(
                np.array_equal(before[key], value) for key, value in before.items()
            )
            before = {key: value.copy() for key, value in observation.items()}
            if terminated:
                break

        record = game.record()
        assert isinstance(record, bytes)
        replay = _native.NativeRunSession.replay(record)
        assert replay["reported_fault"] is None
        assert set(replay["obs"]) == set(observation)
        assert all(
            np.array_equal(observation[key], value)
            for key, value in replay["obs"].items()
        )
    finally:
        game.close()


@pytest.mark.parametrize("ascension", [0, 20])
def test_standard_replay_threads_record_ascension(ascension):
    game = RunEnv(
        "ironclad", ascension, profile="standard", partition="test", record_replay=True
    )
    try:
        observation, _ = game.reset(seed=1900 + ascension)
        legal = np.flatnonzero(observation["action_mask"])
        observation, _, _, _, _ = game.step(int(legal[0]))
        replay = _native.NativeRunSession.replay(game.record())
        assert replay["reported_fault"] is None
        assert int(replay["obs"]["context"][1]) == ascension
        assert all(
            np.array_equal(observation[key], value)
            for key, value in replay["obs"].items()
        )
    finally:
        game.close()
