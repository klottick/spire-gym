# M39b — compact Standard combat view

**Candidate; independent review pending.** Bead `spire-pqs`. M39b adds an opt-in
`observation="compact"` layout for standalone Standard combat sessions and
vectors, the native Standard combat batch, and PublicSearch full branch views.
Dense remains the default.

The schema is `spire-gym-standard-combat-view-compact-v1`. Compact
preserves the dense view's columns, vocabularies, row order and action
indices. Widening values to int64 and zero-padding to the dense shape
reconstructs the dense arrays. Caller overrides extend to the corresponding
dense capacities. A capacity or dtype overflow fails with a typed cause.

| Layout or preset | Array bytes per slot (measured) |
|---|---:|
| Dense | 644,378 |
| `standard-combat-compact-v1` (default when compact is selected) | 229,926 |
| `standard-combat-compact-play-v1` (explicit `capacity_preset` opt-in) | 143,313 |

The sizing sweep included all four A20 characters, ordinary 12–26-card decks,
generator-first play, fights up to 84 turns and supplied decks up to 999
cards. The smaller preset allows 832 combat-card rows. The retained
generation-heavy peak was 403 rows. The row caps remain enforceable at
runtime. A compact search branch requires a fitting initial full view. A
compact batch using Isolate can return typed zero rows even when every
initial reset faults.

Acceptance covers scalar, batch, final and recovered observations, existing
Run-record inputs for reachable recipes and PublicSearch's information-set
tests. Standalone Standard combat has no episode recorder or exact checkpoint
API. Existing Run combat projections and legacy combat layouts keep their
dense defaults. See the [Standard Run API](../standard-run-api.md#standard-combat-view-and-sessions),
[batching contract](../batching.md), and
[freeze and change protocol](../delivery.md#freeze-and-change-protocol).

## Measurements (one host, native Rust and Python extension, dense vs compact)

Median of five interleaved pairs on an idle host, 64-slot batches and the
281-trace wide fixture. These are single-host diagnostics, not training or
strength claims.

| Native step median | Dense | Compact bounds | Compact play |
|---|---:|---:|---:|
| Scalar step | 92 µs | 68 µs | 64 µs |
| 64-slot batch step, 1 worker | 7.80 ms | 4.04 ms | 3.73 ms |
| 64-slot batch step, 16 workers | 1.98 ms | 0.72 ms | 0.68 ms |
| Search `step_many` full, 16 branches, 1 worker | 1.19 ms | 0.95 ms | 0.92 ms |

The Python 64-slot batch step improves from 10.98 ms (1 worker) and 4.87 ms
(16 workers) to about 4.0 ms and 1.1 ms with compact. Compact construction of
a scalar session costs 0.8 µs (dense 70 ns) because it validates the compact group capacities.
The dense default path against the base: scalar and 64-slot batch steps were
within about 2% on one worker; sub-microsecond metrics and 16-worker search
`step_many` vary by more than 3% between runs and are listed in the evidence
index, not hidden.

The sizing fixtures were regenerated on the rules version in this branch (173
bounds starts and 108 play starts; 14,069 and 3,603 accepted actions) because a
preceding resolver fix changed a few random-play trajectories.
