"""Measure bounded VFX loads with geometry/HUD through real MIPS and standalone export.

All phases share one instrumented executable and collect completed native frames.
Capacity checks are correctness gates, not a promised frame rate for every game.
"""
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import statistics
import subprocess
import tempfile
import uuid

from verify_blueprints import ROOT, documents, run, write
from verify_blueprint_performance import tool


FIELDS = ("magic phase frame frame_scanlines simulation_scanlines render_scanlines steps dropped_steps "
          "sequences peak_sequences dropped_sequences conflicts skipped_targets events markers properties "
          "effects layers dropped_effects dropped_bursts cancelled_effects "
          "particles peak_particles spawned_particles dropped_particles dropped_emitters "
          "sprites sprite_triangles dropped_sprites sprite_pixels objects failures "
          "sequence_pool_bytes effect_pool_bytes particle_pool_bytes").split()


def ident(name):
    return str(uuid.uuid5(uuid.NAMESPACE_URL, "epok.validation.vfx-load/" + name))


def create(root):
    shutil.copytree(ROOT / "examples/timeline-spell", root,
                    ignore=shutil.ignore_patterns(".epok", "exports", "Local.epokconfig"))
    source = json.loads((root / "assets/Effects/Fireball.particle-effect.json").read_text())
    effect = copy.deepcopy(source)
    effect.update(id=ident("effect"), name="VFX load acceptance", layers=[])
    timeline = effect["timeline"]
    timeline.update(id=ident("timeline"), name="VFX load timeline", duration_ticks=60 * 4096,
                    slots=[], tracks=[], events=[], markers=[{"id": ident("marker"), "name": "Started", "tick": 0}])
    template = next(layer for layer in source["layers"] if layer["content"]["kind"] == "emitter")
    for index in range(8):
        layer = copy.deepcopy(template)
        layer.update(id=ident(f"layer/{index}"), slot=ident(f"slot/{index}"), name=f"Emitter {index + 1}",
                     position=[(index % 4 - 1.5) * .12, .7 + (index // 4) * .12, 0])
        emitter = layer["content"]["emitter"]
        emitter.update(play_on_start=True, continuous=False, burst=0, max_particles=4,
                       lifetime=60, velocity=[0, 0, 0], spread=[.01, .01, .01], gravity=[0, 0, 0],
                       start_size=.15, end_size=.15, frames=1, frame_columns=1)
        effect["layers"].append(layer)
        timeline["slots"].append({"id": layer["slot"], "name": layer["name"],
                                  "target": {"kind": "effect_layer_ref", "class": "27550d6d-5bba-4618-9d9e-33b9605bfb6c"},
                                  "required": True})
        # Eight vector properties plus eight event tracks reach the shared
        # sixteen-track authoring limit, with 24 evaluated scalar channels.
        timeline["tracks"].append({"id": ident(f"track/{index}"), "name": f"{layer['name']} Position",
            "slot": layer["slot"], "property": "a8ab3325-89dd-416f-a946-082292c297c6",
            "value_type": {"kind": "vector", "length": 3}, "priority": 0,
            "blend": "absolute", "restore": "leave_final", "interpolation": "linear",
            "keys": [{"id": ident(f"key/{index}/{tick}"), "tick": tick, "value": layer["position"]}
                     for tick in (0, 60 * 4096)]})
        timeline["events"].append({"id": ident(f"event/{index}"), "name": "Initial four particles",
            "slot": layer["slot"], "function": "c2650304-c940-408c-a8a8-05641c2a0489",
            "keys": [{"id": ident(f"burst/{index}"), "tick": 0,
                      "arguments": {"count": {"kind": "literal", "value_type": {"kind": "uint32"}, "value": 4}}}]})
    write(root / "assets/Effects/Load.particle-effect.json", effect)
    write(root / "assets/scripts/LoadProbe.hpp", '''#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="CLASS") LoadProbe:public epok::Behaviour {
public:
    void update(epok::Transform&,epok::Fixed)override{}
    void frame_update(epok::Transform&,uint32_t)override;
};
'''.replace('Id="CLASS"', f'Id="{ident("class")}"'))
    write(root / "assets/scripts/LoadProbe.cpp", '''#include "LoadProbe.hpp"
#include "scene.hh"
#include "particle_effect_service.hpp"
#include "particles.hpp"
#include "sprites.hpp"
#include "effects/ASSET.hh"
extern "C" {
volatile uint32_t vfx_load_probe[35]={};
__attribute__((noinline,used)) void vfx_load_capture(){asm volatile("nop":::"memory");}
}
namespace {
unsigned frame=0,phase=0,failures=0,objects_before=0;
epok::effects::Handle handles[8];
void check(bool value,unsigned bit){if(!value)failures|=1u<<bit;}
void compare_planes(){
    using namespace epok;sprite_detail::PlaneCache cache;
    for(int sample=0;sample<24;++sample){
        auto world=Affine<Fixed>::identity(),view=Affine<Fixed>::identity();
        for(int r=0;r<3;++r){
            world.values[r][3]=Fixed((sample-12)*(r+1)*311,Fixed::RAW);
            view.values[r][3]=Fixed((sample-7)*(r+2)*217,Fixed::RAW);
            if(sample%3)for(int c=0;c<3;++c)world.values[r][c]=Fixed((sample-9)*(r+1)*(c+2)*293,Fixed::RAW);
            if(sample%4)for(int c=0;c<3;++c)view.values[r][c]=Fixed((sample-13)*(r+2)*(c+1)*179,Fixed::RAW);
        }
        for(unsigned orientation=0;orientation<3;++orientation){Sprite sprite;sprite.orientation=SpriteOrientation(orientation);
            for(unsigned repeat=0;repeat<2;++repeat){
                Affine<Fixed> model,camera;cache.transform(sprite,world,view,model,camera);
                const auto reference=sprite_detail::plane(sprite,world,view),expected=view.compose(reference);
                for(int r=0;r<3;++r)for(int c=0;c<4;++c)
                    check(model.values[r][c].raw()==reference.values[r][c].raw()&&camera.values[r][c].raw()==expected.values[r][c].raw(),9);
            }
        }
        if(sample%5==0)cache.clear();
    }
}
void stop_all(){for(auto handle:handles)epok::effects::stop(handle);}
void spawn(unsigned count){
    using namespace epok;
    for(unsigned i=0;i<count;++i){Transform t;t.position[0]=Fixed(int(i%4)*4096-6144,Fixed::RAW);t.position[2]=Fixed(int(i/4)*2048,Fixed::RAW);
        handles[i]=effects::spawn(effects::cooked::asset_STEM::asset,t,4242+i);
        check(handles[i].index!=0xffff,0);
    }
}
}
void LoadProbe::frame_update(epok::Transform&,uint32_t){
    using namespace epok;++frame;
    if(frame==1){objects_before=object_count;compare_planes();}
    if(frame>5){
        const uint32_t values[]={0x56465835,phase,performance_stats.frame,
            performance_stats.frame_scanlines,performance_stats.simulation_scanlines,
            performance_stats.render_scanlines,performance_stats.steps,time.dropped_steps,
            sequence_stats.active,sequence_stats.peak,sequence_stats.dropped,sequence_stats.conflicts,
            sequence_stats.skipped_targets,sequence_stats.events,sequence_stats.markers,sequence_stats.properties,
            effect_stats.active,effect_stats.active_layers,effect_stats.dropped,effect_stats.dropped_bursts,effect_stats.cancelled,
            particle_stats.alive,particle_stats.peak,particle_stats.spawned,particle_stats.dropped,particle_stats.dropped_emitters,
            sprite_stats.submitted,sprite_stats.triangles,sprite_stats.dropped,sprite_stats.estimated_pixels,
            object_count,failures,sizeof(timeline::sequences),sizeof(effects::pool),sizeof(ParticlePool)};
        for(unsigned i=0;i<35;++i)vfx_load_probe[i]=values[i];
        vfx_load_capture();
    }
    if(frame==30){spawn(1);phase=1;}
    if(frame==100){check(effect_stats.active==1&&particle_stats.alive==32,1);stop_all();phase=0;}
    if(frame==120){spawn(8);phase=2;}
    if(frame==190){
        check(effect_stats.active==8&&effect_stats.active_layers==64&&particle_stats.alive==256,2);
        Transform t;check(effects::spawn(effects::cooked::asset_STEM::asset,t).index==0xffff,3);
        for(auto handle:handles)check(effects::pool.burst(handle,1024),4);
        phase=3;
    }
    if(frame==210){
        check(effect_stats.dropped==1&&effect_stats.dropped_bursts==49152&&particle_stats.dropped==16384,5);
        entity().particle_emitter.enabled=true;phase=4;
    }
    if(frame==240){
        check(particle_stats.dropped_emitters>0&&particle_stats.alive<=256,6);
        stop_all();entity().particle_emitter.enabled=false;phase=5;
    }
    if(frame==260){
        check(effect_stats.active==0&&sequence_stats.active==0&&particle_stats.alive==0,7);
        check(object_count==objects_before,8);phase=6;
    }
}
'''.replace("ASSET", effect["id"]).replace("STEM", effect["id"].replace("-", "")))
    scene_path = root / "assets/scenes/Combat.epokmap"
    scene = documents.loads(scene_path.read_text())
    for entity in scene["entities"]:
        entity.pop("script", None)
    scene["entities"][5]["text"]["text"] = "VFX capacity test: 1 effect / 8 effects / overflow / cleanup"
    # Ordinary native geometry and the existing textured characters/HUD remain
    # present in every measured phase. No special runtime renderer is substituted.
    for z in range(3):
        for x in range(4):
            scene["entities"].append({"id": ident(f"mesh/{x}/{z}"), "name": f"Block {x} {z}", "kind": "Mesh",
                "position": [(x - 1.5) * 1.3, -.25, z * .8 + .5], "rotation": [0, 0, 0], "scale": [.6, .5, .6],
                "material": {"color": [.3, .4, .5], "unlit": True}})
    extra = copy.deepcopy(template["content"]["emitter"])
    extra.update(enabled=False, play_on_start=True, continuous=False, burst=4, max_particles=4, lifetime=60,
                 velocity=[0, 0, 0], spread=[0, 0, 0], gravity=[0, 0, 0])
    scene["entities"].append({"id": ident("probe"), "name": "Load probe", "kind": "Empty",
        "position": [0, 0, 0], "rotation": [0, 0, 0], "scale": [1, 1, 1], "particle_emitter": extra,
        "particle_effect": {"version": 2, "asset": effect["id"], "bindings": {}, "enabled": True,
                            "play_on_start": False, "seed": 4242, "layer_overrides": {}},
        "script": {"name": "LoadProbe", "class_id": ident("class"), "properties": {},
                   "provider": {"id": "cpp", "version": 1}, "backend": {"id": "native", "version": 1}}})
    write(scene_path, scene)
    run("--project", root, "--build-psx")


def capture(root, symbols, name):
    portable = root / ".epok" / name
    portable.mkdir()
    output = portable / "frames.json"
    script = portable / "capture.lua"
    write(script, '''local ffi=require('ffi')
local probe=ffi.cast('volatile uint32_t*',PCSX.getMemPtr()+ADDRESS)
local output=assert(io.open(OUTPUT,'w'));output:write('[')
local first=true
load_capture=PCSX.addBreakpoint(CAPTURE,'Exec',4,'VFX load frame',function()
 if not first then output:write(',') end;first=false;output:write('[')
 for i=0,34 do if i>0 then output:write(',') end;output:write(tostring(tonumber(probe[i]))) end
 output:write(']');output:flush()
 if tonumber(probe[1])==6 then output:write(']');output:close();PCSX.quit(0) end
 return true
end)
'''.replace("ADDRESS", str(symbols["vfx_load_probe"] & 0x1fffff)).replace("CAPTURE", str(symbols["vfx_load_capture"]))
       .replace("OUTPUT", json.dumps(output.as_posix())))
    exe = ROOT / ".tools/redux/pcsx-redux.exe"
    if os.name == "nt" and exe.with_suffix(".main").is_file():
        exe = exe.with_suffix(".main")
    with (portable / "emulator.log").open("w") as log:
        process = subprocess.Popen([str(exe), "-portable", str(portable), "-run", "-stdout", "-interpreter", "-softgpu", "-2mb",
            "-no-gdb", "-loadexe", str(root / ".epok/build/epok.ps-exe"), "-bios", str(exe.parent / "openbios.bin"),
            "-dofile", str(script), "-debugger"], cwd=portable, stdout=log, stderr=log, stdin=subprocess.DEVNULL,
            creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        try:
            assert process.wait(timeout=180) == 0
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=15)
    rows = [dict(zip(FIELDS, values)) for values in json.loads(output.read_text())]
    assert len(rows) >= 250 and rows[-1]["phase"] == 6, rows[-1:]
    assert all(row["magic"] == 0x56465835 and row["failures"] == 0 for row in rows), rows[-1]
    assert all(row["particles"] <= 256 and row["effects"] <= 8 and row["layers"] <= 64 for row in rows)
    assert rows[-1]["events"] == 72 and rows[-1]["markers"] == 9, rows[-1]
    assert rows[-1]["dropped_effects"] == 1 and rows[-1]["dropped_bursts"] == 49152, rows[-1]
    assert rows[-1]["cancelled_effects"] == 9, rows[-1]
    return rows


def main():
    root = Path(tempfile.mkdtemp(prefix="epok-vfx-load-")) / "Game"
    print("Retained VFX load fixture:", root, flush=True)
    create(root)
    elf = root / ".epok/build/epok.elf"
    symbols = {m[3]: int(m[1], 16) for line in tool("nm", "-S", "--defined-only", elf).splitlines()
               if (m := re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line))}
    sizes = re.search(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+\d+\s+\S+\s+", tool("size", "--format=berkeley", elf), re.MULTILINE)
    report = dict(zip(("text", "data", "bss"), map(int, sizes.groups())))
    export = Path(run("--project", root, "--export-psx").strip())
    env = os.environ.copy();env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
        "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
        cwd=export, env=env, capture_output=True, text=True, timeout=240)
    write(root / ".epok/standalone-build.log", result.stdout + result.stderr)
    assert result.returncode == 0, result.stdout + result.stderr
    assert (export / "epok.ps-exe").read_bytes() == (root / ".epok/build/epok.ps-exe").read_bytes()
    rows = capture(root, symbols, "load-first")
    repeated = capture(root, symbols, "load-repeat")
    deterministic = ["effects", "layers", "particles", "peak_particles", "events", "markers", "cancelled_effects",
                     "dropped_effects", "dropped_bursts", "failures", "objects"]
    assert {k: rows[-1][k] for k in deterministic} == {k: repeated[-1][k] for k in deterministic}
    phases = {}
    for phase, label in [(0, "baseline"), (1, "one_effect"), (2, "eight_effects"), (3, "burst_overflow"),
                         (4, "emitter_overflow"), (5, "cleanup")]:
        complete = [row for row in rows if row["phase"] == phase]
        samples = complete[5:]
        phases[label] = {"samples": len(samples), "warmup_frames_excluded": 5,
                         "all_frame_scanlines_max": max(row["frame_scanlines"] for row in complete),
                         "all_simulation_scanlines_max": max(row["simulation_scanlines"] for row in complete)}
        for field in ["frame_scanlines", "simulation_scanlines", "render_scanlines", "steps", "dropped_steps",
                      "particles", "sprites", "dropped_particles", "dropped_emitters", "dropped_sprites"]:
            values = sorted(row[field] for row in samples)
            phases[label][field] = {"min": values[0], "median": statistics.median(values),
                                   "p95": values[(len(values)*95+99)//100-1], "max": values[-1]}
    report.update(project=str(root), standalone_identical=True, deterministic_capacity_outcomes=True,
                  rendering=documents.loads(next(root.glob("*.epokproject")).read_text())["rendering"],
                  executable_sha256=hashlib.sha256((root / ".epok/build/epok.ps-exe").read_bytes()).hexdigest(),
                  phases=phases, final=rows[-1], frames=rows,
                  notes="Instrumented native frame samples with twelve meshes, textured characters and HUD; no physical-console or universal frame-rate guarantee.")
    write(ROOT / "artifacts/timelines/phase5-vfx-load.json", report)
    print("PASS MIPS VFX capacity, deterministic overflow/cleanup, identical standalone export and repeated PCSX load", flush=True)
    print(json.dumps({key: value for key, value in report.items() if key != "frames"}, indent=2))


if __name__ == "__main__":
    main()
