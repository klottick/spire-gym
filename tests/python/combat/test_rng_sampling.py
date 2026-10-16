"""Exact project sampling boundaries using test-local scripted words."""

from dataclasses import replace
from collections import Counter
from itertools import permutations, product

import pytest

import spire_lab.combat.rng as rng
from spire_lab.combat.rng import shuffle


M = 2**64


@pytest.fixture
def s():
    return rng.make_stream(
        bytes(range(32)), "train", "episode-1",
        rng.StreamId("combat.shuffle", ("combat-1",)),
    )


def force_words(monkeypatch, words):
    """Keep real immutable cursor semantics; script shortage is a test error."""
    words = iter(words)
    requested = []

    def draw(state):
        requested.append(state)
        if state.word_index == M:
            raise rng.RngExhaustedError(state)
        try:
            word = next(words)
        except StopIteration:
            raise AssertionError("forced-word script ran out before exhaustion") from None
        return word, replace(state, word_index=state.word_index + 1)

    monkeypatch.setattr(rng, "draw_u64", draw)
    return requested


def forbid_draw(monkeypatch):
    def draw(state):
        pytest.fail("operation must validate or return without drawing")

    monkeypatch.setattr(rng, "draw_u64", draw)


def test_rejection_consumes_only_local_words(monkeypatch, s):
    # Catches biased modulo, reuse of rejected words and cross-stream consumption.
    saved = replace(s)
    other = rng.make_stream(
        bytes(range(32)), "train", "episode-1",
        rng.StreamId("combat.enemy.ai", ("combat-1", "enemy-1")),
    )
    saved_other = replace(other)
    requested = force_words(monkeypatch, [M - 1, 5])
    value, advanced = rng.uniform_below(s, 10)
    assert value == 5
    assert advanced == replace(s, word_index=s.word_index + 2)
    assert requested == [s, replace(s, word_index=s.word_index + 1)]
    assert s == saved
    assert other == saved_other


def test_full_word_bound(monkeypatch, s):
    # Catches overflow or rejection of the last word at the maximal bound.
    requested = force_words(monkeypatch, [M - 1])
    assert rng.uniform_below(s, M) == (M - 1, replace(s, word_index=s.word_index + 1))
    assert requested == [s]


def test_rejection_exhaustion_retains_cursor(monkeypatch, s):
    # Catches wrapping, stale diagnostic cursors and fabricated fallback results.
    start = replace(s, word_index=M - 1)
    saved = replace(start)
    requested = force_words(monkeypatch, [M - 1])
    with pytest.raises(rng.RngExhaustedError) as caught:
        rng.uniform_below(start, 10)
    assert caught.value.state == replace(start, word_index=M)
    assert requested == [start, replace(start, word_index=M)]
    assert start == saved


@pytest.mark.parametrize("cursor", [0, M])
def test_deterministic_cases_do_not_draw(monkeypatch, s, cursor):
    # Catches eager draws, including at the exhausted sentinel.
    s = replace(s, word_index=cursor)
    saved = replace(s)
    forbid_draw(monkeypatch)
    assert (
        rng.uniform_below(s, 1), rng.bernoulli(s, 0, 7),
        rng.bernoulli(s, 7, 7), rng.weighted_index(s, (0, 1, 0)),
    ) == ((0, s), (False, s), (True, s), (1, s))
    assert rng.bernoulli(s, 0, M) == (False, s)
    assert rng.bernoulli(s, M, M) == (True, s)
    assert s == saved


def test_bernoulli_exact_small_outcomes(monkeypatch, s):
    # Catches inclusive comparisons, floating conversion and non-bool results.
    outcomes = []
    for word in range(5):
        requested = force_words(monkeypatch, [word])
        value, advanced = rng.bernoulli(s, 2, 5)
        assert type(value) is bool
        assert advanced == replace(s, word_index=s.word_index + 1)
        assert requested == [s]
        outcomes.append(value)
    assert outcomes == [True, True, False, False, False]
    assert s.word_index == 0


def test_weighted_zero_entries_and_boundaries(monkeypatch, s):
    # Catches selecting zeros or using >= at cumulative boundaries.
    indices = []
    for word in range(5):
        requested = force_words(monkeypatch, [word])
        value, advanced = rng.weighted_index(s, (0, 2, 0, 3))
        indices.append(value)
        assert advanced == replace(s, word_index=s.word_index + 1)
        assert requested == [s]
    assert indices == [1, 1, 3, 3, 3]
    requested = force_words(monkeypatch, [M - 1])
    assert rng.weighted_index(s, (0, M, 0)) == (
        1, replace(s, word_index=s.word_index + 1),
    )
    assert requested == [s]


