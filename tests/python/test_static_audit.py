"""Synthetic-only operational audit tests; no installed JVM is used."""

import hashlib
import importlib
import json
import os
from pathlib import Path
import stat
import sys
import zipfile

import pytest


def sha(data):
    return hashlib.sha256(data).hexdigest()


@pytest.fixture
def setup(tmp_path):
    module = importlib.import_module("validation.static_audit")
    root = tmp_path / "private"
    stage = root / "inspection" / "study"
    stage.mkdir(parents=True, mode=0o700)
    root.chmod(0o700)
    (root / "inspection").chmod(0o700)
    source = stage / "input.txt"
    source.write_bytes(b"input")
    source.chmod(0o600)
    tools = tmp_path / "tools"
    tools.mkdir()
    script = ("#!" + sys.executable + "\n"
              "import os,sys,json,zipfile\n"
              "if os.getenv('FIXTURE_MARKER'):\n"
              " with open(os.environ['FIXTURE_MARKER'],'ab') as marker: marker.write(b'executed')\n"
              "if sys.argv[1:2] == ['tf']:\n"
              " print('\\n'.join(zipfile.ZipFile(sys.argv[2]).namelist()))\n"
              "else:\n"
              " print(json.dumps({'cwd':os.getcwd(),'env':[os.getenv(k) for k in "
              "['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS']],"
              "'umask':oct(os.umask(0o077))}))\n"
              " sys.stderr.write('diagnostic')\n"
              " sys.exit(9 if os.getenv('FIXTURE_FAIL') else 0)\n")
    paths = {}
    for name in ("java", "javap", "jar", "cfr"):
        paths[name] = tools / name
        paths[name].write_text(script if name != "cfr" else "synthetic cfr")
        paths[name].chmod(0o700)
    game = tools / "game.jar"
    with zipfile.ZipFile(game, "w") as archive:
        archive.writestr("sample/A.class", b"synthetic class")
    paths["game"] = game
    pins = {key: (path, sha(path.read_bytes())) for key, path in paths.items()}

    def make(**kwargs):
        return module.StaticAudit(root=root, stage=stage,
                                  write_root=stage / "fresh", pins=pins,
                                  allowed_argv=kwargs.pop("allowed_argv", []),
                                  duplicate_exception=kwargs.pop("duplicate_exception", {}),
                                  **kwargs)
    return module, make, paths, source, stage


def test_brackets_checks_retention_and_finite_outer_receipt(setup):
    _, make, _, source, stage = setup
    audit = make()
    with audit:
        assert audit.read(source) == b"input"
        audit.write(stage / "fresh" / "result.txt", b"result")
        assert len(audit.inventory([source])) == 1
    receipt = audit.final_receipt
    assert receipt["status"] == "closed"
    assert len(receipt["opening"]) == len(receipt["closing"]) == 5
    assert all(row["match"] for row in receipt["closing"])
    events = receipt["events"]
    assert any(row["operation"] == "read" and row["path"] == str(source) for row in events)
    leaves = sorted((stage / "fresh" / "journal").glob("*.json"))
    assert leaves and [p.name for p in leaves] == [f"{n:06d}.json" for n in range(1, len(leaves) + 1)]
    for leaf in leaves:
        assert stat.S_IMODE(leaf.stat().st_mode) == 0o600
        assert sha(leaf.read_bytes()) not in leaf.read_text()
    retained = json.loads((stage / "fresh" / "receipt.json").read_text())
    assert "closing" not in retained
    assert retained["scope"] == "operations before receipt retention; outer receipt completes audit"
    assert any(row["path"].endswith("receipt.json") for row in receipt["inventory"])
    assert any(row["operation"] == "create" and row["path"].endswith("receipt.json") for row in events)
    with pytest.raises(Exception, match="active"):
        audit.read(source)


def test_opening_mismatch_never_touches_private_tree(setup):
    _, make, paths, _, stage = setup
    audit = make()
    paths["game"].write_bytes(b"changed")
    with pytest.raises(Exception, match="opening"):
        with audit:
            pytest.fail("must not enter")
    assert not (stage / "fresh").exists()
    assert audit.final_receipt["events"] == []
    assert len(audit.final_receipt["closing"]) == 5
    assert audit.final_receipt["closing_error"] == "closing pin mismatch"


