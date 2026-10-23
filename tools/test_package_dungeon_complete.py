import struct
import unittest
from package_dungeon_complete import chunk, chunks, midi_programs, trim_soundfont


class CompletePackage(unittest.TestCase):
    def test_midi_merges_channel_state_and_skips_long_metadata(self):
        track0 = b"\0\xff\x01\x81\x01" + b"x" * 129 + b"\0\xc0\x2a\x64\x90\x3c\x40\0\x3c\0\0\xff\x2f\0"
        track1 = b"\0\x91\x40\x01\x32\xc0\x30\0\xff\x2f\0"
        track2 = b"\0\xc9\x18\x01\x99\x24\x40\0\xff\x2f\0"
        header = b"MThd" + struct.pack(">IHHH", 6, 1, 3, 96)
        midi = header + b"".join(b"MTrk" + struct.pack(">I", len(t)) + t for t in [track0, track1, track2])
        self.assertEqual(midi_programs(midi), {(0, 0), (0, 48), (128, 24)})

    def test_pruning_preserves_pcm_vorbis_pitch_loops_and_program_numbers(self):
        phdr = b"".join(struct.pack("<20sHHHIII", name, program, 0, bag, 0, 0, 0)
                        for name, program, bag in [(b"Unused", 0, 0), (b"Keep", 42, 1), (b"EOP", 0, 2)])
        inst = b"".join(struct.pack("<20sH", name, bag) for name, bag in [(b"Unused", 0), (b"Keep", 1), (b"EOI", 3)])
        pcm = struct.pack("<6h", 0, 20, -30, 40, -50, 0)
        ogg = b"OggS_preserve_this_compressed_sample"
        smpl = b"UNUSED!!" + pcm + ogg
        headers = [(b"Unused", 0, 4, 1, 3, 22050, 60, 0, 0, 1),
                   (b"PCM", 4, 10, 5, 9, 11025, 48, -7, 0, 1),
                   (b"Vorbis", 20, len(smpl), 3, 20, 32000, 67, 9, 0, 17),
                   (b"EOS", 0, 0, 0, 0, 0, 0, 0, 0, 1)]
        tables = [(b"phdr", phdr), (b"pbag", b"".join(struct.pack("<HH", n, 0) for n in range(3))),
                  (b"pmod", bytes(10)), (b"pgen", struct.pack("<6H", 41, 0, 41, 1, 0, 0)),
                  (b"inst", inst), (b"ibag", b"".join(struct.pack("<HH", n, 0) for n in range(4))),
                  (b"imod", bytes(10)), (b"igen", struct.pack("<8H", 53, 0, 53, 1, 53, 2, 0, 0)),
                  (b"shdr", b"".join(struct.pack("<20s5IBbHH", *h) for h in headers))]
        sf = chunk(b"RIFF", b"sfbk" + chunk(b"LIST", b"sdta" + chunk(b"smpl", smpl)) +
                   chunk(b"LIST", b"pdta" + b"".join(chunk(tag, data) for tag, data in tables)))
        result, report = trim_soundfont(sf, {(0, 42)})
        self.assertEqual((report["presets"], report["instruments"], report["samples"]), (1, 1, 2))
        groups = {data[:4]: dict(chunks(data[4:])) for _, data in chunks(result[12:])}
        new = list(struct.iter_unpack("<20s5IBbHH", groups[b"pdta"][b"shdr"]))
        payload = groups[b"sdta"][b"smpl"]
        self.assertEqual(payload[new[0][1] * 2:new[0][2] * 2], pcm)
        self.assertEqual((new[0][3] - new[0][1], new[0][4] - new[0][1]), (1, 5))
        self.assertEqual(payload[new[1][1]:new[1][2]], ogg)
        self.assertEqual(new[1][3:8], headers[2][3:8])
        self.assertEqual(new[0][5:8], headers[1][5:8])
        self.assertEqual(struct.unpack_from("<20sHHHIII", groups[b"pdta"][b"phdr"])[1:4], (42, 0, 0))
        self.assertEqual(list(struct.iter_unpack("<HH", groups[b"pdta"][b"igen"]))[:2], [(53, 0), (53, 1)])
        self.assertEqual(groups[b"pdta"][b"shdr"][-46:][20:], bytes(26))


if __name__ == "__main__":
    unittest.main()
