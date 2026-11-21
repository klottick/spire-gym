# Standard combat entry resources (v1)

The opt-in `spire-lab-combat-entry-resources-v1` descriptor lists the resources
admitted by an owned `standard-combat-v1` entry. Read it before constructing a
catalog. The descriptor is static and requires no session.

- Rust uses `spire_env::combat_entry_resources_descriptor()` and
  `COMBAT_ENTRY_RESOURCES_SCHEMA_ID`.
- Python uses `spire_lab.combat_entry_resources.load()` or the native
  `spire_lab._native.combat_entry_resources()`. Each call returns an owned dictionary.

Match the descriptor's `profile_id` to the session's
`environment_spec()["profile_id"]` and its `catalog_schema_id` to the catalog
schema. Compare `schema_id` and `fingerprint` when pinning resource admission.
The fingerprint is lowercase hex SHA-256 over
`b"spire-lab/combat-entry-resources/v1\0"` followed by canonical JSON of every
other field. Canonical JSON uses UTF-8, recursively sorted object keys,
preserved array order and no extra whitespace. Apply `bounds.entry_id.pattern`
as a full match to the entire ID.

## Resource fields

| Field | Meaning |
| --- | --- |
| `characters` | One row per character with its catalog `relics` and `potions` IDs. Keep the caller's relic acquisition order in each entry. Each relic may appear once. Potion IDs may repeat in distinct slots. |
| `relics[].counter` | `null` forbids a key in `relic_counters`. Otherwise `rule` is `optional` or `required`, and `values` is a list of inclusive `[minimum, maximum]` intervals. `default: null` indicates no counter value is filled in. Rules that read an unset counter get zero. Supply a required key explicitly. |
| `relics[].companion` | `null` indicates no companion field. An owned bottle requires one key for its relic ID in `bottled_cards`. Use `null` for an empty bottle or a matching deck card row of the given `card_type` that is removable. A card instance may only be linked once. |
| `relics[].exclusive_with` | IDs that may not coexist with the row's relic, such as its starter or boss replacement. |
| `potion_slots` | `base_by_ascension` and `allowed` are indexed by ascension. Use exactly one count from `with_potion_belt` or `without_potion_belt`, depending on ownership. Without the Belt, an entry can retain the two extra slots from an earlier Belt. Use `null` for each empty slot. |
| `bounds` | Inclusive numeric and catalog limits. `cards` lists every admitted permanent card with its maximum upgrades and inclusive `misc` interval. The deck maximum includes cards generated at entry, such as Mark of Pain's Wounds. |

For example, Neow's Lament requires a counter of `-2` or `1..=3`. An optional
counter key can be omitted. When supplied, its value must be an integer in one
of the published intervals.

```python
from spire_lab.combat_entry_resources import load

resources = load()
silent = next(row for row in resources["characters"] if row["character"] == "silent")
ascension = 11
slot_counts = resources["potion_slots"]["allowed"][ascension]
empty_slots = [None] * slot_counts["without_potion_belt"][0]
assert "poison_potion" in silent["potions"]
```

The [Standard combat contract](standard-run-api.md#standard-combat-view-and-sessions)
covers encounter setup, burning elites and tactical starts. Catalog validation
checks the full combination and tries entry construction, including
encounter-specific and startup effects. An owned resource may have additional
conditions for use during combat. The descriptor covers fresh starts and the
owned resources of synthetic positions. Reachable recipe entries use the
resources of their validated record prefix.

Catalog decoding and resource validation report the field or resource class
for known inputs, including missing required values, incorrect types and
integers outside the field's range. For example, a negative catalog gold value
names `gold`, and an invalid card upgrade names `deck.upgrades`. Catch
`RunInputError` in Python. Malformed JSON and unknown fields can produce a generic
decode error. Diagnostics omit caller-supplied values and unknown field names.
