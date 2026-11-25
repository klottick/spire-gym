"""Public-belief combat search over an owned native arena."""

from __future__ import annotations

from operator import index
from typing import Any

import numpy as np

from spire_lab import _native

from . import _common
from .run import Observation, RunEnv
from ._vector import pack_rows


_HARD_CAPS = {
    "max_live_branches": 4096,
    "max_branches": 1 << 20,
    "max_steps_per_branch": 100_000,
    "max_total_steps": 100_000_000,
}


def _limits(limits=None, *, max_live_branches=None, max_branches=None,
            max_steps_per_branch=None, max_total_steps=None):
    legacy = {
        "max_live_branches": max_live_branches,
        "max_branches": max_branches,
        "max_steps_per_branch": max_steps_per_branch,
        "max_total_steps": max_total_steps,
    }
    if limits is not None:
        if any(value is not None for value in legacy.values()):
            raise _native.RunInputError("pass limits or individual limit keywords, not both")
        if not hasattr(limits, "keys"):
            raise _native.RunInputError("limits must be a mapping or None")
        unknown = set(limits) - set(_HARD_CAPS)
        if unknown:
            raise _native.RunInputError("limits contains an unknown limit name")
        return {**_HARD_CAPS, **limits}
    if all(value is None for value in legacy.values()):
        return dict(_HARD_CAPS)
    if any(value is None for value in legacy.values()):
        raise _native.RunInputError("all four individual limit keywords are required")
    return legacy


class PublicSearch:
    """A bounded arena of sampled worlds rooted at one public combat decision."""

    def __init__(self, native_search: _native.NativePublicSearch) -> None:
        self._native = native_search

    def __repr__(self) -> str:
        return f"<PublicSearch {self._native!r}>"

    def __reduce__(self):
        raise TypeError("PublicSearch is in-memory only and cannot be pickled")

    def branch(self, sample: int | None = None) -> PublicBranch:
        if sample is not None and (type(sample) is not int or not 0 <= sample < 2**64):
            raise _native.RunInputError("sample must be a nonnegative built-in u64 integer or None")
        branch_id = self._native.branch(sample)
        return PublicBranch(self, branch_id, (branch_id, 0))

    def counters(self) -> dict[str, int]:
        return self._native.counters()

    def spec(self) -> dict[str, Any]:
        return self._native.spec()

    def observation_spec(self) -> dict[str, Any]:
        return self._native.observation_spec()

    def limits(self) -> dict[str, int]:
        return dict(self.spec()["limits"])

    def close(self) -> None:
        self._native.close()

    def step_many(self, branches, actions, *, view="full"):
        """Step distinct branches in one native call, returning owned stacked rows."""
        if view not in ("full", "mask"):
            raise _native.RunInputError("view must be 'full' or 'mask'")
        if not isinstance(branches, (list, tuple)) or not branches:
            raise _native.RunInputError("branches must be a nonempty sequence")
        if not isinstance(actions, (list, tuple, np.ndarray)) or len(actions) != len(branches):
            raise _native.RunInputError("actions must have one integer per branch")
        ids, token_rows, action_rows = [], [], []
        for branch, action in zip(branches, actions, strict=True):
            if not isinstance(branch, PublicBranch) or branch._search is not self:
                raise _native.RunInputError("every branch must belong to this search")
            if isinstance(action, (bool, np.bool_)) or not isinstance(action, (int, np.integer)):
                raise _native.RunInputError("action must be an integer scalar, not bool")
            action_index = index(action)
            if not -(2**63) <= action_index < 2**63:
                raise _native.RunInputError("action is outside the native index range")
            if branch._token is None:
                branch.decision()
            ids.append(branch._branch_id)
            token_rows.append(branch._token)
            action_rows.append(action_index)
        tokens = np.asarray(token_rows, dtype=np.uint64)
        action_array = np.asarray(action_rows, dtype=np.int64)
        raw = self._native.step_many(ids, tokens, action_array, view=view)
        returned_tokens = raw["tokens"]
        for i, branch in enumerate(branches):
            if raw["fault"][i] is None:
                branch._token = (int(returned_tokens[i, 0]), int(returned_tokens[i, 1]))
        if view == "mask":
            observations = {"action_mask": raw["action_mask"]}
        else:
            observations = raw["obs"]
        info = pack_rows(raw["info"])
        faults = np.empty(len(branches), dtype=object)
        faults[:] = raw["fault"]
        info["fault"] = faults
        info["_fault"] = np.asarray([item is not None for item in raw["fault"]], dtype=np.bool_)
        return observations, raw["rewards"], raw["terminated"], raw["truncated"], info

    def playout_many(self, branches, *, rule="uniform_exclude_verbs", exclude_verbs=None,
                     max_steps=None, playout_seed=0) -> dict[str, Any]:
        """Play distinct live branches to combat end or a step limit in one native call (M41).

        Every move is chosen by the declared public rule ``spire-gym-public-playout-v1``: uniform over the legal actions
        whose Standard verb is not in ``exclude_verbs`` (default ``("end_turn", "discard_potion")``; ``()`` means every
        legal action), over all legal actions when none remain, drawn from a research stream keyed by the branch's
        sample, ``playout_seed`` and its step. Returns owned arrays in input order (see ``docs/standard-run-api.md``).
        """
        if not isinstance(branches, (list, tuple)) or not branches:
            raise _native.RunInputError("branches must be a nonempty sequence")
        ids, token_rows = [], []
        for branch in branches:
            if not isinstance(branch, PublicBranch) or branch._search is not self:
                raise _native.RunInputError("every branch must belong to this search")
            if branch._token is None:
                branch.decision()
            ids.append(branch._branch_id)
            token_rows.append(branch._token)
        raw = self._native.playout_many(
            ids, np.asarray(token_rows, dtype=np.uint64), rule=rule, exclude_verbs=exclude_verbs,
            max_steps=max_steps, playout_seed=playout_seed,
        )
        returned_tokens = raw["tokens"]
        for i, branch in enumerate(branches):
            if raw["fault"][i] is None:
                branch._token = (int(returned_tokens[i, 0]), int(returned_tokens[i, 1]))
        result = dict(raw)
        for key in ("end_reason", "refusal", "fault"):
            column = np.empty(len(branches), dtype=object)
            column[:] = raw[key]
            result[key] = column
        result["_refusal"] = np.asarray([item is not None for item in raw["refusal"]], dtype=np.bool_)
        result["_fault"] = np.asarray([item is not None for item in raw["fault"]], dtype=np.bool_)
        return result


