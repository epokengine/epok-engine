"""Exercise cooked scene and transient effects on MIPS and rebuild standalone export.

This is the Phase 3 runtime slice, not acceptance of the unfinished effect editor.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

from verify_blueprints import ROOT, documents, identity, run, write
from verify_blueprint_performance import tool


def create(root):
    run("--create-project", root, "--name", "Effect Runtime Acceptance")
    path = Path(run("--project", root, "--new-particle-effect", "EffectProbe").strip())
    effect = json.loads(path.read_text())
    emitter = effect["layers"][0]["content"]["emitter"]
    emitter.update(continuous=False, play_on_start=False, lifetime=0.25)
    layer = copy.deepcopy(effect["layers"][0])
    layer.update(id=identity(), slot=identity(), name="Flash", content={"kind": "sprite", "sprite": emitter["sprite"], "frames": 1, "columns": 1, "frame_ticks": 410})
    effect["layers"].append(layer)
    slot = copy.deepcopy(effect["timeline"]["slots"][0])
    slot.update(id=layer["slot"], name="Flash")
    effect["timeline"]["slots"].append(slot)
    effect["timeline"]["duration_ticks"] = 4096
    effect["timeline"]["tracks"] = [{"id": identity(), "name": "Size", "slot": layer["slot"],
        "property": "dbd149e3-2164-4bd8-87bc-92f53bdc0f74", "value_type": {"kind": "fixed"},
        "priority": 0, "blend": "absolute", "restore": "leave_final", "interpolation": "linear",
        "keys": [{"id": identity(), "tick": 0, "value": 0}, {"id": identity(), "tick": 4096, "value": 2}]}]
    effect["timeline"]["events"] = [{"id": identity(), "name": "Burst", "slot": effect["layers"][0]["slot"],
        "function": "c2650304-c940-408c-a8a8-05641c2a0489", "keys": [{"id": identity(), "tick": 2048,
        "arguments": {"count": {"kind": "literal", "value_type": {"kind": "uint32"}, "value": 8}}}]}]
    effect["timeline"]["markers"] = [{"id": identity(), "name": "Impact", "tick": 2048}]
    write(path, effect)
    class_id = identity()
    write(root / "assets/scripts/EffectMonitor.hpp", '''#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="CLASS") EffectMonitor:public epok::Behaviour {
public:void update(epok::Transform&,epok::Fixed)override;
};
'''.replace('"CLASS"', '"' + class_id + '"'))
    cpp = '''#include "EffectMonitor.hpp"
#include "scene.hh"
#include "particle_effect_service.hpp"
#include "effects/ASSET.hh"
extern "C" {
volatile int32_t effect_runtime_probe[16]={};
__attribute__((noinline,used)) void effect_runtime_done(){asm volatile("nop":::"memory");}
}
namespace {
unsigned ticks=0;epok::effects::Handle first,extra[7];int32_t paused_size=0;bool reused_completion=false;
void check(bool value,unsigned bit){if(!value)effect_runtime_probe[2]=effect_runtime_probe[2]|(1u<<bit);}
}
void EffectMonitor::update(epok::Transform&,epok::Fixed){
    using namespace epok;++ticks;effect_runtime_probe[0]=0x45465833;
    if(performance_stats.simulation_scanlines>uint32_t(effect_runtime_probe[12]))effect_runtime_probe[12]=performance_stats.simulation_scanlines;
    const auto& asset=effects::cooked::asset_STEM::asset;
    if(ticks==1){
        auto* component=effects::component(handle(&entity()));check(component!=nullptr,0);
        if(component)first=component->playback;
        check(effects::pool.state(first)==effects::State::Playing,1);
        effect_runtime_probe[3]=sizeof(effects::pool);effect_runtime_probe[4]=sizeof(effects::components);
        effect_runtime_probe[13]=object_count;
        auto* layer=timeline::layer_resolver(effects::pool.layer(first,SPRITE));
        check(layer&&layer->position[0].raw()==12288&&layer->color[0].raw()==1024,13);
    }
    if(ticks==10){
        Transform transform;
        for(unsigned i=0;i<7;++i)extra[i]=effects::spawn(asset,transform,123);
        check(effects::pool.stats.active==8,2);
        check(effects::spawn(asset,transform).index==0xffff,3);
        check(object_count==uint32_t(effect_runtime_probe[13]),4);
        auto* layer=timeline::layer_resolver(effects::pool.layer(extra[0],SPRITE));
        check(layer&&layer->position[0].raw()==0&&layer->color[0].raw()==4096,14);
    }
    if(ticks==12)for(auto h:extra)effects::stop(h);
    if(ticks==15){
        auto* layer=timeline::layer_resolver(effects::pool.layer(first,SPRITE));
        check(layer!=nullptr,5);if(layer)paused_size=layer->size.raw();effects::pool.pause(first,true);
    }
    if(ticks==20){
        auto* layer=timeline::layer_resolver(effects::pool.layer(first,SPRITE));
        check(layer&&layer->size.raw()==paused_size,6);effects::pool.pause(first,false);
    }
    if(ticks==45){check(particle_stats.spawned==8,7);check(sequence_stats.markers==1&&sequence_stats.events==1,8);}
    if(ticks>45&&!reused_completion&&timeline::sequences.state(effects::pool.sequence(first))==timeline::State::Completed){
        Transform transform;auto replacement=effects::spawn(asset,transform,123);
        check(replacement.index!=0xffff,11);effects::stop(replacement);reused_completion=true;
    }
    if(ticks==125){
        check(reused_completion,12);
        check(effects::pool.state(first)==effects::State::Completed,9);
        check(effects::pool.stats.active==0&&particle_stats.alive==0,10);
        effect_runtime_probe[5]=effects::pool.stats.spawned;effect_runtime_probe[6]=effects::pool.stats.dropped;
        effect_runtime_probe[7]=effects::pool.stats.cancelled;effect_runtime_probe[8]=effects::pool.stats.completed;
        effect_runtime_probe[9]=particle_stats.spawned;effect_runtime_probe[10]=sequence_stats.events;effect_runtime_probe[11]=sequence_stats.markers;
        effect_runtime_probe[1]=1;effect_runtime_done();
    }
}
'''.replace("ASSET", effect["id"]).replace("STEM", effect["id"].replace("-", ""))
    sprite_index = sorted(l["id"] for l in effect["layers"]).index(layer["id"])
    write(root / "assets/scripts/EffectMonitor.cpp", cpp.replace("SPRITE", str(sprite_index)))
    scene_path = root / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["version"] = 4
    scene["entities"][-1]["particle_effect"] = {"version": 2, "asset": effect["id"], "bindings": {}, "enabled": True, "play_on_start": True, "seed": 123,
        "layer_overrides": {layer["id"]: {
            "a8ab3325-89dd-416f-a946-082292c297c6": {"kind":"literal","value_type":{"kind":"vector","length":3},"value":[3,2,1]},
            "c06779d1-42a0-43ae-8712-fbac4a641bcf": {"kind":"literal","value_type":{"kind":"vector","length":3},"value":[0.25,0.5,1]}}}}
    scene["entities"][-1]["script"] = {"name": "EffectMonitor", "class_id": class_id, "properties": {},
        "provider": {"id": "cpp", "version": 1}, "backend": {"id": "native", "version": 1}}
    write(scene_path, scene)
    run("--project", root, "--build-psx")
    # Reordering authoring layers must preserve both table indices and RNG seeds.
    cooked = root / f".epok/build/effects/{effect['id']}.hh"
    before = cooked.read_bytes()
    effect["layers"].reverse()
    effect["timeline"]["slots"].reverse()
    write(path, effect)
    run("--project", root, "--build-psx")
    assert cooked.read_bytes() == before
    elf = root / ".epok/build/epok.elf"
    symbols = {}
    for line in tool("nm", "-S", "--defined-only", elf).splitlines():
        match = re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line)
        if match:
            symbols[match[3]] = int(match[1], 16)
    size = re.search(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+\d+\s+\S+\s+", tool("size", "--format=berkeley", elf), re.MULTILINE)
    metrics = dict(zip(("text", "data", "bss"), map(int, size.groups())))
    export = Path(run("--project", root, "--export-psx").strip())
    env = os.environ.copy()
    env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
        "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")], cwd=export, env=env, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert (export / "epok.ps-exe").read_bytes() == (root / ".epok/build/epok.ps-exe").read_bytes()
    metrics["standalone_identical"] = True
    return symbols, metrics


def emulator(root, symbols):
    portable = root / ".epok/effect-emulator"
    portable.mkdir(parents=True)
    output = portable / "measurements.json"
    script = portable / "measure.lua"
    write(script, '''local ffi=require('ffi')
local probe=ffi.cast('volatile int32_t*',PCSX.getMemPtr()+PROBE)
effect_done=PCSX.addBreakpoint(DONE,'Exec',4,'Effect runtime completion',function()
  local file=assert(io.open(OUTPUT,'w'));file:write('[')
  for i=0,15 do if i>0 then file:write(',') end file:write(tostring(tonumber(probe[i]))) end
  file:write(']');file:close();PCSX.quit(0);return true
end)
'''.replace("PROBE", str(symbols["effect_runtime_probe"] & 0x1fffff)).replace("DONE", str(symbols["effect_runtime_done"])).replace("OUTPUT", json.dumps(output.as_posix())))
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
    assert data[:3] == [0x45465833, 1, 0], data
    assert data[5:12] == [9, 1, 8, 1, 8, 1, 1], data
    return data


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emulator", action="store_true")
    args = parser.parse_args()
    root = Path(tempfile.mkdtemp(prefix="epok-effect-runtime-")) / "Game"
    symbols, report = create(root)
    report["project"] = str(root)
    if args.emulator:
        report["probe"] = emulator(root, symbols)
        report["effect_pool_bytes"] = report["probe"][3]
        report["component_pool_bytes"] = report["probe"][4]
        report["maximum_completed_frame_simulation_scanlines"] = report["probe"][12]
    write(ROOT / "artifacts/timelines/phase3-runtime.json", report)
    print("PASS effect runtime MIPS, stable layer cooking and identical standalone export" + ("; PCSX lifecycle and saturation checks" if args.emulator else "; emulator not requested"), report)


if __name__ == "__main__":
    main()