def test_shuffle_all_three_element_paths(monkeypatch, s):
    # Catches wrong bounds, loop direction, swaps, or lost cursor advancement.
    start = replace(s, word_index=7)
    outputs = []
    for choices in product(range(3), range(2)):
        scripted = iter(choices)
        requested = []

        def choose(state, bound):
            requested.append(bound)
            return next(scripted), replace(state, word_index=state.word_index + 1)

        monkeypatch.setattr(rng, "uniform_below", choose)
        value, advanced = shuffle(start, ("a", "b", "c"))
        assert requested == [3, 2]
        assert advanced == replace(start, word_index=start.word_index + 2)
        outputs.append(tuple(value))
    assert Counter(outputs) == Counter(permutations(("a", "b", "c")))
    assert start.word_index == 7


def test_shuffle_ownership_and_exhaustion(monkeypatch, s):
    # Catches aliasing/deep-copying elements and leaking partial results on exhaustion.
    a, b = object(), object()
    items = [a, b, a]
    force_words(monkeypatch, [0, 0])
    value, advanced = shuffle(s, items)
    assert type(value) is list and value is not items
    assert sum(item is a for item in value) == 2
    assert sum(item is b for item in value) == 1
    assert items[0] is a and items[1] is b and items[2] is a
    assert advanced == replace(s, word_index=2)
    assert s.word_index == 0

    items = ["a", "b", "c", "d"]
    start = replace(s, word_index=M - 2)
    requested = force_words(monkeypatch, [0, M - 1])
    with pytest.raises(rng.RngExhaustedError) as caught:
        shuffle(start, items)
    assert caught.value.state == replace(start, word_index=M)
    assert requested == [start, replace(start, word_index=M - 1),
                         replace(start, word_index=M)]
    assert items == ["a", "b", "c", "d"]
    assert start.word_index == M - 2


@pytest.mark.parametrize("cursor", [0, M])
def test_shuffle_empty_singleton_and_invalid_inputs(monkeypatch, s, cursor):
    # Catches eager draws, accepting foreign containers, or iterating before validation.
    s = replace(s, word_index=cursor)
    forbid_draw(monkeypatch)
    a = object()
    for items in ([], (), [a], (a,)):
        value, advanced = shuffle(s, items)
        assert type(value) is list and value is not items
        assert len(value) == len(items)
        if items:
            assert value[0] is a
        assert advanced == s

    class ForeignList(list):
        def __iter__(self):
            pytest.fail("invalid list subclass was iterated")

    class ForeignTuple(tuple):
        def __iter__(self):
            pytest.fail("invalid tuple subclass was iterated")

    def generator():
        pytest.fail("invalid generator was iterated")
        yield a

    for items in (None, generator(), ForeignList([a]), ForeignTuple((a,))):
        with pytest.raises(TypeError):
            shuffle(s, items)

    class ForeignState(rng.RandomStreamState):
        pass

    for state in (None, object(), 0, ForeignState(s.stream_id, s.key, cursor)):
        with pytest.raises(TypeError):
            shuffle(state, [])
    assert s.word_index == cursor


class IntSubclass(int):
    pass


class TupleSubclass(tuple):
    pass


INVALID_CASES = [
    ("uniform_below", (bound,), ValueError)
    for bound in (0, -1, M + 1)
] + [
    ("bernoulli", args, ValueError)
    for args in ((-1, 5), (6, 5), (0, 0), (1, -1), (1, M + 1))
] + [
    ("weighted_index", (weights,), ValueError)
    for weights in ((), (0, 0), (1, -1), (M, 1))
] + [
    ("weighted_index", (weights,), TypeError)
    for weights in ([1], (weight for weight in (1,)), TupleSubclass((1,)), (1, object()))
]
for invalid_int in (True, 1.0, "1", IntSubclass(1)):
    INVALID_CASES.extend([
        ("uniform_below", (invalid_int,), TypeError),
        ("bernoulli", (invalid_int, 5), TypeError),
        ("bernoulli", (0, invalid_int), TypeError),
        ("weighted_index", ((1, invalid_int),), TypeError),
    ])


@pytest.mark.parametrize("cursor", [0, M])
@pytest.mark.parametrize("operation,args,error", INVALID_CASES)
def test_invalid_sampler_inputs_do_not_draw(monkeypatch, s, cursor, operation, args, error):
    # Catches coercion, incomplete/late validation and validation after drawing.
    s = replace(s, word_index=cursor)
    saved = replace(s)
    forbid_draw(monkeypatch)
    with pytest.raises(error):
        getattr(rng, operation)(s, *args)
    assert s == saved


@pytest.mark.parametrize("state", [None, object(), 0])
@pytest.mark.parametrize("operation,args", [
    ("uniform_below", (1,)), ("uniform_below", (10,)),
    ("bernoulli", (0, 7)), ("bernoulli", (7, 7)), ("bernoulli", (2, 5)),
    ("weighted_index", ((0, 1, 0),)), ("weighted_index", ((2, 3),)),
])
def test_invalid_sampler_state_does_not_draw(monkeypatch, state, operation, args):
    # Catches deterministic early returns bypassing state validation.
    forbid_draw(monkeypatch)
    with pytest.raises(TypeError):
        getattr(rng, operation)(state, *args)
