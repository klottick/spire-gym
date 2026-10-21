"""Strict single-environment Gymnasium facade over one manual native slot."""

import gymnasium as gym
import numpy as np

from spire_lab import _native
from . import _common
from .spaces import spaces_from_spec


class CombatEnv(gym.Env):
    """Source-grounded combat slice; actions must come from the current mask.

    Integer actions cannot identify stale observations. Native decision tokens are
    private transport, not an additional Gymnasium action or learned feature.
    """

    metadata = {"render_modes": []}

    def __init__(self, *, catalog=None, partition="train", render_mode=None,
                 record_replay=False):
        if render_mode is not None:
            raise _common.input_error("range", "only render_mode=None is supported")
        self.render_mode = None
        self._guard = _common.OperationGuard()
        self._pool = _native.NativeCombatPool(
            catalog, num_envs=1, workers=1, partition=partition,
            autoreset="manual", record_replay=record_replay)
        self.observation_space, self.action_space = spaces_from_spec(self.observation_spec())
        self._tokens = np.zeros((1, 2), dtype=np.uint64)

    def observation_spec(self):
        """Return a fresh native descriptor, including its native fingerprint."""
        return self._pool.observation_spec()

    def reset(self, *, seed=None, options=None):
        with self._guard:
            scenarios = _common.single_reset_inputs(seed, options)

            def package(raw):
                observation, info = _common.package_row(raw["obs"], raw["info"], 0)
                tokens = raw["tokens"].copy()
                effective = raw["effective_seed_bytes"][0]
                if effective is not None:
                    super(CombatEnv, self).reset(seed=int.from_bytes(effective, "big"))
                self._tokens = tokens
                return observation, info

            return self._pool._reset_deliver([0], package, seed=seed, scenario_ids=scenarios)

    def step(self, action):
        with self._guard:
            actions = np.array([_common.action_scalar(action)], dtype=np.int64)

            def package(raw):
                observation, info = _common.package_row(raw["obs"], raw["info"], 0)
                result = (observation, float(raw["rewards"][0]),
                          bool(raw["terminated"][0]), bool(raw["truncated"][0]), info)
                self._tokens = raw["tokens"].copy()
                return result

            return self._pool._step_deliver(actions, self._tokens, package)

    def close(self):
        with self._guard:
            self._pool.close()

    def render(self):
        with self._guard:
            return None

    def diagnostics(self):
        """Explicit privileged native status/fault export (not a snapshot API)."""
        with self._guard:
            return self._pool.diagnostics()

    def reproduction(self):
        """Explicit privileged current record export; recording defaults off."""
        with self._guard:
            return self._pool.reproduction(0)

    def take_completed_records(self):
        """Drain native owned completed/abandoned/faulted records exactly once."""
        with self._guard:
            return self._pool.take_completed_records()
