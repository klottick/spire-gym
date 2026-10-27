# Measuring simulator speed

The performance scaffold follows the complete four-character, A0–20, Acts1–4/
Heart design, with A20 the eventual primary workload. **Only the three declared
Ironclad A0 native workloads below are measured today.** A catalog entry marked
planned is not implemented coverage or a passing test. No gameplay optimizations,
new dependencies or original-runtime fidelity claims accompany this scaffold.

## Current measured baseline — 2026-10-26

Measured clean commit `a8261aa55e64d9c013f418b37711060003ca50b4` (accepted
M14 source) with the existing three-workload harness. These measure the older
explicit Ironclad profiles on that source, not Silent or a complete Heart run.

Host: AMD Ryzen 9 9950X, 16 cores/32 hardware threads, 59.5 GiB RAM,
x86_64 Linux 7.0.0-38-generic; runner `mustafar-9950x`. Rust 1.99.0
(`b940084d7`, LLVM 23.1.1), Cargo 1.99.0, target `x86_64-unknown-linux-gnu`.
The `spire-core` example uses its default features, with no Python extension.
The driver fixes release opt-level 3, LTO off, 16 codegen units, no debug info,
assertions or overflow checks, unwind panics, and no incremental compilation;
it clears inherited Rust flags. Compilation used two jobs.

Each row uses seed 7, one worker, batch size 1, seven measured samples of 100
independently initialized episodes, and one discarded warmup sample of 100
episodes. Episodes are replayed serially; 100 iterations is not a native batch.
Rates below are median accepted actions/s; parentheses give sample minimum–maximum.

| Workload (version) | Accepted actions/episode | Native actions/s (range) | Public-loop actions/s (range) |
| --- | ---: | ---: | ---: |
| Authored Jaw Worm combat (v1) | 7 | 388,411 (385,786–390,615) | 256,972 (256,518–258,476) |
| Captured Act 1 → Act2Ready (v1) | 212 | 173,434 (168,680–173,691) | 130,012 (129,546–130,638) |
| Captured Act 2 prefix → floor 26 (v2) | 397 | 147,372 (146,818–147,720) | 109,523 (109,146–109,932) |

Per sample the denominators are 700, 21,200 and 39,700 accepted actions,
respectively. The endpoints are Jaw Worm Victory; Act2Ready with reward 17 and
95 HP; and the nonterminal Act2PrefixComplete boundary at floor 26 with reward
26 and 91 HP. The last workload includes Act 2 ordinary fights, an elite and
room/economy work, ending before the Act 2 boss. Its "Champ" catalog name
identifies the selected map boss icon, not a measured Champ fight.

Native mode times `Combat::step_full` / `Run::step`. Public-loop mode additionally
obtains public observations/menus before each action and public maps at map
decisions. Compilation, initialization, policy-based script generation,
checkpoint validation, JSON output and complete-state destruction are excluded.
Every measured episode is checked against the expected endpoint afterward.
These are replay-throughput figures, not end-to-end agent episode rates,
Python/Gymnasium timings, batch scaling, policy strength or fidelity evidence.

The immediate repeat had comparable identities for all six rows; median
ns/action changes ranged from −0.15% to +1.12%. This describes two collections,
not a guaranteed noise bound or statistical significance. Collection was serial;
no core pinning or CPU-frequency control was applied. A process snapshot during
collection showed no competing compiler job. First build: 15.238 s; repeat build
check: 0.039 s, both outside runtime timing. Do not infer an optimization or
regression from the historical 2026-10-23 numbers below: source, sampling and the
Act 2 workload version differ.

The result's source fingerprint is
`fca0514edf94a7f69d3074fe44bee14bc1538bb287728b02fe4a2da737f54ffe`.
It hashes the driver-defined Rust/harness/manifests/toolchain inputs, not just a
Git revision. Exact source export, binary hash, commands, raw nanoseconds,
operation counts and machine/build receipts are retained separately from the
managed worktree. To generate the same two result documents from this revision,
using an existing Python 3.12 interpreter and cached locked dependencies:

