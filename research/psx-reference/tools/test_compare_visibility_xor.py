import copy
import unittest
from compare_visibility_xor import compare_rows


class CompareTests(unittest.TestCase):
    def test_identical_pose_requires_identical_output(self):
        a=[dict(performance=dict(frame=1),visibility=dict(sample=0),
                ordered=dict(sample=0,candidates=4,camera_origin_0=123,packets=4,frame=1,frame_microseconds=10))]
        b=copy.deepcopy(a);b[0]['ordered']['frame_microseconds']=12
        self.assertEqual(compare_rows(a,b)['identical_projection_and_ordered_counters'],1)
        b[0]['ordered']['packets']=3
        with self.assertRaisesRegex(ValueError,'Draw outputs'):compare_rows(a,b)
        b[0]['ordered']['camera_origin_0']=124
        self.assertEqual(compare_rows(a,b)['differing_camera_pose'],1)

    def test_rejects_visibility_or_frame_changes(self):
        a=[dict(performance=dict(frame=1),visibility=dict(sample=0),ordered=dict(candidates=4))]
        for block,key,value in [('visibility','sample',1),('ordered','candidates',3),('performance','frame',2)]:
            b=copy.deepcopy(a);b[0][block][key]=value
            with self.assertRaises(ValueError):compare_rows(a,b)


if __name__=='__main__':unittest.main()
