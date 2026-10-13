"""Native counter decoding/report tests; does not build or launch an emulator."""
import struct
import unittest

from profile_runtime import (STREAMING_FIELDS, WARMUP_FIELDS, counter_summary,
                             read_counters, symbol_address, normalize_streamed_chunks, frame_medians, validate_gte_rows,
                             latch_ordered_frame, ORDERED_DRAW_FIELDS, ORDERED_CAMERA_FIELDS,
                             PLAYBACK_STATS, playback_addresses, playback_summary, rail_state_layout, read_rail_state)


class NativeCounterTests(unittest.TestCase):
    def test_ordered_frame_requires_last_published_identity_and_latches_clock(self):
        self.assertEqual(len(ORDERED_DRAW_FIELDS)*4, 116)
        self.assertEqual(len(ORDERED_CAMERA_FIELDS), 17)
        row = {"frame": 10, "frame_microseconds": 999,
               "ordered_draw": {"frame": 10, "sample": 7, "frame_microseconds": 123}}
        self.assertTrue(latch_ordered_frame(row))
        self.assertEqual(row["frame_microseconds"], 123)
        for frame in (0, 9, 11):
            self.assertFalse(latch_ordered_frame(dict(row, ordered_draw=dict(row["ordered_draw"], frame=frame))))
        inactive = dict(row, ordered_draw=dict(row["ordered_draw"], sample=0xffffffff), frame_microseconds=999)
        self.assertTrue(latch_ordered_frame(inactive))
        self.assertEqual(inactive["frame_microseconds"], 999)
        self.assertTrue(latch_ordered_frame({"frame": 1}))
        self.assertTrue(latch_ordered_frame({"frame": 1, "ordered_draw": {"frame": 1}}))

    def test_ordered_validation_preserves_cumulative_failures_and_peak_gauges(self):
        _, fields, gauges = PLAYBACK_STATS["ordered_validation"]
        first = dict.fromkeys(fields, 0)
        first.update(vertices=100, triples=30, camera_errors=1,
                     max_camera_delta=3, first_error_kind=1, first_error_sample=7)
        last = dict(first, vertices=200, triples=60)
        report = playback_summary([
            {"frame": 10, "ordered_validation": first},
            {"frame": 30, "ordered_validation": last},
        ], "ordered_validation", fields, gauges)
        self.assertEqual(report["during_capture"]["vertices"], 100)
        self.assertEqual(report["during_capture"]["camera_errors"], 0)
        self.assertEqual(report["last"]["camera_errors"], 1)
        self.assertNotIn("max_camera_delta", report["during_capture"])
        self.assertNotIn("first_error_sample", report["during_capture"])
        self.assertEqual(report["maximum_observed"]["max_camera_delta"], 3)

    def test_gte_validation_allows_empty_startup_but_requires_projection(self):
        empty = {"gte_vertices": 0, "gte_validation_errors": 0}
        projected = {"gte_vertices": 24, "gte_validation_errors": 0}
        validate_gte_rows([empty, projected])
        for rows in ([], [empty], [projected, {"gte_vertices": 0, "gte_validation_errors": 1}]):
            with self.assertRaises(AssertionError):
                validate_gte_rows(rows)

    def test_gte_validation_rejects_a_cumulative_error_before_the_first_poll(self):
        ordered = dict.fromkeys(PLAYBACK_STATS["ordered_validation"][1], 0)
        row = {"gte_vertices": 24, "gte_validation_errors": 0, "ordered_validation": ordered}
        validate_gte_rows([row])
        for field in ("camera_errors", "batch_errors", "screen_errors", "first_error_kind"):
            with self.assertRaises(AssertionError):
                validate_gte_rows([dict(row, ordered_validation=dict(ordered, **{field: 1}))])

    def test_rail_state_accepts_initialized_data_and_ignores_discarded_section(self):
        symbols=(".data._ZN4epok21rail_navigation_stateE\n 0x00000000 0x58 unused.o\n"
                 ".data._ZN4epok21rail_navigation_stateE\n 0x80000100 0x64 main.o\n"
                 "0x80000100 epok::rail_navigation_state\n")
        address,position=rail_state_layout(symbols)
        self.assertEqual((address,position),(256,True))
        ram=bytearray(512)
        struct.pack_into("<3i",ram,address+88,-4096,3277,409600)
        self.assertEqual(read_rail_state(ram,address,position)["target_position"],[-1,3277/4096,100])
        self.assertEqual(rail_state_layout(""),(None,False))
        with self.assertRaises(ValueError):rail_state_layout(symbols.replace("0x64 main.o","0x70 main.o"))

    def test_playback_layout_uses_live_section_not_discarded_duplicate(self):
        symbols = (".bss._ZN4epok14sequence_statsE\n 0x00000000 0x34 unused.o\n"
                   ".bss._ZN4epok14sequence_statsE\n 0x80000100 0x34 main.o\n"
                   "0x80000100 epok::sequence_stats\n")
        addresses = playback_addresses(symbols)
        self.assertEqual(addresses, {key: 0x100 if key == "sequence" else None for key in PLAYBACK_STATS})
        fields = PLAYBACK_STATS["sequence"][1]
        ram = bytearray(512)
        struct.pack_into("<13I", ram, 0x100, *range(13))
        self.assertEqual(read_counters(ram, addresses["sequence"], fields)["diagnostics_dropped"], 12)
        with self.assertRaises(ValueError):
            playback_addresses(symbols.replace("0x80000100 0x34", "0x80000100 0x30"))

    def test_playback_scalar_accepts_small_data_sections_and_checks_size(self):
        for section in ("bss", "sbss", "data", "sdata"):
            symbols = (f".{section}._ZN4epok27stream_verify_overlap_bytesE\n 0x00000000 0x4 unused.o\n"
                       f".{section}._ZN4epok27stream_verify_overlap_bytesE\n 0x80000100 0x4 main.o\n"
                       "0x80000100 epok::stream_verify_overlap_bytes\n")
            self.assertEqual(playback_addresses(symbols)["streaming_overlap"], 0x100)
            with self.assertRaises(ValueError):
                playback_addresses(symbols.replace("0x80000100 0x4", "0x80000100 0x2"))
        self.assertIsNone(playback_addresses("")["streaming_overlap"])

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

    def test_streaming_verification_separates_work_from_slice_gauges(self):
        _, fields, gauges = PLAYBACK_STATS["streaming_verify"]
        first = dict(zip(fields, [16, 65536, 1008, 63, 64]))
        last = dict(zip(fields, [32, 131072, 2017, 64, 64]))
        report = playback_summary([
            {"frame": 10, "streaming_verify": first},
            {"frame": 30, "streaming_verify": last},
        ], "streaming_verify", fields, gauges)
        self.assertEqual(report["during_capture"], {"batches": 16, "bytes": 65536, "scanlines": 1009})
        self.assertEqual(report["maximum_observed"]["max_scanlines"], 64)

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

    def test_ordered_subtimers_never_report_sentinel_or_partial_medians(self):
        for field in ("vertex_scanlines", "polygon_scanlines"):
            rows = [{"frame": 1, field: 0}, {"frame": 2, field: 12}]
            for row in rows: normalize_streamed_chunks(row)
            self.assertEqual(frame_medians(rows)[field], 6)
            rows[1][field] = 0xffffffff
            normalize_streamed_chunks(rows[1])
            self.assertIsNone(rows[1][field])
            self.assertFalse(rows[1][field + "_available"])
            self.assertIsNone(frame_medians(rows)[field])


if __name__ == "__main__":
    unittest.main()
