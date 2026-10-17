from copy import deepcopy
from dataclasses import replace

import pytest

from spire_lab.combat.engine import initial_combat
from spire_lab.combat.model import (
    CardInstance, DamageEntry, EnemySetup, InvariantError, PendingEffect, PowerState,
    UnsupportedMechanicError, validate_runtime_state, validate_state,
)
from spire_lab.combat.rng import StreamId
from spire_lab.combat.snapshot import compare, restore, snapshot


def test_entry_is_supplied_ready_state(scenario_factory):
    scenario = scenario_factory()
    state = initial_combat(scenario)
    assert (state.phase, state.turn, state.player.hp) == ("decision", 1, 40)
    assert state.enemies[0].move_history == ("incantation",)
    assert state.pending == [] and state.current is None and state.in_play is None
    assert state.resolved_effects == 0 and state.player.draw_per_turn == 5
    assert snapshot(state).schema_version == 2
    assert state.rng == scenario.rng
    assert state.enemies[0].selected_move == "incantation"
    assert state.enemies[0].damage_entries == {"attack": DamageEntry(6, 6)}
    assert state.enemies[0].intent_damage is None
    assert state.card_ids_issued == {1, 2, 3}


def test_jaw_worm_entry_derives_only_static_damage(scenario_factory):
    enemy = EnemySetup("enemy-1", "jaw_worm", 44, 44, 0, {}, "chomp", ("chomp",))
    state = initial_combat(scenario_factory(encounter_id="jaw_worm", enemies=(enemy,)))
    assert state.enemies[0].damage_entries == {"chomp": DamageEntry(11, 11), "thrash": DamageEntry(7, 7)}
    assert state.enemies[0].intent_damage == 11
    assert state.enemies[0].selected_move == "chomp"
    assert state.rng.shuffle.word_index == state.rng.enemy_ai.word_index == 0


@pytest.mark.parametrize("case", [
    "hp_zero", "hp_large", "max_hp_zero", "max_hp_large", "energy_zero", "energy_large",
    "energy_mismatch", "draw_zero", "draw_large", "empty_deck", "large_deck", "overfull_hand",
    "bool_integer", "two_enemies", "encounter", "owner", "history", "intent", "discard",
    "exhaust", "player_powers", "enemy_powers", "player_block", "enemy_block", "enemy_hp",
    "enemy_range", "duplicate_card", "reserved_id", "scope", "malformed_scope", "bad_key",
])
def test_entry_rejects_inconsistent_setup_without_aliasing(scenario_factory, case):
    s = scenario_factory()
    if case == "hp_zero": s.player.hp = 0
    elif case == "hp_large": s.player.hp = 1000
    elif case == "max_hp_zero": s.player.max_hp = 0
    elif case == "max_hp_large": s.player.max_hp = 1000
    elif case == "energy_zero": s.player.energy = s.player.max_energy = 0
    elif case == "energy_large": s.player.energy = s.player.max_energy = 1000
    elif case == "energy_mismatch": s.player.energy = 2
    elif case == "draw_zero": s.player.draw_per_turn = 0
    elif case == "draw_large": s.player.draw_per_turn = 11
    elif case == "empty_deck": s.piles.hand.clear()
    elif case == "large_deck": s.piles.draw = [CardInstance(i, "strike", False) for i in range(4, 1001)]
    elif case == "overfull_hand": s.piles.hand = [CardInstance(i, "strike", False) for i in range(11)]
    elif case == "bool_integer": s.player.energy = True
    elif case == "two_enemies": s.enemies += (deepcopy(s.enemies[0]),)
    elif case == "encounter": s.encounter_id = "jaw_worm"
    elif case == "owner": s.rng = replace(s.rng, enemy_instance_id="other")
    elif case == "history": s.enemies[0].move_history = ("incantation", "attack")
    elif case == "intent": s.enemies[0].intent_id = "attack"
    elif case == "discard": s.piles.discard.append(s.piles.hand.pop())
    elif case == "exhaust": s.piles.exhaust.append(s.piles.hand.pop())
    elif case == "player_powers": s.player.powers["strength"] = PowerState(1)
    elif case == "enemy_powers": s.enemies[0].powers["ritual"] = PowerState(3, True)
    elif case == "player_block": s.player.block = 1
    elif case == "enemy_block": s.enemies[0].block = 1
    elif case == "enemy_hp": s.enemies[0].hp -= 1
    elif case == "enemy_range": s.enemies[0].hp = s.enemies[0].max_hp = 47
    elif case == "duplicate_card": s.piles.draw.append(s.piles.hand[0])
    elif case == "reserved_id":
        s.enemies[0].instance_id = "player"
        s.rng = replace(s.rng, enemy_instance_id="player")
    elif case == "scope":
        s.rng = replace(s.rng, enemy_ai=replace(s.rng.enemy_ai, stream_id=StreamId("combat.enemy.ai", ("other", "private"))))
    elif case == "malformed_scope": object.__setattr__(s.rng.shuffle.stream_id, "scope", ())
    elif case == "bad_key": object.__setattr__(s.rng.shuffle, "key", b"short")
    before = deepcopy(s)
    with pytest.raises(InvariantError): initial_combat(s)
    assert s == before


