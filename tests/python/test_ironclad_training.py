"""Real native transport tests for standalone and manually reset Ironclad combat."""

import importlib
import json
from copy import deepcopy

import gymnasium as gym
import numpy as np
import pytest

from spire_lab import _native


def single_class():
    return importlib.import_module("spire_lab.gymnasium.ironclad").IroncladCombatEnv


def vector_class():
    return importlib.import_module(
        "spire_lab.gymnasium.ironclad_vector"
    ).IroncladCombatVectorEnv


def entry(
    entry_id, *, encounter="cultist", hp=80, deck=None, relics=None, potions=None
):
    return {
        "id": entry_id,
        "encounter": encounter,
        "hp": hp,
        "max_hp": max(hp, 80),
        "deck": deck or [{"card": "strike", "upgrades": 0}] * 10,
        "relics": relics or [],
        "potions": potions or [None, None, None],
        "relic_counters": {},
    }


def catalog(*entries):
    return {"entries": list(entries)}


def columns(spec, tensor):
    fields = next(item["fields"] for item in spec["tensors"] if item["key"] == tensor)
    return {name: i for i, name in enumerate(fields)}


def legal_action(observation, spec, verb, *, source_group=None, source_row=None):
    vocabulary = spec["vocabularies"]
    action_columns = columns(spec, "actions")
    verbs = vocabulary["verb"]
    groups = vocabulary["row_group"]
    for action in np.flatnonzero(observation["action_mask"].reshape(-1)):
        row = observation["actions"][int(action)]
        if verbs[int(row[action_columns["verb"]])] != verb:
            continue
        if (
            source_group is not None
            and groups[int(row[action_columns["source_group"]])] != source_group
        ):
            continue
        if (
            source_row is not None
            and int(row[action_columns["source_row"]]) != source_row
        ):
            continue
        return int(action)
    raise AssertionError(f"no legal {verb} action")


def assert_owned_equal(actual, expected):
    assert actual.keys() == expected.keys()
    for key in actual:
        np.testing.assert_array_equal(actual[key], expected[key])


def test_single_env_uses_catalog_entry_and_keeps_returned_arrays_owned():
    cls = single_class()
    from spire_lab import gymnasium as adapters

    assert adapters.IroncladCombatEnv is cls
    entries = catalog(
        entry("cultist"),
        entry("jaw", encounter="jaw_worm", hp=54),
    )
    env = cls(entries, partition="test")
    spec = env.observation_spec()
    assert spec["schema_id"] == "spire-gym-combat-policy-view-v1"
    assert spec["action_space"] == {"type": "Discrete", "n": 1024, "start": 0}
    assert env.observation_space.contains(env.reset(seed=17)[0])

    cultist_obs, cultist_info = env.reset(seed=17, options={"entry_id": "cultist"})
    jaw_obs, jaw_info = env.reset(seed=17, options={"entry_id": "jaw"})
    enemy_ids = spec["vocabularies"]["enemy"]
    enemy_column = columns(spec, "enemies")["enemy_kind"]
    cultist_id = enemy_ids.index("cultist")
    jaw_id = enemy_ids.index("jaw_worm")
    assert cultist_obs["enemies"][0, enemy_column] == cultist_id
    assert jaw_obs["enemies"][0, enemy_column] == jaw_id
    assert cultist_info["entry_id"] == "cultist"
    assert jaw_info["entry_id"] == "jaw"
    assert env.environment_spec()["profile_id"] == "ironclad-combat-v1"
    assert env.environment_spec()["reward_id"] == "spire-gym-combat-outcome-reward-v1"
    descriptor = env.catalog_descriptor()
    assert {row["id"] for row in descriptor["entries"]} == {"cultist", "jaw"}

    saved = {key: value.copy() for key, value in jaw_obs.items()}
    action = legal_action(jaw_obs, spec, "end_turn")
    env.step(action)
    env.close()
    assert_owned_equal(jaw_obs, saved)


@pytest.mark.parametrize(
    "bad_catalog",
    [
        {"entries": [dict(entry("ok"), hidden_state=1)]},
        {"entries": [dict(entry("ok"), hp=True)]},
        {"entries": [dict(entry("ok"), hp=float("nan"))]},
        {"entries": [dict(entry("ok"), deck=tuple(entry("ok")["deck"]))]},
        {"entries": [entry("same"), entry("same")]},
        {"entries": [dict(entry("ok"), encounter="silent_elite")]},
    ],
)
def test_single_catalog_rejects_hidden_or_invalid_configuration(bad_catalog):
    with pytest.raises(ValueError):
        single_class()(bad_catalog, partition="test")


