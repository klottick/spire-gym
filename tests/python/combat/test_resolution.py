from dataclasses import replace

import pytest

from spire_lab.combat import engine, rng
from spire_lab.combat.model import (
    FaultedStateError, FaultKind, PendingEffect,
    mark_fault, require_steppable,
)
from spire_lab.combat.rules import choose_enemy_move
from spire_lab.combat.snapshot import compare, restore, snapshot


def work(kind, **payload):
    return PendingEffect(kind, None, None, tuple(payload.items()))


def test_top_bottom_do_not_replace_current(minimal_state, monkeypatch):
    state = minimal_state
    state.phase = "player_action"
    original, tail = work("gain_block", amount=1), work("decision")
    top, bottom = work("gain_block", amount=2), work("gain_block", amount=3)
    state.pending = [original, tail]
    real_apply = engine._apply_effect

    def insert(state, effect):
        assert state.current is original
        state.pending.insert(0, top)
        state.pending.append(bottom)
        assert state.current is original

    monkeypatch.setattr(engine, "_apply_effect", insert)
    engine._dispatch_one(state)
    assert state.pending == [top, tail, bottom]
    assert state.current is None and state.resolved_effects == 1
    monkeypatch.setattr(engine, "_apply_effect", real_apply)


def test_checkpoint_preserves_same_step_budget(minimal_state):
    state = minimal_state
    state.phase = "player_action"
    state.pending = [PendingEffect("gain_block", None, "player", (("amount", 5),)), work("decision")]
    engine._dispatch_one(state)
    saved = snapshot(state)
    assert state.resolved_effects == 1 and state.current is None
    resumed = engine._resolve(restore(saved))
    direct = engine._resolve(state)
    assert resumed.status == direct.status == "decision"
    assert direct.state.resolved_effects == 2
    assert compare(resumed.state, direct.state) is None


@pytest.mark.parametrize("count", [99_999, 100_000])
def test_guard_99999_and_100000(minimal_state, monkeypatch, count):
    state = minimal_state
    state.phase = "player_action"
    state.pending = [work("gain_block", amount=0)]
    real_apply = engine._apply_effect

    def repeat(state, effect):
        if state.resolved_effects == count:
            real_apply(state, work("decision"))
        else:
            state.pending.append(work("gain_block", amount=0))

    monkeypatch.setattr(engine, "_apply_effect", repeat)
    result = engine._resolve(state)
    assert state.resolved_effects == count
    if count == 99_999:
        assert result.status == "decision" and state.fault is None
        assert state.current is None
    else:
        assert result.status == "fault"
        assert state.fault.kind is FaultKind.EFFECT_GUARD
        assert state.fault.resolved_effects == 100_000
        assert state.current.kind == "gain_block"
        assert state.phase == "player_action"


def test_partial_fault_is_first_and_not_loss(minimal_state, monkeypatch):
    state = minimal_state
    state.phase = "player_action"
    failing = work("gain_block", amount=0)
    state.pending = [failing, work("decision")]

    def fail(state, effect):
        state.player.hp -= 7
        raise ValueError("injected after HP mutation")

    monkeypatch.setattr(engine, "_apply_effect", fail)
    result = engine._resolve(state)
    assert result.status == "fault" and result.state is state
    assert state.player.hp == 33 and state.outcome is None
    assert state.current is failing and len(state.pending) == 1
    first = state.fault
    assert first.kind is FaultKind.INVARIANT and first.resolved_effects == 1
    assert "injected after HP mutation" in first.message
    mark_fault(state, FaultKind.EFFECT_GUARD, "later", 100_000)
    assert engine._resolve(state).status == "fault"
    assert state.fault is first and state.player.hp == 33 and state.resolved_effects == 1
    with pytest.raises(FaultedStateError):
        require_steppable(state)


@pytest.mark.parametrize("kind", ["unknown"])
def test_unowned_kinds_are_explicitly_unsupported(minimal_state, kind):
    state = minimal_state
    state.phase = "player_action"
    effect = work(kind, **({"amount": 1} if kind == "draw" else {}))
    state.pending = [effect]
    result = engine._resolve(state)
    assert result.status == "fault" and state.fault.kind is FaultKind.UNSUPPORTED
    assert state.current is effect and state.resolved_effects == 1


