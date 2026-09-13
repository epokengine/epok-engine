"""Validate the native map/load-image guard without starting an emulator."""
import importlib.util
from pathlib import Path
import struct
import unittest
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'integration'))

spec = importlib.util.spec_from_file_location("verify_streaming", Path(__file__).resolve().parents[1] / "integration/verify_streaming.py")
verify_streaming = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify_streaming)


class StreamPoolLayout(unittest.TestCase):
    def image(self):
        exe = bytearray(2048 + 0x20000)
        exe[:8] = b"PS-X EXE"
        struct.pack_into("<II", exe, 0x18, 0x80010000, 0x20000)
        return exe

    def test_bss_pool_outside_load_image_passes(self):
        symbols = " .bss._ZN5epok11stream_poolE\n                0x80040000    0x20038 main.o\n"
        result = verify_streaming.assert_stream_pool_in_bss(symbols, self.image(), 2)
        self.assertEqual(result["page_bytes"], 131072)

    def test_serialized_or_overlapping_pool_fails(self):
        for section, address in (("data", "80018000"), ("bss", "80018000")):
            symbols = f" .{section}._ZN5epok11stream_poolE 0x{address} 0x20038 main.o\n"
            with self.assertRaises(AssertionError):
                verify_streaming.assert_stream_pool_in_bss(symbols, self.image(), 2)


if __name__ == "__main__":
    unittest.main()
