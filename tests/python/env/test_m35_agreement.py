"""M35 Python vector/scalar agreement and the rebased Gym66 golden contract."""

from __future__ import annotations

import hashlib
import json
import math
import os
import struct
from pathlib import Path

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import (
    IroncladCombatEnv,
    IroncladCombatVectorEnv,
    RunEnv,
    RunVectorEnv,
    StandardCombatEnv,
    StandardCombatVectorEnv,
)


ROOT = Path(__file__).resolve().parents[3]
DIGEST_FIXTURE = ROOT / "crates/spire-env/tests/fixtures/m35_agreement_digests.json"
R1_CHARACTERS = ("ironclad", "silent", "defect", "watcher")
R1_SEEDS = [9_350_000 + 17 * slot for slot in range(8)]
R3_MASK_STEPS = (150, 400)
UINT64_MASK = 2**64 - 1


def _r1_configs(*, record_replay=False):
    return [
        {
            "character": character,
            "ascension": ascension,
            "partition": "test",
            "record_replay": record_replay,
        }
        for character in R1_CHARACTERS
        for ascension in (0, 20)
    ]


def _r1_index(seed, step, mask):
    legal = np.flatnonzero(mask)
    assert legal.size, f"no legal R1 action at step {step}"
    z = (int(seed) + (int(step) + 1) * 0x9E3779B97F4A7C15) & UINT64_MASK
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & UINT64_MASK
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & UINT64_MASK
    z ^= z >> 31
    return int(legal[z % legal.size])


def _json_safe(value):
    if isinstance(value, dict):
        return {str(key): _json_safe(child) for key, child in value.items()}
    if isinstance(value, (list, tuple)):
        return [_json_safe(child) for child in value]
    if isinstance(value, np.ndarray):
        return _json_safe(value.tolist())
    if isinstance(value, np.generic):
        return value.item()
    return value


def _assert_obs_row(batch_obs, slot, single_obs, *, where):
    assert set(batch_obs) == set(single_obs), where
    for key in single_obs:
        np.testing.assert_array_equal(
            batch_obs[key][slot], single_obs[key], err_msg=f"{where}: {key}"
        )


def _assert_obs_equal(left, right, *, where):
    assert set(left) == set(right), where
    for key in left:
        np.testing.assert_array_equal(left[key], right[key], err_msg=f"{where}: {key}")


def _assert_info_row(batch_info, slot, single_info, *, where):
    for key, expected in single_info.items():
        # The batch contract intentionally publishes only the Run schema/terminal fields (and
        # Standard reward fields); scalar RunEnv additionally exposes its creation descriptor.
        if key == "creation":
            continue
        assert key in batch_info, f"{where}: missing info field {key}"
        actual = batch_info[key][slot]
        assert _json_safe(actual) == _json_safe(expected), (
            f"{where}: info[{key}] slot {slot}: {_json_safe(actual)!r} != {_json_safe(expected)!r}"
        )


def _packed_value(info, key, slot):
    value = info[key]
    if isinstance(value, np.ndarray):
        return _json_safe(value[slot])
    return _json_safe(value[slot])


def _single_run(config, profile):
    kwargs = {
        "profile": profile,
        "partition": config.get("partition", "test"),
        "record_replay": config.get("record_replay", False),
    }
    if "boss_history" in config:
        kwargs["boss_history"] = config["boss_history"]
    return RunEnv(config["character"], config["ascension"], **kwargs)


def _assert_combat_views(vector, singles, *, where):
    views = vector.combat_views()
    assert len(views["info"]) == len(singles)
    for slot, env in enumerate(singles):
        single = env.combat_view()
        _assert_obs_row(views["obs"], slot, single["obs"], where=f"{where}, slot={slot}")
        assert bool(views["in_combat"][slot]) == bool(single["info"]["active"]), where
        assert _json_safe(views["info"][slot]) == _json_safe(single["info"]), where


def _group_keys(spec):
    return [
        tensor["key"]
        for tensor in spec["tensors"]
        if tensor["key"] != "action_mask"
        and not tensor["key"].endswith("_known")
        and not tensor["key"].endswith("_occupied")
    ]


def _u32(hasher, value):
    hasher.update(struct.pack("<I", int(value)))


def _text_frame(hasher, value):
    data = ("" if value is None else str(value)).encode("utf-8")
    _u32(hasher, len(data))
    hasher.update(data)


def _i64_array_frame(hasher, value):
    array = np.asarray(value, dtype=np.dtype("<i8"))
    hasher.update(array.tobytes(order="C"))


def _reward_i64(value):
    numeric = float(value)
    assert math.isfinite(numeric) and numeric.is_integer(), f"non-integral reward: {value!r}"
    return int(numeric)


