"""Python transport checks for Standard synthetic positions and recipes."""

import hashlib
import json

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import RunEnv, StandardCombatEnv


CHARACTER = {
    "ironclad": ("burning_blood", ["strike"] * 5 + ["defend"] * 4 + ["bash"]),
    "silent": ("ring_of_the_snake", ["strike_g"] * 5 + ["defend_g"] * 5 + ["neutralize", "survivor"]),
    "defect": ("cracked_core", ["strike_b"] * 4 + ["defend_b"] * 4 + ["zap", "dualcast"]),
    "watcher": ("pure_water", ["strike_p"] * 4 + ["defend_p"] * 4 + ["eruption", "vigilance"]),
}


def entry(entry_id="fresh", *, character="ironclad", encounter="cultist", deck=None):
    relic, starter = CHARACTER[character]
    return {
        "id": entry_id,
        "character": character,
        "ascension": 0,
        "encounter": encounter,
        "hp": 70,
        "max_hp": 80,
        "gold": 99,
        "deck": [{"card": card, "upgrades": int(card == "true_grit")} for card in (deck or starter)],
        "relics": [relic],
        "potions": [None, None, None],
    }


def row(card, upgrades=0, generated=False):
    return {"card": card, "upgrades": upgrades, "misc": 0, "generated": generated}


def lesson_l1():
    cards = ["strike"] * 4 + ["defend"] * 3 + ["bash", "true_grit", "burning_pact", "second_wind", "shrug_it_off"]
    result = entry("lesson-l1", encounter="gremlin_nob", deck=cards)
    result["position"] = {
        "player": {"powers": {"feel_no_pain": 3, "dark_embrace": 1}},
        "piles": {
            "hand": [row("true_grit", 1), row("burning_pact"), row("second_wind"), row("shrug_it_off"), row("bash")],
            "draw": [row("strike")] * 3 + [row("defend")] * 3,
            "discard": [row("strike")],
        },
    }
    return result


def lesson_l2():
    cards = ["strike_g"] * 3 + ["defend_g"] * 2 + ["neutralize", "survivor", "deadly_poison", "bouncing_flask", "catalyst", "footwork"]
    result = entry("lesson-l2", character="silent", encounter="jaw_worm", deck=cards)
    for deck_row in result["deck"]:
        deck_row["upgrades"] = int(deck_row["card"] == "catalyst")
    result["position"] = {
        "enemies": [{"slot": 0, "powers": {"poison": 4}}],
        "piles": {
            "hand": [row("deadly_poison"), row("bouncing_flask"), row("catalyst", 1), row("neutralize"), row("footwork")],
            "draw": [row("strike_g")] * 3 + [row("defend_g")] * 2 + [row("survivor")],
        },
    }
    return result


def catalog(*entries):
    return {
        "catalog_schema_id": "spire-gym-standard-combat-catalog-v1",
        "profile_id": "standard-combat-v1",
        "entries": list(entries),
    }


def assert_observations_equal(left, right):
    assert left.keys() == right.keys()
    for key in left:
        np.testing.assert_array_equal(left[key], right[key])


def test_synthetic_lesson_constructs_steps_and_descriptor_marks_start_kind():
    env = StandardCombatEnv(catalog(entry(), lesson_l1()), partition="test")
    try:
        descriptor = env.catalog_descriptor()["entries"]
        assert descriptor[0]["start_kind"] == "fresh_start"
        assert descriptor[1]["start_kind"] == "synthetic_position"
        observation, info = env.reset(seed=17, options={"entry_id": "lesson-l1"})
        assert info["start_kind"] == "synthetic_position"
        action = int(np.flatnonzero(observation["action_mask"].reshape(-1))[0])
        next_observation, _, terminated, truncated, _ = env.step(action)
        assert next_observation["action_mask"].any() or terminated
        assert not truncated
    finally:
        env.close()


@pytest.mark.parametrize(
    "mutate, message",
    [
        (lambda value: value["position"]["player"].update(powers={"unknown_power": 1}), "player.powers"),
        (lambda value: value["position"].update(seed=1), "seed"),
        (lambda value: value["position"]["piles"]["discard"].clear(), "piles"),
    ],
)
def test_invalid_positions_are_run_input_errors(mutate, message):
    value = lesson_l1()
    mutate(value)
    with pytest.raises(_native.RunInputError, match=message):
        StandardCombatEnv(catalog(value), partition="test")


def test_same_position_and_seed_reproduce_observation_and_info():
    entries = catalog(lesson_l2())
    first = StandardCombatEnv(entries, partition="test")
    second = StandardCombatEnv(entries, partition="test")
    try:
        first_obs, first_info = first.reset(seed=912)
        second_obs, second_info = second.reset(seed=912)
        assert_observations_equal(first_obs, second_obs)
        assert first_info == second_info
    finally:
        first.close()
        second.close()


def make_recipe_record(*, enter_combat):
    run = RunEnv("ironclad", 0, profile="standard", partition="test", creation={"neow": "short"}, record_replay=True)
    try:
        observation, _ = run.reset(seed=121)
        if enter_combat:
            for _ in range(80):
                if run.combat_view()["obs"]["enemies_occupied"].any():
                    break
                legal = np.flatnonzero(observation["action_mask"].reshape(-1))
                observation, _, terminated, truncated, _ = run.step(int(legal[0]))
                assert not terminated and not truncated
            else:
                raise AssertionError("Standard Run did not reach its first combat decision")
        encoded = run.record()
        assert encoded is not None
        steps = len(json.loads(encoded)["steps"])
        return encoded, steps
    finally:
        run.close()


def test_reachable_recipe_loads_python_standard_run_record():
    record, steps = make_recipe_record(enter_combat=True)
    digest = hashlib.sha256(record).hexdigest()
    recipe = {"id": "route-combat", "recipe": {"record_sha256": digest, "steps": steps}}
    env = StandardCombatEnv(catalog(recipe), records=[record], partition="test")
    try:
        assert env.catalog_descriptor()["entries"][0]["start_kind"] == "reachable_recipe"
        observation, info = env.reset(seed=33)
        assert info["start_kind"] == "reachable_recipe"
        assert observation["enemies_occupied"].any()
    finally:
        env.close()


def test_recipe_errors_and_neow_boundary_mapping():
    record, _ = make_recipe_record(enter_combat=True)
    bad_digest = {"id": "missing", "recipe": {"record_sha256": "0" * 64, "steps": 1}}
    with pytest.raises(_native.RunInputError, match="record not supplied"):
        StandardCombatEnv(catalog(bad_digest), records=[record], partition="test")

    initial, steps = make_recipe_record(enter_combat=False)
    assert steps == 0
    neow = {"id": "neow", "recipe": {"record_sha256": hashlib.sha256(initial).hexdigest(), "steps": 0}}
    with pytest.raises(_native.RunUnsupportedBoundaryError):
        StandardCombatEnv(catalog(neow), records=[initial], partition="test")


def test_standard_native_records_argument_requires_a_sequence_of_bytes():
    with pytest.raises(TypeError, match="records"):
        _native.NativeStandardCombatSession(json.dumps(catalog(entry())), records="not bytes")
    with pytest.raises(TypeError, match="bytes"):
        _native.NativeStandardCombatSession(json.dumps(catalog(entry())), records=["not bytes"])
