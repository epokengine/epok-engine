"""Profile Forest's existing native test-command route without editing the game.

The supplied project must already expose forest_probe/forest_test_command/
forest_test_report. Each phase uses the tick target from its verify_forest.py.
Polling can overshoot targets: compare matching phases and inspect actual ticks,
positions and visibility before treating timing differences as regressions.
"""
import epok_documents as documents
from profile_runtime import EDITOR
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import socket
import statistics
import struct
import subprocess
import time
import urllib.request
import zlib
from project_layout import project_manifest

from profile_runtime import (ROOT, FIELDS, EXTRA_FIELDS, STREAMING_FIELDS,
                             WARMUP_FIELDS, counter_summary, read_counters,
                             symbol_address, normalize_streamed_chunks, frame_medians)

PHASES = [("01-idle", 0, 12), ("02-walk-right", 1, 40),
          ("03-idle-right", 0, 12), ("04-run-left", 2 | 16, 40),
          ("05-run-diagonal", 1 | 4 | 16, 30), ("06-cliff-approach", 4 | 16, 110),
          ("07-cliff-collision", 4 | 16, 35), ("08-idle-back", 0, 12)]
LIMITATION = ("Tick targets match verify_forest.py, but HTTP polling and pause acknowledgement can overshoot. "
              "Actual ticks/positions are recorded; routes and terminal VRAM need not be identical across runs. "
              "Compare equivalent phases, actual trajectory and visibility, not aggregate FPS alone. "
              "PerformanceStats describes a completed frame while live clock/probe fields may already describe the next frame.")


def save(path, data):
    documents.write_text(path, json.dumps(data, indent=2) + "\n", encoding="utf-8")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None


def signed_q12(value):
    return (value if value < 0x80000000 else value - 0x100000000) / 4096


def decode_frame(ram, addresses, extra_count):
    values = struct.unpack_from(f"<{len(FIELDS)}I", ram, addresses["performance"])
    row = dict(zip(FIELDS, values))
    row["frame_microseconds"] = struct.unpack_from("<I", ram, addresses["time"] + 24)[0]
    row["dropped_steps"] = struct.unpack_from("<I", ram, addresses["time"] + 20)[0]
    row["dropped_triangles"] = struct.unpack_from("<6I", ram, addresses["lighting"])[5]
    if extra_count:
        row.update(zip(EXTRA_FIELDS[:extra_count], struct.unpack_from(
            f"<{extra_count}I", ram, addresses["performance"] + 4 * len(FIELDS))))
    normalize_streamed_chunks(row)
    for key, fields in (("streaming", STREAMING_FIELDS), ("streaming_warmup", WARMUP_FIELDS)):
        counters = read_counters(ram, addresses[key], fields)
        if counters is not None:
            row[key] = counters
    row["present_wait_estimate_us"] = max(0, row["frame_microseconds"] - row["frame_scanlines"] * 64)
    return row


def profile(rows, build, observation=None):
    rows = sorted(rows, key=lambda row: row["frame"])
    for row in rows:
        normalize_streamed_chunks(row)
    medians = frame_medians(rows)
    distributions = {}
    for field in ("frame_scanlines", "frame_microseconds", "present_wait_estimate_us"):
        values = sorted(row[field] for row in rows)
        if values:
            distributions[field] = {"min": values[0], "p95": values[math.ceil(len(values) * .95) - 1], "max": values[-1]}
    return dict(samples=len(rows), gte_validation=build.get("gte_validation", False),
                optimization_override=build.get("optimization_override"),
                detail_timers=build.get("detail_timers", False), timer_unit_us_approx=64, build=build,
                median=medians, distributions=distributions, frames=rows,
                streaming=counter_summary(rows, "streaming", STREAMING_FIELDS),
                streaming_warmup=counter_summary(rows, "streaming_warmup", WARMUP_FIELDS),
                streamed_chunks_available=bool(rows) and all(row["streamed_chunks_available"] for row in rows),
                route_phase=observation, comparison_limitation=LIMITATION)


