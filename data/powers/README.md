# Player and enemy buff/debuff reference

**Target:** unmodded Slay the Spire 1, PC v2.3, all four characters and enemies
through Act 4. This is an **offline implementation reference**, not a runtime
registry, implemented-power list, or original-runtime fidelity result. No simulator,
dependency, observation, or gameplay behavior is changed.

## Coverage

[`powers.json`](powers.json) contains **133 records**:

- All **106 existing `power.*` catalog keys**, preserving their documented exact
  ID literals and class aliases rather than renaming them from display names.
- The captured wiki's StS1 Powers roster, excluding its **Orb Slot** entry because
  slot capacity is not a buff/debuff power identity.
- Separate player/enemy Intangible records, and the **Surrounded** indicator from
  the wiki module/enemy page, which is missing from the Cargo roster.

There are 89 player-only, 35 enemy-only, and 9 shared-owner records. These describe
ordinary recipients, **not** the sources applying them or arbitrary debug/mod
compatibility. Hex and Constricted are carried by the player even though enemies
apply them; Poison is normally carried by enemies.

**Identity qualification:** 107 records have documented ID literals (the 106-key
catalog plus the locally documented `Night Terror` family). **26 supplementary
records intentionally have `game_id: null`** because an exact constructor ID was
not established by the retained public documentation. These are listed in the data,
not silently guessed from names such as Flying, Regenerate, or Beat of Death.
They still have effects, owner, stacking/lifecycle information, sources, and explicit
identity uncertainty. Resolve those IDs through scoped evidence before registry
integration. This is complete against the named source roster, **not a claim of an
independently exhaustive original class/debug/obsolete-power census**.

Stances, orb objects, slot capacity, and Status/Curse **cards** are separate domains.
Mantra, Focus and powers that refer to them remain included; a Dazed card does not
become a debuff power merely because it is detrimental. See also the
[card reference](../cards/README.md) and
[relic reference](../relics/README.md).

## Files

| File | Purpose |
|---|---|
| `powers.json` | Generated self-contained reference, global qualifications, and source manifest |
| `annotations.json` | Independently worded refinements, discrepancies, owner-sensitive lifecycle/state notes |
| `sources/catalog-powers.json` | Exact 106-key public catalog extract; identity/direct-source mentions only |
| `sources/wiki-name-aliases.json` | Explicit documented-ID → wiki-display mapping; never guessed constructor IDs |
| `sources/wiki-powers.json` | All 132 retrieved StS1 Cargo rows, including the auditable Orb Slot exclusion |
| `sources/wiki-pages.json` | Supporting public wikitext with revision IDs/timestamps, 125 pages |
| `sources/wiki-retrieval.json` | Query, retrieval time, non-transactional qualification, and reused Vault snapshot |
| `build_dataset.py` | Small offline normalizer; standard library only, no compiler/game/network dependency |

## Fields and interpretation

| Field | Meaning |
|---|---|
| `key` | Existing catalog key, or explicitly supplemental `power.reference.*` key |
| `game_id`, `class_alias`, `identity_grade` | Documented exact ID/class or intentional `null`; evidence boundary |
| `id` | Uppercase convenience label, not a Rust enum, numeric discriminant or game ID |
| `reference_extension` | Whether this is outside the retained 106-key catalog, not a new native registration |
| `name`, `wiki_name`, `owners` | Display identity and normal **recipient** family |
| `classification` | Qualified reference type, practical effect on owner, and type-evidence basis |
| `effect` | Readable primary/refined effect, not executable ordering or byte-exact localization |
| `stacking` | Mode, meaning of amount, extra conceptual state, and only specifically qualified caps |
| `lifecycle` | Semantic triggers, their evidence basis, turn progression, removal and combat-reset limits |
| `wiki` | Unflattened source taxonomy, original summary, and raw cap metadata for comparison |
| `source_owner_mentions` | Catalog's direct producer/reference mentions; **not** the recipient list or reachability proof |
| `notes`, `discrepancies`, `uncertainties` | Selected implementation traps, source disagreements, and unresolved fields |
| `citations`, `evidence` | Versioned page/local-document support at its actual grade; runtime validation remains false |

