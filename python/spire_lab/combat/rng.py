"""Immutable project-owned random streams (intentional deviation RNG-001).

This is a standalone project contract, not an original-game RNG implementation.
Identity, key and next-word cursor are the complete replay state.
"""

from dataclasses import dataclass
import hmac
from types import MappingProxyType


RNG_CONTRACT_ID = "spire-lab-rng-hmac-sha256-v1"
_WORD_LIMIT = 2**64
_DOMAIN_ARITIES = MappingProxyType({
    "combat.shuffle": 1,
    "combat.enemy.hp": 2,
    "combat.enemy.ai": 2,
})
_PARTITIONS = frozenset({"train", "selection", "final", "test"})
_STREAM_PREFIX = b"spire-lab/rng/v1/stream\0"
_BLOCK_PREFIX = b"spire-lab/rng/v1/block\0"


def _validate_field(value: str, name: str) -> None:
    if type(value) is not str:
        raise TypeError(f"{name} must be an exact str")
    if not 1 <= len(value) <= 255 or any(not "!" <= char <= "~" for char in value):
        raise ValueError(f"{name} must contain 1–255 printable non-space ASCII bytes")


def _validate_key(value: bytes, name: str) -> None:
    if type(value) is not bytes:
        raise TypeError(f"{name} must be exact bytes")
    if len(value) != 32:
        raise ValueError(f"{name} must be exactly 32 bytes")


@dataclass(frozen=True)
class StreamId:
    """A registered semantic domain and its immutable scope identifiers."""

    domain: str
    scope: tuple[str, ...]

    def __post_init__(self) -> None:
        _validate_field(self.domain, "domain")
        if self.domain not in _DOMAIN_ARITIES:
            raise ValueError("unknown random stream domain")
        if type(self.scope) is not tuple:
            raise TypeError("scope must be an exact tuple")
        if len(self.scope) != _DOMAIN_ARITIES[self.domain]:
            raise ValueError("scope arity does not match domain")
        for field in self.scope:
            _validate_field(field, "scope identifier")


@dataclass(frozen=True)
class RandomStreamState:
    """Complete replay value; 2**64 is the exhausted next-word sentinel."""

    stream_id: StreamId
    key: bytes
    word_index: int

    def __post_init__(self) -> None:
        if type(self.stream_id) is not StreamId:
            raise TypeError("stream_id must be an exact StreamId")
        _validate_key(self.key, "key")
        if type(self.word_index) is not int:
            raise TypeError("word_index must be an exact int")
        if not 0 <= self.word_index <= _WORD_LIMIT:
            raise ValueError("word_index must be between 0 and 2**64 inclusive")


class RngExhaustedError(Exception):
    """Administrative exhaustion, carrying the latest state without reset."""

    def __init__(self, state: RandomStreamState) -> None:
        self.state = state
        super().__init__("random stream exhausted")


def _frame_fields(fields: tuple[str, ...]) -> bytes:
    encoded = [field.encode("ascii") for field in fields]
    return len(encoded).to_bytes(4, "big") + b"".join(
        len(field).to_bytes(4, "big") + field for field in encoded
    )


def _hmac_sha256(key: bytes, message: bytes) -> bytes:
    return hmac.digest(key, message, "sha256")


def make_stream(
    root_seed: bytes, partition: str, episode_id: str, stream_id: StreamId,
) -> RandomStreamState:
    """Derive a fresh cursor deterministically, without consuming another stream."""
    _validate_key(root_seed, "root_seed")
    _validate_field(partition, "partition")
    if partition not in _PARTITIONS:
        raise ValueError("unknown random stream partition")
    _validate_field(episode_id, "episode_id")
    if type(stream_id) is not StreamId:
        raise TypeError("stream_id must be an exact StreamId")
    frame = _frame_fields((partition, episode_id, stream_id.domain, *stream_id.scope))
    key = _hmac_sha256(root_seed, _STREAM_PREFIX + frame)
    return RandomStreamState(stream_id, key, 0)


def draw_u64(state: RandomStreamState) -> tuple[int, RandomStreamState]:
    """Return one big-endian lane and a new state, never mutating the input."""
    if type(state) is not RandomStreamState:
        raise TypeError("state must be an exact RandomStreamState")
    if state.word_index == _WORD_LIMIT:
        raise RngExhaustedError(state)
    block, lane = divmod(state.word_index, 4)
    digest = _hmac_sha256(state.key, _BLOCK_PREFIX + block.to_bytes(8, "big"))
    value = int.from_bytes(digest[lane * 8:(lane + 1) * 8], "big")
    return value, RandomStreamState(state.stream_id, state.key, state.word_index + 1)


def uniform_below(state: RandomStreamState, bound: int) -> tuple[int, RandomStreamState]:
    """Sample below an exact integer bound, consuming every rejected word."""
    if type(state) is not RandomStreamState:
        raise TypeError("state must be an exact RandomStreamState")
    if type(bound) is not int:
        raise TypeError("bound must be an exact int")
    if not 1 <= bound <= _WORD_LIMIT:
        raise ValueError("bound must be between 1 and 2**64 inclusive")
    if bound == 1:
        return 0, state
    limit = _WORD_LIMIT - _WORD_LIMIT % bound
    while True:
        value, state = draw_u64(state)
        if value < limit:
            return value % bound, state


def bernoulli(
    state: RandomStreamState, numerator: int, denominator: int,
) -> tuple[bool, RandomStreamState]:
    """Sample an exact rational probability; endpoints consume no words."""
    if type(state) is not RandomStreamState:
        raise TypeError("state must be an exact RandomStreamState")
    if type(numerator) is not int or type(denominator) is not int:
        raise TypeError("numerator and denominator must be exact ints")
    if not 1 <= denominator <= _WORD_LIMIT or not 0 <= numerator <= denominator:
        raise ValueError("require 0 <= numerator <= denominator <= 2**64, denominator > 0")
    if numerator == 0:
        return False, state
    if numerator == denominator:
        return True, state
    value, state = uniform_below(state, denominator)
    return value < numerator, state


def weighted_index(
    state: RandomStreamState, weights: tuple[int, ...],
) -> tuple[int, RandomStreamState]:
    """Choose by integer weights, validating every entry before sampling."""
    if type(state) is not RandomStreamState:
        raise TypeError("state must be an exact RandomStreamState")
    if type(weights) is not tuple:
        raise TypeError("weights must be an exact tuple")
    total = 0
    for weight in weights:
        if type(weight) is not int:
            raise TypeError("weights must contain exact ints")
        if weight < 0:
            raise ValueError("weights must be nonnegative")
        total += weight
    if not 1 <= total <= _WORD_LIMIT:
        raise ValueError("weights must have a positive sum at most 2**64")
    value, state = uniform_below(state, total)
    cumulative = 0
    for index, weight in enumerate(weights):
        cumulative += weight
        if cumulative > value:
            return index, state
    raise AssertionError("validated weights must contain the sampled value")


def shuffle[T](
    state: RandomStreamState, items: list[T] | tuple[T, ...],
) -> tuple[list[T], RandomStreamState]:
    """Return a detached Fisher–Yates permutation, preserving element identity."""
    if type(state) is not RandomStreamState:
        raise TypeError("state must be an exact RandomStreamState")
    if type(items) is not list and type(items) is not tuple:
        raise TypeError("items must be an exact list or tuple")
    result = list(items)
    for index in range(len(result) - 1, 0, -1):
        selected, state = uniform_below(state, index + 1)
        result[index], result[selected] = result[selected], result[index]
    return result, state
