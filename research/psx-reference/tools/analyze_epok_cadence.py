"""Validate every-frame Epok CPU/presentation evidence, never host or UI FPS.

GP1(05h) observations are display-origin change requests, not a raster-complete
frame/tearing oracle. Camera waits stay in the distributions. Resource warmup
precedes the first frame (its cumulative stalls remain visible); unlatched frame
publications are separated without trimming long ordered-frame intervals.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

from analyze_original_profile import distribution
from sample_epok_runtime import executable_sections, verify_running_code


def unpack_rows(profile, config):
    if profile.get("version") != 1 or profile.get("cpu_clock_hz") != 33868800:
        raise ValueError("Unsupported observer version/clock")
    if profile.get("overflow") or profile.get("callback_error"):
        raise ValueError("Incomplete/failed observer capture")
    result = []
    for raw in profile["rows"]:
        if len(raw) != config["row_words"] or any(type(v) is not int for v in raw):
            raise ValueError("Invalid completed-frame row")
        row = dict(cycles=raw[0], vsyncs=raw[1], cpu_cycles=raw[2])
        for block in config["blocks"]:
            begin, fields = block["offset"], block["fields"]
            row[block["name"]] = dict(zip(fields, raw[begin:begin + len(fields)]))
        if row["cycles"] < 0 or row["vsyncs"] < 0 or row["cpu_cycles"] < -1:
            raise ValueError("Invalid completed-frame timing")
        if result and (row["cycles"] <= result[-1]["cycles"] or row["vsyncs"] < result[-1]["vsyncs"]
                       or row["performance"]["frame"] != result[-1]["performance"]["frame"] + 1):
            raise ValueError("Non-contiguous frame capture or restored clock")
        result.append(row)
    if not result:
        raise ValueError("No completed frames")
    return result


def analyze(profile, config):
    rows = unpack_rows(profile, config)
    clock = profile["cpu_clock_hz"]
    by_frame = {r["performance"]["frame"]: r for r in rows}
    ms = lambda values: distribution([v * 1000 / clock for v in values])
    presentations = profile["presentations"]
    for i, p in enumerate(presentations):
        if len(p) != 6 or any(type(v) is not int for v in p) or p[2] not in (0x05000000, 0x05040000):
            raise ValueError("Invalid GP1 presentation")
        if i and (p[0] <= presentations[i - 1][0] or p[1] < presentations[i - 1][1]
                  or p[3] != presentations[i - 1][3] + 1 or p[2] == presentations[i - 1][2]):
            raise ValueError("Non-contiguous/non-alternating presentation stream")
        row = by_frame.get(p[3])
        if row is not None and (p[0] < row["cycles"] or p[0] - row["cycles"] != p[4]):
            raise ValueError("GP1 request not paired with its GPU::flip entry")
    def completed(row):
        return row["ordered"]["frame"] == row["performance"]["frame"] and row["ordered"]["frame"] > 0
    gameplay = [r for r in rows if completed(r)]
    pairs = [(a, b) for a, b in zip(presentations, presentations[1:])
             if a[3] in by_frame and b[3] in by_frame and completed(by_frame[a[3]]) and completed(by_frame[b[3]])]
    intervals = [b[0] - a[0] for a, b in pairs]
    vsync_intervals = [b[1] - a[1] for a, b in pairs]
    waits, moving = [], []
    for a, b in pairs:
        before, after = by_frame[a[3]], by_frame[b[3]]
        (waits if after["visibility"]["waiting_frames"] > before["visibility"]["waiting_frames"] else moving).append(b[0] - a[0])
    all_intervals = [b[0] - a[0] for a, b in zip(presentations, presentations[1:])]
    present_frames = [p for p in presentations if p[3] in by_frame and completed(by_frame[p[3]])]
    slow = [(a, b, by_frame[b[3]]) for a, b in pairs if b[0] - a[0] > clock / 30]
    slow_costs = dict(
        intervals=len(slow),
        current_cpu_work_over_30hz=sum(r["cpu_cycles"] > clock / 30 for _, _, r in slow),
        flip_wait_over_1ms=sum(b[4] > clock / 1000 for _, b, _ in slow),
        verification_counter_advanced=sum(r["verify"]["batches"] > by_frame[a[3]]["verify"]["batches"] for a, _, r in slow),
        camera_wait_counter_advanced=sum(r["visibility"]["waiting_frames"] > by_frame[a[3]]["visibility"]["waiting_frames"] for a, _, r in slow),
        current_cpu_work_ms=ms([r["cpu_cycles"] for _, _, r in slow]),
        flip_wait_ms=ms([b[4] for _, b, _ in slow]),
        other_interval_work_ms=ms([b[0] - a[0] - r["cpu_cycles"] - b[4] for a, b, r in slow]),
        current_frame_scanline_counters={field: distribution([r["performance"][field] for _, _, r in slow if field in r["performance"]])
                                        for field in ("render_scanlines", "simulation_scanlines", "prepare_scanlines", "finish_scanlines")},
        note="Verification/wait changes are correlations, not exclusive causal costs. Other work includes the previous flip tail and application dispatch. Scanline counters retain native units (~64 us), not CPU-cycle precision.")
    counters = {}
    counter_fields = [
        ("visibility", ["advances", "waiting_frames", "loops"]),
        ("streaming", ["reads", "bytes", "stalls", "stall_us", "errors", "timeouts", "gameplay_requests", "gameplay_rejected"]),
        ("verify", ["batches", "bytes", "scanlines"]),
        ("time", ["ticks", "dropped_steps"]),
    ]
    if "overlap" in rows[0]:
        counter_fields.append(("overlap", ["bytes"]))
        if any(row["overlap"]["bytes"] > row["verify"]["bytes"] for row in rows):
            raise ValueError("Overlap exceeds verified bytes")
    for block, fields in counter_fields:
        if any(b[block][f] < a[block][f] for a, b in zip(rows, rows[1:]) for f in fields):
            raise ValueError(f"Cumulative counter reset: {block}")
        counters[block] = dict(first={f: rows[0][block][f] for f in fields},
                              last={f: rows[-1][block][f] for f in fields},
                              delta={f: rows[-1][block][f] - rows[0][block][f] for f in fields})
    # Consecutive publications delimit cumulative checksum work. Group equal
    # byte/batch deltas: never divide a mixed interval and call it a slice.
    verification_groups = {}
    for before, after in zip(rows, rows[1:]):
        batches = after["verify"]["batches"] - before["verify"]["batches"]
        byte_count = after["verify"]["bytes"] - before["verify"]["bytes"]
        ticks = after["verify"]["scanlines"] - before["verify"]["scanlines"]
        if batches:
            verification_groups.setdefault((batches, byte_count), []).append(ticks)
    verification = [dict(batches_per_interval=k[0], bytes_per_interval=k[1],
                         scanline_ticks=distribution(v)) for k, v in sorted(verification_groups.items())]
    return dict(scope=__doc__, cpu_clock_hz=clock, frames=len(rows), completed_ordered_frames=len(gameplay),
                first_frame=rows[0]["performance"]["frame"], last_frame=rows[-1]["performance"]["frame"],
                presentation_requests=len(presentations), paired_presentation_intervals=len(pairs),
                all_display_origin_interval_ms=ms(all_intervals),
                completed_ordered_display_origin_interval_ms=ms(intervals),
                completed_ordered_display_origin_rate=len(intervals) * clock / sum(intervals) if intervals else None,
                completed_ordered_display_origin_vsyncs=dict(sorted(Counter(vsync_intervals).items())),
                completed_ordered_intervals_over_30hz=sum(v > clock / 30 for v in intervals),
                frame_cpu_work_ms=ms([r["cpu_cycles"] for r in gameplay if r["cpu_cycles"] >= 0]),
                flip_entry_to_gp1_ms=ms([p[4] for p in present_frames if p[4] >= 0]),
                gp1_phase_since_vsync_ms=ms([p[5] for p in present_frames if p[5] >= 0]),
                camera_wait_interval_ms=ms(waits), camera_non_wait_interval_ms=ms(moving),
                intervals_over_30hz_costs=slow_costs,
                non_latched_frames=len(rows) - len(gameplay), counters=counters,
                verification_between_frames=verification,
                profile_detail_modes=dict(Counter(str(r["performance"]["streamed_chunks"] != 0xffffffff) for r in gameplay)),
                maximum_failures={f"{block}.{field}": max(r[block][field] for r in rows) for block, field in (
                    ("lighting", "dropped_triangles"), ("performance", "stream_failed_chunks"),
                    ("performance", "stream_budget_skipped_chunks"), ("performance", "gte_validation_errors"),
                    ("ordered", "anchor_rejections"), ("texture", "failures"), ("texture", "error"))},
                caveat="Emulated interpreter/SoftGPU, captured study workload, not hardware or exact original-demo workload. GP1 changes are not proof of raster-complete presentation or absence of tearing.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("Refusing to overwrite evidence")
    folder = args.capture
    paths = [folder / name for name in ("profile.json", "observer-config.json", "epok.elf", "before.bin", "after.bin")]
    inputs = {p.name: p.read_bytes() for p in paths}
    report = analyze(json.loads(inputs["profile.json"]), json.loads(inputs["observer-config.json"]))
    sections = executable_sections(inputs["epok.elf"])
    for name in ("before.bin", "after.bin"):
        if len(inputs[name]) != 0x200000:
            raise ValueError("Expected 2 MiB RAM")
        verify_running_code(inputs[name], sections)
    report["running_code_verified_bytes"] = sum(len(code) for _, code in sections)
    report["input_sha256"] = {name: hashlib.sha256(data).hexdigest() for name, data in inputs.items()}
    with args.output.open("x", encoding="utf-8") as target:
        json.dump(report, target, indent=2)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
