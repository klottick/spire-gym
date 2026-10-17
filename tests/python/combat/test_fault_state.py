import pytest

from spire_lab.combat.model import (
    FaultKind, FaultRecord, FaultedStateError, TerminalOutcome, mark_fault, require_steppable,
)


def test_mark_fault_preserves_partial_fields_and_first_record(minimal_state):
    mark_fault(minimal_state, FaultKind.UNSUPPORTED, "first", 3)
    partial = (minimal_state.player.hp, list(minimal_state.piles.hand), set(minimal_state.card_ids_issued))
    mark_fault(minimal_state, FaultKind.INVARIANT, "second", 9)
    assert minimal_state.fault == FaultRecord(FaultKind.UNSUPPORTED, "first", 3)
    assert (minimal_state.player.hp, minimal_state.piles.hand, minimal_state.card_ids_issued) == partial
    with pytest.raises(FaultedStateError):
        require_steppable(minimal_state)


def test_require_steppable_does_not_apply_terminal_policy(minimal_state):
    minimal_state.outcome = TerminalOutcome("victory")
    require_steppable(minimal_state)


def test_rng_exhaustion_is_distinct_and_preserves_first_fault(minimal_state):
    mark_fault(minimal_state, FaultKind.RNG_EXHAUSTED, "exhausted", 5)
    mark_fault(minimal_state, FaultKind.EFFECT_GUARD, "later", 100000)
    assert minimal_state.fault == FaultRecord(FaultKind.RNG_EXHAUSTED, "exhausted", 5)
    assert minimal_state.outcome is None
    with pytest.raises(FaultedStateError): require_steppable(minimal_state)
