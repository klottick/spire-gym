"""Integrity checks for the offline event reference dataset."""

import builtins
from contextlib import redirect_stdout
import io
import json
import os
from pathlib import Path
import re
import runpy
import subprocess
import sys
import unittest
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]
EVENTS = ROOT / "data/events/events.json"
BUILDER = ROOT / "data/events/build_dataset.py"
GRADES = {"S", "BC", "D", "W", "U"}
DECISION = re.compile(r"^(?:E(?:[0-9]|1[0-5])|F[1-8])$")
EXPECTED_IDS = {
    "neow", "upgrade_shrine", "purifier", "ancient_writing", "joust",
    "falling", "tomb_of_lord_red_mask", "big_fish", "dead_adventurer",
    "golden_idol", "mushrooms", "living_wall", "scrap_ooze", "shining_light",
    "cleric", "ssssserpent", "wing_statue", "world_of_goop", "addict",
    "beggar", "colosseum", "cursed_tome", "drug_dealer", "forgotten_altar",
    "ghosts", "masked_bandits", "nest", "library", "mausoleum", "vampires",
    "mind_bloom", "moai_head", "mysterious_sphere", "sensory_stone",
    "winding_halls", "golden_shrine", "match_and_keep", "transmogrifier",
    "wheel_of_change", "accursed_blacksmith", "bonfire_spirits", "face_trader",
    "fountain_of_cleansing", "lab", "note_for_yourself", "we_meet_again",
    "woman_in_blue", "designer", "duplicator", "knowing_skull", "nloth",
    "secret_portal",
}


def _records():
    return json.loads(EVENTS.read_text(encoding="utf-8"))["records"]


def _all_citations(record):
    yield from record["eligibility"]["citations"]
    for branch in record["branches"]:
        yield from branch["citations"]


