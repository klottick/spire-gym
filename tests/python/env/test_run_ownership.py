"""Retained delivery faults and reentry checks on the real Run owner."""

import json
from concurrent.futures import ThreadPoolExecutor

import pytest

from spire_lab import _native
from spire_lab import gymnasium as adapters


def native(**kwargs):
    return _native.NativeRunSession(
        "ironclad", 0, partition="test", record_replay=True, **kwargs
    )


def test_run_returned_buffers_and_decoded_records_cannot_mutate_native_state():
    game = native()
    first = game.reset(seed=11)
    first["obs"]["player"][0] = 999
    first["obs"]["action_mask"][:] = 0
    detached = json.loads(game.record())
    detached["steps"].append({"action": "not native"})
    after = game.step(0, first["token"])
    assert after["obs"]["player"][0] == 80
    assert after["obs"]["action_mask"].any()
    assert len(json.loads(game.record())["steps"]) == 1
    assert first["obs"]["player"][0] == 999


def test_run_post_step_capacity_fault_retains_action_and_rejects_reset_retry():
    game = native(capacities={"map_nodes": 1})
    first = game.reset(seed=11)
    with pytest.raises(RuntimeError):
        game.step(0, first["token"])
    record = json.loads(game.record())
    assert len(record["steps"]) == 1 and record["steps"][0]["reward"] == 0
    assert record["terminal"] is None
    assert record["fault"]["error"]["kind"] == "capacity"
    before = game.record()
    for operation in [lambda: game.reset(seed=7), lambda: game.step(0, first["token"])]:
        with pytest.raises(RuntimeError):
            operation()
        assert game.record() == before
    game.close()
    game.close()
    assert json.loads(game.record())["steps"] == record["steps"]


@pytest.mark.parametrize("operation", ["reset", "step"])
def test_run_packaging_failure_retains_original_base_exception_and_committed_prefix(
    operation,
):
    class Unprintable(BaseException):
        def __str__(self):
            raise AssertionError("must not format the original exception")

    game = native()
    error = Unprintable()

    def fail(raw):
        assert "obs" in raw
        raise error

    with pytest.raises(Unprintable) as caught:
        if operation == "reset":
            game._reset_deliver(fail, seed=11)
        else:
            first = game.reset(seed=11)
            game._step_deliver(0, first["token"], fail)
    assert caught.value is error
    record = json.loads(game.record())
    assert len(record["steps"]) == (1 if operation == "step" else 0)
    assert record["fault"]["error"]["kind"] == "delivery_fault"
    with pytest.raises(RuntimeError):
        game.reset(seed=11)
    assert json.loads(game.record())["fault"] == record["fault"]


def test_run_callback_and_concurrent_reentry_are_busy_without_deadlock():
    game = native()

    def package(raw):
        with pytest.raises(RuntimeError, match="busy"):
            game.reset(seed=7)
        with ThreadPoolExecutor(max_workers=1) as executor:

            def competing():
                with pytest.raises(RuntimeError, match="busy"):
                    game.close()

            executor.submit(competing).result(timeout=2)
        return raw

    game._reset_deliver(package, seed=11)
    assert json.loads(game.record())["fault"] is None


def test_run_facade_tuple_packaging_remains_inside_native_delivery_lease(monkeypatch):
    cls = adapters.RunEnv
    game = cls("ironclad", 0, record_replay=True, partition="test")
    game.reset(seed=11)

    def fail(self, raw):
        raise MemoryError("test final tuple packaging")

    monkeypatch.setattr(cls, "_package_step", fail)
    with pytest.raises(MemoryError):
        game.step(0)
    encoded = game.record()
    assert encoded is not None
    record = json.loads(encoded)
    assert len(record["steps"]) == 1
    assert record["fault"]["committed"]
    with pytest.raises(RuntimeError):
        game.reset()