Use resolved `game_id` for cross-dataset game identity; use `key` to identify an
unresolved reference record. Do not coalesce null IDs into one power. The Bomb's
documented `TheBomb` literal identifies a family, not a promise that every live
instance has the same unsuffixed string. Multiple independent instances/templates
remain distinct even when a documented family literal matches.

### Classification is not benefit

- **Blasphemer / `EndTurnDeath`:** the Watcher draft explicitly reports BUFF
  classification despite its harmful HP-loss request. The wiki table's Debuff
  label is retained as a discrepancy. Artifact/debuff cleansing must not be
  implemented by treating every harmful effect as a DEBUFF.
- **Shackled:** technically a DEBUFF while restoring the affected enemy's Strength.
  Cleansing it can prevent that later benefit.
- **Strength/Dexterity/Focus:** signed state and negative applications cannot be
  reduced to one permanently beneficial label. `SIGN_DEPENDENT` is a reference
  category, not an invented original enum. Exact type changes across sign-crossing
  application/stack/reduction routes still require their named source path.
- **Choked, Corpse Explosion, Block Return and Fasting:** local findings refine the
  wiki's misleading Buff labels, without promoting their existing evidence grade.
- **Surrounded:** technical classification remains explicitly unknown. The enemy
  page identifies a telegraph rather than a second damage modifier; applying a
  1.5 multiplier on both player Surrounded and enemy Back Attack would double count.

### Stacking, state and timing

Wiki “Intensity,” “Duration,” “Counter,” and “BetweenTurns” are descriptive
categories, not an original-game type system. `caps: null` means **not established**,
not unlimited; no blanket 999 cap is imposed. The wiki's Mantra Cap text is visibly
truncated in Cargo and retained raw, not converted to a numeric rule.

Examples of distinctions retained in the records:

- Player Regen diminishes after healing; enemy regeneration does not simply inherit
  that decrement. Player Intangible and enemy Intangible are separate records.
- Nightmare keeps independently selected card templates; The Bomb keeps per-cast
  countdown/damage. Neither becomes one aggregate intensity keyed by display name.
- Deva has separate growth increment and next energy gain. Panache has separate
  damage and remaining-card counter; Invincible has remaining and reset quotas.
- Poison ticks at the afflicted owner's start-turn; Noxious Fumes applies Poison
  at player post-draw. Those clocks are not interchangeable.
- Thorns is a qualifying hit callback, including fully blocked hits; Sharp Hide
  responds to Attack-card plays, not once per hit in a multi-hit Attack.
- Static Discharge's documented positive callback damage is not necessarily final
  HP lost after later modifiers. Raw power Block is not calculated card Block.
- Wraith Form's signed recurring downside has an end-turn finding in the retained
  draft, contrary to the wiki's start-turn description. Retain Cards likewise has
  an end-turn selection finding despite the wiki's start-turn wording.
- The wiki Draw Reduction intensity/duration accounts disagree and are not silently
  converted into a single unqualified amount/expiry law.
- Duplication's Cargo “Conserved” label and potion “This turn” tooltip conflict
  with the retained potion finding: qualifying uses consume charges, and end-round
  removes an already-zero power or reduces a positive amount by one. Neither
  indefinite conservation nor unconditional removal of all charges at turn end is
  silently substituted for that qualified finding.

`DESCRIPTION_*` trigger labels are **only cues from a public description**. They
are not exact source callback names or proof of the complete event inventory.
Annotated semantic triggers likewise retain the cited finding's scope. There is
no universal “tick all durations now,” “remove at zero,” “powers before relics,”
or “drain every queued child before terminal settlement” rule in this dataset.

## Sources and evidence boundaries

