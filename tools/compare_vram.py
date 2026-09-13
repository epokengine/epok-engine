"""Compare two native VRAM captures (profile_runtime.py vram.bin) in the display area."""
import epok_documents as documents
import argparse
import json
import struct
import sys
from pathlib import Path


def rgb(word):
    return ((word & 31) * 255 // 31, ((word >> 5) & 31) * 255 // 31, ((word >> 10) & 31) * 255 // 31)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    parser.add_argument("--width", type=int, default=640)
    parser.add_argument("--height", type=int, default=480)
    parser.add_argument("--png", type=Path, help="Write before/after/difference strip")
    parser.add_argument("--json", type=Path)
    args = parser.parse_args()
    a = args.before.read_bytes()
    b = args.after.read_bytes()
    width, height = args.width, args.height
    differing = 0
    max_delta = 0
    histogram = {}
    diff_rows = []
    for y in range(height):
        row_a = struct.unpack_from(f"<{width}H", a, y * 2048)
        row_b = struct.unpack_from(f"<{width}H", b, y * 2048)
        row = []
        for x in range(width):
            if row_a[x] != row_b[x]:
                differing += 1
                ca, cb = rgb(row_a[x]), rgb(row_b[x])
                delta = max(abs(ca[i] - cb[i]) for i in range(3))
                max_delta = max(max_delta, delta)
                bucket = 8 if delta >= 64 else 4 if delta >= 32 else 2 if delta >= 16 else 1
                histogram[bucket] = histogram.get(bucket, 0) + 1
                row.append(x)
        if row:
            diff_rows.append((y, len(row)))
    total = width * height
    report = {
        "differing_pixels": differing,
        "differing_per_mille": round(differing * 1000 / total, 3),
        "max_channel_delta": max_delta,
        "delta_histogram": {"<16": histogram.get(1, 0), "16-31": histogram.get(2, 0), "32-63": histogram.get(4, 0), ">=64": histogram.get(8, 0)},
        "rows_with_differences": len(diff_rows),
    }
    print(json.dumps(report, indent=2))
    if args.json:
        documents.write_text(args.json, json.dumps(report, indent=2))
    if args.png:
        from PIL import Image
        strip = Image.new("RGB", (width * 3, height))
        for index, data in enumerate((a, b)):
            pixels = bytearray()
            for y in range(height):
                for word in struct.unpack_from(f"<{width}H", data, y * 2048):
                    pixels.extend(rgb(word))
            strip.paste(Image.frombytes("RGB", (width, height), bytes(pixels)), (index * width, 0))
        pixels = bytearray()
        for y in range(height):
            row_a = struct.unpack_from(f"<{width}H", a, y * 2048)
            row_b = struct.unpack_from(f"<{width}H", b, y * 2048)
            for x in range(width):
                pixels.extend((255, 255, 255) if row_a[x] != row_b[x] else (0, 0, 0))
        strip.paste(Image.frombytes("RGB", (width, height), bytes(pixels)), (2 * width, 0))
        strip.save(args.png)
    return 0


if __name__ == "__main__":
    sys.exit(main())
