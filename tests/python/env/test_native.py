import hashlib

import numpy as np
import pytest

from spire_lab import _native
from .support import assert_numeric, template


@pytest.mark.parametrize("batch,workers", [(1, 4), (3, 2), (8, 4)])
def test_real_native_pool_reset_and_step(batch, workers):
    pool = _native.NativeCombatPool(num_envs=batch, workers=workers, record_replay=True)
    spec = pool.observation_spec()
    assert spec["schema_id"] == "spire-lab-combat-observation-v1"
    raw = pool.reset(list(range(batch)), seed=0)
    assert_numeric(raw, batch, spec)
    assert raw["obs"]["player"][0].tolist() == [80, 80, 0, 3, 3, 5, 1]
    assert raw["obs"]["action_kind"][0, :6].tolist() == [1, 2, 2, 2, 2, 2]
    assert raw["obs"]["action_source"][0, :6].tolist() == [-1, 0, 1, 2, 3, 4]
    record = pool.reproduction(0)
    assert record["seed_adapter_id"] == "spire-lab-env-seed-sha256-v1"
    assert record["root"] == hashlib.sha256(b"spire-lab/env-seed/v1\0" + (1).to_bytes(8, "big") + b"\0").digest()
    other = _native.NativeCombatPool(num_envs=batch, workers=1)
    equal = other.reset(list(range(batch)), seed=0)
    for key in raw["obs"]:
        np.testing.assert_array_equal(raw["obs"][key], equal["obs"][key])
    before = {k: v.copy() for k, v in raw["obs"].items()}
    stepped = pool.step_batch(np.ones(batch, dtype=np.int64), raw["tokens"])
    assert_numeric(stepped, batch, spec)
    assert stepped["obs"]["enemy"][0, 0, 0] == 46  # actual Strike, six damage
    assert stepped["obs"]["player"][0, 3] == 2
    assert stepped["rewards"].dtype == np.float32
    assert stepped["terminated"].dtype == np.bool_
    assert stepped["truncated"].dtype == np.bool_
    assert stepped["final_indices"].shape == (0,)
    assert stepped["final_obs"]["player"].shape == (0, 7)
    for key in before:
        np.testing.assert_array_equal(raw["obs"][key], before[key])
    with pytest.raises(_native.CombatInputError) as exc:
        pool.step_batch(np.zeros(batch, dtype=np.int64), raw["tokens"])
    assert exc.value.reason == "token" and exc.value.slot == 0
    pool.close()


@pytest.mark.parametrize("autoreset", ["manual", "same_step"])
def test_compact_finals_and_owned_storage(autoreset):
    config = [template(player_hp=1)]
    pool = _native.NativeCombatPool(config, num_envs=3, autoreset=autoreset, record_replay=True)
    raw = pool.reset([0, 1, 2], seed=2**300 + 7)
    config[0]["hand"][0]["kind"] = "bash"
    raw["obs"]["player"][:] = -100
    spec = pool.observation_spec()
    spec["tensors"][0]["shape"][0] = 999
    assert pool.observation_spec()["tensors"][0]["shape"] == [7]
    for _ in range(3):
        raw = pool.step_batch(np.zeros(3, dtype=np.int64), raw["tokens"])
        if raw["terminated"].any():
            break
    assert raw["terminated"].all() and not raw["truncated"].any()
    assert raw["final_indices"].dtype == np.int64
    assert raw["final_indices"].tolist() == [0, 1, 2]
    assert raw["rewards"].tolist() == [-1, -1, -1]
    assert raw["final_info"]["outcome"].tolist() == [2, 2, 2]
    assert not np.shares_memory(raw["obs"]["player"], raw["final_obs"]["player"])
    assert len(pool.take_completed_records()) == 3
    assert pool.take_completed_records() == []
    if autoreset == "manual":
        assert not raw["tokens"].any()
        with pytest.raises(_native.CombatStateError) as exc:
            pool.step_batch(np.zeros(3, dtype=np.int64), raw["tokens"])
        assert exc.value.reason == "terminal"
    else:
        assert raw["info"]["episode_id"].tolist() == ["episode-1"] * 3
        assert raw["info"]["outcome"].tolist() == [0, 0, 0]
    pool.close()