def test_finish_card_without_in_play_ownership_is_invariant_fault(minimal_state):
    state = minimal_state
    state.phase = "player_action"
    effect = work("finish_card")
    state.pending = [effect]
    result = engine._resolve(state)
    assert result.status == "fault" and state.fault.kind is FaultKind.INVARIANT
    assert state.current is effect and state.resolved_effects == 1
    assert [c.instance_id for c in state.piles.hand] == [1, 2, 3]
    assert not state.piles.discard


def test_empty_busy_queue_is_not_a_decision(minimal_state):
    minimal_state.phase = "player_action"
    assert engine._resolve(minimal_state).status == "fault"
    assert minimal_state.fault.kind is FaultKind.INVARIANT
    assert minimal_state.resolved_effects == 0 and minimal_state.outcome is None


def test_existing_current_is_not_retried(minimal_state):
    state = minimal_state
    state.phase = "player_action"
    state.current = work("gain_block", amount=5)
    state.resolved_effects = 1
    assert engine._resolve(state).status == "fault"
    assert state.player.block == 0 and state.resolved_effects == 1


def test_process_termination_is_not_caught(minimal_state, monkeypatch):
    state = minimal_state
    state.phase = "player_action"
    state.pending = [work("gain_block", amount=0)]

    def interrupt(state, effect):
        raise KeyboardInterrupt

    monkeypatch.setattr(engine, "_apply_effect", interrupt)
    with pytest.raises(KeyboardInterrupt):
        engine._resolve(state)
    assert state.fault is None and state.current is not None
    assert state.resolved_effects == 1


def test_rejection_exhaustion_keeps_latest_cursor(minimal_state, monkeypatch):
    state = minimal_state
    state.phase = "enemy_turn"
    state.rng = replace(state.rng, enemy_ai=replace(state.rng.enemy_ai, word_index=2**64 - 1))
    shuffle = state.rng.shuffle
    state.pending = [work("roll_move")]
    # A valid final u64 above the rejection threshold for bound 100.
    monkeypatch.setattr(rng, "_hmac_sha256", lambda key, message: b"\xff" * 32)

    def sample(state, effect):
        choose_enemy_move("jaw_worm", ("chomp",), state.rng.enemy_ai)

    monkeypatch.setattr(engine, "_apply_effect", sample)
    assert engine._resolve(state).status == "fault"
    assert state.fault.kind is FaultKind.RNG_EXHAUSTED
    assert state.rng.enemy_ai.word_index == 2**64
    assert state.rng.shuffle is shuffle and shuffle.word_index == 0
    assert state.outcome is None and state.current.kind == "roll_move"


def test_shuffle_exhaustion_updates_only_shuffle(minimal_state, monkeypatch):
    state = minimal_state
    state.phase = "player_action"
    state.rng = replace(state.rng, shuffle=replace(state.rng.shuffle, word_index=2**64))
    ai = state.rng.enemy_ai
    state.pending = [work("recycle")]

    def sample(state, effect):
        rng.uniform_below(state.rng.shuffle, 2)

    monkeypatch.setattr(engine, "_apply_effect", sample)
    assert engine._resolve(state).status == "fault"
    assert state.fault.kind is FaultKind.RNG_EXHAUSTED
    assert state.rng.shuffle.word_index == 2**64 and state.rng.enemy_ai is ai


def test_ready_resolve_does_not_reset_counter(minimal_state):
    minimal_state.resolved_effects = 17
    assert engine._resolve(minimal_state).status == "decision"
    assert minimal_state.resolved_effects == 17


def test_checkpoint_at_guard_boundary_does_not_gain_budget(minimal_state):
    state = minimal_state
    state.phase = "player_action"
    state.resolved_effects = 99_998
    state.pending = [PendingEffect("gain_block", None, "player", (("amount", 5),)), work("decision")]
    engine._dispatch_one(state)
    resumed = restore(snapshot(state))
    assert engine._resolve(resumed).status == "fault"
    assert resumed.resolved_effects == 100_000 and resumed.player.block == 5
    assert resumed.fault.kind is FaultKind.EFFECT_GUARD and resumed.current.kind == "decision"


def test_foreign_exhaustion_cursor_is_an_invariant_fault(minimal_state, monkeypatch):
    state = minimal_state
    state.phase = "player_action"
    original = state.rng
    foreign = replace(state.rng.shuffle, key=b"\xff" * 32, word_index=2**64)
    state.pending = [work("recycle")]

    def fail(state, effect):
        raise rng.RngExhaustedError(foreign)

    monkeypatch.setattr(engine, "_apply_effect", fail)
    assert engine._resolve(state).status == "fault"
    assert state.fault.kind is FaultKind.INVARIANT and state.rng is original
