"""Forest route capture helpers; no project mutation or emulator launch."""
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

from profile_forest_route import FIELDS, EXTRA_FIELDS, decode_frame, profile, save_png


class ForestProfileTests(unittest.TestCase):
    def test_native_frame_and_loading_counters_are_separate(self):
        ram = bytearray(512)
        values = [0] * len(FIELDS)
        values[FIELDS.index("frame")] = 17
        values[FIELDS.index("frame_scanlines")] = 444
        struct.pack_into(f"<{len(values)}I", ram, 0, *values)
        struct.pack_into("<3I", ram, 256 + 16, 100, 2, 28800)
        struct.pack_into("<6I", ram, 320, 0, 0, 0, 0, 0, 0)
        struct.pack_into("<7I", ram, 400, 2, 131072, 2, 1000000, 0, 0, 0)
        struct.pack_into("<6I", ram, 440, 1, 2, 2, 1000000, 0, 0)
        row = decode_frame(ram, dict(performance=0, time=256, lighting=320,
                                    streaming=400, streaming_warmup=440), 0)
        self.assertEqual(row["present_wait_estimate_us"], 384)
        self.assertEqual(row["dropped_steps"], 2)
        report = profile([row], {})
        self.assertEqual(report["median"]["frame_microseconds"], 28800)
        self.assertNotIn("streaming_warmup", report["median"])
        self.assertEqual(report["streaming_warmup"]["last"]["stall_us"], 1000000)
        self.assertEqual(report["streaming"]["during_capture"]["reads"], 0)

    def test_vram_png_crop_has_correct_channels_and_dimensions(self):
        ram = bytearray(1024 * 512 * 2)
        struct.pack_into("<2H", ram, 0, 31, 31 << 5)
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "capture.png"
            save_png(ram, path, 2, 1)
            data = path.read_bytes()
        self.assertEqual(data[:8], b"\x89PNG\r\n\x1a\n")
        self.assertEqual(struct.unpack_from(">2I", data, 16), (2, 1))
        position, payload = 8, bytearray()
        while position < len(data):
            size = struct.unpack_from(">I", data, position)[0]
            if data[position + 4:position + 8] == b"IDAT":
                payload.extend(data[position + 8:position + 8 + size])
            position += size + 12
        self.assertEqual(zlib.decompress(payload), bytes([0, 255, 0, 0, 0, 255, 0]))

    def test_uncollected_streamed_chunks_are_null_in_rows_and_summary(self):
        ram = bytearray(512)
        values = [0] * (len(FIELDS) + len(EXTRA_FIELDS))
        values[0] = 21
        values[len(FIELDS) + EXTRA_FIELDS.index("streamed_chunks")] = 0xffffffff
        values[len(FIELDS) + EXTRA_FIELDS.index("stream_failed_chunks")] = 3
        struct.pack_into(f"<{len(values)}I", ram, 0, *values)
        addresses = dict(performance=0,time=256,lighting=320,streaming=None,streaming_warmup=None)
        row = decode_frame(ram, addresses, len(EXTRA_FIELDS))
        report = profile([row], {})
        self.assertIsNone(row["streamed_chunks"])
        self.assertFalse(row["streamed_chunks_available"])
        self.assertFalse(report["streamed_chunks_available"])
        self.assertIsNone(report["median"]["streamed_chunks"])
        self.assertEqual(report["median"]["stream_failed_chunks"],3)


if __name__ == "__main__":
    unittest.main()
