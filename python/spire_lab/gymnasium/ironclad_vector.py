"""Serial native batch with explicit final rows and caller-controlled resets."""

from __future__ import annotations

from typing import Any

import gymnasium as gym
import numpy as np
from gymnasium.vector.utils import batch_space
from numpy.typing import NDArray

from spire_lab import _native

from . import _common
from .ironclad import _catalog_json
from .spaces import spaces_from_spec


def _seed_values(seed, num_envs: int) -> list[int | None]:
    if seed is None:
        return [None] * num_envs
    if type(seed) is int:
        if not 0 <= seed < 2**64:
            raise _common.input_error(
                "seed", "seed must fit in an unsigned 64-bit integer"
            )
        values: list[int | None] = [seed + slot for slot in range(num_envs)]
        if any(value is not None and value >= 2**64 for value in values):
            raise _common.input_error("seed", "scalar seed expansion exceeds u64")
        return values
    if type(seed) is not list or len(seed) != num_envs:
        raise _common.input_error(
            "shape", "seed must be None, an integer, or a length-B list"
        )
    values = []
    for value in seed:
        if value is None:
            values.append(None)
        elif type(value) is int and 0 <= value < 2**64:
            values.append(value)
        else:
            raise _common.input_error(
                "seed", "each batch seed must be a nonnegative built-in integer or None"
            )
    return values


def _reset_inputs(num_envs: int, options):
    if options is None:
        return [None] * num_envs, np.ones(num_envs, dtype=np.bool_)
    if type(options) is not dict:
        raise _common.input_error("shape", "reset options must be a dictionary or None")
    if any(key not in ("entry_ids", "reset_mask") for key in options):
        raise _common.input_error("scenario", "unknown reset option")

    entry_ids = options.get("entry_ids")
    if entry_ids is None:
        entry_ids = [None] * num_envs
    elif type(entry_ids) is not list or len(entry_ids) != num_envs:
        raise _common.input_error("shape", "entry_ids must be a length-B list")
    else:
        entry_ids = list(entry_ids)
    if any(value is not None and type(value) is not str for value in entry_ids):
        raise _common.input_error("dtype", "each entry_id must be a string or None")

    reset_mask = options.get("reset_mask")
    if reset_mask is None:
        reset_mask = np.ones(num_envs, dtype=np.bool_)
    elif (
        not isinstance(reset_mask, np.ndarray)
        or reset_mask.dtype != np.dtype(np.bool_)
        or not reset_mask.dtype.isnative
        or reset_mask.shape != (num_envs,)
    ):
        raise _common.input_error(
            "shape", "reset_mask must be a native NumPy bool array with shape [B]"
        )
    else:
        # Preserve caller memory and make a safe input snapshot before native detach.
        reset_mask = reset_mask.copy()
    return entry_ids, reset_mask


class IroncladCombatVectorEnv(gym.vector.VectorEnv):
    """One native serial batch; terminal rows remain final until masked reset.

    ``seed`` may be None, one base seed expanded by actual slot, or an explicit
    list of per-slot seeds. Every step preflights all actions and tokens before
    dispatch. The native binding never creates one Python environment per row.
    """

    def __init__(
        self,
        num_envs: int,
        entry_catalog: dict[str, Any],
        *,
        partition: str = "train",
    ) -> None:
        if type(num_envs) is not int or not 1 <= num_envs <= 256:
            raise _common.input_error("range", "num_envs must be an integer in 1..=256")
        self.metadata = {
            "render_modes": [],
            "autoreset_mode": gym.vector.AutoresetMode.DISABLED,
        }
        self.render_mode = None
        self._guard = _common.OperationGuard()
        self._batch = _native.NativeIroncladCombatBatch(
            num_envs, _catalog_json(entry_catalog), partition=partition
        )
        self.num_envs = num_envs
        self.closed = False
        self.single_observation_space, self.single_action_space = spaces_from_spec(
            self.observation_spec()
        )
        self.observation_space = batch_space(self.single_observation_space, num_envs)
        self.action_space = batch_space(self.single_action_space, num_envs)
        self._tokens = np.zeros((num_envs, 2), dtype=np.uint64)

    def observation_spec(self) -> dict[str, Any]:
        """Return the native common combat view descriptor."""
        return self._batch.observation_spec()

    def environment_spec(self) -> dict[str, Any]:
        """Return profile and catalog identity metadata."""
        return self._batch.environment_spec()

    def catalog_descriptor(self) -> dict[str, Any]:
        """Return the normalized finite public entry catalog."""
        return self._batch.catalog_descriptor()

    def _package_reset(self, raw):
        info = {"slots": [dict(row) for row in raw["info"]]}
        tokens = raw["tokens"].copy()
        self._tokens = tokens
        return raw["obs"], info

    def reset(self, *, seed=None, options=None):
        with self._guard:
            seeds = _seed_values(seed, self.num_envs)
            entry_ids, reset_mask = _reset_inputs(self.num_envs, options)
            return self._batch._reset_deliver(
                self._package_reset,
                seeds=seeds,
                entry_ids=entry_ids,
                reset_mask=reset_mask,
            )

    def _package_step(self, raw):
        info = {"slots": [dict(row) for row in raw["info"]]}
        result = (
            raw["obs"],
            np.asarray(raw["rewards"], dtype=np.float32),
            raw["terminated"],
            raw["truncated"],
            info,
        )
        self._tokens = raw["tokens"].copy()
        return result

    def step(self, actions: NDArray[np.int64]):
        with self._guard:
            return self._batch._step_deliver(actions, self._tokens, self._package_step)

    def diagnostic(self) -> dict[str, Any]:
        """Return per-session accepted-step counts and retained batch faults."""
        with self._guard:
            return self._batch.diagnostic()

    def close(self, **kwargs: Any) -> None:
        with self._guard:
            self._batch.close()
            self.closed = True

    def render(self) -> None:
        with self._guard:
            return
