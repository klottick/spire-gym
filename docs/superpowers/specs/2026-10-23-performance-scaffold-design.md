# Spire Gym performance scaffold

Date: 2026-10-23
Status: written specification approved by the user on 2026-10-23 ("I approve").

## Intent and boundaries

Evaluate simulator speed and expose the performance impact of later changes, not
merely time today's tests. The scaffold follows the existing full design and
36-milestone roadmap: all four fully unlocked characters, Ascension 0–20, Acts
1–4 and the Heart, with A20 the primary final workload. It must remain useful as
those features arrive without treating planned coverage as implemented.

This work adds measurement, comparison and focused tests. It does not implement
gameplay, optimize mechanics speculatively, build a learner, reopen historical
verification work or claim original-runtime fidelity. No new dependencies,
services, dashboards, commits, publication or package installations are included.

User-approved setup is a dedicated worktree based on accepted commit `5b1cd19`,
with separate build outputs. Preserve the active native controller's M7 work.

## Approach

Use a small versioned catalog, an opt-in Rust benchmark example and a Python
standard-library driver/comparator. Reuse current public fixtures and policies.
Scattered ignored timing tests would be cheap initially but make longitudinal
comparison inconsistent; a benchmark framework or service adds unnecessary scope.
The selected middle ground provides one result contract without introducing a
general-purpose performance platform or runtime instrumentation.

Proposed locations:

- `performance/catalog.json`: workload contracts and future coverage families.
- `crates/spire-core/examples/performance.rs` and adjacent support: native cases.
- `crates/spire-core/tests/performance.rs`: deterministic harness/behavior checks.
- `performance/run.py` and `performance/compare.py`: offline invocation, metadata
  and structured result comparison; no Python simulation.
- `tests/performance/`: standard-library tests for result validation/comparison.
- `docs/performance.md`: commands, baseline findings and extension instructions.

Use existing `serde_json` and SHA-256 support in Rust; Python uses only its
standard library. Keep benchmark support outside production engine modules.
Existing `/tools/` is ignored, so do not place public tooling there.

## Full-design coverage map

The catalog distinguishes executable cases from planned families. Planned entries
carry an unmet capability and roadmap reference; they have no timing samples and
are not passing tests. Listing coverage is not a count-based completion claim.

| Family | Initial executable coverage | Future extension |
|---|---|---|
| Combat transitions | M1 authored Ironclad A0 combat | All character mechanics, multi-actor fights, elites and bosses |
| Run progression | M5 captured Act1; M6 captured Act2 prefix | Complete Acts1–4/Heart, genuine loss and no-key completion |
| Decisions and public views | Public-boundary mode for initial cases | Focused menus, observations, selections and numeric encoding |
| Rooms and economy | Included in declared M5/M6 paths | Isolated shops, events, rewards, rest, keys and settlement |
| RNG and replay | Deterministic action replay in initial cases | Dedicated RNG, reset and replay-cost cases |
| Native batching | Planned; existing legacy combat slice is separately identifiable | Mixed characters, A0/A20, serial/workers, partial reset and finals |
| Python/Gymnasium | Planned; not an alias for native Run throughput | Rebuilt raw bindings, single/vector facade and export overhead |
| Resources | Bounded workloads; no memory metric initially | Separately measured peak memory/allocation/capacity workloads |

Describe character/ascension/act capabilities explicitly rather than generating
thousands of empty combinations. Add real runnable cases when their product
capabilities exist. Keep old workload versions available where supported; never
silently repurpose a captured-prefix identifier as a complete-act benchmark.

## Initial executable workloads

Start with seed7 cases using existing example support:

1. `combat.ironclad-a0.jaw-worm`: M1 authored combat, actual victory and replay.
2. `run.ironclad-a0.act1-guardian`: M5 captured Act1, alive/nonterminal Act2Ready,
   expected reward17 and 212 accepted actions.
3. `run.ironclad-a0.act2-prefix-champ`: M6 from normal start through floor26,
   alive/nonterminal Act2PrefixComplete, no candidates, reward26, 399 accepted
   actions and final HP49. This is not an Act2 boss completion.

Each contract has a version and names its exact fixture/profile, creation inputs,
seed, caller policy, action limit and behavioral endpoint. Reuse existing
mechanics/replay tests; add only harness-specific guards. Counts accompany actual
outcome/resource assertions and never replace mechanics evidence.

Measure two separate boundaries per workload:

- **Native transitions:** generate the public-policy action script and construct
  each fresh initial state outside timing. Time accepted `Combat::step_full` or
  `Run::step` calls, consume returned results, and accumulate rewards. No external
  policy, formatting, snapshots, trace retention or public reads in this interval.
  Work inherently performed inside these public APIs remains included.
- **Public decision loop:** replay the same actions while obtaining the public
  observations/menu required at each boundary and maps at map decisions. Include
  that interface work, not external policy selection or retained trace creation.
  Reuse public policies only for untimed script preparation, avoiding duplicate
  heuristic implementations. Label this boundary explicitly; it is not raw
  effect-resolver timing or a Python/Gymnasium measurement.