@pytest.mark.parametrize("bad", [True, -1, 1.5, np.int64(4), "4"])
def test_seeds_reject_lossy_or_non_builtin_values(bad):
    pool = _native.NativeCombatPool(num_envs=3)
    with pytest.raises(_native.CombatInputError) as exc:
        pool.reset([0], seed=[0, 1, bad])
    assert exc.value.reason == "seed" and exc.value.slot == 2
    with pytest.raises(_native.CombatStateError):
        pool.step_batch(np.zeros(3, dtype=np.int64), np.zeros((3, 2), dtype=np.uint64))


def test_partial_reset_preflight_and_whole_batch_validation():
    pool = _native.NativeCombatPool(num_envs=3, record_replay=True)
    with pytest.raises(_native.CombatStateError) as exc:
        pool.reset([0], seed=5, return_all=True)
    assert exc.value.slot == 1 and exc.value.reason == "uninitialized"
    assert pool.reproduction(0) is None
    raw = pool.reset([0, 1, 2], seed=5)
    with pytest.raises(_native.CombatInputError) as exc:
        pool.reset([0], scenario_ids=[None, None, "missing"])
    assert exc.value.slot == 2
    assert pool.reproduction(0)["episode_id"] == "episode-0"
    reset = pool.reset([2, 0], seed=2**300 + 7, return_all=True)
    assert reset["reset_indices"].tolist() == [2, 0]
    assert reset["obs"]["player"].shape == (3, 7)
    assert reset["effective_seed_bytes"] == [(2**300 + 9).to_bytes(38, "big"), (2**300 + 7).to_bytes(38, "big")]
    assert reset["tokens"][1].tolist() == raw["tokens"][1].tolist()


@pytest.mark.parametrize("actions,tokens", [
    ([0], None), (np.array([0.0]), None), (np.array([False]), None),
    (np.array([0], dtype=np.uint64), None), (np.array([0], dtype=">i8"), None),
    (np.zeros((1, 1), dtype=np.int64), None), (np.zeros(2, dtype=np.int64), None),
    (np.zeros(1, dtype=np.int64), np.zeros((1, 2), dtype=np.int64)),
    (np.zeros(1, dtype=np.int64), np.zeros((1, 2), dtype=">u8")),
    (np.zeros(1, dtype=np.int64), [[1, 2], [3]]),
    (np.zeros(1, dtype=np.int64), np.zeros(2, dtype=np.uint64)),
])
def test_strict_step_inputs_are_atomic(actions, tokens):
    pool = _native.NativeCombatPool(record_replay=True)
    raw = pool.reset([0], seed=0)
    with pytest.raises(_native.CombatInputError):
        pool.step_batch(actions, raw["tokens"] if tokens is None else tokens)
    assert pool.reproduction(0)["actions"].tolist() == []
    assert pool.step_batch(np.zeros(1, dtype=np.int64), raw["tokens"])["info"]["episode_steps"].tolist() == [1]


def test_strided_readonly_inputs_and_retained_tokens_are_owned():
    pool = _native.NativeCombatPool(num_envs=3)
    raw = pool.reset([0, 1, 2], seed=0)
    actions = np.zeros(6, dtype=np.int64)[::2]
    token_storage = np.zeros((3, 4), dtype=np.uint64)
    token_storage[:, ::2] = raw["tokens"]
    tokens = token_storage[:, ::2]
    actions.flags.writeable = tokens.flags.writeable = False
    result = pool.step_batch(actions, tokens)
    raw["tokens"][:] = 0
    result["obs"]["player"][:] = 0
    assert pool.step_batch(actions, result["tokens"])["obs"]["player"][0, 0] > 0


@pytest.mark.parametrize("change", [
    {"extra": 1}, {"player_hp": True}, {"player_hp": -1}, {"player_hp": 2**70},
    {"player_hp": 1.0}, {"id": "bad id"}, {"encounter": "slime"},
    {"context": {"act": True}}, {"context": {"act": 2}},
    {"context": {"extra": 1}}, {"context": {"next_encounter": "slime"}},
    {"hand": [{"kind": "strike", "upgraded": 1}]},
    {"hand": [{"kind": "strike", "upgraded": False, "extra": 1}]},
])
def test_strict_configuration(change):
    with pytest.raises(_native.CombatInputError):
        _native.NativeCombatPool([template(**change)])


