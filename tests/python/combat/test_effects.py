import pytest

from spire_lab.combat import engine
from spire_lab.combat.model import FaultKind, PendingEffect, PowerState
from spire_lab.combat.snapshot import compare, restore, snapshot


def effect(kind, target="enemy-1", source="player", **payload):
    return PendingEffect(kind, source, target, tuple(payload.items()))


def test_damage_payload_ownership(minimal_state):
    state = minimal_state
    state.phase = "player_action"
    enemy = state.enemies[0]
    state.pending = [effect("damage", amount=8),
                     effect("damage", "player", "enemy-1", move="attack"),
                     effect("decision", None, None)]
    # The late-bound entry changes consistently with the owned power state.
    enemy.powers["strength"] = PowerState(3)
    enemy.damage_entries["attack"].output = 9
    assert engine._resolve(state).status == "decision"
    assert enemy.hp == 40 and state.player.hp == 31


@pytest.mark.parametrize("skip_first", [True, False])
def test_power_stacking_preserves_flags(minimal_state, skip_first):
    from spire_lab.combat.effects import apply_effect
    state = minimal_state
    state.phase = "enemy_turn"
    enemy = state.enemies[0]
    vulnerable = PowerState(2, monster_source_grace=True)
    ritual = PowerState(3, skip_first=skip_first)
    enemy.powers.update(vulnerable=vulnerable, ritual=ritual)
    apply_effect(state, effect("apply_power", power_id="vulnerable", amount=3))
    apply_effect(state, effect("apply_power", power_id="ritual", amount=3))
    assert enemy.powers["vulnerable"] is vulnerable
    assert vulnerable.amount == 5 and vulnerable.monster_source_grace
    assert enemy.powers["ritual"] is ritual and ritual.skip_first is skip_first


def test_ritual_stacking_and_queued_removal_checkpoints(minimal_state):
    state = minimal_state
    state.phase = "enemy_turn"
    ritual = PowerState(3, skip_first=True)
    state.enemies[0].powers["ritual"] = ritual
    state.pending = [effect("apply_power", power_id="ritual", amount=3), effect("decision", None, None)]
    engine._dispatch_one(state)
    assert ritual.amount == 6 and ritual.skip_first
    assert compare(restore(snapshot(state)), state) is None
    assert engine._resolve(state).status == "decision"
    state.phase = "round_end"
    state.pending = [effect("reduce_power", power_id="ritual", amount=6), effect("decision", None, None)]
    engine._dispatch_one(state)
    assert ritual.amount == 0
    resumed = restore(snapshot(state))
    assert engine._resolve(resumed).status == "decision"
    assert "ritual" not in resumed.enemies[0].powers


def test_new_power_flags(minimal_state):
    from spire_lab.combat.effects import apply_effect
    state = minimal_state
    state.phase = "enemy_turn"
    apply_effect(state, effect("apply_power", power_id="vulnerable", amount=2))
    apply_effect(state, effect("apply_power", power_id="ritual", amount=3))
    assert state.enemies[0].powers["vulnerable"] == PowerState(2)
    assert state.enemies[0].powers["ritual"] == PowerState(3, skip_first=True)
    apply_effect(state, effect("apply_power", "player", "enemy-1", power_id="vulnerable", amount=2))
    assert state.player.powers["vulnerable"] == PowerState(2, monster_source_grace=True)


def test_reduce_then_remove_not_immediate_replacement(minimal_state):
    from spire_lab.combat.effects import apply_effect, refresh_enemy_values
    state = minimal_state
    state.phase = "enemy_turn"
    enemy = state.enemies[0]
    vulnerable = PowerState(2)
    enemy.powers["vulnerable"] = vulnerable
    tail = effect("gain_block", amount=5)
    state.pending = [tail]
    apply_effect(state, effect("reduce_power", power_id="vulnerable", amount=2))
    assert enemy.powers["vulnerable"] is vulnerable and vulnerable.amount == 0
    assert state.pending[0] == effect("remove_power", power_id="vulnerable")
    assert state.pending[1] is tail
    engine._dispatch_one(state)
    assert "vulnerable" not in enemy.powers
    enemy.powers["strength"] = PowerState(998)
    refresh_enemy_values(state)
    entry = enemy.damage_entries["attack"]
    apply_effect(state, effect("apply_power", power_id="strength", amount=3))
    assert enemy.powers["strength"].amount == 999
    assert enemy.damage_entries["attack"] is entry and entry.output == 1005


