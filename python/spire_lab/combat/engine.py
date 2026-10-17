"""Owned entry construction and ordered, fault-preserving work resolution."""

from copy import deepcopy
from dataclasses import replace

from spire_lab.combat import effects, piles, turns
from spire_lab.combat.content import ENEMY_DAMAGE, SUPPORTED_CONTENT_ID
from spire_lab.combat.model import (
    Action, CombatState, DamageEntry, EndTurn, EnemyState, FaultKind,
    InvalidActionError, InvariantError, PendingEffect, Piles, PlayCard,
    PublicKnowledge, ResolutionGuardError, Scenario, Transition,
    UnsupportedMechanicError, mark_fault, require_steppable,
    validate_runtime_state, validate_scenario,
)
from spire_lab.combat.rng import RandomStreamState, RngExhaustedError
from spire_lab.combat.rules import card_stats, normal_damage
from spire_lab.combat.view import legal_actions


def initial_combat(scenario: Scenario) -> CombatState:
    """Copy a complete first-decision input without sampling or startup defaults."""
    validate_scenario(scenario)
    owned = deepcopy(scenario)
    setup = owned.enemies[0]
    entries = {move: DamageEntry(base, base) for move, base in ENEMY_DAMAGE[setup.enemy_id]}
    enemy = EnemyState(
        setup.instance_id, setup.enemy_id, setup.hp, setup.max_hp, setup.block,
        deepcopy(setup.powers), setup.intent_id, setup.move_history, setup.intent_id,
        entries, entries[setup.intent_id].output if setup.intent_id in entries else None,
    )
    state = CombatState(
        content_id=SUPPORTED_CONTENT_ID, phase="decision", turn=1,
        player=owned.player, enemies=[enemy],
        # Empty caller containers may alias each other; future pile owners must not.
        piles=Piles(owned.piles.hand, owned.piles.draw, [], []),
        card_ids_issued={c.instance_id for c in owned.piles.hand + owned.piles.draw},
        rng=owned.rng, pending=[], current=None, in_play=None, resolved_effects=0,
        public=PublicKnowledge(()), outcome=None, fault=None,
        hp_before_healing=None, hp_after_cleanup=None,
    )
    validate_runtime_state(state)
    return state


def _validate_action(state: CombatState, action: object) -> None:
    """Reject malformed fields before equality can mistake bools for handles."""
    require_steppable(state)
    if type(action) is PlayCard:
        if (type(action.card_instance_id) is not int
                or (action.target_id is not None and type(action.target_id) is not str)):
            raise InvalidActionError("invalid card or target handle type")
    elif type(action) is not EndTurn:
        raise InvalidActionError("unsupported action type")
    if action not in legal_actions(state):
        raise InvalidActionError("action is not an available choice")


def step(state: CombatState, action: Action) -> Transition:
    """Resolve a fully preflighted player choice to the next checkpoint."""
    _validate_action(state, action)
    if type(action) is EndTurn:
        state.resolved_effects = 0
        state.phase = "enemy_turn"
        state.pending.append(PendingEffect("end_player_turn", "player", None, ()))
        return _resolve(state)

    card = next(c for c in state.piles.hand if c.instance_id == action.card_instance_id)
    stats = card_stats(card)
    # Capture target-specific damage before use mutations or Bash's new power.
    strength = state.player.powers.get("strength")
    damage = normal_damage(stats.damage, strength.amount if strength else 0,
                           "vulnerable" in state.enemies[0].powers) if stats.damage else 0
    work = []
    if stats.damage:
        work.append(PendingEffect("damage", card.instance_id, action.target_id,
                                  (("amount", damage),)))
    if stats.block:
        work.append(PendingEffect("gain_block", card.instance_id, "player",
                                  (("amount", stats.block),)))
    if stats.vulnerable:
        work.append(PendingEffect("apply_power", card.instance_id, action.target_id,
                                  (("power_id", "vulnerable"), ("amount", stats.vulnerable))))
    work.extend((PendingEffect("finish_card", card.instance_id, None, ()),
                 PendingEffect("decision", None, None, ())))
    state.resolved_effects = 0
    state.phase = "player_action"
    state.piles.hand.remove(card)
    state.in_play = card
    state.player.energy -= stats.cost
    state.pending.extend(work)
    return _resolve(state)


