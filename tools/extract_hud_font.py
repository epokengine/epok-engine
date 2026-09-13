"""Decode the pinned PsyQo system font for the editor's native HUD preview."""
import argparse
import pathlib
import re


def decode(source):
    body = source[source.index("{") + 1:source.index("}")]
    data = bytes(int(value, 0) for value in re.findall(r"0x[\da-fA-F]+|\b\d+\b", body))
    lut, cursor = data[:2]
    bits = 0x100
    rows = []
    for _ in range(256 * 48 // 8):
        node = 2
        while node > 0:
            if bits == 0x100:
                bits = data[cursor] | 0x10000
                cursor += 1
            bit = bits & 1
            bits >>= 1
            node = data[node + bit]
            if node >= 128:
                node -= 256
        rows.append(data[lut - node])
    return bytes(rows[(glyph // 32 * 16 + y) * 32 + glyph % 32]
                 for glyph in range(96) for y in range(16))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Verify the committed bitmap without changing it")
    args = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parents[1]
    source = root / "third_party/nugget/psyqo/src/system-font.inc"
    destination = root / "resources/editor/psx-font.bin"
    expected = decode(source.read_text(encoding="utf-8"))
    if args.check:
        if destination.read_bytes() != expected:
            raise SystemExit("HUD font differs from the pinned PsyQo font")
        print("PASS editor bitmap matches the pinned PsyQo font")
    else:
        destination.write_bytes(expected)
        print("Wrote resources/editor/psx-font.bin; preserve its attribution notice")


if __name__ == "__main__":
    main()
