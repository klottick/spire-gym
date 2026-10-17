from dataclasses import FrozenInstanceError

import pytest

from spire_lab.combat import rules
from spire_lab.combat.model import CardInstance, InvariantError, UnsupportedMechanicError


@pytest.mark.parametrize("card_id,upgraded,expected", [
    ("strike", False, (1, 6, 0, 0)),
    ("strike", True, (1, 9, 0, 0)),
    ("defend", False, (1, 0, 5, 0)),
    ("defend", True, (1, 0, 8, 0)),
    ("bash", False, (2, 8, 0, 2)),
    ("bash", True, (2, 10, 0, 3)),
])
def test_card_stats(card_id, upgraded, expected):
    stats = rules.card_stats(CardInstance(1, card_id, upgraded))
    assert (stats.cost, stats.damage, stats.block, stats.vulnerable) == expected
    with pytest.raises(FrozenInstanceError):
        stats.cost = 0


@pytest.mark.parametrize("base,strength,vulnerable,expected", [
    (9, 0, True, 13), (6, 999, False, 1005), (11, 999, True, 1515),
    (0, 0, True, 0), (7, 3, True, 15), (8, 0, False, 8),
])
def test_normal_damage(base, strength, vulnerable, expected):
    assert rules.normal_damage(base, strength, vulnerable) == expected


@pytest.mark.parametrize("current,amount,expected", [(998, 8, 999), (0, 5, 5), (999, 0, 999)])
def test_block_after_add(current, amount, expected):
    assert rules.block_after_add(current, amount) == expected


@pytest.mark.parametrize("hp,block,damage,expected", [
    (40, 5, 11, (34, 0)), (5, 0, 11, (0, 0)), (40, 20, 11, (40, 9)),
    (40, 11, 11, (40, 0)), (999, 999, 1515, (483, 0)), (0, 0, 0, (0, 0)),
])
def test_hp_and_block_after_damage(hp, block, damage, expected):
    assert rules.hp_and_block_after_damage(hp, block, damage) == expected


@pytest.mark.parametrize("current,delta,expected", [(998, 3, 999), (0, 3, 3), (999, 0, 999)])
def test_clamped_strength(current, delta, expected):
    assert rules.clamped_strength(current, delta) == expected


@pytest.mark.parametrize("function,args", [
    ("normal_damage", (True, 0, False)), ("normal_damage", (-1, 0, False)),
    ("normal_damage", (12, 0, False)), ("normal_damage", (6, 1000, False)),
    ("normal_damage", (6, -1, False)), ("normal_damage", (6, 0, 1)),
    ("block_after_add", (1000, 5)), ("block_after_add", (0, 9)),
    ("block_after_add", (False, 5)), ("block_after_add", (0, -1)),
    ("hp_and_block_after_damage", (1000, 0, 0)),
    ("hp_and_block_after_damage", (1, -1, 0)),
    ("hp_and_block_after_damage", (1, 0, 1516)),
    ("hp_and_block_after_damage", (1, 0, True)),
    ("clamped_strength", (0, -1)), ("clamped_strength", (1000, 3)),
    ("clamped_strength", (0, 4)), ("clamped_strength", (0, True)),
])
def test_arithmetic_rejects_out_of_slice_arguments(function, args):
    with pytest.raises(InvariantError):
        getattr(rules, function)(*args)


@pytest.mark.parametrize("card", [None, CardInstance(True, "strike", False),
    CardInstance(1, "strike", 1), CardInstance(1, [], False)])
def test_malformed_card(card):
    with pytest.raises(InvariantError):
        rules.card_stats(card)


def test_excluded_card():
    with pytest.raises(UnsupportedMechanicError):
        rules.card_stats(CardInstance(1, "cleave", False))
