"""Native counter decoding/report tests; does not build or launch an emulator."""
import struct
import unittest

from profile_runtime import (STREAMING_FIELDS, WARMUP_FIELDS, counter_summary,
                             read_counters, symbol_address, normalize_streamed_chunks, frame_medians,
                             PLAYBACK_STATS, playback_addresses, playback_summary)


class NativeCounterTests(unittest.TestCase):
    def test_playback_layout_uses_live_section_not_discarded_duplicate(self):
        symbols = (".bss._ZN4epok14sequence_statsE\n 0x00000000 0x34 unused.o\n"
                   ".bss._ZN4epok14sequence_statsE\n 0x80000100 0x34 main.o\n"
                   "0x80000100 epok::sequence_stats\n")
        addresses = playback_addresses(symbols)
        self.assertEqual(addresses, {"sequence": 0x100, "effect": None, "particle": None})
        fields = PLAYBACK_STATS["sequence"][1]
        ram = bytearray(512)
        struct.pack_into("<13I", ram, 0x100, *range(13))
        self.assertEqual(read_counters(ram, addresses["sequence"], fields)["diagnostics_dropped"], 12)
        with self.assertRaises(ValueError):
            playback_addresses(symbols.replace("0x80000100 0x34", "0x80000100 0x30"))

    def test_playback_gauges_deltas_resets_and_saturation(self):
        _, fields, gauges = PLAYBACK_STATS["particle"]
        first = dict(zip(fields, [20, 40, 3, 25, 0]))
        last = dict(zip(fields, [0, 50, 0xffffffff, 30, 1]))
        rows = [{"frame": 50, "particle": last}, {"frame": 6, "particle": first}]
        report = playback_summary(rows, "particle", fields, gauges)
        self.assertFalse(report["counter_reset_observed"])
        self.assertEqual(report["during_capture"]["spawned"], 10)
        self.assertNotIn("alive", report["during_capture"])
        self.assertEqual(report["maximum_observed"]["alive"], 20)
        self.assertEqual(report["saturated"], ["dropped"])
        rows.append({"frame": 60, "particle": dict.fromkeys(fields, 0)})
        reset = playback_summary(rows, "particle", fields, gauges)
        self.assertTrue(reset["counter_reset_observed"])
        self.assertTrue(all(value is None for value in reset["during_capture"].values()))
        self.assertEqual(playback_summary([], "particle", fields, gauges), {"available": False})

    def test_optional_linked_symbols_and_fixed_layout(self):
        symbols = "0x80000100 epok::streaming_stats\n0x80000140 epok::streaming_warmup_stats\n"
        ram = bytearray(512)
        struct.pack_into("<7I", ram, 0x100, 4, 262144, 4, 1900000, 0, 0, 2)
        struct.pack_into("<6I", ram, 0x140, 1, 2, 2, 1100000, 0, 0)
        stream = read_counters(ram, symbol_address(symbols, "epok::streaming_stats"), STREAMING_FIELDS)
        warmup = read_counters(ram, symbol_address(symbols, "epok::streaming_warmup_stats"), WARMUP_FIELDS)
        self.assertEqual(stream["xa_interruptions"], 2)
        self.assertEqual(warmup["stall_us"], 1100000)
        self.assertIsNone(symbol_address(symbols, "epok::missing_stats"))
        self.assertIsNone(read_counters(ram, None, STREAMING_FIELDS))
        with self.assertRaises(ValueError):
            read_counters(ram[:270], 0x100, STREAMING_FIELDS)

    def test_startup_totals_remain_visible_when_capture_has_no_reads(self):
        counters = dict(zip(WARMUP_FIELDS, [1, 2, 2, 1100000, 0, 0]))
        rows = [{"frame": 7, "streaming_warmup": counters},
                {"frame": 80, "streaming_warmup": counters}]
        report = counter_summary(rows, "streaming_warmup", WARMUP_FIELDS)
        self.assertEqual(report["last"]["stall_us"], 1100000)
        self.assertEqual(report["during_capture"]["stall_us"], 0)
        self.assertEqual(report["first"]["reads"], 2)

    def test_missing_is_unavailable_and_counter_wrap_is_unsigned(self):
        self.assertEqual(counter_summary([{"frame": 7}], "streaming", STREAMING_FIELDS),
                         {"available": False})
        rows = [{"frame": 9, "streaming": {"bytes": 16}},
                {"frame": 7, "streaming": {"bytes": 0xfffffff0}}]
        self.assertEqual(counter_summary(rows, "streaming", ["bytes"])["during_capture"]["bytes"], 32)

    def test_descriptive_counter_sentinel_is_not_a_count_or_median(self):
        rows = [{"frame": 1, "frame_scanlines": 400, "streamed_chunks": 0xffffffff,
                 "stream_failed_chunks": 2},
                {"frame": 2, "frame_scanlines": 402, "streamed_chunks": 0xffffffff,
                 "stream_failed_chunks": 4}]
        for row in rows: normalize_streamed_chunks(row)
        self.assertTrue(all(row["streamed_chunks"] is None and not row["streamed_chunks_available"] for row in rows))
        medians = frame_medians(rows)
        self.assertIsNone(medians["streamed_chunks"])
        self.assertEqual(medians["stream_failed_chunks"], 3)
        self.assertEqual(medians["frame_scanlines"], 401)
        self.assertNotIn("streamed_chunks_available", medians)
        missing = {"frame": 3}
        normalize_streamed_chunks(missing)
        self.assertIsNone(missing["streamed_chunks"])
        self.assertFalse(missing["streamed_chunks_available"])

    def test_collected_zero_is_available_and_mixed_availability_is_not_averaged(self):
        rows = [{"frame": 1, "streamed_chunks": 0}, {"frame": 2, "streamed_chunks": 12}]
        for row in rows: normalize_streamed_chunks(row)
        self.assertTrue(all(row["streamed_chunks_available"] for row in rows))
        self.assertEqual(frame_medians(rows)["streamed_chunks"], 6)
        rows[1]["streamed_chunks"] = 0xffffffff
        normalize_streamed_chunks(rows[1])
        self.assertIsNone(frame_medians(rows)["streamed_chunks"])


if __name__ == "__main__":
    unittest.main()
