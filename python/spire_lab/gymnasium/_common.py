"""Facade marshalling and delivery guards; all simulator policy stays native."""

from operator import index
from threading import Lock

import numpy as np

from spire_lab import _native


def input_error(reason, message, *, slot=None):
    error = _native.CombatInputError(message)
    error.slot, error.reason, error.faults = slot, reason, []
    return error


class OperationGuard:
    """Nonblocking facade lease, including Python packaging after native return."""

    def __init__(self):
        self._lock = Lock()

    def __enter__(self):
        if not self._lock.acquire(blocking=False):
            error = _native.CombatStateError("facade operation is busy")
            error.slot, error.reason, error.faults = None, "busy", []
            raise error
        return self

    def __exit__(self, *exc):
        self._lock.release()


def action_scalar(action):
    if isinstance(action, (bool, np.bool_)) or not isinstance(action, (int, np.integer)):
        raise input_error("dtype", "action must be an integer scalar", slot=0)
    if isinstance(action, np.integer) and action.dtype.kind not in ("i", "u"):
        raise input_error("dtype", "action must have an integer dtype", slot=0)
    try:
        value = index(action)
    except TypeError as error:
        raise input_error("dtype", "action must support integer index extraction", slot=0) from error
    if not 0 <= value <= np.iinfo(np.int64).max:
        raise input_error("range", "action is outside the native index range", slot=0)
    return value


def single_reset_inputs(seed, options):
    # Keep byte marshalling, entropy, scheduling and seed derivation in the binding.
    if seed is not None and type(seed) is not int:
        raise input_error("seed", "seed must be a nonnegative built-in integer or None")
    if options is None:
        return None
    if not isinstance(options, dict):
        raise input_error("shape", "reset options must be a dictionary or None")
    if any(key != "scenario_id" for key in options):
        raise input_error("scenario", "unknown reset option")
    if "scenario_id" not in options:
        return None
    scenario = options["scenario_id"]
    if type(scenario) is not str:
        raise input_error("scenario", "scenario_id must be a string", slot=0)
    return [scenario]


def package_row(observations, infos, row):
    """Detach one observation and deterministic scalar info from native columns."""
    observation = {key: values[row].copy() for key, values in observations.items()}
    info = {key: infos[key][row].item()
            for key in ("episode_id", "scenario_id", "episode_steps", "outcome")}
    for key in ("hp_before_healing", "hp_after_cleanup"):
        if infos[key + "_mask"][row]:
            info[key] = int(infos[key][row])
    return observation, info
