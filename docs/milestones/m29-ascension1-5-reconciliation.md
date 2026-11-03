# M29 Ascension 1–5 reconciliation

Status: M29a1, M29a2, M29b and M29c are merged, each accepted through its chunk receipt; M29d (the
rest of M29 and its close) is a candidate for final review. This record accompanies the
[M29 profile](m29-ascension1-5.md) and its [coverage ledger](m29-ascension1-5-coverage-ledger.md).
It is not a public profile or a Python-delivery claim.

Evidence paths below are relative to /home/klottick/.local/share/spire-gym/evidence/m29-execution-20261103
unless they begin with docs/, data/ or crates/. A test name is the exact native test unless a
different lane is stated. Each chunk keeps the original bytes of every amended accepted test in its
`amended-original/` directory (with `SHA256SUMS`); M29d's originals equal main `4546d65e`.

## Disclosed changes

| ID | Chunk | Change and source or ruling | Pinning test | Status |
|---|---|---|---|---|
| D1 | a1 | `RoomScenario::StandardA0` is renamed `Standard`. The M28 records keep the old name, which means `Standard` at A0 (profile compatibility note). String renames: `"Standard stored note …"`, the spire-env "no profile" string and the traversal guard. m29a1/chunk-note-v1.md item 1. | m28_standard_* (renamed scenario); m29_standard_creation_all_characters_a0_to_a5 | disclosed decision |
| D2 | a1, d | Creation gate: `STANDARD_MAX_ASCENSION = 5`, A6+ returns `Unsupported("Standard supports A0–A5")`. M29a1 added a default-build A1–A5 rejection `Unsupported("Standard is A0 only until M29f")`; M29d removes it (plan M29f item 1, erratum n2). | m29_standard_default_build_admits_a1_to_a5_rejects_a6 (formerly `m29_standard_default_build_rejects_a1`; renamed at review m3); m29_public_a1_a5_gate; tests/m28_standard_gate.rs | settled |
| D3 | a1 | Room-type quotas use the source's binary32 law `Math.round((float)n * chance)`; from A1 the elite chance is `0.08f * 1.6f`, left to right. A0 maps are unchanged for every reachable `n`. | m29 run-rule quota tests (ledger §1); A0 control traversal vs M28f2 | settled |
| D4 | a1 | A5 act-transition heal through the ordinary heal owner; `run/numeric.rs` holds the JDK and libGDX rounding helpers (moved byte-identical from the shop). | m29 run-rule heal tests (ledger §1) | settled |
| D5 | a1 | The traversal example records ascension, typed interim rejections, `interim_blocked`, per-act elite target and placement, and transition HP in a separate `m29` report object (A0 projection unchanged). | runner traversal receipts | disclosed decision |
| D6 | a2 | One combat-level `ascension`; the per-actor copies in the Cultist, Jaw Worm, Gremlin Nob and Fungi Beast memories are folded in and `SlaverBlueAscension` is merged into `SlaverBlue`. Values unchanged at every threshold. | m29_a2_owner tests; full_enemies (amended literals) | settled |
| D7 | a2 | State fingerprint: the combat `State` text and hash print `ascension` for every combat (A0 included) and the folded memories. Public observations are unchanged. Frozen replay 323/323 equal on public hashes; the state-hash delta is exactly these fields (erratum n3). | m27 frozen replay lane (m29a2 receipts) | disclosed decision |
| D8 | a2 | Error strings: setup precondition failures return `Unsupported("combat setup preconditions failed")`; ascension above 20 has its own string; entry rejects disagreeing members. | m29_a2_owner tests | settled |
| D9 | a2–d | Playable ceilings per species and per encounter (static species set plus spawn closure, erratum n6); above a ceiling setup, the event executor and combat entry return `Unsupported("enemy behaviour above this ascension lands in M32")`. | m29_every_encounter_every_ascension; m29_d_formations_ceilings; m29_playable_ceiling_rejects_above | settled |
| D10 | d | Close: every species supports A0–A20 numerically, so no encounter returns the typed interim at or below its ceiling; the sweep now fails on any such rejection. | m29_all_species_landed; m29_every_encounter_every_ascension | settled |

