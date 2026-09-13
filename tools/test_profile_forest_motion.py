import unittest
from profile_forest_motion import q12, summarize


class MotionReportTests(unittest.TestCase):
    def rows(self):
        first=[0]*16;last=[0]*16
        first[1]=60;last[1]=180
        first[4]=4096;last[4]=(-7*4096)&0xffffffff
        return [dict(wall=10.,frame=100,probe=first),dict(wall=12.,frame=170,probe=last)]

    def test_signed_distance_and_two_independent_clocks(self):
        report=summarize(self.rows())
        self.assertEqual(report["distance_x"],-8.)
        self.assertEqual(report["units_per_simulation_second"],4.)
        self.assertEqual(report["ticks_per_wall_second"],60.)
        self.assertEqual(report["rendered_frames_per_wall_second"],35.)
        self.assertEqual(q12(0xfffff000),-1.)

    def test_slow_emulation_is_not_confused_with_native_velocity(self):
        rows=self.rows();rows[-1]["wall"]=14.
        report=summarize(rows)
        self.assertEqual(report["units_per_simulation_second"],4.)
        self.assertEqual(report["units_per_wall_second"],2.)
        self.assertEqual(report["ticks_per_wall_second"],30.)

    def test_missing_or_reset_counters_are_rejected(self):
        with self.assertRaises(ValueError):summarize([])
        rows=self.rows();rows[-1]["probe"][1]=0
        with self.assertRaises(ValueError):summarize(rows)


if __name__=="__main__":unittest.main()
