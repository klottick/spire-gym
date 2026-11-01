# M26 source and API reconciliation

Status: **source frozen at freeze v2 (2026-11-01), candidate-v2 for the independent final
review.** Tasks
M0–M3 and Lanes A, B1 and B2 were built early on the frozen, unmerged M25 source and then
rebased onto merged M25 (`main` at `3729b29e`, PR65). The design §16 and §6.3 reruns on the
merged source found no unaccommodated difference. This record accompanies the approved
[design](../superpowers/specs/2026-10-31-m26-shared-cards-design.md) and
[plan](../superpowers/plans/2026-10-31-m26-shared-cards.md) (version 3, design SHA-256
`16db3830…`, plan `ed02ae01…`), which are unchanged. It maps their proposed names to the
actual owners and records every ruling, deviation, global correction and amended assertion.
The profile is [m26-shared-cards.md](m26-shared-cards.md) and the per-kind and per-pool test
map is the [coverage ledger](m26-shared-cards-coverage-ledger.md).

## Source binding

- **Early start (PM decision, 2026-10-31).** Base: `main` at
  `7b036e8d2047f17eb765fe42b1c4bb8e164a37e9` plus the 109 frozen M25 paths (0 mismatches
  against the M25 freeze-gate tree manifest) plus the two approved M26 documents. Record
  `m26-early-base-v1.json` (`0e4365d3…`). Every snapshot since (H0, H1, H1b, H2′, H3, H4-pre)
  is a working-tree manifest; nothing is committed before the freeze.
- **Rebase (2026-11-01).** Onto `main` at `3729b29eed33317cd2d6573b9d5ad7d1c7fc441c` (M25
  merged in PR65) by a per-file three-way merge: base = the frozen M25 tree above, theirs =
  merged main, ours = the M26 tree. Of 1033 files, 19 changed only in M25 (taken), 66 only in
  M26 (kept) and 9 in both. Six of the nine merged cleanly. Three had trivial conflicts: the
  unplayable list in `combat/cards/mod.rs`, where both sides had added Deus Ex Machina; the same
  Watcher-initializer line in `card_state.rs`, now applied once; and the test `mod` lists.
  The method, the per-file classification and the rebased tree manifest are retained in
  `main-v1/rebase/` in the execution evidence.
- M25's final review changed files M26 also edits; any non-accommodated difference would go
  back to the plan reviewer as an M26 amendment (design §16). None was found (below).
- **First freeze (2026-11-01, PM proceed).** The frozen tree is `main` `3729b29e` plus the
  M26 working-tree delta: 1033 files, tree manifest SHA-256 `c5ba3d5f…`, retained with the
  suite logs in `main-v1/freeze/` (index `SHA256SUMS` `0466b706…`). Suites on that exact tree:
  workspace 3456 passed / 0 failed / 11 ignored; `-p spire-core --features m26-diagnostics`
  2830 / 0 / 5; `--lib m26_` 50 / 0; `--test m26_catalog` 5 / 0. No failure is inherited.
  It was reviewed as candidate-v1 and superseded by freeze v2.
- **Freeze v2 (2026-11-01, after the final review v1 corrections).** Same base plus the
  corrected delta: 1035 files (one new test file and the new coverage ledger), tree manifest
  SHA-256 `c050eec2…`, retained in `main-v1/freeze-v2/` (index `SHA256SUMS` `ba45a882…`),
  verified by a separate assistant (`main-v1/freeze-v2-verify/`). Suites on that exact tree:
  workspace 3461 / 0 / 11; `--features m26-diagnostics` 2835 / 0 / 5; `--lib m26_` 55 / 0;
  `--test m26_catalog` 5 / 0. No failure is inherited.

## Task M0: design §16 interface check

Against the frozen M25 source, every §16 row matched or differed only as its "Accommodated
differences" column allows (seam map H0 §1):

- the colour dispatcher (`watcher::initialize_m25_profile`) is a different internal function
  with the same results;
- the Foreign Influence pool lives in `content::watcher::foreign_influence_support()`;
- the permanent-upgrade journal is the `PermanentCardUpgrade` vector beside
  `PermanentCardMutation`;
- the Plated Armor gate uses different wording with the same results;
- M25's routed energy fix is larger than the minimum (the Watcher recharge already applies
  `energized`, `energized_blue` and `energy_down`), so G7 shrinks accordingly.

