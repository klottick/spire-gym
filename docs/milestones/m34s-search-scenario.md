# M34s Search and scenario API: public knowledge, belief branching, tactical starts

Status: M34s-a merged (part 1 of 3, PR 99); M34s-b merged (part 2 of 3, PR 100). M34s-c (tactical starts, part 3 of 3) is
a source candidate, not accepted; its merge closes M34s.

M34s builds a public-information search and scenario layer on the [Standard Run API](../standard-run-api.md): the
Standard combat view, the exact public knowledge a belief model conditions on, standalone Standard combat sessions, public
belief branching (part b) and synthetic or reachable tactical starts (part c). It makes no strategy or win claim and
has no original-runtime validation.

## Profile

### M34s-a — public knowledge, combat view and Standard combat sessions

- **Draw multiset and order privacy.** The Standard observation publishes every unknown draw-pile card as a `combat_cards`
  row in zone `draw` (position unknown), sorted by the canonical card key, never physical order. Six lifecycle columns
  publish the per-card facts that distinguish otherwise equal cards (free-play waivers, rolled and modified costs, Blood
  for Blood's discount, Infernal Blade's cost 0); equal published rows are interchangeable for every field classified below.
  Item 5 (PM ruling) per field: published as lifecycle columns — `free_play`, `free_to_play_once`, `randomized_cost`,
  `mutable_combat_cost`, `combat_cost_reduction`, Infernal Blade's cost-0 flag; non-distinguishing (a swap test shows equal
  traces) — `temporary_upgrades`, `copy_provenance`, combat-local ids, copy-family labels (singleton families); equal by
  invariant on settled cards — `purge`, `rebound_on_finish`, `m21`. The draw knowledge fails closed (`Unrepresentable`) if
  tied rows ever differ in an unpublished field or a copy family is shared (the tie guard).
  The
  core `CombatPublic.draw_unknown` carries the same multiset in canonical order (in combats that maintain public
  knowledge; empty elsewhere). Every grid sourced from the draw pile
  hides order: Secret Technique and Secret Weapon now use Seek's and Omniscience's grid order (card id, instance).
- **Intent display.** The displayed intent damage follows the original `calculateDamage`: Back Attack's ×1.5 truncated
  before the floor, then the player's Intangible cap. Gameplay damage is unchanged; one display law
  (`effects::displayed_damage`) serves the view (`displayed_intent_damage`) and the pre-roll consistent sets
  (`displayed_damage_of_base`).
- **Public knowledge (core).** `Combat::draw_knowledge()` is the exact posterior over the draw pile's card-class sequence
  (`spire-gym-public-knowledge-v1`): a normal-form mixture of known segments and top-excluding bags with u128 weights,
  updated on every draw-pile path and fail-closed (`Unrepresentable` until the next shuffle) on any change no operation
  explains, above 65,536 components, or when tied rows differ in an unpublished field. A brute-force oracle that
  reproduces the engine's real handle choices checks it exactly. `Combat::preroll_sets()` holds the Louse Bite and
  Darkling Nip values consistent with every published intent display (as amended by M34s-b D2: no narrowing at
  execution).
- **Standard combat view.** `combat_view()` / `combat_observation_spec()` on Standard: `spire-gym-standard-combat-view-v1`,
  a projection of the Standard observation with `combat_context` (character, ascension, act, room kind, encounter,
  burning) from `Run::combat_identity()`.
- **`standard-combat-v1`.** The Standard arm of the shared `CombatTrainingSession`: strict catalogs of owned resources for
  every character at A0–A20 and every Standard setup row (optional explicit burning-elite buff), entry through the shared
  `enter_owned`, and a settlement-free combat scope in the core Run that ends at the combat's own terminal (victory +1,
  death −1, escaped 0). Python: `NativeStandardCombatSession`, `StandardCombatEnv`.
