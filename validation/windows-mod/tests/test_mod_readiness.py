"""Fail-closed readiness tests for the ``:mod`` capture subproject (finding I7).

These tests exercise the *real* ``:mod:referenceReadiness`` predicate
(``validation/windows-mod/mod/build.gradle.kts``), not a Python re-implementation.
For each synthetic diagnostic input we copy the whole ``validation/windows-mod``
project into a disposable directory, swap ``docs/validation/reference-build.json``
for a case-specific fixture, optionally add clearly synthetic "jar" text files,
and drive ``gradle :mod:referenceReadiness`` with a fresh, isolated
``GRADLE_USER_HOME`` (no cache reuse, no global config).

The committed ``docs/validation/reference-build.json`` is NEVER mutated: every
case writes its own fixture into the disposable copy. The fixture "jars" are
plain text, clearly NOT game assets, and exist only so the readiness check can
compute real digests to compare against the claimed hashes.

Every case below must come out **BLOCKED** (exit code 1, named failure tag in
the diagnostic, and no ``mod/build/libs`` produced). A successful, empty,
source-less capture artifact must remain impossible even with apparently valid
metadata.

Run with::

    uv run --no-sync pytest validation/windows-mod/tests/test_mod_readiness.py -s

A toolchain (JDK + Gradle) is expected at the repository-root ``tools/``
directory; the test discovers it and skips (rather than silently passes) if it
is absent.
"""

from __future__ import annotations

import json
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
PROJECT_ROOT = REPO_ROOT / "validation" / "windows-mod"
COMMITTED_REFERENCE = REPO_ROOT / "docs" / "validation" / "reference-build.json"

TOOL_JDK = REPO_ROOT / "tools" / "jdk17"
TOOL_GRADLE = REPO_ROOT / "tools" / "gradle-8.14.6" / "bin" / "gradle"

# A 64-lowercase-hex string that is structurally a valid SHA-256 digest.
SYNTH_A = "a" * 64
SYNTH_B = "b" * 64
SYNTH_C = "c" * 64

# Distinct, machine-assertable failure tags the readiness predicate must emit.
TAG_PREFIX = "MOD-READINESS"


def _tools_present() -> bool:
    return TOOL_JDK.is_dir() and TOOL_GRADLE.is_file()


@pytest.fixture(scope="module")
def _require_tools():
    assert _tools_present(), (
        "a JDK (tools/jdk17) and Gradle (tools/gradle-8.14.6) must be present; "
        "these tests drive the real Gradle readiness predicate"
    )


def _base_reference() -> dict:
    """A structurally complete, internally consistent, fully-populated reference.

    Every hash field agrees, both targets are present, and it is marked verified.
    This isolates the *implementation-source* requirement: the one thing this
    subproject still genuinely lacks.
    """
    return {
        "schema": "spire-lab/reference-build/1",
        "status": "verified",
        "verified": True,
        "game": {"expected": "synthetic diagnostic — NOT a game asset",
                "version": "synthetic-1", "sha256": SYNTH_A,
                "evidence_required": "synthetic diagnostic input for a readiness test"},
        "loader": {"expected": "synthetic diagnostic — NOT a game asset",
                   "version": "synthetic-1", "sha256": SYNTH_B,
                   "evidence_required": "synthetic diagnostic input for a readiness test"},
        "game_sha256": SYNTH_A,
        "loader_sha256": SYNTH_B,
        "runtime_target": "17",
        "compiler_target": "17",
        "notes": "SYNTHETIC diagnostic input — never a verified game build",
    }


@dataclass
class Case:
    name: str
    reference: dict | None  # None => reference-build.json file absent
    jars: tuple[str, ...]   # which synthetic jar files to materialise
    expected_tags: tuple[str, ...]  # substrings each required in the failure output


CASES: list[Case] = [
    Case(
        name="missing_reference_file",
        reference=None,
        jars=(),
        expected_tags=(
            "reference-build.json is missing",
            "game jar is not provided",
            "source-less mod artifact is not produced",
        ),
    ),
    Case(
        name="status_not_verified",
        reference={**_base_reference(), "status": "pending", "verified": False},
        jars=("gameJar", "modTheSpireJar"),
        expected_tags=('status is "pending" (need "verified")',),
    ),
    Case(
        name="null_targets",
        reference={**_base_reference(), "runtime_target": None, "compiler_target": None},
        jars=("gameJar", "modTheSpireJar"),
        expected_tags=("runtime_target and compiler_target are both required",),
    ),
    Case(
        name="game_hash_mismatch",
        reference={
            **_base_reference(),
            "game_sha256": SYNTH_A,      # claimed
            "game": {**_base_reference()["game"], "sha256": SYNTH_A},
        },
        jars=("gameJar", "modTheSpireJar"),  # real digest of gameJar will differ
        expected_tags=("game jar SHA-256 does not match the supplied file",),
    ),
    Case(
        name="loader_hash_mismatch",
        reference={
            **_base_reference(),
            "loader_sha256": SYNTH_B,
            "loader": {**_base_reference()["loader"], "sha256": SYNTH_B},
        },
        jars=("gameJar", "modTheSpireJar"),
        expected_tags=("loader jar SHA-256 does not match the supplied file",),
    ),
    Case(
        name="nested_flat_sha_inconsistent",
        reference={
            **_base_reference(),
            "game_sha256": SYNTH_A,
            "game": {**_base_reference()["game"], "sha256": "9" * 64},  # contradicts flat
        },
        jars=("gameJar", "modTheSpireJar"),
        expected_tags=("game.sha256 and game_sha256 disagree",),
    ),
    Case(
        name="populated_but_sourceless",
        reference=_base_reference(),
        jars=("gameJar", "modTheSpireJar"),
        # status verified, hashes claimed-consistent, targets present, jars present,
        # but the subproject has NO capture source. Hash of the synthetic files is
        # recorded separately and will not match SYNTH_A/SYNTH_B, so we ALSO expect
        # the source-less blocker regardless of hash outcome.
        expected_tags=("capture source (implemented capture mod) is not present",),
    ),
]


