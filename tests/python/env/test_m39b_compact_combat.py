"""Compact Standard combat publication contracts."""

import hashlib
import inspect
import json

import numpy as np
import pytest

from spire_lab import _native

from spire_lab.gymnasium import (
    CombatEnv, CombatVectorEnv, IroncladCombatEnv, IroncladCombatVectorEnv,
    PublicBranch, RunEnv, RunVectorEnv, StandardCombatEnv, StandardCombatVectorEnv,
)
from spire_lab.gymnasium.public_search import open_public_search

from .test_public_search import LIVE_SEED, RESEARCH_SEED, entry
from .test_standard_combat import legal_action
from .test_standard_positions import catalog, entry as combat_entry, make_recipe_record, row


LIMITS = dict(max_live_branches=4, max_branches=8, max_steps_per_branch=16,
              max_total_steps=32)


def test_compact_search_full_and_fault_rows():
    session = _native.NativeStandardCombatSession(json.dumps(entry()).encode(), partition="test")
    session.reset(LIVE_SEED)
    search = session.public_search(RESEARCH_SEED, **LIMITS, observation="compact")
    branch = search.branch(0)
    obs = search.decision(branch)["obs"]
    assert obs["player"].dtype == np.int32

BOUNDS = "standard-combat-compact-v1"
PLAY = "standard-combat-compact-play-v1"
COMPACT = {"observation": "compact"}
OVERRIDES = {"observation": "compact", "capacity_preset": PLAY, "capacities": {"offers": 2}}


def typed(obs, spec, batch=None):
    assert obs.keys() == {tensor["key"] for tensor in spec["tensors"]}
    for tensor in spec["tensors"]:
        value = obs[tensor["key"]]
        assert value.dtype == np.dtype(tensor["dtype"])
        assert value.shape == (() if batch is None else (batch,)) + tuple(tensor["shape"])


def widens(small, large, *, vector=False):
    assert small.keys() == large.keys()
    for key in small:
        value = small[key]
        dense = large[key]
        pad = [(0, big - n) for n, big in zip(value.shape, dense.shape, strict=True)]
        if vector:
            assert pad[0] == (0, 0)
        np.testing.assert_array_equal(np.pad(value.astype(dense.dtype), pad), dense, err_msg=key)


@pytest.mark.parametrize("preset, expected_bytes", [(None, 229926), (PLAY, 143313)])
@pytest.mark.parametrize("character", ["ironclad", "silent", "defect", "watcher"])
def test_compact_combat_equivalence_and_spaces(preset, expected_bytes, character):
    resource = combat_entry(character=character)
    resource["ascension"] = 20
    resource["potions"] = [None, None]
    cat = catalog(resource)
    dense = StandardCombatEnv(cat)
    compact = StandardCombatEnv(cat, **COMPACT, capacity_preset=preset)
    try:
        spec = compact.observation_spec()
        assert spec["semantics"]["capacity_preset"] == (preset or BOUNDS)
        assert spec["schema_id"] == "spire-gym-standard-combat-view-compact-v1"
        small, _ = compact.reset(seed=13)
        large, _ = dense.reset(seed=13)
        assert sum(array.nbytes for array in small.values()) == expected_bytes
        assert sum(array.nbytes for array in large.values()) == 644378
        for _ in range(8):
            typed(small, spec)
            assert compact.observation_space.contains(small)
            widens(small, large)
            if not large["action_mask"].any():
                break
            action = int(np.flatnonzero(large["action_mask"])[0])
            a, b = compact.step(action), dense.step(action)
            assert a[1:4] == b[1:4]
            small, large = a[0], b[0]
        assert compact.fault_cause() is None
    finally:
        compact.close()
        dense.close()