```sh
python3.12 -B -m performance.run --runner-id my-controlled-host \
  --samples 7 --iterations 100 --warmup 1 \
  --target-dir target/readme-measurement --output target/readme-first.json
python3.12 -B -m performance.run --runner-id my-controlled-host \
  --samples 7 --iterations 100 --warmup 1 \
  --target-dir target/readme-measurement --output target/readme-repeat.json
python3.12 -B -m performance.compare target/readme-first.json target/readme-repeat.json
```

The collection used the same arguments with host-specific existing interpreter,
runner, target and output paths. Results embed the actual source revision,
fingerprint, sampling, full catalog, compiler/build identity and raw samples.
Use a fresh output filename for another collection. Documentation changes do not
change the benchmark source fingerprint; gameplay/harness changes may do so.

## Run and compare

Use the already-installed pinned Python3.12 interpreter; these tools require only
its standard library, not a rebuilt Python extension or Gymnasium installation.
From the repository root:

```sh
PY312=/home/klottick/.local/share/uv/python/cpython-3.12.15-linux-x86_64-gnu/bin/python3.12

# Ordinary deterministic harness/comparison tests; no speed thresholds.
"$PY312" -B -m unittest discover -s tests/performance -v
CARGO_TARGET_DIR=target/performance-baseline CARGO_BUILD_JOBS=2 RUSTUP_OFFLINE=1 \
  cargo test --release --locked --offline -p spire-core --test performance

# Collection builds first, then runs the actual optimized executable serially.
"$PY312" -B -m performance.run --runner-id my-controlled-host \
  --output target/performance-results/baseline.json
"$PY312" -B -m performance.run --runner-id my-controlled-host \
  --output target/performance-results/candidate.json

# Diagnostic comparison, followed by an optional controlled-host regression gate.
"$PY312" -B -m performance.compare \
  target/performance-results/baseline.json target/performance-results/candidate.json
"$PY312" -B -m performance.compare \
  target/performance-results/baseline.json target/performance-results/candidate.json \
  --strict --max-regression-percent 10
```

Change the interpreter path for a different host's existing Python3.12 install.
No toolchain, package or extension is installed automatically. Rust is pinned by
`rust-toolchain.toml`; missing cached dependencies cause an offline failure.

Choose a runner ID that identifies the same operator-controlled host/configuration,
not a new label for each revision. Run benchmarks without competing builds or
other heavy jobs. This driver does not reserve cores or control CPU frequency;
host contention can still invalidate conclusions.

Collection options: `--case ID`, `--samples 1..100`, `--iterations 1..1000`,
`--warmup 0..100`, `--target-dir PATH`, `--output PATH`, `--overwrite`.
Defaults are seven measured samples, ten independently prepared episodes/sample
and one discarded warmup. Seed7 and the4096-action episode bound are workload
contracts, not overrides. Existing output files are preserved unless explicitly
overwritten. Omit `--output` to create a new timestamped file.

Build outputs live beneath `target/performance-baseline/collected/<source SHA>/`.
This is intentionally separate from other sessions' targets and protects against
stale binaries after timestamp-preserving source changes. A source change may
require a cold build. Build time is recorded separately, never divided into
simulation throughput. The controlled host release profile uses opt-level3,
LTOoff,16codegen units, no debug/assertions/overflow checks and unwind panics;
inherited rustflags are cleared for this command only. No persistent Cargo config
is changed. Custom compiler/target/wrapper environment overrides reject.

The native example also supports `--list` (including planned families) and the
sampling/case options. Direct native output lacks machine/build provenance and
cannot be used by the baseline comparator; use the Python driver for comparisons.
Debug builds may list coverage but reject timing.

## What the numbers mean

Each workload has two distinct timing boundaries:

- **Native transitions:** replay a precomputed legal script through the actual
  `Combat::step_full`/`Run::step` APIs. Their internal allocation/menu work remains
  included; this is not an allocation-free effect-resolver microbenchmark.