def _digest_row(hasher, observation, slot, spec, reward, terminated, truncated, autoreset, reason, entry=None):
    for key in _group_keys(spec):
        values = np.asarray(observation[key][slot])
        _text_frame(hasher, key)
        _u32(hasher, values.ndim)
        for dimension in values.shape:
            _u32(hasher, dimension)
        _i64_array_frame(hasher, values)
        _i64_array_frame(hasher, observation[f"{key}_known"][slot])
        occupied_key = f"{key}_occupied"
        if occupied_key in observation:
            _i64_array_frame(hasher, observation[occupied_key][slot])
    action_mask = np.asarray(observation["action_mask"][slot])
    _text_frame(hasher, "action_mask")
    _u32(hasher, action_mask.ndim)
    for dimension in action_mask.shape:
        _u32(hasher, dimension)
    _i64_array_frame(hasher, action_mask)
    hasher.update(struct.pack("<q", _reward_i64(reward)))
    hasher.update(bytes((bool(terminated), bool(truncated), bool(autoreset))))
    _text_frame(hasher, reason)
    if entry is not None:
        _text_frame(hasher, entry)


def _run_batch_scalar_agreement(configs, seeds, *, profile, workers, steps, compare_views=False, digest=False):
    vector = RunVectorEnv(
        configs,
        profile=profile,
        workers=workers,
        autoreset_mode="next_step",
        fault_policy="freeze",
    )
    singles = [_single_run(config, profile) for config in configs]
    hashes = [hashlib.sha256() for _ in configs] if digest else None
    try:
        current_obs, current_info = vector.reset(seed=list(seeds))
        single_obs = []
        single_info = []
        for slot, (env, seed) in enumerate(zip(singles, seeds, strict=True)):
            obs, info = env.reset(seed=int(seed))
            single_obs.append(obs)
            single_info.append(info)
            _assert_obs_row(current_obs, slot, obs, where=f"{profile} reset worker={workers} slot={slot}")
            _assert_info_row(current_info, slot, info, where=f"{profile} reset worker={workers}")
            if hashes is not None:
                _digest_row(
                    hashes[slot], current_obs, slot, vector.observation_spec(), 0, False, False, False,
                    info.get("terminal_reason"),
                )
        if compare_views:
            _assert_combat_views(vector, singles, where=f"{profile} initial combat view worker={workers}")

        pending = [False] * len(configs)
        for step in range(1, steps + 1):
            actions = np.asarray(
                [
                    -1 if pending[slot] else _r1_index(seeds[slot], step, current_obs["action_mask"][slot])
                    for slot in range(len(configs))
                ],
                dtype=np.int64,
            )
            was_pending = pending.copy()
            obs, rewards, terminated, truncated, info = vector.step(actions)
            next_pending = [False] * len(configs)
            for slot, env in enumerate(singles):
                if was_pending[slot]:
                    expected_obs, expected_info = env.reset()
                    expected_reward, expected_terminated, expected_truncated = 0.0, False, False
                else:
                    expected_obs, expected_reward, expected_terminated, expected_truncated, expected_info = env.step(
                        int(actions[slot])
                    )
                    next_pending[slot] = bool(expected_terminated)
                _assert_obs_row(obs, slot, expected_obs, where=f"{profile} worker={workers} slot={slot} step={step}")
                _assert_info_row(info, slot, expected_info, where=f"{profile} worker={workers} slot={slot} step={step}")
                assert float(rewards[slot]) == float(expected_reward), f"{profile} reward worker={workers} slot={slot} step={step}"
                assert bool(terminated[slot]) == bool(expected_terminated), f"{profile} terminated worker={workers} slot={slot} step={step}"
                assert bool(truncated[slot]) == bool(expected_truncated), f"{profile} truncated worker={workers} slot={slot} step={step}"
                assert bool(info["autoreset"][slot]) == was_pending[slot], f"{profile} autoreset worker={workers} slot={slot} step={step}"
                single_obs[slot], single_info[slot] = expected_obs, expected_info
                if hashes is not None and step < 200:
                    _digest_row(
                        hashes[slot], obs, slot, vector.observation_spec(), rewards[slot], terminated[slot],
                        truncated[slot], info["autoreset"][slot], _packed_value(info, "terminal_reason", slot),
                    )
            pending = next_pending
            current_obs, current_info = obs, info
            if compare_views:
                _assert_combat_views(vector, singles, where=f"{profile} combat view worker={workers} step={step}")
        return [hasher.hexdigest() for hasher in hashes] if hashes is not None else None
    finally:
        vector.close()
        for env in singles:
            env.close()


def test_r1_run_vector_matches_scalar_combat_views_and_r7_digest():
    fixture = json.loads(DIGEST_FIXTURE.read_text())
    assert fixture["schema"] == "m35-agreement-digests-v1" and fixture["steps"] == 200
    for workers in (1, 4):
        digests = _run_batch_scalar_agreement(
            _r1_configs(record_replay=True),
            R1_SEEDS,
            profile="standard",
            workers=workers,
            steps=200,
            compare_views=True,
            digest=True,
        )
        assert digests == fixture["corpora"]["r1"], f"R1 facade digest mismatch at workers={workers}"


