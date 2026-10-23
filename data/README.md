# Game-rule reference library

Start here before researching a mechanic. These offline references target
**unmodded Slay the Spire 1, PC v2.3**, across the four standard characters,
Acts 1–4 and Ascension 0–20 where applicable. They are implementation inputs,
**not an implemented-content registry, executable rules engine or game-fidelity
certificate**. Dataset coverage is broader than the simulator's accepted profiles.

## Find the relevant resource

| Domain | Start with | Data | Coverage / important limits |
|---|---|---|---|
| Cards | [Card guide](cards/README.md) | [cards.json](cards/cards.json) | 370 records: 367 physical kinds plus three Wish choices; printed quantities are not evaluated effects or complete scheduling rules |
| Relics | [Relic guide](relics/README.md) | [relics.json](relics/relics.json) | 180 identities, including two fallbacks; 141 local findings retain their supporting-static/pending-review grade |
| Enemies and encounters | [Enemy guide](enemies/README.md) | [enemies.json](enemies/enemies.json), [encounters.json](enemies/encounters.json) | 65 kinds, 196 moves, 67 encounter contexts; 27 constructor labels remain provisional after thirteen owned-static findings; nominal chances are not exact AI samplers |
| Powers / buffs / debuffs | [Power guide](powers/README.md) | [powers.json](powers/powers.json) | 133 records; 12 constructor IDs intentionally unresolved after fourteen scoped owned-static findings; recipients, technical classification and timing are distinct |

Each domain guide explains fields, citations, uncertainties, attribution and its
offline checks. Source snapshots/manifests live alongside their records; do not
derive constructor IDs from display names or confuse reference keys with Rust IDs.
Relics moved from `docs/research/relics/` to `data/relics/`; do not create a second
copy at the old path. Historical authored reports remain in
[`docs/research/`](../docs/research/) at their stated grades.

## How agents should use these references

1. Read the domain guide, the specific record, its citations and both local/global
   uncertainty notes. Reuse existing qualified facts instead of duplicating research.
2. Check the actual native handler and the relevant
   [accepted milestone profile](../docs/milestones/) to determine what is supported.
   A row, catalog key or passing reference test does not establish implementation.
3. Preserve source disagreements and evidence scope. Wiki revisions identify
   community claims, not the v2.3 executable; a secondary port is a cross-check,
   not an oracle. An empty uncertainty list does not cancel global qualifications.
4. For an actual implementation gap, resolve only the named mechanic needed by
   the approved task and add behavior-focused simulator tests. A reference handoff
   is not permission for a broad/private audit, new tooling, or new product work.
5. Preserve [RNG-001 and TIME-001](../docs/intentional-deviations.md): conditional
   probabilities, history restrictions and deliberate dependencies matter; original
   Java PRNG/seed/call-consumption parity is not required. Portal's time condition
   is assumed satisfied, without a clock/wait subsystem. Keep hidden future draws,
   queues, offers and AI state out of public observations.
6. Coordinate ownership before editing existing datasets. Update records and
   citations together; do not overwrite another worker's files or regenerate/fetch
   new sources merely to start a milestone.

The current product boundary is [M7's captured Ironclad A0 profile](../docs/milestones/m7-act2.md),
ending at nonterminal/no-action Act3Ready. M8 implementation is separately approved
and active; reference coverage alone does not establish native Act3 support.
Consult [AGENTS.md](../AGENTS.md) and the shared tracker for current ownership.

## Offline checks

From the repository root, using an existing Python 3.12 interpreter:

```sh
python3.12 -B -m unittest discover -s tests/python -p 'test_*reference*.py' -v
python3.12 -B data/cards/build_dataset.py --check
python3.12 -B data/powers/build_dataset.py --check
```

These standard-library checks require no network, pytest, installation or game
execution. They verify reference integrity, normalization and selected documented
distinctions—not simulated card/relic/enemy/power behavior. Per-domain commands
are in each guide. Do not silently regenerate data when `--check` reports a drift.

## Licensing

Wiki-derived/adapted datasets and supporting wiki snapshots have **CC BY-SA 4.0**
attribution and scope described in each domain guide; do not relabel them MIT.
The card identifier extract retains its separate upstream MIT notice. Project
builders/tests remain project code under the repository's MIT license. This does
not relicense the gym or grant rights to Mega Crit's game source, binaries or assets.
