# M26 shared and off-colour card coverage design

Date: 2026-10-31. Status: **planning proposal, version 3**. Version 2 resolved the
independent plan review v1 (NOT READY, 0 Critical / 5 Important / 11 Minor, report SHA-256
`468ab9154bd0188691af73f78d321b92d2a2c1af25ff8de00524970069a6b90b`). Version 3 resolves
re-review v2 (NOT READY, 0 Critical / 1 Important / 4 Minor, report SHA-256
`3ba9bafd816f39457cc7b7bf36ca33b9e26376c1a063455785bcb7340987cfda`). Both versions apply
the PM's rulings on the open questions (section 17). The disposition tables are retained
with the planning evidence (section 18). Planning Bead `spire-dxo.48.1`;
roadmap outcome `spire-dxo.48` ("M26 — Shared/off-color card coverage"). Planning source:
`main` at `7b036e8d2047f17eb765fe42b1c4bb8e164a37e9` (M24 merged in PR64).

**Predecessor contract.** M25 (Watcher A0 Acts 1–4 and Heart, complete Watcher coverage) is
approved and executing but not merged. M26 is planned against the M25 planning pair version
2 as a contract: design SHA-256
`04a085d101d6de9198616fabed17b45fc3c91206336670201b79ee6f985e24e0`, plan SHA-256
`20bf21c46525520e18232b7880fd12859b61451d38252f27e8dc4e0d0650a62f`, both at
`/home/klottick/.local/share/spire-gym/evidence/m25-execution-20261030/planning-v2/`, plus
the PM-accepted amendment A1
(`/home/klottick/.local/share/spire-gym/evidence/m25-execution-20261030/pm-ruling-amendment-A1-v1.json`,
a wire-spelling correction that M26 does not touch). M26 execution cannot start until M25
is merged. Section 16 lists every M25 interface M26 depends on and the check Task M0 makes
against the merged source, including when an amendment must go back to the plan reviewer.
The paired [implementation plan](../plans/2026-10-31-m26-shared-cards.md) turns this design
into tasks. This document releases no runtime work by itself.

## 1. What M26 delivers

The [roadmap](2026-10-22-narrow-gym-milestones-design.md) defines M26 as: "Complete ordinary
colorless, curse, status, generated, transformed and copied-card paths; legal
off-color/Note/Shard acquisition uses the same implementations. Complete eligible card
producer pools without hidden cuts, including exceptional upgrades/misc/identity behavior."

M26 is the card layer that M27 (relics and potions) and M28 (events, Neow, shops and the
standard pools) call. After M26:

