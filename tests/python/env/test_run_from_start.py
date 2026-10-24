"""Core -> session -> fresh binding -> Gym agreement, including actual losses."""

import json
import os
import subprocess

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import RunEnv


@pytest.fixture(scope="module")
def native_cases():
    environment = {
        **os.environ,
        "CARGO_TARGET_DIR": "target/m9-native-20261023",
        "CARGO_BUILD_JOBS": "2",
    }
    raw = subprocess.check_output(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "spire-env",
            "--example",
            "m9_gym_agreement",
            "--locked",
            "--offline",
        ],
        env=environment,
    )
    return json.loads(raw)["cases"]


@pytest.mark.parametrize("case_index", range(6))
def test_run_from_start_every_boundary_matches_native_core_and_semantic_record(
    native_cases, case_index
):
    assert len(native_cases) == 6, "expected real seed7/11 x three history traces"
    case = native_cases[case_index]
    game = RunEnv(
        "ironclad",
        0,
        partition="test",
        boss_history=case["history"],
        record_replay=True,
    )
    direct = _native.NativeRunSession(
        "ironclad",
        0,
        partition="test",
        boss_history=case["history"],
        record_replay=True,
    )
    observation, info = game.reset(seed=case["seed"])
    raw = direct.reset(seed=case["seed"])
    spec = game.observation_spec()
    for action, expected in zip(case["indices"], case["record"]["steps"], strict=True):
        assert observation["action_mask"][action] == 1
        raw = direct.step(action, raw["token"])
        observation, reward, terminated, truncated, info = game.step(action)
        assert all(
            np.array_equal(observation[key], value) for key, value in raw["obs"].items()
        )
        assert game.observation_space.contains(observation)
        assert reward == raw["reward"] == expected["reward"]
        assert terminated == raw["terminated"] == (expected["terminal"] is not None)
        assert not truncated and not raw["truncated"]
    record = game.record()
    assert record is not None
    direct_record = direct.record()
    assert direct_record is not None
    assert json.loads(record) == json.loads(direct_record) == case["record"]
    replay = _native.NativeRunSession.replay(record)
    assert all(
        np.array_equal(observation[key], value) for key, value in replay["obs"].items()
    )
    for key, values in case["native_final"].items():
        assert observation[key].ravel().tolist() == values
    for key, maximum in case["high_water"].items():
        assert maximum <= spec["capacities"][key]
    assert info["terminal_reason"] == case["record"]["terminal"]
    # This is actual gameplay evidence, not an in-repo strategy-strength gate.
    assert info["terminal_reason"] in {"death", "act3_no_keys"}
    assert not observation["action_mask"].any()
