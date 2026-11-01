# M26 shared and off-colour card coverage Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development or
> superpowers:executing-plans to carry out this plan task by task, after the one
> independent plan review returns READY, M25 is merged and the PM releases execution. Steps use
> checkbox (`- [ ]`) syntax. Use meaningful TDD and one fresh independent final milestone
> review; do not add per-task review swarms.

**Goal:** Make every standard card exist, play for every character through its own
implementation, come from complete producer pools, and change the permanent deck through
complete shared obtain, upgrade, remove, transform, duplicate and Note-retrieval owners.

**Architecture:** One complete card capability (`content::complete`), one pool owner
(`content::pools`), a combat-context overlay (`with_complete_cards()`) and a Run admission
selector (`CardAdmission::Complete`) added beside the frozen finite profiles; every new card
and mutation lives in the existing shared owners. Evidence is constructed and supplied core
tests; no public profile changes.

**Tech Stack:** The existing locked Rust workspace (`spire-core`, `spire-env`) and its checked
Run RNG. No new dependency, no Python rebuild, no original-game execution.

**Spec:** [M26 design](../specs/2026-10-31-m26-shared-cards-design.md). "Design §N" refers to
it. The design pins every roster, value, pool, law and limit; this plan repeats them only
where a step needs them inline.

## Global constraints

- Work only in the checkout and branch the PM assigns (expected base: `main` at the merged
  M25 revision plus these two documents). Helper lanes work in their own worktrees from the
  snapshot named in their handoff. Preserve unrelated work.
- Execution cannot start until M25 is merged. Task M0 applies design §16's re-review
  trigger.
- One authoritative Rust engine. No copied card lists, pools, samplers, initializers, journals,
  censuses or validators. No public supplied-run or import API. `m26-diagnostics` is default-off.
- No installs, new dependencies, Python rebuild or original-game execution.
- Scope is playing the game, not winning it. No test requires a normal-start route or win.
- A shared-owner defect found in any lane is fixed at the shared law for all characters. An
  accepted assertion that pinned wrong behaviour is amended with a disclosed diff, its
  original bytes retained, and a regression test that fails under any narrower fix. Helper
  lanes report defects outside their files to Main.
- Keep combat-v1, the default and Gym66 profiles, policy-v3 and every accepted M9–M25
  profile, finite list, descriptor, fingerprint, wire value and RNG key unchanged, except
  mechanical exhaustive-match arms and the disclosed corrections of design §11.
- Every expected value is a literal from design §4 or §7, this plan's tables, or a cited
  fact; never computed by the helper under test. A frontier left unsettled after Task M0 is
  implemented as the behaviour the cited sources most directly support, pinned in a test
  labelled as a disclosed project decision, recorded in the reconciliation and reported to
  the PM before the freeze; it never becomes an `Unsupported` rejection of a legal play.
- Roles (PM runbook §5): Main is a Claude Opus 5.5 senior ("Senior Software Engineer / Main
  Runner" profile) who owns every shared seam, bug ruling and review; Lanes A and B are Codex
  assistants (gpt-6-luna) doing pattern-following work under Main's exact contracts and
  review. Seniors rotate at phase boundaries through a short handoff file, and none runs
  past about 60% of context. Long builds and full suites go to an assistant, which writes
  results into the evidence directory; Main reads the result and does not watch the run.
- Cargo commands use one task-owned `CARGO_TARGET_DIR` outside the repository and
  `--locked --offline`. A compile error from a missing symbol is not a RED: watch each new
  test fail for a behavioural reason after minimal wiring.
- Commit at the end of each task, staging only that task's explicit paths (never a blanket
  `git add`), after `git diff --cached --check` passes. Helpers commit on their own branches
  and deliver patch series; Main integrates them with pre- and post-integration manifests.
  Pushing and PRs belong to the PM.

## Review Focus

Five conditions the design implies but no section states as a test. Each has a test in its
owning task.

1. **Rejected mutations leave the Run untouched.** Removing or transforming CurseOfTheBell,
   Necronomicurse or Ascender's Bane, obtaining an upgraded curse, a second upgrade of a
   non-Searing card, and obtaining at the 999-card cap each return an error with deck, HP,
   max HP, RNG counters and hooks unchanged. Test `m26_mutation_rejections_are_atomic`
   (Task M3).
2. **Normality under autoplay.** With Normality in hand, a Double Tap'd Attack, a Burst'd
   Skill, a Havoc play and a Mayhem play each count toward the 3, and a fourth manual play is
   absent from the action mask. Drawing Normality after 3 plays blocks the rest of the turn.
   These are tested in `m26_normality_counts_autoplay_and_late_draw` (Task A1). An autoplay
   that would be the fourth is handled as F9 binds (where the blocked card goes), never as a
   fault; the autoplay owner is Main's, so that half is tested in
   `m26_normality_blocks_autoplay` (Task M3, on H2′, which contains Normality).
3. **Pain killing the player.** Two Pains in hand at 2 HP: playing a card loses 1 HP per
   Pain (2 in total), the player dies once, and the Death is a normal combat terminal, not a fault.
   Test `m26_pain_stacks_and_kills_once` (Task A1).
4. **Necronomicurse with a full hand.** Exhausting Necronomicurse with 10 cards in hand
   puts the copy in the discard pile; exhausting it twice (Fiend Fire, then True Grit)
   yields two combat-local copies and the permanent instance stays in the exhaust pile for
   the census. Test `m26_necronomicurse_full_hand_and_repeat` (Task A1).
5. **Ritual Dagger exclusions and persistence.** A Fatal on a minion, on Awakened One's
   first phase and on a non-final Darkling does not grow it; a real Fatal grows the played
   card and its same-instance family but not a Dual Wield copy; after Victory the Run deck
   card has the grown `misc` exactly once, and after a Death in the same combat nothing
   faults. Test `m26_ritual_dagger_growth_exclusions_and_persistence` (Task M3).