No amendment was needed on frozen M25.

**Rerun on merged M25 (H0 v2, 2026-11-01).** Every row is again the same or accommodated.
Two rows changed in an accommodated way: merged M25 now applies the Watcher initializer inside
`card_state::generated` (M26's correction 2, adopted by M25), and M25 added and amended Master
Reality tests. The §6.3 sweep found no new or removed character or profile gate. The sites
M25 changed (Watcher fixed children, Foreign Influence, Collect with Chemical X) sit behind
M26's existing L gates. No amendment is needed.

## Bindings

| Proposed (design or plan) | Bound to |
| --- | --- |
| Complete capability | `content::complete::{supports_permanent, supports_generated, is_choice_only, is_removable}`, computed from catalog availability and note storage (352 / 14 / 3) |
| Combat overlay | `CombatItemContext::{with_complete_cards, complete_cards}`; its Debug text prints `complete_cards: true` only when set |
| Overlay trackers | At combat entry under the flag: `source_character`, draw and discard history, `thief_gold` (with the `thieves.rs` gold-custody validator), the PlayerStart phase and the opening Decision. All five are **Overlay** class, proved by `m26_capability_overlay_inert` for all four characters |
| Colour dispatcher | `cards::initialize_complete`: Watcher kinds → the M25 Watcher initializer, Blue → `defect::initialize`, everything else → shared defaults; `combat::complete_live_card` |
| Curse hooks | `combat/cards/curses.rs::{play_blocked, end_of_turn_in_hand, on_card_played, on_exhaust}`, called from `validate_card_use` (also the action mask), `status::end_turn_work` (the Burn owner, in hand order), the `CardPlayed` hook and `exhaust::dispatch`. Validator `curses::sourced_from` admits Decay's 2 blockable damage and Necronomicurse's card creation |
| Ritual Dagger journal seam | `defect::record_misc_growth` (the Genetic Algorithm growth generalized), journalling `PermanentCardMutation` for GeneticAlgorithm or RitualDagger. `defect::validate_mutations` and the Run census (`run/combat.rs`) accept both, keyed by the journalled id |
| Run admission | `run::CardAdmission::{Profile (default), Complete}` on `RunState`. `RunState`'s Debug is now hand-written and identical to the derived text, printing `card_admission` only when Complete, so M25 prefix tests that compare Debug text are unchanged |
| Supplied combat start | `run::CompleteCombatStart::Supplied(AuthoredRunStart).normalized(character)`, following `WatcherAdmission`; helpers `complete_upgrades_valid`, `complete_obtain_misc`, `complete_heart_context`; `Run::prepare_complete_combat` (crate-private) |
| Run entry under Complete | `enter_owned` applies `complete_heart_context`; `prepare_entry` no longer routes a Complete Defect Run into the frozen M16 path. Profile Runs are unchanged |
| Gate helpers (§6.3) | `CombatItemContext::m25_cards()` (= M25 profile or complete) and `watcher::cards_active(state)` (= Watcher active or complete) |
| Pool owner | `content::pools::{CardPool, cards}`; `GenerationPool` moved there and re-exported unchanged. **Additive:** `CardPool::ForeignInfluence` (design §7.1 lists it as a combat pool; the plan's enum omitted it). `Catalog::selectable_pool` reads it |
| Sampler | `m15_content::sample_capture_row_for(rng, semantic, slots, scope, incomplete, empty)`, one family per slot; the M15, M20 and M25 wrappers build their slots from their existing families. RNG-identical on 48 captured M0 rows |
| Run mutation owners | `run/mutations.rs`: `obtain_card`, `upgrade_card`/`can_upgrade`, `remove_card`, `transform_card`, `duplicate_card`, `obtain_stored_card`, `draw_card`, `draw_distinct`, one allocator `push_card`. Each mutates a staged clone and commits only on success. The new owners carry `allow(dead_code)` outside tests because their callers are M27/M28 |
| Diagnostics | `run/diagnostics/m26.rs::{complete_run, complete_combat}`; feature `m26-diagnostics = ["m25-diagnostics", "m20-diagnostics"]`, default off |
| Searing Blow bound | The existing `u32` upgrade count; no new bound (F5) |

## Off-colour gates (design §6.3)

Every ruled site was gated by its class: **L** (card law) gains `|| complete_cards()` through
the helpers; **S** (shared path) gains the state that only off-colour play creates; **C**
(character structure) is kept. Full table in seam map H0 §3 and H1b. Notable sites:

- **S:** the damage pipeline (stance not Neutral or Vigor present), the discard law (a held
  Retain card), public `mantra_gained` (> 0), the Defect orb queue position (an orb slot exists).
- **Splits** keep each character's own order: `watcher::hook` (Divinity exit S, power callbacks
  L, recharge C); `defect::turn_started` (Echo Form and Biased Cognition L, Loop and start
  passives S, Emotion Chip, Gold-Plated Cables and recharge Defect-only).
- **C kept:** the deferred-recharge split, `stat_loss_work` timing and the
  `round_end_full_work` flag (switching them would move own-colour Flex timing in M10), and
  Magnetism's M20 routing.
- **Deus Ex Machina's on-draw trigger** was Watcher-gated; it is now an L site and fires
  off-colour.

Rulings that differ from the seam map (the H1b rulings):

1. **Ritual and Regeneration callbacks are not gated.** No card grants them (only potions,
   M27), and outside M15 the Ritual tick belongs to `round_end_full_work`, so gating it would
   apply it twice. Plated Armor (from Wish) is gated `m15 || m25_watcher || complete`.
2. **`watcher::card_values` stays on `active`.** It is a public per-card observation, kept for
   the same reason as `previous_card_type` (recording ungated, public field unchanged). The
   Stance Potion stance-choice validator is also kept (a potion).
3. **`defect::validate_mutations`** accepts GeneticAlgorithm or RitualDagger, keyed by the
   journalled id (it was GeneticAlgorithm-only).
4. **`card_state::generated`** gained arms for every M26 identity plus Regret and Ascender's
   Bane, needed for Necronomicurse's returned copy (F11).

## G1–G7 (design §11)

| # | Result |
| --- | --- |
| G1 | `eligible_for`'s Watcher fence removed; eligibility follows rarity and colour only. Amended `tests/m21_watcher_admission.rs` |
| G2 | `selectable_pool` returns the complete pools: OrdinaryReward → Ordinary, Shard → AnyColour, Note → NoteForYourself, PureWater → [Miracle], Unregistered → `UnknownProducer` (a correction to the G2 row, which did not name the last two). Amended `tests/run_content.rs`, `tests/m21_watcher_admission.rs`, the `content/defect.rs` test |
| G3 | L34 derived from the catalog; same 34 members and order |
| G4 | Ironclad and Watcher combat arms and the Silent Common arm exist through the pool owner. **Infernal Blade** draws from the player's Attack pool; when `source_character` is absent (frozen M10 contexts) it uses the Ironclad pool in M10's recorded order, so M10 draws are byte-identical (16 captured rows). Hello World and Creative AI resolve for every character (F13). Amended `combat/tests/m15_generation.rs:49`, `combat/tests/m20_defect_generation.rs:18` (Ironclad Any is now 70, not an error) |
| G5 | The §6.3 gates above |
| G6 | Alchemize is legal for every character and draws from the player's class pool, limited form (30 shared + 3 class, Fruit Juice excluded), into the first empty slot; with a full belt it gains nothing. New class pools: Ironclad Blood Potion (C), Elixir (U), Heart of Iron (R); Watcher Bottled Miracle (C), Stance Potion (U), Ambrosia (R) (F16). The overlay potion-belt admission in `items::validate_combat_inventory` checks the class pool under the overlay. Silent's pre-M26 seed results are pinned unchanged. Amended `items/m15_potion_generation_tests.rs:37` (Ironclad candidates were an error; now the 32-member pool) |
| G7 | Next-turn energy law (= the Watcher order): `max`, + `energized`, + `energized_blue`, − `energy_down` (the power persists), floor 0, cap 999. The non-deferred owner (`turns.rs`) gained `energized_blue` and `energy_down`; the Defect owner gained `energy_down`. **Plan review v3 Minor 1:** Fasting's `energy_down` is part of this law, pinned by `m26_offcolour_fasting_energy_down` |

## Global corrections found during execution

Each is fixed at the shared owner for every character, with a regression test that fails
under a narrower fix.

1. **Red generated constructors.** 22 red Skills and Powers (Armaments, Havoc, Shrug It Off,
   True Grit, Warcry, Battle Trance, Bloodletting, Burning Pact, Disarm, Dual Wield, Entrench,
   Ghostly Armor, Inflame, Intimidate, Power Through, Seeing Red, Shockwave, Spot Weakness,
   Double Tap, Exhume, Impervious, Offering) had no `card_state::generated` arm, so any
   generator of them returned `Unsupported`. Red C/U/R kinds without a legacy arm now use
   their permanent descriptors, as the other colours already did. Legacy red arms unchanged.
2. **Watcher kinds built bare by `card_state::generated` (M25-owned, M25 G1).** M25 kinds
   came back without the M25 initializer; every M25 constructor applied it by hand
   afterwards, so off-colour generators, Infernal Blade and Stasis's returned card got the
   bare descriptor. M26 found this during M2 and fixed it in its early tree; M25's final
   corrections made the same fix at the same owner (M25 G1: `generated()` applies
   `initialize_m25_base_profile` for M25 kinds). On merged main the fix is M25's, and the
   rebase kept one copy of that line. M26 adds only the regression test
   `m26_generated_watcher_kinds_match_their_m25_constructor`.
3. **Clash base cost read from play legality.** `cards::base_cost` asked Clash's payload,
   which rejects while a non-Attack is in hand, so a plain Ironclad Infernal Blade that rolled
   Clash beside a Skill **faulted** (seeds 38, 78, 85, 87 and 111 of 120, frozen-M10 shape).
   Confused randomization and Liquid Memories also skipped such a Clash. Clash's starting
   cost 0 now comes from `ironclad::starting_cost`, as Silent's Grand Finale does. Test
   `m26_infernal_blade_clash_beside_a_skill`.
4. An unreachable duplicate exhaust arm (Bandage Up, Apparition, Ritual Dagger) in
   `card_state::generated` was removed.
5. **Deus Ex Machina missing from the shared unplayable law (M25-owned, M25 G2).** A Deus
   Ex Machina in hand without its on-draw trigger (Discovery, Foreign Influence, copies) made
   legal-action enumeration return `Unsupported("card handler not implemented")` for every
   character, Watcher included. Lane B1's matrix X found it (`m26_cross_character_equivalence`,
   10 rows RED), and M26 fixed it in its early tree; M25's final corrections added Deus Ex
   Machina to the same unplayable list in `combat/cards/mod.rs` (M25 G2). On merged main the
   entry is M25's, and the rebase kept one copy. M26's own change to that list is its 11
   unplayable curses; `m26_cross_character_equivalence` and `m26_offcolour_deus_ex_machina_draw`
   pin the law for every character.
