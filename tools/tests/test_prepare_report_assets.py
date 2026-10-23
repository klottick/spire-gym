import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("prepare_report_assets", Path(__file__).parents[1] / "prepare_report_assets.py")
assets = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(assets)
# Generated PNG header fixture; no proprietary art, browser decoding claimed separately.
PNG = bytes.fromhex("89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000b49444154789c636000020000050001a5f645400000000049454e44ae426082")


class PrepareAssetsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="spire-asset-test-", dir="/tmp/opencode")
        self.root = Path(self.temp.name)
        self.sources = self.root / "sources.json"
        self.sources.write_text(json.dumps({"card/bash": "https://example.invalid/Bash.png", "card/strike": "https://example.invalid/Strike.png"}))
        self.local = self.root / "originals"
        self.local.mkdir()
        self.output = self.root / "cache"

    def tearDown(self):
        self.temp.cleanup()

    def invoke(self, *extra):
        return assets.main(["--sources", str(self.sources), "--output-dir", str(self.output), *extra])

    def test_local_import_never_downloads_and_preserves_attribution(self):
        (self.local / "Bash.png").write_bytes(PNG)
        with patch.object(assets.urllib.request.OpenerDirector, "open", side_effect=AssertionError("unexpected network")):
            self.assertEqual(self.invoke("--local-dir", str(self.local)), 0)
        report = json.loads((self.output / "manifest.json").read_text())
        self.assertEqual(set(report["assets"]), {"card/bash"})
        self.assertEqual((self.output / report["assets"]["card/bash"]["path"]).read_bytes(), PNG)
        self.assertTrue(any("Mega Crit" in text for text in report["attribution"]))

    def test_download_requires_explicit_named_key(self):
        with patch.object(assets.urllib.request.OpenerDirector, "open", side_effect=AssertionError("unexpected network")):
            self.assertNotEqual(self.invoke("--download"), 0)
        self.assertFalse(self.output.exists())

    def test_explicit_download_uses_only_the_selected_https_image(self):
        class Response(io.BytesIO):
            def geturl(self):
                return "https://example.invalid/Bash.png"
        with patch.object(assets.urllib.request.OpenerDirector, "open", return_value=Response(PNG)) as network:
            self.assertEqual(self.invoke("--download", "--key", "card/bash"), 0)
        self.assertEqual(network.call_count, 1)
        report = json.loads((self.output / "manifest.json").read_text())
        self.assertEqual(set(report["assets"]), {"card/bash"})

    def test_redirect_rejects_downgrade_before_following_it(self):
        request = assets.urllib.request.Request("https://example.invalid/Bash.png")
        handler = assets.HttpsOnlyRedirect()
        with self.assertRaises(ValueError):
            handler.redirect_request(request, None, 302, "Found", {}, "http://example.invalid/Bash.png")

    def test_symlinks_and_existing_different_files_are_not_overwritten(self):
        (self.local / "Bash.png").write_bytes(PNG)
        self.output.mkdir()
        manifest = self.output / "manifest.json"
        manifest.symlink_to(self.root / "missing")
        self.assertNotEqual(self.invoke("--local-dir", str(self.local)), 0)
        self.assertTrue(manifest.is_symlink())
        manifest.unlink()
        manifest.write_text("unrelated content")
        self.assertNotEqual(self.invoke("--local-dir", str(self.local)), 0)
        self.assertEqual(manifest.read_text(), "unrelated content")

    def test_local_symlink_and_invalid_images_are_rejected(self):
        private = self.root / "private.png"
        private.write_bytes(PNG)
        (self.local / "Bash.png").symlink_to(private)
        self.assertNotEqual(self.invoke("--local-dir", str(self.local)), 0)
        (self.local / "Bash.png").unlink()
        (self.local / "Bash.png").write_bytes(b"<svg onload='alert(1)'/>")
        self.assertNotEqual(self.invoke("--local-dir", str(self.local)), 0)

    def test_unknown_keys_and_non_https_sources_fail_before_writes(self):
        self.assertNotEqual(self.invoke("--download", "--key", "card/unknown"), 0)
        self.sources.write_text(json.dumps({"card/bash": "file:///etc/passwd"}))
        self.assertNotEqual(self.invoke("--download", "--key", "card/bash"), 0)
        self.assertFalse(self.output.exists())

    def test_identical_cache_is_reusable_but_directory_symlinks_are_rejected(self):
        (self.local / "Bash.png").write_bytes(PNG)
        self.assertEqual(self.invoke("--local-dir", str(self.local), "--key", "card/bash"), 0)
        original = (self.output / "manifest.json").read_bytes()
        self.assertEqual(self.invoke("--local-dir", str(self.local), "--key", "card/bash"), 0)
        self.assertEqual((self.output / "manifest.json").read_bytes(), original)
        other = self.root / "outside"
        other.mkdir()
        linked = self.root / "linked-cache"
        linked.symlink_to(other, target_is_directory=True)
        self.assertNotEqual(assets.main(["--sources", str(self.sources), "--output-dir", str(linked),
                                        "--local-dir", str(self.local)]), 0)
        self.assertEqual(list(other.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
