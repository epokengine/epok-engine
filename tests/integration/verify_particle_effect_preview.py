"""Compare editor C ABI snapshots with cooked MIPS kernels under PCSX."""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import shutil
import sys

from verify_blueprints import ROOT, documents, identity, run, write
from verify_blueprint_performance import tool

FIREBALL = '--fireball' in sys.argv
STEPS = 350 if FIREBALL else 96
FIELDS = ["state", "tick", "alive", "spawned", "dropped", "peak", "events", "markers",
          "skipped_targets", "skipped_events", "dropped_emitters", "diagnostics_dropped"]


def fingerprint(values):
    result = 2166136261
    for value in values:
        result = ((result ^ (value & 0xffffffff)) * 16777619) & 0xffffffff
    return result


def expected(frame):
    particles = []
    for p in frame["particles"]:
        particles.extend([p["index"], p["layer"], p["age"], p["lifetime"], *p["position"], *p["velocity"]])
    quads = []
    for q in frame["quads"]:
        s = q["sprite"]
        quads.extend(s["region"])
        quads.extend(round(v * 4096) for v in s["size"] + s["pivot"])
        quads.extend(round(v * 255) for v in s["color"])
        quads.extend(round(v * 4096) for row in q["world"] for v in row)
    return [frame["stats"][f] for f in FIELDS] + [fingerprint(particles), len(frame["quads"]), fingerprint(quads)]


