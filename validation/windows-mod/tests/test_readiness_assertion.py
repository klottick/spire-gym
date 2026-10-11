"""N1/F3 assertion-sensitivity regression — test-only, std-lib ONLY (no Gradle).

Why: follow-up 3's N1 showed the matrix's SUCCESS branch ignored the case's
declared expectation (options.release=11), and F3 requires the blocked
compile/artifact paths to prove the compile action never EXECUTED (a task
graph listing alone does not). This file covers the extracted checker
`test_mod_readiness._assert_case_result` directly against CONTROLLED
`subprocess.CompletedProcess` outputs, so the assertions themselves — not
only the Gradle wiring — are regression-covered:

  * N1: a successful inspection output lacking the expected release
    (release=null or a wrong release) is REJECTED; the correct output
    (options.release=11 + pinned compiler 17) is ACCEPTED;
  * F3: for sentinel-injected blocked paths, the compile-execution sentinel in
    the output is REJECTED (the action must not have run) and its absence is
    accepted;
  * F3 positive control: the mutant success requires the sentinel AND the
    compiled DummyCaptureScaffold.class.

This file does not invoke Gradle, does not materialize or mutate any project,
does not touch production code, and invents no game facts.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

import pytest

import test_mod_readiness as T


def _case(name: str) -> T.Case:
    for c in T.CASES:
        if c.name == name:
            return c
    raise KeyError(f"case not found in matrix: {name}")


def _proc(rc: int, stdout: str = "", stderr: str = "") -> "subprocess.CompletedProcess":
    return subprocess.CompletedProcess(args=["gradle-control-output"], returncode=rc,
                                       stdout=stdout, stderr=stderr)


# --- controlled outputs (exact shapes as printed by the accepted build) ---
INSPECT_OK = (
    "inspectCompileTarget: options.release=11 (declared=11)\n"
    "inspectCompileTarget: toolchain.languageVersion=17 (pinned build JDK 17)\n"
    "BUILD SUCCESSFUL in 9s\n"
)
INSPECT_RELEASE_NULL = (
    "inspectCompileTarget: options.release=null (declared=11)\n"
    "inspectCompileTarget: toolchain.languageVersion=17 (pinned build JDK 17)\n"
    "BUILD SUCCESSFUL in 9s\n"
)
INSPECT_WRONG_RELEASE = (
    "inspectCompileTarget: options.release=17 (declared=11)\n"
    "inspectCompileTarget: toolchain.languageVersion=17 (pinned build JDK 17)\n"
    "BUILD SUCCESSFUL in 9s\n"
)
BLOCKED_INTEGRATION = (
    "> Task :mod:referenceReadiness FAILED\n"
    "Execution failed for task ':mod:referenceReadiness'.\n"
    "> capture mod readiness FAILED (fail-closed):\n"
    "    [MOD-READINESS] reference-dependent capture integration is pending — this subproject has NO real implementation\n"
    "BUILD FAILED in 9s\n"
)


@pytest.fixture()
def empty_project(tmp_path: Path) -> Path:
    p = tmp_path / "proj"
    p.mkdir()
    return p


# --- N1: the successful inspection must assert its expected release ---
def test_inspect_success_rejects_missing_release(empty_project: Path) -> None:
    c = _case("valid_target_inspect_compile")
    with pytest.raises(AssertionError, match="options.release=11"):
        T._assert_case_result(c, _proc(0, INSPECT_RELEASE_NULL), empty_project)


def test_inspect_success_rejects_wrong_release(empty_project: Path) -> None:
    c = _case("valid_target_inspect_compile")
    with pytest.raises(AssertionError, match="options.release=11"):
        T._assert_case_result(c, _proc(0, INSPECT_WRONG_RELEASE), empty_project)


def test_inspect_success_accepts_correct_release_and_compiler(empty_project: Path) -> None:
    c = _case("valid_target_inspect_compile")
    T._assert_case_result(c, _proc(0, INSPECT_OK), empty_project)  # must NOT raise


# --- F3: blocked paths must prove the compile action never EXECUTED ---
def test_blocked_gated_case_rejects_executed_sentinel(empty_project: Path) -> None:
    c = _case("valid_dummy_compileJava_gated")
    assert c.inject_compile_sentinel
    with pytest.raises(AssertionError, match="EXECUTED"):
        T._assert_case_result(c, _proc(1, BLOCKED_INTEGRATION + T.COMPILE_SENTINEL + "\n"),
                              empty_project)


def test_blocked_gated_case_accepts_sentinel_absence(empty_project: Path) -> None:
    c = _case("valid_dummy_jar_gated")
    T._assert_case_result(c, _proc(1, BLOCKED_INTEGRATION), empty_project)  # must NOT raise


def test_blocked_case_rejects_produced_artifact(empty_project: Path) -> None:
    c = _case("valid_dummy_jar_gated")
    libs = empty_project / "mod" / "build" / "libs"
    libs.mkdir(parents=True)
    (libs / "mod-0.1.0.jar").write_text("fake jar")
    with pytest.raises(AssertionError, match="artifact"):
        T._assert_case_result(c, _proc(1, BLOCKED_INTEGRATION), empty_project)


# --- F3 positive control: the mutant success needs sentinel AND the class ---
def test_mutant_success_requires_sentinel_and_class(empty_project: Path) -> None:
    c = _case("mutant_gate_removed_scaffold_compiles")
    assert c.expect_success and c.inject_compile_sentinel
    classdir = empty_project / "mod" / "build" / "classes" / "java" / "main" / "spirelab"
    classdir.mkdir(parents=True)
    (classdir / "DummyCaptureScaffold.class").write_bytes(b"fake compiled class")
    ok = "BUILD SUCCESSFUL in 9s\n" + T.COMPILE_SENTINEL + "\n"
    T._assert_case_result(c, _proc(0, ok), empty_project)  # sentinel + class: ACCEPT


def test_mutant_success_rejects_missing_sentinel(empty_project: Path) -> None:
    c = _case("mutant_gate_removed_scaffold_compiles")
    classdir = empty_project / "mod" / "build" / "classes" / "java" / "main" / "spirelab"
    classdir.mkdir(parents=True)
    (classdir / "DummyCaptureScaffold.class").write_bytes(b"fake compiled class")
    with pytest.raises(AssertionError, match="MOD-COMPILE-EXECUTED"):
        T._assert_case_result(c, _proc(0, "BUILD SUCCESSFUL in 9s\n"), empty_project)


def test_mutant_success_rejects_missing_class(empty_project: Path) -> None:
    c = _case("mutant_gate_removed_scaffold_compiles")
    ok = "BUILD SUCCESSFUL in 9s\n" + T.COMPILE_SENTINEL + "\n"
    with pytest.raises(AssertionError, match="should compile"):
        T._assert_case_result(c, _proc(0, ok), empty_project)
