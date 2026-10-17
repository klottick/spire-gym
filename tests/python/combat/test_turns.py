from dataclasses import replace

import pytest

from spire_lab.combat import effects, engine, rng, rules
from spire_lab.combat.model import (
    CardInstance, EndTurn, EnemySetup, FaultKind, PendingEffect, Piles,
    PlayCard, PowerState, mark_fault, validate_runtime_state,
)
from spire_lab.combat.snapshot import compare, restore, snapshot


def work(kind, source=None, target=None, **payload):
    return PendingEffect(kind, source, target, tuple(payload.items()))


def jaw_state(scenario_factory, *, defend=False):
    return engine.initial_combat(scenario_factory(
        encounter_id="jaw_worm",
        enemies=(EnemySetup("enemy-1", "jaw_worm", 40, 40, 0, {}, "chomp", ("chomp",)),),
        piles=Piles([CardInstance(1, "defend" if defend else "strike", defend)], [], [], []),
    ))


def test_cultist_ritual_and_round_growth(minimal_state):
    state = minimal_state
    enemy = state.enemies[0]
    entries = dict(enemy.damage_entries)
    for turn, hp, preview, strength in [(2, 40, 6, None), (3, 34, 9, 3), (4, 25, 12, 6)]:
        assert engine.step(state, EndTurn()).status == "decision"
        assert (state.turn, state.player.hp, enemy.intent_damage) == (turn, hp, preview)
        assert enemy.intent_id == enemy.selected_move == "attack"
        assert enemy.powers["ritual"] == PowerState(3, skip_first=False)
        assert enemy.powers.get("strength") == (PowerState(strength) if strength else None)
        assert state.rng.enemy_ai.word_index == 0
        assert all(enemy.damage_entries[k] is v for k, v in entries.items())
    assert enemy.move_history == ("incantation", "attack", "attack", "attack")


def test_jaw_worm_first_turn_and_roll_publication(scenario_factory):
    state = jaw_state(scenario_factory, defend=True)
    assert engine.step(state, PlayCard(1, None)).status == "decision"
    assert state.player.block == 8
    assert engine.step(state, EndTurn()).status == "decision"
    enemy = state.enemies[0]
    assert (state.player.hp, state.player.block, enemy.block) == (37, 0, 0)
    assert (enemy.intent_id, enemy.selected_move, enemy.intent_damage) == ("thrash", "thrash", 7)
    assert enemy.move_history == ("chomp", "thrash")
    assert state.rng.enemy_ai.word_index == 1
    assert state.player.energy == 3 and state.turn == 2


def test_successful_roll_is_private_until_next_start_and_appends_once(scenario_factory):
    state = jaw_state(scenario_factory)
    enemy = state.enemies[0]
    state.phase = "enemy_turn"
    state.pending = [work("roll_move", enemy.instance_id), work("end_round")]
    engine._dispatch_one(state)
    assert enemy.selected_move == "thrash" and enemy.move_history == ("chomp", "thrash")
    assert enemy.intent_id == "chomp" and enemy.intent_damage == 11
    assert state.rng.enemy_ai.word_index == 1
    validate_runtime_state(state)
    resumed = restore(snapshot(state))
    assert engine._resolve(state).status == engine._resolve(resumed).status == "decision"
    assert enemy.intent_id == "thrash" and enemy.intent_damage == 7
    assert enemy.move_history == ("chomp", "thrash") and state.player.hp == 40
    assert compare(state, resumed) is None


def test_enemy_move_executes_published_not_later_selected_move(scenario_factory):
    state = jaw_state(scenario_factory)
    enemy = state.enemies[0]
    enemy.selected_move = "bellow"
    enemy.move_history += ("bellow",)
    state.phase = "enemy_turn"
    state.pending = [work("enemy_move", enemy.instance_id)]
    engine._dispatch_one(state)
    assert state.pending == [work("damage", "enemy-1", "player", move="chomp"),
                             work("roll_move", "enemy-1"), work("end_round")]
    engine._dispatch_one(state)
    assert state.player.hp == 29 and not enemy.powers


def test_roll_fallback_exhaustion_keeps_history_and_cursor(scenario_factory, monkeypatch):
    state = jaw_state(scenario_factory)
    enemy = state.enemies[0]
    state.phase = "enemy_turn"
    state.rng = replace(state.rng, enemy_ai=replace(state.rng.enemy_ai, word_index=2**64 - 1))
    shuffle = state.rng.shuffle
    state.pending = [work("roll_move", enemy.instance_id)]
    # Keep real bucket/fallback sampling: final word selects blocked Chomp bucket.
    monkeypatch.setattr(rng, "_hmac_sha256", lambda key, message: bytes(32))
    assert engine._resolve(state).status == "fault"
    assert state.fault.kind is FaultKind.RNG_EXHAUSTED
    assert state.rng.enemy_ai.word_index == 2**64 and state.rng.shuffle is shuffle
    assert (enemy.move_history, enemy.selected_move, enemy.intent_id) == (("chomp",), "chomp", "chomp")
    assert state.current.kind == "roll_move" and state.outcome is None
    first = state.fault
    mark_fault(state, FaultKind.INVARIANT, "later")
    assert engine._resolve(state).status == "fault" and state.fault is first