def _apply_effect(state: CombatState, effect: PendingEffect) -> None:
    """Route the finite work catalog; later owners must opt in explicitly."""
    if effect.kind in {"damage", "gain_block", "apply_power", "reduce_power",
                       "remove_power", "decision", "terminal", "finish_card"}:
        effects.apply_effect(state, effect)
    elif effect.kind in {"draw", "recycle", "discard_hand"}:
        piles.apply_effect(state, effect)
    elif effect.kind in {"end_player_turn",
                          "monster_start", "enemy_move", "roll_move", "end_round"}:
        turns.apply_effect(state, effect)
    else:
        raise UnsupportedMechanicError(f"unknown work: {effect.kind}")


def _retain_exhausted_stream(state: CombatState, stream: RandomStreamState) -> None:
    """Install only the identified owner's latest immutable sampling progress."""
    if type(stream) is RandomStreamState:
        for name in ("shuffle", "enemy_ai"):
            owned = getattr(state.rng, name)
            if (stream.stream_id == owned.stream_id and stream.key == owned.key
                    and owned.word_index <= stream.word_index <= 2**64):
                state.rng = replace(state.rng, **{name: stream})
                return
    raise InvariantError("exhaustion cursor does not identify an owned stream")


def _dispatch_one(state: CombatState) -> None:
    """Attempt one payload. Only completed dispatches release current ownership.

    Safe replay checkpoints are between calls, with current None. An interrupted
    or failing current payload is diagnostic work, never an automatic retry.
    """
    if state.fault is not None:
        return
    selected = state.current
    try:
        if state.current is not None:
            raise InvariantError("cannot retry current work")
        if not state.pending:
            raise InvariantError("no work to dispatch")
        if type(state.resolved_effects) is not int or not 0 <= state.resolved_effects < 100_000:
            raise InvariantError("invalid dispatch budget")
        selected = state.pending.pop(0)
        state.current = selected
        state.resolved_effects += 1
        if state.resolved_effects == 100_000:
            raise ResolutionGuardError("effect dispatch limit reached")
        _apply_effect(state, selected)
        state.current = None
        if state.phase in ("decision", "terminal"):
            validate_runtime_state(state)
    except RngExhaustedError as exc:
        try:
            _retain_exhausted_stream(state, exc.state)
        except Exception as invalid:
            mark_fault(state, FaultKind.INVARIANT, str(invalid), state.resolved_effects)
        else:
            mark_fault(state, FaultKind.RNG_EXHAUSTED, str(exc), state.resolved_effects)
    except UnsupportedMechanicError as exc:
        mark_fault(state, FaultKind.UNSUPPORTED, str(exc), state.resolved_effects)
    except ResolutionGuardError as exc:
        mark_fault(state, FaultKind.EFFECT_GUARD, str(exc), state.resolved_effects)
    except Exception as exc:
        mark_fault(state, FaultKind.INVARIANT, str(exc), state.resolved_effects)
    if state.fault is not None:
        state.current = selected


def _resolve(state: CombatState) -> Transition:
    """Resolve to a validated checkpoint without resetting the same-step budget."""
    while state.fault is None:
        if state.current is not None or state.pending:
            _dispatch_one(state)
            continue
        try:
            if state.phase not in ("decision", "terminal"):
                raise InvariantError("empty work outside a decision or terminal checkpoint")
            validate_runtime_state(state)
        except UnsupportedMechanicError as exc:
            mark_fault(state, FaultKind.UNSUPPORTED, str(exc), state.resolved_effects)
        except Exception as exc:
            mark_fault(state, FaultKind.INVARIANT, str(exc), state.resolved_effects)
        else:
            return Transition(state, state.phase)
    return Transition(state, "fault")