def test_refresh_damage_and_published_intent_in_place(minimal_state):
    from spire_lab.combat.effects import apply_effect
    state = minimal_state
    state.phase = "enemy_turn"
    enemy = state.enemies[0]
    enemy.selected_move = "attack"
    enemy.move_history += ("attack",)
    enemy.intent_id = "attack"
    entry = enemy.damage_entries["attack"]
    apply_effect(state, effect("apply_power", power_id="strength", amount=3))
    assert entry.output == enemy.intent_damage == 9
    apply_effect(state, effect("apply_power", "player", "enemy-1", power_id="vulnerable", amount=2))
    assert entry.output == enemy.intent_damage == 13
    apply_effect(state, effect("remove_power", "player", "enemy-1", power_id="vulnerable"))
    assert enemy.damage_entries["attack"] is entry and entry.output == enemy.intent_damage == 9


@pytest.mark.parametrize("hp,block,damage,want_hp,want_block", [(40, 5, 8, 37, 0), (40, 8, 5, 40, 3)])
def test_damage_consumes_block(minimal_state, hp, block, damage, want_hp, want_block):
    state = minimal_state
    state.phase = "enemy_turn"
    state.player.hp, state.player.block = hp, block
    state.pending = [effect("damage", "player", "enemy-1", amount=damage), effect("decision", None, None)]
    assert engine._resolve(state).status == "decision"
    assert (state.player.hp, state.player.block) == (want_hp, want_block)


def test_gain_block_caps(minimal_state):
    state = minimal_state
    state.phase = "player_action"
    state.player.block = 998
    state.pending = [effect("gain_block", "player", amount=8), effect("decision", None, None)]
    assert engine._resolve(state).status == "decision"
    assert state.player.block == 999


def test_partial_power_reduction(minimal_state):
    from spire_lab.combat.effects import apply_effect
    state = minimal_state
    state.phase = "round_end"
    power = PowerState(5, monster_source_grace=True)
    state.player.powers["vulnerable"] = power
    apply_effect(state, effect("reduce_power", "player", power_id="vulnerable", amount=2))
    assert power.amount == 3 and power.monster_source_grace and not state.pending


@pytest.mark.parametrize("kind,payload", [
    ("gain_block", (("amount", True),)),
    ("damage", (("amount", 3), ("move", "attack"))),
    ("apply_power", (("amount", 2), ("power_id", "vulnerable"))),
])
def test_malformed_payload_faults_before_mutation(minimal_state, kind, payload):
    state = minimal_state
    state.phase = "player_action"
    state.pending = [PendingEffect(kind, "player", "enemy-1", payload)]
    assert engine._resolve(state).status == "fault"
    assert state.fault.kind is FaultKind.INVARIANT
    assert state.enemies[0].hp == 48 and state.enemies[0].block == 0
    assert not state.enemies[0].powers


@pytest.mark.parametrize("target", [None, "missing", 1, True])
def test_invalid_actor_target_faults(minimal_state, target):
    minimal_state.phase = "player_action"
    minimal_state.pending = [effect("damage", target, amount=8)]
    assert engine._resolve(minimal_state).status == "fault"
    assert minimal_state.fault.kind is FaultKind.INVARIANT
    assert minimal_state.enemies[0].hp == 48


def test_unsupported_power_faults(minimal_state):
    minimal_state.phase = "player_action"
    minimal_state.pending = [effect("apply_power", power_id="poison", amount=2)]
    assert engine._resolve(minimal_state).status == "fault"
    assert minimal_state.fault.kind is FaultKind.UNSUPPORTED


def test_decision_requires_quiescence(minimal_state):
    minimal_state.phase = "player_action"
    minimal_state.pending = [effect("decision", None, None), effect("gain_block", "player", amount=5)]
    assert engine._resolve(minimal_state).status == "fault"
    assert minimal_state.fault.kind is FaultKind.INVARIANT
    assert minimal_state.current.kind == "decision"


@pytest.mark.parametrize("target,damage,status,hp", [("enemy-1", 48, "victory", 46), ("player", 40, "death", 0)])
def test_lethal_damage_completes_terminal_cleanup(minimal_state, target, damage, status, hp):
    state = minimal_state
    state.phase = "player_action"
    state.pending = [effect("damage", target, amount=damage), effect("gain_block", "player", amount=5)]
    assert engine._resolve(state).status == "terminal"
    assert state.outcome.kind == status and state.player.hp == hp
    assert state.hp_before_healing == (40 if status == "victory" else 0)
    assert state.hp_after_cleanup == hp
    assert not state.pending and state.current is None and state.player.block == 0
    assert state.resolved_effects == 2


