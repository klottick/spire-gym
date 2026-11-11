"""Shared packaging helpers for native vector batches.

All game state, seed expansion and reward calculation stay in the native batch.
This module only translates owned batch rows to Gymnasium's dict-of-arrays form.
"""

from __future__ import annotations

from typing import Any

import numpy as np

from . import _common


def combat_v1_package_info(columns, present, *, indices=None):
    """Preserve combat-v1's established fixed-column info representation."""
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


def combat_v1_package_finals(raw, num_envs, info):
    """Preserve combat-v1's compact terminal rows and observation shape."""
    indices = raw["final_indices"]
    if not indices.size:
        return
    present = np.zeros(num_envs, dtype=bool)
    present[indices] = True
    observations = np.full(num_envs, None, dtype=object)
    for row, slot in enumerate(indices):
        from . import _common as common
        observations[slot], _ = common.package_row(raw["final_obs"], raw["final_info"], row)
    info["final_obs"], info["_final_obs"] = observations, present
    info["final_info"] = combat_v1_package_info(raw["final_info"], present, indices=indices)
    info["_final_info"] = present.copy()


def reset_options(num_envs: int, options: Any, *, entry_key: str | None = "entry_ids"):
    """Parse the common reset_mask and optional per-slot entry list."""
    entries = None
    selected = np.ones(num_envs, dtype=np.bool_)
    if options is None:
        return entries, selected
    if type(options) is not dict:
        raise _common.input_error("shape", "reset options must be a dictionary or None")
    allowed = {"reset_mask"}
    if entry_key is not None:
        allowed.add(entry_key)
    if any(key not in allowed for key in options):
        raise _common.input_error("scenario", "unknown reset option")
    if entry_key is not None and entry_key in options:
        entries = options[entry_key]
        if type(entries) is not list or len(entries) != num_envs:
            raise _common.input_error("shape", f"{entry_key} must be a length-B list")
        if any(value is not None and type(value) is not str for value in entries):
            raise _common.input_error("dtype", f"each {entry_key[:-1]} must be a string or None")
        entries = list(entries)
    if "reset_mask" in options:
        mask = options["reset_mask"]
        if (not isinstance(mask, np.ndarray) or mask.dtype != np.dtype(np.bool_)
                or not mask.dtype.isnative or mask.shape != (num_envs,)
                or not mask.any()):
            raise _common.input_error("shape", "reset_mask must be nonempty native bool[B] with a true row")
        selected = mask.copy()
    return entries, selected


def pack_rows(rows, *, selected=None):
    """Pack per-slot dictionaries into Gymnasium values and `_key` masks."""
    count = len(rows)
    keys = list(dict.fromkeys(key for row in rows if row for key in row))
    result = {}
    for key in keys:
        present = np.asarray([row is not None and key in row for row in rows], dtype=np.bool_)
        if selected is not None:
            present &= selected
        sample = next((row[key] for row in rows if row is not None and key in row), None)
        values = [row[key] if row is not None and key in row else _empty_like(sample) for row in rows]
        # Numeric scalars and strings retain useful homogeneous NumPy dtypes;
        # nested JSON values (reward events, faults) use independent objects.
        if isinstance(sample, (bool, int, float, str, np.generic)):
            try:
                array = np.asarray(values)
            except (TypeError, ValueError):
                array = np.asarray(values, dtype=object)
        else:
            array = np.empty(count, dtype=object)
            array[:] = values
        result[key] = array
        result["_" + key] = present.copy()
    return result


def _empty_like(value):
    if isinstance(value, (bool, np.bool_)):
        return False
    if isinstance(value, (int, np.integer)):
        return 0
    if isinstance(value, (float, np.floating)):
        return 0.0
    if isinstance(value, str):
        return ""
    return None


def package_observation_rows(observations, present):
    """Scatter a stacked observation dict into independent object rows."""
    count = len(present)
    output = np.full(count, None, dtype=object)
    indices = np.flatnonzero(present)
    for slot in indices:
        output[slot] = {key: values[slot].copy() for key, values in observations.items()}
    return output


def package_finals(raw, info, num_envs):
    mask = np.asarray(raw.get("final_mask", np.zeros(num_envs, dtype=np.bool_)), dtype=np.bool_).copy()
    if raw.get("final_obs") is None or not mask.any():
        return
    info["final_obs"] = package_observation_rows(raw["final_obs"], mask)
    info["_final_obs"] = mask.copy()
    final_info = raw.get("final_info", [None] * num_envs)
    packed = pack_rows(final_info, selected=mask)
    info["final_info"] = packed
    info["_final_info"] = mask.copy()


def package_batch(raw, num_envs, *, slots=False, include_autoreset=False, include_fault=False):
    """Package one common native batch result; token arrays remain private."""
    info = {"slots": [dict(row or {}) for row in raw["info"]]} if slots else pack_rows(raw["info"])
    if include_autoreset:
        info["autoreset"] = np.asarray(raw["autoreset"], dtype=np.bool_).copy()
        info["_autoreset"] = np.ones(num_envs, dtype=np.bool_)
    if include_fault:
        faults = np.empty(num_envs, dtype=object)
        faults[:] = raw["fault"]
        info["fault"] = faults
        info["_fault"] = np.asarray(raw["slot_fault"], dtype=np.bool_).copy()
        info["slot_fault"] = np.asarray(raw["slot_fault"], dtype=np.bool_).copy()
        info["_slot_fault"] = np.ones(num_envs, dtype=np.bool_)
    package_finals(raw, info, num_envs)
    return info


def package_recovered(raw, num_envs, *, slots=False):
    info = package_batch(raw, num_envs, slots=slots, include_autoreset=True, include_fault=True)
    recovered = np.asarray(raw["recovered"], dtype=np.bool_).copy()
    rewards = np.asarray(raw["rewards"], dtype=np.float32)
    return raw["obs"], rewards, raw["terminated"], raw["truncated"], info, recovered
