"""Synthetic metadata only: no owned-game evidence belongs in these fixtures."""

import hashlib
import json
import os
from pathlib import Path

import pytest

from validation.evidence_report import (
    PrivateTree,
    ReportError,
    artifact_bindings,
    claim_sections,
    line_ranges,
    allowlist_binding,
    bytecode_anchor,
    source_anchor,
    validate_archive,
    tool_bindings,
    instruction_ranges,
    listing_anchor,
    named_instruction_ranges,
)


@pytest.fixture
def private(tmp_path):
    root = tmp_path / "private"
    root.mkdir(mode=0o700)
    inspection = root / "inspection"
    inspection.mkdir(mode=0o700)
    stage = inspection / "stage"
    stage.mkdir(mode=0o700)
    return PrivateTree(root, stage, stage / "report" / "new")


def put(path, content=b"evidence"):
    missing = []
    parent = path.parent
    while not parent.exists():
        missing.append(parent)
        parent = parent.parent
    for parent in reversed(missing):
        parent.mkdir(mode=0o700)
    path.write_bytes(content)
    path.chmod(0o600)
    return path


def test_read_checks_private_leaf_and_every_task_ancestor(private):
    path = put(private.stage / "old" / "deep" / "record.json")
    assert private.read(path) == b"evidence"
    path.chmod(0o644)
    with pytest.raises(ReportError, match="mode"):
        private.read(path)
    path.chmod(0o600)
    path.parent.chmod(0o755)
    with pytest.raises(ReportError, match="mode"):
        private.read(path)


def test_inspection_container_is_also_private(private):
    path = put(private.stage / "record.md")
    private.stage.parent.chmod(0o755)
    with pytest.raises(ReportError, match="mode"):
        private.read(path)


def test_read_rejects_symlinks_hardlinks_and_path_escape(private):
    original = put(private.stage / "record.md")
    link = private.stage / "link.md"
    link.symlink_to(original)
    with pytest.raises(ReportError, match="symlink"):
        private.read(link)
    hard = private.stage / "hard.md"
    os.link(original, hard)
    with pytest.raises(ReportError, match="link"):
        private.read(original)
    with pytest.raises(ReportError, match="outside|lexical"):
        private.read(private.stage / ".." / "outside.md")


