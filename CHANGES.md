# Changelog

Changes from the versioned contract freeze onward. The earlier `## CHANGES`
entries are retained in [observation vocabulary history](docs/observation-vocabulary.md).

## Unreleased

- Add M37a profiling diagnostics: the ignored `m37_phase_timing` test in
  `spire-env`, `examples/run_vector_phase_timing.py`, [profiling](docs/profiling.md) and
  a README Speed section. These are diagnostics and documentation only. No gameplay,
  identity, fingerprint, observation or API change.
- M37b internal performance work, with no contract change. Gameplay, RNG use, IDs, rules
  IDs, fingerprints, observations, masks, records and replay bytes are unchanged; the
  freeze tests, frozen replay, traversal and fuzz hashes match v1.0.0. See
  [performance](docs/performance.md) and [profiling](docs/profiling.md) for the numbers.
  - Card preparation clones the combat state once per legal-action enumeration, not once
    per (card, target) payload.
  - `RunSession` returns each observation by move and keeps no copy. `decision()`
    re-encodes it from the unchanged Run, so the bytes are identical.
  - The session step no longer re-enumerates legal actions to check an action it has
    just published.
  - Record headers reuse a per-session observation-spec fingerprint.
  - The Python batch binding stacks observations per slot on the batch's own workers,
    with the GIL released. Each call still returns new arrays.
  - Two internal, `#[doc(hidden)]` additive Rust methods support this: `Run::step_offered`
    and `SessionBatch::map_jobs`. They are not part of the documented API and add no
    Python names.