def test_original_exception_survives_distinct_closing_mismatch(setup):
    _, make, paths, _, _ = setup
    audit = make()
    original = RuntimeError("body failure")
    with pytest.raises(RuntimeError) as caught:
        with audit:
            paths["cfr"].write_bytes(b"changed")
            raise original
    assert caught.value is original
    assert audit.final_receipt["status"] == "failed"
    assert audit.final_receipt["closing_error"]
    assert any("closing" in note for note in original.__notes__)


def test_static_command_scrubs_env_and_retains_only_explicit_stdout(setup, monkeypatch):
    _, make, paths, _, stage = setup
    argv = [str(paths["java"]), "-version"]
    for key in ("JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS"):
        monkeypatch.setenv(key, "untrusted")
    audit = make(allowed_argv=[argv])
    with audit:
        result = audit.run(argv, stdout_path=stage / "fresh" / "version.txt")
        output = json.loads(result.stdout)
        assert output == {"cwd": str(stage / "fresh"), "env": [None, None, None], "umask": "0o77"}
    command = next(row for row in audit.final_receipt["operations"] if row["operation"] == "command")
    assert command["argv"] == argv
    assert command["exit"] == 0
    assert command["stdout_sha256"] == sha(result.stdout)
    assert command["stderr_sha256"] == sha(b"diagnostic")
    assert command["stdout_path"] == str(stage / "fresh" / "version.txt")
    assert command["scrubbed"] == ["JAVA_TOOL_OPTIONS", "_JAVA_OPTIONS", "JDK_JAVA_OPTIONS"]


def test_nonzero_exit_is_journaled_and_closes(setup, monkeypatch):
    _, make, paths, _, stage = setup
    monkeypatch.setenv("FIXTURE_FAIL", "1")
    argv = [str(paths["javap"]), "-version"]
    audit = make(allowed_argv=[argv])
    with pytest.raises(Exception, match="exit 9"):
        with audit:
            audit.run(argv)
    assert len(audit.final_receipt["closing"]) == 5
    command = next(row for row in audit.final_receipt["operations"] if row["operation"] == "command")
    assert command["exit"] == 9 and command["stdout_sha256"]
    assert not (stage / "fresh" / "stdout").exists()


@pytest.mark.parametrize("kind", ["execute", "not_allowlisted"])
def test_rejects_game_execution_or_unapproved_argv(setup, kind):
    _, make, paths, _, _ = setup
    argv = [str(paths["java"]), "-jar", str(paths["game"])]
    if kind == "execute":
        with pytest.raises(Exception, match="static"):
            make(allowed_argv=[argv])
    else:
        with make() as audit:
            with pytest.raises(Exception, match="allowlist"):
                audit.run([str(paths["java"]), "-version"])
        rejected = next(row for row in audit.final_receipt["operations"] if row["operation"] == "command_rejected")
        assert rejected["argv"] == [str(paths["java"]), "-version"]


def test_archive_guard_precedes_exact_extraction(setup):
    _, make, _, _, _ = setup
    audit = make()
    with audit:
        assert audit.extract_class("sample/A.class") == b"synthetic class"
        with pytest.raises(Exception, match="unique"):
            audit.extract_class("missing.class")
    ops = audit.final_receipt["operations"]
    assert next(i for i, row in enumerate(ops) if row["operation"] == "archive_guard") < next(i for i, row in enumerate(ops) if row["operation"] == "extract")


@pytest.mark.parametrize("unsafe", ["../bad.class", "symlink", "duplicate"])
def test_archive_name_type_duplicate_guard_stops_extraction(setup, unsafe):
    _, make, paths, _, _ = setup
    with zipfile.ZipFile(paths["game"], "w") as archive:
        if unsafe == "symlink":
            info = zipfile.ZipInfo("bad.class")
            info.external_attr = (stat.S_IFLNK | 0o777) << 16
            archive.writestr(info, b"target")
        elif unsafe == "duplicate":
            archive.writestr("bad.class", b"one")
            with pytest.warns(UserWarning):
                archive.writestr("bad.class", b"two")
        else:
            archive.writestr(unsafe, b"bad")
    audit = make()
    audit.pins["game"] = (paths["game"], sha(paths["game"].read_bytes()))
    with audit:
        with pytest.raises(Exception, match="unsafe|duplicate"):
            audit.extract_class("bad.class")
    assert not any(row["operation"] == "extract" for row in audit.final_receipt["operations"])


