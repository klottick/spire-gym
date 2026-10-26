"""Standalone owned Ironclad A0 combat transport; game rules stay in Rust."""

from __future__ import annotations

import json
import math
from operator import index
from typing import Any, cast

import gymnasium as gym
import numpy as np
from numpy.typing import NDArray

from spire_lab import _native

from . import _common
from .spaces import spaces_from_spec

Observation = dict[str, NDArray[np.int64] | NDArray[np.int8]]


def _catalog_json(entry_catalog: dict[str, Any]) -> str:
    if type(entry_catalog) is not dict:
        raise _common.input_error("dtype", "entry_catalog must be a dictionary")

    def validate(value, active):
        if value is None or type(value) in (bool, int, str):
            return
        if type(value) is float:
            if not math.isfinite(value):
                raise ValueError("nonfinite JSON number")
            return
        if type(value) in (dict, list):
            identity = id(value)
            if identity in active:
                raise ValueError("circular JSON value")
            active.add(identity)
            try:
                if type(value) is dict:
                    for key, child in value.items():
                        if type(key) is not str:
                            raise ValueError("JSON object keys must be strings")
                        validate(child, active)
                else:
                    for child in value:
                        validate(child, active)
            finally:
                active.remove(identity)
            return
        raise ValueError("value is not strict JSON")

    try:
        validate(entry_catalog, set())
        return json.dumps(
            entry_catalog,
            ensure_ascii=True,
            allow_nan=False,
            separators=(",", ":"),
        )
    except (TypeError, ValueError, OverflowError) as error:
        raise _common.input_error(
            "scenario", "entry_catalog must contain strict JSON values"
        ) from error


def _reset_inputs(seed, options):
    if seed is not None and type(seed) is not int:
        raise _common.input_error(
            "seed", "seed must be a nonnegative built-in integer or None"
        )
    if seed is not None and not 0 <= seed < 2**64:
        raise _common.input_error("seed", "seed must fit in an unsigned 64-bit integer")
    if options is None:
        return None
    if type(options) is not dict:
        raise _common.input_error("shape", "reset options must be a dictionary or None")
    if any(key != "entry_id" for key in options):
        raise _common.input_error("scenario", "unknown reset option")
    if "entry_id" not in options:
        return None
    entry_id = options["entry_id"]
    if type(entry_id) is not str:
        raise _common.input_error("dtype", "entry_id must be a string")
    return entry_id


class IroncladCombatEnv(gym.Env[Observation, int]):
    """Independent native Ironclad A0 combats selected from a finite catalog.

    Actions are indices into the current native legal mask. The decision token
    stays private to the facade and ensures an old integer cannot be replayed.
    """

    def __init__(
        self, entry_catalog: dict[str, Any], *, partition: str = "train"
    ) -> None:
        self.metadata = {"render_modes": []}
        self.render_mode = None
        self._guard = _common.OperationGuard()
        self._session = _native.NativeIroncladCombatSession(
            _catalog_json(entry_catalog), partition=partition
        )
        observations, actions = spaces_from_spec(self.observation_spec())
        self.observation_space = cast(gym.spaces.Space[Observation], observations)
        self.action_space = cast(gym.spaces.Space[int], actions)
        self._token = (0, 0)
        self.closed = False

    def observation_spec(self) -> dict[str, Any]:
        """Return the fresh Rust-authored common combat view descriptor."""
        return self._session.observation_spec()

    def environment_spec(self) -> dict[str, Any]:
        """Return profile, rules, reward, seed and catalog identities."""
        return self._session.environment_spec()

    def catalog_descriptor(self) -> dict[str, Any]:
        """Return the normalized, finite public entry catalog."""
        return self._session.catalog_descriptor()

    def _package_reset(self, raw):
        result = (raw["obs"], dict(raw["info"]))
        super().reset(seed=raw["effective_seed"])
        self._token = raw["token"]
        return result

    def reset(self, *, seed: int | None = None, options: dict[str, Any] | None = None):
        with self._guard:
            entry_id = _reset_inputs(seed, options)
            return self._session._reset_deliver(
                self._package_reset, seed=seed, entry_id=entry_id
            )

    def _package_step(self, raw):
        result = (
            raw["obs"],
            float(raw["reward"]),
            raw["terminated"],
            raw["truncated"],
            dict(raw["info"]),
        )
        self._token = raw["token"]
        return result

    def step(self, action: int):
        with self._guard:
            if isinstance(action, (bool, np.bool_)) or not isinstance(
                action, (int, np.integer)
            ):
                raise _common.input_error(
                    "dtype", "action must be an integer scalar, not bool"
                )
            try:
                action_index = index(action)
            except TypeError as error:
                raise _common.input_error(
                    "dtype", "action must be an integer scalar"
                ) from error
            if not 0 <= action_index <= np.iinfo(np.int64).max:
                raise _common.input_error(
                    "range", "action is outside the native index range"
                )
            return self._session._step_deliver(
                action_index, self._token, self._package_step
            )

    def diagnostic(self) -> dict[str, Any]:
        """Return explicit native fault and accepted-step diagnostics."""
        with self._guard:
            return self._session.diagnostic()

    def close(self) -> None:
        with self._guard:
            self._session.close()
            self.closed = True

    def render(self) -> None:
        return None
