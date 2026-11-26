"""Python contract for native public-search playouts (M41, rule ``spire-gym-public-playout-v1``)."""

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import RunEnv, StandardCombatEnv
from .test_public_search import LIVE_SEED, RESEARCH_SEED, entry


LIMITS = dict(max_live_branches=64, max_branches=512, max_steps_per_branch=60, max_total_steps=100_000)
SAMPLES = (0, 3, 3, 7, 11, 19)


def standard_env(**changes):
    catalog = entry()
    catalog["entries"][0].update(changes)
    env = StandardCombatEnv(catalog, partition="test")
    env.reset(seed=LIVE_SEED)
    return env


def run_env():
    # Normal creation (the short test route meets 1-HP enemies), the first-legal route into the first combat.
    run = RunEnv("ironclad", 20, profile="standard", partition="test")
    observation, _ = run.reset(seed=11)
    for _ in range(80):
        if observation["enemies_occupied"].any():
            return run
        legal = np.flatnonzero(observation["action_mask"].reshape(-1))
        observation, _, terminated, _, _ = run.step(int(legal[0]))
        assert not terminated
    pytest.fail("Standard route did not reach a combat decision")


def searches(env, workers=1, **layout):
    return env.public_search(RESEARCH_SEED, limits=LIMITS, workers=workers, **layout)


def step_reference(search, samples, *, view="mask", rule="uniform_exclude_verbs", exclude_verbs=None,
                   never_verbs=None, last_resort_verbs=None, max_steps=None, playout_seed=0):
    """T14/M42: the declared rule one action at a time — ``playout_action`` then ``branch.step``."""
    rows = []
    for sample in samples:
        branch = search.branch(sample)
        steps, reward, terminated, truncated, reason, refusal = 0, 0.0, False, False, None, None
        while reason is None:
            if max_steps is not None and steps == max_steps:
                reason = "max_steps"
                break
            action = branch.playout_action(
                rule=rule, exclude_verbs=exclude_verbs, never_verbs=never_verbs,
                last_resort_verbs=last_resort_verbs, playout_seed=playout_seed,
            )
            try:
                _, reward, terminated, truncated, info = branch.step(action, view=view)
            except _native.RunUnsupportedBoundaryError:
                reason, refusal = "refused", "HIDDEN_ORDER_PICK"
                break
            steps += 1
            if terminated:
                reason = info.get("terminal_reason") or info["end_reason"]
            elif truncated:
                reason = "step_limit"
        rows.append((branch.actions(), reward, terminated, truncated, reason, refusal, steps))
    return rows


def assert_matches(result, branches, reference):
    for i, (actions, reward, terminated, truncated, reason, refusal, steps) in enumerate(reference):
        assert branches[i].actions() == actions
        assert (int(result["rewards"][i]), bool(result["terminated"][i]), bool(result["truncated"][i])) == \
            (int(reward), terminated, truncated)
        assert (result["end_reason"][i], result["refusal"][i], int(result["steps"][i])) == (reason, refusal, steps)


@pytest.mark.parametrize("workers", [1, 4])
@pytest.mark.parametrize("layout", [{}, {"observation": "compact"}])
def test_playout_many_equals_one_action_at_a_time(workers, layout):
    env = standard_env()
    try:
        playout, mask_ref, full_ref = searches(env, workers, **layout), searches(env, 1, **layout), searches(env, 1, **layout)
        branches = [playout.branch(k) for k in SAMPLES]
        before = playout.counters()["total_steps"]
        result = playout.playout_many(branches)
        assert_matches(result, branches, step_reference(mask_ref, SAMPLES, view="mask"))
        assert_matches(result, branches, step_reference(full_ref, SAMPLES, view="full"))
        assert playout.counters()["total_steps"] == before + int(result["steps"].sum())
        assert playout.counters() == mask_ref.counters()
    finally:
        env.close()


def test_run_combat_playouts_and_call_caps_match_the_reference():
    run = run_env()
    try:
        playout, reference = searches(run, 4), searches(run, 1)
        branches = [playout.branch(k) for k in SAMPLES]
        result = playout.playout_many(branches, exclude_verbs=["end_turn"], max_steps=9, playout_seed=5)
        assert_matches(result, branches, step_reference(reference, SAMPLES, exclude_verbs=["end_turn"], max_steps=9, playout_seed=5))
        assert set(result["end_reason"]) <= {"max_steps", "victory", "death", "step_limit", "refused"}
        live = [b for b, r in zip(branches, result["end_reason"], strict=True) if r == "max_steps"]
        assert live and all(not t for t, r in zip(result["truncated"], result["end_reason"]) if r == "max_steps")
        again = playout.playout_many(live, exclude_verbs=["end_turn"], playout_seed=5)
        assert not again["_fault"].any()
    finally:
        run.close()


