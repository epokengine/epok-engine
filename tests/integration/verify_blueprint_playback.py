"""Real generated Blueprint playback waits, MIPS lifecycle and standalone rebuild."""
import copy
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

from verify_blueprints import ROOT, documents, identity, literal, node, run, write
from verify_blueprint_performance import tool


def link(source):
    return {"kind": "link", "node": source["id"], "pin": "value"}


def create(root, direct=False, subscribe=False):
    run("--create-project", root, "--name", "Blueprint Playback Acceptance")
    cls, monitor, glow = identity(), identity(), identity()
    header = '''#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="CLASS") SpellBridge:public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere) uint32_t mode=0;
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="GLOW") epok::Fixed glow=0.0;
    EPOK_FUNCTION(BlueprintCallable) void remember_sequence(epok::timeline::Handle value){playback=value;}
    epok::timeline::Handle playback;
    EPOK_FUNCTION(BlueprintEvent) virtual void cast_spell(epok::Fixed power){}
    EPOK_FUNCTION(BlueprintEvent) virtual void cast_effect(){}
    EPOK_FUNCTION(BlueprintCallable) void apply_damage(epok::Fixed power);
    EPOK_FUNCTION(BlueprintCallable) void record(uint32_t outcome);
    void start(epok::Transform&)override;
    void update(epok::Transform&,epok::Fixed)override;
    unsigned ticks=0;
};
class EPOK_CLASS(Blueprintable,Id="MONITOR") PlaybackMonitor:public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere) bool after=false;
    void update(epok::Transform&,epok::Fixed)override;
    unsigned ticks=0;
};
'''.replace('"CLASS"', f'"{cls}"').replace('"MONITOR"', f'"{monitor}"').replace('"GLOW"', f'"{glow}"')
    write(root / "assets/scripts/SpellBridge.hpp", header)
    write(root / "assets/scripts/SpellBridge.cpp", '#include "SpellBridge.hpp"\n')
    reflected = json.loads(run("--project", root, "--reflect"))
    functions = {f["name"]: f for c in reflected["classes"] if c["id"] == cls for f in c["functions"]}
    assert functions["remember_sequence"]["parameters"][0]["value_type"] == {"kind": "sequence_handle"}
    timeline_path = Path(run("--project", root, "--new-timeline", "SpellTiming").strip())
    timeline = json.loads(timeline_path.read_text())
    marker = identity()
    timeline.update(duration_ticks=512, loop_mode="repeat" if subscribe else "once", markers=[{"id": marker, "name": "Impact", "tick": 256}])
    binding_slot, optional_slot = identity(), identity()
    def add_binding(source, slot, required):
        source["slots"].append({"id": slot, "name": "Caster" if required else "Optional target", "target": {"kind": "entity_ref", "class": cls}, "required": required})
        source["tracks"].append({"id": identity(), "name": "Glow", "slot": slot, "property": glow, "value_type": {"kind": "fixed"},
            "priority": 0, "blend": "absolute", "restore": "restore_initial", "interpolation": "linear",
            "keys": [{"id": identity(), "tick": 0, "value": 0}, {"id": identity(), "tick": 512, "value": 1}]})
    if direct:
        add_binding(timeline, binding_slot, True)
        add_binding(timeline, optional_slot, False)
    write(timeline_path, timeline)
    effect_path = Path(run("--project", root, "--new-particle-effect", "SpellTail", "--preset", "sparks").strip())
    effect = json.loads(effect_path.read_text())
    # Ordinary source effect: use a short timeline and a bounded particle tail.
    effect["timeline"].update(duration_ticks=512, tracks=[], events=[], markers=[])
    effect_binding = identity()
    if direct:
        add_binding(effect["timeline"], effect_binding, True)
    write(effect_path, effect)
    bp_path = Path(run("--project", root, "--new-blueprint", "BP_SpellBridge", "--parent", cls).strip())
    bp = documents.loads(bp_path.read_text(encoding="utf-8"))
    effect_asset = effect
    def build_graph(function, effect=False):
        entry = node("entry")
        owner = node("builtin", operation={"kind": "self_entity"})
        play = node("builtin", {"target": link(owner)}, operation={"kind": "play_effect_component" if effect else "play_sequence_component"})
        extra = []
        if direct:
            typed = node("builtin", {"target": link(owner)}, operation={"kind": "cast", "class": cls})
            extra.append(typed)
            if effect:
                transform = node("builtin", {"target": link(owner)}, operation={"kind": "get_transform"})
                extra.append(transform)
                play["kind"]["operation"] = {"kind": "spawn_particle_effect", "asset": effect_asset["id"]}
                play["inputs"] = {"transform": link(transform), "owner": literal("entity_ref", None), "seed": literal("uint32", 77),
                                  f"binding:{effect_binding}": link(typed)}
            else:
                play["kind"]["operation"] = {"kind": "play_timeline_asset", "asset": timeline["id"]}
                play["inputs"] = {"owner": link(owner), f"binding:{binding_slot}": link(typed)}
        entry["outputs"]["next"] = [play["id"]]
        completed = node("call", {"outcome": literal("uint32", 0)}, function=functions["record"]["id"])
        cancelled = node("call", {"outcome": literal("uint32", 1)}, function=functions["record"]["id"])
        wait = node("wait_playback", {"playback": link(play)}, condition={"kind": "effect_complete" if effect else "sequence_complete"})
        wait["outputs"] = {"completed": [completed["id"]], "cancelled": [cancelled["id"]]}
        nodes = [entry, owner, play, wait, completed, cancelled]
        nodes.extend(extra)
        if effect:
            burst = node("builtin", {"playback": link(play), "count": literal("uint32", 4)}, operation={"kind": "burst_effect"})
            nodes.append(burst)
            play["outputs"]["next"] = [burst["id"]]
            burst["outputs"]["next"] = [wait["id"]]
        else:
            impact = node("wait_playback", {"playback": link(play)}, condition={"kind": "marker", "timeline": timeline["id"], "marker": marker})
            damage = node("call", {"power": {"kind": "parameter", "name": "power"}}, function=functions["apply_damage"]["id"])
            impact["outputs"] = {"reached": [damage["id"]], "cancelled": [cancelled["id"]]}
            damage["outputs"]["next"] = [wait["id"]]
            remember = node("call", {"value": link(play)}, function=functions["remember_sequence"]["id"])
            play["outputs"]["next"] = [remember["id"]]
            remember["outputs"]["next"] = [impact["id"]]
            nodes.append(remember)
            nodes.extend([impact, damage])
            if subscribe:
                impact["kind"]["condition"]["kind"] = "subscribe_marker"
                impact["outputs"]["completed"] = [completed["id"]]
                pause = node("delay", {"seconds": literal("fixed", 0.05)})
                damage["outputs"]["next"] = [pause["id"]]
                nodes.remove(wait)
                nodes.append(pause)
                immediate = node("call", {"outcome": literal("uint32", 2)}, function=functions["record"]["id"])
                parent_delay = node("delay", {"seconds": literal("fixed", 0.05)})
                after_delay = node("call", {"outcome": literal("uint32", 3)}, function=functions["record"]["id"])
                impact["outputs"]["next"] = [immediate["id"]]
                immediate["outputs"]["next"] = [parent_delay["id"]]
                parent_delay["outputs"]["next"] = [after_delay["id"]]
                nodes.extend([immediate,parent_delay,after_delay])
        return {"id": identity(), "name": function["name"], "override_id": function["id"], "parameters": function["parameters"],
                "returns": function["returns"], "entry": entry["id"], "nodes": nodes}
    bp["functions"] = [build_graph(functions["cast_spell"]), build_graph(functions["cast_effect"], True)]
    for item, position in zip(bp["functions"][0]["nodes"], [(0,0),(0,125),(175,0),(555,165),(355,350),(100,350),(355,0),(555,0)]):
        bp["layout"]["positions"][item["id"]] = list(position)
    write(bp_path, bp)
    run("--project", root, "--compile-blueprints")
    # Invalid marker edits are rejected without rewriting the Blueprint source.
    original = bp_path.read_bytes()
    stale = copy.deepcopy(bp)
    next(n for n in stale["functions"][0]["nodes"] if n["kind"].get("condition", {}).get("kind") in ("marker", "subscribe_marker"))["kind"]["condition"]["marker"] = identity()
    write(bp_path, stale)
    stale_bytes = bp_path.read_bytes()
    assert "Missing timeline/marker reference" in run("--project", root, "--compile-blueprints", ok=False)
    assert bp_path.read_bytes() == stale_bytes
    bp_path.write_bytes(original)
    if direct:
        bad = copy.deepcopy(bp)
        play = next(n for n in bad["functions"][0]["nodes"] if n["kind"].get("operation", {}).get("kind") == "play_timeline_asset")
        del play["inputs"][f"binding:{binding_slot}"]
        write(bp_path, bad)
        bad_bytes = bp_path.read_bytes()
        assert "Required playback binding Caster" in run("--project", root, "--compile-blueprints", ok=False)
        assert bp_path.read_bytes() == bad_bytes
        bp_path.write_bytes(original)
    scene_path = root / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    camera = copy.deepcopy(scene["entities"][0])
    def binding(name, class_id, properties, visual=False):
        return {"name": name, "class_id": class_id, "properties": properties,
                "provider": {"id": "blueprint" if visual else "cpp", "version": 1}, "backend": {"id": "native", "version": 1}}
    observers = copy.deepcopy(camera)
    observers.update(id=identity(), name="Monitor", kind="Empty", script=binding("PlaybackMonitor", monitor, {}))
    entities = [camera, observers]
    for mode in range(5):
        actor = copy.deepcopy(camera)
        actor.update(id=identity(), name=f"Caster{mode}", kind="Empty", position=[0,0,2],
                     script=binding(bp["name"], bp["id"], {"mode": mode}, True))
        if direct:
            pass  # Source assets are reachable exclusively through Blueprint nodes.
        elif mode == 4:
            actor["particle_effect"] = {"version": 2, "asset": effect["id"], "bindings": {}, "enabled": True, "play_on_start": False}
        else:
            actor["timeline"] = {"version": 1, "asset": timeline["id"], "bindings": {}, "enabled": True, "play_on_start": False}
        entities.append(actor)
    scene.update(version=4, entities=entities)
    write(scene_path, scene)
    after = copy.deepcopy(scene)
    after.update(name="After", entities=[camera, copy.deepcopy(observers)])
    after["entities"][1]["script"]["properties"] = {"after": True}
    for entity in after["entities"]:
        entity["id"] = identity()
    write(root / "assets/scenes/After.epokmap", after)
    write(root / "ProjectSettings/Maps.epoksettings", {"scenes": ["assets/scenes/After.epokmap"]})
    runtime_source = '''#include "SpellBridge.hpp"
#include "timeline_service.hpp"
#include "particle_effect_service.hpp"
extern "C" {
volatile int32_t playback_probe[26]={};
__attribute__((noinline,used)) void playback_done(){asm volatile("nop":::"memory");}
}
static epok::EntityHandle actors[5];
static SpellBridge* casters[5]={};
void SpellBridge::apply_damage(epok::Fixed power){++playback_probe[2+mode];playback_probe[17]+=power.raw();AFTER_DAMAGE}
void SpellBridge::record(uint32_t outcome){if(outcome<2)++playback_probe[7+mode+5*outcome];else if(outcome<4)++playback_probe[22+outcome];}
void SpellBridge::start(epok::Transform&){actors[mode]=epok::handle(&entity());casters[mode]=this;if(mode==4)cast_effect();else cast_spell(7.0);}
void SpellBridge::update(epok::Transform&,epok::Fixed){
    ++ticks;if(ticks==2){if(mode==1)epok::timeline::sequences.stop(playback);if(mode==2)epok::destroy_entity(&entity());}
}
void PlaybackMonitor::update(epok::Transform&,epok::Fixed){
    using namespace epok;++ticks;
    if(performance_stats.simulation_scanlines>uint32_t(playback_probe[22]))playback_probe[22]=performance_stats.simulation_scanlines;
    if(after){if(ticks==30){playback_probe[0]=0x42505734;playback_probe[1]=1;playback_done();}return;}
    if(ticks==2)set_active(actors[3].get(),false);
    if(ticks==6)set_active(actors[3].get(),true);
    // Reentry replaces this event's frame. Scene replacement cancels it before Impact.
    if(ticks==100){timeline::sequences.stop(casters[0]->playback);casters[0]->cast_spell(99.0);timeline::sequences.stop(casters[0]->playback);casters[0]->cast_spell(99.0);}
    if(ticks==101){playback_probe[18]=sizeof(bp::Continuations<8>);playback_probe[19]=sizeof(bp::PlaybackWait);playback_probe[20]=effect_stats.completed;playback_probe[21]=particle_stats.dropped;playback_probe[23]=request_scene("After");}
}
'''
    runtime_source = runtime_source.replace("AFTER_DAMAGE", "if(playback_probe[2+mode]==3)epok::timeline::sequences.stop(playback);if(mode==0&&playback_probe[2]==1){epok::timeline::sequences.stop(playback);cast_spell(9.0);}" if subscribe else "")
    if subscribe:
        runtime_source = runtime_source.replace("sizeof(bp::PlaybackWait)", "sizeof(bp::PlaybackSubscription)")
    write(root / "assets/scripts/SpellBridge.cpp", runtime_source)
    run("--project", root, "--build-psx")
    flags = (root / ".epok/build/sources.mk").read_text()
    assert "-DEPOK_PLAYBACK_WAITS" in flags
    if direct:
        assert (root / ".epok/build/scripts/generated/playback_calls.cpp").is_file()
        assert not any(e.get("timeline") or e.get("particle_effect") for e in scene["entities"])
    elf = root / ".epok/build/epok.elf"
    symbols = {m[3]: int(m[1], 16) for line in tool("nm", "-S", "--defined-only", elf).splitlines()
               if (m := re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line))}
    sizes = re.search(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+\d+\s+\S+\s+", tool("size", "--format=berkeley", elf), re.MULTILINE)
    report = dict(zip(("text", "data", "bss"), map(int, sizes.groups())))
    export = Path(run("--project", root, "--export-psx").strip())
    env = os.environ.copy()
    env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
        "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
        cwd=export, env=env, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert (export / "epok.ps-exe").read_bytes() == (root / ".epok/build/epok.ps-exe").read_bytes()
    report["standalone_identical"] = True
    return symbols, report


def emulator(root, symbols, subscribe=False):
    portable = root / ".epok/playback-emulator"
    portable.mkdir(parents=True)
    output = portable / "measurements.json"
    script = portable / "measure.lua"
    write(script, '''local ffi=require('ffi')
local probe=ffi.cast('volatile int32_t*',PCSX.getMemPtr()+PROBE)
playback_break=PCSX.addBreakpoint(DONE,'Exec',4,'Blueprint playback completion',function()
 local file=assert(io.open(OUTPUT,'w'));file:write('[')
 for i=0,25 do if i>0 then file:write(',') end file:write(tostring(tonumber(probe[i]))) end
 file:write(']');file:close();PCSX.quit(0);return true
end)
'''.replace("PROBE", str(symbols["playback_probe"] & 0x1fffff)).replace("DONE", str(symbols["playback_done"]))
        .replace("OUTPUT", json.dumps(output.as_posix())))
    executable = ROOT / ".tools/redux/pcsx-redux.exe"
    with (portable / "emulator.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(executable), "-portable", str(portable), "-run", "-stdout", "-interpreter", "-softgpu", "-2mb",
            "-no-gdb", "-loadexe", str(root / ".epok/build/epok.ps-exe"), "-bios", str(executable.parent / "openbios.bin"),
            "-dofile", str(script), "-debugger"], cwd=portable, stdout=log, stderr=log, stdin=subprocess.DEVNULL,
            creationflags=0x08000000 if os.name == "nt" else 0)
        try:
            assert process.wait(timeout=150) == 0
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    data = json.loads(output.read_text())
    assert data[:2] == [0x42505734, 1], data
    assert data[2:7] == ([3,0,0,3,0] if subscribe else [1,0,0,1,0]), data
    assert data[7:12] == ([0,0,0,0,1] if subscribe else [1,0,0,1,1]), data
    assert data[12:17] == ([1,1,0,1,0] if subscribe else [0,1,0,0,0]), data
    assert data[17] == (46 if subscribe else 14)*4096 and data[20] == 1 and data[21] == 0 and data[23] == 1, data
    if subscribe:
        # At the first Impact, caster 0 reenters before its old 205-Q12 parent
        # Delay dispatches. That old parent frame is cancelled along with the
        # running listener; the new parent/listener remain independent.
        assert data[24:26] == [7,3], data
    return data


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--direct-assets", action="store_true")
    parser.add_argument("--subscribe-markers", action="store_true")
    args = parser.parse_args()
    root = Path(tempfile.mkdtemp(prefix="epok-blueprint-playback-")) / "Game"
    print("Retained playback fixture:", root, flush=True)
    direct = args.direct_assets or args.subscribe_markers
    symbols, report = create(root, direct, args.subscribe_markers)
    report["probe"] = emulator(root, symbols, args.subscribe_markers)
    report["fixture"] = str(root)
    report["direct_assets"] = direct
    report["subscribe_markers"] = args.subscribe_markers
    report_name = "phase4-subscriptions" if args.subscribe_markers else "phase4-direct-assets" if direct else "phase4-bridge"
    write(ROOT / f"artifacts/timelines/{report_name}.json", report)
    print("PASS typed Blueprint playback, marker damage once, cancellation, scene replacement, MIPS and identical export.")


if __name__ == "__main__":
    main()
