# M25 finite Watcher A0 Acts 1–4 and Heart Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development or
> superpowers:executing-plans to carry out this plan task by task, after the one
> independent plan review returns READY, M24 is merged and the PM releases execution. Steps use
> checkbox (`- [ ]`) syntax. Use meaningful TDD and one fresh independent final
> milestone review; do not add per-task review swarms.

**Goal:** Continue the exact M24 Watcher A0 Run through the Act4 door, Rest, Shop, Spire
Shield and Spear, the Corrupt Heart and the final Proceed, and complete the Watcher at A0:
all 75 ordinary cards base and upgraded, all generated children and Wish options, the
106-kind Foreign Influence closure, all nine exclusive relics and three exclusive potions,
and the seven reachable events with Watcher semantics.

**Architecture:** Add `RoomScenario::M25WatcherHeart` and route it through the shared
full-Act3, Watcher-content and Act4 owners, keeping M24's Acts 1–3 producers so the prefix is
exact. Implement every new card, power, child and relic once in the shared combat and item
owners behind an additive complete-Watcher capability, and give the new Act4 producers the
complete roster. Add an additive native profile through the existing schema, session,
replay and batch owners, and gather evidence with the shared rollout and a default-off
`m25-diagnostics` feature.

**Tech Stack:** The existing locked Rust workspace (`spire-core`, `spire-env`), serde with
the existing strict raw JSON parser, int64 native observation transport and checked Run RNG.
No new dependency, no Python rebuild, no original-game execution.

**Spec:** [M25 design](../specs/2026-10-30-m25-watcher-heart-design.md). "Design §N" below
refers to it. The design pins every roster, value, law, loadout, fixture and limit; this
plan repeats them only where a step needs them inline.

## Global constraints

- Work only in the checkout and branch the PM assigns for execution (expected base: `main`
  at the merged M24 revision plus these two planning documents). Preserve unrelated work.
  Each helper lane works in its own worktree from the snapshot named in its handoff.
- Execution cannot start until M24 is merged. Task M0 applies the re-review trigger of
  design §15: if any depended-on M24 interface differs from M24 version 2 in a way the
  documents do not already accommodate, Main writes a short amendment and execution waits
  for the same plan reviewer's READY on it.
- One authoritative Rust engine. No copied Run, combat, encounter, generation, offer,
  capture, encoder, session, replay, scheduler, rollout, caller loop or validator. No second
  inventory, census or journal. No public supplied-run or import API.
- No installs, new dependencies, Python rebuild or original-game execution.
- Scope is playing the game, not winning it. No test may require a normal-start Act 3
  entry, key set, Act 4 entry or Heart Victory. Canonical outcomes are policy results.
- A shared-owner defect found in any lane is fixed at the shared law for all characters. An
  accepted assertion that pinned wrong behaviour is amended with a disclosed diff, the
  original bytes are retained, and a regression must fail under any narrower fix. The
  Content lane fixes defects in its files and reports Run-side defects to Main; the Native
  lane reports every core defect to Main.
- Keep combat-v1, the default and standalone profiles and every accepted M9–M24 profile,
  descriptor, fingerprint, wire value and test unchanged, except mechanical new
  exhaustive-match arms and disclosed global corrections. The finite M21 Watcher lists and
  the M22–M24 unions stay finite; M25's complete capability is additive (design §5.7).
- Every expected value is a literal from design §5 or a cited source fact (design §5.9),
  never computed by the helper under test. A frontier part that stays unsettled after Task
  M0 is implemented as the behaviour the cited sources most directly support, pinned in a
  test labelled as a disclosed project decision, recorded in the reconciliation with its
  reasoning, and reported to the PM before the freeze. It never becomes an `Unsupported`
  rejection of a legal play.
- Freeze cases, caller, loadouts, fixtures and source before any outcome, behind the freeze
  gate of design §12.8. Run each prescribed case once. Keep every Death, unfinished budget,
  operational error and missing checkpoint.
- Native Rust results do not establish Python delivery or original-runtime parity.
- Cargo commands use one task-owned `CARGO_TARGET_DIR` outside the repository and
  `--locked --offline`. A compile error from a missing symbol is not a RED: watch each new
  test fail for a behavioural reason after minimal wiring.
- Commit at the end of each task, staging only that task's explicit paths (never a blanket
  `git add`), after `git diff --cached --check` passes. Helpers commit on their own branches
  and deliver patch series; Main integrates them. Pushing and PRs belong to the PM.

## Review Focus

These five conditions follow from the design but are easy to miss. Each has a test in its
owning task.

1. **Zero-X and empty-pile cards.** ConjureBlade and Collect at X = 0, Omniscience with an
   empty draw pile and Scrawl with nothing left to draw each finish, exhaust as their source
   says, count as one played card and fault nothing; a zero-hit Expunger is created and
   playing it deals no damage. Test `m25_state_zero_x_and_empty_pile_edges` (Task C3).
2. **Creation into a full hand.** With 10 cards in hand, the Foreign Influence choice,
   Deus Ex Machina's Miracles and Collect's upgraded Miracle go to the discard pile, Master
   Reality does not upgrade the overflowed copy, and Insight, Through Violence, Beta, Omega
   and Expunger still go to the draw pile. Test `m25_generation_full_hand_overflow`
   (Task C5).
3. **Lesson Learned with nothing to upgrade.** A fatal Lesson Learned when every permanent
   card is already upgraded or the only other card is Parasite upgrades nothing, makes no
   RNG request and commits nothing; a kill of a minion, a half-dead Darkling or Awakened One
   phase 1 is not fatal. Test `m25_content_lesson_learned_no_eligible_card` (Task M5).
4. **Act4 producers with exhausted or unaffordable rows.** With Damaru, Duality and
   CloakClasp owned, the Shield/Spear reward has no relic row and makes no RNG request but
   still offers Proceed; with Melange owned the merchant omits it; with 0 gold every priced
   row is disabled and Leave stays legal. Test
   `m25_act4_producers_exhausted_and_unaffordable` (Task M3).
5. **Stacked turn endings.** Vault or Conclude played as Time Eater's twelfth card ends the
   turn once; Omniscience choosing Vault ends the turn after the first play and suppresses
   the second; Blasphemy then Vault kills at the extra turn's start. Test
   `m25_powers_stacked_turn_endings` (Task C4).

## Lanes, sub-phases and file ownership

One main senior and two helper lanes. Main owns every Run seam, integrates helper
deliveries with recorded pre- and post-integration manifests, and owns the evidence and
documents. Assistants do tool calling, search, parsing and builds for each lane.

Paths start at the repository root. `core/` means `crates/spire-core/src/`; `env/` means
`crates/spire-env/src/`.

| Lane | Owns (creates or edits) | Must not edit |
| --- | --- | --- |
| **Main** (senior) | `core/run/**` (including `rooms/{mod,act3,act3_end,act4,shop,rewards,rest,transition,m23_content}.rs`, new `rooms/m25_content.rs`, `run/{mod,rng,mutations,combat,training}.rs`, `diagnostics/mod.rs`, `diagnostics/m23.rs` for the admission-profile parameter only, new `diagnostics/m25.rs`, new `run/tests/m25_watcher_heart.rs`, new `run/tests/m25_watcher_content.rs` and their `mod` lines); all of `core/items/mod.rs` (single owner: the M25 inventory context, its validation and any item facade change); `crates/spire-core/Cargo.toml` (feature only); `crates/spire-core/examples/**` (`m13_support/mod.rs`, new `m25_support/{mod,policy,inputs,fixtures}.rs`, new `m25_watcher_heart.rs`, new `m25_supplied_diagnostics.rs`); new `crates/spire-core/tests/m25_watcher_heart.rs`; all docs | Content-lane and Native-lane files |
| **Content** (helper 1) | `core/combat/**` (including `cards/watcher.rs`, new `cards/watcher/` submodules if the file grows, the shared engine, turn, hook, power, effect, pile, card-state, generation and offer owners, new `combat/tests/m25_watcher_{direct,state,powers,generation,foreign_influence,interactions}.rs` and their `mod` lines); `core/content/**` (identities, catalog metadata, `content/watcher.rs`); `core/items/relics.rs` and every other `core/items/*` file except `items/mod.rs`, which it never edits (a facade change there is a written request to Main); new `core/items/tests/m25_watcher.rs` and its `mod` line | `core/run/**`, `core/items/mod.rs`, examples, `crates/spire-env/**` |
| **Native** (helper 2) | New `env/run_schema_m25.rs`; M25 arms in `env/{lib,run_observation,run_session,run_replay,run_batch}.rs`; new `crates/spire-env/tests/run_m25_{schema,session,replay,batch}.rs`; mechanical arms in `crates/spire-env/tests/{run_m10_session,run_m10_batch}.rs`; the `m25-diagnostics` dev-dependency feature in `crates/spire-env/Cargo.toml` | Any `crates/spire-core` file |

If a lane needs a change in a file it does not own, it sends the owner a written request
with the reason and a proposed diff; it never edits that file.

