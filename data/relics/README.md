# All-relic correctness reference

Compiled 2026-10-23 for **Slay the Spire 1, PC v2.3**, all four fully
unlocked characters. This is reference data for future gym implementation,
**not a runtime registry, completed relic implementation, or game-fidelity proof**.
No simulator code, dependencies, original game files, or private evidence are included.

## Files and coverage

- [`relics.json`](relics.json): **180 identities** — the 178 registered relics
  in the existing public catalog, plus Circlet and Red Circlet.
- [`sources.json`](sources.json): revision-pinned wiki citations and qualified
  local-document references. Sources identify what they support.
- [`test_relic_reference_dataset.py`](../../tests/python/test_relic_reference_dataset.py):
  nine small offline integrity checks.

The roster includes 4 starter, 36 common, 36 uncommon, 34 rare, 30 boss,
20 shop, and 20 special/fallback identities. Character-exclusive totals are
Ironclad 11, Silent 11, Defect 9, Watcher 9. Event relics use the game's
`SPECIAL` tier, rather than the wiki's presentation-only `Event` label.
Circlet is repeatable fallback content. Red Circlet is recorded separately as
normally unobtainable, not silently inserted into ordinary offers. `INVALID`
is not a relic. Discerning Monocle is an obsolete external-catalog item and
is excluded from this target.

## Record fields

| Field | Meaning |
|---|---|
| `key` | Existing `relic.*` public catalog key; fallback keys are additions to this reference only |
| `game_id` | Exact game constructor ID, separate from the display name |
| `id` | Uppercase convenience label; **not** a Rust enum/discriminant |
| `name`, `tier`, `characters` | Display identity, game tier, normal acquisition restrictions |
| `availability`, `acquisition` | Normal/fallback/unobtainable status, source families, restrictions, starter replacement |
| `effect` | Readable summary; not byte-exact localization or an executable rule |
| `triggers`, `state_and_reset` | Semantic events and required conceptual state lifetime |
| `notes` | Correctness-sensitive distinctions and documented interactions |
| `rng` | Local/direct/indirect randomness and qualified outcome support |
| `local_static` | Existing local primary-rule finding, its grade and remaining boundaries, or `null` |
| `citations`, `uncertainties` | Source references and unresolved/conflicting claims |

Normal source families are not an exhaustive table of every event branch.
Combine them with the stated restrictions and global rules. A normal-character
restriction describes acquisition, not an invented rule disabling an off-color
relic's effect after nonstandard acquisition. The file is sorted by `key`,
**not execution order**.

For example, Ring of the Snake's existing key is `relic.snake_ring`, while
its game ID is `Ring of the Snake`. Paper Krane's constructor ID is
`Paper Crane`; Duality's is `Yang`. Do not derive exact IDs from display names.

## Evidence and conflicts

Reuse the existing [141-relic inventory](../../docs/research/2026-10-18-act1-relics-closure.md),
[primary mechanics report](../../docs/research/2026-10-19-act1-relic-mechanics.md), and
[178-registration catalog](../../docs/research/2026-10-19-act1-complete-catalog-numeric.md).
The 141 local primary findings retain the mechanics report's explicit
**supporting static, pending independent review** grade. Its pinned-archive,
Ironclad A0 Act 1 scope is **not promoted** to all-character, transitive,
full-run or original-runtime validation. The remaining records do not acquire
that evidence grade just because their identity appears in the catalog.

The retained local boundary codes are: `X` execution/queue/terminal composition,
`H` acquisition/continuations, `G` generation/history laws, `P` transitive payloads,
`V` public-information integration, and `R` a traced local resource rule. `R`
does not cancel the other limits. An empty `uncertainties` list is not a claim
that every possible interaction has been proved; global limits still apply.

Wiki revisions are frozen with retrieval dates, revision IDs, links, and content
hashes. They are current community documentation, **not version-pinned v2.3
runtime evidence**. The external Spire Archive catalog was used for discovery,
not trusted as authoritative: it has incomplete counter text, incorrect character
metadata, and an obsolete item. Short descriptions were normalized; long wiki
strategy/flavor sections and original code/assets are not imported.

Concrete differences are retained rather than flattened into tooltips:

- Strange Spoon prevention follows ordinary destination flags; it is not always
  discard. Actual exhaust, marked-to-exhaust cleanup, and manual discard differ.
- Golden Idol affects eligible stored gold rewards, not all `gainGold` calls.
- Ssserpent Head checks the resolved EventRoom, not every original map `?` icon.
- Meat on the Bone's local checkpoint precedes ordinary player victory hooks.
- Campfire relic floor/count predicates, N'loth's Hungry Face's reward removal,
  and the wiki's incorrect Hungry Face source field have explicit conflict notes.
- Mercury Hourglass's wiki-only floor cutoff is disputed by existing local
  spawn-predicate findings; it is an uncertainty, not an enforced restriction.
- Offering a relic/potion/card, selecting it, and actually obtaining it are
  different checkpoints. Tiny House, Calling Bell, bottles, eggs and Omamori
  must not become one universal synchronous-grant operation.

For 141-record conflicts, the retained local primary rule refines the public
summary **at that source's existing grade**. Remaining questions belong in focused
future implementation tests, not a new private-source audit or generic verifier.

## Randomness and visibility

The gym's approved [RNG-001](../../docs/intentional-deviations.md#rng-001--decorrelated-reproducible-project-randomness)
policy applies. Preserve conditional probabilities, candidate restrictions,
without-replacement behavior, history and deliberate dependencies. **Original
Java PRNG, seed mapping, call consumption and incidental stream coupling are not
acceptance requirements.** This dataset does not prove every conditional law.

The catalog itself is static public rules, not an observation encoder. Do not
expose future relic offers or hidden draw order merely because the reference
describes them. Frozen Eye and Runic Dome have real information-visibility rules.

## Reading and checking

From the repository root, using the configured Python 3.12 environment:

```sh
python3.12 -m unittest discover -s tests/python -p 'test_relic_reference_dataset.py' -v
```

The check file uses only the standard library and performs no network requests.
It checks roster/tier totals, unique identities, all existing catalog keys and
141 inventory IDs, required fields, citations, source qualifications, character
restrictions, replacements, fallbacks and the RNG-policy boundary. These are
**data integrity checks, not executed tests of 180 relic mechanics**.

```python
import json
from pathlib import Path

reference = json.loads(Path("data/relics/relics.json").read_text())
by_key = {relic["key"]: relic for relic in reference["relics"]}
print(by_key["relic.happy_flower"]["state_and_reset"])
```

Update records and their citations together. Preserve constructor IDs and
existing keys; record source disagreements and evidence grades rather than
silently replacing an unresolved finding. No task, milestone, or fidelity gate
is closed by compiling this reference.

## Attribution and licensing

Wiki-derived/adapted dataset content (`relics.json` and wiki attribution metadata
in `sources.json`) is provided under **CC BY-SA 4.0**, not the repository's blanket
MIT code license. Attribution: **Slay the Spire Wiki contributors**, the individual
revision-pinned pages in `sources.json`; each linked page's history identifies its
contributors. Changes include normalized text, structured metadata, selected
mechanics summaries, and reconciliation with local authored research.
License: <https://creativecommons.org/licenses/by-sa/4.0/>.
Existing local research and the small test remain under their existing project
terms. Mega Crit retains rights in Slay the Spire; no game code, binaries or assets
are licensed or distributed here.