def test_single_reset_validation_and_stale_tokens_are_recoverable():
    native = _native.NativeIroncladCombatSession(
        json.dumps(catalog(entry("base"))), partition="test"
    )
    first = native.reset(seed=19, entry_id="base")
    for seed, entry_id in ((True, "base"), (-1, "base"), (7, "missing")):
        with pytest.raises(ValueError):
            native.reset(seed=seed, entry_id=entry_id)
        current = native.decision()
        assert tuple(current["token"]) == tuple(first["token"])

    stale = (first["token"][0], first["token"][1] + 1)
    legal = int(np.flatnonzero(first["obs"]["action_mask"])[0])
    with pytest.raises((ValueError, RuntimeError)):
        native.step(legal, stale)
    accepted = native.step(legal, first["token"])
    assert accepted["info"]["active"] is True
    native.close()


def test_single_delivery_failure_retains_owner_and_disallows_retry():
    native = _native.NativeIroncladCombatSession(
        json.dumps(catalog(entry("base"))), partition="test"
    )

    def fail_after_reset(_raw):
        raise LookupError("python packaging failed")

    with pytest.raises(LookupError, match="packaging failed"):
        native._reset_deliver(fail_after_reset, seed=21, entry_id="base")
    diagnostic = native.diagnostic()
    assert diagnostic["fault"] is not None
    with pytest.raises(RuntimeError):
        native.reset(seed=21, entry_id="base")
    native.close()


def test_potion_selection_is_owned_by_combat_policy_and_consumes_the_potion():
    deck = [{"card": "strike", "upgrades": 0}] * 10
    env = single_class()(
        catalog(entry("elixir", deck=deck, potions=["elixir", None, None])),
        partition="test",
    )
    observation, _ = env.reset(seed=31, options={"entry_id": "elixir"})
    spec = env.observation_spec()
    potion_columns = columns(spec, "potions")
    potion_id = spec["vocabularies"]["potion"].index("elixir")
    row = next(
        int(i)
        for i in np.flatnonzero(observation["potions_occupied"].reshape(-1))
        if observation["potions"][int(i), potion_columns["potion_kind"]] == potion_id
    )
    action = legal_action(
        observation, spec, "use_potion", source_group="potions", source_row=row
    )
    observation, reward, terminated, truncated, info = env.step(action)
    assert reward == 0 and not terminated and not truncated
    assert info["routing"] == "combat_selection"
    assert info["selection_owner"] == "potion"
    assert not np.any(
        (observation["potions_occupied"].reshape(-1) == 1)
        & (observation["potions"][:, potion_columns["potion_kind"]] == potion_id)
    )
    selection_actions = [
        int(i)
        for i in np.flatnonzero(observation["action_mask"].reshape(-1))
        if spec["vocabularies"]["verb"][
            int(observation["actions"][int(i), columns(spec, "actions")["verb"]])
        ]
        == "combat_select_card"
    ]
    assert selection_actions
    env.close()


def test_single_natural_death_has_terminal_reward_and_checkpoint():
    env = single_class()(
        catalog(
            entry(
                "low",
                encounter="cultist",
                hp=1,
                deck=[{"card": "defend", "upgrades": 0}],
            )
        ),
        partition="test",
    )
    observation, _ = env.reset(seed=7)
    spec = env.observation_spec()
    for _ in range(64):
        action = legal_action(observation, spec, "end_turn")
        observation, reward, terminated, truncated, info = env.step(action)
        if terminated:
            break
    else:
        pytest.fail("one-HP Ironclad did not reach native Death")
    assert reward == -1 and terminated and not truncated
    assert info["last_exit"]["outcome"] == "death"
    assert info["last_exit"]["hp_before_healing"] == 0
    assert info["combats_settled"] == 1
    assert not observation["action_mask"].any()
    env.close()


