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

- Python: the exact patch version is pinned in `.python-version`
  (currently 3.13.16). uv installs and uses that interpreter in `.venv`;
  do not use system Python.
- Rust: pinned in `rust-toolchain.toml` (currently 1.99.0); dependencies are
  pinned in `Cargo.lock`.

## Setup in a fresh checkout

```sh
uv lock              # resolves with .python-version; verifies uv.lock
uv sync --dev        # installs runtime + dev dependencies into .venv
cargo test --workspace --locked
uv run --no-sync maturin develop --release --locked
```

`maturin develop` compiles the `extension` feature (PyO3 +
`pyo3/extension-module`) and installs `spire_lab`, including
`spire_lab._native`, editable into `.venv`. The compiled module is written
into `python/spire_lab/` and is git-ignored.

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
cd tests/python && uv run --no-sync pytest -q   # or: uv run --no-sync pytest tests/python/test_readiness.py -q
```

## Build configuration notes

- `pyproject.toml` sets the maturin layout:
  `python-source="python"`, `module-name="spire_lab._native"`,
  `manifest-path="crates/spire-python/Cargo.toml"`, and enables the
  `extension` feature for extension builds.
- `spire-python` keeps PyO3 behind the non-default `extension` feature so
  `cargo test --workspace` compiles and links with only the Rust toolchain.
- `crates/spire-python/build.rs` records the git revision (or
  `SPIRE_REVISION`, or `unknown`) and the Rust compiler version at build
  time; `_native.build_info()` exposes them.
- Crates: `spire-core` (dependency-free simulation core) <- `spire-env`
  (environment abstractions) <- `spire-python` (bindings) / `spire-cli`
  (developer CLI). No game state enters the readiness probe.

See [toolchain.md](toolchain.md) for the exact resolved version matrix.
