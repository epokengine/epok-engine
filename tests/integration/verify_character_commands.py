"""Run derived command producers through the actual PSX provider/runtime boundary."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import struct
import subprocess
import time
import urllib.request
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write


def verify(out, domain, modes, prepare_only=False, standalone_only=False):
    out = out / domain
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    dimension = "3D" if domain == "world3d" else "2D"
    namespace = uuid.UUID("bf00d5a2-24f2-580e-85db-d10fa09e640d")
    identity = lambda name: str(uuid.uuid5(namespace, domain + ":" + name))
    magic = 0x51000000 | (uuid.uuid4().int & 0xffffff)
    counter = 0

    def run(*args, label=None):
        nonlocal counter
        counter += 1
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / (label or f"command-{counter}.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if standalone_only:
        export = Path(run("--project", project, "--export-psx", label="standalone-export.log"))
        for header in ("character_commands.hpp", "character_input.hpp"):
            assert (export / header).read_bytes() == (ROOT / "runtime" / header).read_bytes()
        env = os.environ.copy()
        env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
        result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
                                 "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
                                cwd=export, env=env, capture_output=True, text=True, timeout=900)
        (out / "standalone-build.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        original = (project / ".epok/build/epok.ps-exe").read_bytes()
        assert (export / "epok.ps-exe").read_bytes() == original
        (out / "standalone-result.json").write_text(json.dumps({"domain": domain, "identical": True, "export": str(export),
                                               "sha256": hashlib.sha256(original).hexdigest()}, indent=2) + "\n", encoding="utf-8")
        print("PASS", domain, "standalone rebuild, byte-identical", flush=True)
        return

    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Commands " + dimension)
    source = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t character_command_probe[24]={};}
struct CommandProbeState {uint32_t steps=0,submissions=0,samples=0,failures=0,grounded=0,jumps=0,hash=2166136261u;epok::GameplayVector3 start{},previous{};};
inline CommandProbeState command_states[3];
inline epok::GameplayVector3 command_position(epok::CharacterDIM& actor){POSITION}
inline uint32_t command_hash(epok::CharacterCommandResultDIM value){uint32_t hash=2166136261u;const uint32_t words[]={COMMAND_WORDS};for(auto word:words)hash=(hash^word)*16777619u;return hash;}
class EPOK_CLASS(Abstract,Blueprintable,Domain=DOMAIN,Owners=DOMAIN,Id="BASE") CommandProviderProbe:public epok::CharacterCommandSourceDIMComponent {
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 EPOK_FUNCTION(BlueprintCallable) epok::CharacterCommandResultDIM next_command(epok::Fixed dt){
  epok::CharacterCommandResultDIM value;if(probe_slot>=3)return value;
  auto& state=command_states[probe_slot];const auto step=state.steps;
  if(dt<=0.0||step>=120){value.status=epok::CharacterCommandStatus::NoStep;return value;}
  COMMAND
  value.command.jump=step==15;return value;
 }
 EPOK_FUNCTION(BlueprintCallable) void observe_submission(epok::CharacterCommandResultDIM value){
  if(probe_slot>=3)return;auto& state=command_states[probe_slot];
  if(command_hash(value)!=command_hash(next_command(epok::Fixed(1.0/60.0))))state.failures|=1;
  if(command_hash(value)!=command_hash(sample()))state.failures|=2;
  if(update(epok::Fixed(1.0/60.0)).status!=epok::CharacterCommandStatus::Busy)state.failures|=4;
  state.hash=(state.hash^command_hash(value))*16777619u;++state.steps;++state.submissions;
 }
 void begin_play()override{epok::CharacterCommandSourceDIMComponent::begin_play();if(probe_slot<3){auto& state=command_states[probe_slot];state={};auto* actor=epok::active_object_registry->resolve<epok::CharacterDIM>(owner_id());state.previous=state.start=command_position(*actor);}}
};
class EPOK_CLASS(Blueprintable,Id="CPP") CppCommandProviderProbe:public CommandProviderProbe {
public:
 epok::CharacterCommandResultDIM calculate_command(epok::Fixed dt)override{return next_command(dt);}
 void command_submitted(epok::CharacterCommandResultDIM value)override{observe_submission(value);}
};
class EPOK_CLASS(Blueprintable,Domain=DOMAIN,Owners=DOMAIN,Id="REPORTER") CommandProbeReporter:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 void tick(epok::Fixed){
  if(probe_slot>=3)return;auto& state=command_states[probe_slot];if(state.samples>=120)return;
  auto* actor=epok::active_object_registry->resolve<epok::CharacterDIM>(owner_id());if(!actor)return;
  if(state.steps!=state.samples+1||state.submissions!=state.steps)state.failures|=8;
  const auto motion=actor->motion();if(motion.status!=epok::CharacterMotionStatus::Applied)state.failures|=16;
  const auto position=command_position(*actor);DELTA
  state.grounded+=motion.grounded;state.jumps+=motion.jumped;state.previous=position;++state.samples;
  if(state.samples==120){auto* p=character_command_probe+probe_slot*8;p[1]=state.steps;p[2]=state.failures;p[3]=state.grounded;p[4]=state.jumps;p[5]=state.hash;p[6]=TRAVEL;p[7]=state.submissions;p[0]=MAGIC+EPOK_LUA_MODE;}
 }
};
'''
    if dimension == "3D":
        position = "auto* d=actor.data();return {d->transform.position[0],d->transform.position[1],d->transform.position[2]};"
        command = "value.command.velocity.y=step<75?epok::Fixed(6.0):epok::Fixed(0.0);value.command.heading=step<60?epok::Fixed(15.0):epok::Fixed(345.0);value.command.turn=true;"
        words = "uint32_t(value.status),uint32_t(value.command.velocity.x.raw()),uint32_t(value.command.velocity.y.raw()),uint32_t(value.command.heading.raw()),value.command.turn,value.command.jump"
        delta = "if(position.x-state.previous.x!=motion.displacement.x||position.y-state.previous.y!=motion.displacement.y||position.z-state.previous.z!=motion.displacement.z)state.failures|=32;"
        travel = "uint32_t((position.z-state.start.z).raw())"
    else:
        position = "return {actor.root.transform.position[0],actor.root.transform.position[1],0.0};"
        command = "value.command.velocity.x=step<75?epok::Fixed(96.0):epok::Fixed(0.0);"
        words = "uint32_t(value.status),uint32_t(value.command.velocity.x.raw()),uint32_t(value.command.velocity.y.raw()),value.command.jump"
        delta = "if(position.x-state.previous.x!=motion.displacement.x||position.y-state.previous.y!=motion.displacement.y)state.failures|=32;"
        travel = "uint32_t((position.x-state.start.x).raw())"
    for key, value in {"POSITION": position, "COMMAND_WORDS": words, "COMMAND": command, "DELTA": delta,
                       "TRAVEL": travel, "DOMAIN": "World3D" if dimension == "3D" else "World2D",
                       "DIM": dimension, "MAGIC": str(magic) + "u"}.items():
        source = source.replace(key, value)
    for key in ("base", "cpp", "reporter"):
        source = source.replace('"' + key.upper() + '"', '"' + identity(key) + '"')
    write(project / "assets/scripts/CommandProviderProbe.hpp", source)
    classes = {c["cpp_name"]: c for c in json.loads(run("--project", project, "--reflect"))["classes"]}
    base = classes["CommandProviderProbe"]
    functions = {f["name"]: f for f in base["functions"]}
    source_class = classes["epok::CharacterCommandSource" + dimension + "Component"]
    events = {f["name"]: f for f in source_class["functions"]}
    bp_path = project / "assets/Blueprints/BP_CommandProviderProbe.epokbp"
    if not bp_path.exists():
        run("--project", project, "--new-blueprint", "BP_CommandProviderProbe", "--parent", identity("base"))
    asset = documents.loads(bp_path.read_text(encoding="utf-8"))
    asset["functions"] = []
    for method, called, parameter in (("calculate_command", "next_command", "delta_seconds"),
                                       ("command_submitted", "observe_submission", "result")):
        event = events[method]
        name = functions[called]["parameters"][0]["name"]
        entry = {"id": str(uuid.uuid4()), "kind": {"kind": "entry"}, "inputs": {}, "outputs": {}}
        call = {"id": str(uuid.uuid4()), "kind": {"kind": "call", "function": functions[called]["id"]},
                "inputs": {name: {"kind": "parameter", "name": parameter}}, "outputs": {}}
        returned = {"id": str(uuid.uuid4()), "kind": {"kind": "return"}, "inputs": {}, "outputs": {}}
        if method == "calculate_command":
            returned["inputs"]["value"] = {"kind": "link", "node": call["id"], "pin": "value"}
        entry["outputs"]["next"] = [call["id"]]
        call["outputs"]["next"] = [returned["id"]]
        asset["functions"].append({"id": str(uuid.uuid4()), "name": method, "override_id": event["id"],
                                   "parameters": event["parameters"], "returns": event["returns"],
                                   "entry": entry["id"], "nodes": [entry, call, returned]})
    write(bp_path, asset)
    write(project / "assets/scripts/LuaCommandProviderProbe.lua", '''---@class LuaCommandProviderProbe : CommandProviderProbe
local LuaCommandProviderProbe = CommandProviderProbe:extend()
---@override
function LuaCommandProviderProbe:calculate_command(delta_seconds)
    return self:next_command(delta_seconds)
end
---@override
function LuaCommandProviderProbe:command_submitted(result)
    self:observe_submission(result)
end
return LuaCommandProviderProbe
''')
    write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/LuaCommandProviderProbe.lua": identity("lua")}})
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["actors"] = scene["actors"][:1]
    write(scene_path, scene)

    def place(name, class_id):
        run("--project", project, "--add-actor", class_id + ":" + name)
        return documents.loads(scene_path.read_text(encoding="utf-8"))

    def component(name, cls, props):
        return {"id": str(uuid.uuid4()), "name": name, "class": {"name": cls, "class_id": classes[cls]["id"]},
                "properties": props, "overrides": list(props)}

    scene = place("Floor", classes["epok::Actor" + dimension]["id"])
    floor = scene["actors"][-1]
    floor["components"][0]["properties"]["position"] = [0, -.25, 0] if dimension == "3D" else [200, 16]
    floor["components"].append(component("Collider", "epok::Collider" + dimension + "Component", {
        "collider" if dimension == "3D" else "collider2d": {"half_extents": [100, .25, 100]} if dimension == "3D" else {"size": [2000, 16]}}))
    write(scene_path, scene)
    for slot, cls in enumerate(("CppCommandProviderProbe", "BP_CommandProviderProbe", "LuaCommandProviderProbe")):
        scene = place("Repeated display name", classes["epok::Character" + dimension]["id"])
        actor = scene["actors"][-1]
        assert len(actor["components"]) == (3 if dimension == "3D" else 4)
        actor["components"][0]["properties"]["position"] = [slot * 10, 1, 0] if dimension == "3D" else [slot * 200, 0]
        if dimension == "3D":
            shape = next(c for c in actor["components"] if c["class"]["class_id"] == classes["epok::Collider3DComponent"]["id"])
            shape["properties"]["collider"] = {"half_extents": [.4, 1, .4]}
        type_id = identity("cpp") if slot == 0 else asset["id"] if slot == 1 else identity("lua")
        producer = component("Source", "CommandProviderProbe", {"probe_slot": slot})
        producer["class"] = {"name": cls, "class_id": type_id}
        actor["components"].extend([producer, component("Reporter", "CommandProbeReporter", {"probe_slot": slot})])
        write(scene_path, scene)
    run("--project", project, "--compile-blueprints")
    if prepare_only:
        print("PASS", domain, "three derived command sources prepared", flush=True)
        return
    results = []
    for mode in modes or ("native_cpp", "vm_bytecode", "vm_source"):
        manifest["lua_execution"] = mode
        write(descriptor, manifest)
        run("--project", project, "--analyze-memory", label=mode + "-build.log")
        build = project / ".epok/build"
        execution = int(re.search(r"-DEPOK_LUA_MODE=(\d+)", (build / "sources.mk").read_text())[1])
        expected = magic + execution
        address = int(re.findall(r"0x([0-9a-fA-F]+)\s+character_command_probe\s*$", (build / "epok.map").read_text(), re.M)[-1], 16) & 0x1fffff
        analyzed_hash = hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest()
        with socket.socket() as probe:
            if os.name == "nt":
                probe.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
            probe.bind(("127.0.0.1", 8077))
        with (out / (mode + "-play.log")).open("w", encoding="utf-8") as log:
            process = subprocess.Popen([str(EDITOR), "--project", str(project), "--play-psx", "--stop-after", "30"], cwd=ROOT,
                                       stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            values = None
            try:
                deadline = time.monotonic() + 240
                while process.poll() is None and time.monotonic() < deadline:
                    try:
                        with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                            values = list(struct.unpack_from("<24I", response.read(), address))
                        if all(values[i * 8] == expected for i in range(3)):
                            break
                    except (OSError, ValueError):
                        pass
                    time.sleep(.05)
                assert values is not None and values[:8] == values[8:16] == values[16:], (domain, mode, values)
                assert values[:3] == [expected, 120, 0] and values[3] > 0 and values[4] == 1 and values[6] > 0 and values[7] == 120, (domain, mode, values)
                assert process.wait(timeout=60) == 0
                assert hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest() == analyzed_hash, "Play changed the analyzed executable"
            finally:
                (out / (mode + "-observation.json")).write_text(json.dumps({
                    "expected": expected, "probe_address": address, "values": values,
                    "analyzed_exe_sha256": analyzed_hash,
                    "played_exe_sha256": hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest(),
                    "played_probe_address": int(re.findall(r"0x([0-9a-fA-F]+)\s+character_command_probe\s*$", (build / "epok.map").read_text(), re.M)[-1], 16) & 0x1fffff,
                }, indent=2) + "\n", encoding="utf-8")
                if process.poll() is None:
                    if os.name == "nt":
                        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True, timeout=15)
                    else:
                        process.terminate()
                    process.wait(timeout=15)
        results.append({"domain": domain, "mode": mode, "cpp": values[:8], "blueprint": values[8:16], "lua": values[16:],
                        "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(),
                        "exe_sha256": hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest(),
                        "ram_static_bytes": json.loads((build / "memory-report.json").read_text())["ram"]["used"]})
        (out / "result.json").write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
        print("PASS", domain, mode, "three derived command sources, result callbacks and automatic physics", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/character-commands")
    parser.add_argument("--domain", choices=("world3d", "world2d"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    for domain in args.domain or ("world3d", "world2d"):
        verify(args.output.resolve(), domain, args.mode, args.prepare_only, args.standalone_only)