def test_read_verifies_expected_digest(private):
    path = put(private.stage / "record.md", b"abc")
    assert private.read(path, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad") == b"abc"
    with pytest.raises(ReportError, match="digest"):
        private.read(path, "0" * 64)


def test_write_is_exclusive_private_and_revision_bounded(private):
    path = private.write_root / "run-records" / "phase.json"
    private.write(path, b"new")
    assert path.read_bytes() == b"new"
    assert path.stat().st_mode & 0o777 == 0o600
    assert path.parent.stat().st_mode & 0o777 == 0o700
    with pytest.raises(ReportError, match="exists"):
        private.write(path, b"replacement")
    assert path.read_bytes() == b"new"
    with pytest.raises(ReportError, match="outside"):
        private.write(private.stage / "old.md", b"wrong revision")
    assert [event["operation"] for event in private.events].count("create") == 1


def test_inventory_verifies_children_not_just_root(private):
    path = put(private.stage / "old" / "record.md", b"abc")
    rows = private.inventory([private.stage / "old"])
    assert rows == [{"path": str(path), "bytes": 3, "sha256": "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}]
    path.chmod(0o644)
    with pytest.raises(ReportError, match="mode"):
        private.inventory([private.stage / "old"])


def test_markdown_claim_ranges_include_whole_exact_record():
    text = "# Study\n\n## a\n- **id:** `A`; **area:** shared\n- **evidence_class:** source-derived\n- **finding:** one\n\n## b\n- **id:** B; **area:** combat\n- **evidence_class:** bytecode-confirmed\n- **finding:** two\n\n## Limitations\nnot a claim\n"
    rows = claim_sections(text)
    assert [(r["id"], r["lines"]) for r in rows] == [("A", [3, 7]), ("B", [8, 12])]
    assert "one" in rows[0]["text"]
    assert "Limitations" not in rows[1]["text"]


def test_nested_markdown_claim_headings_do_not_merge_sibling_claims():
    text = "# Study\n## Claims\n### alpha\n- **id:** A; **area:** combat\n- **finding:** first\n### beta\n- **id:** B; **area:** combat\n- **finding:** second\n## Limits\nend\n"
    rows = claim_sections(text)
    assert [(row["id"], row["lines"]) for row in rows] == [("A", [3, 5]), ("B", [6, 8])]
    assert "second" not in rows[0]["text"]


def test_json_claim_ranges_handle_source_only_record():
    record = {"id": "S", "evidence_class": "source-derived", "anchors": {"source_file": "/private/a.java", "source_lines": "2-3"}}
    text = "# Study\n```json\n" + json.dumps(record, indent=2) + "\n```\n"
    row, = claim_sections(text)
    assert row["id"] == "S"
    assert row["data"] == record
    assert row["lines"][0] == 2
    assert row["lines"][1] == len(text.splitlines())


def test_root_paths_and_separate_digest_fields_are_paired_by_kind():
    text = '- **anchors:** source `/private/a.java:2-3`; bytecode `/private/a.txt`: `run` offsets 0-9.\n- **retained artifacts:** raw `/private/a.class` SHA256 `' + '1' * 64 + '`; source SHA256 `' + '2' * 64 + '`; bytecode SHA256 `' + '3' * 64 + '`.\n'
    assert artifact_bindings(text) == [
        {"kind": "raw_class", "path": "/private/a.class", "sha256": "1" * 64},
        {"kind": "source", "path": "/private/a.java", "sha256": "2" * 64},
        {"kind": "bytecode", "path": "/private/a.txt", "sha256": "3" * 64},
    ]


@pytest.mark.parametrize("label", ["javap lines", "javap listing lines", "lines"])
def test_line_ranges_accept_all_retained_anchor_labels(label):
    assert line_ranges(f"{label} 2-4,7–9", 10) == [[2, 4], [7, 9]]


@pytest.mark.parametrize("text", ["lines 0-2", "lines 4-2", "lines 2-11", "no range"])
def test_line_ranges_fail_closed_on_missing_or_invalid_bounds(text):
    with pytest.raises(ReportError, match="range"):
        line_ranges(text, 10)


BYTECODE = """public class example.Unit {
  public void run();
    descriptor: ()V
    Code:
       0: iconst_0
       1: return
  public void run(int);
    descriptor: (I)V
    Code:
       0: return
}
"""


def test_bytecode_anchor_binds_descriptor_lines_and_real_instruction_offsets():
    result = bytecode_anchor(BYTECODE, [["example.Unit", "run", "()V"]], [[4, 6]], [[0, 1]])
    assert result["members"] == [["example.Unit", "run", "()V"]]
    assert result["offsets"] == [[0, 1]]
    with pytest.raises(ReportError, match="offset"):
        bytecode_anchor(BYTECODE, [["example.Unit", "run", "(I)V"]], [[9, 10]], [[0, 1]])
    with pytest.raises(ReportError, match="member"):
        bytecode_anchor(BYTECODE, [["example.Unit", "missing", "()V"]], [[4, 6]], [[0, 1]])
    with pytest.raises(ReportError, match="range"):
        bytecode_anchor(BYTECODE, [["example.Unit", "run", "()V"]], [[9, 10]], [[0, 1]])


def test_bytecode_classfile_metadata_is_not_a_declaring_class_header():
    text = "Classfile /private/Unit.class\n  Last modified yesterday\n" + BYTECODE
    assert bytecode_anchor(text, [["example.Unit", "run", "()V"]], [[6, 8]], [[0, 1]])["result"] == "PASS"


def test_source_anchor_requires_declaration_not_call_and_valid_bounds():
    source = "public class Unit {\n  public void run() {\n    other();\n  }\n}\n"
    assert source_anchor(source, [["example.Unit", "run", "()V"]], [[2, 4]])["declaration_lines"] == [2]
    with pytest.raises(ReportError, match="declaration"):
        source_anchor(source, [["example.Unit", "other", "()V"]], [[2, 4]])
    with pytest.raises(ReportError, match="range"):
        source_anchor(source, [["example.Unit", "run", "()V"]], [[2, 7]])


def test_source_anchor_rejects_range_inside_an_unrelated_method():
    source = "public class Unit {\n  public void run() {\n    int n = 1;\n  }\n  public void other() {\n    int n = 2;\n  }\n}\n"
    with pytest.raises(ReportError, match="range|member"):
        source_anchor(source, [["example.Unit", "run", "()V"]], [[5, 7]])


def test_bytecode_anchor_cannot_borrow_offsets_from_another_selected_method():
    text = BYTECODE.replace("run(int)", "other(int)").replace("       0: return\n}", "       0: iconst_0\n       2: return\n}")
    with pytest.raises(ReportError, match="offset|member"):
        bytecode_anchor(text, [["example.Unit", "run", "()V"], ["example.Unit", "other", "(I)V"]], [[4, 6]], [[0, 2]])


def test_bytecode_anchor_preserves_explicit_member_name_for_each_offset_range():
    text = BYTECODE.replace("run(int)", "other(int)")
    with pytest.raises(ReportError, match="offset|member"):
        bytecode_anchor(text, [["example.Unit", "run", "()V"], ["example.Unit", "other", "(I)V"]], [[4, 10]], [[0, 1]], offset_names=["other"])


def test_allowlist_binding_does_not_mix_members_between_classes():
    text = "### example/A.class\n- raw_class_sha256: `" + "1" * 64 + "`\n- `run` . `()V`\n### example/B.class\n- raw_class_sha256: `" + "2" * 64 + "`\n- `run` . `(I)V`\n"
    assert allowlist_binding(text, "example/A.class", [["example.A", "run", "()V"]], "1" * 64)["member_count"] == 1
    with pytest.raises(ReportError, match="member"):
        allowlist_binding(text, "example/A.class", [["example.A", "run", "(I)V"]], "1" * 64)
    with pytest.raises(ReportError, match="digest"):
        allowlist_binding(text, "example/A.class", [["example.A", "run", "()V"]], "2" * 64)


def test_archive_validation_accepts_only_exact_authorized_duplicate_pairs():
    import zipfile
    entries = [zipfile.ZipInfo("a"), zipfile.ZipInfo("allowed"), zipfile.ZipInfo("allowed")]
    assert validate_archive(entries, ["a", "allowed", "allowed"], {"allowed": 2})["entries"] == 3
    with pytest.raises(ReportError, match="duplicate"):
        validate_archive(entries, ["a", "allowed", "allowed"], {})
    with pytest.raises(ReportError, match="listing"):
        validate_archive(entries, ["a"], {"allowed": 2})
    unsafe = zipfile.ZipInfo("../outside")
    with pytest.raises(ReportError, match="unsafe"):
        validate_archive([unsafe], ["../outside"], {})
    unsafe = zipfile.ZipInfo("symlink")
    unsafe.external_attr = 0o120777 << 16
    with pytest.raises(ReportError, match="type"):
        validate_archive([unsafe], ["symlink"], {})


def test_tool_binding_accepts_named_single_tool_or_explicit_multi_tool_pins():
    allowed = {"a" * 64, "b" * 64}
    assert tool_bindings({"name": "reader", "sha256": "a" * 64}, allowed) == ["a" * 64]
    assert tool_bindings({"reader": "a" * 64, "crosscheck": "b" * 64}, allowed) == ["a" * 64, "b" * 64]
    with pytest.raises(ReportError, match="tool"):
        tool_bindings({"name": "reader", "sha256": "c" * 64}, allowed)
    with pytest.raises(ReportError, match="tool"):
        tool_bindings({"name": "reader"}, allowed)


def test_instruction_ranges_do_not_treat_dates_in_artifact_paths_as_offsets():
    assert instruction_ranges("`/private/2026-10/artifact.txt`: run offsets0–9, other 0-12") == [[0, 9], [0, 12]]
    with pytest.raises(ReportError, match="offset"):
        instruction_ranges("no offsets recorded")


def test_listing_anchor_keeps_offsets_separate_from_parenthetical_listing_lines():
    text = "`/private/2026-10/a.txt`: constructor offsets 0-9 (javap listing lines 2-8), run 0–12 (10-20)."
    assert listing_anchor(text, 20) == {"lines": [[2, 8], [10, 20]], "offsets": [[0, 9], [0, 12]]}
    with pytest.raises(ReportError, match="range"):
        listing_anchor(text, 19)


def test_named_instruction_ranges_preserve_member_names_and_explicit_aliases():
    text = "`/private/2026-10/a.txt`: constructor offsets 0-9 (lines 2-8), `run` instruction offsets 0–12 (10-20)."
    assert named_instruction_ranges(text, {"constructor": "<init>"}) == [("<init>", [0, 9]), ("run", [0, 12])]
    with pytest.raises(ReportError, match="named"):
        named_instruction_ranges("offsets0-9", {})
