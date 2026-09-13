"""Measure Forest movement against both simulation time and host wall time.

Uses the existing native test pad/probe, never rewrites the controller. Run on
an isolated ForestClearing project and a free web port. Results are emulator
measurements, not hardware benchmarks; HTTP observation adds host overhead.
"""
import argparse
import hashlib
import json
import socket
import struct
import subprocess
import time
import urllib.request
from pathlib import Path

import epok_documents as documents
from profile_runtime import ROOT, EDITOR, symbol_address
from profile_forest_route import save_png


def q12(value):
    return (value if value < 0x80000000 else value - 0x100000000) / 4096


def summarize(rows):
    if len(rows)<2:
        raise ValueError("Need at least two movement samples")
    first, last = rows[0], rows[-1]
    seconds = last["wall"] - first["wall"]
    ticks = last["probe"][1] - first["probe"][1]
    if seconds<=0 or ticks<=0 or last["frame"]<first["frame"]:
        raise ValueError("Movement clock stopped or reset during capture")
    distance = q12(last["probe"][4]) - q12(first["probe"][4])
    return dict(samples=len(rows), wall_seconds=seconds, ticks=ticks,
                simulation_seconds=ticks / 60, ticks_per_wall_second=ticks / seconds,
                distance_x=distance, units_per_simulation_second=abs(distance) / (ticks / 60),
                units_per_wall_second=abs(distance) / seconds,
                rendered_frames_per_wall_second=(last["frame"] - first["frame"]) / seconds,
                dropped_steps=last["probe"][14] - first["probe"][14],
                max_dropped_triangles=max(row["probe"][9] for row in rows))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--editor", type=Path, default=EDITOR)
    args = parser.parse_args()
    if args.output.exists() and any(args.output.iterdir()):
        parser.error("Output must be new or empty")
    args.output.mkdir(parents=True, exist_ok=True)
    project, editor = args.project.resolve(), args.editor.resolve()
    config = documents.loads((project / "Local.epokconfig").read_text())
    port = int(config["web_port"])
    with socket.socket() as available:
        available.bind(("127.0.0.1", port))
    command = [str(editor), "--project", str(project)]
    result = subprocess.run(command + ["--build-psx", "--use-play-profile"], capture_output=True, text=True, timeout=240)
    documents.write_text(args.output / "build.log", result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError("Build failed; see build.log")
    symbols = (project / ".epok/build/epok.map").read_text()
    import re
    display = (project / ".epok/build/display.hh").read_text()
    width, height = [int(re.search(name + r"\s*=\s*(\d+)", display)[1])
                     for name in ("display_width", "display_height")]
    addresses = {name: symbol_address(symbols, symbol) for name, symbol in
                 (("probe", "forest_probe"), ("command", "forest_test_command"), ("frame", "epok::performance_stats"))}
    if any(value is None for value in addresses.values()):
        raise RuntimeError("The scene must expose the Forest test probe")

    def request(path, data=None):
        req = urllib.request.Request(f"http://127.0.0.1:{port}/api/v1/" + path, data=data)
        with urllib.request.urlopen(req, timeout=3) as response:
            return response.read()

    def sample():
        before = time.monotonic()
        # Redux's GET endpoint returns the full RAM even when offset/size are
        # present (those parameters belong to POST). One coherent snapshot.
        ram = request("cpu/ram/raw")
        probe = struct.unpack_from("<16I", ram, addresses['probe'])
        after = time.monotonic()
        frame = struct.unpack_from("<I", ram, addresses['frame'])[0]
        return dict(wall=(before + after) / 2, request_seconds=after-before, probe=list(probe), frame=frame)

    report = dict(project=str(project), editor=str(editor),
                  display_header=display,
                  controller_sha256=hashlib.sha256((project / "assets/scripts/ForestController.cpp").read_bytes()).hexdigest(),
                  executable_sha256=hashlib.sha256((project / ".epok/build/epok.ps-exe").read_bytes()).hexdigest(),
                  phases={})
    # --stop-after owns cleanup, avoiding interference with another editor.
    with (args.output / "runtime.log").open("w") as log:
        process = subprocess.Popen(command + ["--play-psx", "--use-play-profile", "--stop-after", "15"],
                                   stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        try:
            deadline = time.monotonic() + 90
            while True:
                if process.poll() is not None or time.monotonic() > deadline:
                    raise RuntimeError("Forest did not start")
                try:
                    row = sample()
                    if row["probe"][0] == 0x464f5253 and row["probe"][1] > 60:
                        break
                except OSError:
                    pass
                time.sleep(.1)
            for seq, (name, bits) in enumerate((("walk_right", 1), ("run_left", 2 | 16)), 1):
                request(f"cpu/ram/raw?offset={addresses['command']}&size=12", struct.pack("<3I", bits, 0x54455354, seq))
                time.sleep(.15)  # Exclude the command-delivery edge, keep both phases clear of walls.
                rows = []
                end = time.monotonic() + 2
                while time.monotonic() < end:
                    rows.append(sample())
                    time.sleep(.05)
                report["phases"][name] = dict(summary=summarize(rows), rows=rows)
                summary=report["phases"][name]["summary"]
                expected_speed=2.3 if name=="walk_right" else 4.2
                assert abs(summary["units_per_simulation_second"]-expected_speed)<.05, summary
                assert summary["dropped_steps"]==0 and summary["max_dropped_triangles"]==0, summary
                save_png(request("gpu/vram/raw"), args.output / (name + ".png"), width, height)
                print(name, json.dumps(report["phases"][name]["summary"]), flush=True)
            request(f"cpu/ram/raw?offset={addresses['command']}&size=12", bytes(12))
        finally:
            # The app's timer terminates its own emulator and closes the PTY.
            try:
                process.wait(timeout=45)
            except subprocess.TimeoutExpired:
                # Only the process tree created by this diagnostic is in scope.
                if hasattr(subprocess,"CREATE_NO_WINDOW"):
                    subprocess.run(["taskkill","/PID",str(process.pid),"/T","/F"],
                                   stdout=log,stderr=log,creationflags=subprocess.CREATE_NO_WINDOW,timeout=15)
                else:
                    process.terminate()
                process.wait(timeout=15)
                report["forced_cleanup"]=True
            report["exit_code"] = process.returncode
            documents.write_text(args.output / "motion.json", json.dumps(report, indent=2) + "\n")
    if process.returncode:
        raise RuntimeError("Owned editor exited unsuccessfully")


if __name__ == "__main__":
    main()
