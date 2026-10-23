"""Small offline data checks, not mechanics tests or a simulation verifier."""

from collections import Counter
import json
from pathlib import Path
import re
import unittest
from urllib.parse import urlparse


ROOT = Path(__file__).resolve().parents[2]
HERE = ROOT / "data" / "relics"
CHARACTERS = {"IRONCLAD", "SILENT", "DEFECT", "WATCHER"}


class RelicDatasetTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue((HERE / "relics.json").is_file(), "Missing relic dataset")
        self.data = json.loads((HERE / "relics.json").read_text())
        self.records = self.data["relics"]
        self.by_id = {record["id"]: record for record in self.records}
        self.sources = json.loads((HERE / "sources.json").read_text())["sources"]

    def test_explicit_target_and_evidence_grade(self):
        self.assertEqual(self.data["game"]["name"], "Slay the Spire")
        self.assertEqual(self.data["game"]["version"], "2.3")
        self.assertEqual(self.data["game"]["platform"], "PC")
        self.assertEqual(self.data["schema_version"], 1)
        self.assertEqual(self.data["verification_level"], "reference_not_runtime_verified")
        self.assertEqual(self.data["project_policy"]["rng"], "RNG-001")

    def test_full_roster_tiers_and_unique_identities(self):
        self.assertEqual(len(self.records), 180)  # 178 registered + two fallback identities
        self.assertEqual(len(self.by_id), 180)
        self.assertEqual(len({r["game_id"] for r in self.records}), 180)
        self.assertEqual(len({r["key"] for r in self.records}), 180)
        self.assertEqual(Counter(r["tier"] for r in self.records), {
            "STARTER": 4, "COMMON": 36, "UNCOMMON": 36, "RARE": 34,
            "BOSS": 30, "SHOP": 20, "SPECIAL": 20,
        })
        self.assertNotIn("INVALID", self.by_id)
        self.assertNotIn("DISCERNING_MONOCLE", self.by_id)

    def test_preserves_existing_141_inventory_ids_and_keys(self):
        inventory = (ROOT / "docs/research/2026-10-18-act1-relics-closure.md").read_text()
        expected = set(re.findall(r"^\| [^|]+ \| `([^`]+)` \| `[^`]+` \| `(?:STARTER|COMMON|UNCOMMON|RARE|BOSS|SHOP|SPECIAL)`", inventory, re.M))
        self.assertEqual(len(expected), 141)
        self.assertLessEqual(expected, {r["game_id"] for r in self.records})
        keys_doc = (ROOT / "docs/research/2026-10-19-act1-complete-catalog-numeric.md").read_text()
        for key, game_id in re.findall(r"^\| `(relic\.[^`]+)` \| `([^`]+)`", keys_doc, re.M):
            record = next(r for r in self.records if r["game_id"] == game_id)
            self.assertEqual(record["key"], key)

    def test_required_mechanics_and_acquisition_fields(self):
        for record in self.records:
            with self.subTest(relic=record["id"]):
                for field in ("name", "effect", "state_and_reset", "key"):
                    self.assertTrue(record[field].strip())
                self.assertTrue(record["triggers"])
                self.assertTrue(record["acquisition"]["sources"])
                self.assertIsInstance(record["acquisition"]["restrictions"], list)
                self.assertTrue(record["characters"])
                self.assertLessEqual(set(record["characters"]), CHARACTERS)
                self.assertIsInstance(record["notes"], list)
                self.assertIsInstance(record["uncertainties"], list)
                self.assertIn(record["availability"], {"normal", "fallback", "unobtainable"})

    def test_all_citations_resolve_and_local_research_keeps_its_grade(self):
        for record in self.records:
            with self.subTest(relic=record["id"]):
                self.assertTrue(record["citations"])
                for key in record["citations"]:
                    source = self.sources[key]
                    self.assertTrue(source["supports"])
                    self.assertTrue(source["retrieved_at"])
                    if source["kind"] == "local_research":
                        self.assertTrue((ROOT / source["local_path"]).is_file())
                    else:
                        self.assertEqual(urlparse(source["url"]).scheme, "https")
                    if source["kind"] == "wiki":
                        self.assertGreater(source["revision_id"], 0)
                        self.assertIn("oldid=", source["url"])
                if record["local_static"] is not None:
                    self.assertEqual(record["local_static"]["grade"], "supporting_static_pending_independent_review")
                    self.assertTrue(record["local_static"]["remaining_boundaries"])
        self.assertEqual(sum(r["local_static"] is not None for r in self.records), 141)

    def test_character_restrictions(self):
        counts = Counter()
        for record in self.records:
            if len(record["characters"]) == 1:
                counts[record["characters"][0]] += 1
        self.assertEqual(counts, {"IRONCLAD": 11, "SILENT": 11, "DEFECT": 9, "WATCHER": 9})
        self.assertEqual(self.by_id["BRIMSTONE"]["characters"], ["IRONCLAD"])
        self.assertEqual(self.by_id["PURE_WATER"]["characters"], ["WATCHER"])
        self.assertEqual(set(self.by_id["CAPTAINS_WHEEL"]["characters"]), CHARACTERS)

    def test_replacements_and_fallback_availability(self):
        for upgrade, starter in {
            "BLACK_BLOOD": "BURNING_BLOOD", "RING_OF_THE_SERPENT": "RING_OF_THE_SNAKE",
            "FROZEN_CORE": "CRACKED_CORE", "HOLY_WATER": "PURE_WATER",
        }.items():
            self.assertEqual(self.by_id[upgrade]["acquisition"]["replaces"], starter)
            self.assertTrue(self.by_id[upgrade]["acquisition"]["restrictions"])
        self.assertEqual(self.by_id["CIRCLET"]["availability"], "fallback")
        self.assertEqual(self.by_id["RED_CIRCLET"]["availability"], "unobtainable")

    def test_randomness_is_qualified_not_an_original_prng_requirement(self):
        for record in self.records:
            rng = record["rng"]
            self.assertIn(rng["kind"], {"none", "direct", "indirect"})
            self.assertTrue(rng["behavior"])
            self.assertEqual(rng["original_prng_parity_required"], False)
            if rng["kind"] != "none":
                self.assertTrue(record["uncertainties"])

    def test_mark_of_pain_wound_insertion_is_random(self):
        self.assertEqual(self.by_id["MARK_OF_PAIN"]["rng"]["kind"], "direct")
        self.assertTrue(self.by_id["MARK_OF_PAIN"]["uncertainties"])

    def test_m8_red_mask_keeps_startup_snapshot_and_artifact_source_qualified(self):
        record = self.by_id["RED_MASK"]
        facts = record["static_qualification"]
        self.assertEqual(facts["target_snapshot"], "CURRENT_GROUP_IN_ORDER_AT_BATTLE_START")
        self.assertEqual(facts["application"], "PLAYER_SOURCE_WEAK1_ORDINARY_ARTIFACT")
        self.assertFalse(facts["upstream_build_channel_verified"])
        self.assertFalse(facts["original_runtime_validated"])
        self.assertIn("local:m8-events", record["citations"])
        self.assertIsNone(record["local_static"], "new finding must not upgrade the historical 141-record lane")


if __name__ == "__main__":
    unittest.main()
