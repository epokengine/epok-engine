"""Compare CPU work at exact camera/work identities from every-frame observers.

No aggregate FPS ratio: captures may have different CD scheduling and route
durations. Empty ordered passes are excluded from exact-view comparisons.
"""
import argparse
import hashlib
import json
from pathlib import Path

from analyze_epok_cadence import analyze, unpack_rows
from capture_epok_cadence import elf_symbols
from compare_epok_profiles import CAMERA_FIELDS, FIELDS, groups, counter_comparisons
from sample_epok_runtime import executable_sections, verify_running_code


def normalized_rows(profile, config):
    return [dict(row["performance"], cpu_cycles=row["cpu_cycles"],
                 ordered_draw=row["ordered"], visibility_track=row["visibility"])
            for row in unpack_rows(profile, config) if row["cpu_cycles"] >= 0
            and row["ordered"]["frame"] > 0
            and row["ordered"]["frame"] == row["performance"]["frame"]]


def compare_rows(before, after):
    modes = [{r["streamed_chunks"] != 0xffffffff for r in rows} for rows in (before, after)]
    if len(modes[0]) != 1 or modes[0] != modes[1]:
        raise ValueError("Different/inconsistent profiling detail modes")
    grouped = [groups(rows, "camera") for rows in (before, after)]
    common = sorted(grouped[0].keys() & grouped[1].keys())
    if not common:
        raise ValueError("No exact camera/work groups")
    counters = counter_comparisons(grouped, common, ["cpu_cycles", *FIELDS])
    return dict(matching_groups=len(common), group_keys=common, counters=counters,
                detail_enabled=next(iter(modes[0])),
                matched_records=[sum(len(g[key]) for key in common) for g in grouped])


def load(folder):
    files = {name: (folder / name).read_bytes() for name in
             ("profile.json", "observer-config.json", "epok.elf", "before.bin", "after.bin", "display.hh", "build-summary.json")}
    elf = files["epok.elf"]
    sections = executable_sections(elf)
    for name in ("before.bin", "after.bin"):
        if len(files[name]) != 0x200000: raise ValueError("Expected complete 2 MiB RAM")
        verify_running_code(files[name], sections)
    profile, config = json.loads(files["profile.json"]), json.loads(files["observer-config.json"])
    summary = analyze(profile, config)
    manifest = json.loads(files["build-summary.json"])
    if manifest["inputs"]["epok.elf"] != hashlib.sha256(elf).hexdigest():
        raise ValueError("Stale linked build manifest")
    validation = any("ordered_validation_stats" in name for name in elf_symbols(elf))
    return dict(rows=normalized_rows(profile, config), summary=summary, manifest=manifest,
                validation=validation, display=files["display.hh"],
                input_sha256={name: hashlib.sha256(data).hexdigest() for name, data in files.items()})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists(): parser.error("Refusing to overwrite evidence")
    before, after = load(args.before), load(args.after)
    if before["display"] != after["display"]: raise ValueError("Different display configuration")
    if before["validation"] != after["validation"]: raise ValueError("Different heavy GTE validation modes")
    if before["manifest"]["profile"] != after["manifest"]["profile"]: raise ValueError("Different build profiles")
    archives = [v["manifest"]["inputs"]["GEOMETRY.BIN"] for v in (before, after)]
    if archives[0] != archives[1]: raise ValueError("Different geometry archive hashes")
    report = dict(scope=__doc__, **compare_rows(before["rows"], after["rows"]),
                  heavy_validation_linked=before["validation"], archive_sha256=archives[0],
                  input_sha256=[v["input_sha256"] for v in (before, after)])
    with args.output.open("x", encoding="utf-8") as output: json.dump(report, output, indent=2)
    print(json.dumps({k:v for k,v in report.items() if k != "group_keys"}, indent=2))


if __name__ == "__main__": main()
