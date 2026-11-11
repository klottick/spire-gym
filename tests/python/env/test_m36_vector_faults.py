"""M36 fault isolation and SAME_STEP episode statistics coverage."""

import numpy as np
import pytest
from gymnasium.wrappers.vector import RecordEpisodeStatistics

from spire_lab import _native
from spire_lab.gymnasium import RunVectorEnv, StandardCombatVectorEnv
from spire_lab.gymnasium.ironclad_vector import IroncladCombatVectorEnv


def _gym66_entry(entry_id, deck, *, hp=80, max_hp=80, relics=None):
    return {
        "id": entry_id,
        "encounter": "cultist",
        "hp": hp,
        "max_hp": max_hp,
        "deck": deck,
        "relics": relics or [],
        "potions": [None, None, None],
        "relic_counters": {},
    }


def _action_columns(spec):
    tensor = next(row for row in spec["tensors"] if row["key"] == "actions")
    return {name: index for index, name in enumerate(tensor["fields"])}


def _legal_action(observation, spec, verb, *, source_group=None):
    columns = _action_columns(spec)
    verbs = spec["vocabularies"]["verb"]
    groups = spec["vocabularies"]["row_group"]
    for action in np.flatnonzero(observation["action_mask"].reshape(-1)):
        row = observation["actions"][int(action)]
        if verbs[int(row[columns["verb"]])] != verb:
            continue
        if source_group is not None and groups[int(row[columns["source_group"]])] != source_group:
            continue
        return int(action)
    raise AssertionError(f"no legal {verb} action")


def _assert_other_row_unchanged(before, after, slot):
    for key in before:
        np.testing.assert_array_equal(before[key][slot], after[key][slot])


def test_record_episode_statistics_same_step_terminal_plus_fault():
    catalog = {
        "entries": [
            _gym66_entry("winner", [{"card": "searing_blow", "upgrades": 10}] * 32),
            _gym66_entry(
                "limit",
                [{"card": "feed", "upgrades": 0}] * 5,
                hp=900,
                max_hp=999,
                relics=["neows_lament"],
            ),
        ]
    }
    vector = IroncladCombatVectorEnv(
        2,
        catalog,
        partition="test",
        autoreset_mode="same_step",
        fault_policy="isolate",
    )
    wrapped = RecordEpisodeStatistics(vector)
    try:
        observations, _ = wrapped.reset(
            seed=[7, 41], options={"entry_ids": ["winner", "limit"]}
        )
        action = _legal_action(
            {key: value[0] for key, value in observations.items()},
            vector.observation_spec(),
            "play_card",
            source_group="combat_cards",
        )
        observations, rewards, terminated, truncated, info = wrapped.step(
            np.asarray([action, 0], dtype=np.int64)
        )

        assert terminated.tolist() == [True, False]
        assert truncated.tolist() == [False, False]
        assert info["slot_fault"].tolist() == [False, True]
        assert info["_episode"].tolist() == [True, False]
        assert info["episode"]["l"].tolist() == [1, 0]
        assert info["episode"]["r"].tolist() == [1.0, 0.0]
        assert rewards.tolist() == [1.0, 0.0]
        assert info["_final_info"].tolist() == [True, False]
    finally:
        wrapped.close()