## Lanes and file ownership

Main is the senior; Lanes A and B are Codex assistants working from exact contracts under
Main's review. Paths start at the repository root; `core/` is `crates/spire-core/src/`.

| Lane | Owns | Must not edit |
| --- | --- | --- |
| **Main** (senior) | Everything not listed for A or B, including all of `core/content/**`, `core/run/**`, `core/items/**`, `crates/spire-env/**` (mechanical exhaustive-match arms only), `crates/spire-core/Cargo.toml` (feature only), every `core/combat/**` file outside Lane A's list (among them `cards/mod.rs`, `card_state.rs`, `engine.rs`, `view.rs`, `effects.rs`, `powers.rs`, `piles.rs`, `turns.rs`, `exhaust.rs`, `generation.rs`, `characters.rs`, `cards/watcher.rs`, `cards/defect.rs`, `cards/ironclad/{copies,reactive}.rs`), the curse hook call sites and the Ritual Dagger journal seam, new `crates/spire-core/tests/m26_catalog.rs`, `core/combat/tests/m26_offcolour_laws.rs`, `core/run/tests/m26_card_mutations.rs`, amended accepted tests (design §11), all docs | During sub-phase 2, Lane A's files except by written request to Lane A; fixes there land in M4 |
| **A** (Codex) | The bodies in `core/combat/cards/curses.rs` (Main creates the file with stub bodies in M1); the new colourless handlers in `core/combat/cards/colorless.rs` and `core/combat/cards/colorless/**`; a Ritual Dagger arm in `core/combat/cards/ironclad/fatal.rs` that calls Main's journal seam; new `core/combat/tests/m26_cards.rs` and its `mod` line | Every other file |
| **B** (Codex) | New tests only: `core/combat/tests/m26_{cross_character,generation,copies,support}.rs`, `core/run/tests/m26_supplied.rs`, their `mod` lines | Every non-test file |

A lane needing a change in a file it does not own sends the owner a written request with
the reason and a proposed diff.

| Sub-phase | Main | A | B |
| --- | --- | --- | --- |
| 1. Seams | M0 → H0; M1 → H1 | — | — |
| 2. Build | M1b → H1b; M2 → H2; integrate HA, H1b and H2 → H2′; M3 on H2′ → H3 | A1 on H1 → HA | B1 on H2′; B2 on H3 → HB |
| 3. Integrate and close | M4, M5 | fixes only | fixes only |

| Handoff | From → to | Contents | Gate |
| --- | --- | --- | --- |
| H0 | Main → A, B | Seam map: §16 check results; the completed design §6.3 site table with classes; card-list sweep; frontier bindings; orb-card list; RunState text exposure; Searing Blow bound; the stripped matrix context, encounter and own-colour scripts; B2's encounter; captured fixed-seed rows for the sampler and M10 Infernal Blade checks | A and B read it |
| H1 | Main → A | Snapshot with the 15 identities, catalog rows, `card_state.rs` keyword and `misc` rows for the new cards, the unplayable and `base_cost` lists in `cards/mod.rs`, engine validator acceptance for curse-sourced damage, Weak, Frail, HP loss and Necronomicurse's card creation, the curse hook call sites with stub bodies, the Ritual Dagger journal seam, `content::complete`, `with_complete_cards()`, the colour dispatcher, `CardAdmission` and `CompleteCombatStart`; `m26_identity` and `m26_capability` green | A starts A1 |
| H1b | Main (internal) | Design §6.3 gates and Alchemize, with `m26_offcolour` green | Integrated into H2′ |
| HA | A → Main | The 17 card behaviours with `m26_cards_` green and each first RED with its cited fact | Main integrates into H2′ |
| H2 | Main (internal) | `content::pools`, generator arms, Infernal Blade, `selectable_pool`, scopes, G1–G4, with `m26_pools` green | Integrated into H2′ |
| H2′ | Main → B | H2 + H1b + HA integrated, with manifest; all `m26_` core tests so far green | B starts B1; Main starts M3 |
| H3 | Main → B | H2′ plus every Run mutation owner, the Ritual Dagger journal and `m26-diagnostics`, with `m26_mutation` green | B starts B2 |
| HB | B → Main | Tests X, G, Y, S with every RED, its suspected owner and evidence | Main fixes and integrates in M4 |

---

## Sub-phase 1: Seams

### Task M0: Bind source and map seams (Main)

**Files:** none in the repository. Write the seam map and source receipt to
`/home/klottick/.local/share/spire-gym/evidence/m26-execution-20261031/main-v1/task0/`. An
assistant may run the sweeps and write raw lists there; Main reviews and decides.

- [ ] Run `pwd`, `git status --short --branch`, `git branch --show-current`,
  `git rev-parse HEAD`. Confirm HEAD is the merged M25 revision the PM names plus only the
  two planning documents. Claim the execution Bead with your own actor.
- [ ] Record a SHA-256 manifest of all tracked files, retained outside the session.
- [ ] Apply design §16: check every row against merged M25 and record owner, signature and
  result. On a difference not in "Accommodated differences", write the amendment, send it
  through the PM to the plan reviewer, and stop until READY.
- [ ] Sweep every function on the `TurnStartedPreDraw`, `TurnStarted` and
  `PlayerPreEndTurnCards` paths (`core/combat/hooks.rs` ~170–185: `defect::turn_started`,
  `generation::start_turn`, `blue_start_turn`, `defect::power_hook`, `watcher::hook`,
  `silent_power_hook`, `colorless_power_hook` and the `m15_player` block) and split each
  mixed owner by the design §6.3 split rule. Record on merged M25 whether and how the routed
  next-turn energy fix landed (design §16), and so what G7 must still do.
