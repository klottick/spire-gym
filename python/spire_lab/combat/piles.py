"""Immediate card ownership changes and ordered draw/recycle work."""

from dataclasses import replace

from spire_lab.combat import rng
from spire_lab.combat.model import (
    CombatState, InvariantError, PendingEffect, UnsupportedMechanicError,
    _valid_pending, _validate_work,
)


def discard_card(state: CombatState, card_id: int, *, end_turn: bool) -> None:
    """Move a hand/in-play owner to discard before any presentation work.

    Ordinary end-turn discard suppresses manual-discard callbacks, not this
    membership bookkeeping. The supported cards have no such callbacks.
    """
    if type(card_id) is not int or type(end_turn) is not bool:
        raise InvariantError("invalid discard arguments")
    card = state.in_play
    if card is not None and card.instance_id == card_id:
        state.in_play = None
    else:
        card = next((c for c in state.piles.hand if c.instance_id == card_id), None)
        if card is None:
            raise InvariantError("discard must identify a hand-owned or in-play card")
        state.piles.hand.remove(card)
    state.piles.discard.append(card)


def _draw(state: CombatState, effect: PendingEffect, amount: int) -> None:
    amount = min(amount, 10 - len(state.piles.hand),
                 len(state.piles.draw) + len(state.piles.discard))
    if amount <= 0:
        return
    existing = len(state.piles.draw)
    if amount > existing:
        # These children run before the caller's remaining work. Each draw
        # recomputes current pile sizes/cap rather than capturing card owners.
        children = []
        if existing:
            children.append(PendingEffect("draw", effect.source_id, effect.target_id,
                                          (("amount", existing),)))
        children.extend((
            PendingEffect("recycle", effect.source_id, effect.target_id, ()),
            PendingEffect("draw", effect.source_id, effect.target_id,
                          (("amount", amount - existing),)),
        ))
        state.pending[0:0] = children
        return
    for _ in range(amount):
        card = state.piles.draw.pop()
        state.piles.hand.append(card)
        state.public.revealed_order += (str(card.instance_id),)


def apply_effect(state: CombatState, effect: PendingEffect) -> None:
    """Apply the finite pile work catalog using the accepted pure shuffle."""
    if not _valid_pending(effect):
        raise InvariantError("invalid effect value")
    _validate_work(effect)
    if effect.kind == "draw":
        _draw(state, effect, dict(effect.payload)["amount"])
    elif effect.kind == "discard_hand":
        while state.piles.hand:
            discard_card(state, state.piles.hand[-1].instance_id, end_turn=True)
    elif effect.kind == "recycle":
        # Shuffle owns only a detached permutation. On exhaustion the caller's
        # piles remain unchanged; the engine retains the exception's cursor.
        shuffled, stream = rng.shuffle(state.rng.shuffle, state.piles.discard)
        state.rng = replace(state.rng, shuffle=stream)
        state.piles.draw.extend(shuffled)
        state.piles.discard.clear()
    else:
        raise UnsupportedMechanicError(f"not a pile effect: {effect.kind}")
