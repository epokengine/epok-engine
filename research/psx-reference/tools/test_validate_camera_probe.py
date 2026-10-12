import unittest
from validate_camera_probe import validate


class ProbeTests(unittest.TestCase):
    def fixture(self):
        probe={'rows':[[0,-1]+[0]*27]}
        draw={'frame':10,'sample':0,'frame_microseconds':30000}
        draw.update({f'camera_origin_{c}':0 for c in range(3)})
        draw.update({f'camera_basis_{r}_{c}':0 for r in range(3) for c in range(4)})
        draw.update({'row_scale_0':0,'row_scale_1':0,'camera_origin_0':0xffffffff})
        profile={'rows':[{'frame':10,'frame_microseconds':30000,'ordered_draw':draw}]}
        return probe,profile

    def test_signed_matching(self):
        probe,profile=self.fixture()
        self.assertEqual(validate(probe,profile)['exact_live_records'],1)

    def test_nearby_is_not_equivalent(self):
        probe,profile=self.fixture();profile['rows'][0]['ordered_draw']['camera_origin_0']=0xfffffffe
        with self.assertRaises(ValueError):validate(probe,profile)

    def test_torn_publication_rejected(self):
        probe,profile=self.fixture();profile['rows'][0]['ordered_draw']['frame']=9
        with self.assertRaises(ValueError):validate(probe,profile)

    def test_empty_completed_pass_is_counted_not_matched(self):
        probe,profile=self.fixture()
        profile['rows'].append({'frame':11,'frame_microseconds':20000,
                               'ordered_draw':{'frame':11,'sample':0xffffffff,'frame_microseconds':0}})
        result=validate(probe,profile)
        self.assertEqual(result['completed_frames_without_ordered_pass'],1)
        self.assertEqual(result['exact_live_records'],1)


if __name__=='__main__':unittest.main()
