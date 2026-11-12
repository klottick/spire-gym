"""Python contract for the opt-in compact Standard Run observation."""

import random
import gc

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import RunEnv, RunVectorEnv


FINGERPRINT = "633b9424be5070a2890b01725949447badba919c5c8c5bd798922f8c2d2a2b2d"
CHARS = ("ironclad", "silent", "defect", "watcher")


def _assert_observation_contract(env, obs):
    spec = env.observation_spec()
    assert spec["schema_id"] == "spire-gym-standard-run-observation-compact-v1"
    assert spec["fingerprint"] == FINGERPRINT
    tensors = {row["key"]: row for row in spec["tensors"]}
    assert set(obs) == set(tensors)
    for key, value in obs.items():
        tensor = tensors[key]
        assert value.shape == tuple(tensor["shape"])
        assert value.dtype == np.dtype(tensor["dtype"])
        assert value.flags.c_contiguous and value.dtype.isnative
    space = env.single_observation_space if hasattr(env, "single_observation_space") else env.observation_space
    assert space.contains(obs)


def _widen(obs, specs):
    dense = {}
    for key, value in obs.items():
        shape = tuple(specs[key]["shape"])
        target = np.zeros(shape, dtype=np.int64 if key not in ("action_mask",) and not key.endswith(("_known", "_occupied")) else np.int8)
        slices = tuple(slice(0, n) for n in value.shape)
        target[slices] = value
        dense[key] = target
    return dense


@pytest.mark.parametrize("character", CHARS)
def test_compact_scalar_dtypes_contains_and_widens_to_dense(character):
    compact = RunEnv(character, 0, profile="standard", partition="test", observation="compact")
    dense = RunEnv(character, 0, profile="standard", partition="test")
    try:
        compact_spec = {row["key"]: row for row in compact.observation_spec()["tensors"]}
        dense_spec = {row["key"]: row for row in dense.observation_spec()["tensors"]}
        rng = random.Random(913 + CHARS.index(character))
        compact_obs, _ = compact.reset(seed=913)
        dense_obs, _ = dense.reset(seed=913)
        for _ in range(300):
            _assert_observation_contract(compact, compact_obs)
            widened = _widen(compact_obs, dense_spec)
            for key in dense_obs:
                np.testing.assert_array_equal(widened[key], dense_obs[key])
            legal = np.flatnonzero(compact_obs["action_mask"])
            if not len(legal):
                break
            action = int(rng.choice(legal))
            compact_obs, _, compact_term, _, _ = compact.step(action)
            dense_obs, _, dense_term, _, _ = dense.step(action)
            assert compact_term == dense_term
            if compact_term:
                break
    finally:
        compact.close()
        dense.close()


@pytest.mark.parametrize("bad", ["other", "Compact", ""])
def test_compact_rejects_unknown_observation_and_captured_profile(bad):
    with pytest.raises(_native.RunInputError, match="observation must be 'dense' or 'compact'"):
        _native.NativeRunSession("ironclad", 0, profile="standard", observation=bad)
    with pytest.raises(_native.RunInputError, match="observation must be 'dense' or 'compact'"):
        _native.NativeRunBatch([{"character": "ironclad", "ascension": 0}], observation=bad)
    with pytest.raises(_native.RunInputError, match="requires the standard profile"):
        _native.NativeRunSession("ironclad", 0, profile="m10-ironclad-heart", observation="compact")
    with pytest.raises(_native.RunInputError, match="requires the standard profile"):
        _native.NativeRunBatch(
            [{"character": "ironclad", "ascension": 0}],
            profile="m10-ironclad-heart", observation="compact",
        )


def test_compact_capacity_bounds_and_preset_identity():
    env = RunEnv("ironclad", 0, profile="standard", observation="compact", partition="test")
    try:
        assert env.environment_spec()["capacity_preset"] == "standard-compact-v1"
        custom = RunEnv(
            "ironclad", 0, profile="standard", observation="compact", partition="test",
            capacities={"powers": 64},
        )
        try:
            assert custom.observation_spec()["capacities"]["powers"] == 64
            assert custom.environment_spec()["capacity_preset"] == "custom"
        finally:
            custom.close()
        with pytest.raises(_native.RunInputError):
            _native.NativeRunSession("ironclad", 0, profile="standard", observation="compact", capacities={"powers": 2049})
        with pytest.raises(_native.RunInputError):
            _native.NativeRunSession("ironclad", 0, profile="standard", observation="compact", capacities={"powers": 0})
    finally:
        env.close()


