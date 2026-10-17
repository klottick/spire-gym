from dataclasses import replace
from fractions import Fraction

import pytest

from spire_lab.combat import rules
from spire_lab.combat.model import InvariantError, UnsupportedMechanicError
from spire_lab.combat.rng import RngExhaustedError


@pytest.mark.parametrize("history,expected", [
    ((), (1, 0, 0)),
    (("chomp",), (0, Fraction(131, 320), Fraction(189, 320))),
    (("chomp", "bellow"), (Fraction(73349989, 167772160), Fraction(94422171, 167772160), 0)),
    (("chomp", "thrash"), (Fraction(1, 4), Fraction(3, 10), Fraction(9, 20))),
    (("chomp", "thrash", "thrash"), (Fraction(29955719, 83886080), 0, Fraction(53930361, 83886080))),
])
def test_exact_history_laws(history, expected):
    masses = dict.fromkeys(("chomp", "thrash", "bellow"), Fraction(0))
    for roll in range(100):
        probability = rules.jaw_worm_fallback(history, roll)
        if probability is None:
            move = rules.jaw_worm_move(history, roll, None)
            assert rules.jaw_worm_move(history, roll, True) == move
            assert rules.jaw_worm_move(history, roll, False) == move
            masses[move] += Fraction(1, 100)
        else:
            p = Fraction(*probability)
            for fallback, weight in ((True, p), (False, 1 - p)):
                masses[rules.jaw_worm_move(history, roll, fallback)] += weight / 100
    assert tuple(masses.values()) == expected
    assert sum(masses.values()) == 1


@pytest.mark.parametrize("history,roll,probability,yes,no", [
    (("chomp",), 24, (9, 16), "bellow", "thrash"),
    (("chomp",), 25, None, "thrash", "thrash"),
    (("chomp",), 54, None, "thrash", "thrash"),
    (("chomp",), 55, None, "bellow", "bellow"),
    (("bellow",), 99, (3489661, 8388608), "chomp", "thrash"),
    (("thrash", "thrash"), 25, (2994733, 8388608), "chomp", "bellow"),
    (("thrash", "thrash"), 54, (2994733, 8388608), "chomp", "bellow"),
    (("thrash", "thrash"), 24, None, "chomp", "chomp"),
    (("thrash", "thrash"), 55, None, "bellow", "bellow"),
])
def test_bucket_boundaries(history, roll, probability, yes, no):
    assert rules.jaw_worm_fallback(history, roll) == probability
    assert rules.jaw_worm_move(history, roll, True) == yes
    assert rules.jaw_worm_move(history, roll, False) == no


@pytest.mark.parametrize("enemy,history,move", [
    ("jaw_worm", (), "chomp"), ("cultist", (), "incantation"),
    ("cultist", ("incantation",), "attack"),
    ("cultist", ("incantation", "attack", "attack"), "attack"),
])
def test_forced_moves_consume_no_words(scenario_factory, enemy, history, move):
    stream = replace(scenario_factory().rng.enemy_ai, word_index=2**64)
    selected, after = rules.choose_enemy_move(enemy, history, stream)
    assert selected == move
    assert after is stream


@pytest.mark.parametrize("history,roll,probability,move", [
    (("chomp",), 24, (9, 16), "bellow"),
    (("chomp",), 25, None, "thrash"),
    (("thrash", "thrash"), 54, (2994733, 8388608), "chomp"),
    (("bellow",), 55, (3489661, 8388608), "chomp"),
    (("thrash",), 99, None, "bellow"),
])
def test_only_blocked_bucket_draws_fallback(monkeypatch, scenario_factory, history, roll, probability, move):
    stream = scenario_factory().rng.enemy_ai
    bucket_stream = replace(stream, word_index=1)
    fallback_stream = replace(stream, word_index=2)
    calls = []

    def uniform(state, bound):
        assert state is stream
        calls.append(("bucket", bound))
        return roll, bucket_stream

    def bernoulli(state, numerator, denominator):
        assert state is bucket_stream
        calls.append(("fallback", numerator, denominator))
        return True, fallback_stream

    monkeypatch.setattr(rules, "uniform_below", uniform)
    monkeypatch.setattr(rules, "bernoulli", bernoulli)
    selected, after = rules.choose_enemy_move("jaw_worm", history, stream)
    assert selected == move
    assert after is (fallback_stream if probability else bucket_stream)
    assert calls == [("bucket", 100)] + ([("fallback", *probability)] if probability else [])
    assert history[-1] in {"chomp", "thrash", "bellow"}
    assert stream.word_index == 0


