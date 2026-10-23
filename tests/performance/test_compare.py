"""Protect compatibility, exact budget edges, and consumer-visible exit codes."""
import contextlib
import copy
import io
import json
import tempfile
import unittest
from pathlib import Path
from support import result, digest
from performance.compare import compare_results, main


def rehash(doc):
    doc["catalog_sha256"] = digest(doc["catalog"])
    contracts = {c["id"]: c for c in doc["catalog"]["cases"]}
    for row in doc["cases"]:
        row["contract_sha256"] = digest(contracts[row["id"]])


class CompareTests(unittest.TestCase):
    def test_identical_and_doubled_samples(self):
        baseline = result(); candidate = copy.deepcopy(baseline)
        self.assertEqual(compare_results(baseline, candidate)["rows"][0]["latency_change_percent"], 0)
        for sample in candidate["cases"][0]["samples"]: sample["elapsed_ns"] *= 2
        row = compare_results(baseline, candidate)["rows"][0]
        self.assertEqual(row["latency_change_percent"], 100)
        self.assertEqual(row["throughput_change_percent"], -50)

    def test_exact_threshold_and_invalid_budgets(self):
        for numerator, passed in [(110, True), (120, False), (90, True)]:
            candidate = result()
            for s in candidate["cases"][0]["samples"]: s["elapsed_ns"] = s["elapsed_ns"] * numerator // 100
            self.assertEqual(compare_results(result(), candidate, max_regression_percent=10)["passed"], passed)
        for budget in [-1, float("nan"), float("inf"), True]:
            with self.assertRaises(ValueError): compare_results(result(), result(), max_regression_percent=budget)

    def test_work_identity_changes_are_incompatible(self):
        for field, value in [("input_sha256", "0" * 64), ("actions_sha256", "1" * 64),
                             ("version", 2), ("expected_contract_version", 2), ("endpoint", {"outcome": "death"}),
                             ("accepted_per_episode", 11)]:
            with self.subTest(field=field):
                candidate = result(); row = candidate["cases"][0]; row[field] = value
                if field in {"version", "expected_contract_version"}: candidate["catalog"]["cases"][0][field] = value
                if field == "endpoint": candidate["catalog"]["cases"][0]["expected"] = value
                if field == "accepted_per_episode":
                    for sample in row["samples"]: sample["accepted_actions"] = 11
                rehash(candidate)
                report = compare_results(result(), candidate, strict=True)
                self.assertEqual(report["rows"][0]["status"], "incompatible")
                self.assertFalse(report["passed"])

    def test_machine_build_sampling_changes_are_incompatible(self):
        for section, field, value in [("machine", "cpu", "other"), ("machine", "runner_id", "other"),
                                       ("machine", "cpu", "unknown"), ("build", "flags", ["-Ctarget-cpu=native"]),
                                       ("build", "rustc_vv", "other"), ("sampling", "warmup", 0)]:
            candidate = result(); candidate[section][field] = value
            self.assertEqual(compare_results(result(), candidate)["rows"][0]["status"], "incompatible")
        a = result(); a["machine"]["runner_id"] = ""
        self.assertEqual(compare_results(a, a)["comparable"], 0)

    def test_source_and_catalog_expansion_do_not_invalidate_existing_work(self):
        candidate = result(); candidate["provenance"]["git_revision"] = "f" * 40
        candidate["provenance"]["source_sha256"] = "0" * 64
        candidate["catalog"]["families"].append({"id": "future", "status": "planned", "requires": ["future"], "milestones": ["M36"]})
        rehash(candidate)
        self.assertEqual(compare_results(result(), candidate)["comparable"], 1)

    def test_added_missing_and_changed_boundary_are_visible(self):
        candidate = result(); candidate["cases"][0]["boundary"] = "public-decision-loop"
        report = compare_results(result(), candidate, strict=True)
        self.assertEqual({r["status"] for r in report["rows"]}, {"missing", "added"})
        self.assertFalse(report["passed"])
        candidate = result(); contract = copy.deepcopy(candidate["catalog"]["cases"][0]); contract["id"] = "new"
        candidate["catalog"]["cases"].append(contract)
        row = copy.deepcopy(candidate["cases"][0]); row["id"] = "new"; candidate["cases"].append(row); rehash(candidate)
        self.assertTrue(compare_results(result(), candidate, strict=True)["passed"])

    def test_cli_reports_exit_codes_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as tmp, contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            a, b = Path(tmp)/"a.json", Path(tmp)/"b.json"
            original = json.dumps(result()); a.write_text(original); b.write_text(original)
            self.assertEqual(main([str(a), str(b), "--strict", "--max-regression-percent", "0", "--json"]), 0)
            slower = result()
            for s in slower["cases"][0]["samples"]: s["elapsed_ns"] *= 2
            b.write_text(json.dumps(slower))
            self.assertEqual(main([str(a), str(b), "--max-regression-percent", "10"]), 1)
            b.write_text("{}")
            self.assertEqual(main([str(a), str(b)]), 2)
            self.assertEqual(a.read_text(), original)


if __name__ == "__main__": unittest.main()