- [ ] Complete the design §6.3 site table on the merged tree: every `Character::`,
  `watcher::active`, `source_character` and profile-flag (`m15_profile`, `m20_profile`,
  `m21_profile`, `m25_profile`, `shared_generated_profile`) comparison in `core/combat/**` and
  `core/items/**` that can change card behaviour, each with path:line, class L, S or C by
  the design §6.3 rule, and its new gate. Check whether the Watcher discard law reproduces the
  ordinary law on hands without a self-Retain card (design §6.3 `piles.rs` row).
- [ ] Card-list sweep: list every literal `&[CardId]` list and every card filter in
  `crates/spire-core` and `crates/spire-env` (including `m10_content::infernal_candidates`,
  `m25_content::purple_candidates`,
  the M10 and M15 rarity arrays, `TRAINING_CARDS`, the L34 token list, the `eligible_for`
  fence). For each, record whether it is a frozen profile law (kept), a pool M26 replaces
  behind `content::pools`, or a defect.
- [ ] Bind frontiers F1–F14 (design §12): the cited fact, or "open" with the behaviour the
  sources most directly support. List the orb cards for F12 from the Defect metadata.
- [ ] Record whether `RunState` or `CombatItemContext` text reaches any checkpoint,
  fingerprint or receipt, and the largest Searing Blow upgrade count the upgrade rule
  represents.
- [ ] Record the exact constructors of the four complete Heart contexts (M10, M15, M20, M25)
  and of their Run scenarios, every combat-wide tracker each context gates (for the
  overlay, design §5.2), and the own-colour script for `m26_capability_overlay_inert`.
- [ ] Capture fixed-seed M15, M20 and M25 reward and merchant rows from the unchanged
  sampler for `m26_pools_sampler_generalization_is_rng_identical`, and fixed-seed M10
  Infernal Blade draws for `m26_pools_infernal_blade_player_pool`.
- [ ] Name the stripped matrix context, encounter and draw pile (design §14.3), one
  unstripped own-colour script per character for the inertness test, and B2's encounter.
- [ ] Publish H0.

### Task M1: Identities, unions and the complete capability (Main) → H1

**Files:** `core/content/ids/cards.rs`; `core/content/cards.rs`; new `core/content/complete.rs`
and its `mod`/`pub use` in `core/content/mod.rs`; `core/combat/cards/mod.rs` (colour
dispatcher, unplayable and `base_cost` lists, curse hook call sites); `core/combat/card_state.rs`
(keyword and initial `misc` rows); `core/combat/engine.rs` (combat-entry admission, overlay
trackers, validator acceptance for curse-sourced effects); `core/combat/exhaust.rs` and the
end-of-turn owner beside Burn (hook call sites); new `core/combat/cards/curses.rs` (stubs);
`core/combat/cards/defect.rs` (journal seam); `core/items/mod.rs`; `core/run/model.rs`;
`core/run/combat.rs`; `core/run/training.rs`; new `crates/spire-core/tests/m26_catalog.rs`.

**Interfaces:**
- Produces appended `CardId::{BandageUp, Apparition, Bite, Jax, RitualDagger, Clumsy,
  CurseOfTheBell, Decay, Doubt, Injury, Necronomicurse, Normality, Pain, Shame, Writhe}`
  with `as_str` spellings `bandage_up`, `apparition`, `bite`, `jax`, `ritual_dagger`,
  `clumsy`, `curse_of_the_bell`, `decay`, `doubt`, `injury`, `necronomicurse`, `normality`,
  `pain`, `shame`, `writhe` (Task M0 confirms no collision). Catalog rows per design §4.2–4.3;
  Regret and AscendersBane no longer `definition_only`. Note storage:
  `NonremovableCurse` for CurseOfTheBell, Necronomicurse, AscendersBane; `RemovablePermanent`
  for every other new card.
- Produces in `core/content/complete.rs`: `pub fn supports_permanent(id: CardId) -> bool`,
  `pub fn supports_generated(id: CardId) -> bool`, `pub fn is_choice_only(id: CardId) -> bool`,
  `pub fn is_removable(id: CardId) -> bool`.
- Produces `pub(crate) fn with_complete_cards(self) -> Self` and
  `pub(crate) fn complete_cards(&self) -> bool` on `CombatItemContext` (private field
  `complete_cards: bool`; `Debug` prints it only when true).
- Produces the colour dispatcher, proposed
  `pub(crate) fn initialize_complete(card: &mut LiveCard) -> Result<(), InputError>` in
  `core/combat/cards/mod.rs`, calling the existing per-colour initializer by the card's
  catalog colour.