def _materialize_jars(project: Path, names: tuple[str, ...]) -> None:
    """Create clearly-synthetic text 'jar' files (NOT game assets).

    The readiness predicate computes the real digest of these files and compares
    it to the claimed hash; since the claimed hashes are SYNTH_A/… and the file
    contents are this fixed text, the digests deliberately differ — that is the
    point of the hash-mismatch cases.
    """
    names = set(names)
    if "gameJar" in names:
        (project / "synthetic-game.jar").write_text(
            "SYNTHETIC diagnostic input — NOT a Slay the Spire jar\n")
    if "modTheSpireJar" in names:
        (project / "synthetic-loader.jar").write_text(
            "SYNTHETIC diagnostic input — NOT a loader jar\n")


def _copy_project(dest: Path) -> Path:
    shutil.copytree(PROJECT_ROOT, dest, ignore=shutil.ignore_patterns(
        ".gradle", "build", ".git"))
    return dest


def _write_reference(project: Path, reference: dict | None) -> Path:
    """Write the per-case fixture into the disposable project (never the repo).

    Returns the path the readiness predicate is pointed at (-PreferenceBuild).
    """
    doc = project / "docs" / "validation" / "reference-build.json"
    doc.parent.mkdir(parents=True, exist_ok=True)
    if reference is None:
        if doc.exists():
            doc.unlink()
    else:
        doc.write_text(json.dumps(reference, indent=2) + "\n")
    return doc


def _run_readiness(project: Path, jars: tuple[str, ...],
                   reference_build: Path, tmp_path: Path):
    ghome = tmp_path / "gradle-user-home"
    env = {
        "JAVA_HOME": str(TOOL_JDK),
        "GRADLE_USER_HOME": str(ghome),
        "PATH": f"{TOOL_JDK / 'bin'}:" + __import__("os").environ.get("PATH", ""),
        "GRADLE_OPTS": "",
    }
    props = ["-PreferenceBuild=" + str(reference_build)]
    if "gameJar" in jars:
        props += ["-PgameJar=" + str(project / "synthetic-game.jar")]
    if "modTheSpireJar" in jars:
        props += ["-PmodTheSpireJar=" + str(project / "synthetic-loader.jar")]
    cmd = [str(TOOL_GRADLE), "--no-daemon", "--console=plain",
           "clean", ":mod:referenceReadiness", *props]
    proc = subprocess.run(cmd, cwd=project, env=env,
                          capture_output=True, text=True, timeout=300)
    return proc


@pytest.mark.skipif(not _tools_present(),
                    reason="JDK/Gradle toolchain not available (tools/jdk17, tools/gradle-8.14.6)")
@pytest.mark.parametrize("case", CASES, ids=lambda c: c.name)
def test_mod_readiness_is_blocked_for_synthetic_case(
        case: Case, tmp_path: Path, _require_tools: None) -> None:
    project = _copy_project(tmp_path / "proj")
    reference_build = _write_reference(project, case.reference)
    _materialize_jars(project, case.jars)

    proc = _run_readiness(project, case.jars, reference_build, tmp_path)
    combined = (proc.stdout or "") + (proc.stderr or "")

    assert proc.returncode != 0, (
        "readiness unexpectedly PASSED for %s — an empty source-less "
        "mod artifact must stay blocked\nSTDOUT:\n%s\nSTDERR:\n%s"
        % (case.name, proc.stdout, proc.stderr)
    )
    for expected in case.expected_tags:
        assert expected in combined, (
            "expected diagnostic tag %r for case %s missing\n"
            "STDOUT:\n%s\nSTDERR:\n%s"
            % (expected, case.name, proc.stdout, proc.stderr)
        )
    assert TAG_PREFIX in combined, "readiness diagnostics missing stable prefix"

    libs = project / "mod" / "build" / "libs"
    assert not libs.exists() or any(libs.iterdir()) is False, (
        "a mod jar artifact was produced for synthetic case %s — not allowed" % case.name
    )


@pytest.mark.skipif(not _tools_present(),
                    reason="JDK/Gradle toolchain not available (tools/jdk17, tools/gradle-8.14.6)")
def test_committed_reference_stays_pending(_require_tools: None) -> None:
    """The committed pending reference must never be silently promoted."""
    data = json.loads(COMMITTED_REFERENCE.read_text())
    assert data["status"] == "pending"
    assert data["verified"] is False
    assert data["game_sha256"] is None
    assert data["loader_sha256"] is None