def test_missing_setup_fields_and_constructor_types():
    for key in template():
        config = template()
        del config[key]
        with pytest.raises(_native.CombatInputError):
            _native.NativeCombatPool([config])
    for kwargs in [dict(num_envs=True), dict(workers=0), dict(workers=1.0),
                   dict(record_replay=1), dict(partition="invalid"), dict(autoreset="invalid")]:
        with pytest.raises(_native.CombatInputError):
            _native.NativeCombatPool(**kwargs)


def test_diagnostics_first_fault_and_record_transport():
    pool = _native.NativeCombatPool(record_replay=True)
    raw = pool.reset([0], seed=2**300 + 7)
    pool.step_batch(np.zeros(1, dtype=np.int64), raw["tokens"])
    pool.mark_export_fault("delivery failed")
    first = pool.diagnostics()
    assert first["status"] == "faulted"
    assert first["faults"][0]["stage"] == "export"
    assert "snapshot" not in first["slots"][0]
    pool.mark_export_fault("later failure")
    assert pool.diagnostics() == first
    record = pool.reproduction(0)
    assert record["status"] == "faulted" and record["actions"].tolist() == [0]
    assert record["attempted_action"] is None
    assert int.from_bytes(record["effective_seed_bytes"], "big") == 2**300 + 7
    with pytest.raises(_native.CombatStateError) as exc:
        pool.reset([0])
    assert exc.value.reason == "faulted"
    pool.close()
    assert len(pool.take_completed_records()) == 1
    assert pool.take_completed_records() == []


def test_descriptor_and_record_conversion_failure_is_nonmutating_and_retains_drain(monkeypatch):
    import json

    pool = _native.NativeCombatPool(record_replay=True)
    raw = pool.reset([0], seed=0)
    pool.step_batch(np.zeros(1, dtype=np.int64), raw["tokens"])
    pool.reset([0])  # abandons one real accepted prefix into the native queue
    original_loads = json.loads

    def fail_conversion(*args, **kwargs):
        raise MemoryError("explicit descriptor conversion failed")

    monkeypatch.setattr(json, "loads", fail_conversion)
    with pytest.raises(MemoryError):
        pool.observation_spec()
    with pytest.raises(MemoryError):
        pool.reproduction(0)
    with pytest.raises(MemoryError):
        pool.take_completed_records()
    assert pool.diagnostics()["status"] == "active"
    # The binding, not native gameplay, owns the already-drained evidence.
    pool.close()
    monkeypatch.setattr(json, "loads", original_loads)
    records = pool.take_completed_records()
    assert [r["episode_id"] for r in records] == ["episode-0", "episode-1"]
    assert records[0]["actions"].tolist() == [0]
    assert [r["status"] for r in records] == ["abandoned", "abandoned"]
    assert pool.take_completed_records() == []


def test_explicit_export_holds_operation_lease(monkeypatch):
    import json

    pool = _native.NativeCombatPool(record_replay=True)
    other = _native.NativeCombatPool()
    pool.reset([0], seed=0)
    original_loads = json.loads
    calls = []

    def inspect_lease(*args, **kwargs):
        for call in [pool.diagnostics, pool.close, pool.take_completed_records,
                     lambda: pool.reproduction(0), lambda: pool.reset([0]),
                     lambda: pool.step_batch(np.zeros(1, dtype=np.int64), np.zeros((1, 2), dtype=np.uint64))]:
            with pytest.raises(_native.CombatStateError) as exc:
                call()
            assert exc.value.reason == "busy"
        assert other.diagnostics()["status"] == "uninitialized"
        calls.append(True)
        return original_loads(*args, **kwargs)

    monkeypatch.setattr(json, "loads", inspect_lease)
    assert pool.reproduction(0)["episode_id"] == "episode-0"
    assert calls == [True]
    assert pool.diagnostics()["status"] == "active"


