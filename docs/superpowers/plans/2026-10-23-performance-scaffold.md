# Spire Gym Performance Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for recommended native execution, or superpowers:subagent-driven-development if the user selects it. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add reproducible speed measurements and valid before/after comparisons for current native workloads, with explicit extension space for the complete gym design.

**Architecture:** A versioned JSON catalog describes executable cases and planned coverage. An opt-in Rust example reuses public fixtures and emits raw samples; a standard-library Python driver records build/machine provenance, and a comparator validates compatibility before reporting changes. No production simulation code changes.

**Tech Stack:** Pinned Rust1.99.0, existing serde/serde_json/sha2 dependencies, CPython3.12.15 standard library, Cargo tests and unittest.

**Spec:** `docs/superpowers/specs/2026-10-23-performance-scaffold-design.md` (user-approved).

## Global Constraints

- Full target: all four fully unlocked characters, Ascension0–20, Acts1–4/Heart; A20 primary. Future benchmark coverage is explicitly planned, not passing.
- Initial executable cases: seed7 M1 Jaw Worm combat, M5 Guardian Act1, M6 Champ-icon Act2 prefix; no complete Act2/full-run/Python claim.
- Existing dependencies only; no installs, commits, pushes, cleanup, original execution or gameplay/fixture weakening.
- Work only in `gym-performance-scaffold-20261023`, baseline `5b1cd19`; do not touch the active native controller/M7 files or historical worktrees.
- Use `CARGO_TARGET_DIR=target/performance-baseline CARGO_BUILD_JOBS=2 RUSTUP_OFFLINE=1` and `--locked --offline`; compilation is outside timing.
- Defaults: 1 warmup, 7 measured samples, 10 episodes/sample. Ranges: warmup0–100, samples1–100, episodes1–1000; episode limit4096 actions.
- No timer thresholds in ordinary correctness tests; opt-in comparison budgets operate only on compatible workloads.
- Reuse existing public policies only for untimed script preparation. Keep public, native, batching and Python timing boundaries distinct.
- Use the already installed interpreter, without creating/installing a venv: `PY312=/home/klottick/.local/share/uv/python/cpython-3.12.15-linux-x86_64-gnu/bin/python3.12`.
- Authoritative task: `spire-3ar`; actor `opencode-speed-ses_e96b1b1b4ffeKOup0kwuyFEXIF`. Shared tracker only; check context/ping before writes and use normal ownership transitions.

## Review Focus

- A renamed/changed fixture, action stream or timing boundary must be incompatible even when action counts match; Task4 tests pin this.
- A planned/unregistered case, duplicate result row, nonfinite value or bool masquerading as an integer must never produce a successful benchmark; Tasks1/3/4 test this.
- A stale executable, failed build or source edit during collection must not be attributed to the selected revision; Task5 mocks those failures.
- Missing CPU/runner/build identity must not silently permit cross-machine comparison; Task4 tests explicit incompatibility.
- Rerunning a command must not silently replace baseline evidence or report unfinished/faulted episodes as fast completions; Tasks2/5 test this.

---

## File map and shared contracts

Create:

- `performance/__init__.py`: package description, no eager runtime work.
- `performance/catalog.json`: schema1, full-design target, three executable contracts and planned coverage families.
- `performance/results.py`: catalog/result validation and sample summaries.
- `performance/compare.py`: compatibility, per-case deltas and CLI exit policy.
- `performance/run.py`: build/invoke driver and sanitized provenance.
- `tests/performance/test_results.py`, `test_compare.py`, `test_run.py`: ordinary unittest coverage, no external packages.
- `crates/spire-core/examples/performance.rs`: bounded CLI and JSON output only.
- `crates/spire-core/examples/performance_support/{mod.rs,contract.rs,workloads.rs,sampling.rs}`: shared testable harness, separated by responsibility.
- `crates/spire-core/tests/performance.rs`: imports shared support using existing `#[path="../examples/..."]` convention.
- `docs/performance.md`: execution, baseline report, limitations and extension recipe.

Modify only `README.md` for a small performance-doc link and the approved spec/this plan for status. No Cargo/lockfile, production engine, existing fixture or existing test changes planned.