@pytest.mark.parametrize("workers", [1, 4])
@pytest.mark.parametrize("mode", ["disabled", "next_step", "same_step"])
def test_compact_vector_matches_scalar_and_masked_reset(workers, mode):
    configs = [{"character": c, "ascension": 0, "partition": "test"} for c in CHARS]
    env = RunVectorEnv(configs, workers=workers, autoreset_mode=mode, observation="compact")
    scalars = [RunEnv(c, 0, profile="standard", partition="test", observation="compact") for c in CHARS]
    try:
        obs, _ = env.reset(seed=431)
        compact_fingerprint = env.observation_spec()["fingerprint"]
        scalar_obs = [game.reset(seed=431 + i)[0] for i, game in enumerate(scalars)]
        for slot in range(len(configs)):
            for key in obs:
                np.testing.assert_array_equal(obs[key][slot], scalar_obs[slot][key])
        # Review m3: many steps on the typed slab path; each slot is compared with its scalar session until it first
        # terminates (autoreset seeding differs from a scalar reset after that).
        rng = random.Random(431)
        live = [True] * len(configs)
        dtypes = {t["key"]: np.dtype(t["dtype"]) for t in env.observation_spec()["tensors"]}
        for _ in range(100):
            actions = np.asarray([rng.choice(list(np.flatnonzero(row))) if row.any() else 0 for row in obs["action_mask"]], dtype=np.int64)
            obs, _, terminated, _, info = env.step(actions)
            for slot, game in enumerate(scalars):
                if not live[slot]:
                    continue
                scalar_obs[slot], _, done, _, _ = game.step(int(actions[slot]))
                if done:
                    live[slot] = False
                    if mode == "same_step":
                        final = info["final_obs"][slot]
                        for key in final:
                            assert final[key].dtype == dtypes[key]
                            np.testing.assert_array_equal(final[key], scalar_obs[slot][key])
                    continue
                for key in obs:
                    np.testing.assert_array_equal(obs[key][slot], scalar_obs[slot][key])
            if mode == "disabled" and terminated.any():
                break
            assert all(obs[key].dtype == dtypes[key] for key in obs)
        assert all(value.dtype == np.dtype("int16") for key, value in obs.items() if key in {t["key"] for t in env.observation_spec()["tensors"] if t["dtype"] == "int16"})
        if mode == "disabled":
            before = {key: value.copy() for key, value in obs.items()}
            obs, _ = env.reset(seed=900, options={"reset_mask": np.asarray([True, False, False, False])})
            for key in obs:
                np.testing.assert_array_equal(obs[key][1:], before[key][1:])
        _assert_observation_contract(env, {key: value[0] for key, value in obs.items()})
    finally:
        env.close()
        for game in scalars:
            game.close()


def test_compact_record_replay_and_from_record_keep_layout():
    env = RunEnv("ironclad", 0, profile="standard", partition="test", record_replay=True, observation="compact")
    try:
        obs, _ = env.reset(seed=121)
        action = int(np.flatnonzero(obs["action_mask"])[0])
        env.step(action)
        record = env.record()
        assert isinstance(record, bytes)
        replay = _native.NativeRunSession.replay(record)
        assert replay["obs"]["player"].dtype == np.dtype("int32")
        restored = RunEnv.from_record(record, 1)
        try:
            assert restored.observation_spec()["fingerprint"] == FINGERPRINT
            assert restored.observation_spec()["schema_id"].endswith("compact-v1")
        finally:
            restored.close()
    finally:
        env.close()


@pytest.mark.parametrize("character,expected", [("silent", 12)])
def test_compact_reset_overflow_has_fault_cause(character, expected):
    game = RunEnv(character, 0, profile="standard", partition="test", observation="compact", capacities={"permanent_cards": 11}, record_replay=True)
    try:
        with pytest.raises(_native.RunFaultError, match=rf"Run capacity permanent_cards: {expected} > 11"):
            game.reset(seed=9)
        cause = game.fault_cause()
        assert cause == {"kind": "capacity", "message": "Run capacity exceeded", "capacity": {"group": "permanent_cards", "required": expected, "capacity": 11}}
    finally:
        game.close()


def test_compact_batch_reset_fault_keeps_python_message_and_stage():
    env = RunVectorEnv(
        [{"character": "silent", "ascension": 0}], workers=1,
        autoreset_mode="disabled", observation="compact", capacities={"permanent_cards": 11},
    )
    try:
        with pytest.raises(_native.BatchFaultError) as caught:
            env.reset(seed=9)
        assert caught.value.operation == "reset"
        assert caught.value.slots[0]["stage"] == "reset"
        assert caught.value.slots[0]["message"] == "Run capacity permanent_cards: 12 > 11"
        assert env.fault_cause(0)["capacity"] == {"group": "permanent_cards", "required": 12, "capacity": 11}
        with pytest.raises(_native.RunInputError, match="batch slot index out of range"):
            env.fault_cause(1)
    finally:
        env.close()


def test_compact_committed_export_recovery_is_typed(monkeypatch):
    import spire_lab.gymnasium.run_vector as module

    env = RunVectorEnv(
        [{"character": "ironclad", "ascension": 0}, {"character": "defect", "ascension": 0}],
        workers=1, autoreset_mode="disabled", observation="compact",
    )
    try:
        obs, _ = env.reset(seed=8)
        actions = np.asarray([int(np.flatnonzero(row)[0]) for row in obs["action_mask"]], dtype=np.int64)
        original = module.package_batch
        monkeypatch.setattr(module, "package_batch", lambda *args, **kwargs: (_ for _ in ()).throw(LookupError("test export failure")))
        with pytest.raises(LookupError, match="test export failure"):
            env.step(actions)
        monkeypatch.setattr(module, "package_batch", original)
        recovered = env.recover_committed()
        assert recovered[5].all()
        for tensor in env.observation_spec()["tensors"]:
            assert recovered[0][tensor["key"]].dtype == np.dtype(tensor["dtype"])
    finally:
        env.close()


