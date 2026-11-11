# Development setup

See [repository scope](scope.md) for the current delivery path: a usable headless
gym with a Rust simulator, native batching and thin Python/Gymnasium interfaces
for external ML consumers. Models, training, learner checkpoints and experiments
are outside this repo. No playable client or rendering is part of the product.

Use the [work-status history archive](handoffs/2026-11-11-work-status-history.md)
for dated coordination evidence and the [milestone index](milestones/README.md)
for current merge status. Each finite profile records its own evidence and limits.
Gym66 is the supplied-entry Ironclad A0 surface; M35 added shared batching and
Gymnasium facades. Check Beads and the current primary session for live ownership.
A native source result, a verified extension and Python/Gym behavior are separate
evidence. Models and learners remain external; scoped checks establish neither
original-runtime validation nor a full-workspace-green claim.

## Agent lane selection and artifact identity

Choose the narrowest existing lane for the changed surface:

| Work | Command | Limit |
| --- | --- | --- |
| Rust core, environment, or CLI | `cargo test --workspace --locked --offline` | Native Rust only; default features do not build the Python extension. Use a profile's focused command when available. |
| Gym/Python facade | `uv run --locked --no-sync python -B -m pytest tests/python/env tests/python/test_ironclad_training.py tests/python/test_combat_policy_view.py -q` | Requires an already provisioned Gym environment and a source-matched extension; `tests/python/env` includes `test_build_identity.py`. |
| Python combat reference / legacy comparator | `uv run --locked --no-sync python -B -m pytest tests/python/combat -q` | Separate compatibility lane. Known legacy comparator mismatches are baseline-reported in [work-status history](handoffs/2026-11-11-work-status-history.md) and [M9 limits](milestones/m9-act3.md#verification-boundaries); this is not a Python-package delivery claim. |

For Python/Gym results, first inspect the imported module in the same environment:

```sh
uv run --locked --no-sync python -B -c 'import spire_lab._native as n; print(n.__file__); print(n.build_info())'
```

If the scoped environment is not the project default, set
`UV_PROJECT_ENVIRONMENT` to that existing environment for both commands. The
module path identifies what Python imported. Record it with the build context
(Python environment, build command, Rust toolchain, Cargo features and revision).
`build_info()` reports `source_digest`, `file_digest`, enabled `features`, build
`profile` and `dirty`, in addition to revision and Rust version. `source_digest`
covers the extension dependency closure, enabled crate features and profile;
`file_digest` covers the source-file portion. `dirty` is informational
(`true`/`false`, or `"unknown"` when Git is unavailable). Lane P requires
`features == ["spire-python/extension"]` and a release profile. Its read-only
candidate export has `dirty="unknown"`; the post-merge build must report
`dirty=false` and the same `source_digest` as Lane P. The identity test
recomputes the digest from the checkout and runs in the routine `tests/python/env`
lane as `tests/python/env/test_build_identity.py`.

Still record `_native.__file__`, the Python environment and build command, Rust
toolchain, revision, source and file digests, enabled features, profile, dirty
value, native module or wheel SHA-256, Python facade hashes, descriptor
fingerprints and exact test results. A Git revision or descriptor fingerprint
alone does not identify source bytes. A different HEAD does not invalidate a
verified receipt when only documentation changed or the build used verified
source bytes; equal HEAD alone does not make a stale artifact current. If those
inputs cannot be matched, report that limitation and do not count the run as
Python proof. The [Gym66 build guide](combat-training.md#shared-implementation-and-extension-build)
shows an isolated wheel workflow for authorized binding validation that preserves
other environments.

**M35 post-merge build (2026-11-11).** The verified post-merge packet for PR102
(`09b8fc81`) reports `source_digest`
`bb1888874447b0fd40090323b10311be45349f568d84f35cccb7813a05deb43a` and
427 passing Python tests. This receipt establishes the M35 extension build at
that source identity; it is not M36 evidence. The [batching contract](batching.md)
describes the merged API and compatibility limits.

**Historical core callers (M20 and M22–M25).** These legacy callers depended on
pre-run freeze markers from the retired ceremony and cannot be run as documented.
Their retained outcomes are historical evidence; Standard records and replay are
the current consumer path.

Consumer source, manifests, artifacts and required ignored evidence must outlive
managed sessions. Before archiving, preserve them in independently retained
storage, verify their hashes, and coordinate consumer release. Paseo session
archival can remove managed worktrees; a path there is not a retention guarantee.
Prefer an accepted Git commit plus a versioned read-only source export with its
manifest stored alongside it.

`maturin develop` builds and installs the extension. `uv sync` changes the
environment and removes the editable package, so neither is a routine test step.
Use setup or refresh commands only when the current task requires and authorizes
that change. For a documentation-only change, verify changed links, paths, and
documented commands. The unstaged `git diff --check` omits untracked files; before
commit, stage only explicit task-owned paths and run `git diff --cached --check`
to include new files. Never use blanket `git add`. Runtime suites do not validate
prose.

Validation/capture tools stay outside simulation and training execution. Neither
requires a running original game or an active Windows connection. The approved
[source-first amendment](scope.md#approved-source-first-sequencing-amendment-2026-10-16)
allows Rust development before original captures; design/plan/start review still
applies. Linux capture work is deferred, and original execution needs its own scope.
Original-runtime validation remains a later fidelity-promotion gate, not a label
granted by Python/Rust agreement.

## Environment

- Linux workstation (gym development/test host; external learners may use it too).
- Rust simulation and the gym do not require CUDA. Existing optional device checks
  are separate from environment correctness; a failing CUDA probe is not itself a
  simulator failure. The known readiness-test isolation issue is `spire-vhh`.
- Network access is required for first dependency downloads only; all later
  builds are reproducible from the lockfiles (`uv.lock`, `Cargo.lock`).
- No system-wide installs: toolchains land in user-local locations
  (`~/.local/bin`, `~/.cargo`) and the project virtualenv in `.venv`.

## One-time toolchain setup

These install commands are for a human-owned workstation that lacks the tools.
For an agent task, first inspect the provided environment and use its existing
toolchain; do not run setup merely to start reading or editing documentation.

```sh
# uv (user-local, ~/.local/bin)
curl -LsSf https://astral.sh/uv/install.sh | sh

# rustup with a minimal profile (user-local, ~/.cargo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
```

`rust-toolchain.toml` pins rustc 1.99.0 for this repository; rustup installs
or switches to it automatically on first cargo run.

## Python and Rust

- Python: the approved execution baseline is the CPython 3.12 family. The
  exact patch version is pinned in `.python-version` (currently 3.12.15).
  uv installs and uses that interpreter in `.venv`; do not use system Python.
- Rust: pinned in `rust-toolchain.toml` (currently 1.99.0); dependencies are
  pinned in `Cargo.lock`.

## Setup in a fresh checkout

This is a full Python/Gymnasium setup, not a Rust-only prerequisite. The
project-level dependencies include pinned Torch, so `uv sync` may download a
large runtime. Use it only when a full Python environment is intended; for Rust
work, the Cargo commands are sufficient. The command consumes the committed
lockfiles and does not re-resolve dependencies:

```sh
uv sync --locked --group gym-dev
cargo build --workspace --locked
uv run --no-sync maturin develop --release --locked
```

`maturin develop` compiles the `extension` feature (PyO3,
`pyo3/extension-module`, rust-numpy) and installs `spire_lab`, including
`spire_lab._native`, editable into `.venv`. The compiled module is written
into `python/spire_lab/` and is git-ignored.

Order matters: `uv sync` makes the venv mirror the project specification and
*removes* the editable `spire-lab` install. Therefore always run
`maturin develop` **after** any `uv sync`, and keep using `uv run --no-sync`
(from setup onward) so a sync does not remove the editable package before
the tests or probes run.

## Optional workflows

### Historical M10 Gym-only environment and binding refresh

> This recipe records the one-off M10 refresh and its named environment. It is
> not a general agent setup command. M11 did not refresh the extension. Use this
> recipe only when the current task explicitly selects the same environment and
> requires an artifact refresh.

Use the existing `target/m9-gym-dev-venv` (Python 3.12.15, NumPy 2.5.4,
Gymnasium 1.4.0, pytest 9.1.1, maturin 1.15.0); Torch is absent. Before the
approved bounded M10 extension refresh, preserve the currently installed native
binary byte-for-byte in ignored evidence. Keep the old M9 Cargo artifacts in
`target/m9-python-extension-20261024`; build M10 separately with the approved
worktree-local target:

```sh
UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv CARGO_TARGET_DIR=target/m10-python-extension-20261024 CARGO_BUILD_JOBS=2 CARGO_NET_OFFLINE=true uv run --locked --no-sync maturin develop --release --locked --offline --skip-install
```

This refresh replaces the in-place module only after the old binary is preserved;
`--skip-install`, `--no-sync`, and locked/offline flags avoid dependency expansion
or environment metadata changes. Verify the imported module path, build revision,
source/artifact snapshot, and Torch absence before Python tests. The controller-
verified Task 27 candidate passed 235 environment tests (195 existing plus 40 M10).
For subsequent changes, run
`UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv uv run --locked --no-sync python -B -m pytest tests/python/env -q`.
Pass only touched `.py`/`.pyi` paths to `prek --config prek.toml --files ...`; it
runs Ruff lint, non-rewriting Ruff format checks, and ty. No shared Git hooks or
broad formatting are part of this workflow. Do not use dependency-expanding
`maturin develop`; broader Python collection has historical Torch/readiness and
diff-schema/metadata-fixture limitations. Gym correctness has no CUDA gate.

### Full-dependency and readiness workflow

```sh
# Readiness/device tests are separate from Gym correctness
uv run --locked --no-sync python -B -m pytest tests/python/test_readiness.py -q

# Run the Rust test suite (links without a Python interpreter). `[profile.test]`
# builds tests at opt-level 2 with debug assertions and overflow checks still on;
# a warm full run takes about 3 minutes (was about 16).
cargo test --workspace --locked --offline

# Optional historical device-readiness probe (real computation; requires a GPU)
uv run --no-sync python -m spire_lab.readiness --require-cuda

# CLI version smoke check
cargo run --locked --offline -q -p spire-cli -- version
```

## Rebuilding after dependency or binding changes

These commands replace the selected Python environment and install the
extension. Use this recipe only when environment provisioning/refresh is part of
the active task. `gym-dev` includes Gymnasium; `dev` alone does not. For binding
validation that should preserve an existing environment, use the isolated wheel
workflow in the [Gym66 build guide](combat-training.md#shared-implementation-and-extension-build).
A default Cargo workspace build does not refresh the Python artifact.

```sh
uv sync --locked --group gym-dev
cargo build --workspace --locked
uv run --no-sync maturin develop --release --locked
uv run --locked --no-sync python -B -m pytest \
  tests/python/env \
  tests/python/test_ironclad_training.py \
  tests/python/test_combat_policy_view.py -q

# Run the known legacy comparator lane separately; see its baseline limits above.
uv run --locked --no-sync python -B -m pytest tests/python/combat -q
```

## Dependency updates (intentional lock regeneration)

Only when the set of dependencies itself changes, as a deliberate, recorded
step. These commands intentionally change the lockfiles, so they run without
`--locked` (which would merely refuse the change that is the point of the
step); the locked forms below verify the result.

```sh
uv add --dev <pkg>==<ver>       # or: uv remove <pkg>   (updates uv.lock on purpose)
uv lock                         # or: regenerate uv.lock from the manifest
cargo update -p <crate>         # or: cargo add, then verify --locked
cargo check --workspace --locked
uv sync --locked --group gym-dev # re-verify the full Gym venv against the new lock
uv run --no-sync maturin develop --release --locked
```

After such a change, rebuild the extension and rerun the affected/full suites,
then update [toolchain.md](toolchain.md). Device/CUDA checks are relevant only when
the scoped change affects that tooling; do not add a GPU gate to ordinary gym
work. Keep the known readiness-test failures explicit rather than calling a
qualified test result fully green. Dependency changes and commits need their own
authority; this guide is not permission to install or publish.

## Build configuration notes

- `pyproject.toml` sets the maturin layout:
  `python-source="python"`, `module-name="spire_lab._native"`,
  `manifest-path="crates/spire-python/Cargo.toml"`, and enables the
  `extension` feature for extension builds.
- `spire-python` keeps PyO3 and rust-numpy behind the non-default `extension`
  feature so `cargo test --workspace` compiles and links with only the Rust
  toolchain. The binding's numeric module uses rust-numpy for the implemented
  native environment buffers; it is not merely a future batching dependency.
- `crates/spire-python/build.rs` records the git revision (or
  `SPIRE_REVISION`, or `unknown`) and the Rust compiler version at build
  time, and watches the resolved git `HEAD` plus the branch ref inputs so
  incremental rebuilds refresh the revision after commit/checkout without
  `cargo clean`; `_native.build_info()` exposes the values.
- Crates: `spire-core` (Python-independent simulation core) <- `spire-env`
  (implemented bounded native environments/batching) <- `spire-python` (bindings)
  / `spire-cli` (developer CLI). The core contains trace-reader tooling and the
  narrow combat simulator; `python/spire_lab/gymnasium` provides the facades.
  No game state enters the device-readiness probe. See
  [environment usage](combat-environments.md) for the existing rollout example.

See [toolchain.md](toolchain.md) for the exact resolved version matrix.

## Native fuzz harness

`spire-fuzz` runs deterministic native campaigns over the catalog combat and
accepted Run profiles, including Standard at A0, A10 and A20 for all four
characters. Its default feature set leaves the workspace's existing
`spire-core` and `spire-env` features unchanged. Default catalog combat episodes
use a 200-step limit and run to terminal where possible; Run profile samples use
6 decisions to reach combat. Each catalog episode uses a distinct entry. The
smoke checks advertised-action preflight, invalid-input stability, public
encoding, descriptor/observation fingerprint stability within each run, and
fresh-prefix replay on representative catalog episodes. Every Run
profile must reach combat unless that episode records an engine failure. Two
complete smoke passes must produce the same summary hash; engine-class failures
are written and re-detected by `repro`, while
determinism, replay, and panic failures must remain zero. The summary names
fingerprint scope in `fingerprint_stability_scope` and does not assert
cross-version compatibility.

```sh
cargo test --workspace --locked --offline -j2
```

The opt-in diagnostics lane adds read-only engine validators, clone checks for
every advertised action, inner-Run equality checks, and the longer smoke with
the generated CompleteCombat and CompleteItemsRun drivers for all four
characters. Its smoke runs all-legal clone checks after step zero and compares
two complete summary hashes. This lane explicitly enables M26/M27 diagnostics and does not change
the workspace's default feature set; `spire-fuzz` enables
`spire-core/m27-diagnostics` only through its default-off `diagnostics` feature.
CompleteItemsRun generates item supplies
from the complete engine catalogs and starts directly at the Act 4 map, without
Neow. No accepted numeric profile encoder applies there, so the summary records
`encode: "n/a"`; policy selection uses candidate action kinds and the legal mask.
For CompleteItemsRun, `choose_none` switches to deterministic first-legal choice
after it revisits the same normalized public boundary, which breaks selection
cancel loops.
CompleteCombat remains its existing complete-card start with profile item
admission: M27 has no general complete-items combat constructor, and its
`m27_pair_combat` API is a fixed pair fixture rather than a natural driver start.
Its `encode` is `harness projection (spire-fuzz-complete-public-v1)` and its
fingerprint is `n/a`; CompleteItemsRun continues to report `encode: "n/a"`.
The lane does not establish original-runtime parity or coverage beyond the
enumerated encounters and Run profiles.

Known limits: `spire-core/fuzz-diagnostics` alone is inert; use `spire-fuzz/diagnostics`, which enables the required M20+ diagnostic features.
Known limit: CompleteCombat and CompleteItemsRun classify non-`InvalidAction` errors from illegal probes as `probe_rejected`.
Known limit: diagnostics replay prefixes depend on `max_steps`, so replay/determinism repros may use another prefix and those failure kinds do not shrink.
Standard Run fuzz profiles use `RunSession::new_standard` with replay recording enabled. They include one profile for each character at A0, A10 and A20; the profile identity and ascension are included in repro configuration, and both sampled smoke and replay checks exercise these profiles. In the diagnostics feature, the driver compares the Standard encoder's output with the live session observation at each visited boundary. Standard boundaries are not treated as finite-profile cutoffs.

The core-only `RoomScenario::Standard` constructor (named `StandardA0` until M29a1) remains available at A0–A20 (A0 since M28f2, A1–A5 since M29d, A6–A10 since M30, A11–A15 since M31, A16–A19 since M32, A20 since M33). `cargo run --release -p spire-core --example m28_traversal [--features m28-diagnostics|m29-diagnostics] -- --out DIR [--ascension N | --ascensions A-B]` drives it (seeded uniform, reach and supplied modes; see the [M28 profile](milestones/m28-events-economy.md), the [M29 profile](milestones/m29-ascension1-5.md), the [M30 profile](milestones/m30-ascension6-10.md), the [M31 profile](milestones/m31-ascension11-15.md), the [M32 profile](milestones/m32-ascension16-19.md) and the [M33 profile](milestones/m33-ascension20.md)). For the M31 A0–A15 range use `--ascensions 0-15`; for M32 use `--ascensions 16-19`; for M33 use `--ascensions 0-20`. Each level writes `DIR/a<N>/traversal.json`, with M30, M31, M32 and M33 facts under their `m30`, `m31`, `m32` and `m33` keys, and `DIR/a<N>/digests.jsonl` with per-step decision digests. To attribute A0–A19 differences, build the same example on the base tree and join the two directories by first differing digest; M32's global Louse-opening change is marked per Run by `m32.first_louse_step` (see the [M32 profile](milestones/m32-ascension16-19.md)). M33's `m33` key records A20 boss-fight, second-boss, Act 4 and Heart reach facts (see the [M33 profile](milestones/m33-ascension20.md)).
The `native_fuzz run` CLI defaults to `--threads 2`.

```sh
cargo test -p spire-fuzz --features diagnostics --locked --offline -j2
CARGO_TARGET_DIR=target/native-fuzz cargo run --release -p spire-fuzz \
  --features diagnostics --locked --offline --example native_fuzz -- \
  run --seed-start 1 --seed-end 100 --threads 2 --budget-secs 120 \
  --out native-fuzz-out --drivers combat,complete,run,items \
  --weights catalog=1,complete=1,run=1,items=1 --combat-max-steps 2000 \
  --run-max-steps 20000 --determinism-every 16 --replay-every 16 \
  --all-legal-every 16
```

Both lanes write a summary that records the active feature set. Campaigns can
load one or more accepted combat catalogs with `--catalog PATH`; entries are
scheduled round-robin. Driver weights use deterministic interleaving, and the
determinism/replay options sample episodes by seed while all-legal checks sample
steps. Summaries count only the step limit as capped and report accepted
nonterminal profile cutoffs separately; any other nonterminal zero-action
decision is saved as a shrunk `stuck` failure. Each grouped failure has one
shrunk repro. `native_fuzz repro PATH` reruns a saved failure case.

**Cadence.** After each main merge, run these diagnostics release phases on the
current `main` (without pinning an older revision): 500k combat episodes, 5k full
Runs, and 200k complete-items episodes, using eight threads per phase. Save each
summary in a dated evidence directory with the source revision/archive and
binary hashes; the harness deduplicates and shrinks failures for the PM to file
as `native-fault,fuzz` Beads. Keep the default-lane smoke in
`cargo test --workspace --locked --offline -j2`.

```sh
export CARGO_TARGET_DIR=target/native-fuzz-longrun
EVIDENCE_DIR="evidence/native-fuzz-$(date -u +%Y%m%d)"
mkdir -p "$EVIDENCE_DIR"
cargo run --release -p spire-fuzz --features diagnostics --locked --offline -j2 --example native_fuzz -- run --seed-start 1 --seed-end 250000 --threads 8 --out "$EVIDENCE_DIR/combat" --drivers combat,complete --weights catalog=1,complete=1,run=0,items=0 --min-combat 500000 --determinism-every 16 --replay-every 16 --all-legal-every 16
cargo run --release -p spire-fuzz --features diagnostics --locked --offline -j2 --example native_fuzz -- run --seed-start 1 --seed-end 5000 --threads 8 --out "$EVIDENCE_DIR/runs" --drivers run --weights catalog=0,complete=0,run=1,items=0 --min-runs 5000 --determinism-every 16 --replay-every 16 --all-legal-every 16
cargo run --release -p spire-fuzz --features diagnostics --locked --offline -j2 --example native_fuzz -- run --seed-start 1 --seed-end 200000 --threads 8 --out "$EVIDENCE_DIR/items" --drivers items --weights catalog=0,complete=0,run=0,items=1 --min-runs 200000 --determinism-every 16 --replay-every 16 --all-legal-every 16
{
  printf 'source_revision='; git rev-parse HEAD
  printf 'source_status='; git status --short --branch
  printf 'source_archive_sha256='; git archive --format=tar HEAD | sha256sum | cut -d ' ' -f1
  printf 'native_fuzz_sha256='; sha256sum "$CARGO_TARGET_DIR/release/examples/native_fuzz" | cut -d ' ' -f1
} > "$EVIDENCE_DIR/manifest.txt"
```
