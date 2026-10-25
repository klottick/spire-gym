"""M10 core/binding/Gym agreement on the accepted refreshed extension."""

import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

import numpy as np
import pytest

from spire_lab import _native
from spire_lab.gymnasium import RunEnv

ROOT = next(
    parent
    for parent in Path(__file__).resolve().parents
    if (parent / "Cargo.toml").is_file()
)
PROFILE = "m10-ironclad-heart"
OBSERVATION_ENCODING = "spire-gym-run-observation-canonical-json-v1"
EXAMPLE = ROOT / "examples" / "run_m10_gymnasium.py"


def _canonical_json(value):
    # Mirrors spire_env::canonical_json: recursively sorted object keys, compact
    # UTF-8 JSON, preserving all array order and explicit nulls.
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")


def _feature_tensors(spec):
    """Use descriptor order rather than guessing the native group table."""
    return [
        tensor
        for tensor in spec["tensors"]
        if tensor["key"] != "action_mask"
        and not tensor["key"].endswith("_known")
        and not tensor["key"].endswith("_occupied")
    ]


def _observation_payload(observation, spec):
    groups = []
    for tensor in _feature_tensors(spec):
        key = tensor["key"]
        occupied_key = f"{key}_occupied"
        groups.append(
            {
                "key": key,
                "shape": tensor["shape"],
                "values": observation[key].ravel().tolist(),
                "known": observation[f"{key}_known"].ravel().tolist(),
                "occupied": (
                    observation[occupied_key].ravel().tolist()
                    if occupied_key in observation
                    else None
                ),
            }
        )
    return {
        "groups": groups,
        "action_mask": observation["action_mask"].ravel().tolist(),
    }


def _observation_sha256(observation, spec):
    return hashlib.sha256(
        _canonical_json(_observation_payload(observation, spec))
    ).hexdigest()


def _assert_spec(spec, case):
    assert spec["schema_id"] == case["observation_schema_id"]
    assert spec["schema_id"] == "spire-gym-run-observation-v2"
    assert spec["profile_id"] == case["profile_id"] == "M10IroncladHeart"
    assert spec["capacities"] == case["capacities"]

    # The native exporter hashes the canonical descriptor before the Python
    # binding appends its fingerprint field. The raw fingerprint itself is not
    # SHA-256-hashed again.
    descriptor = {key: value for key, value in spec.items() if key != "fingerprint"}
    assert (
        hashlib.sha256(_canonical_json(descriptor)).hexdigest()
        == case["spec_descriptor_sha256"]
    )
    record_fingerprint = bytes(case["record"]["observation_fingerprint"]).hex()
    assert spec["fingerprint"] == case["spec_fingerprint"] == record_fingerprint
    assert case["spec_fingerprint_domain"] == r"spire-gym/run-schema/v2\0"


def _column(spec, group, field):
    tensor = next(row for row in spec["tensors"] if row["key"] == group)
    return tensor["fields"].index(field)


def _room_token(debug_name):
    if debug_name is None:
        return None
    # Native evidence uses the Rust variant's Debug name; room IDs are read
    # from the actual public observation descriptor vocabulary.
    variant = debug_name.split("(", 1)[0]
    return re.sub(r"(?<!^)(?=[A-Z])", "_", variant).lower()


def _assert_decision_projection(observation, spec, boundary):
    decision = boundary["decision"]
    context = observation["context"].ravel()
    vocabularies = spec["vocabularies"]
    assert int(context[_column(spec, "context", "character")]) == vocabularies[
        "character"
    ].index("ironclad")
    assert int(context[_column(spec, "context", "ascension")]) == 0
    assert int(context[_column(spec, "context", "content_profile")]) == vocabularies[
        "profile"
    ].index("M10IroncladHeart")
    assert int(context[_column(spec, "context", "act")]) == decision["act"]
    assert int(context[_column(spec, "context", "floor")]) == decision["floor"]
    assert (
        int(context[_column(spec, "context", "reward_total")])
        == decision["reward_total"]
    )
    assert (
        int(np.count_nonzero(observation["action_mask"])) == decision["candidate_count"]
    )

    room_id = int(context[_column(spec, "context", "room_kind")])
    room_name = _room_token(decision["room_kind"])
    if room_name is None:
        assert room_id == 0
    else:
        assert room_id == vocabularies["room"].index(room_name)

    terminal_id = int(context[_column(spec, "context", "terminal_kind")])
    terminal_name = decision["terminal"] or "ongoing"
    assert terminal_id == vocabularies["terminal"].index(terminal_name)