- Produces the curse seams Lane A fills, as functions in new `core/combat/cards/curses.rs`
  called from Main's sites, with bodies that return "no effect" until A1:
  `pub(crate) fn play_blocked(state: &State) -> bool` (read by `validate_card_use` and the
  action mask in `engine.rs`/`view.rs`; Normality), `pub(crate) fn end_of_turn_in_hand(state:
  &State, card: &LiveCard, work: &mut Vec<Effect>)` (Decay, Doubt, Shame, Regret, beside
  Burn's call), `pub(crate) fn on_card_played(state: &State, played: &LiveCard, work: &mut
  Vec<Effect>)` (Pain) and `pub(crate) fn on_exhaust(state: &State, card: &LiveCard, work:
  &mut Vec<Effect>)` (Necronomicurse). The new curses are unplayable in `validate_card_use`.
- Produces the overlay's tracker list (design §5.2) switched on under `complete_cards()`.
- Produces `card_state.rs` rows for the new cards: Ethereal for Clumsy, AscendersBane and
  base Apparition; Innate for Writhe; Exhaust for BandageUp, Apparition and RitualDagger;
  initial `misc` 15 for RitualDagger. Produces engine validator acceptance for curse-sourced
  THORNS damage (Decay), Weak (Doubt), Frail (Shame), HP loss (Regret, Pain) and
  Necronomicurse's hand creation.
- Produces the Ritual Dagger journal seam in the combat journal producer
  (`core/combat/cards/defect.rs`, whose `PermanentCardMutation` is hard-coded to Genetic
  Algorithm today): `pub(crate) fn record_misc_growth(state: &mut State, card: CardInstanceId,
  amount: i32) -> Result<(), EffectError>`, accepting GeneticAlgorithm and RitualDagger. The
  Run side (`core/run/combat.rs`) accepts it in Task M3.
- Produces `pub enum CardAdmission { Profile, Complete }` with `RunState.card_admission`
  defaulting to `Profile`, and a new validator `CompleteCombatStart::Supplied(AuthoredRunStart)`
  with `normalized(character)` in `core/run/training.rs` (not a rename of `WatcherAdmission`;
  validates 1–128 cards in `supports_permanent`, upgrades per design §8.1, unique nonzero
  identities, `misc` 0 except Genetic Algorithm and Ritual Dagger at their obtain values).

- [ ] Write `m26_identity_catalog_rows`: the 15 identities exist with design §4.2–4.3 colour,
  rarity, type and storage; Regret and AscendersBane are not `definition_only`; Pride has no
  identity; `is_healing(BandageUp)` is true.
- [ ] Write `m26_identity_complete_unions`: `supports_permanent` admits exactly 352 kinds,
  generated-only exactly the 14 of design §4.4, `is_choice_only` exactly the 3 Wish options;
  `is_removable` is false for exactly CurseOfTheBell, Necronomicurse and AscendersBane among
  permanents; every finite list (M16, M20, M21, M25 Watcher, M10/M15/M20/M24 Run lists,
  `TRAINING_CARDS`) is unchanged and a subset of the complete union.
- [ ] Write `m26_capability_combat_admission`: under each of the four Heart contexts plus
  `with_complete_cards()`, every permanent and generated kind constructs in base and (where
  it upgrades) upgraded form with its catalog cost; a purple card under an Ironclad context
  carries the same Watcher fields as under the Watcher context; a Wish option never
  constructs; without the flag, the M10/M15/M20/M25 rejections and `Debug` text are
  unchanged.
- [ ] Write `m26_capability_run_admission`: a Run with `CardAdmission::Complete` enters
  combat with `complete_cards()` true; a default Run's combat context and text are
  unchanged; `CompleteCombatStart` accepts an Ironclad deck holding StrikePurple,
  Zap, Apparition, Regret and SearingBlow+3, and rejects a deck holding a status, a Shiv,
  an upgraded curse or a Wish option before any Run state is built.
- [ ] Write `m26_capability_overlay_inert`: for each character, the unstripped own-colour
  combat script H0 names gives byte-equal public outcomes with and without
  `with_complete_cards()`. (Task M1b reruns it after the gate changes.)
- [ ] Run `cargo test -p spire-core --locked --offline m26_identity` and `m26_capability`
  through RED, minimal change, GREEN; then the `m21_`, `m24_`, `m25_` filters and
  `--test m21_watcher_admission` once.
- [ ] Commit; publish H1 with its manifest.

## Sub-phase 2: Build

### Task M1b: Character-sensitive laws and Alchemize (Main) → H1b

**Files:** `core/combat/cards/watcher.rs` (including `hook` and `capture`); `core/combat/hooks.rs`; `core/combat/powers.rs`; `core/combat/effects.rs`;
`core/combat/engine.rs`; `core/combat/turns.rs`; `core/combat/cards/mod.rs` (damage gate);
`core/combat/piles.rs` (discard gate); `core/combat/cards/defect.rs`;
`core/combat/cards/ironclad/reactive.rs`; `core/combat/generation.rs` (Alchemize, Hello World
and Creative AI gates); `core/items/potion_generation.rs` (Ironclad and Watcher class pools);
`core/items/mod.rs` (overlay potion-belt admission); every further site H0 lists; new
`core/combat/tests/m26_offcolour_laws.rs` and its `mod` line.

**Interfaces:**
- Consumes H1 (`complete_cards()`, the four base contexts, the stripped matrix context
  named in H0).
- Produces the new gate at every design §6.3 site, by its class.
- Produces `IRONCLAD_POTIONS` and `WATCHER_POTIONS` (30 shared plus BloodPotion, Elixir,
  HeartOfIron; plus BottledMiracle, StancePotion, Ambrosia) and their limited forms
  (FruitJuice removed) in `items/potion_generation.rs`, with `candidates(character, limited)`
  answering for all four characters. The Silent and Defect pools are unchanged.

- [ ] Write `m26_offcolour_stance_mantra_scry`:
  - Ironclad: Eruption enters Wrath and then Strike deals 12 to a fresh Jaw Worm. Vigilance
    enters Calm; leaving Calm with Eruption gives 2 energy. With Mental Fortress installed
    (base, 4), each stance change gives 4 Block.
  - Silent: two Worships give 10 Mantra, enter Divinity and give 3 energy; Strike then deals
    18.
  - Defect: Third Eye opens a Scry of 3 selection over the draw pile.
  - None of these faults.
- [ ] Write `m26_offcolour_turn_structure`: an Ironclad in Divinity at end of turn leaves it
  at the next turn start. Deva Form gives 1 energy, then 2, at successive Ironclad turn starts.
  Collect (X = 2) adds an upgraded Miracle on each of the next 2 turns. Simmering Fury enters
  Wrath and draws 2 at the next turn start. Each follows the order bound in H0 (M25 F13).
- [ ] Write `m26_offcolour_orbs_zero_slots`: for Ironclad, Silent and Watcher:
  - Zap and Cold Snap channel nothing; Cold Snap still deals 6.
  - Dualcast evokes nothing; Ball Lightning deals 7 and channels nothing.
  - Defragment gives 1 Focus. Barrage and Thunder Strike hit 0 times.
  - Consume gives 2 Focus and the slots stay 0.
  - No fault, and no orb effect is enqueued at turn start or end.
- [ ] Write `m26_offcolour_orb_passives_after_capacitor`: an Ironclad plays Capacitor (2
  slots), then Zap (1 Lightning). At end of turn the Lightning passive deals 3 to the only
  enemy. Coolheaded then channels 1 Frost into the second slot; at the next end of turn the
  Lightning deals 3 and the Frost gives 2 Block. Loop (base) at the
  next turn start invokes the front orb's passive once more. All of this happens in the
  F15-bound queue position.
- [ ] Write `m26_offcolour_turn_powers`. Each case ends the turn through the real end-turn
  action and asserts the next turn start, with base max energy 3 for every character:
  - Ironclad at 0 slots: Echo Form makes the first card of each later turn play twice (a
    Strike on turn 2 and on turn 3 each hits twice). Biased Cognition gives 4 Focus, then 3
    and 2 at the next two turn starts.
  - Charge Battery: an Ironclad, a Silent and a Watcher each have 4 energy at the next turn
    start, once.
  - Outmaneuver: a Defect and a Watcher each have 5 energy at the next turn start, once.
  - Flying Knee: a Watcher has 4 energy at the next turn start (asserting the M25 fix).
  - Ironclad: Foresight (base 3) opens a Scry 3 at turn start; Battle Hymn adds 1 Smite to
    hand at each turn start; Devotion gives 2 Mantra at each turn start (2, then 4).
  - An Ironclad with a Capacitor slot and Outmaneuver has exactly 5 energy at the next turn
    start (no Defect recharge runs, nothing is applied twice).
  - The same scripts played by each power's owning character give the pre-M26 results byte
    for byte.
- [ ] Write `m26_offcolour_retain_discard`: an Ironclad holding Protect and two Strikes at
  end of turn keeps Protect and discards the Strikes. An Ironclad hand without a self-Retain
  card discards byte-identically to the pre-M26 discard.
- [ ] Write `m26_offcolour_alchemize_all_characters`: for each character with one empty
  slot, over 200 seeds every gained potion is in that character's limited class pool, and
  every member appears. With a full belt, nothing is gained and nothing faults. After Victory
  the potion settles into the Run belt. Silent Alchemize results equal the pre-M26 results
  for fixed seeds. For the Defect, which had no Alchemize before M26, the gained potion
  equals `potion_generation::sample(Defect, true, …)` on the same stream.
- [ ] Rerun `m26_capability_overlay_inert`, then the M21–M25 Watcher and M16–M20 Defect
  suites (`m21_`, `m22_`, `m23_`, `m24_`, `m25_`, `m16_`, `m17_`, `m18_`, `m19_`, `m20_`
  filters), through an assistant.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m26_offcolour` through RED,
  minimal change, GREEN.