class PublicBranch:
    """A sampled combat trajectory with a private, branch-specific decision token."""

    def __init__(self, search: PublicSearch, branch_id: int, token=None) -> None:
        self._search = search
        self._branch_id = branch_id
        self._token: tuple[int, int] | None = token

    def __repr__(self) -> str:
        return "<PublicBranch (redacted)>"

    def __reduce__(self):
        raise TypeError("PublicBranch is in-memory only and cannot be pickled")

    def decision(self) -> tuple[Observation, dict[str, Any]]:
        raw = self._search._native.decision(self._branch_id)
        self._token = raw["token"]
        return raw["obs"], dict(raw["info"])

    def step(self, action: int, view: str = "full"):
        if view not in ("full", "mask"):
            raise _native.RunInputError("view must be 'full' or 'mask'")
        if isinstance(action, (bool, np.bool_)) or not isinstance(action, (int, np.integer)):
            raise _native.RunInputError("action must be an integer scalar, not bool")
        try:
            action_index = index(action)
        except TypeError as error:
            raise _native.RunInputError("action must be an integer scalar") from error
        if self._token is None:
            self.decision()
        raw = self._search._native.step(
            self._branch_id, self._token, action_index, view
        )
        self._token = raw["token"]
        observation = raw["obs"] if raw["obs"] is not None else raw["action_mask"]
        return (
            observation,
            float(raw["reward"]),
            raw["terminated"],
            raw["truncated"],
            dict(raw["info"]),
        )

    def playout_action(self, *, rule="uniform_exclude_verbs", exclude_verbs=None, playout_seed=0) -> int:
        """The action index the declared playout rule picks at this branch's boundary (no effect; M41)."""
        return self._search._native.playout_action(
            self._branch_id, rule=rule, exclude_verbs=exclude_verbs, playout_seed=playout_seed
        )

    def fork(self) -> PublicBranch:
        branch_id = self._search._native.fork(self._branch_id)
        return PublicBranch(self._search, branch_id, (branch_id, 0))

    def actions(self) -> list[int]:
        return self._search._native.actions(self._branch_id)

    def close(self) -> None:
        self._search._native.close(self._branch_id)

    def public_search(
        self,
        research_seed: int,
        *,
        limits=None,
        max_live_branches=None,
        max_branches=None,
        max_steps_per_branch=None,
        max_total_steps=None,
        workers=1,
        observation="dense", capacity_preset=None, capacities=None,
    ) -> PublicSearch:
        if type(research_seed) is not int or not 0 <= research_seed < 2**64:
            raise _native.RunInputError("research_seed must be a nonnegative built-in u64 integer")
        child = self._search._native.public_search(
            self._branch_id,
            research_seed,
            **_limits(limits, max_live_branches=max_live_branches, max_branches=max_branches,
                      max_steps_per_branch=max_steps_per_branch, max_total_steps=max_total_steps),
            workers=workers, observation=observation, capacity_preset=capacity_preset, capacities=capacities,
        )
        return PublicSearch(child)


def open_public_search(env, research_seed, limits=None, *, workers=1,
                       observation="dense", capacity_preset=None, capacities=None, **legacy) -> PublicSearch:
    """Open a search at ``env``'s current Standard combat decision (``RunEnv`` and ``StandardCombatEnv``).

    ``limits`` is a mapping of any of the four limit names (missing names take the hard caps); ``None`` with no
    individual keywords means the hard caps. The four individual keywords remain accepted, all together.
    """
    if type(research_seed) is not int or not 0 <= research_seed < 2**64:
        raise _native.RunInputError("research_seed must be a nonnegative built-in u64 integer")
    from .standard_combat import StandardCombatEnv

    if not isinstance(env, (RunEnv, StandardCombatEnv)):
        _common.require_dense_combat(observation, capacity_preset, capacities)
    resolved = _limits(limits, **legacy)
    with env._guard:
        native = env._session.public_search(
            research_seed, workers=workers, observation=observation, capacity_preset=capacity_preset, capacities=capacities, **resolved
        )
    return PublicSearch(native)
