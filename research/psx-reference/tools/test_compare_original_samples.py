import unittest
from compare_original_samples import compare


class MatchingTests(unittest.TestCase):
    def fixture(self):
        row=[100,100,20,1,-1,0,0,2]+[0]*40
        row[40]=2;row[47]=40
        snapshot=[90,0x80090000,2,0,256,512,768,0,0,0]+[0]*32+[10,20]
        profile=dict(version=3,cpu_clock_hz=33868800,overflow=False,callback_error=None,
                     rows=[row],world_samples=[snapshot])
        routes=dict(coordinate_scale=.00025,routes=[dict(Zone="z",Camera=0,Samples=[
            dict(Position=[.00025,.0005,.00075],Polygons=[10,20])])])
        return profile,routes

    def test_exact_order_and_position(self):
        profile,routes=self.fixture()
        result=compare(profile,routes)
        self.assertEqual(result["exact_ordered_list_snapshots"],1)
        self.assertEqual(result["rows"][0]["exact_ordered_list_matches"][0]["camera_distance_native"],0)

    def test_same_camera_and_membership_do_not_prove_order(self):
        profile,routes=self.fixture()
        profile["world_samples"][0][-2:]=[20,10]
        result=compare(profile,routes)
        self.assertEqual(result["exact_ordered_list_snapshots"],0)
        self.assertEqual(result["rows"][0]["nearest_source_camera"]["only_export"],0)

    def test_truncated_snapshot_rejected(self):
        profile,routes=self.fixture();profile["world_samples"][0].pop()
        with self.assertRaises(ValueError):compare(profile,routes)


if __name__=="__main__":unittest.main()