The [StS1 wiki Powers table](https://slaythespire.wiki.gg/wiki/Special:CargoTables/Powers)
was queried with `Game='1'`, excluding StS2 and board-game rows. Each record points
to a permanent revision and supporting snapshot. The general
[Buffs](https://slaythespire.wiki.gg/wiki/Buffs) and
[Debuffs](https://slaythespire.wiki.gg/wiki/Debuffs) pages explicitly warn that they
are under construction. A newer community edit is not automatically PC v2.3 truth.
Cargo and page revisions were fetched separately, not as one transactional revision.
Vault's supporting revision is reused from the existing public card snapshot, as
recorded in retrieval metadata.

Existing authored public research is reused, not private archives/source:

- [Exact catalog IDs/classes and direct source mentions, §2.9](../../docs/research/2026-10-19-act1-complete-catalog-numeric.md)
- [Ironclad/shared effects](../../docs/research/2026-10-19-act1-content-red-colorless.md)
- [Silent, Poison, retaliation and delayed lifetimes](../../docs/research/2026-10-19-act1-content-silent.md)
- [Defect/orb calculations and callbacks](../../docs/research/2026-10-19-act1-content-defect.md)
- [Watcher classification, state and Mark](../../docs/research/2026-10-19-act1-content-watcher.md)
- [Enemy reactions and regeneration](../../docs/research/2026-10-19-act1-content-enemies.md)
- [Potion charge/lifetime findings](../../docs/research/2026-10-19-act1-content-economy-potions.md)
- [Site-specific power application/callback ordering, §3](../../docs/research/2026-10-19-act1-transitive-execution-closure.md)

These findings retain their historical source-grounded/draft/local grade, mostly
Ironclad A0 Act 1 and off-color reachability, with unclosed composition fields.
They do not establish full-character starts, all-act acceptance, original version
provenance, or original-runtime behavior. Only the named local findings are reused;
unrelated historical clock/timer proposals are not adopted. Hashes identify source
bytes, **not mechanics proof**. Every record explicitly says it is not
original-runtime validated.

The gym's existing [RNG-001 policy](../../docs/intentional-deviations.md) applies:
preserve applicable conditional probabilities, candidate restrictions, history and
deliberate dependencies; do not claim original Java PRNG, seed/cursor consumption or
incidental stream coupling parity. Descriptions such as “random card” do not close
those generator laws or make hidden state public observations.

## Regenerate and check

Using the already-installed Python 3.12 interpreter, without installs:

```sh
python3.12 data/powers/build_dataset.py
python3.12 data/powers/build_dataset.py --check
python3.12 -m unittest discover -s tests/python -p 'test_power_reference*.py' -v
```

The twelve focused checks protect source normalization/identity coverage and selected
owner/classification/lifecycle distinctions, not the simulator's mechanics.
`--check` performs an in-memory regeneration and byte-for-byte comparison.

For an intentional source update, use the exact recorded Cargo query and the
MediaWiki revision API (`prop=revisions`, `rvprop=ids|timestamp|content`,
`rvslots=main`) for supporting pages. Preserve formats and attribution, document
reuse/retrieval, and review changed identities and annotations before regenerating.
An unresolved ID is not an invitation to infer it from a display name or to inspect
private game artifacts without separate authority.

## Attribution and licensing

Public wiki text/snapshots are attributed to **Slay the Spire Wiki contributors**,
[Creative Commons Attribution-ShareAlike 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
The adapted reference data and independently worded annotations in this directory
are provided under **CC BY-SA 4.0**. Changes include removing presentation markup,
preserving energy icons as `[E]`, distinguishing owner/identity variants, and adding
qualified research notes, discrepancies and provenance. Revision links preserve
access to the original pages and contributor/history records.

The normalizer/tests remain project code under the repository's **MIT** license.
The separate data license does not relicense the gym. Slay the Spire is owned by
Mega Crit; no original game source, binaries, images/assets, private evidence or
executable game artifacts are included, and these notices grant no rights to them.