def _record_into_watcher_combat():
    env = RunEnv("watcher", 20, profile="standard", partition="test", record_replay=True)
    try:
        observation, _ = env.reset(seed=4242)
        steps = 0
        while not env.combat_view()["info"]["active"]:
            action = int(np.flatnonzero(observation["action_mask"])[0])
            observation, _, terminated, truncated, _ = env.step(action)
            assert not terminated and not truncated, "Watcher A20 route ended before its first combat"
            steps += 1
            assert steps < 400, "Watcher A20 route did not reach combat"
        record = env.record()
        assert record is not None
        return record, steps
    finally:
        env.close()


def _standard_entry(entry_id, character, ascension, encounter):
    starter = {
        "ironclad": ("burning_blood", ["strike"] * 5 + ["defend"] * 4 + ["bash"]),
        "silent": ("ring_of_the_snake", ["strike_g"] * 5 + ["defend_g"] * 5 + ["neutralize", "survivor"]),
        "defect": ("cracked_core", ["strike_b"] * 4 + ["defend_b"] * 4 + ["zap", "dualcast"]),
        "watcher": ("pure_water", ["strike_p"] * 4 + ["defend_p"] * 4 + ["eruption", "vigilance"]),
    }
    relic, cards = starter[character]
    return {
        "id": entry_id,
        "character": character,
        "ascension": ascension,
        "encounter": encounter,
        "hp": 70,
        "max_hp": 80,
        "gold": 99,
        "deck": [{"card": card, "upgrades": 0} for card in cards],
        "relics": [relic],
        "potions": [None] * (2 if ascension >= 11 else 3),
    }


def _r4_catalog():
    record, steps = _record_into_watcher_combat()
    entries, ids = [], []
    encounters = ("cultist", "jaw_worm")
    for char_index, character in enumerate(R1_CHARACTERS):
        for asc_index, ascension in enumerate((0, 20)):
            entry_id = f"{character}_a{ascension}"
            entry = _standard_entry(entry_id, character, ascension, encounters[(char_index + asc_index) % 2])
            if character == "defect" and ascension == 0:
                entry["position"] = {
                    "player": {"powers": {"focus": 1}, "orb_slots": 3, "orbs": ["frost", "lightning"]}
                }
            if character == "watcher" and ascension == 20:
                entry = {
                    "id": entry_id,
                    "recipe": {"record_sha256": hashlib.sha256(record).hexdigest(), "steps": steps},
                }
            entries.append(entry)
            ids.append(entry_id)
    catalog = {
        "catalog_schema_id": "spire-gym-standard-combat-catalog-v1",
        "profile_id": "standard-combat-v1",
        "entries": entries,
    }
    return catalog, [record], ids


def _final_obs_at(info, slot):
    values = info["final_obs"]
    return values[slot]


def _assert_final_info_row(info, slot, expected_info, *, where):
    _assert_info_row(info["final_info"], slot, expected_info, where=where)


