"""Execute isolated renderer instructions from the user's SCUS_942.44.

No ROM bytes are embedded. This is a deliberately small MIPS-I interpreter,
not a PSX emulator: branch/load delay slots are modeled, GTE projection/GPU
rasterization are NOT. Packet tests supply already projected SXY registers.
Unknown instructions and uninitialized reads fail closed.
Usage: python tools/verify_renderer_contract.py path/to/SCUS_942.44
"""
import hashlib
from pathlib import Path
import random
import struct
import sys
import unittest

SHA256 = "1b93cc563c90966e1d79b750f7d19bc1662859cceeb991775e9e9a22656d71d1"
MASK = 0xFFFFFFFF
SCRATCH, OT, PACKETS, TEXTURES = 0x1F800000, 0x80080000, 0x80090000, 0x800A0000
STOP = 0x800410EC


def signed(x, bits=32):
    return (x & ((1 << (bits - 1)) - 1)) - (x & (1 << (bits - 1)))


class Machine:
    def __init__(self, exe):
        self.exe = exe
        self.base = struct.unpack_from("<I", exe, 0x18)[0]
        self.reg = [0] * 32
        self.mem = {}
        self.gte = {12: 0x00100020, 13: 0x00300040, 14: 0x00500060}
        self.load = None

    def word(self, addr):
        if addr in self.mem:
            return self.mem[addr]
        offset = addr - self.base + 0x800
        if 0x800 <= offset <= len(self.exe) - 4:
            return struct.unpack_from("<I", self.exe, offset)[0]
        raise AssertionError(f"Uninitialized read {addr:08x}")

    def byte(self, addr):
        return (self.word(addr & ~3) >> ((addr & 3) * 8)) & 255

    def run(self, pc, stops, limit=20000):
        pending_branch = None
        for _ in range(limit):
            if pc in stops:
                if self.load is not None or pending_branch is not None:
                    raise AssertionError("Stopped before a delayed operation completed")
                return
            code = self.word(pc)
            op, rs, rt, rd = code >> 26, code >> 21 & 31, code >> 16 & 31, code >> 11 & 31
            shift, fn, imm = code >> 6 & 31, code & 63, signed(code & 0xFFFF, 16)
            a, b = self.reg[rs], self.reg[rt]
            write, new_load, branch = None, None, None
            if op == 0:
                if fn == 0: write = (rd, b << shift)
                elif fn == 2: write = (rd, b >> shift)
                elif fn == 3: write = (rd, signed(b) >> shift)
                elif fn == 4: write = (rd, b << (a & 31))
                elif fn == 6: write = (rd, b >> (a & 31))
                elif fn == 7: write = (rd, signed(b) >> (a & 31))
                elif fn == 8: branch = a
                elif fn in (0x20, 0x21): write = (rd, a + b)
                elif fn in (0x22, 0x23): write = (rd, a - b)
                elif fn == 0x24: write = (rd, a & b)
                elif fn == 0x25: write = (rd, a | b)
                elif fn == 0x27: write = (rd, ~(a | b))
                else: raise AssertionError(f"Unsupported SPECIAL {code:08x} at {pc:08x}")
            elif op in (1, 4, 5, 6, 7):
                if op == 1:
                    if rt not in (0, 1): raise AssertionError("Unsupported REGIMM")
                    taken = signed(a) < 0 if rt == 0 else signed(a) >= 0
                else:
                    taken = {4: a == b, 5: a != b, 6: signed(a) <= 0, 7: signed(a) > 0}[op]
                branch = pc + 4 + (imm << 2) if taken else pc + 8
            elif op in (8, 9): write = (rt, a + imm)
            elif op == 0xC: write = (rt, a & (code & 0xFFFF))
            elif op == 0xD: write = (rt, a | (code & 0xFFFF))
            elif op == 0xF: write = (rt, (code & 0xFFFF) << 16)
            elif op == 0x23: new_load = (rt, self.word((a + imm) & MASK))
            elif op == 0x24: new_load = (rt, self.byte((a + imm) & MASK))
            elif op == 0x2B: self.mem[(a + imm) & MASK] = b
            elif op == 0x3A: self.mem[(a + imm) & MASK] = self.gte[rt]
            else: raise AssertionError(f"Unsupported opcode {code:08x} at {pc:08x}")
            if self.load and (write is None or self.load[0] != write[0]):
                self.reg[self.load[0]] = self.load[1] & MASK
            if write: self.reg[write[0]] = write[1] & MASK
            self.reg[0] = 0
            self.load = new_load
            next_pc = pending_branch if pending_branch is not None else pc + 4
            if pending_branch is not None and branch is not None:
                raise AssertionError("Branch in delay slot")
            pending_branch, pc = branch, next_pc & MASK
        raise AssertionError("Instruction budget exceeded")


