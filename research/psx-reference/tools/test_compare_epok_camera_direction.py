import unittest
from compare_epok_camera_direction import compare


class DirectionComparisonTests(unittest.TestCase):
    def test_signed_wrap_and_separate_stages(self):
        rows=[dict(look=[4095,0,0],angles=[1,0,0]),dict(look=[0,0,0],angles=[0,0,0])]
        result=compare(rows,[[0,0,4095,0,1,0,0],[1,0,0,0,4095,2,0]])
        self.assertEqual(result['max_abs_direction_turn12'],[1,1,0])
        self.assertEqual(result['max_abs_blended_turn12'],[1,2,0])
        self.assertEqual(result['exact_direction_updates'],1)
        self.assertEqual(result['exact_blended_updates'],1)

    def test_rejects_missing_and_reordered_probe_rows(self):
        rows=[dict(look=[0]*3,angles=[0]*3)]
        for output in ([],[[1,0,0,0,0,0,0]],[[0,0,0,0,0,0]]):
            with self.assertRaises(ValueError):compare(rows,output)


if __name__=='__main__':unittest.main()
