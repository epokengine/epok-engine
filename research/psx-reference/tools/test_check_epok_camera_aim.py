import unittest
from check_epok_camera_aim import inputs_for_row
from test_replay_original_camera import row, prop

class CameraAimAdapterTests(unittest.TestCase):
    def test_normal_parameters_and_missing_defaults(self):
        r = row()
        self.assertEqual(inputs_for_row(r, [1, 2, 3]),
                         [0, 0, 0, 1, 2, 3, 4097, 0, 0, 128, 128, 128, 0,
                          2048, 2048, 2048, 2048, 2048, 2048, 0, 0])
        r['properties'] = [prop(0x183, [166,128,128]), prop(0x119, [147,148,341,11])]
        r['previous'][0] = 0
        r['flags'][3] = 287
        self.assertEqual(inputs_for_row(r, [0]*3)[9:],
                         [166,128,128,287,341,11,147,148,2048,2048,1,0])
        for flags_index, value in ((1, 256), (2, 2)):
            r['flags'][flags_index] = value
            self.assertEqual(inputs_for_row(r, [0]*3)[19], 0)
            r['flags'][flags_index] = 0

    def test_unsupported_modes_are_not_silently_approximated(self):
        for index, value in ((0,256), (4,1), (6,1), (3,32768)):
            r = row(); r['flags'][index] = value
            with self.assertRaises(ValueError): inputs_for_row(r, [0]*3)
        for identifier, values in ((0x16e,[0,0]),(0x183,[257,128,128]),(0x119,[-1,1,1,1])):
            r = row(); r['properties'] = [prop(identifier, values)]
            with self.assertRaises(ValueError): inputs_for_row(r, [0]*3)

if __name__ == '__main__': unittest.main()