| Sub-phase | Main | Content | Native |
| --- | --- | --- | --- |
| 1. Seams | M0, M1 → H0, H1 | C1 → HC0 | — |
| 2. Build | M2, M3, M4 → H2 | C2, C3, C4, C5, C6 → HC1 | N1, N2 on H1, HC0 and H2 (everything not needing card behaviour) |
| 3. Compose and integrate | M5 (integrates HC1 → H2b, then HC2 and H3) | C7 → HC2 | finish N1 and N2 on H2b → H3 |
| 4. Evidence and close | M6 → H4, M7 | fixes only | N3 |

### Handoff points

| Handoff | From → to | Contents | Gate |
| --- | --- | --- | --- |
| H0 | Main → both | Task M0 seam map: bound M24 interfaces, every `is_m24` branch and its M25 arm, the `uses_act4` re-check, the Foreign Influence support enumeration, and the source fact (or "open") for each frontier F1–F16 | Content lane starts C1; Native lane reads it |
| H1 | Main → both | Snapshot compiling `RoomScenario::M25WatcherHeart`, `is_m25`, the predicate membership, `CombatItemContext::with_m25_profile()` and `m25_profile()`, with `cargo test -p spire-core --lib --locked --offline m25_start` green | Content lane may read `m25_profile()`; Native lane starts N1 |
| HC0 | Content → Main, Native | Snapshot compiling every new `CardId` and `RelicId`, catalog rows, `content::watcher` complete lists and predicates, the construction admission under `m25_profile()`, and the combat-side journal variant for a permanent upgrade, with `m25_identity_` green | Main starts M3; Native adds vocabulary |
| H2 | Main → Native | Snapshot with the Act4 door, route, producers, unions and the full `m25-diagnostics` API, with every Main `m25_` core test green except those waiting on HC1 | Native writes and runs every N1 and N2 test that does not need card behaviour |
| HC1 | Content → Main | All 47 cards, 5 new children, Wish, the Foreign Influence closure and 4 relics, with C2–C6 tests green and the list of each first RED with its cited fact | Main integrates in M5 |
| H2b | Main → Native | The integrated snapshot after Task M5's first step (H2 plus HC1), with its manifest, and the HC1-dependent `m25-diagnostics` stages (Foreign Influence, Wish, Omniscience) re-run green on it | Native completes the card-behaviour tests of N1 and N2 on this snapshot |
| HC2 | Content → Main | Composition tests T1–T12 and W15–W20 with every RED, its fix and disclosure | Main integrates in M5 |
| H3 | Native → Main | Env files plus focused test output on the H2b snapshot | Main integrates unchanged except for agreed fixes |
| H3b | Native → Main, only if needed | A re-delivery when HC2's fixes make an env test fail on the integrated tree | Before the freeze gate |
| H4 | Main → Native | Frozen once-only transcripts from Task M6 | Native agreement runs on recorded actions only |

---

## Sub-phase 1: Seams

### Task M0: Bind source and map seams (Main)

**Files:** none in the repository. Write the seam map and source receipt to
`/home/klottick/.local/share/spire-gym/evidence/m25-execution-20261030/`.

- [ ] Run `pwd`, `git status --short --branch`, `git branch --show-current` and
  `git rev-parse HEAD`. Confirm HEAD is the merged M24 revision the PM names plus only the
  two planning documents. Claim the execution Bead with your own actor.
- [ ] Record a SHA-256 manifest of all tracked files, retained outside the session.
- [ ] Apply the re-review trigger of design §15: diff every row of its table against the
  merged M24 source using that row's check, and record the actual owner, signature and
  result for each in the seam map. If every row matches M24 version 2, or differs only in a
  way the row's "Accommodated differences" column names, continue. Otherwise write a short
  M25 amendment (the interface, what changed, the affected design sections and plan tasks,
  and the proposed change), send it through the PM to the same plan reviewer, and do not
  start Task M1 until that reviewer returns READY on it.
- [ ] Record exactly where merged M24 validates supplied resources (the M23 helpers in
  `core/run/diagnostics/m23.rs`, its own `m24.rs`, or elsewhere), with path and line; Task M4
  threads the M25 admission profile through that point.
- [ ] List every `is_m24` branch in `crates/spire-core` and `crates/spire-env` and record
  the M25 arm for each, following design §4.4. A branch that affects Acts 1–3 and is not in
  that table gets the M24 behaviour for M25.
- [ ] Re-read every `uses_act4` call site and confirm the design §4.3 table on the merged
  source. A new pre-door site that is not inert for Watcher content stops the plan; report
  it to the PM.
- [ ] Retain the six M24 canonical receipts and their hashes; they are Task M6's expected
  endpoints.
- [ ] Enumerate the Foreign Influence support from the native catalog: ordinary Attacks of
  Common, Uncommon and Rare rarity in every colour, excluding the HEALING-tagged kinds
  (record how the catalog marks them). Compare with design §5.3 (43/48/15, 106 kinds) and
  record any mismatch for resolution at its owner.
