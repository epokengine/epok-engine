import unittest
import tempfile
import json
from pathlib import Path
from compare_epok_profiles import compare, groups, CAMERA_FIELDS


class WorkGroupingTests(unittest.TestCase):
    def test_comparison_rejects_different_detail_or_unknown_mode(self):
        for detail in (False, None):
            with self.subTest(detail=detail), tempfile.TemporaryDirectory() as folder:
                before,after=Path(folder)/"before",Path(folder)/"after"
                for path,value in ((before,True),(after,detail)):
                    path.mkdir()
                    (path/"profile.json").write_text(json.dumps({"running_code_verified_bytes":4,
                        "rows":[{"ordered_validation":None,"streamed_chunks_available":value}]}))
                with self.assertRaisesRegex(ValueError,"profiling detail modes"):
                    compare(before,after)

    def test_comparison_rejects_different_gte_instrumentation(self):
        with tempfile.TemporaryDirectory() as folder:
            before, after = Path(folder)/"before", Path(folder)/"after"
            for path, validation in ((before, None), (after, {})):
                path.mkdir()
                (path/"profile.json").write_text(json.dumps({"running_code_verified_bytes": 4,
                    "rows": [{"ordered_validation": validation}]}))
            with self.assertRaisesRegex(ValueError, "heavy GTE validation"):
                compare(before, after)

    def test_camera_comparison_rejects_a_different_display_before_work_matching(self):
        with tempfile.TemporaryDirectory() as folder:
            before, after = Path(folder)/"before", Path(folder)/"after"
            before.mkdir(); after.mkdir()
            (before/"display.hh").write_text("512x240")
            (after/"display.hh").write_text("320x240")
            with self.assertRaisesRegex(ValueError, "display configuration"):
                compare(before, after, "camera")

    def test_exact_camera_mode_uses_latched_view_and_rejects_torn_or_missing_frames(self):
        ordered = dict.fromkeys(CAMERA_FIELDS, 0)
        ordered.update(frame=8, sample=7, candidates=20)
        row = {"frame": 8, "visibility_track": {"sample": 9}, "steps": 2,
               "ordered_draw": ordered, "gte_vertices": 60, "software_vertices": 0,
               "triangles": 20, "tested_chunks": 0}
        same = dict(row, visibility_track={"sample": 10})
        self.assertEqual(list(groups([row], "camera")), list(groups([same], "camera")))
        for field in CAMERA_FIELDS + ["sample"]:
            changed = dict(row, ordered_draw=dict(ordered, **{field: ordered[field]+1}))
            self.assertNotEqual(list(groups([row], "camera")), list(groups([changed], "camera")))
        for state in (dict(ordered, frame=0), dict(ordered, frame=9), dict(ordered, sample=0xffffffff)):
            self.assertFalse(groups([dict(row, ordered_draw=state)], "camera"))
        with self.assertRaises(ValueError):
            groups([dict(row, ordered_draw={"frame": 8})], "camera")

    def test_cpu_to_gte_migration_matches_without_hiding_work_changes(self):
        row = {"visibility_track": {"sample": 635}, "steps": 2,
               "ordered_draw": {"candidates": 914}, "gte_vertices": 2700,
               "software_vertices": 540, "triangles": 1373, "tested_chunks": 0}
        moved = dict(row, gte_vertices=3240, software_vertices=0)
        self.assertNotEqual(list(groups([row])), list(groups([moved])))
        self.assertEqual(list(groups([row], "ordered")), list(groups([moved], "ordered")))
        for changed in (dict(moved, software_vertices=1), dict(moved, triangles=1372),
                        dict(moved, ordered_draw={"candidates": 913}), dict(moved, steps=3)):
            self.assertNotEqual(list(groups([row], "ordered")), list(groups([changed], "ordered")))


if __name__ == "__main__":
    unittest.main()
