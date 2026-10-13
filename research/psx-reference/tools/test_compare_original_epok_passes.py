import unittest

from compare_original_epok_passes import original_passes


class OriginalPassMatchTests(unittest.TestCase):
    def test_lists_match_whole_token_order_and_map_to_concatenated_samples(self):
        routes = dict(coordinate_scale=.00025, routes=[
            dict(Samples=[dict(Polygons=[1, 2, 3]), dict(Polygons=[1, 2, 4])]),
            dict(Samples=[dict(Polygons=[7, 8]), dict(Polygons=[3, 2, 1])]),
        ])
        profile = dict(world_lists=[[1, 2, 4], [3, 2, 1], [2, 1, 3]], world_passes=[
            [0, 100, 3, 1, 10], [5, 110, 3, 1, 12],  # route 0 sample 1, twice
            [9, 90, 3, 2, 9],                         # route 1 sample 1: reversed order is distinct
            [12, 80, 3, 3, 8],                        # same members, unknown order: unmatched
            [15, 5, 0, 0, 0],                         # empty pass: ignored
        ])
        matched, unmatched, total = original_passes(profile, routes)
        self.assertEqual(total, 5)
        self.assertEqual(unmatched, 1)
        self.assertEqual(sorted(matched), [1, 3])
        self.assertEqual(matched[1]["cycles"], [100, 110])
        self.assertEqual((matched[3]["route"], matched[3]["sample"]), (1, 1))

    def test_stored_list_must_agree_with_pass_count(self):
        routes = dict(coordinate_scale=.00025, routes=[dict(Samples=[dict(Polygons=[1, 2])])])
        with self.assertRaises(ValueError):
            original_passes(dict(world_lists=[[1, 2]], world_passes=[[0, 1, 3, 1, 0]]), routes)


if __name__ == "__main__":
    unittest.main()
