import pytest

from spire_lab.combat.content import SUPPORTED_CONTENT_ID
from spire_lab.combat.model import (
    FaultKind, FaultRecord, FaultedStateError, InvariantError, PendingEffect, PowerState, TerminalOutcome,
)
from spire_lab.combat.snapshot import (
    CombatSnapshot, SnapshotCompatibilityError, compare, restore, snapshot,
)


def test_snapshot_is_deep_clone_and_mutation_does_not_alias(minimal_state):
    saved = snapshot(minimal_state)
    minimal_state.player.powers["self-test"] = PowerState(2)
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
    restored.player.powers["self-test"] = PowerState(1)
    restored.enemies[0].powers["self-test"] = PowerState(1)
    restored.card_ids_issued.clear()
    restored.pending.append(PendingEffect("self-test", None, None, ()))
    assert len(saved.state.piles.hand) == 3 and saved.state.player.powers == {}
    assert saved.state.enemies[0].powers == {}
    assert saved.state.card_ids_issued == {1, 2, 3} and saved.state.pending == []


def test_restore_rejects_content_or_schema_mismatch(minimal_state):
    saved = snapshot(minimal_state)
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot("other-content", 2, saved.state))
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot(saved.content_id, 1, saved.state))
    wrong_state = snapshot(minimal_state).state
    wrong_state.content_id = "other-content"
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot(saved.content_id, 2, wrong_state))


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


def test_schema1_rejected_before_fault_validation(minimal_state):
    minimal_state.fault = FaultRecord(FaultKind.INVARIANT, "partial", 1)
    minimal_state.card_ids_issued.clear()
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot(SUPPORTED_CONTENT_ID, 1, minimal_state))
    minimal_state.content_id = "spire-lab-combat-reference-v1"
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot(minimal_state.content_id, 2, minimal_state))


def test_busy_checkpoint_preserves_work_flags_and_detaches(minimal_state):
    state = minimal_state
    state.phase = "player_action"
    state.in_play = state.piles.hand.pop()
    state.current = PendingEffect("damage", state.in_play.instance_id, "enemy-1", (("amount", 8),))
    state.pending = [PendingEffect("finish_card", state.in_play.instance_id, None, ())]
    state.resolved_effects = 7
    state.enemies[0].powers["ritual"] = PowerState(3, True)
    saved = snapshot(state)
    restored = restore(saved)
    assert compare(state, restored) is None
    restored.enemies[0].powers["ritual"].skip_first = False
    restored.enemies[0].damage_entries["attack"].output = 9
    restored.current.payload = (("amount", 9),)
    restored.pending.clear()
    assert saved.state.enemies[0].powers["ritual"].skip_first is True
    assert saved.state.enemies[0].damage_entries["attack"].output == 6
    assert saved.state.current.payload == (("amount", 8),)
    assert len(saved.state.pending) == 1
    assert restored.in_play.instance_id == 3 and restored.resolved_effects == 7


@pytest.mark.parametrize("component", ["current", "in_play", "resolved_effects"])
def test_compare_new_components_precede_public_and_are_detached(minimal_state, component):
    other = restore(snapshot(minimal_state))
    values = {"current": PendingEffect("draw", "player", "player", (("amount", 1),)),
              "in_play": other.piles.hand[0], "resolved_effects": 3}
    setattr(other, component, values[component])
    other.public.revealed_order = ("x",)
    diff = compare(minimal_state, other)
    assert diff.component == component
    if component == "current":
        other.current.kind = "changed"
        assert diff.right.kind == "draw"


def test_restore_rejects_noninteger_schema_version(minimal_state):
    with pytest.raises(SnapshotCompatibilityError):
        restore(CombatSnapshot(SUPPORTED_CONTENT_ID, 2.0, minimal_state))


def test_pending_still_precedes_new_comparison_components(minimal_state):
    other = restore(snapshot(minimal_state))
    other.pending.append(PendingEffect("draw", "player", "player", (("amount", 1),)))
    other.current = PendingEffect("recycle", None, None, ())
    other.resolved_effects = 1
    assert compare(minimal_state, other).component == "pending"


def test_restore_preserves_zero_power_awaiting_queued_removal(minimal_state):
    minimal_state.phase = "round_end"
    minimal_state.enemies[0].powers["vulnerable"] = PowerState(0)
    minimal_state.pending = [PendingEffect("remove_power", "player", "enemy-1", (("power_id", "vulnerable"),))]
    restored = restore(snapshot(minimal_state))
    assert restored.enemies[0].powers["vulnerable"].amount == 0
    assert restored.pending == minimal_state.pending


@pytest.mark.parametrize("outcome,hp,before,after", [
    ("death", 40, 40, 40), ("death", 0, 1, 0), ("victory", 0, 0, 0), ("victory", 40, 30, 40),
])
def test_restore_rejects_inconsistent_terminal_hp(minimal_state, outcome, hp, before, after):
    minimal_state.phase = "terminal"
    minimal_state.outcome = TerminalOutcome(outcome)
    minimal_state.player.hp = hp
    minimal_state.enemies[0].hp = 0 if outcome == "victory" else 48
    minimal_state.hp_before_healing = before
    minimal_state.hp_after_cleanup = after
    with pytest.raises(InvariantError): restore(snapshot(minimal_state))


@pytest.mark.parametrize("outcome,hp,before", [("death", 0, 0), ("victory", 46, 40), ("victory", 50, 49)])
def test_restore_accepts_terminal_hp_and_retained_in_play(minimal_state, outcome, hp, before):
    minimal_state.phase = "terminal"
    minimal_state.outcome = TerminalOutcome(outcome)
    minimal_state.player.hp = hp
    minimal_state.enemies[0].hp = 0 if outcome == "victory" else 48
    minimal_state.hp_before_healing = before
    minimal_state.hp_after_cleanup = hp
    minimal_state.in_play = minimal_state.piles.hand.pop()
    restored = restore(snapshot(minimal_state))
    assert compare(minimal_state, restored) is None
