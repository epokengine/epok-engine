import struct
import unittest

from analyze_original_profile import CODE_RANGES, analyze, verify_code


def frame(cycles, delta, calls, vsync):
    row = [cycles, delta, 200 if calls else 0, calls, -1, 0, 0, vsync] + [0]*40
    row[40] = 10 if calls else 0
    row[47] = 400 if calls else 0
    return row


class ProfileTests(unittest.TestCase):
    def profile(self):
        return dict(version=2, cpu_clock_hz=33868800, overflow=False, callback_error=None,
                    rows=[frame(100,100,0,1),frame(10100,10000,1,20),
                          frame(11100,1000,1,22),frame(12100,1000,1,24)],
                    presentations=[[10150,20,0],[10600,21,0],[11150,22,240<<16]])

    def test_loading_gap_separate_and_duplicate_display_not_counted(self):
        result=analyze(self.profile())
        self.assertEqual(result["world_rows"],3)
        self.assertEqual(result["world_runs"],[[1,3]])
        self.assertEqual(result["continuous_world_finalization_interval_ms"]["count"],2)
        self.assertEqual(result["world_entry_transition_interval_ms"]["count"],1)
        self.assertEqual(result["continuous_world_presentation_vsyncs"],{2:1})

    def test_failures_overflow_and_clock_rewind_rejected(self):
        for field, value in (("overflow",True),("callback_error","bad pointer"),("cpu_clock_hz",1)):
            profile=self.profile();profile[field]=value
            with self.assertRaises(ValueError):analyze(profile)
        profile=self.profile();profile["rows"][-1][0]=1
        with self.assertRaises(ValueError):analyze(profile)

    def test_slow_display_interval_is_not_trimmed(self):
        profile=self.profile()
        profile["presentations"][-1][1]=29
        self.assertEqual(analyze(profile)["continuous_world_presentation_vsyncs"],{9:1})

    def test_corrected_count_does_not_reinterpret_old_capture(self):
        profile=self.profile()
        self.assertIsNone(analyze(profile)["slst_header_count"])
        profile["version"]=3
        result=analyze(profile)
        self.assertIsNone(result["legacy_slst_count_minus_one"])
        self.assertEqual(result["slst_header_count"]["median"],10)

    def test_read_only_code_verification(self):
        exe=bytearray(0x50800);exe[:8]=b"PS-X EXE"
        struct.pack_into("<II",exe,0x18,0x80010000,0x50000)
        ram=bytearray(2097152)
        self.assertEqual(len(verify_code(exe,ram)),len(CODE_RANGES))
        ram[CODE_RANGES[0][0]-0x80000000]=1
        with self.assertRaisesRegex(ValueError,"differs"):verify_code(exe,ram)

    def test_optional_phase_keeps_old_captures_unknown_and_ignores_duplicate_origin(self):
        profile = self.profile()
        report = analyze(profile)
        self.assertFalse(report["presentation_phase_available"])
        self.assertIsNone(report["continuous_world_presentation_phase_since_vsync_ms"])
        profile["presentation_phases"] = [[10150, 150], [10600, 600], [11150, 200]]
        report = analyze(profile)
        self.assertTrue(report["presentation_phase_available"])
        phases = report["continuous_world_presentation_phase_since_vsync_ms"]
        self.assertEqual(phases["count"], 2)
        self.assertAlmostEqual(phases["maximum"], 200 * 1000 / 33868800)
        profile["presentation_phases"][0][1] = -1
        self.assertEqual(analyze(profile)["continuous_world_presentation_phase_since_vsync_ms"]["count"], 1)

    def test_phase_requires_exact_pair_and_valid_cycles(self):
        for phases in ([], [[10149, 150], [10600, 600], [11150, 200]],
                       [[10150, -2], [10600, 600], [11150, 200]],
                       [[10150, 10151], [10600, 600], [11150, 200]],
                       [[10150, 1.5], [10600, 600], [11150, 200]]):
            profile = self.profile(); profile["presentation_phases"] = phases
            with self.assertRaisesRegex(ValueError, "presentation phase"):
                analyze(profile)


if __name__ == "__main__":unittest.main()