- [ ] Commit; record H1b.

### Task A1: New card behaviours (Lane A) → HA

**Files:** bodies in `core/combat/cards/curses.rs`; `core/combat/cards/colorless.rs` and
`core/combat/cards/colorless/**`; a Ritual Dagger arm in `core/combat/cards/ironclad/fatal.rs`
if the shared Fatal owner needs one; new `core/combat/tests/m26_cards.rs` and its `mod` line.

**Interfaces:**
- Consumes H1: identities, `card_state.rs` rows, the curse hook seams and their call sites,
  validator acceptance, `with_complete_cards()` and `record_misc_growth`.
- Produces executable behaviour for the 5 colourless cards and 12 curse rows of design
  §4.2–4.3. Decay, Doubt, Shame and Regret use the owner Burn uses for end-of-turn-in-hand
  effects. Ritual Dagger's damage reads the card's `misc`; its Fatal growth calls the
  shared Fatal owner and calls `record_misc_growth`; A1's tests assert the in-combat `misc`
  only (Run persistence is Task M3).

- [ ] Write `m26_cards_colourless_values`: a table test, base and upgraded, with literal
  values: BandageUp heals 4 → 6 and exhausts; Apparition gives 1 Intangible, exhausts, is
  Ethereal only in base; Bite deals 7 → 8 and heals 2 → 3; Jax loses 3 HP and gives 2 → 3
  Strength; RitualDagger with `misc` 15 deals 15 and exhausts, and a Fatal raises `misc` to
  18 (base) or 20 (upgraded).
- [ ] Write `m26_cards_curse_rules`: each curse is unplayable; Clumsy and AscendersBane
  exhaust at end of turn as Ethereal; Writhe starts in the opening hand; at end of turn in
  hand Decay deals 2 (reduced by Block), Doubt gives 1 Weak, Shame 1 Frail, Regret loses HP
  equal to the hand size at turn end including itself (hand of 5 → 5); Injury and
  CurseOfTheBell do nothing in combat.
- [ ] Write `m26_cards_end_of_turn_order`: Burn, Decay, Doubt, Shame and Regret together in
  hand resolve in the order bound at Task M0 (F8), with Retain and Ethereal as bound.
- [ ] Write `m26_normality_counts_autoplay_and_late_draw` (Review Focus 2).
- [ ] Write `m26_pain_stacks_and_kills_once` (Review Focus 3).
- [ ] Write `m26_necronomicurse_full_hand_and_repeat` (Review Focus 4).
- [ ] Run `cargo test -p spire-core --lib --locked --offline m26_cards` and the two Review
  Focus filters through RED, minimal change, GREEN; then `m10_`, `m15_`, `m20_`, `m25_` once.
- [ ] Commit on the Lane A branch; deliver HA with the RED list.

### Task M2: Complete pools (Main) → H2