def test_lethal_checkpoint_and_healing_cap_preserve_in_play_ownership(minimal_state):
    state = minimal_state
    state.phase = "player_action"
    state.player.hp = 49
    state.in_play = state.piles.hand.pop()
    card = state.in_play
    state.pending = [effect("damage", amount=48), effect("finish_card", None, card.instance_id)]
    engine._dispatch_one(state)
    assert state.enemies[0].hp == 0 and state.outcome is None
    assert state.current is None and state.pending[0].kind == "terminal"
    resumed = restore(snapshot(state))
    assert engine._resolve(state).status == engine._resolve(resumed).status == "terminal"
    assert state.player.hp == state.hp_after_cleanup == 50 and state.hp_before_healing == 49
    assert state.in_play is card and card not in state.piles.discard
    assert compare(state, resumed) is None


@pytest.mark.parametrize("source,move", [("player", "attack"), ("missing", "attack"), ("enemy-1", "incantation")])
def test_invalid_damage_reference_faults(minimal_state, source, move):
    minimal_state.phase = "enemy_turn"
    minimal_state.pending = [effect("damage", "player", source, move=move)]
    assert engine._resolve(minimal_state).status == "fault"
    assert minimal_state.fault.kind is FaultKind.INVARIANT and minimal_state.player.hp == 40


def test_stacking_overflow_faults_without_replacing_power(minimal_state):
    state = minimal_state
    state.phase = "player_action"
    power = PowerState(2**31 - 1)
    state.enemies[0].powers["vulnerable"] = power
    state.pending = [effect("apply_power", power_id="vulnerable", amount=1)]
    assert engine._resolve(state).status == "fault"
    assert state.fault.kind is FaultKind.INVARIANT
    assert state.enemies[0].powers["vulnerable"] is power and power.amount == 2**31 - 1


@pytest.mark.parametrize("hp,expected", [(40, 46), (48, 50)])
def test_terminal_healing_and_no_invented_pile_reset(scenario_factory, hp, expected):
    from spire_lab.combat.model import CardInstance, Piles, PlayCard, PlayerState
    from spire_lab.combat.view import legal_actions, public_view

    state = engine.initial_combat(scenario_factory(
        player=PlayerState(hp, 50, 0, 8, 8, 5, {}),
        piles=Piles([CardInstance(i, "bash", True) for i in range(1, 5)], [], [], [])))
    for i in range(1, 5):
        result = engine.step(state, PlayCard(i, "enemy-1"))
    assert result.status == "terminal" and state.outcome.kind == "victory"
    assert state.hp_before_healing == hp and state.hp_after_cleanup == expected
    assert state.player.hp == expected and not state.pending and state.current is None
    assert [c.instance_id for c in state.piles.discard] == [1, 2, 3]
    assert not state.piles.draw and not state.piles.exhaust
    assert legal_actions(state) == ()
    public = public_view(state)
    assert public.outcome == "victory" and public.hp_before_healing == hp
    assert public.hp_after_cleanup == expected
    state.outcome.kind = "death"
    assert public.outcome == "victory"


def test_terminal_death_has_no_choices(minimal_state):
    from spire_lab.combat.view import legal_actions, public_view
    minimal_state.phase = "enemy_turn"
    minimal_state.pending = [effect("damage", "player", "enemy-1", amount=40)]
    assert engine._resolve(minimal_state).status == "terminal"
    public = public_view(minimal_state)
    assert public.outcome == "death" and public.hp_before_healing == public.hp_after_cleanup == 0
    assert legal_actions(minimal_state) == ()


def test_unneeded_exhausted_stream_does_not_fault(scenario_factory):
    from dataclasses import replace
    from spire_lab.combat.model import CardInstance, Piles, PlayCard, PlayerState
    state = engine.initial_combat(scenario_factory(
        player=PlayerState(40, 50, 0, 8, 8, 5, {}),
        piles=Piles([CardInstance(i, "bash", True) for i in range(1, 5)], [], [], [])))
    state.rng = replace(state.rng, enemy_ai=replace(state.rng.enemy_ai, word_index=2**64))
    rng = state.rng
    for i in range(1, 5):
        result = engine.step(state, PlayCard(i, "enemy-1"))
    assert result.status == "terminal" and state.fault is None
    assert state.rng == rng and state.outcome.kind == "victory"