def test_real_project_first_roll(scenario_factory):
    stream = scenario_factory().rng.enemy_ai
    move, after = rules.choose_enemy_move("jaw_worm", ("chomp",), stream)
    assert move == "thrash"
    assert after.word_index == 1
    assert stream.word_index == 0


@pytest.mark.parametrize("history,roll", [
    (("chomp",), True), (("chomp",), -1), (("chomp",), 100),
    (("foreign",), 0), (["chomp"], 0), ((1,), 0),
])
def test_malformed_bucket_input(history, roll):
    for function, args in ((rules.jaw_worm_fallback, (history, roll)),
                           (rules.jaw_worm_move, (history, roll, None))):
        with pytest.raises(InvariantError):
            function(*args)


@pytest.mark.parametrize("enemy,history,error", [
    ("jaw_worm", ("foreign",), InvariantError),
    ("cultist", ("chomp",), InvariantError),
    ("jaw_worm", ["chomp"], InvariantError),
    ("jaw_worm", (True,), InvariantError),
    ("louse", (), UnsupportedMechanicError),
    (True, (), InvariantError), ([], (), InvariantError),
])
def test_malformed_selection_input_does_not_sample(monkeypatch, scenario_factory, enemy, history, error):
    def unexpected(*args):
        pytest.fail("invalid selection sampled")
    monkeypatch.setattr(rules, "uniform_below", unexpected)
    monkeypatch.setattr(rules, "bernoulli", unexpected)
    with pytest.raises(error):
        rules.choose_enemy_move(enemy, history, scenario_factory().rng.enemy_ai)


@pytest.mark.parametrize("fallback", [None, 0, 1, "yes"])
def test_blocked_bucket_requires_boolean(fallback):
    with pytest.raises(InvariantError):
        rules.jaw_worm_move(("chomp",), 0, fallback)


def test_selection_exhaustion_preserves_stream(scenario_factory):
    stream = replace(scenario_factory().rng.enemy_ai, word_index=2**64)
    with pytest.raises(RngExhaustedError) as caught:
        rules.choose_enemy_move("jaw_worm", ("chomp",), stream)
    assert caught.value.state is stream


def test_fallback_exhaustion_preserves_bucket_progress(monkeypatch, scenario_factory):
    stream = replace(scenario_factory().rng.enemy_ai, word_index=2**64 - 1)
    exhausted = replace(stream, word_index=2**64)

    def uniform(state, bound):
        assert state is stream
        assert bound == 100
        return 0, exhausted

    monkeypatch.setattr(rules, "uniform_below", uniform)
    with pytest.raises(RngExhaustedError) as caught:
        rules.choose_enemy_move("jaw_worm", ("chomp",), stream)
    assert caught.value.state is exhausted
    assert stream.word_index == 2**64 - 1


@pytest.mark.parametrize("enemy,history", [("cultist", ()), ("jaw_worm", ("chomp",))])
@pytest.mark.parametrize("bad_stream", [None, "shuffle"])
def test_invalid_stream_does_not_sample(monkeypatch, scenario_factory, enemy, history, bad_stream):
    def unexpected(*args):
        pytest.fail("invalid stream sampled")
    monkeypatch.setattr(rules, "uniform_below", unexpected)
    monkeypatch.setattr(rules, "bernoulli", unexpected)
    stream = scenario_factory().rng.shuffle if bad_stream == "shuffle" else None
    with pytest.raises(InvariantError):
        rules.choose_enemy_move(enemy, history, stream)
