"""Exercise launcher configuration without a compositor, GPU or Chromium."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


LAUNCHER = Path(__file__).resolve().parents[2] / "appliance/kiosk/launch.sh"


class KioskConfiguration(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        bin_dir = self.root / "bin"
        bin_dir.mkdir()
        for name, script in {
            "cage": '#!/bin/sh\n[ "$1" = -- ] || exit 1\nshift\nexec "$@"\n',
            "chromium": '#!/bin/sh\nprintf "%s\\n" "$@"\n',
        }.items():
            command = bin_dir / name
            command.write_text(script)
            command.chmod(0o755)
        self.env = {
            key: value for key, value in os.environ.items()
            if not key.startswith(("SONGVERSE_", "PULSE_"))
        }
        self.env["PATH"] = str(bin_dir) + os.pathsep + os.environ["PATH"]
        self.profile = str(self.root / "profile with spaces")

    def launch(self, **config):
        return subprocess.run(
            ["sh", str(LAUNCHER)], env={**self.env, **config},
            text=True, capture_output=True, check=False,
        )

    def test_default_screen_and_private_profile(self):
        Path(self.profile).mkdir(mode=0o755)
        result = self.launch(PULSE_CHROMIUM_PROFILE=self.profile)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines()[-1], "https://screen.oneverse.top")
        self.assertIn("--user-data-dir=" + self.profile, result.stdout.splitlines())
        self.assertEqual(Path(self.profile).stat().st_mode & 0o777, 0o700)

    def test_legacy_variables_warn_and_work(self):
        result = self.launch(
            SONGVERSE_SCREEN_URL="http://127.0.0.1:8787/screen",
            SONGVERSE_CHROMIUM_PROFILE=self.profile,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines()[-1], "http://127.0.0.1:8787/screen")
        self.assertIn("--user-data-dir=" + self.profile, result.stdout.splitlines())
        self.assertIn("SONGVERSE_SCREEN_URL is deprecated", result.stderr)
        self.assertIn("SONGVERSE_CHROMIUM_PROFILE is deprecated", result.stderr)

    def test_new_variables_win(self):
        result = self.launch(
            PULSE_SCREEN_URL="https://screen.example.test",
            PULSE_CHROMIUM_PROFILE=self.profile,
            SONGVERSE_SCREEN_URL="https://legacy.example.test",
            SONGVERSE_CHROMIUM_PROFILE=str(self.root / "legacy"),
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines()[-1], "https://screen.example.test")
        self.assertIn("--user-data-dir=" + self.profile, result.stdout.splitlines())
        self.assertFalse((self.root / "legacy").exists())

    def test_refuses_non_loopback_http_and_explicit_empty_url(self):
        for url in ["http://192.168.1.2/screen", "file:///etc/passwd", ""]:
            with self.subTest(url=url):
                result = self.launch(
                    PULSE_SCREEN_URL=url, PULSE_CHROMIUM_PROFILE=self.profile,
                    SONGVERSE_SCREEN_URL="https://legacy.example.test",
                )
                self.assertEqual(result.returncode, 2)
                self.assertIn("Refusing unsupported screen URL", result.stderr)

    def test_empty_profile_does_not_fall_back_to_legacy(self):
        result = self.launch(
            PULSE_CHROMIUM_PROFILE="", SONGVERSE_CHROMIUM_PROFILE=self.profile,
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("Chromium profile must not be empty", result.stderr)


if __name__ == "__main__":
    unittest.main()
