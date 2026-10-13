import math
import unittest

from replay_original_camera import Direction, clamp_angle, delta, follow, lerp, midpoint, property_values, smooth


def row():
    return dict(base=[0, 0, 0], previous=[0x1001, 0, 0], flags=[0]*11,
                transition=[0, 0, 0], properties=[])


def prop(identifier, values):
    return [identifier, 0x80000000, 0, 0, 0x80000000, *[v & 65535 for v in values]]


class CameraReplayTests(unittest.TestCase):
    def test_wrapped_half_turn_and_negative_rounding(self):
        self.assertEqual(delta(0, 2048), -2048)
        self.assertEqual(delta(2048, 0), -2048)
        self.assertEqual(midpoint(0, 4095), 4095)
        self.assertEqual(lerp(0, 4095, 128), 4095)
        self.assertEqual(lerp(4095, 0, 256), 0)
        # These retain a positive half-turn at this branch-based clamp.
        self.assertEqual(clamp_angle(2048, 0, 100, 20), 100)
        self.assertEqual(clamp_angle(4095, 0, 100, 20), 4095)

    def test_smoothing_caps_delta_before_blending(self):
        self.assertEqual(smooth(0, 1000, 128, 45), 22)
        self.assertEqual(smooth(0, 4096-1000, 128, 45), 4096-23)
        self.assertEqual(smooth(0, 1, 128, 45), 0)
        self.assertEqual(smooth(1, 0, 128, 45), 0)

    def test_target_base_weights_and_limits(self):
        sample = row()
        sample['properties'] = [prop(0x183, [256, 0, 128]), prop(0x119, [147, 147, 341, 11])]
        self.assertEqual(follow(sample, [100, 300, 50]), [0, 147, 25])
        # In the first observed mode, interpolation is from target to base.
        sample['properties'] = [prop(0x183, [166, 128, 128])]
        sample['base'] = [3870, 120, 4095]
        sample['previous'] = [3875, 128, 4095]
        sample['flags'][2] = 1
        sample['flags'][3] = 287
        self.assertEqual(follow(sample, [3600, 138, 0]), [3875, 128, 4095])

    def test_overrides_and_pitch_effect(self):
        sample = row()
        sample['base'] = [20, 30, 40]
        sample['flags'][0] = 1 << 8
        self.assertEqual(follow(sample, [100, 200, 0]), [20, 30, 40])
        sample['flags'][0] = 2 << 8
        self.assertEqual(follow(sample, [100, 200, 0]), [100, 200, 0])
        sample['flags'][0] = 0
        sample['properties'] = [prop(0x183, [0, 0, 0]), prop(0x16e, [-10, 10])]
        sample['flags'][10] = 512
        self.assertEqual(follow(sample, [100, 200, 0]), [100, 200, 0])

    def test_cut_transition_and_skip_smoothing(self):
        sample = row()
        sample['base'] = [100, 200, 300]
        sample['properties'] = [prop(0x183, [256]*3)]
        sample['flags'][6:10] = [1, 100, 100, 150]
        self.assertEqual(follow(sample, [0]*3), [50, 100, 150])
        sample['flags'][9] = 200
        self.assertEqual(follow(sample, [0]*3), [100, 200, 300])
        sample['previous'] = [0, 0, 0]
        sample['flags'][1] = 1 << 8
        self.assertEqual(follow(sample, [0]*3), [100, 200, 300])

    def test_null_properties_and_bad_shapes(self):
        sample = row()
        sample['properties'] = [[0x183, 1, 0, 2, 0]]
        self.assertIsNone(property_values(sample, 0x183))
        self.assertEqual(follow(sample, [2, 4, 6]), [1, 2, 3])
        sample['properties'] = [prop(0x183, [1, 2])]
        with self.assertRaises(ValueError): property_values(sample, 0x183)
        sample['properties'] = [prop(0x183, [1, 2, 3])]*2
        with self.assertRaises(ValueError): property_values(sample, 0x183)

    def test_direction_axes_quadrants_and_behind_reference(self):
        # Mathematical fixtures test control flow only. The live replay reads
        # and validates the original tables, whose atan approximation differs.
        direction = Direction([round(math.sin(i*math.pi/2048)*4096) for i in range(1025)],
                              [round(math.atan(i/1024)*4096/(2*math.pi)) for i in range(1025)])
        self.assertEqual([direction.ratan2(y, x) for y, x in ((0, 0), (0, 1), (1, 0), (0, -1), (-1, 0))],
                         [0, 0, 1024, 2048, -1024])
        for angle in (0, 1024, 2048, 3072):
            self.assertEqual(direction.sin(angle), (0, 4096, 0, -4096)[angle//1024])
        self.assertEqual(direction.look([0, 0, -4096], [0]*3, 0), [0, 0, 0])
        self.assertEqual(direction.look([0, 0, 4096], [0]*3, 0), [0xfffffc00, 2048, 0])
        with self.assertRaises(ValueError): direction.ratan2(-0x80000000, 1)
        # Exercises the overflow-avoidance division branch.
        self.assertEqual(direction.ratan2(0x400000, 0x400000), 512)


if __name__ == '__main__': unittest.main()