- [ ] For each frontier F1–F16 (design §5.9), record the cited fact that settles it or mark
  the open part. Confirm the action representation of Foreign Influence offers, Wish
  choices and Omniscience selections (for Task M6's caller rule), the Act3 target codec
  spellings, the existing `CardTag` metadata and every existing STRIKE or HEALING list
  (for Task C1's single card-tag owner), and the Hand of Greed gold path.
- [ ] Publish H0.

### Task M1: Profile, predicates, prefix and inert pre-door sites (Main) → H1

**Files:** `core/run/rooms/mod.rs`; `core/run/mod.rs` (re-exports); `core/run/combat.rs`
(context selection); `core/items/mod.rs` (seam only); `core/run/rooms/{shop,m23_content,act3,act3_end}.rs`
(M25 arms from Task M0's list); new `core/run/tests/m25_watcher_heart.rs` and its `mod`
line in `core/run/tests/mod.rs`.

**Interfaces:**
- Produces `RoomScenario::M25WatcherHeart`; `pub(crate) fn is_m25(state: &RunState) -> bool`;
  `is_m25` added to `uses_full_act3`, `uses_watcher_content` and `uses_act4`.
- Produces `pub(crate) fn with_m25_profile(self) -> Self` and
  `pub(crate) fn m25_profile(&self) -> bool` on `CombatItemContext`, with private field
  `watcher_heart: bool`. Until Task M3, validation under the flag uses the M24 relic union;
  Task M3 switches it to the M25 union. `Debug` prints `watcher_heart` only when true.
  `run/combat.rs` selects it when `is_m25`.
- Produces M25 fault labels `run.m25.room`, `run.m25.encounter`, `run.m25.act3.end` in the
  existing selectors.

- [ ] Write `m25_rejects_unsupported_creation`: a non-Watcher character, A1 and full Neow
  each return `Unsupported("M25 requires Watcher A0 short Neow")` before mutation; a
  not-fully-unlocked profile returns the existing `"profile must be fully unlocked with Act 4
  enabled"`.
- [ ] Write `m25_start_and_prefix_match_m24_played` (design §4.2 check 1): build M24 and M25
  from the same Watcher A0 config, `CreationProfile`, seed, partition and ordinal; at each
  step take the first legal M24 candidate under one fixed test rule, apply the same action
  to M25, and assert equal HP, max HP, gold, deck, relics and counters, potions, keys,
  records, captures, RNG bytes and counters, maps, boss orders, committed events and public
  choices (room screens compared through `public()`), until M24 is terminal, at `Act4Ready`
  or at 4096 steps. Report how far it got.
- [ ] Write `m25_act3_owner_rows_match_m24` (check 2): from equal constructed Act3 states
  one edge before a merchant with counter 0, a merchant with counter 1, an Act3 elite, the
  floor-43 chest, Falling and Tomb, enter each room in both profiles; rows, captured relic,
  Falling captures and RNG are equal, and the scopes are `run.m24.relic.capture` (Act3) and
  `run.m23.relic.capture` (Act2) for both.
- [ ] Write `m25_pre_door_act4_sites_are_inert`: for a Watcher merchant in each of Acts 1–3
  the M25 RNG after the room equals M24's; a Watcher chest uses the Watcher branch in both;
  at a campfire with Ruby held the public Recall row is Disabled in both and choosing it is
  rejected in both. State that the stored row status is the only internal difference.
- [ ] Write `m25_combat_context_selects_m25_profile`: an M25 combat enters with
  `m25_profile()` true and `m24`'s flag false; M22, M23 and M24 combats keep their
  contexts and their `Debug` text is unchanged.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m25_rejects`, then the filters
  `m25_start`, `m25_act3_owner`, `m25_pre_door`, `m25_combat_context`, through RED, minimal
  change, GREEN. Then run `cargo test -p spire-core --lib --locked --offline m24_` and the
  `m23_` and `m22_` filters once.
- [ ] Commit; publish the snapshot as H1 with its manifest.

### Task C1: Identities, metadata and the complete capability (Content) → HC0

**Files:** `core/content/ids/cards.rs`, `core/content/cards.rs`, `core/content/ids/items.rs`,
`core/content/items.rs`, `core/content/watcher.rs`; `core/combat/cards/watcher.rs` and the
combat-entry branch in `core/combat/engine.rs` (construction admission); the combat-side
exit journal owner (the file Task M0 names) for one new variant; new
`core/combat/tests/m25_watcher_generation.rs` and its `mod` line.

**Interfaces:**
- Consumes H0, and H1's `m25_profile()` for the construction admission step.
- Produces appended `CardId` variants, without renumbering any existing identity: the 44
  ordinary kinds without an ID (design §5.1 tables minus Consecrate, EmptyBody and
  EmptyMind), the 5 children Insight, ThroughViolence, Beta, Omega and Expunger, and the 3
  choice-only options BecomeAlmighty, FameAndFortune and LiveForever.
- Produces appended `RelicId::{Damaru, Duality, CloakClasp, Melange}` with tiers Common,
  Uncommon, Rare and Shop.
- Produces in `content/watcher.rs`: `pub const M25_WATCHER_CARDS: &[CardId]` (75),
  `pub const M25_WATCHER_GENERATED: &[CardId]` (8: Miracle, Smite, Safety and the 5 new
  children), `pub const M25_WATCHER_CHOICES: &[CardId]` (3),
  `pub fn supports_m25_permanent(id: CardId) -> bool` and
  `pub fn supports_m25_generated(id: CardId) -> bool`. `M25_WATCHER_CARDS` is built from the
  M21 list plus the 47, not as a second copy of the 28. `M21_WATCHER_CARDS`,
  `M21_WATCHER_GENERATED`, `supports_permanent` and `supports_generated` are unchanged.
- Produces the construction admission (design §5.7): when the combat was entered with
  `m25_profile()`, `cards::watcher::initialize_profile` admits every kind in
  `M25_WATCHER_CARDS` and `M25_WATCHER_GENERATED` with Watcher initialization and the design
  §5.1 and §5.2 costs and values, and keeps M24's Curse rule; Task C5 extends it to Foreign
  Influence outputs. Without the flag the M21 law, its rejections and its `Debug` text are
  unchanged.
- Produces the journal variant for a permanent upgrade requested during combat (proposed
  `PermanentMutation::Upgrade { instance: CardInstanceId }`; Task M0 binds the actual
  journal type). The Wish gold gain reuses the existing combat gold path.
- Produces the single card-tag owner of design §5.3, proposed
  `pub fn card_tags(id: CardId) -> &'static [CardTag]` in `content/cards.rs`, with
  `pub fn is_healing(id: CardId) -> bool` and `pub fn is_strike(id: CardId) -> bool` reading
  it. For Defect cards it returns the existing `BlueCardMetadata` tags unchanged. For every
  other kind it returns `CardTag::Healing` for exactly Feed, Reaper, Alchemize, LessonLearned,
  Wish and BandageUp (SelfRepair already carries it in its Defect metadata), citing
  `docs/research/2026-10-19-act1-native-contract.md`'s HEALING exclusions list, and
  `CardTag::Strike` for exactly Strike, StrikeGreen, StrikePurple, PerfectedStrike,
  PommelStrike, TwinStrike, WildStrike, SwiftStrike, SneakyStrike and WindmillStrike
  (StrikeBlue, ThunderStrike and MeteorStrike already carry it in their Defect metadata),
  citing the red and colourless research's Perfected Strike paragraph. The colourless
  generator's existing Bandage Up exclusion is not changed.

- [ ] Write `m25_identity_catalog_rows_and_capability`: the catalog has 75 purple ordinary
  rows partitioned 4/19/35/17 by rarity and 27/36/12 by type with the design §5.1
  rarities and types; the 8 children are generated-only and the 3 options choice-only;
  `is_healing` is true for exactly Feed, Reaper, Alchemize, SelfRepair, LessonLearned, Wish
  and BandageUp and false for HandOfGreed; `is_strike` is true for exactly the thirteen
  STRIKE kinds and false for Thunderclap; for every Defect card `card_tags` equals its
  `BlueCardMetadata` tags; the M20 Defect and colourless generator pools are unchanged; the four new relics have their tiers; `supports_m25_permanent` admits exactly the 75,
  `supports_m25_generated` exactly the 75 and the 8; the M21 predicates still admit exactly
  their 28 and 3; no new identity collides with an existing spelling.
- [ ] Write `m25_identity_construction_admission`: under an M25 combat context every kind in
  `M25_WATCHER_CARDS` and `M25_WATCHER_GENERATED` constructs in base and upgraded form with
  its design §5.1 or §5.2 cost, and a Parasite constructs with no Watcher fields; under the
  M22, M23 and M24 contexts the 47 new kinds and 5 new children are still rejected, and a
  Status in a starting deck is rejected under every context.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m25_identity` through RED,
  minimal change, GREEN, then `cargo test -p spire-core --lib --locked --offline m21_` once.
- [ ] Commit on the Content branch; publish the snapshot as HC0.

## Sub-phase 2: Build

### Task M2: Act4 door, route, Heart Proceed, keys and labels (Main)

**Files:** `core/run/rooms/{act3_end,act4,rest}.rs`; `core/run/combat.rs` (the Act4
settlement and Heart Victory labels it hard-codes); `core/run/training.rs` (the admission
profile); new `core/run/diagnostics/m25.rs` and its `mod` line in
`core/run/diagnostics/mod.rs`; `crates/spire-core/Cargo.toml` adds
`m25-diagnostics = ["m24-diagnostics"]`; `core/run/tests/m25_watcher_heart.rs`.

**Interfaces:**
- Consumes H1.
- Produces, in `act4.rs`, `pub(in crate::run) fn fault_label(state: &RunState, site: Act4FaultSite) -> &'static str`
  with `Act4FaultSite { Transition, Room, Encounter, Settlement, HeartVictory, HeartProceed }`,
  returning the design §7 M25 labels for M25 and today's strings for M10, M15 and M20.
- The selector covers every Act4 label site, including `run.m10.act4.settlement` and
  `run.m10.heart.victory`, which `core/run/combat.rs` hard-codes.
- Produces, in `core/run/diagnostics/m25.rs` under
  `#[cfg(any(test, feature = "m25-diagnostics"))]`, M25's own Act3 boss Victory source
  (design §12.7). It does not use any M24 diagnostics helper:

  ```rust
  pub enum M25DiagnosticBoss { AwakenedOne, TimeEater, DonuAndDeca }
  pub(super) fn act3_victory_source(scenario: RoomScenario, boss: M25DiagnosticBoss,
      keys: [bool; 3], resources: Option<AuthoredRunStart>, admission: WatcherAdmission)
      -> Result<Run, RunError>;
  ```

  `scenario` is `RoomScenario::M25WatcherHeart`, or `RoomScenario::M24WatcherAct3` for the
  prefix check. Resources are canonical when `None`; otherwise they are checked with
  `normalized_for(admission)` before `Run::from_validated_combat_resources`.
- Produces, in `core/run/training.rs`, `pub enum WatcherAdmission { M21 }` and
  `normalized_for(self, admission: WatcherAdmission)` on `M21WatcherCombatStart`, with the
  existing `normalized()` meaning `normalized_for(WatcherAdmission::M21)` and its behaviour
  unchanged. Task M3 adds the `M25` variant once the M25 unions exist; this task's tests use
  canonical resources only.

- [ ] Write `m25_act3_victory_source_runs_the_real_ending`: for each boss and each of the
  eight key masks, `act3_victory_source(RoomScenario::M25WatcherHeart, boss, keys, None, ..)`
  returns a Run with zero accepted actions at act 3, floor 50, an Act3 map whose current,
  visited floor-50 Boss node holds the boss and whose icon is the boss, `combat_progress`
  entered and settled at 1, `last_combat` and one `CombatFinished` event for the Victory
  record, the given keys, the `Act3BossComplete` screen with one Proceed, and the shared
  `Act3EndFrame` at `AwaitingSource`; accepting Proceed and stages 1–3 reaches stage 4. A
  supplied start holding a card outside the M21 admission set is rejected under
  `WatcherAdmission::M21` before any Run state is returned (Task M4 adds the M25 cases).
- [ ] Write `m25_ending_matches_m24_until_stage_four` (design §4.2 check 3): for each Act3
  boss and each of the eight key masks, the M24-scenario and M25-scenario sources from
  `act3_victory_source` are equal apart from metadata, then run Proceed and stages 1–3
  identically; with all keys M24 shows `Act4Ready` with zero candidates and M25 shows stage 4
  with one `Act3Continue{stage:4}`; every other mask commits `Act3NoKeys` once with reward 0
  in both.
- [ ] Write `m25_door_commits_once`: accepting stage 4 with all keys sets act 4, clears Act1–3
  progress, heals to max HP exactly once through the healing owner, publishes the fixed map
  (Rest 51, Shop 52, Elite 53, Boss 54) and commits `ActAdvanced{from:3,to:4}` with reward 0;
  a stale or repeated door choice changes nothing; a forced post-accept failure is retained
  once under `run.m25.act4.transition` with no second heal.
- [ ] Write `m25_act4_route_and_heart_proceed`: the Rest 51 campfire shows Recall disabled;
  the Heart cannot be entered before the Shield and Spear Victory record and closed rewards;
  a constructed Heart Victory shows `HeartVictory` with one Proceed and no reward screen;
  budget 0 there is nonterminal; accepting Proceed commits floor 55, `TrueVictoryRoom`,
  terminal `HeartVictory` and +110 once; Death and EmptyDeck in Act 4 commit their own
  terminals.
- [ ] Write `m25_keys_earned_through_shared_owners`: on constructed Act 3 states, Recall
  acquires Ruby (+5 once), Sapphire at the chest acquires Sapphire instead of the relic, and
  the Emerald row after a burning-elite Victory acquires Emerald; a second attempt for a held
  key is rejected; the Act 4 map has no burning node.
- [ ] Write `m25_act4_fault_labels_name_m25`: forced faults at each `Act4FaultSite` retain the
  M25 label; one M20 fault keeps its existing label.
- [ ] Run each filter (`m25_act3_victory_source`, `m25_ending`, `m25_door`,
  `m25_act4_route`, `m25_keys`, `m25_act4_fault`) through RED, minimal change, GREEN. Then
  run the M10, M15 and M20 Act4, key and ending tests once
  (`cargo test -p spire-core --lib --locked --offline m10_act4`, `m10_keys`, `m15_`,
  `m20_defect_content`) and the `m24_` filter.
- [ ] Commit.

### Task M3: Act4 producers and admission unions (Main)

**Files:** new `core/run/rooms/m25_content.rs`; `core/run/rooms/{shop,rewards,m23_content}.rs`;
`core/run/rng.rs`; `core/run/mutations.rs`; `core/run/training.rs` (the `M25` admission
variant); `core/items/mod.rs` (switch the M25 validation to the M25 union);
`core/run/tests/m25_watcher_heart.rs`.

**Interfaces:**
- Consumes HC0 and the shared sampler `m15_content::sample_capture_row_for`.
- Produces in `m25_content.rs`: `M25_WATCHER_HEART_CARDS` (76: the complete roster plus
  Parasite), `M25_WATCHER_HEART_RELICS` (23: the M24 union followed by Damaru, Duality,
  CloakClasp, Melange), `M25_WATCHER_HEART_POTIONS` (the M22 seven), `GENERATED_ONLY` (8),
  `CHOICES` (3), `STATUSES` (5), `RELIC_TAXONOMY_ADDITIONS` (the four new relics with their
  reference tiers), `ACT4_ELITE_RELICS = [Damaru, Duality, CloakClasp]`,
  `ACT4_SHOP_RELIC = Melange`, `ACT4_SHOP_RELIC_PRICE = 150`,
  `ACT4_SHOP_CARD_PRICES = [50, 75, 150]`, `ACT4_SHOP_POTION = StancePotion`,
  `pub(in crate::run) fn purple_candidates(rarity: CardRarity) -> &'static [CardId]` and
  `pub(in crate::run) fn sample_capture_row(rng: &mut RunRng, semantic: &[u64], rarities: &[CardRarity]) -> Result<Vec<CardId>, RunError>`,
  which calls an internal
  `fn sample_capture_row_with(rng: &mut RunRng, semantic: &[u64], rarities: &[CardRarity], family: fn(CardRarity) -> &'static [CardId]) -> Result<Vec<CardId>, RunError>`
  with `purple_candidates`. `sample_capture_row_with` first requires family sizes 19/35/17
  (the M20 wrapper pattern) and then delegates to `sample_capture_row_for`, which rejects an
  empty family or a definition-only kind. Lists are built from the content owners, never as
  copied literals.
- Produces `WatcherAdmission::M25`: `normalized_for(WatcherAdmission::M25)` admits exactly the
  M25 permanent, relic and potion unions, with the same structural checks as `M21`.
- Produces `RunRngScope::M25CardOffer` (`run.m25.card.offer`) and
  `RunRngScope::M25RelicCapture` (`run.m25.relic.capture`), appended last;
  `relic_capture_scope` returns the M25 scope only for M25 in act 4.
- Produces `mutations::supported_in` admitting `M25_WATCHER_HEART_CARDS` for M25.

- [ ] Write `m25_content_unions_are_closed`: exact 76/8/3/5/23/7 members as design §5.6;
  every M25 card is upgradable once except Parasite; M22, M23 and M24 admission is unchanged
  (no new card or relic admitted there); the M25 combat context accepts the 23 relics and
  rejects any other.
- [ ] Write `m25_act4_merchant_rows`: entering floor 52 shows one Common, one Uncommon and one
  Rare, distinct, from the purple families at 50/75/150, sampled under `run.m25.card.offer`
  with key `[4, 52, node]` plus the slot; Melange at 150; StancePotion at 50; purge at the
  lifetime cost; Leave. The RNG is committed once; re-reading the screen does not resample.
- [ ] Write `m25_act4_elite_rewards_and_relic_capture`: a constructed Shield and Spear Victory
  prepares Gold (25 plus the bounded roll), a card child of one Common and two distinct
  Uncommons with Skip under `run.m25.card.offer` keyed `[4, 53, node, visit, kind]`, the
  potion by the global visit modulo 7, and one relic from Damaru, Duality, CloakClasp under
  `run.m25.relic.capture` keyed `[4, 53, node, visit, 0]`; no Emerald row; decline, stale
  claims and a publication fault never reroll.
- [ ] Write `m25_act4_producers_exhausted_and_unaffordable` (Review Focus 4) as described
  there.
- [ ] Write `m25_act4_card_families_complete_or_reject`: `purple_candidates` returns exactly
  19, 35 and 17 nonbasic purple kinds, including LessonLearned and Wish;
  `sample_capture_row_with` given a test family function that drops one member of a tier
  returns `RunError::Unsupported` before any RNG request, leaving the RNG unchanged (the
  wrapper's size check); `sample_capture_row_for` given a family that contains a
  definition-only kind returns `RunError::Unsupported` before any RNG request (the shared
  sampler's check).
- [ ] Write `m25_admission_profile_reads_the_m25_unions`: `normalized_for(M25)` accepts a
  start using any member of the M25 unions and rejects one card, relic or potion outside
  them; `normalized_for(M21)` and `normalized()` keep rejecting the new cards and relics
  exactly as before.
- [ ] Run each filter (`m25_content_unions`, `m25_act4_merchant`, `m25_act4_elite`,
  `m25_act4_producers`, `m25_act4_card_families`, `m25_admission_profile`) through RED,
  minimal change, GREEN. Then run the M22–M24 reward, shop, capture and RNG tests once.
- [ ] Commit.

### Task M4: `m25-diagnostics` constructors and supplied validation (Main) → H2

**Files:** `core/run/diagnostics/m25.rs` (created in Task M2); `core/run/diagnostics/m23.rs`
(the admission-profile parameter only); `core/run/tests/m25_watcher_heart.rs`.

**Interfaces** (all behind `#[cfg(any(test, feature = "m25-diagnostics"))]`; `M25DiagnosticBoss`
and `act3_victory_source` come from Task M2):

```rust
pub enum M25DiagnosticLoadout { CoverageA, CoverageB, CoverageC, Strong }
pub fn m25_supplied_traversal(boss: M25DiagnosticBoss) -> Result<Run, RunError>;
pub fn m25_supplied_act4(loadout: M25DiagnosticLoadout) -> Result<Run, RunError>;
pub enum M25PlayedFixture { FacingShieldSpear, BurnsShieldSpear, GenerationShieldSpear,
    PowersShieldSpear, BeatHeart, InvincibleHeart, StatusesHeart, TurnControlHeart }
pub fn m25_played_fixture(f: M25PlayedFixture) -> Result<Run, RunError>;
pub enum M25Diagnostic { DoorSource { boss: M25DiagnosticBoss, keys: [bool; 3] }, DoorFault,
    Act4Room { floor: u32 }, HeartVictoryPending, HeartProceedFault,
    ForeignInfluenceOffer, WishChoice, OmniscienceChoice }
pub fn m25_diagnostic(d: M25Diagnostic) -> Result<Run, RunError>;
```

- In `core/run/diagnostics/m23.rs`, add one `admission: WatcherAdmission` parameter to the
  helper that validates supplied resources (the point Task M0 recorded; in M24 version 2 it
  is reached through `boss_source(boss, scenario, act3, resources)`), replacing its
  `normalized()` call with `normalized_for(admission)`. Every M23 and M24 call site passes
  `WatcherAdmission::M21`. If merged M24 validates somewhere else, thread the parameter at
  that point instead; a change outside `m23.rs` follows the Task M0 amendment rule.
- `m25_supplied_traversal` calls M23's parameterized `boss_source` with
  `RoomScenario::M25WatcherHeart`, the boss's Act3 history, no keys, the strong loadout v1
  (design §12.4) and `WatcherAdmission::M25`.
- `m25_supplied_act4` calls `act3_victory_source(RoomScenario::M25WatcherHeart, boss, [true; 3],
  Some(resources), WatcherAdmission::M25)` with the loadout's design §12.5 resources and Act3
  source (A AwakenedOne, B TimeEater, C DonuAndDeca, Strong AwakenedOne).
- `m25_played_fixture` starts from `act3_victory_source` with all keys and the fixture's design
  §12.6 resources (72/72 HP, 99 gold, three empty potion slots), drives the real ending and
  door through the shared owners, then sets only the Act4 map position one edge before floor
  53 (Shield and Spear) or floor 54 (Heart, after a constructed recorded Shield and Spear
  Victory with its rewards closed).
- `M25Diagnostic::DoorSource` is `act3_victory_source` with canonical resources and the given
  keys; the other stages start from it.

- [ ] Write `m25_supplied_validation_uses_m25_unions`: every design §12.4–12.6 loadout passes
  under `WatcherAdmission::M25`; a card, relic or potion outside the M25 unions, an upgrade
  above 1, HP above max HP or max HP above 999 fails before any Run state is returned.
- [ ] Write `m25_m23_and_m24_diagnostics_unchanged_by_admission_threading`: every M23 and M24
  supplied and constructed start built after the change is byte-identical to the same start
  built before it (compare against the values the existing M23 and M24 diagnostics tests pin),
  and those tests still pass unchanged.
- [ ] Write `m25_supplied_and_constructed_starts_are_exact_without_actions`: each start has
  exactly the pinned HP, gold, deck IDs, order and upgrades, relics, potions, keys, act,
  floor and screen; zero accepted actions; the canonical test RNG; one `Move` where an edge
  is expected; and the next room is the intended encounter.
- [ ] Write `m25_constructed_stages_reach_actual_owners`: every `M25Diagnostic` stage that
  does not need card behaviour reaches its intended screen or retained fault through the real
  owner. The `ForeignInfluenceOffer`, `WishChoice` and `OmniscienceChoice` stages need HC1;
  they are written here and run green first on H2b (Task M5).
- [ ] Run `cargo test -p spire-core --lib --locked --offline m25_` (all Main tests not waiting
  on HC1 green) and the M23 and M24 diagnostics test filters (`m23_supplied_and_played`,
  `m23_constructed_stages` and the `m24_` diagnostics tests) to show they are unchanged.
- [ ] Commit; publish the snapshot as H2 with its manifest.

### Task C2: Direct and stance cards (Content)

**Files:** `core/combat/cards/watcher.rs` (or `cards/watcher/direct.rs`), shared damage, block,
draw, stance and random-target owners as needed; new `core/combat/tests/m25_watcher_direct.rs`
and its `mod` line.

**Interfaces:** Consumes HC0 and `m25_profile()` from H1. Covers BowlingBash, Consecrate,
Crescendo, EmptyBody, FlyingSleeves, Halt, Protect, Tranquility, EmptyMind, FearNoEvil,
Indignation, InnerPeace, WheelKick, Ragnarok, Scrawl and SpiritShield.

- [ ] Write `m25_direct_base_plus_values_and_flags`: a table over the 16 cards and both
  upgrade faces, entering a real setup as an actual Watcher under the M25 capability, with
  the design §5.1 literal costs, targets, Retain/Exhaust flags and damage, Block, draw and
  stance results. Include: BowlingBash against one, two and three living enemies hits 1, 2
  and 3 times; Halt in Wrath gains 3 + 9 (4 + 14) Block in one gain with Dexterity applied to
  each component as the source says; Indignation in Wrath applies 3 (5) Vulnerable to every
  enemy and otherwise only enters Wrath; InnerPeace in Calm draws 3 (4) and otherwise only
  enters Calm; FearNoEvil requests Calm only when the target's action-time intent is ATTACK,
  ATTACK_BUFF, ATTACK_DEBUFF or ATTACK_DEFEND, with its hit queued before that request;
  SpiritShield counts the other cards in hand at use; Scrawl draws until 10 cards and stops
  when both piles are empty.
- [ ] Write `m25_direct_existing_payloads_match_watcher_values`: Consecrate, EmptyBody and
  EmptyMind's existing generic handlers produce the design §5.1 values under the M25
  capability; a mismatch is fixed at the shared handler under rule 2.
- [ ] Write `m25_direct_ragnarok_random_targets`: 5 (6) hits of 5 (6), each target chosen at
  that hit from the living enemies through the shared random-target owner, with the
  remaining hits continuing when a target dies and stopping on Victory.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m25_direct` through RED, minimal
  change, GREEN, then the `m21_` and `m23_watcher` filters once.
- [ ] Commit on the Content branch.

### Task C3: State, history, selection and Lesson Learned (Content)

**Files:** play-history, legality, damage-result, pile, selection and autoplay owners in
`core/combat/`; the fatal callback owner; new `core/combat/tests/m25_watcher_state.rs` and
its `mod` line.

**Interfaces:** Covers CrushJoints, FollowUp, SashWhip, Sanctity, SignatureMove, Tantrum,
Wallop, Judgment, LessonLearned and Omniscience. Produces the shared "type of the previous
card played this combat" query used by these cards and by the native projection (Task N1),
and the Lesson Learned journal request consumed by Task M5.

- [ ] Write `m25_state_previous_card_type`: after an Attack, FollowUp gains 1 energy and
  SashWhip applies 1 (2) Weak; after a Skill, CrushJoints applies 1 (2) Vulnerable and
  Sanctity draws 2; after a Power or at the start of combat none of them triggers; the
  history is per combat and counts autoplayed and created plays.
- [ ] Write `m25_state_signature_wallop_tantrum`: SignatureMove is illegal while any other
  Attack is in hand, even an unplayable one, and legal when it is the only one; Wallop gains Block equal to the
  target's positive unblocked damage (zero against full Block, bound per F12); Tantrum hits 3
  (4) times, enters Wrath and returns the same instance to the draw pile rather than the
  discard pile.
- [ ] Write `m25_state_judgment_and_omniscience`: Judgment requests an Instant Kill of a
  monster at 30 (40) HP or less and does nothing above it (F9 binding for Invincible,
  half-dead actors and minions); Omniscience opens a
  selection over the draw pile, plays the chosen card twice without its energy cost, then
  exhausts it, with an X-cost choice using the remaining energy without spending it.
- [ ] Write `m25_state_lesson_learned_requests_permanent_upgrade`: a fatal Lesson Learned
  records exactly one permanent-upgrade journal request for an eligible permanent instance
  using the RNG bound at F8; a non-fatal hit records none; minion, half-dead Darkling and
  Awakened One phase 1 kills record none.
- [ ] Write `m25_state_zero_x_and_empty_pile_edges` (Review Focus 1) for Omniscience and
  Scrawl here; Task C4 adds Collect and Task C5 adds ConjureBlade to the same test.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m25_state` through RED, minimal
  change, GREEN, then the `m10_autoplay`, `m15_silent_history` and `m23_` filters once.
- [ ] Commit on the Content branch.

### Task C4: Powers and turn control (Content)

**Files:** power, hook, turn, engine, energy and creation owners in `core/combat/`; new
`core/combat/tests/m25_watcher_powers.rs` and its `mod` line.

**Interfaces:** Covers PressurePoints, TalkToTheHand, WreathOfFlame, WaveOfTheHand, Swivel,
SimmeringFury, Fasting, DevaForm, Collect, Blasphemy, Vault, Conclude and DeusExMachina.
Produces the shared Mark, Block Return, Vigor, block-gained callback, next-free-Attack,
delayed Wrath-and-draw, Energy Down, Deva Form, Collect, Blasphemer and extra-turn owners,
each as a named power or turn state readable by Task N1.

- [ ] Write `m25_powers_constructor_stack_and_callbacks`: for each power, the constructed
  amount, stacking, callback timing and removal in base and upgraded form. Include: Mark 8
  (11) applied then every marked enemy loses HP equal to its Mark (HP loss, not attack
  damage); Block Return 2 (3) installed after Talk to the Hand's own hit (that hit returns no
  Block), blocked by one enemy Artifact charge, and otherwise returning Block on later player
  attack hits on that enemy (F11 binding); Vigor 5 (8) added to NORMAL damage through the
  damage-give hook and removed by the next Attack's use callback; Wave of the Hand applying
  1 (2) Weak to all enemies on each positive Block gain, never on a zero gain, removed at the
  end of the round; the next Attack costing 0 once; next turn Wrath then draw 2 (3); Fasting
  giving 3 (4) Strength and Dexterity and one less energy each turn, with its Energy Down
  blocked by a player Artifact charge (AncientPotion) while the Strength and Dexterity still
  apply; Deva Form granting 1, 2, 3 energy on successive turns; Collect at X giving one
  upgraded Miracle at each of the next X (X+1) turn starts; Deus Ex Machina on actual draw
  adding 2 (3) Miracles and then exhausting itself, and not on creation, Scry or Meditate.
- [ ] Write `m25_powers_turn_control`: Conclude deals 12 (16) to all enemies and ends the
  turn once; Vault ends the turn, skips the enemy turn and end-of-round work and starts a new
  player turn with player end-of-turn and start-of-turn effects run once each (VAULT note);
  Blasphemy enters Divinity and its Blasphemer deals 99,999 HP loss at the next player turn
  start, survives stance changes and is not blocked by player Artifact (BLASPHEMY note).
- [ ] Write `m25_powers_stacked_turn_endings` (Review Focus 5) as described there.
- [ ] Extend `m25_state_zero_x_and_empty_pile_edges` with Collect at X = 0.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m25_powers` through RED, minimal
  change, GREEN, then the `m10_power_turns`, `m21_watcher_stances` and `m8_time_eater`
  filters once.
- [ ] Commit on the Content branch.

### Task C5: Generated children, Wish and the Foreign Influence closure (Content)

**Files:** `core/combat/generation.rs`, `core/combat/card_offers.rs`,
`core/combat/cards/ironclad.rs` (`has_strike_tag` correction), the card-state and
creation owners, `core/combat/cards/watcher.rs` (construction under the M25 capability, see
design §5.7); `core/combat/tests/m25_watcher_generation.rs`; new
`core/combat/tests/m25_watcher_foreign_influence.rs` and its `mod` line.

**Interfaces:** Covers Evaluate, Pray, Study, ReachHeaven, Alpha, ConjureBlade, Wish and
ForeignInfluence, the five new children and the three options. Produces
`GenerationPool::AnyColorAttack` (or the bound equivalent) with the 106-kind tiered support,
and extends Task C1's construction admission: under `m25_profile()`, a Foreign Influence
output gets its own colour's existing initialization. Off-colour cards in a permanent deck
still reject.

- [ ] Write `m25_generation_children_and_options`: Evaluate and Pray shuffle a base Insight
  into the draw pile; Study does so at each end of turn; ReachHeaven shuffles a base Through
  Violence; Alpha a base Beta and Beta a base Omega; ConjureBlade an Expunger with X (X+1)
  stored hits, created in the draw pile even at 0; the children have the design §5.2 values
  in base and upgraded form; Omega deals 50 (60) to every enemy at end of turn, unmodified by
  Strength, Vulnerable or stance; Wish opens a mandatory choice of its three options that
  never become physical cards, applying 3 (4) Strength, 6 (8) Plated Armor (with its
  end-of-turn hook admitted for the Watcher) or a 25 (30) gold gain through the existing
  combat gold path; Master Reality never upgrades an option.
- [ ] Write `m25_generation_all_watcher_cards_base_plus_census`: for every kind in
  `M25_WATCHER_CARDS` and `M25_WATCHER_GENERATED`, base and upgraded, construction under the
  M25 capability, public values, temporary upgrade and stat copy succeed, with exactly the
  design §5.1 and §5.2 costs; without the capability, the M21 construction law and its
  rejections are unchanged.
- [ ] Write `m25_generation_full_hand_overflow` (Review Focus 2) as described there.
- [ ] Write `m25_fi_support_and_proposal_law`: the support is exactly design §5.3 by colour
  and tier (43/48/15) and excludes the HEALING Attacks Feed, Reaper and LessonLearned through `is_healing`; three
  proposals each roll the tier first (0–54, 55–84, 85–99) and then a uniform kind, with a
  duplicate retrying the whole proposal; a scripted RNG sequence reproduces the conditional
  law; an incomplete tier rejects before any RNG or payment.
- [ ] Write `m25_fi_choice_and_delivery`: zero or one choice; the chosen kind becomes a fresh
  stat copy in hand; upgraded Foreign Influence sets its turn cost to 0; Master Reality
  upgrades it once in hand; Foreign Influence exhausts.
- [ ] Write `m25_fi_every_output_playable_by_watcher`: for each of the 82 non-Watcher kinds,
  base and upgraded, a Watcher with that card in hand plays it legally to completion through
  its existing owner with the correct finish; the table asserts each card's own literal
  primary value at base and upgraded, taken from its research row; it covers zero orb slots
  for Defect Attacks, Silent history Attacks and Searing Blow with exactly one Master
  Reality upgrade on the hand route. A blocked kind or a wrong value is a shared defect fixed
  under rule 2 for every character, never removed from the support.
- [ ] Write `m25_fi_perfected_strike_counts_the_source_strike_set` (known shared defect,
  design §5.3): Perfected Strike counts exactly the thirteen kinds for which Task C1's
  `is_strike` is true (Strike, StrikeGreen, StrikeBlue, StrikePurple, PerfectedStrike,
  PommelStrike, TwinStrike, WildStrike, SwiftStrike, SneakyStrike, ThunderStrike,
  MeteorStrike and WindmillStrike) in hand, draw and discard, and does not count
  Thunderclap. Watch it fail on the current hard-coded `has_strike_tag` in
  `core/combat/cards/ironclad.rs`, then make that function read `is_strike`, so only one
  STRIKE list remains. Add the same assertion for an Ironclad player so a Watcher-only fix
  would fail. Amend, with a disclosed diff, every accepted assertion that pinned the old set
  in M10 and in the training profile (`core/run/training.rs` `TRAINING_CARDS`, exposed by
  `crates/spire-env/src/combat_training.rs` and pinned by
  `crates/spire-core/tests/combat_training.rs`), and record both retained-outcome impact
  sweeps for Task M7's reconciliation. Task M0's list of existing STRIKE lists tells which
  tests to check.
- [ ] Extend `m25_state_zero_x_and_empty_pile_edges` with ConjureBlade at X = 0.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m25_generation` and
  `m25_fi` through RED, minimal change, GREEN, then the `m15_generation`, `m15_card_offers`,
  `m20_defect_generation`, `m10_generation` and `m21_watcher_creation` filters once.
- [ ] Commit on the Content branch.

### Task C6: Exclusive relics (Content) → HC1

**Files:** `core/items/relics.rs`; the post-shuffle hook in `core/combat/piles.rs` and
`core/combat/hooks.rs`; new `core/items/tests/m25_watcher.rs` and its `mod` line.

**Interfaces:** Consumes `m25_profile()` and the four `RelicId`s. Produces the Damaru,
Duality, CloakClasp and Melange callbacks and a post-shuffle hook other relics can reuse.
This task does not edit `core/items/mod.rs`. It implements the relics through the paths that
already reach relic logic from combat: `owns_relic` checks inside the combat owners (as
VioletLotus and GoldenEye do in `core/combat/cards/watcher.rs`) or new `Hook` variants
handled in `items/relics.rs::on_combat_hook`, which `items::relic_hook_effects` already calls.
If a facade change in `items/mod.rs` is unavoidable, it is a written request to Main.

- [ ] Write `m25_items_new_relics_through_real_combat`: Damaru gives 1 Mantra at each turn
  start (reaching Divinity at 10 through the shared owner); Duality gives 1 temporary
  Dexterity per Attack played, removed at end of turn; CloakClasp gives 1 Block per card in
  hand at end of turn in the order bound at F7; Melange Scries 3 whenever the draw pile is
  shuffled, with GoldenEye adding 2 and Weave and Nirvana reacting (F5 binding).
- [ ] Write `m25_items_accepted_relics_and_potions_preserved`: PureWater, HolyWater,
  VioletLotus, TeardropLocket and GoldenEye and the three Watcher potions behave as accepted
  under the M25 context.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m25_items` through RED, minimal
  change, GREEN, then `cargo test -p spire-core --lib --locked --offline m21_` and the item
  tests once.
- [ ] Run all Content filters (`m25_identity`, `m25_direct`, `m25_state`, `m25_powers`,
  `m25_generation`, `m25_fi`, `m25_items`) green; commit; deliver HC1 with the patch series,
  the test output and the list of first REDs and cited facts.

### Task N1: Native descriptor and encoding (Native, from H1 and HC0; card-behaviour tests on H2b)

**Files:** new `env/run_schema_m25.rs`; `env/run_observation.rs`; `env/lib.rs`; new
`crates/spire-env/tests/run_m25_schema.rs`.

**Interfaces:** `RunEncodingProfile::M25WatcherHeart`;
`m25_run_observation_spec(caps: &RunCapacities) -> Result<RunObservationSpec, RunSessionError>`;
`encode_run_m25(decision: &PublicDecision, map: Option<&PublicMap>, caps: &RunCapacities) -> Result<RunObservation, RunSessionError>`;
constants for the design §11 identities. Compose the M24 schema, the Act4 vocabulary the M20
schema takes from the M15 and M10 builders, the complete Watcher vocabulary and the
non-Watcher Foreign Influence outputs' existing card-value projections through their
builders; copy no table. Follow `env/run_schema_m20.rs`, which starts from the M15
descriptor's vocabularies and replaces or appends rows: here the card, relic and potion rows
come from the M25 content owners, and the power, intent and selection-operation rows keep
every row the 82 Foreign Influence outputs can create when a Watcher plays them (for
example Poison and Choke on enemies, Lock-On, Focus, next-turn energy and draw, and the
Headbutt and Dagger Throw selections), while class-only rows no M25 card can create stay
excluded as in M20.

- [ ] Write `m25_descriptor_vocabulary_and_old_profiles`: 171 card kinds (76 permanent, 8
  children, 82 Foreign Influence outputs, 5 statuses), 3 choice rows, 23 relics, 7 potions,
  42 encounters and 57 enemy identities; every Act4 intent and power is representable; every
  power and selection operation that a Watcher-played Foreign Influence output creates in
  Task C5's playability table is representable; the four new relics carry their M25 taxonomy
  rows after the unchanged M22 and M24 rows; a
  foreign ID fails closed; every older descriptor and fingerprint, including M24's, is
  unchanged.
- [ ] Write `m25_encoding_new_watcher_state`: Mark and Block Return per enemy; Vigor, Wave of
  the Hand, the next-free Attack, Simmering Fury, Study, Energy Down, Deva Form gain and
  growth, Collect's remaining turns, Omega, Blasphemer, Plated Armor and temporary Dexterity on
  the player; the previous card type; Expunger's stored hits and current card values;
  SignatureMove's legality in the action mask; candidate *i* at row *i*.
- [ ] Write `m25_encoding_offers_choices_and_selections`: Foreign Influence offer rows and Wish
  choice rows use the shared offer codec; an Omniscience selection over two draw piles with
  the same contents in different orders encodes identically.
- [ ] Write `m25_encoding_act4_and_terminal`: the Act4 map, Shield and Spear facing and
  Surrounded, Invincible remaining and cap, the `HeartVictory` view with one Proceed and the
  terminal `TrueVictory` row (`HeartVictory` = 4) encode; there is no cutoff row.
- [ ] Write `m25_numeric_privacy_and_capacity`: equal public state with different hidden draw
  order, AI, future offers or RNG encodes identically; each too-small capacity group returns
  its structured error without truncation.
- [ ] Run `cargo test -p spire-env --test run_m25_schema --locked --offline` through RED,
  minimal change, GREEN. On H1, HC0 and H2, run every test that needs only identities and
  Run state (the descriptor, Act4 views, privacy and capacity). The tests that need card
  behaviour (`m25_encoding_new_watcher_state` and `m25_encoding_offers_choices_and_selections`)
  are written against the HC0 interfaces and run to GREEN first on the H2b snapshot.

### Task N2: Native session, replay and batch (Native, from H2; card-behaviour tests on H2b)

**Files:** `env/run_session.rs`, `env/run_replay.rs`, `env/run_batch.rs`, `env/lib.rs`; new
`crates/spire-env/tests/run_m25_{session,replay,batch}.rs`; mechanical arms in
`crates/spire-env/tests/{run_m10_session,run_m10_batch}.rs`; `crates/spire-env/Cargo.toml`
adds `m25-diagnostics` to the existing dev-dependency feature list.

**Interfaces:** `RunProfile::M25WatcherHeart`;
`RunBossHistory::M25WatcherHeart { act1, act2, act3 }`; `RunSession::new_m25`;
`RunBatch::new_m25`; M25 in the three `create_run` complement matches; no `check_boundary`
cutoff arm and no `at_m14_cutoff` membership.

- [ ] Write `m25_config_history_validation_precedes_initialization`: all 27 triples accept
  Watcher A0 only; M22, M23 and M24 histories, other profiles' triples, and partial,
  duplicate-key, extra-key, wrong-type and unknown values reject before Run initialization.
- [ ] Write `m25_raw_json_strict`: duplicate and unknown keys at every new nesting level
  reject before `serde_json::Value` conversion; the exact history object round-trips.
- [ ] Write `m25_lifecycle_no_cutoff_and_terminals`: a constructed stage-4 door decision
  publishes healthily and stepping it reaches the Act4 `Map`; a pending `HeartVictory` is
  nonterminal; the final Proceed is terminal with reward 110 counted once; `Act3NoKeys` is
  terminal; reset with an explicit seed and an implicit ordinal, record, reproduction and
  close stay valid; stale tokens and invalid indices change nothing.
- [ ] Write `m25_semantic_replay` and `m25_replay_preflight`: recorded Foreign Influence,
  Wish and Omniscience choices, the door, Act4 rooms and the Heart Proceed replay through
  strict JSON at the recorded ordinal; identity, fingerprint and history mismatches reject
  before construction.
- [ ] Write `m25_delivery_fault_record`: a door or Heart Proceed publication fault keeps the
  owner, serial and first structured error; the record stays readable.
- [ ] Write `m25_batch_order_matches_singles`, `m25_batch_preflight_full_reset`,
  `m25_batch_postdispatch_faults`, `m25_batch_rejects_step_with_terminal_slot` (a slot at the
  `HeartVictory` or `Act3NoKeys` terminal rejects the whole step before any peer changes) and
  `m25_coexistence` (M25, M24, one accepted Silent and one accepted Defect profile).
- [ ] Run `cargo test -p spire-env --test run_m25_session --locked --offline` and the
  `run_m25_replay` and `run_m25_batch` targets through RED, minimal change, GREEN, then the
  existing M24, M20 and M10 env tests once. Everything except `m25_semantic_replay` runs on
  H2; `m25_semantic_replay` (recorded Foreign Influence, Wish and Omniscience choices) runs
  to GREEN on H2b.
- [ ] Rerun all four `run_m25_*` targets on the H2b snapshot and deliver H3 with that output
  and the snapshot manifest. If Main later reports an env failure caused by HC2's fixes,
  reproduce it on the integrated tree Main names and re-deliver as H3b.

## Sub-phase 3: Compose and integrate

### Task C7: Watcher × Act4 and new-card × Act3 compositions (Content) → HC2

**Files:** new `core/combat/tests/m25_watcher_interactions.rs` and its `mod` line; reproduced
defects only in the owning shared combat or item file.

Follow `core/combat/tests/m20_defect_interactions.rs`: real setups from `enemies::setup`,
the player set to Watcher under the M25 capability, legal actions only, and intents set
only through the existing refreshing helper.

- [ ] Write one test per design §10 row: `m25_t01_beat_of_death_counts_watcher_plays`,
  `m25_t02_invincible_allowance`, `m25_t03_debilitate_statuses_draw_and_scry`,
  `m25_t04_painful_stabs_and_block_return`, `m25_t05_heart_artifact_and_watcher_debuffs`,
  `m25_t06_shield_spear_facing`, `m25_t07_shield_bash_and_watcher_strength`,
  `m25_t08_spear_burns_retain_and_wrath`, `m25_t09_vault_against_act4`,
  `m25_t10_blasphemy_at_act4`, `m25_t11_lesson_learned_and_wish_settlement` (combat journal
  entries only; Run persistence is Task M5), `m25_t12_generation_and_master_reality_at_act4`,
  `m25_w15_time_warp_turn_control`, `m25_w16_curiosity_new_powers`,
  `m25_w17_mark_intangible_and_half_dead`, `m25_w18_writhing_new_multi_hits`,
  `m25_w19_slow_new_multi_hits` and `m25_w20_awakened_rebirth_mark_and_omega`. Extend, do not
  duplicate, the M24 W-row file where a case already exists.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m25_t` and `m25_w`. A test that
  passes first time is a regression; record it as such. For each RED, record the output and
  the cited fact, fix the shared law under rule 2 with a disclosed amendment and a regression
  that fails under a narrower fix, and rerun the affected M10, M15, M20 and M24 filters.
- [ ] Commit on the Content branch; deliver HC2 with the RED list, fixes and disclosures.

### Task M5: Integration and Run-level content (Main) → H2b

**Files:** apply HC1, HC2 and H3; `core/run/combat.rs` (settlement of the Lesson Learned
journal request and the Wish gold sync); `core/run/mutations.rs`; new
`core/run/tests/m25_watcher_content.rs` and its `mod` line.

- [ ] Integrate HC1 first, unchanged except for agreed fixes, recording pre- and
  post-integration manifests and rerunning every Content filter (`m25_identity`,
  `m25_direct`, `m25_state`, `m25_powers`, `m25_generation`, `m25_fi`, `m25_items`) and every
  Main `m25_` filter on the integrated tree. Run the HC1-dependent `m25-diagnostics` stages
  (`ForeignInfluenceOffer`, `WishChoice`, `OmniscienceChoice`) to GREEN. Publish this
  snapshot and its manifest as H2b.
- [ ] Integrate HC2 and then H3 the same way. After HC2, rerun all four `run_m25_*` env
  targets on the integrated tree; any failure goes back to the Native lane with that tree's
  manifest for H3b, or to the Content lane if the cause is a combat owner.
- [ ] Write `m25_content_every_card_obtain_upgrade_remove_next_combat`: for each of the 75
  kinds under M25, obtain it through the shared owner, upgrade it at Smith and at
  UpgradeShrine, enter the next combat with it in the deck, and remove it at Purifier; Parasite
  is obtained only through the journal, never upgraded, and its removal costs max HP once.
- [ ] Write `m25_content_lesson_learned_and_wish_persist_once`: a fatal Lesson Learned's
  upgrade, a FameAndFortune gain, and the gold from a fatal Hand of Greed created by Foreign
  Influence (through the existing gold sync in `core/run/combat.rs`) each appear in the Run
  exactly once after Victory and after Death settlement, and never twice after a later
  fault.
- [ ] Write `m25_content_lesson_learned_no_eligible_card` (Review Focus 3) as described there.
- [ ] Write `m25_content_relic_hooks_across_combats`: Damaru, Duality, CloakClasp and Melange
  fire in each of two consecutive combats and leave no combat state in the Run.
- [ ] Write `m25_events_have_watcher_semantics`: the seven events with the design §9
  semantics, including the `[Parasite, X]` and `[X]` Falling cases and Tomb at gold 0, 99 and
  with a mask.
- [ ] Write `m25_content_old_profiles_reject_new_content`: M22, M23 and M24 reject every new
  card and relic at admission and at combat entry, and the M21 validator rejects them.
- [ ] Run the filters `m25_content` and `m25_events` through RED, minimal change, GREEN, then
  every `m25_` core filter and the M21–M24 and M20 core filters.
- [ ] Commit.

## Sub-phase 4: Evidence and close

### Task M6: Caller, freeze gate, freeze and once-only outcomes (Main; Native consumes H4)

**Files:** `crates/spire-core/examples/m13_support/mod.rs`; new
`crates/spire-core/examples/m25_support/{mod,policy,inputs,fixtures}.rs`; new
`crates/spire-core/examples/m25_watcher_heart.rs`; new
`crates/spire-core/examples/m25_supplied_diagnostics.rs`; new
`crates/spire-core/tests/m25_watcher_heart.rs`.

**Interfaces:**
- `CallerProfile::M25`; `M25CallerOutcome { HeartVictory, Act3NoKeys, Death, EmptyDeck,
  BudgetExhausted }`; `rollout_m25(run, caller, policy)` in the existing `rollout_shared`
  loop.
- `m25_support/policy.rs`: `pub fn choose(d: &PublicDecision, map: Option<&PublicMap>,
  progress: &PublicCallerProgress) -> Result<GameAction, String>`. In act 3 or later it
  first returns `public_keys::key_action(d, map, RoomScenario::M25WatcherHeart)` when that
  gives an action, then delegates to the M24 `choose` unchanged. If Task M0 found a new
  selection action the M22 rule does not recognise, it takes the first offered candidate of
  that selection. The M22, M23 and M24 policy files are not edited.
- The example takes `--seed 7|11`,
  `--scenario awakened-one|time-eater|donu-and-deca|death|all` and `--budget 0..=32768`,
  default `7`, `all`, `32768`, rejecting anything else before building a Run, with the M24
  triple mapping. The supplied example runs the traversal, Act4 and fixture series in that
  order, applying the design §12.5 strong-case rule. The chooser never sees the seed or case.

- [ ] Write `m25_caller_keeps_m24_choices_in_acts_1_and_2`: on supplied public decisions in
  Acts 1 and 2 the M25 choice equals the M24 choice, including a campfire with Ruby missing
  and a chest with Sapphire missing.
- [ ] Write `m25_caller_key_objective_from_act3`: in Act 3, a campfire without Ruby returns
  Recall, a chest without Sapphire returns Sapphire, a map without Emerald returns the move
  toward the reachable burning elite, stage 4 returns `Act3Continue{stage:4}` and a
  `HeartVictory` screen returns Proceed.
- [ ] Write `m25_caller_handles_every_act4_and_new_selection`: each Act4 map decision (one
  `Move` per floor) returns that `Move` through the shared route helper; the Rest 51 campfire
  at full HP with nothing upgradable returns Rest; the merchant returns the first affordable
  card, then Leave; the elite rewards, card child, Foreign Influence offer, Wish choice and
  Omniscience selection each return a legal action.
- [ ] Write `m25_caller_new_selections_close_and_return_to_play`: starting from each of the
  constructed `ForeignInfluenceOffer`, `WishChoice` and `OmniscienceChoice` stages, applying
  the caller's chosen action repeatedly closes the selection or offer within at most four
  caller steps and returns to a combat decision whose candidates include a card play or
  `EndTurn`; a caller that cycles (for example by toggling a selection) fails the test. This
  test runs on H2b, because the stages need HC1.
- [ ] Write `m25_caller_input_contract_rejects_invalid_values_without_creation`: an unknown
  scenario, seed 13, budget 32769 and a malformed budget each return an error before any Run
  is built; the six valid seed and scenario pairs build the design §12.3 triples with the
  canonical resources and zero accepted actions.
- [ ] Run `cargo test -p spire-core --test m25_watcher_heart --locked --offline m25_caller`
  through RED, minimal change, GREEN.
- [ ] **Freeze gate** (design §12.8). Confirm and record on the exact tree to be frozen: HC1,
  HC2 and H3 (and any H3b) integrated; every RED resolved with its fix and disclosure; and
  these pass:
  `cargo test -p spire-core --lib --locked --offline m25_`,
  `cargo test -p spire-core --features m25-diagnostics --lib --locked --offline m25_`,
  `cargo test -p spire-core --test m25_watcher_heart --locked --offline`,
  `cargo test -p spire-env --test run_m25_schema --locked --offline` (and the session,
  replay and batch targets), the `m24_`, `m23_`, `m22_`, `m21_` and `m20_` core filters, the
  full workspace suite `cargo test --workspace --locked --offline --no-fail-fast`, and the
  feature-on core suite `cargo test -p spire-core --features m25-diagnostics --locked
  --offline`. The two workspace runs may show only the inherited, documented failures and
  holds; anything else blocks the freeze. Record the exact counts.
- [ ] Freeze, in one record retained outside the session: the full source manifest, the
  built example binaries and their hashes, the caller and policy hashes, the six canonical
  cases and controls, the strong loadout v1 and coverage loadouts A, B and C, the three
  traversal cases, the three Act4 cases and the strong-case rule, the eight fixtures and
  their budgets, the M24 comparison control, and the zero-action initial snapshot of every
  case.
- [ ] Run once each:
  `cargo run -p spire-core --example m25_watcher_heart --locked --offline -- --seed 7 --scenario all --budget 32768`,
  the same with `--seed 11`, the controls (`--scenario death`, budgets 0 and 1, the
  input-error controls), and
  `cargo run -p spire-core --example m25_supplied_diagnostics --features m25-diagnostics --locked --offline`.
  Record every endpoint, action count, combat entry and matching record, key state, ending
  and observed checkpoint.
- [ ] Run the M24 comparison control once on the same frozen tree:
  `cargo run -p spire-core --example m24_watcher_act3 --locked --offline -- --seed 7 --scenario all --budget 32768`
  and the same with `--seed 11`, labelled as an M25 comparison control that does not replace
  M24's retained evidence.
- [ ] Compare each canonical M25 case step by step with the control (design §4.2 check 4);
  any difference before Act 3 is a defect to investigate. Then compare with M24's retained
  receipts and attribute every difference to a disclosed global fix, naming its first
  differing step; an unexplained difference is a defect.
- [ ] Deliver H4.

### Task N3: Recorded native agreement (Native, from H4)

**Files:** an explicitly ignored, default-off consumer in the env test targets.

- [ ] Write `m25_recorded_native_agreement`: for every frozen transcript, replay the recorded
  semantic actions through native single sessions, strict JSON replay for normal starts or
  internal same-initializer replay for supplied starts, and paired batches with workers 1
  and 2; compare every boundary, reward, record and terminal; invoke no policy.
- [ ] Run it once on the frozen tree:
  `cargo test -p spire-env --test run_m25_batch --locked --offline m25_recorded_native_agreement -- --ignored --nocapture`.
  Report agreement per case.

### Task M7: Stable matrix, documents and review packet (Main)

**Files:** new `docs/milestones/m25-watcher-heart.md`,
`docs/milestones/m25-watcher-heart-coverage-ledger.md` and
`docs/milestones/m25-watcher-heart-reconciliation.md`; a dated entry at the top of
`docs/work-status.md`.

- [ ] Run once on the frozen tree, as the stable matrix that repeats the freeze gate's
  workspace runs: `cargo test --workspace --locked --offline --no-fail-fast`;
  `cargo test -p spire-core --features m25-diagnostics --locked --offline`;
  `cargo check -p spire-core --lib --locked --offline` and the same with
  `--features m25-diagnostics`. Record source manifest, toolchain, features, commands and
  exact counts. The two scope-ignored strategy gates and inherited holds stay as they are.
- [ ] Write the profile (unions, laws, identities, every outcome at its grade, design §13
  limits); the coverage ledger, with one row per card, child, option, Foreign Influence
  output, relic, potion and event naming its test key (design §12.2) and evidence; and the
  reconciliation (proposed-to-actual owner map, each frontier's binding or open part, every
  global correction and amended assertion, lane provenance with integration hashes).
- [ ] Check every changed link, path and command; run
  `git diff --no-index --check /dev/null <file>` on each new file and `git diff --check` on
  edited files.
- [ ] Self-check design §14 against the delivered tests. Send the PM one immutable candidate
  (source, documents, evidence manifest) for the one fresh independent final review, and
  handle that reviewer's bounded findings with the same reviewer.

## Planning delivery check

This planning task adds only this plan and the paired design. It runs no build, test or
episode, and stages nothing. Links, paths and commands were checked, and whitespace was
checked with `git diff --no-index --check /dev/null <file>`. Version 1 and its hashes are
retained at
`/home/klottick/.local/share/spire-gym/evidence/m25-execution-20261030/planning-v1/`;
version 2, the plan review v1 disposition table and `SHA256SUMS` are at
`/home/klottick/.local/share/spire-gym/evidence/m25-execution-20261030/planning-v2/`.
