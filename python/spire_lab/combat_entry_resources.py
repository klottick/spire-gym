"""Static resource admission for owned ``standard-combat-v1`` entries."""

from __future__ import annotations

from typing import Any

SCHEMA_ID = "spire-lab-combat-entry-resources-v1"


def load() -> dict[str, Any]:
    """Return a fresh copy of the native resource descriptor."""
    from spire_lab import _native

    resources = _native.combat_entry_resources()
    if resources.get("schema_id") != SCHEMA_ID:
        raise ValueError(f"unexpected combat entry resources schema {resources.get('schema_id')!r}")
    return resources
