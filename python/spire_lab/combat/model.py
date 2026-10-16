from dataclasses import dataclass
from enum import Enum
from typing import TypeAlias


@dataclass(frozen=True)
class CardInstance:
    instance_id: int
    card_id: str
    upgraded: bool


@dataclass
class Piles:
    hand: list[CardInstance]
    draw: list[CardInstance]
    discard: list[CardInstance]
    exhaust: list[CardInstance]


@dataclass
class PlayerState:
    hp: int
    max_hp: int
    block: int
    energy: int
    max_energy: int
    powers: dict[str, int]


@dataclass
class EnemyState:
    instance_id: str
    enemy_id: str
    hp: int
    max_hp: int
    block: int
    powers: dict[str, int]
    intent_id: str | None
    move_history: tuple[str, ...]


PayloadValue: TypeAlias = int | str | bool
SourceTarget: TypeAlias = int | str | None


@dataclass
class PendingEffect:
    kind: str
    source_id: SourceTarget
    target_id: SourceTarget
    payload: tuple[tuple[str, PayloadValue], ...]


@dataclass
class PublicKnowledge:
    revealed_order: tuple[str, ...]


@dataclass(frozen=True)
class RngState:
    slots: tuple[int | str | bool, ...]


@dataclass
class Scenario:
    encounter_id: str
    player: PlayerState
    enemies: tuple[EnemyState, ...]
    piles: Piles
    rng: RngState


@dataclass
class TerminalOutcome:
    kind: str


class FaultKind(Enum):
    UNSUPPORTED = "unsupported"
    INVARIANT = "invariant"
    EFFECT_GUARD = "effect_guard"


@dataclass
class FaultRecord:
    kind: FaultKind
    message: str
    resolved_effects: int


class InvalidActionError(Exception):
    pass


class UnsupportedMechanicError(Exception):
    pass


class InvariantError(Exception):
    pass


class ResolutionGuardError(Exception):
    pass


class FaultedStateError(Exception):
    pass


@dataclass
class CombatState:
    content_id: str
    phase: str
    turn: int
    player: PlayerState
    enemies: list[EnemyState]
    piles: Piles
    card_ids_issued: set[int]
    rng: RngState
    pending: list[PendingEffect]
    public: PublicKnowledge
    outcome: TerminalOutcome | None
    fault: FaultRecord | None
    hp_before_healing: int | None
    hp_after_cleanup: int | None


@dataclass(frozen=True)
class PlayCard:
    card_instance_id: int
    target_id: int | str | None


@dataclass(frozen=True)
class EndTurn:
    pass


Action: TypeAlias = PlayCard | EndTurn


@dataclass
class Transition:
    state: CombatState
    status: str


def _integer(value: object) -> bool:
    return type(value) is int


def _string(value: object) -> bool:
    return type(value) is str


def _int_or_string(value: object) -> bool:
    return _integer(value) or _string(value)


def _valid_piles(piles: object) -> bool:
    return (
        type(piles) is Piles
        and all(type(getattr(piles, name)) is list for name in ("hand", "draw", "discard", "exhaust"))
        and all(type(card) is CardInstance and _integer(card.instance_id) and _string(card.card_id)
                and type(card.upgraded) is bool
                for name in ("hand", "draw", "discard", "exhaust") for card in getattr(piles, name))
    )


def _valid_player(player: object) -> bool:
    return (type(player) is PlayerState and all(_integer(getattr(player, name)) for name in
            ("hp", "max_hp", "block", "energy", "max_energy"))
            and type(player.powers) is dict and all(_string(k) and _integer(v) for k, v in player.powers.items()))


def _valid_enemy(enemy: object) -> bool:
    return (type(enemy) is EnemyState and all(_string(getattr(enemy, n)) for n in ("instance_id", "enemy_id"))
            and all(_integer(getattr(enemy, n)) for n in ("hp", "max_hp", "block"))
            and type(enemy.powers) is dict and all(_string(k) and _integer(v) for k, v in enemy.powers.items())
            and (enemy.intent_id is None or _string(enemy.intent_id))
            and type(enemy.move_history) is tuple and all(_string(x) for x in enemy.move_history))


def _valid_pending(effect: object) -> bool:
    return (type(effect) is PendingEffect and _string(effect.kind)
            and (effect.source_id is None or _int_or_string(effect.source_id))
            and (effect.target_id is None or _int_or_string(effect.target_id))
            and type(effect.payload) is tuple
            and all(type(pair) is tuple and len(pair) == 2 and _string(pair[0])
                    and type(pair[1]) in (int, str, bool) for pair in effect.payload))


def validate_state(state: CombatState) -> None:
    """Validate declared structural types and card identity ownership only."""
    if type(state) is not CombatState:
        raise InvariantError("state must be CombatState")
    if not _string(state.content_id) or not _string(state.phase) or not _integer(state.turn):
        raise InvariantError("invalid state scalar")
    if not _valid_player(state.player) or type(state.enemies) is not list or not all(_valid_enemy(e) for e in state.enemies):
        raise InvariantError("invalid player or enemies")
    if not _valid_piles(state.piles):
        raise InvariantError("invalid piles")
    if type(state.card_ids_issued) is not set or not all(_integer(i) for i in state.card_ids_issued):
        raise InvariantError("invalid issued card IDs")
    cards = [card for name in ("hand", "draw", "discard", "exhaust") for card in getattr(state.piles, name)]
    ids = [card.instance_id for card in cards]
    if len(ids) != len(set(ids)) or any(i not in state.card_ids_issued for i in ids):
        raise InvariantError("pile card IDs must be unique and issued")
    if type(state.rng) is not RngState or type(state.rng.slots) is not tuple or not all(
        type(slot) in (int, str, bool) for slot in state.rng.slots
    ):
        raise InvariantError("invalid RNG state")
    if type(state.pending) is not list or not all(_valid_pending(e) for e in state.pending):
        raise InvariantError("invalid pending effects")
    if type(state.public) is not PublicKnowledge or type(state.public.revealed_order) is not tuple or not all(
        _string(x) for x in state.public.revealed_order
    ):
        raise InvariantError("invalid public knowledge")
    if state.outcome is not None and (type(state.outcome) is not TerminalOutcome or not _string(state.outcome.kind)
                                      or state.outcome.kind not in ("victory", "death")):
        raise InvariantError("invalid terminal outcome")
    if state.fault is not None and (type(state.fault) is not FaultRecord or type(state.fault.kind) is not FaultKind
                                    or not _string(state.fault.message) or not _integer(state.fault.resolved_effects)):
        raise InvariantError("invalid fault record")
    for observed in (state.hp_before_healing, state.hp_after_cleanup):
        if observed is not None and not _integer(observed):
            raise InvariantError("invalid HP observation")


def mark_fault(state: CombatState, kind: FaultKind, message: str, resolved_effects: int = 0) -> None:
    """Record the initial fault while retaining any diagnostic partial state."""
    if state.fault is None:
        state.fault = FaultRecord(kind, message, resolved_effects)


def require_steppable(state: CombatState) -> None:
    """Reject faulted state; action and terminal policy are handled elsewhere."""
    if state.fault is not None:
        raise FaultedStateError("faulted combat state cannot be stepped")