def test_r4_standard_combat_vector_matches_scalar_and_r7_digest():
    catalog, records, ids = _r4_catalog()
    seeds = [7100 + slot for slot in range(8)]
    fixture = json.loads(DIGEST_FIXTURE.read_text())
    digests_by_workers = {}
    for workers in (1, 4):
        for mode in ("next_step", "same_step"):
            vector = StandardCombatVectorEnv(
                catalog,
                8,
                records=records,
                partition="test",
                entry_ids=ids,
                workers=workers,
                autoreset_mode=mode,
                fault_policy="freeze",
            )
            singles = [StandardCombatEnv(catalog, records=records, partition="test") for _ in ids]
            try:
                obs, info = vector.reset(seed=seeds, options={"entry_ids": ids})
                single_obs, single_info = [], []
                start_kinds = []
                for slot, env in enumerate(singles):
                    one_obs, one_info = env.reset(seed=seeds[slot], options={"entry_id": ids[slot]})
                    single_obs.append(one_obs)
                    single_info.append(one_info)
                    start_kinds.append(one_info["start_kind"])
                    _assert_obs_row(obs, slot, one_obs, where=f"R4 reset workers={workers} mode={mode} slot={slot}")
                    _assert_info_row(info, slot, one_info, where=f"R4 reset workers={workers} mode={mode}")
                    assert info["entry_id"][slot] == ids[slot]
                assert {"fresh_start", "synthetic_position", "reachable_recipe"}.issubset(set(start_kinds))

                hashes = [hashlib.sha256() for _ in ids] if mode == "next_step" else None
                if hashes is not None:
                    for slot in range(8):
                        _digest_row(
                            hashes[slot], obs, slot, vector.observation_spec(), 0, False, False, False,
                            single_info[slot].get("terminal_reason"), ids[slot],
                        )
                pending = [False] * 8
                episodes = [0] * 8
                current_obs = obs
                for step in range(1, 451):
                    actions = np.asarray(
                        [
                            -1 if pending[slot] else _r1_index(seeds[slot], step, current_obs["action_mask"][slot])
                            for slot in range(8)
                        ],
                        dtype=np.int64,
                    )
                    was_pending = pending.copy()
                    obs, rewards, terminated, truncated, info = vector.step(actions)
                    next_pending = [False] * 8
                    for slot, env in enumerate(singles):
                        if was_pending[slot]:
                            expected_obs, expected_info = env.reset(
                                seed=None, options={"entry_id": ids[slot]}
                            )
                            expected_reward, expected_terminated, expected_truncated = 0.0, False, False
                        else:
                            terminal_obs, expected_reward, expected_terminated, expected_truncated, terminal_info = env.step(
                                int(actions[slot])
                            )
                            expected_obs, expected_info = terminal_obs, terminal_info
                            if expected_terminated:
                                episodes[slot] += 1
                                if mode == "same_step":
                                    _assert_obs_equal(
                                        _final_obs_at(info, slot), terminal_obs,
                                        where=f"R4 final observation workers={workers} slot={slot} step={step}",
                                    )
                                    _assert_final_info_row(
                                        info, slot, terminal_info,
                                        where=f"R4 final info workers={workers} slot={slot} step={step}",
                                    )
                                    expected_obs, expected_info = env.reset(
                                        seed=None, options={"entry_id": ids[slot]}
                                    )
                                elif mode == "next_step":
                                    next_pending[slot] = True
                        _assert_obs_row(
                            obs, slot, expected_obs,
                            where=f"R4 workers={workers} mode={mode} slot={slot} step={step}",
                        )
                        _assert_info_row(
                            info, slot, expected_info,
                            where=f"R4 info workers={workers} mode={mode} slot={slot} step={step}",
                        )
                        assert float(rewards[slot]) == float(expected_reward)
                        assert bool(terminated[slot]) == bool(expected_terminated)
                        assert bool(truncated[slot]) == bool(expected_truncated)
                        assert bool(info["autoreset"][slot]) == was_pending[slot]
                        assert info["entry_id"][slot] == ids[slot]
                        assert info["start_kind"][slot] == start_kinds[slot]
                        single_obs[slot], single_info[slot] = expected_obs, expected_info
                        if hashes is not None and step < 200:
                            _digest_row(
                                hashes[slot], obs, slot, vector.observation_spec(), rewards[slot],
                                terminated[slot], truncated[slot], info["autoreset"][slot],
                                _packed_value(info, "terminal_reason", slot), ids[slot],
                            )
                    pending = next_pending
                    current_obs = obs
                assert all(count >= 3 for count in episodes), f"R4 episodes workers={workers} mode={mode}: {episodes}"
                if hashes is not None:
                    digests = [hasher.hexdigest() for hasher in hashes]
                    assert digests == fixture["corpora"]["r4"], f"R4 facade digest mismatch at workers={workers}"
                    digests_by_workers[workers] = digests
            finally:
                vector.close()
                for env in singles:
                    env.close()
    assert digests_by_workers[1] == digests_by_workers[4]


def _r3_pending_reset_point():
    """Find the first deterministic R1 terminal and freeze its following reset step."""
    singles = [_single_run(config, "standard") for config in _r1_configs()]
    try:
        observations = [env.reset(seed=seed)[0] for env, seed in zip(singles, R1_SEEDS, strict=True)]
        for step in range(1, 601):
            for slot, env in enumerate(singles):
                action = _r1_index(R1_SEEDS[slot], step, observations[slot]["action_mask"])
                observations[slot], _, terminated, truncated, _ = env.step(action)
                if terminated or truncated:
                    assert terminated, "R3 Standard Run corpus must not truncate"
                    return step + 1, slot
        raise AssertionError("R1 corpus produced no terminal to schedule a pending reset")
    finally:
        for env in singles:
            env.close()


