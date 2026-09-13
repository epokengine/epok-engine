"""Hash-pinned external MIDI/library through the real PSX backend, in isolation.

No source corpus is committed or modified. Default run checks two complete song
loops; --seconds 12 is a diagnostic excerpt and cannot satisfy that gate.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import re
import shutil
import socket
import struct
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
EXE = ROOT / "target/debug/epok-editor.exe"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--seconds", type=int, default=228)
    parser.add_argument("--release-ms", type=int, default=500)
    parser.add_argument("--voice-limit", type=int, default=24)
    args = parser.parse_args()
    assert 5 <= args.seconds <= 300
    assert 10 <= args.release_ms <= 30000 and 1 <= args.voice_limit <= 24
    sources = [
        (Path(os.environ.get("EPOK_MIDI_SOURCE", ROOT.parent / "EpokDemos/Ironwood/assets/sounds/bgm/opening_02.mid")),
         "5709b0b1b24d93b7c83199b168d573f1e915add144b340633502da2e4b299efe", "song.mid"),
        (Path(os.environ.get("EPOK_SOUNDFONT_LIBRARY", ROOT / "artifacts/midi-completion/library-audit/FluidR3Mono_GM.sf3")),
         "cda013d8c370a48ae8dad271e761078d2e77455488dabdedbfbe5fc76a38c682", "library.sf3"),
    ]
    for source, digest, _ in sources:
        assert hashlib.sha256(source.read_bytes()).hexdigest() == digest, source
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 8077))
    stamp = time.time_ns()
    art = ROOT / "artifacts/midi-completion" / f"p4-psx-song-{stamp}"
    art.mkdir(parents=True)
    project = ROOT / ".epok" / f"psx-song-{stamp}"
    print(f"Project: {project}\nEvidence: {art}", flush=True)

    def run(*cmd):
        result = subprocess.run([str(EXE), *map(str, cmd)], capture_output=True,
                                text=True, timeout=300, creationflags=FLAGS)
        with (art / "commands.log").open("a", encoding="utf-8") as log:
            log.write(f"{cmd}\nexit={result.returncode}\n{result.stdout}{result.stderr}\n")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout

    def ids():
        return {r["path"]: r["id"] for r in json.loads(run("--project", project, "--scan-assets"))["assets"]}

    run("--create-project", project, "--template", "sample")
    for source, _, filename in sources:
        shutil.copyfile(source, project / "assets" / filename)
    run("--project", project, "--import-sound-bank", "assets/library.sf3")
    recipe = dict(preset="Custom", max_sample_rate=10208, maximum_release_ms=args.release_ms,
                  other_resident_bytes=4672, effects="Dry")
    documents.write_text(project / "assets/recipe.json", json.dumps(recipe))
    run("--project", project, "--import-audio", "assets/song.mid", "--sound-bank",
        ids()["assets/library.epokasset"], "--sequence-loop", "whole", "--voice-limit", str(args.voice_limit),
        "--psx-music-recipe", "assets/recipe.json")
    scene_path = project / "assets/scenes/SampleScene.epokmap"
    scene = documents.loads(scene_path.read_text())
    spinner = next(e for e in scene["entities"] if (e.get("script") or {}).get("name") == "Spinner")
    spinner["audio"] = dict(clip=ids()["assets/song.epokasset"], volume=0.7)
    documents.write_text(scene_path, json.dumps(scene))
    documents.write_text(project / "assets/scripts/Spinner.cpp", '''
#include "Spinner.hpp"
uint32_t song_probe[4]={};
void Spinner::start(epok::Transform&){song_probe[0]=0x534f4e47;}
void Spinner::update(epok::Transform&,epok::Fixed){
    ++song_probe[1];song_probe[2]=entity().audio.is_playing();
}
''')
    run("--project", project, "--build-psx")
    build = project / ".epok/build"
    symbols = (build / "epok.map").read_text()

    def address(name):
        match = re.search(r"0x([0-9a-f]+)\s+" + re.escape(name) + r"\b", symbols)
        assert match, name
        return int(match[1], 16) & 0x1fffff

    def request(path, post=False):
        req = urllib.request.Request("http://127.0.0.1:8077/api/v1/" + path,
                                     data=b"" if post else None)
        with urllib.request.urlopen(req, timeout=5) as response:
            return response.read()

    report = dict(project=str(project), sources=[dict(path=str(p), sha256=h) for p, h, _ in sources],
                  recipe=recipe, two_loop_acceptance=args.seconds >= 225,
                  cook=json.loads((build / "audio/sequence-report.json").read_text()), samples=[])
    with (art / "emulator.log").open("w") as log:
        process = subprocess.Popen([str(EXE), "--project", str(project), "--play-psx",
                                    "--stop-after", str(args.seconds + 12)],
                                   stdout=log, stderr=log, creationflags=FLAGS)
        try:
            deadline = time.monotonic() + 80
            while True:
                assert process.poll() is None, "Emulator boot failed"
                try:
                    ram = request("cpu/ram/raw")
                    if struct.unpack_from("<I", ram, address("song_probe"))[0] == 0x534f4e47:
                        break
                except (OSError, ValueError):
                    pass
                assert time.monotonic() < deadline, "Song fixture did not boot"
                time.sleep(.1)
            start = time.monotonic()
            while True:
                elapsed = time.monotonic() - start
                final = elapsed >= args.seconds
                if final:
                    request("execution-flow?function=pause", True)
                ram = request("cpu/ram/raw")
                stats = struct.unpack_from("<16I", ram, address("epok::music_sequence_stats"))
                probe = struct.unpack_from("<4I", ram, address("song_probe"))
                timing = struct.unpack_from("<3I", ram, address("epok::sequence_timing_stats"))
                total_ticks=struct.unpack_from("<Q",ram,address("epok::sequence_timing_stats")+16)[0]
                row = dict(wall_seconds=elapsed, stats=stats, probe=probe,
                           service_max_us=stats[14] * 625 / 2646, gap_max_us=stats[13],
                           deferred_events=timing[0],key_on_max_delay_us=timing[1],key_ons=timing[2],
                           service_average_us=total_ticks*625/2646/max(1,stats[1]),
                           service_cpu_percent=total_ticks*625/2646/max(1,stats[15])*100)
                report["samples"].append(row)
                documents.write_text(art / "evidence.json", json.dumps(report, indent=2))
                print(json.dumps(row), flush=True)
                if final:
                    break
                time.sleep(min(5, args.seconds - elapsed))
            assert stats[11:13] == (0, 0), "Runtime/clock fault"
            assert stats[5:9] == (0, 0, 0, 0), "Steals, denied notes, capacity errors or pitch clamps"
            assert stats[9] <= 24 and probe[2] == 1, "Voice demand or song lifecycle"
            assert row["service_max_us"] <= 2000 and row["gap_max_us"] <= 3000, "Normal CPU timing gate"
            assert row["key_on_max_delay_us"] <= 3100, "Note-on scheduling gate"
            if report["two_loop_acceptance"]:
                assert stats[4] >= 2 and stats[15] >= 224_000_000, "Two musical loops did not complete"
            assert process.wait(timeout=25) == 0
            print("PASS " + ("two complete real-song loops" if report["two_loop_acceptance"] else "diagnostic excerpt only"), flush=True)
        finally:
            if process.poll() is None:
                process.wait(timeout=args.seconds + 30)


if __name__ == "__main__":
    main()
