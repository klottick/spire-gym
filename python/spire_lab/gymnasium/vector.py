"""Gymnasium SAME_STEP facade over one native batched combat pool."""

import gymnasium as gym
from gymnasium.utils import seeding
from gymnasium.vector.utils import batch_space
import numpy as np

from spire_lab import _native
from . import _common
from .spaces import spaces_from_spec


def _reset_inputs(num_envs, options):
    """Validate facade-only options; native validates every seed/scenario entry."""
    selected = np.ones(num_envs, dtype=bool)
    if options is None:
        return selected, None
    if not isinstance(options, dict):
        raise _common.input_error("shape", "reset options must be a dictionary or None")
    if any(key not in ("reset_mask", "scenario_ids") for key in options):
        raise _common.input_error("scenario", "unknown reset option")
    if "scenario_ids" in options and type(options["scenario_ids"]) is not list:
        raise _common.input_error("shape", "scenario_ids must be a length-B list")
    if "reset_mask" in options:
        mask = options["reset_mask"]
        if not isinstance(mask, np.ndarray) or mask.dtype != np.dtype(bool):
            raise _common.input_error("dtype", "reset_mask must be a NumPy bool array")
        if mask.shape != (num_envs,) or not mask.any():
            raise _common.input_error("shape", "reset_mask must be nonempty bool[B] with a true row")
        selected = mask.copy()
    return selected, options.get("scenario_ids")


def _package_info(columns, present, *, indices=None):
    """Batch/scatter numeric columns without dispatching per-slot Python rules.

    ``present`` is a full-width presence mask. Compact final columns use indices;
    current columns are already full-width and pass through on ordinary steps.
    """
    result = {}
    for key in ("episode_id", "scenario_id", "episode_steps", "outcome",
                "hp_before_healing", "hp_after_cleanup"):
        mask = present.copy()
        if key.startswith("hp_"):
            if indices is None:
                mask &= columns[key + "_mask"]
            else:
                mask[indices] &= columns[key + "_mask"]
            if not mask.any():
                continue
        values = columns[key]
        if indices is not None:
            scattered = np.zeros(present.shape, dtype=values.dtype)
            scattered[indices] = values
            values = scattered
        elif not present.all():
            values = values.copy()
            values[~present] = "" if values.dtype.kind == "U" else 0
        result[key], result["_" + key] = values, mask
    return result


def _package_finals(raw, num_envs, info):
    """Only terminal rows become independent observation dictionaries."""
    indices = raw["final_indices"]
    if not indices.size:
        return
    present = np.zeros(num_envs, dtype=bool)
    present[indices] = True
    observations = np.full(num_envs, None, dtype=object)
    for row, slot in enumerate(indices):
        observations[slot], _ = _common.package_row(raw["final_obs"], raw["final_info"], row)
    info["final_obs"], info["_final_obs"] = observations, present
    info["final_info"] = _package_info(raw["final_info"], present, indices=indices)
    info["_final_info"] = present.copy()