@pytest.mark.parametrize("mode", ["disabled", "next_step", "same_step"])
@pytest.mark.parametrize("workers", [1, 16])
def test_compact_combat_vector_lifecycle(mode, workers):
    resource = combat_entry(deck=["bludgeon"] * 10)
    for card in resource["deck"]:
        card["upgrades"] = 1
    cat = catalog(resource)
    dense = StandardCombatVectorEnv(cat, 2, workers=workers, autoreset_mode=mode)
    small = StandardCombatVectorEnv(cat, 2, workers=workers, autoreset_mode=mode, **COMPACT, capacity_preset=PLAY)
    spec = small.observation_spec()
    try:
        left, _ = small.reset(seed=27)
        right, _ = dense.reset(seed=27)
        held = left
        copied = {k: v.copy() for k, v in held.items()}
        finals = 0
        for _ in range(100):
            typed(left, spec, 2)
            widens(left, right, vector=True)
            actions = np.asarray([int(np.flatnonzero(mask)[0]) if mask.any() else 0
                                  for mask in right["action_mask"]], dtype=np.int64)
            left, lr, lt, lx, li = small.step(actions)
            right, rr, rt, rx, ri = dense.step(actions)
            for a, b in [(lr, rr), (lt, rt), (lx, rx)]:
                np.testing.assert_array_equal(a, b)
            if "_final_obs" in li:
                np.testing.assert_array_equal(li["_final_obs"], ri["_final_obs"])
                for slot in np.flatnonzero(li["_final_obs"]):
                    typed(li["final_obs"][slot], spec)
                    widens(li["final_obs"][slot], ri["final_obs"][slot])
                    finals += 1
            if mode == "disabled" and lt.any():
                options = {"reset_mask": lt}
                left, _ = small.reset(options=options)
                right, _ = dense.reset(options=options)
        if mode == "same_step":
            assert finals > 0
        assert small.take_completed_records() == []
    finally:
        small.close()
        dense.close()
    for key in held:
        np.testing.assert_array_equal(held[key], copied[key])


def test_compact_recovered_rows(monkeypatch):
    import spire_lab.gymnasium.standard_combat_vector as module

    resource = combat_entry()
    resource["potions"][0] = "smoke_bomb"
    env = StandardCombatVectorEnv(catalog(resource), 2, partition="test",
                                  autoreset_mode="same_step", workers=16, **COMPACT)
    try:
        obs, _ = env.reset(seed=27)
        spec = env.observation_spec()
        actions = np.asarray([legal_action({k: v[i] for k, v in obs.items()}, spec, "use_potion")
                              for i in range(2)], dtype=np.int64)

        def fail(*args, **kwargs):
            raise LookupError("test export failure")

        with monkeypatch.context() as patch:
            patch.setattr(module, "package_batch", fail)
            with pytest.raises(LookupError, match="test export failure"):
                env.step(actions)
        first = env.recover_committed()
        second = env.recover_committed()
        assert first[5].all()
        typed(first[0], spec, 2)
        assert first[4]["_final_obs"].all()
        for slot in range(2):
            typed(first[4]["final_obs"][slot], spec)
            for key in first[4]["final_obs"][slot]:
                np.testing.assert_array_equal(first[4]["final_obs"][slot][key], second[4]["final_obs"][slot][key])
        for key in first[0]:
            np.testing.assert_array_equal(first[0][key], second[0][key])
    finally:
        env.close()


def test_compact_search_step_many_fault_rows_and_nested_layout():
    resource = combat_entry()
    resource["potions"][0] = "attack_potion"
    data = json.dumps(catalog(resource))
    session = _native.NativeStandardCombatSession(data, partition="test")
    session.reset(LIVE_SEED)
    search = session.public_search(RESEARCH_SEED, **LIMITS, workers=16, **OVERRIDES)
    try:
        spec = search.observation_spec()
        assert spec["semantics"]["capacity_preset"] == PLAY
        branches = [search.branch(i) for i in range(3)]
        rows = [search.decision(branch) for branch in branches]
        tokens = np.asarray([r["token"] for r in rows], dtype=np.uint64)
        potion = legal_action(rows[0]["obs"], spec, "use_potion")
        end_turn = legal_action(rows[2]["obs"], spec, "end_turn")
        mixed = search.step_many([branches[0], branches[2]], np.asarray([tokens[0], tokens[2]]),
                                 np.asarray([potion, end_turn], dtype=np.int64))
        typed(mixed["obs"], spec, 2)
        assert all(not arr[0].any() for arr in mixed["obs"].values())
        assert mixed["fault"][0]["capacity"] == {"group": "combat_card_offers", "required": 3, "capacity": 2}
        assert (mixed["fault"][0]["stage"], mixed["fault"][0]["committed"]) == ("encode", True)
        assert mixed["fault"][1] is None
        faulted = search.step_many([branches[1]], tokens[1:2], np.asarray([potion], dtype=np.int64))
        typed(faulted["obs"], spec, 1)
        assert all(not arr.any() for arr in faulted["obs"].values())
        assert faulted["action_mask"].shape == (1, 192)
        assert search.actions(branches[1]) == [potion]
        with pytest.raises(_native.RunFaultError, match="Run capacity combat_card_offers: 3 > 2"):
            search.decision(branches[1])
        fork = search.fork(branches[2])
        assert search.decision(fork)["obs"]["player"].dtype == np.int32
        child = search.public_search(fork, RESEARCH_SEED, **LIMITS)
        try:
            assert child.decision(child.branch())["obs"]["player"].dtype == np.int64
        finally:
            child.close()
    finally:
        search.close()
        session.close()


