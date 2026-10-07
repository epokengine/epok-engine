"""Read counters/VRAM from an already running Play PSX session; never launches it."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import statistics
import struct
import sys
import time
import urllib.request


def executable_sections(elf):
    """ELF32 MIPS instruction ranges; data/BSS are expected to change at runtime."""
    assert elf[:7] == b"\x7fELF\x01\x01\x01" and struct.unpack_from("<H", elf, 18)[0] == 8
    offset = struct.unpack_from("<I", elf, 32)[0]
    stride, count = struct.unpack_from("<HH", elf, 46)
    assert stride == 40 and offset + stride * count <= len(elf)
    sections = []
    for i in range(count):
        _, kind, flags, address, start, size, *_ = struct.unpack_from("<10I", elf, offset + stride * i)
        if kind == 1 and flags & 4 and size:
            assert start + size <= len(elf)
            sections.append((address & 0x1fffffff, elf[start:start + size]))
    assert sections, "No instruction sections in linked ELF"
    return sections


def verify_running_code(ram, sections):
    for address, code in sections:
        assert address + len(code) <= len(ram), f"Instruction section outside PSX RAM: {address:#x}"
        assert ram[address:address + len(code)] == code, f"Running PSX code does not match linked ELF at {address:#x}"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, default=Path("D:/GitProjects/GameEngines/Epok/EpokEngine"))
    parser.add_argument("--project", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seconds", type=float, default=30)
    args = parser.parse_args()
    if not 1 <= args.seconds <= 600:
        parser.error("seconds must be 1..600")
    if args.output.exists() and any(args.output.iterdir()):
        parser.error("output must be new or empty")
    sys.path.insert(0, str(args.engine / "tools"))
    import profile_runtime as profile
    args.output.mkdir(parents=True, exist_ok=True)
    build = args.project / ".epok/build"
    snapshot = {name: (build / name).read_bytes() for name in
                ("epok.map", "epok.elf", "epok.ps-exe", "scene.hh", "display.hh", "memory-report.json")}
    if (build / "GEOMETRY.BIN").is_file():
        snapshot["GEOMETRY.BIN"] = (build / "GEOMETRY.BIN").read_bytes()
    if (build / "scene-stream-bake.json").is_file():
        snapshot["scene-stream-bake.json"] = (build / "scene-stream-bake.json").read_bytes()
    # Preserve the implementation behind experimental renderer measurements,
    # not only its executable. Later edits must not erase the tested sources.
    for name in ("main.cpp", "ordered_renderer.hpp", "ordered_projection.hpp",
                 "ordered_draw.hpp", "ordered_draw_types.hpp", "baked_packet.hpp", "baked_draw_types.hpp",
                 "gte_geometry.hpp", "fixed_trigonometry.hpp", "retained.hpp", "streaming.hpp", "streaming_pool.hpp", "streaming_read.hpp",
                 "visibility_track.hpp", "texture_types.hpp", "texture.hpp"):
        if (build / name).is_file():
            snapshot[name] = (build / name).read_bytes()
    hashes = {name: hashlib.sha256(data).hexdigest() for name, data in snapshot.items()}
    assert json.loads(snapshot["memory-report.json"])["executable_hash"] == hashes["epok.ps-exe"], "Memory report is stale"
    code = executable_sections(snapshot["epok.elf"])
    symbols = snapshot["epok.map"].decode()
    addresses = {name: profile.symbol_address(symbols, "epok::" + name) for name in
                 ("performance_stats", "time", "lighting_stats", "streaming_stats", "streaming_warmup_stats", "texture_load_stats")}
    assert all(addresses[k] is not None for k in ("performance_stats", "time", "lighting_stats"))
    section = re.search(r"\.bss\._ZN4epok17performance_statsE\s+0x[0-9a-fA-F]+\s+0x([0-9a-fA-F]+)", symbols)
    extras = min(len(profile.EXTRA_FIELDS), max(0, int(section[1], 16) // 4 - len(profile.FIELDS))) if section else 0
    fields = profile.FIELDS + profile.EXTRA_FIELDS[:extras]
    playback = profile.playback_addresses(symbols)
    configuration = next(p for p in (args.project / "Local.epokconfig", args.engine / "Local.epokconfig", args.engine / "Editor.epokconfig") if p.exists())
    port = profile.documents.loads(configuration.read_text())["web_port"]
    def read(resource):
        with urllib.request.urlopen(f"http://127.0.0.1:{port}/api/v1/{resource}", timeout=3) as response:
            return response.read()
    frames = {}
    rejected_publications = 0
    verify_running_code(read("cpu/ram/raw"), code)
    end = time.monotonic() + args.seconds
    while time.monotonic() < end:
        ram = read("cpu/ram/raw")
        row = profile.read_counters(ram, addresses["performance_stats"], fields)
        if row["frame"] > 5:
            row["frame_microseconds"] = struct.unpack_from("<I", ram, addresses["time"] + 24)[0]
            row["dropped_steps"] = struct.unpack_from("<I", ram, addresses["time"] + 20)[0]
            row["dropped_triangles"] = struct.unpack_from("<I", ram, addresses["lighting_stats"] + 20)[0]
            profile.normalize_streamed_chunks(row)
            for key, address, names in [("streaming", addresses["streaming_stats"], profile.STREAMING_FIELDS),
                                         ("streaming_warmup", addresses["streaming_warmup_stats"], profile.WARMUP_FIELDS),
                                         ("texture_load", addresses["texture_load_stats"], ["attempts", "uploads", "bytes", "failures", "error"])]:
                row[key] = profile.read_counters(ram, address, names)
            for key, (_, names, _) in profile.PLAYBACK_STATS.items():
                row[key] = profile.read_counters(ram, playback[key], names)
            if profile.latch_ordered_frame(row):
                frames[row["frame"]] = row
            else:
                rejected_publications += 1
        time.sleep(0.1)
    assert len(frames) >= 5, f"Insufficient advancing frames: {len(frames)}"
    verify_running_code(read("cpu/ram/raw"), code)
    for name, data in snapshot.items():
        assert (build / name).read_bytes() == data, f"Build changed during capture: {name}"
        (args.output / name).write_bytes(data)
    (args.output / "vram.bin").write_bytes(read("gpu/vram/raw"))
    rows = list(frames.values())
    timings = [row["frame_microseconds"] for row in rows]
    result = {"source": "Attached to existing Play PSX; no launch, input or scene mutation",
              "build_sha256": hashes, "running_code_verified_bytes": sum(len(data) for _, data in code),
              "project": str(args.project.resolve()), "sampled_frames": len(rows),
              "rejected_mid_publication_snapshots": rejected_publications,
              "median_frame_us": statistics.median(timings), "p95_frame_us": sorted(timings)[int((len(timings)-1)*0.95)],
              "max_frame_us": max(timings), "medians": profile.frame_medians(rows),
              "streaming": profile.counter_summary(rows, "streaming", profile.STREAMING_FIELDS),
              "rows": rows}
    (args.output / "profile.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps({k: v for k, v in result.items() if k != "rows"}, indent=2))


if __name__ == "__main__":
    main()
