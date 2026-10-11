"""Regression tests for build metadata (compiled Git revision tracking).

These tests never touch this repository's branch. In a fresh, disposable
repository they build a minimal cargo crate that uses *this* repository's
``crates/spire-python/build.rs`` and verify that the compiled
``SPIRE_REVISION`` tracks the real repository HEAD across commits, branch
checkouts, and detached HEAD without any ``cargo clean``.

They build real binaries and assert on their output; no assertion is
expected to be skipped when a cargo toolchain is present.
"""

from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_RS = REPO_ROOT / "crates" / "spire-python" / "build.rs"

CANDIDATES = [
    shutil.which("cargo"),
    str(Path.home() / ".cargo" / "bin" / "cargo"),
]
CARGO = next((c for c in CANDIDATES if c and Path(c).exists()), None)

_LIB = 'pub const REVISION: &str = env!("SPIRE_REVISION");\n'
_BIN = 'fn main() { println!("{}", probe::REVISION); }\n'
_MANIFEST = (
    "[package]\n"
    'name = "probe"\n'
    'version = "0.1.0"\n'
    'edition = "2024"\n'
    "\n"
    "[lib]\n"
    'path = "src/lib.rs"\n'
    "\n"
    "[[bin]]\n"
    'name = "show-revision"\n'
    'path = "src/main.rs"\n'
)

OVERRIDE = "deadbeef" * 5  # 40 hex chars, valid SPIRE_REVISION value


@pytest.mark.skipif(CARGO is None, reason="cargo toolchain not available")
def _run(base: Path, args: list[str], env: dict[str, str] | None = None):
    merged = dict(os.environ)
    merged["CARGO_TERM_COLOR"] = "never"
    if env:
        merged.update(env)
    result = subprocess.run(
        args, cwd=base, env=merged, capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        raise AssertionError(
            f"command failed ({result.returncode}): {args}\n"
            f"stdout: {result.stdout}\nstderr: {result.stderr}"
        )
    return result.stdout.strip()


def _git(base: Path, *args: str) -> str:
    return _run(
        base,
        ["git", "-c", "user.name=Spire Test", "-c", "user.email=test@spire", *args],
    )


def _setup(tmp: Path, with_git: bool = True) -> Path:
    (tmp / "crate" / "src").mkdir(parents=True)
    (tmp / "outside").mkdir()
    (tmp / "crate" / "Cargo.toml").write_text(_MANIFEST)
    (tmp / "crate" / "build.rs").write_text(BUILD_RS.read_text())
    (tmp / "crate" / "src" / "lib.rs").write_text(_LIB)
    (tmp / "crate" / "src" / "main.rs").write_text(_BIN)
    (tmp / "outside" / "a.txt").write_text("one\n")
    if with_git:
        _git(tmp, "init", "-b", "main")
        _git(tmp, "add", "crate", "outside")
        _git(tmp, "commit", "-m", "initial")
    return tmp / "crate"


def _build(crate: Path, env: dict[str, str] | None = None) -> None:
    _run(crate, [CARGO, "build", "--quiet"], env=env)


def _revision(crate: Path) -> str:
    return _run(crate, [str(crate / "target" / "debug" / "show-revision")])


def test_revision_refreshes_after_commit_outside_crate_without_clean(tmp_path):
    """Committing only a file outside the crate must refresh the revision."""
    crate = _setup(tmp_path)
    head_before = _git(tmp_path, "rev-parse", "HEAD")
    _build(crate)
    assert _revision(crate) == head_before

    (tmp_path / "outside" / "a.txt").write_text("two\n")
    _git(tmp_path, "commit", "-am", "update outside data")
    head_after = _git(tmp_path, "rev-parse", "HEAD")
    assert head_after != head_before

    _build(crate)  # incremental: no cargo clean
    assert _revision(crate) == head_after


def test_revision_survives_checkout_and_detached_head(tmp_path):
    crate = _setup(tmp_path)
    head = _git(tmp_path, "rev-parse", "HEAD")
    _build(crate)
    assert _revision(crate) == head

    _git(tmp_path, "checkout", "-b", "side")
    _build(crate)
    assert _revision(crate) == head

    _git(tmp_path, "checkout", "--detach")
    _build(crate)
    assert _revision(crate) == head

    _git(tmp_path, "checkout", "main")
    _build(crate)
    assert _revision(crate) == head


def test_revision_env_override_and_restore(tmp_path):
    crate = _setup(tmp_path)
    head = _git(tmp_path, "rev-parse", "HEAD")
    _build(crate)
    assert _revision(crate) == head

    _build(crate, env={"SPIRE_REVISION": OVERRIDE})
    assert _revision(crate) == OVERRIDE

    _build(crate)
    assert _revision(crate) == head


def test_source_archive_without_git_reports_unknown(tmp_path):
    crate = _setup(tmp_path, with_git=False)
    _build(crate)
    assert _revision(crate) == "unknown"
