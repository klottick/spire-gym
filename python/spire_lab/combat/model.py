from dataclasses import dataclass
from enum import Enum
from typing import TypeAlias

from spire_lab.combat.content import (
    ENEMY_DAMAGE, ENEMY_FIRST_MOVE, ENEMY_HP_RANGE,
    SUPPORTED_CARD_IDS, SUPPORTED_CONTENT_ID, SUPPORTED_ENCOUNTER_IDS,
)
from spire_lab.combat.rng import RNG_CONTRACT_ID, RandomStreamState, StreamId


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
class PowerState:
    amount: int
    skip_first: bool = False
    monster_source_grace: bool = False


@dataclass
class DamageEntry:
    base: int
    output: int


@dataclass
class PlayerState:
    hp: int
    max_hp: int
    block: int
    energy: int
    max_energy: int
    draw_per_turn: int
    powers: dict[str, PowerState]


@dataclass
class EnemySetup:
    instance_id: str
    enemy_id: str
    hp: int
    max_hp: int
    block: int
    powers: dict[str, PowerState]
    intent_id: str | None
    move_history: tuple[str, ...]


@dataclass
class EnemyState(EnemySetup):
    selected_move: str
    damage_entries: dict[str, DamageEntry]
    intent_damage: int | None


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
    contract_id: str
    shuffle: RandomStreamState
    enemy_ai: RandomStreamState
    enemy_instance_id: str


@dataclass
class Scenario:
    encounter_id: str
    player: PlayerState
    enemies: tuple[EnemySetup, ...]
    piles: Piles
    rng: RngState


@dataclass
class TerminalOutcome:
    kind: str


class FaultKind(Enum):
    UNSUPPORTED = "unsupported"
    INVARIANT = "invariant"
    EFFECT_GUARD = "effect_guard"
    RNG_EXHAUSTED = "rng_exhausted"


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
    current: PendingEffect | None
    in_play: CardInstance | None
    resolved_effects: int
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
            ("hp", "max_hp", "block", "energy", "max_energy", "draw_per_turn"))
            and _valid_powers(player.powers))


def _valid_powers(powers: object) -> bool:
    return (type(powers) is dict and all(
        _string(k) and type(v) is PowerState and _integer(v.amount)
        and type(v.skip_first) is bool and type(v.monster_source_grace) is bool
        for k, v in powers.items()))


def _valid_enemy_setup(enemy: object) -> bool:
    return (type(enemy) in (EnemySetup, EnemyState) and all(_string(getattr(enemy, n)) for n in ("instance_id", "enemy_id"))
            and all(_integer(getattr(enemy, n)) for n in ("hp", "max_hp", "block"))
            and _valid_powers(enemy.powers)
            and (enemy.intent_id is None or _string(enemy.intent_id))
            and type(enemy.move_history) is tuple and all(_string(x) for x in enemy.move_history))


def _valid_enemy(enemy: object) -> bool:
    return (type(enemy) is EnemyState and _valid_enemy_setup(enemy)
            and _string(enemy.selected_move) and type(enemy.damage_entries) is dict
            and all(_string(k) and type(v) is DamageEntry and _integer(v.base) and _integer(v.output)
                    for k, v in enemy.damage_entries.items())
            and (enemy.intent_damage is None or _integer(enemy.intent_damage)))


def _validate_rng(rng: RngState, owner: str) -> None:
    if (type(rng) is not RngState or rng.contract_id != RNG_CONTRACT_ID
            or not _string(rng.contract_id) or not _string(rng.enemy_instance_id)
            or rng.enemy_instance_id != owner):
        raise InvariantError("invalid RNG bundle or owner")
    for stream, domain in ((rng.shuffle, "combat.shuffle"), (rng.enemy_ai, "combat.enemy.ai")):
        if type(stream) is not RandomStreamState or type(stream.stream_id) is not StreamId:
            raise InvariantError("invalid RNG stream")
        try:
            # Recheck complete replay values, including values from edited snapshots.
            StreamId(stream.stream_id.domain, stream.stream_id.scope)
            RandomStreamState(stream.stream_id, stream.key, stream.word_index)
        except (TypeError, ValueError) as exc:
            raise InvariantError("invalid RNG replay value") from exc
        if stream.stream_id.domain != domain:
            raise InvariantError("wrong RNG domain")
    if rng.shuffle.stream_id.scope[0] != rng.enemy_ai.stream_id.scope[0]:
        raise InvariantError("RNG streams must share combat scope")


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
    if state.in_play is not None:
        if not _valid_piles(Piles([state.in_play], [], [], [])):
            raise InvariantError("invalid in-play card")
        cards.append(state.in_play)
    ids = [card.instance_id for card in cards]
    if len(ids) != len(set(ids)) or any(i not in state.card_ids_issued for i in ids):
        raise InvariantError("pile card IDs must be unique and issued")
    if len(state.enemies) != 1 or not state.enemies[0].instance_id or state.enemies[0].instance_id == "player":
        raise InvariantError("expected one unambiguous enemy owner")
    _validate_rng(state.rng, state.enemies[0].instance_id)
    if type(state.pending) is not list or not all(_valid_pending(e) for e in state.pending):
        raise InvariantError("invalid pending effects")
    if state.current is not None and not _valid_pending(state.current):
        raise InvariantError("invalid current effect")
    if not _integer(state.resolved_effects) or not 0 <= state.resolved_effects <= 100_000:
        raise InvariantError("invalid resolved effect count")
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