class CombatVectorEnv(gym.vector.VectorEnv):
    """One native pool, not a collection of Python Env instances.

    Actions must be native-endian int64 NumPy arrays of shape [B]; strided and
    read-only arrays are accepted, lists and coercions are not. Choose actions
    from the current observation's mask. Bare indices cannot identify stale
    observations; native decision tokens remain private transport.

    SAME_STEP is not supported by every wrapper (notably Gymnasium 1.4.0's generic
    VectorObservationWrapper). This is not an SB3 VecEnv or reflection-compatible
    SyncVectorEnv replacement. Compatibility NumPy generators do not drive combat.
    """

    metadata = {"render_modes": [], "autoreset_mode": gym.vector.AutoresetMode.SAME_STEP}

    def __init__(self, num_envs: int, *, catalog=None, workers=1, partition="train",
                 render_mode=None, record_replay=False):
        if render_mode is not None:
            raise _common.input_error("range", "only render_mode=None is supported")
        self.render_mode = None
        self._guard = _common.OperationGuard()
        self._pool = _native.NativeCombatPool(
            catalog, num_envs=num_envs, workers=workers, partition=partition,
            autoreset="same_step", record_replay=record_replay)
        self.num_envs = num_envs
        self.closed = False
        self.single_observation_space, self.single_action_space = spaces_from_spec(
            self.observation_spec())
        self.observation_space = batch_space(self.single_observation_space, num_envs)
        self.action_space = batch_space(self.single_action_space, num_envs)
        self._tokens = np.zeros((num_envs, 2), dtype=np.uint64)
        self._generators = (None,) * num_envs
        self._seeds = (None,) * num_envs

    def observation_spec(self):
        """Fresh native-authoritative descriptor and schema fingerprint."""
        return self._pool.observation_spec()

    def _initialize_compatibility(self):
        generators, seeds = list(self._generators), list(self._seeds)
        for slot, generator in enumerate(generators):
            if generator is None:
                generators[slot], seeds[slot] = seeding.np_random()
        self._generators, self._seeds = tuple(generators), tuple(seeds)

    @property
    def np_random(self):
        with self._guard:
            self._initialize_compatibility()
            return self._generators

    @property
    def np_random_seed(self):
        with self._guard:
            self._initialize_compatibility()
            return self._seeds

    def reset(self, *, seed=None, options=None):
        """Reset selected rows and return full observations, selected-only info.

        Seed is None, a nonnegative built-in int, or a length-B list of int/None.
        Native scalar expansion uses seed + actual slot, even on partial resets;
        None continues a sequence. Options accept only scenario_ids (length-B
        list of ID/None) and reset_mask (NumPy bool[B], at least one true).
        Every entry is validated, even if unselected. Unselected slots must
        already be initialized. Caller containers are never modified.
        """
        with self._guard:
            selected, scenarios = _reset_inputs(self.num_envs, options)
            indices = np.flatnonzero(selected).tolist()
            # Native validates the full B-entry inputs and stages reset + all-row
            # export atomically. Never reset selected slots then separately read.
            def package(raw):
                info = _package_info(raw["info"], selected)
                generators, seeds = list(self._generators), list(self._seeds)
                for slot, effective in zip(indices, raw["effective_seed_bytes"], strict=True):
                    if effective is not None:
                        # Marshal actual native-expanded bytes only; no Python
                        # scalar seed addition, game derivation or scheduling.
                        generators[slot], seeds[slot] = seeding.np_random(
                            int.from_bytes(effective, "big"))
                self._generators, self._seeds = tuple(generators), tuple(seeds)
                self._tokens = raw["tokens"]
                return raw["obs"], info

            return self._pool._reset_deliver(
                indices, package, seed=seed, scenario_ids=scenarios, return_all=True)

    def step(self, actions):
        with self._guard:
            # The binding strictly checks dtype/shape/endian and entire-batch
            # legality before mutation, including read-only/strided input copies.
            def package(raw):
                info = _package_info(raw["info"], np.ones(self.num_envs, dtype=bool))
                _package_finals(raw, self.num_envs, info)
                result = (raw["obs"], raw["rewards"], raw["terminated"], raw["truncated"], info)
                self._tokens = raw["tokens"]
                return result

            return self._pool._step_deliver(actions, self._tokens, package)

    def close(self):
        with self._guard:
            self._pool.close()
            self.closed = True

    def render(self):
        with self._guard:
            return None

    def diagnostics(self):
        """Explicit privileged native status/fault metadata."""
        with self._guard:
            return self._pool.diagnostics()

    def reproduction(self, slot):
        """Explicit privileged record for one slot; recording defaults off."""
        with self._guard:
            return self._pool.reproduction(slot)

    def take_completed_records(self):
        with self._guard:
            return self._pool.take_completed_records()
