import unittest
from compare_epok_cadence import CAMERA_FIELDS, FIELDS, compare_rows, normalized_rows
from test_epok_cadence import fixture


def row(cpu=100):
    return dict({f:10 for f in FIELDS}, frame=1, cpu_cycles=cpu, steps=1,
                streamed_chunks=0xffffffff, triangles=4, tested_chunks=0,
                ordered_draw=dict({f:0 for f in CAMERA_FIELDS}, frame=1, sample=2, candidates=2))


class CompareTests(unittest.TestCase):
    def test_startup_zero_is_not_a_detail_build(self):
        profile, config = fixture()
        ordered = next(b for b in config["blocks"] if b["name"] == "ordered")
        profile["rows"][0][ordered["offset"]] = 0
        profile["rows"][0][4] = 0  # Uninitialized streamed_chunks during loading.
        rows = normalized_rows(profile, config)
        self.assertEqual(len(rows), 2)
        self.assertTrue(all(r["streamed_chunks"] == 0xffffffff for r in rows))

    def test_exact_view_and_work(self):
        result = compare_rows([row()], [row(90)])
        self.assertEqual(result["matching_groups"], 1)
        self.assertEqual(result["counters"]["cpu_cycles"]["median_group_percent"], -10)

    def test_changed_camera_not_matched(self):
        after = row(90); after["ordered_draw"][CAMERA_FIELDS[-1]] = 1
        with self.assertRaisesRegex(ValueError, "No exact"): compare_rows([row()], [after])

    def test_changed_work_not_matched(self):
        after = row(); after["triangles"] += 1
        with self.assertRaisesRegex(ValueError, "No exact"): compare_rows([row()], [after])

    def test_instrumentation_modes(self):
        after = row(); after["streamed_chunks"] = 0
        with self.assertRaisesRegex(ValueError, "detail"): compare_rows([row()], [after])

    def test_unavailable_subtimers_are_not_speedups_or_huge_costs(self):
        for unavailable in (0xffffffff, None):
            after = row(90); after["vertex_scanlines"] = unavailable
            result = compare_rows([row()], [after])
            counter = result["counters"]["vertex_scanlines"]
            self.assertFalse(counter["available"])
            self.assertIsNone(counter["median_group_percent"])
            self.assertEqual(result["counters"]["cpu_cycles"]["median_group_percent"], -10)

    def test_sentinel_only_applies_to_optional_counters(self):
        result = compare_rows([row(0xffffffff)], [row(0xffffffff)])
        self.assertTrue(result["counters"]["cpu_cycles"]["available"])
        self.assertEqual(result["counters"]["cpu_cycles"]["median_group_percent"], 0)

    def test_unlatched_and_empty_not_matched(self):
        for key, value in (("frame", 0), ("sample", 0xffffffff)):
            after = row(); after["ordered_draw"][key] = value
            with self.assertRaisesRegex(ValueError, "No exact"): compare_rows([row()], [after])


if __name__ == "__main__": unittest.main()