**Files:** new `core/content/pools.rs` and its `mod` line; `core/run/rooms/m15_content.rs`
(`sample_capture_row_for` family parameter, RNG-identical); `core/combat/cards/ironclad/copies.rs`
(Infernal Blade reads the player's Attack pool); `core/content/mod.rs`
(`selectable_pool`); `core/content/cards.rs` (G1); `core/combat/generation.rs` (read the
pool owner; Ironclad and Watcher arms; turn-start hooks for Hello World and Creative AI per
F13); `core/run/rooms/m10_content.rs` (`infernal_candidates` reads the pool owner; its list moves there);
`core/run/rng.rs` (three scopes); amended tests of design §11; `crates/spire-core/tests/m26_catalog.rs`.

**Interfaces:**
- Produces in `core/content/pools.rs`:
  ```rust
  pub enum CardPool {
      Ordinary { character: Character, rarity: Option<CardRarity>, card_type: Option<CardType> },
      AnyColour { rarity: Option<CardRarity> },
      Colourless { rarity: Option<CardRarity> },
      Curse,
      Transform { character: Character, input: CardId },
      Combat { character: Character, pool: GenerationPool },
      NoteForYourself,
  }
  pub fn cards(pool: CardPool) -> Result<Vec<CardId>, RegistryError>;
  ```
  Order per design §7: accepted families keep their recorded order (Silent and Defect combat
  pools, L34, Ironclad Attack = M10's `infernal_candidates` order, Foreign Influence);
  new families are sorted Common → Uncommon → Rare, then `CardId::as_str`. Each family's size is checked
  against the literal tables below before returning; a mismatch or a `definition_only`
  member returns `RegistryError::IncompletePool`. `GenerationPool` moves to or is re-exported
  from `content` without changing its variants.
- Produces `RunRngScope::{M26CardOffer, M26CardTransform, M26CardCurse}` with spellings
  `run.m26.card.offer`, `run.m26.card.transform`, `run.m26.card.curse`, appended last.

Permanent typed sizes (no HEALING exclusion), Common/Uncommon/Rare:

| Colour | Attack | Skill | Power | Total |
| --- | --- | --- | --- | --- |
| Red | 14/11/5 | 6/17/5 | 0/8/6 | 20/36/16 |
| Green | 9/13/4 | 10/14/10 | 0/6/5 | 19/33/19 |
| Blue | 10/8/5 | 8/20/6 | 0/8/6 | 18/36/17 |
| Purple | 10/12/3 | 9/15/10 | 0/8/4 | 19/35/17 |
| Colourless | 0/4/1 | 0/16/10 | 0/0/4 | 0/20/15 |

Combat sizes (HEALING excluded), Common/Uncommon/Rare: Ironclad Attack 14/11/3, Skill
6/17/5, Power 0/8/6 (Any 70, Common 20); Watcher Attack 10/12/2, Skill 9/15/9, Power 0/8/4
(Any 69, Common 19); Silent, Defect and L34 as accepted (design §7.1).

- [ ] Write `m26_pools_permanent_sizes_and_order`: every `Ordinary`, `AnyColour`
  (76/160/84), `Colourless` and `Curse` (the 10 of design §4.3) family has the sizes above
  and sorted order; Feed, Reaper and BandageUp are present; `NoteForYourself` has exactly
  349 kinds and excludes CurseOfTheBell, Necronomicurse, AscendersBane, every generated
  child, status and Wish option.
- [ ] Write `m26_pools_transform_supports`: Ironclad Strike → 72 kinds; Ironclad Anger →
  71 without Anger; Silent holding StrikePurple → Silent's 71; Apparition → 35; BandageUp →
  34 without BandageUp; Regret → 9 without Regret; CurseOfTheBell → all 10 random curses
  (the pool does not judge removability; the transform owner rejects that input first, Task M3).
- [ ] Write `m26_pools_combat_all_characters`: the combat sizes above for Ironclad and
  Watcher; Silent, Defect and L34 pools equal the pre-M26 members and order exactly; the
  Foreign Influence pool equals M25's 106 in M25's order; the Ironclad Attack pool equals the
  pre-M26 `infernal_candidates` list element for element, and `infernal_candidates` now reads
  it.
- [ ] Write `m26_pools_sampler_generalization_is_rng_identical`: for fixed seeds and keys,
  the M15, M20 and M25 reward and merchant rows from the generalized
  `sample_capture_row_for` equal the rows captured before the change (captured in the
  evidence directory by Task M0).
- [ ] Write `m26_pools_infernal_blade_player_pool`: in frozen M10 contexts (no
  `source_character`) the Infernal Blade draws for the captured seeds equal the H0 captures
  byte for byte. A Silent's, Defect's and Watcher's Infernal Blade, over 200 seeds each,
  creates only that character's Attack-pool members; an Ironclad's creates only red Attacks.
- [ ] Write `m26_pools_selectable_and_eligibility` (G1, G2): `selectable_pool(OrdinaryReward,
  Watcher)` returns the 71 purple cards; every one of the 71 is `OrdinaryReward`- and
  `ShardOrdinaryReward`-eligible for the Watcher; amend the three accepted assertions of
  design §11 with the original bytes retained in the evidence directory.
- [ ] Write `m26_pools_rng_scopes`: the three scopes follow M25's last scope with the
  spellings above; every earlier scope's position and spelling is unchanged.
- [ ] Run `cargo test -p spire-core --locked --offline m26_pools` through RED, minimal
  change, GREEN; then `m10_`, `m15_`, `m20_`, `m21_`, `m25_` and
  `--test run_content --test m21_watcher_admission` once.
- [ ] Commit; publish H2.

### Task M3: Run mutations, journal and diagnostics (Main, on H2′) → H3

**Files:** `core/combat/cards/ironclad/autoplay.rs` (Normality's blocked-autoplay
disposition, F9); `core/run/mutations.rs`; `core/run/combat.rs` (journal acceptance at ~40 and
~366, which today reject nonzero `misc` for any card but Genetic Algorithm, and the census);
new `core/run/diagnostics/m26.rs` and its `mod` line; `crates/spire-core/Cargo.toml`
(`m26-diagnostics = ["m25-diagnostics", …]` plus any feature whose scenario helpers H0 says it needs, such as `m20-diagnostics`); new `core/run/tests/m26_card_mutations.rs` and
its `mod` line.

**Interfaces:**
- Consumes H2′: `CardAdmission`, `content::complete`, `content::pools`, scopes, and HA's
  Ritual Dagger, Bite and curses.
- Produces, in `core/run/mutations.rs`, under `CardAdmission::Complete` (and unchanged
  under `Profile`):
  - `obtain_card(state, id, upgrades) -> Result<u64, RunError>` (existing; design §8.1 limits
    and `misc` 15 for RitualDagger);
  - `upgrade_card`, `can_upgrade`, `remove_card` (existing; design §8.2);
  - `pub(super) fn transform_card(state: &mut RunState, instance: u64, auto_upgrade: bool,
    key: &[u64]) -> Result<u64, RunError>` (design §8.3; returns the new instance);
  - `pub(super) fn duplicate_card(state: &mut RunState, instance: u64) -> Result<u64, RunError>`
    (design §8.4);
  - `pub(super) fn obtain_stored_card(state: &mut RunState, id: CardId, upgrades: u32)
    -> Result<u64, RunError>` (design §8.5);
  - `pub(super) fn draw_card(state: &mut RunState, pool: CardPool, key: &[u64])
    -> Result<CardId, RunError>` and `draw_distinct(state, pool, count, key)` over
    `m15_content::sample_capture_row_for` and the `run.m26.card.offer` and
    `run.m26.card.curse` scopes.
- Produces the Ritual Dagger `misc` journal entry next to Genetic Algorithm's, applied once
  at settlement, and the census accounting for it.
- Produces in `core/run/diagnostics/m26.rs` (feature `m26-diagnostics`):
  `pub fn complete_run(character: Character, start: AuthoredRunStart) -> Result<Run, RunError>`
  (the character's complete Heart scenario, `CardAdmission::Complete`, resources checked by
  `CompleteCombatStart`) and `pub fn complete_combat(character: Character,
  start: AuthoredRunStart, encounter: EncounterId) -> Result<Run, RunError>` (the same, then
  combat entered through `Run::from_validated_combat_resources` and `Run::enter_owned`).

- [ ] Write `m26_mutation_obtain_upgrade_remove`: under `Complete`, for each character,
  every one of the 352 permanent kinds is obtained base, and upgraded where it upgrades;
  SearingBlow is obtained with 3 upgrades and upgraded at Smith twice more; RitualDagger
  starts at `misc` 15 and GeneticAlgorithm at 1; every removable kind is removed with
  `CardRemoved` once; Parasite removal loses 3 max HP.
- [ ] Write `m26_mutation_rejections_are_atomic` (Review Focus 1).
- [ ] Write `m26_mutation_transform`: with a fixed seed and key, an Ironclad Strike becomes
  a red Common/Uncommon/Rare card other than Strike, fresh (0 upgrades, base `misc`); a
  Silent's StrikePurple becomes a green card; Apparition becomes an ordinary colourless card;
  Regret becomes one of the other 9 curses; Parasite transform loses 3 max HP;
  `auto_upgrade` upgrades an upgradeable output; over 2,000 seeded transforms of one input
  every support member appears and no other kind does.
- [ ] Write `m26_mutation_duplicate_and_note`: duplicating SearingBlow+4, RitualDagger at
  `misc` 21 and GeneticAlgorithm at `misc` 4 gives new instances with equal id, upgrades and
  `misc`; `obtain_stored_card(SearingBlow, 4)` gives 4 upgrades and `obtain_stored_card(
  RitualDagger, 1)` gives `misc` 15; a Miracle or Wound cannot be stored or retrieved.
- [ ] Write `m26_mutation_pool_draws`: `draw_distinct(Ordinary{Watcher, Rare, None}, 3)`
  returns 3 distinct purple Rares and commits RNG only on success; an incomplete family
  fails before any RNG counter moves.
- [ ] Write `m26_ritual_dagger_growth_exclusions_and_persistence` (Review Focus 5), using
  `complete_combat`.
- [ ] Write `m26_normality_blocks_autoplay` (Review Focus 2, autoplay half): with Normality
  in hand after 3 plays, a Havoc, a Mayhem top card and a Double Tap or Burst repeat do not
  play. Each blocked card ends where F9 binds, with no fault and no fourth count.
- [ ] Write `m26_diagnostics_constructors`: `complete_run` and `complete_combat` build each
  character's Run with zero accepted actions and reject resources outside the complete
  union before returning a Run. The module compiles under
  `cfg(any(test, feature = "m26-diagnostics"))` like the existing ones.
- [ ] Run `cargo test -p spire-core --locked --offline m26_mutation` and
  `--features m26-diagnostics m26_` through RED, minimal change, GREEN; then the
  `m10_`, `m15_`, `m20_`, `m24_`, `m25_` Run filters once.
- [ ] Commit; publish H3.

### Task B1: Cross-character, generation and copy matrices (Lane B, on H2′)

**Files:** new `core/combat/tests/m26_support.rs`, `m26_cross_character.rs`,
`m26_generation.rs`, `m26_copies.rs` and their `mod` lines.

**Interfaces:**
- Consumes H2′: `with_complete_cards()`, `content::complete`, `content::pools::cards`, the
  stripped matrix context H0 names, and HA's cards.
- Produces `m26_support::play_delta(character: Character, card: CardId, upgrades: u32)
  -> PublicDelta`: the stripped context of design §14.3 (no relics, 3 empty potion slots, player 80/80,
  energy 10, H0's encounter and five-Defend draw pile), the card in hand, played on the first
  enemy if playable, any selection resolved by "first legal candidate", the turn not ended.
  `PublicDelta` is the public state after the play minus the same character's state just
  before it: player and enemy HP, Block, energy and powers, stance, pile contents by kind and
  created cards, plus orb slots and orbs only when requested for a design §6.2 orb row.

- [ ] Write `m26_cross_character_equivalence`: for every permanent and generated kind,
  base and upgraded, and every character, `play_delta` equals the owning colour's delta
  (Ironclad for colourless, curses and statuses), except the kinds in design §6.2.
- [ ] Write `m26_cross_character_sensitive_rows`: for every design §6.2 orb card, the
  literal delta for each non-Defect at 0 slots (as in Task M1b's zero-slot test); for every
  generator, membership of each created card in the player's pool; for Alchemize, membership
  in the class pool.
- [ ] Write `m26_generation_player_pool`: for each character, Infernal Blade, Distraction,
  White Noise, Discovery, Chrysalis, Metamorphosis, Hello World and Creative AI create only
  members of that character's pool of the right type (over 200 seeded plays each, every
  created kind is in the pool); Jack of All Trades, Transmutation and Magnetism create only
  L34 members for every character.
- [ ] Write `m26_copies_every_kind`: for every kind, base and upgraded, plus SearingBlow+3,
  RitualDagger `misc` 21 and GeneticAlgorithm `misc` 4, `stat_equivalent_copy` keeps id,
  upgrades and `misc`, gives a new instance and copy family, and rebuilds Ethereal,
  Exhaust, Innate and X from the kind; Dual Wield, Nightmare, Anger, Amplify and Echo Form
  copies of off-colour cards played by each character match; Necronomicurse's returned card
  is a fresh base copy with a new combat-local instance.
- [ ] Write `m26_generated_and_status_every_character`: each of the 9 children and 5
  statuses is created by its accepted producer and played or resolved by each character
  with the owning-colour outcome.
- [ ] Run `cargo test -p spire-core --lib --locked --offline m26_cross m26_generation
  m26_copies m26_generated`. Record every failing row with the kind, character, outcome
  diff and suspected owner. Do not change non-test files.
- [ ] Commit on the Lane B branch.

### Task B2: Supplied mixed decks (Lane B, on H3) → HB

**Files:** new `core/run/tests/m26_supplied.rs` and its `mod` line.

- [ ] Write `m26_supplied_mixed_deck_<character>` for each character: `complete_combat`
  against the encounter H0 names, with a deck of 15 cards mixing that character's starters, three off-colour ordinary cards
  (one per other colour), two colourless, RitualDagger, Bite and two curses; play to the
  combat end with the fixed rule "first legal non-end-turn candidate, else End Turn",
  bounded at 400 actions (reaching the bound fails the test; it is not a fault and not a
  loss); the combat ends in Victory or Death without a fault, settlement
  passes the census, and the Run deck equals the supplied deck plus any recorded journal
  change.
- [ ] Run `cargo test -p spire-core --features m26-diagnostics --locked --offline
  m26_supplied`. Record each result at grade "Supplied played".
- [ ] Commit; deliver HB (B1 and B2) with the RED list.

## Sub-phase 3: Integrate and close

### Task M4: Integration, guard fixes and freeze gate (Main)

**Files:** the files named by each RED's owner; amended accepted tests with disclosures.

- [ ] Integrate HB and any Lane A fixes delivered after H2′, each with recorded pre- and
  post-integration manifests.
- [ ] For each residual RED in HB: decide source law (design §6.2 row, test expectation
  corrected with its citation) or defect (fix at the shared owner for all characters,
  regression test that fails under a narrower fix, disclosure entry). A site missing from
  the design §6.3 table is added with its class and gate by that rule.
- [ ] Re-run every `m26_` suite to GREEN.
- [ ] Freeze gate (design §14.4): on the exact tree run
  `cargo test --workspace --locked --offline --no-fail-fast` and
  `cargo test -p spire-core --features m26-diagnostics --locked --offline`; failures other
  than the inherited, documented ones block the freeze. Retain the tree manifest and logs in
  `/home/klottick/.local/share/spire-gym/evidence/m26-execution-20261031/main-v1/freeze/`.
- [ ] Commit.

### Task M5: Documents and review packet (Main)

**Files:** new `docs/milestones/m26-shared-cards.md` (profile),
`docs/milestones/m26-shared-cards-coverage-ledger.md` (one row per kind and pool with its
test key K/U/X/G/Y/R/S and grade), `docs/milestones/m26-shared-cards-reconciliation.md`
(global corrections with original bytes' locations, disclosed project decisions, frontier
bindings, guard audit result); `docs/work-status.md` (new dated snapshot); `AGENTS.md`
table row for M26.

- [ ] Write the three milestone documents from the evidence; every claim names its grade
  and test.
- [ ] Check every changed link, path and command; `git diff --cached --check` on the staged
  explicit paths.
- [ ] Commit; build the read-only candidate export with manifest and evidence index; hand
  it to the PM for the one fresh final review (design §14, M25 design §14 pattern).

## Planning delivery check

| Design section | Task |
| --- | --- |
| §4 cards, §5.1 catalog and unions | M1, A1 |
| §6.2 Alchemize, §1.1 potion row | M1b |
| §5.2–5.3 capability and admission | M1 |
| §6 off-colour play, gates and inertness | M0, M1, M1b, B1, M4 |
| §7 pools, draw law, scopes | M2, M3 |
| §8 Run mutations, journal, census | M3 |
| §9 copies, §10 generated and statuses | B1 |
| §11 corrections | M2, M4 |
| §12 frontiers | M0, A1, M3, M4 |
| §13 diagnostics | M3 |
| §14 evidence and freeze | B2, M4, M5 |
| §16 M25 interfaces | M0 |
