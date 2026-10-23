# Toolchain and tested versions

## Bounded combat environment status

The counts below are retained verification records, not a rerun during the
2026-10-22 documentation/tracker cleanup. Current scope is the gym for external
ML consumers; model/training code is outside this repo. Existing tensor/device
smoke dependencies remain pinned and unchanged. CUDA is not a gym prerequisite;
the readiness-test isolation follow-up remains `spire-vhh`.

The existing native batching and single/vector Gymnasium slice passed a fresh
offline run of 169 Rust workspace tests. Controller verification with a freshly
built candidate-revision extension passed all 167 `tests/python/env` tests. The
full Python suite had 995 passes and two unchanged baseline readiness-test failures:

- `test_check_fails_closed_when_required_cuda_report_is_unsuccessful`
- `test_check_normalizes_cuda_probe_exception`

These current CPU-only failures report `CPU readiness probe failed: No CUDA GPUs
are available`; they are separate from historical required-CUDA checks that failed
with CUDA allocation OOMs. This is not a fully green suite, CUDA-readiness
certification, original-runtime validation or full-run support.

Exact versions resolved for phase 1a task 1 (including fix round 1). The
lockfiles (`uv.lock`, `Cargo.lock`), `.python-version`, and
`rust-toolchain.toml` are authoritative; this file documents what was
actually tested.

## Toolchain

| Component | Version |
| --- | --- |
| OS | Ubuntu 26.04 LTS, x86_64 |
| uv | 0.13.0 |
| rustup | 1.29.1 |
| rustc | 1.99.0 (b940084d7 2026-09-28) |
| cargo | 1.99.0 (5f94df478 2026-08-27) |
| Python (uv-managed) | CPython 3.12.15 (approved CPython 3.12 baseline) |
| NVIDIA driver | 595.91.07 (host max CUDA 13.2) |
| GPU | NVIDIA GeForce RTX 5090, compute capability 12.0 (sm_120) |

## Python packages (uv.lock, Python 3.12.15)

| Package | Pinned version |
| --- | --- |
| torch | 2.14.1 (wheel: `2.14.1+cu130`, CUDA 13.0 runtime libs, cp312 build) |
| numpy | 2.5.4 |
| pytest | 9.1.1 |
| maturin | 1.15.0 |
| triton | 3.8.0 (torch dependency) |
| cuda-toolkit | 13.0.3.0 (torch dependency) |

## Rust crates (Cargo.lock)

| Crate | Version | Notes |
| --- | --- | --- |
| pyo3 | 0.29.3 | extension binding, `pyo3/extension-module` in extension builds |
| rust-numpy (`numpy`) | 0.29.0 | optional, extension-feature only; requires `pyo3 ^0.29` (compatible with the locked 0.29.3) and targets the NumPy 2.x runtime (2.5.4); used by native numeric environment bindings |
| spire-core / spire-env / spire-python / spire-cli | 0.1.0 (workspace) | narrow combat core, native environment/batching, bindings and developer CLI are implemented; not full-run support |

## Test evidence (fix round 1, on CPython 3.12.15)

- `cargo test --workspace --locked`: 7/7 suites `ok`, 0 failures; Rust
  tests link without a Python interpreter (PyO3/rust-numpy gated behind the
  non-default `extension` feature).
- `uv run --no-sync pytest tests/python -q`: 10 passed
  (`test_readiness.py`: 6 tests incl. fail-closed optimizer/CUDA report
  regressions; `test_build_metadata.py`: 4 disposable-repo revision-tracking
  tests incl. commit-outside-crate refresh without `cargo clean`,
  checkout/detached-head, `SPIRE_REVISION` override, and source-archive
  `unknown`).
- `uv run --no-sync python -m spire_lab.readiness --require-cuda`: real RTX
  5090 computation succeeded (tiny forward/backward/Adam step), `status: ok`,
  extension `spire-python`, `revision: bcc69a7529d36fa504b1d47eb2fc178ab4b51b41`
  (HEAD at build time), Rust 1.99.0, wheel `torch 2.14.1+cu130`, CUDA
  runtime `13.0`, driver `595.91.07`, GPU `NVIDIA GeForce RTX 5090`,
  capability `12.0`, `finite_gradient: true`, `weights_changed: true`,
  `synchronized: true` on both CPU and CUDA.
- Fail-closed verification: with `torch.optim.Adam.step` no-oped in memory,
  the real CPU probe reports `weights_changed: false` and both
  `check(require_cuda=False)` and the CLI exit nonzero with
  `"status": "failed"`; with a required-CUDA report faked to all-false
  flags, `check(require_cuda=True)` raises `RuntimeError`. Simulated
  unavailability of CUDA still raises before probing.
- Historical note: the original 3.13.16 baseline run (commit `bcc69a7`) is
  preserved in the report below as history; all fix-round-1 evidence above is
  from the 3.12 baseline.