def test_batch_matches_independent_seeds_and_manual_masked_reset_preserves_other_rows():
    entries = catalog(
        entry(
            "winner",
            deck=[{"card": "searing_blow", "upgrades": 10}] * 32,
        ),
        entry("survivor", hp=999, deck=[{"card": "defend", "upgrades": 0}] * 32),
    )
    vector = vector_class()(2, entries, partition="test")
    assert isinstance(vector, gym.vector.VectorEnv)
    assert vector.metadata["autoreset_mode"] is gym.vector.AutoresetMode.DISABLED
    observations, info = vector.reset(
        seed=[7, 8], options={"entry_ids": ["survivor", "winner"]}
    )
    assert vector.observation_space.contains(observations)
    assert [slot["entry_id"] for slot in info["slots"]] == ["survivor", "winner"]

    # Independently reset sessions with the same per-slot seeds have the same
    # public state; the vector binding must not call Python slots one at a time.
    singles = [single_class()(entries, partition="test") for _ in range(2)]
    for slot, seed in enumerate((7, 8)):
        separate, _ = singles[slot].reset(
            seed=seed, options={"entry_id": ("survivor", "winner")[slot]}
        )
        for key in observations:
            np.testing.assert_array_equal(observations[key][slot], separate[key])

    spec = vector.observation_spec()
    for _ in range(8):
        action0 = legal_action(observations_row(observations, 0), spec, "end_turn")
        action1 = legal_action(
            observations_row(observations, 1),
            spec,
            "play_card",
            source_group="combat_cards",
        )
        observations, rewards, terminated, truncated, info = vector.step(
            np.asarray([action0, action1], dtype=np.int64)
        )
        assert not truncated.any()
        if terminated[1]:
            break
    else:
        pytest.fail("high-damage entry did not reach native Victory")

    assert rewards[1] == 1 and terminated[1] and not terminated[0]
    assert info["slots"][1]["last_exit"]["outcome"] == "victory"
    assert not observations["action_mask"][1].any()
    ongoing_before = {key: value[0].copy() for key, value in observations.items()}
    ongoing_token = vector._tokens[0].copy()
    with pytest.raises((ValueError, RuntimeError)):
        vector.step(
            np.asarray(
                [
                    legal_action(observations_row(observations, 0), spec, "end_turn"),
                    0,
                ],
                dtype=np.int64,
            )
        )
    np.testing.assert_array_equal(vector._tokens[0], ongoing_token)
    for key, old in ongoing_before.items():
        np.testing.assert_array_equal(observations[key][0], old)

    survivor_before = {key: value[0].copy() for key, value in observations.items()}
    survivor_token = vector._tokens[0].copy()
    new_observations, reset_info = vector.reset(
        seed=[None, 99],
        options={
            "entry_ids": [None, "survivor"],
            "reset_mask": np.asarray([False, True], dtype=np.bool_),
        },
    )
    assert reset_info["slots"][1]["entry_id"] == "survivor"
    np.testing.assert_array_equal(vector._tokens[0], survivor_token)
    for key, old in survivor_before.items():
        np.testing.assert_array_equal(new_observations[key][0], old)
        np.testing.assert_array_equal(observations[key][0], old)
    for env in singles:
        env.close()
    vector.close()


def observations_row(observations, row):
    return {key: value[row] for key, value in observations.items()}


@pytest.mark.parametrize(
    "bad_actions",
    [
        np.zeros(2, dtype=np.int32),
        np.zeros((2, 1), dtype=np.int64),
        np.zeros(3, dtype=np.int64),
    ],
)
def test_batch_bad_action_arrays_are_preflighted_without_advancing_any_row(bad_actions):
    vector = vector_class()(2, catalog(entry("first")), partition="test")
    observations, _ = vector.reset(
        seed=[3, 4], options={"entry_ids": ["first", "first"]}
    )
    tokens = vector._tokens.copy()
    saved = {key: value.copy() for key, value in observations.items()}
    diagnostic = deepcopy(vector.diagnostic())
    with pytest.raises(_native.CombatInputError):
        vector.step(bad_actions)
    np.testing.assert_array_equal(vector._tokens, tokens)
    assert vector.diagnostic() == diagnostic
    for key, value in saved.items():
        np.testing.assert_array_equal(observations[key], value)
    vector.close()


def test_batch_reset_preflights_unselected_entry_ids_before_resetting_selected_rows():
    vector = vector_class()(
        2, catalog(entry("first"), entry("second")), partition="test"
    )
    observations, _ = vector.reset(
        seed=[3, 4], options={"entry_ids": ["first", "second"]}
    )
    tokens = vector._tokens.copy()
    diagnostic = deepcopy(vector.diagnostic())
    with pytest.raises(ValueError):
        vector.reset(
            seed=[9, 10],
            options={
                "entry_ids": ["second", "unknown"],
                "reset_mask": np.asarray([True, False], dtype=np.bool_),
            },
        )
    np.testing.assert_array_equal(vector._tokens, tokens)
    assert vector.diagnostic() == diagnostic

    current, _ = vector.reset(
        seed=[None, None],
        options={
            "entry_ids": [None, None],
            "reset_mask": np.asarray([False, False], dtype=np.bool_),
        },
    )
    for key in observations:
        np.testing.assert_array_equal(current[key], observations[key])
    vector.close()


@pytest.mark.parametrize(
    "reset_mask",
    [
        [True, False],
        np.ones(2, dtype=np.int8),
        np.ones((2, 1), dtype=np.bool_),
        np.ones(1, dtype=np.bool_),
    ],
)
def test_batch_reset_mask_requires_exact_native_bool_vector(reset_mask):
    vector = vector_class()(2, catalog(entry("first")), partition="test")
    vector.reset(seed=[3, 4], options={"entry_ids": ["first", "first"]})
    tokens = vector._tokens.copy()
    diagnostic = deepcopy(vector.diagnostic())
    with pytest.raises(ValueError):
        vector.reset(
            seed=[9, 10],
            options={"entry_ids": ["first", "first"], "reset_mask": reset_mask},
        )
    np.testing.assert_array_equal(vector._tokens, tokens)
    assert vector.diagnostic() == diagnostic
    vector.close()