6. **Non-removable curses.** `remove_card` now rejects Curse of the Bell, Necronomicurse and
   Ascender's Bane for every admission (catalog-derived). No frozen profile can hold them.

### Corrections from the final review v1 (2026-11-01)

The independent final review of candidate-v1 (`m26-independent-senior-review-20261101/review-v1.md`)
found three shared defects (I1–I3) and four Minors (M1–M4). PM ruling
`pm-ruling-final-review-v1-corrections-v1.json`: fix each at its shared owner for every character.
All new tests are in `combat/tests/m26_corrections.rs`; each fails on the candidate-v1 source.

7. **End-of-turn debuffs from cards are just-applied (I1).** Rule: a Weak, Frail or Vulnerable
   applied by a monster, or by a card that plays itself at the end of the player's turn, skips its
   first end-of-round decrement. Source: Doubt and Shame apply `WeakPower`/`FrailPower(p, 1, true)`,
   the same just-applied flag monsters pass. Owner: `combat/powers.rs::just_applied`, which sets
   `monster_source_grace` in both power lanes; it was enemy-source only, so Doubt's Weak and
   Shame's Frail expired before the player's next turn. Scope: every character; only Doubt and
   Shame apply a power from end-of-turn self-play, and no frozen profile can hold them. Test
   `m26_end_turn_curse_debuffs_last_through_the_next_turn` (both curses, and a monster's Weak as
   the non-curse user of the law).
