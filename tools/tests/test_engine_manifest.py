"""Executable bundle integrity must include every file and reject links."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "engine-manifest.py"
SPEC = importlib.util.spec_from_file_location("engine_manifest", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class EngineManifest(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        for name in ("runtime/bin/node", "engine/dist/main.js", "console/pulse.html"):
            file = self.root / name
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_text("fixture")
        self.manifest = {"format": 1, "source_revision": "a" * 40, "files": MODULE.files(self.root)}
        (self.root / "manifest.json").write_text(json.dumps(self.manifest))

    def test_verifies_complete_bundle_and_rejects_tampering(self):
        MODULE.verify(self.root)
        (self.root / "engine/dist/main.js").write_text("tampered")
        with self.assertRaisesRegex(ValueError, "mismatch"):
            MODULE.verify(self.root)

    def test_rejects_unlisted_executable_files(self):
        (self.root / "engine/injected.js").write_text("unlisted")
        with self.assertRaisesRegex(ValueError, "mismatch"):
            MODULE.verify(self.root)

    def test_rejects_links_even_to_internal_files(self):
        (self.root / "engine/linked.js").symlink_to("dist/main.js")
        with self.assertRaisesRegex(ValueError, "symlink"):
            MODULE.verify(self.root)


if __name__ == "__main__":
    unittest.main()
