import os
from pathlib import Path
import tempfile
import unittest

from redux_host import configured_emulator, default_engine, openbios


class ReduxHostTests(unittest.TestCase):
    def test_default_engine_is_this_checkout(self):
        self.assertTrue((default_engine() / "research/psx-reference/tools/redux_host.py").is_file())

    def test_openbios_prefers_beside_executable_then_macos_bundle_fast_boot_first(self):
        with tempfile.TemporaryDirectory() as root:
            macos = Path(root, "PCSX-Redux.app/Contents/MacOS")
            resources = Path(root, "PCSX-Redux.app/Contents/Resources/share/pcsx-redux/resources")
            macos.mkdir(parents=True)
            resources.mkdir(parents=True)
            with self.assertRaises(FileNotFoundError):
                openbios(macos)
            (resources / "openbios.bin").write_bytes(b"")
            self.assertEqual(openbios(macos), (resources / "openbios.bin").resolve())
            (resources / "openbios-fastboot.bin").write_bytes(b"")
            self.assertEqual(openbios(macos).name, "openbios-fastboot.bin")
            (macos / "openbios.bin").write_bytes(b"")
            self.assertEqual(openbios(macos), (macos / "openbios.bin").resolve())

    def test_configured_emulator_is_relative_to_engine_and_must_exist(self):
        with tempfile.TemporaryDirectory() as root:
            engine = Path(root)
            (engine / "tools").mkdir()
            (engine / "tools/epok_documents.py").write_bytes((default_engine() / "tools/epok_documents.py").read_bytes())
            (engine / "Local.epokconfig").write_text('emulator: ".tools/redux/pcsx-redux.exe"\n', encoding="utf-8")
            with self.assertRaises(FileNotFoundError):
                configured_emulator(engine)
            redux = engine / ".tools/redux"
            redux.mkdir(parents=True)
            (redux / "pcsx-redux.exe").write_bytes(b"")
            self.assertEqual(configured_emulator(engine), (redux / "pcsx-redux.exe").resolve())
            (redux / "pcsx-redux.main").write_bytes(b"")
            self.assertEqual(configured_emulator(engine), (redux / "pcsx-redux.main").resolve())


if __name__ == "__main__":
    unittest.main()