8. **Every end-of-turn self-playing card discards itself when retained (I2; M25 R4
   generalized).** Rule: Burn, Decay, Doubt, Shame and Regret use themselves at end of turn and
   then discard even when a retain effect selected them; they leave in the ordinary hand-discard
   order. Source: the wiki pages for the five cards ("This happens before Retain happens") and
   `implementation-notes.json` BURN and REGRET. Owner: one card property,
   `content::plays_itself_at_end_of_turn`, read by the `piles.rs` DiscardHand withdrawal (it
   was `CardId::Burn` only) and by the just-applied law above. Scope: every character; Burn's
   behaviour is unchanged. Test `m26_end_turn_self_playing_cards_discard_when_retained` (five
   cards × four characters). Amended assertion: `combat/tests/m26_cards.rs` (below).
9. **Fatal on the final Darkling (I3).** Rule: the hit that downs the last Darkling is Fatal;
   a non-final Darkling stays half-dead and is not Fatal. Source: `Darkling.damage` clears
   `halfDead` and calls `die()` for every Darkling when all are down, so the Feed and Ritual
   Dagger `!halfDead` checks pass; the Ritual Dagger wiki ("only considered Fatal if it kills
   the last Darkling") and the design §8.6 pin (only non-final kills are excluded). Owner:
   `enemies/act3/darklings.rs::final_down`, read by the shared Fatal check
   (`cards/ironclad/fatal.rs::after_damage`), which rejected every non-Active target. Scope:
   Feed, Hand of Greed, Lesson Learned and Ritual Dagger, every character. Test
   `m26_fatal_fires_on_the_final_darkling` (also pins the two non-final kills).
10. **Master Reality upgrades any created card that can still upgrade (M1).** The creation law
    in `piles::insert_generated` used `upgrades == 0`; it now uses the shared
    `can_upgrade(id, upgrades)`, so a Dual Wield or Nightmare copy of Searing Blow+3 is +4, as
    the source's unguarded `upgrade()` gives. Ordinary upgraded cards are unchanged (they
    cannot upgrade). Test `m26_master_reality_upgrades_a_copied_searing_blow`.
