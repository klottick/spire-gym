from dataclasses import FrozenInstanceError, fields, replace

import pytest

from spire_lab.combat import engine, view
from spire_lab.combat.model import (
    CardInstance, EndTurn, FaultKind, FaultedStateError, InvalidActionError,
    PlayCard, PowerState, mark_fault,
)
from spire_lab.combat.rng import StreamId, make_stream
from spire_lab.combat.snapshot import compare, restore, snapshot


def test_legal_actions_hand_order_then_end_turn(minimal_state):
    state = minimal_state
    before = snapshot(state)
    assert view.legal_actions(state) == (
        PlayCard(1, "enemy-1"), PlayCard(2, None), PlayCard(3, "enemy-1"), EndTurn())
    assert compare(state, before.state) is None
    state.player.energy = 1
    assert view.legal_actions(state) == (PlayCard(1, "enemy-1"), PlayCard(2, None), EndTurn())
    state.player.energy = 0
    assert view.legal_actions(state) == (EndTurn(),)


def test_public_view_detached_frozen_and_ordered(minimal_state):
    state = minimal_state
    state.player.powers = {"vulnerable": PowerState(2), "strength": PowerState(3)}
    state.enemies[0].powers = {"vulnerable": PowerState(2), "ritual": PowerState(3, skip_first=True)}
    from spire_lab.combat.effects import refresh_enemy_values
    refresh_enemy_values(state)
    state.piles.discard.append(state.piles.hand.pop(0))
    state.piles.exhaust.append(state.piles.hand.pop(0))
    public = view.public_view(state)
    assert public.player.powers == (("strength", 3), ("vulnerable", 2))
    assert public.enemies[0].powers == (("ritual", 3), ("vulnerable", 2))
    assert tuple(c.instance_id for c in public.hand) == (3,)
    assert tuple(c.instance_id for c in public.discard) == (1,)
    assert tuple(c.instance_id for c in public.exhaust) == (2,)
    assert public.enemies[0].intent_id == "incantation"
    assert public.enemies[0].intent_damage is None
    assert public.draw_count == 0 and public.outcome is None
    assert {f.name for f in fields(public)} == {
        "content_id", "turn", "player", "enemies", "hand", "draw_count",
        "discard", "exhaust", "revealed_order", "outcome", "hp_before_healing", "hp_after_cleanup"}
    assert {f.name for f in fields(public.enemies[0])} == {
        "instance_id", "enemy_id", "hp", "max_hp", "block", "powers", "intent_id", "intent_damage"}
    for obj, field, value in [(public, "turn", 99), (public.player, "hp", 0),
                              (public.enemies[0], "hp", 0), (public.hand[0], "card_id", "strike")]:
        with pytest.raises(FrozenInstanceError):
            setattr(obj, field, value)
    state.player.powers["strength"].amount = 8
    state.player.hp = 1
    state.piles.hand.clear()
    assert public.player.hp == 40 and public.player.powers[0] == ("strength", 3)
    assert public.hand[0].card_id == "bash"


def test_public_hidden_state_noninterference(minimal_state):
    a = minimal_state
    a.piles.draw = [CardInstance(4, "strike", True), CardInstance(5, "defend", False)]
    a.card_ids_issued.update((4, 5))
    b = restore(snapshot(a))
    b.piles.draw.reverse()
    b.rng = replace(b.rng,
        shuffle=make_stream(bytes([1]) * 32, "train", "private-episode",
                            StreamId("combat.shuffle", ("hidden",))),
        enemy_ai=make_stream(bytes([1]) * 32, "train", "private-episode",
                             StreamId("combat.enemy.ai", ("hidden", "secret"))))
    b.resolved_effects = 81
    assert view.public_view(a) == view.public_view(b)
    assert view.legal_actions(a) == view.legal_actions(b)
    assert view.public_view(a).draw_count == 2


def test_faulted_projection_and_enumeration_reject(minimal_state):
    mark_fault(minimal_state, FaultKind.INVARIANT, "test")
    for project in (view.public_view, view.legal_actions):
        with pytest.raises(FaultedStateError):
            project(minimal_state)


def test_busy_state_is_not_a_decision(minimal_state):
    minimal_state.phase = "player_action"
    with pytest.raises(InvalidActionError):
        view.legal_actions(minimal_state)
