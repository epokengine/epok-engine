"""Real MIPS Play profiles: scene scope, PCDrv geometry and synchronized fades.

Uses a fresh project and port 8097. Never connects to physical serial hardware.
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
from verify_streaming import package, uid
import copy
import json
import math
import re
import socket
import struct
import subprocess
import time
import urllib.request
import wave
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
EXE = ROOT / "target/debug/epok-editor.exe"
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
PORT = 8097
PROBE = r'''#include "PlayProbe.hpp"
#include "common/hardware/spu.h"
extern "C" {volatile uint32_t play_probe[12]={},play_command=0,play_meta[4]={};}
void PlayProbe::start(epok::Transform&){
    play_probe[0]=0x504c4159;play_probe[4]=play_probe[4]+1;
    play_meta[0]=uint32_t(&epok::transition.phase);
    play_meta[1]=uint32_t(&epok::transition.audio_gain);
    play_meta[2]=uint32_t(&epok::transition.opacity);
    play_meta[3]=uint32_t(epok::transition.text);
}
void PlayProbe::on_destroy(){play_probe[5]=play_probe[5]+1;}
void PlayProbe::update(epok::Transform&,epok::Fixed){}
void PlayProbe::frame_update(epok::Transform&,uint32_t dt){
    play_probe[1]=play_probe[1]+1;play_probe[2]=epok::current_scene();
    if(dt>play_probe[6])play_probe[6]=dt;
    play_probe[7]=SPU_VOICES[0].volumeLeft;
    play_probe[8]=entity().audio.volume.raw();
    if(play_command){
        epok::TransitionOptions options;options.fade_out_ms=1200;options.fade_in_ms=1200;
        char message[]="Entering second scene";options.loading.text=message;
        play_probe[3]=epok::request_scene("Second",options)?1:2;
        message[0]='X';play_command=0;
    }
}
'''


def main():
    out = ROOT / "artifacts/play-profiles" / str(time.time_ns())
    out.mkdir(parents=True)
    folder = out / "project"
    with socket.socket() as available:
        available.bind(("127.0.0.1", PORT))

    def save(path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        documents.write_text(path, json.dumps(value, indent=2), encoding="utf-8")

    def run(*args, success=True):
        result = subprocess.run([str(EXE), *map(str, args)], capture_output=True, text=True,
                                creationflags=FLAGS, timeout=240)
        with (out / "build.log").open("a", encoding="utf-8") as log:
            log.write(result.stdout + result.stderr)
        assert (result.returncode == 0) == success, result.stdout + result.stderr
        return result.stdout + result.stderr

    def request(path, data=None):
        with urllib.request.urlopen(urllib.request.Request(
                f"http://127.0.0.1:{PORT}/api/v1/" + path, data=data), timeout=5) as response:
            return response.read()

    run("--create-project", folder, "--template", "basic")
    config = documents.loads((ROOT / "Editor.epokconfig").read_text())
    for key in ("make", "toolchain_bin", "nugget", "emulator", "psxavenc", "mkpsxiso"):
        config[key] = str((ROOT / config[key]).resolve())
    config["web_port"] = PORT
    save(folder / "Local.epokconfig", config)
    with wave.open(str(folder / "assets/tone.wav"), "wb") as wav:
        wav.setparams((1, 2, 22050, 0, "NONE", "not compressed"))
        wav.writeframes(b"".join(struct.pack("<h", round(math.sin(i * 2 * math.pi * 440 / 22050) * 10000)) for i in range(2205)))
    run("--project", folder, "--import-audio", "assets/tone.wav", "--loop")
    Image.new("RGBA", (32, 16), (32, 192, 255, 255)).save(folder / "assets/loading.png")
    run("--project", folder, "--import-texture", "assets/loading.png")
    records = json.loads(run("--project", folder, "--scan-assets"))["assets"]
    ids = {r["path"]: r["id"] for r in records}
    geometry, group, material = uid(), uid(), uid()
    mesh = dict(version=1, vertices=[[-1,-1,6],[-1,1,6],[1,1,6],[1,-1,6]],
                faces=[dict(id=uid(), vertices=[0,1,2,3], group=group, material=material)],
                groups=[dict(id=group, name="Plane", parent=None)],
                materials=[dict(id=material, name="Blue", material=dict(color=[0.2,0.6,1.0], unlit=True))])
    package(folder / "assets/plane.epokasset", geometry, mesh)
    manifest_path = project_manifest(folder)
    manifest = documents.loads(manifest_path.read_text())
    manifest["transition"] = dict(fade_out_ms=300, fade_in_ms=300, text="Now loading...", image=ids["assets/loading.epokasset"])
    scene = dict(version=1, name="First", entities=[
        dict(name="Camera", kind="Camera", position=[0,0,0]),
        dict(name="Plane", kind="Mesh", editable_mesh=dict(asset=geometry)),
        dict(name="Probe", kind="Empty", script=dict(name="PlayProbe"),
             audio=dict(clip=ids["assets/tone.epokasset"], volume=0.5, play_on_start=True))])
    for entity in scene["entities"]:
        entity.setdefault("position", [0,0,0])
        entity["rotation"] = [0,0,0]
        entity["scale"] = [1,1,1]
    save(folder / manifest["startup_scene"], scene)
    second = copy.deepcopy(scene)
    second["name"] = "Second"
    save(folder / "assets/scenes/Second.epokmap", second)
    save(folder / "ProjectSettings/Maps.epoksettings", dict(scenes=["assets/scenes/Second.epokmap"]))
    documents.write_text(folder / "assets/scripts/PlayProbe.hpp", '#pragma once\n#include "epok.hpp"\nclass PlayProbe:public epok::Behaviour {public:void start(epok::Transform&)override;void update(epok::Transform&,epok::Fixed)override;void frame_update(epok::Transform&,uint32_t)override;void on_destroy()override;};\n')
    documents.write_text(folder / "assets/scripts/PlayProbe.cpp", PROBE)
    save(folder / "assets/scripts/PlayProbe.epokscript", dict(name="PlayProbe", properties=[]))
    results = {}
    for mode in ("current", "whole", "host", "disc"):
        external = mode in ("host", "disc")
        manifest["play"] = dict(target="window", content="current_scene" if mode=="current" else "whole_game",
                                data=mode if external else "executable")
        manifest["rendering"] = dict(width=320, height=240, streaming_geometry=external, streaming_pool_pages=2)
        save(manifest_path, manifest)
        run("--project", folder, "--build-psx", "--use-play-profile")
        build = folder / ".epok/build"
        symbols = (build / "epok.map").read_text()
        def address(name):
            match = re.search(r"0x([0-9a-fA-F]+)\s+" + re.escape(name) + r"\b", symbols)
            assert match, name
            return int(match[1],16) & 0x1fffff
        probe, command, meta = map(address, ("play_probe", "play_command", "play_meta"))
        counters = address("epok::streaming_stats") if external else None
        expected_count = 1 if mode == "current" else 2
        header = (build / "scene.hh").read_text()
        assert len(re.findall(r",load_bank_\d+\}", header)) == expected_count
        log_path = out / f"{mode}-runtime.log"
        samples = []
        with log_path.open("w") as log:
            process = subprocess.Popen([str(EXE), "--project", str(folder), "--play-psx", "--use-play-profile", "--stop-after", "120"],
                                       stdout=log, stderr=log, creationflags=FLAGS)
            try:
                deadline = time.monotonic() + 90
                while True:
                    assert process.poll() is None, log_path.read_text(errors="replace")
                    try:
                        ram = request("cpu/ram/raw")
                        values = struct.unpack_from("<12I", ram, probe)
                        if values[0] == 0x504c4159 and values[1] >= 5:
                            break
                    except (OSError, ValueError):
                        pass
                    assert time.monotonic() < deadline, "No completed native frame"
                    time.sleep(0.1)
                pointers = [v & 0x1fffff for v in struct.unpack_from("<4I", ram, meta)]
                request(f"cpu/ram/raw?offset={command}&size=4", struct.pack("<I",1))
                deadline = time.monotonic() + 20
                captured = False
                while True:
                    ram = request("cpu/ram/raw")
                    values = struct.unpack_from("<12I", ram, probe)
                    phase = ram[pointers[0]]
                    gain = struct.unpack_from("<H",ram,pointers[1])[0]
                    opacity = ram[pointers[2]]
                    samples.append([phase,gain,opacity,values[1],values[4],values[5]])
                    if phase == 2 and not captured:
                        vram = request("gpu/vram/raw")
                        (out / f"{mode}-loading.vram").write_bytes(vram)
                        pixels = struct.unpack("<524288H",vram)
                        img = Image.new("RGB",(320,240))
                        img.putdata([((p&31)*255//31,((p>>5)&31)*255//31,((p>>10)&31)*255//31)
                                     for y in range(240) for p in pixels[y*1024:y*1024+320]])
                        img.save(out / f"{mode}-loading.png")
                        captured = True
                    if mode == "current" and values[3] == 2:
                        assert values[4] == 1 and values[5] == 0
                        break
                    if mode != "current" and values[2] == 1 and values[1] >= 10 and values[7] == 8192:
                        assert values[3] == 1 and values[4:6] == (2,1)
                        assert values[7:9] == (8192,2048), values
                        break
                    assert time.monotonic() < deadline, (mode,values,samples[-5:])
                    time.sleep(0.01)
                if mode != "current":
                    outgoing = [v for v in samples if v[0]==1]
                    incoming = [v for v in samples if v[0]==3]
                    assert any(0<v[1]<4096 for v in outgoing) and any(0<v[1]<4096 for v in incoming)
                    assert all(abs(v[2]-(4096-v[1])*255/4096)<=1 for v in outgoing+incoming)
                    assert all(v[5]==0 for v in outgoing), "Scene released before fade out"
                    assert ram[pointers[3]:pointers[3]+21] == b"Entering second scene"
                if external:
                    stats = struct.unpack_from("<7I",ram,counters)
                    assert stats[0]>0 and stats[4:6]==(0,0),stats
                results[mode] = dict(samples=samples,probe=list(values),loading_capture=captured)
                print(f"PASS {mode}: scene scope, native frames, loading and audio restoration",flush=True)
            finally:
                if process.poll() is None:
                    subprocess.run(["taskkill","/PID",str(process.pid),"/T","/F"],capture_output=True,creationflags=FLAGS)
                process.wait(timeout=15)
        save(out / "report.json",results)
    print(f"PASS Play profiles and native transitions: {out}",flush=True)


if __name__ == "__main__":
    main()
