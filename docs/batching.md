# Native batching and vector facades (M35)

## Breaking changes

`spire_lab._native.NativeIroncladCombatBatch` has been removed and has no alias. Replace
`NativeIroncladCombatBatch(n, catalog, partition=p)` with
`NativeCombatTrainingBatch(n, catalog_json, kind="ironclad", partition=p)`. The supported
`IroncladCombatVectorEnv` facade retains its existing positional arguments and reset/step
return shapes; M35 adds optional `workers`, `autoreset_mode` and `fault_policy` keywords.

Gym66 `diagnostic()` keeps its existing keys and adds `slots`, one delivery-status record per
slot. After a sibling faults, healthy `diagnostic()["sessions"][k]["fault"]` values are `null`
(P5). Gym66 batch faults now say `native combat fault: batch <operation> failed`; they raise
`BatchFaultError`, a `RunFaultError` subclass. Python packaging/export failures propagate the
caller's original exception and freeze the batch. Later reset or step calls raise
`BatchFaultError(operation="export")`; use `recover_committed()` to package retained results.

## Owner and construction

`spire-env::SessionBatch<S>` in `crates/spire-env/src/session_batch.rs` owns Run and combat-training
batches. `RunBatch` remains a thin wrapper for finite-profile Rust callers. `CombatTrainingBatch`
is `SessionBatch<CombatTrainingSession>` over a shared `Arc<CombatTrainingCatalog>`. The Gym66
PyO3 batch loop was removed. Python code packages results and implements no game, reward, seed or
batch progression rules.

`NativeRunBatch(configs, *, profile, capacities=None, workers=1, autoreset="next_step",
fault_policy="freeze")` accepts 1–256 Run configurations. The slots use one profile and observation
descriptor; Run slots must have matching descriptor fingerprints, including capacities. Configs
may vary by character, ascension, partition, creation and recording.

`NativeCombatTrainingBatch(num_envs, catalog_json, *, kind, records=None, partition="train",
entry_ids=None, workers=1, autoreset="disabled", fault_policy="freeze")` accepts `kind="ironclad"`
for Gym66 v1/v2 or `kind="standard"` for `standard-combat-v1`. The catalog must match the kind;
the scalar-compatible `RunUnsupportedBoundaryError` is used for a kind mismatch. Nonempty records
with `kind="ironclad"` and unknown kinds are input errors. Standard records support
`reachable_recipe` entries. All slots share the catalog and descriptor.

The Python facades are:

| Facade | Signature |
| --- | --- |
| `RunVectorEnv` | `RunVectorEnv(configs, *, profile="standard", workers=1, autoreset_mode="next_step", fault_policy="freeze", capacities=None)` |
| `StandardCombatVectorEnv` | `StandardCombatVectorEnv(entry_catalog, num_envs, *, records=None, partition="train", entry_ids=None, workers=1, autoreset_mode="next_step", fault_policy="freeze")` |
| `IroncladCombatVectorEnv` | `IroncladCombatVectorEnv(num_envs, entry_catalog, *, partition="train", workers=1, autoreset_mode="disabled", fault_policy="freeze")` |

They are Gymnasium 1.4 `VectorEnv` facades. `metadata["autoreset_mode"]` reflects the selected
mode. `close(**kwargs)` is accepted by all three. Spaces come from native descriptors and
`batch_space`; returned arrays are owned copies. New Run and Standard-combat vector info uses
dict-of-arrays with `_key` presence masks. Gym66 retains its existing per-slot `info["slots"]`
list and disclosed `effective_seed` field.

All three facades expose Gymnasium `reset(*, seed=None, options=None)` and `step(actions)`.
Native batches expose `reset`/`step`; Run reset takes seeds and a mask, while combat-training
reset also takes per-slot `entry_ids`. Both native batch types expose slot `decision`, diagnostics,
completed/fault record drains and `close`; Run also exposes `combat_views`, and combat training
also exposes `catalog_descriptor`. Both expose `committed_result`, `recover_committed`, record
and reproduction reads where that session type supports them.

## Slots, seeds and workers

Rows and fault records use slot order. Tokens, reset ordinals and records remain slot-local. A
batch has 1–256 slots, requires at least one worker, and requires one descriptor fingerprint for
all slots. Input and token preflight happens before dispatch. A valid slot's trajectory matches
the same scalar session and actions; completion order and worker count do not select game state.