def test_summary_matches_the_published_rows_and_learns_balance():
    env = standard_env()
    try:
        search = searches(env, 2)
        branches = [search.branch(k) for k in SAMPLES]
        result = search.playout_many(branches, max_steps=4)
        assert result["rule_id"] == "spire-gym-public-playout-v1"
        for i, branch in enumerate(branches):
            observation, _ = branch.decision()
            assert (int(result["player_hp"][i]), int(result["player_max_hp"][i])) == (int(observation["player"][0]), int(observation["player"][1]))
        result2 = search.playout_many([b for b, r in zip(branches, result["end_reason"]) if r == "max_steps"])
        # The final rows of the second call equal each branch's published rows.
        finished = [b for b, r in zip(branches, result["end_reason"]) if r == "max_steps"]
        for i, branch in enumerate(finished):
            observation, _ = branch.decision()
            assert (int(result2["player_hp"][i]), int(result2["player_max_hp"][i])) == (int(observation["player"][0]), int(observation["player"][1]))
            rows = observation["enemies"][observation["enemies"][:, 2] > 0]
            present = result2["enemy_present"][i].astype(bool)
            np.testing.assert_array_equal(result2["enemy_hp"][i][present], rows[:, 1])
            np.testing.assert_array_equal(result2["enemy_max_hp"][i][present], rows[:, 2])

            def balance(player_hp, player_max, enemy_hp, enemy_max):
                total = float(enemy_max.sum()) or 1.0
                return max(player_hp, 0) / max(player_max, 1) - float(enemy_hp.clip(min=0).sum()) / total

            assert balance(int(result2["player_hp"][i]), int(result2["player_max_hp"][i]), result2["enemy_hp"][i][present], result2["enemy_max_hp"][i][present]) == \
                balance(int(observation["player"][0]), int(observation["player"][1]), rows[:, 1], rows[:, 2])
    finally:
        env.close()


def test_common_random_numbers_and_seeds():
    env = standard_env()
    try:
        search = searches(env)
        pair = [search.branch(5), search.branch(5)]
        search.playout_many(pair)
        assert pair[0].actions() == pair[1].actions()
        other = [search.branch(5), search.branch(6)]
        search.playout_many(other[:1], playout_seed=1)
        search.playout_many(other[1:])
        assert other[0].actions() != pair[0].actions() or other[1].actions() != pair[0].actions()
    finally:
        env.close()


def test_dtypes_shapes_and_token_updates():
    env = standard_env()
    try:
        search = searches(env, 2)
        branches = [search.branch(k) for k in SAMPLES]
        tokens = [b._token for b in branches]
        result = search.playout_many(branches, max_steps=3)
        n = len(branches)
        for key, dtype, shape in [("rewards", np.int32, (n,)), ("terminated", np.bool_, (n,)), ("truncated", np.bool_, (n,)),
                                  ("steps", np.uint64, (n,)), ("branch_steps", np.uint64, (n,)), ("player_hp", np.int32, (n,)),
                                  ("player_max_hp", np.int32, (n,)), ("_refusal", np.bool_, (n,)), ("_fault", np.bool_, (n,))]:
            assert result[key].dtype == np.dtype(dtype) and result[key].shape == shape, key
        width = result["enemy_hp"].shape[1]
        assert width >= 1 and result["enemy_max_hp"].shape == (n, width) and result["enemy_present"].dtype == np.dtype(np.int8)
        for key in ("end_reason", "refusal", "fault"):
            assert result[key].dtype == object and result[key].shape == (n,)
        assert all(b._token != t for b, t in zip(branches, tokens, strict=True))
        steps = [b.step(b.playout_action(), view="mask") for b in branches if b.decision()[1]["end_reason"] is None]
        assert steps, "updated tokens are current"
    finally:
        env.close()


def test_validation_rejects_bad_arguments_without_effect():
    env = standard_env()
    try:
        search = searches(env)
        other = searches(env)
        branch = search.branch(0)
        foreign = other.branch(0)
        before = search.counters()
        for kwargs in ({"max_steps": 0}, {"max_steps": -1}, {"max_steps": True}, {"max_steps": 1.0},
                       {"playout_seed": -1}, {"playout_seed": 2**64}, {"playout_seed": True},
                       {"exclude_verbs": "end_turn"}, {"exclude_verbs": b"end_turn"}, {"exclude_verbs": ["nope"]},
                       {"exclude_verbs": ["padding"]}, {"exclude_verbs": [1]}, {"exclude_verbs": {"end_turn"}},
                       {"rule": "uniform"}, {"rule": "greedy"}):
            with pytest.raises(_native.RunInputError):
                search.playout_many([branch], **kwargs)
        with pytest.raises(_native.RunInputError):
            branch.playout_action(exclude_verbs="end_turn")
        with pytest.raises(_native.RunInputError):
            search.playout_many([])
        with pytest.raises(_native.RunInputError):
            search.playout_many([branch, foreign])
        with pytest.raises(_native.RunInputError):
            search.playout_many([branch, branch])
        with pytest.raises(_native.RunInputError):
            search._native.playout_many([branch._branch_id], np.zeros((2, 2), dtype=np.uint64))
        assert search.counters() == before and branch.actions() == []
        result = search.playout_many([branch], exclude_verbs=(), max_steps=1)
        assert int(result["steps"][0]) == 1
    finally:
        env.close()


