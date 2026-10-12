"""Cadence arithmetic, fail-closed code hooks and publication checks; no emulator."""
import struct
import unittest

from analyze_epok_cadence import analyze
from capture_epok_cadence import block_leaders, flip_hook, validate_executable


def fixture():
    layouts = {
        "performance": "frame streamed_chunks stream_failed_chunks stream_budget_skipped_chunks gte_validation_errors",
        "ordered": "frame anchor_rejections",
        "visibility": "advances waiting_frames loops",
        "streaming": "reads bytes stalls stall_us errors timeouts gameplay_requests gameplay_rejected",
        "verify": "batches bytes scanlines",
        "time": "ticks dropped_steps",
        "lighting": "dropped_triangles",
        "texture": "failures error",
    }
    config = dict(blocks=[], row_words=3)
    for name, fields in layouts.items():
        fields = fields.split()
        config["blocks"].append(dict(name=name, offset=config["row_words"], fields=fields))
        config["row_words"] += len(fields)
    rows, presentations = [], []
    for i, cycle in enumerate((100000, 1200000, 36000000)):
        row = [cycle, i * 2, 90000]
        for block in config["blocks"]:
            values = {f: 0 for f in block["fields"]}
            if block["name"] in ("performance", "ordered"):
                values["frame"] = i + 1
            if block["name"] == "performance":
                values["streamed_chunks"] = 0xffffffff
            if block["name"] == "visibility":
                values["waiting_frames"] = int(i == 2)
            row.extend(values[f] for f in block["fields"])
        rows.append(row)
        presentations.append([cycle + 170, i * 2, 0x05000000 if i % 2 == 0 else 0x05040000, i + 1, 170, 1000])
    return dict(version=1, cpu_clock_hz=33868800, overflow=False, callback_error=None,
                rows=rows, presentations=presentations), config


class CadenceTests(unittest.TestCase):
    def test_block_leaders_cover_branches_calls_and_fallthrough(self):
        entry = 0x80010000
        words = [
            0x10400003,  # 00 beq v0,zero,+3 -> 0x10
            0x00000000,  # 04 delay slot
            0x0c004010,  # 08 jal 0x80010040 (outside)
            0x00000000,  # 0c delay slot
            0x24420001,  # 10 addiu (branch target, also after jal)
            0x03e00008,  # 14 jr ra
            0x00000000,  # 18 delay slot
            0x24420002,  # 1c unreachable tail
        ]
        code = struct.pack("<8I", *words)
        leaders, indirect = block_leaders(code, entry)
        self.assertEqual(leaders, [entry, entry + 0x08, entry + 0x10, entry + 0x1c])
        self.assertEqual(indirect, 1)

    def test_aggregate_rate_keeps_long_wait(self):
        profile, config = fixture()
        result = analyze(profile, config)
        self.assertEqual(result["completed_ordered_display_origin_vsyncs"], {2: 2})
        self.assertAlmostEqual(result["completed_ordered_display_origin_rate"], 2 * 33868800 / 35900000)
        self.assertGreater(result["camera_wait_interval_ms"]["maximum"], 1000)
        self.assertEqual(result["completed_ordered_intervals_over_30hz"], 1)
        self.assertEqual(result["profile_detail_modes"], {"False": 3})
        self.assertEqual(result["intervals_over_30hz_costs"]["flip_wait_over_1ms"], 0)

    def test_reject_incomplete_and_callback_failure(self):
        for key, value in (("overflow", True), ("callback_error", "bad hook")):
            profile, config = fixture(); profile[key] = value
            with self.assertRaisesRegex(ValueError, "Incomplete"):
                analyze(profile, config)

    def test_reject_frame_gap_duplicate_and_wrong_pair(self):
        for field, value in (("frame", 5), ("command", 0x05000000), ("wait", 99)):
            profile, config = fixture()
            if field == "frame": profile["rows"][1][3] = value
            elif field == "command": profile["presentations"][1][2] = value
            else: profile["presentations"][1][4] = value
            with self.assertRaises(ValueError): analyze(profile, config)

    def test_exclude_unlatched_not_empty_complete(self):
        profile, config = fixture()
        ordered = next(b for b in config["blocks"] if b["name"] == "ordered")
        profile["rows"][0][ordered["offset"]] = 0
        result = analyze(profile, config)
        self.assertEqual(result["non_latched_frames"], 1)
        self.assertEqual(result["paired_presentation_intervals"], 1)

    def test_reject_counter_reset(self):
        profile, config = fixture()
        counter = next(b for b in config["blocks"] if b["name"] == "streaming")
        profile["rows"][0][counter["offset"]] = 4
        with self.assertRaisesRegex(ValueError, "counter reset"): analyze(profile, config)

    def test_verification_uses_deltas_and_separates_batch_sizes(self):
        profile, config = fixture()
        block = next(b for b in config["blocks"] if b["name"] == "verify")
        offset = block["offset"]
        # Startup already consumed ten slices. Later intervals have one then
        # two slices; the baseline must not leak into either measurement.
        for row, values in zip(profile["rows"], ((10,40960,640), (11,45056,704), (13,53248,830))):
            row[offset:offset+3] = values
        groups = analyze(profile, config)["verification_between_frames"]
        self.assertEqual([(g["batches_per_interval"], g["bytes_per_interval"]) for g in groups],
                         [(1,4096), (2,8192)])
        self.assertEqual([g["scanline_ticks"]["median"] for g in groups], [64,126])

    def test_code_pattern_unique_aligned_and_exact(self):
        pattern = struct.pack("<5I", 0x3c021f80, 0x12320002, 0x3c030504, 0x3c030500, 0xac431814)
        self.assertEqual(flip_hook(b"\0" * 8 + pattern, 0x80010000), 0x80010018)
        for bad in (pattern + pattern, b"\0" + pattern, pattern[:-1] + b"\x00"):
            with self.assertRaisesRegex(ValueError, "Unsupported/ambiguous"): flip_hook(bad, 0x80010000)

    def test_optional_overlap_keeps_old_captures_unknown(self):
        profile, config = fixture()
        self.assertNotIn("overlap", analyze(profile, config)["counters"])
        offset = config["row_words"]
        config["blocks"].append(dict(name="overlap", offset=offset, fields=["bytes"]))
        config["row_words"] += 1
        verify = next(b["offset"] for b in config["blocks"] if b["name"] == "verify")
        for row, value in zip(profile["rows"], (2048, 4096, 8192)):
            row.append(value); row[verify+1] = 16384
        result = analyze(profile, config)["counters"]["overlap"]
        self.assertEqual(result["delta"]["bytes"], 6144)
        profile["rows"][-1][offset] = 0
        with self.assertRaisesRegex(ValueError, "counter reset"): analyze(profile, config)
        profile["rows"][-1][offset] = 16385
        with self.assertRaisesRegex(ValueError, "Overlap exceeds"): analyze(profile, config)

    def test_executable_code_must_match_elf(self):
        exe = bytearray(2052); exe[:8] = b"PS-X EXE"
        struct.pack_into("<II", exe, 0x18, 0x80010000, 4)
        exe[2048:] = b"code"
        validate_executable(exe, [(0x10000, b"code")])
        for sections in ([(0x10000, b"bad!")], [(0x10004, b"code")]):
            with self.assertRaisesRegex(ValueError, "do not match"): validate_executable(exe, sections)


if __name__ == "__main__":
    unittest.main()