## Global fixes

| ID | Chunk | Change and source | Pinning test | Amended accepted assertions | Status |
|---|---|---|---|---|---|
| G1 | d | Writhing Mass Malleable starts and resets at 3, not 4, at every ascension (spire-oto.24). Source: `WrithingMass.usePreBattleAction` applies `new MalleablePower(this)`, and the one-argument constructor is `this(owner, 3)`. | m29_writhing_mass_malleable_is_3; m29_writhing_mass_ascension_values; frozen replay (1 input, the Writhing Mass implant) and A0 supplied traversal (2 Runs) differ only after a Writhing Mass fight (m29d/c1-checks/) | 32 literals in `m8_writhing.rs` (3 tests), `m24_watcher_act3.rs::m24_w12_writhing_reactive_malleable_implant` and `m25_watcher_interactions_w.rs::m25_w18_writhing_new_multi_hits`: every Malleable amount drops by 1 and each queued Block by 1 per qualifying hit (for example Tantrum's three hits 4+5+6 = 15 become 3+4+5 = 12). Log: m29d/writhing-amend-v1.md; originals m29d/amended-original/. | settled |
| G2 | d | The Awakened One's pre-battle powers are in the source's queue order: Regenerate, Curiosity, Unawakened, then Strength from A4 (`AwakenedOne.usePreBattleAction`). All four powers have priority 5, so `ApplyPowerAction`'s stable sort keeps queue order. The engine listed Curiosity first. The power list order is visible in observations (PM ruling: fix, not disclose). | m29_awakened_one_ascension_values; m29_awakened_one_live_a4_strength_and_a9_rebirth (live order) | No accepted assertion read the order. Frozen replay: 4 of 323 inputs that reach an Awakened One fight now stop at that decision (the recorded public hash includes the power list); the other 318 are byte-identical (m29d/c1-checks/r2-replay-classification.md, main-replay-raw-compare.md). The frozen inputs are M28 evidence and are not edited. | settled |

## Amended accepted tests (names kept)

| Chunk | Test | Change | Originals |
|---|---|---|---|
| a1 | tests/m28_standard_gate.rs; run/tests/m28_standard.rs | A1 rejection string renamed and cfg-gated to the default build; A6 rejection added (erratum n2). | m29a1/amended-original/ |
| a2 | combat/tests/full_enemies.rs, m6_partners.rs, m6_bosses.rs, m6_summons.rs, m8_reptomancer.rs, m8_time_eater.rs | Memory literals lose `ascension`; per-actor ascension reads move to `state.ascension`; `EnemyRollContext` literals gain `ascension: 0`. No assertion value changed. | m29a2/amended-original/ |
| b | combat/tests/m29_a2_owner.rs (3 tests) | "Typed interim until M29b" witnesses retargeted to the landed Act 1 ladder. | m29b/amended-original/ |
| c | run/tests/m28_review.rs, m28_events_act2.rs (2), m29_a2_event_gate.rs, combat/tests/m29_a2_owner.rs, m29_b_act1_normals.rs | Interim witnesses move to the next unlanded encounter or above the ceiling (A17). | m29c/amended-original/ |
| d | combat/tests/m29_a2_owner.rs (`m29_spawned_and_split_children_read_combat_ascension`, `m29_playable_ceiling_rejects_above`, plus the Collector summons check) | The Collector's Torch Heads at A9 carry the A9 HP band (was the interim); TwoThieves A16 is admitted (no interim witness remains); solo Slime Boss and Gremlin Leader at A16 enter (M29c review m2). | m29d/amended-original/ |
| d | combat/tests/m29_b_act1_normals.rs, m29_c_elites_bosses.rs | TwoThieves joins the admitted list; helpers made `pub(super)` for reuse (no assertion change). | m29d/amended-original/ |
| d | combat/tests/m6_ordinary.rs, m8_shapes.rs | The "invalid setup ascension" witness moves from A1 to A21 (A1 is now valid). | m29d/amended-original/ |
| d | run/tests/m28_event_enemies.rs, m28_events_act2.rs (2) | Masked Bandits witnesses move from A1/A15 to A17, above its encounter ceiling 16. | m29d/amended-original/ |
| d | run/tests/m28_events_a15.rs (Mysterious Sphere, Secret Portal) | The Fight is now Offered above A0 and enters at the Run's ascension; Secret Portal enters its boss at the Run's ascension. | m29d/amended-original/ |
| d | run/tests/m28_standard.rs, tests/m28_standard_gate.rs, run/tests/m29_run_rules.rs (`m29_standard_default_build_rejects_a1`, `m29_standard_creation_all_characters_a0_to_a5`) | Erratum n2 close retarget: A1 is admitted in every build; the default-build test checks the A6 rejection; the creation test is unconditional. The default-build test is renamed `m29_standard_default_build_admits_a1_to_a5_rejects_a6` (review m3; the only renamed accepted test of M29; no compatibility promise, the old name no longer matched its assertion). | m29d/amended-original/ |
| d | combat/tests/m8_writhing.rs, m24_watcher_act3.rs, m25_watcher_interactions_w.rs | G1 (Malleable 3). | m29d/amended-original/ |

## Reference data (`data/enemies/enemies.json`)

The reviewed spec sheets were cross-checked against the reference record. Corrections and
source-resolved disagreements:

| Species | Reference says | Source says | Action |
|---|---|---|---|
| Writhing Mass | Malleable 4 (`starting_state`, lifecycle note, TwinStrike Block 4/5) | 3 (one-argument constructor) | Corrected in `enemies.json` and `data/enemies/m8-static-findings.md` (dated note); regression in `tests/python/test_enemy_reference_dataset.py` (stdlib `unittest`, 27/27). |
| Mystic | Heal cutoff A19 / 21 missing HP (wiki, marked disputed against A17) | A17: heal when missing HP > 20 | Record left as is (the dispute is already recorded); the engine follows the source; the selection change is an M32 behaviour row. |
| Spire Growth | Constrict 12 from A2 or A17 (disputed in the move prose) | A17 (amount and forced selection) | Record left as is (dispute recorded); amount 12 at A17 is table-level (ceiling 16); forced selection is M32. |
| Champ | Move-selection change at A19, no numeric cutoff | Forge roll cutoff 15 → 30 at A19 | Record left as is (omission, not an error); M32 behaviour row. |
| Writhing Mass | Block Attack "Gains 16 Block" without a tier | Block equals damage index 2: 15, 16 from A2 | Record left as is (its ascension note keeps 15/16); the engine follows the source. |
| Taskmaster (M29c) | Wounds change at A2 | A3 (elite threshold) | Resolved by the source in M29c; record left as is. |
| Deca | A19 summary omits the `DEFEND_BUFF` intent | Square of Protection intent `DEFEND_BUFF` at A19 | M32 behaviour row; record left as is. |

## Other disclosures

- **Giant Head A18 countdown.** The A18 first countdown of 4 is numeric memory set at construction,
  not a selection change, so the Giant Head's ceiling stays 20 (inventory §3 lists it numeric).
- **Reptomancer A18.** Up to two Daggers per Summon go into the first free slots (source loop),
  capped at four Daggers.
- **`EnemyMemory::valid_for`.** For the Darkling and the Transient the stored-memory check accepts the
  union over all ascensions; the runtime validator, which knows the combat ascension, checks the
  exact Nip range and Transient base.
- **Painful Stabs.** The source applies the Heart's Painful Stabs without an amount; the engine
  stores amount 1 (unchanged by M29).
- **Event reference manifest.** `data/events/events.json` records the sha256 of `docs/intentional-deviations.md` (a builder
  support file). M29d's one-line edit there changes that hash, so `events.json` was regenerated with
  `python3 -B data/events/build_dataset.py`; the only change is that hash.
- **Rebase.** M29d was rebased from `93d14a89` onto `4546d65e` (fuzz chunk D); the record is
  m29d/rebase-v1/NOTE.md (per-file patch identity of the three files both chunks touch).
