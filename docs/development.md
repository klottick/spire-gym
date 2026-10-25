# Development setup

See [repository scope](scope.md) for the current delivery path: a usable headless
gym with a Rust simulator, native batching and thin Python/Gymnasium interfaces
for external ML consumers. Models, training, learner checkpoints and experiments
are outside this repo. No playable client or rendering is part of the product.

Current state: M9 is accepted at its captured Ironclad A0/Act3 boundary. M10 native
implementation and the preservation matrix are controller-verified on the final
607-file source baseline. The source-matched fresh Python environment passed 251
tests (235 prior plus 16 Task 28); changed-file Ruff lint, format, and `ty` checks
passed. Task 28 verified a real optional Elixir selector and its Python record/replay
and accepted-child delivery-fault behavior. The M10 profile is accepted at its
declared captured scope after ONE clean independent review and exact verification
of the 608 reviewed source files in both checkouts. Earlier combat-reference
and bounded-environment counts recorded in the [dated work-status history](work-status.md)
are historical checkpoints, not current M10 evidence. Models and learners remain
external; no original-runtime validation or full-Python/workspace-green claim is made.

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

Consumes the committed lockfiles; it never re-resolves dependencies:

```sh
uv sync --locked --dev    # Python runtime + dev dependencies, locked, editable-ready
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

## Everyday workflow

### Existing Gym-only environment and binding refresh

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

### Existing full-dependency workflow

```sh
# Run the Python behavior tests
uv run --no-sync pytest tests/python/test_readiness.py -q

# Run the Rust test suite (links without a Python interpreter)
cargo test --workspace --locked

# Optional historical device-readiness probe (real computation; requires a GPU)
uv run --no-sync python -m spire_lab.readiness --require-cuda

# CLI version smoke check
cargo run --locked -q -p spire-cli -- version
```

## Rebuilding after dependency or binding changes

```sh
uv sync --locked --dev
cargo build --workspace --locked
uv run --no-sync maturin develop --release --locked
uv run --no-sync pytest tests/python -q
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
uv sync --locked --dev          # re-verify the venv against the new lock
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
