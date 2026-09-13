"""Offline Windows package installation checks, isolated from the editor's tools."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[2]


@unittest.skipUnless(os.name == "nt", "Windows package installer")
class DependencySetup(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="epok-dependencies-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "tools").mkdir()
        (self.root / ".tools").mkdir()
        shutil.copy2(ROOT / "tools/setup.ps1", self.root / "tools/setup.ps1")
        self.archive = self.root / ".tools/psxavenc.zip"
        with zipfile.ZipFile(self.archive, "w") as archive:
            archive.writestr("bin/psxavenc.exe", b"fixture executable")
            archive.writestr("bin/support.dll", b"fixture companion")
        self.manifest = {
            "schema_version": 1,
            "nugget": {"path": "must-not-touch-sdk"},
            "archives": [{
                "name": "psxavenc", "version": "test", "directory": "psxavenc",
                "archive": "psxavenc.zip", "url": "https://invalid.example/not-used",
                "sha256": hashlib.sha256(self.archive.read_bytes()).hexdigest(),
            }, {
                "name": "unselected", "directory": "mips",
                "url": "https://invalid.example/must-not-download",
            }],
        }
        self.write_manifest()
        (self.root / "Local.epokconfig").write_text("psxavenc: custom/location.exe\n")

    def write_manifest(self):
        (self.root / "tools/dependencies.json").write_text(json.dumps(self.manifest))

    def run_setup(self, *args):
        return subprocess.run([
            "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
            "-File", str(self.root / "tools/setup.ps1"), "-Dependency", "psxavenc", *args,
        ], capture_output=True, text=True, timeout=30,
            creationflags=subprocess.CREATE_NO_WINDOW)

    def test_selected_package_repairs_companions_and_preserves_custom_files(self):
        result = self.run_setup()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        folder = self.root / ".tools/psxavenc/bin"
        self.assertEqual((folder / "support.dll").read_bytes(), b"fixture companion")
        (folder / "support.dll").unlink()
        (folder / "custom.txt").write_text("keep")
        self.assertNotEqual(self.run_setup().returncode, 0)
        result = self.run_setup("-Repair")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertTrue((folder / "support.dll").is_file())
        self.assertEqual((folder / "custom.txt").read_text(), "keep")
        self.assertFalse((self.root / ".tools/mips").exists())
        self.assertFalse((self.root / "must-not-touch-sdk").exists())
        self.assertEqual((self.root / "Local.epokconfig").read_text(), "psxavenc: custom/location.exe\n")

    def test_bad_checksum_does_not_extract(self):
        self.archive.write_bytes(b"corrupt cached archive")
        self.assertNotEqual(self.run_setup("-Repair").returncode, 0)
        self.assertFalse((self.root / ".tools/psxavenc").exists())

    def junction(self, link, target):
        # Static script and structured argv keep paths out of PowerShell source.
        script = self.root / "junction.ps1"
        script.write_text("param($LinkPath, $TargetPath)\nNew-Item -ItemType Junction -Path $LinkPath -Target $TargetPath -ErrorAction Stop | Out-Null\n")
        subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(script), str(link), str(target)], check=True, capture_output=True,
                       creationflags=subprocess.CREATE_NO_WINDOW)

    def test_repair_preserves_dangling_junction_and_installs_real_directory(self):
        target = self.root / "old-installation"
        target.mkdir()
        package = self.root / ".tools/psxavenc"
        self.junction(package, target)
        target.rmdir()
        self.assertNotEqual(self.run_setup().returncode, 0)
        result = self.run_setup("-Repair")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(package.is_junction())
        self.assertEqual((package / "bin/psxavenc.exe").read_bytes(), b"fixture executable")
        backups = list((self.root / ".tools").glob("psxavenc.link-backup-*"))
        self.assertEqual(len(backups), 1)
        self.assertTrue(backups[0].is_junction())
        self.assertFalse(target.exists())

    def test_repair_does_not_write_through_live_junction(self):
        target = self.root / "shared-package"
        target.mkdir()
        (target / "keep.txt").write_text("untouched")
        package = self.root / ".tools/psxavenc"
        self.junction(package, target)
        result = self.run_setup("-Repair")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(list(target.iterdir()), [target / "keep.txt"])
        self.assertTrue((package / "bin/support.dll").is_file())

    def test_nested_junction_cannot_redirect_extraction(self):
        target = self.root / "shared-bin"
        target.mkdir()
        package = self.root / ".tools/psxavenc"
        package.mkdir()
        self.junction(package / "bin", target)
        result = self.run_setup("-Repair")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(list(target.iterdir()), [])

    def test_archive_cannot_escape_selected_package(self):
        with zipfile.ZipFile(self.archive, "w") as archive:
            archive.writestr("../escaped.txt", b"must not escape")
        self.manifest["archives"][0]["sha256"] = hashlib.sha256(self.archive.read_bytes()).hexdigest()
        self.write_manifest()
        self.assertNotEqual(self.run_setup("-Repair").returncode, 0)
        self.assertFalse((self.root / ".tools/escaped.txt").exists())


if __name__ == "__main__":
    unittest.main()