**Native sample document:** `schema="spire-gym-native-samples-v1"`, `catalog` (the versioned catalog snapshot), `catalog_sha256`, `sampling={warmup,samples,iterations}`, `cases=[...]`. Each case row has `id`, `version`, `contract_sha256`, `boundary`, `profile`, `character`, `ascension`, `seed`, `batch_size`, `workers`, `input_sha256`, `actions_sha256`, `accepted_per_episode`, `expected_contract_version`, `endpoint`, `preflight_passed`, `samples` and `preparation_ns`. A sample is `{elapsed_ns,episodes,accepted_actions}`. Boundary strings are `native-transitions` and `public-decision-loop`; native rows use batch_size/workers1 (iterations is episode repetition, not native batching). Validate sampled IDs/contracts against the embedded executable catalog, not whichever catalog happens to be in the current checkout; this preserves old-baseline readability while rejecting planned entries masquerading as measurements.

**Final result document:** `schema="spire-gym-performance-v1"`, native document fields, `provenance={git_revision,dirty,source_sha256,build_seconds}`, `machine={runner_id,cpu,os,arch}`, `build={rustc_vv,target,profile,flags,config_sha256}`. Keep provenance outside comparison keys. All raw counts/durations are JSON integers; timing durations are positive. Derived summaries are calculated from validated samples, never trusted from input.

**Catalog rows:** require unique `id`, positive `version`, status `executable` or `planned`, profile/character/ascension/seed, `expected_contract_version`, `creation`, `expected` and `requires`. Planned families may instead declare target character/ascension/act ranges and milestone references; they cannot appear in timing rows. Version/hash each executable contract separately; whole-catalog expansion must not invalidate unchanged cases.

## Task 1: Versioned catalog and result validation

**Files:** create `performance/{__init__.py,catalog.json,results.py}` and `tests/performance/test_results.py`.

**Interfaces:** produces `load_catalog(path: Path) -> dict`, `validate_result(document: dict) -> dict`, `summarize(samples: list[dict]) -> dict`; validation raises `ValueError`. Summaries contain median elapsed_ns, median ns/action, median actions/s and min/max ns/action.

- [ ] **1. Write failing unittest cases.** Pin three executable IDs from the spec, all four target characters, A0–20 and Heart/A20-primary coverage. Verify eight named coverage families have unmet capabilities/roadmap references where planned. Author minimal valid result fixtures in test support rather than running the simulator.
- [ ] **2. Add validator failure tests.** `test_rejects_duplicate_ids`, `test_planned_case_cannot_have_samples`, `test_rejects_bool_zero_negative_nonfinite_samples`, `test_counts_equal_episodes_times_actions`, `test_all_samples_and_rows_match_sampling`, `test_unknown_schema_rejects`. Assert `ValueError`, not a warning or successful empty result.
- [ ] **3. Run RED:** `$PY312 -B -m unittest discover -s tests/performance -p test_results.py -v`; expect missing module/implementation or failed catalog assertions.
- [ ] **4. Implement catalog and validators.** Catalog expected values: M5 reward17/actions212/Act2Ready/alive/nonterminal; M6 reward26/actions399/HP49/floor26/Act2PrefixComplete/no candidates/nonterminal; M1 Victory/dead enemy. Use strict integer checks (`type(value) is int`), duplicate-ID checks, finite positive samples and required SHA-256 strings. Bound counts exactly as global constraints. Native documents are validated before the driver wraps them; final results additionally require provenance/build/machine structure.
- [ ] **5. Pin arithmetic:** samples `(elapsed_ns=1000,accepted_actions=10)` and `(2000,10)` summarize to median150ns/action. Do not derive throughput by dividing total-suite wall time. Run the same test command GREEN.

## Task 2: Deterministic workload preparation and preflight

**Files:** create `performance_support/{mod.rs,contract.rs,workloads.rs}` and `crates/spire-core/tests/performance.rs`.

**Interfaces:** produces `CaseId`, `Boundary`, `PreparedCase`, `ReplayTotals`, `BenchState`; `prepare(id: CaseId) -> Result<PreparedCase,String>`, `fresh(case: &PreparedCase) -> Result<BenchState,String>`, `preflight(case: &PreparedCase) -> Result<serde_json::Value,String>`, `canonical_hash(value: &Value) -> String`. `PreparedCase` owns typed combat/run action scripts and expected checkpoints; policy work never occurs in `BenchState` stepping.

