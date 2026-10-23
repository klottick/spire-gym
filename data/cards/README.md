# All-character card reference

**Target:** unmodded Slay the Spire 1, PC v2.3. This is an offline implementation
reference for Spire Gym, **not** a runtime registry, implemented-card list, or
original-runtime validation result. No simulator code is changed by this dataset.

## Coverage and files

[`cards.json`](cards.json) contains **370 records**: 75 each for Ironclad, Silent,
Defect and Watcher, plus 70 shared records. There are 367 physical card kinds and
three Wish choice-only records. Character-specific Strikes and Defends remain
distinct. Every row in the retrieved StS1 wiki table is represented, including
event/generated cards, statuses, curses, Ascender's Bane, Burn+, and Pride.
Pride is explicitly marked outside standard runs (Endless mode), not quietly
included in the standard-run reward pool.

| File | Purpose |
|---|---|
| `cards.json` | Generated, self-contained reference records |
| `implementation-notes.json` | Selected independently worded, wiki-page-grounded edge cases |
| `research-annotations.json` | Selected notes/discrepancies from existing local research |
| `sources/wiki-cards.json` | Retrieved StS1 Cargo rows, including original combined upgrade text |
| `sources/wiki-pages.json` | Supporting public page wikitext with revision IDs/timestamps; includes detailed interactions and known bugs |
| `sources/wiki-retrieval.json` | Retrieval time, exact query, and non-transactional fetch qualification |
| `sources/identifiers.json` | Attributed reference-ID/game-string-ID mapping only |
| `build_dataset.py` | Small offline data normalizer; no game execution, compiler or dependencies |

## Record interpretation

- `id`: stable uppercase reference key, **not** a Rust enum or numeric registry ID.
  `game_id` preserves historical game/save spellings such as `Venomology`
  (Alchemize), `Gash` (Claw), and `PathToVictory` (Pressure Points). It comes from
  the pinned public identifier source, not a runtime save-file experiment.
- `character`: card family, or `null` for shared cards. Off-color acquisition does
  not change the actual run character or grant that character's starting resources.
- `color`, `type`, `rarity`: reference classification. Curse color is normalized
  to `CURSE`; the wiki's original color/rarity are retained under `source`.
  Wiki `STATUS` rarity and its catch-all `CURSE` label are display taxonomy,
  **not** a claimed conversion to the game's CardRarity enum. Local research
  distinguishes COMMON statuses and SPECIAL curses where appropriate.
- `base` / `upgraded`: fresh printed variants. Costs use `-1` for X and `-2` for
  unplayable/choice-only displays; `[E]` means one energy. There is no `-3` sentinel.
  `upgraded: null` means no upgraded variant, not an unknown upgrade.
- `printed_damage` / `printed_block`: first literal `Deal N damage` / `Gain N Block`
  found in that variant. These can be conditional or triggered quantities, **not**
  evaluated damage, total multi-hit damage, a canonical baseDamage/baseBlock field,
  or a full list of effect parameters. `null` means no such literal was extracted;
  it does not mean zero. Read the complete `description` for the effect.
- `requires_enemy_target`: a player-selected enemy is required, not a complete
  legality rule or original target enum. AoE/random attacks do not require one;
  mixed effects such as Leg Sweep and Pressure Points do. Card selection remains a
  separate choice. Blind/Trip change this requirement on upgrade.
- `flags`: intrinsic self Innate, Ethereal, Exhaust, Retain and Unplayable, not all
  keyword mentions. True Grit exhausts another card, not itself; Powers leaving
  play is not Exhaust. Deus Ex Machina's self-exhaust happens **on draw**; the flag
  alone never establishes trigger timing. External modifiers can change behavior.
- `keywords`: terms mentioned in the rendered text, including effects on other
  cards or enemies; unlike `flags`, this is not a list of intrinsic properties.
- `upgrade.mode`: `standard`, `none`, `repeatable` (Searing Blow), `enemy_only`
  (Burn+), or `parent_choice` (Wish options). Searing Blow's upgraded record is
  **+1 only**; its integer-count formula is retained separately. Upgrading a parent
  generally does not upgrade a generated child. Expunger's cost is 1 and its X is
  a stored hit count, not an X energy cost.
- `availability`: a broad acquisition category, not pool membership rules,
  generator filters, encounter eligibility, or probabilities. Ascender's Bane's
  A10 threshold and Pride's non-standard-mode status are explicit.
- `related_cards`: cards referenced by the primary effect, plus Wish's options.
  These links do not all mean “generates,” and do not encode a complete interaction
  graph. Some English-only references may not be tagged by the source wiki.
