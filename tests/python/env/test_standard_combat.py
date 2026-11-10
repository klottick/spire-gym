"""Python facade and transport contract for Standard fresh-start combats."""

import json
from pathlib import Path

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import IroncladCombatEnv, RunEnv, StandardCombatEnv
from spire_lab.gymnasium.spaces import spaces_from_spec


CHARACTER = {
    "ironclad": ("burning_blood", ["strike"] * 5 + ["defend"] * 4 + ["bash"]),
    "silent": ("ring_of_the_snake", ["strike_g"] * 5 + ["defend_g"] * 5 + ["neutralize", "survivor"]),
    "defect": ("cracked_core", ["strike_b"] * 4 + ["defend_b"] * 4 + ["zap", "dualcast"]),
    "watcher": ("pure_water", ["strike_p"] * 4 + ["defend_p"] * 4 + ["eruption", "vigilance"]),
}


def entry(entry_id="base", *, character="ironclad", ascension=0, encounter="cultist", deck=None, potions=None, burning=None):
    relic, starter_deck = CHARACTER[character]
    row = {
        "id": entry_id,
        "character": character,
        "ascension": ascension,
        "encounter": encounter,
        "hp": 70,
        "max_hp": 80,
        "gold": 99,
        "deck": [{"card": card, "upgrades": 0} for card in (deck or starter_deck)],
        "relics": [relic],
        "potions": potions if potions is not None else [None] * (2 if ascension >= 11 else 3),
    }
    if burning is not None:
        row["burning"] = burning
    return row


def catalog(*entries):
    return {
        "catalog_schema_id": "spire-gym-standard-combat-catalog-v1",
        "profile_id": "standard-combat-v1",
        "entries": list(entries),
    }


def observation_equal(left, right):
    assert left.keys() == right.keys()
    for key in left:
        np.testing.assert_array_equal(left[key], right[key])


def legal_action(observation, spec, verb):
    action_tensor = next(tensor for tensor in spec["tensors"] if tensor["key"] == "actions")
    columns = {name: index for index, name in enumerate(action_tensor["fields"])}
    verbs = spec["vocabularies"]["verb"]
    for action in np.flatnonzero(observation["action_mask"].reshape(-1)):
        row = observation["actions"][int(action)]
        if verbs[int(row[columns["verb"]])] == verb:
            return int(action)
    raise AssertionError(f"no legal {verb} action")


def maybe_legal_action(observation, spec, verb):
    action_tensor = next(tensor for tensor in spec["tensors"] if tensor["key"] == "actions")
    columns = {name: index for index, name in enumerate(action_tensor["fields"])}
    verbs = spec["vocabularies"]["verb"]
    for action in np.flatnonzero(observation["action_mask"].reshape(-1)):
        row = observation["actions"][int(action)]
        if verbs[int(row[columns["verb"]])] == verb:
            return int(action)
    return None


def rollout(env, seed):
    observation, _ = env.reset(seed=seed)
    assert env.observation_space.contains(observation)
    observations = [{key: value.copy() for key, value in observation.items()}]
    rewards = []
    for _ in range(100):
        spec = env.observation_spec()
        action = maybe_legal_action(observation, spec, "play_card")
        if action is None:
            action = maybe_legal_action(observation, spec, "end_turn")
        if action is None:
            action = int(np.flatnonzero(observation["action_mask"].reshape(-1))[0])
        observation, reward, terminated, truncated, _ = env.step(action)
        assert env.observation_space.contains(observation)
        observations.append({key: value.copy() for key, value in observation.items()})
        rewards.append(reward)
        if terminated:
            break
        assert not truncated
    return observations, rewards


@pytest.mark.parametrize("character", list(CHARACTER))
@pytest.mark.parametrize("ascension", [0, 20])
def test_standard_combat_spaces_and_observations_match_native_spec(character, ascension):
    env = StandardCombatEnv(catalog(entry(character=character, ascension=ascension)), partition="test")
    try:
        spec = env.observation_spec()
        observations, actions = spaces_from_spec(spec)
        assert env.observation_space == observations
        assert env.action_space == actions
        observation, info = env.reset(seed=17)
        assert env.observation_space.contains(observation)
        assert info["start_kind"] == "fresh_start"
        assert "seed" not in info
        environment_spec = env.environment_spec()
        assert "seed" not in environment_spec
        assert environment_spec["seed_adapter_id"] != 17
    finally:
        env.close()


