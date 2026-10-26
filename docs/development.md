# Development setup

See [repository scope](scope.md) for the current delivery path: a usable headless
gym with a Rust simulator, native batching and thin Python/Gymnasium interfaces
for external ML consumers. Models, training, learner checkpoints and experiments
are outside this repo. No playable client or rendering is part of the product.

Use the [dated work-status snapshot](work-status.md) for current milestone
coordination and each [accepted profile](milestones/) for its exact evidence and
limits. M11 is accepted as a finite native Silent A0 authored-combat slice; it did
not refresh the Python extension or establish Python delivery. Gym66 adds an
opt-in standalone Ironclad A0 interface and common combat-policy view; see the
[training guide](combat-training.md). M12's approved design and plan passed
independent review, and implementation is authorized against the accepted
post-`.66` baseline. Check Beads and the current primary session for live task
ownership. A native source result, an installed extension, and Python/Gym behavior
are separate evidence. Models and learners remain external; no original-runtime
validation or full-workspace-green claim follows from a scoped profile.

## Agent lane selection and artifact identity

Choose the narrowest existing lane for the changed surface:

| Work | Command | Limit |
| --- | --- | --- |
| Rust core, environment, or CLI | `cargo test --workspace --locked --offline` | Native Rust only; default features do not build the Python extension. Use a profile's focused command when available. |
| Gym/Python facade | `uv run --locked --no-sync python -B -m pytest tests/python/env tests/python/test_ironclad_training.py tests/python/test_combat_policy_view.py -q` | Requires an already provisioned Gym environment and a source-matched extension; covers the existing facade and Gym66 session/policy-view tests. |
| Python combat reference / legacy comparator | `uv run --locked --no-sync python -B -m pytest tests/python/combat -q` | Separate compatibility lane. Known legacy comparator mismatches are baseline-reported in [work status](work-status.md) and [M9 limits](milestones/m9-act3.md#verification-boundaries); this is not a Python-package delivery claim. |

For Python/Gym results, first inspect the imported module in the same environment:

```sh
uv run --locked --no-sync python -B -c 'import spire_lab._native as n; print(n.__file__); print(n.build_info())'
```

If the scoped environment is not the project default, set
`UV_PROJECT_ENVIRONMENT` to that existing environment for both commands. The
module path identifies what Python imported. Record it with the build context
(Python environment, build command, Rust toolchain, Cargo features, and
revision/dirty state). `build_info()` reports revision and Rust version but
contains no source hashes. Use the build context and source-match
receipt: hashes for the Rust engine, bindings, and Python inputs that affect the
extension; native module or wheel SHA-256; relevant descriptor fingerprint(s);
and exact test results. A Git revision or descriptor fingerprint alone does not
identify source bytes. A different HEAD does not invalidate a verified source
receipt when only documentation changed or the build used verified dirty source;
equal HEAD alone does not make an unverified or stale artifact current. If those
inputs cannot be matched, report that limitation and do not count the run as
Python proof. The [Gym66 build guide](combat-training.md#shared-implementation-and-extension-build)
shows an isolated wheel workflow for authorized binding validation that preserves
other environments.

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

# Run the Rust test suite (links without a Python interpreter)
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
