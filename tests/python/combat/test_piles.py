from dataclasses import replace

import pytest

from spire_lab.combat import engine
from spire_lab.combat.model import (
    CardInstance, FaultKind, InvariantError, PendingEffect, Piles, validate_state,
)
from spire_lab.combat.snapshot import compare, restore, snapshot


def work(kind, **payload):
    return PendingEffect(kind, None, None, tuple(payload.items()))


def pile_state(state, *, hand=(), draw=(), discard=()):
    def cards(ids):
        return [CardInstance(i, "strike", False) for i in ids]

    state.piles = Piles(cards(hand), cards(draw), cards(discard), [])
    state.card_ids_issued = set(hand) | set(draw) | set(discard)
    state.phase = "player_action"
    validate_state(state)
    return state


def resolve(state, *effects):
    state.pending = [*effects, work("decision")]
    result = engine._resolve(state)
    assert result.status == "decision", state.fault
    validate_state(state)
    return state


def ids(cards):
    return [c.instance_id for c in cards]


def test_draw_top_is_storage_end(minimal_state):
    state = pile_state(minimal_state, draw=[1, 2, 3])
    cards = state.piles.draw.copy()
    resolve(state, work("draw", amount=2))
    assert ids(state.piles.hand) == [3, 2]
    assert ids(state.piles.draw) == [1]
    assert state.piles.hand[0] is cards[2] and state.piles.hand[1] is cards[1]
    assert state.rng.shuffle.word_index == state.rng.enemy_ai.word_index == 0


def test_discard_all_reverses_hand_storage(minimal_state):
    state = pile_state(minimal_state, hand=[1, 2, 3], discard=[9])
    cards = state.piles.hand.copy()
    resolve(state, work("discard_hand"))
    assert state.piles.hand == []
    assert ids(state.piles.discard) == [9, 3, 2, 1]
    assert state.piles.discard[1] is cards[2]


def test_recycle_fixed_project_order(minimal_state):
    state = pile_state(minimal_state, discard=[1, 2, 3])
    state.pending = [work("recycle"), work("draw", amount=3), work("decision")]
    ai = state.rng.enemy_ai
    engine._dispatch_one(state)
    assert state.fault is None
    assert ids(state.piles.draw) == [2, 3, 1]
    assert state.piles.discard == []
    assert state.rng.shuffle.word_index == 2 and state.rng.enemy_ai is ai
    assert engine._resolve(state).status == "decision"
    assert ids(state.piles.hand) == [1, 3, 2]


def test_split_draw_existing_then_recycle_then_residual(minimal_state):
    state = pile_state(minimal_state, draw=[4], discard=[1, 2, 3])
    decision = work("decision")
    state.pending = [work("draw", amount=3), decision]
    engine._dispatch_one(state)
    assert state.fault is None
    assert state.pending == [work("draw", amount=1), work("recycle"),
                             work("draw", amount=2), decision]
    assert ids(state.piles.draw) == [4] and state.piles.hand == []
    engine._dispatch_one(state)
    assert ids(state.piles.hand) == [4] and state.piles.draw == []
    assert state.public.revealed_order == ("4",)
    resumed = restore(snapshot(state))
    assert engine._resolve(state).status == "decision"
    assert engine._resolve(resumed).status == "decision"
    assert compare(state, resumed) is None
    assert ids(state.piles.hand) == [4, 1, 3]
    assert ids(state.piles.draw) == [2] and state.piles.discard == []
    assert state.public.revealed_order == ("4", "1", "3")


def test_hand_cap_and_deck_exhaustion(minimal_state):
    state = pile_state(minimal_state, hand=range(1, 10), draw=[10], discard=[11, 12])
    resolve(state, work("draw", amount=5))
    assert ids(state.piles.hand) == list(range(1, 11))
    assert ids(state.piles.discard) == [11, 12]
    assert state.rng.shuffle.word_index == 0
    state.phase = "player_action"
    state.piles.exhaust.extend(state.piles.hand)
    state.piles.hand.clear()
    state.piles.exhaust.extend(state.piles.discard)
    state.piles.discard.clear()
    resolve(state, work("draw", amount=5))
    assert state.piles.hand == [] and len(state.piles.exhaust) == 12
    assert state.public.revealed_order == ("10",)


@pytest.mark.parametrize("discard", [[], [1]])
def test_singleton_recycle_consumes_no_words(minimal_state, discard):
    state = pile_state(minimal_state, hand=[2], discard=discard)
    state.rng = replace(state.rng, shuffle=replace(state.rng.shuffle, word_index=2**64))
    original = state.rng.shuffle
    resolve(state, work("recycle"))
    assert ids(state.piles.draw) == discard and state.piles.discard == []
    assert state.rng.shuffle is original


def test_recycle_exhaustion_preserves_piles_and_latest_stream(minimal_state):
    state = pile_state(minimal_state, discard=[1, 2, 3, 4])
    state.rng = replace(state.rng, shuffle=replace(state.rng.shuffle, word_index=2**64 - 1))
    ai = state.rng.enemy_ai
    discard = state.piles.discard
    original = discard.copy()
    effect = work("recycle")
    state.pending = [effect, work("decision")]
    assert engine._resolve(state).status == "fault"
    assert state.fault.kind is FaultKind.RNG_EXHAUSTED
    assert state.rng.shuffle.word_index == 2**64 and state.rng.enemy_ai is ai
    assert state.piles.discard is discard and state.piles.discard == original
    assert all(a is b for a, b in zip(discard, original))
    assert state.piles.draw == [] and state.piles.hand == []
    assert state.current is effect and state.pending == [work("decision")]
    assert state.outcome is None
    validate_state(state)


def test_draw_records_public_reveal_order(minimal_state):
    state = pile_state(minimal_state, draw=[1, 2, 3])
    resolve(state, work("draw", amount=1), work("draw", amount=1))
    assert state.public.revealed_order == ("3", "2")
    assert ids(state.piles.draw) == [1]


def test_draw_recomputes_cap_and_discard_at_execution(minimal_state):
    state = pile_state(minimal_state, hand=range(1, 11))
    resolve(state, work("discard_hand"), work("draw", amount=2))
    assert len(state.piles.hand) == 2 and len(state.piles.draw) == 8
    assert state.piles.discard == [] and state.rng.shuffle.word_index >= 9


@pytest.mark.parametrize("in_play", [False, True])
def test_discard_card_moves_only_owned_source(minimal_state, in_play):
    from spire_lab.combat.piles import discard_card

    state = pile_state(minimal_state, hand=[1, 2], discard=[3])
    card = state.piles.hand[0]
    if in_play:
        state.in_play = state.piles.hand.pop(0)
    discard_card(state, 1, end_turn=False)
    assert ids(state.piles.hand) == [2] and state.in_play is None
    assert ids(state.piles.discard) == [3, 1] and state.piles.discard[-1] is card
    validate_state(state)
    with pytest.raises(InvariantError):
        discard_card(state, 1, end_turn=False)
    assert ids(state.piles.discard) == [3, 1]


@pytest.mark.parametrize("card_id", [True, 99])
def test_discard_card_rejects_invalid_owner_without_mutation(minimal_state, card_id):
    from spire_lab.combat.piles import discard_card

    before = snapshot(minimal_state)
    with pytest.raises(InvariantError):
        discard_card(minimal_state, card_id, end_turn=True)
    assert snapshot(minimal_state) == before