def test_compact_combat_recipe_and_record_surface():
    record, steps = make_recipe_record(enter_combat=True)
    recipe = {"id": "recipe", "recipe": {"record_sha256": hashlib.sha256(record).hexdigest(), "steps": steps}}
    dense = StandardCombatEnv(catalog(recipe), records=[record], partition="test")
    small = StandardCombatEnv(catalog(recipe), records=[record], partition="test", **COMPACT, capacity_preset=PLAY)
    try:
        left, info = small.reset(seed=33)
        right, _ = dense.reset(seed=33)
        assert info["start_kind"] == "reachable_recipe"
        widens(left, right)
        left_search = small.public_search(RESEARCH_SEED, **COMPACT, limits=LIMITS)
        right_search = dense.public_search(RESEARCH_SEED, limits=LIMITS)
        try:
            assert left_search.spec()["root_digest"] == right_search.spec()["root_digest"]
            widens(left_search.branch(0).decision()[0], right_search.branch(0).decision()[0])
        finally:
            left_search.close()
            right_search.close()
    finally:
        small.close()
        dense.close()
    altered = json.loads(record)
    altered["observation"] = "compact"
    bad = json.dumps(altered).encode()
    recipe["recipe"]["record_sha256"] = hashlib.sha256(bad).hexdigest()
    with pytest.raises(_native.RunInputError):
        StandardCombatEnv(catalog(recipe), records=[bad], **COMPACT)
    with pytest.raises(_native.RunInputError):
        StandardCombatEnv(dict(catalog(combat_entry()), observation="compact"), **COMPACT)


def test_combat_layout_validation_and_legacy_rejection():
    for options in [
        {"observation": False}, {"observation": "other"}, {"capacity_preset": PLAY},
        {"capacities": {}}, {**COMPACT, "capacity_preset": "other"},
        *({**COMPACT, "capacities": cap} for cap in
          [{"map_nodes": 2}, {"potions": True}, {"potions": 0}, {"actions": 2049}, {"known_draw": 2}]),
    ]:
        with pytest.raises(_native.RunInputError):
            _native.NativeStandardCombatSession(object(), **options)
    for construct in [
        lambda: _native.NativeCombatPool(object(), **COMPACT),
        lambda: _native.NativeIroncladCombatSession(object(), **COMPACT),
        lambda: _native.NativeCombatTrainingBatch(1, object(), kind="ironclad", **COMPACT),
        lambda: CombatEnv(catalog=object(), **COMPACT),
        lambda: CombatVectorEnv(1, catalog=object(), **COMPACT),
        lambda: IroncladCombatEnv(object(), **COMPACT),
        lambda: IroncladCombatVectorEnv(1, object(), **COMPACT),
        lambda: open_public_search(object(), RESEARCH_SEED, LIMITS, **COMPACT),
    ]:
        with pytest.raises(_native.RunInputError, match="compact combat observation is unsupported"):
            construct()
    run = RunEnv("ironclad", 0)
    vector = RunVectorEnv([{"character": "ironclad", "ascension": 0}])
    try:
        for call in [run.combat_view, run.combat_observation_spec, vector.combat_views,
                     lambda **opts: run.public_search(RESEARCH_SEED, **opts)]:
            with pytest.raises(_native.RunInputError, match="compact combat observation is unsupported"):
                call(**COMPACT)
    finally:
        run.close()
        vector.close()


def test_combat_layout_selector_signatures_and_order():
    selectors = [
        _native.NativeCombatPool, _native.NativeCombatTrainingBatch, _native.NativeStandardCombatSession,
        _native.NativeIroncladCombatSession, _native.NativeRunSession.combat_view,
        _native.NativeRunSession.combat_observation_spec, _native.NativeRunBatch.combat_views,
        _native.NativeRunSession.public_search, _native.NativePublicSearch.public_search,
        _native.NativeStandardCombatSession.public_search, _native.NativeIroncladCombatSession.public_search,
        CombatEnv, CombatVectorEnv, IroncladCombatEnv, IroncladCombatVectorEnv,
        StandardCombatEnv, StandardCombatVectorEnv, StandardCombatEnv.public_search, RunEnv.public_search,
        RunEnv.combat_view, RunEnv.combat_observation_spec, RunVectorEnv.combat_views,
        PublicBranch.public_search, open_public_search,
    ]
    for selector in selectors:
        params = [p for p in inspect.signature(selector).parameters.values()
                  if p.kind == inspect.Parameter.KEYWORD_ONLY]
        assert [p.name for p in params[-3:]] == ["observation", "capacity_preset", "capacities"]
        assert params[-3].default in ("dense", Ellipsis)
        assert params[-2].default is params[-1].default is None
    dense = _native.NativeStandardCombatSession(json.dumps(entry()))
    explicit = _native.NativeStandardCombatSession(json.dumps(entry()), observation="dense")
    try:
        assert dense.observation_spec() == explicit.observation_spec()
        widens(dense.reset(13)["obs"], explicit.reset(13)["obs"])
        a = dense.public_search(RESEARCH_SEED, **LIMITS)
        b = dense.public_search(RESEARCH_SEED, **LIMITS, observation="dense")
        assert a.observation_spec() == b.observation_spec() == dense.observation_spec()
    finally:
        dense.close()
        explicit.close()