Run identical untimed preflight replays for both modes and compare public
boundaries, reward deltas and endpoints. Time only workloads that pass preflight;
an invalid action, operational fault or unfinished budget invalidates the result,
never becomes a fast loss or a skipped successful case.

## Sampling and result contract

Use release builds, monotonic timers and black-box consumption. Initial defaults:
one warmup sample, seven measured samples and ten episodes per sample. Overrides
permit 1–100 measured samples, 1–1000 episodes per sample and 0–100 warmup samples.
Each episode retains the existing maximum4096-action bound.
Run cases serially; future worker sweeps are explicit workloads, not concurrent
benchmark jobs. Reject malformed or unbounded input before running cases.

Exclude compilation, script preparation, initialization, validation and output
formatting from samples. Measure short episodes in batches of independently
prepared states to reduce timer overhead; keep state destruction outside timing.
Emit raw elapsed nanoseconds, episodes and accepted-action counts per sample.
Summarize median elapsed time, median actions/s and min/max; do not claim a p95
latency estimate from seven samples. Record setup/build durations separately.

Write a versioned JSON result containing:

- Suite schema version and catalog contract digest.
- Workload ID/version, boundary, content profile, character, ascension, seed,
  batch/worker configuration and sampling parameters.
- Canonical creation-input and ordered-action digests plus expected behavioral
  contract version; never use unstable Debug formatting as workload identity.
- Observed outcome/resource summary and correctness-preflight status.
- Raw samples and denominators, with derived summaries recalculable from them.
- Git revision and dirty status/fingerprint; compiler/target, build profile and
  effective build flags; machine/OS/CPU identity and operator-provided runner ID.

The driver records environment/build metadata and invokes the already-built
native binary outside sampling. Do not infer the executable revision from a
possibly stale binary: build it from the selected source with locked/offline Cargo
and an isolated target directory, checking successful completion before execution.
Record metadata/provenance separately from compatibility keys so an engine source
change is allowed to be compared, rather than automatically rejected as different.
Likewise, adding a catalog entry changes the suite digest but does not invalidate
unchanged per-workload contracts.
Do not record credentials, private game material or entire environment variables.

## Baselines and comparison

Save independent result files; baseline replacement is always an explicit action.
The comparator validates result schemas, finite positive durations, counts,
sample consistency and successful behavioral checks before calculating deltas.
Report per-case elapsed/action changes, actions/s, and sample spread, not one
aggregate score hiding regressions in another subsystem.

Classify rows as comparable, added, missing or incompatible. Compatibility
requires matching workload version, actual input/action digests, behavioral
contract, timing boundary, denominators/configuration, compiler/build settings and
runner/machine identity. Unknown identity is not silently comparable. Report
changed outcomes, action scripts, fixtures or APIs as incompatible; require a new
version/rebaseline after reviewing intentional semantic changes. An unchanged
action count alone does not prove comparable work.

Default comparison is diagnostic. An explicit regression-budget option can fail
on a configurable percentage increase in median nanoseconds per accepted action
for compatible rows. Missing/incompatible rows are visible; strict mode fails
instead of silently dropping them. Do not describe threshold crossings as
statistical significance, or impose shared-host wall-clock assertions on ordinary
correctness tests. New cases appear as added coverage, not improvement over zero.

Keep generated local results under ignored `target/performance-results/` by
default, allowing an explicit output path. Publish only a small qualified baseline
summary in documentation; no automatic baseline commits or dashboard storage.

## Tests and acceptance

Normal tests stay fast and deterministic. Use authored sample result files for
comparator tests: identical samples, known slowdown/speedup, malformed/zero/nonfinite
values, wrong counts, changed work/boundary/host, missing and new cases, and strict
threshold exit behavior. Verify that planned families cannot produce successful
timing rows. No sleeps, network setup or arbitrary time-limit assertions.

Rust harness tests protect accepted-action accounting, fresh-state replay, workload
endpoints and parity between the two measurement modes. Existing M1/M5/M6
correctness tests remain unmodified unless a concrete harness defect requires it.
Opt-in measurements exercise actual optimized simulator work, emit valid JSON and
compare against themselves successfully; a synthetic controlled slowdown verifies
the comparison failure path without changing gameplay or sleeping in tests.

Deliver a measured baseline for the three declared workloads, candidate
bottlenecks labeled measured versus source-inspected, commands to reproduce and
compare results, and instructions for registering a later workload. Do not claim
the repo is blazing fast solely because the scaffold runs, or extrapolate these
IroncladA0 samples to A20, full runs, batching or Python performance.

## Evidence and next approval

Fresh isolated pre-change release checks: M1 5 passed, M5 4 passed, M6 6 passed;
zero failures. Cargo reported build12.02s and test execution0.00s/0.02s/0.04s,
with six existing dead-code warnings. These are correctness-run diagnostics, not
throughput results or a whole-workspace acceptance claim. Seven known historical
core draft failures remain outside this focused run.

The user approved the in-chat architecture, isolated build setup and this written
specification. Implementation-plan review and execution-method selection precede
scaffold code. No commit authority is inferred. M7 planning/product ownership
remains separate.