def test_run_vector_isolate_freezes_capacity_fault_and_resumes_masked():
    configs = [
        {"character": "ironclad", "ascension": 0},
        {"character": "defect", "ascension": 0},
    ]
    env = RunVectorEnv(
        configs,
        workers=1,
        autoreset_mode="disabled",
        fault_policy="isolate",
        capacities={"actions": 6},
    )
    try:
        observations, _ = env.reset(seed=[1, 255])
        actions = np.asarray(
            [int(np.flatnonzero(row)[0]) for row in observations["action_mask"]],
            dtype=np.int64,
        )
        observations, _, _, _, info = env.step(actions)
        assert info["slot_fault"].tolist() == [True, False]
        assert info["fault"][0]["stage"] == "encode"
        assert "post-step Run encoding failed" in info["fault"][0]["message"]
        assert env.diagnostics()["isolated"] == [True, False]

        fault = info["fault"][0]
        for _ in range(2):
            other_action = int(np.flatnonzero(observations["action_mask"][1])[0])
            observations, _, _, _, info = env.step(
                np.asarray([0, other_action], dtype=np.int64)
            )
            assert info["slot_fault"].tolist() == [True, False]
            assert info["fault"][0] == fault
            assert env.diagnostics()["isolated"] == [True, False]
            assert env.diagnostics()["slots"][1] == "delivered"

        unaffected = {key: value.copy() for key, value in observations.items()}
        before = env.diagnostics()
        with pytest.raises(_native.RunInputError, match="faulted slot needs an explicit seed"):
            env.reset(seed=None)
        with pytest.raises(_native.RunInputError, match="faulted slot needs an explicit seed"):
            env.reset(seed=None, options={"reset_mask": np.asarray([True, False])})
        assert env.diagnostics() == before

        observations, _ = env.reset(
            seed=999, options={"reset_mask": np.asarray([True, False])}
        )
        _assert_other_row_unchanged(unaffected, observations, 1)
        assert env.diagnostics()["isolated"] == [False, False]
    finally:
        env.close()


def test_standard_combat_vector_isolate_freezes_limit_fault_and_resumes_masked():
    """The admitted Standard form of the Gym66 Feed limit fixture is seed-pinned."""
    catalog = {
        "catalog_schema_id": "spire-gym-standard-combat-catalog-v1",
        "profile_id": "standard-combat-v1",
        "entries": [
            {
                "id": "limit",
                "character": "ironclad",
                "ascension": 0,
                "encounter": "cultist",
                "hp": 900,
                "max_hp": 999,
                "gold": 0,
                "deck": [{"card": "feed", "upgrades": 0}] * 5,
                "relics": ["burning_blood"],
                "potions": [None, None, None],
            },
            {
                "id": "healthy",
                "character": "ironclad",
                "ascension": 0,
                "encounter": "cultist",
                "hp": 70,
                "max_hp": 80,
                "gold": 99,
                "deck": [{"card": "strike", "upgrades": 0}] * 10,
                "relics": ["burning_blood"],
                "potions": [None, None, None],
            },
        ],
    }
    env = StandardCombatVectorEnv(
        catalog,
        2,
        entry_ids=["limit", "healthy"],
        partition="test",
        autoreset_mode="disabled",
        fault_policy="isolate",
    )
    try:
        # The Feed kill crosses the 999 max-HP bound on the sixth public action.
        observations, _ = env.reset(seed=[255, 1])
        for step in range(6):
            healthy_action = int(np.flatnonzero(observations["action_mask"][1])[0])
            observations, _, terminated, truncated, info = env.step(
                np.asarray([0, healthy_action], dtype=np.int64)
            )
            if step < 5:
                assert info["slot_fault"].tolist() == [False, False]
                assert not terminated.any() and not truncated.any()

        assert info["slot_fault"].tolist() == [True, False]
        fault = info["fault"][0]
        assert fault["stage"] == "step" and fault["committed"] is True
        assert len(fault["message"]) < 256 and "combat" in fault["message"].lower()
        assert env.diagnostics()["isolated"] == [True, False]

        for _ in range(2):
            healthy_action = int(np.flatnonzero(observations["action_mask"][1])[0])
            observations, _, _, _, info = env.step(
                np.asarray([0, healthy_action], dtype=np.int64)
            )
            assert info["slot_fault"].tolist() == [True, False]
            assert info["fault"][0] == fault
            assert env.diagnostics()["isolated"] == [True, False]
            assert env.diagnostics()["slots"][1] == "delivered"

        unaffected = {key: value.copy() for key, value in observations.items()}
        before = env.diagnostics()
        with pytest.raises(_native.RunInputError, match="faulted slot needs an explicit seed"):
            env.reset(seed=None)
        with pytest.raises(_native.RunInputError, match="faulted slot needs an explicit seed"):
            env.reset(seed=None, options={"reset_mask": np.asarray([True, False])})
        assert env.diagnostics() == before

        observations, _ = env.reset(
            seed=257, options={"reset_mask": np.asarray([True, False])}
        )
        _assert_other_row_unchanged(unaffected, observations, 1)
        assert env.diagnostics()["isolated"] == [False, False]
    finally:
        env.close()
