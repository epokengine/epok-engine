import unittest

from reference_sweep_cameras import epok_camera


class EpokCameraTests(unittest.TestCase):
    def test_reference_matrix_converts_to_unit_basis_and_focal_scales(self):
        # Rows X and Z unit (Q12); row Y = -(5/8) * up, as the reference loads it.
        reference = dict(rotation=[[4096, 0, 0], [0, -2560, 0], [0, 0, 4096]],
                         origin=[400.0, -800.0, 1200.0], h=288, centre=(256, 120), translation=[0, 0, 0])
        camera = epok_camera(reference)
        self.assertEqual(camera["basis"], [[4096, 0, 0], [0, 4096, 0], [0, 0, 4096]])
        self.assertEqual(camera["origin"], [4096, -8192, 12288])
        self.assertEqual(camera["row_scale"], [round(288 * 4096 / 160), round(288 * 0.625 * 4096 / 160)])

    def test_unexpected_matrix_or_translation_is_rejected(self):
        base = dict(rotation=[[4096, 0, 0], [0, -2560, 0], [0, 0, 4096]], origin=[0, 0, 0], h=288,
                    centre=(256, 120), translation=[0, 0, 0])
        self.assertIsNone(epok_camera(dict(base, rotation=[[4096, 0, 0], [0, -4096, 0], [0, 0, 4096]])))
        self.assertIsNone(epok_camera(dict(base, translation=[1, 0, 0])))


if __name__ == "__main__":
    unittest.main()
