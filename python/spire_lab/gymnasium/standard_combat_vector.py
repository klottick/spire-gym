"""Gymnasium vector facade for native `standard-combat-v1` batches."""

from __future__ import annotations

from typing import Any

import gymnasium as gym
import numpy as np
from gymnasium.vector.utils import batch_space

from spire_lab import _native
from . import _common
from ._vector import package_batch, package_recovered, reset_options
from .ironclad import _catalog_json
from .spaces import spaces_from_spec


class StandardCombatVectorEnv(gym.vector.VectorEnv):
    metadata = {"render_modes": []}

    def __init__(self, entry_catalog: dict[str, Any], num_envs: int, *, records=None,
                 partition="train", entry_ids=None, workers=1,
                 autoreset_mode="next_step", fault_policy="freeze"):
        if type(num_envs) is not int or not 1 <= num_envs <= 256:
            raise _common.input_error("range", "num_envs must be an integer in 1..=256")
        modes = {"disabled": gym.vector.AutoresetMode.DISABLED,
                 "next_step": gym.vector.AutoresetMode.NEXT_STEP,
                 "same_step": gym.vector.AutoresetMode.SAME_STEP}
        if autoreset_mode not in modes:
            raise _native.RunInputError("unknown autoreset mode")
        self.metadata = {"render_modes": [], "autoreset_mode": modes[autoreset_mode]}
        self.render_mode = None
        self._guard = _common.OperationGuard()
        self._batch = _native.NativeCombatTrainingBatch(
            num_envs, _catalog_json(entry_catalog), kind="standard", records=records,
            partition=partition, entry_ids=entry_ids, workers=workers,
            autoreset=autoreset_mode, fault_policy=fault_policy,
        )
        self.num_envs = num_envs
        self.closed = False
        self.single_observation_space, self.single_action_space = spaces_from_spec(self.observation_spec())
        self.observation_space = batch_space(self.single_observation_space, num_envs)
        self.action_space = batch_space(self.single_action_space, num_envs)
        self._tokens = np.zeros((num_envs, 2), dtype=np.uint64)
        self._autoreset_active = autoreset_mode != "disabled"
        self._fault_active = fault_policy == "isolate"

    def observation_spec(self):
        return self._batch.observation_spec()

    def environment_spec(self):
        return self._batch.environment_spec()

    def catalog_descriptor(self):
        return self._batch.catalog_descriptor()

    def reset(self, *, seed=None, options=None):
        with self._guard:
            entries, mask = reset_options(self.num_envs, options)
            if entries is None:
                entries = [None] * self.num_envs
            def package(raw):
                self._tokens = raw["tokens"].copy()
                return raw["obs"], package_batch(
                    raw, self.num_envs, include_autoreset=self._autoreset_active,
                    include_fault=self._fault_active,
                )
            return self._batch._reset_deliver(
                package, seeds=seed, entry_ids=entries, reset_mask=mask)

    def step(self, actions):
        with self._guard:
            def package(raw):
                self._tokens = raw["tokens"].copy()
                return (raw["obs"], np.asarray(raw["rewards"], dtype=np.float32),
                        raw["terminated"], raw["truncated"],
                        package_batch(raw, self.num_envs,
                                      include_autoreset=self._autoreset_active,
                                      include_fault=self._fault_active))
            return self._batch._step_deliver(actions, self._tokens, package)

    def recover_committed(self):
        with self._guard:
            return package_recovered(self._batch.recover_committed(), self.num_envs)

    def diagnostics(self):
        with self._guard:
            return self._batch.diagnostics()

    def take_completed_records(self):
        with self._guard:
            return self._batch.take_completed_records()

    def take_fault_records(self):
        with self._guard:
            return self._batch.take_fault_records()

    def close(self, **kwargs: Any):
        with self._guard:
            self._batch.close()
            self.closed = True

    def render(self):
        with self._guard:
            return None