11. **Dead Dual Wield guard removed (M2).** `ironclad/choices.rs` rejected Master Reality with
    `amount > 0`, which no valid state has (its amount is −1). The guard is gone; Dual Wield
    follows the creation law. No behaviour change.

M3 (F10 not pinned) is closed by `m26_pain_autoplay_duplicate_and_end_of_turn`: Havoc's
autoplayed card and Double Tap's duplicate each trigger Pain, and end-of-turn auto-use by Burn,
Decay, Doubt and Shame does not. M4 corrected a test comment (Ritual Dagger grows +3 base, +5
upgraded).

### Corrections that change frozen behaviour

These are global under rule 2 and change what a frozen profile can observe:

- **Clash (3), frozen M10 Ironclad.** A rolled Clash beside a non-Attack no longer faults the
  combat. An unplayable Clash's public `effective_cost` is now `Some(0)` instead of `None`.
- **Fatal on the final Darkling (9).** Frozen profiles can reach it: an Ironclad Feed (M9/M10),
  a Hand of Greed wherever a profile can hold or generate it, or a Watcher Lesson Learned
  (M24/M25) that downs the last Darkling in Act 3 now gains its Fatal reward. The retained-outcome sweep (below) found
  no accepted receipt that reaches it.
- Corrections 7, 8, 10 and 11 change nothing a frozen profile can reach: no frozen profile can
  hold Doubt, Shame, Decay or Regret, Burn already followed R4, and only an off-colour Searing
  Blow meets Master Reality.
- Corrections 2 and 5 change M25 Watcher behaviour (Stasis-returned exhaust kinds; a Deus Ex
  Machina in hand without a draw), but they are M25's own corrections on merged main, so
  relative to the M26 base they are not M26 changes.

### Retained-outcome sweep (PM ruling, 2026-11-01)

