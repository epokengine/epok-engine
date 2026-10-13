"""Compare completed-frame work counters for matching observed route/work groups.

Route state is read separately from the published performance counters and may
be one frame ahead. This is a grouped diagnostic, not a synchronized/pixel-exact
replay comparison. Global frame-time medians from different route intervals are
reported separately, never subtracted as a controlled before/after result.
The camera mode instead requires coherent published frame IDs and the exact
rendered Q12 origin, full 3x4 basis, row scales and committed ordered sample.
It still matches observed workloads, not synchronized CD service or gameplay.
"""
import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
from statistics import median


FIELDS = ["frame_scanlines", "simulation_scanlines", "world_scanlines",
          "prepare_scanlines", "camera_scanlines", "render_scanlines",
          "vertex_scanlines", "polygon_scanlines", "finish_scanlines",
          "gte_vertices", "software_vertices"]
CAMERA_FIELDS = ([f"camera_origin_{c}" for c in range(3)]
                + [f"camera_basis_{r}_{c}" for r in range(3) for c in range(4)]
                + [f"row_scale_{r}" for r in range(2)])


def counter_comparisons(grouped, common, fields):
    counters = {}
    for field in fields:
        available = [all(type(r.get(field)) in (int, float) and
                         not (field in ("vertex_scanlines", "polygon_scanlines", "streamed_chunks") and r[field] == 0xffffffff)
                         for key in common for r in g[key]) for g in grouped]
        if not all(available):
            counters[field] = dict(available=False, before_available=available[0], after_available=available[1],
                                   median_group_delta=None, median_group_percent=None)
            continue
        pairs = [[median(r[field] for r in g[key]) for g in grouped] for key in common]
        counters[field] = dict(available=True, before_group_median=median(a for a, _ in pairs),
                               after_group_median=median(b for _, b in pairs),
                               median_group_delta=median(b-a for a, b in pairs),
                               median_group_percent=median(100*(b-a)/a for a, b in pairs if a) if any(a for a, _ in pairs) else None)
    return counters


def groups(rows, mode="legacy"):
    result = defaultdict(list)
    for row in rows:
        if mode == "camera":
            ordered = row.get("ordered_draw")
            if not ordered or "sample" not in ordered or any(k not in ordered for k in CAMERA_FIELDS):
                raise ValueError("Exact camera matching requires the latched ordered-frame layout")
            if not ordered["frame"] or ordered["frame"] != row["frame"] or ordered["sample"] == 0xffffffff:
                continue
            key = (ordered["sample"], tuple(ordered[k] for k in CAMERA_FIELDS), row["steps"],
                   ordered["candidates"], row["gte_vertices"]+row["software_vertices"],
                   row["triangles"], row["tested_chunks"])
        elif mode == "ordered":
            # A CPU -> GTE optimization deliberately changes their individual
            # counts. Match the unchanged input count and output slots instead,
            # so the affected work is not excluded from the comparison.
            key = (row["visibility_track"]["sample"], row["steps"],
                   row["ordered_draw"]["candidates"], row["gte_vertices"]+row["software_vertices"],
                   row["triangles"], row["tested_chunks"])
        else:
            key = (row["visibility_track"]["sample"], row["steps"], row["gte_vertices"],
                   row["triangles"], row["tested_chunks"])
        result[key].append(row)
    return result


def without_page_dictionary(header):
    prefixes = ("inline constexpr uint32_t track_page_ids[]",
                "inline constexpr VisibilityPageSet track_page_sets[]",
                "inline constexpr uint16_t track_page_samples[]")
    return "\n".join(line.replace(",track_page_sets,track_page_samples};", "};")
                     for line in header.splitlines() if not line.startswith(prefixes)) + "\n"


def compare(before, after, mode="legacy"):
    if mode == "camera" and (before / "display.hh").read_bytes() != (after / "display.hh").read_bytes():
        raise ValueError("Exact camera comparison requires identical display configuration")
    documents = [json.loads((p / "profile.json").read_text(encoding="utf-8")) for p in (before, after)]
    assert all(d["running_code_verified_bytes"] > 0 for d in documents)
    validation = [any(r.get("ordered_validation") is not None for r in d["rows"]) for d in documents]
    if validation[0] != validation[1]:
        raise ValueError("Cannot compare heavy GTE validation with a normal runtime")
    detail = [{r.get("streamed_chunks_available") for r in d["rows"]} for d in documents]
    if any(len(values)!=1 for values in detail) or detail[0]!=detail[1]:
        raise ValueError("Cannot compare different or inconsistent profiling detail modes")
    grouped = [groups(d["rows"], mode) for d in documents]
    common = sorted(grouped[0].keys() & grouped[1].keys())
    assert common, "No matching route/work groups"
    counters = counter_comparisons(grouped, common, FIELDS)
    headers = [(p / "scene.hh").read_text(encoding="utf-8") for p in (before, after)]
    archive = [(p / "GEOMETRY.BIN").read_bytes() for p in (before, after)]
    return {
        "scope": __doc__, "before": str(before.resolve()), "after": str(after.resolve()),
        "key_strategy": mode, "matching_groups": len(common), "group_keys": common, "counters": counters,
        "streamed_chunk_detail_available": next(iter(detail[0])),
        "same_archive_bytes": archive[0] == archive[1],
        "archive_sha256": [hashlib.sha256(b).hexdigest() for b in archive],
        "headers_equal_except_page_dictionary": without_page_dictionary(headers[0]) == without_page_dictionary(headers[1]),
        "descriptive_global_frame_us": [{k: d[k] for k in ("sampled_frames", "median_frame_us", "p95_frame_us", "max_frame_us")} for d in documents],
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--key-mode", choices=("legacy", "ordered", "camera"), default="legacy")
    args = parser.parse_args()
    result = compare(args.before, args.after, args.key_mode)
    if args.output:
        with args.output.open("x", encoding="utf-8") as stream:
            json.dump(result, stream, indent=2)
            stream.write("\n")
    print(json.dumps({k: v for k, v in result.items() if k != "group_keys"}, indent=2))
