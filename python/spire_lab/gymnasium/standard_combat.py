"""Standalone fresh-start combat sessions for the Standard profile."""

from typing import Any

from spire_lab import _native

from .ironclad import Observation, _CombatEnv
from .public_search import PublicSearch, open_public_search


class StandardCombatEnv(_CombatEnv):
    """Independent native Standard combats selected from a finite catalog.

    Actions are indices into the current native legal mask. The decision token
    stays private to the facade and ensures an old integer cannot be replayed.
    """

    def __init__(
        self, entry_catalog: dict[str, Any], *, records=None, partition: str = "train"
    ) -> None:
        super().__init__(
            entry_catalog,
            partition=partition,
            native_session_type=_native.NativeStandardCombatSession,
            records=records,
            supports_records=True,
        )

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
        """Open bounded public-belief search at the current combat decision."""
        return open_public_search(
            self, research_seed, limits, max_live_branches=max_live_branches, max_branches=max_branches,
            max_steps_per_branch=max_steps_per_branch, max_total_steps=max_total_steps,
        )
