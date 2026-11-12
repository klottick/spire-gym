# M38e Compact Standard observation

Status: source candidate, 2026-11-12, verified READY by a fresh pre-seal verifier. Bead `spire-qu9.6` (parent
`spire-qu9`); the user approved an opt-in compact schema on 2026-11-12 (`spire-qu9.4`). Base: main `aafb80ea` (M38b).
One merge.

## Profile

**Objective.** Add an opt-in, compact encoding of the Standard Run observation next to the frozen dense one. It carries
the same public information in far fewer bytes: the same groups, columns, column order, vocabularies, references and
semantics as `spire-gym-standard-run-observation-v1`, with fewer rows per group and narrower integer value types. The
dense observation stays the default and does not change.

**Why.** [M38a](../profiling.md#m38a-parallel-scaling) found that observation bytes, not instructions, limit batch
scaling. The dense observation is 763,720 bytes per slot and is written in full every step. [M38b](../profiling.md#after-m38b)
made the transfer faster without changing any bytes. M38e reduces the bytes themselves.

**Delivered.** The contract is in [Standard Run API](../standard-run-api.md#compact-observation-m38e); the batch notes are
in [batching](../batching.md#faults-recovery-and-export).
- Schema `spire-gym-standard-run-observation-compact-v1` with its own fingerprint domain, capacity preset
  `standard-compact-v1` and fingerprint `633b9424be5070a2890b01725949447badba919c5c8c5bd798922f8c2d2a2b2d` at that preset,
  added to the freeze identity table.
- Rows sized from measured maxima (supplied Heart routes and the A0–A20 sweep, supplied full-length fights, random
  A0/A10/A20 play, stress and strong-play generation fights), values `int16`/`int32` per group: 136,842 bytes of arrays
  per slot (17.9% of dense); native `i64` rows 280,438 bytes.
- Whenever compact encodes, widening it to `int64` and zero-padding its rows gives the dense observation byte for byte.
- A state that does not fit is a typed `Capacity` fault through the existing paths (row overflow by group; a value
  outside its dtype by `"<group>.<column>"`), never a truncation. `fault_cause()` names the cause with or without
  recording.
- Records are identified by their fingerprint only: compact records replay, restore and export compact; checkpoints,
  forks and isolate resumes keep the layout.
- API (additive, defaults unchanged): keyword `observation="dense"|"compact"` on `NativeRunSession`, `NativeRunBatch`,
  `RunEnv` and `RunVectorEnv`; `fault_cause()` methods; Rust `RunObservationLayout`, `new_standard_with_layout`,
  `observation_layout`, `fault_cause`, `standard_compact`, `standard_compact_run_observation_spec`,
  `record_observation_layout`, `encode_run_standard_decision_with_layout`; `RunRecordError::from_error` is now public.
  See [CHANGES](../../CHANGES.md).
- Diagnostics: `M37_LAYOUT` and `M37_STACK=slab` (`m37_phase_timing`), `--observation` (`a20_checks`,
  `examples/run_vector_phase_timing.py`, `examples/speed_check.py`).

**Out of scope.** No compact combat view: `combat_view()` and public search of a compact session are the existing
combat view at the session's capacities. `standard-combat-v1`, tactical starts and the captured and legacy profiles
(M9, M10, Gym66, combat-v1) are unchanged. A native recycle API stays deferred (`spire-qu9.5`).

## Checks (candidate v1)

- Native: workspace suite 5,175 passed, 0 failed, 25 ignored; 34 added tests, none removed. The compact tests cover the
  descriptor difference from dense, the pinned identity, the dtype check on every write, typed targets, sessions,
  records and replay (including value faults), capacity edges, layout carry-over, views, batches (freeze, isolate,
  NEXT_STEP and SAME_STEP autoreset overflow) and the widening equivalence on the 8 supplied Heart routes, the supplied
  full-length variants and 96 random runs (4 characters × A0/A10/A20 × 8 seeds), every room kind through the Heart.
- Dense unchanged against the base: dense observation digests of 80 routes byte-identical; the M38b Python packaging
  differential all equal; frozen replay 648/648 files equal; fuzz, A0–A20 traversal, freeze tests, Heart pins and the
  84-route Heart sweep equal or passing; the 9 frozen descriptor fingerprints equal the base packet.
- Python (source-matched release extension): full suite 1,257 passed with the 60 pre-existing `test_rust_differential.py`
  failures identical by ID; the compact facade tests passed.
- Final review: the reviewer re-ran the dense digests on candidate v2 (byte-identical) and found that every Python session
  construction, fork and `from_record` built the observation descriptor (about 1.6 ms per fork). Candidate v3 removes
  that; those operations are back at the base within noise.

## Measurements

Quiet machine, interleaved runs, base `aafb80ea` against the candidate; medians
([profiling](../profiling.md#after-m38e), [performance](../performance.md#a20-standard-checks-after-m38e--2026-11-12)).
- Compact against dense (ratios are medians of per-run paired ratios): native batch, 64 slots, rows kept, 2.65× at 16 workers and 3.06× at 32 (1.49× and 1.70× when
  rows are dropped early); `a20_checks` B64 17.6k / 54.5k / 92.5k slot steps/s at 1 / 4 / 16 workers against 12.8k /
  31.7k / 39.1k; Python `NativeRunBatch.step` 3.26 / 0.58 ms against 4.83 / 1.34 ms at 1 / 16 workers; both speed
  checks about 1.4–1.5× at 1 worker and 2.3× at 16. A 256-slot batch uses about 267 MB instead of 647 MB.
- Dense against the base: native unchanged within noise (`a20_checks` B64 +0.7% / +1.5% / −2.6%). The dense Python
  batch step is about 3–5% slower, over the plan's ~3% gate; a bisect places it in the engine and a profile shows no
  hotspot, so it is disclosed rather than fixed here (PM ruling; a dense-only writer is a possible follow-up).

## Limits

Capacities come from measured maxima, not engine bounds. A policy that builds unusually large decks, very long
generation fights (past about 40 turns) or repeated Courier restocks can exceed them; it then gets a typed fault, never
wrong data, and larger capacities (up to the dense preset) cure a row overflow but not a value fault. Performance
numbers are single-host diagnostics. Python delivery counts only for a source-matched imported extension.
