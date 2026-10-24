# Development setup

See [repository scope](scope.md) for the current delivery path: a usable headless
gym with a Rust simulator, native batching and thin Python/Gymnasium interfaces
for external ML consumers. Models, training, learner checkpoints and experiments
are outside this repo. No playable client or rendering is part of the product.

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
are not claimed. Models and learners are external, not planned in-repo work.
The existing optimizer probe is retained historical device-readiness smoke code,
not a training pipeline or a gym prerequisite. No dependencies were removed as
part of this scope correction.

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

### Gym-only development and on-demand quality checks

The approved M9 environment is `target/m9-gym-dev-venv`, managed by uv's locked
`gym-dev` group (NumPy/Gymnasium/pytest/maturin plus pinned prek/Ruff/ty). It excludes
the project's existing Torch runtime dependency. Setup requires explicit authority;
routine checks do not sync or install:

```sh
# Only during separately approved initial setup:
UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv uv sync --locked --only-group gym-dev
# Build in place WITHOUT maturin's automatic project-dependency installation:
UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv CARGO_TARGET_DIR=target/m9-python-extension-20261024 CARGO_BUILD_JOBS=2 uv run --locked --no-sync maturin develop --release --locked --offline --skip-install
# Editable package metadata, no dependency expansion (needed once / after sync):
uv pip install --python target/m9-gym-dev-venv/bin/python --no-deps --no-build-isolation --editable .
# Routine checks, using the existing environment and freshly built extension:
UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv uv run --locked --no-sync python -B -m pytest tests/python/env -q
UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv uv run --locked --no-sync prek run --config prek.toml --files python/spire_lab/gymnasium/run.py
```

Pass only touched `.py`/`.pyi` paths to prek. It checks Ruff lint, Ruff format
without rewrites, and ty. No Git hook is installed and no broad formatting is
authorized. Native extension signatures are in `python/spire_lab/_native.pyi`.
**Do not use ordinary `maturin develop` for a Gym-only environment**: it installs
project runtime dependencies, including Torch/CUDA, even after gym-only uv sync.
The original affected M9 `.venv` is preserved, not silently removed or repurposed.
The actual isolated verification environment has Torch confirmed absent.
Full `tests/python` collection includes the historical Torch readiness lane;
without Torch it reports that missing dependency. Gym correctness has no CUDA gate.

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