class EventReferenceTests(unittest.TestCase):
    def test_reference_has_the_complete_event_identity_set(self):
        records = _records()
        self.assertEqual(len(records), 52)
        self.assertEqual({record["id"] for record in records}, EXPECTED_IDS)

    def test_records_and_branches_have_the_required_fields_and_grades(self):
        for record in _records():
            with self.subTest(event=record.get("id")):
                for key in ("id", "candidate_source_id", "candidate_source_grade",
                            "display_name", "acts", "pool", "eligibility", "branches"):
                    self.assertIn(key, record)
                self.assertIn(record["candidate_source_grade"], GRADES)
                self.assertIsInstance(record["acts"], list)
                self.assertTrue(record["branches"])
                eligibility = record["eligibility"]
                self.assertIn("rule", eligibility)
                self.assertIn(eligibility["grade"], GRADES)
                self.assertIn("citations", eligibility)
                for branch in record["branches"]:
                    self.assertIn("id", branch)
                    self.assertIn("grade", branch)
                    self.assertIn(branch["grade"], GRADES)
                    for key in ("amounts", "draws", "selections", "citations", "decisions"):
                        self.assertIn(key, branch)
                    if branch["grade"] == "U":
                        self.assertTrue(branch.get("note"), "U branch needs an uncertainty note")
                        self.assertTrue(branch["decisions"], "U branch needs an E/F number")
                    for decision in branch["decisions"]:
                        self.assertRegex(decision, DECISION)

    def test_citations_resolve_to_repository_lines_or_captured_revisions(self):
        page_revisions = set()
        for pages_file in (ROOT / "data").glob("*/sources/wiki-pages*.json"):
            for page in json.loads(pages_file.read_text(encoding="utf-8")):
                page_revisions.add((page["title"], str(page["revision_id"])))

        for record in _records():
            for citation in _all_citations(record):
                with self.subTest(event=record["id"], citation=citation):
                    self.assertIn(citation["grade"], GRADES)
                    if citation["grade"] == "W":
                        self.assertIn((citation["page"], str(citation["revision_id"])), page_revisions)
                    if citation["type"] == "repo":
                        path = ROOT / citation["path"]
                        self.assertTrue(path.is_file(), citation["path"])
                        lines = path.read_text(encoding="utf-8").splitlines()
                        line_number = citation["line"]
                        self.assertGreaterEqual(line_number, 1)
                        self.assertLessEqual(line_number, len(lines))
                        self.assertTrue(lines[line_number - 1].strip())

    def test_disclosed_decisions_have_stable_project_numbers(self):
        annotations = json.loads((ROOT / "data/events/annotations.json").read_text(encoding="utf-8"))
        for item in annotations["decisions"]:
            with self.subTest(decision=item.get("id")):
                self.assertRegex(item["id"], DECISION)
                self.assertTrue(item["description"])

    def test_pm_ruled_event_eligibility_is_pinned(self):
        expected = {
            "we_meet_again": (
                "Acts 1–3; no event-entry gold gate (the ≥50-gold condition applies only to Give gold).",
                "S",
            ),
            "addict": (
                "Act 2 ordinary pool; no spawn gate (Beggar owns the ≥75-gold gate).",
                "S",
            ),
            "note_for_yourself": (
                "Outside Daily, eligible at A0–A14 under UNLOCK-001 (highest unlocked Ascension treated as 20).",
                "S",
            ),
            "secret_portal": (
                "Act 3; the original 800-second playtime gate is treated as met under TIME-001.",
                "S",
            ),
        }
        records = {record["id"]: record for record in _records()}
        for event_id, (rule, grade) in expected.items():
            with self.subTest(event=event_id):
                eligibility = records[event_id]["eligibility"]
                self.assertEqual(eligibility["rule"], rule)
                self.assertEqual(eligibility["grade"], grade)

    def test_pm_ruled_empty_grids_and_event_combat_escapes_are_pinned(self):
        records = {record["id"]: record for record in _records()}

        def selection(event_id, branch_id, selection_name):
            branch = next(b for b in records[event_id]["branches"] if b["id"] == branch_id)
            return branch, next(s for s in branch["selections"] if s["name"] == selection_name)

        def assert_source_lines(branch, expected):
            actual = {(c["path"], c["line"]) for c in branch["citations"] if c["type"] == "repo"}
            for path, line in expected:
                self.assertIn((path, line), actual)

        m28 = "docs/milestones/m28-events-economy.md"
        deviation = "docs/intentional-deviations.md"
        with self.subTest(event="ancient_writing"):
            branch, item = selection("ancient_writing", "elegance", "empty-grid behavior")
            self.assertEqual(item["grade"], "S")
            text = " ".join(item["options"]).lower()
            self.assertIn("purgeable", text)
            self.assertIn("unbottled", text)
            self.assertIn("skip", text)
            self.assertIn("leave screen", text)
            self.assertIn("removes nothing", text)
            self.assertIn("events.city.backtobasics", branch.get("note", "").lower())
            assert_source_lines(branch, {(m28, 119)})

        for event_id, branch_id, profile_line, deviation_line in (
            ("purifier", "purify", 120, 132),
            ("transmogrifier", "transmogrify", 153, 133),
        ):
            with self.subTest(event=event_id):
                branch, item = selection(event_id, branch_id, "empty-grid behavior")
                self.assertEqual(item["grade"], "S")
                text = " ".join(item["options"]).lower()
                self.assertIn("original", text)
                self.assertIn("strands", text)
                self.assertIn("empty grid", text)
                self.assertIn("completes", text)
                self.assertTrue(
                    "removes nothing" in text or "without removing" in text or "no removal" in text
                )
                self.assertIn("softlock-001", text)
                assert_source_lines(branch, {(m28, profile_line), (deviation, deviation_line)})

        with self.subTest(event="wing_statue"):
            branch, item = selection("wing_statue", "pray", "empty-grid behavior")
            self.assertEqual(item["grade"], "S")
            text = " ".join(item["options"]).lower()
            for phrase in ("original", "7 damage", "removal grid", "no local check", "strands", "empty grid"):
                self.assertIn(phrase, text)
            for phrase in ("softlock-001", "deals 7 damage", "removes nothing", "shows the result"):
                self.assertIn(phrase, text)
            research = "docs/research/2026-10-19-act1-content-event-neow.md"
            assert_source_lines(
                branch,
                {(research, 266), (m28, 153), (deviation, 134)},
            )

        for event_id, branch_id in (
            ("dead_adventurer", "ambush_and_fight"),
            ("mushrooms", "fight"),
            ("colosseum", "second_taskmaster_and_nob_fight"),
            ("masked_bandits", "fight"),
            ("mysterious_sphere", "fight_orb_walkers"),
        ):
            with self.subTest(event=event_id, branch=branch_id):
                branch, item = selection(event_id, branch_id, "Smoke Bomb escape")
                self.assertEqual(item["grade"], "S")
                text = " ".join(item["options"]).lower()
                self.assertIn("only proceed", text)
                for phrase in ("rewards", "potion roll", "proceed", "map"):
                    self.assertIn(phrase, text)
                assert_source_lines(branch, {(m28, 122)})

        with self.subTest(event="colosseum_first_fight"):
            branch, item = selection("colosseum", "first_slaver_fight", "Smoke Bomb escape")
            self.assertEqual(item["grade"], "S")
            text = " ".join(item["options"]).lower()
            for phrase in ("rewards disabled", "reopens", "post-fight"):
                self.assertIn(phrase, text)
            assert_source_lines(branch, {(m28, 123)})

        with self.subTest(event="mind_bloom_boss"):
            branch, item = selection("mind_bloom", "fight_act_1_boss", "combat")
            text = " ".join(item["options"]).lower()
            self.assertIn("smoke bomb is blocked", text)
            assert_source_lines(branch, {(m28, 124)})

    def test_builder_reads_only_repository_paths(self):
        real_open = builtins.open

        def guarded_open(file, *args, **kwargs):
            path = Path(os.fspath(file)).resolve()
            self.assertTrue(path == ROOT or ROOT in path.parents,
                            f"builder opened a path outside repository: {path}")
            return real_open(file, *args, **kwargs)

        with mock.patch("builtins.open", side_effect=guarded_open), \
             mock.patch.object(sys, "argv", [str(BUILDER), "--check"]), \
             redirect_stdout(io.StringIO()):
            with self.assertRaises(SystemExit) as raised:
                runpy.run_path(str(BUILDER), run_name="__main__")
        self.assertEqual(raised.exception.code, 0)

    def test_builder_check_reproduces_events_json_byte_for_byte(self):
        before = EVENTS.read_bytes()
        completed = subprocess.run(
            [sys.executable, "-B", str(BUILDER), "--check"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(completed.returncode, 0, completed.stdout + completed.stderr)
        self.assertEqual(EVENTS.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
