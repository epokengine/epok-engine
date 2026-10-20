"""Cross-compile the real GTE transfer helpers and audit register/delay slots.

This checks emitted instructions, not GTE arithmetic or physical-PSX execution.
The emulator's independent ordered-validation pass remains required.
"""
from pathlib import Path
import re
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / ".tools/mips/bin"
SUFFIX = ".exe" if (BIN / "mipsel-none-elf-g++.exe").exists() else ""


def instructions(disassembly, function):
    body = re.search(rf"^[0-9a-f]+ <{function}>:\n(.*?)(?=\n[0-9a-f]+ <|\Z)",
                     disassembly, re.M | re.S)
    assert body, function
    return [re.sub(r"\s+", " ", match.group(1)).strip()
            for match in re.finditer(r"^\s*[0-9a-f]+:\s+[0-9a-f]{8}\s+(.+)$", body[1], re.M)]


class Transfers(unittest.TestCase):
    def test_mips_registers_and_delays(self):
        with tempfile.TemporaryDirectory(prefix="epok-gte-codegen-") as directory:
            for optimization in ("-O2", "-O3"):
                with self.subTest(optimization=optimization):
                    obj = Path(directory) / "transfer.o"
                    compilation = subprocess.run([str(BIN / f"mipsel-none-elf-g++{SUFFIX}"),
                                    "-std=c++20", optimization, "-march=r3000", "-mabi=32", "-ffreestanding",
                                    "-I", str(ROOT / "runtime"), "-I", str(ROOT / "third_party/nugget"),
                                    "-I", str(ROOT / "third_party/nugget/third_party/EASTL/include"),
                                    "-I", str(ROOT / "third_party/nugget/third_party/EABase/include/Common"),
                                    "-c", str(ROOT / "tests/runtime/gte_transfer_codegen.cpp"),
                                    "-o", str(obj)], capture_output=True, text=True)
                    self.assertEqual(compilation.returncode, 0, compilation.stdout + compilation.stderr)
                    disassembly = subprocess.check_output(
                        [str(BIN / f"mipsel-none-elf-objdump{SUFFIX}"), "-d", "-z", str(obj)], text=True)
                    triple = instructions(disassembly, "load_triple")
                    loads = [line for line in triple if line.startswith("lwc2 ")]
                    self.assertEqual(loads, [f"lwc2 ${i},{4*i}(a0)" for i in range(6)])
                    command = triple.index("c2 0x280030")
                    self.assertEqual(triple[command-3:command], ["lwc2 $5,20(a0)", "nop", "nop"])
                    stores = instructions(disassembly, "store_triple")
                    self.assertEqual([line for line in stores if line.startswith("swc2 ")],
                                     [f"swc2 ${r},{i*4}({pointer})"
                                      for pointer, registers in (("a0", (12, 13, 14)), ("a1", (17, 18, 19)))
                                      for i, r in enumerate(registers)])
                    flag = next(i for i, line in enumerate(stores) if line.startswith("cfc2 "))
                    self.assertEqual(stores[flag+1], "nop")
                    self.assertLess(flag, next(i for i, line in enumerate(stores) if line.startswith("swc2 ")))
                    single = instructions(disassembly, "project_single")
                    command = single.index("c2 0x180001")
                    self.assertEqual(single[command-4:command],
                                     ["lwc2 $0,0(a0)", "lwc2 $1,4(a0)", "nop", "nop"])
                    self.assertEqual([line for line in single if line.startswith("swc2 ")],
                                     ["swc2 $27,0(a2)", "swc2 $14,0(a1)"])
                    flag = next(i for i, line in enumerate(single) if line.startswith("cfc2 "))
                    self.assertEqual(single[flag+1], "nop")
                    self.assertFalse(any(line.startswith("mfc2 ") for line in stores + single))
                    for name, registers, opcode in (("project_geometry_full", [0, 1], "c2 0x180001"),
                                                     ("project_geometry_batch", list(range(6)), "c2 0x280030")):
                        geometry = instructions(disassembly, name)
                        self.assertEqual([int(line.rsplit("$", 1)[1]) for line in geometry if line.startswith("mtc2 ")], registers)
                        command = geometry.index(opcode)
                        self.assertEqual(geometry[command-2:command], ["nop", "nop"])
                        if name == "project_geometry_full":
                            self.assertEqual([int(line.split("$")[1].split(",")[0])
                                              for line in geometry if line.startswith("swc2 ")], [25, 27, 14])
                            reads = [line for line in geometry if line.startswith("mfc2 ")]
                            self.assertEqual(len(reads), 1)
                            self.assertTrue(reads[0].endswith("$26"), reads)
                            flag = next(i for i, line in enumerate(geometry) if line.startswith("cfc2 "))
                            self.assertEqual(geometry[flag+1], "nop")
                    compact = instructions(disassembly, "project_geometry_compact")
                    command = compact.index("c2 0x280030")
                    self.assertEqual(compact[command-2:command], ["nop", "nop"])
                    self.assertEqual([line for line in compact if line.startswith("swc2 ")],
                                     [f"swc2 ${r},{4*i}(a2)" for i, r in enumerate((12,17,13,18,14,19))])
                    direct = instructions(disassembly, "load_direct")
                    self.assertEqual([int(line.rsplit("$", 1)[1]) for line in direct if line.startswith("mtc2 ")],
                                     list(range(6)))
                    command = direct.index("c2 0x280030")
                    self.assertEqual(direct[command-2:command], ["nop", "nop"])
                    self.assertFalse(any(line.startswith(("sw ", "lwc2 ", "jal ")) for line in direct), direct)


if __name__ == "__main__":
    unittest.main()