@pytest.mark.parametrize("unsafe", ["mode", "symlink", "exclusive"])
def test_private_access_fails_closed_without_repairs(setup, unsafe):
    _, make, _, source, stage = setup
    audit = make()
    with audit:
        if unsafe == "mode":
            source.chmod(0o644)
            with pytest.raises(Exception, match="mode"):
                audit.read(source)
            assert stat.S_IMODE(source.stat().st_mode) == 0o644
        elif unsafe == "symlink":
            link = stage / "link"
            link.symlink_to(source)
            with pytest.raises(Exception, match="symlink"):
                audit.read(link)
        else:
            destination = stage / "fresh" / "output"
            audit.write(destination, b"first")
            with pytest.raises(Exception, match="exists"):
                audit.write(destination, b"second")
            assert destination.read_bytes() == b"first"
    assert any(row["operation"] == "failure" for row in audit.final_receipt["operations"])


def test_existing_output_directory_is_never_used_for_retention(setup):
    _, make, _, _, stage = setup
    destination = stage / "fresh"
    destination.mkdir(mode=0o700)
    sentinel = destination / "sentinel"
    sentinel.write_bytes(b"untouched")
    sentinel.chmod(0o600)
    audit = make()
    with pytest.raises(Exception, match="exists"):
        with audit:
            pytest.fail("existing revision must not open")
    assert sorted(path.name for path in destination.iterdir()) == ["sentinel"]
    assert sentinel.read_bytes() == b"untouched"
    assert len(audit.final_receipt["closing"]) == 5


def test_failed_process_start_records_nullable_output_and_closes(setup):
    _, make, paths, _, _ = setup
    paths["java"].chmod(0o600)
    argv = [str(paths["java"]), "-version"]
    audit = make(allowed_argv=[argv])
    with pytest.raises(PermissionError):
        with audit:
            audit.run(argv)
    command = next(row for row in audit.final_receipt["operations"] if row["operation"] == "command")
    assert command["exit"] is None
    assert command["stdout_sha256"] is command["stderr_sha256"] is None
    assert len(audit.final_receipt["closing"]) == 5


def test_pin_read_does_not_mistake_access_time_for_mutation(setup):
    _, make, paths, _, _ = setup
    for path in paths.values():
        info = path.stat()
        os.utime(path, ns=(1, info.st_mtime_ns))
    audit = make()
    with audit:
        pass
    assert audit.final_receipt["status"] == "closed"


def test_duplicate_exception_is_exact_but_never_selects_ambiguous_entry(setup):
    _, make, paths, _, _ = setup
    with zipfile.ZipFile(paths["game"], "w") as archive:
        archive.writestr("repeat.class", b"one")
        with pytest.warns(UserWarning):
            archive.writestr("repeat.class", b"two")
        archive.writestr("unique.class", b"unique")
    audit = make(duplicate_exception={"repeat.class": 2})
    audit.pins["game"] = (paths["game"], sha(paths["game"].read_bytes()))
    with audit:
        assert audit.extract_class("unique.class") == b"unique"
        with pytest.raises(Exception, match="unique"):
            audit.extract_class("repeat.class")
    assert [row["entry"] for row in audit.final_receipt["operations"] if row["operation"] == "extract"] == ["unique.class"]


def test_retention_failure_preserves_original_and_reports_closing_separately(setup):
    _, make, _, _, stage = setup
    audit = make()
    original = RuntimeError("original")
    with pytest.raises(RuntimeError) as caught:
        with audit:
            # Simulate an external producer colliding with the next exclusive leaf.
            collision = stage / "fresh" / "journal" / "000002.json"
            collision.write_bytes(b"collision")
            collision.chmod(0o600)
            raise original
    assert caught.value is original
    assert audit.final_receipt["retention_error"]
    assert len(audit.final_receipt["closing"]) == 5
    assert collision.read_bytes() == b"collision"
    assert any("retention" in note for note in original.__notes__)


def test_closing_mismatch_without_body_exception_raises(setup):
    _, make, paths, _, _ = setup
    audit = make()
    with pytest.raises(Exception, match="closing"):
        with audit:
            paths["java"].write_bytes(b"changed")
    assert audit.final_receipt["closing_error"] == "closing pin mismatch"


def test_standalone_archive_guard_does_not_extract_any_class(setup):
    _, make, _, _, _ = setup
    audit = make()
    with audit:
        guard = audit.guard_archive()
        assert guard["result"] == "PASS" and guard["entries"] == 1
    assert not any(row["operation"] == "extract" for row in audit.final_receipt["operations"])


