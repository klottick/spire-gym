"""Gymnasium contract for standard-combat-v1 native batches."""

import gymnasium as gym
import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import StandardCombatEnv, StandardCombatVectorEnv
from .test_standard_combat import catalog, entry, legal_action, maybe_legal_action
from .vector_conformance import assert_vector_conformance


@pytest.mark.parametrize("mode", ["disabled", "next_step", "same_step"])
def test_standard_vector_conformance_and_mode_metadata(mode):
    cat = catalog(entry("a"), entry("b", character="silent"))
    env = StandardCombatVectorEnv(cat, 2, entry_ids=["a", "b"],
                                  autoreset_mode=mode)
    try:
        assert env.metadata["autoreset_mode"] == {
            "disabled": gym.vector.AutoresetMode.DISABLED,
            "next_step": gym.vector.AutoresetMode.NEXT_STEP,
            "same_step": gym.vector.AutoresetMode.SAME_STEP,
        }[mode]
        _, info = assert_vector_conformance(env)
        assert "effective_seed" not in info
        obs, info = env.reset(seed=9)
        assert info["entry_id"].tolist() == ["a", "b"]
        before = {key: value.copy() for key, value in obs.items()}
        mask = np.asarray([True, False], dtype=np.bool_)
        after, _ = env.reset(seed=10, options={"reset_mask": mask})
        for key in after:
            np.testing.assert_array_equal(after[key][1], before[key][1])
    finally:
        env.close(reason="test")


def test_standard_vector_rejects_gym66_catalog_and_exports_recovery_surface():
    gym66 = {"profile_id": "ironclad-combat-v2", "entries": [entry("a")]}
    with pytest.raises(_native.RunUnsupportedBoundaryError):
        StandardCombatVectorEnv(gym66, 1)
    env = StandardCombatVectorEnv(catalog(entry("a")), 1)
    try:
        assert callable(env.recover_committed)
        assert callable(env.diagnostics)
    finally:
        env.close()


@pytest.mark.parametrize("mode", ["disabled", "next_step", "same_step"])
def test_standard_vector_autoreset_rows_match_scalar_and_keep_mixed_entries(mode):
    entries = []
    for entry_id in ("heavy-a", "heavy-b"):
        row = entry(entry_id, deck=["bludgeon"] * 10)
        row["deck"] = [{"card": "bludgeon", "upgrades": 1} for _ in range(10)]
        entries.append(row)
    cat = catalog(*entries)
    entry_ids = ["heavy-a", "heavy-b"]
    vector = StandardCombatVectorEnv(
        cat, 2, entry_ids=entry_ids, partition="test", autoreset_mode=mode,
    )
    singles = [StandardCombatEnv(cat, partition="test") for _ in range(2)]
    try:
        observations, _ = vector.reset(seed=27)
        scalar_observations = [single.reset(seed=27 + slot, options={"entry_id": entry_ids[slot]})[0]
                               for slot, single in enumerate(singles)]
        spec = vector.observation_spec()
        for slot, scalar in enumerate(scalar_observations):
            for key in scalar:
                np.testing.assert_array_equal(observations[key][slot], scalar[key])
        pending_next = np.zeros(2, dtype=np.bool_)
        saw_terminal = False
        for _ in range(30):
            actions = np.zeros(2, dtype=np.int64)
            for slot in range(2):
                if not pending_next[slot]:
                    action = maybe_legal_action(
                        {key: values[slot] for key, values in observations.items()}, spec, "play_card"
                    )
                    if action is None:
                        action = maybe_legal_action(
                            {key: values[slot] for key, values in observations.items()}, spec, "end_turn"
                        )
                    if action is None:
                        action = int(np.flatnonzero(observations["action_mask"][slot].reshape(-1))[0])
                    actions[slot] = action

            scalar_rows = []
            for slot, single in enumerate(singles):
                if pending_next[slot]:
                    scalar_rows.append((single.reset(seed=None, options={"entry_id": entry_ids[slot]})[0],
                                        0.0, False, False, None))
                else:
                    scalar_rows.append(single.step(int(actions[slot])))
            observations, rewards, terminated, truncated, info = vector.step(actions)
            for slot, scalar_row in enumerate(scalar_rows):
                scalar_obs, scalar_reward, scalar_term, scalar_trunc, _ = scalar_row
                assert info["entry_id"][slot] == entry_ids[slot]
                assert rewards[slot] == scalar_reward
                assert bool(terminated[slot]) == scalar_term
                assert bool(truncated[slot]) == scalar_trunc
                if scalar_term and mode == "same_step":
                    final = info["final_obs"][slot]
                    for key in scalar_obs:
                        np.testing.assert_array_equal(final[key], scalar_obs[key])
                    scalar_obs = singles[slot].reset(
                        seed=None, options={"entry_id": entry_ids[slot]}
                    )[0]
                    assert info["_final_obs"][slot]
                elif pending_next[slot]:
                    assert info["autoreset"][slot]
                for key in scalar_obs:
                    np.testing.assert_array_equal(observations[key][slot], scalar_obs[key])
                saw_terminal |= scalar_term

            if mode == "disabled" and terminated.any():
                mask = terminated.astype(np.bool_)
                observations, _ = vector.reset(options={"reset_mask": mask})
                for slot in np.flatnonzero(mask):
                    scalar_observations[slot] = singles[slot].reset(
                        seed=None, options={"entry_id": entry_ids[slot]}
                    )[0]
                    for key in scalar_observations[slot]:
                        np.testing.assert_array_equal(observations[key][slot], scalar_observations[slot][key])
            pending_next = terminated.copy() if mode == "next_step" else np.zeros(2, dtype=np.bool_)
            if saw_terminal and not pending_next.any():
                break
        assert saw_terminal
    finally:
        vector.close()
        for single in singles:
            single.close()