def _r3_run_mode(mode, pending_reset_point):
    configs = _r1_configs(record_replay=True)
    seeds = R1_SEEDS
    vector = RunVectorEnv(
        configs, workers=4, autoreset_mode=mode, fault_policy="freeze"
    )
    singles = [_single_run(config, "standard") for config in configs]
    expected_records = [[] for _ in configs]
    try:
        current_obs, info = vector.reset(seed=list(seeds))
        single_obs, single_info = [], []
        for slot, (env, seed) in enumerate(zip(singles, seeds, strict=True)):
            obs, one_info = env.reset(seed=seed)
            single_obs.append(obs)
            single_info.append(one_info)
            _assert_obs_row(current_obs, slot, obs, where=f"R3 {mode} initial slot={slot}")
            _assert_info_row(info, slot, one_info, where=f"R3 {mode} initial")

        pending = [False] * len(configs)
        terminal_now = [False] * len(configs)
        masked_pending_seen = False
        terminal_count = 0
        pending_reset_step, pending_reset_slot = pending_reset_point
        for step in range(1, 601):
            mask = np.zeros(len(configs), dtype=np.bool_)
            if step in R3_MASK_STEPS:
                mask[1::2] = True
            if mode == "next_step" and step == pending_reset_step:
                assert pending[pending_reset_slot], (
                    f"slot {pending_reset_slot} must be pending at fixed reset step {step}"
                )
                mask[pending_reset_slot] = True
                masked_pending_seen = True
            if mode == "disabled":
                mask |= np.asarray(terminal_now, dtype=np.bool_)
            if mask.any():
                obs, info = vector.reset(seed=None, options={"reset_mask": mask})
                for slot, env in enumerate(singles):
                    if mask[slot]:
                        record = env.record()
                        if record is not None:
                            expected_records[slot].append(record)
                        one_obs, one_info = env.reset()
                        pending[slot] = False
                        terminal_now[slot] = False
                    else:
                        # Native masked reset returns an unselected slot's fresh decision boundary,
                        # whose info omits step-only reward_events.
                        one_obs, one_info = env.decision()
                    single_obs[slot], single_info[slot] = one_obs, one_info
                    _assert_obs_row(obs, slot, single_obs[slot], where=f"R3 {mode} masked reset step={step} slot={slot}")
                    _assert_info_row(info, slot, single_info[slot], where=f"R3 {mode} masked reset step={step}")
                current_obs = obs
                continue

            actions = np.asarray(
                [
                    -1 if pending[slot] else _r1_index(seeds[slot], step, current_obs["action_mask"][slot])
                    for slot in range(len(configs))
                ],
                dtype=np.int64,
            )
            was_pending = pending.copy()
            obs, rewards, terminated, truncated, info = vector.step(actions)
            next_pending = [False] * len(configs)
            next_terminal_now = [False] * len(configs)
            for slot, env in enumerate(singles):
                if was_pending[slot]:
                    record = env.record()
                    if record is not None:
                        expected_records[slot].append(record)
                    expected_obs, expected_info = env.reset()
                    expected_reward, expected_terminated, expected_truncated = 0.0, False, False
                else:
                    terminal_obs, expected_reward, expected_terminated, expected_truncated, terminal_info = env.step(
                        int(actions[slot])
                    )
                    expected_obs, expected_info = terminal_obs, terminal_info
                    terminal_count += int(bool(expected_terminated))
                    if expected_terminated and mode == "same_step":
                        record = env.record()
                        if record is not None:
                            expected_records[slot].append(record)
                        expected_obs, expected_info = env.reset()
                    elif expected_terminated and mode == "next_step":
                        next_pending[slot] = True
                    elif expected_terminated and mode == "disabled":
                        next_terminal_now[slot] = True
                _assert_obs_row(obs, slot, expected_obs, where=f"R3 {mode} slot={slot} step={step}")
                if not (mode == "same_step" and bool(expected_terminated) and not was_pending[slot]):
                    _assert_info_row(info, slot, expected_info, where=f"R3 {mode} info slot={slot} step={step}")
                else:
                    combined_info = dict(expected_info)
                    if "reward_events" in info and "reward_events" in terminal_info:
                        combined_info["reward_events"] = terminal_info["reward_events"]
                    _assert_info_row(info, slot, combined_info, where=f"R3 {mode} same-step info slot={slot} step={step}")
                    _assert_obs_equal(
                        info["final_obs"][slot], terminal_obs,
                        where=f"R3 {mode} final observation slot={slot} step={step}",
                    )
                    _assert_final_info_row(
                        info, slot, terminal_info,
                        where=f"R3 {mode} final info slot={slot} step={step}",
                    )
                assert float(rewards[slot]) == float(expected_reward)
                assert bool(terminated[slot]) == bool(expected_terminated)
                assert bool(truncated[slot]) == bool(expected_truncated)
                if mode == "disabled":
                    assert "autoreset" not in info
                else:
                    assert bool(info["autoreset"][slot]) == was_pending[slot]
                single_obs[slot], single_info[slot] = expected_obs, expected_info
            pending = next_pending
            terminal_now = next_terminal_now
            current_obs = obs

        if mode == "next_step":
            assert masked_pending_seen, "a fixed-step masked reset must clear a pending NEXT_STEP autoreset"
        assert terminal_count > 0, f"R3 {mode} must exercise terminal/autoreset transitions"
        actual_records = [[] for _ in configs]
        for slot, record in vector.take_completed_records():
            actual_records[slot].append(record)
        assert actual_records == expected_records, f"R3 {mode} completed records differ from scalar resets"
    finally:
        vector.close()
        for env in singles:
            env.close()


def test_r3_autoreset_modes_and_masked_pending_reset_match_scalar():
    pending_reset_point = _r3_pending_reset_point()
    for mode in ("disabled", "next_step", "same_step"):
        _r3_run_mode(mode, pending_reset_point)


