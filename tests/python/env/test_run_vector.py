"""Gymnasium contract for mixed native Run batches."""

import numpy as np
import gymnasium as gym
import pytest
from spire_lab import _native

from spire_lab.gymnasium import RunVectorEnv
from .vector_conformance import assert_vector_conformance


CONFIGS = [
    {"character": "ironclad", "ascension": 0},
    {"character": "silent", "ascension": 0},
]


@pytest.mark.parametrize("mode", ["disabled", "next_step", "same_step"])
def test_run_vector_conformance_modes_and_close_kwargs(mode):
    env = RunVectorEnv(CONFIGS, autoreset_mode=mode)
    try:
        assert env.metadata["autoreset_mode"] == {
            "disabled": gym.vector.AutoresetMode.DISABLED,
            "next_step": gym.vector.AutoresetMode.NEXT_STEP,
            "same_step": gym.vector.AutoresetMode.SAME_STEP,
        }[mode]
        _, info = assert_vector_conformance(env)
        assert "effective_seed" not in info
        obs, _ = env.reset(seed=91)
        before = {key: values.copy() for key, values in obs.items()}
        mask = np.asarray([True, False], dtype=np.bool_)
        after, _ = env.reset(seed=92, options={"reset_mask": mask})
        for key in after:
            np.testing.assert_array_equal(after[key][1], before[key][1])
        assert env.combat_views()["in_combat"].shape == (2,)
    finally:
        env.close(something="accepted")


def test_run_vector_export_failure_is_recoverable(monkeypatch):
    import spire_lab.gymnasium.run_vector as module

    env = RunVectorEnv(CONFIGS)
    try:
        observations, _ = env.reset(seed=18)
        actions = np.asarray([
            int(np.flatnonzero(observations["action_mask"][slot])[0])
            for slot in range(env.num_envs)
        ], dtype=np.int64)
        original = module.package_batch

        def fail_package(*args, **kwargs):
            raise LookupError("test export failure")

        monkeypatch.setattr(module, "package_batch", fail_package)
        with pytest.raises(LookupError, match="test export failure"):
            env.step(actions)
        monkeypatch.setattr(module, "package_batch", original)
        with pytest.raises(_native.BatchFaultError) as caught:
            env.step(actions)
        assert caught.value.operation == "export"
        assert all(row["status"] == "committed_undelivered" for row in caught.value.slots)
        recovered = env.recover_committed()
        assert recovered[5].all()
        assert recovered[1].dtype == np.dtype(np.float32)
        assert recovered[0]["action_mask"].shape[0] == env.num_envs
    finally:
        env.close()