@pytest.mark.parametrize("field", ["card", "enemy", "encounter"])
def test_entry_rejects_excluded_content(scenario_factory, field):
    s = scenario_factory()
    if field == "card": s.piles.hand[0] = CardInstance(1, "anger", False)
    elif field == "enemy": s.enemies[0].enemy_id = "slime"
    else: s.encounter_id = "slime"
    with pytest.raises(UnsupportedMechanicError): initial_combat(s)


def test_entry_and_restore_own_nested_values(scenario_factory):
    s = scenario_factory()
    state = initial_combat(s)
    saved = snapshot(state)
    restored = restore(saved)
    s.player.hp = 1
    s.piles.hand.clear()
    s.enemies[0].powers["strength"] = PowerState(1)
    assert state.player.hp == restored.player.hp == 40
    assert len(state.piles.hand) == len(restored.piles.hand) == 3
    assert state.enemies[0].powers == restored.enemies[0].powers == {}
    state.enemies[0].damage_entries["attack"].output = 12
    state.player.powers["strength"] = PowerState(2)
    restored.enemies[0].damage_entries["attack"].base = 8
    restored.piles.hand.pop()
    assert saved.state.enemies[0].damage_entries["attack"] == DamageEntry(6, 6)
    assert state.enemies[0].damage_entries["attack"].base == 6
    assert restored.enemies[0].damage_entries["attack"].output == 6
    assert restored.player.powers == s.player.powers == {}
    assert len(state.piles.hand) == 3 and len(saved.state.piles.hand) == 3


def test_runtime_is_not_entry_validation(minimal_state):
    s = minimal_state
    s.turn = 2
    s.player.energy = 0
    s.player.hp = 0
    s.phase = "enemy_turn"
    s.enemies[0].selected_move = "attack"
    s.enemies[0].move_history += ("attack",)
    s.pending = [PendingEffect("terminal", None, None, (("outcome", "death"),))]
    validate_runtime_state(s)
    restored = restore(snapshot(s))
    assert compare(s, restored) is None


def test_in_play_card_has_unique_issued_ownership(minimal_state):
    minimal_state.in_play = minimal_state.piles.hand[0]
    with pytest.raises(InvariantError): validate_state(minimal_state)
    minimal_state.in_play = minimal_state.piles.hand.pop()
    validate_state(minimal_state)


@pytest.mark.parametrize("case", ["phase", "turn", "dead_ready", "busy_ready", "move", "damage",
                                  "intent_damage", "negative_block", "strength", "flag", "count"])
def test_runtime_rejects_inconsistent_values(minimal_state, case):
    s = minimal_state
    if case == "phase": s.phase = "unknown"
    elif case == "turn": s.turn = 0
    elif case == "dead_ready": s.player.hp = 0
    elif case == "busy_ready": s.pending.append(PendingEffect("draw", "player", "player", (("amount", 1),)))
    elif case == "move": s.enemies[0].selected_move = "attack"
    elif case == "damage": s.enemies[0].damage_entries["attack"].output = 7
    elif case == "intent_damage": s.enemies[0].intent_damage = 6
    elif case == "negative_block": s.player.block = -1
    elif case == "strength": s.enemies[0].powers["strength"] = PowerState(1000)
    elif case == "flag": s.player.powers["strength"] = PowerState(1, True)
    elif case == "count": s.resolved_effects = True
    with pytest.raises(InvariantError): validate_runtime_state(s)


