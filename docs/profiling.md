# Profiling the Standard A20 workload (M37a)

This report explains where the time goes in the M36 Standard A20 measurements
([performance](performance.md)). It covers the native scalar run, native batch
scaling and the Python vector path, and it proposes fixes for a later milestone
(M37b). It is diagnostic only. No engine code changed for it. Gameplay, IDs,
fingerprints and observations are unchanged, and the numbers below are not
thresholds or promises. M37b has since implemented five of the six fixes. The "After
M37b" columns and [After M37b](#after-m37b) give the re-measured numbers; the rest of the
report describes v1.0.0.

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