- [ ] **1. Write failing Rust tests:** `three_real_workloads_have_expected_endpoints`, `fresh_replays_match_prepared_public_boundaries_and_rewards`, `altered_or_truncated_script_is_not_completion`, `creation_and_ordered_action_changes_change_digest`, `canonical_objects_ignore_key_insertion_order`. M5 asserts212actions/Act2Ready/reward17; M6 asserts399actions/HP49/floor26/reward26/no candidates/nonterminal, real ordinary and Leader elite victories plus AncientWriting as existing tests require. M1 asserts actual Victory, enemyHP0 and empty terminal menu.
- [ ] **2. Run RED:** `cargo test -p spire-core --test performance --locked --offline` with the global environment; expect missing support or unimplemented preparation.
- [ ] **3. Prepare using existing support.** Import `m1_support`, `m5_support`, `m6_support` by sibling paths. M1: `fixture(EnemyId::JawWorm,7)` and `run_fight(...,true,4096,observer)` capture actions. M5: `run(7,Scenario::Guardian)`, `Caller::new(4096)`, `rollout`. M6: `prefix(7,vec!["Automaton","Collector"],4096)` and `start` for fresh states. Extract scripts/checkpoints outside timing, retaining typed equality comparisons rather than serializing all public game objects.
- [ ] **4. Implement canonical identity without changing production derives.** Encode action enums explicitly as tagged JSON arrays/objects: stable string tags plus numeric instance/target/choice/node/selection IDs; preserve array order and null versus absent target. Serialize actual M1 `CombatEntry` fields, card-local state and RNG domain/scope/key/cursor into canonical input; fail closed if future unsupported setup fields appear rather than formatting Debug. Run descriptors include exact config, experienced creation profile, room scenario, root/partition/ordinal and initial public resources/deck/map. Use `.as_str()` content IDs and explicit enum tags. Hash sorted-key compact UTF-8 JSON with existing SHA-256; Python uses `ensure_ascii=False,sort_keys=True,separators=(",",":")` for the same catalog/contract encoding. Never serialize original/private artifacts.
- [ ] **5. Implement preflight.** Replay each original-start script in both modes; compare public checkpoint values, rewards/maps and endpoints against the prepared reference outside measurement. Reject invalid actions, operational faults and unfinished endpoints. A final no-action/nonterminal cutoff is not a Heart victory. Run Task2 tests GREEN and rerun existing `--test m1_combat --test m5_act1 --test m6_act2` (15 previously passing tests).

## Task 3: Opt-in native sampling and bounded CLI

**Files:** create `performance_support/sampling.rs` and `examples/performance.rs`; extend `tests/performance.rs`.

**Interfaces:** produces `Sampling { warmup: usize, samples: usize, iterations: usize }`, `Sampling::validate(&self) -> Result<(),String>`, `measure(case: &PreparedCase,boundary: Boundary,options: Sampling) -> Result<Value,String>`, `run_suite(options: Sampling,case_filter: Option<&str>) -> Result<Value,String>`.

- [ ] **1. Write failing tests:** `sampling_defaults_and_boundaries` pins1/7/10 and all inclusive limits; rejects negative/overflow/zero measured values and maxima+1. `planned_or_unknown_filter_rejects` pins failure without timing success. `native_and_public_modes_have_same_work_and_outcome` compares input/action hashes, accepted actions and endpoint values, not elapsed durations. `warmups_are_not_samples` uses0warmup/1sample/1iteration and then1warmup/1sample/1iteration, asserting one measured row each and correct action denominator.
- [ ] **2. Run RED:** `cargo test -p spire-core --test performance --locked --offline`; expect missing sampling implementation.
- [ ] **3. Implement sampling boundaries.** Prepare all `iterations` fresh states before starting `Instant`; time one batch of complete scripted episodes, preserving states until after timer stop. Native mode only steps, consumes outputs and adds rewards. Public mode additionally calls `public_full`/`legal_actions_full` before each combat action, or `Run::decision` and `public_map` only at map decisions. No external policy, retained traces, snapshots, JSON, output or validation in the timed interval. Black-box each call's results; do not clone scripts in timing. Validate every resulting state/reward after timing. Warmups execute identical work but are excluded from samples.
- [ ] **4. Implement CLI.** Accept `--samples N --iterations N --warmup N --case ID --list`; defaults1/7/10 as above. No seed override in version1: catalog seed7 is a contract, not a convenience flag. Reject unknown/duplicate flags, missing values and debug timing builds; `--list` remains available in debug. Emit only native JSON to stdout and diagnostics to stderr; errors exit nonzero with no successful document. `--list` reports planned entries as planned, with no timing rows.
- [ ] **5. Run GREEN, then build outside measurements:** `cargo build --release --locked --offline -p spire-core --example performance`. Run `target/performance-baseline/release/examples/performance --samples 1 --iterations 1 --warmup 0`; parse stdout through Task1 validation and assert six rows (three workloads × two boundaries). Do not assert universal speed limits.

