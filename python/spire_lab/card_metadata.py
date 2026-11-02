"""Opt-in static public card metadata (``spire-lab-card-metadata-v1``).

The native descriptor is generated from the engine's card owners and carries no
seed, RNG or hidden state. It describes printed faces only: live observations,
card views and legal actions remain authoritative for current cost, upgrades,
targets and playability, and a row never implies admission in a profile (join
through that session's ``environment_spec()``). Learn owns any encoding of it.
"""

from __future__ import annotations

from typing import Any

SCHEMA_ID = "spire-lab-card-metadata-v1"


def load() -> dict[str, Any]:
    """Return the native descriptor; requires an extension built with it."""
    from spire_lab import _native

    metadata = _native.card_metadata()
    if metadata.get("schema_id") != SCHEMA_ID:
        raise ValueError(f"unexpected card metadata schema {metadata.get('schema_id')!r}")
    return metadata


def by_id(metadata: dict[str, Any]) -> dict[str, dict[str, Any]]:
    """Index rows by card id, the same strings as ``vocabularies['card']``."""
    return {row["id"]: row for row in metadata["cards"]}


def render_card(
    metadata: dict[str, Any],
    card_id: str,
    upgrades: int = 0,
    *,
    observed_cost: int | None = None,
) -> str:
    """Example renderer: one line from metadata plus optional public observations.

    ``observed_cost`` is the live public cost; when given it wins over the face.
    Unknown ids and upgrades past the static faces are reported, not guessed.
    """
    row = by_id(metadata).get(card_id)
    if row is None:
        return f"{card_id}: unknown card"
    faces = row["faces"]
    face = faces[min(upgrades, len(faces) - 1)] if faces else None
    name = card_id + ("+" * min(upgrades, 1)) + (str(upgrades) if upgrades > 1 else "")
    if face is None:
        return f"{name}: {row['color']} {row['type']}, not a physical card"
    cost = face["cost"] if observed_cost is None else f"{observed_cost} (printed {face['cost']})"
    target = {"enemy": "targets an enemy", "untargeted": "no target"}.get(face["target"], "target depends on state")
    keywords = ", ".join(face["keywords"]) or "none"
    note = "" if upgrades <= face["upgrades"] else "; later upgrades not described statically"
    return f"{name}: {row['color']} {row['rarity']} {row['type']}, cost {cost}, {target}, keywords {keywords}{note}"