def _validate_content(player: PlayerState, enemies: tuple[EnemySetup, ...] | list[EnemyState],
                      piles: Piles, in_play: CardInstance | None = None) -> None:
    cards = [c for name in ("hand", "draw", "discard", "exhaust") for c in getattr(piles, name)]
    if in_play is not None:
        cards.append(in_play)
    if any(c.card_id not in SUPPORTED_CARD_IDS for c in cards):
        raise UnsupportedMechanicError("unsupported card")
    for enemy in enemies:
        if enemy.enemy_id not in SUPPORTED_ENCOUNTER_IDS:
            raise UnsupportedMechanicError("unsupported enemy")
    for actor in (player, *enemies):
        for name, power in actor.powers.items():
            allowed = {"strength", "vulnerable"}
            if type(actor) in (EnemySetup, EnemyState) and actor.enemy_id == "cultist":
                allowed.add("ritual")
            if name not in allowed:
                raise UnsupportedMechanicError("unsupported power")
            # A completed reduction can leave zero until queued removal dispatches.
            if not 0 <= power.amount <= 2**31 - 1:
                raise InvariantError("power outside supported integer surface")
            if name == "strength" and power.amount > 999:
                raise InvariantError("Strength exceeds cap")
            # Ritual is introduced with 3 by its payload owner, but stacking
            # and reduction must remain replayable (including queued removal).
            if (power.skip_first and name != "ritual"
                    or power.monster_source_grace and name != "vulnerable"):
                raise InvariantError("inconsistent power flags")


def _validate_player_ranges(player: PlayerState) -> None:
    if (not 1 <= player.max_hp <= 999 or not 0 <= player.hp <= player.max_hp
            or not 0 <= player.block <= 999 or not 1 <= player.max_energy <= 999
            or not 0 <= player.energy <= player.max_energy or not 1 <= player.draw_per_turn <= 10):
        raise InvariantError("player outside supported ranges")


def validate_scenario(scenario: Scenario) -> None:
    """Validate supplied first-decision conditions without modifying caller values."""
    if (type(scenario) is not Scenario or not _string(scenario.encounter_id)
            or not _valid_player(scenario.player) or not _valid_piles(scenario.piles)
            or type(scenario.enemies) is not tuple or len(scenario.enemies) != 1
            or type(scenario.enemies[0]) is not EnemySetup or not _valid_enemy_setup(scenario.enemies[0])):
        raise InvariantError("invalid scenario structure")
    enemy = scenario.enemies[0]
    if not enemy.instance_id or enemy.instance_id == "player":
        raise InvariantError("invalid enemy owner")
    _validate_rng(scenario.rng, enemy.instance_id)
    _validate_content(scenario.player, scenario.enemies, scenario.piles)
    if scenario.encounter_id not in SUPPORTED_ENCOUNTER_IDS:
        raise UnsupportedMechanicError("unsupported encounter")
    if scenario.encounter_id != enemy.enemy_id:
        raise InvariantError("encounter and enemy disagree")
    _validate_player_ranges(scenario.player)
    player = scenario.player
    if (player.hp == 0 or player.energy != player.max_energy or player.block != 0 or player.powers
            or enemy.block != 0 or enemy.powers or enemy.hp != enemy.max_hp):
        raise InvariantError("not a first ready entry")
    low, high = ENEMY_HP_RANGE[enemy.enemy_id]
    first = ENEMY_FIRST_MOVE[enemy.enemy_id]
    if not low <= enemy.hp <= high or enemy.intent_id != first or enemy.move_history != (first,):
        raise InvariantError("invalid first enemy selection or HP")
    cards = scenario.piles.hand + scenario.piles.draw
    ids = [c.instance_id for c in cards]
    if (scenario.piles.discard or scenario.piles.exhaust or len(scenario.piles.hand) > 10
            or not 1 <= len(cards) <= 999 or len(ids) != len(set(ids))):
        raise InvariantError("invalid initial card ownership or piles")


def _validate_work(effect: PendingEffect) -> None:
    schemas = {
        "damage": ((('amount', int),), (('move', str),)),
        "gain_block": ((('amount', int),),),
        "apply_power": ((('power_id', str), ('amount', int)),),
        "reduce_power": ((('power_id', str), ('amount', int)),),
        "remove_power": ((('power_id', str),),),
        "terminal": ((('outcome', str),),),
        "draw": ((('amount', int),),),
    }
    empty = {"finish_card", "decision", "discard_hand", "recycle", "end_player_turn",
             "monster_start", "enemy_move", "roll_move", "end_round"}
    if effect.kind in empty:
        expected = ((),)
    elif effect.kind in schemas:
        expected = schemas[effect.kind]
    else:
        raise UnsupportedMechanicError("unsupported pending work")
    actual = tuple((key, type(value)) for key, value in effect.payload)
    if actual not in expected:
        raise InvariantError("invalid pending payload schema")
    for key, value in effect.payload:
        if key == "amount" and not 0 <= value <= 2**31 - 1:
            raise InvariantError("pending amount outside supported integer surface")
        if key == "outcome" and value not in ("victory", "death"):
            raise InvariantError("invalid pending outcome")


