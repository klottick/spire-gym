"""Project-owned known answers, replay, isolation and strict input boundaries."""

import copy
from dataclasses import FrozenInstanceError, replace
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

import pytest

import spire_lab.combat.rng as rng
from spire_lab.combat.rng import shuffle


M = 2**64
FIXTURE = Path(__file__).parents[1] / "fixtures/combat/project_rng_v1.json"
ROOT = bytes(range(32))


def vectors():
    return json.loads(FIXTURE.read_bytes())


def stream(row):
    return rng.make_stream(
        ROOT, row["partition"], row["episode_id"],
        rng.StreamId(row["domain"], tuple(row["scope"])),
    )


def take(state, count):
    values = []
    for _ in range(count):
        value, state = rng.draw_u64(state)
        values.append(value)
    return values, state


def test_fixture_identity_and_coverage():
    # Catches fixture refreshes, omissions, and contract/version drift.
    raw = FIXTURE.read_bytes()
    assert len(raw) == 8889
    assert hashlib.sha256(raw).hexdigest() == (
        "369d91e2ce3be38c191ef5ceab6fe6e1bf5c73f21504e1748074e9c35d962bad"
    )
    data = vectors()
    assert data["schema"] == "spire-lab-project-rng-fixtures-v1"
    assert data["contract_id"] == rng.RNG_CONTRACT_ID == "spire-lab-rng-hmac-sha256-v1"
    assert data["root_seed_hex"] == ROOT.hex()
    assert [row["id"] for row in data["primitive_cases"]] == ["rfc4231-1", "rfc4231-2"]
    assert [row["id"] for row in data["streams"]] == [
        "shuffle-train", "hp-train", "ai-train", "ai-enemy2", "ai-final",
    ]
    assert sum(len(row["blocks"]) for row in data["streams"]) == 15
    assert sum(len(row["words"]) for row in data["streams"]) == 45
    for row in data["streams"]:
        assert [int(word["index"]) for word in row["words"]] == [*range(8), M - 1]
        assert set(row["blocks"]) == {"0", "1", str((M - 1) // 4)}


def test_public_hmac_known_answers():
    # Catches a wrong digest/argument order or an inappropriate 32-byte restriction.
    for row in vectors()["primitive_cases"]:
        assert rng._hmac_sha256(
            bytes.fromhex(row["key_hex"]), bytes.fromhex(row["message_hex"])
        ).hex() == row["digest_hex"]


def test_framing_is_unambiguous():
    first = rng._frame_fields(("a", "bc"))
    second = rng._frame_fields(("ab", "c"))
    assert first.hex() == "000000020000000161000000026263"
    assert second.hex() == "000000020000000261620000000163"
    assert first != second


def test_project_stream_and_word_vectors():
    # Catches prefixes, field order, endian, block counter, and lane selection bugs.
    for row in vectors()["streams"]:
        initial = stream(row)
        assert initial.word_index == 0
        assert initial.stream_id == rng.StreamId(row["domain"], tuple(row["scope"]))
        assert initial.key.hex() == row["key_hex"]
        assert rng._frame_fields((
            row["partition"], row["episode_id"], row["domain"], *row["scope"],
        )).hex() == row["frame_hex"]
        for index, digest in row["blocks"].items():
            assert rng._hmac_sha256(
                initial.key, b"spire-lab/rng/v1/block\0" + int(index).to_bytes(8, "big")
            ).hex() == digest
        for word in row["words"]:
            index = int(word["index"])
            saved = replace(initial, word_index=index)
            before = copy.deepcopy(saved)
            value, advanced = rng.draw_u64(saved)
            assert value == int(word["hex"], 16)
            assert type(value) is int and 0 <= value < M
            assert advanced == replace(saved, word_index=index + 1)
            assert saved == before


def test_replay_from_each_lane():
    # Catches hidden caches or state not represented by the explicit cursor.
    for row in vectors()["streams"]:
        expected = [int(word["hex"], 16) for word in row["words"][:8]]
        for lane in range(4):
            saved = replace(stream(row), word_index=lane)
            first = take(saved, 8 - lane)
            assert first[0] == expected[lane:]
            assert first[1].word_index == 8
            assert take(saved, 8 - lane) == first
            assert take(copy.deepcopy(saved), 8 - lane) == first
            assert saved.word_index == lane


def test_project_composed_replay():
    # Catches changed sampler composition, consumption, or hidden mutable replay state.
    data = vectors()
    row = next(row for row in data["streams"] if row["id"] == "shuffle-train")
    saved = stream(row)
    restored = copy.deepcopy(saved)
    integration = data["integration"]
    assert integration == {
        "stream_id": "shuffle-train",
        "operations": ["uniform_below(10)", "bernoulli(2,5)",
                       "weighted_index((0,2,0,3))", "shuffle(('a','b','c'))"],
        "values": [9, False, 1, ["c", "b", "a"]],
        "next_word_index": "5",
    }
    results = []
    for state in (saved, saved, restored):
        values = []
        value, state = rng.uniform_below(state, 10)
        values.append(value)
        value, state = rng.bernoulli(state, 2, 5)
        assert value is False
        values.append(value)
        value, state = rng.weighted_index(state, (0, 2, 0, 3))
        values.append(value)
        value, state = shuffle(state, ("a", "b", "c"))
        values.append(value)
        assert values == [9, False, 1, ["c", "b", "a"]] == integration["values"]
        assert state == replace(saved, word_index=5)
        assert state.word_index == int(integration["next_word_index"])
        results.append((values, state))
    assert results[0] == results[1] == results[2]
    assert saved == restored and saved.word_index == 0


def test_interleaving_preserves_other_streams():
    # Catches a shared generator/cursor between domains or enemies.
    rows = {row["id"]: row for row in vectors()["streams"]}
    saved = {name: stream(rows[name]) for name in rows}
    before = copy.deepcopy(saved)
    for schedule in [("hp-train", "ai-enemy2") * 5, ("ai-enemy2",) * 5 + ("hp-train",) * 5]:
        states = saved.copy()
        for name in schedule:
            _, states[name] = rng.draw_u64(states[name])
        for name in ("shuffle-train", "ai-train"):
            values, final = take(states[name], 8)
            assert values == [int(word["hex"], 16) for word in rows[name]["words"][:8]]
            assert final.word_index == 8
        assert saved == before


def test_identity_dimensions_and_creation_order(monkeypatch):
    # Catches omitted identity fields or using root bytes as framed identity.
    rows = vectors()["streams"]
    forward = {row["id"]: stream(row) for row in rows}
    reverse = {row["id"]: stream(row) for row in reversed(rows)}
    assert forward == reverse
    for name in forward:
        assert take(forward[name], 8) == take(reverse[name], 8)

    calls = []
    real_hmac = rng._hmac_sha256

    def capture(key, message):
        calls.append((key, message))
        return real_hmac(key, message)

    monkeypatch.setattr(rng, "_hmac_sha256", capture)
    identity = rng.StreamId("combat.enemy.ai", ("combat-1", "enemy-1"))
    baseline = rng.make_stream(ROOT, "train", "episode-1", identity)
    base_args = calls[-1]
    base_frame = bytes.fromhex(rows[2]["frame_hex"])
    assert base_args == (ROOT, b"spire-lab/rng/v1/stream\0" + base_frame)
    variants = [
        ("train", "episode-2", identity, base_frame.replace(b"episode-1", b"episode-2")),
        ("train", "episode-1", rng.StreamId(identity.domain, ("combat-2", "enemy-1")),
         base_frame.replace(b"combat-1", b"combat-2")),
        ("train", "episode-1", rng.StreamId(identity.domain, ("combat-1", "enemy-2")),
         base_frame.replace(b"enemy-1", b"enemy-2")),
        ("train", "episode-1", rng.StreamId("combat.enemy.hp", identity.scope),
         base_frame.replace(b"combat.enemy.ai", b"combat.enemy.hp")),
    ]
    for partition in ("train", "selection", "final", "test"):
        # Replace the fixed first field only, including its BE32 length.
        frame = base_frame[:4] + len(partition).to_bytes(4, "big") + partition.encode() + base_frame[13:]
        variants.append((partition, "episode-1", identity, frame))
    for partition, episode, sid, expected_frame in variants:
        rng.make_stream(ROOT, partition, episode, sid)
        assert calls[-1] == (ROOT, b"spire-lab/rng/v1/stream\0" + expected_frame)
        assert (expected_frame == base_frame) == (partition == "train" and episode == "episode-1" and sid == identity)
    rng.make_stream(ROOT[::-1], "train", "episode-1", identity)
    assert calls[-1] == (ROOT[::-1], base_args[1])
    assert rng.make_stream(ROOT, "train", "episode-1", identity) == baseline
    # Output inequality is deliberately not used as an independence proof.


def test_last_word_and_exhaustion():
    for row in vectors()["streams"]:
        saved = replace(stream(row), word_index=M - 1)
        value, exhausted = rng.draw_u64(saved)
        assert value == int(row["words"][-1]["hex"], 16)
        assert exhausted == replace(saved, word_index=M)
        with pytest.raises(rng.RngExhaustedError) as caught:
            rng.draw_u64(exhausted)
        assert caught.value.state == exhausted
        assert caught.value.state is exhausted
        assert saved.word_index == M - 1
        assert exhausted.word_index == M


def test_values_are_frozen_and_owned():
    saved = stream(vectors()["streams"][2])
    for obj, field, value in [
        (saved.stream_id, "domain", "combat.enemy.hp"),
        (saved.stream_id, "scope", ("other", "enemy")),
        (saved, "stream_id", rng.StreamId("combat.shuffle", ("other",))),
        (saved, "key", b"x" * 32), (saved, "word_index", 1),
    ]:
        with pytest.raises(FrozenInstanceError):
            setattr(obj, field, value)
    assert type(saved.key) is bytes
    assert type(saved.stream_id.scope) is tuple
    with pytest.raises(TypeError):
        saved.key[0] = 1
    with pytest.raises(TypeError):
        saved.stream_id.scope[0] = "other"
    _, advanced = rng.draw_u64(saved)
    assert advanced is not saved
    assert saved == stream(vectors()["streams"][2])
    assert advanced.word_index == 1 and saved.word_index == 0


def test_import_and_hash_seed_independence():
    # Catches accidental native/tensor imports and randomized hash derivation.
    script = '''
import json, sys
from spire_lab.combat.rng import StreamId, make_stream, draw_u64
state = make_stream(bytes(range(32)), "train", "episode-1", StreamId("combat.shuffle", ("combat-1",)))
values = []
for _ in range(8):
    value, state = draw_u64(state)
    values.append(value)
assert not any(name == prefix or name.startswith(prefix + ".")
               for name in sys.modules for prefix in ("numpy", "torch", "spire_lab._native"))
print(json.dumps(values))
'''
    expected = [int(word["hex"], 16) for word in vectors()["streams"][0]["words"][:8]]
    repository = Path(__file__).resolve().parents[3]
    for seed in ("1", "2"):
        env = {**os.environ, "PYTHONHASHSEED": seed, "PYTHONPATH": str(repository / "python")}
        result = subprocess.run([sys.executable, "-B", "-c", script], env=env,
                                cwd=repository, capture_output=True, text=True, check=True)
        assert result.stderr == ""
        assert json.loads(result.stdout) == expected


class BytesSubclass(bytes):
    pass


class StrSubclass(str):
    pass


class TupleSubclass(tuple):
    pass


class IntSubclass(int):
    pass


class StreamIdSubclass(rng.StreamId):
    pass


class StateSubclass(rng.RandomStreamState):
    pass


def validation_cases():
    cases = []
    def add(boundary, values, error):
        cases.extend(pytest.param(boundary, value, error, id=f"{boundary}-{value!r}") for value in values)

    for boundary in ("root", "key"):
        add(boundary, [bytearray(32), 32, BytesSubclass(b"x" * 32)], TypeError)
        add(boundary, [b"x" * 31, b"x" * 33], ValueError)
        add(boundary, [b"x" * 32], None)
    for boundary in ("episode", "scope-field", "domain"):
        add(boundary, [1, StrSubclass("combat.shuffle")], TypeError)
        add(boundary, ["", " ", "\0", "\x7f", "é", "x" * 256], ValueError)
    for boundary in ("episode", "scope-field"):
        add(boundary, ["!", "~", "x" * 255], None)
    add("domain", ["unknown"], ValueError)
    add("scope", [["combat-1"], TupleSubclass(("combat-1",))], TypeError)
    add("identity", [None, StreamIdSubclass("combat.shuffle", ("combat-1",))], TypeError)
    add("partition", [1, StrSubclass("train")], TypeError)
    add("partition", ["Train", "unknown"], ValueError)
    add("partition", ["train", "selection", "final", "test"], None)
    add("cursor", [True, 1.0, "0", IntSubclass(0)], TypeError)
    add("cursor", [-1, M + 1], ValueError)
    add("cursor", [0, M - 1, M], None)
    for domain, arity in [("combat.shuffle", 1), ("combat.enemy.hp", 2), ("combat.enemy.ai", 2)]:
        for size in (arity - 1, arity, arity + 1):
            cases.append(pytest.param("arity", (domain, ("id",) * size),
                                      None if size == arity else ValueError,
                                      id=f"{domain}-arity{size}"))
    add("operation", [None], TypeError)
    return cases


@pytest.mark.parametrize("boundary,value,error", validation_cases())
def test_identity_and_state_validation(boundary, value, error, monkeypatch):
    # Each case catches accepting a malformed owned value or rejecting a valid edge.
    sid = rng.StreamId("combat.shuffle", ("combat-1",))
    if error is not None:
        def forbidden(*args):
            pytest.fail("invalid input reached HMAC derivation")
        monkeypatch.setattr(rng, "_hmac_sha256", forbidden)

    def operation():
        if boundary == "root":
            return rng.make_stream(value, "train", "episode-1", sid)
        if boundary == "partition":
            return rng.make_stream(ROOT, value, "episode-1", sid)
        if boundary == "episode":
            return rng.make_stream(ROOT, "train", value, sid)
        if boundary == "identity":
            return rng.make_stream(ROOT, "train", "episode-1", value)
        if boundary == "domain":
            return rng.StreamId(value, ("combat-1",))
        if boundary == "scope":
            return rng.StreamId("combat.shuffle", value)
        if boundary == "scope-field":
            return rng.StreamId("combat.shuffle", (value,))
        if boundary == "arity":
            return rng.StreamId(*value)
        if boundary == "key":
            return rng.RandomStreamState(sid, value, 0)
        if boundary == "cursor":
            return rng.RandomStreamState(sid, ROOT, value)
        if boundary == "operation":
            return rng.draw_u64(value)
        raise AssertionError(boundary)

    if error is None:
        operation()
    else:
        with pytest.raises(error):
            operation()


@pytest.mark.parametrize("value", [None, StreamIdSubclass("combat.shuffle", ("combat-1",))])
def test_state_rejects_foreign_identity(value):
    with pytest.raises(TypeError):
        rng.RandomStreamState(value, ROOT, 0)


def test_draw_rejects_state_subclass():
    state = StateSubclass(rng.StreamId("combat.shuffle", ("combat-1",)), ROOT, 0)
    with pytest.raises(TypeError):
        rng.draw_u64(state)