def test_lossless_scalar_expansion_and_all_seed_entries():
    seed = 2**300 + 7
    pool = _native.NativeCombatPool(num_envs=3, record_replay=True)
    raw = pool.reset([2, 0, 1], seed=seed)
    assert [int.from_bytes(b, "big") for b in raw["effective_seed_bytes"]] == [seed + 2, seed, seed + 1]
    for slot in range(3):
        record = pool.reproduction(slot)
        b = (seed + slot).to_bytes(38, "big")
        assert record["root"] == hashlib.sha256(b"spire-lab/env-seed/v1\0" + len(b).to_bytes(8, "big") + b).digest()
    continued = pool.reset([1], seed=[9, None, 10])
    assert continued["effective_seed_bytes"] == [None]
    assert pool.reproduction(1)["episode_id"] == "episode-1"
    assert int.from_bytes(pool.reproduction(1)["effective_seed_bytes"], "big") == seed + 1


def test_late_action_rejection_and_foreign_tokens_leave_entire_batch_unchanged():
    pool = _native.NativeCombatPool(num_envs=3, record_replay=True)
    raw = pool.reset([0, 1, 2], seed=0)
    with pytest.raises(_native.CombatInputError) as exc:
        pool.step_batch(np.array([0, 0, 31], dtype=np.int64), raw["tokens"])
    assert exc.value.slot == 2 and exc.value.reason == "illegal_action"
    other = _native.NativeCombatPool(num_envs=3)
    foreign = other.reset([0, 1, 2], seed=0)["tokens"]
    with pytest.raises(_native.CombatInputError) as exc:
        pool.step_batch(np.zeros(3, dtype=np.int64), foreign)
    assert exc.value.reason == "token"
    for slot in range(3):
        assert pool.reproduction(slot)["actions"].size == 0
    assert pool.step_batch(np.zeros(3, dtype=np.int64), raw["tokens"])["info"]["episode_steps"].tolist() == [1, 1, 1]


def test_unaligned_and_negative_stride_inputs_are_copied_safely():
    pool = _native.NativeCombatPool(num_envs=3)
    raw = pool.reset([0, 1, 2], seed=0)
    actions = np.ndarray((3,), dtype=np.int64, buffer=bytearray(25), offset=1)
    tokens = np.ndarray((3, 2), dtype=np.uint64, buffer=bytearray(49), offset=1)
    tokens[:] = raw["tokens"]
    assert not actions.flags.aligned and not tokens.flags.aligned
    actions.flags.writeable = tokens.flags.writeable = False
    stepped = pool.step_batch(actions, tokens)
    # Negative strides on both axes, preserving logical row/word order.
    token_store = stepped["tokens"][::-1, ::-1].copy()
    negative_tokens = token_store[::-1, ::-1]
    stepped = pool.step_batch(np.zeros(3, dtype=np.int64)[::-1], negative_tokens)
    assert stepped["info"]["episode_steps"].tolist() == [2, 2, 2]


def test_catalog_descriptor_config_and_current_tokens_do_not_alias():
    config = [template()]
    pool = _native.NativeCombatPool(config)
    config[0]["hand"][0]["kind"] = "bash"
    desc = pool.catalog_descriptor()
    assert desc["templates"][0]["hand"] == [{"kind": "strike", "upgraded": False}]
    assert desc["templates"][0]["context"] == {"act": None, "next_encounter": None}
    desc["templates"][0]["hand"][0]["kind"] = "defend"
    assert pool.catalog_descriptor()["templates"][0]["hand"][0]["kind"] == "strike"
    raw = pool.reset([0], seed=0)
    valid = raw["tokens"].copy()
    raw["tokens"][:] = 0
    with pytest.raises(_native.CombatInputError):
        pool.step_batch(np.zeros(1, dtype=np.int64), raw["tokens"])
    assert pool.step_batch(np.ones(1, dtype=np.int64), valid)["obs"]["enemy"][0, 0, 0] == 46


def test_empty_reset_selected_and_all_rows_are_reads():
    pool = _native.NativeCombatPool(record_replay=True)
    first = pool.reset([0], seed=0)
    empty = pool.reset([], seed=123)
    assert empty["obs"]["player"].shape == (0, 7)
    assert empty["tokens"].shape == (0, 2)
    assert empty["effective_seed_bytes"] == []
    all_rows = pool.reset([], return_all=True)
    np.testing.assert_array_equal(all_rows["tokens"], first["tokens"])
    assert pool.reproduction(0)["episode_id"] == "episode-0"