@pytest.mark.parametrize("kind", ["symlink", "directory"])
def test_unsafe_pin_type_stops_before_private_reads(setup, kind):
    _, make, paths, _, stage = setup
    audit = make()
    replacement = paths["java"].parent / "replacement"
    if kind == "symlink":
        replacement.symlink_to(paths["java"])
    else:
        replacement.mkdir()
    audit.pins["java"] = (replacement, audit.pins["java"][1])
    with pytest.raises(Exception, match="opening"):
        with audit:
            pytest.fail("unsafe pins must not open")
    assert not (stage / "fresh").exists()
    assert audit.final_receipt["events"] == []


def test_private_class_input_to_javap_and_cfr_is_checked_and_read(setup):
    _, make, paths, _, stage = setup
    raw = stage / "selected.class"
    raw.write_bytes(b"synthetic raw")
    raw.chmod(0o600)
    javap = [str(paths["javap"]), "-p", "-s", "-c", str(raw)]
    cfr = [str(paths["java"]), "-jar", str(paths["cfr"]), str(raw),
           "--methodname", "run", "--innerclasses", "false", "--clobber", "false", "--silent", "true"]
    audit = make(allowed_argv=[javap, cfr])
    with audit:
        audit.run(javap)
        audit.run(cfr)
    assert sum(row["operation"] == "read" and row["path"] == str(raw) for row in audit.final_receipt["events"]) == 2


def test_listing_disagreement_prevents_extraction(setup):
    _, make, paths, _, _ = setup
    paths["jar"].write_text("#!" + sys.executable + "\nprint('wrong.class')\n")
    audit = make()
    audit.pins["jar"] = (paths["jar"], sha(paths["jar"].read_bytes()))
    with audit:
        with pytest.raises(Exception, match="listing mismatch"):
            audit.extract_class("sample/A.class")
    assert not any(row["operation"] == "extract" for row in audit.final_receipt["operations"])


def test_no_retry_after_journal_collision_even_if_caller_catches_error(setup):
    _, make, _, source, stage = setup
    audit = make()
    with pytest.raises(Exception, match="journal previously failed"):
        with audit:
            collision = stage / "fresh" / "journal" / "000002.json"
            collision.write_bytes(b"collision")
            collision.chmod(0o600)
            with pytest.raises(Exception, match="exists"):
                audit.read(source)
            with pytest.raises(Exception, match="no retry"):
                audit.read(source)
    assert sum(row["operation"] == "read" and row["path"] == str(source) for row in audit.final_receipt["events"]) == 1
    assert len(audit.final_receipt["closing"]) == 5


@pytest.mark.parametrize("failure", ["digest", "mode", "missing"])
def test_caught_safety_failure_blocks_further_read_and_command(setup, failure, monkeypatch):
    _, make, paths, source, stage = setup
    argv = [str(paths["java"]), "-version"]
    marker = stage / "subprocess.marker"
    monkeypatch.setenv("FIXTURE_MARKER", str(marker))
    audit = make(allowed_argv=[argv])
    with audit:
        if failure == "mode":
            source.chmod(0o644)
        with pytest.raises(Exception, match="digest mismatch|mode|missing") as original:
            audit.read(source if failure != "missing" else stage / "absent",
                       expected="0" * 64 if failure == "digest" else None)
        previous_reads = sum(row["operation"] == "read" for row in audit.tree.events)
        with pytest.raises(Exception, match="failed.*context|context.*failed") as blocked:
            audit.read(source)
        assert blocked.value.__cause__ is original.value
        with pytest.raises(Exception, match="failed.*context|context.*failed"):
            audit.run(argv)
        assert sum(row["operation"] == "read" for row in audit.tree.events) == previous_reads
        assert not marker.exists()
    assert audit.final_receipt["status"] == "failed"
    assert len(audit.final_receipt["closing"]) == 5