- **Public decision loop:** replay the same script with public observations and
  menus before actions, plus public maps at map decisions. No external heuristic
  policy selection or retained trace construction is timed.

Fresh initialization, policy-based script generation, behavior/preflight checks,
JSON formatting and complete-state destruction happen outside sample timing.
Returned transition/public values are consumed; accepted-action counts exclude
warmup, setup, rejected attempts and fictional effect counts. Samples retain raw
nanoseconds/episode/action denominators; displayed medians and min/max are
recalculated, not trusted from a result's precomputed summary. Seven samples do
not establish p95 latency or statistical significance.

Before timing, both modes must reproduce typed public checkpoints, rewards and
endpoints. Every measured episode is checked afterward. Invalid actions,
operational faults, changed endpoints and exhausted budgets fail collection,
rather than becoming fast losses or silently skipped cases.

## Initial measured baseline — 2026-10-23

Host: AMD Ryzen9 9950X16-Core Processor, x86_64, Linux7.0.0-38-generic;
runner `local-cpu`, pinned Rust1.99.0, controlled release profile above. Source
baseline is accepted `5b1cd19` plus this uncommitted benchmark scaffold, not a
published optimized simulator revision. Raw results are local generated evidence
at `target/performance-results/baseline.json` and `repeat.json`; each records its
exact source fingerprint, machine/build identity and catalog snapshot.

| Workload | Accepted actions/episode | Native actions/s | Public-loop actions/s | Native ns/action min–max | Public ns/action min–max |
|---|---:|---:|---:|---:|---:|
| M1 authored Jaw Worm |7|563,911|342,057|1,742–1,966|2,912–3,590|
| M5 captured Act1 Guardian |212|227,004|167,749|4,358–4,776|5,949–6,010|
| M6 captured Act2 prefix/Champ icon |399|204,612|147,385|4,870–4,894|6,771–6,803|

These are median **accepted decisions per second**, not complete games or fights
per second. The initial build took10.98s outside timing. The immediate repeat's
median ns/action changes ranged−1.43%to+1.59%; that describes these two collections,
not a guaranteed noise bound. Self-comparison passed a strict0%budget. A separate
synthetic result with doubled durations failed a10%budget with exit1; real
baseline samples were not changed.

Endpoints: actual M1 victory; M5 nonterminal Act2Ready/reward17/HP95; M6
floor26/nonterminal Act2PrefixComplete/reward26/HP49 and no candidates. M6 includes
real Act2 ordinary/Leader elite/event/chest work, **not Act2 boss completion**.
Neither captured profile represents complete standard random content pools.

Public-loop median ns/action was approximately65%/35%/39%higher than native mode
for M1/M5/M6 respectively. This measures the extra public-boundary API work under
these workloads; it does not isolate one internal bottleneck. Source-inspected
candidates for later focused profiling include repeated card-effect preparation
in legal menus/visible-cost queries, public observation allocations, map-menu
searches, settlement copies and RNG HMAC work. These remain hypotheses about
internal cost, not measured attribution or justification to alter mechanics.

No initial timings measure Python/Gymnasium, native batching, memory/allocations,
other characters, A20 or complete Heart runs. At the time of this initial baseline, the Python/batching facade used the older
combat slice; those unmeasured interfaces must not be relabeled M6 Run speed.

## Valid comparisons and exit codes

The result embeds its catalog, so old baselines do not depend on today's checkout.
Compatibility requires matching per-workload version/contract, actual creation and
ordered-action hashes, timing boundary, observed endpoint, sampling/denominators,
build/compiler and machine/runner identity. Unknown identity is not comparable.
Changed git revisions/source fingerprints and newly added catalog families do
not by themselves invalidate unchanged work.

Rows are classified as comparable, added, missing or incompatible. There is no
single aggregate score hiding a subsystem regression. Intentional gameplay,
fixture/policy or endpoint changes require an explicit reviewed workload version
and new baseline; an unchanged action count alone does not establish equal work.

