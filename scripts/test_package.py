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
    def test_zip_extraction_rejects_unsafe_members_before_writing(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in ("../outside", "/absolute", "C:/drive", "folder\\outside"):
                archive_path = root / "unsafe.zip"
                with zipfile.ZipFile(archive_path, "w") as archive:
                    archive.writestr("valid/file.txt", "safe")
                    archive.writestr(name, "unsafe")
                destination = root / "extracted"
                with zipfile.ZipFile(archive_path) as archive:
                    with self.assertRaises(ValueError):
                        package.extract_zip(archive, destination)
                self.assertFalse(destination.exists())
            with zipfile.ZipFile(archive_path, "w") as archive:
                archive.writestr("Tourne-la-page/file.txt", "content")
            with zipfile.ZipFile(archive_path) as archive:
                package.extract_zip(archive, destination)
            self.assertEqual((destination / "Tourne-la-page/file.txt").read_text(), "content")

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
                self.assertFalse(any(part in {".local", "Cargo.toml"}
                                     for name in names for part in Path(name).parts))
                metadata = [name for name in names if Path(name).name.startswith("Cargo.toml")]
                self.assertTrue(metadata)
                self.assertTrue(all(name.startswith("Tourne-la-page/licenses/rust/")
                                    and name.endswith(".txt") for name in metadata))
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

    def test_install_smoke_resolves_the_launcher_before_changing_working_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package_root = root / "extracted app"
            package_root.mkdir()
            launcher = package_root / "tourne-la-page.exe"
            launcher.write_bytes(b"native-test-executable")
            environment = {"TOURNEPAGE_ENV": "homologacao"}

            def launch(arguments, *, executable, cwd, env, check):
                self.assertTrue(Path(executable).is_absolute())
                self.assertEqual(executable, str(launcher.resolve()))
                self.assertEqual(arguments[0], "tourne-la-page")
                self.assertEqual(arguments[1:], ["--verify-install", "install-verification.json"])
                self.assertEqual(cwd, package_root)
                self.assertEqual(env, environment)
                self.assertTrue(check)
                (cwd / "install-verification.json").write_text(json.dumps({"database": "ok", "version": "0.2.0"}))

            with patch("scripts.package.subprocess.run", side_effect=launch):
                package.verify_installation(launcher, package_root, environment, "0.2.0")

    def test_install_smoke_rejects_a_failed_database_or_a_different_binary_version(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "tourne-la-page.exe").write_bytes(b"native-test-executable")
            for details in ({"database": "failed", "version": "0.2.0"},
                            {"database": "ok", "version": "0.1.0"}):
                def launch(*args, **kwargs):
                    (root / "install-verification.json").write_text(json.dumps(details))
                with patch("scripts.package.subprocess.run", side_effect=launch):
                    with self.assertRaises(SystemExit):
                        package.verify_installation(root / "tourne-la-page.exe", root, {}, "0.2.0")

    def test_install_smoke_rejects_an_executable_outside_the_expected_package_paths(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package_root = root / "package"
            package_root.mkdir()
            for launcher in (root / "tourne-la-page.exe", package_root / "unexpected.exe"):
                launcher.write_bytes(b"not-the-package-launcher")
                with patch("scripts.package.subprocess.run") as run:
                    with self.assertRaises(ValueError):
                        package.verify_installation(launcher, package_root, {}, "0.2.0")
                    run.assert_not_called()