1. **Every standard card exists and plays.** Every card in `data/cards/cards.json` whose
   `availability.standard_run` is true has a native identity and an executable
   implementation in base and upgraded form (where it upgrades). M26 adds the last 5
   colourless cards and 10 curses that M25 does not add, and gives the 2 definition-only
   curses (Regret, Ascender's Bane) their behaviour.
2. **Any character can hold and play any legal card with the card's own implementation.**
   One complete card capability admits, for every character, every card a standard run can
   put in a deck or create in combat. An off-colour card is initialized and resolved by its
   own colour's owner. Where the source makes behaviour depend on the player's character
   (generators that draw from "your" pool, orb cards without orb slots), the shared owner
   follows the source law for that character.
3. **Every card producer has a complete pool in one owner.** Combat generators for all four
   characters, the ordinary, any-colour (Prismatic Shard), colourless and random-curse
   pools, the transform supports and the Note For Yourself eligibility are computed from the
   catalog by one owner, with exact expected sizes checked before any RNG is used. No pool
   is a profile's finite list.
4. **Permanent deck changes are complete and shared.** Obtain (any legal card, any legal
   upgrade count), upgrade (including Searing Blow's repeatable upgrade), remove (with
   curse rules), transform, duplicate and Note retrieval are single Run owners that keep
   the source's identity, upgrade and `misc` rules. Combat-to-Run persistence covers Ritual
   Dagger's growth alongside the existing Genetic Algorithm and M25's Lesson Learned.
5. **Copy, generated and status paths are complete for every card kind.** The shared
   stat-equivalent copy owner is checked for every card kind; every generated child and
   status works for every character.

Every item is implemented to source law and exercised by meaningful tests. Constructed and
supplied setups are valid. No outcome run, normal-start route or policy win is required
(section 2, rule 1).

### 1.1 What M26 owns and what it leaves

| Area | M26 owns | Left to |
| --- | --- | --- |
| Cards | Identity, catalog metadata, combat behaviour and upgrade of every standard card; Pride is excluded (Endless Mode only, `standard_run: false`) | — |
| Ascender's Bane | Card behaviour (Unplayable, Ethereal, cannot be removed) | Its A10 creation: M30 |
| Card pools | Complete membership and the in-pool draw law for every producer listed in section 7 | — |
| Card reward screens, shop card rows, Neow, Library, Match and Keep and other event card grants | The pools and Run mutations they call | Room wiring, rarity schedule (rarity roll and its offset, upgrade chance by act), prices: M28 |
| Relic-driven card behaviour (Prismatic Shard enabling the any-colour pool, Toolbox, Dead Branch, Nilry's Codex, Enchiridion, Dolly's Mirror, Astrolabe, Pandora's Box, Empty Cage, eggs, bottles, Calling Bell, Cursed Key, Necronomicon, Darkstone Periapt, Omamori, Blue Candle, Medical Kit) | The pools, transform, duplicate and obtain owners those relics call | The relic hooks, flags and choices: M27 |
| Potion-driven card generation (Attack, Skill, Power and Colourless Potions) | Complete typed pools for all four characters | Potion factories, use and inventories: M27 |
| Alchemize (a Silent card) | The class potion pools for Ironclad and Watcher (the Silent and Defect pools are accepted) and Alchemize for every character, because M26 makes the card legal for every character (section 6.2) | Other random-potion producers (Entropic Brew, rewards, merchants), and potion use: M27 |
| Note For Yourself | Eligibility for every card; retrieval of a stored `(card, upgrades)` through the shared obtain owner | The event: M28. The stored card's persistence between runs: M34 |
| Public observation of the complete card set | — (section 13) | The first profile that publishes the complete capability (M28 standard profile or M34) |

## 2. Rules that shape this design

Binding user rules, restated so the builder does not need to look them up.

1. **The repository plays the game; it does not need to win it**
   (`/home/klottick/.local/share/spire-gym/evidence/primary-scope-clarification-20261030/gameplay-not-strategy-v1.txt`).
   "Complete" means implemented to source law and exercised by meaningful tests. Supplied
   fixtures are acceptable. No normal-start win, route or outcome gate.
2. **Fix real bugs globally.** A defect in a shared owner is fixed at the shared rule for
   every character. An accepted assertion that pinned wrong behaviour is amended, the
   change is disclosed, the original bytes are retained, and a regression test must fail
   under any narrower fix. Section 11 lists the corrections planning already found.
3. **One engine.** Extend the existing catalog, combat, resolver, generation, offer, pile,
   Run mutation, journal, census and diagnostics owners. No copied loops, tables, samplers,
   encoders or validators. No public supplied-run or import API. Diagnostics stay
   default-off.
4. **No installs.** No new dependencies, no Python rebuild, no original-game execution.
5. **Frozen profiles stay frozen.** Combat-v1, the default and standalone (Gym66) profiles,
   policy-v3, and every accepted M9–M25 profile, finite list, descriptor, fingerprint, wire
   value and RNG key stay unchanged, except mechanical new exhaustive-match arms and the
   disclosed global corrections of section 11.

## 3. Approaches considered

| Approach | Result | Decision |
| --- | --- | --- |
| One complete card capability and one pool owner in the shared engine, used by an overlay on existing character contexts; complete Run mutation owners; tests as the evidence; public exposure left to the first profile that uses the capability | Closes every card path once for all four characters. No frozen profile changes. M27/M28 call finished owners instead of inventing per-relic or per-event card code. | **Selected.** |
| Widen the existing M10, M15, M20 and M25 Heart profiles to admit all cards | Breaks their frozen descriptors, prefixes and finite unions. | Rejected. |
| Add a new public Run profile for M26 | Would need card reward, shop and event wiring that M28 owns, so it becomes a second, partial standard profile. | Rejected (PM ruling 1). |
| Add an additive all-character complete-card combat profile to the Gym66 standalone interface | Gives an immediate public way to play every card, but is a new public surface whose Python facade cannot be rebuilt or tested under rule 4. | Not selected (PM ruling 1). |
| Implement only the 15 missing cards | Leaves generators unsupported for Ironclad and Watcher, transform and duplicate absent, and every Run admission finite: the roadmap's "hidden cuts". | Rejected. |

## 4. The complete card set

Values come from `data/cards/cards.json` (and its `implementation_notes`), the
[card reference](../../../data/cards/README.md), the
[event and Neow research](../../research/2026-10-19-act1-content-event-neow.md) §5.1, and
the planning research retained at
`/home/klottick/.local/share/spire-gym/evidence/m26-execution-20261031/planning-v1/`
(`research-A-catalog.md`, `research-B-owners.md`). They are implementation inputs, not
original-runtime validation.

### 4.1 Inventory after M25

The dataset has 370 rows. After M25 merges, 354 have a native identity. M26 adds 15 and
excludes Pride.

| Group | Rows | Native after M25 | M26 work |
| --- | --- | --- | --- |
| Ironclad, Silent, Defect, Watcher ordinary (incl. 4 or 3 basics each) | 300 | 300 (M10, M15, M20, M25) | Off-colour play (section 6) |
| Colourless ordinary (Uncommon 20, Rare 15) | 35 | 34; Bandage Up missing | Bandage Up |
| Colourless event cards (Special) | 4 | 0 | Apparition, Bite, J.A.X., Ritual Dagger |
| Generated children (Shiv, Miracle, Smite, Safety, Insight, Through Violence, Beta, Omega, Expunger) | 9 | 9 (5 from M25) | Off-colour creation and play |
| Wish choices | 3 | 3 (M25) | None |
| Statuses (Burn, Dazed, Slimed, Wound, Void) | 5 | 5 | Every-character coverage |
| Curses | 14 | Parasite; Regret and Ascender's Bane definition-only | 10 new curses; Regret and Ascender's Bane behaviour; Pride excluded |

### 4.2 New colourless cards

Costs and values are base → upgraded. E = Exhaust, Eth = Ethereal.

| Card (data key) | Type | Cost | Effect, base → upgraded | Keywords | Tags |
| --- | --- | --- | --- | --- | --- |
| BandageUp (`BANDAGE_UP`) | Skill, Uncommon | 0 | Heal 4 → 6 HP | E | Healing |
| Apparition (`APPARITION`, game ID `Ghostly`) | Skill, Special | 1 | Gain 1 Intangible | E; base Eth, upgraded not Eth | — |
| Bite (`BITE`) | Attack, Special | 1 | Deal 7 → 8 damage; heal 2 → 3 HP | — | — |
| Jax (`JAX`, game ID `J.A.X.`) | Skill, Special | 0 | Lose 3 HP; gain 2 → 3 Strength | — | — |
| RitualDagger (`RITUAL_DAGGER`) | Attack, Special | 1 | Deal damage equal to its `misc` (15 when obtained); if Fatal, permanently increase this card's `misc` by 3 → 5 | E | — |

Bandage Up's HEALING tag excludes it from combat generators (section 7.1) and keeps it in
every permanent pool (section 7.2). Bite's heal is not HEALING-tagged in the reference; Task
M0 records the tag set it binds (frontier F4).

### 4.3 Curses

None upgrades. "In hand at end of turn" uses the owner Burn already uses.

| Curse | Rule | Removable | In random curse pool |
| --- | --- | --- | --- |
| Clumsy | Unplayable, Ethereal | Yes | Yes |
| CurseOfTheBell | Unplayable | **No** | No |
| Decay | Unplayable; at the end of the player's turn while in hand, take 2 damage (blockable) | Yes | Yes |
| Doubt | Unplayable; at end of turn while in hand, gain 1 Weak | Yes | Yes |
| Injury | Unplayable | Yes | Yes |
| Necronomicurse | Unplayable; when exhausted, a copy is added to hand | **No** | No |
| Normality | Unplayable; while in hand, no more than 3 cards can be played this turn (counts duplicated and autoplayed cards; drawing it after 3 plays blocks further plays) | Yes | Yes |
| Pain | Unplayable; while in hand, lose 1 HP whenever another card is played | Yes | Yes |
| Parasite (accepted) | Unplayable; removing or transforming it from the deck loses 3 max HP | Yes | Yes |
| Regret (behaviour new) | Unplayable; at end of turn while in hand, lose HP equal to the hand size snapshotted when the turn end begins, including Regret | Yes | Yes |
| Shame | Unplayable; at end of turn while in hand, gain 1 Frail | Yes | Yes |
| Writhe | Unplayable, Innate | Yes | Yes |
| AscendersBane (behaviour new) | Unplayable, Ethereal | **No** | No |

"Removable" is the purgeable-card predicate used by every remove and transform selection
(`NoteStorage::NonremovableCurse` in `content/cards.rs`). The random curse pool is exactly
the ten "Yes" rows (event and Neow research §5.1).

### 4.4 Union totals

| Union | Members | Count |
| --- | --- | --- |
| Complete permanent (deck-legal) | 300 character cards + 35 ordinary colourless + 4 event colourless + 13 curses (all but Pride) | 352 |
| Complete generated-only | 9 children + 5 statuses | 14 |
| Choice-only | Wish's 3 options | 3 |
| Total admitted | | 369 (370 rows minus Pride) |

## 5. One complete card capability

### 5.1 Catalog and tags

- Every M26 identity is appended to `CardId` without renumbering, with a catalog row that
  is not `definition_only`. Regret and Ascender's Bane stop being `definition_only`.
- M25's single card-tag owner (`content::cards::card_tags`, `is_healing`, `is_strike`,
  section 16) gains `Healing` for Bandage Up (already in its HEALING list). No other tag
  changes.
- A new owner, proposed `content::complete` (file `crates/spire-core/src/content/complete.rs`),
  defines the unions of section 4.4 from the catalog: `supports_permanent(CardId)`,
  `supports_generated(CardId)` (permanent or generated-only) and `is_choice_only(CardId)`,
  with the literal counts 352/14/3 asserted by test. It is computed from catalog colour,
  rarity, type and availability, not written as a literal list, and it never includes
  Pride. The finite M10–M25 lists and predicates are unchanged.

### 5.2 Combat admission

Every character's combat construction today admits only its profile's finite cards (for
example `cards::watcher::initialize_profile`, `content::defect::supports_m20_*`). M26
follows the M20 rule: **add a distinct complete capability; do not widen the finite
predicates.**

- `CombatItemContext::with_complete_cards()` sets a private `complete_cards` flag, read by
  `complete_cards()`. It is an overlay on an existing character context: the character's
  complete Heart context (M10 Ironclad, M15 Silent, M20 Defect, M25 Watcher; Task M0 binds
  the exact constructors) plus the flag. Relic and potion validation stay that context's.
  `Debug` prints the flag only when true. The flag also admits, in potion-belt validation,
  the character's class potion pool (30 shared and 3 class potions), so that an Alchemize
  potion is a valid inventory item through settlement. Potion use stays with the accepted
  potion owners (M27 owns the rest of the lifecycle). Each Heart context has 3 potion slots
  at A0, which is Alchemize's capacity precondition.
- Under the flag, construction admits every card in the complete unions and initializes it
  through **its own colour's existing initializer** (purple through the Watcher owner, blue
  through the Defect metadata, and so on), whatever the player's character. Curses and
  statuses use their shared initializers. A choice-only card never becomes a physical card.
  This generalizes M25's rule for Foreign Influence outputs to every card; it is one
  dispatcher, not four copies.
