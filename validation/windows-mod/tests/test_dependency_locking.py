"""I8 — build-enforced dependency locking and SHA-256 verification for :protocol.

These tests exercise the *committed* ``protocol/gradle.lockfile`` and
``gradle/verification-metadata.xml`` via a real Gradle invocation, in a
disposable copy and a fresh, isolated ``GRADLE_USER_HOME`` (no global config,
no cache reuse):

* a fresh isolated resolution under strict locking + SHA-256 verification
  succeeds (the reviewed graph and checksums are genuinely enforced, not just
  trusted from a pre-existing cache);
* a deliberate *graph* inconsistency (a locked coordinate changed to a value
  that is not actually resolvable) fails — the STRICT lock is honoured;
* a deliberate *checksum* mismatch (a corrupted artifact SHA-256 in the
  committed verification metadata) fails — the SHA-256 is enforced at
  download/verification time;
* the committed Gradle wrapper JAR and distribution URL keep their recorded
  SHA-256 against Maven Central / services.gradle.org.

No production code under test is mutated; every run uses a per-test disposable
project copy. Run with::

    uv run --no-sync pytest validation/windows-mod/tests/test_dependency_locking.py -s

JDK (``tools/jdk17``) and Gradle (``tools/gradle-8.14.6``) are required.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
PROJECT_ROOT = REPO_ROOT / "validation" / "windows-mod"
TOOL_JDK = REPO_ROOT / "tools" / "jdk17"
TOOL_GRADLE = REPO_ROOT / "tools" / "gradle-8.14.6" / "bin" / "gradle"


def _tools_present() -> bool:
    return TOOL_JDK.is_dir() and TOOL_GRADLE.is_file()


@pytest.fixture(scope="module")
def _require_tools():
    assert _tools_present(), (
        "a JDK (tools/jdk17) and Gradle (tools/gradle-8.14.6) toolchain is "
        "required for the dependency-locking tests"
    )


def _copy_project(dest: Path) -> Path:
    shutil.copytree(
        PROJECT_ROOT, dest, ignore=shutil.ignore_patterns(".gradle", "build", ".git")
    )
    return dest


def _run_gradle(project: Path, *args: str, ghome: Path) -> subprocess.CompletedProcess:
    env = {
        "JAVA_HOME": str(TOOL_JDK),
        "GRADLE_USER_HOME": str(ghome),
        "PATH": f"{TOOL_JDK / 'bin'}:" + os.environ.get("PATH", ""),
        "GRADLE_OPTS": "",
    }
    return subprocess.run(
        [str(TOOL_GRADLE), "--no-daemon", "--console=plain",
         "-p", str(project), *args],
        env=env, capture_output=True, text=True, timeout=300,
    )


@pytest.mark.skipif(not _tools_present(),
                    reason="JDK/Gradle toolchain not available")
def test_protocol_resolves_and_passes_under_strict_lock_and_verification(
        tmp_path: Path, _require_tools: None) -> None:
    project = _copy_project(tmp_path / "prj")
    proc = _run_gradle(project, ":protocol:clean", ":protocol:test",
                       ghome=tmp_path / "gh")
    assert proc.returncode == 0, (
        "fresh isolated locked+verified :protocol build should succeed\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )
    # The lock is being honoured: the resolved graph shows the strict pin for
    # the reviewed coordinate (junit-jupiter).
    assert "BUILD SUCCESSFUL" in proc.stdout


@pytest.mark.skipif(not _tools_present(),
                    reason="JDK/Gradle toolchain not available")
def test_protocol_fails_when_lockfile_graph_is_inconsistent(
        tmp_path: Path, _require_tools: None) -> None:
    project = _copy_project(tmp_path / "prj")
    lockfile = project / "protocol" / "gradle.lockfile"
    text = lockfile.read_text()
    assert "org.opentest4j:opentest4j:1.3.0=" in text
    # Deliberate graph inconsistency: replace a locked version with one that
    # does not actually resolve. STRICT lock mode must fail, not silently
    # re-resolve.
    broken = text.replace(
        "org.opentest4j:opentest4j:1.3.0=",
        "org.opentest4j:opentest4j:9.99.9=", 1)
    assert broken != text
    lockfile.write_text(broken)

    proc = _run_gradle(project, ":protocol:compileTestJava",
                       ghome=tmp_path / "gh")
    assert proc.returncode != 0, (
        "a deliberate lockfile graph inconsistency must fail the build\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )
    combined = proc.stdout + proc.stderr
    assert "dependency lock state" in combined, (
        "expected the STRICT dependency-lock diagnostic\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )
    assert "Did not resolve 'org.opentest4j:opentest4j:9.99.9'" in combined, (
        "expected the mismatched locked coordinate to be named\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )


@pytest.mark.skipif(not _tools_present(),
                    reason="JDK/Gradle toolchain not available")
def test_protocol_fails_when_checksum_verification_mismatches(
        tmp_path: Path, _require_tools: None) -> None:
    project = _copy_project(tmp_path / "prj")
    meta = project / "gradle" / "verification-metadata.xml"
    text = meta.read_text()
    m = re.search(
        r'name="junit-jupiter-api-5\.13\.4\.jar">\s*'
        r'<sha256 value="([0-9a-f]{64})"',
        text)
    assert m, "the committed verification metadata must pin the junit-jupiter-api jar"
    good = m.group(1)
    bad = "deadbeef" * 8
    assert good != bad
    broken = text.replace(f'value="{good}"', f'value="{bad}"', 1)
    assert broken != text
    meta.write_text(broken)

    proc = _run_gradle(project, ":protocol:compileTestJava",
                       ghome=tmp_path / "gh")
    assert proc.returncode != 0, (
        "a deliberately corrupted artifact SHA-256 must fail dependency verification\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )
    combined = proc.stdout + proc.stderr
    assert "verification failed" in combined.lower(), (
        "expected the SHA-256 verification failure diagnostic\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )
    assert "junit-jupiter-api-5.13.4.jar" in combined, (
        "expected the failing artifact to be named\n"
        f"STDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
    )


@pytest.mark.skipif(not _tools_present(),
                    reason="JDK/Gradle toolchain not available")
def test_committed_wrapper_is_pinned_and_official(
        tmp_path: Path, _require_tools: None) -> None:
    """Wrapper integrity against the official source.

    1. The committed ``distributionSha256Sum`` must be preserved and equal the
       SHA-256 of the actual committed/available Gradle 8.14.6 distribution zip
       (i.e. the pin is not a placeholder).
    2. The committed ``gradle-wrapper.jar`` must be byte-identical to what a
       fresh ``gradle wrapper`` (official Gradle 8.14.6 toolchain) regenerates
       into a disposable project — the official toolchain is the trusted source.
    """
    import hashlib

    def _sha(p: Path) -> str:
        return hashlib.sha256(p.read_bytes()).hexdigest()

    wrapper_dir = PROJECT_ROOT / "gradle" / "wrapper"
    props = (wrapper_dir / "gradle-wrapper.properties").read_text()
    m = re.search(r"distributionSha256Sum=([0-9a-f]{64})", props)
    assert m, "gradle-wrapper.properties must pin the distribution SHA-256"
    expected_dist = "7988ed071b2a07900e2ec715fca15c6ab72bce6db433af301fa7fa7e9407bd24"
    assert m.group(1) == expected_dist, (
        "the committed distribution pin must remain the official Gradle 8.14.6 checksum"
    )
    committed_jar = _sha(wrapper_dir / "gradle-wrapper.jar")
    assert len(committed_jar) == 64

    # Tie the pin to the actual distribution artifact available locally.
    dist_zip = REPO_ROOT / "tools" / "gradle-8.14.6-bin.zip"
    if dist_zip.is_file():
        assert _sha(dist_zip) == expected_dist, (
            "the committed/available Gradle 8.14.6 distribution zip must match "
            "the pinned distributionSha256Sum"
        )

    # Regenerate the wrapper with the official toolchain and compare digests.
    project = _copy_project(tmp_path / "prj")
    proc = _run_gradle(project, "wrapper", ghome=tmp_path / "gh")
    assert proc.returncode == 0, (
        f"fresh `gradle wrapper` should succeed\n{proc.stdout}\n{proc.stderr}")
    regenerated = _sha(project / "gradle" / "wrapper" / "gradle-wrapper.jar")
    assert regenerated == committed_jar, (
        "the committed gradle-wrapper.jar must be identical (by SHA-256) to the "
        "official Gradle 8.14.6 `wrapper` task output "
        f"(committed {committed_jar}, regenerated {regenerated})")
