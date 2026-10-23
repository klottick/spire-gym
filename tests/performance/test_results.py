"""Protect result validity so less work cannot masquerade as greater speed."""
import unittest
from pathlib import Path
from support import result, digest
from performance.results import load_catalog, validate_result, summarize


class ResultsTests(unittest.TestCase):
    def test_catalog_tracks_real_cases_and_full_future_scope(self):
        catalog = load_catalog(Path("performance/catalog.json"))
        self.assertEqual({c["id"] for c in catalog["cases"]}, {
            "combat.ironclad-a0.jaw-worm", "run.ironclad-a0.act1-guardian",
            "run.ironclad-a0.act2-prefix-champ"})
        self.assertEqual(catalog["target"]["characters"], ["ironclad", "silent", "defect", "watcher"])
        self.assertEqual(catalog["target"]["ascensions"], [0, 20])
        self.assertEqual(catalog["target"]["goal"], "heart")
        self.assertEqual(catalog["target"]["primary_ascension"], 20)
        self.assertEqual(len(catalog["families"]), 8)
        for family in catalog["families"]:
            self.assertTrue(family["requires"])
            self.assertTrue(family["milestones"])

    def test_valid_result_and_hand_checked_summary(self):
        doc = result()
        self.assertIs(validate_result(doc), doc)
        summary = summarize(doc["cases"][0]["samples"])
        self.assertEqual(summary["median_elapsed_ns"], 1500)
        self.assertEqual(summary["median_ns_per_action"], 150)
        self.assertEqual(summary["min_ns_per_action"], 100)
        self.assertEqual(summary["max_ns_per_action"], 200)

    def test_rejects_duplicate_ids(self):
        doc = result()
        doc["cases"].append(doc["cases"][0])
        with self.assertRaises(ValueError): validate_result(doc)

    def test_planned_case_cannot_have_samples(self):
        doc = result()
        doc["catalog"]["cases"][0]["status"] = "planned"
        doc["catalog_sha256"] = digest(doc["catalog"])
        doc["cases"][0]["contract_sha256"] = digest(doc["catalog"]["cases"][0])
        with self.assertRaises(ValueError): validate_result(doc)

    def test_rejects_bool_zero_negative_nonfinite_samples(self):
        for value in [True, 0, -1, float("nan"), float("inf"), "1000"]:
            with self.subTest(value=value):
                doc = result(); doc["cases"][0]["samples"][0]["elapsed_ns"] = value
                with self.assertRaises(ValueError): validate_result(doc)

    def test_counts_equal_episodes_times_actions(self):
        doc = result(); doc["cases"][0]["samples"][0]["accepted_actions"] = 9
        with self.assertRaises(ValueError): validate_result(doc)

    def test_all_samples_and_rows_match_sampling(self):
        for key, value in [("episodes", 2), ("episodes", True), ("accepted_actions", 0)]:
            doc = result(); doc["cases"][0]["samples"][0][key] = value
            with self.assertRaises(ValueError): validate_result(doc)
        doc = result(); doc["sampling"]["samples"] = 3
        with self.assertRaises(ValueError): validate_result(doc)

    def test_unknown_schema_rejects(self):
        doc = result(); doc["schema"] = "future"
        with self.assertRaises(ValueError): validate_result(doc)

    def test_contract_hash_and_preflight_cannot_be_forged(self):
        for key, value in [("contract_sha256", "f" * 64), ("preflight_passed", False),
                           ("input_sha256", "not-a-hash"), ("version", True)]:
            doc = result(); doc["cases"][0][key] = value
            with self.assertRaises(ValueError): validate_result(doc)


if __name__ == "__main__": unittest.main()
