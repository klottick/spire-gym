"""Pure bounded calculations for the supported combat slice."""

from spire_lab.combat.content import CARD_STATS, CardStats
from spire_lab.combat.model import CardInstance, InvariantError, UnsupportedMechanicError
from spire_lab.combat.rng import RandomStreamState, bernoulli, uniform_below


def _bounded(value: int, maximum: int, name: str) -> None:
    if type(value) is not int or not 0 <= value <= maximum:
        raise InvariantError(f"{name} must be an exact int in [0, {maximum}]")


def card_stats(card: CardInstance) -> CardStats:
    if (type(card) is not CardInstance or type(card.instance_id) is not int
            or type(card.card_id) is not str or type(card.upgraded) is not bool):
        raise InvariantError("malformed card instance")
    try:
        return CARD_STATS[card.card_id, card.upgraded]
    except KeyError:
        raise UnsupportedMechanicError("unsupported card") from None


def normal_damage(base: int, strength: int, vulnerable: bool) -> int:
    _bounded(base, 11, "base damage")
    _bounded(strength, 999, "Strength")
    if type(vulnerable) is not bool:
        raise InvariantError("Vulnerable must be an exact bool")
    damage = base + strength
    # At most 1010 before multiplication; half-integers through 1515 are
    # exactly representable in binary32, so integer floor is equivalent here.
    return (3 * damage) // 2 if vulnerable else damage


def block_after_add(current: int, amount: int) -> int:
    _bounded(current, 999, "block")
    _bounded(amount, 8, "block gain")
    return min(999, current + amount)


def hp_and_block_after_damage(hp: int, block: int, damage: int) -> tuple[int, int]:
    _bounded(hp, 999, "HP")
    _bounded(block, 999, "block")
    _bounded(damage, 1515, "damage")
    return max(0, hp - max(0, damage - block)), max(0, block - damage)


def clamped_strength(current: int, delta: int) -> int:
    _bounded(current, 999, "Strength")
    _bounded(delta, 3, "Strength gain")
    return min(999, current + delta)


def _history(history: tuple[str, ...], moves: tuple[str, ...]) -> None:
    if type(history) is not tuple or any(
        type(move) is not str or move not in moves for move in history
    ):
        raise InvariantError("history must be a tuple of supported enemy moves")


def jaw_worm_fallback(history: tuple[str, ...], roll: int) -> tuple[int, int] | None:
    """Return the exact Bernoulli arguments only for a blocked bucket."""
    _history(history, ("chomp", "thrash", "bellow"))
    _bounded(roll, 99, "AI bucket")
    if not history:
        return None
    if roll < 25 and history[-1] == "chomp":
        return 9, 16
    if 25 <= roll < 55 and history[-2:] == ("thrash", "thrash"):
        return 2994733, 8388608
    if roll >= 55 and history[-1] == "bellow":
        return 3489661, 8388608
    return None


def jaw_worm_move(history: tuple[str, ...], roll: int, fallback: bool | None) -> str:
    """Resolve a supplied bucket and optional fallback without changing history."""
    probability = jaw_worm_fallback(history, roll)
    if fallback is not None and type(fallback) is not bool:
        raise InvariantError("fallback must be an exact bool or None")
    if probability is not None and fallback is None:
        raise InvariantError("blocked bucket requires a fallback result")
    if not history:
        return "chomp"
    if roll < 25:
        return ("bellow" if fallback else "thrash") if probability else "chomp"
    if roll < 55:
        return ("chomp" if fallback else "bellow") if probability else "thrash"
    return ("chomp" if fallback else "thrash") if probability else "bellow"


def choose_enemy_move(
    enemy_id: str, history: tuple[str, ...], stream: RandomStreamState,
) -> tuple[str, RandomStreamState]:
    """Select, but do not append/publish/execute, the next enemy move.

    Sampling exceptions propagate with their latest immutable stream cursor;
    the effect owner must install that cursor when recording an RNG fault.
    """
    if type(enemy_id) is not str:
        raise InvariantError("enemy ID must be an exact str")
    if enemy_id not in ("cultist", "jaw_worm"):
        raise UnsupportedMechanicError("unsupported enemy")
    _history(history, ("incantation", "attack") if enemy_id == "cultist"
             else ("chomp", "thrash", "bellow"))
    if type(stream) is not RandomStreamState or stream.stream_id.domain != "combat.enemy.ai":
        raise InvariantError("selection requires an enemy AI stream")
    if enemy_id == "cultist":
        return ("attack" if history else "incantation"), stream
    if not history:
        return "chomp", stream
    roll, stream = uniform_below(stream, 100)
    probability = jaw_worm_fallback(history, roll)
    fallback = None
    if probability is not None:
        fallback, stream = bernoulli(stream, *probability)
    return jaw_worm_move(history, roll, fallback), stream
