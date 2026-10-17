"""Core payload mutations; queue membership never replaces current work."""

from spire_lab.combat import piles
from spire_lab.combat.model import (
    CombatState, EnemyState, InvariantError, PendingEffect, PlayerState, PowerState,
    TerminalOutcome, UnsupportedMechanicError, _valid_pending, _validate_work,
)
from spire_lab.combat.rules import (
    block_after_add, clamped_strength, hp_and_block_after_damage, normal_damage,
)


def _actor(state: CombatState, actor_id: object) -> PlayerState | EnemyState:
    if type(actor_id) is str:
        if actor_id == "player":
            return state.player
        for enemy in state.enemies:
            if enemy.instance_id == actor_id:
                return enemy
    raise InvariantError("effect must identify an owned actor")


def refresh_enemy_values(state: CombatState) -> None:
    """Refresh owned entries in place, then the separately published intent."""
    for enemy in state.enemies:
        strength = enemy.powers.get("strength")
        for entry in enemy.damage_entries.values():
            entry.output = normal_damage(entry.base, strength.amount if strength else 0,
                                         "vulnerable" in state.player.powers)
        published = enemy.damage_entries.get(enemy.intent_id)
        enemy.intent_damage = published.output if published else None


def _power_id(actor: PlayerState | EnemyState, name: str) -> None:
    allowed = {"strength", "vulnerable"}
    if type(actor) is EnemyState and actor.enemy_id == "cultist":
        allowed.add("ritual")
    if name not in allowed:
        raise UnsupportedMechanicError(f"unsupported power: {name}")


def _apply_power(state: CombatState, effect: PendingEffect, actor: PlayerState | EnemyState,
                 name: str, amount: int) -> None:
    _power_id(actor, name)
    if amount == 0:
        return
    existing = actor.powers.get(name)
    current = existing.amount if existing else 0
    if name == "strength":
        updated = clamped_strength(current, amount)
    else:
        updated = current + amount
        if updated > 2**31 - 1:
            raise InvariantError("power stacking exceeds supported integer surface")
        if name == "ritual" and amount != 3:
            raise InvariantError("unsupported Ritual gain")
    if existing is not None:
        existing.amount = updated
    else:
        actor.powers[name] = PowerState(
            updated, skip_first=name == "ritual",
            monster_source_grace=name == "vulnerable" and type(effect.source_id) is str
            and any(e.instance_id == effect.source_id for e in state.enemies),
        )
    refresh_enemy_values(state)


def _terminal(state: CombatState, outcome: str) -> None:
    enemy = state.enemies[0]
    if outcome == "victory":
        if enemy.hp != 0 or state.player.hp <= 0:
            raise InvariantError("victory requires a living player and defeated enemy")
    elif state.player.hp != 0 or enemy.hp <= 0:
        raise InvariantError("death requires a dead player and living enemy")
    state.hp_before_healing = state.player.hp
    if outcome == "victory":
        state.player.hp = min(state.player.max_hp, state.player.hp + 6)
    state.hp_after_cleanup = state.player.hp
    state.outcome = TerminalOutcome(outcome)
    state.pending.clear()
    state.phase = "terminal"
    # Current is released by the dispatcher, not by queue clearing. Retain an
    # in-play diagnostic card rather than inventing a terminal pile destination.


def apply_effect(state: CombatState, effect: PendingEffect) -> None:
    """Apply an exact core payload, or fail explicitly for a later-task kind."""
    if not _valid_pending(effect):
        raise InvariantError("invalid effect value")
    _validate_work(effect)
    data = dict(effect.payload)
    if effect.kind == "finish_card":
        card = state.in_play
        if (card is None or type(effect.source_id) is not int
                or effect.source_id != card.instance_id or effect.target_id is not None):
            raise InvariantError("finish_card must identify the owned in-play card")
        piles.discard_card(state, card.instance_id, end_turn=False)
        return
    if effect.kind == "decision":
        if (state.pending or state.in_play is not None or state.player.hp <= 0
                or any(enemy.hp <= 0 for enemy in state.enemies) or state.outcome is not None):
            raise InvariantError("decision marker requires live quiescent work")
        state.phase = "decision"
        return
    if effect.kind == "terminal":
        _terminal(state, data["outcome"])
        return
    if effect.kind not in {"damage", "gain_block", "apply_power", "reduce_power", "remove_power"}:
        raise UnsupportedMechanicError(f"not a core effect: {effect.kind}")
    actor = _actor(state, effect.target_id)
    if effect.kind == "damage":
        if "move" in data:
            source = _actor(state, effect.source_id)
            if type(source) is not EnemyState or actor is not state.player:
                raise InvariantError("enemy damage reference requires enemy source and player target")
            if data["move"] not in source.damage_entries:
                raise InvariantError("enemy move has no damage entry")
            amount = source.damage_entries[data["move"]].output
        else:
            # Fixed amounts are calculated by the scheduler, not recalculated
            # from current Strength/Vulnerable at dispatch time.
            amount = data["amount"]
        actor.hp, actor.block = hp_and_block_after_damage(actor.hp, actor.block, amount)
        if actor.hp == 0:
            outcome = "death" if actor is state.player else "victory"
            state.pending.insert(0, PendingEffect("terminal", effect.source_id,
                                                 effect.target_id, (("outcome", outcome),)))
    elif effect.kind == "gain_block":
        actor.block = block_after_add(actor.block, data["amount"])
    else:
        name = data["power_id"]
        _power_id(actor, name)
        if effect.kind == "apply_power":
            _apply_power(state, effect, actor, name, data["amount"])
        elif effect.kind == "reduce_power":
            power = actor.powers.get(name)
            if power is not None:
                power.amount = max(0, power.amount - data["amount"])
                if power.amount == 0:
                    state.pending.insert(0, PendingEffect("remove_power", effect.source_id,
                                                         effect.target_id, (("power_id", name),)))
                refresh_enemy_values(state)
        else:
            actor.powers.pop(name, None)
            refresh_enemy_values(state)