@pytest.mark.parametrize("kwargs", [
    {"catalog": [template(id="\ud800")]},
    {"catalog": [template(encounter="\ud800")]},
    {"catalog": [template(context={"next_encounter": "\ud800"})]},
    {"catalog": [template(hand=[{"kind": "\ud800", "upgraded": False}])]},
    {"catalog": [dict(template(), **{"\ud800": 1})]},
    {"partition": "\ud800"},
    {"autoreset": "\ud800"},
])
def test_malformed_utf8_setup_strings_have_typed_input_errors(kwargs):
    with pytest.raises(_native.CombatInputError) as exc:
        _native.NativeCombatPool(**kwargs)
    assert exc.value.slot is None
    assert exc.value.reason == "scenario"
    assert exc.value.faults == []


@pytest.mark.parametrize("return_all", [False, True])
def test_malformed_unselected_scenario_preserves_reset_state(return_all):
    pool = _native.NativeCombatPool(num_envs=3, workers=2, record_replay=True,
                                    partition="test", autoreset="same_step")
    initial = pool.reset([0, 1, 2], seed=2**300 + 7)
    before = pool.diagnostics()
    before_records = [pool.reproduction(slot) for slot in range(3)]
    with pytest.raises(_native.CombatInputError) as exc:
        pool.reset([0], seed=[999, None, 888],
                   scenario_ids=["starter-jaw-worm-a0-v1", None, "\ud800"],
                   return_all=return_all)
    assert exc.value.slot == 2
    assert exc.value.reason == "scenario"
    assert exc.value.faults == []
    assert pool.diagnostics() == before
    unchanged = pool.reset([], return_all=True)
    for key in initial["obs"]:
        np.testing.assert_array_equal(unchanged["obs"][key], initial["obs"][key])
    for key in initial["info"]:
        np.testing.assert_array_equal(unchanged["info"][key], initial["info"][key])
    np.testing.assert_array_equal(unchanged["tokens"], initial["tokens"])
    for slot, previous in enumerate(before_records):
        current = pool.reproduction(slot)
        np.testing.assert_array_equal(current.pop("actions"), previous.pop("actions"))
        assert current == previous  # seed/root, ordinal, scenario, partition, recording
    assert pool.take_completed_records() == []
    # No decision serial or episode ordinal was consumed, nor was a reseed or
    # scenario override installed. Original tokens remain usable for every slot.
    stepped = pool.step_batch(np.zeros(3, dtype=np.int64), initial["tokens"])
    assert stepped["tokens"][:, 1].tolist() == [4, 5, 6]
    continued = pool.reset([0, 1, 2], return_all=True)
    assert continued["effective_seed_bytes"] == [None, None, None]
    assert continued["tokens"][:, 1].tolist() == [7, 8, 9]
    assert continued["info"]["episode_id"].tolist() == ["episode-1"] * 3
    assert continued["info"]["scenario_id"].tolist() == ["starter-jaw-worm-a0-v1"] * 3
    pool.close()


def test_malformed_export_fault_message_is_recoverable_input_error():
    pool = _native.NativeCombatPool()
    before = pool.diagnostics()
    with pytest.raises(_native.CombatInputError) as exc:
        pool.mark_export_fault("\ud800")
    assert exc.value.slot is None and exc.value.reason == "scenario"
    assert pool.diagnostics() == before


def test_observation_spec_publishes_native_fingerprint_outside_hashed_descriptor():
    import copy
    import json

    pool = _native.NativeCombatPool()  # metadata must not require recording
    spec = pool.observation_spec()
    golden = "8fd66d212259d2bdd27139988a772e1a6c223f8d65524f38c74174a0d0843357"
    assert spec["fingerprint"] == golden
    original = copy.deepcopy(spec)
    descriptor = copy.deepcopy(spec)
    fingerprint = descriptor.pop("fingerprint")
    canonical = json.dumps(descriptor, ensure_ascii=False, sort_keys=True,
                           separators=(",", ":")).encode("utf-8")
    assert hashlib.sha256(b"spire-lab/env-schema/v1\0" + canonical).hexdigest() == fingerprint
    assert descriptor["schema_id"] == "spire-lab-combat-observation-v1"
    spec["fingerprint"] = "modified"
    spec["capacities"]["hand"] = 999
    spec["tensors"][0]["shape"][0] = 999
    assert pool.observation_spec() == original
    assert pool.reset([0], seed=0)["obs"]["player"].shape == (1, 7)