def _update_high_water(observation, spec, maxima):
    tensors = {tensor["key"]: tensor for tensor in _feature_tensors(spec)}
    for occupied_key in (key for key in observation if key.endswith("_occupied")):
        group = occupied_key[: -len("_occupied")]
        tensor = tensors[group]
        columns = tensor["shape"][-1]
        occupied_rows = int(np.count_nonzero(observation[occupied_key] == 1))
        known = observation[f"{group}_known"].reshape(-1, columns)
        known_rows = int(np.count_nonzero(np.any(known == 1, axis=1)))
        maxima.setdefault(
            group,
            {"occupied_rows": 0, "known_rows": 0, "m9_compatible_high_water": 0},
        )
        current = maxima[group]
        current["occupied_rows"] = max(current["occupied_rows"], occupied_rows)
        current["known_rows"] = max(current["known_rows"], known_rows)
        reported = known_rows if group == "potions" else occupied_rows
        current["m9_compatible_high_water"] = max(
            current["m9_compatible_high_water"], reported
        )


def _assert_boundary(
    observation, native_observation, spec, encoding_id, boundary, maxima
):
    assert set(observation) == set(native_observation)
    for key in observation:
        assert np.array_equal(observation[key], native_observation[key]), key

    assert encoding_id == OBSERVATION_ENCODING
    assert boundary["observation_encoding_id"] == encoding_id
    assert boundary["session_matches_core"] is True
    assert boundary["core_observation_sha256"] == boundary["observation_sha256"]
    assert _observation_sha256(observation, spec) == boundary["observation_sha256"]
    _assert_decision_projection(observation, spec, boundary)

    for tensor in spec["tensors"]:
        array = observation[tensor["key"]]
        assert array.shape == tuple(tensor["shape"]), tensor["key"]
        assert array.dtype == np.dtype(tensor["dtype"]), tensor["key"]
    _update_high_water(native_observation, spec, maxima)


@pytest.fixture(scope="module")
def native_cases():
    # The controller/test harness supplies the existing per-author absolute
    # Cargo target and TMPDIR. No new target or dependency setup is done here.
    environment = {
        **os.environ,
        "CARGO_BUILD_JOBS": os.environ.get("CARGO_BUILD_JOBS", "2"),
    }
    raw = subprocess.check_output(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "spire-env",
            "--example",
            "m10_gym_agreement",
            "--locked",
            "--offline",
        ],
        cwd=ROOT,
        env=environment,
    )
    evidence = json.loads(raw)
    assert "six public normal-start M10 episodes" in evidence["scope"]
    assert {(case["seed"], case["history"]) for case in evidence["cases"]} == {
        (seed, history)
        for seed in (7, 11)
        for history in (
            "awakened-unseen",
            "time-eater-unseen",
            "donu-deca-unseen",
        )
    }
    return evidence


