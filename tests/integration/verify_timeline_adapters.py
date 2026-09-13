"""Install real component adapters, cook scene bindings, and verify MIPS side effects.

Uses the production Clang catalog and generated property/event adapters. Runtime
assertions inspect actual components, not their proxy script fields. Fixtures are
retained; the standalone executable must rebuild byte-identically.
"""
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

from verify_blueprints import ROOT, documents, identity, run, write
from verify_blueprint_performance import tool


def main():
    root = Path(tempfile.mkdtemp(prefix="epok-timeline-adapters-")) / "Game"
    print("Retained component-adapter fixture:", root, flush=True)
    run("--create-project", root, "--name", "Timeline Component Adapters")
    source = Path(run("--project", root, "--install-timeline-adapters").strip())
    original = source.read_bytes()
    for value, property_id in re.findall(r"case UINT64_C\((\d+)\): // ([0-9a-f-]+)", original.decode()):
        assert int(value) == int.from_bytes(hashlib.sha256(property_id.encode()).digest()[:8], "little")
    stamp = source.stat().st_mtime_ns
    run("--project", root, "--install-timeline-adapters")
    assert source.stat().st_mtime_ns == stamp
    write(source, original.decode() + "\n// Project-owned addition\n")
    assert "preserved" in run("--project", root, "--install-timeline-adapters", ok=False)
    assert source.read_text().endswith("// Project-owned addition\n")
    write(source, original.decode())
    manifest = json.loads(run("--project", root, "--reflect"))
    assert manifest["schema_version"] == 6
    classes = {c["cpp_name"]: c for c in manifest["classes"]}
    expected = {"TimelineTransform": None, "TimelineCamera": "Camera", "TimelineAudio": "AudioSource",
        "TimelineLight": "Light", "TimelinePalette": "PaletteAnimator", "TimelineEmitter": "ParticleEmitter",
        "TimelineRect": "RectTransform", "TimelineText": "Text", "TimelineImage": "Image", "TimelineProgress": "ProgressBar"}
    for name, requirement in expected.items():
        assert classes[name]["timeline_component"] == requirement, classes[name]
        assert not classes[name]["abstract_class"]
        assert all(p["timeline"] is not None for p in classes[name]["properties"])
        assert all(f["name"] != "timeline_sync" for f in classes[name]["functions"])
    for replacement, diagnostic in [("TimelineRequires=UnknownComponent", "Unknown TimelineRequires"),
                                    ("TimelineRequires=Camera,TimelineRequires=AudioSource", "only one TimelineRequires")]:
        write(source, original.decode().replace("TimelineRequires=Camera", replacement))
        assert diagnostic in run("--project", root, "--reflect", ok=False)
    write(source, original.decode())
    assert json.loads(run("--project", root, "--reflect"))["classes"] == manifest["classes"]
    bp_path = Path(run("--project", root, "--new-blueprint", "BP_TimelineCamera", "--parent", classes["TimelineCamera"]["id"]).strip())
    bp = documents.loads(bp_path.read_text(encoding="utf-8"))
    monitor_id, impact_id = identity(), identity()
    write(root / "assets/scripts/AdapterMonitor.hpp", f'''#pragma once
#include "TimelineAdapters.hpp"
class EPOK_CLASS(Blueprintable,Id="{monitor_id}") AdapterMonitor:public epok::Behaviour {{
public:
    EPOK_FUNCTION(TimelineCallable,Id="{impact_id}") void impact();
    void update(epok::Transform&,epok::Fixed)override;
}};
''')
    scene_path = root / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    prototype = copy.deepcopy(scene["entities"][0])
    entities = {}
    for name in expected:
        entity = copy.deepcopy(prototype)
        entity.update(id=identity(), name=name, kind="Empty", position=[0, 0, 0], rotation=[0, 0, 0], scale=[1, 1, 1])
        entity["script"] = {"name": name, "class_id": classes[name]["id"], "properties": {},
            "provider": {"id": "cpp", "version": 1}, "backend": {"id": "native", "version": 1}}
        entities[name] = entity
    entities["TimelineTransform"]["position"] = [13, 0, 0]
    entities["TimelineCamera"].update(kind="Camera", camera_fov=107, position=[0, 3, -6])
    entities["TimelineCamera"]["script"].update(name=bp["name"], class_id=bp["id"], provider={"id": "blueprint", "version": 1})
    entities["TimelineAudio"]["audio"] = {"volume": 0.8, "pitch": 1, "play_on_start": False}
    entities["TimelineLight"]["light"] = {"enabled": True}
    entities["TimelinePalette"]["palette_animator"] = {"enabled": True}
    entities["TimelineEmitter"]["particle_emitter"] = {"enabled": True, "play_on_start": False, "continuous": False}
    for name in ("TimelineRect", "TimelineText", "TimelineImage", "TimelineProgress"):
        entities[name]["rect"] = {}
    entities["TimelineText"]["text"] = {"enabled": True, "text": "Timeline adapters"}
    entities["TimelineImage"]["image"] = {"enabled": True}
    entities["TimelineProgress"]["progress"] = {"enabled": True}
    monitor = copy.deepcopy(entities["TimelineTransform"])
    monitor.update(id=identity(), name="Monitor", position=[0, 0, 0])
    monitor["script"].update(name="AdapterMonitor", class_id=monitor_id)
    entities["Monitor"] = monitor
    canvas = copy.deepcopy(monitor)
    canvas.update(id=identity(), name="Canvas", script=None, canvas={"enabled": True})
    canvas_index = len(entities)
    entities["Canvas"] = canvas
    for name in ("TimelineRect", "TimelineText", "TimelineImage", "TimelineProgress"):
        entities[name]["parent"] = canvas_index
    directors = []
    def prop(class_name, field):
        return next(p for p in classes[class_name]["properties"] if p["name"] == field)
    def make_sequence(label, names, tracks, events):
        path = Path(run("--project", root, "--new-timeline", label).strip())
        asset = json.loads(path.read_text())
        asset["duration_ticks"] = 4096
        slots = {name: identity() for name in names}
        asset["slots"] = [{"id": slots[name], "name": name, "target": {"kind": "entity_ref", "class":
            bp["id"] if name == "TimelineCamera" else monitor_id if name == "Monitor" else classes[name]["id"]},
            "required": name != "TimelinePalette"} for name in names]
        asset["tracks"] = []
        for target, declaring, field, start, end, restore in tracks:
            reflected = prop(declaring, field)
            asset["tracks"].append({"id": identity(), "name": field, "slot": slots[target], "property": reflected["id"],
                "value_type": reflected["value_type"], "priority": 0, "blend": "absolute", "restore": restore,
                "interpolation": "step" if isinstance(start, bool) else "linear",
                "keys": [{"id": identity(), "tick": 0, "value": start}, {"id": identity(), "tick": 4096, "value": end}]})
        asset["events"] = []
        for target, action, tick in events:
            function = impact_id if target == "Monitor" else next(f["id"] for f in classes[target]["functions"] if f["name"] == action)
            asset["events"].append({"id": identity(), "name": action, "slot": slots[target], "function": function,
                "keys": [{"id": identity(), "tick": tick, "arguments": {"count": {
                    "kind": "literal", "value_type": {"kind": "uint32"}, "value": 300}} if action == "burst" else {}}]})
        write(path, asset)
        director = copy.deepcopy(monitor)
        director.update(id=identity(), name=label, script=None)
        director["timeline"] = {"version": 1, "asset": asset["id"], "enabled": True, "play_on_start": True,
            "bindings": {slots[name]: entities[name]["id"] for name in names}}
        directors.append(director)
    leave, restore = "leave_final", "restore_initial"
    make_sequence("WorldAdapters", list(expected)[:6] + ["Monitor"], [
        ("TimelineTransform", "TimelineTransform", "position", [0, 0, 0], [4, 0, 0], restore),
        ("TimelineCamera", "TimelineCamera", "field_of_view", 90, 60, restore),
        ("TimelineAudio", "TimelineAudio", "volume", 0, 1, leave),
        ("TimelineAudio", "TimelineAudio", "pitch", 1, 2, leave),
        ("TimelineLight", "TimelineLight", "intensity", 0, 2, leave),
        ("TimelineLight", "TimelineLight", "color", [1, 1, 1], [1, 0, 0], restore),
        ("TimelinePalette", "TimelinePalette", "speed", 0, 4, leave),
        ("TimelinePalette", "TimelinePalette", "reverse", False, True, leave),
        ("TimelineEmitter", "TimelineEmitter", "rate", 0, 16, leave),
    ], [("Monitor", "impact", 2048), ("TimelinePalette", "reset", 2048), ("TimelineEmitter", "play", 0), ("TimelineEmitter", "burst", 256), ("TimelineEmitter", "stop", 4096)])
    make_sequence("HudAdapters", list(expected)[6:], [
        ("TimelineRect", "TimelineRect", "position", [0, 0], [8, 12], leave),
        ("TimelineText", "TimelineRect", "size", [100, 32], [160, 48], leave),
        ("TimelineText", "TimelineText", "color", [1, 1, 1], [0, 1, 0], leave),
        ("TimelineText", "TimelineText", "wrap", True, False, leave),
        ("TimelineImage", "TimelineImage", "color", [1, 1, 1], [0, 0, 1], leave),
        ("TimelineProgress", "TimelineProgress", "value", 0, 0.5, leave),
        ("TimelineProgress", "TimelineProgress", "background", [0, 0, 0], [1, 0, 1], leave),
    ], [])
    scene.update(version=4, entities=list(entities.values()) + directors)
    write(scene_path, scene)
    # Required component failures use the real scene cooker and inherited class metadata.
    light = entities["TimelineLight"].pop("light")
    write(scene_path, scene)
    assert "requires an enabled Light" in run("--project", root, "--build-psx", ok=False)
    entities["TimelineLight"]["light"] = light
    # A Blueprint inherits the camera requirement through native ancestry.
    entities["TimelineCamera"]["kind"] = "Empty"
    write(scene_path, scene)
    assert "requires an enabled Camera" in run("--project", root, "--build-psx", ok=False)
    entities["TimelineCamera"]["kind"] = "Camera"
    write(scene_path, scene)
    write(root / "assets/scripts/AdapterMonitor.cpp", r'''#include "AdapterMonitor.hpp"
#include "timeline_service.hpp"
extern "C" {
volatile int32_t timeline_adapter_probe[16]={};
__attribute__((noinline,used)) void timeline_adapter_done(){asm volatile("nop":::"memory");}
}
namespace {
using namespace epok;
unsigned ticks=0,impacts=0;timeline::Handle playback;
void check(bool value,unsigned bit){if(!value)timeline_adapter_probe[2]=timeline_adapter_probe[2]|(1u<<bit);}
Entity& object(const char* name){return *find_entity(name);}
}
void AdapterMonitor::impact(){
    ++impacts;
    check(object("TimelineTransform").transform.position[0].raw()==2*4096,0);
    check(object("TimelineCamera").camera_settings.field_of_view.raw()==75*4096,1);
    check(object("TimelineAudio").audio.volume.raw()==2048,2);
}
void AdapterMonitor::update(epok::Transform&,epok::Fixed){
    ++ticks;
    timeline_adapter_probe[0]=0x544c4131;
    if(ticks==65){
        check(impacts==1,3);
        check(object("TimelineTransform").transform.position[0].raw()==13*4096,4);
        check(object("TimelineCamera").camera_settings.field_of_view.raw()==107*4096,5);
        check(object("TimelineAudio").audio.volume.raw()==4096&&object("TimelineAudio").audio.pitch.raw()==8192,6);
        check(object("TimelineLight").light.intensity.raw()==8192&&object("TimelineLight").light.color[1]==255,7);
        check(object("TimelinePalette").palette_animator.speed.raw()==16384&&object("TimelinePalette").palette_animator.reverse,8);
        check(object("TimelineEmitter").particle_emitter.rate.raw()==65536&&!object("TimelineEmitter").particle_emitter.playing,9);
        check(object("TimelineRect").rect.position[0].raw()==8*4096&&object("TimelineRect").rect.position[1].raw()==12*4096,10);
        check(object("TimelineText").rect.size[0].raw()==160*4096&&object("TimelineText").text.color[0]==0&&!object("TimelineText").text.wrap,11);
        check(object("TimelineImage").image.color[2]==255&&object("TimelineImage").image.color[0]==0,12);
        check(object("TimelineProgress").progress.value.raw()==2048&&object("TimelineProgress").progress.background[2]==255,13);
        object("TimelinePalette").palette_animator.enabled=false;
        playback=timeline::play(handle(&object("WorldAdapters")));
        check(timeline::sequences.state(playback)==timeline::State::Playing,14);
    }
    if(ticks==68){
        // Destroying a required target after play never writes through its old handle.
        destroy_entity(&object("TimelineCamera"));
    }
    if(ticks==70){
        check(sequence_stats.skipped_targets>0,15);
        destroy_entity(&object("WorldAdapters"));
        check(timeline::sequences.state(playback)==timeline::State::Cancelled,16);
        timeline_adapter_probe[1]=1;
        timeline_adapter_probe[3]=impacts;
        timeline_adapter_probe[4]=sequence_stats.skipped_targets;
        timeline_adapter_probe[5]=sequence_stats.cancelled;
        timeline_adapter_probe[6]=sequence_stats.events;
        timeline_adapter_probe[7]=particle_stats.dropped;
        timeline_adapter_done();
    }
}
''')
    run("--project", root, "--build-psx")
    elf = root / ".epok/build/epok.elf"
    symbols = {}
    for line in tool("nm", "-S", "--defined-only", elf).splitlines():
        match = re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line)
        if match:
            symbols[match[3]] = (int(match[1], 16), int(match[2], 16))
    size = tool("size", "--format=berkeley", elf)
    match = re.search(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+\d+\s+\S+\s+", size, re.MULTILINE)
    report = dict(zip(("text", "data", "bss"), map(int, match.groups())))
    export = Path(run("--project", root, "--export-psx").strip())
    env = os.environ.copy()
    env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
        "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
        cwd=export, env=env, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert (export / "epok.ps-exe").read_bytes() == (root / ".epok/build/epok.ps-exe").read_bytes()
    report["standalone_identical"] = True
    portable = root / ".epok/adapter-emulator"
    portable.mkdir()
    output, script = portable / "measurements.json", portable / "measure.lua"
    write(script, '''local ffi=require('ffi')
local probe=ffi.cast('volatile int32_t*',PCSX.getMemPtr()+PROBE)
adapter_done=PCSX.addBreakpoint(DONE,'Exec',4,'Timeline adapters complete',function()
  local file=assert(io.open(OUTPUT,'w'));file:write('[')
  for i=0,15 do if i>0 then file:write(',') end file:write(tostring(tonumber(probe[i]))) end
  file:write(']');file:close();PCSX.quit(0);return true
end)
'''.replace("PROBE", str(symbols["timeline_adapter_probe"][0] & 0x1fffff))
       .replace("DONE", str(symbols["timeline_adapter_done"][0])).replace("OUTPUT", json.dumps(output.as_posix())))
    executable = ROOT / ".tools/redux/pcsx-redux.exe"
    with (portable / "emulator.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(executable), "-portable", str(portable), "-run", "-stdout", "-interpreter", "-softgpu",
            "-2mb", "-no-gdb", "-loadexe", str(root / ".epok/build/epok.ps-exe"), "-bios", str(executable.parent / "openbios.bin"),
            "-dofile", str(script), "-debugger"], cwd=portable, stdout=log, stderr=log, stdin=subprocess.DEVNULL,
            creationflags=0x08000000 if os.name == "nt" else 0)
        try:
            assert process.wait(timeout=150) == 0
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    data = json.loads(output.read_text())
    assert data[:4] == [0x544c4131, 1, 0, 1], data
    assert data[4] > 0 and data[5] == 1 and data[7] >= 44, data
    report.update(probe=data, fixture=str(root))
    write(ROOT / "artifacts/timelines/phase2-component-adapters.json", report)
    print("PASS reusable adapters: real reflection/inheritance, required-component cooking, component writes at event time, restoration, optional skips and destruction; MIPS, identical standalone export and PCSX.")


if __name__ == "__main__":
    main()
