# Development setup

Phase 1a state: a reproducible Rust/Python package foundation and a device
readiness probe. No simulator, pool, model, or learning code exists yet; the
optimizer probe is a tiny smoke computation and must not be presented as a
validated learning result.

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
step:

```sh
uv add --locked --dev <pkg>==<ver>   # or: uv remove <pkg>
uv lock                              # regenerates uv.lock from the manifest
cargo update -p <crate>              # or: cargo add/update, then verify --locked
cargo check --workspace --locked
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
- Crates: `spire-core` (dependency-free simulation core) <- `spire-env`
  (environment abstractions) <- `spire-python` (bindings) / `spire-cli`
  (developer CLI). Core and env are documentation-only in phase 1a, and no
  game state enters the readiness probe.

See [toolchain.md](toolchain.md) for the exact resolved version matrix.