- Under the flag, the character-gated card laws are re-gated as section 6.3 states.
- Under the flag, every combat-wide tracker that some admitted card reads is on for every
  character: the union of what the four Heart contexts enable (for example the draw and
  discard history and `source_character` that `combat/engine.rs` gates on
  `shared_generated_profile` and `m21_profile`, and the `PlayerStart` phase gated on
  `m20_profile`). Task M0 lists them. Character-dependent laws (orb slots, the player's
  generator pools) still follow the player's character (section 6.2). The overlay must be
  inert for a character's own cards: an own-colour play under the overlay equals the same
  play under the base context without it (matrix X, section 14.3).
- Without the flag, every existing profile's admission, rejections and `Debug` text are
  byte-for-byte unchanged.

### 5.3 Run admission

- `RunState` gains a card-admission selector, proposed `card_admission: CardAdmission` with
  `CardAdmission::{Profile, Complete}`, defaulting to `Profile`. `Profile` keeps
  `run::mutations::supported_in` exactly as it is for every scenario. `Complete` makes the
  Run mutation owners (section 8) admit `content::complete::supports_permanent`. Task M0
  records whether `RunState` reaches any checkpoint text, fingerprint or receipt; if it
  does, the field is omitted from that text when it is `Profile`, as M25 does for its
  context flag.
- `run/combat.rs` adds `with_complete_cards()` to the combat context when the Run's
  admission is `Complete`.
- The supplied-start normalization owner (`run/training.rs`) gains a new, separate,
  character-independent validator, proposed `CompleteCombatStart::Supplied(AuthoredRunStart)`
  with `normalized(character)`, that validates supplied decks against the complete permanent
  union with the upgrade limits of section 8.1. It is a new type next to M25's
  `WatcherAdmission`, not a rename, so the M23–M25 call sites in `run/diagnostics/m23.rs` are
  untouched. It is reached only from tests and the default-off diagnostics of section 13.
- Nothing public selects `Complete` in M26. The first profile that uses it (M28) selects it
  at creation.

## 6. Off-colour play uses the same implementations

### 6.1 Rule

A card played by any character resolves through its own implementation. The only
differences are the ones the source makes (6.2). Today the Watcher and Defect runtimes are
switched on by the player's character, so a Watcher card played by an Ironclad is
silently ignored or faults (`combat/cards/watcher.rs` `active()` is
`source_character == Watcher`, and `validate` raises an Invariant fault without it). M26
re-gates every such site by one rule (6.3) so that:

- off-colour cards follow their source law for every character; and
- each character's own cards stay byte-identical, with or without the overlay (6.4).

### 6.2 Source law for character mechanics played off-colour

| Mechanic | Law for any character | Source |
| --- | --- | --- |
| Stances (Wrath, Calm, Divinity) | Entering and leaving stances, their damage multipliers (Wrath 2×, Divinity 3×, both dealt and Wrath received), Calm's 2 energy on exit, Divinity's 3 energy on entry and exit at the next turn start, Mental Fortress and Rushdown triggers: all as for the Watcher | Watcher research §2–3 (`docs/research/2026-10-19-act1-content-watcher.md`); stance is player state, not class state, in the source's `AbstractPlayer.stance` (semantic-contract closure, `player[16]` "stance object") |
| Mantra and Divinity | Mantra accumulates and enters Divinity at 10 for any character | Watcher research; Mantra is a player power |
| Scry, Miracle, Watcher powers (Like Water, Master Reality, Foresight, Establishment, Devotion, Battle Hymn, Nirvana and the M25 powers) | Character-independent player powers and actions | Watcher research; M25 design §5.8 ("each mechanic … serves every character") |
| Orb slots | Only the Defect starts with orb slots (3). Every other character starts with 0, unless something grants slots: Capacitor and Potion of Capacity add slots (maximum 10), and Prismatic Shard grants one slot on equip to a non-Defect that has 0 | `combat/characters.rs` (accepted); `data/relics/relics.json` PRISMATIC_SHARD `state_and_reset` ("equip grants one master orb slot only for non-Defect with zero existing master slots"); Capacitor wiki note "The maximum amount of Orb Slots is 10" (`data/cards/sources/wiki-pages.json`) |
| Channel and Evoke at 0 slots | Have no effect: no orb is channelled, nothing is evoked, no "channelled this combat" count increases. Focus changes still apply | Consume wiki notes: "If all of your Orb Slots are removed, attempting to Channel or Evoke Orbs will have no effect"; "If Consume is played while you have no Orb Slots, you will still gain Focus" (`wiki-pages.json`); M25 accepted Foreign Influence tests (Watcher Hyperbeam −3 Focus; zero-hit Barrage and Thunder Strike) |
| Orbs once slots exist | Channel, Evoke, Focus and orb passives behave as for the Defect: Lightning, Frost and Dark passives at the end of turn, Plasma at the start of turn; Loop and Darkness+ invoke passives; with no orbs these do nothing | Defect research §5.1 and the orb table (`docs/research/2026-10-19-act1-content-defect.md`, "Empty orb hooks do nothing"; Loop wiki note "If you have no Channeled Orbs at the start of your turn, Loop will not do anything") |
| Generators that draw from "your" pool | Infernal Blade (Attack), Distraction (Skill), White Noise and Creative AI (Power), Hello World (Common), Discovery (Any, offer), Chrysalis (Skill), Metamorphosis (Attack), and the Attack/Skill/Power potion selectors draw from the **player's** character pool (section 7.1) | Event and Neow research §5.1 ("actual current character"); `data/relics/relics.json` note "Uses current-character combat-generation support"; M15 ledger G1/G5 |
| Colourless and any-colour generators | Jack of All Trades, Transmutation and Magnetism draw from L34; Foreign Influence draws from M25's 106. Both are the same for every character | Same sources; M25 design §5.3 |
| Alchemize | Gains one random potion from the **player's class** potion pool (30 shared and the class's 3), limited form (Fruit Juice excluded), into the first empty slot; with a full belt it gains nothing and does not fault | M15 ledger G2/G4 (`docs/milestones/m15-silent-coverage-ledger.md`); `items/potion_generation.rs` (accepted Silent and Defect pools) |

