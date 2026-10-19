from pathlib import Path
import hashlib
import json
import plistlib
import tarfile
import tempfile
import unittest
import zipfile
from unittest.mock import patch

from scripts import package


class Packages(unittest.TestCase):
    def test_linux_bundle_contains_launcher_binary_licenses_and_verified_manifest(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / "binary"
            binary.write_bytes(b"native-test-executable")
            output = root / "output"
            archive = package.bundle(binary, "linux-x64", "0.2.0", output)
            self.assertEqual(len(list(output.iterdir())), 1)
            with tarfile.open(archive) as source:
                names = source.getnames()
                self.assertIn("Tourne-la-page/AppRun", names)
                self.assertIn("Tourne-la-page/bin/tourne-la-page", names)
                self.assertIn("Tourne-la-page/licenses/Inter-OFL.txt", names)
                self.assertFalse(any(".local" in name or "Cargo.toml" in name for name in names))
                manifest = json.load(source.extractfile("Tourne-la-page/manifest.json"))
                digest = hashlib.sha256(source.extractfile("Tourne-la-page/bin/tourne-la-page").read()).hexdigest()
                self.assertEqual(manifest["files"]["bin/tourne-la-page"], digest)
                self.assertTrue(source.getmember("Tourne-la-page/AppRun").mode & 0o111)

    def test_windows_and_mac_bundles_have_platform_layouts(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / "binary"
            binary.write_bytes(b"native-test-executable")
            windows = package.bundle(binary, "windows-arm64", "0.2.0", root / "windows")
            with zipfile.ZipFile(windows) as source:
                self.assertIn("Tourne-la-page/tourne-la-page.exe", source.namelist())
                self.assertIn("Tourne-la-page/tourne-la-page.ico", source.namelist())
            with patch("scripts.package.platform.system", return_value="Linux"):
                mac = package.bundle(binary, "macos-arm64", "0.2.0", root / "mac")
            with zipfile.ZipFile(mac) as source:
                prefix = "Tourne-la-page/Tourne la page.app/Contents/"
                info = plistlib.loads(source.read(prefix + "Info.plist"))
                self.assertEqual(info["CFBundleIdentifier"], "io.github.BrunodosSantosVaz.TourneLaPage")
                self.assertEqual(info["CFBundleShortVersionString"], "0.2.0")
                self.assertIn(prefix + "MacOS/tourne-la-page", source.namelist())
                self.assertTrue((source.getinfo(prefix + "MacOS/tourne-la-page").external_attr >> 16) & 0o111)
                self.assertEqual(info["LSMinimumSystemVersion"], "12.0")
                self.assertIn(prefix + "Resources/licenses/rust/index.json", source.namelist())

    def test_invalid_platform_or_version_never_creates_an_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / "binary"
            binary.write_bytes(b"test")
            for system, version in [("bad-platform", "0.2.0"), ("linux-x64", "../../outside")]:
                with self.assertRaises(ValueError):
                    package.bundle(binary, system, version, root / "output")
            self.assertFalse((root / "output").exists())