@pytest.mark.parametrize("selector", ["run", "$synthetic_1", "<init>", "<clinit>"])
def test_selected_cfr_grammar_and_stderr_retention(setup, selector):
    _, make, paths, _, stage = setup
    raw = stage / "selected.class"
    raw.write_bytes(b"synthetic class")
    raw.chmod(0o600)
    selected = [str(paths["java"]), "-jar", str(paths["cfr"]), str(raw),
                "--methodname", selector, "--innerclasses", "false", "--clobber", "false",
                "--extraclasspath", str(paths["game"]), "--silent", "true"]
    help_argv = [str(paths["java"]), "-jar", str(paths["cfr"]), "--help"]
    audit = make(allowed_argv=[selected, help_argv])
    with audit:
        audit.run(selected, stdout_path=stage / "fresh" / "selected.java")
        result = audit.run(help_argv)
        audit.write(stage / "fresh" / "cfr-help.stderr", result.stderr)
    assert (stage / "fresh" / "cfr-help.stderr").read_bytes() == b"diagnostic"
    commands = [row for row in audit.final_receipt["operations"] if row["operation"] == "command"]
    assert [row["argv"] for row in commands] == [selected, help_argv]


@pytest.mark.parametrize("options", [
    [],
    ["--methodname", "run"],
    ["--methodname", "run", "--innerclasses", "true", "--clobber", "false"],
    ["--methodname", "run", "--innerclasses", "false", "--clobber", "true"],
    ["--methodname", "*", "--innerclasses", "false", "--clobber", "false"],
    ["--methodname", "run.*", "--innerclasses", "false", "--clobber", "false"],
    ["--methodname", "A.run", "--innerclasses", "false", "--clobber", "false"],
    ["--methodname", "<other>", "--innerclasses", "false", "--clobber", "false"],
    ["--methodname", "run", "--methodname", "other", "--innerclasses", "false", "--clobber", "false"],
    ["--methodname", "run", "--innerclasses", "false", "--clobber", "false", "--outputdir", "/unapproved"],
    ["--methodname", "run", "--innerclasses", "false", "--clobber", "false", "--extraclasspath", "/unapproved.jar"],
])
def test_cfr_rejects_whole_class_unsafe_selector_and_output_options(setup, options):
    _, make, paths, _, stage = setup
    argv = [str(paths["java"]), "-jar", str(paths["cfr"]), str(stage / "selected.class"), *options]
    with pytest.raises(Exception, match="static|CFR|selector"):
        make(allowed_argv=[argv])


@pytest.mark.parametrize("command,role,mutation", [
    ("version", "java", "symlink"),
    ("version", "java", "digest"),
    ("listing", "jar", "symlink"),
    ("listing", "game", "directory"),
    ("cfr", "cfr", "symlink"),
    ("cfr", "game", "digest"),
])
def test_every_command_rechecks_pinned_external_files_before_execution(setup, monkeypatch, command, role, mutation):
    module, make, paths, _, stage = setup
    raw = stage / "selected.class"
    raw.write_bytes(b"synthetic class")
    raw.chmod(0o600)
    argv = {"version": [str(paths["java"]), "-version"],
            "listing": [str(paths["jar"]), "tf", str(paths["game"])],
            "cfr": [str(paths["java"]), "-jar", str(paths["cfr"]), str(raw),
                    "--methodname", "run", "--innerclasses", "false", "--clobber", "false",
                    "--extraclasspath", str(paths["game"])]}[command]
    marker = stage / "subprocess.marker"
    monkeypatch.setenv("FIXTURE_MARKER", str(marker))
    audit = make(allowed_argv=[argv])
    with pytest.raises(module.AuditError, match="closing"):  # not a swallowed body assertion
        with audit:
            target = paths[role]
            if mutation == "digest":
                target.write_bytes(target.read_bytes() + b"\n# changed\n")
            else:
                prior = target.with_suffix(".prior")
                target.rename(prior)
                if mutation == "symlink":
                    target.symlink_to(prior)
                else:
                    target.mkdir()
            with pytest.raises(module.ReportError, match="symlink|regular|pin.*mismatch"):
                audit.run(argv)
            assert not marker.exists()
    assert len(audit.final_receipt["closing"]) == 5


def test_version_invocation_retains_fresh_executable_checks_and_reads(setup):
    _, make, paths, _, _ = setup
    argv = [str(paths["java"]), "-version"]
    audit = make(allowed_argv=[argv])
    with audit:
        audit.run(argv)
        audit.run(argv)
    events = audit.final_receipt["events"]
    assert sum(row["operation"] == "check" and row["path"] == str(paths["java"]) for row in events) == 2
    assert sum(row["operation"] == "read" and row["path"] == str(paths["java"]) for row in events) == 2