### 6.3 Gate rule and the owned call sites

Every character or profile comparison in `combat/**` and `items/**` that can change card
behaviour falls into exactly one class:

- **Class L (card law): capability gate.** The site is reached only when a particular card,
  power or effect of one colour is in play (a Watcher effect, a Mantra application, a
  stance transition, a player-sourced orb effect). The new gate is
  `old condition || complete_cards()`. A character's own cards never reach the site, so
  the change cannot alter them.
- **Class S (shared path): state gate.** The site is on every combat's path, but its result
  depends on state that only exists once a mechanic is used: the stance is not Neutral,
  Mantra has been gained, an orb slot exists. The new gate is
  `old condition || <that state exists>`. While the state is absent, the old branch runs
  unchanged.
- **Class C (character structure): kept.** The site encodes the character's own turn
  structure or numeric surface, not a card law: the Defect's starting slots, the deferred
  turn-start energy order of the Watcher and Defect, and the signed/width numeric variants
  of the Block formula. Off-colour cards must still meet their source law inside the other
  character's structure; a test proves each one does.

**Split rule.** Some turn-start and turn-end owners combine a character's own energy
recharge with power callbacks. Each such owner is split. The recharge is Class C and kept
per character. Each power callback is Class L and runs for any character holding the power.
A part that depends on orbs is Class S and runs when an orb slot exists. Splitting a
function must not reorder its parts for the character that owns it.

**Next-turn energy law.** For every character, `energized` (Outmaneuver, Flying Knee,
Doppelganger) and `energized_blue` (Charge Battery) are each applied once at turn start, by
that character's recharge owner, after energy is reset to the maximum, and then removed.
Today:

- the non-deferred owner (`combat/turns.rs` ~45–46) applies only `energized`;
- the Defect recharge (`cards/defect.rs` ~347–354) applies both;
- the Watcher recharge (`cards/watcher.rs` ~371) applies neither.

The Watcher's loss of `energized` is already reachable in M25 through Foreign Influence
(Flying Knee). The PM has routed that global fix to M25, and M26 depends on it (section 16).
M26 completes whatever the merged M25 fix leaves out: at least `energized_blue` in the
non-deferred owner, and in the Watcher owner if M25 did not add it. Each is a disclosed
correction (G7).

Known sites (planning sweep at `7b036e8d`; Task M0 completes the list on merged M25, including
every `m25_profile()` gate such as Tantrum's shuffle in `combat/piles.rs`, and assigns each
new site a class by this rule):

| Site | Today | Class | New gate |
| --- | --- | --- | --- |
| `cards/watcher.rs` `stance_transition` | returns `Ok(false)` unless Watcher | L | `active || complete_cards()` |
| `cards/watcher.rs` `validate` | Invariant fault unless Watcher | L | `active || complete_cards()` |
| `powers.rs` Master Reality, Mantra application, Like Water and Watcher power clamps (lines 182–201) | Watcher only | L | `active || complete_cards()` |
| `engine.rs` Watcher power validation (`ApplyFullPower`, ~1894) | Watcher only | L | `active || complete_cards()` |
| `effects.rs` ~804, `engine.rs` ~920 and ~2405 Foresight selection | Watcher only | L | `active || complete_cards()` |
| `turns.rs` Establishment (~103) | Watcher only | L | `active ||` Establishment power present |
| `cards/mod.rs` card damage (~517) | stance pipeline only for the Watcher; others double for Wrath only | S | stance pipeline when `active ||` stance is not Neutral |
| `engine.rs` public `mantra_gained` (~1214) | `Some` only for the Watcher | S | `Some` when `active ||` Mantra gained > 0 |
| `piles.rs` end-of-turn hand discard (~59) | Watcher discard law only for the Watcher | S | Watcher discard law when `active ||` the hand holds a self-Retain card (Task M0 checks the Watcher law reproduces the ordinary law on every hand without one; if not, the self-Retain handling moves into the shared law) |
| `cards/defect.rs` `valid_orb_source` (~387–390) | player-sourced orb work only for the Defect | L | `defect || complete_cards()` |
| `cards/defect.rs` `turn_started` (322–356), orb part: Loop, start-of-turn passives, Gold-Plated Cables; and the end-of-turn orb passive owner | Defect only | S | runs when `defect ||` an orb slot exists; with no orbs it enqueues nothing |
| `cards/defect.rs` `turn_started`, Echo Form's per-turn reset (`echo_doubled_this_turn`, ~325; read at ~306–310) | Defect only | L | `defect ||` Echo Form present |
| `cards/defect.rs` `turn_started`, Biased Cognition's Focus loss (`"bias"`, ~337) | Defect only | L | `defect ||` Biased Cognition present |
| `cards/defect.rs` `turn_started`, Defect recharge with `energized` and `energized_blue` (~347–354) | Defect only | C | kept, Defect only (never runs for a non-Defect holding a slot) |
| `cards/defect.rs` `turn_started`, Emotion Chip impulse | Defect with the relic | Out of scope | relic hooks are M27; unchanged |
| `cards/watcher.rs` `hook` (349–380), Divinity exit at turn start (~353) | Watcher only | S | `active ||` stance is Divinity |
| `cards/watcher.rs` `hook`, Foresight, Battle Hymn and Devotion callbacks | Watcher only | L | `active ||` the power present |
| `cards/watcher.rs` `hook`, Like Water's end-of-turn Block in Calm (~380) | Watcher only | L | `active ||` Like Water present |
| `cards/watcher.rs` `hook`, Watcher recharge `energy = max_energy` (~371) | Watcher only | C | kept, Watcher only; applies next-turn energy per the law above (M25 fix, then G7) |
| `cards/watcher.rs` `capture` (~339; ~608–609 in M25's tree): `GainMantra` and `CreateRequest` conversion | Watcher only | L | `active || complete_cards()` (must open together with `validate`) |
| `combat/turns.rs` non-deferred `energized` application (~45–46) | `energized` only | C, with law | kept; also applies `energized_blue` (G7) |
| `cards/ironclad/reactive.rs` `append_before_decision` (~54) | Defect queue position | S | Defect position when `defect ||` an orb slot exists (frontier F15) |
| `generation.rs` Hello World and Creative AI turn-start hooks (~236) | Defect only | L | `defect || complete_cards()`, drawing from the player's pool |
| `generation.rs` Alchemize (~74–77), `engine.rs` Alchemize validation (~1489) and its frame ownership (~1566) | actual Silent with `m15_profile()` | L | any character with `complete_cards()` and a valid potion belt, using that character's class pool |
| `cards/ironclad/copies.rs` Infernal Blade (~19) | M10 red list for every character | L | the player's Attack pool (section 7.1) |
| `characters.rs` starting orb slots (~36) | 3 for Defect, 0 otherwise | C | kept (source law, 6.2) |
| `turns.rs` turn-start energy deferral (~43–44) | Defect and Watcher deferred | C | kept; tests that Deva Form, Collect, Simmering Fury, Divinity exit and next-turn energy meet their law for an Ironclad, and next-turn energy for a Silent, Defect and Watcher |
| `cards/mod.rs` Block formula (~558) | Watcher signed/width variant | C | kept; equal within the supported numeric surface |
| `engine.rs` `source_character`, draw history and `thief_gold` set-up (~493–499); `PlayerStart` phase under `m20_profile` (~587) | per base profile | Overlay | switched on by `with_complete_cards()` for every character (section 5.2) |
| `engine.rs` Watcher potion sources (~266), Entropic Brew (~1505) | per profile | Out of scope | potion use is M27; unchanged |

### 6.4 Inertness

Class L sites are reached only by off-colour content. Class S sites run their old branch
until the state exists, and a character's own cards never create that state (no Ironclad,
Silent or Defect card enters a stance or gains Mantra, and no Ironclad, Silent or Watcher
card adds an orb slot). Class C sites are unchanged. The overlay switches on trackers that
only record history. The inertness test (section 14.3) proves all of this for every
character's own cards.