def _gym66_entry(entry_id, *, encounter="cultist", hp=80, deck=None):
    return {
        "id": entry_id,
        "encounter": encounter,
        "hp": hp,
        "max_hp": max(hp, 80),
        "deck": deck or [{"card": "strike", "upgrades": 0}] * 10,
        "relics": [],
        "potions": [None, None, None],
        "relic_counters": {},
    }


def _gym66_catalog(v2):
    entries = [
        _gym66_entry("cultist"),
        _gym66_entry("jaw", encounter="jaw_worm", hp=54),
        _gym66_entry("low", hp=1, deck=[{"card": "defend", "upgrades": 0}]),
    ]
    catalog = {"entries": entries}
    if v2:
        catalog["profile_id"] = "ironclad-combat-v2"
    return catalog


def test_r5_ironclad_vector_matches_scalar_v1_v2():
    for v2 in (False, True):
        catalog = _gym66_catalog(v2)
        ids = ["cultist", "jaw", "low", "cultist", "jaw", "low"]
        seeds = [8300 + slot * 19 for slot in range(6)]
        for workers in (1, 4):
            vector = IroncladCombatVectorEnv(
                6, catalog, partition="test", workers=workers,
                autoreset_mode="disabled", fault_policy="freeze",
            )
            singles = [IroncladCombatEnv(catalog, partition="test") for _ in ids]
            try:
                obs, info = vector.reset(
                    seed=seeds,
                    options={"entry_ids": ids},
                )
                single_obs, single_info = [], []
                for slot, env in enumerate(singles):
                    one_obs, one_info = env.reset(seed=seeds[slot], options={"entry_id": ids[slot]})
                    single_obs.append(one_obs)
                    single_info.append(one_info)
                    _assert_obs_row(obs, slot, one_obs, where=f"R5 v2={v2} worker={workers} reset slot={slot}")
                    assert info["slots"][slot] == one_info
                terminal_resets = 0
                last_done = np.zeros(6, dtype=np.bool_)
                for step in range(1, 301):
                    if last_done.any():
                        terminal_resets += int(last_done.sum())
                        obs, info = vector.reset(
                            seed=None,
                            options={"reset_mask": last_done, "entry_ids": ids},
                        )
                        for slot, env in enumerate(singles):
                            if last_done[slot]:
                                one_obs, one_info = env.reset(seed=None, options={"entry_id": ids[slot]})
                                single_obs[slot], single_info[slot] = one_obs, one_info
                            _assert_obs_row(obs, slot, single_obs[slot], where=f"R5 v2={v2} worker={workers} masked slot={slot}")
                            assert info["slots"][slot] == single_info[slot]
                    actions = np.asarray(
                        [
                            _r1_index(seeds[slot], step, obs["action_mask"][slot])
                            for slot in range(6)
                        ],
                        dtype=np.int64,
                    )
                    obs, rewards, terminated, truncated, info = vector.step(actions)
                    next_done = np.asarray(terminated, dtype=np.bool_) | np.asarray(truncated, dtype=np.bool_)
                    for slot, env in enumerate(singles):
                        one_obs, one_reward, one_terminated, one_truncated, one_info = env.step(int(actions[slot]))
                        _assert_obs_row(obs, slot, one_obs, where=f"R5 v2={v2} worker={workers} slot={slot} step={step}")
                        assert float(rewards[slot]) == float(one_reward)
                        assert bool(terminated[slot]) == bool(one_terminated)
                        assert bool(truncated[slot]) == bool(one_truncated)
                        assert info["slots"][slot] == one_info
                        assert info["slots"][slot]["entry_id"] == ids[slot]
                        single_obs[slot], single_info[slot] = one_obs, one_info
                    last_done = next_done
                assert terminal_resets > 0, f"R5 v2={v2} worker={workers} did not exercise masked terminal resets"
            finally:
                vector.close()
                for env in singles:
                    env.close()


def _r6_configs():
    histories = ("awakened-unseen", "time-eater-unseen", "donu-deca-unseen", "awakened-unseen")
    return [
        {
            "character": "ironclad",
            "ascension": 0,
            "partition": "test",
            "boss_history": history,
        }
        for history in histories
    ]


def test_r6_m9_m10_run_vector_smoke_matches_scalar():
    seeds = [31, 32, 33, 34]
    configs = _r6_configs()
    for profile in ("m9-ironclad-act3", "m10-ironclad-heart", "m10-ironclad-policy"):
        _run_batch_scalar_agreement(
            configs,
            seeds,
            profile=profile,
            workers=4,
            steps=100,
        )


def _assert_golden_diag(actual, expected, *, where):
    assert "slots" in actual, f"{where}: expected the disclosed slots diagnostic"
    assert set(actual) == set(expected) | {"slots"}, f"{where}: unexpected diagnostic keys"
    assert len(actual["slots"]) == 6, f"{where}: expected six slot delivery rows"
    without_slots = {key: value for key, value in actual.items() if key != "slots"}
    assert _json_safe(without_slots) == _json_safe(expected), where


