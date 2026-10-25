"""M10 profile retains the same native delivery and ownership guarantees."""

import json
from concurrent.futures import ThreadPoolExecutor

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import RunEnv


def _native_m10():
    return _native.NativeRunSession(
        "ironclad",
        0,
        partition="test",
        profile="m10-ironclad-heart",
        record_replay=True,
    )


def _gym_m10():
    return RunEnv(
        "ironclad",
        0,
        partition="test",
        profile="m10-ironclad-heart",
        record_replay=True,
    )


@pytest.mark.parametrize("api", ["native", "gym"])
def test_m10_returned_arrays_info_and_records_survive_step_reset_and_close(api):
    game = _native_m10() if api == "native" else _gym_m10()
    try:
        if api == "native":
            first = game.reset(seed=11)
            observation, info = first["obs"], first["info"]
            token = first["token"]
        else:
            observation, info = game.reset(seed=11)
            token = None
            assert game.np_random_seed == 11
        retained = {key: value.copy() for key, value in observation.items()}
        retained_info = dict(info)
        initial_record = game.record()
        assert initial_record is not None

        action = int(np.flatnonzero(observation["action_mask"])[0])
        if api == "native":
            stepped = game.step(action, token)
            next_observation = stepped["obs"]
        else:
            next_observation, _, terminated, truncated, _ = game.step(action)
            assert not terminated and not truncated
        retained_after_step = {
            key: value.copy() for key, value in next_observation.items()
        }
        after_step_record = game.record()
        assert after_step_record is not None and after_step_record != initial_record
        assert len(json.loads(after_step_record)["steps"]) == 1
        assert all(
            np.array_equal(observation[key], value) for key, value in retained.items()
        )

        if api == "native":
            game.reset(seed=7)
        else:
            game.reset(seed=7)
            assert game.np_random_seed == 7
        after_reset_record = game.record()
        assert after_reset_record is not None
        assert json.loads(after_reset_record)["seed"] == 7
        assert json.loads(initial_record)["seed"] == 11
        assert len(json.loads(after_step_record)["steps"]) == 1

        game.close()
        assert all(
            np.array_equal(observation[key], value) for key, value in retained.items()
        )
        assert all(
            np.array_equal(next_observation[key], value)
            for key, value in retained_after_step.items()
        )
        assert info == retained_info
        closed = json.loads(game.record())
        assert closed["closed"] is True
        assert closed["seed"] == 7
    finally:
        game.close()


@pytest.mark.parametrize("operation", ["reset", "step"])
def test_m10_native_delivery_preserves_original_baseexception_and_first_fault(
    operation,
):
    class Unprintable(BaseException):
        def __str__(self):
            raise AssertionError("the original exception must not be formatted")

    game = _native_m10()
    try:
        first = game.reset(seed=11)
        error = Unprintable()

        def fail(raw):
            assert raw["info"]["schema_id"] == "spire-gym-run-observation-v2"
            raise error

        with pytest.raises(Unprintable) as caught:
            if operation == "reset":
                game._reset_deliver(fail, seed=7)
            else:
                action = int(np.flatnonzero(first["obs"]["action_mask"])[0])
                game._step_deliver(action, first["token"], fail)
        assert caught.value is error

        encoded = game.record()
        assert encoded is not None
        record = json.loads(encoded)
        assert record["schema_id"] == "spire-gym-run-replay-v2"
        assert record["profile_id"] == "M10IroncladHeart"
        assert record["fault"]["error"]["kind"] == "delivery_fault"
        assert len(record["steps"]) == (1 if operation == "step" else 0)
        assert record["fault"]["committed"] is (operation == "step")
        assert record["fault"]["gameplay_recorded"] is (operation == "step")

        with pytest.raises(_native.RunFaultError):
            game.reset(seed=11)
        assert game.record() == encoded
    finally:
        game.close()


def test_m10_native_delivery_lease_rejects_nested_and_concurrent_reentry():
    game = _native_m10()
    try:

        def package(raw):
            assert raw["info"]["schema_id"] == "spire-gym-run-observation-v2"
            with pytest.raises(_native.RunStateError, match="busy"):
                game.reset(seed=7)
            with ThreadPoolExecutor(max_workers=1) as executor:

                def competing_close():
                    with pytest.raises(_native.RunStateError, match="busy"):
                        game.close()

                executor.submit(competing_close).result(timeout=2)
            return raw

        boundary = game._reset_deliver(package, seed=11)
        assert boundary["info"]["schema_id"] == "spire-gym-run-observation-v2"
        assert json.loads(game.record())["fault"] is None
    finally:
        game.close()


@pytest.mark.parametrize("operation", ["reset", "step"])
def test_m10_gym_final_tuple_delivery_fault_is_retained_without_retry(
    operation, monkeypatch
):
    class Unprintable(BaseException):
        def __str__(self):
            raise AssertionError("the original exception must not be formatted")

    game = _gym_m10()
    try:
        observation, _ = game.reset(seed=11)
        error = Unprintable()

        def fail(self, raw):
            assert raw["info"]["schema_id"] == "spire-gym-run-observation-v2"
            raise error

        with pytest.raises(Unprintable) as caught:
            if operation == "reset":
                monkeypatch.setattr(RunEnv, "_package_reset", fail)
                game.reset(seed=7)
            else:
                monkeypatch.setattr(RunEnv, "_package_step", fail)
                game.step(int(np.flatnonzero(observation["action_mask"])[0]))
        assert caught.value is error

        encoded = game.record()
        assert encoded is not None
        record = json.loads(encoded)
        assert record["schema_id"] == "spire-gym-run-replay-v2"
        assert record["profile_id"] == "M10IroncladHeart"
        assert record["fault"]["error"]["kind"] == "delivery_fault"
        assert len(record["steps"]) == (1 if operation == "step" else 0)
        assert record["fault"]["committed"] is (operation == "step")
        assert record["fault"]["gameplay_recorded"] is (operation == "step")

        with pytest.raises(_native.RunFaultError):
            game.reset(seed=11)
        assert game.record() == encoded
    finally:
        game.close()
