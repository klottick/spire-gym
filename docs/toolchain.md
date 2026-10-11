# Toolchain and tested versions

Exact versions resolved for phase 1a task 1. The lockfiles
(`uv.lock`, `Cargo.lock`), `.python-version`, and `rust-toolchain.toml` are
authoritative; this file documents what was actually tested.

## Toolchain

| Component | Version |
| --- | --- |
| OS | Ubuntu 26.04 LTS, x86_64 |
| uv | 0.13.0 |
| rustup | 1.29.1 |
| rustc | 1.99.0 (b940084d7 2026-09-28) |
| cargo | 1.99.0 (5f94df478 2026-08-27) |
| Python (uv-managed) | CPython 3.13.16 |
| NVIDIA driver | 595.91.07 (host max CUDA 13.2) |
| GPU | NVIDIA GeForce RTX 5090, compute capability 12.0 (sm_120) |

## Python packages (uv.lock, Python 3.13.16)

| Package | Pinned version |
| --- | --- |
| torch | 2.14.1 (wheel: `2.14.1+cu130`, CUDA 13.0 runtime libs) |
| numpy | 2.5.4 |
| pytest | 9.1.1 |
| maturin | 1.15.0 |
| triton | 3.8.0 (torch dependency) |
| cuda-toolkit | 13.0.3.0 (torch dependency) |

## Rust crates (Cargo.lock)

| Crate | Version |
| --- | --- |
| pyo3 | 0.29.3 |
| spire-core / spire-env / spire-python / spire-cli | 0.1.0 (workspace) |

## Test evidence (2026-10-11)

- `cargo test --workspace --locked`: all suites `ok`, 0 failures (scaffolding
  crates carry no behavior tests yet).
- `uv run --no-sync pytest tests/python/test_readiness.py -q`: 4 passed.
- `uv run --no-sync python -m spire_lab.readiness --require-cuda`: real RTX
  5090 computation succeeded (tiny forward/backward/Adam step), report shows
  `status: ok`, extension `spire-python`, wheel `torch 2.14.1+cu130`,
  CUDA runtime `13.0`, driver `595.91.07`, GPU
  `NVIDIA GeForce RTX 5090`, capability `12.0`,
  `finite_gradient: true`, `weights_changed: true`, `synchronized: true` on
  both CPU and CUDA.
- Mocked unavailability: `check(require_cuda=True)` with
  `torch.cuda.is_available` patched to `False` raises `RuntimeError` (tested);
  readiness never infers a pass from device enumeration and never fakes GPU
  success.
