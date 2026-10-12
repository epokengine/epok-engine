import struct
import unittest

from verify_epok_texture_vram import verify


class TextureVramTests(unittest.TestCase):
    def fixture(self, streamed):
        pixels = [0x1111, 0x1212, 0x1313, 0x1414]
        palette = list(range(16))
        vram = [0] * (1024 * 512)
        vram[640:642] = pixels[:2]
        vram[1664:1666] = pixels[2:]
        vram[300 * 1024 + 640:300 * 1024 + 656] = palette
        archive = bytearray(2 * 65536)
        struct.pack_into("<4H", archive, len(archive) - 8, *pixels)
        source = "nullptr,texture_0_palette,4,1u,65528" if streamed else "texture_0_pixels,texture_0_palette,4"
        header = "uint16_t texture_0_palette[]={" + ",".join(map(str, palette)) + "};\n"
        if not streamed:
            header += "uint16_t texture_0_pixels[]={" + ",".join(map(str, pixels)) + "};\n"
        header += "Texture bank_textures_0[]={{7,2,640,0,2,640,300," + source + "},{}};"
        return header, struct.pack("<524288H", *vram), bytes(archive)

    def test_resident_and_streamed_exact_page_end(self):
        for streamed in (False, True):
            header, vram, archive = self.fixture(streamed)
            self.assertEqual(verify(header, vram, archive=archive),
                             {"textures": 1, "streamed": int(streamed), "bytes": 40})

    def test_corruption_is_not_silently_skipped(self):
        header, vram, archive = self.fixture(True)
        for bad in (header.replace("65528", "65532"), header.replace("1u", "2u"),
                    header.replace("nullptr", "texture_0_pixels"), header.replace(",4,", ",16,"),
                    header.replace("texture_0_palette,4", "unrecognized,4")):
            with self.assertRaises(AssertionError):
                verify(bad, vram, archive=archive)
        with self.assertRaises(AssertionError):
            verify(header, vram)
        bad_vram = bytearray(vram)
        bad_vram[640 * 2] ^= 1
        with self.assertRaisesRegex(AssertionError, "Texture 0, row 0 mismatch"):
            verify(header, bad_vram, archive=archive)


if __name__ == "__main__":
    unittest.main()