class RendererContract(unittest.TestCase):
    exe = b""

    def machine(self):
        m = Machine(self.exe)
        m.reg[3], m.reg[23], m.reg[30] = SCRATCH, PACKETS, OT  # v1,s7,s8
        m.reg[29] = 0x800B1000  # backward SLST cursor
        m.reg[12], m.reg[9] = OT + 2047 * 4, 0xFFFFFF  # t4,t1
        m.mem[SCRATCH + 0x9C] = 0x800B0000
        m.mem[SCRATCH + 0x74] = 0x2000
        return m

    def ordered(self, mode, buckets, world_zero=False):
        m = self.machine()
        for i in range(2048):
            # Default path must construct this itself, without ClearOTag.
            if mode != 0x80041550 or i == 2047:
                m.mem[OT + i * 4] = ((OT + (i + 1) * 4) & 0xFFFFFF) if i < 2047 else 0xFFFFFF
        labels = {}
        effective = []
        if world_zero: m.mem[SCRATCH + 0x74] = 0
        for i, bucket in enumerate(buckets):
            pointer = m.reg[23]
            labels[pointer & 0xFFFFFF] = i
            m.reg[2], m.reg[4], m.reg[25] = OT + bucket * 4, 40, 9 << 24
            m.run(mode, {STOP})
            self.assertEqual(m.reg[23], pointer + 40)
            self.assertEqual(m.mem[pointer] >> 24, 9)
            effective.append((m.reg[12] - OT) // 4)
        if mode == 0x80041550:
            m.mem[SCRATCH + 0x70] = mode
            m.run(0x800414D4, {0x8004150C})
        cursor, order, visited = OT & 0xFFFFFF, [], set()
        while cursor != 0xFFFFFF:
            self.assertNotIn(cursor, visited)
            visited.add(cursor)
            if cursor in labels: order.append(labels[cursor])
            cursor = m.word(cursor | 0x80000000) & 0xFFFFFF
        return order, effective

    def test_default_preserves_list_order_not_depth_sort(self):
        order, buckets = self.ordered(0x80041550, [300, 1600, 200, 1000])
        self.assertEqual(order, [3, 2, 1, 0])
        self.assertEqual(buckets, [300, 300, 200, 200])

    def test_default_order_randomized_and_boundaries(self):
        rng = random.Random(94244)
        for values in ([0], [2047], [7] * 12, [2047, 0, 2047], [rng.randrange(1905) for _ in range(200)]):
            order, buckets = self.ordered(0x80041550, values)
            self.assertEqual(order, list(reversed(range(len(values)))))
            self.assertEqual(buckets, [min(values[:i+1]) for i in range(len(values))])

    def test_absolute_depth_variant(self):
        values = [300, 1600, 200, 1000, 200]
        order, _ = self.ordered(0x80041618, values)
        self.assertEqual(order, sorted(range(len(values)), key=lambda i: (values[i], -i)))

    def test_precleared_monotone_and_world_zero_variants(self):
        for world_zero in (False, True):
            order, _ = self.ordered(0x800415A4, [300, 1600, 200, 1000], world_zero)
            self.assertEqual(order, [3, 2, 1, 0])

    def packet(self, quad, texture, blend=3, second_half=False):
        m = self.machine()
        m.reg[13:16] = [0x112233, 0x445566, 0x778899]  # t5..t7 colors
        m.reg[5] = 0xAABBCC  # fourth color
        m.reg[20], m.reg[21] = SCRATCH, texture  # s4,s5
        m.reg[17], m.reg[16] = TEXTURES, TEXTURES + 0x10000
        m.reg[31] = 128  # computed depth >> 3, not a return address
        m.mem[SCRATCH + 0x294] = 0x800C0000
        m.mem[SCRATCH + 0x70] = 0x8007FFF0  # stop before OT insertion
        m.mem[SCRATCH + 0x184] = int(second_half)
        m.mem[TEXTURES] = 0x12340201  # CLUT + UV1
        m.mem[TEXTURES + 4] = (blend << 21) | 0x0403  # TPAGE + UV2
        m.mem[TEXTURES + 8] = 0x08070605  # UV3/UV4
        return m

    def test_untextured_gouraud_triangle_and_quad_packets(self):
        for quad in (False, True):
            m = self.packet(quad, 0)
            m.run(0x80041A3C if quad else 0x80041768, {0x8007FFF0})
            self.assertEqual(m.mem[PACKETS + 4], (0x38000000 if quad else 0x30000000) | 0x112233)
            self.assertEqual(m.reg[4], 36 if quad else 28)
            self.assertEqual(m.reg[25] >> 24, 8 if quad else 6)
            self.assertEqual(m.mem[PACKETS + 12], 0x445566)
            self.assertEqual(m.mem[PACKETS + 20], 0x778899)
            if quad: self.assertEqual(m.mem[PACKETS + 28], 0xAABBCC)

    def test_textured_packets_and_all_blend_codes(self):
        for quad in (False, True):
            for blend in range(4):
                m = self.packet(quad, 1, blend)
                m.run(0x80041A3C if quad else 0x80041768, {0x8007FFF0})
                opcode = (0x3C if quad else 0x34) | (2 if blend != 3 else 0)
                self.assertEqual(m.mem[PACKETS + 4], opcode << 24 | 0x112233)
                self.assertEqual(m.reg[4], 52 if quad else 40)
                self.assertEqual(m.reg[25] >> 24, 12 if quad else 9)
                self.assertEqual(m.mem[PACKETS + 12], 0x12340201)
                self.assertEqual(m.mem[PACKETS + 24], blend << 21 | 0x0403)
                self.assertEqual(m.mem[PACKETS + 36], 0x08070605)
                if quad: self.assertEqual(m.mem[PACKETS + 48], 0x0807)

    def test_second_half_uv_rotation_preserves_clut_tpage(self):
        m = self.packet(False, 1, blend=2, second_half=True)
        m.run(0x80041768, {0x8007FFF0})
        self.assertEqual(m.mem[PACKETS + 12], 0x12340403)
        self.assertEqual(m.mem[PACKETS + 24], 2 << 21 | 0x0605)
        self.assertEqual(m.mem[PACKETS + 36], 0x0807)

    def test_packet_colors_preserve_all_channels_without_neutral_clamp(self):
        # Supply already decoded colours, including the textured neutral value
        # and overbright channels. This executes the EXE, not a replica emitter.
        rng = random.Random(41768)
        examples = [[0, 0x808080, 0xB9A091, 0xFFFFFF]]
        examples += [[rng.randrange(1 << 24) for _ in range(4)] for _ in range(64)]
        for quad in (False, True):
            for textured in (False, True):
                for blend in range(4):
                    for colors in examples:
                        m = self.packet(quad, int(textured), blend)
                        m.reg[13:16], m.reg[5] = colors[:3], colors[3]
                        m.run(0x80041A3C if quad else 0x80041768, {0x8007FFF0})
                        offsets = (4, 16, 28, 40) if textured else (4, 12, 20, 28)
                        for offset, wanted in zip(offsets[:4 if quad else 3], colors):
                            self.assertEqual(m.mem[PACKETS + offset] & 0xFFFFFF, wanted)

    def test_color_lookup_address_all_indices_and_effect_flags(self):
        # Execute the original address-generation block independently of the
        # NSF decoder. Stop before LW so the interpreter needs no GTE extension.
        for index in range(1024):
            for effects in range(4):
                m = self.machine()
                m.reg[8] = ((index >> 4) & 15) | ((index >> 8) << 16) | (effects << 18) | (0xA53 << 4) | (0xC27 << 20)
                m.reg[15] = (0xB91 << 4) | (index & 15)
                m.reg[18] = TEXTURES
                m.run(0x8004171C, {0x80041740})
                self.assertEqual(m.reg[15], TEXTURES + index * 4)

    def test_texture_lod_all_distance_bands(self):
        shifts = [29]*4 + [27]*2 + [25]*2 + [23]*2 + [21]*2 + [19]*4 + [17]*8 + [15]*8
        descriptor = 0x80000001 | sum((i % 4) << s for i, s in enumerate(range(29, 14, -2)))
        for band, shift in enumerate(shifts):
            for quad in (False, True):
                m = self.packet(quad, 0x800)
                m.mem[TEXTURES + 0x10000] = descriptor
                m.mem[SCRATCH + 0x22C] = 6
                for i in range(0, 32, 4):
                    m.mem[SCRATCH + 0xD0 + i] = int.from_bytes(bytes(shifts[i:i+4]), 'little')
                m.reg[31] = band << 6
                m.run(0x80041A3C if quad else 0x80041768, {0x80041B14 if quad else 0x8004183C})
                self.assertEqual(m.reg[21], (1 + ((descriptor >> shift) & 3)) * 12)

    def test_texture_flipbook_and_indirect_lod(self):
        for indirect in (False, True):
            for quad in (False, True):
                for tick in (0, 1, 3, 4, 31, 255):
                    m = self.packet(quad, 0x800)
                    base, mask, phase, rate = 5, 3, 1, 2
                    descriptor = base | (mask << 12) | (phase << 19) | (rate << 26) | (int(indirect) << 11)
                    m.mem[TEXTURES + 0x10000] = descriptor
                    m.mem[SCRATCH + 0x22C], m.mem[SCRATCH + 0xD0] = 6, 0x1D1D1D1D
                    m.mem[SCRATCH + 0x1C] = tick
                    frame = base + (((tick >> rate) + phase) & mask)
                    if indirect:
                        m.mem[TEXTURES + 0x10000 + frame*4] = 0x80000009 | (2 << 29)
                    m.run(0x80041A3C if quad else 0x80041768, {0x80041B14 if quad else 0x8004183C})
                    self.assertEqual(m.reg[21], (11 if indirect else frame) * 12)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    RendererContract.exe = Path(sys.argv.pop()).read_bytes()
    if hashlib.sha256(RendererContract.exe).hexdigest() != SHA256:
        raise SystemExit("Wrong EXE: this address contract is specific to SCUS_942.44 USA")
    unittest.main(verbosity=2)
