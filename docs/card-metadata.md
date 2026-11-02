# Card metadata descriptor (opt-in, v1)

`spire-lab-card-metadata-v1` is a static, public, machine-readable description of
every card catalog row (spire-90f). It is additive: no observation schema,
vocabulary or existing fingerprint changes, and nothing reads it during play.

## Access

- Rust: `spire_env::card_metadata_descriptor()` and `CARD_METADATA_SCHEMA_ID`.
- Python (source only until an extension is rebuilt from it):
  `spire_lab._native.card_metadata()`, or `spire_lab.card_metadata.load()`, with
  `by_id()` and the example `render_card()` renderer.

## Sources and identity

The descriptor is generated on read from the existing owners; there is no card
table in `spire-env`:

| Field | Owner |
| --- | --- |
| `id`, `color`, `rarity`, `type`, `tags` | `content::Catalog::standard()` rows and `card_tags` |
| `support.complete_*`, `choice_only`, `removable` | `content::complete` and note storage |
| `faces[].keywords` | `combat::complete_live_card` |
| `faces[].base_damage`, `base_block`, `magic_number` | `cards::public_values`, the owner live card views use; else the constructor |
| `faces[].cost`, `faces[].target` | `combat::static_card_face`: the public value owner's X/unplayable class, then the existing cost and payload owners, probed read-only |
| `max_upgrades` | the complete upgrade bound (`null` = unbounded) |
| `faces[].catalog_target` (blue only) | the M20 blue catalog |

`static_card_face` evaluates one neutral complete-card state (one full-HP
Cultist, no powers, relics or potions, three energy). It never steps a combat and
samples no RNG. The `fingerprint` is SHA-256 over `spire-lab/card-metadata/v1\0`
and the canonical JSON of every other field, so any content, constructor, cost or
payload change moves it; the Rust test pins the current value.
`content_revision.cards_sha256` hashes only the `cards` array (domain
`spire-lab/card-metadata/v1/cards\0`), so it identifies the content and rules
revision independently of descriptor prose. A session's own `content_id` and
`rules_id` in `environment_spec()` identify its profile.

## Semantics and limits

- Registry metadata is not admission. A session's `environment_spec()` support or
  `finite_content` and its `vocabularies['card']` decide what a profile,
  character and Ascension admit; a card absent there is unsupported, and profile
  upgrade bounds may be narrower than `max_upgrades`. Item admission (profile
  items or M27 complete items) never widens card admission.
- Faces are printed values. Live observations, card views and legal actions are
  authoritative for current cost, upgrades, reductions, powers, targets and
  playability, including held relic or potion effects that the item-free probe
  cannot show; the descriptor never overrides them.
- `unresolved` means the neutral state cannot decide the field: a play condition
  on pile contents or generation state, or a kind that is unplayable by its own
  rule. The 11 such ids are listed in `unresolved_cards` (Deus Ex Machina left the list on
  2026-11-02 when its public value became `unplayable`, spire-oto.7). Kinds the public value
  owner marks unplayable, and statuses and curses with no playable payload,
  report cost `unplayable`.
- Not modeled in v1: display names (there is no existing public name owner; ids
  are snake_case), natural-language mechanic summaries, effect structure beyond
  cost, target, keywords and constructor numbers, and upgrades past the first.
- Public-information safe: no seed, RNG cursor, hidden identity or order, queue,
  listener state, unrevealed outcome or diagnostic.
- Provenance: fields are generated from native owners. No game text, wiki prose,
  binaries or assets are included.