Before freeze v2, a Codex assistant scanned 364 retained outcome artifacts of the accepted
M13–M25 profiles (attempt JSON, traces, session transcripts, outcome manifests) for Doubt, Shame,
Decay, Regret, a held Burn with a retain effect at End Turn, a Fatal card in a Darkling combat,
and Master Reality with Searing Blow. In scope: 39 M14 traces with a Burn held under
`retain_cards` and one M15 attempt/trace pair (End Turn steps 389, 403, 410). No receipt has a
Fatal card in a Darkling combat or any of the four curses. All 40 replayed same-action on the
corrected source with `identical: true` and equal endpoints, using the M25 no-drift harness;
there was no divergence. No M9 gameplay receipt exists; the retained M10 receipts are command and
test receipts, not replayable transcripts, and the accepted M10 Fatal and Darkling suites pass.
Evidence: `main-v1/corrections-v1/sweep/` (`inventory.md`, `results.jsonl`, `REPORT.md`).

### M25 final-review laws M26 inherits

Merged M25 changed shared owners after M26's early base. M26 follows them:

- **Master Reality creation law (M25 I-1).** `piles::insert_generated` upgrades every created
  non-Curse, non-Status base card once, for every character and destination. Only Watcher fixed
  children and Foreign Influence use their own checkpoints (`MasterRealityRoute::RouteChecked`).
  A creation-route sweep of `combat/**` and `items/**` found no site that creates a card
  without this owner. **Ruling:** the queued autoplay replays (Double Tap, Burst, Duplication,
  the Defect repeat, Omniscience) are not creations, because they are same-instance card-queue
  replays, not "make a temporary card" actions; they are unchanged.
  `m26_generation_master_reality_every_character` covers all 11 M26 generators plus Dual Wield,
  for all four characters. It passed on its first run, because the merged law already covered
  them, so it is a regression test. Disabling the law at `piles.rs:190` turns every row RED,
  and restoring it turns them GREEN again. `m26_generation_master_reality_curse_return_stays_base`
  checks that a returned Necronomicurse stays base.
- **Collect counts Chemical X** (+2): Collect's X-cost and its power validation read
  Chemical X (`m25_powers.rs`, `watcher.rs`). Both sit behind M26's existing L gates
  (`cards_active`, `m25_cards`), so an off-colour Collect follows the same law. No M26
  change.