- **Costs** (release, ignored measurements, not gates). Draw rows add about 1–5 µs to a ~22–25 µs median Standard
  combat encode. `draw_knowledge()` costs a median of 6–21 µs per combat decision (six supplied routes reached the Heart;
  both Watcher routes stopped at the 4,000-step cap, about 3,900 combat decisions each); every route stayed at one
  knowledge component. **Knowledge maintenance and the core draw multiset are gated** (PM ruling on final review M7):
  one entry flag (`Combat::enter_with_public_knowledge`, set by the Run for Standard and `standard-combat-v1`, never
  inferred) turns them on; every other profile (Gym66, M10–M25, M21, all captured profiles) does no knowledge work and
  `CombatPublic.draw_unknown` is empty there, `draw_knowledge()` is `Unrepresentable` and `preroll_sets()` empty.
  Measured core step (release): Gym66-style Ironclad 13.6 µs vs base 13.5 µs, M10 Run 12.0/10.9 µs vs base 11.8/10.7 µs
  (identical step counts); on supplied Standard A20 routes maintenance is 8–19 µs per core step (14–28% of 55–67 µs), before
  encoding ([Standard Run API](../standard-run-api.md#standard-combat-view-and-sessions)).
- **Records.** The Standard fingerprint changes once (`efbc1253…` → `bf8e1d11…`) and the Standard rules id moves to v2;
  pre-M34s Standard records are typed `IncompatibleReplay` (re-record).

### M34s-b — belief and branching

- **Combat-scope transplant.** `Run::combat_scope()` copies one positive list of the Run-side inputs a combat and its
  Standard view read, plus the live combat, into a settlement-free Run. Run-level hidden state is absent by construction
  (an exhaustive `RunState` literal; a new field fails to compile). The transplant steps identically to the original combat
  at every decision to the combat's end: 8 supplied Heart routes (337 transplants), 4 event-seeking routes (including an
  event combat) and 24 standalone starts.
- **Belief `spire-gym-public-belief-v1` (core `combat/belief.rs`).** Canonical public roots, the fail-closed continuation
  allow-list, and research-keyed re-sampling: R1 the draw order from the exact draw knowledge, R2 every stream, and R3 each
  ambiguous pre-roll. No exact oracle (`exact_*`, `true_draw_classes`, `fuzz_combat`) feeds it.
- **Search `spire-gym-public-search-v1` (spire-env `public_search.rs`).** `RunSession::public_root`,
  `CombatTrainingSession::public_root`, and the `PublicSearch` arena (tokens, limits, Full or Mask steps, fork, close,
  `actions`, nested roots, counters, spec) with distinct budget, fault, truncation and terminal outcomes. Python:
  `public_search(...)` on `RunEnv` and `StandardCombatEnv`, `PublicSearch`, `PublicBranch`, and `RunBudgetError`. See the
  [Standard Run API](../standard-run-api.md#public-belief-and-search-m34s-b). M35 adds batched branch stepping; see
  [Batched public search](../standard-run-api.md#batched-public-search-m35).
- **Rulings (b0, PM-accepted).** D1: root equality and the digest use part a's sound normal form, and roots whose equal
  distributions have different structure stay distinct (independent streams; never a privacy issue). D2 (final review M2):
  pre-roll sets narrow only on the intent value published at a boundary. The execution-time narrowing on a recomputed
  display is removed. Part a's test `consistent_set_updates_at_execution_after_an_ally_buff` assumed it and was replaced
  (wrong premise). b0's D3 ("the grid instance rank is public history, so nothing is relabelled") was **wrong** (final
  review I1): the instance rank of combat-local copies of one card class is hidden once they have passed through a
  hidden draw or shuffle, and the order-hiding grids ordered such copies by it. See "Global fixes (M34s-b)" below. Card
  and enemy handles stay unrelabelled; no encoded output depends on them (the core typed `Run::decision()` names cards by
  handle and is outside the belief's information set; [Standard Run API](../standard-run-api.md#public-belief-and-search-m34s-b)). The allow-list over-denies replayed player
  targets.
- **Gameplay delta:** the grid order of combat-local copies of one card id with different classes (I1-c below), and
  which copy a given Stasis roll takes (R1-b below; the distribution is unchanged). Everything else changes public
  knowledge, published views or the search only.
- **Costs** (release medians, not gates): `public_root()` 0.36–0.54 ms, `branch()` 24–113 µs, Mask step 24–241 µs, Full
  step 62–287 µs, `fork` under 3 µs. Retained bytes (final review I2): `spec()["max_retained_bytes"]` bounds the whole
  arena (fixed part, live branches, and per-created-branch entries). The only measured term is the Run heap estimate
  (64 KiB; allocator-measured at most 12,889 bytes at the first Act 3 boss); the draw knowledge is capped at 2 MiB
  (`KNOWLEDGE_BYTES_CAP`; a larger one is `Unrepresentable`). The gated test `arena_heap_stays_within_the_published_bound`
  checks real allocations against it. Part b adds no per-step cost on other profiles: the shared Python observation helper is unchanged from base, and the
  core step is measured unchanged on Gym66 and M10 ([Standard Run API](../standard-run-api.md#public-belief-and-search-m34s-b)).

### M34s-c — tactical starts

- **Start kinds.** `standard-combat-v1` entries are `fresh_start`, `synthetic_position` or `reachable_recipe`, labelled in
  each catalog descriptor entry and in every reset's info (search branches stay `branch`). A synthetic position is never
  presented as reachable gameplay ([Standard Run API](../standard-run-api.md#tactical-starts-m34s-c)).
- **Positions (`spire-gym-combat-position-v1`, core `combat/position.rs`).** Enemy `hp` and powers are installed in
  `enemies::setup_with_overrides` before the opening move selection (the opening intent follows the edited state; no damage
  event is implied). The player, piles, scalars, ledgers and public knowledge are applied by `Combat::apply_position` at the
  native opening's first main decision, on a scratch copy, per the overwrite table, then `validate_runtime`. Every check runs
  before any state change: static checks at catalog parse (`Position::validate`), one trial construction, and a scratch Run at
  reset (a rejected position leaves the session untouched; an engine fault during entry freezes it, as for every start).
  Enemy edits need a fixed formation with seed-independent species (the shapes encounters reject them), and `slot` is the
  formation index at validation, setup and the opening decision alike. Fail-closed allowlists: 35 amount-only player powers;
  enemy start-relic powers, the entry's declared burning buff on every member with one amount, and each species'
  amount-only setup powers; generated statuses and the character's generated cards. An empty hand the hand-emptied seam
  would refill (Unceasing Top) is rejected. Hidden state (`order`, `seed`, `rng`, `intent`, `memory`, `instance`, `handle`, `turn`) is rejected by name.
- **Recipes.** A record prefix (records map by SHA-256; ≤ 8 MiB each, ≤ 64 MiB total) is validated once with
  `RunSession::from_record`; its M34s-b `PublicRoot` is cached and each reset samples a world keyed by the reset's seed,
  partition and ordinal (`run::research_start_key`, research purpose `start`). Resets never read the record's hidden state.
- **Lessons and paired starts.** Two lessons native-consistent at turn 1 (Ironclad exhaust engine vs Gremlin Nob before
  Enrage; Silent poison vs Jaw Worm) and one smoke position each for Defect and Watcher, at A0 and A20, continue with random
  legal actions to the combat's terminal (`tests/standard_positions.rs`). Python: `StandardCombatEnv(catalog, records=...)`,
  `examples/tactical_lessons.py` (lessons, a recipe and paired evaluation starts).
- **Fuzz.** spire-fuzz `standard-position-<character>-a<0|20>` specs build a random valid position per episode from the
  allowlists and step it to a terminal or 40 decisions.

### Global fix and disclosed rulings (M34s-c)

| Id | Change | Profiles affected |
| --- | --- | --- |
| C-bottle (PM ruling c0 (6), gameplay) | A `standard-combat-v1` entry that owns a Bottled relic names its bottled deck card (`bottled_cards`; the pickup rule: a purgeable card of the bottle's type); the card is innate as native. Part a's standalone entries linked no card, so the bottled card was not innate. An owned bottle with no card is named explicitly (`null`: native when no card was eligible at pickup, or the card was removed, e.g. by Vampires); an owned bottle missing from `bottled_cards` is now `Input` naming the field | `standard-combat-v1` only (the Standard Run links bottles at pickup) |
| Catalog descriptor | every Standard combat catalog entry carries `start_kind`; entries with bottles or a position carry their normalised `bottled_cards` / `position`; so every `standard-combat-v1` catalog fingerprint changes | `standard-combat-v1` |
| Amended assertion | part a's `standard_combat_catalog_is_strict` row "position is rejected" now checks that an invalid position is rejected (M34s-c admits `position`) | test only |
| Amended assertion (final review m3) | c's own `position_rejects_history_inconsistent_enemy_state` control lists Metallicize on a Gremlin Nob; it now declares `burning: "metallicize"` (an undeclared buff is rejected) | test only |

An unknown position key is `Input` naming its object level and the admitted keys (`position`, `position.enemies[]`,
`position.player`, `position.piles`, `position.piles card row`; verifier v1 note 1).

Errata and rulings against the planning contract (planning-v2.1, c0 `task0-v1.md`): design §4.8 "channel counts zero" — with
`orbs` listed, the Frost and Lightning channel totals equal the listed orbs of each kind (PM ruling c0 (5); zero would
contradict a native Cracked Core start); design §4.1's `belief_branch` label is the merged `branch`; enemy `block` is set at
the opening decision (the entry requires setup block 0) and listed enemy powers are also re-asserted there; enemy `hp` is
bounded by the species' minimum max HP (the plan's "HP 0 or > max" bound depends on the seed's HP roll) and enemy edits need a
fixed formation (slot species must not depend on the seed; `enemies::seed_chosen_species` names the shapes encounters, the
only fixed formations whose species setup re-chooses, per the census test `enemy_edits_need_seed_independent_species`).

### Global fixes (M34s-b, final review I1 and recheck R1: a leak in a merged surface)

The order-hiding draw-pile grids (Seek, Omniscience, Secret Technique, Secret Weapon) ordered their candidates by
(card id, instance), and selection rows and legal actions follow that order. Once copies of one card class have been
shuffled together, which copy is which is hidden, so the order showed which copy sat at a published draw position, and
the draw knowledge, root digests and belief samples followed the pick of a copy chosen by that hidden order. Part a
(merged, PR 99) shipped this in the Standard surface; the M17–M25 extended encoders showed the same order. The fix is
global and reads public facts only:

| Id | Change | Profiles affected |
| --- | --- | --- |
| I1-a forgetting at the grid | Opening an order-hiding grid forgets every published draw position (`known_draw`, `known_draw_bottom`, cut at the first such entry) of a class with two or more copies among the candidates. View only: no rule reads public knowledge | Standard Run, Standard combat view, `standard-combat-v1`; M17–M20 Defect (Seek) and M21–M25 Watcher (Omniscience) extended encoders |
| I1-b public grid law | The draw knowledge's grid removal reads public facts only: a class with a published copy and another copy is `Unrepresentable` until the next shuffle; Scry discards use their own published-position law | Knowledge (Standard surfaces only) |
| I1-c grid order (PM ruling (c), gameplay) | Within one card id, combat-local candidates take their slots in (card class, instance) order. Permanent cards and one class of local copies keep (card id, instance) exactly. Standard rules id `spire-gym-native-standard-rules-v3`; captured ids unchanged (no compatibility promise; disclosed) | Every surface whose content reaches these grids: Standard, `standard-combat-v1`, M15 (colorless Secret Technique/Weapon), M17–M20, M21–M25; a reachable different-class local pair is not established for every bounded profile. Captured ids unchanged (replay caveat) |
| I1-d branch refusal | A search branch refuses, with no effect, a pick among two or more same-class candidates that are not all published (`UnsupportedBoundary`, `HIDDEN_ORDER_PICK`): sampled worlds keep the true labels of identical copies | Search only |
| I1-e root equality | `PublicRoot` equality is the digest and capacities (public content); the canonical worlds of public-equivalent combats may differ in copy labels | Search only |
| R1-a forgetting before a hidden pick | A draw-pile card taken by a hidden choice (Bronze Orb's Stasis and Violence each take one uniformly at random) shows its class, never which copy. When the draw pile holds two or more copies of the taken class, every published draw position of that class is forgotten first (`known_draw`, `known_draw_bottom`, cut at the first such entry). One shared owner, `piles::take_draw_card`, runs this law for every `Removal::Uniform` pick. View only; the draw knowledge's uniform removal is the exact mixture | Standard Run, Standard combat view, `standard-combat-v1`; M17–M25 extended encoders (they publish `known_draw`) wherever Bronze Automaton or Violence is reachable |
| R1-b Stasis roll mapping (gameplay, PM-approved) | Stasis maps its uniform roll through its candidates ordered by (card id, physical pile position) instead of (card id, instance). Each candidate stays equally likely; only which copy a given roll takes changes. The order no longer depends on the hidden rank of identical copies, and the original game also sorts by card id over its pile order. The Standard rules id v3 covers it | Run profiles: Standard (all characters, A0–A20, including the M34 Run API) and the captured M7, M8, M9, M10, M13, M14, M15, M18, M19, M20, M23, M24 and M25 (every captured profile that reaches the Act 2 boss). Combat profiles: Gym66 combat-v1 and v2, the M21 Watcher combat profile and `standard-combat-v1` (their encounter lists include Bronze Automaton). Not M6, M11, M12, M16, M17, M21 Run or M22 (no Act 2 boss). Ids and fingerprints are unchanged (PM ruling on recheck v2 S1: accepted globally under the user's 2026-11-02 override, so these profiles are not frozen until the roadmap is complete). A retained record from these profiles that contains a Stasis capture may replay to a different copy. In replay paths that do not compare observations, this happens without a typed `IncompatibleReplay` error. The one measured change is the frozen M23 supplied-boss Bronze Automaton replay, step 12 (Stasis takes the other Carve Reality copy) |

The R1 census (all of `spire-core`: every zone with published positions, every specific-card operation and every RNG
call site) found no other hidden copy pick. Hand, discard, exhaust and Stasis custody are fully published, so random
picks there choose public positions. Havoc, Mayhem, Distilled Chaos and draws take the top card. Insertions choose no
copy. Lesson Learned ranks public permanent instances. Thieves take gold only.

Tests: the review's repro (Strike and two Wounds shuffled, Scry shows a Wound, Seek takes a Wound: true posterior 3/4,
the knowledge `Unrepresentable` in both hidden orders) and an upgraded-copy pair, each with a negative control that fails
under part a's rule; env label-swap families (Defect Seek with Wounds, Watcher Secret Technique with Insights, Wounds
swapped across hand and draw, Ironclad Secret Weapon with Shivs, and two Burns of which one is upgraded) with identical
views, roots, digests and branches at the decision and at the grid; their red evidence is mutation (each I1 part reverted
in turn fails them).

### Global fixes and disclosed rulings (M34s-a)

Observation values change in exactly these cases; gameplay in exactly G-a4 and R-a1:

| Id | Change | Profiles affected |
| --- | --- | --- |
| Draw rows and lifecycle columns | Standard observation: new rows and columns (fingerprint `bf8e1d11…`) | Standard |
| Intangible display | intent damage capped at 1 while the player has Intangible | Standard Run, Standard combat view, `standard-combat-v1`; M15SilentHeart Run (Wraith Form, Ghost in a Jar). No other captured, M21 or Gym66 surface admits a source |
| Back Attack display (R-a2) | ×1.5 truncated before the floor in the display | every surface admitting Shield and Spear: Standard Run, Standard combat view, `standard-combat-v1`; M10 Run-v2, policy Run-v3 and combat sidecar; M15, M20, M25 Runs; M21 Watcher combat sessions; **Gym66 `ironclad-combat-v1` and `-v2`** |
| G-a4 (gameplay) | a card played by a non-purge autoplay (Havoc, Mayhem, Distilled Chaos, Omniscience) no longer keeps its free-play waiver after its use (original `UseCardAction`) | where such a card can come back into play: Standard Run, Standard combat view, `standard-combat-v1`; M10 Run-v2, policy Run-v3 and combat sidecar, **Gym66 `ironclad-combat-v1/v2`** (Havoc then Exhume); M25 Run (Omniscience then Tantrum) |
| R-a1 (selection order) | Secret Technique / Secret Weapon candidates in grid order (card id, instance) | Standard Run, Standard combat view, `standard-combat-v1` (no captured, M21 or Gym66 surface admits them) |
| Rules id | `spire-gym-native-standard-rules-v2` (G-a4, R-a1) | Standard, `standard-combat-v1`; captured ids unchanged |
| Gym66 Escaped law | `Escaped` reward 0 at the Gym66 step and the M21 replay validator | none observable (no Gym66 catalog admits Smoke Bomb) |

Errata against the planning contract (planning-v2.1): design §4.1 said the rules id does not change (it does, for G-a4
and R-a1); design §4.5.1 / recheck R-I1's Seek example has Strike on top with probability 3/4, not 1 (the knowledge is
`Unrepresentable` there by the fail-closed row, as required). The normal form is sound (equal normal forms mean equal
distributions) but not complete for structurally different mixtures; M34s-b decides root equality and digests.

## Coverage across M34s (design goals; M34s-c closes the milestone)

| Goal | Part | Where |
| --- | --- | --- |
| G1 Standard combat view | a | ledger a3; branches publish it (b3) |
| G2 draw multiset, Intangible and Back Attack intent display | a | ledger a1 |
| G3 `standard-combat-v1` (4 characters, A0–A20) | a, c | ledger a4; c adds start kinds and the bottled-card link (c1–c3) |
| G4 exact public knowledge (draw posterior, pre-roll sets) | a, b | ledger a2; global fixes I1 and R1 (b) |
| G5 belief model v1 | b | ledger b2 |
| G6 branch arena | b | ledger b3 |
| G7 privacy oracle with negative controls | b, c | ledger b2/b3 (P1–P7); c: hidden fields rejected by name, recipes re-sample (c1, c3) |
| G8 synthetic positions and reachable recipes with labels | c | ledger c1–c3 |
| G9 Python on the built extension | a, b, c | ledgers a5, b4, c4; lane P in each reconciliation |
| G10 examples and measured costs | b, c | ledgers b5, c4; costs in the [Standard Run API](../standard-run-api.md) |

## Coverage ledger (M34s-a)

| Plan item | Tests |
| --- | --- |
| a1 draw rows, order privacy, lifecycle, Intangible, Back Attack, records | `spire-env` `tests::m34s_draw_rows::*`, `tests::m34s_records::*`; `spire-core` `combat::tests::{m34s_draw_rows, m34s_intent_display, m34s_card_ties}::*` |
| a2 public knowledge | `spire-core` `combat::tests::m34s_knowledge::*` (brute-force oracle, negative controls, pre-roll sets) |
| a3 Standard combat view | `spire-env` `tests::m34s_standard_combat_view::*` (supplied Heart, keys, event-seeking and random routes) |
| a4 `standard-combat-v1` | `spire-env` `tests/standard_combat.rs`, `tests::m34s_standard_combat::*`; `spire-core` `run::standard_combat::tests::*` |
| a5 Python | `tests/python/env/test_standard_combat.py` |
| a6 fuzz | `spire-fuzz` Standard combat specs and the draw-knowledge check |

## Coverage ledger (M34s-b)

| Plan item | Tests |
| --- | --- |
| b1 transplant | `spire-env` `tests::m34s_transplant::*`; `spire-core` `run::standard_combat::tests::transplant_holds_no_run_level_state` |
| b2 belief (P2, P4–P7, negative controls) | `spire-core` `combat::tests::m34s_belief::*`, `combat::tests::m34s_knowledge::{sample_follows_the_knowledge_distribution, true_order_sampler_fails_p5, execution_does_not_narrow_on_an_unpublished_display, stale_published_display_fails_closed}`, `combat::knowledge::tracker_census_classifies_every_field`, `items::combat_item_context_census_classifies_every_field`, `run::rng::m34c_reserved_domain::research_label_is_disjoint` |
| b3 arena (P1, P3, P7) | `spire-env` `tests::m34s_search::*` |
| Global fixes I1 (a–e), I2 | `spire-core` `combat::tests::m34s_knowledge::{grid_pick_among_identical_copies_is_public, hidden_grid_rule_leaks_the_instance_order, grid_order_of_different_class_copies_is_public, old_grid_sort_leaks_the_rank_of_different_class_copies, knowledge_over_bound_is_unrepresentable_until_shuffle}`, `combat::tests::m34s_belief::knowledge_over_the_byte_cap_makes_the_root_unsupported`; `spire-env` `tests::m34s_dup_pairs::*`, `tests::m34s_search::a_publication_error_after_the_engine_commits_faults_the_branch` (M3) |
| b4 Python | `tests/python/env/test_public_search.py` |
| b5 examples, costs | `crates/spire-env/examples/public_lookahead.rs`, `examples/public_lookahead.py`, `tests::m34s_search_measure` (ignored) |
| b6 fuzz | `spire-fuzz` search mode (P2 perturbation, token rejection, live-root invariance) |

## Coverage ledger (M34s-c)

| Plan item | Tests |
| --- | --- |
| c0 rebind, overwrite table, allowlists, lessons | evidence `m34s-c/task0-v1.md` (lanes 1–2 under `task0/`) |
| c1 validation | `spire-env` `tests/standard_positions.rs` `position_rejects_invalid_inputs`, `position_rejects_history_inconsistent_enemy_state`, `rejected_reset_has_no_effect`; `tests::m34s_recipes::position_rejected_at_reset_has_no_effect`; `spire-core` `combat::tests::m34s_positions::{enemy_edits_need_seed_independent_species, setup_override_failure_is_a_position_rejection, enemy_burning_powers_follow_the_native_buff, empty_hand_under_unceasing_top_is_rejected}` |
| c2 construction | `spire-core` `combat::tests::m34s_positions::*` (setup before the roll, named fields, generated cards, relic-start powers, Snecko costs, armed relic state, debuff flags, draw order and top, knowledge reset, ledgers, refreshed values, opening boundary, bottled card linked and unlinked, formation slots, continuation); `spire-env` `tests/standard_positions.rs` `position_is_labelled_synthetic`, `position_draw_order_is_gym_owned_and_seeded`, `position_continues_with_normal_actions_to_combat_end` |
| c3 recipes | `spire-env` `tests::m34s_recipes::*`; `spire-core` `run::rng::m34c_reserved_domain::research_label_is_disjoint` |
| c4 Python, lessons, paired starts | `tests/python/env/test_standard_positions.py`, `examples/tactical_lessons.py` |
| c5 fuzz, docs | `spire-fuzz` `standard-position-*` specs and `tests/standard_positions.rs`; this profile and the API doc |

## Reconciliation

Checks on the read-only candidate snapshot against base `d6572940` (M34s-a evidence `c1-checks/`):
- **Workspace:** `cargo test --workspace` exit 0, 4,949 passed, 0 failed, 14 ignored; 67 test names added, none removed.
  Core diagnostics (m28/m29 and m34 features) and the spire-fuzz default and diagnostics lanes all pass.
- **Traversal** (`m28_traversal`, `M28_DIGEST_PROJECTION=pre-m34a` on both trees; A0 control, A1–A20, A20): every run
  fault-free on both trees; M33 gate PASS A1–A20. 88 run configurations differ from base, and each is reproduced by exactly
  one base-plus-one-fix tree: 75 by G-a4, 7 by R-a1 and 6 by the intent display (Intangible / Back Attack).
- **Fuzz gate:** 0 failures on candidate (500 episodes, including the 8 `standard-combat-*` specs) and base. All 12 Standard
  Run profiles differ in trace hash with equal gameplay hashes (draw rows and lifecycle columns only); all 15 captured Run
  profiles are equal in trace and gameplay hash. The diagnostics draw-knowledge check found no true pile outside the support.
- **Frozen replay** 323/323 equal; **examples** equal to base.
- **Lane P** (built extension from the snapshot): 331 Python tests passed; captured fingerprints equal the d6572940 packet;
  Standard `bf8e1d11…` and Standard combat view `5d8147e1…`; the retained n14g/bhl/tsw replays equal the packet.
- **Fixture limitation:** on the A20 keys route (seed 0xC3) the test-only supplied hit (`supplied_wound_enemies`) leaves the
  Run with no legal action; base reproduces it, and the same route with public actions only ends in a natural death.

### Reconciliation (M34s-b)

Checks on the read-only snapshots against base `bdd82639` (M34s-b evidence `c2-checks/` to `c5-checks/`). c2 holds the
final review v1 fixes; c3 and c4 change only the search arena, tests and docs. c5 holds the recheck v1 fixes (R1 in core,
R2 in the arena), so every gate was rerun on c5.
- **Workspace:** `cargo test --workspace` (c5 tree) exit 0, 5,016 passed, 0 failed, 17 ignored. Against base, 66 test names
  are added and one is removed (`consistent_set_updates_at_execution_after_an_ally_buff`, replaced under D2). Core
  diagnostics pass (m28/m29: 4,266; m34: 3,637), as do spire-fuzz default (18) and diagnostics (29).
- **Gameplay and the I1/R1 effects, attributed:** every candidate output equals an attribution tree built as base plus
  only I1-a, I1-c, R1-a, R1-b and the rules id.
  - **Traversal:** 23/23 byte-identical, with the M33 gate passing for A0–A20; base also equals attribution in 23/23.
  - **Fuzz gate:** 500/500 gameplay hashes and 498/498 trace hashes equal between candidate and attribution. Between base
    and attribution, gameplay is 500/500 equal and 140 trace hashes differ: the published view change I1-a.
  - **Frozen replay:** 323/323 equal to attribution. Base differs from attribution on one input, reproduced exactly: the
    supplied-boss Bronze Automaton replay, where Stasis takes the other Carve Reality copy at step 12 (R1-b).
  - **Examples:** equal. The released M22, M23 and M24 replays exit 1 identically on base, attribution and candidate, as
    on c2.
  - **Gameplay:** the one measured gameplay change is the R1-b Stasis capture above. The I1-c order change did not arise on
    these inputs.
- **Search driver** (100 seeds, all characters × {A0, A20}; identical on c2 to c5 apart from time):
  - 4,280 searches, 16,687 branches, 46,557 steps;
  - 4,280 hidden perturbations with equal roots (P2);
  - 0 hidden-order refusals and 0 failures;
  - unsupported roots: 102 hidden intents (Runic Dome), 2 draw knowledge beyond the model.
- **Lane P** (built extension from each snapshot): 337 Python tests passed. Every captured fingerprint equals the `bdd82639`
  packet; Standard `bf8e1d11…` and the Standard combat view `5d8147e1…` are unchanged. The Python and Rust lookahead
  examples exit 0.
- **Mutations:** 39 mutants (`scripts/mutations_b.py`, logs v3 to v5), 37 killed, including all six R1 mutants. M-i1b-true-handle survives in core and
  env. It is equivalent on every opened grid, because forgetting at the opening makes the public law read the same as the
  handle rule. The one difference is a disclosed fail-closed over-denial: when Seek takes the whole draw pile without
  opening a grid, the mutant stays exact, while the candidate is `Unrepresentable` until the next shuffle.
- **Verifiers:**
  - v2 FAIL: the `PublicDecision` scope and the retained-bytes bound.
  - v3 FAIL: arena core errors and `used_samples`.
  - v4 PASS, after the fixes listed under "Global fixes (M34s-b)" and in the API doc.
  - v5 PASS: an independent R1 census agrees that Stasis and Violence are the only hidden draw-pile copy picks.

- M34s-b's belief is combat-scope only (L4) and sampling only (L3); Runic Dome roots are Unsupported (L2); the belief
  conditions on more than the view shows (L6) and not on dealt damage (L5). Run-level belief, Runic Dome belief, chance
  enumeration and branch serialisation are out of M34s.
- `standard-combat-v1` ends at the engine's combat terminal: settlement effects (Meat on the Bone, Face of Cleric) and
  rewards never apply, although the original applies some of them at victory.
- The original randomises the Seek and Secret Technique/Secret Weapon grid order and sorts Omniscience's grid (by name,
  then rarity, status and type); the engine shows all four in one deterministic (card id, instance) order, which hides the
  physical draw order but is not the original's display order (design S19).

### Reconciliation (M34s-c)

Checks on the read-only c2 snapshot against base `f99f7de0` (M34s-c evidence `c2-checks/`; c1 in `c1-checks/`). c2 is c1
plus verifier v1 note 1 (an unknown position key's `Input` names its object level) and the final review v1 fixes
(`fixes-v2.md`: I1 seed-chosen species, I2 formation slots, I3 pre-roll test, m1 unlinked bottles, m2 Unceasing Top, m3
burning buffs, m4 docs). Base outputs are reused from c1 (the base tree is unchanged).
- **Workspace:** `cargo test --workspace` (c2) exit 0, 5,049 passed, 0 failed, 19 ignored. Against base, 35 test names
  are added (2 ignored: `tactical_start_costs` and the 500-episode position fuzz) and none removed; part a's
  `standard_combat_catalog_is_strict` is amended in place. Core diagnostics pass (m28/m29: 4,285; m34: 3,656), as do
  spire-fuzz default (20) and diagnostics (31).
- **Gameplay, attributed** (every difference from base maps to exactly one listed change):
  - **Traversal:** every configuration equal to base (A0 control, A1–A20, A20 diagnostics); M33 gate PASS for A1–A20.
  - **Fuzz gate** (500 episodes, 0 failure groups): all 400 non-Standard gameplay hashes and their trace hashes are equal.
    Of the 100 `standard-combat-*` runs, every trace hash differs through the catalog descriptor (`start_kind`, and
    `bottled_cards`), and exactly two gameplay hashes differ, both entries linking a bottled card (C-bottle: Watcher A0
    Three Louse, Bottled Tornado; Defect A0 Gremlin Nob, Bottled Flame and Bottled Lightning). Entries owning a bottle with
    no eligible card name it `null` and play as base. Against c1, gameplay is identical.
  - **Frozen replay** 323/323 equal; **examples** equal (the released M22–M24 comparisons exit 1 identically on both).
  - **Search driver** (6b): identical counts to M34s-b c5 (4,280 searches, 16,687 branches, 46,557 steps, 0 failures).
- **Positions fuzz:** the eight `standard-position-*` specs, 500 episodes each (4,000), 0 failure groups; rejected
  positions 6.9–9.9% per spec.
- **Lane P** (built extension from the c2 snapshot): 345 Python tests passed. All nine captured fingerprints equal the
  `f99f7de0` packet, including Standard `bf8e1d11…` and the Standard combat view `5d8147e1…`. `tactical_lessons.py`,
  `public_lookahead.py` and `standard_run.py` (4 characters × A0, A20) exit 0. The retained n14g, bhl and tsw replays stop
  exactly where base `f99f7de0` stops (n14g rechecked on the packet wheel: same batch, slot, seed and action).
- **Mutations:** 32 mutants (`scripts/mutations_c2.py`, `logs/mutations-v3.log`), 31 killed, including all seven for the
  final review fixes and M-c2-no-preroll-reset (the Philosopher's Stone row: a start relic shifts an enemy power during the
  native opening and the position re-asserts it, so the published display is coarser than the native one; c1 wrongly
  recorded this mutant as equivalent). One is equivalent: M-c1-extra-deck-rows (the core `replace_piles` re-checks missing
  deck cards at the trial).
- **Verifier v1** PASS-WITH-NOTES (one Minor, fixed red-first). **Final review v1** NOT READY (I1–I3, m1–m4); the fixes are
  red-first (`logs/review-v1-red.log`), except the I3 row, whose red is the killed mutant.
- **Tests not written red-first (disclosed):** the c1 env tests in `tests/standard_positions.rs` and `tests::m34s_recipes`
  were written before the implementation but not run red, and the Python tests were written with it; the mutations above
  show that they can fail.
- **Costs** (release, `logs/measure-tactical-starts-v1.log`): recipe load 4.66–5.05 ms for a 3-step A20 prefix; recipe reset
  76–116 µs; synthetic-position reset 163 µs against 151 µs fresh.