@pytest.mark.parametrize("payload", [(("amount", True),), (("wrong", 1),), (("amount", 1), ("move", "attack"))])
def test_runtime_rejects_malformed_known_payload(minimal_state, payload):
    minimal_state.phase = "player_action"
    minimal_state.pending.append(PendingEffect("damage", "player", "enemy-1", payload))
    with pytest.raises(InvariantError): validate_runtime_state(minimal_state)


def test_runtime_rejects_unknown_work_explicitly(minimal_state):
    minimal_state.phase = "player_action"
    minimal_state.pending.append(PendingEffect("callback", None, None, ()))
    with pytest.raises(UnsupportedMechanicError): validate_runtime_state(minimal_state)


@pytest.mark.parametrize("enemy_id,hp,first", [
    ("cultist", 48, "incantation"), ("cultist", 54, "incantation"),
    ("jaw_worm", 40, "chomp"), ("jaw_worm", 44, "chomp"),
])
def test_entry_accepts_explicit_numeric_boundaries(scenario_factory, enemy_id, hp, first):
    s = scenario_factory(encounter_id=enemy_id,
                         enemies=(EnemySetup("enemy-1", enemy_id, hp, hp, 0, {}, first, (first,)),))
    s.player.hp = s.player.max_hp = 999
    s.player.energy = s.player.max_energy = 999
    s.player.draw_per_turn = 10
    s.piles.hand = [CardInstance(i, "strike", False) for i in range(10)]
    s.piles.draw = [CardInstance(i, "bash", True) for i in range(10, 999)]
    state = initial_combat(s)
    assert len(state.card_ids_issued) == 999 and len(state.piles.hand) == 10
    assert state.player.hp == state.player.energy == 999
    s.player.hp = s.player.max_hp = s.player.energy = s.player.max_energy = s.player.draw_per_turn = 1
    s.piles.hand = []
    s.piles.draw = [CardInstance(1, "defend", False)]
    assert initial_combat(s).player.hp == 1


@pytest.mark.parametrize("change", ["power_bool", "flag_int", "damage_bool", "current", "in_play", "negative_count", "large_count"])
def test_structural_new_fields_reject_foreign_or_bool_values(minimal_state, change):
    s = minimal_state
    if change == "power_bool": s.player.powers["strength"] = PowerState(True)
    elif change == "flag_int": s.player.powers["strength"] = PowerState(1, 1)
    elif change == "damage_bool": s.enemies[0].damage_entries["attack"].base = True
    elif change == "current": s.current = object()
    elif change == "in_play": s.in_play = object()
    elif change == "negative_count": s.resolved_effects = -1
    else: s.resolved_effects = 100001
    with pytest.raises(InvariantError): validate_state(s)


def test_ready_damage_refresh_validation_uses_strength_and_vulnerable(minimal_state):
    s = minimal_state
    s.enemies[0].powers["strength"] = PowerState(999)
    s.player.powers["vulnerable"] = PowerState(2, monster_source_grace=True)
    s.enemies[0].damage_entries["attack"].output = 1507
    validate_runtime_state(s)
    s.enemies[0].damage_entries["attack"].output = 1508
    with pytest.raises(InvariantError): validate_runtime_state(s)


def test_entry_separates_distinct_gameplay_owners_even_if_input_aliases_empty_containers(scenario_factory):
    s = scenario_factory()
    s.enemies[0].powers = s.player.powers
    s.piles.discard = s.piles.exhaust
    state = initial_combat(s)
    state.enemies[0].powers["ritual"] = PowerState(3, True)
    state.piles.discard.append(state.piles.hand.pop())
    assert state.player.powers == {}
    assert state.piles.exhaust == []
    assert s.player.powers == {} and s.piles.discard == s.piles.exhaust == []
