"""Protect source attribution and baseline files; builds are external doubles."""
import argparse
import contextlib
import copy
import io
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from support import result
from performance.run import build_and_collect, main, source_fingerprint


class DriverTests(unittest.TestCase):
    def options(self, root):
        return argparse.Namespace(runner_id="test-host", target_dir=root/"target"/"bench",
                                  samples=2, iterations=1, warmup=1, case=None)

    def native(self):
        d = result(); d["schema"] = "spire-gym-native-samples-v1"
        for field in ["provenance", "machine", "build"]: d.pop(field)
        public = copy.deepcopy(d["cases"][0]); public["boundary"] = "public-decision-loop"
        d["cases"].append(public)
        return d

    def fake_process(self, root, *, fail=False, artifact=True, malformed=False, wrong_artifact=False, bad_profile=False, missing_row=False):
        def execute(args, **kwargs):
            if args[:2] == ["cargo", "build"]:
                # A missing offline flag simulates forbidden dependency fetching.
                if not {"--offline", "--locked", "--release"}.issubset(args):
                    raise AssertionError("benchmark build is not offline/locked/release")
                if fail: raise subprocess.CalledProcessError(101, args, stderr="build failed")
                target = Path(kwargs["env"]["CARGO_TARGET_DIR"])
                exe = target/"x86_64-unknown-linux-gnu"/"release"/"examples"/"performance"
                exe.parent.mkdir(parents=True, exist_ok=True); exe.write_text("fake executable")
                message = {"reason": "compiler-artifact", "target": {"name": "performance", "kind": ["example"]},
                           "executable": str(exe), "profile": {"opt_level": "3", "debug_assertions": False,
                                                                    "overflow_checks": False, "test": False}}
                if wrong_artifact: message["executable"] = str(root/"other-executable")
                if bad_profile: message["profile"]["opt_level"] = "0"
                return subprocess.CompletedProcess(args, 0, json.dumps(message) if artifact else "", "")
            if args[:2] == ["rustc", "-vV"]:
                return subprocess.CompletedProcess(args, 0, "rustc 1.99.0\nhost: x86_64-unknown-linux-gnu\n", "")
            if args[:2] == ["git", "rev-parse"]: return subprocess.CompletedProcess(args, 0, "a"*40+"\n", "")
            if args[:2] == ["git", "status"]: return subprocess.CompletedProcess(args, 0, "", "")
            if args[:2] == ["git", "ls-files"]: return subprocess.CompletedProcess(args, 0, "performance/catalog.json\0", "")
            native = self.native()
            if missing_row: native["cases"].pop()
            return subprocess.CompletedProcess(args, 0, "{}" if malformed else json.dumps(native), "")
        return execute

    def set_catalog(self, root):
        (root/"performance").mkdir()
        (root/"performance"/"catalog.json").write_text(json.dumps(result()["catalog"]))

    def test_builds_offline_and_uses_returned_release_artifact(self):
        with tempfile.TemporaryDirectory(dir="/tmp/opencode") as tmp:
            root = Path(tmp); self.set_catalog(root)
            with patch("performance.run.subprocess.run", side_effect=self.fake_process(root)):
                d = build_and_collect(root, self.options(root))
            self.assertEqual(d["schema"], "spire-gym-performance-v1")
            self.assertEqual(d["provenance"]["git_revision"], "a"*40)
            self.assertEqual(d["build"]["profile"]["opt_level"], "3")
            self.assertEqual(d["machine"]["runner_id"], "test-host")
            self.assertTrue(d["cases"][0]["preflight_passed"])

    def test_failed_build_missing_artifact_and_malformed_output_are_not_results(self):
        for flags in [{"fail": True}, {"artifact": False}, {"malformed": True},
                      {"wrong_artifact": True}, {"bad_profile": True}, {"missing_row": True}]:
            with tempfile.TemporaryDirectory(dir="/tmp/opencode") as tmp:
                root = Path(tmp); self.set_catalog(root)
                with patch("performance.run.subprocess.run", side_effect=self.fake_process(root, **flags)):
                    with self.assertRaises((ValueError, subprocess.CalledProcessError)):
                        build_and_collect(root, self.options(root))

    def test_source_edit_during_collection_rejects_attribution(self):
        with tempfile.TemporaryDirectory(dir="/tmp/opencode") as tmp:
            root = Path(tmp); self.set_catalog(root)
            with patch("performance.run.subprocess.run", side_effect=self.fake_process(root)), \
                 patch("performance.run.source_fingerprint", side_effect=["a"*64, "b"*64]):
                with self.assertRaisesRegex(ValueError, "source"):
                    build_and_collect(root, self.options(root))

    def test_source_fingerprint_sees_untracked_edits_but_not_target_outputs(self):
        with tempfile.TemporaryDirectory(dir="/tmp/opencode") as tmp:
            root = Path(tmp); (root/"source.rs").write_text("first")
            def files(args, **kwargs):
                paths = "source.rs\0" + ("new.rs\0" if (root/"new.rs").exists() else "")
                return subprocess.CompletedProcess(args, 0, paths, "")
            with patch("performance.run.subprocess.run", side_effect=files):
                a = source_fingerprint(root); (root/"source.rs").write_text("second")
                b = source_fingerprint(root); self.assertNotEqual(a, b)
                (root/"new.rs").write_text("new public source"); c = source_fingerprint(root)
                self.assertNotEqual(b, c)
                (root/"target").mkdir(); (root/"target"/"ignored").write_text("output")
                self.assertEqual(source_fingerprint(root), c)

    def test_existing_baseline_and_failed_collect_never_truncate_evidence(self):
        with tempfile.TemporaryDirectory(dir="/tmp/opencode") as tmp, \
             contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            output = Path(tmp)/"baseline.json"; output.write_text("original evidence")
            args = ["--runner-id", "test-host", "--output", str(output)]
            with patch("performance.run.build_and_collect", return_value=result()):
                self.assertEqual(main(args), 2)
                self.assertEqual(output.read_text(), "original evidence")
                self.assertEqual(main(args+["--overwrite"]), 0)
            retained = output.read_bytes()
            with patch("performance.run.build_and_collect", side_effect=ValueError("bad run")):
                self.assertEqual(main(args+["--overwrite"]), 2)
            self.assertEqual(output.read_bytes(), retained)

    def test_invalid_samples_or_runner_reject_before_build(self):
        with tempfile.TemporaryDirectory(dir="/tmp/opencode") as tmp, \
             contextlib.redirect_stderr(io.StringIO()):
            for flags in [["--samples", "0"], ["--iterations", "1001"], ["--warmup", "101"]]:
                self.assertEqual(main(["--runner-id", "test", "--output", str(Path(tmp)/"unused"), *flags]), 2)


if __name__ == "__main__": unittest.main()