def _exception_record(error):
    error_type = type(error)
    return {"type": f"{error_type.__module__}.{error_type.__qualname__}", "message": str(error)}


def _compare_golden_row(archive, catalog_name, row_index, row_meta, call, obs, info, *, rewards=None, terminated=None, truncated=None):
    expected = row_meta["rows"][row_index]
    assert expected["call"] == call, f"{catalog_name} row {row_index} call metadata"
    assert set(info) == {"slots"}, f"{catalog_name} row {row_index} added top-level info keys"
    assert info["slots"] == expected["info"]["slots"], f"{catalog_name} row {row_index} info slots"
    for key, values in obs.items():
        golden_key = f"c{catalog_name}/r{row_index:03d}/obs/{key}"
        assert golden_key in archive, f"missing golden array {golden_key}"
        np.testing.assert_array_equal(values, archive[golden_key], err_msg=f"{golden_key}")
    if rewards is not None:
        np.testing.assert_array_equal(
            rewards, archive[f"c{catalog_name}/r{row_index:03d}/rewards"],
            err_msg=f"{catalog_name} row {row_index} rewards",
        )
        np.testing.assert_array_equal(
            terminated, archive[f"c{catalog_name}/r{row_index:03d}/terminated"],
            err_msg=f"{catalog_name} row {row_index} terminated",
        )
        np.testing.assert_array_equal(
            truncated, archive[f"c{catalog_name}/r{row_index:03d}/truncated"],
            err_msg=f"{catalog_name} row {row_index} truncated",
        )


def _replay_gym66_golden_catalog(catalog_name, catalog, metadata, archive):
    definition = metadata["catalogs"][catalog_name]
    ids = definition["entry_ids"]
    entry_ids = [ids["e1"], ids["e2"], ids["e0"], ids["e1"], ids["e2"], ids["e0"]]
    vector = IroncladCombatVectorEnv(6, catalog)
    singles = [IroncladCombatEnv(catalog) for _ in range(6)]
    try:
        obs, info = vector.reset(seed=[0, 1, 2, 3, 4, 5], options={"entry_ids": entry_ids})
        single_obs, single_info = [], []
        for slot, env in enumerate(singles):
            one_obs, one_info = env.reset(seed=slot, options={"entry_id": entry_ids[slot]})
            single_obs.append(one_obs)
            single_info.append(one_info)
            _assert_obs_row(obs, slot, one_obs, where=f"golden {catalog_name} initial scalar slot={slot}")
            assert info["slots"][slot] == one_info
        row_index = 0
        _compare_golden_row(
            archive, catalog_name, row_index, definition,
            {"kind": "reset", "tag": "initial_step_0", "seed": [0, 1, 2, 3, 4, 5],
             "options": {"entry_ids": entry_ids}},
            obs, info,
        )
        _assert_golden_diag(vector.diagnostic(), definition["diagnostic_snapshots"]["after_step_0"], where=f"{catalog_name} diagnostic step 0")
        row_index += 1

        for step in range(1, 301):
            if step in (101, 201):
                if step == 101:
                    mask = np.asarray([True, False, False, True, False, False], dtype=np.bool_)
                    selected_ids = [None, None, None, ids["e2"], None, None]
                else:
                    mask = np.asarray([False, True, False, False, True, False], dtype=np.bool_)
                    selected_ids = [None, ids["e0"], None, None, None, None]
                options = {"reset_mask": mask, "entry_ids": selected_ids}
                obs, info = vector.reset(seed=None, options=options)
                for slot, env in enumerate(singles):
                    if mask[slot]:
                        entry_id = selected_ids[slot]
                        kwargs = {"options": {"entry_id": entry_id}} if entry_id is not None else {}
                        one_obs, one_info = env.reset(seed=None, **kwargs)
                        single_obs[slot], single_info[slot] = one_obs, one_info
                    _assert_obs_row(obs, slot, single_obs[slot], where=f"golden {catalog_name} masked reset {step} slot={slot}")
                    assert info["slots"][slot] == single_info[slot]
                _compare_golden_row(
                    archive, catalog_name, row_index, definition,
                    {"kind": "reset", "tag": f"masked_reset_before_step={step}", "seed": None,
                     "options": {"reset_mask": mask.tolist(), "entry_ids": selected_ids}},
                    obs, info,
                )
                row_index += 1

            actions = np.asarray(
                [_r1_index(slot, step, obs["action_mask"][slot]) for slot in range(6)],
                dtype=np.int64,
            )
            obs, rewards, terminated, truncated, info = vector.step(actions)
            for slot, env in enumerate(singles):
                one_obs, one_reward, one_terminated, one_truncated, one_info = env.step(int(actions[slot]))
                _assert_obs_row(obs, slot, one_obs, where=f"golden {catalog_name} step={step} slot={slot}")
                assert float(rewards[slot]) == float(one_reward)
                assert bool(terminated[slot]) == bool(one_terminated)
                assert bool(truncated[slot]) == bool(one_truncated)
                assert info["slots"][slot] == one_info
                single_obs[slot], single_info[slot] = one_obs, one_info
            _compare_golden_row(
                archive, catalog_name, row_index, definition,
                {"kind": "step", "tag": f"step={step}", "actions": actions.tolist()},
                obs, info, rewards=rewards, terminated=terminated, truncated=truncated,
            )
            row_index += 1

            done = np.asarray(terminated, dtype=np.bool_) | np.asarray(truncated, dtype=np.bool_)
            if done.any():
                obs, info = vector.reset(seed=None, options={"reset_mask": done.copy()})
                for slot, env in enumerate(singles):
                    if done[slot]:
                        one_obs, one_info = env.reset(seed=None)
                        single_obs[slot], single_info[slot] = one_obs, one_info
                    _assert_obs_row(obs, slot, single_obs[slot], where=f"golden {catalog_name} terminal reset step={step} slot={slot}")
                    assert info["slots"][slot] == single_info[slot]
                _compare_golden_row(
                    archive, catalog_name, row_index, definition,
                    {"kind": "reset", "tag": f"auto_reset_after_step={step}", "seed": None,
                     "options": {"reset_mask": done.tolist()}},
                    obs, info,
                )
                row_index += 1
            if step in (150, 300):
                key = f"after_step_{step}"
                _assert_golden_diag(vector.diagnostic(), definition["diagnostic_snapshots"][key], where=f"{catalog_name} diagnostic step {step}")

        assert row_index == definition["row_count"], f"{catalog_name}: compared {row_index} of {definition['row_count']} rows"
        faults = definition["fault_runs"]
        assert _json_safe(faults["standard_catalog"]) == _json_safe(_standard_catalog_for_golden())
        try:
            candidate = IroncladCombatVectorEnv(6, faults["standard_catalog"])
        except Exception as error:
            standard_attempt = {"exists": False, **_exception_record(error)}
        else:
            standard_attempt = {"exists": True, "unexpected_constructed": True}
            candidate.close()
        assert standard_attempt == faults["standard_constructor"], f"{catalog_name} standard catalog rejection"

        closed = IroncladCombatVectorEnv(6, catalog)
        try:
            closed_obs, _ = closed.reset(seed=0)
            close_actions = np.asarray(
                [int(np.flatnonzero(closed_obs["action_mask"][slot])[0]) for slot in range(6)],
                dtype=np.int64,
            )
            closed.close()
            try:
                diagnostic_after_close = _json_safe(closed.diagnostic())
            except Exception as error:
                diagnostic_after_close = {"exception": _exception_record(error)}
            try:
                closed.step(close_actions)
            except Exception as error:
                step_after_close = _exception_record(error)
            else:
                step_after_close = {"unexpected_success": True}
            assert step_after_close == faults["closed_env_step"], f"{catalog_name} closed-env step error"
            _assert_golden_diag(
                diagnostic_after_close,
                faults["diagnostic_after_close"],
                where=f"{catalog_name} diagnostic after close",
            )
        finally:
            closed.close()
    finally:
        vector.close()
        for env in singles:
            env.close()


