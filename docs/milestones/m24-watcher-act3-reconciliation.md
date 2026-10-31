# M24 source and API reconciliation

Status: **candidate record; independent final review pending**. This record
accompanies the approved [design](../superpowers/specs/2026-10-30-m24-watcher-act3-design.md)
and [plan](../superpowers/plans/2026-10-30-m24-watcher-act3.md) (version 2, design SHA-256
`ca0474de…c516`, plan `c98de65e…a545`). Those documents are unchanged. This record maps their
proposed names to the actual owners, records every deviation and global correction, and keeps
the evidence grades separate.

## Source binding

Base: `main` at `38496f922d1b272f5454034cb3bb11110e6843fd` (M23 merged in PR62). Before any
production edit, all 953 tracked files were hashed into a preimage manifest, together with the
two approved documents. Focused baseline suites at that revision all passed: core `m21_` 66,
`m22_` 26, `m23_` 34, `m8_` 115, `m19_` 18, M23 diagnostics 3; env `m23_watcher_act2` 15,
`run_m19_session` 6; core `m23_watcher_act2` integration 6.

## Proposed names and actual owners

| Proposed interface | Actual owner |
| --- | --- |
| `RoomScenario::M24WatcherAct3`, `is_m24` | `crates/spire-core/src/run/rooms/mod.rs`; M24 joins `uses_full_act3` and `uses_watcher_content` only, not `uses_act4`. Watcher A0 short Neow only (`"M24 requires Watcher A0 short Neow"`); a not-fully-unlocked or Act4-disabled profile keeps the existing `Run::new` error. |
| Finite unions | `run/rooms/m24_content.rs`: `M24_WATCHER_ACT3_CARDS` (M22's 28 + Parasite) and `M24_WATCHER_ACT3_RELICS` (M22's 18 + Red Mask) are built by a `const` extension of the M22 slices, so no list is copied. Also potions (= M22's 7), `GENERATED_ONLY`, `STATUSES` (+ Void), `RELIC_TAXONOMY_ADDITIONS` and the Act3 capture scope. Re-exported as `spire_core::run::m24_content`. |
| The two M23-only branches | `run/rooms/shop.rs`: the Act2 counter is read for M23 or M24, and the Act3 counter for M24. `run/rooms/m23_content.rs::relic_capture_scope`: Act2 → `run.m23.relic.capture` for M23 or M24; Act3 → `run.m24.relic.capture` for M24. |
| `RunRngScope::M24RelicCapture` | `run/rng.rs`, appended last; spelling `run.m24.relic.capture`. |
| Parasite in the Run deck | `run/mutations.rs::supported_in` admits it for M24 only; `obtain_card` rejects an upgraded Parasite for M20 or M24. |
| Watcher Curse construction law | `combat/cards/watcher.rs::initialize_profile` returns unchanged for a catalog `CardType::Curse` card in every Watcher combat profile (M21–M24). Status and off-color cards still reject. |
| Red Mask in Watcher combats | `items/mod.rs`: new `watcher_act3` field and `CombatItemContext::with_m24_profile()`. Validation uses the M24 relic union when the flag is set; `Debug` prints the flag only when it is true; `with_potions` copies it (review N3). `run/combat.rs` selects it for `is_m24`. |
| Fault labels | `run/rooms/act3.rs` (`run.m24.room`, `run.m24.encounter`), `run/rooms/act3_end.rs` (`run.m24.act3.end`). |
| Diagnostics | `run/diagnostics/m24.rs` and feature `m24-diagnostics = ["m23-diagnostics"]`. The private helpers in `run/diagnostics/m23.rs` take the scenario, Act3 history and optional resources as parameters; M23 call sites pass the old values and M23's diagnostics tests are unchanged. |
| Legacy env schema | `crates/spire-env/src/run_schema.rs::profile` rejects M24 like M22/M23. Main added this mechanical arm before the native lane started so the workspace compiles. |
| Caller | `examples/m13_support/mod.rs`: `CallerProfile::M24`, `M24CallerOutcome`, `Caller::new_m24`, `rollout_m24` in the same `rollout_shared` loop. `examples/m24_support/{mod,policy,inputs}.rs`. The M23 receipt writer became a generic `receipt(...)`; M23's `persist` is a wrapper and its receipt bytes are unchanged. M24 receipts add `invocation` (argv, executable path and SHA-256). |
| Native consumer | `crates/spire-env/examples/m24_same_choices.rs` (mirrors `m23_same_choices`). |

## Deviations and rulings (all recorded before the freeze)

- **Review N1.** The supplied traversal's Act2 bosses are the seed-7 canonical pairings: Champ before AwakenedOne, BronzeAutomaton before TimeEater, and Collector before DonuAndDeca (`M24DiagnosticBoss::traversal_act2`).
- **Review N2.** The Act2 elite Victory in `m24_act2_merchant_and_elite_match_m23` comes from the M23 test pattern (`diagnostic_room` plus `settle_lament`), applied identically to both profiles.
- **Review N3.** `with_potions` copies `watcher_act3`. This is checked in `items/tests.rs` (`m24_watcher_act3_context_admits_red_mask_and_survives_rebuild`) and in `m24_red_mask_admitted_through_real_tomb`, which uses a potion during combat and then enters a later combat.
- **W7 split.** The plan's single W7 test is now `m24_w07_artifact_vs_ancient_potion` (combat lane, before H1) and `m24_w07b_red_mask_consumes_artifact`. W7b is Main-authored, after the combat lane was archived (PM ownership change).
- **`EndingTerminalFault` → `EndingPreflightReject`.** The `Act3NoKeys` terminal commit cannot fail once its reward preflight has passed. The stage therefore pins rejection before acceptance: Proceed with an exhausted choice ledger changes nothing.
- **`m24_supplied_boss_at(boss, keys, seed)`.** Added because design §11 runs the supplied bosses at seeds 7 and 11. The contract function `m24_supplied_boss(boss, keys)` is unchanged and equals seed 7. Reported to the PM.
- **No `m24_support/fixtures.rs`.** The design's interaction fixtures use the frozen caller's combat rules unchanged, so there are no per-fixture scripts. The only fixture-specific behaviour is the shared "fixture combat settled" stop, and a settled Victory counts as success, as in M23.
- **Prefix equality.** Before the `(2,3,34)` continuation, matched M23 and M24 Runs differ only in the scenario tag and, inside an active combat, the M24 item-context flag `watcher_act3`. The flag follows from the design §5 Red Mask decision. Test-only markers (`CombatItemContext::mark_watcher_act3_for_test`, `Combat::mark_watcher_act3_for_test`) normalize exactly that field.
- **Played prefix proofs.**
  - The lib test applies one fixed test rule to the M23 Run and the same action to M24. Full state stays equal for 89 steps, until Death on Act1 floor 6. That rule is a test driver, not a policy.
  - The integration test drives the frozen M23 caller on canonical seed 7 SlimeBoss/Collector. Public decisions, maps and transitions match for 425 steps, until Death on Act2 floor 23, with 6 real Act2 rooms. This equals the retained M23 receipt.

## Open or unexercisable W-row parts (final review M1)

These parts of design §9 are not asserted. Each is either unsettled by any cited source fact
(see `main-v1/task0/w-row-main-binding-v1.md`) or cannot be exercised with the finite M24 card
union:

- **W1.** No claim about FlurryOfBlows beyond "the Time Warp counter does not change when a stance change returns it from discard".
- **W2.** Rushdown's queued draw versus the Time Warp barrier is not ordered by a test. MentalFortress is asserted only as Block present for the enemy turn.
- **W2b.** Meditate at Time Warp 11 is covered by the accepted `m21_review_i2_*` tests, whose fixture is private to `m21_watcher_scry`. No finite Watcher card opens a stance choice (only StancePotion does, and a potion is not a card use), so that variant has no card case.
- **W3.** The relative order of LikeWater block and Constricted damage is not asserted. Ethereal/Establishment interleaving beyond the M21 ruling is not asserted.
- **W5.** The overkill amount is not asserted; only HP 0 and no carry-over are.
- **W6.** Where Void's draw falls inside queued stance transitions is not asserted; only the settled final energy is.
- **W7.** Which of a two-debuff move's debuffs Ancient Potion blocks is not asserted; only "exactly one applies and Artifact goes 1 → 0" is.
- **W8.** The combined Meditate + Dazed + Establishment case beyond the two M21 tests is not asserted.
- **W9, W11, W12 (unexercisable).** No card in the finite M21/M24 Watcher union is multi-hit. "Slow does not rise between a card's own hits", mid-card cancellation of a multi-hit card by Spiker retaliation, and per-hit Malleable growth within one card therefore cannot be exercised. The tests use consecutive single-hit cards, and W11's lethal case is a single-hit Spiker Death.
- **W10.** "Zero damage stays 0" is not exercised, because no finite Watcher card deals zero. The provenance of Burn's damage type in the original game remains open.
- **W13.** The Shifting amount in overkill/clamp cases is not asserted. Reptomancer target order is asserted as live target IDs 1–4 in actor order.

## Post-review test additions (final review M3, M4; no production change)

- **M3.** `combat::tests::m24_watcher_act3::m24_curse_parasite_watcher_view_scry_and_meditate_custody` unconditionally pins the Parasite's public view in hand: no price, no card values, and a Watcher view with combat and turn cost `None`. It also pins that the Parasite is never a legal play, that Scry selects and discards it without drawing, and that Meditate returns it to hand with Retain across Meditate's own end of turn. The run-level test no longer has a conditional view check. Design §5 said "no Watcher values"; the precise behaviour is an empty Watcher price block, which these tests pin.
- **M4 (corrected after review v2, N1).** `m24_act3_elite_claim_never_rerolls_on_inspect_exhausted_ledger_or_decline` (new) and the extended `m24_act3_claims_never_reroll` cover inspection, a claim after an exhausted choice ledger, and a decline (Proceed without claiming), on both the Act3 elite relic and the chest relic. Relic claims allocate no choice id (`crates/spire-core/src/run/rooms/rewards.rs`), so the exhausted-ledger claim is not a fault: it succeeds exactly once, without a fault, and a second claim is rejected. The captured row and the RNG never change. A real publication fault after acceptance is covered by the native env test `m24_delivery_fault_record` (`crates/spire-env/tests/m24_watcher_act3.rs`).

## Global correction M24-D1: Time Warp barrier (all characters)

- **Found.** The combat lane's `m24_w03_early_end_turn_effects_once` failed. A twelfth-card Worship reaches 10 Mantra and Divinity. Divinity's +3 energy was queued through `enemies::act2::bottom` behind Time Eater's `TimeWarpEndTurn` barrier, and `time_eater::force_end` faulted with "Time Warp reached before owned card work completed". Main reproduced this with its own binary.
- **Source law.** The static findings (Task 8) say the early end keeps ordinary actions: the end turn starts only after every queued ordinary action has resolved.
- **Breadth.** A read-only sweep found many owners that insert work just before the decision. The most general is Void's draw callback (`combat/piles.rs`), which is reachable after a twelfth card through Watcher Rushdown, Silent Reflex or Ironclad Dark Embrace draws.
- **Fix part 1.** `combat/enemies/act2.rs::bottom` treats `TimeWarpEndTurn` as a sentinel.
- **Fix part 2.** `combat/enemies/act3/time_eater.rs::force_end` re-queues the single barrier just before the trailing decision when only ordinary work is ahead of it. It still faults when a card is in play, a selection is open, or a queued card dispatch or scheduler sentinel is pending.
- **Tests.**
  - Watcher W3 (combat lane): RED, then GREEN.
  - `m24_time_warp_barrier_ironclad_dark_embrace_draws_void_after_twelfth_card`: RED with part 1 alone, GREEN after part 2.
  - Two Defect regressions (a twelfth-card Zap into full Lightning slots, and a twelfth-card Power with Storm): pass both before and after the fix. They are regressions, not REDs.
- **Accepted assertions.** No accepted assertion pinned the old behaviour.
- **Retained outcomes.** No retained outcome receipt under the evidence root contains the old fault, so no fixture was regenerated. Original file bytes are retained with the evidence.

## Lane provenance

- **Combat lane.** `delivery-v1` (manifest `85644b55…`) was integrated unchanged: new `combat/tests/m24_watcher_act3.rs` (`f447724f…`) and one `mod` line (`84cb987a…`). Pre/post hashes are recorded.
- **Native lane.** `delivery-v1` (manifest `6e95311c0bf0f3081687f458e6c1be74c15fc0219f39b52ce16a99e39a6dec4c`) was integrated unchanged against `core-snapshot-v3`. All 11 owned files matched their snapshot-v3 bytes before the copy and their delivered SHA-256 after it (`main-v1/integration/native-v1/{pre,post}.json`). Main's pre-integrated legacy-schema arm was kept. After integration, Main added the seed to `M24DiagnosticStart::Boss` (seed-aware session, record and replay) and the supplied consumer test module. The combat lane's record is in `main-v1/integration/combat-v1/`.

## TDD record (excerpt; full logs retained)

- **RED.**
  - M24 Act2 merchant entry (I6 drift); `supported_in` Parasite.
  - Next combat after an Implant (I1); Parasite obtainable upgraded.
  - Tomb → combat entry (I2); fault labels.
  - W3 Divinity; Ironclad Void after a twelfth card.
- **Passed first time (regressions).** Act3 shop counter, Act3 capture scope and keys, empty/singleton/saturated pools, no-reroll, exhausted pools, Act3 rewards, Falling/Tomb, Parasite plus last card, capture owner, diagnostics exactness, W7b, the caller tests, and the two Defect barrier tests.
- **Mutation check.** Removing M24 from `uses_full_act3` makes `m24_start_and_prefix_match_m23` fail; the file was restored byte for byte.

## Freeze identity note

The six canonical zero-action initial snapshots in the freeze are byte-identical to each other.
They capture only the public Neow view; seed, creation history and RNG state are not part of
that view (the same holds for M23's accepted freeze). Each case's start identity is bound by
the frozen commands (seed and scenario) together with the frozen executables' hashes.

## Freeze and once-only sequence

- **Freeze.** `freeze.json` (SHA-256 `e072c4e84e51c975df99fa1719c69611c9839a059658debb67d2d050fef65802`) binds the 971-file source manifest (`e277103c…`), three executables, the caller and policy file hashes, the cases, controls, supplied series, loadout v2 and 24 zero-action initial snapshots. The PM verified it before giving "proceed".
- **Freeze gate.** On the exact frozen tree: workspace 3210 passed / 0 failed / 10 ignored; core lib with `m24-diagnostics` 2101/0/3; core tests with `m24-diagnostics` 2601/0/5; example builds exit 0.
- **Pre-freeze consumer validation (non-outcome input).**
  - The canonical consumer's `released-dry` mode ran on three budget-1 controls under the all-zero sentinel freeze; every lane matched.
  - `m24_supplied_consumer_validates_on_labelled_non_outcome_input` passed.
- **Execution.** Every once-only command ran from read-only copies of the frozen executables (`freeze-v1-bin/`, hashes verified). `invocations.jsonl` records 14 invocations with argv, executable path and SHA-256, times and exit codes. The worktree was verified equal to the frozen source after the sequence. The 62 outcome files are hashed in `outcomes-v1-manifest.sha256` (`f5a444de…`).

## Outcomes and evidence grades

- **Normal played (policy outcomes, not gates).**
  - The six canonical cases all ended in Death: seed 7 at Act1 floor 11 (146 actions), Act1 floor 16 (232) and Act2 floor 23 (425); seed 11 at Act1 floor 7 (114), Act1 floor 7 (133) and Act1 floor 16 (324).
  - Each is identical to its retained M23 receipt at every step (actions, public decisions, maps, transitions and endpoint) once the scenario tag is normalized (`canonical-vs-m23.json`). This is the predicted exact M23 prefix.
  - No normal run reached Act3, an Act3 boss or an ending.
  - Controls: the nonattacking control died after 9 actions on floor 1; budgets 0 and 1 ended unfinished; four input errors exited 1 before any Run.
- **Supplied played (unearned resources, never normal traversal).**
  - **Boss series.** The seed 7 Time Eater (no keys) won, then took Proceed, the three dialogs and stage 4 to `Act3NoKeys` (56 policy actions): **a played supplied witness of a boss Victory into the real Act3 ending**. The other five boss cases died, including all three seed-11 all-key cases. So **`Act4Ready` has constructed coverage only** (`m24_act3_ending_key_masks`, the `BossSource` stages and the native cutoff tests), with no played witness.
  - **Traversal.** All three cases played the real Act2 boss treasure, the `(2,3,34)` transition and Act3 rooms, then died to SpireGrowth on Act3 floor 39 after 88 actions. They are identical: same seed, loadout and Act3 map before any boss-dependent room.
  - **Interaction fixtures.** All nine died in combat. Their Watcher × Act3 interactions remain pinned by the combat composition tests (W1–W14, W2b, W7b).
- **Constructed.** Ending key masks, ending faults, preflight rejection, the Act3 continuation stage, Falling/Tomb/chest stages and the Scry/Meditate/stance choice stages are owner-level evidence only.
- **Native.**
  - The frozen `m24_same_choices` consumed the six canonical receipts (release `6d0d237d…`). Scalar session, strict typed/JSON round trip, portable replay, and homogeneous batches with workers 1 and 2 all matched. Coexistence ran 32 actions in each of the M24, M23, M15 and M20 lanes, and each lane crossed an actual combat (`native-result-v1.json`).
  - The 18 supplied and fixture receipts (release `cc517d2c…`) were replayed by the env test `m24_supplied_recorded_native_agreement`. State matched at every step, and the internal diagnostic replay agreed. That test binary was **built from the frozen source**, not taken from the frozen executables; the invocation log says so.

## Not established

Python delivery, original-runtime parity, Java RNG/map parity, a normal-start Act3 entry, boss
Victory or ending, and a played `Act4Ready` are not established. All original losses are
retained.
