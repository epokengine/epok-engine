"""Validate and summarize the read-only original-game observer, not host FPS.

Frame-finalization intervals include pacing/waits. World intervals include the
selected original scenery callbacks, but exclude gameplay and dynamic actors.
Presentation cadence is measured separately from actual display-origin changes.
No Epok speed ratio is inferred from unrelated cameras or aggregate percentiles.
"""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import statistics
import struct


CODE_RANGES = ((0x80016634, 0x80016CE4), (0x80040ED8, 0x80041C64),
               (0x8004CE30, 0x8004CEA0))


def digest(data):
    return hashlib.sha256(data).hexdigest()


def signed(value):
    return value if value < 0x80000000 else value - 0x100000000


def distribution(values):
    ordered = sorted(values)
    if not ordered:
        return None
    return dict(count=len(ordered), minimum=ordered[0], median=statistics.median(ordered),
                p95=ordered[math.ceil(.95*len(ordered))-1], maximum=ordered[-1])


def verify_code(exe, ram):
    if len(ram) != 2097152 or exe[:8] != b"PS-X EXE":
        raise ValueError("Expected PS-X EXE and 2 MiB RAM")
    base, length = struct.unpack_from("<II", exe, 0x18)
    verified = []
    for first, end in CODE_RANGES:
        if not base <= first < end <= base + length:
            raise ValueError("Observed code range outside executable")
        start = 0x800 + first - base
        expected = exe[start:start+end-first]
        actual = ram[first-0x80000000:end-0x80000000]
        if actual != expected:
            raise ValueError(f"Original observed code differs from EXE at range {first:08x}")
        verified.append(dict(first=f"{first:08x}", end_exclusive=f"{end:08x}",
                             bytes=len(expected), sha256=digest(expected)))
    return verified


def analyze(profile):
    if profile.get("version") not in (2,3) or profile.get("cpu_clock_hz") != 33868800:
        raise ValueError("Unsupported observer version or emulated CPU clock")
    if profile.get("overflow") or profile.get("callback_error"):
        raise ValueError("Incomplete/failed observer capture")
    rows = profile["rows"]
    if not rows:
        raise ValueError("No frame-finalization samples")
    for i, row in enumerate(rows):
        if len(row) != 48 or any(not isinstance(v, (int, float)) or not math.isfinite(v) for v in row):
            raise ValueError("Invalid observer row")
        if row[1] <= 0 or row[2] < 0 or row[3] < 0 or row[40] < 0 or row[47] < 0:
            raise ValueError("Invalid observed work")
        if i and (row[0]-rows[i-1][0] != row[1] or row[7] < rows[i-1][7]):
            raise ValueError("Non-contiguous capture or restored clock inside profile")
    clock = profile["cpu_clock_hz"]
    world = [row for row in rows if row[3]]
    # A loading gap before the first world frame is reported separately, not
    # silently removed from the all-frame distribution or the presentation log.
    steady = [row for i, row in enumerate(rows) if i and row[3] and rows[i-1][3]]
    transitions = [row for i, row in enumerate(rows) if row[3] and (not i or not rows[i-1][3])]
    presentations = profile.get("presentations", [])
    for i, row in enumerate(presentations):
        if len(row) != 3 or any(not isinstance(v, int) for v in row):
            raise ValueError("Invalid presentation row")
        if i and (row[0] <= presentations[i-1][0] or row[1] < presentations[i-1][1]):
            raise ValueError("Non-monotonic presentations")
    changed = [row for i, row in enumerate(presentations)
               if not i or row[2] != presentations[i-1][2]]
    phase_by_cycle = None
    if "presentation_phases" in profile:
        phases = profile["presentation_phases"]
        if len(phases) != len(presentations):
            raise ValueError("Incomplete presentation phases")
        phase_by_cycle = {}
        for p, phase in zip(presentations, phases):
            if (len(phase) != 2 or any(type(v) is not int for v in phase)
                    or phase[0] != p[0] or not -1 <= phase[1] <= phase[0]):
                raise ValueError("Invalid presentation phase pairing")
            phase_by_cycle[phase[0]] = phase[1]
    # Use world-run boundaries to avoid mixing the menu/loading pause with the
    # gameplay interval. No upper-duration outlier filter is applied.
    runs = []
    for i, row in enumerate(rows):
        if not row[3]:
            continue
        if not i or not rows[i-1][3]:
            runs.append([i, i])
        else:
            runs[-1][1] = i
    present_deltas, vsync_deltas, world_phases = [], [], []
    for begin, end in runs:
        selected = [row for row in changed if rows[begin][0] <= row[0] <= rows[end][0]]
        present_deltas.extend(b[0]-a[0] for a, b in zip(selected, selected[1:]))
        vsync_deltas.extend(b[1]-a[1] for a, b in zip(selected, selected[1:]))
        if phase_by_cycle is not None:
            world_phases.extend(phase_by_cycle[r[0]] for r in selected if phase_by_cycle[r[0]] >= 0)
    ms = lambda values: distribution([v*1000/clock for v in values])
    return dict(scope=__doc__, observer_version=profile["version"], cpu_clock_hz=clock,
                rows=len(rows), world_rows=len(world), world_runs=runs,
                all_finalization_interval_ms=ms([r[1] for r in rows]),
                continuous_world_finalization_interval_ms=ms([r[1] for r in steady]),
                world_entry_transition_interval_ms=ms([r[1] for r in transitions]),
                scenery_pass_ms=ms([r[2] for r in world]),
                slst_header_count=(distribution([r[40] for r in world]) if profile["version"]==3 else None),
                legacy_slst_count_minus_one=(distribution([r[40] for r in world]) if profile["version"]==2 else None),
                scenery_packet_bytes=distribution([r[47] for r in world]),
                scenery_callbacks=dict(Counter(f"{r[44]:08x}" for r in world)),
                presentation_records=len(presentations), display_origin_changes=max(0,len(changed)-1),
                all_presentation_change_interval_ms=ms([b[0]-a[0] for a,b in zip(changed,changed[1:])]),
                continuous_world_presentation_interval_ms=ms(present_deltas),
                continuous_world_presentation_vsyncs=dict(sorted(Counter(vsync_deltas).items())),
                presentation_phase_available=phase_by_cycle is not None,
                continuous_world_presentation_phase_since_vsync_ms=ms(world_phases),
                continuous_world_presentation_rate=(len(present_deltas)*clock/sum(present_deltas)
                                                     if present_deltas else None),
                caveat="Original demo only, not complete playable-level coverage, original hardware, exact camera/work matching, or Epok speedup.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profile", type=Path)
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--ram", type=Path, nargs="+", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("Refusing to overwrite evidence")
    inputs = {p: p.read_bytes() for p in [args.profile, args.exe, *args.ram]}
    report = analyze(json.loads(inputs[args.profile]))
    report["input_sha256"] = {str(p):digest(data) for p,data in inputs.items()}
    report["verified_observed_code"] = {str(p):verify_code(inputs[args.exe], inputs[p]) for p in args.ram}
    with args.output.open("x", encoding="utf-8") as target:
        json.dump(report, target, indent=2)
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
