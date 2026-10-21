# Development setup

See [repository scope](scope.md) for the goals and current delivery path: an AI
that learns to play StS, enabled by an optimized, game-correct **headless Rust
simulator**, native batching and Python learning. No playable client or rendering
is part of this repository's product.

Current state: the Rust/Python package foundation, device-readiness probe, narrow
supplied-entry Python combat reference and bounded native batched combat/Gymnasium
interfaces are implemented. The reference is a readable
correctness aid, not the production simulator or a Python optimization target;
it is not original-runtime validated. Its final recorded implementation run passed
735 Python tests. The narrow Rust combat engine and differential harness are also
implemented and reviewed; the final recorded core-handoff verification passed 124
native tests in each workspace/release run and 830 Python tests. The bounded
environment slice adds supplied-entry scenarios, public observations, native
batching and single/vector Gymnasium facades. A fresh offline Rust workspace run
passed 169 tests. Controller verification with a freshly built candidate-revision
extension passed all 167 environment Python tests; the full Python suite had 995
passes and the same two baseline readiness-test failures as main. Those CPU-only
failures are separate from historical required-CUDA checks that failed with CUDA
allocation OOMs. A fully green suite, CUDA readiness and original-runtime validation
are not claimed. Models and learners are
not implemented. The optimizer probe is only a
tiny smoke computation, not a validated learning result or training pipeline.

Validation/capture tools stay outside simulation and training execution. Neither
requires a running original game or an active Windows connection. The approved
[source-first amendment](scope.md#approved-source-first-sequencing-amendment-2026-10-16)
allows Rust development before original captures; design/plan/start review still
applies. Linux capture work is deferred, and original execution needs its own scope.
Original-runtime validation remains a later fidelity-promotion gate, not a label
granted by Python/Rust agreement.

## Environment

- Linux workstation (development and training host).
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

```sh
# Run the Python behavior tests
uv run --no-sync pytest tests/python/test_readiness.py -q

# Run the Rust test suite (links without a Python interpreter)
cargo test --workspace --locked

# Device readiness (real computation; requires a working GPU)
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

After such a change, rebuild the extension and rerun the full suites and the
real CUDA readiness probe before committing, and update
[toolchain.md](toolchain.md).

## Build configuration notes

- `pyproject.toml` sets the maturin layout:
  `python-source="python"`, `module-name="spire_lab._native"`,
  `manifest-path="crates/spire-python/Cargo.toml"`, and enables the
  `extension` feature for extension builds.
- `spire-python` keeps PyO3 and rust-numpy behind the non-default `extension`
  feature so `cargo test --workspace` compiles and links with only the Rust
  toolchain. rust-numpy 0.29.0 is resolved for this baseline and is not used
  by code yet (prepared for later batched numeric outputs).
- `crates/spire-python/build.rs` records the git revision (or
  `SPIRE_REVISION`, or `unknown`) and the Rust compiler version at build
  time, and watches the resolved git `HEAD` plus the branch ref inputs so
  incremental rebuilds refresh the revision after commit/checkout without
  `cargo clean`; `_native.build_info()` exposes the values.
- Crates: `spire-core` (Python-independent simulation core) <- `spire-env`
  (environment abstractions) <- `spire-python` (bindings) / `spire-cli`
  (developer CLI). The core contains trace-reader tooling and the narrow combat
  simulator; the environment crate remains scaffolding. No game state enters the
  device-readiness probe.

See [toolchain.md](toolchain.md) for the exact resolved version matrix.
