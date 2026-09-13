"""Real PSX sequence contention, geometry/CD, XA switching and scene retirement."""
from pathlib import Path
import argparse
import json
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
from project_paths import project_manifest
from verify_streaming import uid, save, package

EXE = ROOT / "target/debug/epok-editor.exe"
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
PORT = 8077

PROBE = r'''#include "SequenceStress.hpp"
extern "C" {volatile uint32_t sequence_stress_command=0,sequence_stress_probe[16]={};}
epok::AudioSource stress_sfx[24];
uint32_t stress_seen=0xffffffffu;
void SequenceStress::start(epok::Transform&){}
void SequenceStress::update(epok::Transform&,epok::Fixed){}
void SequenceStress::frame_update(epok::Transform&,uint32_t){
  using namespace epok;
  const uint32_t phase=sequence_stress_command;
  auto* camera=find_entity("Camera");auto* owner=find_entity("Music");
  camera->transform.position[0]=Fixed(phase==0?-70:(phase==1?0:(phase==2?70:-70)),0);
  camera->transform.position[1]=Fixed(1);
  if(phase!=stress_seen){
    stress_seen=phase;
    if(phase==1)for(auto& s:stress_sfx){s.enabled=true;s.clip=SFX_INDEX;s.volume=0.0;s.priority=255;s.play();}
    if(phase==2)for(auto& s:stress_sfx)s.stop();
    if(phase==3){owner->audio.clip=XA_INDEX;owner->audio.play();}
    if(phase==4){owner->audio.clip=MIDI_INDEX;owner->audio.play();}
    if(phase==5){sequence_stress_probe[12]=owner->generation;sequence_stress_probe[15]=request_scene(size_t(0));}
  }
  sequence_stress_probe[0]=0x53455153;sequence_stress_probe[1]=sequence_stress_probe[1]+1;
  sequence_stress_probe[3]=sequence_stress_probe[2]==phase?sequence_stress_probe[3]+1:0;sequence_stress_probe[2]=phase;
  sequence_stress_probe[4]=music_stats.state;sequence_stress_probe[5]=music_stats.starts;
  sequence_stress_probe[6]=music_stats.errors;sequence_stress_probe[7]=music_stats.error_code;
  sequence_stress_probe[8]=performance_stats.stream_failed_chunks;sequence_stress_probe[9]=performance_stats.triangles;
  sequence_stress_probe[10]=owner->audio.is_playing();sequence_stress_probe[11]=scene_loading();
  sequence_stress_probe[13]=owner->generation;
  uint32_t playing=0;for(auto& s:stress_sfx)playing+=s.is_playing();sequence_stress_probe[14]=playing;
}
'''


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", action="store_true", help="Use the owned two-layer EPSB v2 library; default preserves the legacy bank case")
    args = parser.parse_args()
    prefix = "audio-library-stress" if args.library else "audio-phase-d-stress"
    out = ROOT / "artifacts" / f"{prefix}-{time.time_ns()}"
    out.mkdir(parents=True, exist_ok=False)
    project = ROOT / ".epok" / out.name
    print(f"Evidence: {out}\nProject: {project}", flush=True)
    def run(*args):
        result = subprocess.run([str(EXE), *map(str, args)], capture_output=True, text=True,
                                timeout=240, creationflags=FLAGS)
        with (out / "commands.log").open("a", encoding="utf-8") as log:
            log.write(f"{args}\nexit={result.returncode}\n{result.stdout}{result.stderr}\n")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout
    run("--create-project", project, "--template", "sample")
    config = documents.loads((ROOT / "Editor.epokconfig").read_text())
    for key in ("make", "toolchain_bin", "nugget", "emulator", "psxavenc", "mkpsxiso", "libclang"):
        value = config.get(key)
        if value and (key in ("toolchain_bin", "nugget", "libclang") or "/" in value or "\\" in value):
            config[key] = str((ROOT / value).resolve())
    config["web_port"] = PORT
    save(project / "Local.epokconfig", config)
    manifest_path = project_manifest(project)
    manifest = documents.loads(manifest_path.read_text())
    manifest["rendering"] = dict(width=320, height=240, retained_geometry=False,
                                 streaming_pool_pages=2, streaming_geometry=True, precomputed_visibility=True)
    save(manifest_path, manifest)
    run("--project", project, "--create-starter-bank", "assets/Retro.epokasset")
    run("--project", project, "--reimport-asset", "assets/Retro-triangle.epokasset", "--snapshot", "--loop")
    recipe_args = []
    if args.library:
        from verify_psx_library import font
        (project / "assets/Layered.sf2").write_bytes(font())
        run("--project", project, "--import-sound-bank", "assets/Layered.sf2")
        save(project / "assets/recipe.json", dict(preset="Custom", max_sample_rate=22000, effects="Room", reverb_depth_permille=250))
        recipe_args = ["--psx-music-recipe", "assets/recipe.json"]
    records = json.loads(run("--project", project, "--scan-assets"))["assets"]
    ids = {r["path"]: r["id"] for r in records}
    body = b"".join(bytes([0, 0x90, key, 90]) for key in range(48, 73))
    body += bytes([96, 0x80, 48, 0])
    body += b"".join(bytes([0, 0x80, key, 0]) for key in range(49, 73)) + b"\0\xff\x2f\0"
    midi = b"MThd" + struct.pack(">IHHH", 6, 0, 1, 96) + b"MTrk" + struct.pack(">I", len(body)) + body
    (project / "assets/Chord.mid").write_bytes(midi)
    bank_path = "assets/Layered.epokasset" if args.library else "assets/Retro.epokasset"
    run("--project", project, "--import-audio", "assets/Chord.mid", "--sound-bank", ids[bank_path], "--sequence-loop", "whole", "--voice-limit", "16", *recipe_args)
    shutil.copyfile(ROOT / "tests/fixtures/stereo-tone.mp3", project / "assets/Music.mp3")
    run("--project", project, "--import-audio", "assets/Music.mp3", "--audio-role", "music", "--load-mode", "stream", "--rate", "37800", "--loop")
    records = json.loads(run("--project", project, "--scan-assets"))["assets"]
    ids = {r["path"]: r["id"] for r in records}
    song, sfx, xa = (ids[k] for k in ("assets/Chord.epokasset", "assets/Retro-triangle.epokasset", "assets/Music.epokasset"))
    group, slot, identity = uid(), uid(), uid()
    mesh = dict(version=1, vertices=[], faces=[], groups=[dict(id=group, name="Panels", parent=None)],
                materials=[dict(id=slot, name="Surface", material=dict(color=[0.25, 0.75, 1.0], unlit=True))])
    for center in (-70, 0, 70):
        for row in range(20):
            for col in range(25):
                x, y = center - 1.25 + col * 0.1, row * 0.1
                base = len(mesh["vertices"])
                mesh["vertices"].extend([[x,y,8], [x,y+0.09,8], [x+0.09,y+0.09,8], [x+0.09,y,8]])
                mesh["faces"].append(dict(id=uid(), vertices=list(range(base,base+4)), group=group, material=slot))
    package(project / "assets/Streaming.epokasset", identity, mesh)
    def entity(name, kind="Empty", **extra):
        return dict(name=name, kind=kind, position=[0,0,0], rotation=[0,0,0], scale=[1,1,1], **extra)
    save(project / "assets/scenes/SampleScene.epokmap", dict(version=1, name="SampleScene", entities=[
        entity("Camera", "Camera"), entity("Panels", "Mesh", editable_mesh=dict(asset=identity), material=dict(unlit=True)),
        entity("Music", audio=dict(clip=song, volume=0.3, priority=200)),
        entity("SfxResource", audio=dict(clip=sfx, play_on_start=False)),
        entity("XaResource", audio=dict(clip=xa, play_on_start=False)), entity("Probe", script=dict(name="SequenceStress"))]))
    source = PROBE
    for name, id in (("MIDI_INDEX", song), ("SFX_INDEX", sfx), ("XA_INDEX", xa)):
        source = source.replace(name, str(sorted([song, sfx, xa]).index(id)))
    documents.write_text(project / "assets/scripts/SequenceStress.hpp", '#pragma once\n#include "epok.hpp"\nclass SequenceStress:public epok::Behaviour{public:void start(epok::Transform&)override;void update(epok::Transform&,epok::Fixed)override;void frame_update(epok::Transform&,uint32_t)override;};\n')
    documents.write_text(project / "assets/scripts/SequenceStress.cpp", source)
    save(project / "assets/scripts/SequenceStress.epokscript", dict(name="SequenceStress", properties=[]))
    run("--project", project, "--build-psx")
    build = project / ".epok/build"
    assert (build / "GEOMETRY.BIN").stat().st_size > 2*65536 and (build / "epok.cue").is_file()
    with socket.socket() as available:
        available.bind(("127.0.0.1", PORT))
    def request(path, data=None):
        req = urllib.request.Request(f"http://127.0.0.1:{PORT}/api/v1/" + path, data=data)
        with urllib.request.urlopen(req, timeout=5) as response:
            return response.read()
    symbols = (build / "epok.map").read_text()
    def address(name):
        found = re.search(r"0x([0-9a-fA-F]+)\s+" + re.escape(name) + r"\b", symbols)
        assert found, name
        return int(found[1], 16) & 0x1fffff
    command, probe, sequence, streaming = map(address, ("sequence_stress_command", "sequence_stress_probe", "epok::music_sequence_stats", "epok::streaming_stats"))
    report = dict(project=str(project), library=args.library, phases=[])
    with (out / "runtime.log").open("w") as log:
        process = subprocess.Popen([str(EXE), "--project", str(project), "--play-psx", "--stop-after", "45"], stdout=log, stderr=log, creationflags=FLAGS)
        try:
            deadline = time.monotonic() + 65
            while True:
                assert process.poll() is None, (out / "runtime.log").read_text(errors="replace")
                try:
                    ram = request("cpu/ram/raw")
                    values = struct.unpack_from("<16I", ram, probe)
                    if values[0] == 0x53455153 and values[1] >= 10:
                        break
                except (OSError, ValueError):
                    pass
                assert time.monotonic() < deadline, "Stress fixture did not boot"
                time.sleep(0.1)
            for phase in range(6):
                request("execution-flow?function=pause", b"")
                request(f"cpu/ram/raw?offset={command}&size=4", struct.pack("<I", phase))
                request("execution-flow?function=resume", b"")
                deadline = time.monotonic()+15
                while True:
                    ram = request("cpu/ram/raw")
                    values = struct.unpack_from("<16I", ram, probe)
                    stats = struct.unpack_from("<16I", ram, sequence)
                    cd = struct.unpack_from("<7I", ram, streaming)
                    ready = values[2]==phase and values[3]>=35 and not values[11] and values[9]>0
                    ready &= values[4]==(4 if phase==3 else 2)
                    if phase==1: ready &= stats[6]>0 and values[14]==24
                    if phase==5: ready &= values[12]!=values[13] and values[15]==1
                    assert not values[6] and not values[8] and not stats[11] and not stats[12], (phase, values, stats, cd)
                    if ready: break
                    assert time.monotonic()<deadline, ("Stress phase timeout", phase, values, stats, cd)
                    time.sleep(0.1)
                row = dict(phase=phase, probe=values, sequence=stats, geometry=cd, max_irq_us=stats[14]*625/2646)
                report["phases"].append(row)
                save(out / "report.json", report)
                assert values[10] == 1 and cd[4:6] == (0,0), row
                assert row["max_irq_us"] <= 4000 and stats[13] <= 5000, ("Stress CPU/gap gate", row)
                if phase==0:
                    before=cd[0];time.sleep(0.8)
                    later=struct.unpack_from("<7I", request("cpu/ram/raw"), streaming)
                    assert later[0]==before, ("Sequence caused continuing CD reads at stationary geometry", before, later)
                if phase==2: assert cd[0]>report["phases"][0]["geometry"][0], row
                print(f"PASS stress phase {phase}: voices={stats[9]}, steals={stats[5]}, refused={stats[6]}, geometry reads={cd[0]}, max IRQ={row['max_irq_us']:.1f} us", flush=True)
        finally:
            process.wait(timeout=55)
    print(f"PASS MIDI+SFX pressure, geometry/CD, XA transitions and scene retirement: {out}", flush=True)


if __name__ == "__main__":
    main()
