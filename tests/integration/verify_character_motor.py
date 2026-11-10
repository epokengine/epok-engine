"""Run the shared 3D motor from C++, Blueprint and every Lua execution mode.

Each provider implements an event that submits a command and returns the motor's
motion record. The fixture produces commands and checks actual collision motion;
it contains no replacement movement algorithm.
"""
from __future__ import annotations
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import struct
import subprocess
import sys
import time
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

EDITOR = ROOT / "target/debug/epok-editor.exe"
NAMESPACE = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
MOTOR = "34a2d5d2-b0bb-5faf-8dc4-7f34f07f9c95"


def identity(name):
    return str(uuid.uuid5(NAMESPACE, "motor-acceptance:" + name))


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    documents.write_text(path, documents.dumps(value) if isinstance(value, dict) else value)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/motor-providers")
    parser.add_argument("--mode", action="append", choices=("native_cpp", "vm_bytecode", "vm_source"))
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    run_magic = 0x43000000 | (uuid.uuid4().int & 0xffffff)
    project = out / "project"
    counter = 0

    def run(*command, label=None):
        nonlocal counter
        counter += 1
        result = subprocess.run([str(EDITOR), *map(str, command)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / (label or f"command-{counter}.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if args.standalone_only:
        export = Path(run("--project", project, "--export-psx", label="standalone-export.log"))
        assert (export / "character3d.hpp").is_file()
        env = os.environ.copy()
        env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
        result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
                                 "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
                                cwd=export, env=env, capture_output=True, text=True, timeout=900)
        (out / "standalone-build.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        original = (project / ".epok/build/epok.ps-exe").read_bytes()
        assert (export / "epok.ps-exe").read_bytes() == original
        mode = documents.loads(next(project.glob("*.epokproject")).read_text())["lua_execution"]
        (out / "standalone-result.json").write_text(json.dumps({"mode": mode, "export": str(export), "identical": True,
            "sha256": hashlib.sha256(original).hexdigest(),
            "motor_sha256": hashlib.sha256((export / "character3d.hpp").read_bytes()).hexdigest()}, indent=2) + "\n", encoding="utf-8")
        print("PASS standalone motor rebuild, byte-identical", mode, flush=True)
        return

    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Motor Providers")
    source = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t character_motor_probe[24]={};}
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="BASE") MotorProviderProbe:public epok::CharacterMotor3DComponent {
 uint32_t frame_=0,failures_=0,jumps_=0,grounded_=0,hash_=2166136261u;epok::Fixed start_=0.0;
 void mix(uint32_t word){hash_=(hash_^word)*16777619u;}
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 EPOK_FUNCTION(BlueprintEvent) virtual epok::CharacterMotionSample3D run_command(epok::CharacterCommand3D command,epok::Fixed dt){(void)command;(void)dt;return {};}
 void begin_play()override{epok::CharacterMotor3DComponent::begin_play();automatic_tick=false;if(get_owner()&&get_owner()->data())start_=get_owner()->data()->transform.position[0];}
 void on_frame(uint32_t)override{
  if(frame_>=120||probe_slot>=3)return;auto* owner=get_owner();auto* data=owner?owner->data():nullptr;if(!data)return;
  epok::CharacterCommand3D command;command.velocity.x=frame_<75?epok::Fixed(6.0):epok::Fixed(0.0);
  command.heading=90.0;command.turn=true;command.jump=frame_==15;
  const auto before=data->transform;const auto sample=run_command(command,epok::Fixed(1.0/60.0));
  if(sample.status!=epok::CharacterMotionStatus::Applied)failures_|=1;
  const epok::Fixed delta[3]={sample.displacement.x,sample.displacement.y,sample.displacement.z};
  for(unsigned i=0;i<3;++i)if(delta[i]!=data->transform.position[i]-before.position[i])failures_|=2;
  jumps_+=sample.jumped;grounded_+=sample.grounded;
  const uint32_t words[]={uint32_t(sample.status),uint32_t(sample.velocity.x.raw()),uint32_t(sample.velocity.y.raw()),uint32_t(sample.velocity.z.raw()),uint32_t(sample.displacement.x.raw()),uint32_t(sample.displacement.y.raw()),uint32_t(sample.displacement.z.raw()),uint32_t(sample.facing.raw()),uint32_t(sample.speed.raw()),sample.grounded,sample.jumped,sample.blocked,sample.unresolved_overlap};
  for(auto word:words)mix(word);++frame_;
  if(frame_==120){auto* p=character_motor_probe+probe_slot*8;p[1]=frame_;p[2]=failures_;p[3]=jumps_;p[4]=grounded_;p[5]=hash_;p[6]=uint32_t(data->transform.position[1].raw());p[7]=uint32_t((data->transform.position[0]-start_).raw());p[0]=0x43484D35;}
 }
};
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="CPP") CppMotorProviderProbe:public MotorProviderProbe {
public:
 epok::CharacterMotionSample3D run_command(epok::CharacterCommand3D command,epok::Fixed dt)override{submit(command);simulate(dt);return motion();}
};
'''.replace('"BASE"', '"' + identity("base") + '"').replace('"CPP"', '"' + identity("cpp") + '"')
    source = source.replace("0x43484D35", str(run_magic) + "u+EPOK_LUA_MODE")
    write(project / "assets/scripts/MotorProviderProbe.hpp", source)
    metadata = json.loads(run("--project", project, "--reflect"))
    classes = {c["cpp_name"]: c for c in metadata["classes"]}
    base = classes["MotorProviderProbe"]
    functions = {f["name"]: f for f in classes["epok::CharacterMotor3DComponent"]["functions"]}
    event = next(f for f in base["functions"] if f["name"] == "run_command")
    bp_path = project / "assets/Blueprints/BP_MotorProviderProbe.epokbp"
    if not bp_path.exists():
        run("--project", project, "--new-blueprint", "BP_MotorProviderProbe", "--parent", base["id"])
    asset = documents.loads(bp_path.read_text(encoding="utf-8"))
    nodes = []

    def node(kind, inputs=None, **fields):
        value = {"id": str(uuid.uuid4()), "kind": {"kind": kind, **fields}, "inputs": inputs or {}, "outputs": {}}
        nodes.append(value)
        return value

    entry = node("entry")
    submitted = node("call", {"command": {"kind": "parameter", "name": "command"}}, function=functions["submit"]["id"])
    simulated = node("call", {"delta_seconds": {"kind": "parameter", "name": "dt"}}, function=functions["simulate"]["id"])
    sampled = node("call", function=functions["motion"]["id"])
    returned = node("return", {"value": {"kind": "link", "node": sampled["id"], "pin": "value"}})
    for first, second in zip((entry, submitted, simulated), (submitted, simulated, returned)):
        first["outputs"]["next"] = [second["id"]]
    asset["functions"] = [{"id": str(uuid.uuid4()), "name": "run_command", "override_id": event["id"],
                           "parameters": event["parameters"], "returns": event["returns"], "entry": entry["id"], "nodes": nodes}]
    write(bp_path, asset)
    write(project / "assets/scripts/LuaMotorProviderProbe.lua", '''---@class LuaMotorProviderProbe : MotorProviderProbe
