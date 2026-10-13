"""Compare every static cooked texture/CLUT upload to an actual PSX VRAM dump."""
import argparse
from pathlib import Path
import re
import struct


def verify(header, raw, bank_index=0, archive=None):
    assert len(raw) == 1024 * 512 * 2, len(raw)
    vram = struct.unpack("<524288H", raw)
    arrays = {(int(i), kind): [int(v) for v in data.split(",") if v.strip()] for i, kind, data in
              re.findall(r"uint16_t texture_(\d+)_(pixels|palette)\[\]=\{([^}]+)\}", header)}
    match = re.search(rf"Texture bank_textures_{bank_index}\[\]=\{{(.*?)\}};", header, re.S)
    assert match, f"Texture bank {bank_index} not found"
    checked = count = streamed = 0
    for descriptor in re.findall(r"\{([^{}]*)\}", match[1]):
        if not descriptor.strip():
            continue
        fields = [field.strip() for field in descriptor.split(",")]
        assert len(fields) in (9, 10, 12), f"Unrecognized descriptor: {descriptor}"
        width, height, x, y, row, cx, cy = map(int, fields[:7])
        palette_match = re.fullmatch(r"texture_(\d+)_palette", fields[8])
        assert palette_match, f"Missing palette in descriptor: {descriptor}"
        index = int(palette_match[1])
        bits = int(fields[9]) if len(fields) > 9 else 8
        assert bits in (4, 8) and 0 < width <= 256 and 0 < height <= 256
        assert row > 0 and row % 2 == 0 and row * (16 // bits) >= width
        assert x + row <= 1024 and y + height <= 512
        assert (x % 64) * (16 // bits) + width <= 256 and y % 256 + height <= 256
        if len(fields) == 12:
            assert fields[7] == "nullptr" and archive is not None, "Streamed pixels require GEOMETRY.BIN"
            page, offset = (int(field.removesuffix("u")) for field in fields[10:])
            size = row * height * 2
            assert offset % 4 == 0 and offset + size <= 65536
            begin = page * 65536 + offset
            assert len(archive) % 65536 == 0 and begin + size <= len(archive)
            pixels = struct.unpack_from(f"<{row * height}H", archive, begin)
            streamed += 1
        else:
            assert fields[7] == f"texture_{index}_pixels"
            pixels = arrays[index, "pixels"]
        assert len(pixels) == row * height
        for line in range(height):
            actual = vram[(y + line) * 1024 + x:(y + line) * 1024 + x + row]
            assert tuple(pixels[line * row:(line + 1) * row]) == actual, f"Texture {index}, row {line} mismatch"
        palette = arrays[index, "palette"]
        assert len(palette) == (16 if bits == 4 else 256)
        assert cx % 16 == 0 and cx + len(palette) <= 1024 and cy < 512
        assert tuple(palette) == vram[cy * 1024 + cx:cy * 1024 + cx + len(palette)], f"Palette {index} mismatch"
        checked += 2 * (len(pixels) + len(palette))
        count += 1
    assert count, "No cooked texture descriptors found"
    return {"textures": count, "streamed": streamed, "bytes": checked}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("header", type=Path)
    parser.add_argument("vram", type=Path)
    parser.add_argument("--bank", type=int, default=0)
    parser.add_argument("--archive", type=Path, help="Defaults to GEOMETRY.BIN beside the header")
    args = parser.parse_args()
    archive = args.archive or args.header.parent / "GEOMETRY.BIN"
    result = verify(args.header.read_text(), args.vram.read_bytes(), args.bank,
                    archive.read_bytes() if archive.is_file() else None)
    print(f"VRAM readback matches all {result['textures']} cooked textures and CLUTs "
          f"({result['streamed']} streamed): {result['bytes']} bytes.")


if __name__ == "__main__":
    main()
