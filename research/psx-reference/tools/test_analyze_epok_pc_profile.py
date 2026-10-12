import subprocess
import unittest
from unittest import mock

import analyze_epok_pc_profile


class Addr2LineTests(unittest.TestCase):
    def test_inline_chains_are_innermost_first_per_address(self):
        output = "\n".join([
            "0x80053010", "epok::pack_projection_input(long const*)", "/src/gte_geometry.hpp:111",
            "draw_ordered_world(...)", "/src/ordered_renderer.hpp:134 (discriminator 2)",
            "0x80053014", "draw_ordered_world(...)", "/src/ordered_renderer.hpp:107", ""])
        with mock.patch.object(subprocess, "run", return_value=mock.Mock(stdout=output)):
            chains = analyze_epok_pc_profile.addr2line("tool", "elf", [0x80053010, 0x80053014])
        self.assertEqual(chains[0x80053010][0], ("epok::pack_projection_input(long const*)", "/src/gte_geometry.hpp", 111))
        self.assertEqual(chains[0x80053010][-1][2], 134)
        self.assertEqual(chains[0x80053014], [("draw_ordered_world(...)", "/src/ordered_renderer.hpp", 107)])


if __name__ == "__main__":
    unittest.main()
