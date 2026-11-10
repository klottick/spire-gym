# M34s Search and scenario API: public knowledge, belief branching, tactical starts

Status: M34s-a source candidate (part 1 of 3), not accepted. M34s-b (belief and branching) and M34s-c (tactical starts)
follow, one merge each, on M34s-a's merge.

M34s builds a public-information search and scenario layer on the [Standard Run API](../standard-run-api.md): the
Standard combat view, the exact public knowledge a belief model conditions on, standalone Standard combat sessions, then
(parts b and c) public belief branching and synthetic or reachable tactical starts. It makes no strategy or win claim and
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
  Darkling Nip values consistent with every published intent display and execution.
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

## Coverage ledger (M34s-a)

| Plan item | Tests |
| --- | --- |
| a1 draw rows, order privacy, lifecycle, Intangible, Back Attack, records | `spire-env` `tests::m34s_draw_rows::*`, `tests::m34s_records::*`; `spire-core` `combat::tests::{m34s_draw_rows, m34s_intent_display, m34s_card_ties}::*` |
| a2 public knowledge | `spire-core` `combat::tests::m34s_knowledge::*` (brute-force oracle, negative controls, pre-roll sets) |
| a3 Standard combat view | `spire-env` `tests::m34s_standard_combat_view::*` (supplied Heart, keys, event-seeking and random routes) |
| a4 `standard-combat-v1` | `spire-env` `tests/standard_combat.rs`, `tests::m34s_standard_combat::*`; `spire-core` `run::standard_combat::tests::*` |
| a5 Python | `tests/python/env/test_standard_combat.py` |
| a6 fuzz | `spire-fuzz` Standard combat specs and the draw-knowledge check |

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

## Limits and outs

- M34s-a publishes no belief, branch or position API (parts b and c). Run-level belief, Runic Dome belief, chance
  enumeration and branch serialisation are out of M34s.
- `standard-combat-v1` ends at the engine's combat terminal: settlement effects (Meat on the Bone, Face of Cleric) and
  rewards never apply, although the original applies some of them at victory.
- The original randomises the Seek and Secret Technique/Secret Weapon grid order and sorts Omniscience's grid (by name,
  then rarity, status and type); the engine shows all four in one deterministic (card id, instance) order, which hides the
  physical draw order but is not the original's display order (design S19).