def _run_example(*arguments):
    assert EXAMPLE.is_file(), f"missing M10 example: {EXAMPLE}"
    return subprocess.run(
        [sys.executable, str(EXAMPLE), *arguments],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def _legal_action_indices(observation, spec, verb, source_group=None, source_row=None):
    verbs = spec["vocabularies"]["verb"]
    groups = spec["vocabularies"]["row_group"]
    actions = observation["actions"]
    legal = np.flatnonzero(observation["action_mask"].reshape(-1))
    result = []
    for action_index in legal:
        row = actions[int(action_index)]
        if verbs[int(row[_column(spec, "actions", "verb")])] != verb:
            continue
        if (
            source_group is not None
            and groups[int(row[_column(spec, "actions", "source_group")])]
            != source_group
        ):
            continue
        if (
            source_row is not None
            and int(row[_column(spec, "actions", "source_row")]) != source_row
        ):
            continue
        result.append(int(action_index))
    return result


def _advance_m10_fixed_prefix(direct, game, case, action_count):
    spec = direct.observation_spec()
    assert spec == game.observation_spec()
    raw = direct.reset(seed=case["seed"])
    observation, _info = game.reset(seed=case["seed"])
    for key in observation:
        assert np.array_equal(observation[key], raw["obs"][key]), key
    assert (
        _observation_sha256(observation, spec)
        == case["boundaries"][0]["observation_sha256"]
    )

    for boundary_index, action in enumerate(case["indices"][:action_count]):
        assert observation["action_mask"][action] == 1
        native_step = direct.step(action, raw["token"])
        observation, reward, terminated, truncated, _info = game.step(action)
        raw = native_step
        assert reward == raw["reward"]
        assert terminated == raw["terminated"]
        assert truncated == raw["truncated"] is False
        for key in observation:
            assert np.array_equal(observation[key], raw["obs"][key]), key
        assert (
            _observation_sha256(observation, spec)
            == case["boundaries"][boundary_index + 1]["observation_sha256"]
        )
    return raw, observation


@pytest.mark.parametrize("case_index", range(6))
def test_m10_from_start_matches_native_binding_gym_and_replay(native_cases, case_index):
    assert len(native_cases["cases"]) == 6, "fixed seed7/11 × three-history evidence"
    case = native_cases["cases"][case_index]
    assert case["profile_id"] == "M10IroncladHeart"
    assert case["completion"] == "death"
    assert case["session_run_observed_through"] == "death"
    expected_prefix = {7: (177, 11, 10), 11: (110, 7, 6)}
    expected_steps, expected_floor, expected_reward = expected_prefix[case["seed"]]
    assert len(case["record"]["steps"]) == expected_steps
    assert case["boundaries"][-1]["decision"]["floor"] == expected_floor
    assert case["boundaries"][-1]["decision"]["reward_total"] == expected_reward
    assert case["supplied_task23_chain_included"] is False
    assert all(
        section in case["unobserved_sections"]
        for section in (
            "session-run Act 3 boss",
            "session-run Act 4 traversal",
            "session-run Heart combat",
        )
    )
    assert "heart_victory" not in [step["terminal"] for step in case["record"]["steps"]]

    direct = _native.NativeRunSession(
        "ironclad",
        0,
        partition="test",
        boss_history=case["history"],
        record_replay=True,
        profile=PROFILE,
    )
    game = RunEnv(
        "ironclad",
        0,
        partition="test",
        boss_history=case["history"],
        record_replay=True,
        profile=PROFILE,
    )
    try:
        native_spec = direct.observation_spec()
        spec = game.observation_spec()
        _assert_spec(native_spec, case)
        _assert_spec(spec, case)
        assert native_spec == spec

        observation_space = game.observation_space
        action_space = game.action_space
        space_spec = json.loads(json.dumps(spec))
        raw = direct.reset(seed=case["seed"])
        observation, info = game.reset(seed=case["seed"])
        assert raw["info"]["schema_id"] == info["schema_id"] == spec["schema_id"]
        expected_initial = case["boundaries"][0]
        assert expected_initial["incoming_reward"] is None
        assert expected_initial["incoming_terminal"] is None
        assert (
            raw["info"]["terminal_reason"]
            == info["terminal_reason"]
            == expected_initial["terminal"]
        )
        assert game.observation_space is observation_space
        assert game.action_space is action_space
        assert observation_space.contains(observation)
        initial_observation = observation
        initial_snapshot = {key: value.copy() for key, value in observation.items()}
        maxima = {}
        _assert_boundary(
            observation,
            raw["obs"],
            spec,
            native_cases["observation_encoding_id"],
            expected_initial,
            maxima,
        )

        steps = case["record"]["steps"]
        boundaries = case["boundaries"]
        assert len(case["indices"]) == len(steps)
        assert len(boundaries) == len(steps) + 1
        for index, (action, expected_step) in enumerate(
            zip(case["indices"], steps, strict=True)
        ):
            before = boundaries[index]
            after = boundaries[index + 1]
            assert before["offered_action_index"] == action
            assert before["offered_action"] == expected_step["action"]
            assert (
                observation["action_mask"][action]
                == raw["obs"]["action_mask"][action]
                == 1
            )

            native_step = direct.step(action, raw["token"])
            observation, reward, terminated, truncated, info = game.step(action)
            raw = native_step
            assert game.observation_space is observation_space
            assert game.action_space is action_space
            assert observation_space.contains(observation)
            _assert_boundary(
                observation,
                raw["obs"],
                spec,
                native_cases["observation_encoding_id"],
                after,
                maxima,
            )
            assert (
                reward
                == raw["reward"]
                == expected_step["reward"]
                == after["incoming_reward"]
            )
            assert terminated == raw["terminated"] == (after["terminal"] is not None)
            assert after["terminal"] == expected_step["terminal"]
            assert after["incoming_terminal"] == expected_step["terminal"]
            assert (
                not truncated and not raw["truncated"] and after["truncated"] is False
            )
            assert (
                info["terminal_reason"]
                == raw["info"]["terminal_reason"]
                == after["terminal"]
            )
            assert raw["info"]["schema_id"] == info["schema_id"] == spec["schema_id"]
            assert after["core_decision_matches_source_trace"] is True
            assert game.observation_spec() == space_spec

        # Spaces and previously returned arrays remain owned/immutable.
        assert game.observation_space is observation_space
        assert game.action_space is action_space
        assert all(
            np.array_equal(initial_observation[key], value)
            for key, value in initial_snapshot.items()
        )
        gym_record = game.record()
        native_record = direct.record()
        assert gym_record is not None
        assert native_record is not None
        assert gym_record == native_record
        assert json.loads(gym_record) == json.loads(native_record) == case["record"]
        assert hashlib.sha256(gym_record).hexdigest() == case["record_sha256"]

        observed_high_water = case["high_water"]
        assert set(maxima) == set(observed_high_water)
        for group, measured in maxima.items():
            reported = observed_high_water[group]
            assert measured["occupied_rows"] == reported["max_occupied_rows"]
            assert measured["known_rows"] == reported["max_known_rows"]
            assert (
                measured["m9_compatible_high_water"]
                == reported["m9_compatible_high_water"]
            )
            capacity_key = reported["capacity_key"]
            assert reported["capacity"] == spec["capacities"][capacity_key]
            assert reported["max_occupied_rows"] <= spec["capacities"][capacity_key]
            assert reported["max_known_rows"] <= spec["capacities"][capacity_key]
            assert (
                reported["m9_compatible_high_water"] <= spec["capacities"][capacity_key]
            )
        assert observed_high_water["known_draw"]["capacity_key"] == "combat_cards"
        assert observed_high_water["known_draw"]["max_occupied_rows"] == 0
        assert observed_high_water["known_draw"]["max_known_rows"] == 0

        replay = _native.NativeRunSession.replay(gym_record)
        assert (
            replay["accepted_count"] == case["replay"]["accepted_count"] == len(steps)
        )
        assert replay["terminal_reason"] == case["replay"]["final_terminal"] == "death"
        assert replay["rewards"] == case["replay"]["rewards"]
        assert replay["terminals"] == case["replay"]["terminals"]
        assert replay["rewards"] == [step["reward"] for step in steps]
        assert replay["terminals"] == [step["terminal"] for step in steps]
        assert case["replay"]["final_decision_matches_core"] is True
        assert case["replay"]["final_map_matches_core"] is True
        assert replay["reported_fault"] is None
        assert replay["final_decision_error"] is None
        assert (
            _observation_sha256(replay["obs"], spec)
            == case["replay"]["final_observation_sha256"]
        )
        assert (
            case["replay"]["final_observation_sha256"]
            == boundaries[-1]["observation_sha256"]
        )
        for key, value in observation.items():
            assert np.array_equal(replay["obs"][key], value), key
    finally:
        game.close()
        direct.close()


@pytest.mark.parametrize(
    ("arguments", "diagnostic"),
    [
        (("--budget", "0"), "unfinished"),
        (("--budget", "1"), "unfinished"),
        (("--budget", "8193"), "0..8192"),
        (("--profile", "invalid-profile", "--budget", "0"), None),
    ],
)
def test_m10_example_rejects_unfinished_budget_and_invalid_profile(
    arguments, diagnostic
):
    result = _run_example(*arguments)
    assert result.returncode != 0
    if diagnostic is not None:
        assert diagnostic in result.stdout + result.stderr


def test_m10_example_reports_a_real_nonattacking_death_as_success():
    result = _run_example(
        "--profile",
        PROFILE,
        "--character",
        "ironclad",
        "--ascension",
        "0",
        "--history",
        "awakened-unseen",
        "--seed",
        "11",
        "--budget",
        "8192",
        "--nonattacking",
    )
    assert result.returncode == 0, result.stderr
    assert "death:" in result.stdout


@pytest.mark.parametrize("seed", [7, 11])
def test_m10_example_runs_the_masked_caller_to_a_real_death(seed):
    result = _run_example(
        "--profile",
        PROFILE,
        "--seed",
        str(seed),
        "--budget",
        "8192",
    )
    assert result.returncode == 0, result.stderr
    assert re.fullmatch(r"death: \d+ actions\n?", result.stdout)


def test_m10_session_rejects_a_real_reset_capacity_overflow():
    # Two initial Neow candidates are present in the accepted native evidence;
    # actions=1 forces the public fixed-capacity boundary without injected state.
    session = _native.NativeRunSession(
        "ironclad",
        0,
        partition="test",
        boss_history="awakened-unseen",
        capacities={"actions": 1},
        record_replay=True,
        profile=PROFILE,
    )
    try:
        with pytest.raises(_native.RunFaultError, match="Run capacity actions"):
            session.reset(seed=7)
    finally:
        session.close()


def test_fixed_m10_prefix_opens_and_confirms_the_optional_elixir_selector(
    native_cases,
):
    # This bounded branch starts at the existing seed11/awakened prefix; all 62
    # parent actions remain the exact public actions from the normal-start trace.
    case = next(
        row
        for row in native_cases["cases"]
        if row["seed"] == 11 and row["history"] == "awakened-unseen"
    )
    prefix_actions = 62
    assert case["boundaries"][prefix_actions]["decision"]["floor"] == 5
    assert case["boundaries"][prefix_actions]["decision"]["room_kind"] == "Combat"
    direct = _native.NativeRunSession(
        "ironclad",
        0,
        partition="test",
        boss_history=case["history"],
        record_replay=True,
        profile=PROFILE,
    )
    game = RunEnv(
        "ironclad",
        0,
        partition="test",
        boss_history=case["history"],
        record_replay=True,
        profile=PROFILE,
    )
    try:
        raw, observation = _advance_m10_fixed_prefix(direct, game, case, prefix_actions)
        spec = direct.observation_spec()
        prefix_record = direct.record()
        assert prefix_record is not None
        assert (
            json.loads(prefix_record)["steps"]
            == case["record"]["steps"][:prefix_actions]
        )

        potion_kind = _column(spec, "potions", "potion_kind")
        elixir_id = spec["vocabularies"]["potion"].index("elixir")
        potion_occupied = observation["potions_occupied"].reshape(-1)
        elixir_rows = [
            int(row)
            for row in np.flatnonzero(potion_occupied == 1)
            if int(observation["potions"][int(row), potion_kind]) == elixir_id
        ]
        assert len(elixir_rows) == 1
        elixir_row = elixir_rows[0]
        use_actions = _legal_action_indices(
            observation, spec, "use_potion", "potions", elixir_row
        )
        assert len(use_actions) == 1
        use_action = use_actions[0]

        use_step = direct.step(use_action, raw["token"])
        observation, reward, terminated, truncated, info = game.step(use_action)
        assert reward == use_step["reward"] == 0
        assert not terminated and not use_step["terminated"]
        assert not truncated and not use_step["truncated"]
        assert info["schema_id"] == use_step["info"]["schema_id"] == spec["schema_id"]
        for key in observation:
            assert np.array_equal(observation[key], use_step["obs"][key]), key
        assert game.observation_space.contains(observation)

        # The live Elixir child exposes five eligible/legal card choices and no
        # selected cards. Its ConfirmSelection row is already legal at count zero.
        selection_columns = {
            field: _column(spec, "selection", field)
            for field in ("source_group", "source_row", "eligible", "selected")
        }
        occupied_selection = observation["selection_occupied"].reshape(-1) == 1
        selection_rows = observation["selection"][occupied_selection]
        eligible_rows = [
            row
            for row in selection_rows
            if int(row[selection_columns["eligible"]]) == 1
        ]
        assert len(eligible_rows) == 5
        assert all(
            int(row[selection_columns["selected"]]) == 0 for row in eligible_rows
        )
        combat_cards = spec["vocabularies"]["row_group"].index("combat_cards")
        eligible_card_rows = {
            int(row[selection_columns["source_row"]]) for row in eligible_rows
        }
        assert all(
            int(row[selection_columns["source_group"]]) == combat_cards
            for row in eligible_rows
        )
        count_fields = {
            field: _column(spec, "selection_counts", field)
            for field in (
                "required",
                "selected_count",
                "minimum",
                "maximum",
                "confirm_available",
                "operation",
            )
        }
        selection_counts = observation["selection_counts"].reshape(-1)
        assert np.all(observation["selection_counts_known"].reshape(-1) == 1)
        assert int(selection_counts[count_fields["required"]]) == 0
        assert int(selection_counts[count_fields["selected_count"]]) == 0
        assert int(selection_counts[count_fields["minimum"]]) == 0
        assert int(selection_counts[count_fields["maximum"]]) == 5
        assert int(selection_counts[count_fields["confirm_available"]]) == 1
        assert (
            spec["vocabularies"]["selection_operation"][
                int(selection_counts[count_fields["operation"]])
            ]
            == "move_exhaust"
        )
        select_actions = _legal_action_indices(
            observation, spec, "combat_select_card", "combat_cards"
        )
        assert len(select_actions) == 5
        assert {
            int(observation["actions"][action, _column(spec, "actions", "source_row")])
            for action in select_actions
        } == eligible_card_rows
        confirm_actions = _legal_action_indices(observation, spec, "confirm_selection")
        assert len(confirm_actions) == 1
        confirm_action = confirm_actions[0]
        assert observation["action_mask"][confirm_action] == 1

        after_use_potions = observation["potions"][:, potion_kind]
        after_use_occupied = observation["potions_occupied"].reshape(-1) == 1
        assert not np.any(after_use_potions[after_use_occupied] == elixir_id)
        use_record_bytes = direct.record()
        assert use_record_bytes is not None
        use_record = json.loads(use_record_bytes)
        assert len(use_record["steps"]) == prefix_actions + 1
        assert (
            use_record["steps"][:prefix_actions]
            == case["record"]["steps"][:prefix_actions]
        )
        assert use_record["steps"][-1]["action"]["verb"] == "combat.use_potion"
        assert use_record["steps"][-1]["action"]["slot"] == elixir_row

        confirm_step = direct.step(confirm_action, use_step["token"])
        observation, reward, terminated, truncated, info = game.step(confirm_action)
        assert reward == confirm_step["reward"] == 0
        assert not terminated and not confirm_step["terminated"]
        assert not truncated and not confirm_step["truncated"]
        for key in observation:
            assert np.array_equal(observation[key], confirm_step["obs"][key]), key
        assert game.observation_space.contains(observation)
        after_confirm_occupied = observation["potions_occupied"].reshape(-1) == 1
        assert not np.any(
            observation["potions"][:, potion_kind][after_confirm_occupied] == elixir_id
        )

        gym_record = game.record()
        native_record = direct.record()
        assert gym_record is not None and gym_record == native_record
        record = json.loads(gym_record)
        assert record["schema_id"] == "spire-gym-run-replay-v2"
        assert record["profile_id"] == "M10IroncladHeart"
        assert record["steps"][-1]["action"]["verb"] == "combat.confirm_selection"
        assert (
            sum(
                step["action"]["verb"] == "combat.use_potion"
                for step in record["steps"]
            )
            == 1
        )
        replay = _native.NativeRunSession.replay(gym_record)
        assert replay["accepted_count"] == prefix_actions + 2
        assert replay["reported_fault"] is None
        assert replay["final_decision_error"] is None
        assert all(
            np.array_equal(replay["obs"][key], observation[key]) for key in observation
        )
    finally:
        game.close()
        direct.close()


def test_fixed_m10_elixir_child_delivery_fault_retains_last_readable_selector(
    native_cases, monkeypatch
):
    class Unprintable(BaseException):
        def __str__(self):
            raise AssertionError("the original selector delivery error must not format")

    case = next(
        row
        for row in native_cases["cases"]
        if row["seed"] == 11 and row["history"] == "awakened-unseen"
    )
    prefix_actions = 62
    direct = _native.NativeRunSession(
        "ironclad",
        0,
        partition="test",
        boss_history=case["history"],
        record_replay=True,
        profile=PROFILE,
    )
    game = RunEnv(
        "ironclad",
        0,
        partition="test",
        boss_history=case["history"],
        record_replay=True,
        profile=PROFILE,
    )
    try:
        raw, observation = _advance_m10_fixed_prefix(direct, game, case, prefix_actions)
        spec = direct.observation_spec()
        potion_kind = _column(spec, "potions", "potion_kind")
        elixir_id = spec["vocabularies"]["potion"].index("elixir")
        occupied = observation["potions_occupied"].reshape(-1) == 1
        elixir_rows = [
            int(row)
            for row in np.flatnonzero(occupied)
            if int(observation["potions"][int(row), potion_kind]) == elixir_id
        ]
        assert len(elixir_rows) == 1
        use_actions = _legal_action_indices(
            observation, spec, "use_potion", "potions", elixir_rows[0]
        )
        assert len(use_actions) == 1
        use_action = use_actions[0]

        # A separate native session takes this same offered action normally so
        # the failed Gym packaging boundary has a public-array comparator.
        reference = direct.step(use_action, raw["token"])
        reference_child = reference["obs"]
        confirm_actions = _legal_action_indices(
            reference_child, spec, "confirm_selection"
        )
        assert len(confirm_actions) == 1

        error = Unprintable()
        captured = {}
        packaging_calls = 0

        def fail_on_selector(self, payload):
            nonlocal packaging_calls
            packaging_calls += 1
            assert self is game
            assert payload["info"]["schema_id"] == spec["schema_id"]
            captured["observation"] = {
                key: value.copy() for key, value in payload["obs"].items()
            }
            captured["token"] = payload["token"]
            assert (
                len(_legal_action_indices(payload["obs"], spec, "confirm_selection"))
                == 1
            )
            selection_counts = payload["obs"]["selection_counts"].reshape(-1)
            count_fields = {
                field: _column(spec, "selection_counts", field)
                for field in (
                    "minimum",
                    "maximum",
                    "selected_count",
                    "confirm_available",
                )
            }
            assert int(selection_counts[count_fields["minimum"]]) == 0
            assert int(selection_counts[count_fields["maximum"]]) == 5
            assert int(selection_counts[count_fields["selected_count"]]) == 0
            assert int(selection_counts[count_fields["confirm_available"]]) == 1
            with pytest.raises(_native.CombatStateError, match="busy") as nested:
                game.step(confirm_actions[0])
            assert nested.value.reason == "busy"
            raise error

        monkeypatch.setattr(RunEnv, "_package_step", fail_on_selector)
        with pytest.raises(Unprintable) as caught:
            game.step(use_action)
        assert caught.value is error
        assert packaging_calls == 1
        assert set(captured["observation"]) == set(reference_child)
        assert all(
            np.array_equal(captured["observation"][key], reference_child[key])
            for key in reference_child
        )

        encoded = game.record()
        assert encoded is not None
        record = json.loads(encoded)
        assert record["schema_id"] == "spire-gym-run-replay-v2"
        assert record["profile_id"] == "M10IroncladHeart"
        assert len(record["steps"]) == prefix_actions + 1
        assert (
            record["steps"][:prefix_actions] == case["record"]["steps"][:prefix_actions]
        )
        assert record["steps"][-1]["action"]["verb"] == "combat.use_potion"
        assert record["fault"]["error"]["kind"] == "delivery_fault"
        assert record["fault"]["committed"] is True
        assert record["fault"]["gameplay_recorded"] is True
        reproduction = game.reproduction()
        assert reproduction["record"] == encoded
        replay = _native.NativeRunSession.replay(encoded)
        assert replay["accepted_count"] == prefix_actions + 1
        assert all(
            np.array_equal(replay["obs"][key], captured["observation"][key])
            for key in reference_child
        )

        with pytest.raises(_native.RunFaultError):
            game.step(confirm_actions[0])
        assert packaging_calls == 1
        assert game.record() == encoded
    finally:
        game.close()
        direct.close()
