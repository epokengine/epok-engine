import struct
import unittest

from sample_epok_runtime import executable_sections, verify_running_code


class RunningCodeTests(unittest.TestCase):
    def test_elf_ranges_and_live_identity(self):
        elf = bytearray(108)
        elf[:7] = b"\x7fELF\x01\x01\x01"
        struct.pack_into("<H", elf, 18, 8)
        struct.pack_into("<I", elf, 32, 64)
        struct.pack_into("<HH", elf, 46, 40, 1)
        struct.pack_into("<10I", elf, 64, 0, 1, 6, 0x80000010, 104, 4, 0, 0, 4, 0)
        elf[104:] = b"code"
        sections = executable_sections(elf)
        self.assertEqual(sections, [(16, b"code")])
        ram = bytearray(64)
        ram[16:20] = b"code"
        verify_running_code(ram, sections)
        ram[16] ^= 1
        with self.assertRaisesRegex(AssertionError, "does not match"):
            verify_running_code(ram, sections)
        with self.assertRaisesRegex(AssertionError, "outside PSX RAM"):
            verify_running_code(bytes(18), sections)
        with self.assertRaises(AssertionError):
            executable_sections(elf[:-1])


if __name__ == "__main__":
    unittest.main()
