"""Convert reference scenery-pass cameras into an Epok visibility-sweep table.

Reads an original_observer.lua profile with world_passes/world_lists and its
per-frame rows (GTE control registers after pass setup, camera position), and
the source route export. Each reference list is matched word for word to a
(route, sample); for every matched sample the pass of median duration is
chosen, and the camera of the frame that pass closed in is converted:

  reference: screen = centre + H * (R . (p - origin) / 4) / z, with p in native
             units, R rows X and Z unit Q12 and row Y = -(5/8) * up
  Epok:      world = native / 400 (same axes); basis rows right, up, forward
             (Q12); row_scale folds H and the 5/8 into rows 0 and 1 for the
             renderer's 160-pixel focal length.

The reference's dynamic screen-centre offsets are not reproduced (the Epok
centre stays at the display middle); their range is reported. Frames that
drew more than one scenery pass, or whose matrix does not have the expected
row lengths, are skipped and counted. Writes a C++ header for
EPOK_SWEEP_CAMERAS and a JSON summary. Read-only on its inputs.
"""
import argparse
import json
import math
from pathlib import Path
import statistics

ROW_GTE, ROW_CAMERA, ROW_CALLS = 8, 41, 3
EPOK_FOCAL = 160


def s16(value):
    value &= 0xffff
    return value - 65536 if value >= 32768 else value


def s32(value):
    value &= 0xffffffff
    return value - (1 << 32) if value >= 1 << 31 else value


def reference_camera(row):
    g = row[ROW_GTE:ROW_GTE + 32]
    rotation = [[s16(g[0]), s16(g[0] >> 16), s16(g[1])],
                [s16(g[1] >> 16), s16(g[2]), s16(g[2] >> 16)],
                [s16(g[3]), s16(g[3] >> 16), s16(g[4])]]
    origin = [s32(v) / 256 for v in row[ROW_CAMERA:ROW_CAMERA + 3]]
    return dict(rotation=rotation, origin=origin, h=g[26] & 0xffff,
                centre=(s32(g[24]) / 65536, s32(g[25]) / 65536 + 12),
                translation=[s32(g[i]) for i in (5, 6, 7)])


def epok_camera(reference):
    rows = reference["rotation"]
    norms = [math.sqrt(sum(v * v for v in row)) for row in rows]
    if abs(norms[0] - 4096) > 8 or abs(norms[2] - 4096) > 8 or abs(norms[1] - 2560) > 16:
        return None
    if any(reference["translation"]):
        return None
    up = [round(-v * 4096 / norms[1]) for v in rows[1]]
    basis = [rows[0], up, rows[2]]
    h = reference["h"]
    row_scale = [round(h * 4096 / EPOK_FOCAL), round(h * (norms[1] / 4096) * 4096 / EPOK_FOCAL)]
    origin = [round(v * 4096 / 400) for v in reference["origin"]]
    return dict(basis=basis, row_scale=row_scale, origin=origin)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("original", type=Path)
    parser.add_argument("routes", type=Path)
    parser.add_argument("--header", type=Path, required=True)
    parser.add_argument("--summary", type=Path, required=True)
    parser.add_argument("--all-passes", type=Path,
                        help="Also write every matched pass's camera (not only the median) as JSON")
    args = parser.parse_args()
    for path in (args.header, args.summary, args.all_passes):
        if path is None:
            continue
        if path.exists():
            parser.error(f"Refusing to overwrite {path}")
    profile = json.loads(args.original.read_text())
    routes = json.loads(args.routes.read_text())
    if routes["coordinate_scale"] != .00025 or profile.get("overflow") or profile.get("callback_error"):
        raise SystemExit("Unexpected route scale or incomplete reference profile")
    index, first = {}, 0
    for route in routes["routes"]:
        for sample_index, sample in enumerate(route["Samples"]):
            index.setdefault(tuple(sample["Polygons"]), []).append(first + sample_index)
        first += len(route["Samples"])
    rows = profile["rows"]
    ends = [row[0] for row in rows]
    lists = profile["world_lists"]
    passes = {}
    for start, cycles, count, list_id, *_ in profile["world_passes"]:
        if not list_id:
            continue
        matches = index.get(tuple(lists[list_id - 1]), [])
        if len(matches) != 1:
            continue
        frame = next((i for i, end in enumerate(ends) if end >= start), None)
        if frame is None:
            continue
        passes.setdefault(matches[0], []).append((cycles, frame))
    if args.all_passes:
        every = []
        for sample in sorted(passes):
            for cycles, frame in passes[sample]:
                row = rows[frame]
                camera = epok_camera(reference_camera(row)) if row[ROW_CALLS] == 1 else None
                if camera:
                    every.append(dict(sample=sample, frame=frame, **camera))
        args.all_passes.write_text(json.dumps(dict(scope="Every matched pass camera", cameras=every)), encoding="utf-8")
        print(f"all passes: {len(every)} cameras over {len({c['sample'] for c in every})} samples")
    table, skipped, centres = [], dict(multi_pass_frame=0, unexpected_matrix=0), []
    for sample in sorted(passes):
        candidates = sorted(passes[sample])
        cycles, frame = candidates[len(candidates) // 2]
        row = rows[frame]
        if row[ROW_CALLS] != 1:
            skipped["multi_pass_frame"] += 1
            continue
        reference = reference_camera(row)
        camera = epok_camera(reference)
        if camera is None:
            skipped["unexpected_matrix"] += 1
            continue
        centres.append(reference["centre"])
        table.append(dict(sample=sample, reference_cycles=cycles, **camera))
    entries = ",\n".join(
        "{%du,{%s},{%s},{%s}}" % (t["sample"], ",".join(map(str, t["origin"])),
                                 ",".join(str(v) for row in t["basis"] for v in row),
                                 ",".join(map(str, t["row_scale"]))) for t in table)
    args.header.write_text(
        "// Generated by reference_sweep_cameras.py: diagnostic sweep cameras.\n"
        "#pragma once\n#include <stdint.h>\nnamespace epok {\n"
        "struct SweepCamera {uint16_t sample;int32_t origin[3];int16_t basis[9];int32_t row_scale[2];};\n"
        f"inline constexpr SweepCamera sweep_cameras[]={{\n{entries}\n}};\n"
        f"inline constexpr unsigned sweep_camera_count={len(table)};\n}}\n", encoding="utf-8")
    summary = dict(scope=__doc__, samples=len(table), skipped=skipped,
                   centre_x=[min(c[0] for c in centres), max(c[0] for c in centres)] if centres else None,
                   centre_y=[min(c[1] for c in centres), max(c[1] for c in centres)] if centres else None,
                   h=sorted({t["row_scale"][0] for t in table}) and statistics.median(t["row_scale"][0] for t in table) * EPOK_FOCAL / 4096,
                   cameras=table)
    args.summary.write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(json.dumps({k: v for k, v in summary.items() if k not in ("scope", "cameras")}))


if __name__ == "__main__":
    main()
