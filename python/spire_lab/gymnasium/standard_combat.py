"""Standalone fresh-start combat sessions for the Standard profile."""

from typing import Any

from spire_lab import _native

from .ironclad import Observation, _CombatEnv


class StandardCombatEnv(_CombatEnv):
    """Independent native Standard combats selected from a finite catalog.

    Actions are indices into the current native legal mask. The decision token
    stays private to the facade and ensures an old integer cannot be replayed.
    """

    def __init__(
        self, entry_catalog: dict[str, Any], *, partition: str = "train"
    ) -> None:
        super().__init__(
            entry_catalog,
            partition=partition,
            native_session_type=_native.NativeStandardCombatSession,
        )
