"""Check the actual R3000 hash helper stays inline and uses byte-safe loads.

This checks generated instructions, not runtime performance or FNV equivalence;
stream_hash.cpp and the complete emulator route supply those separate checks.
"""
from pathlib import Path
import subprocess
import tempfile
import unittest

from test_gte_transfer_codegen import BIN, ROOT, SUFFIX, instructions


class HashCodegen(unittest.TestCase):
    def test_r3000_hot_loop(self):
        with tempfile.TemporaryDirectory(prefix="epok-hash-codegen-") as directory:
            for optimization in ("-O2", "-O3"):
                with self.subTest(optimization=optimization):
                    obj = Path(directory) / "hash.o"
                    result = subprocess.run([
                        str(BIN / f"mipsel-none-elf-g++{SUFFIX}"),
                        "-std=c++20", optimization, "-march=r3000", "-mabi=32", "-ffreestanding",
                        "-I", str(ROOT / "runtime"), "-c", str(ROOT / "tests/runtime/stream_hash_codegen.cpp"),
                        "-o", str(obj)], capture_output=True, text=True)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    assembly = subprocess.check_output([
                        str(BIN / f"mipsel-none-elf-objdump{SUFFIX}"), "-d", "-z", str(obj)], text=True)
                    body = instructions(assembly, "hash_update")
                    opcodes = [line.split()[0] for line in body]
                    self.assertFalse(set(opcodes) & {"jal", "jalr", "bal", "mult", "multu", "mflo"})
                    self.assertFalse(set(opcodes) & {"lw", "lwl", "lwr", "lh", "lhu", "sw"})
                    self.assertGreaterEqual(opcodes.count("lbu"), 4)


if __name__ == "__main__":
    unittest.main()