local LuaMotorProviderProbe = MotorProviderProbe:extend()
---@override
function LuaMotorProviderProbe:run_command(command, dt)
    self:submit(command)
    self:simulate(dt)
    return self:motion()
end
return LuaMotorProviderProbe
''')
    write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/LuaMotorProviderProbe.lua": identity("lua")}})
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    camera = copy.deepcopy(scene["actors"][0])
    scene["actors"] = [camera]

    def component(name, class_id, props):
        return {"id": str(uuid.uuid4()), "name": name, "class": {"name": name, "class_id": class_id},
                "properties": props, "overrides": list(props)}

    def actor(name, position, half):
        value = copy.deepcopy(camera)
        value.update(id=str(uuid.uuid4()), name=name, properties={}, overrides=[])
        value["components"] = [c for c in value["components"] if c.get("root")]
        root = value["components"][0]
        root["id"] = str(uuid.uuid4())
        root["properties"].update(position=position, rotation=[0, 0, 0])
        collider = classes["epok::Collider3DComponent"]
        value["components"].append(component("Collider", collider["id"], {"collider": {
            "enabled": True, "trigger": False, "center": [0, 0, 0], "half_extents": half,
            "layer": 1, "mask": 4294967295}}))
        return value

    scene["actors"].append(actor("Floor", [0, -.25, 0], [100, .25, 100]))
    for slot, (name, class_id) in enumerate((("CppMotorProviderProbe", identity("cpp")),
                                           ("BP_MotorProviderProbe", asset["id"]), ("LuaMotorProviderProbe", identity("lua")))):
        body = actor("Same display name", [slot * 10, 1, 0], [.4, 1, .4])
        body["components"].append(component(name, class_id, {"probe_slot": slot, "automatic_tick": False}))
        scene["actors"].append(body)
    write(scene_path, scene)
    run("--project", project, "--compile-blueprints")
    if args.prepare_only:
        print("PASS three provider classes compiled against the common motor", flush=True)
        return

    def emulate(mode):
        flags = (project / ".epok/build/sources.mk").read_text()
        execution = int(re.search(r"-DEPOK_LUA_MODE=(\d+)", flags)[1])
        magic = run_magic + execution
        with socket.socket() as probe:
            if os.name == "nt":
                probe.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
            probe.bind(("127.0.0.1", 8077))
        mapping = (project / ".epok/build/epok.map").read_text()
        address = int(re.findall(r"0x([0-9a-fA-F]+)\s+character_motor_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
        with (out / f"{mode}-play.log").open("w", encoding="utf-8") as log:
            process = subprocess.Popen([str(EDITOR), "--project", str(project), "--play-psx", "--stop-after", "8"],
                                       cwd=ROOT, stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            values = None
            try:
                deadline = time.monotonic() + 120
                while process.poll() is None and time.monotonic() < deadline:
                    try:
                        with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                            values = list(struct.unpack_from("<24I", response.read(), address))
                        if all(values[i * 8] == magic for i in range(3)):
                            break
                    except (OSError, ValueError):
                        pass
                    time.sleep(.1)
                assert values is not None and values[:8] == values[8:16] == values[16:], (mode, values)
                assert values[0:4] == [magic, 120, 0, 1], (mode, values)
                assert values[4] > 20 and 4000 < values[6] < 4200 and 12000 < values[7] < 40000, (mode, values)
                assert process.wait(timeout=30) == 0
            finally:
                if process.poll() is None:
                    if os.name == "nt":
                        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                                       capture_output=True, timeout=15)
                    else:
                        process.terminate()
                    process.wait(timeout=15)
        return values

    result_path = out / "result.json"
    results = json.loads(result_path.read_text()) if args.mode and result_path.exists() else []
    for mode in args.mode or ("native_cpp", "vm_bytecode", "vm_source"):
        manifest["lua_execution"] = mode
        write(descriptor, manifest)
        run("--project", project, "--analyze-memory", label=f"{mode}-build.log")
        values = emulate(mode)
        row = {"mode": mode, "run_magic": run_magic, "frames_per_provider": 120, "cpp": values[:8], "blueprint": values[8:16], "lua": values[16:],
               "ram_static_bytes": json.loads((project / ".epok/build/memory-report.json").read_text())["ram"]["used"],
               "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(),
               "exe_sha256": hashlib.sha256((project / ".epok/build/epok.ps-exe").read_bytes()).hexdigest(),
               "runtime_sources": {name: hashlib.sha256((project / ".epok/build" / name).read_bytes()).hexdigest()
                                   for name in ("epok.hpp", "object_model.hpp", "character3d.hpp")}}
        results = [r for r in results if r["mode"] != mode] + [row]
        result_path.write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
        print("PASS", mode, "C++, Blueprint and Lua share 120 identical collision/motion record frames", flush=True)


if __name__ == "__main__":
    main()
