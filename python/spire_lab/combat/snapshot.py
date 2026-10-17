from copy import deepcopy
from dataclasses import dataclass

from spire_lab.combat.content import SUPPORTED_CONTENT_ID
from spire_lab.combat.model import (
    CombatState, FaultedStateError, validate_runtime_state,
)


_SCHEMA_VERSION = 2
_COMPONENTS = (
    "phase", "turn", "player", "enemies", "piles", "card_ids_issued", "rng",
    "pending", "current", "in_play", "resolved_effects", "public", "outcome", "fault", "hp_before_healing", "hp_after_cleanup",
)


class SnapshotCompatibilityError(Exception):
    pass


@dataclass(frozen=True)
class CombatSnapshot:
    content_id: str
    schema_version: int
    state: CombatState


@dataclass(frozen=True)
class Divergence:
    component: str
    left: object
    right: object


def snapshot(state: CombatState) -> CombatSnapshot:
    return CombatSnapshot(state.content_id, _SCHEMA_VERSION, deepcopy(state))


def restore(value: CombatSnapshot, *, expected_content_id: str = SUPPORTED_CONTENT_ID) -> CombatState:
    if (value.content_id != expected_content_id
            or value.state.content_id != expected_content_id
            or value.state.content_id != value.content_id
            or type(value.schema_version) is not int or value.schema_version != _SCHEMA_VERSION):
        raise SnapshotCompatibilityError("snapshot content or schema is incompatible")
    if value.state.fault is not None:
        raise FaultedStateError("faulted snapshot cannot be restored")
    validate_runtime_state(value.state)
    return deepcopy(value.state)


def compare(left: CombatState, right: CombatState) -> Divergence | None:
    if left.content_id != right.content_id:
        return Divergence("content_id", deepcopy(left.content_id), deepcopy(right.content_id))
    for component in _COMPONENTS:
        left_value = getattr(left, component)
        right_value = getattr(right, component)
        if left_value != right_value:
            return Divergence(component, deepcopy(left_value), deepcopy(right_value))
    return None