- Add M38a scaling diagnostics: the ignored `m38_component_scaling` test, `M37_CAPACITIES`,
  `M37_STACK=parallel|parallel-reuse` and `M37_EARLY_DROP` in `m37_phase_timing`, the
  `--loops` and `--drop-before-step` options and a release timer in
  `examples/run_vector_phase_timing.py`, and a
  [parallel scaling](docs/profiling.md#m38a-parallel-scaling) report. These are test and
  example diagnostics only. No engine, binding, gameplay, identity, fingerprint, observation
  or API change.
- M38b internal performance work, with no contract change. Gameplay, RNG use, IDs, rules
  IDs, fingerprints, observations, masks, records, replay bytes and the bytes, dtypes and
  shapes of every returned array are unchanged; the freeze tests, frozen replay, traversal and
  fuzz hashes match M38a, and a packaging differential compares the Python batch outputs with
  the previous binding call by call. See [performance](docs/performance.md) and
  [profiling](docs/profiling.md#after-m38b) for the numbers.
  - A Standard `NativeRunBatch.step` encodes each slot's observation straight into its rows
    of the returned stacked arrays on the batch's workers, instead of encoding per-slot rows
    and copying them. Fault rows, isolated slots and other profiles are filled as before.
  - The batch keeps a bounded pool of observation array buffers. When NumPy releases a
    returned `obs` array, its memory goes back to the pool and the next step reuses it, so a
    batch may hold up to two calls' observation memory between calls. Each call still
    returns new arrays that share no memory with any array still alive; `close()` empties the
    pool. Arrays released later, after `close()` or after the batch object is freed, still
    return to it within the same bound, and the pool is freed only when the last array from
    that batch has been released. The owner object behind those arrays (`a.base.base`) is now an internal
    `PooledColumn` instead of rust-numpy's `PySliceContainer`.
  - Run and combat-training batches give each worker a home range of slots, so a slot normally
    stays on the same worker thread between calls; idle workers then take unclaimed slots from
    other ranges ([batching](docs/batching.md#slots-seeds-and-workers)).
  - The Standard encoder looks vocabulary identities up in tables built once, builds its
    group layout once per capacities and worker thread, and pre-sizes card rows; Run RNG
    draws feed their HMAC input without building a message buffer (same bytes).
  - Two internal, `#[doc(hidden)]` additive Rust items support this:
    `spire_env::ObservationTarget` and `SessionBatch::step_into`. They are not part of the
    documented API and add no Python names.
  - The `a20_checks` example gains `--speed-check`, and `examples/speed_check.py` is new: short
    fixed diagnostic runs with JSON output for build records. They have no thresholds.
  - [Batching](docs/batching.md#slots-seeds-and-workers) now advises native callers to copy
    what they need and drop the previous rows before the next step.

- M38e: an opt-in compact Standard Run observation. Additive; the dense
  `spire-gym-standard-run-observation-v1` (`bf8e1d11…`), every default, frozen
  identity, replay digest and Heart pin are unchanged. See
  [Standard Run API](docs/standard-run-api.md#compact-observation-m38e).
  - New schema `spire-gym-standard-run-observation-compact-v1` with its own
    fingerprint domain, capacity preset `standard-compact-v1` and fingerprint
    `633b9424be5070a2890b01725949447badba919c5c8c5bd798922f8c2d2a2b2d` at that
    preset; added to the freeze identity table. Same groups, columns and
    semantics as dense; fewer rows and `int16`/`int32` values (136,842 bytes per
    slot of arrays instead of 763,720). Its descriptor's `semantics.capacity_preset`
    is `standard-compact-v1` and it adds two `semantics` keys, `layout` and
    `dense_schema`; a compact session's `environment_spec()` reports
    `capacity_preset` `standard-compact-v1` (or `custom`).
  - Python: keyword `observation="dense"|"compact"` (default `"dense"`) on
    `NativeRunSession`, `NativeRunBatch`, `RunEnv` and `RunVectorEnv`; methods
    `fault_cause()` (sessions, `RunEnv`) and `fault_cause(slot)` (batches,
    `RunVectorEnv`). The `run.py` `Observation` alias gains `int32` and `int16`
    arrays. No new module-level names.
  - Rust (`spire-env`): `RunObservationLayout`, `RunSession::new_standard_with_layout`,
    `RunSession::observation_layout`, `RunSession::fault_cause`,
    `RunCapacities::standard_compact`, `standard_compact_run_observation_spec`,
    `RUN_OBSERVATION_STANDARD_COMPACT_SCHEMA_ID`, `STANDARD_COMPACT_CAPACITY_PRESET_ID`,
    `record_observation_layout` and `encode_run_standard_decision_with_layout`;
    `RunRecordError::from_error` is now public. Internal, `#[doc(hidden)]`:
    `ObservationChunk` and `ObservationTarget::typed`.
  - A compact state that does not fit is a typed `Capacity` fault through the
    existing paths: a row overflow names the row group; a value outside its dtype
    names `"<group>.<column>"` with the dtype maximum as `capacity`. The record
    codec accepts such value-fault groups only in compact records.
  - `fault_cause()` exists for every layout and profile; it is `None` unless the
    session froze. Python redacts a core cause as other Python errors do.
  - Dense speed: native dense is unchanged within noise; the dense Python batch
    step is about 3–5% slower than after M38b (cause diffuse, likely the
    encoder's code generation; see [profiling](docs/profiling.md#after-m38e)).
  - Diagnostics: `M37_LAYOUT` and `M37_STACK=slab` in `m37_phase_timing`,
    `--observation` in `a20_checks`, `examples/run_vector_phase_timing.py` and
    `examples/speed_check.py`.
- spire-3xq combat fixes in the shared engine, found by the M38e capacity stress. Rules IDs
  move to `spire-gym-native-standard-rules-v4`, `spire-gym-native-m10-rules-v2` (also Gym66)
  and `spire-gym-native-m9-rules-v3`; the engine is shared, so every frozen profile is
  treated as affected. Records with the old IDs fail closed. Each fix changes only a path
  that used to fault, so the frozen replay digests, the combat-v1 digest and the Heart pins
  are unchanged. No observation, action or descriptor layout changes.
  - Berserk stacked to 5 or more grants its whole amount at the start of each turn, as the
    original does. The energy check used to reject gains above 4.
  - A queued Fire Breathing hit keeps the amount it was queued with, as the original's
    damage snapshot does. It used to fault when another Fire Breathing stacked first (for
    example through Mayhem).
  - A won combat opens no player choice. Reptomancer's cleanup queues its victory behind
    work already queued, so a Gambling Chip discard or a killing card's follow-up choice
    (Dagger Throw's discard) used to open with no living enemy and fault. One deviation
    remains: if a Wish choice came up in that window, the original could still pay Wish's
    gold when its screen opened before the battle ended (frame timing); the engine offers no
    choice there and is not known to reach that window.
  - A choice opened between a finished card and its queued repeat is accepted: Melange's
    Scry (after a reshuffle) resolves first, then Necronomicon's repeat plays, as in the
    original. It used to fault.
  - Reaching the 999 card identities a combat can issue is the typed `Unsupported` fault
    "generated card count/runtime boundary" on every generation path. Card creation through
    the CreateCard and CopyCard paths (for example enemy status cards or Mark of Pain's Wound)
    used to report it as `Invariant`. See [delivery](docs/delivery.md#7-known-limits-by-surface).
  - A step that stops at an inconsistent choice is a retained `Invariant` fault. It used
    to surface later as `InvalidAction` for an action the mask had offered.
- Correct the `standard-default-v1` capacity note: `offers` is bounded by purchases, not by
  the engine. With The Courier a shop keeps bought rows and appends a restock per purchase,
  so more than 512 rows needs at least 498 purchases in one visit. It fails closed with a
  typed `Capacity` fault ([Standard Run API](docs/standard-run-api.md)). No identity change.
- spire-0kr fixes in the shared shop and reward owners. The Standard rules ID moves to
  `spire-gym-native-standard-rules-v5`; records with v4 fail closed. M9, M10 (and Gym66) and
  the other legacy profiles are exempt: these paths need Complete item admission, which only
  the Standard Run and default-off diagnostics use. The m27 diagnostic Act 4 shop
  (`complete_act4_run`) also restocks in place now; it has no rules ID. Combat-v1 has no shop.
  No observation, action or descriptor layout changes; fingerprints are unchanged.
  - With The Courier, a bought card, relic or potion is restocked in its own row, as in the
    original's 13 fixed item slots. The restock takes a new choice ID at the bought row's
    index. The shop no longer keeps the bought row as `claimed` and appends the restock.
    Restock identities, prices and RNG draws are unchanged. After a Courier purchase, only
    the row order differs: the restock sits at the bought row's index, not at the end, and so
    do the later shop action candidates. The row count also differs. A shop now has at most
    15 rows, and `offers` needs at most 20 (the Library's card board). This replaces the
    Courier capacity note above. Without The Courier, a bought row stays `claimed` as before.
  - Circlet, the relic offered when the relic pools are exhausted, is priced from the
    original's Special-tier base of 400 in a shop, before the usual jitter and discounts.
    Shop entry and a Courier relic restock used to fault (`InvalidAction`) there. Only paths
    that used to fault change.
  - A Circlet reward row while Circlet is already owned stacks Circlet's counter, as the
    original does. The reward chooser (chests, combat and event rewards, Dig, Calling Bell)
    used to reject it with `InvalidAction`, although the mask offered it. Only that
    previously faulting choice changes.

## 1.0.0 — effective at the merge of PR #103

See the [1.0.0 release notes](docs/releases/v1.0.0.md).

- Freeze the documented Standard, card metadata, `build_info()`, batching,
  Python facade and selected legacy consumer contracts. Add native identity and
  semantic replay pins, Heart pins and Python public-name checks.
- Bound `PublicSearch` worker counts to `1..=256`; reject invalid values with
  `search workers must be 1..=256`. Use weak per-count pool handles and a single
  strong hot pool; a live search holds its selected pool until it drops or `with_workers`
  changes its count, reversing
  M35 P13's per-count residency.
- Fix the M34b diagnostics-only (`m34-diagnostics`) supplied-wound hook so its
  scripted hit cannot kill a target. This is test-only and does not change gameplay or a rules ID.
- Add the supplied from-start Heart acceptance for all four characters at A0
  and A20, plus the ignored A0–A20 supplied sweep.
- Add `a20_checks`, `run_vector_rollout.py`, `standard_combat_vector.py`, and a
  `standard_rollout --policy random|first_legal` option; add example and vector
  fault coverage. Isolate the readiness test's mocked CPU probe so it passes
  with the GPU hidden or visible. Repair the `test_build_metadata.py` probe
  crate, broken since M35 made `build.rs` digest the workspace through
  `src/identity.rs` (test-only).
- Retire Rust examples `m22_same_choices.rs`, `m23_same_choices.rs`, and
  `m24_same_choices.rs`; they depended on stale private receipts. The M21
  example remains historical support.
- Set the project and Python package version to `1.0.0`.
- Publish M36 contract, delivery, performance, profile and release documentation.