- **Burn retention (M25 R4)** and the end-of-turn card-queue cancel: shared and
  character-independent (M25's `combat/tests/m25_burn_retain_all_characters.rs`). M26's curse
  end-of-turn work runs in the same Burn owner (F8), and correction 8 extends R4's retention
  withdrawal from Burn to every card with the same property.

The workspace suite on the frozen tree (above) includes every M25 test for these laws.

## Frontier bindings (design §12)

**Settled** by a cited source: F2, F3, F6, F7, F13, F14, and the counting half of F9. The
rest are **disclosed project decisions**, each pinned in a test labelled with its number. F1 is
`m26_any_colour_draw_is_uniform_over_the_flat_family`, F4 is in `m26_identity_catalog_rows` and
F12 is `m26_offcolour_orbs_zero_slots`; all three were labelled at the rebase.

| # | Decision |
| --- | --- |
| F1 | Any-colour draws are uniform over the flat family within a rarity (76/160/84); no per-colour weighting |
| F4 | Bite and Ritual Dagger carry no tags; Bandage Up is HEALING. Special cards are in no combat pool, so this changes no pool |
| F5 | A stored Searing Blow keeps any count the `u32` upgrade rule represents |
| F8 | Decay, Doubt, Shame and Regret run in the Burn owner, one per card in hand order, after the pre-end-turn power work (Metallicize) and before Ethereal exhaust and the hand discard. A retained curse still triggers, then discards itself (correction 8; the retain half is source law, not a decision). Regret's amount is the hand size when the packets are built, including Regret. None emits `CardPlayed` |
| F9 | See "F9 Normality ruling" below |
| F10 | Pain triggers on every other `CardPlayed`, autoplayed and duplicated plays included; not on end-of-turn status or curse auto-use (`m26_pain_autoplay_duplicate_and_end_of_turn`) |
| F11 | Necronomicurse's returned card is a fresh base copy with a new combat-local instance, created after the exhaust triggers; to the discard pile with a full hand. The permanent instance stays in the exhaust pile for the census |
| F12 | Consume at 0 slots gives its Focus and slots stay 0; Fission and Multi-Cast at 0 orbs do nothing (Fission 0 energy, 0 draw); Recursion with no orb does nothing; Barrage, Compile Driver and Thunder Strike read 0; Capacitor adds slots for anyone (cap 10) |
| F15 | The Defect orb phase position is used for every character with an orb slot |
| F16 | Watcher class potions Bottled Miracle / Stance Potion / Ambrosia (Ironclad's are settled by the research record); Fruit Juice is the only limited exclusion for every class |

### F9 Normality ruling (amended at M3)

Counting is settled: autoplayed and duplicated plays count, and drawing Normality after 3 plays
blocks the rest of the turn. For a blocked autoplay, H0 said "Havoc and Mayhem cards go to the
discard pile". **M3 amended this**: a blocked autoplay child uses the existing shared
unplayable-autoplay owner (`FinishSuppressed` with the frame's disposition), with no code
change:

- a Havoc'd card is **exhausted** (Havoc's own exhaust, as the accepted
  `m10_havoc_recycles_without_draw_callback_and_suppresses_unplayable_status` pins for Dazed);
- a Mayhem card is discarded;
- a Double Tap or Burst repeat is purged.

No `CardPlayed`, no count, no energy and no fault. A single Mayhem cannot be blocked (the count
resets first); the reachable case is four Mayhems with a retained Normality. Time Eater's count
and Vault/Conclude are unaffected. Tests `m26_normality_blocks_autoplay`,
`m26_normality_counts_autoplay_and_late_draw`.

## Amended accepted assertions

Each amendment keeps the original assertion as a comment in place (`pre-M26:` or "Original
line:"). Pre-M26 bytes are main `7b036e8d` for tracked files and the frozen M25 tree for M25
files; edits from M2 and M3 are also retained under `main-v1/m2/original-bytes/`,
`m2/tests-original-bytes/` and `m3/original-bytes/` in the execution evidence.

| File | Was | Now | Reason |
| --- | --- | --- | --- |
| `combat/tests/m25_watcher_generation.rs` | HEALING = six kinds | plus Bandage Up | Design §5.1 |
| `run/tests/m25_watcher_heart.rs` (~493) | a definition-only row exists | none exists; the empty family still exercises the incomplete-family rejection before RNG | Regret and Ascender's Bane now executable |
| `run/tests/m25_watcher_heart.rs:497` | old sampler call | slot sampler call, same assertion | Sampler generalization |
| `tests/run_content.rs` | Ascender's Bane `definition_only` | not definition-only | Design §5.1 |
| `tests/run_content.rs::unpopulated_selectable_pools_fail_closed` | `IncompletePool` | Ironclad 72; Unregistered → `UnknownProducer` | G2 |
| `tests/m21_watcher_admission.rs` | ordinary/Shard reward "EmptyFist only"; `selectable_pool` error | G1 eligibility; selectable 71 | G1, G2 |
| `content/defect.rs` test | `IncompletePool` | Defect 71 | G2 |
| `combat/tests/m15_generation.rs:49`, `combat/tests/m20_defect_generation.rs:18` | Ironclad Any is an error | 70 members | G4 |
| `items/m15_potion_generation_tests.rs:37` | Ironclad potion candidates are an error | 32-member limited pool | G6, F16 |
| `combat/tests/m26_cards.rs` (`m26_cards_end_of_turn_order`, ~236) | "retained Doubt remains in hand"; no draw pile, so discarded curses were redrawn | a retained Doubt discards itself and is in the discard pile; five Defends stay in the draw pile | Correction 8 (final review v1 I2); the original lines are kept as comments |

The rebase added no amendment: the only conflicts were the three trivial ones under
"Source binding", and M25's own test changes (for example `m25_generation_full_hand_overflow`)
are M25's.

## Guard audit (design §6.3)

The audit covered every character or profile comparison in `combat/**` and `items/**`.

- **Frozen M25 (seam map H0 §3, H1b).** Every site was classed L, S, C, Overlay or out of
  scope and gated by its class ("Off-colour gates" above). Where Main's class differs from
  the seam map's proposal, H1b rulings 1–4 apply; the Deus Ex Machina on-draw trigger is an
  L site.
- **Merged M25 (H0 v2, 2026-11-01).** A changed-hunk sweep over the 28 paths that M25 changed
  found no new or removed character or profile gate. The changed sites (Watcher fixed
  children, Foreign Influence, Collect with Chemical X, the `card_state::generated`
  initializer) sit behind M26's existing gates; `piles::insert_generated` is universal with no
  gate. The rest are M25 test fixtures and M25's public Run replay codec, which is outside
  the overlay.
- **Creation routes.** Every in-combat card creation goes through `piles::insert_generated`,
  except the two RouteChecked sites M25 cites. Nothing bypasses it.

Result: no unaccommodated site and no new amendment. Evidence: `main-v1/task0/` (seam map
and its v2 delta) and `main-v1/r6-verify/` (`sweep-16.md`, `sweep-sites.md`,
`sweep-creation.md`).

## Lane provenance

Main (Claude Opus 5.5 senior, Bead `spire-dxo.48.3`, actor `codex-m26-senior-20261031-v1`,
rotations 1–7) owns every shared seam, ruling, review and integration. Pattern-following work
went to Codex assistants (gpt-6-luna) under written contracts with exact writable paths, each
in its own scratch copy under `~/.local/share/spire-gym/build/`. Main verified preimages,
reviewed each delivery and integrated it with pre- and post-integration manifests. Contracts
and reports are retained in the execution evidence (`main-v1/assistant-contracts/` and each
task's directory).

| Assistant | Work | Main's changes on review |
| --- | --- | --- |
| M0 sweeps (`6ccee007`, `e6d89f6e`, `c8df2d03`, `2a608640`), read-only | Gate sites and turn paths; §16 facts and card lists; fixed-seed sampler and M10 Infernal Blade captures; frontier citations | Main's rulings override the sweeps' proposals (seam map H0) |
| M1 identity rows (`820bddc8`) | `CardId`s, catalog rows, `card_state` rows, unplayable lists, `tests/m26_catalog.rs` | — |
| **Lane A** (`2182562e`) | Curse bodies, the 5 colourless cards, Ritual Dagger's Fatal arm, `combat/tests/m26_cards.rs` (each first RED recorded) | Values checked against design §4.2–4.3 |
| M1b tests (`f5beb8f0`) | `combat/tests/m26_offcolour_laws.rs` (9/9 RED at H1) | Fixed 4 test bugs (targets for Eruption and Cold Snap, none for Thunder Strike; Block accumulation; Loop hit measured as absorbed + HP lost) |
| M2 tests (`11321a66`) | `tests/m26_catalog.rs` pool tests, `run/tests/m26_pools.rs`, `combat/tests/m26_pools.rs`, the G1/G2 amendments | — (its RED led to correction 2) |
| M3 tests (`f32f7ff7`) | `run/tests/m26_card_mutations.rs`, `combat/tests/m26_autoplay.rs` | Fixed 2 fixture bugs (Victory auto-settles in `choose_combat`, so in-combat checks use Three Cultists; Dual Wield filler). Normality case (b) rewritten as four Mayhems with a retained Normality |
| **Lane B1** (`79c76f1d`) | `combat/tests/m26_{support,cross_character,generation,copies}.rs` | Added `m26_cross_character_own_inert` (600 own rows). Its one RED group led to correction 5 |
| **Lane B2** (`d77ec257`) | `run/tests/m26_supplied.rs` | Replaced its "final action is not Ritual Dagger" guard with the literal growth expectation |
| Rebase verification (`501dcdc3`) | §16, site and creation-route sweeps on merged M25; the Master Reality tests in `combat/tests/m26_generation.rs` and their mutation check | Ruled queued autoplay replays not creations; integrated the test |
| Freeze suites (`84b95550`), read-only | The four freeze commands on a fresh copy of the exact tree | Re-summed the logs |
| Corrections build and freeze v2 (`fe6be8a2`) | Focused and full suites on the corrected source; the freeze v2 run | Fixed the one RED (the amended `m26_cards` fixture) |
| Retained-outcome sweep (`e9185229`) | Inventory of 364 accepted M13–M25 artifacts; same-action replay of the 40 in scope | — (no divergence) |
| Freeze v2 verification (`cb004664`), read-only | Hashes, logs and the delta against freeze v1 | — |

The plan's lane table lists the M1b, M2 and M3 test files as Main-owned. Main kept ownership
and review of them but had Codex assistants draft them; this is a staffing deviation, not a
scope change.

## Not established

Python delivery; original-runtime or Java parity; normal-start traversal or wins; observable
`CardAcquired`/`CardRemoved` relic effects (no observer before M27); any public surface.
Acceptance waits for the one independent final review of the candidate export.