def main():
    root = Path(tempfile.mkdtemp(prefix="epok-effect-preview-")) / "Game"
    run("--create-project", root, "--name", "Effect Preview Parity")
    if FIREBALL:
        source = ROOT / 'examples/timeline-spell'
        shutil.copytree(source / 'assets/textures', root / 'assets/textures', dirs_exist_ok=True)
        path = root / 'assets/Effects/Fireball.particle-effect.json'
        effect = json.loads((source / 'assets/Effects/Fireball.particle-effect.json').read_text())
    else:
        path = Path(run("--project", root, "--new-particle-effect", "Preview", "--preset", "Sparks").strip())
        effect = json.loads(path.read_text())
        effect["seed"] = 91873
        effect["timeline"]["duration_ticks"] = 4096
        for key in effect["timeline"]["tracks"][0]["keys"]:
            key["tick"] = min(key["tick"], 4096)
        # Independent authored curve, rather than relying on preset duration ratios.
        effect["timeline"]["tracks"][0]["keys"] = [
            {"id": identity(), "tick": 0, "value": 0.25},
            {"id": identity(), "tick": 2048, "value": 1.0},
            {"id": identity(), "tick": 4096, "value": 0.0}]
        effect["timeline"]["tracks"][0]["interpolation"] = "smoothstep"
        emitter = effect["layers"][0]["content"]["emitter"]
        emitter.update(lifetime=0.75, frames=3, frame_columns=4, frame_duration=0.125)
        emitter["sprite"]["region"] = [0, 0, 16, 16]
        effect["layers"][0]["position"] = [-0.5, 0.25, 0.75]
        effect["timeline"]["markers"] = [{"id": identity(), "name": "Impact", "tick": 2048}]
        effect["timeline"]["events"][0]["keys"].append({"id": identity(), "tick": 2048,
            "arguments": {"count": {"kind": "literal", "value_type": {"kind": "uint32"}, "value": 11}}})
    write(path, effect)
    trace = json.loads(run("--project", root, "--preview-particle-effect", path, "--steps", STEPS))
    assert trace == json.loads(run("--project", root, "--preview-particle-effect", path, "--steps", STEPS))
    assert "limited to 482" in run("--project", root, "--preview-particle-effect", path, "--steps", 483, ok=False)
    class_id = identity()
    write(root / "assets/scripts/PreviewProbe.hpp", '''#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="CLASS") PreviewProbe:public epok::Behaviour {
public:void update(epok::Transform&,epok::Fixed)override;
};
'''.replace('"CLASS"', '"' + class_id + '"'))
    write(root / "assets/scripts/PreviewProbe.cpp", '''#include "PreviewProbe.hpp"
#include "scene.hh"
#include "particles.hpp"
#include "effects/ASSET.hh"
extern "C" {
volatile uint32_t preview_probe[STEPS*15]={};
__attribute__((noinline,used)) void preview_done(){asm volatile("nop":::"memory");}
}
namespace {
epok::timeline::Director<8> director;
epok::ParticlePool particles;
void release(epok::EffectLayerHandle h){particles.remove_layer(h);}
epok::effects::Pool pool(director,release);
bool ran=false;
}
void PreviewProbe::update(epok::Transform&,epok::Fixed){
 if(ran)return;ran=true;using namespace epok;particles.clear();
 auto handle=pool.spawn(effects::cooked::asset_STEM::asset,Affine<Fixed>::identity(),1);
 for(unsigned step=0;step<STEPS;++step){
  Fixed dt(68,Fixed::RAW);pool.prepare(1);director.advance(dt,1);pool.observe();pool.advance(dt);
  particles.begin(dt);pool.emitters([&](EffectLayerHandle h,ParticleEmitter& e,const Affine<Fixed>& w){particles.emitter(h,e,w);});particles.advance();
  const auto& s=director.stats;uint32_t hash=2166136261,count=0;
  auto add=[&](uint32_t v){hash=(hash^v)*16777619u;};
  for(unsigned index=0;index<256;++index){const auto& p=particles.particles[index];if(!p.alive)continue;
   add(index);add(p.owner.layer.index%8);add(p.age.raw());add(p.lifetime.raw());
   for(auto v:p.position)add(v.raw());for(auto v:p.velocity)add(v.raw());}
  uint32_t particle_hash=hash;hash=2166136261;
  auto quad=[&](EffectLayerHandle,const Sprite& s,const Affine<Fixed>& w){++count;
   for(auto v:s.region)add(v);for(auto v:s.size)add(v.raw());for(auto v:s.pivot)add(v.raw());for(auto v:s.color)add(v);
   for(unsigned r=0;r<3;++r)for(unsigned c=0;c<4;++c)add(w.values[r][c].raw());};
  pool.sprites(quad);
  particles.each_all([&](timeline::BoundTarget owner,const Sprite& source,const Affine<Fixed>& w){if(auto* layer=owner.effect_layer()){auto s=source;effects::Pool::tint(*layer,s);quad(owner.layer,s,w);}});
  uint32_t row[]={uint32_t(pool.state(handle)),uint32_t(director.tick(pool.sequence(handle))),particle_stats.alive,particle_stats.spawned,particle_stats.dropped,particle_stats.peak,s.events,s.markers,s.skipped_targets,s.skipped_events,particle_stats.dropped_emitters,s.diagnostics_dropped,particle_hash,count,hash};
  for(unsigned i=0;i<15;++i)preview_probe[step*15+i]=row[i];
 }
 preview_done();
}
'''.replace("ASSET", effect["id"]).replace("STEM", effect["id"].replace("-", "")).replace("STEPS", str(STEPS)))
    scene_path = root / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text())
    scene["version"] = 4
    entity = scene["entities"][-1]
    entity["particle_effect"] = {"version": 1, "asset": effect["id"], "enabled": True, "play_on_start": False}
    entity["script"] = {"name": "PreviewProbe", "class_id": class_id, "properties": {},
        "provider": {"id": "cpp", "version": 1}, "backend": {"id": "native", "version": 1}}
    write(scene_path, scene)
    run("--project", root, "--build-psx")
    elf = root / ".epok/build/epok.elf"
    symbols = {m[3]: int(m[1], 16) for line in tool("nm", "-S", "--defined-only", elf).splitlines()
               if (m := re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line))}
    portable = root / ".epok/preview-emulator"
    portable.mkdir(parents=True)
    output = portable / "trace.json"
    script = portable / "measure.lua"
    write(script, '''local ffi=require('ffi')
local probe=ffi.cast('volatile uint32_t*',PCSX.getMemPtr()+PROBE)
preview_break=PCSX.addBreakpoint(DONE,'Exec',4,'Preview parity',function()
 local file=assert(io.open(OUTPUT,'w'));file:write('[')
 for i=0,COUNT-1 do if i>0 then file:write(',') end file:write(tostring(tonumber(probe[i]))) end
 file:write(']');file:close();PCSX.quit(0);return true
end)
'''.replace("PROBE", str(symbols["preview_probe"] & 0x1fffff)).replace("DONE", str(symbols["preview_done"]))
       .replace("OUTPUT", json.dumps(output.as_posix())).replace("COUNT", str(STEPS * 15)))
    executable = ROOT / ".tools/redux/pcsx-redux.exe"
    if os.name == "nt" and executable.with_suffix(".main").is_file():
        executable = executable.with_suffix(".main")
    with (portable / "emulator.log").open("w") as log:
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
    actual = json.loads(output.read_text())
    for i, frame in enumerate(trace["frames"]):
        assert actual[i*15:(i+1)*15] == expected(frame), (i, actual[i*15:(i+1)*15], expected(frame))
    report = {"project": str(root), "steps": STEPS, "tick_q12": 68, "all_particle_state_hashes_match": True,
              "all_stats_and_quad_counts_match": True, "all_quad_values_match": True, "replay_identical": True}
    report_name = "phase3-fireball-parity.json" if FIREBALL else "phase3-preview-parity.json"
    write(ROOT / "artifacts/timelines" / report_name, report)
    print("PASS host C ABI versus cooked MIPS/PCSX Q12 particles, RNG, events, markers and lifecycle:", report)


if __name__ == "__main__":
    main()