@pytest.mark.parametrize("mixed", [False, True])
@pytest.mark.parametrize("policy", ["isolate", "freeze"])
def test_compact_first_reset_policy(mixed, policy):
    bad = combat_entry("value")
    bad["gold"] = 2**31
    good = combat_entry("fitting")
    env = StandardCombatVectorEnv(catalog(bad, good), 2, partition="test", **COMPACT,
                                  fault_policy=policy, entry_ids=["value", "fitting" if mixed else "value"])
    try:
        if policy == "freeze":
            with pytest.raises(_native.BatchFaultError) as caught:
                env.reset(seed=13)
            assert caught.value.slots[0]["stage"] == "reset"
        else:
            obs, info = env.reset(seed=13)
            typed(obs, env.observation_spec(), 2)
            assert info["slot_fault"].tolist() == [True, not mixed]
            assert all(not arr[0].any() for arr in obs.values())
            assert any(arr[1].any() for arr in obs.values()) is mixed
        assert env.fault_cause(0)["capacity"] == {
            "group": "player.gold", "required": 2**31, "capacity": 2**31 - 1,
        }
    finally:
        env.close()


def test_compact_retired_fault_cause():
    bad = combat_entry("value")
    bad["gold"] = 2**31
    row_fault = combat_entry("row", deck=["strike"] * 97)
    good = combat_entry("fitting")
    env = StandardCombatVectorEnv(catalog(bad, good, row_fault), 2, partition="test",
                                  **COMPACT, capacity_preset=PLAY,
                                  fault_policy="isolate", entry_ids=["fitting", "value"])
    try:
        env.reset(seed=13)
        first = env.fault_cause(1)
        select = np.asarray([False, True])
        env.reset(seed=27, options={"entry_ids": [None, "row"], "reset_mask": select})
        second = env.fault_cause(1)
        assert second["capacity"] == {"group": "permanent_cards", "required": 97, "capacity": 96}
        obs, info = env.reset(seed=28, options={"entry_ids": [None, "fitting"], "reset_mask": select})
        assert env.fault_cause(1) is None
        assert not info["slot_fault"].any()
        typed(obs, env.observation_spec(), 2)
        diagnostic = env.take_fault_records()
        assert [r["fault_cause"] for r in diagnostic] == [first, second]
        assert all(r["record"] is None and r["slot"] == 1 for r in diagnostic)
        assert env.take_fault_records() == env.take_completed_records() == []
    finally:
        env.close()


def test_compact_search_step_many_all_fault_mask():
    resource = combat_entry(character="defect", deck=["seek"])
    resource["deck"][0]["upgrades"] = 1
    resource["position"] = {
        "player": {"energy": 9},
        "piles": {"hand": [row("seek", 1)], "draw": [row("dazed", generated=True)] * 193,
                  "draw_top": [], "discard": [], "exhaust": []},
    }
    with_options = dict(COMPACT, capacity_preset=PLAY)
    session = _native.NativeStandardCombatSession(json.dumps(catalog(resource)), partition="test")
    session.reset(13)
    search = session.public_search(17, **LIMITS, **with_options)
    try:
        branches = [search.branch(i) for i in (0, 1)]
        rows = [search.decision(branch) for branch in branches]
        spec = search.observation_spec()
        actions = [legal_action(r["obs"], spec, "play_card") for r in rows]
        result = search.step_many(branches, np.asarray([r["token"] for r in rows], dtype=np.uint64),
                                  np.asarray(actions, dtype=np.int64), view="mask")
        assert result["obs"] is None
        assert result["action_mask"].shape == (2, 192)
        assert result["action_mask"].dtype == np.int8
        assert not result["action_mask"].any()
        for fault in result["fault"]:
            assert fault["capacity"] == {"group": "actions", "required": 193, "capacity": 192}
    finally:
        search.close()
        session.close()