def validate_runtime_state(state: CombatState) -> None:
    """Validate replayable combat, without reimposing first-entry conditions."""
    validate_state(state)
    if state.content_id != SUPPORTED_CONTENT_ID:
        raise UnsupportedMechanicError("unsupported content identity")
    _validate_content(state.player, state.enemies, state.piles, state.in_play)
    _validate_player_ranges(state.player)
    if state.phase not in {"decision", "player_action", "enemy_turn", "round_end", "player_start", "terminal"}:
        raise InvariantError("invalid phase")
    if not 1 <= state.turn <= 2**31 - 1:
        raise InvariantError("turn outside supported integer surface")
    cards = [c for name in ("hand", "draw", "discard", "exhaust") for c in getattr(state.piles, name)]
    if state.in_play is not None:
        cards.append(state.in_play)
    if (not 1 <= len(cards) <= 999 or len(state.piles.hand) > 10
            or {c.instance_id for c in cards} != state.card_ids_issued):
        raise InvariantError("incomplete runtime card ownership")
    enemy = state.enemies[0]
    low, high = ENEMY_HP_RANGE[enemy.enemy_id]
    if not low <= enemy.max_hp <= high or not 0 <= enemy.hp <= enemy.max_hp or not 0 <= enemy.block <= 999:
        raise InvariantError("enemy outside supported ranges")
    moves = {"incantation", "attack"} if enemy.enemy_id == "cultist" else {"chomp", "thrash", "bellow"}
    if (not enemy.move_history or enemy.move_history[0] != ENEMY_FIRST_MOVE[enemy.enemy_id]
            or any(move not in moves for move in enemy.move_history)
            or enemy.selected_move != enemy.move_history[-1] or enemy.intent_id not in moves):
        raise InvariantError("inconsistent enemy selection")
    bases = dict(ENEMY_DAMAGE[enemy.enemy_id])
    if (set(enemy.damage_entries) != set(bases)
            or any(entry.base != bases[move] or not 0 <= entry.output <= 1515
                   for move, entry in enemy.damage_entries.items())):
        raise InvariantError("invalid enemy damage entries")
    for effect in state.pending:
        _validate_work(effect)
    if state.current is not None:
        _validate_work(state.current)
    if state.phase == "decision":
        if (state.pending or state.current is not None or state.in_play is not None
                or state.player.hp == 0 or enemy.hp == 0 or state.outcome is not None
                or enemy.intent_id != enemy.selected_move):
            raise InvariantError("decision must be ready and live")
        if any(power.amount == 0 for actor in (state.player, enemy) for power in actor.powers.values()):
            raise InvariantError("ready power must be positive")
        strength = enemy.powers.get("strength")
        for entry in enemy.damage_entries.values():
            expected = entry.base + (strength.amount if strength else 0)
            if "vulnerable" in state.player.powers:
                expected = expected * 3 // 2
            if entry.output != expected:
                raise InvariantError("stale ready damage")
        intent = enemy.damage_entries.get(enemy.intent_id)
        if enemy.intent_damage != (intent.output if intent else None):
            raise InvariantError("stale ready intent")
    if state.phase == "terminal":
        if (state.outcome is None or state.pending or state.current is not None
                or state.hp_before_healing is None or state.hp_after_cleanup != state.player.hp):
            raise InvariantError("incomplete terminal checkpoint")
        if state.outcome.kind == "death":
            if state.player.hp != 0 or state.hp_before_healing != 0 or enemy.hp == 0:
                raise InvariantError("inconsistent death checkpoint")
        elif (enemy.hp != 0 or state.hp_before_healing <= 0
              or state.player.hp != min(state.player.max_hp, state.hp_before_healing + 6)):
            raise InvariantError("inconsistent victory checkpoint")
    elif state.outcome is not None:
        raise InvariantError("outcome outside terminal checkpoint")
    for hp in (state.hp_before_healing, state.hp_after_cleanup):
        if hp is not None and not 0 <= hp <= state.player.max_hp:
            raise InvariantError("HP observation outside supported ranges")


def mark_fault(state: CombatState, kind: FaultKind, message: str, resolved_effects: int = 0) -> None:
    """Record the initial fault while retaining any diagnostic partial state."""
    if state.fault is None:
        state.fault = FaultRecord(kind, message, resolved_effects)


def require_steppable(state: CombatState) -> None:
    """Reject faulted state; action and terminal policy are handled elsewhere."""
    if state.fault is not None:
        raise FaultedStateError("faulted combat state cannot be stepped")
