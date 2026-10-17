import pytest

from spire_lab.combat import engine
from spire_lab.combat.model import (
    CardInstance, EndTurn, FaultedStateError, FaultKind, InvalidActionError,
    InvariantError, PendingEffect, Piles, PlayCard, PowerState, mark_fault,
)
from spire_lab.combat.snapshot import compare, snapshot


def test_bash_then_strike_calculates_before_new_vulnerable(minimal_state):
    state = minimal_state
    rng = state.rng
    assert engine.step(state, PlayCard(3, "enemy-1")).status == "decision"
    assert state.enemies[0].hp == 40
    assert state.enemies[0].powers["vulnerable"] == PowerState(2)
    assert state.player.energy == 1
    assert engine.step(state, PlayCard(1, "enemy-1")).status == "decision"
    assert state.enemies[0].hp == 31 and state.player.energy == 0
    assert [c.instance_id for c in state.piles.discard] == [3, 1]
    assert state.in_play is None and state.rng == rng


@pytest.mark.parametrize("upgraded,block", [(False, 5), (True, 8)])
def test_defend_finishes_into_discard(scenario_factory, upgraded, block):
    card = CardInstance(1, "defend", upgraded)
    state = engine.initial_combat(scenario_factory(piles=Piles([card], [], [], [])))
    owned = state.piles.hand[0]
    assert engine.step(state, PlayCard(1, None)).status == "decision"
    assert state.player.block == block and state.player.energy == 2
    assert state.piles.discard == [card] and state.piles.discard[0] is owned
    assert not state.piles.hand and state.in_play is None


def test_upgraded_strike_against_vulnerable(scenario_factory):
    state = engine.initial_combat(scenario_factory(
        piles=Piles([CardInstance(1, "strike", True)], [], [], [])))
    state.enemies[0].powers["vulnerable"] = PowerState(2)
    assert engine.step(state, PlayCard(1, "enemy-1")).status == "decision"
    assert state.enemies[0].hp == 35


def test_card_use_owns_card_and_spends_before_payload_dispatch(minimal_state, monkeypatch):
    state = minimal_state
    card = state.piles.hand[2]
    state.resolved_effects = 99_999
    dispatch = engine._dispatch_one
    observed = []

    def inspect_then_dispatch(owned):
        if not observed:
            assert owned.phase == "player_action" and owned.resolved_effects == 0
            assert owned.in_play is card and card not in owned.piles.hand
            assert owned.player.energy == 1 and owned.enemies[0].hp == 48
            assert owned.pending == [
                PendingEffect("damage", 3, "enemy-1", (("amount", 8),)),
                PendingEffect("apply_power", 3, "enemy-1", (("power_id", "vulnerable"), ("amount", 2))),
                PendingEffect("finish_card", 3, None, ()),
                PendingEffect("decision", None, None, ()),
            ]
        observed.append(owned.pending[0].kind)
        dispatch(owned)

    monkeypatch.setattr(engine, "_dispatch_one", inspect_then_dispatch)
    assert engine.step(state, PlayCard(3, "enemy-1")).status == "decision"
    assert observed == ["damage", "apply_power", "finish_card", "decision"]
    assert state.piles.discard[0] is card and state.resolved_effects == 4


@pytest.mark.parametrize("action", [
    PlayCard(True, "enemy-1"), PlayCard(False, "enemy-1"),
    PlayCard("1", "enemy-1"), PlayCard(None, "enemy-1"),
    PlayCard(999, "enemy-1"), PlayCard(1, None), PlayCard(1, True),
    PlayCard(1, 1), PlayCard(1, "player"), PlayCard(1, "unrelated"),
    PlayCard(2, "enemy-1"), PlayCard(2, False), object(), None,
])
def test_invalid_action_is_pre_mutation(minimal_state, action):
    state = minimal_state
    state.resolved_effects = 17
    before = snapshot(state)
    with pytest.raises(InvalidActionError):
        engine.step(state, action)
    assert compare(state, before.state) is None


@pytest.mark.parametrize("condition", ["spent", "energy", "busy", "pending", "current", "in_play", "terminal"])
def test_unavailable_choice_is_pre_mutation(minimal_state, condition):
    state = minimal_state
    if condition == "spent":
        engine.step(state, PlayCard(1, "enemy-1"))
    elif condition == "energy":
        state.player.energy = 0
    elif condition == "busy":
        state.phase = "player_action"
    elif condition == "pending":
        state.pending = [PendingEffect("decision", None, None, ())]
    elif condition == "current":
        state.current = PendingEffect("decision", None, None, ())
    elif condition == "in_play":
        state.in_play = state.piles.hand.pop()
    else:
        state.phase = "player_action"
        state.pending = [PendingEffect("damage", "player", "enemy-1", (("amount", 48),))]
        engine._resolve(state)
    before = snapshot(state)
    with pytest.raises(InvalidActionError):
        engine.step(state, PlayCard(1, "enemy-1"))
    assert compare(state, before.state) is None


def test_foreign_action_subclasses_are_rejected(minimal_state):
    class ForeignPlay(PlayCard):
        pass

    class ForeignEnd(EndTurn):
        pass

    for action in (ForeignPlay(1, "enemy-1"), ForeignEnd()):
        before = snapshot(minimal_state)
        with pytest.raises(InvalidActionError):
            engine.step(minimal_state, action)
        assert compare(minimal_state, before.state) is None


def test_faulted_step_is_rejected(minimal_state):
    mark_fault(minimal_state, FaultKind.INVARIANT, "test")
    before = snapshot(minimal_state)
    with pytest.raises(FaultedStateError):
        engine.step(minimal_state, EndTurn())
    assert compare(minimal_state, before.state) is None


def test_end_turn_returns_next_ready_decision(minimal_state):
    result = engine.step(minimal_state, EndTurn())
    assert result.status == "decision"
    assert minimal_state.turn == 2 and minimal_state.fault is None
    assert minimal_state.current is None and not minimal_state.pending


def test_card_in_play_is_owned_until_finish_and_can_resume(minimal_state):
    from spire_lab.combat import effects
    from spire_lab.combat.snapshot import restore

    state = minimal_state
    state.phase = "player_action"
    state.in_play = state.piles.hand.pop()
    card = state.in_play
    state.pending = [PendingEffect("finish_card", card.instance_id, None, ()),
                     PendingEffect("decision", None, None, ())]
    resumed = restore(snapshot(state))
    engine._dispatch_one(state)
    assert state.in_play is None and state.piles.discard[0] is card
    assert engine._resolve(state).status == engine._resolve(resumed).status == "decision"
    assert compare(state, resumed) is None
    state.phase = "player_action"
    state.in_play = state.piles.hand.pop()
    before = snapshot(state)
    with pytest.raises(InvariantError):
        effects.apply_effect(state, PendingEffect("finish_card", 999, None, ()))
    assert compare(state, before.state) is None
