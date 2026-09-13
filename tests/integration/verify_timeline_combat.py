"""Execute the authored Fireball Blueprint, cancellation and owner destruction."""
import json
import hashlib
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

from verify_blueprints import ROOT, documents, run, write, identity
from verify_blueprint_performance import tool


def main():
    root = Path(tempfile.mkdtemp(prefix="epok-timeline-combat-")) / "Game"
    shutil.copytree(ROOT / "examples/timeline-spell", root,
                    ignore=shutil.ignore_patterns(".epok", "exports", "Local.epokconfig"))
    print("Retained combat fixture:", root, flush=True)
    header = root / "assets/scripts/SpellCombat.hpp"
    probe_class = identity()
    write(header, header.read_text() + '''
class EPOK_CLASS(Blueprintable,Id="CLASS") CombatProbe:public epok::Behaviour {
    unsigned ticks=0;
    epok::EntityHandle caster,victim;
public:
    void start(epok::Transform&)override;
    void update(epok::Transform&,epok::Fixed)override;
};
'''.replace('"CLASS"', f'"{probe_class}"'))
    cpp = root / "assets/scripts/SpellCombat.cpp"
    source = cpp.read_text().replace('#include "psyqo/xprintf.h"', '''#include "psyqo/xprintf.h"
#include "blueprint_spawn.hpp"
#include "particle_effect_service.hpp"
extern "C" {
volatile uint32_t combat_probe[12]={};
__attribute__((noinline,used)) void combat_capture(){asm volatile("nop":::"memory");}
}
''')
    source = source.replace("void SpellCombat::finished(){", "void SpellCombat::finished(){++combat_probe[4];")
    source = source.replace("void SpellCombat::cancelled(){", "void SpellCombat::cancelled(){++combat_probe[5];")
    source += '''
void CombatProbe::start(epok::Transform&){
    caster=epok::handle(epok::find_entity("Fireball Caster"));
    victim=epok::handle(epok::find_entity("Target"));
}
void CombatProbe::update(epok::Transform&,epok::Fixed){
    using namespace epok;++ticks;
    if(performance_stats.simulation_scanlines>combat_probe[9])combat_probe[9]=performance_stats.simulation_scanlines;
    auto* controller=static_cast<SpellCombat*>(bp::behaviour(caster));
    auto* target_object=static_cast<SpellTarget*>(bp::behaviour(victim));
    // The initial graph captured the target argument. Editing the instance's
    // future-cast target cannot redirect that suspended Impact continuation.
    if(ticks==3&&controller)controller->target={};
    if(ticks==410&&controller)controller->cast_spell(25,victim);
    if(ticks==430&&controller)controller->cancel_spell();
    if(ticks==470&&controller)controller->cast_spell(25,victim);
    if(ticks==480&&caster.get())destroy_entity(caster.get());
    if(ticks==600&&victim.get())combat_probe[10]=request_scene("After");
    unsigned phase=target_object?(ticks==110?1:ticks==450?2:0):(ticks==30?3:0);
    if(phase){
        combat_probe[0]=0x434f4d42;combat_probe[1]=phase;
        if(target_object){combat_probe[2]=target_object->health;combat_probe[3]=target_object->hits;}
        combat_probe[6]=particle_stats.dropped;combat_probe[7]=effect_stats.active;
        combat_probe[8]=particle_stats.alive;combat_probe[11]=sprite_stats.dropped;
        combat_capture();
    }
}
'''
    write(cpp, source)
    scene_path = root / "assets/scenes/Combat.epokmap"
    scene = documents.loads(scene_path.read_text())
    observer = {"id": identity(), "name": "Combat Probe", "kind": "Empty", "position": [0,0,0], "rotation": [0,0,0], "scale": [1,1,1],
                "script": {"class_id": probe_class, "name": "CombatProbe", "properties": {}, "provider": {"id": "cpp", "version": 1}, "backend": {"id": "native", "version": 1}}}
    scene["entities"].append(observer)
    write(scene_path, scene)
    after = dict(scene, name="After", entities=[scene["entities"][0], dict(observer, id=identity())])
    write(root / "assets/scenes/After.epokmap", after)
    write(root / "ProjectSettings/Maps.epoksettings", {"scenes": ["assets/scenes/After.epokmap"]})
    # Required-asset cooking must survive unrelated malformed authoring files.
    # Strict all-source validation still reports them, and a broken used effect
    # must fail rather than fall back to a previously compiled artifact.
    write(root / "assets/Timelines/UnusedBroken.timeline.json", "{ broken timeline")
    write(root / "assets/Effects/UnusedBroken.particle-effect.json", "{ broken effect")
    rejected = run("--project", root, "--compile-timelines", ok=False)
    assert "UnusedBroken.timeline.json" in rejected, rejected
    run("--project", root, "--build-psx")
    elf = root / ".epok/build/epok.elf"
    symbols = {m[3]: int(m[1], 16) for line in tool("nm", "-S", "--defined-only", elf).splitlines()
               if (m := re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line))}
    sizes = re.search(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+\d+\s+\S+\s+", tool("size", "--format=berkeley", elf), re.MULTILINE)
    report = dict(zip(("text", "data", "bss"), map(int, sizes.groups())))
    export_path = run("--project", root, "--export-psx").strip()
    # The Rust workspace returns canonical Windows paths with a verbatim prefix.
    # pathlib preserves that prefix, so normalize it for project-relative IDs.
    if os.name == "nt":
        if export_path.startswith("\\\\?\\UNC\\"):
            export_path = "\\\\" + export_path[8:]
        else:
            export_path = export_path.removeprefix("\\\\?\\")
    export = Path(export_path)
    env = os.environ.copy();env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
        "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
        cwd=export, env=env, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert (export / "epok.ps-exe").read_bytes() == (root / ".epok/build/epok.ps-exe").read_bytes()
    # Both destinations must carry provenance for the actual generated effect,
    # including its embedded shared timeline. Export does not certify an old
    # build, and a build does not certify a previous standalone export.
    graph = json.loads((root / ".epok/ArtifactDependencies.json").read_text())["nodes"]
    effect_id = "eedb2855-bbab-564e-b08f-6fd0db327aea"
    for destination in (root / ".epok/build", export):
        target = destination.relative_to(root).as_posix()
        output = f"generated-playback:{target}/effects/{effect_id}.hh"
        assert output in graph and not graph[output]["stale"], graph
        assert graph[output]["signature"] == hashlib.sha256((destination / f"effects/{effect_id}.hh").read_bytes()).hexdigest()
        assert f"effect:{effect_id}" in graph[output]["dependencies"]
        assert any(d.startswith("cooked-timeline:") for d in graph[output]["dependencies"])
        assert output in graph[f"stage-playback:{target}"]["dependencies"]
        assert not graph[f"stage-playback:{target}"]["stale"]
        effect_source = json.loads((root / "assets/Effects/Fireball.particle-effect.json").read_text())
        audio_projection = f"timeline-audio:{effect_source['timeline']['id']}"
        bank = graph[f"generated-resource:{target}/audio-bank.hh"]
        assert not bank["stale"] and not graph[audio_projection]["stale"]
        assert bank["signature"] == hashlib.sha256((destination / "audio-bank.hh").read_bytes()).hexdigest()
        assert audio_projection in bank["dependencies"]
        assert not any(key.startswith("generated-playback:") for key in bank["dependencies"])
        scene_output = graph[f"generated-scene:{target}/scene.hh"]
        assert not scene_output["stale"]
        assert scene_output["signature"] == hashlib.sha256((destination / "scene.hh").read_bytes()).hexdigest()
        assert {"scene-file:assets/scenes/Combat.epokmap", "scene-file:assets/scenes/After.epokmap",
                "scene-catalog", "scene-registry", "scene-render-settings", output}.issubset(scene_output["dependencies"])
        native = graph[f"generated-script:{target}/scripts/SpellCombat.cpp"]
        assert not native["stale"] and native["dependencies"] == ["native-file:assets/scripts/SpellCombat.cpp"]
        bp_id = "0396f8d7-dc13-5493-a5ec-dd3ad23f3672"
        assert graph[f"generated-script:{target}/scripts/generated/{bp_id}.hpp"]["dependencies"] == [f"generated-blueprint:{bp_id}"]
        stage = graph[f"stage:{target}"]
        assert not stage["stale"]
        assert {f"staged-file:{target}/scene.hh", f"staged-file:{target}/main.cpp",
                f"staged-file:{target}/scripts/SpellCombat.cpp"}.issubset(stage["dependencies"])
    executable = graph["executable:.epok/build/epok.ps-exe"]
    assert not executable["stale"]
    assert executable["signature"] == hashlib.sha256((root / ".epok/build/epok.ps-exe").read_bytes()).hexdigest()
    native_build = graph["native-build:.epok/build"]
    assert not native_build["stale"] and "native-build:.epok/build" in executable["dependencies"]
    assert "native-build:configuration" in native_build["dependencies"]
    sdk = graph["native-sdk:.epok/build"]
    assert not sdk["stale"]
    assert "native-sdk:.epok/build" in native_build["dependencies"]
    assert "native-metadata:file:project:.epok/build/sdk/libpsyqo.a" in native_build["dependencies"]
    for suffix in ("crt0cxx.s", "hwregs.inc", "memory-c.c"):
        inputs = [key for key in sdk["dependencies"] if key.endswith(suffix)]
        assert inputs and set(inputs).issubset(native_build["dependencies"]), (suffix, sdk)
    for suffix in ("libpsyqo.a", "libgcc.a", "ps-exe.ld", "build-inputs.mk",
                   f"effects/{effect_id}.hh"):
        inputs = [key for key in native_build["dependencies"] if key.endswith(suffix)]
        assert inputs, (suffix, native_build)
        assert all(not graph[key]["stale"] for key in inputs)
    assert not graph[f"export:{export.relative_to(root).as_posix()}"]["stale"]
    portable = root / ".epok/combat-emulator";portable.mkdir(parents=True)
    script = portable / "capture.lua"
    write(script, '''local ffi=require('ffi')
local probe=ffi.cast('volatile uint32_t*',PCSX.getMemPtr()+PROBE)
combat_break=PCSX.addBreakpoint(CAPTURE,'Exec',4,'Combat capture',function()
 local phase=tonumber(probe[1]);local name=FOLDER..'/'..phase
 local file=assert(io.open(name..'.json','w'));file:write('[')
 for i=0,11 do if i>0 then file:write(',') end file:write(tostring(tonumber(probe[i]))) end
 file:write(']');file:close()
 local ss=PCSX.GPU.takeScreenShot()
 if ss.width>0 and ss.height>0 then
  local raw=assert(io.open(name..'.raw','wb'));raw:write(tostring(ss.data));raw:close()
  local size=assert(io.open(name..'.size','w'));size:write(ss.width..' '..ss.height..' '..(tonumber(ss.bpp)==0 and 2 or 3));size:close()
 end
 if phase==3 then PCSX.quit(0) end
 return true
end)
'''.replace("PROBE", str(symbols["combat_probe"] & 0x1fffff)).replace("CAPTURE", str(symbols["combat_capture"]))
       .replace("FOLDER", json.dumps(portable.as_posix())))
    exe = ROOT / ".tools/redux/pcsx-redux.exe"
    # Match the Play worker: own the emulator, not the Windows updater wrapper.
    # Otherwise a timeout terminates only the wrapper and leaves a live child.
    if os.name == "nt" and exe.with_suffix(".main").is_file():
        exe = exe.with_suffix(".main")
    with (portable / "emulator.log").open("w") as log:
        process = subprocess.Popen([str(exe), "-portable", str(portable), "-run", "-stdout", "-interpreter", "-softgpu", "-2mb",
            "-no-gdb", "-loadexe", str(root / ".epok/build/epok.ps-exe"), "-bios", str(exe.parent / "openbios.bin"),
            "-dofile", str(script), "-debugger"], cwd=portable, stdout=log, stderr=log, stdin=subprocess.DEVNULL,
            creationflags=0x08000000 if os.name == "nt" else 0)
        try:
            assert process.wait(timeout=180) == 0
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=15)
    rows = [json.loads((portable / f"{i}.json").read_text()) for i in range(1,4)]
    assert all(r[0]==0x434f4d42 and r[2:4]==[75,1] and r[6]==0 and r[11]==0 for r in rows), rows
    assert rows[0][4:6]==[0,0] and rows[1][4:6]==[1,1] and rows[2][4:6]==[1,1], rows
    assert rows[-1][7:9]==[0,0] and rows[-1][10]==1, rows
    from PIL import Image
    for i in range(1,3):
        w,h,depth=map(int,(portable / f"{i}.size").read_text().split())
        raw=(portable / f"{i}.raw").read_bytes()
        if depth==2:
            rgb=bytearray()
            for offset in range(0,w*h*2,2):
                pixel=int.from_bytes(raw[offset:offset+2],"little")
                rgb.extend(((pixel>>shift)&31)*255//31 for shift in (0,5,10))
        else:rgb=raw[:w*h*3]
        Image.frombytes("RGB",(w,h),bytes(rgb)).save(portable / f"{i}.png")
    effect_path = next(path for path in root.rglob("*.particle-effect.json")
                       if path.name != "UnusedBroken.particle-effect.json"
                       and json.loads(path.read_text())["id"] == effect_id)
    valid_effect = effect_path.read_bytes()
    try:
        effect_path.write_text("{ broken required effect", encoding="utf-8")
        rejected = run("--project", root, "--build-psx", ok=False)
        assert effect_id in rejected, rejected
        failed = json.loads((root / ".epok/ArtifactDependencies.json").read_text())["nodes"]
        assert failed["stage:.epok/build"]["stale"], failed["stage:.epok/build"]
    finally:
        effect_path.write_bytes(valid_effect)
    report.update(fixture=str(root), standalone_identical=True, probe=rows,
                  unrelated_malformed_sources_ignored=True, required_broken_effect_rejected=True)
    write(ROOT / "artifacts/timelines/phase4-combat.json", report)
    print("PASS authored Fireball Blueprint: captured target, Impact damage once, drain completion, stop, owner destruction, scene change, identical export.")


if __name__ == "__main__":
    main()