def test_compact_same_step_final_observation_is_typed():
    env = RunVectorEnv(
        [{"character": "ironclad", "ascension": 20, "partition": "test"}],
        workers=1, autoreset_mode="same_step", observation="compact",
    )
    try:
        obs, _ = env.reset(seed=1901)
        found_final = False
        tensors = {row["key"]: row for row in env.observation_spec()["tensors"]}
        for _ in range(500):
            action = int(np.flatnonzero(obs["action_mask"][0])[0])
            obs, _, terminated, truncated, info = env.step(np.asarray([action], dtype=np.int64))
            if terminated[0] or truncated[0]:
                assert info["_final_obs"].tolist() == [True]
                final = info["final_obs"][0]
                assert isinstance(final, dict)
                for key, value in final.items():
                    assert value.dtype == np.dtype(tensors[key]["dtype"])
                    assert value.shape == tuple(tensors[key]["shape"])
                assert env.single_observation_space.contains(final)
                found_final = True
                break
        assert found_final, "seeded first-legal rollout did not reach a terminal boundary"
    finally:
        env.close()


def test_compact_vector_fault_isolation_preserves_layout():
    env = RunVectorEnv(
        [{"character": "ironclad", "ascension": 10, "partition": "test"}, {"character": "defect", "ascension": 0, "partition": "test"}],
        workers=1, autoreset_mode="disabled", fault_policy="isolate", observation="compact",
        capacities={"permanent_cards": 11},
    )
    try:
        obs, _ = env.reset(seed=[1, 2])
        compact_fingerprint = env.observation_spec()["fingerprint"]
        for _ in range(300):
            actions = np.asarray([int(np.flatnonzero(row)[0]) if row.any() else 0 for row in obs["action_mask"]], dtype=np.int64)
            obs, _, terminated, truncated, info = env.step(actions)
            if info["slot_fault"][0]:
                break
            if terminated[0] or truncated[0]:
                break
        assert info["slot_fault"][0]
        assert env.fault_cause(0)["capacity"]["group"] == "permanent_cards"
        assert obs["player"].dtype == np.dtype("int32")
        assert all(not values[0].any() for values in obs.values())
        obs, _ = env.reset(seed=777, options={"reset_mask": np.asarray([True, False])})
        assert env.observation_spec()["fingerprint"] == compact_fingerprint
        assert obs["player"].dtype == np.dtype("int32")
    finally:
        env.close()


def test_compact_scalar_step_overflow_text_and_cause():
    game = RunEnv(
        "ironclad", 10, profile="standard", partition="test", observation="compact",
        capacities={"permanent_cards": 11},
    )
    try:
        obs, _ = game.reset(seed=1)
        for _step in range(300):
            action = int(np.flatnonzero(obs["action_mask"])[0])
            try:
                obs, _, terminated, _, _info = game.step(action)
            except _native.RunFaultError as error:
                assert str(error) == "post-step Run encoding failed"
                break
            if terminated:
                break
        else:
            pytest.fail("expected permanent_cards overflow on a card gain")
        assert game.fault_cause()["capacity"]["group"] == "permanent_cards"
    finally:
        game.close()


def test_compact_batch_step_overflow_keeps_encode_stage_and_committed_status():
    env = RunVectorEnv(
        [{"character": "ironclad", "ascension": 10, "partition": "test"}],
        workers=1, autoreset_mode="disabled", fault_policy="freeze", observation="compact",
        capacities={"permanent_cards": 11},
    )
    try:
        obs, _ = env.reset(seed=1)
        for _ in range(300):
            action = int(np.flatnonzero(obs["action_mask"][0])[0])
            try:
                obs, _, _, _, _ = env.step(np.asarray([action], dtype=np.int64))
            except _native.BatchFaultError as error:
                assert error.operation == "step"
                row = error.slots[0]
                assert row["status"] == "faulted"
                assert row["stage"] == "encode"
                assert row["committed"] is True
                assert row["message"] == "post-step Run encoding failed"
                break
        else:
            pytest.fail("expected permanent_cards step overflow")
        assert env.fault_cause(0)["capacity"]["group"] == "permanent_cards"
    finally:
        env.close()


def test_compact_column_pool_reuses_released_buffers():
    env = RunVectorEnv([{"character": "ironclad", "ascension": 0}], workers=1, observation="compact")
    try:
        obs, _ = env.reset(seed=91)
        action = np.asarray([int(np.flatnonzero(obs["action_mask"][0])[0])], dtype=np.int64)
        obs, _, _, _, _ = env.step(action)
        key = "permanent_cards"
        pointer = obs[key].__array_interface__["data"][0]
        del obs
        gc.collect()
        obs, _, _, _, _ = env.step(action)
        assert obs[key].__array_interface__["data"][0] == pointer
    finally:
        env.close()