def test_budget_rejects_the_whole_call_without_effect():
    env = standard_env()
    try:
        search = env.public_search(RESEARCH_SEED, limits=dict(LIMITS, max_steps_per_branch=10, max_total_steps=25))
        branches = [search.branch(k) for k in (0, 1, 2)]
        before = search.counters()
        with pytest.raises(_native.RunBudgetError):
            search.playout_many(branches)
        assert search.counters() == before and all(b.actions() == [] for b in branches)
        result = search.playout_many(branches, max_steps=8)
        assert search.counters()["total_steps"] == int(result["steps"].sum())
    finally:
        env.close()


def test_identical_copy_grids_match_the_reference():
    """Seek grids over identical draw-pile copies (M40a picks) give the reference trajectories. An unresolvable pick is
    not reachable from a catalog entry; its refusal (`HIDDEN_ORDER_PICK`, no effect) is covered natively (M41 T9)."""
    seek_deck = [{"card": "seek", "upgrades": 1}] * 3 + [{"card": "strike_b", "upgrades": 0}] * 5 + [{"card": "defend_b", "upgrades": 0}] * 4
    catalog = entry()
    catalog["entries"][0].update(character="defect", deck=seek_deck, relics=["cracked_core"])
    env = StandardCombatEnv(catalog, partition="test")
    env.reset(seed=LIVE_SEED)
    try:
        search, reference = searches(env, 4), searches(env)
        samples = tuple(range(24))
        branches = [search.branch(k) for k in samples]
        result = search.playout_many(branches, exclude_verbs=("end_turn",))
        assert_matches(result, branches, step_reference(reference, samples, exclude_verbs=("end_turn",)))
        assert not result["_refusal"].any() and not result["_fault"].any()
    finally:
        env.close()


def test_fault_isolation_on_a_compact_capacity_overflow():
    env = standard_env(encounter="two_louse")
    try:
        # The root's action table fits exactly; a later hand with more targeted cards does not.
        probe = searches(env)
        width = int(probe.branch(0).decision()[0]["action_mask"].sum())
        tiny = searches(env, 2, observation="compact", capacities={"actions": width})
        branches = [tiny.branch(k) for k in SAMPLES]
        result = tiny.playout_many(branches, exclude_verbs=())
        faulted = np.flatnonzero(result["_fault"])
        assert faulted.size, "a hand wider than the action capacity faults its branch"
        for i in faulted:
            fault = result["fault"][i]
            assert fault["stage"] == "encode" and fault["committed"] and fault["capacity"]["group"] == "actions"
            assert result["end_reason"][i] == "fault" and int(result["tokens"][i, 0]) == 0
            assert int(result["player_gold"][i]) == 0
            assert not result["potion_kind"][i].any() and not result["potion_present"][i].any()
            with pytest.raises(_native.RunStateError):
                tiny.playout_many([branches[i]])
        assert tiny.counters()["total_steps"] == int(result["steps"].sum())
    finally:
        env.close()


@pytest.mark.parametrize("workers", [1, 4])
@pytest.mark.parametrize("layout", [{}, {"observation": "compact"}])
def test_tiered_playout_many_equals_one_action_at_a_time(workers, layout):
    env = standard_env(potions=["fire_potion", "block_potion", None])
    try:
        playout, mask_ref, full_ref = searches(env, workers, **layout), searches(env, 1, **layout), searches(env, 1, **layout)
        branches = [playout.branch(k) for k in SAMPLES]
        result = playout.playout_many(branches, rule="tiered_verbs")
        assert result["rule_id"] == "spire-gym-public-playout-tiered-v1"
        assert_matches(result, branches, step_reference(mask_ref, SAMPLES, view="mask", rule="tiered_verbs"))
        assert_matches(result, branches, step_reference(full_ref, SAMPLES, view="full", rule="tiered_verbs"))
        assert playout.counters() == mask_ref.counters()
    finally:
        env.close()


