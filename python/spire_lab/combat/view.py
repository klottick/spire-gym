"""Detached public observations and non-speculative player choices."""

from dataclasses import dataclass

from spire_lab.combat.model import (
    Action, CardInstance, CombatState, EndTurn, InvalidActionError, PlayCard,
    PowerState, require_steppable,
)
from spire_lab.combat.rules import card_stats


@dataclass(frozen=True)
class CardView:
    instance_id: int
    card_id: str
    upgraded: bool


@dataclass(frozen=True)
class PlayerView:
    hp: int
    max_hp: int
    block: int
    energy: int
    max_energy: int
    powers: tuple[tuple[str, int], ...]


@dataclass(frozen=True)
class EnemyView:
    instance_id: str
    enemy_id: str
    hp: int
    max_hp: int
    block: int
    powers: tuple[tuple[str, int], ...]
    intent_id: str
    intent_damage: int | None


@dataclass(frozen=True)
class PublicView:
    content_id: str
    turn: int
    player: PlayerView
    enemies: tuple[EnemyView, ...]
    hand: tuple[CardView, ...]
    draw_count: int
    discard: tuple[CardView, ...]
    exhaust: tuple[CardView, ...]
    revealed_order: tuple[str, ...]
    outcome: str | None
    hp_before_healing: int | None
    hp_after_cleanup: int | None


def legal_actions(state: CombatState) -> tuple[Action, ...]:
    """Enumerate hand-order choices using only ready public gameplay values."""
    require_steppable(state)
    if state.phase == "terminal":
        return ()
    if (state.phase != "decision" or state.pending or state.current is not None
            or state.in_play is not None or state.outcome is not None
            or state.player.hp <= 0 or any(e.hp <= 0 for e in state.enemies)):
        raise InvalidActionError("combat is not at a ready player decision")
    actions: list[Action] = []
    for card in state.piles.hand:
        stats = card_stats(card)
        if stats.cost <= state.player.energy:
            target = state.enemies[0].instance_id if stats.damage else None
            actions.append(PlayCard(card.instance_id, target))
    actions.append(EndTurn())
    return tuple(actions)


def _powers(powers: dict[str, PowerState]) -> tuple[tuple[str, int], ...]:
    return tuple((name, power.amount) for name, power in sorted(powers.items()))


def _cards(cards: list[CardInstance]) -> tuple[CardView, ...]:
    return tuple(CardView(c.instance_id, c.card_id, c.upgraded) for c in cards)


def public_view(state: CombatState) -> PublicView:
    """Copy public primitives only; never expose hidden work or mutable owners."""
    require_steppable(state)
    player = state.player
    return PublicView(
        state.content_id, state.turn,
        PlayerView(player.hp, player.max_hp, player.block, player.energy,
                   player.max_energy, _powers(player.powers)),
        tuple(EnemyView(e.instance_id, e.enemy_id, e.hp, e.max_hp, e.block,
                        _powers(e.powers), e.intent_id, e.intent_damage)
              for e in state.enemies),
        _cards(state.piles.hand), len(state.piles.draw), _cards(state.piles.discard),
        _cards(state.piles.exhaust), tuple(state.public.revealed_order),
        state.outcome.kind if state.outcome is not None else None,
        state.hp_before_healing, state.hp_after_cleanup,
    )