`reset` accepts per-slot `Some(seed)` or `None`. A first `None` uses the scalar session's existing
default (seed 0 for Run, entropy for combat training); later `None` continues that slot's ordinal.
A Python scalar seed is expanded natively to `seed + slot` for each selected slot, with overflow
rejected before dispatch. A length-B list supplies individual seeds. Python facades do not derive
seeds. The M35 `SessionBatch` binding path uses `spire_env::expand_scalar_seed` for u64 seeds.
The existing combat-v1 pool keeps `expand_seeds`/`IntegerSeed`; it uses the same `seed + slot`
rule for u64 seeds, as checked by `scalar_seed_expansion_shared`, and continues to accept wider
integer seeds.

Run and combat-training batches use a private persistent Rayon pool when their effective worker
count exceeds one. Worker stacks are 8 MiB. Since M38b, reset and step jobs give each worker
a home range: with `w` workers and `n` slots, worker `i` first runs slots `i*n/w` to
`(i+1)*n/w - 1`, so a slot's state normally stays with one worker thread between calls. A worker
that finishes its range then takes still-unclaimed slots from the other ranges, so one long job,
such as a combat setup or an autoreset that builds a new Run, does not hold up the call. An
earlier version without that second step was slower on some trajectories
([profiling](profiling.md#after-m38b)). Results and fault records still come back in slot order,
each slot's job still catches its own panic, and results do not depend on the worker count or
on which worker ran a slot. `PublicSearch.step_many` instead uses
`Scheduler::shared(workers)`, a lazily created process-wide pool per count; workers=1 is serial
and touches no pool.

**Keep the live observations small (M38b).** Each returned row owns its observation. A Standard
observation with the default capacities is about 764 KB, so 64 rows hold about 49 MB. A caller
that keeps the previous call's rows until the next `step` returns has about 98 MB of
observations live while the workers encode, more than the host's last-level caches. Native
callers should copy what they need from the rows (actions, tokens, the few features a policy
reads) and drop the rows before the next `step`. On the M38a host, at 64 slots with recording
off, doing that raised the batch from 47.6k–50.7k to 65.1k–84.5k slot steps/s at 16 workers and
from 43.4k–46.0k to 86.9k–89.5k at 32; at 1 worker the gain is about 3–5%
([profiling](profiling.md#where-the-16-worker-time-goes)). Nothing else changes: the rows, their
order and their bytes are the same either way.

**Python output memory (M38b).** `NativeRunBatch.step` (Standard) still returns new arrays on
every call, with the same bytes, dtypes, shapes and flags as before. When NumPy releases a
returned `obs` array (no array or view refers to it any more), its memory goes back to a pool
owned by the batch and a later step reuses it. Memory that any live array can reach is never
reused. The pool holds at most two calls' observation arrays (about 98 MB at 64 slots) between
calls. `close()` frees what it holds. Arrays released later, after `close()` or even after the
batch object itself is freed, still return to the pool, within the same bound, and the pool is
freed only when the last array from that batch has been released. Releasing the previous `obs` before the next `step`, keeping only
what the policy needs, lets that step reuse the memory at once; on the M38a host a 16-worker step
then took 0.71–0.84 ms instead of 1.25–1.31 ms
([performance](performance.md#a20-standard-checks-after-m38b--2026-11-12)). Keeping arrays alive
is always safe; it only costs memory and that reuse.

## Reset and autoreset

The first reset must select every slot. Later masked resets leave unselected slots' boundary,
token, record, entry and pending autoreset unchanged. Results remain full width; unselected rows
are reread without drawing. Resetting a slot clears its pending NEXT_STEP autoreset.

| Mode | Behavior |
| --- | --- |
| `disabled` | Stepping a terminal slot rejects the whole call with no effects. Reset terminal slots explicitly. |
| `next_step` | On the call after a terminal result, the slot resets. Its action and token are ignored; the row has zero reward and false terminal flags plus `info["autoreset"]`. This is the default for new Run and Standard-combat facades. |
| `same_step` | The terminal call returns the new reset row. The terminal boundary/info are available as `final_obs`/`final_info` with `_final_obs`/`_final_info` masks. This remains supported for combat-v1 consumers. |

For combat-training slots, automatic NEXT_STEP and SAME_STEP resets keep the slot's current
catalog entry, including character, ascension and start kind. Explicit resets keep the scalar
rule: a supplied per-slot entry is used; if omitted, the constructor's `entry_ids[slot]` is used,
then `entries[0]`. The automatic rule never falls back to `entries[0]`.

The terminal observation is the scalar session's terminal boundary: Standard TrueVictory after
HeartVictory, Gym66's post-settlement terminal, or the settlement-free `standard-combat-v1`
combat terminal. Batch Run sessions have no native time limit; `truncated` is false. On reset,
recording slots move completed records to `take_completed_records()` before the next episode is
installed. Drain the queue to bound caller-owned memory. Each finished episode lands in exactly one
queue: if the reset that follows it faults, the episode stays in `take_completed_records()` and the
later isolate resume reports only the reset fault in `take_fault_records()` (record `None`).

`combat_views()` on Run batches returns each slot's scalar `combat_view()` verbatim, stacked with
`in_combat`. Noncombat rows therefore retain the scalar view's current resources and empty mask;
they are not replaced with zero rows. Faulted and isolated slots have no view and are packaged as
zero rows. Profiles without scalar `combat_view()` support return `RunUnsupportedBoundaryError`.

## Faults, recovery and export

Each operation reports one status per slot when a runtime fault is raised:

| Status | Meaning |
| --- | --- |
| `delivered` | The row was returned. |
| `committed_undelivered` | The transition committed; the session is healthy and the result can be recovered. |
| `not_committed` | The operation was validated but did not commit. |
| `faulted` | The slot froze at the reported stage; `committed` describes its prefix. |
| `unaffected` | The slot was not selected or was skipped under isolate policy. |

Stages are `reset`, `step`, `autoreset`, `encode` and `panic`. A SAME_STEP autoreset composite
always reports stage `autoreset`, including when the reset panics; the panic text remains in the
message. `BatchFaultError` has `.operation` (`reset`, `step` or `export`), `.policy` (`freeze` or
`isolate`) and `.slots`, a tuple of maps with exactly `status`, `stage`, `committed`, `recoverable`
and `message` keys. `gameplay_recorded` is not exposed in these maps. Python preflight errors retain
the existing `RunInputError`, `RunStateError`, `RunUnsupportedBoundaryError` and `RunBudgetError`
classes and their scalar messages unchanged, with `.slot` (the slot index, or `None`). Only the isolate
resume rejection names the slot in its text: `faulted slot needs an explicit seed (slot k)`.

Under the default `freeze` policy the batch rejects later reset and step calls after a runtime
fault. Healthy peers remain readable and are not frozen. `decision`, `committed_result`,
`recover_committed`, records, diagnostics and fault-record drains remain available.

With `fault_policy="isolate"`, only the faulted slot freezes. It returns an all-zero observation,
empty mask, zero reward and false terminal flags, with `info["slot_fault"]` and bounded fault
details. Its action and token are ignored; it is never automatically replaced. The caller may
resume it only with an explicit masked reset and an explicit seed for that slot, either from the
per-slot seed list or scalar `seed + slot`. A reset with `None` for a faulted slot is rejected.
Likewise, a full `reset(seed=None)` is rejected while any slot is faulted. The replacement is a
new session; the original fault/record moves to `take_fault_records()`.

The batch retains only the non-boundary parts of the last committed result. Recovery rebuilds
observations from each session's published boundary; it does not copy a second view. A boundary
is retained by value only when it is no longer published, such as SAME_STEP `final_obs` or the
terminal boundary of a slot whose autoreset faulted. The worst case retains B terminal views in
one SAME_STEP call (about 0.5–0.7 MB per Standard view); the retained result is released by that
slot's next successful operation. `committed_result(slot)` is read-only and repeatable.

If Python packaging fails after native work, the caller's exception propagates unchanged and
the batch freezes under either policy. Committed rows are marked recoverable. A later reset/step
raises `BatchFaultError(operation="export")`; `recover_committed()` repackages the retained data
without replaying an action. A fault is never a loss, reward or retry. A recovered row carries the
committed transition only: an isolated slot's recovered SAME_STEP terminal has no `slot_fault`
flag; read the slot's fault from `BatchFaultError.slots` or `diagnostics()`.

A panic outside the per-slot jobs (scheduler, record reads, post-join bookkeeping) freezes the
batch. Each slot's status then comes from its publication serial: a slot that published in the
failed operation is `committed_undelivered` with `recoverable: false` (its earlier retained result
is dropped), an isolated slot is `unaffected`, and every other slot is `not_committed`. Known limit:
a slot whose isolate resume is interrupted by such a panic is judged by serials of two different sessions
and can be reported `unaffected`; the batch is frozen either way.

`CombatPool`/combat-v1 `CombatFaultError.slots` is informational only. `CombatPool` has no
`decision()` or `recover_committed()` API, so an informational committed-undelivered status there
does not imply recovery.

## Gymnasium 1.4 compatibility tested

The compatibility tests check descriptor-derived spaces and vector conformance: every observation
row and final observation is contained in the single space; action masks and terminal dtypes are
checked; info presence masks have boolean slot shapes; same-seed resets match; returned arrays do
not alias later calls. They check `VectorObservationWrapper` on DISABLED/NEXT_STEP and its
`ValueError` rejection of SAME_STEP. The combat-v1 vector statistics wrapper is exercised for
SAME_STEP returns/lengths and masked-reset isolation; combat-v1 `DictInfoToList` and single
`CombatEnv` identity `TransformObservation` are also exercised. The supported single-env checker subset uses
mask-valid actions. These tests do not establish universal wrapper support or unmodified,
mask-blind `check_env` acceptance.

## Gym66 compatibility contract

| Surface | M35 contract |
| --- | --- |
| Facade constructor | Existing positional and keyword call shape remains; optional `workers=1`, `autoreset_mode="disabled"` and `fault_policy="freeze"` are added. |
| Catalog kind | `IroncladCombatVectorEnv` accepts Gym66 v1/v2 only. `StandardCombatVectorEnv` accepts `standard-combat-v1` only. The opposite catalog kind is rejected with the scalar-compatible `RunUnsupportedBoundaryError`. |
| Reset/step | Shapes, dtypes and values match the retained task 0 golden. The gated comparison reports 427/427 rows equal for each of v1 and v2. |
| `info["slots"]` | Row dictionaries keep the existing keys, including `effective_seed`; no row keys are added. New `autoreset`, `slot_fault` and `fault` keys are top-level and only appear when active. |
| `diagnostic()` | Existing keys remain; adds `slots` with delivery statuses. `first_fault` is from the lowest-index faulted slot. `sessions[k].fault` is null for healthy peers after a sibling fault (P5). |
| Fault errors | `BatchFaultError` remains catchable as `RunFaultError`; the message changes to “native combat fault”. Input/state error classes stay the same. |
| Replay logs | Compare n14g/bhl/tsw with prior baselines modulo the added `diagnostic().slots`, the P5 healthy-peer `sessions[k].fault` value where faulted batches occur, and the documented message change. |

## Batched public search (M35)

`PublicSearch::step_many(&[BranchStepRequest], view)` accepts distinct live branches and returns one
`BranchStepOutcome` per entry in input order. It validates every request before dispatch: duplicate,
foreign, closed, faulted, terminal or step-limit-truncated branches, stale tokens, illegal actions,
hidden-order picks and insufficient total-step budget reject the whole call without advancing any
branch. Whole-call preflight errors keep the scalar `step` error class and message and do not name
the offending entry index. A job panic is reported conservatively as `committed=true`; that branch
is frozen either way. Each worker touches only its own sampled Run; the shared resolver is used, and the arena
applies counters, action history, step counts, truncations and faults after the join in input
order. A fault freezes that branch only; siblings are not retried or changed. Forks inherit the
parent's step count.

`PublicSearch::with_workers(n)` configures execution, not a budget. `n=1` creates no pool; larger
values use the lazily shared process pool for that worker count. The Python API is
`public_search(research_seed, ..., workers=1)` and
`NativePublicSearch.step_many(branches, tokens, actions, *, view="full")`. It returns stacked
results; `view="mask"` omits encoded observations. Branch handles keep per-branch tokens. Budget
errors map to `RunBudgetError`; the M34s privacy guarantees continue to apply to every output.

The public search worker count is bounded to `1..=256`; zero and values above
256 return `Input("search workers must be 1..=256")` (Python exposes this as
`RunInputError`). `workers <= 1` creates or touches no pool. For
parallel calls, the registry retains weak handles for each worker count and one
strong hot pool for the most recently taken count. A live search takes its pool
on its first parallel `step_many` and holds it until that search is dropped or
`with_workers` changes its worker count.
Thus resident pools are bounded by the live searches' worker counts plus one
hot pool, rather than one permanently resident pool per count. This reverses
M35 P13's per-count residency rule.

## Agreement corpus and commands

The M35 corpus covers Standard Run and `standard-combat-v1` across Ironclad, Silent, Defect and
Watcher at A0 and A20; scalar, serial, worker and vector paths; autoreset/final rows; Gym66 task 0
goldens; and the batched M34s search/privacy oracles. R8 uses 16 roots: one Run combat and one
standalone Standard-combat root for each character/ascension pair. Native test declarations
are `mixed_standard_batch_matches_single_sessions`,
`worker_count_and_slot_permutation_do_not_change_rows`,
`standard_terminal_views_match_single`, `next_step_autoreset_keeps_slot_entry`,
`same_step_autoreset_keeps_slot_entry`, `r4_negative_control_detects_entry_zero_rule`,
`gym66_batch_matches_scalar_sessions`, `standard_combat_mixed_entries_match_single`,
`r6_python_run_profiles_match_single_sessions`, `m35_agreement_digests`, and the nine
`step_many_*` tests in the crate-private module.

| Corpus | Coverage and source tests |
| --- | --- |
| R1 | Mixed Standard Run slots (4 characters × A0/A20), scalar/worker/permutation agreement: `mixed_standard_batch_matches_single_sessions`, `worker_count_and_slot_permutation_do_not_change_rows`; Python vector/scalar and combat-view agreement: `test_r1_run_vector_matches_scalar_combat_views_and_r7_digest`. |
| R2 | HeartVictory/TrueVictory, Act3NoKeys and Death terminal boundaries: `standard_terminal_views_match_single`; no Standard EmptyDeck route is retained. |
| R3 | Reset cycles, masked reset, pending autoreset and completed records: `autoreset_cycles_match_single_reset_none`, `masked_reset_clears_pending_autoreset`, `completed_records_drain_in_order`; Python mode and pending-reset comparison: `test_r3_autoreset_modes_and_masked_pending_reset_match_scalar`. |
| R4 / R4-neg | Standard combat mixed entries and sticky automatic entries in both autoreset modes; the old entries[0] rule is a negative control: `standard_combat_mixed_entries_match_single`, `next_step_autoreset_keeps_slot_entry`, `same_step_autoreset_keeps_slot_entry`, `r4_negative_control_detects_entry_zero_rule`; Python vector/scalar comparison: `test_r4_standard_combat_vector_matches_scalar_and_r7_digest`. |
| R5 | Gym66 v1/v2 batch versus scalar: `gym66_batch_matches_scalar_sessions`, `test_batch_matches_independent_seeds_and_manual_masked_reset_preserves_other_rows`, `test_r5_ironclad_vector_matches_scalar_v1_v2`. `test_gym66_task0_golden_comparison` runs when `SPIRE_M35_GYM66_GOLDEN` names the golden directory; evidence records 427/427 rows equal for v1 and v2, with only the added `diagnostic().slots` delta. |
| R6 | Native M9, M10 Heart and M10 policy-v3 through the generic owner at workers 1/4: `r6_python_run_profiles_match_single_sessions`; Python vector/scalar smoke: `test_r6_m9_m10_run_vector_smoke_matches_scalar`. |
| R7 | Native observation digest fixture and worker-count comparison: `m35_agreement_digests`; Python facade-array digests are checked by `test_r1_run_vector_matches_scalar_combat_views_and_r7_digest` and `test_r4_standard_combat_vector_matches_scalar_and_r7_digest`. |
| R8 | 16 roots, branch stepping, and M34s privacy/fault oracles: nine `step_many_*` tests in `crates/spire-env/src/tests/m35_step_many.rs`; Python tests `test_step_many_full_and_mask_match_sequential_and_update_owned_tokens`, `test_step_many_rejects_mixed_search_owners_before_mutation`, `test_step_many_parallel_search_owners_match_sequential_results` and `test_step_many_budget_preflight_is_atomic`. |

Run these native lanes from the repository root:

```sh
cargo test -p spire-env --locked --offline --test m35_session_batch
cargo test -p spire-env --locked --offline --test m35_autoreset
cargo test -p spire-env --locked --offline --test m35_combat_training_batch
cargo test -p spire-env --locked --offline --test m35_agreement_digests
cargo test -p spire-env --locked --offline --lib m35_r2
cargo test -p spire-env --locked --offline --lib m35_batch
cargo test -p spire-env --locked --offline --lib m35_step_many
cargo test -p spire-env --locked --offline --lib m34s_search
cargo test -p spire-python --locked --offline
```

On the already provisioned, source-matched Gym environment, Lane P runs:

```sh
uv run --locked --no-sync python -B -m pytest tests/python/env -q
uv run --locked --no-sync python -B -m pytest tests/python/test_ironclad_training.py tests/python/test_combat_policy_view.py tests/python/test_card_metadata.py -q
SPIRE_M35_GYM66_GOLDEN=<private-evidence>/m35-execution-20261110/m35/task0/golden uv run --locked --no-sync python -B -m pytest tests/python/env/test_m35_agreement.py -q -k test_gym66_task0_golden_comparison
```

Without `SPIRE_M35_GYM66_GOLDEN`, the task 0 comparison is skipped with an explicit message.

The spire-python Rust test command intentionally uses default features; its non-PyO3 helpers are
tested there. The extension build and import are checked in Lane P, not by a separate Rust
extension-feature test lane.

These commands identify lanes, not results. Python results count only when the imported
`spire_lab._native.__file__` and build identity receipt match the candidate sources. The
[development guide](development.md#agent-lane-selection-and-artifact-identity) describes the
source-digest check and Lane P receipt.