def test_tiered_rule_keeps_potions_out_and_falls_back():
    env = standard_env(potions=["fire_potion", "block_potion", None], gold=145)
    try:
        s_tiered = searches(env, 2)
        b_tiered = [s_tiered.branch(k) for k in SAMPLES]
        res_t = s_tiered.playout_many(b_tiered, rule="tiered_verbs")
        assert res_t["rule_id"] == "spire-gym-public-playout-tiered-v1"
        for i in range(len(b_tiered)):
            present = res_t["potion_present"][i].astype(bool)
            assert int(present.sum()) == 2, "tiered default keeps both potions untouched"
            assert int(res_t["player_gold"][i]) == 145
        # Fallback when only never_verbs are legal (put end_turn in never_verbs too, empty last_resort).
        s_fall = searches(env, 1)
        b_fall = s_fall.branch(0)
        res_f = s_fall.playout_many([b_fall], rule="tiered_verbs",
                                    never_verbs=("use_potion", "discard_potion", "end_turn"),
                                    last_resort_verbs=(), max_steps=1)
        assert int(res_f["steps"][0]) == 1
    finally:
        env.close()


def test_summary_potions_gold_and_terminal_cleanup_hp():
    env = standard_env(hp=50, max_hp=80, gold=123, encounter="cultist", potions=["smoke_bomb", "fairy_potion", None])
    try:
        search = searches(env, 2)
        branches = [search.branch(k) for k in SAMPLES]
        result = search.playout_many(branches, rule="tiered_verbs", max_steps=60)
        n = len(branches)
        assert result["player_gold"].dtype == np.dtype(np.uint32) and result["player_gold"].shape == (n,)
        p_width = result["potion_kind"].shape[1]
        assert p_width >= 3 and result["potion_kind"].dtype == np.dtype(np.int32)
        assert result["potion_present"].dtype == np.dtype(np.int8) and result["potion_present"].shape == (n, p_width)
        for i, branch in enumerate(branches):
            obs, info = branch.decision()
            assert int(result["player_gold"][i]) == int(obs["player"][2])
            occ = obs["potions_occupied"].astype(bool)
            present = result["potion_present"][i].astype(bool)
            np.testing.assert_array_equal(result["potion_kind"][i][present], obs["potions"][occ, 0])
            if result["terminated"][i]:
                assert int(obs["player_known"][11]) == 1
                assert int(result["player_hp"][i]) == int(obs["player"][11])
                assert int(result["player_hp"][i]) == int(info["hp_after_cleanup"])
        # Smoke Bomb escape: terminal cleanup HP includes Burning Blood (+6) and removes Smoke Bomb from potions.
        sb = search.branch(99)
        res_smoke = search.playout_many(
            [sb], rule="tiered_verbs",
            never_verbs=("play_card", "discard_potion", "end_turn"), last_resort_verbs=(), max_steps=5,
        )
        assert bool(res_smoke["terminated"][0]) and res_smoke["end_reason"][0] == "escaped"
        obs, info = sb.decision()
        assert int(res_smoke["player_hp"][0]) == 56 == int(obs["player"][11]) == int(info["hp_after_cleanup"])
        assert int(res_smoke["potion_present"][0].sum()) == 1
    finally:
        env.close()


def test_tiered_validation_and_mutual_exclusion():
    env = standard_env()
    try:
        search = searches(env)
        branch = search.branch(0)
        before = search.counters()
        bad_cases = (
            {"rule": "uniform_exclude_verbs", "never_verbs": ("use_potion",)},
            {"rule": "uniform_exclude_verbs", "last_resort_verbs": ("end_turn",)},
            {"rule": "tiered_verbs", "exclude_verbs": ("end_turn",)},
            {"rule": "tiered_verbs", "never_verbs": "use_potion"},
            {"rule": "tiered_verbs", "last_resort_verbs": b"end_turn"},
            {"rule": "tiered_verbs", "never_verbs": {"use_potion"}},
            {"rule": "tiered_verbs", "never_verbs": ("nope",)},
            {"rule": "tiered_verbs", "last_resort_verbs": ("padding",)},
            {"rule": "tiered_verbs", "never_verbs": ("use_potion", "end_turn"), "last_resort_verbs": ("end_turn",)},
            # Partial-default overlap: default last_resort_verbs is ("end_turn",).
            {"rule": "tiered_verbs", "never_verbs": ("end_turn",)},
            # Partial-default overlap: default never_verbs is ("use_potion", "discard_potion").
            {"rule": "tiered_verbs", "last_resort_verbs": ("use_potion",)},
        )
        for kwargs in bad_cases:
            with pytest.raises(_native.RunInputError):
                search.playout_many([branch], **kwargs)
            with pytest.raises(_native.RunInputError):
                branch.playout_action(**kwargs)
        assert search.counters() == before and branch.actions() == []
        # Independent None -> default with empty partner is valid.
        res = search.playout_many([branch], rule="tiered_verbs", never_verbs=(), max_steps=1)
        assert int(res["steps"][0]) == 1
    finally:
        env.close()