def test_standard_combat_seeded_rollout_is_reproducible():
    entries = catalog(entry())
    first, second = StandardCombatEnv(entries, partition="test"), StandardCombatEnv(entries, partition="test")
    try:
        left, left_rewards = rollout(first, 71)
        right, right_rewards = rollout(second, 71)
        assert len(left) == len(right)
        for left_obs, right_obs in zip(left, right):
            observation_equal(left_obs, right_obs)
        assert left_rewards == right_rewards
    finally:
        first.close()
        second.close()


@pytest.mark.parametrize(
    "bad_entry",
    [
        dict(entry(), draw_order=["strike"]),
        entry(ascension=21),
        entry(encounter="jaw_worm", burning="strength"),
    ],
)
def test_standard_combat_rejects_invalid_catalog(bad_entry):
    with pytest.raises(_native.RunInputError):
        StandardCombatEnv(catalog(bad_entry), partition="test")


def test_combat_facades_reject_the_other_catalog_boundary():
    standard = catalog(entry())
    gym66 = {"profile_id": "ironclad-combat-v2", "entries": [{
        "id": "base", "encounter": "cultist", "hp": 80, "max_hp": 80,
        "deck": [{"card": "strike", "upgrades": 0}] * 10,
        "relics": [], "potions": [None, None, None], "relic_counters": {},
    }]}
    with pytest.raises(_native.RunUnsupportedBoundaryError):
        IroncladCombatEnv(standard, partition="test")
    with pytest.raises(_native.RunUnsupportedBoundaryError):
        StandardCombatEnv(gym66, partition="test")


def test_standard_combat_terminals_and_smoke_bomb_escape():
    victory = entry("win", deck=["bludgeon"] * 10)
    victory["deck"] = [{"card": "bludgeon", "upgrades": 1} for _ in range(10)]
    env = StandardCombatEnv(catalog(victory), partition="test")
    try:
        observation, info = env.reset(seed=5)
        assert info["start_kind"] == "fresh_start"
        terminal_reason = None
        for _ in range(100):
            spec = env.observation_spec()
            action = maybe_legal_action(observation, spec, "play_card")
            if action is None:
                action = maybe_legal_action(observation, spec, "end_turn")
            if action is None:
                action = int(np.flatnonzero(observation["action_mask"].reshape(-1))[0])
            observation, _, terminated, truncated, info = env.step(action)
            assert env.observation_space.contains(observation)
            assert not truncated
            if terminated:
                terminal_reason = info["terminal_reason"]
                break
        assert terminal_reason in {"victory", "death", "escaped"}
    finally:
        env.close()

    escape = entry("escape", character="defect", potions=["smoke_bomb", None, None])
    env = StandardCombatEnv(catalog(escape), partition="test")
    try:
        observation, _ = env.reset(seed=5)
        action = legal_action(observation, env.observation_spec(), "use_potion")
        _, reward, terminated, truncated, info = env.step(action)
        assert terminated and not truncated
        assert (reward, info["terminal_reason"]) == (0.0, "escaped")
    finally:
        env.close()


def test_standard_run_combat_view_layout_matches_standard_combat_env():
    combat = StandardCombatEnv(catalog(entry()), partition="test")
    run = RunEnv("ironclad", 0, profile="standard", partition="test")
    try:
        combat_obs, _ = combat.reset(seed=43)
        run.reset(seed=43)
        view = run.combat_view()
        run_obs = view["obs"]
        assert combat.observation_spec() == run.combat_observation_spec()
        assert set(run_obs) == set(combat_obs)
        for key in combat_obs:
            assert run_obs[key].shape == combat_obs[key].shape
            assert run_obs[key].dtype == combat_obs[key].dtype
    finally:
        combat.close()
        run.close()


def test_standard_combat_arrays_are_owned_and_decision_is_unchanged_by_mutation():
    native = _native.NativeStandardCombatSession(json.dumps(catalog(entry())), partition="test")
    try:
        boundary = native.reset(seed=29)
        decision = native.decision()
        for array in boundary["obs"].values():
            assert array.flags.writeable
            array[...] = 0
        observation_equal(native.decision()["obs"], decision["obs"])
    finally:
        native.close()


def test_pre_m34s_standard_record_is_typed_input_error():
    fixture = Path(__file__).resolve().parents[3] / "crates/spire-env/src/tests/fixtures/pre_m34s_standard_record_v1.json"
    # Rules id v2 (M34s-a) is checked before the fingerprint; both make a pre-M34s record incompatible.
    with pytest.raises(_native.RunInputError, match="Run identity mismatch"):
        _native.NativeRunSession.replay(fixture.read_bytes())
