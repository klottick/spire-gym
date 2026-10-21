"""Attached delivery is part of the original native operation, never a new lease."""

import numpy as np
import pytest

from spire_lab import _native
from .support import template


@pytest.mark.parametrize("recording", [False, True])
def test_delivery_completion_rejection_and_no_escaped_ownership(recording):
    pool = _native.NativeCombatPool(
        [template(encounter="jaw_worm", enemy_hp=42, player_hp=1)],
        autoreset="same_step", record_replay=recording)
    initial = pool._reset_deliver([0], lambda raw: raw, seed=7)
    actions = np.zeros(1, dtype=np.int64)

    def package(raw):
        # Even raw-pool access cannot drain records or begin another operation
        # while this call still owns delivery. Static metadata remains available.
        assert pool.observation_spec()["schema_id"]
        for call in (pool.diagnostics, pool.take_completed_records,
                     lambda: pool.reproduction(0), pool.close,
                     lambda: pool.reset([0]),
                     lambda: pool.step_batch(actions, initial["tokens"]),
                     lambda: pool.mark_export_fault("unrelated")):
            with pytest.raises(_native.CombatStateError) as busy:
                call()
            assert busy.value.reason == "busy"
        return raw

    delivered = pool._step_deliver(actions, initial["tokens"], package)
    assert delivered["terminated"].tolist() == [True]
    assert delivered["info"]["episode_id"].tolist() == ["episode-1"]
    assert pool.diagnostics()["status"] == "active"

    def must_not_package(raw):
        pytest.fail("rejected inputs must not invoke delivery")

    for bad_actions, bad_tokens in (
        (actions, initial["tokens"]),  # consumed token, not reusable delivery authority
        (np.array([99], dtype=np.int64), delivered["tokens"]),
        (np.array([0], dtype=np.int32), delivered["tokens"]),
    ):
        with pytest.raises(_native.CombatInputError):
            pool._step_deliver(bad_actions, bad_tokens, must_not_package)
        assert pool.diagnostics()["status"] == "active"
    with pytest.raises(_native.CombatInputError):
        pool._reset_deliver([0], must_not_package, seed=-1)

    # A later transaction's failure must not inherit the successful delivery's
    # queue range, even though its old record is undrained.
    failure = KeyboardInterrupt("later reset packaging")

    def fail(raw):
        raise failure

    with pytest.raises(KeyboardInterrupt) as caught:
        pool._reset_deliver([0], fail)
    assert caught.value is failure
    assert pool.diagnostics()["status"] == "faulted"
    records = pool.take_completed_records()
    current = pool.reproduction(0)
    if recording:
        earlier, abandoned = records
        assert (earlier["episode_id"], earlier["status"], earlier["fault"]) == (
            "episode-0", "completed", None)
        assert earlier["actions"].tolist() == [0]
        assert abandoned["episode_id"] == "episode-1"
        assert abandoned["status"] == current["status"] == "faulted"
        assert abandoned["fault"]["stage"] == current["fault"]["stage"] == "export"
        assert abandoned["actions"].tolist() == current["actions"].tolist() == []
        assert abandoned["attempted_action"] is current["attempted_action"] is None
    else:
        assert records == [] and current is None
    pool.close()


def test_empty_reset_delivery_failure_is_read_only():
    pool = _native.NativeCombatPool(record_replay=True)
    initial = pool.reset([0], seed=7)

    def fail(raw):
        raise MemoryError("cached rows only")

    with pytest.raises(MemoryError):
        pool._reset_deliver([], fail, return_all=True)
    assert pool.diagnostics()["status"] == "active"
    assert pool.reproduction(0)["status"] == "active"
    # The unchanged original decision can still be consumed exactly once.
    pool.step_batch(np.zeros(1, dtype=np.int64), initial["tokens"])
    pool.close()