def test_batch_stale_token_in_one_row_rejects_every_row_before_dispatch():
    native = _native.NativeIroncladCombatBatch(
        2, json.dumps(catalog(entry("first"))), partition="test"
    )
    initial = native.reset(
        seeds=[5, 6],
        entry_ids=["first", "first"],
        reset_mask=np.ones(2, dtype=np.bool_),
    )
    tokens = initial["tokens"].copy()
    tokens[1, 1] += 1
    actions = np.asarray(
        [
            int(np.flatnonzero(initial["obs"]["action_mask"][0])[0]),
            int(np.flatnonzero(initial["obs"]["action_mask"][1])[0]),
        ],
        dtype=np.int64,
    )
    before = native.diagnostic()
    with pytest.raises((ValueError, RuntimeError)):
        native.step(actions, tokens)
    assert native.diagnostic() == before
    native.close()


def test_batch_native_step_accepts_strided_readonly_integer_inputs():
    native = _native.NativeIroncladCombatBatch(
        2, json.dumps(catalog(entry("first"))), partition="test"
    )
    initial = native.reset(
        seeds=[5, 6],
        entry_ids=["first", "first"],
        reset_mask=np.ones(2, dtype=np.bool_),
    )
    legal = np.asarray(
        [
            int(np.flatnonzero(initial["obs"]["action_mask"][row])[0])
            for row in range(2)
        ],
        dtype=np.int64,
    )
    action_storage = np.zeros(4, dtype=np.int64)
    actions = action_storage[::2]
    actions[:] = legal
    token_storage = np.zeros((2, 4), dtype=np.uint64)
    tokens = token_storage[:, ::2]
    tokens[:] = initial["tokens"]
    actions.flags.writeable = False
    tokens.flags.writeable = False

    result = native.step(actions, tokens)
    assert result["tokens"].shape == (2, 2)
    assert result["terminated"].dtype == np.dtype(np.bool_)
    native.close()


def test_batch_native_step_rejects_non_native_endian_arrays_atomically():
    native = _native.NativeIroncladCombatBatch(
        2, json.dumps(catalog(entry("first"))), partition="test"
    )
    initial = native.reset(
        seeds=[5, 6],
        entry_ids=["first", "first"],
        reset_mask=np.ones(2, dtype=np.bool_),
    )
    actions = np.asarray(
        [
            int(np.flatnonzero(initial["obs"]["action_mask"][row])[0])
            for row in range(2)
        ],
        dtype=np.int64,
    )
    foreign_order_i64 = np.dtype(np.int64).newbyteorder("S")
    foreign_order_u64 = np.dtype(np.uint64).newbyteorder("S")
    bad_actions = actions.astype(foreign_order_i64)
    bad_tokens = initial["tokens"].astype(foreign_order_u64)
    assert not bad_actions.dtype.isnative and not bad_tokens.dtype.isnative
    before = native.diagnostic()
    with pytest.raises(_native.CombatInputError):
        native.step(bad_actions, initial["tokens"])
    with pytest.raises(_native.CombatInputError):
        native.step(actions, bad_tokens)
    assert native.diagnostic() == before
    native.step(actions, initial["tokens"])
    native.close()


def test_batch_package_failure_freezes_all_rows_and_reports_accepted_counts():
    native = _native.NativeIroncladCombatBatch(
        2, json.dumps(catalog(entry("first"))), partition="test"
    )
    initial = native.reset(
        seeds=[5, 6],
        entry_ids=["first", "first"],
        reset_mask=np.ones(2, dtype=np.bool_),
    )
    actions = np.asarray(
        [
            int(np.flatnonzero(initial["obs"]["action_mask"][0])[0]),
            int(np.flatnonzero(initial["obs"]["action_mask"][1])[0]),
        ],
        dtype=np.int64,
    )

    def fail_after_step(_raw):
        raise LookupError("batch packaging failed")

    with pytest.raises(LookupError, match="batch packaging failed"):
        native._step_deliver(actions, initial["tokens"], fail_after_step)
    diagnostic = native.diagnostic()
    assert diagnostic["status"] == "faulted"
    assert diagnostic["accepted_steps"] == [1, 1]
    with pytest.raises(RuntimeError):
        native.reset(
            seeds=[None, None],
            entry_ids=[None, None],
            reset_mask=np.ones(2, dtype=np.bool_),
        )
    native.close()