## 7. Complete producer pools

One owner, proposed `content::pools` (file `crates/spire-core/src/content/pools.rs`),
computes every pool's **membership** from the catalog and `card_tags`, checks the literal
expected size below, and rejects an incomplete family or any `definition_only` member with
an explicit error **before** any RNG is used. `combat/generation.rs::pool` and
`Catalog::selectable_pool` become thin readers of it.

**Order.** Draw order is not source law (RNG-001), but frozen profiles draw by index, so
order is part of their accepted behaviour. A pool that is already accepted keeps its
accepted order, which the pool owner records once as that family's order and checks to be
a permutation of the derived membership: the Silent and Defect combat pools (rarity, then
`as_str`), L34 (`as_str` only), M10's Infernal Blade list for the Ironclad Attack pool, and
M25's Foreign Influence order. Every new family is ordered by rarity Common → Uncommon →
Rare, then `CardId::as_str`. The accepted lists move into the pool owner (their old call
sites read it), so no second copy remains.

### 7.1 Combat generator pools

Source: `returnTrulyRandomCardInCombat` family as cited in the M15 ledger G1/G5 and
`combat/generation.rs`; ordinary Common/Uncommon/Rare of the player's colour, HEALING
excluded; colourless pool excludes HEALING (Bandage Up).

| Pool | Ironclad | Silent | Defect | Watcher |
| --- | --- | --- | --- | --- |
| Any | 70 (72 − Feed, Reaper) | 70 (accepted) | 70 (accepted) | 69 (71 − Lesson Learned, Wish) |
| Attack | 28 | 26 (accepted) | 23 (accepted) | 24 |
| Skill | 28 | 33 (accepted) | 34 (accepted) | 33 |
| Power | 14 | 11 (accepted) | 13 (accepted) | 12 |
| Common (Hello World) | 20 | 19 | 18 (accepted) | 19 |
| Colourless L34 | 34 for every character (accepted members) | | | |
| Foreign Influence | 106 for every character (M25) | | | |

The Ironclad Attack pool's membership must equal M10's Infernal Blade support
(`run/rooms/m10_content.rs::infernal_candidates`), and its order is that list's order
(above). Infernal Blade today draws from that red list by index in
`combat/cards/ironclad/copies.rs` whatever the player's character. M26 changes it to draw
from `Combat { character: player, pool: Attack }`. When `source_character` is absent, as in
frozen M10 contexts (`combat/engine.rs` sets it only for some profiles), it uses the Ironclad
pool, which is in M10's order, so M10 draws stay byte-identical (Task M2 test). A
membership difference is a defect resolved at its owner under rule 2.

### 7.2 Run pools

These have **no HEALING exclusion** (event and Neow research §5.1: Feed, Reaper and Bandage
Up stay in permanent acquisition supports).

| Pool | Members | Sizes C/U/R (total) | Used by (caller milestone) |
| --- | --- | --- | --- |
| Ordinary(character, rarity, type?) | The character's Common/Uncommon/Rare cards, optionally one type | Red 20/36/16 (72); Green 19/33/19 (71); Blue 18/36/17 (71); Purple 19/35/17 (71) | Card rewards, shop rows by type, Neow, Library, Match and Keep (M28); Dream Catcher, Tiny House and Orrery card choices (M27) |
| AnyColour(rarity) | Ordinary Common/Uncommon/Rare of all four colours plus ordinary colourless (`ShardOrdinaryReward` eligibility) | 76/160/84 (320) | Prismatic Shard card rewards (M27/M28) |
| Colourless(rarity) | Ordinary colourless, including Bandage Up | 0/20/15 (35) | Shop colourless rows, Neow colourless, Match and Keep, Knowing Skull (M28) |
| Curse | Section 4.3 random pool | 10 | Neow drawback, Match and Keep (M28). Fixed curses (Calling Bell's CurseOfTheBell, Necronomicon's Necronomicurse, event curses) use `obtain_card` directly, not this pool |
| Transform(character, input) | Curse input: Curse minus input. Colourless input (ordinary or Special): Colourless minus input. Any other input: Ordinary(character) minus input | 9 or 10; 34 or 35; 71 or 72 (Ironclad), 70 or 71 (others) | Transform owner (section 8.3) |
| NoteForYourself | Every `NoteStorage::RemovablePermanent` permanent card of any colour | 349 (352 minus the 3 non-removable curses) | Note retrieval (section 8.5); event M28 |

Combat-time relic producers use the combat pools of section 7.1, not these: Dead Branch,
Nilry's Codex and Enchiridion use the player's combat pools (Any, Any and Power), and
Toolbox uses L34 (`data/relics/relics.json` notes; callers in M27).

Typed reward sizes by rarity, for shop rows, are in the plan's Task M2 table (from
`data/cards/cards.json`).

### 7.3 Draw law and RNG

- Run pools are sampled through the existing staged sampler
  `m15_content::sample_capture_row_for` (without replacement within one request, staged RNG
  committed only on success), not a new sampler. Its family parameter today is
  `fn(CardRarity) -> &'static [CardId]` with one slot per rarity; M26 generalizes it so each
  slot names its own `&[CardId]` family. The change must be RNG-identical for the M15, M20
  and M25 callers, which pass their existing families unchanged, and their accepted tests
  plus a fixed-seed byte comparison prove it. Single draws (transform, curse) use one
  uniform index over the complete support.
- M26 appends three scopes after M25's last scope in `crates/spire-core/src/run/rng.rs`:
  `RunRngScope::M26CardOffer` (`run.m26.card.offer`), `M26CardTransform`
  (`run.m26.card.transform`) and `M26CardCurse` (`run.m26.card.curse`). Callers supply the
  semantic key, which names the producer; M27/M28 reuse these scopes rather than adding one
  per producer.
- Combat generators keep the combat card stream and the existing capture-before-delivery
  law.
- RNG-001 (`docs/intentional-deviations.md`) applies: the support, uniformity and
  dependencies (no replacement where specified) are required; Java call parity is not.

## 8. Permanent deck changes

All in `crates/spire-core/src/run/mutations.rs` unless stated, all reading the Run's card
admission (section 5.3), all preflighting before any mutation, all firing the existing
`CardAcquired` and `CardRemoved` hooks exactly once per card.

### 8.1 Obtain

