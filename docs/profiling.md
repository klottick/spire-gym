# Profiling the Standard A20 workload (M37a)

This report explains where the time goes in the M36 Standard A20 measurements
([performance](performance.md)). It covers the native scalar run, native batch
scaling and the Python vector path, and it proposes fixes for a later milestone
(M37b). It is diagnostic only. No engine code changed for it. Gameplay, IDs,
fingerprints and observations are unchanged, and the numbers below are not
thresholds or promises. M37b has since implemented five of the six fixes. The "After
M37b" columns and [After M37b](#after-m37b) give the re-measured numbers; the rest of the
report describes v1.0.0, except [M38a: parallel scaling](#m38a-parallel-scaling), which
measures why the batch scales poorly after M37b and ranks the next fixes.

## Method

| Item | Value |
| --- | --- |
| Host | AMD Ryzen 9 9950X, 16 cores / 32 threads, L2 1 MiB per core, L3 2 × 32 MiB; 59 GiB RAM |
| OS / kernel / governor | Ubuntu, Linux 7.0.0-42-generic, `amd-pstate` with the `powersave` governor (default); no core pinning |
| Toolchain | rustc 1.99.0; `perf` 7.0.14 with `perf_event_paranoid` 1 and `kptr_restrict` 0 |
| Source | v1.0.0 plus a README rewrite (`52219686`), plus the uncommitted M37a harness below; rules `spire-gym-native-standard-rules-v3`, observation fingerprint `bf8e1d11…` |
| Python | CPython 3.12.15, NumPy 2.5.4, Gymnasium 1.4.0; the v1.0.0 wheel's `_native` (SHA-256 `5398d2fd…`), source-matched to this checkout: `file_digest` and `source_digest` recompute equal to `build_info()` |
| Date | 2026-11-11, on a quiet machine (no other build or test running) |

Two builds of the same source were used, each in its own target directory:

- **Timing:** a plain `--release` build. All throughput and phase numbers come from it.
- **Profile:** `--release` with `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only` and
  `RUSTFLAGS="-C force-frame-pointers=yes"`, sampled with `perf record -F 1999` or
  `-F 4999 -g`. Its frame pointers cost a few percent; use it for shares, not rates.

Shares come from `perf script` call chains. *Self* is the leaf frame, and *inclusive*
counts every sample whose stack contains the frame (the same as `perf report --children`).
Inlined helpers show up under their own names: `branch<RunObservation, …>` is the `?`
after `RunObservation::owned`, and `branch<Vec<i64>, …>` is the zeroed `reserved`
allocation inside the encoder. The memset and memcpy loops in libc have no symbols.
We attribute them to the nearest named caller. No flamegraph tool is installed, so this
report summarises the call chains as tables. Counter data comes from `perf stat`. The AMD
`ls_any_fills_from_sys.dram_io_all` event estimates DRAM fills.

### Phase-timing harness

The phase-timing harness has two parts, and neither changes gameplay or a public API.

- **`m37_phase_timing`**, an ignored test in `crates/spire-env/src/tests/m37_phase_timing.rs`.
  It runs the `a20_checks` workload: Standard A20, all four characters, and the seeded
  R1 legal-action rule.
  - *Scalar pass 1* times the public `RunSession::reset` and `step`.
  - *Scalar pass 2* replays the same seeds and actions through the steps that
    `RunSession::step` and `publish` run, with one timer per step. It must reproduce the
    episode lengths and terminals of pass 1. It then times the fixed Standard layout alone.
  - *Batch* wraps `RunSession` in a timing session inside a `SessionBatch`. It records
    each job's worker, start and end, and splits each call into serial head, parallel span
    and serial tail. It also times a copy of the rows in the order `spire-python`'s
    `stacking::stack` uses (see Workload P for how that figure compares with the binding).
  - Environment variables select the part, seeds, slots, steps, workers, recording and
    output (see the file header). The test build includes dev-only diagnostic features,
    and its public step is about 8% slower than the `a20_checks` example. Compare shares,
    not absolute rates, between the two.
- **`examples/run_vector_phase_timing.py`** times three loops on one batch shape:
  - the `run_vector_rollout.py` loop (per-slot `rng.choice` plus `RunVectorEnv.step`);
  - `RunVectorEnv.step` alone;
  - `_native.NativeRunBatch.step` alone.

## Workload S: scalar Standard A20

`a20_checks` on the timing build: 1,024 episodes, 42,783 actions, **10,278 actions/s**
(step plus observation read; 97 µs per action). The M36 figure was 10,297.

Harness, scalar pass 2. The values are µs per accepted action, averaged over 42,783
actions; the two runs agree within 2%.

| Phase | µs/step | Share | After M37b, µs/step |
| --- | ---: | ---: | ---: |
| `Run::step` (game logic, including its own legal-action check; after M37b `Run::step_offered`, without it) | 42.1 | 44.0% | 24.5 |
| `encode_run_standard` (dense observation and action mask) | 17.9 | 18.7% | 20.3 |
| `Run::decision` (public decision and legal candidates) | 17.6 | 18.4% | 7.2 |
| `RunObservation::owned` (the returned copy) | 12.5 | 13.1% | none |
| Episode reset (`create_run`, amortised: 109 µs per episode) | 2.6 | 2.7% | 2.6 |
| Observation drops, public map, policy, reward ledger | 3.0 | 3.1% | 2.6 |
| **Sum** (pass 1's public `step` is 105.8 µs; about 10 µs are session bookkeeping and cache effects; after M37b, 49.2 µs) | 95.7 | | 57.1 |

Every Standard observation has the same dense layout of **763,720 bytes**: values are
`i64`; known flags, occupancy and mask are `i8`. The `standard-default-v1` capacities
make `combat_cards` 268 KB, `actions` 168 KB, `offers` 97 KB, `powers` 76 KB and
`selection` 56 KB. Random A20 play fills at most 47 combat-card and 26 action rows
([capacity headroom](performance.md#c--capacity-headroom)). Layout probes give the
following floors:

| Probe | µs |
| --- | ---: |
| Fresh zeroed allocation and drop | 4.1 |
| Zeroing a reused buffer | 3.3 |
| Fresh copy and drop (`owned`) | 11.0 |
| Copy into a reused buffer | 9.3 |

perf, public `step` (pass 1 samples, profile build):

| Path (inclusive) | Share |
| --- | ---: |
| `RunSession::step_with_status` | 93.2% |
| ├ `Run::step` | 40.8% |
| │ ├ legal-action check (`Combat::legal_actions_full` from `choose_combat`) | 16.8% |
| │ ├ `Combat::step_full` (the actual rules resolution) | 10.9% |
| │ └ the rest: `settle_combat` 2.6%, combat public view 2.5%, room and map steps 8.0% | 13.1% |
| ├ `publish` | 37.8% |
| │ ├ `Run::decision`: legal-action enumeration 16.6%, the rest 2.3% | 18.9% |
| │ └ `encode_run_with_card_encoder`: its zeroed `reserved` allocations are 5.6% | 17.8% |
| └ `RunObservation::owned` | 13.3% |
| `Combat::legal_actions_full`, both callers | 33.3% |
| └ `cards::prepare` → `payload_with_wrist_blade` | 31.1% |
| ├ `combat::model::State::clone` | 17.8% |
| └ drop of that `State` | 10.0% |

Self time by DSO: libc 58% (memset, memcpy, malloc and free), the binary 38% and the
kernel 3%. The largest named sources of libc time are the following:

| libc source | Share |
| --- | ---: |
| `owned` copy | 8.9% |
| `HookEvent` vectors cloned with `State` | 5.9% |
| Encoder zeroing | 5.0% |
| `Actor` drops | 4.6% |

**Game logic compared with observation building.** The public scalar step divides as
follows:

| Part | Share |
| --- | ---: |
| Rules work (`Run::step` without its legal check) | 24% |
| Legal-action enumeration (both callers) | 33% |
| Observation building (decision projection, encode, copy) | 34% |

Rules resolution proper (`Combat::step_full`) is 11%.

**Why the older Ironclad profiles were 147k–388k actions/s.** Two differences come from
reading the source; they were not measured here. Card preparation clones `State` for
each (card, target) pair it prepares whenever the combat has a complete item context
(`complete_items`), which every Standard combat has; it also clones for Vigor or an
eligible Wrist Blade attack. Those older measurements also timed native transitions and public menus
without the 764 KB Standard encoding. Their scripted workloads differ from random A20
play, so the rates are not directly comparable.

## Workload B: native batch scaling

Harness batch part: 64 mixed-character slots, NextStep autoreset, 100 calls per row.
These are run 1 numbers. Run 2 is within 7% (for example, 16 workers with recording on
gave 13,175 slot steps/s). In the After M37b column, run 2 differs from run 1 by up to 11%
(16 workers, recording on: 40,307). `a20_checks` measured 5,195, 10,997 and 12,128 slot steps/s at
1, 4 and 16 workers on the same host.

| Workers | `record_replay` | Slot steps/s | Step job (µs) | Busy sum (ms) | Critical path (ms) | Wall (ms) | Head + tail (ms) | After M37b: slot steps/s, step job (µs) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | on | 4,791 | 161 | 1,169 | 1,169 | 1,336 | 9.4 | 12,433, 78 |
| 4 | on | 11,099 | 258 | 1,803 | 500 | 577 | 8.0 | 30,442, 121 |
| 16 | on | 14,111 | 726 | 4,723 | 331 | 454 | 8.5 | 45,356, 296 |
| 1 | off | 6,131 | 161 | 1,036 | 1,036 | 1,044 | 4.7 | 12,593, 77 |
| 4 | off | 15,017 | 252 | 1,625 | 416 | 426 | 7.6 | 30,976, 118 |
| 16 | off | 19,695 | 725 | 4,666 | 313 | 325 | 9.5 | 46,183, 295 |

**The serial part is small.** The work before the first job and after the last job
(preflight, joining rows, scheduler wake-up) takes 5–10 ms of a 325–1,336 ms run, about
1–2%. Locks and the scheduler are not the bottleneck.

**The per-job cost grows with workers.** The mean step job takes 161 µs at 1 worker,
about 255 µs at 4 and about 725 µs at 16. That is 4.5 times the cost for the same work.

`perf stat` over 300 calls (recording off, no stacking):

| Workers | Instructions | IPC | Context switches |
| ---: | ---: | ---: | ---: |
| 1 | 25.1 G | 1.36 | 14 |
| 4 | 26.5 G | 0.94 | 67,924 |
| 16 | 29.1 G | 0.36 | 120,905 |

DRAM fills are about 1 MB per slot step. Dividing the whole-process fills (setup included)
by the batch wall time gives about 18 GB/s at 16 workers, a rough figure rather than a
bandwidth measurement. In the worker
frames, the encoder's zeroing and the `owned` memcpy together take **30% of samples at
1 worker and 55% at 16** (25% zeroing and 29% copying at 16). The kernel rises from
1% to 6%.

So instructions stay nearly flat while cycles per instruction quadruple, mostly inside
the bulk writes and copies of the 764 KB observations. Batch scaling is limited by
memory traffic from dense observations, not by a serial section. Even at 1 worker, a
batch step costs 161 µs against the scalar 106 µs. The 64 sessions, each holding about
1.5 MB of observations, do not fit in cache.

**`record_replay` is expensive at reset.** With recording on, an autoreset job takes
1.3–1.5 ms, against 0.2, 0.3 and 0.9 ms with recording off at 1, 4 and 16 workers. Exporting the finished record (`batch_record`) adds another
1.3–2.5 ms. Both call `new_record()`, which rebuilds the observation spec and its SHA-256
fingerprint (`record()` then overwrites it with the stored value, so that rebuild is
wasted): `observation_spec` is 22% of batch worker samples at 1 worker. Throughput with
recording on is 22–28% lower than with it off (4.8k against 6.1k, 11.1k against 15.0k
and 14.1k against 19.7k slot steps/s at 1, 4 and 16 workers). These long
jobs also unbalance workers: at 16 workers the critical path is 12% above busy time
divided by workers.

## Workload P: Python vector

`examples/run_vector_phase_timing.py`: 64 slots, A20, 2,000 calls per loop, the v1.0.0
`.so` (source-matched as above). Partition `train` and seed 7 differ from the native
harness, which uses `test` and per-slot seeds. Both are random A20 play.

| Workers | Loop | Policy (ms/call) | Step (ms/call) | Slot steps/s |
| ---: | --- | ---: | ---: | ---: |
| 1 | `run_vector_rollout.py` loop | 0.42 | 13.53 | 4,590 |
| 1 | `RunVectorEnv.step` | 0.44 | 13.72 | 4,522 |
| 1 | `NativeRunBatch.step` | 0.46 | 13.61 | 4,548 |
| 16 | `run_vector_rollout.py` loop | 0.42 | 6.98 | 8,646 |
| 16 | `RunVectorEnv.step` | 0.43 | 6.78 | 8,881 |
| 16 | `NativeRunBatch.step` | 0.45 | 6.83 | 8,781 |

- The Gymnasium facade adds no measurable cost per call. Facade and binding differ by
  less than run-to-run noise.
- The example's per-slot Python action choice is 3% of the call.
- **The binding's packaging is the Python-specific cost.** The native batch step is
  10.4 ms at 1 worker and 3.3 ms at 16 (harness, recording off). That leaves about
  **3.2 ms and 3.5 ms per call** for packaging: about 22% of the call at 1 worker and
  about 50% at 16. Packaging stacks 64 dense observations, **48.9 MB per call**, into
  fresh vectors handed to NumPy, and it builds per-slot info dicts. It runs on the
  calling thread with the GIL held, after the parallel part, so it is the serial part of
  the Python vector.
- That 3.2–3.5 ms comes from subtracting two processes with different partitions, seeds
  and slot order, and the test build is about 8% slower, so treat it as approximate. The
  harness's own stacking copy, timed in Rust inside one process, disagrees: 8.4 ms per call at
  1 worker and 7.2–7.6 ms at 4 and 16 with recording off, but 3.1 ms with recording on.
  That timer includes faulting in and freeing about 49 MB of fresh memory per call, and
  its spread with allocator state (recording changes the heap) shows the cost depends on
  page-fault and allocator behaviour. Both methods show a large serial copy; neither pins
  its size closely.
- perf on that thread at 16 workers (a fresh capture, summary kept with the evidence):
  46.6% of its samples are in libc (unsymbolised; the copy and free paths) and 45.7% in
  the kernel (`_raw_spin_lock`, `zap_present_ptes` unmapping freed arrays,
  `lock_vma_under_rcu` and page-fault handling). A first capture gave 47.6% and 43.9%.

## The three M36 symptoms

- **Scalar 10.3k against 147k–388k actions/s.** About 24% of a Standard step is rules
  work. 33% enumerates legal actions, mostly by cloning and dropping the combat `State`
  for each (card, target) pair. 34% builds and copies a 764 KB observation. The older profiles skip the
  `State` clone and the dense Standard encoding.
- **Batch 5.2k → 11.2k → 12.7k.** These M36 figures have `record_replay` on. The batch
  has no meaningful serial section. Two causes account for the rate. First, per-job
  cost grows 4.5 times from 1 to 16 workers, and the step becomes memory-bound in the
  zeroing and copying of dense observations (IPC falls to 0.36). Second, recording
  rebuilds the observation-spec fingerprint on every reset and record export (22% of
  worker time at 1 worker); without recording the harness reaches about 6.1k, 15.0k and
  19.7k.
- **Python 4.5k / 8.7k.** Recording is off here. The native step, plus single-threaded
  packaging under the GIL (49 MB copied per call; about 3–3.5 ms by subtraction), caps
  the 16-worker gain at less than half the native recording-off rate.

## Proposed fixes, ranked

The gains below are the M37a estimates from the shares above. The last column gives
the M37b outcome; [After M37b](#after-m37b) has the measured results. Each fix must leave gameplay, IDs, fingerprints, observations and replay
digests unchanged. Check it with the freeze tests, the traversal, frozen replay and the
fuzz hashes ([freeze protocol](delivery.md#freeze-and-change-protocol)). The frozen
replay corpus covers only short Act 1 runs, so ranks 1, 4 and 5 also need a differential
check. Compare old and new card payloads, legal-action lists or encoded observations
over a broad set of states (late-game decks, Watcher stances, relics, Vigor and Wrist
Blade).

| Rank | Bead | Fix | Measured share | Expected gain | Risk | Observable change | M37b |
| ---: | --- | --- | --- | --- | --- | --- | --- |
| 1 | spire-4yo.2.1 | Project only the mutated parts (item context, Vigor) in `payload_with_wrist_blade` instead of cloning `State` per (card, target) pair | 28% of scalar step (clone 17.8% + drop 10.0%) | about −25 µs/step, scalar about 1.3× | Medium: core rules path | None if payloads stay identical | done |
| 2 | spire-4yo.2.3 | Return the encoded observation by move; stop the per-step `owned` copy (shared or re-encoded published copy, internal to the session) | 13% scalar; most of the 29% memcpy share at 16 workers | about −12 µs scalar; a large part of 16-worker memory traffic | Low–medium: fault and recovery paths read the published boundary; the allocation-failure freeze path of `owned` goes away | None; public `RunBoundary` and `RunObservation` types unchanged | done |
| 3 | spire-4yo.2.5 | Build the stacked Python columns inside the GIL-free section, per slot in parallel | about 3–3.5 ms per call by subtraction (about half the 16-worker call; see Workload P) | 16-worker Python vector up to about 1.7× (rough estimate) | Medium | None, provided each call still returns new arrays; reusing returned buffers would be observable and is excluded | done |
| 4 | spire-4yo.2.2 | Validate the chosen action against the decision's candidates instead of re-enumerating legal actions in `Run::step` | 16.8% of scalar step | about −15 µs/step; overlaps rank 1 | Low–medium: cache validity | None; public types unchanged | done |
| 5 | spire-4yo.2.4 | Avoid zero-allocating the full 764 KB layout each step: reuse buffers, clear only used rows, or encode straight into the batch slab (pairs with rank 3) | 5% scalar; 25% of worker time at 16 workers | a few µs scalar; with rank 2, the main batch-scaling lever | Medium | None | not done (see [After M37b](#after-m37b)) |
| 6 | spire-4yo.2.6 | Compute the observation-spec fingerprint once per session (or memoise per profile and capacities) in `new_record` | 22% of batch time with `record_replay` at 1 worker; 1.3–2.5 ms per reset or export | about +25–30% batch throughput with recording on; less worker imbalance | Very low | None (same value) | done |

Suggested order: rank 6 first (small and safe), then ranks 2, 1 and 4, then ranks 5 and 3
together, since both concern how observations reach the batch output. Re-measure after
each step: removing memory traffic changes the batch shares.

Not proposed: smaller capacity presets also cut the bytes moved. A caller can already
pass smaller `capacities`, but that is an opt-in observable change: different shapes and
a different fingerprint. It is not a safe speedup under the freeze.

## After M37b

M37b (Bead `spire-4yo.2`) implemented ranks 1, 2, 3, 4 and 6 on 2026-11-11. Nothing
observable changed: gameplay, RNG use, IDs, rules IDs, fingerprints, observations, masks,
records and replay bytes all stay the same. The freeze tests, frozen replay (323/323),
traversal and fuzz hashes match base `bad72c7f`. Two lockstep checks compare old and new
paths. The routine one covers 14,900 decisions and 145,576 (card, target) payloads, and
the broad ignored one covers 82,511 decisions and 802,317 payloads. Both span four
characters at A0 and A20, with injected Wrist Blade, Akabeko and Pen Nib, up to Act 3.

- **Rank 1:** card preparation clones `State` once per enumeration and undoes each
  card's projection edits. The scalar `Run::decision` falls from 17.6 to 7.2 µs.
- **Rank 4:** the session steps with the internal `Run::step_offered`, which skips the
  re-enumeration for an action from the table it just published. `Run::step` falls from
  42.1 to 24.5 µs.
- **Rank 2:** the session returns the observation by move and keeps no copy;
  `decision()` re-encodes it from the unchanged Run. This removes the 12.5 µs copy, and
  exact checkpoints shrink from about 0.9 MB to 0.07–0.17 MB. The allocation-failure
  freeze path that belonged to the removed `owned` copy goes with it.
  Reading an observation again now costs a re-encode instead of a copy, with the same
  bytes. That covers `decision()`, the kept slots of a masked batch reset and
  `committed_result`, none of which is on the step path.
- **Rank 6:** each session builds the fingerprint once. An autoreset job with recording
  on falls from about 1.3 ms to 0.15–0.48 ms. Recording now costs about 1–9% of batch
  throughput across the two runs, down from 22–28%.
- **Rank 3:** `NativeRunBatch` stacks each slot's rows on the batch's own workers, with
  the GIL released, and drops the rows there. Each call still returns new arrays. The
  1-worker Python step falls from 13.5 to 8.5 ms per call, and the 16-worker step from
  6.6 to about 3.8–5.1 ms, depending on allocator state.

**Rank 5 (not done; PM ruling).** After rank 2 the scalar API returns a fresh, owned
dense observation every step, so writing the full layout once per step is the floor.
The zeroing left is about 4 µs of a step of about 57 µs. Clearing only the rows used
before is not exact without more work: the encoder writes reference `-1`s below
`occupied_count` without marking occupancy, and it writes values directly in three
places. Exact clearing would need dirty-extent tracking across about 30 write sites plus
a new recycle API. **Possible future item:** a batch-only recycle, in which the stacking
job hands each buffer back to its session, with dirty-extent tracking and a
byte-identical differential. This is not planned.

What remains in a scalar step is rules work (`Run::step`, about 24 µs) and encoding the
764 KB observation (about 20 µs). At 16 workers, batch step jobs still cost about 3.8
times the 1-worker job (296 µs against 78 µs), because every step writes a fresh dense
observation. Measurements: [performance](performance.md#a20-standard-checks-after-m37b--2026-11-11).

## M38a: parallel scaling

M38a asks why the native batch gains only about 3.8 times from 1 to 16 workers, and it ranks
what to fix in phase 2 (Bead `spire-qu9.2`). This is measurement only. No engine, binding or
facade code changed. Gameplay, IDs, fingerprints, observations and replay bytes are the same
as after M37b. The numbers are diagnostic, not thresholds.

**Answer.** The dense observation costs the batch through how many observation bytes are live,
and touched, during each call. It does not cost extra instructions or clock speed. At 64 slots and 16
workers there are three effects. The first two are of similar size, each worth about 1.3–2
times; only the first was measured inside the batch itself. The third is smaller:

1. **The live working set.** A caller keeps the previous call's 64 rows (about 49 MB) until
   the next call returns. While the workers encode 64 new observations, about 98 MB is live,
   more than the two L3 caches (64 MB). Dropping the previous rows before the call, with
   nothing else changed, raises the batch from 47.6k–50.7k to 65.1k–84.5k slot steps/s at 16
   workers, and from 43.4k–46.0k to 86.9k–89.5k at 32. Even at 1 worker, where no slot changes
   worker, a slot step fills about 420–450 KiB from DRAM.
2. **Lockstep synchronisation.** Plain threads that keep their own runs reach 160k–166k
   turns/s at 16 threads without a barrier, and 83k–99k with a barrier after every round. The
   barrier's cost inside the batch was not measured.
3. **Slots changing worker.** Rayon runs 84–86% of slots on a different worker each call at
   16 workers. At the batch's live set, plain threads lose about 10–15% when their runs rotate
   between them. With half that live set, the loss is about 40%, because then the data would
   otherwise stay in cache. That figure is for plain threads, which also move their
   observations; the batch moves only Run and session state (see below).

Page faults are few, and glibc settings do not help the native step. Whether DRAM bandwidth
is also a limit was not determined: reads stay near 20 GB/s in the slow cases, and bandwidth
itself was not measured. In the Python binding at 16 workers, the stacking copy and the
allocator's handling of fresh 49 MB outputs dominate.

### Method

Same host and governor as [Method](#method): 16 cores and 32 threads in two CCDs (L3 per CCD;
CPUs *n* and *n*+16 are SMT siblings), no core pinning unless stated. Source: the M38a source,
which differs from M37b only by the test and example harness below (rules
`spire-gym-native-standard-rules-v3`, observation fingerprint `bf8e1d11…`). The timing,
profile and Python builds are the same kinds as in M37a. The Python extension was built from
the same source with `maturin build --release`; its `build_info()` reports features
`["spire-python/extension"]`, profile `release` and `source_digest` `0c84720e…`, with CPython
3.12.15 and NumPy 2.5.4. Each point is a separate process on a quiet machine, run twice unless a table says otherwise.

Counters come from `perf stat` per process. Per-call or per-step values subtract a one-call
run of the same process, to remove setup. Group A counts software events, cycles and
instructions. Group B counts cycles, instructions and four AMD fill events:
`ls_any_fills_from_sys.dram_io_all` (fills from DRAM), `near_cache` (from another CCX),
`local_ccx` (from L3 or another core's L2 in the same CCX) and `local_l2`. Each fill is a 64-byte line.
The host exposes no AMD data-fabric or memory-controller PMU, and `perf_event_paranoid` 1
forbids system-wide counters, so **memory bandwidth is not measured**. Read traffic is
estimated from DRAM fills. Write traffic has no counter and is bounded by the bytes written.

Harness additions (test and example code only):

- **`m38_component_scaling`**, a new ignored test (`crates/spire-env/src/tests/m38_scaling.rs`).
  Plain threads, without the batch or rayon, drive 64 Standard A20 runs through the scalar
  pass-2 steps. They time game logic (decision, public map, reward snapshot, `step_offered`)
  apart from the encode. Every thread count plays the same trajectories: 64 runs, 500
  rounds, 32,000 turns. Modes:
  - `logic`: no observation;
  - `encode`: encode, then drop at once;
  - `hold`: keep each run's last observation, as batch rows are kept;
  - `hold-compact`: `hold` with smaller capacities;
  - `hold-sync`: a barrier after each round;
  - `hold-rotate`: a barrier, and each round a thread takes the next thread's runs, so runs
    change thread every round;
  - `hold-sync-batch` and `hold-rotate-batch`: the same two with the batch's buffer lifetime.
    A replaced observation stays alive until the round's barrier, then one thread frees all of
    them, as a batch caller drops the previous rows after the call. That is about 98 MB live
    at 64 runs.
- **`m37_phase_timing`** batch part:
  - `M37_CAPACITIES=compact` gives the slots smaller capacities: combat cards, permanent cards
    and actions 128; powers, relics, offers and selection 64; map edges 256; the rest
    unchanged. That is an 86,710-byte observation instead of 763,720. Smaller capacities are
    an existing opt-in with a different shape and fingerprint. Here they only size the effect
    of fewer bytes.
  - `M37_STACK=parallel` and `parallel-reuse` stack the rows as the M37b binding does: per
    slot, on the batch's workers, into fresh columns that the caller then drops, or into the
    same columns every call.
  - `M37_EARLY_DROP=1` drops the previous rows after choosing the actions and before the next
    call. Only one call's observations are then live while the workers step.
  - The output reports the share of slots whose job (step, or autoreset) ran on a different
    worker than in the previous call.
- **`examples/run_vector_phase_timing.py`**: `--loops` selects loops. The binding loop also
  reports the time to release the previous call's arrays, and `--drop-before-step` releases
  them before the call instead of after it.

### Scaling curve

`m37_phase_timing`, NextStep autoreset, no stacking; 300 calls at 64 slots and 100 at 256
slots. Values are run 1 / run 2, in slot steps/s.

| Workers | 64 slots, `record_replay` off | 64, on | 256, off | 256, on |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 13,034 / 12,342 | 12,695 / 12,635 | 11,042 / 10,984 | 10,941 / 10,941 |
| 2 | 21,476 / 22,127 | 22,418 / 21,695 | 18,946 / 18,845 | 18,921 / 18,992 |
| 4 | 32,984 / 32,955 | 32,005 / 32,216 | 27,803 / 28,005 | 26,394 / 27,381 |
| 8 | 42,076 / 40,898 | 40,712 / 40,630 | 33,549 / 34,401 | 33,715 / 34,177 |
| 16 | 50,068 / 49,004 | 48,136 / 48,413 | 34,153 / 33,859 | 34,092 / 33,839 |
| 32 | 47,198 / 43,977 | 45,154 / 43,372 | 33,212 / 33,308 | 33,350 / 33,210 |

From 1 to 16 workers, 64 slots gain 3.8–4.0 times and 256 slots 3.1 times. 256 slots stop
gaining at 8 workers. Recording costs at most about 5%. The mean step job grows from 74 µs at
1 worker to 265 µs at 16 and 506 µs at 32 (64 slots, off). The serial head and tail stay at
11–21 ms of a 383–1,473 ms run (64 slots, off). `a20_checks` (recording on, 100 calls) gave 12.0k–12.2k,
18.7k–18.8k, 26.4k–28.5k, 33.3k–35.9k, 34.2k–37.0k and 32.2k–33.3k slot steps/s at 1, 2, 4,
8, 16 and 32 workers (four runs at 1, 4 and 16 workers, two at the others).

**Counters**, 64 slots, recording off. The software counters and IPC are from group A, run 1.
The fills are from a group B rerun with `near_cache` and are KiB per slot step.

| Workers | Cycles | Instructions | IPC | GHz | Page faults | Context switches | Sys share | DRAM | Other CCX | Same CCX | L2 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 445k | 736k | 1.65 | 5.58 | 0.36 | 0.00 | 0.4% | 452 | 0 | 234 | 711 |
| 2 | 521k | 745k | 1.43 | 5.63 | 0.55 | 0.07 | 0.9% | 450 | 3 | 244 | 700 |
| 4 | 649k | 774k | 1.19 | 5.53 | 0.97 | 0.98 | 3.2% | 470 | 12 | 284 | 672 |
| 8 | 982k | 808k | 0.82 | 5.54 | 1.27 | 1.65 | 3.7% | 500 | 22 | 304 | 629 |
| 16 | 1,527k | 895k | 0.59 | 5.45 | 2.49 | 3.39 | 7.1% | 397 | 53 | 345 | 702 |
| 32 | 3,059k | 1,066k | 0.35 | 5.46 | 2.67 | 4.00 | 6.5% | 322 | 67 | 352 | 1,062 |

- **Clock speed is not the cause.** GHz is cycles over task clock, and it stays at 5.4–5.7.
- **The extra work is small.** Instructions per slot step rise 22% from 1 to 16 workers,
  probably in rayon's idle and steal loops: the plain-thread harness below adds none. Cycles
  rise 3.4 times. The extra time is stalls.
- **The kernel is not the cause.** There are at most 2.7 page faults and 4.0 context switches
  per slot step, and the sys share is at most 7%.
- **DRAM saturation was not determined, and bandwidth was not measured.** At 16 workers, DRAM fills
  are about 400 KiB per slot step, about 20 GB/s of reads. That is close to the 8-worker figure
  (about 21 GB/s) and to `hold-rotate` below (about 20 GB/s). Even if every byte of every
  observation were also written back, writes would add at most about 38 GB/s. This is an
  estimate, not a measurement. The host's DRAM peak is not known, so a bandwidth share cannot
  be ruled out. The next section shows that the live working set accounts for a large part of
  the DRAM traffic. Fills from the other CCX are small: 53 KiB per slot step at 16 workers.

### Where the 16-worker time goes

**In the batch: halving the live observations.** `M37_EARLY_DROP=1` makes the harness drop the
previous rows after choosing the actions and before the next call. Only about 49 MB of
observations is then live during a call, instead of about 98 MB, and nothing else changes.
64 slots, recording off, slot steps/s. The 4- to 32-worker rows show three runs, gated so
that no other process used more than half a CPU before any run. The 1- and 2-worker rows
show two runs. "Moved" is the share of slots whose job ran on a different worker than in the
previous call.

| Workers | Rows kept until the call returns (as callers do) | Rows dropped before the call | Moved |
| ---: | ---: | ---: | ---: |
| 1 | 12,974 / 13,104 | 13,637 / 13,552 | 0 |
| 2 | 22,226 / 22,229 | 23,133 / 22,721 | 0.01 |
| 4 | 32,929 / 33,956 / 34,055 | 37,427 / 38,543 / 37,476 | 0.53–0.57 |
| 8 | 40,750 / 44,932 / 43,378 | 53,927 / 64,491 / 50,393 | 0.69–0.75 |
| 16 | 49,886 / 50,669 / 47,618 | 82,632 / 84,468 / 65,125 | 0.84–0.86 |
| 32 | 46,000 / 43,367 / 44,531 | 86,912 / 88,873 / 89,504 | 0.91–0.93 |

At 128 slots and 16 workers, dropping the rows early raises 38.2k / 39.4k to 46.6k / 51.6k.
With the compact capacities, early drop changes nothing: 113.7k–119.4k at 16 workers either
way, since even with the previous rows kept, compact observations take only about 11 MB. The counters below are group B, per slot
step, after a one-call baseline (two runs):

| Case | Cycles | DRAM KiB | Other CCX KiB |
| --- | ---: | ---: | ---: |
| 1 worker, rows kept | 445k–451k | 421–425 | 0 |
| 1 worker, rows dropped early | 427k–429k | 290–297 | 0–1 |
| 16 workers, rows kept | 1,546k–1,779k | 372–400 | 19–52 |
| 16 workers, rows dropped early | 989k–1,357k | 213–340 | 17–45 |

**In plain threads: live set, barrier and rotation apart.** `m38_component_scaling`, 64 runs,
turns/s, three gated runs. The `-batch` rates include each round's serial free by one thread
and an extra barrier, which a caller's drop between calls resembles. At 16 threads that free
is 0.21–0.27 ms of a 1.4 ms round. Without it, `hold-sync-batch` runs at 54.9k–56.3k and
`hold-rotate-batch` at 47.2k–50.7k. The last row repeats the batch rates above for
comparison. They count only the call, not the caller's drop of the previous rows (0.2–0.3 ms
per call).

| Mode | Live observations | Barrier per round | Runs change thread | 8 threads | 16 threads |
| --- | ---: | --- | --- | ---: | ---: |
| `hold` | about 49 MB | no | no | 62.3k–76.9k | 160.5k–166.2k |
| `hold-sync` | about 49 MB | yes | no | 49.8k–63.7k | 82.7k–98.6k |
| `hold-rotate` | about 49 MB | yes | every round | 42.4k–43.1k | 53.2k–56.9k |
| `hold-sync-batch` | about 98 MB | yes | no | 38.5k–39.5k | 45.4k–46.5k |
| `hold-rotate-batch` | about 98 MB | yes | every round | 35.6k–36.5k | 40.4k–40.9k |
| Batch, rows kept / dropped early | about 98 / 49 MB | yes (the call) | 69–86% of slots | 40.8k–44.9k / 50.4k–64.5k | 47.6k–50.7k / 65.1k–84.5k |

With 128 runs at 16 threads (two runs), `hold` gives 74.1k / 74.3k, `hold-sync` 49.1k / 49.9k,
`hold-rotate` 41.7k / 39.7k, `hold-sync-batch` 34.6k / 34.8k and `hold-rotate-batch`
31.3k / 31.9k. Counters per turn at 16 threads and 64 runs (group B, after a one-round
baseline; one run each, at the turns/s shown):

| Mode | Turns/s | Cycles | DRAM KiB | Other CCX KiB |
| --- | ---: | ---: | ---: | ---: |
| `hold` | 135k | 524k | 115 | 0 |
| `hold-sync` | 78k | 736k | 198 | 5 |
| `hold-rotate` | 57k | 1,002k | 235 | 73 |
| `hold-sync-batch` | 42k | 1,302k | 339 | 17 |
| `hold-rotate-batch` | 40k | 1,360k | 367 | 49 |
| `hold-sync`, 1 thread | 13.1k | 431k | 332 | 0 |
| `hold-sync-batch`, 1 thread | 12.2k | 465k | 440 | 0 |

- **The live working set is the largest effect.**
  - Plain threads: giving own-run, barrier-synchronised threads the batch's buffer lifetime
    cuts their throughput at 16 threads by about 35–45% without counting the serial free
    (83k–99k → 54.9k–56.3k turns/s), or by about half with it (45k–47k). Both figures still
    include one extra barrier per round. Their DRAM fills per
    turn rise from 198 to 339 KiB. With 128 runs, about 98 MB live with one barrier and no
    serial free, `hold-sync` gives 49k–50k.
  - The batch: dropping the rows early lifts it 1.1 times at 4 workers, 1.1–1.6 times at 8,
    1.3–1.8 times at 16 and about 2 times at 32. DRAM fills per slot step fall.
  - At 1 worker the effect is about 3–5%.
- **The barrier comes next.** Own-run threads lose 40–50% at 16 threads when they wait for
  each other every round (160k–166k → 83k–99k turns/s). They wait for the slowest thread of
  each round, and threads woken from the barrier can resume on other cores.
- **Changing worker matters most once the live set fits in cache.**
  - Rotation costs plain threads about 10–15% with about 98 MB live (54.9k–56.3k →
    47.2k–50.7k without the serial free) but about 40% with about 49 MB live (83k–99k →
    53k–57k). At 98 MB the observation memory is
    cold anyway.
  - `hold-rotate` overstates the cost for the batch. In the batch, a step job neither reads
    nor frees the slot's previous observation, because the caller owns it. The job writes the
    new observation into its own worker's allocator arena, and the caller frees the old rows
    on its own thread between calls (0.2–0.3 ms per call at 16 workers). So only the slot's
    Run and session state move with it. `hold-rotate` instead frees each moved run's
    observation from another thread inside the timed encode. That is where its allocator lock
    waits, futex wake-ups and higher kernel share come from (perf below).
  - With its rows dropped early, the batch reaches 65k–84k at 16 workers while moving 85% of
    its slots. That is close to `hold-sync` (83k–99k, no moves), not to `hold-rotate` (53k–57k).
- **Game logic and encoding alone scale well.** See the next table.
- **Placement does not fix it.** See [Placement](#placement).

First-pass component curves for the modes without a live-set or barrier question (64 runs,
turns/s, run 1 / run 2):

| Threads | `logic` | `encode` | `hold` | `hold-compact` |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 27.6k / 28.3k | 18.1k / 18.0k | 13.2k / 13.2k | 20.0k / 19.9k |
| 4 | 111k / 108k | 69.1k / 69.1k | 43.2k / 44.1k | 77.3k / 77.8k |
| 16 | 335k / 306k | 227k / 253k | 168k / 166k | 248k / 268k |
| 32 | 386k / 472k | 271k / 263k | 158k / 143k | 348k / 301k |

`logic` and `encode` scale 11–14 times at 16 threads. All `hold` runs at 16 threads, including
those under `perf stat`, gave 135k–168k turns/s, so `hold` scales about 10–13 times.

**perf** (frame-pointer build, 64 slots, recording off):
- From 1 to 16 batch workers, worker samples per slot step grow 4.1 times.
- In the batch, the kernel share of worker samples grows from 1.6% to 8.4%, and the encoder
  subtree from 47% to 56% of worker samples.
- From `hold` to `hold-rotate`, the kernel share grows from 1.7% to 13.6%. The largest growth
  per turn is in unsymbolised libc (the allocator and the bulk copies) and `realloc`, with
  glibc lock waits and futex wake-ups that appear only in `hold-rotate`.

### Allocator: glibc settings, environment only

Same harness, 64 slots, recording off. Values are run 1 / run 2. The stacking column adds the
M37b-style stacking copy on the workers, into fresh columns the caller then drops. Its rate
counts the step, the copy and the drop of the columns (the harness drops them right after the copy). The Python rows are
`run_vector_phase_timing.py --loops binding`, 1,000 calls.

| Setting | Native step, 1 worker | Native step, 16 workers | Native step + stacking + drop, 16 workers | Python step, 1 worker (ms/call) | Python step, 16 workers (ms/call) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Default | 12,685 / 12,466 | 49,165 / 49,276 | 10,887 / 10,564 | 8.32 / 8.29 | 5.02 / 5.03 |
| `MALLOC_ARENA_MAX=1` | 12,604 / 12,604 | 5,215 / 5,928 | 4,692 / 3,798 | not run | not run |
| `MALLOC_ARENA_MAX=4` | 12,419 / 12,391 | 22,393 / 22,242 | 11,779 / 12,338 | 8.29 / 8.26 | 5.42 / 5.49 |
| No trim (below) | 12,496 / 12,389 | 47,001 / 43,811 | 16,019 / 16,111 | 8.30 / 8.21 | 3.89 / 3.96 |
| `MALLOC_MMAP_THRESHOLD_=131072` | 12,369 / 12,387 | 43,068 / 43,040 | 10,086 / 9,940 | 14.95 / 14.60 | 5.97 / 5.95 |
| `glibc.malloc.hugetlb=1` | 12,377 / 12,416 | 47,205 / 47,137 | 10,308 / 10,388 | 8.19 / 8.24 | 5.67 / 3.93 |
| No trim + `hugetlb=1` | 13,144 / 13,299 | 48,517 / 46,319 | 16,167 / 16,127 | 8.15 / 8.21 | 3.87 / 3.88 |

"No trim" is `MALLOC_TRIM_THRESHOLD_=4294967296 MALLOC_TOP_PAD_=268435456
MALLOC_MMAP_THRESHOLD_=33554432`. The native columns are slot steps/s.

- **Native step:** no setting helps. Capping arenas is very harmful: with one arena, 16 workers
  fall to about 5k–6k slot steps/s. By default the native step takes at most 3.2 page faults
  per slot step.
- **Native step with fresh stacked outputs:** no trim helps by about +50% (16.0k against
  10.6k–10.9k slot steps/s). The default drop of the columns costs 1.5–1.6 ms per
  call; with no trim it costs nothing. That is the same effect as in Python, below.
- **Python at 16 workers:** the allocator matters. By default the step takes 84 page faults per
  slot step, with 29% of CPU time in the kernel, and releasing the previous call's arrays
  costs 0.93–0.96 ms. With no trim, faults fall to 0.9 per slot step, the kernel share to 3%,
  the release to 0.03 ms, and the step from 5.02 to 3.89 ms per call (+26% slot steps/s).
  At 1 worker nothing changes. The 3.8–5.1 ms range that M37b saw at 16 workers is
  probably this allocator state.
- jemalloc and mimalloc were not tried, since they would be new dependencies. The results
  above give little reason to: glibc settings do not move the native step, and fixes 1 and 3
  below address the fresh-output effect directly.

### Placement

`taskset` restricts the whole process to a CPU set. 64 slots, recording off, slot steps/s,
run 1 / run 2.

| Workers | Unpinned | One CCD | Both CCDs | SMT pairs | Other |
| ---: | ---: | ---: | ---: | ---: | --- |
| 4 | 32,984 / 32,955 | 32,068 / 30,175 (CPUs 0–3) | 20,349 / 19,869 (0, 1, 8, 9) | 23,953 / 21,766 (0, 1, 16, 17) | |
| 8 | 42,076 / 40,898 | 38,679 / 38,925 (0–7) | 26,868 / 31,536 (0–3, 8–11) | 35,006 / 35,435 (0–3, 16–19) | |
| 16 | 46,522 / 43,263 | 38,299 / 38,796 (0–7, 16–23, SMT) | | | 42,427 / 35,862 (0–15, one thread per core) |
| 32 | 42,365 / 42,237 | | | | 37,086 / 35,887 (0–15) |

The 4- and 8-worker unpinned values are from the scaling curve. Restricting the process was
never faster than leaving it unpinned. Splitting a small pool across the two CCDs costs up to
38%, and SMT siblings cost 25–28% at 4 workers (both against one CCD). Plain threads,
measured with the earlier harness version (its own trajectories), turns/s, run 1 / run 2:

| Mode, threads | Unpinned | CPUs 0–15 (one thread per core) | CPUs 0–7, 16–23 (one CCD, SMT) | CPUs 0–7 (one CCD) | CPUs 0–3, 8–11 (two CCDs) |
| --- | ---: | ---: | ---: | ---: | ---: |
| `hold-sync`, 16 | 88.2k / 91.7k | 45.9k / 64.1k | 38.8k / 38.9k | | |
| `hold-rotate`, 16 | 51.7k / 51.1k | 46.2k / 46.6k | 38.6k / 39.8k | | |
| `hold-sync`, 8 | | | | 35.8k / 36.3k | 50.2k / 32.1k |
| `hold-rotate`, 8 | | | | 35.0k / 35.4k | 35.5k / 34.8k |

At 16 threads, pinning plain threads to a set did not help either. The 8-thread rows have no
unpinned reference in that harness version. Per-worker pinning (one CPU each) was not tested,
since it needs a code change.

### Python vector

Default environment. Values are step ms per call, 2,000 calls, run 1 / run 2.

| Workers | `run_vector_rollout.py` loop | `RunVectorEnv.step` | `NativeRunBatch.step` |
| ---: | ---: | ---: | ---: |
| 1 | 8.13 / 8.10 | 8.16 / 8.19 | 8.03 / 8.08 |
| 16 | 5.22 / 5.12 | 3.99 / 3.97 | 3.80 / 4.81 |

Breakdown of one `NativeRunBatch.step` call at 64 slots. The parts are estimates combined from
perf samples on the profile build, the harness's own timers and the native batch rate.

| Part | 1 worker (ms) | 16 workers (ms) |
| --- | ---: | ---: |
| Native batch step (workers) | about 5.1 | about 1.3 |
| Stacking copy of 48.9 MB (`SlotCopy::run`, on the workers, GIL released) | 2.8–3.7 | about 2.3 |
| Packaging under the GIL (arrays, info dicts) and column reservation | under 0.5 (the remainder) | 0.3–0.4 |
| Releasing the previous call's arrays | 0.03 | 0.03–0.96, by allocator state |
| Waiting for the GIL | none: one Python thread | none |
| **Measured step** | 8.0–8.3 | 3.8–5.0 |

- **The stacking copy is the largest Python cost at 16 workers**, about 60% of the step. In
  the native harness it takes 2.73–2.80 ms per call into fresh columns and 2.28 ms into reused
  ones. It moves 49 MB at about 21 GB/s, which suggests memory traffic rather than compute
  (an inference).
- **A Python call has more live observation data than the native batch.** The caller's
  previous arrays (49 MB) stay alive while the workers write 64 new per-slot rows (49 MB) and
  copy them into 49 MB of fresh columns. The rows are dropped on the workers after the copy.
- **Fresh outputs cost a further 1.5–1.8 ms per call to drop in the native harness**, because
  unmapping pages touched by 16 threads needs TLB shootdowns. In Python the same work appears
  as the 0.93 ms release plus page faults when the allocator returns memory to the kernel.

**Caller-supplied output buffers** (an additive API) would remove allocation, page faults and
freeing, but not the copy. The estimate is about 1.1–1.5 ms per call at 16 workers when the
allocator is in its returning state (5.0 to about 3.5–3.9 ms, 22–30% less step time) and
about 0.4 ms when it is not. At 1 worker it is at most 0.9 ms (native harness: 3.68 to
2.80 ms), with no change in Python. These are estimates from the no-trim run and from
`parallel-reuse`.

**Releasing the previous arrays before the step** (`--drop-before-step`, 1,000 calls, step ms
per call including the release, run 1 / run 2):

| Workers, environment | Released after the step (default) | Released before the step |
| --- | ---: | ---: |
| 1, default | 8.12 / 8.10 | 8.02 / 8.01 |
| 16, default | 4.91 / 3.87 | 5.90 / 6.09 |
| 16, no trim | 3.83 / 3.80 | 3.70 / 3.53 |

With the default allocator at 16 workers, releasing first is slower: 1.84–1.93 ms goes to the
release itself. The freed memory goes back to the kernel and the step faults it in again.
With no trim it is 3–8% faster. On the Python path, a smaller live set helps only together
with memory reuse.

### Game logic

perf on the profile build, scalar pass 2 (`m37_phase_timing`, 42,783 steps, about 59 µs per
step). Shares are of samples. µs is the share times the pass-2 step time. Shares within a frame
overlap.

| Frame (inclusive) | Share of pass 2 | About µs/step |
| --- | ---: | ---: |
| `Run::step_offered` | 38.8% | 22.9 |
| `encode_run_standard` | 34.7% | 20.5 |
| `Run::decision` | 12.0% | 7.1 |

| Inside `Run::step_offered` | Share of the frame |
| --- | ---: |
| `Run::step_checked` (combat steps, card preparation) | 68.1% |
| └ card preparation's single `State` clone (`projected_payload`) and its drop | about 7% |
| `map::enter` → `Run::enter_owned_with` (combat setup when entering a room) | 22.7% / 18.4% |
| `settle_combat` (rewards) | 11.4% |
| `RunState` clones (room snapshots, rewards) | 6.6% |
| `RunRng::draw_scoped` (HMAC-SHA256 per draw, a fresh key vector per draw) | 6.0% |
| Clones and drops of the RNG counter map `BTreeMap<(RunRngScope, Vec<u64>), u64>` | about 8% |
| `HookEvent` vectors (with strings) cloned with `State` | 5.9% |

| Inside the encoder | Share of the frame |
| --- | ---: |
| Zeroed full-layout allocations (`reserved`) and the page faults they take | 7.9% + kernel 17% |
| Vocabulary lookups by name (`card_id`, `named`: linear searches over `serde_json` arrays) | 9.1% + 5.8% (overlapping) |
| `Vec` growth (`grow_one` → `realloc`) | 6.6% |
| Rebuilding the group specification (`run_schema_standard::groups`) each call | 3.1% |

Easy wins, each with no observable change:

| Change | Estimated saving per step | Risk |
| --- | ---: | --- |
| Index vocabulary lookups once (identity → row tables built at first use) instead of searching names | about 2 µs | Low; check every catalogue identity maps to the same row |
| Build the group specification once per capacities; reserve vectors to their final size | about 1.5–2 µs | Low |
| `draw_scoped`: no key vector allocation when the counter exists; preallocated HMAC message | about 0.5 µs | Low; HMAC input bytes unchanged |
| Avoid whole-`RunState` snapshot clones on read-only room probes | up to 1.5 µs (needs a closer look) | Medium |

Together that is about 5–6 µs of the 49 µs public scalar step (M37b), about 10–12% scalar. In the
batch it is the same few µs out of a 265 µs job at 16 workers. Combat setup, rewards and the
remaining card preparation are rules work with no obvious cheap cut.

### Proposed phase-2 fixes, ranked

Gains are estimates from the experiments above. Each fix must keep gameplay, IDs,
fingerprints, observations, masks and replay bytes unchanged, under the
[freeze protocol](delivery.md#freeze-and-change-protocol), with the freeze tests, frozen
replay, traversal and fuzz hashes. Fixes 1 and 3 also need a byte-identical batch packaging
check against the current binding, and fix 2(b) a byte-identical row check.

| Rank | Fix | Evidence | Expected gain | Risk | Change class |
| ---: | --- | --- | --- | --- | --- |
| 1 | **Python: write and keep fewer observation bytes per call.** Encode each slot straight into its chunk of the fresh stacked columns, with no per-slot rows and no second copy. Or recycle the row buffers inside the binding, and write and copy only the rows used (dirty-extent tracking). | Per-slot rows add about 49 MB live to each call; the stacking copy is about 2.3 ms of 3.8–5.0 ms (16 workers) and 2.8–3.7 of 8.0–8.3 ms (1 worker); the live set is the largest native effect | Python 16 workers about 1.5–2× (step about 2–2.5 ms, with fix 3 or in the non-returning allocator state; otherwise the columns' first-touch page faults move into the encode); 1 worker about 1.4–1.5× | Medium–high: an encoder write target, or exact clearing; fault and recovery paths | No observable change if each call still returns new arrays with the same bytes, including zeroed fault rows and unaffected rows |
| 2 | **Native batch: shrink the live set.** (a) Tell native callers to copy what they need from the rows and drop them before the next step. (b) Optionally, let a caller hand the previous rows back, and have the batch reuse their buffers and write only the rows used. | Early drop: 1.1× at 4 workers, 1.3–1.8× at 16, about 2× at 32 (measured). Compact capacities, about 11 MB live, reach 114k–119k at 16 | (a) as measured, for callers that can drop early; (b) at least the early-drop gain for callers that recycle, possibly more since fewer bytes are touched (estimate) | (a) none; (b) medium–high: exact clearing, aliasing rules | (a) documentation only; (b) additive API with unchanged defaults and a `CHANGES.md` entry |
| 3 | **Reuse Python output memory.** (a) An internal pool: when NumPy releases a returned array, its buffer returns to a bounded pool for the next call. Or (b) optional caller-supplied `out` buffers. | No-trim run 5.02 → 3.89 ms at 16 workers; native stacking with no trim +50%; `parallel-reuse` avoids the 1.5–1.8 ms drop; releasing early helps only with reuse | 16 workers about −1.1 to −1.5 ms per call in the returning allocator state, about −0.4 ms otherwise; about 0 at 1 worker. Overlaps fix 1 | Low (a: memory held between calls); low–medium (b: aliasing rules) | (a) no observable change; (b) additive API with unchanged defaults and a `CHANGES.md` entry. `test_frozen_api` checks names only, so an added parameter needs no test change unless a new public name is added; signatures are checked in review |
| 4 | **Sticky slot-to-worker assignment.** Each worker keeps the same slots every call (for example `ThreadPool::broadcast` over fixed slot ranges, or ranges plus stealing when idle), and runs those slots' stacking copies too. | The batch moves 84–86% of slots per call at 16 workers; rotation costs plain threads 10–15% at the batch's live set and about 40% at half of it; the early-drop batch is already near `hold-sync` | Today about 1.1×. After fixes 1–2, up to about 1.2× (estimate). It saves only the Run and session state part, since observation frees stay on the caller | Low–medium. To stay unobservable and within the [batching contract](batching.md), keep rows and fault records in slot order, per-slot panic isolation inside a worker's range, the inline path at one effective worker and the 8 MiB worker stacks. Static ranges lose load balance on long reset jobs | No observable change |
| 5 | **Game-logic and encoder easy wins** (table above) | Profile shares | About 5–6 µs per step, about 10–12% scalar | Low | No observable change |
| 6 | **Lockstep wait.** Measure again after fixes 1–4, then consider keeping workers spinning briefly between calls, or opt-in per-worker CPU pinning | Own-run threads lose 40–50% to a per-round barrier at 16 threads (160k–166k → 83k–99k turns/s) | Uncertain; up to the remaining barrier share | Medium; pinning to a CPU set measured worse | No observable change (an opt-in knob would be additive) |

**Suggested order:**
1. Fix 2(a) now: it is documentation.
2. Fixes 1 and 3 together, since both decide how outputs reach Python.
3. Fix 5, which is small.
4. Then fix 4, and fixes 2(b) and 6, re-measuring after each step.

Not proposed: capping arenas or pinning the process with `taskset` (both measured worse), and
a glibc setting as a default. Until fix 3 lands, a Python user at 16 workers can set the
no-trim environment above. It measured +26%, and it keeps freed memory in the process.

### Separate estimate: an opt-in compact observation

This is a separate decision (`spire-qu9.4`) and is not part of phase 2. A
compact observation would be a new, opt-in schema with its own fingerprint, next to the frozen
dense one. The measurements below use the existing opt-in capacities knob: the same encoder
with the compact capacities above, an 86,710-byte observation (11% of the dense layout).
A real design would also choose dtypes (the values are `i64`) and capacities that cover the
engine's bounds. These capacities are below the bounds `standard-default-v1` covers, so
some late-game states could overflow them.

Native batch, 64 slots, recording off, slot steps/s (one run each, with counters per slot step):

| Workers | Dense (763,720 B) | Compact (86,710 B) | Ratio | Compact: IPC, DRAM KiB |
| ---: | ---: | ---: | ---: | --- |
| 1 | 12,491 | 19,599 | 1.57 | 2.37, 1 |
| 4 | 32,271 | 56,748 | 1.76 | 2.10, 5 |
| 8 | 41,191 | 82,516 | 2.00 | 1.71, 14 |
| 16 | 50,430 | 111,809 | 2.22 | 1.43, 12 |
| 32 | 45,249 | 125,613 | 2.78 | 1.08, 27 |

- **Native batch:** compact scales 5.7 times from 1 to 16 workers (dense: 4.0) and still gains
  at 32. Plain threads gain 1.5 times per step at both 1 and 16 threads (`hold-compact`
  against `hold`).
- **Python (estimate):** the stacked bytes per call fall from 48.9 MB to 5.5 MB, so the
  stacking copy would fall from about 2.3 ms to about 0.3 ms at 16 workers, and allocator
  effects would mostly disappear. With the compact native step (about 0.6 ms at 16 workers)
  plus packaging, a 16-worker Python step of about 1.2–1.5 ms is plausible, against
  3.8–5.0 ms now. This is an estimate, not a measurement.
- **Against fixes 1–3:** they shrink the live and touched bytes without a schema change,
  so they recover part of this. Compact shrinks them about nine times. At 128 and 256 slots,
  where even half the live observations exceed the caches, the byte count is what limits
  scaling.
- **A second pass** gave 19.4k–19.6k at 1 worker and 113.7k–119.4k at 16, with or without
  early drop.

### Reproduce (M38a)

```sh
# Scaling curve point and the compact comparison (timing build)
M37_PART=batch M37_SLOTS=64 M37_RECORD=off M37_STACK=off M37_WORKERS=16 M37_BATCH_STEPS=300 \
  cargo test --release --locked --offline -p spire-env m37_phase_timing -- --ignored --nocapture
M37_CAPACITIES=compact M37_PART=batch M37_STACK=off M37_WORKERS=1,16 \
  cargo test --release --locked --offline -p spire-env m37_phase_timing -- --ignored --nocapture
# Stacking as the binding does it, fresh or reused columns
M37_PART=batch M37_STACK=parallel-reuse M37_WORKERS=16 \
  cargo test --release --locked --offline -p spire-env m37_phase_timing -- --ignored --nocapture
# Live set: drop the previous rows before each call
M37_EARLY_DROP=1 M37_PART=batch M37_SLOTS=64 M37_RECORD=off M37_STACK=off M37_WORKERS=16 M37_BATCH_STEPS=300 \
  cargo test --release --locked --offline -p spire-env m37_phase_timing -- --ignored --nocapture
# Component scaling (all modes, 1 to 32 threads; also with M38_SLOTS=128)
cargo test --release --locked --offline -p spire-env m38_component_scaling -- --ignored --nocapture
M38_THREADS=8,16 M38_MODES=hold,hold-sync,hold-rotate,hold-sync-batch,hold-rotate-batch \
  cargo test --release --locked --offline -p spire-env m38_component_scaling -- --ignored --nocapture
# Counters (run the test binary directly; also once with M37_BATCH_STEPS=1 to subtract setup)
perf stat -e cycles,instructions,ls_any_fills_from_sys.dram_io_all,ls_any_fills_from_sys.near_cache,ls_any_fills_from_sys.local_ccx,ls_any_fills_from_sys.local_l2 \
  <test binary> --ignored --exact tests::m37_phase_timing::m37_phase_timing
# Python binding alone, 16 workers, with and without the no-trim environment and --drop-before-step
uv run --locked --no-sync python examples/run_vector_phase_timing.py --loops binding --workers 16 --steps 1000
```

### Limits (M38a)

- One host. Explicit placement was only tested as process-wide CPU sets. Most table points
  are two or three runs, and some runs differ by 10–30%. Examples are `hold` and `hold-sync`
  at 8 threads, pinned `hold-sync`, and the batch with rows dropped early at 8 and 16 workers.
- The first round of live-set runs started while an unrelated single-threaded process was
  running on the host, and how long it ran was not recorded. The 1- and 2-worker early-drop
  rows, the 128-slot and 128-run figures, the counters and the Python release-order table
  come from that round. The gated rerun repeated the 4–32-worker batch points and the 8- and
  16-thread component points. It agreed with that round in ordering, but some first-round
  values were 10–20% lower, for example `hold` at 16 threads and the batch with rows dropped
  early at 16 workers. The first-round batch rates also ran under `perf stat`.
- Bandwidth is estimated from fill counts, not measured, and the host's DRAM peak is not
  known. There are no uncore counters, and the CPU's memory-bandwidth monitoring (resctrl) needs
  root.
  Group B shares counters (about 83% running time), and perf scales the counts.
- The component harness reproduces the session's steps, not the session or the batch. Its
  modes model the batch's live set, synchronisation and placement; they are not the batch
  itself. The batch's own early-drop runs are the direct evidence for the live set. The share
  of slots that change worker is measured; what moving costs the batch is inferred from the
  plain-thread modes.
- The Python breakdown combines perf samples, harness timers and native rates, so its parts
  are approximate. The workload is random A20 play, mostly Act 1.
- The raw JSON, counter output and perf summaries are kept with the M38a evidence, outside the
  repository; raw `perf.data` files are not.

## After M38b

M38b (Bead `spire-qu9.2`) made the M38a phase-2 fixes on 2026-11-12. Nothing observable
changed: gameplay, RNG use, IDs, rules IDs, fingerprints, observations, masks, records, replay
bytes, and the bytes, dtypes, shapes and flags of every returned array are the same. A packaging
differential compares the Python batch outputs of the old and new bindings call by call. It
covers four characters, A0 and A20, 1 to 16 workers, every autoreset mode, masked resets,
capacity faults under both fault policies, `recover_committed`, recording on and off, and reused
memory that the caller has scribbled over. Every call matched.

- **Fix 1, encode into the stacked arrays.** A Standard `NativeRunBatch.step` hands each slot's
  rows of the new stacked `obs` arrays to that slot's step job. The job zero-fills them and
  encodes the observation straight into them. There are no per-slot rows and no stacking copy.
  Rows that were not encoded there (faults, isolated slots, other profiles) are copied in as
  before. A SAME_STEP terminal observation is copied out before the reset reuses the rows.
- **Fix 3(a), reuse the output memory.** When NumPy releases a returned `obs` array, its buffer
  returns to a pool owned by the batch, and the next step takes it instead of allocating. The
  pool holds at most two calls' columns, and `close()` empties it. Arrays released later, after
  `close()` or after the batch object is freed, still refill it within the same bound, and the
  pool is freed only when the last array from that batch has been released. A buffer goes back only when no
  array or view can reach it, so an array held forever is never reused. Fix 3(b), a
  caller-supplied `out=`, was not added. With the pool, releasing the previous arrays costs
  0.015 ms, and `out=` would still need the same writes.
- **Fix 4, home ranges.** Each worker first runs a fixed range of slots, then takes unclaimed
  slots from the other ranges. Measured negative result: an earlier version with purely static
  ranges gained on the phase harness (16 workers: 50.1k to 55.7k slot steps/s), but `a20_checks`
  B64 at 16 workers fell from 37.5k–38.6k to 28.7k–29.4k. When one slot's job was long, the
  other workers waited for it. With the stealing step, the batch gains at every worker count
  and in `a20_checks` (below). At 16 workers about 13–20% of slot jobs run outside their home
  range, against 85% before.
- **Fix 5, easy wins.** Standard vocabulary lookups use tables built once, and the group layout
  is built once per capacities and worker thread. Card rows are pre-sized, and Run RNG draws feed
  their HMAC input in parts instead of building a message buffer. The scalar encode falls from
  17.6 to 13.7 µs per step (pass 2 of the phase harness). The `RunState` clones on read-only
  room probes (estimated at up to 1.5 µs) were not changed: no clear site was found without a
  new profile.
- **Fix 2(a)** is documentation: [batching](batching.md#slots-seeds-and-workers) tells native
  callers to drop the previous rows before the next step. **Fix 2(b)**, a native recycle API, is
  deferred (Bead `spire-qu9.5`). **Fix 6** (lockstep wait) is measured only. With the fix 5
  build, plain threads still run 155k–165k turns/s without a barrier and 101k–103k with one at
  16 threads (`m38_component_scaling`, three runs).

**Measured** on the M38a host, base `fc12db1e` against the M38b code, interleaved runs on a
quiet machine (three runs, two for P4/P5, N1r and N2; five for `a20_checks`). The figures are
ranges, or medians where stated. Python: `run_vector_phase_timing.py`, 64 slots, A20, 1,000
calls, `record_replay` off. Native: `m37_phase_timing`, 64 slots, NextStep, no stacking.

| Measure | Before | After |
| --- | ---: | ---: |
| Python `NativeRunBatch.step`, 1 worker, ms/call | 8.09 | 4.60–5.00 |
| Python `NativeRunBatch.step`, 16 workers, ms/call | 4.90–5.03 | 1.25–1.31 |
| Releasing the previous arrays, 16 workers, ms/call | 0.98–1.05 | 0.015 |
| Python `RunVectorEnv.step`, 1 / 16 workers, ms/call | 8.15 / 5.16 | 4.67 / 1.31 |
| `run_vector_rollout.py`-style loop, 1 / 16 workers, slot steps/s | 7.5k / 11.4k | 12.7k / 38.0k |
| Native batch, `record_replay` off, 1 / 4 / 8 / 16 / 32 workers, median slot steps/s | 12.7k / 33.4k / 43.7k / 49.9k / 44.5k | 13.7k / 34.8k / 48.3k / 54.0k / 45.7k |
| Native batch, rows dropped early, 16 / 32 workers, median | 66.4k / 92.3k | 88.1k / 97.4k |
| Native batch, `record_replay` on, 1 / 16 workers, median | 12.8k / 48.3k | 13.6k / 52.7k |
| Native batch, 128 slots, 16 workers, median | 38.1k | 38.6k |

With the pool, the glibc no-trim environment no longer matters. The base binding gained about 25%
from it at 16 workers (4.98–5.00 to 3.82–3.86 ms per call). A development build with fixes 1, 3(a)
and 5 measured 1.49–1.51 ms per call with it and 1.51–1.58 without. A Python caller that drops the previous `obs` before the next step
gets about 0.71–0.84 ms per call at 16 workers (about 4.42 at 1 worker). The released arrays
then feed the pool for the next call.

**Remaining limits.** The dense observation is still about 764 KB per slot, written in full every
step. At 16 workers the native batch still gains about 1.6 times when callers drop rows early,
and the per-call barrier remains. The opt-in compact observation (Bead `spire-qu9.4`) is
the next step, and a native recycle API is deferred to `spire-qu9.5`. Raw JSON and receipts are
kept with the M38b evidence, outside the repository.

## After M38e

M38e (Bead `spire-qu9.6`) adds the opt-in compact Standard observation
([Standard Run API](standard-run-api.md#compact-observation-m38e)): the same public information with fewer rows
and `int16`/`int32` values, 136,842 bytes of arrays per slot instead of 763,720. Native rows stay `i64` (280,438 bytes
per slot); the Python batch encodes straight into typed arrays.

**Measured** on the M38a host, 2026-11-12, base `aafb80ea` against the M38e candidate, interleaved runs on a quiet
machine (three runs; two for N2 and P2; five for dense `a20_checks`). Medians unless a range is given. Native:
`m37_phase_timing`, 64 slots, A20, NextStep, `record_replay` off, no stacking unless stated. Python:
`run_vector_phase_timing.py`, 64 slots, A20, 1,000 calls, `record_replay` off. The "Compact ÷ dense" column is the
median of per-run paired ratios (each compact run divided by the dense run of the same round), so it can differ a little
from the ratio of the medians shown beside it.

| Measure | Dense (candidate) | Compact | Compact ÷ dense |
| --- | ---: | ---: | ---: |
| Native batch, rows kept, 1 / 4 / 8 / 16 / 32 workers, slot steps/s | 13.5k / 35.1k / 42.8k / 52.0k / 47.1k | 17.8k / 59.9k / 93.2k / 137.7k / 143.1k | 2.65× at 16, 3.06× at 32 |
| Native batch, rows dropped early, 16 / 32 workers | 90.3k / 93.2k | 143.1k / 157.1k | 1.49× / 1.70× |
| Native batch into typed stacked columns (`M37_STACK=slab`, as the binding), 1 / 16 / 32 workers | 14.0k / 81.3k / 91.4k | 21.1k / 131.1k / 131.2k | 1.50× / 1.61× / 1.36× |
| Native batch, 128 / 256 slots, 16 workers | 37.9k / 33.5k | 123.3k / 88.0k | 3.25× / 2.62× |
| `a20_checks` B64 (`record_replay` on), 1 / 4 / 16 workers | 12.8k / 31.7k / 39.1k | 17.6k / 54.5k / 92.5k | 1.38× / 1.72× / 2.37× |
| `a20_checks` 256 slots, RSS | 647 MB | 267 MB | |
| Python `NativeRunBatch.step`, 1 / 16 workers, ms/call | 4.83 / 1.34 | 3.26 / 0.58 | 0.68 / 0.43 |
| Python `RunVectorEnv.step`, 1 / 16 workers, ms/call | 4.82 / 1.40 | 3.36 / 0.62 | |
| Python, drop the previous `obs` first, 16 workers, ms/call | 0.76–1.03 | 0.56–0.58 | |
| Speed checks, 1 / 16 workers, slot steps/s: native (`a20_checks --speed-check`) | 13.1k / 39.8k | 17.9k / 92.1k | 1.36× / 2.31× |
| Speed checks, 1 / 16 workers, slot steps/s: Python (`speed_check.py`) | 13.3k / 46.8k | 19.5k / 106.8k | 1.49× / 2.28× |

**Dense.** The dense observation's bytes are unchanged (dense digests, frozen replay and a packaging differential
equal the base). Native dense speed is unchanged within noise: `a20_checks` B64 at 1 / 4 / 16 workers +0.7% / +1.5% /
−2.6% (medians against the base), the native speed check −2.4% / +0.1%, the phase harness equal or faster. The dense
Python batch step is about 3–5% slower than after M38b: `speed_check.py` 3.3% / 3.8% fewer slot steps/s at 1 / 16
workers and `NativeRunBatch.step` 5.7% / 2.6% slower (medians of four alternating runs with `OPENBLAS_NUM_THREADS=1`,
three valid runs for the base binding loop; without it 4.2% / 4.1% and 3.5%). The dense scalar
`NativeRunSession.step` is 0.2–1.1% slower (1,500 steps, 4 characters × A0/A20, two sweeps), and session
construction, `exact_fork` and `from_record` are at the base within noise. A bisect puts it in the engine, not the binding: the M38b binding built on the
M38e engine is as slow. A profile shows no hotspot (the encoder's own time is equal or lower; one helper lost its
inlining), so the cause is diffuse, likely the encoder's code generation now that it also writes typed values. A
dense-only writer is a possible follow-up.

**Reading.** At 16 workers compact roughly halves to thirds the per-step cost wherever the dense observation's bytes
dominated: the native batch with rows kept, larger batches and the Python step. Where M38b had already removed most of
the memory pressure (rows dropped early, or encoded straight into reused stacked columns), the gain is smaller, about
1.4–1.7 times. Compact also cuts the batch's memory: 256 slots use about 267 MB instead of 647 MB. At one worker the
gain is 1.4–1.5 times, the cost of writing fewer bytes. The encode's own work (rules and public view) is unchanged.

## Reproduce

```sh
# Throughput (timing build)
cargo run --release --locked --offline -p spire-env --example a20_checks -- --out target/a20-checks.json

# Phase timing (JSON on stdout, or M37_OUT=path)
cargo test --release --locked --offline -p spire-env m37_phase_timing -- --ignored --nocapture
M37_PART=batch M37_RECORD=off M37_STACK=off M37_WORKERS=16 M37_BATCH_STEPS=300 \
  cargo test --release --locked --offline -p spire-env m37_phase_timing -- --ignored --nocapture

# Python side (needs a source-matched extension; check _native.__file__ and build_info() first)
uv run --locked --no-sync python examples/run_vector_phase_timing.py --workers 16 --steps 2000

# Profiles (separate target dir; frame pointers and line tables)
CARGO_PROFILE_RELEASE_DEBUG=line-tables-only RUSTFLAGS="-C force-frame-pointers=yes" \
  cargo test --release --locked --offline -p spire-env --lib --no-run
M37_PART=scalar M37_RECORD=off perf record -F 4999 -g -o scalar.data \
  <test binary> --ignored --exact tests::m37_phase_timing::m37_phase_timing
perf report -i scalar.data --children --sort sym --stdio
perf stat -e cycles,instructions,page-faults,context-switches <test binary> ...
```

## Limits

These are single-host measurements without core pinning or frequency control, on one
random A20 workload that ends in Act 1. Late-game states (bigger decks, more powers)
weigh card preparation and encoding differently. The decomposed scalar pass re-runs the
session's steps in the same order but is not the session itself, so about 10 µs per step
stays unattributed. Shares from the frame-pointer build carry its overhead. The native
and Python batch workloads use different partitions and seeds. DRAM fill counts are a
lower bound on traffic, not a bandwidth measurement, so memory saturation was not
measured. The raw JSON, `perf` summaries and counter output are retained with the M37a
evidence, outside the repository; raw `perf.data` files are not.
