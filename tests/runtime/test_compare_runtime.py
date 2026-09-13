"""Regression gate tests: p95 spikes, startup loss and unsafe/incomplete samples."""
import copy
import importlib.util
from pathlib import Path
import unittest
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))

spec = importlib.util.spec_from_file_location("compare_runtime", Path(__file__).resolve().parents[2] / "tools/compare_runtime.py")
compare_runtime = importlib.util.module_from_spec(spec)
spec.loader.exec_module(compare_runtime)


def capture():
    return {"frames": [dict(frame=i + 6, frame_microseconds=33333, frame_scanlines=400,
                            dropped_steps=0, dropped_triangles=0) for i in range(40)],
            "detail_timers": False, "gte_validation": False}


class NativeRegressionGate(unittest.TestCase):
    def test_identical_step_costs_with_different_mix_do_not_fail_raw_median(self):
        baseline, candidate = capture(), capture()
        for report, one_step_samples in ((baseline, 21), (candidate, 19)):
            for index, row in enumerate(report["frames"]):
                one_step = index < one_step_samples
                row.update(steps=1 if one_step else 2, frame_scanlines=300 if one_step else 400)
        result = compare_runtime.compare(baseline, candidate)
        self.assertTrue(result["passed"])
        self.assertTrue(result["cpu_by_steps"]["covers_all_samples"])
        median = result["metrics"]["frame_scanlines.median"]
        self.assertEqual(median["delta"], 100)
        self.assertIn("non_gating_reason", median)
        self.assertTrue(all(metric["delta"] == 0 for key, metric in result["metrics"].items()
                            if ".by_steps." in key))
        # A real one-step cost increase must fail despite the raw median's
        # informational status and unchanged global CPU p95/max.
        candidate["frames"][0]["frame_scanlines"] += 1
        self.assertIn("frame_scanlines.by_steps.1.max",
                      compare_runtime.compare(baseline, candidate)["regressions"])
        for row in candidate["frames"]:
            if row["steps"] == 1:
                row["frame_scanlines"] += 1
        self.assertIn("frame_scanlines.by_steps.1.median",
                      compare_runtime.compare(baseline, candidate)["regressions"])

    def test_incomplete_step_coverage_keeps_raw_median_gate(self):
        baseline, candidate = capture(), capture()
        for report, one_step_samples in ((baseline, 21), (candidate, 19)):
            for index, row in enumerate(report["frames"]):
                one_step = index < one_step_samples
                row.update(steps=1 if one_step else 2, frame_scanlines=300 if one_step else 400)
            # An observed group below the five-sample threshold prevents
            # replacing the aggregate median, even with all other groups equal.
            for row in report["frames"][-4:]:
                row["steps"] = 3
        result = compare_runtime.compare(baseline, candidate)
        self.assertFalse(result["cpu_by_steps"]["covers_all_samples"])
        self.assertIn("frame_scanlines.median", result["regressions"])
        self.assertNotIn("non_gating_reason", result["metrics"]["frame_scanlines.median"])

    def test_full_step_coverage_does_not_relax_tails_frame_times_or_safety(self):
        baseline = capture()
        for row in baseline["frames"]:
            row["steps"] = 1
        for field, expected in (("frame_scanlines", "frame_scanlines.p95"),
                                ("frame_microseconds", "frame_microseconds.p95"),
                                ("dropped_triangles", "dropped_triangles.max"),
                                ("dropped_steps", "dropped_steps.total")):
            candidate = copy.deepcopy(baseline)
            for row in candidate["frames"][-3:]:
                row[field] += 1
            result = compare_runtime.compare(baseline, candidate)
            self.assertTrue(result["cpu_by_steps"]["covers_all_samples"])
            self.assertIn(expected, result["regressions"])

    def test_steps_group_detects_regression_hidden_by_changed_mixture(self):
        baseline, candidate = capture(), capture()
        # Both global p95/max stay at 400, while the global median falls from
        # 400 to 310. The one-step workload nonetheless became ten units slower.
        for index, row in enumerate(baseline["frames"]):
            row.update(steps=1 if index < 10 else 2, frame_scanlines=300 if index < 10 else 400)
        for index, row in enumerate(candidate["frames"]):
            row.update(steps=1 if index < 30 else 2, frame_scanlines=310 if index < 30 else 400)
        result = compare_runtime.compare(baseline, candidate)
        self.assertFalse(result["passed"])
        self.assertEqual(result["metrics"]["frame_scanlines.median"]["delta"], -90)
        self.assertEqual(set(result["regressions"]), {
            "frame_scanlines.by_steps.1.median", "frame_scanlines.by_steps.1.p95", "frame_scanlines.by_steps.1.max"})
        self.assertEqual(result["cpu_by_steps"]["groups"]["1"]["samples"], {"baseline": 10, "candidate": 30})

    def test_missing_and_small_step_groups_do_not_fail(self):
        baseline, candidate = capture(), capture()
        for index, row in enumerate(baseline["frames"]):
            row["steps"] = 1 if index < 36 else 2
        for row in candidate["frames"]:
            row["steps"] = 1
        result = compare_runtime.compare(baseline, candidate)
        self.assertTrue(result["passed"])
        self.assertEqual(result["cpu_by_steps"]["groups"]["1"]["status"], "compared")
        self.assertEqual(result["cpu_by_steps"]["groups"]["2"], {
            "samples": {"baseline": 4, "candidate": 0}, "status": "insufficient_samples"})
        self.assertFalse(any(key.startswith("frame_scanlines.by_steps.2.") for key in result["metrics"]))

    def test_step_groups_require_five_samples_on_both_sides(self):
        baseline, candidate = capture(), capture()
        for report in (baseline, candidate):
            for index, row in enumerate(report["frames"]):
                row["steps"] = 1 if index < 35 else 2
        self.assertEqual(compare_runtime.compare(baseline, candidate)["cpu_by_steps"]["groups"]["2"]["status"], "compared")
        candidate["frames"][-1]["steps"] = 1
        self.assertEqual(compare_runtime.compare(baseline, candidate)["cpu_by_steps"]["groups"]["2"]["status"], "insufficient_samples")

    def test_legacy_captures_and_invalid_step_samples(self):
        baseline, candidate = capture(), capture()
        for row in candidate["frames"]:
            row["steps"] = 1
        self.assertTrue(compare_runtime.compare(baseline, candidate)["passed"])
        self.assertFalse(compare_runtime.compare(baseline, candidate)["cpu_by_steps"]["available"])
        for bad in (True, -1, 1.5):
            broken = copy.deepcopy(candidate)
            broken["frames"][0]["steps"] = bad
            with self.assertRaisesRegex(ValueError, "steps"):
                compare_runtime.compare(baseline, broken)
        del candidate["frames"][0]["steps"]
        with self.assertRaisesRegex(ValueError, "steps"):
            compare_runtime.compare(baseline, candidate)

    def test_identical_and_faster_workloads_pass(self):
        baseline = capture()
        self.assertTrue(compare_runtime.compare(baseline, baseline)["passed"])
        candidate = copy.deepcopy(baseline)
        for row in candidate["frames"]:
            row["frame_microseconds"] -= 1000
            row["frame_scanlines"] -= 10
        self.assertTrue(compare_runtime.compare(baseline, candidate)["passed"])

    def test_p95_spike_rejected_despite_same_median(self):
        baseline, candidate = capture(), capture()
        for row in candidate["frames"][-3:]:
            row["frame_microseconds"] *= 2
        report = compare_runtime.compare(baseline, candidate)
        self.assertEqual(report["regressions"], ["frame_microseconds.max", "frame_microseconds.p95"])

    def test_one_long_frame_rejected_even_below_p95(self):
        baseline, candidate = capture(), capture()
        candidate["frames"][-1]["frame_microseconds"] *= 10
        self.assertEqual(compare_runtime.compare(baseline, candidate)["regressions"], ["frame_microseconds.max"])

    def test_lower_cpu_does_not_hide_fps_regression(self):
        baseline, candidate = capture(), capture()
        for row in candidate["frames"]:
            row["frame_scanlines"] = 200
            row["frame_microseconds"] = 50000
        self.assertIn("frame_microseconds.median", compare_runtime.compare(baseline, candidate)["regressions"])

    def test_startup_drops_and_dropped_geometry_rejected(self):
        baseline, candidate = capture(), capture()
        for row in candidate["frames"]:
            row["dropped_steps"] = 6
        candidate["frames"][20]["dropped_triangles"] = 1
        report = compare_runtime.compare(baseline, candidate)
        self.assertIn("dropped_steps.total", report["regressions"])
        self.assertIn("dropped_triangles.max", report["regressions"])

    def test_insufficient_missing_duplicate_or_reset_samples_rejected(self):
        baseline = capture()
        broken = []
        candidate = capture(); candidate["frames"] = candidate["frames"][:4]; broken.append(candidate)
        candidate = capture(); del candidate["frames"][0]["dropped_triangles"]; broken.append(candidate)
        candidate = capture(); candidate["frames"][1]["frame"] = 6; broken.append(candidate)
        candidate = capture(); candidate["frames"][0]["dropped_steps"] = 2; broken.append(candidate)
        candidate = capture(); candidate["frames"][0]["frame_microseconds"] = float("nan"); broken.append(candidate)
        candidate = capture(); candidate["detail_timers"] = True; broken.append(candidate)
        for candidate in broken:
            with self.assertRaises(ValueError):
                compare_runtime.compare(baseline, candidate)

    def test_recorded_build_flags_must_match_and_not_disappear(self):
        baseline = capture()
        baseline.update(optimization_override="O2", timer_unit_us_approx=64)
        for key, changed in (("detail_timers", True), ("gte_validation", True),
                             ("optimization_override", "Os"), ("timer_unit_us_approx", 65)):
            candidate = copy.deepcopy(baseline)
            candidate[key] = changed
            with self.assertRaisesRegex(ValueError, key):
                compare_runtime.compare(baseline, candidate)
            candidate = copy.deepcopy(baseline)
            del candidate[key]
            with self.assertRaisesRegex(ValueError, key):
                compare_runtime.compare(baseline, candidate)

    def test_optional_visual_gate_is_explicit_and_rejects_changes(self):
        profile = capture()
        vram = bytes(1024 * 512 * 2)
        self.assertEqual(compare_runtime.compare(profile, profile)["visual_validation"]["status"], "not_checked")
        self.assertTrue(compare_runtime.compare(profile, profile, baseline_vram=vram, candidate_vram=vram)["passed"])
        changed = bytes([1]) + vram[1:]
        result = compare_runtime.compare(profile, profile, baseline_vram=vram, candidate_vram=changed)
        self.assertEqual(result["regressions"], ["vram"])
        self.assertEqual(result["visual_validation"]["status"], "different")
        with self.assertRaises(ValueError):
            compare_runtime.compare(profile, profile, baseline_vram=vram)
        with self.assertRaises(ValueError):
            compare_runtime.compare(profile, profile, baseline_vram=b"short", candidate_vram=b"short")


if __name__ == "__main__":
    unittest.main()