- `implementation_notes`: selected wiki-page-grounded implementation traps, cited
  by the card's `source`. `research_notes` and `discrepancies` refer to the linked
  local draft. Empty lists do **not** mean a card has no subtleties or that every
  source agrees; this is not an exhaustive mechanics/conflict audit.

## Sources, qualifications and discrepancies

Primary pairs come from the [StS1 Cards table](https://slaythespire.wiki.gg/wiki/Cards_List)
and [Cargo schema](https://slaythespire.wiki.gg/wiki/Special:CargoTables/Cards).
The query explicitly filters `Game='1'`, excluding StS2 and the board game.
Each record links its page and permanent revision; the associated raw page is in
the snapshot. Some specials share a parent page, explaining 370 records across
362 pages. Cargo and page text are separate requests, **not a transactional
snapshot of one common revision**. PC v2.3 is the target, not a binary-verified
version assertion. Community sources can be wrong.

The identity-only map is extracted from
[`gamerpuppy/sts_lightspeed` at `7476a81954020087da31d41d16fddf475746ec2d`](https://github.com/gamerpuppy/sts_lightspeed/blob/7476a81954020087da31d41d16fddf475746ec2d/include/constants/Cards.h).
Its costs, colors, flags and combat implementations are **not** imported as a
correctness authority or confused with Spire Gym's registry.

Existing local research is reused without re-running private source work:

- [Ironclad/shared/status/curse](../../docs/research/2026-10-19-act1-content-red-colorless.md)
- [Silent/Shiv](../../docs/research/2026-10-19-act1-content-silent.md)
- [Defect/orbs](../../docs/research/2026-10-19-act1-content-defect.md)
- [Watcher/dependent specials/Wish](../../docs/research/2026-10-19-act1-content-watcher.md)

These documents are historical **draft/source-grounded** studies with explicit
unclosed composition fields, largely scoped to Ironclad A0 Act 1 and off-color
reachability. Linking them does not accept all their findings, implement other
character starts, or turn them into original-runtime proof. Source hashes establish
which public document bytes were used, not whether the game behaves that way.

For example, Brutality's printed text lists HP loss before draw while the local
draft describes queued draw before HP loss; FTL similarly has a printed-order vs
queued-order distinction. Both accounts are retained under `discrepancies` rather
than silently rewriting either. More generally, text order is not a scheduler
specification: copying/identity, conditional action-time capture, RNG mapping,
modifier rounding, autoplay legality and terminal pruning still need the scoped
implementation work's evidence. Every record is marked **not original-runtime
validated**. RNG parity is not established or claimed.

## Regenerate and check

Use the repository's existing Python 3.12 interpreter; no install is needed:

```sh
python3.12 data/cards/build_dataset.py
python3.12 data/cards/build_dataset.py --check
python3.12 -m unittest discover -s tests/python -p 'test_card_reference*.py' -v
```

The tests check normalization, identity coverage, source integrity, and selected
hand-checked pairs/edge cases. They are **not** simulator tests or proof of game
mechanics. `--check` compares a fresh in-memory regeneration with the stored file.
Neither command fetches data or touches simulation/registry code.

For an intentional source refresh, retrieve the exact query in
`sources/wiki-retrieval.json`, then use the MediaWiki revisions API
(`action=query`, `prop=revisions`, `rvprop=ids|timestamp|content`, `rvslots=main`)
for the returned unique `_pageName` values. Preserve the snapshot formats and new
retrieval time, review changed rows/pages/notes and attribution, then regenerate.
Do not treat a newer wiki edit as automatic validation of PC v2.3 behavior.

## Attribution and licensing

Wiki text in `sources/wiki-cards.json` and `sources/wiki-pages.json`, and its
rendered/adapted descriptions and wiki-derived notes, is attributed to **Slay the
Spire Wiki contributors**, licensed under
[Creative Commons Attribution-ShareAlike 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
The adapted reference data and authored reference annotations in this directory
are also provided under CC BY-SA 4.0. Changes include stripping presentation markup,
expanding base/upgrade alternatives, preserving energy as `[E]`, normalizing
classification and adding independently worded notes/provenance. Original page
links, permanent revisions and contributor/history access remain in each record.

The identifier-only extract is attributed to **gamerpuppy**, MIT, copyright 2021;
see [`sources/LICENSE.sts-lightspeed.txt`](sources/LICENSE.sts-lightspeed.txt).
The normalizer and tests remain project code under the repository's MIT license.
This separate data license does not relicense the Rust/Python gym. Slay the Spire
is owned by Mega Crit; no original game source, binaries, art or private evidence
is included, and these notices do not grant rights to the original game.