## Task 4: Compatible baseline comparison and optional slowdown gate

**Files:** create `performance/compare.py`, `tests/performance/test_compare.py`; extend `results.py` only where shared validation is needed.

**Interfaces:** consumes Task1 documents/summaries; produces `compare_results(baseline: dict,candidate: dict,*,max_regression_percent: float|None=None,strict: bool=False) -> dict` and `main(argv: list[str]|None=None) -> int`.

- [ ] **1. Write failing tests.** Identical inputs→0%; doubled elapsed times with unchanged work→+100%ns/action and−50%throughput; threshold10% fails+20% and passes exactly+10%. Reject negative/nonfinite budgets. Changed input/action hashes, version, expected contract, boundary, outcome, counts, sampling, host or build settings→incompatible. Changed git/source/catalog-wide digests alone remain comparable. Added and missing IDs are reported explicitly; strict mode fails missing/incompatible rows, not newly added cases. Duplicate `(id,boundary)` rows reject.
- [ ] **2. Run RED:** `$PY312 -B -m unittest discover -s tests/performance -p test_compare.py -v`.
- [ ] **3. Implement classification/arithmetic.** Match rows by `(id,boundary)`; compare per-row workload contracts and machine/build/sampling keys before arithmetic. A case whose boundary changed appears missing/added, never comparable; version differences retain the same key and report incompatible. Require nonempty runner/CPU/compiler/target identity; record missing/unknown identity as incompatible, not compatible because both are empty. Different source revisions are expected. Summarize raw samples via Task1; use exact integer-derived `fractions.Fraction` medians and `Decimal(str(budget))` for threshold decisions so exactly+10% passes a10%budget. Convert only display/JSON summaries to floats. Report median ns/action percentage and throughput plus min/max spread, no significance claim.
- [ ] **4. Implement CLI:** `$PY312 -B -m performance.compare BASELINE CANDIDATE [--max-regression-percent N] [--strict] [--json]`. Exit0 for valid diagnostic output or passing requested checks; exit1 for budget failure or strict missing/incompatible rows; exit2 for invalid input/I/O. No automatic writes/rebaseline. Console output says when zero rows are comparable.
- [ ] **5. Run GREEN.** Use synthetic result transformations, not sleeps or gameplay edits, to verify exact CLI exit codes and slowdown paths. Tests validate both entrypoint and library behavior.

## Task 5: Provenance driver, actual baseline and extension guide

**Files:** create `performance/run.py`, `tests/performance/test_run.py`, `docs/performance.md`; add one README link.

**Interfaces:** consumes Tasks1–4; produces `collect_metadata(root: Path,runner_id: str) -> dict`, `build_and_collect(root: Path,options: argparse.Namespace) -> dict`, `main(argv: list[str]|None=None) -> int`.

