"""Fail-closed :mod readiness tests — I7 follow-up 2 (R1–R4).

Drives the REAL :mod Gradle predicate and its artifact/compile paths in
disposable project copies (fresh isolated GRADLE_USER_HOME, no cache reuse).
Covers (one requirement at a time, with its *specific* diagnostic):

  R1 optional version aliases by KEY PRESENCE (null / wrong type / blank /
     mismatch, both game and loader);
  R2 canonical targets (ASCII positive int, not Unicode digits), the pinned
     local compiler's supported --release set, the current protocol release
     floor, compiler<=runtime, and the actual JavaCompile binding
     (release=declared, toolchain=pinned JDK 17) inspected via
     :mod:inspectCompileTarget while the integration gate stays blocked;
  R3 every required field independently (objects, nested SHA, aliases, targets);
  R4 a VALID source-bearing scaffold through compileJava/jar/assemble/build
     (asserting blocked + no class + no artifact), a corrected scaffold file
     name matching its class, and a *mutant* proof that removing only the
     compileJava gate is what lets the harmless scaffold compile.

All inputs are clearly-synthetic text diagnostics (never game assets) with
matching real digests for the otherwise-valid baseline. The committed
docs/validation/reference-build.json is NEVER mutated.

Run:
    uv run --no-sync pytest validation/windows-mod/tests/test_mod_readiness.py -s
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
LOCAL_JDK_MAJOR = "17"

# Two synthetic text diagnostics (clearly NOT game assets); their real digests
# are the "matching" values used in the valid baseline.
GAME_DIAG = "SYNTHETIC diagnostic game jar — not a real Slay the Spire byte stream\n"
LOADER_DIAG = "SYNTHETIC diagnostic loader jar — not a real Mod The Spire byte stream\n"
HASH_A = hashlib.sha256(GAME_DIAG.encode("utf-8")).hexdigest()
HASH_B = hashlib.sha256(LOADER_DIAG.encode("utf-8")).hexdigest()

# R4: a VALID, harmless Java scaffold whose file name matches its public class.
DUMMY_SRC = "package spirelab;\npublic class DummyCaptureScaffold {\n}\n"
DUMMY_SRC_REL = Path("mod/src/main/java/spirelab/DummyCaptureScaffold.java")

INTEGRATION_TAG = "reference-dependent capture integration is pending"
TAG_PREFIX = "MOD-READINESS"


def _tools_present() -> bool:
    return TOOL_JDK.is_dir() and TOOL_GRADLE.is_file()


@pytest.fixture(scope="module")
def _require_tools():
    assert _tools_present(), "tools/jdk17 and tools/gradle-8.14.6 are required"


def _base() -> dict:
    return {
        "schema": "spire-lab/reference-build/1",
        "status": "verified",
        "verified": True,
        "game": {"expected": "synthetic diagnostic game — not a game asset",
                 "version": "synthetic-game-1", "sha256": HASH_A,
                 "evidence_required": "synthetic diagnostic input"},
        "loader": {"expected": "synthetic diagnostic loader — not a game asset",
                   "version": "synthetic-loader-1", "sha256": HASH_B,
                   "evidence_required": "synthetic diagnostic input"},
        "game_sha256": HASH_A,
        "loader_sha256": HASH_B,
        "runtime_target": LOCAL_JDK_MAJOR,
        "compiler_target": LOCAL_JDK_MAJOR,
        "notes": "SYNTHETIC diagnostic input — never a verified game build",
    }


@dataclass
class Case:
    name: str
    reference: dict | None
    expected_tag: str
    extra_substrings: tuple[str, ...] = ()
    run_task: str = "referenceReadiness"
    assert_only_integration: bool = False
    check_no_artifact: bool = False
    check_no_class: bool = False
    add_dummy_source: bool = False
    omit_jars: bool = False
    expect_success: bool = False     # only the mutant case
    remove_gate: str | None = None   # R4 mutant: gate line to strip


CASES: list[Case] = [
    # --- reference + jars ---
    Case("missing_reference_file", None, "reference-build.json is missing"),
    Case("jars_not_provided", _base(), "game jar is not provided", omit_jars=True),
    # --- S1 status/schema/verified ---
    Case("status_pending", {**_base(), "status": "pending"}, 'status is "pending"'),
    Case("verified_false", {**_base(), "verified": False}, "verified must be boolean true"),
    Case("schema_unsupported", {**_base(), "schema": "spire-lab/reference-build/99"}, "schema is not supported"),
    Case("schema_missing", {k: v for k, v in _base().items() if k != "schema"}, "schema is not supported"),
    # --- R3 game/loader object required + shape (both sides) ---
    Case("game_object_missing", {k: v for k, v in _base().items() if k != "game"}, "game object is required"),
    Case("game_object_wrong_type", {**_base(), "game": "not-an-object"}, "game must be a JSON object"),
    Case("loader_object_missing", {k: v for k, v in _base().items() if k != "loader"}, "loader object is required"),
    Case("loader_object_wrong_type", {**_base(), "loader": "not-an-object"}, "loader must be a JSON object"),
    Case("game_version_missing", {**_base(), "game": {**_base()["game"], "version": None}}, "game.version"),
    Case("loader_version_missing", {**_base(), "loader": {**_base()["loader"], "version": ""}}, "loader.version"),
    # --- R3 nested SHA missing / wrong type / malformed (both sides) ---
    Case("game_nested_sha256_missing", {**_base(), "game": {**_base()["game"], "sha256": None}},
         "game.sha256 is not a 64-char lowercase hex SHA-256"),
    Case("game_nested_sha256_wrong_type", {**_base(), "game": {**_base()["game"], "sha256": 12345}},
         "game.sha256 is not a 64-char lowercase hex SHA-256"),
    Case("loader_nested_sha256_missing", {**_base(), "loader": {**_base()["loader"], "sha256": None}},
         "loader.sha256 is not a 64-char lowercase hex SHA-256"),
    Case("loader_nested_sha256_wrong_type", {**_base(), "loader": {**_base()["loader"], "sha256": "xy"}},
         "loader.sha256 is not a 64-char lowercase hex SHA-256"),
    Case("game_flat_sha256_malformed", {**_base(), "game_sha256": "nothex"},
         "game_sha256 is not a 64-char lowercase hex SHA-256"),
    Case("loader_flat_sha256_malformed", {**_base(), "loader_sha256": 999},
         "loader_sha256 is not a 64-char lowercase hex SHA-256"),
    # --- R3: hash contradiction via NESTED change only (flat stays = diagnostic file) ---
    Case("game_nested_sha256_contradicting_flat",
         {**_base(), "game": {**_base()["game"], "sha256": "9" * 64}},
         "game.sha256 and game_sha256 disagree"),
    Case("loader_nested_sha256_contradicting_flat",
         {**_base(), "loader": {**_base()["loader"], "sha256": "f" * 64}},
         "loader.sha256 and loader_sha256 disagree"),
    # --- R1 optional aliases by KEY PRESENCE (both sides) ---
    Case("game_version_null_alias", {**_base(), "game_version": None}, "game_version is supplied as null"),
    Case("game_version_wrong_type_alias", {**_base(), "game_version": 1234}, "game_version if supplied must be a string"),
    Case("game_version_blank_alias", {**_base(), "game_version": "  "}, "game_version if supplied must be a nonblank string"),
    Case("game_version_contradictory_alias", {**_base(), "game_version": "alias-mismatch"},
         "game_version differs from game.version"),
    Case("loader_version_null_alias", {**_base(), "loader_version": None}, "loader_version is supplied as null"),
    Case("loader_version_wrong_type_alias", {**_base(), "loader_version": 555}, "loader_version if supplied must be a string"),
    Case("loader_version_contradictory_alias", {**_base(), "loader_version": "other"},
         "loader_version differs from loader.version"),
    # --- S2/R2 canonical target form + constraints ---
    Case("runtime_target_missing", {k: v for k, v in _base().items() if k != "runtime_target"},
         "runtime_target must be a positive decimal integer string"),
    Case("runtime_target_null", {**_base(), "runtime_target": None},
         "runtime_target must be a positive decimal integer string"),
    Case("runtime_target_wrong_type", {**_base(), "runtime_target": 17},
         "runtime_target must be a positive decimal integer string"),
    Case("runtime_target_junk", {**_base(), "runtime_target": "not-a-JVM-target"},
         "must be a positive decimal integer"),
    Case("runtime_target_zero", {**_base(), "runtime_target": "0"}, "is zero or negative"),
    Case("runtime_target_fraction", {**_base(), "runtime_target": "1.5"},
         "must be a positive decimal integer"),
    Case("runtime_target_leading_zero", {**_base(), "runtime_target": "017"}, "leading zero"),
    Case("runtime_target_unicode_digit", {**_base(), "runtime_target": "٠١٧"},
         "must be a positive decimal integer"),
    Case("runtime_target_overflow", {**_base(), "runtime_target": "99999999999999999999"},
         "not a valid positive integer (overflow?)"),
    Case("compiler_target_missing", {k: v for k, v in _base().items() if k != "compiler_target"},
         "compiler_target must be a positive decimal integer string"),
    Case("compiler_target_wrong_type", {**_base(), "compiler_target": ["7"]},
         "compiler_target must be a positive decimal integer string"),
    Case("compiler_release_4_unsupported", {**_base(), "compiler_target": "4", "runtime_target": "17"},
         "compiler_target (4) is not a release supported"),
    Case("compiler_release_19_unsupported", {**_base(), "compiler_target": "19"},
         "compiler_target (19) is not a release supported"),
    Case("runtime_11_below_protocol", {**_base(), "runtime_target": "11", "compiler_target": "11"},
         "runtime_target (11) is below the current protocol bytecode release"),
    Case("compiler_gt_runtime_incompatible", {**_base(), "runtime_target": "11", "compiler_target": "12"},
         "compiler_target (12) is incompatible with runtime_target (11)"),
    # --- R1 retain actual-file digest mismatch (both sides) ---
    Case("game_hash_mismatch_file", {**_base(), "game_sha256": "c" * 64,
         "game": {**_base()["game"], "sha256": "c" * 64}}, "game jar SHA-256 does not match the supplied file"),
    Case("loader_hash_mismatch_file", {**_base(), "loader_sha256": "d" * 64,
         "loader": {**_base()["loader"], "sha256": "d" * 64}}, "loader jar SHA-256 does not match the supplied file"),
    # --- S3 unconditional integration-only baseline ---
    Case("valid_baseline_integration_only", _base(), INTEGRATION_TAG,
         assert_only_integration=True),
    # --- R2 valid target binding inspected while gate stays blocked ---
    Case("valid_target_inspect_compile",
         {**_base(), "compiler_target": "11", "runtime_target": "17"},
         "options.release=11",
         extra_substrings=("toolchain.languageVersion=17",),
         run_task="inspectCompileTarget", expect_success=True),
    # --- R3/R4 valid source-bearing scaffold through artifact/compile paths ---
    Case("valid_dummy_compileJava_gated", _base(), INTEGRATION_TAG,
         run_task="compileJava", add_dummy_source=True, check_no_class=True, check_no_artifact=True),
    Case("valid_dummy_jar_gated", _base(), INTEGRATION_TAG,
         run_task="jar", add_dummy_source=True, check_no_class=True, check_no_artifact=True),
    Case("valid_dummy_assemble_gated", _base(), INTEGRATION_TAG,
         run_task="assemble", add_dummy_source=True, check_no_class=True, check_no_artifact=True),
    Case("valid_dummy_build_gated", _base(), INTEGRATION_TAG,
         run_task="build", add_dummy_source=True, check_no_class=True, check_no_artifact=True),
    # --- R4 mutant: removing only the compileJava gate lets the scaffold compile ---
    Case("mutant_gate_removed_scaffold_compiles", _base(), "build/classes",
         run_task="compileJava", add_dummy_source=True, expect_success=True,
         remove_gate='tasks.named("compileJava") { dependsOn(referenceReadiness) }\n'),
    # --- jar path with invalid metadata (gate still applies) ---
    Case("jar_path_invalid_metadata", {**_base(), "verified": False},
         "verified must be boolean true", run_task="jar", check_no_artifact=True),
]


def _copy_project(dest: Path) -> Path:
    return shutil.copytree(PROJECT_ROOT, dest,
                           ignore=shutil.ignore_patterns(".gradle", "build", ".git"))


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


def _add_dummy_source(project: Path) -> None:
    p = project / DUMMY_SRC_REL
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(DUMMY_SRC)


def _remove_gate(project: Path, line: str) -> None:
    script = project / "mod" / "build.gradle.kts"
    text = script.read_text()
    assert line in text, f"gate line to remove not found in disposable copy: {line!r}"
    script.write_text(text.replace(line, "", 1))


def _run(project: Path, task: str, game_jar, mod_jar, reference_build, tmp_path):
    ghome = tmp_path / "gradle-user-home"
    env = {"JAVA_HOME": str(TOOL_JDK), "GRADLE_USER_HOME": str(ghome),
           "PATH": f"{TOOL_JDK / 'bin'}:" + __import__("os").environ.get("PATH", ""),
           "GRADLE_OPTS": ""}
    props = [f"-PreferenceBuild={reference_build}"]
    if game_jar:
        props.append(f"-PgameJar={game_jar}")
    if mod_jar:
        props.append(f"-PmodTheSpireJar={mod_jar}")
    cmd = [str(TOOL_GRADLE), "--no-daemon", "--console=plain", "clean", f":mod:{task}", *props]
    return subprocess.run(cmd, cwd=project, env=env, capture_output=True, text=True, timeout=300)


def _artifacts(project: Path) -> list[str]:
    libs = project / "mod" / "build" / "libs"
    if not libs.exists():
        return []
    return [p.name for p in libs.iterdir() if p.name.endswith(".jar")]


def _classes(project: Path) -> list[str]:
    main = project / "mod" / "build" / "classes" / "java" / "main"
    if not main.exists():
        return []
    return [str(p.relative_to(project)) for p in main.rglob("*.class")]


@pytest.mark.skipif(not _tools_present(), reason="JDK/Gradle toolchain not available")
@pytest.mark.parametrize("case", CASES, ids=lambda c: c.name)
def test_mod_readiness_case(case: Case, tmp_path: Path, _require_tools: None) -> None:
    project = _copy_project(tmp_path / "proj")
    reference_build = _write_reference(project, case.reference)
    game_jar = mod_jar = None
    if not case.omit_jars:
        _materialize_jars(project)
        game_jar, mod_jar = (project / "synthetic-game.diagnostic",
                            project / "synthetic-loader.diagnostic")
    if case.add_dummy_source:
        _add_dummy_source(project)
    if case.remove_gate:
        _remove_gate(project, case.remove_gate)

    proc = _run(project, case.run_task, game_jar, mod_jar, reference_build, tmp_path)
    combined = (proc.stdout or "") + (proc.stderr or "")

    if case.expect_success:
        assert proc.returncode == 0, (
            f"{case.name}: expected SUCCESS (exit 0)\n{proc.stdout}\n{proc.stderr}")
        if case.run_task == "compileJava":
            assert _classes(project), (
                f"{case.name}: the harmless scaffold should compile (class produced) once the "
                f"gate is removed — proves the gate is what blocked it\n{proc.stdout}"
            )
        for sub in case.extra_substrings:
            assert sub in combined, f"{case.name}: missing {sub!r}\n{combined}"
        return

    # Blocked expectation.
    assert proc.returncode != 0, (
        f"{case.name}: :mod:{case.run_task} unexpectedly PASSED\n{proc.stdout}\n{proc.stderr}")
    assert TAG_PREFIX in combined, f"{case.name}: no [MOD-READINESS] tag\n{combined}"
    assert case.expected_tag in combined, (
        f"{case.name}: expected diagnostic tag {case.expected_tag!r} missing\n{combined}")
    for sub in case.extra_substrings:
        assert sub in combined, f"{case.name}: missing substring {sub!r}\n{combined}"

    if case.assert_only_integration:
        relevant = [m for m in re.findall(r"\[MOD-READINESS\][^\n]*", combined)
                    if "reference-dependent capture integration" in m]
        others = [m for m in re.findall(r"\[MOD-READINESS\][^\n]*", combined)
                  if "reference-dependent capture integration" not in m]
        assert relevant, f"{case.name}: no integration-pending diagnostic\n{combined}"
        assert not others, f"{case.name}: expected ONLY integration blocker, found:\n" + "\n".join(others)

    if case.check_no_artifact:
        assert not _artifacts(project), (
            f"{case.name}: a :mod jar artifact was produced: {_artifacts(project)}")
    if case.check_no_class:
        assert not _classes(project), (
            f"{case.name}: mod compileJava produced classes (gated compile must not run): "
            f"{_classes(project)}")


@pytest.mark.skipif(not _tools_present(), reason="JDK/Gradle toolchain not available")
def test_committed_reference_stays_pending(_require_tools: None) -> None:
    data = json.loads(COMMITTED_REFERENCE.read_text())
    assert data["status"] == "pending"
    assert data["verified"] is False
    assert data["game_sha256"] is None
    assert data["loader_sha256"] is None
    assert data["runtime_target"] is None
    assert data["compiler_target"] is None