`obtain_card(state, id, upgrades)` under `Complete` admits any complete permanent card.
Upgrade limits: Searing Blow any count (the source has no cap; Task M0 records the largest
count the existing upgrade rule represents, and any bound kept is a disclosed limit); curses
0; every other card 0 or 1. Initial `misc`: Genetic
Algorithm 1 (existing), Ritual Dagger 15, otherwise 0. A new physical instance is allocated
as today.

### 8.2 Upgrade and remove

- Upgrade under `Complete` accepts any card for which the shared upgrade rule allows one
  more upgrade (Searing Blow repeatedly; curses, statuses and choice rows never). The
  instance keeps its identity and `misc`.
- Remove under `Complete` rejects non-removable curses (CurseOfTheBell, Necronomicurse,
  Ascender's Bane) and keeps Parasite's max-HP loss.

### 8.3 Transform

New `transform_card(state, instance, auto_upgrade, key)`: preflight that the input is
removable; remove it through the remove owner (Parasite's loss applies, event and Neow
research §5.1 and Parasite's implementation note); draw one output uniformly from
`Transform(character, input)` under `run.m26.card.transform`; obtain it fresh (base `misc`,
0 upgrades, or 1 if `auto_upgrade` and the output can upgrade) through the obtain owner.
An off-colour input transforms into the player's character pool. Repeated transforms may
produce the same output. `auto_upgrade` exists for M27's Astrolabe; event and Neow
transforms pass false.

### 8.4 Duplicate

New `duplicate_card(state, instance)`: a new physical instance with the same id, upgrades
and `misc` (Dolly's Mirror and the Duplicator event). The copy then grows independently. It
carries no bottle state (bottles are M27 and are not part of `CardInstance`).

### 8.5 Note retrieval

New `obtain_stored_card(state, id, upgrades)`: eligibility `NoteForYourself`, then a fresh
card with `upgrades` applied through the shared upgrade rule that many times (so a stored
Searing Blow keeps its count) and **base `misc`** (stored scaling is not restored; event
and Neow research E23). Storing the outgoing card is M28's event and M34's persistence.

### 8.6 Combat-to-Run persistence and census

- The existing absolute `misc` journal (Genetic Algorithm) also accepts Ritual Dagger. Ritual
  Dagger's Fatal growth uses the shared Fatal owner that Feed, Hand of Greed and M25's
  Lesson Learned use, with its exclusions (minions, Awakened One's first phase, non-final
  Darkling kills; Ritual Dagger implementation note). Growth applies to the played card and
  its same-instance family, as Genetic Algorithm's does; a stat-equivalent copy is a new
  family and grows on its own (frontier F7).
- M25's permanent-upgrade journal entry (Lesson Learned) is reused unchanged.
- The settlement census keeps its rule (each permanent instance appears exactly once with
  the same id and `misc`, upgrades reconstructed) and accounts for the Ritual Dagger journal
  entry exactly as for Genetic Algorithm.

## 9. Copied-card paths

The shared stat-equivalent copy (`combat/card_state.rs::stat_equivalent_copy` and its
wrappers) is the only copy owner. M26 checks, for every card kind in the complete unions in
base and upgraded form (plus Searing Blow with 3 upgrades, Ritual Dagger with `misc` 21 and
Genetic Algorithm with `misc` 4), that a copy keeps id, upgrades and `misc`, gets a new
instance and the right copy family, and rebuilds kind defaults (Ethereal, Exhaust, Innate,
X). Producers covered: Anger, Endless Agony, Dual Wield, Nightmare, Tantrum (same instance),
Duplication, Double Tap, Burst, Echo Form, Amplify, Omniscience, Necronomicurse's returned
copy (a fresh base copy, `makeCopy`, not a stat-equivalent one), and Run duplicate (section
8.4).

## 10. Generated and status paths

Every generated child (Shiv, Miracle, Smite, Safety, Insight, Through Violence, Beta, Omega,
Expunger) and every status (Burn, Dazed, Slimed, Wound, Void) is created and played or
resolved by every character through its existing owner. Generated cards keep cost and
legality sentinels: a generated X or Unplayable card is not made playable by being
generated.

## 11. Global corrections planning already found

Each is fixed at the shared owner for every character and disclosed in the M26
reconciliation with the original assertion bytes retained.

| # | Defect | Fix | Accepted assertions amended |
| --- | --- | --- | --- |
| G1 | `CardDefinition::eligible_for` (`content/cards.rs`) excludes 23 ordinary Watcher Common/Uncommon/Rare cards (the M21 finite set, e.g. FlurryOfBlows, Worship, MasterReality) from `OrdinaryReward` and `ShardOrdinaryReward`. That is a finite-profile fence written into source eligibility; the M25 Watcher cards would be eligible and these not | Remove the fence; eligibility follows rarity and colour only | `crates/spire-core/tests/m21_watcher_admission.rs` (the `ordinary_reward` and `shard_ordinary_reward` expectations, which pin "EmptyFist only") |
| G2 | `Catalog::selectable_pool` always returns `IncompletePool` | Return the complete pools of section 7 | `crates/spire-core/tests/run_content.rs::unpopulated_selectable_pools_fail_closed`; the `selectable_pool` assertion in `m21_watcher_admission.rs`; the `IncompletePool` check in `content/defect.rs` tests |
| G3 | The L34 colourless pool is a declared token list | Derive it from the catalog (ordinary colourless minus HEALING) and assert the same 34 members and order | None (members and order unchanged) |
| G4 | `generation::pool` has no Ironclad or Watcher arm (and Silent has no Common arm), so White Noise, Distraction, Discovery, Chrysalis, Metamorphosis, Hello World and Creative AI are rejected for an Ironclad or Watcher; Infernal Blade works for the Ironclad but draws red Attacks for every other character | Section 7.1 arms; Infernal Blade reads the player's pool | None expected; any accepted test pinning an `Unsupported` rejection for these is amended |
| G5 | Watcher and Defect card laws are gated on the player's character (section 6.3), so off-colour stance, Mantra, Scry and orb cards are ignored or fault | Section 6.3 gates | None expected; M21–M25 Watcher and M16–M20 Defect suites must stay green |
| G7 | Next-turn energy is lost for some characters: the non-deferred owner ignores `energized_blue`, and the Watcher recharge ignores both `energized` and `energized_blue` | The next-turn energy law of section 6.3, after M25's routed fix (section 16) | Any accepted test pinning the old energy |
| G6 | Alchemize is legal only for an actual Silent with `m15_profile()`, and the class potion pools exist only for Silent and Defect | Section 6.2 law; Ironclad and Watcher class pools in `items/potion_generation.rs` | Any accepted test pinning Alchemize's `Unsupported` rejection for a non-Silent |

Task M0 and the guard audit (section 6.3) may add rows; each new row follows the same rule.

## 12. Named source frontiers

Each part must be bound to a cited fact (research documents, `data/`, accepted tests or
retained static findings) at Task M0. A part still unsettled is handled as in M25 (design
§5.9): the lane implements the behaviour the cited sources most directly support, records
it as a disclosed project decision with its reasoning, pins it in a test labelled as that
decision, and tells the PM before the freeze. An unsettled frontier never becomes an
`Unsupported` rejection of a legal play.

| # | Frontier | Settled by | Still to bind |
| --- | --- | --- | --- |
| F1 | Any-colour pool | `ShardOrdinaryReward` eligibility (basics and Special excluded); `data/relics/relics.json` PRISMATIC_SHARD "Combat reward screens now contain Colorless cards and cards from other colors" | The pool is drawn per rarity (76/160/84); whether colours are weighted within a rarity |
| F2 | Transform | Event and Neow research §5.1 | Astrolabe's `auto_upgrade` (relic record, M27 caller) |
| F3 | Random curse | Research §5.1, 10 kinds | — |
| F4 | Tags of new cards | `card_tags` owner; HEALING list | Bite and Ritual Dagger tags |
| F5 | Note retrieval | Research E23 | Upper bound of a stored Searing Blow count |
| F6 | Duplicate | Dolly's Mirror relic record | — |
| F7 | Ritual Dagger | Implementation note | Copy-family growth matching Genetic Algorithm's existing law |
| F8 | End-of-turn curses | Burn's owner; Regret note | Order among Decay, Doubt, Shame, Regret and Burn, and against Retain, Ethereal and end-of-turn powers |
| F9 | Normality | Implementation note: autoplayed and duplicated plays count | Where a blocked autoplay goes (a Havoc card, a Mayhem top card, a Double Tap or Burst repeat); interaction with Time Eater's 12-card count and Vault/Conclude |
| F10 | Pain | Card record | Whether autoplayed and duplicated plays trigger it |
| F11 | Necronomicurse | Card record; purgeable predicate | The returned copy's identity (combat-local) and repeated exhaust |
| F12 | Orb cards without slots | Section 6.2 (Consume and Loop wiki notes, Prismatic Shard record, M25 accepted tests) | Consume's slot loss at 0 slots (no change below 0), Fission's and Multi-Cast's X at 0 orbs, Recursion with no orb |
| F13 | Generators use the player's pool | Section 6.2 sources | Hello World and Creative AI for non-Defect (turn-start hooks gated to Defect today) |
| F14 | Apparition | Card record | Intangible stacking with Wraith Form's existing owner |
| F15 | Orb queue position for a non-Defect holding orbs | Defect research §5.1 (passive timing) | Whether the Defect's natural orb phase position (`reactive.rs` `append_before_decision`) is the source position for every character |
| F16 | Alchemize class pools | Silent and Defect accepted pools; M15 ledger G2 | Ironclad and Watcher class potion rarities, and that the limited form excludes only Fruit Juice for every class |

## 13. Native exposure and diagnostics

- **No new public profile or descriptor in M26** (PM ruling 1, section 17).
  Every existing descriptor and fingerprint is unchanged; appended `CardId` variants do not
  enter any existing explicit vocabulary. Mechanical new arms in exhaustive matches in
  `crates/spire-env` are allowed.
- A default-off `m26-diagnostics` feature in `crates/spire-core` (depending on
  `m25-diagnostics`, and on `m20-diagnostics` or any other feature whose scenario helpers it
  needs, as Task M0 binds), compiled like the existing modules under
  `cfg(any(test, feature = "m26-diagnostics"))`, provides constructors used by core tests and, later, by the M28
  profile's tests: a Run of each character's complete Heart scenario with
  `CardAdmission::Complete` and supplied resources validated by
  `CompleteCombatStart`, built as the existing diagnostics do (a template from
  `Run::new_room_scenario`, the normalization owner, then
  `Run::from_validated_combat_resources`). No public supplied-run API and no arbitrary state
  importer.

## 14. Evidence

### 14.1 Grades

The M25 grades apply (M25 design §12.1). M26 uses **Constructed** (an owner checkpoint
built directly) and **Supplied played** (a supplied start, then only legal actions through
the real owners). It has no **Normal played** cases and no **Native** agreement runs,
because it publishes no profile.

### 14.2 Coverage tests

Every expected value is a literal from section 4 or 7 or a cited fact, never computed by
the helper under test.

| Key | File | Covers |
| --- | --- | --- |
| K | `crates/spire-core/src/combat/tests/m26_cards.rs` | The 5 new colourless cards and 12 curse behaviours, base and upgraded, literal values; curse end-of-turn order; Normality, Pain, Necronomicurse |
| U | `crates/spire-core/src/content/tests` or `crates/spire-core/tests/m26_catalog.rs` | Identities, catalog rows, `content::complete` unions 352/14/3, tags, `content::pools` membership and sizes, the G1–G3 corrections |
| W | `crates/spire-core/src/combat/tests/m26_offcolour_laws.rs` | Every section 6.3 site: off-colour stances, Mantra and Divinity, Scry and Watcher powers; orbs at 0 slots and after Capacitor; class C structure tests; Alchemize for all four characters; Infernal Blade per character; overlay inertness |
| X | `crates/spire-core/src/combat/tests/m26_cross_character.rs` | The cross-character matrix (14.3) and every section 6.2 row with literal expectations |
| G | `crates/spire-core/src/combat/tests/m26_generation.rs` | Section 7.1 pools for all four characters; each generator card played by each character draws from the right pool; capture-before-RNG failure |
| Y | `crates/spire-core/src/combat/tests/m26_copies.rs` | Section 9 copy matrix and producers; section 10 children and statuses for every character |
| R | `crates/spire-core/src/run/tests/m26_card_mutations.rs` | Section 8 for every complete permanent card and every character; Run pools 7.2 and draw law 7.3; journal and census for Ritual Dagger |
| S | `crates/spire-core/src/run/tests/m26_supplied.rs` | Supplied played: for each character, one supplied deck mixing off-colour, colourless, event and curse cards played to a combat end with a fixed legal-action rule, then settled into the Run; Ritual Dagger growth persisting once after Victory |

### 14.3 The cross-character matrix

**Basis.** Each combat uses a stripped variant of the character's base Heart context plus
`with_complete_cards()`: no relics, 3 empty potion slots, player HP 80/80, energy 10, one fixed
encounter (Task M0 names it; default a single Jaw Worm at fixed HP), a fixed five-Defend draw
pile, and the card under test in hand. Stripping removes starter-relic effects at combat
start (Burning Blood, Ring of the Snake, Cracked Core, Pure Water). What remains of each
character's starting state (the Defect's 3 empty orb slots, the turn structure of section 6.3
class C) is kept, but it is never compared directly.

**Comparison.** For each card in the complete permanent and generated unions, base and
upgraded, and each character, the test records a **delta**: the public state after the
play minus the same character's state just before it. The recorded fields are player and
enemy HP, Block, energy and powers, the stance, pile contents by kind, and created cards.
The orb fields (slots and orbs) are included only for section 6.2 orb rows.

- **Rows not in section 6.2:** each character's delta must equal the delta for the card's
  owning colour (Ironclad for colourless cards, curses and statuses).
- **Section 6.2 rows:** these have literal expectations per character.

The owning-colour delta is itself pinned by the accepted per-card tests of M10, M15, M20 and
M25 and by key K. A difference is either a section 6.2 law or a defect fixed under rule 2.

**Inertness.** For each character's own cards, the delta and the full public state under
the stripped context with the overlay must equal those without the overlay. In addition,
one unstripped own-colour combat script per character (Task M0 names it) gives byte-equal
public outcomes with and without the overlay.

### 14.4 Freeze and preservation

Nothing is frozen until the source is final: every RED from a helper lane is resolved with
its fix and disclosure; the guard audit is complete; the full `m26_` suites, the M10, M15,
M20 and M21–M25 suites, the training profile suite, the full workspace suite
(`cargo test --workspace --locked --offline --no-fail-fast`) and the feature-on core suite
(`cargo test -p spire-core --features m26-diagnostics --locked --offline`) pass on the
exact tree with no failure other than the inherited, documented ones. The tree's SHA-256
manifest and the suite logs are retained outside the session. There are no once-only
outcome runs.

## 15. Limits

M26 does not claim: any public profile, descriptor or Python surface for the complete card
set; card reward screens, shop rows, Neow, events or their rarity schedules (M28); any relic
or potion hook (M27); Ascender's Bane creation (M30); Note persistence between runs (M34);
Pride (Endless Mode); A1–A20; original-runtime or Java RNG parity; any strategy or win.
Under the overlay, a potion that Alchemize gains is a valid belt item through settlement,
but its use by a character whose base context does not already admit it is M27's potion
lifecycle work.
Constructed and supplied results are never presented as normal traversal.

## 16. M25 interfaces this design depends on

**Re-review trigger.** At Task M0, before any code, Main diffs each interface below against
the merged M25 source using the check in the last column and records the result in the seam
map. If every row matches the M25 version 2 contract, or differs only as the "Accommodated
differences" column allows, execution proceeds. Otherwise Main writes a short M26
amendment (the interface, what changed, the affected sections and tasks, the proposed
change) and sends it through the PM to the same plan reviewer; execution does not continue
until that reviewer returns READY on it. M25 amendment A1 (Act3 history spelling) is already
accommodated: M26 touches no history codec.

| Interface (M25 v2) | Used by M26 for | Task M0 check against merged M25 | Accommodated differences |
| --- | --- | --- | --- |
| The 52 appended `CardId` variants (44 ordinary, 5 children, 3 choices) and their catalog rows | Inventory of section 4.1; M26 appends after them | All present, not `definition_only`, spellings as M25 §5.1–5.2 | Order of appended variants |
| `content::cards::card_tags`, `is_healing`, `is_strike` | Pool exclusions; tags of new cards | HEALING exactly Feed, Reaper, Alchemize, SelfRepair, LessonLearned, Wish, BandageUp-as-listed; STRIKE as M25 §5.3; `has_strike_tag` reads it | Function names; BandageUp tag present or added by M26 |
| `content::watcher::{M25_WATCHER_CARDS, M25_WATCHER_GENERATED, M25_WATCHER_CHOICES, supports_m25_permanent, supports_m25_generated}` | Watcher part of the complete unions; the purple initializer | Sizes 75/8/3 | Different names, same members |
| `CombatItemContext::with_m25_profile()` / `m25_profile()` and the Watcher construction admission that initializes Foreign Influence outputs by their own colour | Pattern and base for `with_complete_cards()` and the colour dispatcher | The admission routes an off-colour Attack through its own colour's initializer | A different internal dispatcher with the same results |
| Foreign Influence pool in `combat/generation.rs` (106 = 43/48/15) | The any-colour Attack row of section 7.1; moved behind `content::pools` without change | Membership and order; whole-proposal retry | Location of the pool function |
| `WatcherAdmission::{M21, M25}` in `run/training.rs` and its threading through `run/diagnostics/m23.rs` | Pattern for the new `CompleteCombatStart` validator (not a rename) | Signature and the validator it calls | A different enum name |
| The permanent-upgrade journal entry (proposed `PermanentMutation::Upgrade`) and the census extension | Reused unchanged; Ritual Dagger extends the `misc` journal next to it | Variant exists; census accounts for it | Actual journal type name |
| The Hand of Greed / FameAndFortune gold path | Unchanged; covered by matrix X | Gold reaches the Run once | — |
| Plated Armor hook admitting the Watcher (M25 §5.8) | Character-independence of shared powers | The gate no longer names a character, or names all four | Other gate wording with the same results |
| `RunRngScope::M25CardOffer`, `M25RelicCapture` appended last | M26 appends its three scopes after them | Position and spelling | None (spellings are RNG keys) |
| M25's sampler caller `m25_content::sample_capture_row_with` with `purple_candidates` (19/35/17) | Must stay RNG-identical under the section 7.3 generalization; `purple_candidates` moves behind `content::pools` as `Ordinary(Watcher)` | Signature, families and the RNG keys it uses | Different helper names, same families and keys |
| The M25 arm of `run::mutations::supported_in` | Unchanged under `CardAdmission::Profile`; the complete arm sits beside it | The arm admits exactly the M25 union | Different predicate expression with the same members |
| Every `m25_profile()` gate in `combat/**` (for example Tantrum's shuffle in `combat/piles.rs`) | Class L sites of section 6.3 | List them | Any number; each gets `|| complete_cards()` |
| M25's routed global fix: the Watcher recharge applies next-turn energy (`energized`, at least; the Flying Knee case reachable through Foreign Influence) | The next-turn energy law of section 6.3; G7 completes any part the fix leaves out | On merged M25, a Watcher's Flying Knee and Outmaneuver give their energy at the next turn start, once | The fix also covers `energized_blue` or the non-deferred owner (M26's G7 then shrinks accordingly); if M25 merged without the fix, M26 implements it as G7 and discloses it, with no amendment needed |
| `m25-diagnostics` feature and the M25 complete Watcher Heart context | Base of `m26-diagnostics` and of the Watcher complete overlay | Feature exists; context constructor | Different constructor name |
| M25's per-card and Foreign Influence tests (`m25_watcher_*`) green | Owning-colour baseline for matrix X | Present and green on the merged tree | Different test names |

## 17. PM rulings on the planning questions

Version 1 asked four questions. The PM ruled on all of them
(`/home/klottick/.local/share/spire-gym/evidence/m26-execution-20261031/pm-open-question-rulings-v1.json`,
SHA-256 `a039ddf117a3491c6ccf5d14c6f9c30633e8f7ef5e75228ad11d4560881ec5a6`), and these
documents apply the rulings:

1. **Public profile:** none in M26; core constructed and supplied tests; the full Run API is
   M34 (sections 1.1 and 13).
2. **Reward rarity schedule and shop rows:** deferred to M28 (section 1.1).
3. **Pride:** excluded (Endless Mode only, not normally reachable) (section 4).
4. **Ascender's Bane:** behaviour now, creation in M30 (sections 4.3 and 15).

The PM's guidance on review v1 (Alchemize owned in M26 if the card is in scope; stance,
Mantra and orb gates designed as capability or state gates with provable inertness) is
applied in sections 1.1, 6.2–6.4 and 14.3. No question is open.

## 18. Planning record

Version 1, its research reports (catalog inventory; owners, seams and precedents), the
builder check and its resolutions, the decisions log and `SHA256SUMS` are retained at
`/home/klottick/.local/share/spire-gym/evidence/m26-execution-20261031/planning-v1/`.
Version 2, the review v1 disposition table and `SHA256SUMS` are at
`/home/klottick/.local/share/spire-gym/evidence/m26-execution-20261031/planning-v2/`.
Version 3, the review v2 disposition table and `SHA256SUMS` are at
`/home/klottick/.local/share/spire-gym/evidence/m26-execution-20261031/planning-v3/`.