- [ ] **1. Write failing driver tests with mocked subprocesses/temp directories.** Pin offline/locked/release Cargo arguments and a unique target path, parsing `compiler-artifact` JSON for the actual `performance` executable rather than guessing/debug fallback. Build failure or missing/mismatched artifact→no result. Source fingerprint differs before/after build/collection→reject. Existing output path→refuse unless `--overwrite` is explicit. Verify native JSON/schema/catalog/action counts are validated before publication and malformed output never replaces evidence.
- [ ] **2. Add provenance tests.** Record full `rustc -vV`, target and actual artifact profile; record a narrow build-flags/config fingerprint rather than environment dumps. Missing runner ID/CPU/compiler info cannot satisfy comparison identity. Include tracked and nonignored untracked public Rust/catalog/manifest sources in a sorted path+bytes SHA-256; exclude target/private/ignored material. Test modification and untracked-source sensitivity plus target-output exclusion. Git revision/dirty/source digest are provenance, not workload compatibility keys.
- [ ] **3. Run RED:** `$PY312 -B -m unittest discover -s tests/performance -p test_run.py -v`.
- [ ] **4. Implement driver.** CLI: `--runner-id ID` required; optional `--output PATH --target-dir PATH --samples N --iterations N --warmup N --case ID --overwrite`. Validate all options first. Default target is `target/performance-baseline`, output a new timestamped JSON in `target/performance-results/`. Build synchronously using `cargo build --release --locked --offline -p spire-core --example performance --message-format=json`; bounded jobs2, offline rustup. Measure build seconds separately and invoke only the returned executable. Keep all status/build diagnostics on stderr. Atomically write a complete validated result, never truncate a prior baseline on failed collection.
- [ ] **5. Define reproducible build metadata.** For version1 use a controlled host release build: clear inherited encoded/plain rustflags, set explicit release opt-level3/debugfalse/debug-assertionsfalse/overflow-checksfalse/ltofalse/codegen-units16/panicunwind; pass these as command-scoped Cargo overrides and record them. Reject alternate targets/compilers or wrappers the driver cannot characterize. Record public Cargo configuration digests separately without printing contents or credentials. This is an isolated benchmark build, not a persistent project/user configuration change. Capture CPU model, OS/kernel and architecture; runner ID identifies the operator-controlled test host, not a claim of exclusive CPU access.
- [ ] **6. Run tests GREEN and focused release verification:** all `tests/performance` unittests; `cargo test --release --locked --offline -p spire-core --test performance --test m1_combat --test m5_act1 --test m6_act2 -- --test-threads=1`. Run `cargo test --release --locked --offline -p spire-env` as an unchanged consumer check, not a benchmark. No Python extension install or full-suite-green claim.
- [ ] **7. Collect baseline and repeat serially:** `$PY312 -B -m performance.run --runner-id local-cpu --output target/performance-results/baseline.json`, then the same command with `repeat.json`. Compare baseline to itself with `--strict --max-regression-percent 0` (must pass), then compare the repeat diagnostically. Preserve both actual sample files; report spread and host contention caveats. Check failure behavior by doubling sample durations in a separate synthetic JSON file and expecting exit1 with a10%budget. Do not rewrite real baseline samples.
- [ ] **8. Write documentation from actual results.** Include per-case median throughput/spread, native versus public-boundary differences, precise IroncladA0/captured-content limits, build/setup exclusions and known seven historical draft failures. Source-inspected menu/observation/RNG/map/settlement candidates remain hypotheses unless these measurements support a narrower claim; aggregate differences do not identify an internal bottleneck. Link full-design coverage families and show adding one future workload/version plus expected-state test and comparison keys. Explain that changed semantics require reviewed rebaseline, not silently accepted speedups. Mention Python/batching/memory as not measured initially.
- [ ] **9. Final review and handoff.** Inspect diff for engine/lockfile/fixture changes (expect none), validate docs commands, run whitespace checks, and request one independent whole-change review in its own newly materialized worktree and unique target. No per-test review swarm or commit step. Address actual findings, rerun affected checks and report exact evidence/remaining limitations. Update only this task's state; leave M7 ownership untouched.

## Self-review and execution handoff

Coverage: catalog/future families→Task1; canonical workload/behavior→Task2; boundaries/samples/CLI→Task3; compatibility/budgets→Task4; source/build attribution, actual measurements and extension docs→Task5. All five Review Focus conditions have named owning tests. No undefined production API or new dependency is required; current public types lack Serialize, so Task2 explicitly owns narrow benchmark encoding.

The user approved this plan and native implementation with one isolated final reviewer ("Yes"). Execution evidence and rulings are in the plan's dedicated ignored ledger, with measured results/limits in `docs/performance.md`. No commits are authorized.
