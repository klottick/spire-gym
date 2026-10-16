import pytest

from spire_lab.combat.content import SUPPORTED_CONTENT_ID
from spire_lab.combat.model import (
    FaultKind, FaultRecord, FaultedStateError, InvariantError, PendingEffect,
)
from spire_lab.combat.snapshot import (
    CombatSnapshot, SnapshotCompatibilityError, compare, restore, snapshot,
)


def test_snapshot_is_deep_clone_and_mutation_does_not_alias(minimal_state):
    saved = snapshot(minimal_state)
    minimal_state.player.powers["self-test"] = 2
    minimal_state.piles.hand.clear()
    minimal_state.card_ids_issued.clear()
    minimal_state.enemies.clear()
    minimal_state.pending.append(PendingEffect("self-test", None, None, ()))
    assert saved.state.player.powers == {}
    assert [c.instance_id for c in saved.state.piles.hand] == [1, 2, 3]
    assert saved.state.card_ids_issued == {1, 2, 3}
    assert len(saved.state.enemies) == 1 and saved.state.pending == []


def test_restore_clone_does_not_alias_snapshot(minimal_state):
    saved = snapshot(minimal_state)
    restored = restore(saved)
    restored.piles.hand.pop()
    restored.player.powers["self-test"] = 1
    restored.enemies[0].powers["self-test"] = 1
    restored.card_ids_issued.clear()
    restored.pending.append(PendingEffect("self-test", None, None, ()))
    assert len(saved.state.piles.hand) == 3 and saved.state.player.powers == {}
    assert saved.state.enemies[0].powers == {}
    assert saved.state.card_ids_issued == {1, 2, 3} and saved.state.pending == []


def test_restore_rejects_content_or_schema_mismatch(minimal_state):
    saved = snapshot(minimal_state)
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot("other-content", 1, saved.state))
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot(saved.content_id, 2, saved.state))
    wrong_state = snapshot(minimal_state).state
    wrong_state.content_id = "other-content"
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot(saved.content_id, 1, wrong_state))


def test_compare_checks_content_then_declared_field_order(minimal_state):
    other = restore(snapshot(minimal_state))
    other.phase = "other"
    other.player.hp -= 1
    assert compare(minimal_state, other).component == "phase"
    other.content_id = "other-content"
    assert compare(minimal_state, other).component == "content_id"


def test_faulted_snapshot_is_diagnostic_but_cannot_be_restored(minimal_state):
    minimal_state.fault = FaultRecord(FaultKind.INVARIANT, "partial", 1)
    minimal_state.card_ids_issued.clear()
    saved = snapshot(minimal_state)
    assert saved.state.fault == minimal_state.fault
    assert saved.state.card_ids_issued == set()
    with pytest.raises(FaultedStateError):
        restore(saved)


def test_restore_rejects_invalid_card_ownership(minimal_state):
    saved = snapshot(minimal_state)
    saved.state.card_ids_issued.clear()
    with pytest.raises(InvariantError):
        restore(saved)


def test_compare_uses_declared_component_precedence(minimal_state):
    other = restore(snapshot(minimal_state))
    other.turn += 1
    other.player.hp -= 1
    assert compare(minimal_state, other).component == "turn"


def test_restore_compatibility_precedes_fault_and_validation(minimal_state):
    saved = snapshot(minimal_state)
    saved.state.fault = FaultRecord(FaultKind.INVARIANT, "partial", 0)
    saved.state.card_ids_issued.clear()
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot("wrong", 2, saved.state), expected_content_id="expected")


def test_restore_uses_requested_content_id(minimal_state):
    saved = snapshot(minimal_state)
    with pytest.raises(SnapshotCompatibilityError):
        restore(saved, expected_content_id="another-supported-content")