def save_png(raw, path, width, height):
    if not (0 < width <= 1024 and 0 < height <= 512) or len(raw) < 1024 * 512 * 2:
        raise ValueError("Invalid PSX VRAM/display dimensions")
    pixels = bytearray()
    for y in range(height):
        pixels.append(0)  # PNG filter: none.
        for x in range(width):
            p = struct.unpack_from("<H", raw, (y * 1024 + x) * 2)[0]
            pixels.extend(((p & 31) * 255 // 31, ((p >> 5) & 31) * 255 // 31, ((p >> 10) & 31) * 255 // 31))
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">2I5B", width, height, 8, 2, 0, 0, 0)) +
                     chunk(b"IDAT", zlib.compress(pixels)) + chunk(b"IEND", b""))


def verify_gameplay(observations, samples):
    idle, walk, rest, run, diag, _, blocked, _ = observations
    w = abs(walk["to_position"][0] - walk["from_position"][0]) / walk["simulation_seconds"]
    r = abs(run["to_position"][0] - run["from_position"][0]) / run["simulation_seconds"]
    d = math.hypot(diag["to_position"][0] - diag["from_position"][0],
                   diag["to_position"][2] - diag["from_position"][2]) / diag["simulation_seconds"]
    assert idle["probe"][2] == 3 and rest["probe"][2] == 5, "Idle direction clips changed"
    assert walk["probe"][2] == 2 and run["probe"][2] == 8 and diag["probe"][2] == 8, "Movement clips changed"
    assert 1.75 < r / w < 1.9, ("Run/walk ratio", r, w)
    assert abs(d / r - 1) < .04, ("Diagonal normalization", d, r)
    assert abs(rest["to_position"][0] - rest["from_position"][0]) < .001, "Idle drift"
    assert abs(blocked["to_position"][2] - blocked["from_position"][2]) < .002, "Cliff collision failed"
    assert 5.5 < blocked["to_position"][2] < 8.8, blocked
    assert all(abs(signed_q12(p[5])) < .005 and p[9] == 0 for p in samples), "Height or dropped triangles changed"
    return dict(run_walk_speed_ratio=r / w, diagonal_speed_ratio=d / r,
                max_dropped_triangles=max(p[9] for p in samples))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--editor", type=Path, default=EDITOR,
                        help="Exact editor/exporter binary, including a separately built baseline")
    parser.add_argument("--seconds", type=int, default=110, help="Owned editor/emulator stop timer, in host seconds")
    parser.add_argument("--poll-seconds", type=float, default=.01)
    parser.add_argument("--phase-timeout", type=float, default=45)
    args = parser.parse_args()
    if args.seconds < 20 or not 0 < args.poll_seconds <= .1 or args.phase_timeout <= 0:
        parser.error("Use seconds>=20, 0<poll-seconds<=0.1 and phase-timeout>0")
    if args.output.exists() and any(args.output.iterdir()):
        parser.error("Output must be new or empty; measurements are never overwritten")
    args.output.mkdir(parents=True, exist_ok=True)
    project = args.project.resolve()
    editor = args.editor.resolve()
    if not editor.is_file():
        parser.error(f"Editor binary does not exist: {editor}")
    config_path = next(path for path in (project / "Local.epokconfig", ROOT / "Local.epokconfig", ROOT / "Editor.epokconfig") if path.is_file())
    config = documents.loads(config_path.read_text(encoding="utf-8-sig"))
    port = int(config["web_port"])
    with socket.socket() as available:
        available.bind(("127.0.0.1", port))
    flags = getattr(subprocess, "CREATE_NO_WINDOW", 0)
    build_result = subprocess.run([str(editor), "--project", str(project), "--build-psx"],
                                  capture_output=True, text=True, creationflags=flags, timeout=240)
    documents.write_text(args.output / "build.log", build_result.stdout + build_result.stderr, encoding="utf-8")
    assert build_result.returncode == 0, build_result.stdout + build_result.stderr
    build_path = project / ".epok/build"
    symbols = (build_path / "epok.map").read_text()
    names = dict(performance="epok::performance_stats", time="epok::time", lighting="epok::lighting_stats",
                 streaming="epok::streaming_stats", streaming_warmup="epok::streaming_warmup_stats",
                 probe="forest_probe", command="forest_test_command", report="forest_test_report")
    addresses = {key: symbol_address(symbols, name) for key, name in names.items()}
    for key in ("performance", "time", "lighting", "probe", "command", "report"):
        assert addresses[key] is not None, f"Project lacks native telemetry symbol: {names[key]}"
    size_match = re.search(r"\.bss\._ZN5epok17performance_statsE\s+0x[0-9a-fA-F]+\s+0x([0-9a-fA-F]+)", symbols)
    extra_count = max(0, min(len(EXTRA_FIELDS), int(size_match[1], 16) // 4 - len(FIELDS))) if size_match else 0
    display = (build_path / "display.hh").read_text()
    width, height = [int(re.search(name + r"\s*=\s*(\d+)", display)[1]) for name in ("display_width", "display_height")]
    manifest_path = project_manifest(project)
    runtime_opt = os.environ.get("EPOK_RUNTIME_OPT")
    build = dict(project=str(project), config=config, config_path=str(config_path.resolve()),
                 editor=str(editor), editor_sha256=digest(editor),
                 optimization_override=runtime_opt.removeprefix("-") if runtime_opt else None,
                 gte_validation=os.environ.get("EPOK_VALIDATE_GTE") == "1",
                 detail_timers=os.environ.get("EPOK_PROFILE_DETAIL") == "1",
                 executable_sha256=digest(build_path / "epok.ps-exe"),
                 scene_header_sha256=digest(build_path / "scene.hh"),
                 display_header_sha256=digest(build_path / "display.hh"),
                 manifest=documents.loads(manifest_path.read_text(encoding="utf-8-sig")),
                 controller_sha256=digest(project / "assets/scripts/ForestController.cpp"))
    def request(path, data=None):
        req = urllib.request.Request(f"http://127.0.0.1:{port}/api/v1/" + path, data=data)
        with urllib.request.urlopen(req, timeout=4) as response:
            return response.read()
    observations, samples, full_frames = [], [], {}
    report = dict(passed=False, resolution=[width, height], phase_targets=PHASES,
                  poll_seconds=args.poll_seconds, comparison_limitation=LIMITATION,
                  build=build, observations=observations)
    stopped_by_driver = False
    with (args.output / "runtime.log").open("w") as log:
        process = subprocess.Popen([str(editor), "--project", str(project), "--play-psx", "--stop-after", str(args.seconds)],
                                   stdout=log, stderr=log, creationflags=flags)
        try:
            deadline = time.monotonic() + 60
            while True:
                assert process.poll() is None, "Owned emulator stopped before Forest initialized"
                try:
                    ram = request("cpu/ram/raw")
                    probe = struct.unpack_from("<16I", ram, addresses["probe"])
                    if probe[0] == 0x464f5253 and probe[1] >= 20:
                        break
                except (OSError, ValueError):
                    pass
                assert time.monotonic() < deadline, "Forest did not expose its initialized probe"
                time.sleep(.1)
            for sequence, (name, bits, ticks) in enumerate(PHASES, 1):
                folder = args.output / name
                folder.mkdir()
                request("execution-flow?function=pause", b"")
                ram = request("cpu/ram/raw")
                start_frame = decode_frame(ram, addresses, extra_count)["frame"]
                start_probe = list(struct.unpack_from("<16I", ram, addresses["probe"]))
                request(f"cpu/ram/raw?offset={addresses['command']}&size=12", struct.pack("<3I", bits, 0x54455354, sequence))
                request("execution-flow?function=resume", b"")
                deadline = time.monotonic() + args.phase_timeout
                frames = {}
                def observe(ram):
                    p = struct.unpack_from("<16I", ram, addresses["probe"])
                    phase = struct.unpack_from("<8I", ram, addresses["report"])
                    samples.append(p)
                    row = decode_frame(ram, addresses, extra_count)
                    if phase[0] == sequence and row["frame"] > start_frame:
                        row["route"] = dict(name=name, sequence=sequence, phase_ticks=phase[1], probe=list(p))
                        frames[row["frame"]] = row
                        full_frames[row["frame"]] = row
                    return p, phase
                while True:
                    assert process.poll() is None, f"Owned emulator stopped during {name}"
                    ram = request("cpu/ram/raw")
                    _, phase = observe(ram)
                    if phase[0] == sequence and phase[1] >= ticks:
                        break
                    assert time.monotonic() < deadline, ("Phase timed out", name, list(phase))
                    time.sleep(args.poll_seconds)
                request("execution-flow?function=pause", b"")
                ram = request("cpu/ram/raw")
                end, phase = observe(ram)
                vram = request("gpu/vram/raw")
                (folder / "vram.bin").write_bytes(vram)
                save_png(vram, folder / "screen.png", width, height)
                observation = dict(name=name, sequence=sequence, bits=bits, target_ticks=ticks,
                                   ticks=phase[1], overshoot_ticks=phase[1] - ticks,
                                   simulation_seconds=phase[6] / 4096,
                                   from_position=[signed_q12(phase[2]), signed_q12(start_probe[5]), signed_q12(phase[3])],
                                   to_position=[signed_q12(phase[4]), signed_q12(end[5]), signed_q12(phase[5])],
                                   start_probe=start_probe, probe=list(end), test_report=list(phase),
                                   samples=len(frames), vram_sha256=hashlib.sha256(vram).hexdigest())
                observations.append(observation)
                save(folder / "profile.json", profile(list(frames.values()), build, observation))
                save(folder / "terminal.json", observation)
                save(args.output / "route.json", report)
                save(args.output / "profile.json", profile(list(full_frames.values()), build))
                print(f"PASS phase {name}: {len(frames)} frames, {phase[1]}/{ticks} ticks, position={observation['to_position']}", flush=True)
            report.update(verify_gameplay(observations, samples))
            mapping_path, source_path = project / "assets/sprites/547369-frame-map.json", project / "assets/sprites/547369.png"
            mapping = documents.loads(mapping_path.read_text())
            assert mapping["source_sha256"] == digest(source_path), "Original sprite source changed"
            assert build["controller_sha256"] == digest(project / "assets/scripts/ForestController.cpp"), "Controller source changed during capture"
            report.update(passed=True, original_preserved=True,
                          frame_microseconds_range=[min(p[13] for p in samples), max(p[13] for p in samples)],
                          dropped_steps=samples[-1][14])
            save(args.output / "route.json", report)
        except Exception as error:
            report["error"] = f"{type(error).__name__}: {error}"
            save(args.output / "route.json", report)
            save(args.output / "profile.json", profile(list(full_frames.values()), build))
            raise
        finally:
            if process.poll() is None:
                try:
                    request(f"cpu/ram/raw?offset={addresses['command']}&size=12", bytes(12))
                    request("execution-flow?function=resume", b"")
                except (OSError, ValueError):
                    pass
                # End only the editor process tree created by this capture.
                # Completed/error routes need not wait for the safety timer.
                if os.name == "nt":
                    stopped = subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                             creationflags=flags, timeout=15)
                    stopped_by_driver = stopped.returncode == 0
                else:
                    process.terminate()
                    stopped_by_driver = True
            process.wait(timeout=15)
    report["shutdown"] = dict(requested_by_driver=stopped_by_driver, return_code=process.returncode)
    save(args.output / "route.json", report)
    assert stopped_by_driver or process.returncode == 0, "Owned editor/emulator exited unsuccessfully"
    print(f"PASS native Forest route gameplay and per-phase profiles: {args.output}", flush=True)


if __name__ == "__main__":
    main()
