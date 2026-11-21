import hashlib
import json
import re

import pytest

from spire_lab import _native
from spire_lab.combat_entry_resources import SCHEMA_ID, load
from spire_lab.gymnasium import StandardCombatEnv


def test_resource_listing_fingerprint_and_ownership():
    resources = load()
    assert resources == _native.combat_entry_resources()
    assert resources["schema_id"] == SCHEMA_ID
    fingerprint = resources.pop("fingerprint")
    canonical = json.dumps(resources, sort_keys=True, separators=(",", ":")).encode()
    assert fingerprint == hashlib.sha256(b"spire-lab/combat-entry-resources/v1\0" + canonical).hexdigest()
    assert fingerprint == "4eafca3b0089e15735fb6aef24d594a084030438daa9738a1ac8cf6eab75dc5a"
    resources["characters"][0]["relics"].clear()
    assert load()["characters"][0]["relics"]


def test_resource_listing_builds_real_catalog_entry():
    resources = load()
    ironclad = next(row for row in resources["characters"] if row["character"] == "ironclad")
    assert "neows_lament" in ironclad["relics"]
    assert "fire_potion" in ironclad["potions"]
    lament = next(row for row in resources["relics"] if row["id"] == "neows_lament")
    assert lament["counter"]["rule"] == "required"
    entry = {
        "id": "probe", "character": ironclad["character"], "ascension": 0, "encounter": "jaw_worm",
        "hp": 50, "max_hp": 70, "gold": 0, "deck": [{"card": "strike", "upgrades": 0}],
        "relics": [lament["id"]],
        "potions": ["fire_potion", *([None] * (resources["potion_slots"]["base_by_ascension"][0] - 1))],
    }
    catalog = {
        "catalog_schema_id": resources["catalog_schema_id"],
        "profile_id": resources["profile_id"],
        "entries": [entry],
    }
    with pytest.raises(_native.RunInputError, match="relic_counters:"):
        StandardCombatEnv(catalog)
    entry["relic_counters"] = {lament["id"]: lament["counter"]["values"][-1][0]}
    env = StandardCombatEnv(catalog)
    try:
        assert env.environment_spec()["profile_id"] == resources["profile_id"]
        assert env.catalog_descriptor()["catalog_schema_id"] == resources["catalog_schema_id"]
        _, info = env.reset(seed=3, options={"entry_id": "probe"})
        assert info["entry_id"] == "probe"
    finally:
        env.close()


def _catalog(entry):
    resources = load()
    return {
        "catalog_schema_id": resources["catalog_schema_id"],
        "profile_id": resources["profile_id"],
        "entries": [entry],
    }


def _entry():
    return {
        "id": "probe", "character": "ironclad", "ascension": 0, "encounter": "jaw_worm",
        "hp": 50, "max_hp": 70, "gold": 0, "deck": [{"card": "strike", "upgrades": 0}],
        "relics": [], "potions": [None, None, None],
    }


@pytest.mark.parametrize(("updates", "missing", "field"), [
    ({"gold": -1}, (), "gold"),
    ({"gold": 4_294_967_296}, (), "gold"),
    ({"deck": [{"card": "strike", "upgrades": -1}]}, (), "deck.upgrades"),
    ({"relics": ["circlet"], "relic_counters": {"circlet": 2_147_483_648}}, (), "relic_counters"),
    ({}, ("hp",), "hp"),
    ({"potions": 3}, (), "potions"),
])
def test_known_entry_decode_errors_preserve_input_error(updates, missing, field):
    entry = _entry() | updates
    for key in missing:
        del entry[key]
    with pytest.raises(_native.RunInputError, match=rf"^{re.escape(field)}:"):
        StandardCombatEnv(_catalog(entry))


def test_catalog_entry_ids_match_published_length_pattern_and_uniqueness():
    rules = load()["bounds"]["entry_id"]
    minimum = rules["minimum_bytes"]
    maximum = rules["maximum_bytes"]
    candidates = (
        "a" * (minimum - 1), "a" * minimum, "a" * maximum, "a" * (maximum + 1),
        "aB0_-", "a.b", "a/b", "a b", "a\n", "aé", "Aß",
    )
    for candidate in candidates:
        book = _catalog(_entry() | {"id": candidate})
        allowed = minimum <= len(candidate.encode()) <= maximum and re.fullmatch(rules["pattern"], candidate)
        if allowed:
            with_env = StandardCombatEnv(book)
            try:
                assert with_env.catalog_descriptor()["entries"][0]["id"] == candidate
            finally:
                with_env.close()
        else:
            with pytest.raises(_native.RunInputError, match="invalid or duplicate combat entry ID"):
                StandardCombatEnv(book)

    book = _catalog(_entry())
    book["entries"].append(_entry())
    if rules["unique"]:
        with pytest.raises(_native.RunInputError, match="invalid or duplicate combat entry ID"):
            StandardCombatEnv(book)
    else:
        StandardCombatEnv(book).close()
