from spire_lab.combat.content import (
    SUPPORTED_CARD_IDS, SUPPORTED_CONTENT_ID, SUPPORTED_ENCOUNTER_IDS,
    SUPPORTED_RELIC_IDS,
)
import pytest
from spire_lab.combat.model import (
    CardInstance, EndTurn, FaultKind, FaultRecord, InvariantError,
    PendingEffect, Piles, PlayCard, RngState, TerminalOutcome, validate_state,
)


def test_minimal_state_owns_typed_values(minimal_state):
    assert isinstance(minimal_state.piles, Piles)
    assert [c.instance_id for c in minimal_state.piles.hand] == [1, 2, 3]
    assert minimal_state.player.powers == {}
    assert minimal_state.enemies[0].powers == {}
    assert minimal_state.content_id == SUPPORTED_CONTENT_ID
    validate_state(minimal_state)


def test_instance_ids_unique_across_all_piles(minimal_state):
    minimal_state.piles.draw.append(CardInstance(1, "strike", False))
    with pytest.raises(InvariantError):
        validate_state(minimal_state)


def test_piles_and_pending_keep_order(minimal_state):
    minimal_state.pending[:] = [PendingEffect("a", 1, "enemy-1", ()),
                                PendingEffect("b", "enemy-1", 2, ())]
    assert [effect.kind for effect in minimal_state.pending] == ["a", "b"]


def test_fault_is_not_terminal_outcome():
    assert FaultRecord(FaultKind.INVARIANT, "x", 0) != TerminalOutcome("death")


def test_action_union_is_semantic():
    assert isinstance(PlayCard(1, "enemy-1"), PlayCard)
    assert isinstance(EndTurn(), EndTurn)


def test_content_manifest_is_exact_scope():
    assert SUPPORTED_CONTENT_ID == "spire-lab-combat-reference-v1"
    assert SUPPORTED_CARD_IDS == {"strike", "defend", "bash"}
    assert SUPPORTED_ENCOUNTER_IDS == {"cultist", "jaw_worm"}
    assert SUPPORTED_RELIC_IDS == {"burning_blood"}


def test_model_rejects_foreign_mutable_objects(minimal_state):
    minimal_state.pending.append(PendingEffect("x", None, None, (("x", object()),)))
    with pytest.raises(InvariantError):
        validate_state(minimal_state)


def test_rng_slot_is_opaque_provisional_value():
    value = RngState((1, "x", True))
    assert value.slots == (1, "x", True)


def test_model_rejects_bool_for_integer_state_field(minimal_state):
    minimal_state.turn = True
    with pytest.raises(InvariantError):
        validate_state(minimal_state)


def test_model_rejects_unissued_pile_id(minimal_state):
    minimal_state.piles.discard.append(CardInstance(4, "strike", False))
    with pytest.raises(InvariantError):
        validate_state(minimal_state)


def test_model_rejects_foreign_enemy_payload(minimal_state):
    minimal_state.enemies[0].powers["bad"] = object()
    with pytest.raises(InvariantError):
        validate_state(minimal_state)
