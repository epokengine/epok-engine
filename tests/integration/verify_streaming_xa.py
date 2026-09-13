"""Actual shared CD acceptance: geometry demand reads interrupt and restart XA.

Owns port 8093. Run serially with other emulator acceptance tests.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import socket
import struct
import subprocess
import time
import urllib.request

from verify_streaming import ROOT, EXE, FLAGS, uid, save, package

PORT = 8093
PROBE = r'''#include "XaStreamProbe.hpp"
extern "C" {volatile uint32_t xa_stream_command=0,xa_stream_probe[16]={};}
void XaStreamProbe::start(epok::Transform&){}
void XaStreamProbe::update(epok::Transform&,epok::Fixed){}
void XaStreamProbe::frame_update(epok::Transform&,uint32_t){
  using namespace epok;
  uint32_t phase=xa_stream_command;
  auto* camera=find_entity("Camera");auto* source=find_entity("Music");
  camera->transform.position[0]=Fixed(phase==0?-70:(phase==1?0:(phase==2?70:-70)),0);
  camera->transform.position[1]=Fixed(1);
  if(phase==3&&!xa_stream_probe[11]){source->audio.stop();xa_stream_probe[11]=1;}
  if(phase==4&&xa_stream_probe[11]==1){
    xa_stream_probe[12]=source->generation;
    xa_stream_probe[11]=2;
    request_scene(size_t(0));
  }
  if(phase==5&&xa_stream_probe[11]==2){
    xa_stream_probe[12]=source->generation;
    xa_stream_probe[11]=3;
    request_scene(size_t(0));
  }
  xa_stream_probe[0]=0x58415354;xa_stream_probe[1]=xa_stream_probe[1]+1;
  xa_stream_probe[3]=xa_stream_probe[2]==phase?xa_stream_probe[3]+1:0;xa_stream_probe[2]=phase;
  auto& m=music_stats;
  xa_stream_probe[4]=m.state;xa_stream_probe[5]=m.starts;xa_stream_probe[6]=m.loops;
  xa_stream_probe[7]=m.errors;xa_stream_probe[8]=m.error_code;
  xa_stream_probe[9]=performance_stats.stream_failed_chunks;
  xa_stream_probe[10]=performance_stats.triangles;
  xa_stream_probe[13]=source->generation;
  xa_stream_probe[14]=scene_loading();
  xa_stream_probe[15]=source->audio.is_playing();
}
'''


def request(path, data=None):
    req = urllib.request.Request(f"http://127.0.0.1:{PORT}/api/v1/" + path, data=data)
    with urllib.request.urlopen(req, timeout=5) as response:
        return response.read()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    parser.add_argument("--build-only", action="store_true")
    args = parser.parse_args()
    out = args.output or ROOT / "artifacts/streaming-xa" / str(time.time_ns())
    out.mkdir(parents=True, exist_ok=False)
    folder = out / "project"

    def run(*arguments):
        result = subprocess.run([str(EXE), *map(str, arguments)], capture_output=True,
                                text=True, creationflags=FLAGS, timeout=240)
        with (out / "build.log").open("a", encoding="utf-8") as log:
            log.write(result.stdout + result.stderr)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout

    run("--create-project", folder, "--template", "sample")
    config = documents.loads((ROOT / "Editor.epokconfig").read_text())
    # The copied Local configuration belongs to the fixture. Preserve the
    # original tool locations, including the XA encoder and disc packager.
    for key in ("make", "toolchain_bin", "nugget", "emulator", "psxavenc", "mkpsxiso", "libclang"):
        value = config.get(key)
        if value and (key in ("toolchain_bin", "nugget", "libclang") or "/" in value or "\\" in value):
            config[key] = str((ROOT / value).resolve())
    config["web_port"] = PORT
    save(folder / "Local.epokconfig", config)
    manifest_path = project_manifest(folder)
    manifest = documents.loads(manifest_path.read_text())
    manifest["rendering"] = dict(width=320, height=240, retained_geometry=False,
                                 streaming_pool_pages=2, streaming_geometry=True,
                                 precomputed_visibility=True)
    save(manifest_path, manifest)
    shutil.copyfile(ROOT / "tests/fixtures/stereo-tone.mp3", folder / "assets/Music.mp3")
    run("--project", folder, "--import-audio", "assets/Music.mp3", "--audio-usage", "bgm", "--loop")
    records = json.loads(run("--project", folder, "--scan-assets"))["assets"]
    music = next(r["id"] for r in records if r["path"] == "assets/Music.epokasset")
    group, slot, identity = uid(), uid(), uid()
    doc = dict(version=1, vertices=[], faces=[], groups=[dict(id=group, name="Panels", parent=None)],
               materials=[dict(id=slot, name="Surface", material=dict(color=[0.25, 0.75, 1.0], unlit=True))])
    for center in (-70, 0, 70):
        for row in range(20):
            for col in range(25):
                x, y = center - 1.25 + col * 0.1, row * 0.1
                base = len(doc["vertices"])
                doc["vertices"].extend([[x,y,8], [x,y+0.09,8], [x+0.09,y+0.09,8], [x+0.09,y,8]])
                doc["faces"].append(dict(id=uid(), vertices=list(range(base,base+4)), group=group, material=slot))
    package(folder / "assets/Streaming.epokasset", identity, doc)
    def entity(name, kind="Empty", **extra):
        return dict(name=name, kind=kind, position=[0,0,0], rotation=[0,0,0], scale=[1,1,1], **extra)
    save(folder / "assets/scenes/SampleScene.epokmap", dict(version=1, name="SampleScene", entities=[
        entity("Camera", "Camera"),
        entity("Panels", "Mesh", editable_mesh=dict(asset=identity), material=dict(unlit=True)),
        entity("Music", audio=dict(clip=music, volume=0.5, priority=200)),
        entity("Probe", script=dict(name="XaStreamProbe"))]))
    documents.write_text(folder / "assets/scripts/XaStreamProbe.hpp", '#pragma once\n#include "epok.hpp"\nclass XaStreamProbe:public epok::Behaviour{public:void start(epok::Transform&)override;void update(epok::Transform&,epok::Fixed)override;void frame_update(epok::Transform&,uint32_t)override;};\n')
    documents.write_text(folder / "assets/scripts/XaStreamProbe.cpp", PROBE)
    save(folder / "assets/scripts/XaStreamProbe.epokscript", dict(name="XaStreamProbe", properties=[]))
    run("--project", folder, "--build-psx")
    build = folder / ".epok/build"
    pages = (build / "GEOMETRY.BIN").stat().st_size // 65536
    assert pages > 2 and (build / "epok.cue").is_file(), "Fixture needs more pages than pool and an XA CD"
    provenance = json.loads((folder / ".epok/ArtifactDependencies.json").read_text())["nodes"]
    payloads = list((build / "music").glob("*.XA"))
    assert len(payloads) == 1, "The fixture must stage its imported XA stream"
    payload = payloads[0]
    resource = provenance[f"generated-resource:.epok/build/music/{payload.name}"]
    assert not resource["stale"] and resource["dependencies"] == [f"asset:{music}"]
    assert resource["signature"] == hashlib.sha256(payload.read_bytes()).hexdigest()
    bank = provenance["generated-resource:.epok/build/audio-bank.hh"]
    assert not bank["stale"] and f"asset:{music}" in bank["dependencies"]
    assert bank["signature"] == hashlib.sha256((build / "audio-bank.hh").read_bytes()).hexdigest()
    assert any(key.startswith("audio-selection:scene-") for key in bank["dependencies"])
    assert not any(key.startswith(("scene-file:", "scene-editor:")) for key in bank["dependencies"])
    if args.build_only:
        print(f"PASS XA+streaming fixture target build: {out}", flush=True)
        return
    with socket.socket() as available:
        available.bind(("127.0.0.1", PORT))
    symbols = (build / "epok.map").read_text()
    def address(name):
        match = re.search(r"0x([0-9a-fA-F]+)\s+" + re.escape(name) + r"\b", symbols)
        assert match, name
        return int(match[1],16) & 0x1fffff
    command, probe, counters = map(address, ("xa_stream_command", "xa_stream_probe", "epok::streaming_stats"))
    report = dict(pages=pages, pool_pages=2, phases=[])
    log_path = out / "runtime.log"
    with log_path.open("w") as log:
        process = subprocess.Popen([str(EXE), "--project", str(folder), "--play-psx", "--stop-after", "80"],
                                   stdout=log, stderr=log, creationflags=FLAGS)
        try:
            deadline = time.monotonic() + 60
            while True:
                assert process.poll() is None, log_path.read_text(errors="replace")
                try:
                    if not json.loads(request("execution-flow"))["running"]:
                        request("execution-flow?function=resume", b"")
                    ram = request("cpu/ram/raw")
                    values = struct.unpack_from("<16I", ram, probe)
                    if values[0] == 0x58415354 and values[4] == 4 and values[6] >= 1:
                        break
                except (OSError, ValueError):
                    pass
                assert time.monotonic() < deadline, "XA never reached looping playback"
                time.sleep(0.1)
            for phase in range(6):
                request("execution-flow?function=pause", b"")
                request(f"cpu/ram/raw?offset={command}&size=4", struct.pack("<I", phase))
                request("execution-flow?function=resume", b"")
                deadline = time.monotonic() + 25
                while True:
                    assert process.poll() is None, log_path.read_text(errors="replace")
                    ram = request("cpu/ram/raw")
                    values = struct.unpack_from("<16I", ram, probe)
                    state_ok = values[4] == (2 if phase == 3 else 4)
                    generation_ok = phase < 4 or (values[12] != values[13] and not values[14])
                    if values[2] == phase and values[3] >= 10 and state_ok and generation_ok:
                        break
                    assert not values[7] and not values[9], (phase, values)
                    assert time.monotonic() < deadline, ("XA geometry phase stalled", phase, values)
                    time.sleep(0.1)
                stats = list(struct.unpack_from("<7I", ram, counters))
                row = dict(phase=phase, probe=list(values), streaming=stats)
                assert values[7] == values[9] == 0 and values[10] > 0 and stats[4:6] == [0,0], row
                if phase == 2:
                    baseline = report["phases"][0]
                    assert stats[0] > baseline["streaming"][0] and stats[6] > baseline["streaming"][6], row
                    assert values[5] > baseline["probe"][5] and values[15] == 1, row
                if phase == 3:
                    assert values[15] == 0, row
                if phase >= 4:
                    assert values[15] == 1 and values[12] != values[13], row
                if phase == 5:
                    assert values[5] > report["phases"][4]["probe"][5], row
                report["phases"].append(row)
                save(out / "report.json", report)
                print(f"PASS XA+geometry phase {phase}: reads={stats[0]}, interruptions={stats[6]}, starts={values[5]}", flush=True)
        finally:
            process.wait(timeout=90)
    print(f"PASS real CD geometry + XA loop, interruption, restart, stop and scene generation: {out}", flush=True)


if __name__ == "__main__":
    main()
