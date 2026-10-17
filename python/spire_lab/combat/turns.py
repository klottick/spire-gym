"""Enemy execution, later selection and ordered round/player scheduling."""

from dataclasses import replace

from spire_lab.combat import effects, rules
from spire_lab.combat.model import (
    CombatState, InvariantError, PendingEffect, UnsupportedMechanicError,
    _valid_pending, _validate_work,
)


def schedule_next_player_turn(state: CombatState) -> None:
    """Perform start/construction mutations now, leaving round payloads queued.

    Recharge and intent publication precede resolution of already queued Ritual
    Strength. Only draw and the quiescent decision marker wait behind that work.
    """
    if type(state.turn) is not int or not 1 <= state.turn <= 2**31 - 1:
        raise InvariantError("turn outside supported integer surface")
    if state.turn == 2**31 - 1:
        raise UnsupportedMechanicError("next turn exceeds supported integer surface")
    state.phase = "player_start"
    state.turn += 1
    state.player.block = 0
    state.player.energy = state.player.max_energy
    for enemy in state.enemies:
        enemy.intent_id = enemy.selected_move
    effects.refresh_enemy_values(state)
    state.pending.extend((
        PendingEffect("draw", "player", "player", (("amount", state.player.draw_per_turn),)),
        PendingEffect("decision", None, None, ()),
    ))


def _end_round(state: CombatState) -> None:
    state.phase = "round_end"
    # The monster's end-turn hook is monster-side (not player-side). None of
    # this slice's powers acts there: Ritual and Vulnerable use round hooks.
    # Player round hooks precede monster round hooks; payloads append in order.
    enemy = state.enemies[0]
    for actor_id, actor in (("player", state.player), (enemy.instance_id, enemy)):
        for name, power in actor.powers.items():
            if name == "vulnerable":
                if power.monster_source_grace:
                    power.monster_source_grace = False
                else:
                    state.pending.append(PendingEffect(
                        "reduce_power", actor_id, actor_id,
                        (("power_id", "vulnerable"), ("amount", 1)),
                    ))
            elif name == "ritual":
                if power.skip_first:
                    power.skip_first = False
                else:
                    state.pending.append(PendingEffect(
                        "apply_power", actor_id, actor_id,
                        (("power_id", "strength"), ("amount", power.amount)),
                    ))
    schedule_next_player_turn(state)


def apply_effect(state: CombatState, effect: PendingEffect) -> None:
    """Apply the five finite turn kinds, keeping execution separate from roll."""
    if not _valid_pending(effect):
        raise InvariantError("invalid effect value")
    _validate_work(effect)
    enemy = state.enemies[0]
    owner = enemy.instance_id
    if effect.kind == "end_player_turn":
        state.phase = "enemy_turn"
        state.pending[0:0] = [
            PendingEffect("discard_hand", "player", None, ()),
            PendingEffect("monster_start", owner, None, ()),
        ]
    elif effect.kind == "monster_start":
        state.phase = "enemy_turn"
        enemy.block = 0
        effects.refresh_enemy_values(state)
        state.pending.insert(0, PendingEffect("enemy_move", owner, None, ()))
    elif effect.kind == "enemy_move":
        # Execute the published move, never the later selected move. Damage
        # retains the owned entry reference by move ID until its dispatch.
        move = enemy.intent_id
        children = []
        if move in enemy.damage_entries:
            children.append(PendingEffect("damage", owner, "player", (("move", move),)))
            if move == "thrash":
                children.append(PendingEffect("gain_block", owner, owner, (("amount", 5),)))
        elif enemy.enemy_id == "cultist" and move == "incantation":
            children.append(PendingEffect("apply_power", owner, owner,
                                          (("power_id", "ritual"), ("amount", 3))))
        elif enemy.enemy_id == "jaw_worm" and move == "bellow":
            children.extend((
                PendingEffect("apply_power", owner, owner, (("power_id", "strength"), ("amount", 3))),
                PendingEffect("gain_block", owner, owner, (("amount", 6),)),
            ))
        else:
            raise UnsupportedMechanicError("unsupported published enemy move")
        children.extend((PendingEffect("roll_move", owner, None, ()),
                         PendingEffect("end_round", None, None, ())))
        state.pending[0:0] = children
    elif effect.kind == "roll_move":
        move, stream = rules.choose_enemy_move(enemy.enemy_id, enemy.move_history, state.rng.enemy_ai)
        # On failure nothing below runs; engine retains the exception's latest
        # sampling cursor without committing a fabricated selection/history.
        state.rng = replace(state.rng, enemy_ai=stream)
        enemy.selected_move = move
        enemy.move_history += (move,)
    elif effect.kind == "end_round":
        _end_round(state)
    else:
        raise UnsupportedMechanicError(f"not a turn effect: {effect.kind}")