The Mystic group-healing correction advances `run.ironclad-a0.act2-prefix-champ`
and its expected endpoint contract to **version 2**: the unchanged seed-7 fixture
and public policy now complete in 397 actions with 91 HP, rather than version 1's
399 actions/49 HP. The floor-26 boundary and reward 26 are unchanged. Keep the
historical version-1 measurements; they are incompatible with version 2, not a
measured speedup or regression. The 2026-10-26 section above supplies a version-2 baseline. The correctness fix
itself made no new timing claim; version-1 timings remain historical evidence.

- Exit0: valid diagnostic report, or passing requested checks.
- Exit1: requested slowdown budget exceeded, or `--strict` finds missing/
  incompatible rows. Added rows are reported, not compared against zero.
- Exit2: malformed data, I/O or invalid arguments. Collection errors also exit2.

Use `--strict` with a regression budget so incompatible/missing evidence cannot
be overlooked. The budget uses exact integer-derived median ns/action arithmetic;
an increase exactly equal to the budget passes. Threshold crossings are not
statistical significance. `--json` emits a machine-readable comparison report.

## Add later workloads without losing history

1. In [`performance/catalog.json`](../performance/catalog.json), add a unique ID,
   version, explicit content/character/ascension/creation contract, behavior
   expectations and milestone requirement. Keep unsupported families planned.
2. Add a narrow native adapter in
   `crates/spire-core/examples/performance_support/`, reusing an accepted public
   fixture/caller. Canonically encode actual input and ordered semantic actions;
   never hash unstable Debug output or implement another game resolver.
3. Add a deterministic behavior/preflight test in
   `crates/spire-core/tests/performance.rs`. Prove genuine endpoints/resources and
   accepted-action accounting; do not certify game mechanics with counts alone.
4. Register the executable case only when it runs. Keep current case IDs/versions
   intact wherever still supported; don't rename a prefix into a full act.
5. Collect a new result. Existing compatible rows remain comparable, while the
   newly measured case appears added. Review intentional semantic changes before
   explicitly replacing any baseline.

Coverage families reserve space for combat, complete runs, decisions/encoding,
rooms/economy, RNG/replay/reset, batching, Python and resource measurements. Future
batch adapters must distinguish batch size/workers from repeated episodes, and
test identical per-slot behavior across schedules. Future Python collection needs
an explicitly refreshed extension and separate timing boundaries; the current
driver does not install it. Memory needs its own metric contract, not guessed
numbers mixed into actions/s.

## Historical scaffold verification — 2026-10-23

These counts describe the original scaffold revision, not current source or a
fresh workspace run. The 2026-10-26 collection above performs the existing
per-workload preflight and endpoint checks; documentation edits do not establish
new gameplay acceptance.

The original focused release checks passed 24 Rust tests (9 harness checks plus
15 M1/M5/M6 checks), and unchanged `spire-env` passed 38 tests. The scaffold's
Python suite passed 22 tests. At that revision the broader core suite recorded
518 passes and the seven historical failures below. These are dated results,
not the current failure inventory; see the [profile contracts](milestones/) for
subsequent mechanics acceptance.

- `combat::tests::full_cards::artifact_prevents_one_poison_application_without_erasing_existing_poison`
- `combat::tests::full_cards::catalyst_adds_twice_the_current_poison_and_exhausts_but_absence_is_noop`
- `combat::tests::full_cards::catalyst_permanent_card_state_carries_its_exhaust_flag`
- `combat::tests::full_cards::poison_ticks_at_afflicted_enemy_start_ignoring_block_then_decrements_and_can_kill`
- `combat::tests::full_cards::silent_deadly_poison_applies_base_and_upgraded_amount_for_one_energy`
- `combat::tests::full_cards::silent_poisoned_stab_resolves_attack_before_base_or_upgraded_poison`
- `combat::tests::full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`
