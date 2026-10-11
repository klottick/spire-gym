"""Fail-closed :mod readiness tests — I7 follow-up (S1–S4).

Covers the full diagnostic matrix with *real* synthetic diagnostic files:
a valid "otherwise correct" baseline whose hashes actually match the diagnostic
file digests, and one-requirement-at-a-time violations that each surface their
own ``[MOD-READINESS]`` tag.  The integration-pending gate is asserted
**unconditionally** (source-less, dummy-Java, and artifact-path cases all
blocked the same way) and the ``:mod:jar`` / ``:mod:assemble`` / ``:mod:build``
paths are exercised in disposable copies (not just ``referenceReadiness``).

Run with::

    uv run --no-sync pytest validation/windows-mod/tests/test_mod_readiness.py -s

The tools dir (JDK + Gradle) must be present; the test skips rather than
silently passes when they are not.
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
PROJECT_ROOT = REPO_ROOT / "validation" / "windows-mod"
COMMITTED_REFERENCE = REPO_ROOT / "docs" / "validation" / "reference-build.json"

TOOL_JDK = REPO_ROOT / "tools" / "jdk17"
TOOL_GRADLE = REPO_ROOT / "tools" / "gradle-8.14.6" / "bin" / "gradle"
LOCAL_JDK_MAJOR = "17"   # tools/jdk17

# Two clearly-synthetic text diagnostics — NOT game assets. Their digests are
# used as the "valid" reference in the baseline, so the readiness predicate can
# verify them against real files while we isolate one requirement at a time.
GAME_DIAG   = "SYNTHETIC diagnostic game jar — not a real Slay the Spire byte stream\n"
LOADER_DIAG = "SYNTHETIC diagnostic loader jar — not a real Mod The Spire byte stream\n"
HASH_A = hashlib.sha256(GAME_DIAG.encode("utf-8")).hexdigest()
HASH_B = hashlib.sha256(LOADER_DIAG.encode("utf-8")).hexdigest()

DUMMY_JAVA = "package spirelab;\npublic class DummyCaptureScaffold {\n}\n"

# The one tag the predicate must emit in every case — unconditional.
INTEGRATION_TAG = "reference-dependent capture integration is pending"
TAG_PREFIX = "MOD-READINESS"


def _tools_present() -> bool:
    return TOOL_JDK.is_dir() and TOOL_GRADLE.is_file()


@pytest.fixture(scope="module")
def _require_tools():
    assert _tools_present(), (
        "a JDK (tools/jdk17) and Gradle (tools/gradle-8.14.6) toolchain is "
        "required for these readiness tests"
    )


def _base() -> dict:
    """Valid baseline: matching digests, valid schema/status/verified, nonblank
    versions, valid compatible targets, no optional aliases. The predicate
    should fail ONLY on the unconditional integration-pending gate."""
    return {
        "schema": "spire-lab/reference-build/1",
        "status": "verified",
        "verified": True,
        "game": {
            "expected": "synthetic diagnostic game — not a game asset",
            "version": "synthetic-game-1",
            "sha256": HASH_A,
            "evidence_required": "synthetic diagnostic input for a readiness test",
        },
        "loader": {
            "expected": "synthetic diagnostic loader — not a game asset",
            "version": "synthetic-loader-1",
            "sha256": HASH_B,
            "evidence_required": "synthetic diagnostic input for a readiness test",
        },
        "game_sha256": HASH_A,
        "loader_sha256": HASH_B,
        "runtime_target": LOCAL_JDK_MAJOR,
        "compiler_target": LOCAL_JDK_MAJOR,
        "notes": "SYNTHETIC diagnostic input — never a verified game build",
    }


def _omit(d: dict, *keys: str) -> dict:
    out = {k: v for k, v in d.items() if k not in keys}
    return out


@dataclass
class Case:
    name: str
    reference: dict | None          # dict (may be mutated) or None (file absent)
    expected_tag: str               # substring the predicate must emit for this case
    run_task: str = "referenceReadiness"
    assert_only_integration: bool = False
    check_no_artifact: bool = False
    check_no_compiled: bool = False
    add_dummy_java: bool = False
    omit_jars: bool = False


CASES: list[Case] = [
    # S1 — required-field validation
    Case("missing_reference_file",  reference=None,
         expected_tag="reference-build.json is missing"),
    Case("jars_not_provided",       reference=_base(),
         omit_jars=True,
         expected_tag="game jar is not provided"),
    Case("status_pending",          reference={**_base(), "status": "pending"},
         expected_tag='status is "pending"'),
    Case("verified_false",          reference={**_base(), "verified": False},
         expected_tag="verified must be boolean true"),
    Case("schema_unsupported",      reference={**_base(), "schema": "spire-lab/reference-build/99"},
         expected_tag="schema"),
    Case("schema_missing",          reference=_omit(_base(), "schema"),
         expected_tag="schema"),
    Case("game_version_missing",    reference={**_base(), "game": {**_base()["game"], "version": None}},
         expected_tag="game.version"),
    Case("loader_version_missing",  reference={**_base(), "loader": {**_base()["loader"], "version": ""}},
         expected_tag="loader.version"),
    Case("game_sha256_malformed",   reference={**_base(), "game": {**_base()["game"], "sha256": "zzz"}},
         expected_tag="game.sha256"),
    Case("loader_sha256_malformed", reference={**_base(), "loader": {**_base()["loader"], "sha256": "xx"}},
         expected_tag="loader.sha256"),
    Case("game_flat_malformed",     reference={**_base(), "game_sha256": "nothex"},
         expected_tag="game_sha256"),
    Case("nested_flat_sha_inconsistent",
         reference={**_base(),
                    "game_sha256": "9" * 64,
                    "game": {**_base()["game"], "sha256": HASH_A}},
         expected_tag="game.sha256 and game_sha256 disagree"),
    Case("game_version_alias_mismatch",
         reference={**_base(), "game_version": "alias-mismatch"},
         expected_tag="game_version differs from game.version"),

    # S2 — target representation / compatibility
    Case("runtime_target_junk",     reference={**_base(), "runtime_target": "not-a-JVM-target"},
         expected_tag="runtime_target must be a positive integer"),
    Case("runtime_target_zero",     reference={**_base(), "runtime_target": "0"},
         expected_tag="is zero or negative"),
    Case("runtime_target_fraction", reference={**_base(), "runtime_target": "1.5"},
         expected_tag="runtime_target must be a positive integer"),
    Case("runtime_target_leading_zero",
         reference={**_base(), "runtime_target": "017"},
         expected_tag="leading zero"),
    Case("compiler_gt_runtime",
         reference={**_base(), "compiler_target": "21", "runtime_target": "17"},
         expected_tag="compiler_target is incompatible with runtime_target"),
    Case("compiler_gt_local_jdk",
         reference={**_base(), "compiler_target": "19", "runtime_target": "19"},
         expected_tag="compiler_target is higher than the local JDK"),

    # Digest / actual-file mismatch (S1)
    Case("game_hash_mismatch_file",
         reference={**_base(), "game_sha256": "c" * 64, "game": {**_base()["game"], "sha256": "c" * 64}},
         expected_tag="game jar SHA-256 does not match the supplied file"),
    Case("loader_hash_mismatch_file",
         reference={**_base(), "loader_sha256": "d" * 64, "loader": {**_base()["loader"], "sha256": "d" * 64}},
         expected_tag="loader jar SHA-256 does not match the supplied file"),

    # S3 — unconditional integration-pending gate
    Case("valid_baseline_integration_only",
         reference=_base(),
         expected_tag=INTEGRATION_TAG,
         assert_only_integration=True),
    Case("dummy_java_still_blocked",
         reference=_base(),
         add_dummy_java=True,
         expected_tag=INTEGRATION_TAG,
         assert_only_integration=True),

    # S4 — actual artifact-task gating
    Case("jar_path_valid_baseline",
         reference=_base(),
         run_task="jar",
         expected_tag=INTEGRATION_TAG,
         check_no_artifact=True, check_no_compiled=True),
    Case("assemble_path_valid_baseline",
         reference=_base(),
         run_task="assemble",
         expected_tag=INTEGRATION_TAG,
         check_no_artifact=True),
    Case("build_path_valid_baseline",
         reference=_base(),
         run_task="build",
         expected_tag=INTEGRATION_TAG,
         check_no_artifact=True),
    Case("jar_path_invalid_metadata",
         reference={**_base(), "verified": False},
         run_task="jar",
         expected_tag="verified must be boolean true",
         check_no_artifact=True),
]


def _copy_project(dest: Path) -> Path:
    shutil.copytree(PROJECT_ROOT, dest, ignore=shutil.ignore_patterns(
        ".gradle", "build", ".git"))
    return dest


def _write_reference(project: Path, reference: dict | None) -> Path:
    doc = project / "docs" / "validation" / "reference-build.json"
    doc.parent.mkdir(parents=True, exist_ok=True)
    if reference is None:
        if doc.exists():
            doc.unlink()
    else:
        doc.write_text(json.dumps(reference, indent=2) + "\n")
    return doc


def _materialize_jars(project: Path) -> None:
    (project / "synthetic-game.diagnostic").write_text(GAME_DIAG)
    (project / "synthetic-loader.diagnostic").write_text(LOADER_DIAG)


def _add_dummy_java(project: Path) -> None:
    p = project / "mod" / "src" / "main" / "java" / "spirelab"
    p.mkdir(parents=True, exist_ok=True)
    (p / "Dummy.java").write_text(DUMMY_JAVA)


def _run_gradle(project: Path, task: str,
                game_jar: Path | None, mod_jar: Path | None,
                reference_build: Path, tmp_path: Path) -> subprocess.CompletedProcess:
    ghome = tmp_path / "gradle-user-home"
    env = {
        "JAVA_HOME": str(TOOL_JDK),
        "GRADLE_USER_HOME": str(ghome),
        "PATH": f"{TOOL_JDK / 'bin'}:" + __import__("os").environ.get("PATH", ""),
        "GRADLE_OPTS": "",
    }
    props = [f"-PreferenceBuild={reference_build}"]
    if game_jar is not None:
        props.append(f"-PgameJar={game_jar}")
    if mod_jar is not None:
        props.append(f"-PmodTheSpireJar={mod_jar}")
    cmd = [str(TOOL_GRADLE), "--no-daemon", "--console=plain",
           "clean", f":mod:{task}", *props]
    return subprocess.run(cmd, cwd=project, env=env,
                          capture_output=True, text=True, timeout=300)


def _artifacts_absent(project: Path) -> str:
    libs = project / "mod" / "build" / "libs"
    if libs.exists():
        for p in libs.iterdir():
            if p.name.endswith(".jar"):
                return f"unexpected artifact: {p}"
    return ""


@pytest.mark.skipif(not _tools_present(),
                    reason="JDK/Gradle toolchain not available (tools/jdk17, tools/gradle-8.14.6)")
@pytest.mark.parametrize("case", CASES, ids=lambda c: c.name)
def test_mod_readiness_is_blocked_for_synthetic_case(case: Case,
        tmp_path: Path, _require_tools: None) -> None:
    project = _copy_project(tmp_path / "proj")
    reference_build = _write_reference(project, case.reference)
    game_jar = None
    mod_jar = None
    if not case.omit_jars:
        _materialize_jars(project)
        game_jar = project / "synthetic-game.diagnostic"
        mod_jar = project / "synthetic-loader.diagnostic"
    if case.add_dummy_java:
        _add_dummy_java(project)

    proc = _run_gradle(project, case.run_task, game_jar, mod_jar,
                       reference_build, tmp_path)
    combined = (proc.stdout or "") + (proc.stderr or "")

    assert proc.returncode != 0, (
        f"{case.name}: :mod:{case.run_task} unexpectedly PASSED\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )
    assert TAG_PREFIX in combined, (
        f"{case.name}: no [MOD-READINESS] diagnostic in output\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )
    assert case.expected_tag in combined, (
        f"{case.name}: expected diagnostic tag {case.expected_tag!r} missing\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )

    if case.assert_only_integration:
        # Count the [MOD-READINESS] diagnostics in the output — there should be
        # exactly one (the integration-pending gate), proving nothing else fired.
        matches = re.findall(r"\[MOD-READINESS\][^\n]*", combined)
        relevant = [m for m in matches if "reference-dependent capture integration" in m]
        assert len(relevant) >= 1, f"{case.name}: no integration-pending diagnostic"
        others = [m for m in matches if "reference-dependent capture integration" not in m]
        assert not others, (
            f"{case.name}: expected ONLY the integration-pending diagnostic, but "
            f"found additional blockers:\n"
            + "\n".join(others)
        )

    if case.check_no_artifact:
        bad = _artifacts_absent(project)
        assert bad == "", f"{case.name}: {bad}"
    if case.check_no_compiled:
        main_classes = project / "mod" / "build" / "classes" / "java" / "main"
        assert not main_classes.exists() or list(main_classes.rglob("*.class")) == [], (
            f"{case.name}: mod compileJava ran (gated compile must not happen)"
        )


@pytest.mark.skipif(not _tools_present(),
                    reason="JDK/Gradle toolchain not available (tools/jdk17, tools/gradle-8.14.6)")
def test_committed_reference_stays_pending(_require_tools: None) -> None:
    data = json.loads(COMMITTED_REFERENCE.read_text())
    assert data["status"] == "pending"
    assert data["verified"] is False
    assert data["game_sha256"] is None
    assert data["loader_sha256"] is None
    assert data["runtime_target"] is None
    assert data["compiler_target"] is None
