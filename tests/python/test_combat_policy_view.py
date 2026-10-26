"""Additive normal-Run public transport; original schemas remain frozen."""

import numpy as np

from spire_lab.gymnasium import RunEnv


def test_normal_run_public_identities_and_owned_common_sidecar():
    env = RunEnv("ironclad", 0, profile="m10-ironclad-policy", partition="test")
    legacy = env.observation_spec()
    identity = env.environment_spec()
    assert identity["rules_id"] == "spire-gym-native-m10-rules-v1"
    assert identity["reward_id"] == "spire-gym-floor-key-terminal-reward-v1"
    assert identity["mode"] == "normal_run"
    obs, _ = env.reset(seed=11)
    view = env.combat_view()
    spec = env.combat_observation_spec()
    assert spec["schema_id"] == "spire-gym-combat-policy-view-v1"
    assert view["info"]["routing"] == "noncombat"
    assert not view["obs"]["action_mask"].any()
    assert "map_nodes" not in view["obs"]
    assert "token" not in view["obs"]
    assert "effective_seed" not in view["obs"]
    view["obs"]["player"][0] = -123
    assert env.combat_view()["obs"]["player"][0] == obs["player"][0]
    env.step(int(np.flatnonzero(obs["action_mask"])[0]))
    assert env.observation_spec() == legacy
    env.close()


def test_policy_run_replay_identity_is_new_and_old_profile_remains_frozen():
    from spire_lab import _native

    old = RunEnv("ironclad", 0, profile="m10-ironclad-heart")
    env = RunEnv("ironclad", 0, profile="m10-ironclad-policy", record_replay=True)
    assert old.observation_spec()["schema_id"] == "spire-gym-run-observation-v2"
    assert (
        old.observation_spec()["fingerprint"]
        == "7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db"
    )
    assert env.observation_spec()["schema_id"] == "spire-gym-run-observation-v3"
    obs, _ = env.reset(seed=11)
    env.step(int(np.flatnonzero(obs["action_mask"])[0]))
    record = env.record()
    assert record is not None
    replay = _native.NativeRunSession.replay(record)
    assert replay["accepted_count"] == 1
    old.close()
    env.close()


def test_cross_mode_native_tokens_are_foreign_even_after_reset_and_in_batches():
    import os
    import subprocess
    import sys

    script = r"""
import json
import numpy as np
from spire_lab import _native
catalog = json.dumps({"entries": [{"id": "case", "encounter": "cultist", "hp":80,"max_hp":80,"deck":[{"card":"strike","upgrades":0}],"relics":[],"potions":[None,None,None]}]})
a = _native.NativeIroncladCombatSession(catalog, partition="test")
r = _native.NativeRunSession("ironclad",0,profile="m10-ironclad-policy",partition="test")
b = a.reset(seed=41)
r0 = r.reset(seed=41)
assert b["token"] != r0["token"], "different owner token namespaces collided"
for owner, wrong in [(a,r0["token"]),(r,b["token"])]:
    try: owner.step(0,wrong)
    except _native.RunStateError: pass
    else: raise AssertionError("foreign owner token dispatched")
assert a.diagnostic()["accepted_steps"] == 0
assert r.reproduction()["last_accepted_action"] is None
b2 = a.reset(seed=42)
try: a.step(0,r0["token"])
except _native.RunStateError: pass
else: raise AssertionError("foreign token accepted after reset")
assert a.decision()["token"] == b2["token"]
batch = _native.NativeIroncladCombatBatch(2,catalog,partition="test")
rows = batch.reset(seeds=[1,2])
r1 = _native.NativeRunSession("ironclad",0,profile="m10-ironclad-policy")
r2 = _native.NativeRunSession("ironclad",0,profile="m10-ironclad-policy")
foreign = np.array([r1.reset(seed=1)["token"],r2.reset(seed=2)["token"]],dtype=np.uint64)
try: batch.step(np.zeros(2,dtype=np.int64),foreign)
except _native.RunStateError: pass
else: raise AssertionError("foreign batch tokens dispatched")
assert batch.diagnostic()["accepted_steps"] == [0,0]
a.close(); r.close(); batch.close(); r1.close(); r2.close()
"""
    subprocess.run([sys.executable, "-c", script], env=dict(os.environ), check=True)


def test_actual_runtime_fault_diagnostic_is_bounded_and_batch_prefix_cannot_retry():
    import json

    import pytest

    from spire_lab import _native

    entries = [
        {
            "id": "healthy",
            "encounter": "cultist",
            "hp": 80,
            "max_hp": 80,
            "deck": [{"card": "strike", "upgrades": 0}],
            "relics": [],
            "potions": [None, None, None],
        },
        {
            "id": "limit",
            "encounter": "cultist",
            "hp": 900,
            "max_hp": 999,
            "deck": [{"card": "feed", "upgrades": 0}] * 5,
            "relics": ["neows_lament"],
            "potions": [None, None, None],
        },
    ]
    catalog = json.dumps({"entries": entries})
    single = _native.NativeIroncladCombatSession(catalog, partition="test")
    b = single.reset(seed=41, entry_id="limit")
    with pytest.raises(_native.RunFaultError):
        single.step(0, b["token"])
    diagnostic = single.diagnostic()
    assert diagnostic["accepted_steps"] == 1
    assert diagnostic["last_step_committed"] is True
    text = diagnostic["fault"]
    assert len(text) < 256
    assert all(
        marker not in text
        for marker in [
            "current:",
            "pending:",
            "selection:",
            "constructor_cursor",
            "latest_child_cursor",
            "attempted_action:",
        ]
    )
    assert "combat" in text.lower()
    with pytest.raises(_native.RunFaultError):
        single.reset(seed=42)
    batch = _native.NativeIroncladCombatBatch(2, catalog, partition="test")
    rows = batch.reset(seeds=[1, 41], entry_ids=["healthy", "limit"])
    with pytest.raises(_native.RunFaultError):
        batch.step(np.zeros(2, dtype=np.int64), rows["tokens"])
    diagnostic = batch.diagnostic()
    assert diagnostic["accepted_steps"] == [1, 1]
    assert diagnostic["first_fault"]["failing_slot"] == 1
    assert all(len(row["fault"]) < 256 for row in diagnostic["sessions"])
    with pytest.raises(_native.RunFaultError):
        batch.reset(seeds=[2, 42])
    with pytest.raises(_native.RunFaultError):
        batch.step(np.zeros(2, dtype=np.int64), rows["tokens"])
    assert batch.diagnostic() == diagnostic
    single.close()
    batch.close()
