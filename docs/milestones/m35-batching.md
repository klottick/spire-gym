# M35 Batching — source candidate

This record covers shared native session batching, Python/Gymnasium vector facades, delivery
identity and batched public-search stepping. It is a finite source candidate. Lane P on the candidate
export passed (Checks); the post-merge build is pending. See the [batching
contract](../batching.md) for the public API and [development guide](../development.md) for
artifact identity requirements.

## Breaking change

`spire_lab._native.NativeIroncladCombatBatch` was removed without an alias. The replacement is
`NativeCombatTrainingBatch(..., kind="ironclad")`. The supported `IroncladCombatVectorEnv`
facade remains the Gym66 entry point. Its `diagnostic()` adds per-slot `slots` statuses, and
batch-fault messages use `native combat fault`; export failures propagate the caller's exception
and freeze the batch. Full details are in [Batching](../batching.md#breaking-changes).

## Profile

**Objective.** Reuse one native `SessionBatch<S>` owner for Run and combat-training sessions,
with uniform observation descriptors, mixed configurations, ordered outputs, partial reset,
three autoreset modes, per-slot fault/recovery reporting, and thin Python vector facades. Extend
M34s `PublicSearch` with `step_many` over its shared scheduler. Add a source digest to
`build_info()` and verify the loaded extension in Lane P.

**Owner boundaries.** `spire-core` remains the rules and state owner. `spire-env` owns
`SessionBatch`, Run/combat sessions and `PublicSearch.step_many`. `spire-python` transports typed
arrays, tokens, reports and build identity. The Python/Gymnasium modules remain thin facades. No
second game engine or Python batch loop is introduced.

**Delivery path.** Source checks and native tests are separate from Python extension delivery.
Lane P uses the candidate export and records `_native.__file__`, `build_info()`, source and
artifact digests, facade hashes, descriptor fingerprints and exact results. Post-merge identity
also requires `dirty=false` with the same source digest as Lane P.

## Coverage ledger

| Contract | Evidence in source | Evidence path | Status |
| --- | --- | --- | --- |
| Shared owner, 1–256 slots, descriptor uniformity, worker pool and scalar seed expansion | `SessionBatch`, `Scheduler`, native mixed-session and seed tests | `crates/spire-env/src/session_batch.rs`; `crates/spire-env/tests/m35_session_batch.rs`; `crates/spire-env/tests/m35_combat_training_batch.rs`; `crates/spire-env/src/tests/m35_batch_unit.rs` | Source candidate; passed (Checks) |
| Partial reset, sticky automatic entries, DISABLED/NEXT_STEP/SAME_STEP, final rows, records and `combat_views()` | Native autoreset and terminal comparisons | `crates/spire-env/tests/m35_autoreset.rs`; `crates/spire-env/src/tests/m35_r2.rs` | Source candidate; passed (Checks) |
| Freeze/isolate fault policies, committed-result recovery, export failure and fault records | Fault-stage and recovery tests | `crates/spire-env/src/tests/m35_batch_faults.rs` | Source candidate; passed (Checks) |
| Gym66 batch rebase and `standard-combat-v1` mixed catalog | Shared combat owner and catalog-kind tests | `crates/spire-env/tests/m35_combat_training_batch.rs`; `crates/spire-python/src/batch.rs` | Source candidate; passed on the lane P extension (Checks) |
| R1–R7 native and Python agreement, plus R4-neg entry negative control | Mixed Run, autoreset, combat, finite-profile and digest-fixture tests | `crates/spire-env/tests/m35_session_batch.rs`; `crates/spire-env/tests/m35_autoreset.rs`; `crates/spire-env/tests/m35_combat_training_batch.rs`; `crates/spire-env/tests/m35_agreement_digests.rs`; `crates/spire-env/tests/fixtures/m35_agreement_digests.json`; `tests/python/env/test_m35_agreement.py` | Source candidate; passed (Checks) |
| R8 `PublicSearch.step_many`, privacy, fault, preflight and shared pool | Crate-private M35 search tests | `crates/spire-env/src/public_search.rs`; `crates/spire-env/src/tests/m35_step_many.rs` | Source candidate; passed (Checks) |
| Gymnasium facades and extension identity | Vector conformance, wrapper compatibility, export recovery, agreement/search batch and identity tests | `python/spire_lab/gymnasium/_vector.py`; `python/spire_lab/gymnasium/run_vector.py`; `python/spire_lab/gymnasium/standard_combat_vector.py`; `tests/python/env/test_m35_agreement.py`; `tests/python/env/test_build_identity.py` | Source candidate; passed on the lane P extension (Checks) |
| Native class removal and Gym66 compatibility contract | Removal, facade and gated task 0 golden tests | `tests/python/test_ironclad_training.py`; `tests/python/env/test_compatibility.py`; `tests/python/env/test_m35_agreement.py`; `/home/klottick/.local/share/spire-gym/evidence/m35-execution-20261110/m35/task0/golden/` | Source candidate; golden comparison v1 427/427 and v2 427/427 rows equal on the lane P extension (Checks) |

## Checks

Results are from the read-only candidate snapshots c3 (Rust lanes) and c4 (lane P), recorded under
`evidence/m35-execution-20261110/m35/c3-checks/` and `c4-checks/`. c3→c4 changed only the extension-only
`crates/spire-python/src/batch.rs` (error text) and docs.

| Lane | Command | Result |
| --- | --- | --- |
| Native Run batch | `cargo test -p spire-env --locked --offline --test m35_session_batch` | c3, with the next three binaries (`--release`): 29 passed, 0 failed, 29.2 s |
| Native autoreset and records | `cargo test -p spire-env --locked --offline --test m35_autoreset` | c3: included above |
| Native combat-training batch and R5 | `cargo test -p spire-env --locked --offline --test m35_combat_training_batch` | c3: included above |
| Native R7 digest fixture | `cargo test -p spire-env --locked --offline --test m35_agreement_digests` | c3: included above; workers 1 and 4 equal |
| Native terminal, batching unit/fault, R8 and unchanged M34s search | `cargo test -p spire-env --locked --offline --lib m35_r2`; `cargo test -p spire-env --locked --offline --lib m35_batch`; `cargo test -p spire-env --locked --offline --lib m35_step_many`; `cargo test -p spire-env --locked --offline --lib m34s_search` | c3 (`--release --lib -- m35_ workers_faults seed`): 46 passed, 0 failed |
| Python Lane P | `uv run --locked --no-sync python -B -m pytest tests/python/env -q`; `uv run --locked --no-sync python -B -m pytest tests/python/test_ironclad_training.py tests/python/test_combat_policy_view.py tests/python/test_card_metadata.py -q` | c4 extension built from the read-only export (`features=["spire-python/extension"]`, `profile=release`, `dirty=unknown`, `source_digest=083f4061…`): 427 passed, 0 skipped (with `tests/python/test_static_audit.py`); 10 examples exit 0; all 9 descriptor fingerprints equal the e3574ed6 packet; n14g/bhl/tsw replays line-equal to the e3574ed6 baselines, no `RunFaultError` |
| Gated Gym66 task 0 golden comparison | `SPIRE_M35_GYM66_GOLDEN=/home/klottick/.local/share/spire-gym/evidence/m35-execution-20261110/m35/task0/golden uv run --locked --no-sync python -B -m pytest tests/python/env/test_m35_agreement.py -q -k test_gym66_task0_golden_comparison` | c4 lane P (gate set): passed; v1 and v2 427/427 rows equal, only delta the added `diagnostic()` `slots` key |
| Binding helper tests (default features; PyO3 extension is built in Lane P) | `cargo test -p spire-python --locked --offline` | c4: 15 passed, 0 failed |
| Workspace (runner A) | `cargo test --workspace --locked --offline --no-fail-fast` | c3: 5,123 passed, 0 failed, 19 ignored; 74 test names added, none removed |
| Diagnostics, fuzz, traversal, frozen replay, examples (runner B) | as the M34s-c runner brief steps 1, 2, 5–8 | c1 (carried to c3/c4: those lanes do not link the batch modules): diagnostics 7,941 passed; fuzz 51 passed; traversal byte-identical to base, M33 gate A1–A20; fuzz trace and gameplay hashes equal to base, no failure group; frozen replay 323/323 equal; c3 examples equal to base |
| Gym66 throughput (report only) | `evidence/.../m35/throughput/bench_gym66.py`, B=16, 1,500 steps × 3, alternating | c4: 37.31 µs per slot-step vs base 38.61 (no regression) |
| Final review | fresh independent senior | c1 NOT READY (6 Important), fixed; c2 and c3 READY (`evidence/.../m35/verify-v1/`) |

## Reconciliation

The implementation follows the M35 design with these bound changes and disclosed limits:

- `SearchConfig` remains unchanged for existing M34s callers. Native search worker selection is
  `PublicSearch::with_workers(n)`; Python accepts `workers=1` on `public_search(...)`.
- The merged M34s-c catalog uses `entry_catalog`, `records` as a byte sequence and includes
  `bottled_cards`. R8 tests are crate-private in `src/tests/m35_step_many.rs` to use test hooks.
- Gym66/Standard catalog-kind mismatches keep the scalar-compatible
  `RunUnsupportedBoundaryError`, as pinned by M34s and task 0. A nonempty Gym66 `records` value
  and an unknown kind remain input errors.
- `combat_views()` reports each slot's scalar `combat_view()`; noncombat rows keep the scalar
  view's current resources and empty action mask. Faulted or isolated slots have no view and are
  returned as zero rows. A SAME_STEP autoreset composite always reports stage `autoreset`, even
  when reset panics; the panic text remains in the error message.
- R2 has no Standard EmptyDeck route. The supplied Standard routes cover HeartVictory/TrueVictory
  and Act3NoKeys; a separate A20 R1 route covers Death.
- The binding expands u64 scalar seeds through `spire_env::expand_scalar_seed` (`seed + slot`).
  The existing combat-v1 pool retains `expand_seeds`/`IntegerSeed`; the u64 rule is equivalent,
  checked by `scalar_seed_expansion_shared`, and the pool still accepts wider integer seeds.
- Step-many whole-call preflight errors retain the scalar step error class and message but do not
  name the offending entry index. A job panic is reported as committed conservatively; that branch
  is frozen either way.
- P5 also changes Rust finite-profile `RunBatch`: after a sibling fault, healthy peers in M10,
  M14, M15 and M23–M25 stay healthy and readable (not frozen; the batch itself refuses further resets and steps) and no
  longer receive `DeliveryFault`. Nine accepted tests
  were amended in `/home/klottick/.local/share/spire-gym/evidence/m35-execution-20261110/m35/amendments/ledger.md`:
  `m23_batch_postdispatch_faults_keep_indexed_peer_delivery_with_actual_core_fault`,
  `m24_batch_postdispatch_faults_with_actual_core_errors`,
  `m23_batch_postdispatch_faults_keep_indexed_peer_delivery`, `m24_batch_postdispatch_faults`,
  `m14_batch_retains_first_capacity_fault_and_accepted_peer_work`, `m25_batch_postdispatch_faults`,
  `execution_fault_keeps_failing_owner_and_marks_only_committed_undelivered_peers`,
  `reset_capacity_fault_retains_first_error_and_marks_successful_peer_resets_undelivered`, and
  `m15_capacity_fault_retains_first_prefix_and_quiesces_delivered_peers`.
- `tests/python/env/test_m35_agreement.py` covers the planned R1, R3, R4, R5 and R6 comparisons
  and recomputes the R7 facade-array digests. The task 0 golden comparison is gated by
  `SPIRE_M35_GYM66_GOLDEN`; its evidence record reports 427/427 rows equal for each of v1 and v2,
  with only the added `diagnostic().slots` key as a diagnostic delta.
- M35 does not promise original-game runtime parity, strategy performance, throughput or a
  completed Python delivery until Lane P and the post-merge build are verified.

The relevant source and test evidence is retained under
`/home/klottick/.local/share/spire-gym/evidence/m35-execution-20261110/m35/`, including
`task0/`, `amendments/`, the task briefs, runner logs and delivery drafts. A managed worktree
path is not an independent retention copy.