def test_bellow_then_thrash_block_and_strength_order(scenario_factory, monkeypatch):
    state = jaw_state(scenario_factory)
    enemy = state.enemies[0]
    choices = iter(["bellow", "thrash", "chomp"])
    monkeypatch.setattr(rules, "choose_enemy_move", lambda enemy_id, history, stream: (next(choices), stream))
    assert engine.step(state, EndTurn()).status == "decision"
    observed = []
    real_apply = engine._apply_effect

    def inspect(owned, effect):
        if effect.kind == "gain_block":
            observed.append((effect.payload, enemy.powers["strength"].amount, enemy.block))
        real_apply(owned, effect)

    monkeypatch.setattr(engine, "_apply_effect", inspect)
    assert engine.step(state, EndTurn()).status == "decision"
    assert observed == [((("amount", 6),), 3, 0)]
    assert enemy.block == 6 and enemy.intent_damage == 10
    # Defend does not reset the enemy's persistent block at a player decision.
    state.player.block = 8
    state.phase = "enemy_turn"
    state.pending = [work("monster_start", enemy.instance_id)]
    engine._dispatch_one(state)
    assert enemy.block == 0 and state.player.block == 8
    assert engine._resolve(state).status == "decision"
    assert state.player.hp == 27 and enemy.block == 5


def test_vulnerable_stack_and_round_expiry(minimal_state):
    state = minimal_state
    assert engine.step(state, PlayCard(3, "enemy-1")).status == "decision"
    power = state.enemies[0].powers["vulnerable"]
    assert power == PowerState(2)
    assert engine.step(state, EndTurn()).status == "decision"
    assert power.amount == 1
    assert engine.step(state, EndTurn()).status == "decision"
    assert "vulnerable" not in state.enemies[0].powers
    state.phase = "player_action"
    state.pending = [work("apply_power", "enemy-1", "player", power_id="vulnerable", amount=2)]
    engine._dispatch_one(state)
    grace = state.player.powers["vulnerable"]
    effects.apply_effect(state, work("apply_power", 3, "player", power_id="vulnerable", amount=2))
    assert state.player.powers["vulnerable"] is grace
    assert grace == PowerState(4, monster_source_grace=True)
    state.pending = [work("end_round")]
    assert engine._resolve(state).status == "decision"
    assert grace == PowerState(4, monster_source_grace=False)
    assert engine.step(state, EndTurn()).status == "decision"
    assert grace.amount == 3


def test_scheduling_side_effects_precede_queued_round_power(minimal_state):
    state = minimal_state
    enemy = state.enemies[0]
    enemy.powers["ritual"] = PowerState(3)
    enemy.selected_move = "attack"
    enemy.move_history += ("attack",)
    state.player.energy = 0
    state.player.block = 7
    enemy.block = 6
    state.phase = "round_end"
    state.pending = [work("end_round")]
    engine._dispatch_one(state)
    assert state.current is None and state.phase == "player_start"
    assert state.turn == 2 and state.player.energy == 3 and state.player.block == 0
    assert enemy.block == 6 and enemy.intent_id == "attack" and enemy.intent_damage == 6
    assert "strength" not in enemy.powers
    assert state.pending == [
        work("apply_power", "enemy-1", "enemy-1", power_id="strength", amount=3),
        work("draw", "player", "player", amount=5), work("decision"),
    ]
    validate_runtime_state(state)
    resumed = restore(snapshot(state))
    assert engine._resolve(state).status == engine._resolve(resumed).status == "decision"
    assert enemy.intent_damage == 9 and enemy.move_history == ("incantation", "attack")
    assert compare(state, resumed) is None


def test_turn_counter_outside_numeric_surface_faults(minimal_state):
    state = minimal_state
    state.turn = 2**31 - 1
    assert engine.step(state, EndTurn()).status == "fault"
    assert state.fault.kind is FaultKind.UNSUPPORTED
    assert state.turn == 2**31 - 1 and state.outcome is None
    assert state.current.kind == "end_round"


@pytest.mark.parametrize("move,hp", [("chomp", 11), ("thrash", 7)])
def test_lethal_enemy_damage_preempts_block_roll_round_and_draw(scenario_factory, move, hp):
    state = jaw_state(scenario_factory)
    enemy = state.enemies[0]
    if move == "thrash":
        enemy.move_history += (move,)
        enemy.selected_move = enemy.intent_id = move
        effects.refresh_enemy_values(state)
    state.player.hp = hp
    state.rng = replace(state.rng, enemy_ai=replace(state.rng.enemy_ai, word_index=2**64))
    before_rng = state.rng
    history = enemy.move_history
    assert engine.step(state, EndTurn()).status == "terminal"
    assert state.outcome.kind == "death" and state.fault is None
    assert state.hp_before_healing == state.hp_after_cleanup == 0
    assert enemy.block == 0 and enemy.move_history == history and state.rng is before_rng
    assert state.turn == 1 and state.player.energy == 3
    assert not state.pending and state.current is None and not state.piles.hand
    assert len(state.piles.discard) == 1


def test_end_player_and_monster_phases_checkpoint_ownership(minimal_state):
    state = minimal_state
    cards = list(state.piles.hand)
    state.phase = "enemy_turn"
    state.pending = [work("end_player_turn", "player")]
    engine._dispatch_one(state)
    assert state.piles.hand == cards
    assert [e.kind for e in state.pending] == ["discard_hand", "monster_start"]
    engine._dispatch_one(state)
    assert not state.piles.hand and state.piles.discard == cards[::-1]
    assert all(a is b for a, b in zip(state.piles.discard, cards[::-1]))
    engine._dispatch_one(state)
    engine._dispatch_one(state)
    assert [e.kind for e in state.pending] == ["apply_power", "roll_move", "end_round"]
    resumed = restore(snapshot(state))
    assert engine._resolve(state).status == engine._resolve(resumed).status == "decision"
    assert compare(state, resumed) is None