def _standard_catalog_for_golden():
    cards = ["strike"] * 5 + ["defend"] * 4 + ["bash"]
    return {
        "catalog_schema_id": "spire-gym-standard-combat-catalog-v1",
        "profile_id": "standard-combat-v1",
        "entries": [{
            "id": "base", "character": "ironclad", "ascension": 0,
            "encounter": "cultist", "hp": 70, "max_hp": 80, "gold": 99,
            "deck": [{"card": card, "upgrades": 0} for card in cards],
            "relics": ["burning_blood"], "potions": [None, None, None],
        }],
    }


def test_gym66_task0_golden_comparison():
    golden_value = os.environ.get("SPIRE_M35_GYM66_GOLDEN")
    if not golden_value:
        pytest.skip("SPIRE_M35_GYM66_GOLDEN is unset; task-0 Gym66 golden comparison not requested")
    golden = Path(golden_value)
    sums = golden / "SHA256SUMS"
    for line in sums.read_text().splitlines():
        digest, filename = line.split(maxsplit=1)
        path = golden / filename.strip()
        assert hashlib.sha256(path.read_bytes()).hexdigest() == digest, f"SHA256SUMS mismatch: {path.name}"
    metadata = json.loads((golden / "gym66-golden-v1.json").read_text())
    with np.load(golden / "gym66-golden-v1.npz", allow_pickle=False) as archive:
        for catalog_name, v2 in (("v1", False), ("v2", True)):
            catalog = _gym66_catalog(v2)
            assert _json_safe(catalog) == metadata["catalogs"][catalog_name]["catalog"]
            _replay_gym66_golden_catalog(catalog_name, catalog, metadata, archive)
