"""Public-belief combat search over an owned native arena."""

from __future__ import annotations

from operator import index
from typing import Any

import numpy as np

from spire_lab import _native

from .run import Observation


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

    def limits(self) -> dict[str, int]:
        return dict(self.spec()["limits"])

    def close(self) -> None:
        self._native.close()


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
    ) -> PublicSearch:
        if type(research_seed) is not int or not 0 <= research_seed < 2**64:
            raise _native.RunInputError("research_seed must be a nonnegative built-in u64 integer")
        child = self._search._native.public_search(
            self._branch_id,
            research_seed,
            **_limits(limits, max_live_branches=max_live_branches, max_branches=max_branches,
                      max_steps_per_branch=max_steps_per_branch, max_total_steps=max_total_steps),
        )
        return PublicSearch(child)


def open_public_search(env, research_seed, limits=None, **legacy) -> PublicSearch:
    """Open a search at ``env``'s current Standard combat decision (``RunEnv`` and ``StandardCombatEnv``).

    ``limits`` is a mapping of any of the four limit names (missing names take the hard caps); ``None`` with no
    individual keywords means the hard caps. The four individual keywords remain accepted, all together.
    """
    if type(research_seed) is not int or not 0 <= research_seed < 2**64:
        raise _native.RunInputError("research_seed must be a nonnegative built-in u64 integer")
    resolved = _limits(limits, **legacy)
    with env._guard:
        native = env._session.public_search(research_seed, **resolved)
    return PublicSearch(native)
