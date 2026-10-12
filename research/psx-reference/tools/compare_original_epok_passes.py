"""Compare the original scenery pass with Epok's ordered pass on identical lists.

The original side is an original_observer.lua profile with world_passes and
world_lists: every scenery pass with its cycles and the complete ordered token
list it drew. Each list is matched word for word, in order, against the source
route export; position or list size alone is never treated as a match. The
matched (route, sample) is translated to Epok's concatenated sample index.

The Epok side is a capture_epok_cadence.py --function-costs capture: each
ordered-render call is attributed to its completed frame and that frame's
published ordered sample. Only samples present on both sides are compared.

Epok's ordered samples omit the source list's entries for sky worlds and for
worlds outside the sample's world mask, so a sample may draw fewer candidates
than the original list holds; both counts and per-entry costs are reported.
Epok drawing MORE candidates than the original list is an error.

Both captures must come from the same emulator build. This compares the same
visibility list, not identical cameras, pixels or viewport: the original pass
also runs its setup callbacks, Epok's includes CPU clipping, and the original
draw area is shorter. It is a matched-workload cost comparison, not a frame
rate claim.
"""
import argparse
import bisect
import json
from pathlib import Path
import statistics

from analyze_epok_cadence import unpack_rows


def original_passes(profile, routes):
    if routes["coordinate_scale"] != .00025:
        raise ValueError("Expected the native /4000 source route export")
    lists = profile.get("world_lists")
    passes = profile.get("world_passes")
    if lists is None or passes is None:
        raise ValueError("Original profile lacks world_passes/world_lists")
    by_tokens, first = {}, 0
    for route_index, route in enumerate(routes["routes"]):
        for sample_index, sample in enumerate(route["Samples"]):
            by_tokens.setdefault(tuple(sample["Polygons"]), []).append(
                dict(route=route_index, sample=sample_index, epok_sample=first + sample_index))
        first += len(route["Samples"])
    matched = {}
    unmatched = 0
    for start, cycles, count, list_id, packet_bytes, *_ in passes:
        if not list_id:
            continue
        tokens = tuple(lists[list_id - 1])
        if len(tokens) != count:
            raise ValueError("Pass count differs from its stored list")
        candidates = by_tokens.get(tokens, [])
        if len(candidates) != 1:
            unmatched += 1
            continue
        key = candidates[0]["epok_sample"]
        entry = matched.setdefault(key, dict(candidates[0], entries=count, cycles=[], packet_bytes=[]))
        entry["cycles"].append(cycles)
        entry["packet_bytes"].append(packet_bytes)
    return matched, unmatched, len(passes)


def epok_passes(folder):
    profile = json.loads((folder / "profile.json").read_text())
    config = json.loads((folder / "observer-config.json").read_text())
    hooks = config.get("function_costs") or []
    index = next((i + 1 for i, h in enumerate(hooks) if h["label"] == "ordered_render"), None)
    if index is None:
        raise ValueError("Epok capture lacks ordered_render function costs")
    rows = unpack_rows(profile, config)
    ends = [r["cycles"] for r in rows]
    by_sample = {}
    for call, start, end, _, _ in profile["function_costs"]["calls"]:
        if call != index:
            continue
        frame = bisect.bisect_left(ends, end)
        if frame >= len(rows):
            continue
        row = rows[frame]
        ordered = row["ordered"]
        if row["cpu_cycles"] < 0 or ordered["frame"] != row["performance"]["frame"] \
                or start < ends[frame] - row["cpu_cycles"]:
            continue
        entry = by_sample.setdefault(ordered["sample"], dict(candidates=ordered["candidates"],
                                                             packets=ordered["packets"], cycles=[]))
        entry["cycles"].append(end - start)
    return by_sample, profile["cpu_clock_hz"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("original", type=Path, help="original_observer.lua profile JSON")
    parser.add_argument("routes", type=Path, help="source route export JSON")
    parser.add_argument("epok", type=Path, help="capture_epok_cadence.py --function-costs folder")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("Refusing to overwrite comparison output")
    original = json.loads(args.original.read_text())
    if original.get("overflow") or original.get("callback_error"):
        raise SystemExit("Incomplete original profile")
    clock = original["cpu_clock_hz"]
    matched, unmatched, total = original_passes(original, json.loads(args.routes.read_text()))
    epok, epok_clock = epok_passes(args.epok)
    if epok_clock != clock:
        raise SystemExit("Captures declare different CPU clocks")
    rows = []
    for sample in sorted(set(matched) & set(epok)):
        o, e = matched[sample], epok[sample]
        if e["candidates"] > o["entries"]:
            raise SystemExit(f"Sample {sample}: Epok drew {e['candidates']} candidates, original list has {o['entries']}")
        om, em = statistics.median(o["cycles"]), statistics.median(e["cycles"])
        rows.append(dict(epok_sample=sample, route=o["route"], route_sample=o["sample"], entries=o["entries"],
                         epok_candidates=e["candidates"], omitted_entries=o["entries"] - e["candidates"],
                         original_passes=len(o["cycles"]), epok_passes=len(e["cycles"]),
                         original_median_ms=om * 1000 / clock, epok_median_ms=em * 1000 / clock,
                         original_cycles_per_entry=om / o["entries"], epok_cycles_per_entry=em / e["candidates"],
                         epok_over_original=em / om, original_packet_bytes=statistics.median(o["packet_bytes"]),
                         epok_packets=e["packets"]))
    ratios = [r["epok_over_original"] for r in rows]
    report = dict(scope=__doc__, original_passes=total, original_unmatched_passes=unmatched,
                  original_samples=len(matched), epok_samples=len(epok), common_samples=len(rows),
                  median_epok_over_original=statistics.median(ratios) if ratios else None,
                  min_epok_over_original=min(ratios) if ratios else None,
                  max_epok_over_original=max(ratios) if ratios else None,
                  samples=rows)
    args.output.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps({k: v for k, v in report.items() if k not in ("scope", "samples")}))


if __name__ == "__main__":
    main()
