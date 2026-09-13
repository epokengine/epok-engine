"""Cook a gate/camera scene, exercise production lifecycle on MIPS and rebuild export.

The emulator runs in its own portable directory. Measurements are linked RAM and
completed-frame simulation scanlines (including other systems), not hardware FPS.
"""
import argparse
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


def create(root):
    run("--create-project", root, "--name", "Timeline Director Acceptance")
    cls, monitor, position, fov, burst = [identity() for _ in range(5)]
    header = '''#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="CLASS") SequenceTarget:public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="POSITION") epok::Fixed position[3]={0.0,3.0,-6.0};
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="FOV") epok::Fixed fov=90.0;
    EPOK_FUNCTION(TimelineCallable,Id="BURST") void burst(int32_t count);
    void update(epok::Transform& transform,epok::Fixed)override {
        for(unsigned i=0;i<3;++i)transform.position[i]=position[i];
        if(entity().camera)entity().camera_settings.field_of_view=fov;
    }
};
class EPOK_CLASS(Blueprintable,Id="MONITOR") SequenceMonitor:public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere) int32_t phase=0;
    void update(epok::Transform&,epok::Fixed)override;
};
'''
    for key, value in dict(CLASS=cls, POSITION=position, FOV=fov, BURST=burst, MONITOR=monitor).items():
        header = header.replace('"' + key + '"', '"' + value + '"')
    write(root / "assets/scripts/SequenceTarget.hpp", header)
    bp_path = Path(run("--project", root, "--new-blueprint", "BP_SequenceTarget", "--parent", cls).strip())
    bp = documents.loads(bp_path.read_text(encoding="utf-8"))
    reflected = json.loads(run("--project", root, "--reflect"))
    # Match blueprint_refs::compact_id; never infer identity from a display name.
    def compact(value):
        return int.from_bytes(hashlib.sha256(value.encode()).digest()[:8], "little")
    assert any(c["id"] == cls for c in reflected["classes"])
    source = Path(run("--project", root, "--new-timeline", "GateAndCamera").strip())
    asset = json.loads(source.read_text())
    gate_slot, camera_slot, marker = identity(), identity(), identity()
    asset["slots"] = [{"id": slot, "name": name, "target": {"kind": "entity_ref", "class": cls}, "required": required}
                      for slot, name, required in [(gate_slot, "Gate", False), (camera_slot, "Camera", True)]]
    asset["tracks"] = []
    for slot, prop, kind, name, start, end, restore in [
        (gate_slot, position, {"kind": "vector", "length": 3}, "Gate lift", [0, 0, 0], [0, 4, 0], "leave_final"),
        (camera_slot, position, {"kind": "vector", "length": 3}, "Camera dolly", [0, 3, -6], [0, 4, -4], "restore_initial"),
        (camera_slot, fov, {"kind": "fixed"}, "Camera lens", 90, 60, "restore_initial"),
    ]:
        asset["tracks"].append({"id": identity(), "name": name, "slot": slot, "property": prop, "value_type": kind,
            "priority": 0, "blend": "absolute", "restore": restore, "interpolation": "linear",
            "keys": [{"id": identity(), "tick": 0, "value": start}, {"id": identity(), "tick": 4096, "value": end}]})
    asset["events"] = [{"id": identity(), "name": "Gate dust", "slot": gate_slot, "function": burst,
        "keys": [{"id": identity(), "tick": 2048, "arguments": {"count": {
            "kind": "literal", "value_type": {"kind": "int32"}, "value": 8}}}]}]
    asset["markers"] = [{"id": marker, "name": "Impact", "tick": 2048}]
    write(source, asset)
    scene_path = root / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    camera = copy.deepcopy(scene["entities"][0])
    gate = copy.deepcopy(camera)
    gate.update(id=identity(), kind="Mesh", position=[0, 0, 0])
    def binding(name, class_id, properties=None, visual=False):
        return {"name": name, "class_id": class_id, "properties": properties or {},
                "provider": {"id": "blueprint" if visual else "cpp", "version": 1},
                "backend": {"id": "native", "version": 1}}
    camera["name"] = "Camera"
    camera["script"] = binding(bp["name"], bp["id"], visual=True)
    gate["name"] = "Gate"
    gate["script"] = binding("SequenceTarget", cls, {"position": [0, 0, 0]})
    gate["particle_emitter"] = {"enabled": True, "play_on_start": False, "continuous": False, "burst": 8}
    director = copy.deepcopy(gate)
    director.update(id=identity(), name="Director", kind="Empty", script=None, particle_emitter=None)
    director["timeline"] = {"version": 1, "asset": asset["id"], "enabled": True, "play_on_start": True,
                            "bindings": {gate_slot: gate["id"], camera_slot: camera["id"]}}
    observer = copy.deepcopy(director)
    observer.update(id=identity(), name="Monitor", timeline=None, script=binding("SequenceMonitor", monitor))
    scene.update(version=4, entities=[camera, gate, director, observer])
    write(scene_path, scene)
    second = copy.deepcopy(scene)
    second.update(name="After", entities=[copy.deepcopy(camera), copy.deepcopy(observer)])
    for entity in second["entities"]:
        entity["id"] = identity()
    second["entities"][1]["script"]["properties"] = {"phase": 1}
    write(root / "assets/scenes/After.epokmap", second)
    write(root / "ProjectSettings/Maps.epoksettings", {"scenes": ["assets/scenes/After.epokmap"]})
    prototype = copy.deepcopy(director)
    prototype.update(id=identity(), name="Template Root")
    prototype["timeline"].update(play_on_start=False, bindings={camera_slot: prototype["id"], gate_slot: None})
    bp["template"]["entities"] = [{"entity": prototype, "parent": None}]
    write(bp_path, bp)
    compiled = json.loads(run("--project", root, "--compile-timelines"))[0]
    gate_index = next(i for i, s in enumerate(compiled["slots"]) if s["id"] == gate_slot)
    cpp = r'''#include "SequenceTarget.hpp"
#include "timeline_service.hpp"
#include "blueprint_spawn.hpp"
extern "C" {
volatile int32_t timeline_director_probe[32]={};
__attribute__((noinline,used)) void timeline_director_done(){asm volatile("nop":::"memory");}
}
namespace {
using namespace epok;
unsigned ticks=0;timeline::Handle playback;EntityHandle dynamic,replacement,owner;
int32_t frozen=0;bool requested=false;
void check(bool result,unsigned bit){if(!result)timeline_director_probe[2]=timeline_director_probe[2]|(1u<<bit);}
SequenceTarget* target(EntityHandle value){return static_cast<SequenceTarget*>(bp::behaviour(value));}
}
void SequenceTarget::burst(int32_t count){
    timeline_director_probe[3]=timeline_director_probe[3]+1;timeline_director_probe[4]=position[1].raw();
    entity().particle_emitter.burst(uint16_t(count));
}
void SequenceMonitor::update(epok::Transform&,epok::Fixed){
    using namespace epok;
    timeline_director_probe[0]=0x544c4432;
    if(phase==1){
        check(requested&&sequence_stats.active==0&&sequence_stats.cancelled>=2,12);
        timeline_director_probe[1]=1;
        timeline_director_probe[12]=sequence_stats.cancelled;
        timeline_director_probe[13]=sequence_stats.skipped_targets;
        timeline_director_done();return;
    }
    ++ticks;timeline_director_probe[5]=ticks;
    if(performance_stats.simulation_scanlines>uint32_t(timeline_director_probe[15]))timeline_director_probe[15]=performance_stats.simulation_scanlines;
    if(ticks==1){owner=handle(find_entity("Director"));playback=timeline::component(owner)->playback;}
    if(ticks==62){
        check(timeline::sequences.state(playback)==timeline::State::Completed,0);
        check(timeline_director_probe[3]==1&&timeline_director_probe[4]==8192,1);
        check(timeline::sequences.marker_revision(playback,MARKERull)==1,2);
        auto* camera=target(handle(find_entity("Camera")));
        check(camera&&camera->fov.raw()==90*4096&&camera->position[2].raw()==-6*4096,3);
        check(target(handle(find_entity("Gate")))->position[1].raw()==4*4096,4);
        timeline_director_probe[6]=sizeof(timeline::sequences);
        timeline_director_probe[7]=sizeof(timeline::components);
        timeline_director_probe[8]=particle_stats.spawned;
    }
    if(ticks==65){
        dynamic=bp::spawn(CLASSull,"Dynamic Gate");check(dynamic.get()!=nullptr,5);
        auto* spawned=timeline::component(dynamic);
        check(spawned&&!spawned->automatic&&spawned->targets[CAMERA].get()==dynamic.get(),16);
        auto* component=timeline::component(owner);component->targets[GATE]=dynamic;
        playback=timeline::play(owner);check(timeline::sequences.state(playback)==timeline::State::Playing,6);
    }
    if(ticks==70){
        const auto old=dynamic;check(destroy_entity(old.get()),7);replacement=bp::spawn(CLASSull,"Replacement");
        check(replacement.get()&&replacement.index==old.index&&replacement.generation!=old.generation&&!old.get(),8);
        if(replacement.get())target(replacement)->position[1]=77.0;
    }
    if(ticks==75){frozen=timeline::sequences.tick(playback);set_active(owner.get(),false);}
    if(ticks==80){check(timeline::sequences.tick(playback)==frozen,9);set_active(owner.get(),true);}
    if(ticks==135){
        check(timeline::sequences.state(playback)==timeline::State::Completed&&timeline::sequences.marker_revision(playback,MARKERull)==1,10);
        check(replacement.get()&&target(replacement)->position[1].raw()==77*4096&&timeline_director_probe[3]==1,11);
        auto* component=timeline::component(owner);
        const auto* asset=component->asset;EntityHandle targets[8];for(unsigned i=0;i<8;++i)targets[i]=component->targets[i];
        playback=timeline::play(owner);destroy_entity(owner.get());
        check(timeline::sequences.state(playback)==timeline::State::Cancelled,13);
        playback=timeline::sequences.play(*asset,handle(&entity()),targets,blueprint_scene_generation);
        check(timeline::sequences.state(playback)==timeline::State::Playing,14);
        requested=request_scene("After");check(requested,15);
    }
    timeline_director_probe[9]=sequence_stats.events;
    timeline_director_probe[10]=sequence_stats.markers;
    timeline_director_probe[11]=sequence_stats.completed;
    timeline_director_probe[14]=sequence_stats.diagnostics_dropped;
    if(ticks>160){timeline_director_probe[1]=-1;timeline_director_done();}
}
'''.replace("MARKER", str(compact(marker))).replace("CLASS", str(compact(bp["id"]))).replace("GATE", str(gate_index))
    cpp = cpp.replace("CAMERA", str(next(i for i, s in enumerate(compiled["slots"]) if s["id"] == camera_slot)))
    write(root / "assets/scripts/SequenceTarget.cpp", cpp)
    run("--project", root, "--build-psx")
    symbols = {}
    elf = root / ".epok/build/epok.elf"
    for line in tool("nm", "-S", "--defined-only", elf).splitlines():
        match = re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line)
        if match:
            symbols[match[3]] = (int(match[1], 16), int(match[2], 16))
    size = tool("size", "--format=berkeley", elf)
    match = re.search(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+\d+\s+\S+\s+", size, re.MULTILINE)
    metrics = dict(zip(("text", "data", "bss"), map(int, match.groups())))
    export = Path(run("--project", root, "--export-psx").strip())
    env = os.environ.copy()
    env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
        "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
        cwd=export, env=env, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert (export / "epok.ps-exe").read_bytes() == (root / ".epok/build/epok.ps-exe").read_bytes()
    metrics["standalone_identical"] = True
    return symbols, metrics


def emulator(root, symbols):
    portable = root / ".epok/director-emulator"
    portable.mkdir(parents=True)
    output = portable / "measurements.json"
    script = portable / "measure.lua"
    write(script, '''local ffi=require('ffi')
local probe=ffi.cast('volatile int32_t*',PCSX.getMemPtr()+PROBE)
timeline_done=PCSX.addBreakpoint(DONE,'Exec',4,'Timeline director completion',function()
  local file=assert(io.open(OUTPUT,'w'));file:write('[')
  for i=0,31 do if i>0 then file:write(',') end file:write(tostring(tonumber(probe[i]))) end
  file:write(']');file:close();PCSX.quit(0);return true
end)
'''.replace("PROBE", str(symbols["timeline_director_probe"][0] & 0x1fffff))
       .replace("DONE", str(symbols["timeline_director_done"][0])).replace("OUTPUT", json.dumps(output.as_posix())))
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
    assert data[:3] == [0x544c4432, 1, 0], data
    assert data[8] == 8 and data[11] == 2 and data[12] >= 2 and data[13] > 0, data
    return data


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emulator", action="store_true")
    args = parser.parse_args()
    os.environ["EPOK_RUNTIME_OPT"] = "-Os"
    root = Path(tempfile.mkdtemp(prefix="epok-timeline-director-")) / "Game"
    print("Retained director fixture:", root, flush=True)
    symbols, report = create(root)
    if args.emulator:
        report["probe"] = emulator(root, symbols)
        report["director_pool_bytes"] = report["probe"][6]
        report["component_pool_bytes"] = report["probe"][7]
        report["maximum_completed_frame_simulation_scanlines"] = report["probe"][15]
    report["fixture"] = str(root)
    write(ROOT / "artifacts/timelines/phase2.json", report)
    print("PASS production scene director MIPS and identical standalone rebuild" + ("; gate/camera lifecycle emulator acceptance" if args.emulator else "; emulator not requested"))


if __name__ == "__main__":
    main()
