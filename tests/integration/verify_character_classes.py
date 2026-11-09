"""Exercise Character2D/3D inheritance and automatic tick stages on PSX."""
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


def verify(out, domain, modes, prepare_only, standalone_only=False):
    out = out / domain
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    dimension = "3D" if domain == "world3d" else "2D"
    native = "epok::Character" + dimension
    namespace = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
    identity = lambda name: str(uuid.uuid5(namespace, "character-class:" + domain + ":" + name))
    magic = 0x47000000 | (uuid.uuid4().int & 0xffffff)
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
        assert (export / "character2d.hpp").is_file() and (export / "character3d.hpp").is_file()
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
        run("--create-project", project, "--template", "basic", "--name", "Character " + dimension)
    source = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t character_class_probe[24]={};}
struct CharacterTestState {uint32_t frame=0,samples=0,failures=0,grounded=0,jumps=0,hash=2166136261u;epok::GameplayVector3 previous{},start{};};
inline CharacterTestState character_test_states[3];
inline epok::GameplayVector3 character_position(epok::CharacterDIM& actor){POSITION}
inline uint32_t character_motion_hash(epok::CharacterMotionSampleDIM motion){
 uint32_t hash=2166136261u;const uint32_t words[]={WORDS};for(auto word:words)hash=(hash^word)*16777619u;return hash;
}
class EPOK_CLASS(Abstract,Blueprintable,Id="BASE") CharacterProviderProbe:public epok::CharacterDIM {
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 EPOK_FUNCTION(BlueprintEvent) virtual epok::CharacterMotionSampleDIM apply_command(epok::CharacterCommandDIM command){(void)command;return {};}
 void begin_play()override{epok::CharacterDIM::begin_play();if(probe_slot<3){auto& s=character_test_states[probe_slot];s={};s.previous=s.start=character_position(*this);}}
 void tick(epok::Fixed)override{
  if(probe_slot>=3)return;auto& s=character_test_states[probe_slot];if(s.frame>=120)return;
  epok::CharacterCommandDIM command;COMMAND
  command.jump=s.frame==15;
  const auto expected=motion();const auto returned=apply_command(command);
  if(character_motion_hash(expected)!=character_motion_hash(returned))s.failures|=1;
  ++s.frame;
 }
};
class EPOK_CLASS(Blueprintable,Id="CPP") CppCharacterProviderProbe:public CharacterProviderProbe {
public:
 epok::CharacterMotionSampleDIM apply_command(epok::CharacterCommandDIM command)override{submit_command(command);return motion();}
};
class EPOK_CLASS(Blueprintable,Domain=DOMAIN,Owners=DOMAIN,Id="REPORTER") CharacterStepReporter:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 void tick(epok::Fixed dt)override{
  if(probe_slot>=3)return;auto& s=character_test_states[probe_slot];if(s.samples>=120)return;
  auto* actor=epok::active_object_registry->resolve<epok::CharacterDIM>(owner_id());if(!actor)return;
  if(s.frame!=s.samples+1)s.failures|=2;
  const auto view=actor->motion();if(view.status!=epok::CharacterMotionStatus::Applied)s.failures|=4;
  const auto at=character_position(*actor);DELTA
  CAMERA
  s.previous=at;++s.samples;s.grounded+=view.grounded;s.jumps+=view.jumped;s.hash=(s.hash^character_motion_hash(view))*16777619u;
  if(s.samples==120){
   const auto saved=at;uint32_t api=0;
   if(actor->motor.simulate(0.0).status==epok::CharacterMotionStatus::NoStep&&character_position(*actor).x==saved.x)api|=1;
   const auto gravity=actor->motor.gravity;actor->motor.gravity=-1.0;
   if(actor->motor.simulate(dt).status==epok::CharacterMotionStatus::InvalidConfiguration&&character_position(*actor).x==saved.x)api|=2;
   actor->motor.gravity=gravity;
   auto* p=character_class_probe+probe_slot*8;p[1]=s.samples;p[2]=s.failures;p[3]=s.grounded;p[4]=s.jumps;p[5]=s.hash;p[6]=TRAVEL;p[7]=api;p[0]=MAGIC+EPOK_LUA_MODE;
  }
 }
};
'''
    if dimension == "3D":
        position = "auto* data=actor.data();return data?epok::GameplayVector3{data->transform.position[0],data->transform.position[1],data->transform.position[2]}:epok::GameplayVector3{};"
        words = "uint32_t(motion.status),uint32_t(motion.velocity.x.raw()),uint32_t(motion.velocity.y.raw()),uint32_t(motion.velocity.z.raw()),uint32_t(motion.displacement.x.raw()),uint32_t(motion.displacement.y.raw()),uint32_t(motion.displacement.z.raw()),uint32_t(motion.facing.raw()),uint32_t(motion.speed.raw()),motion.grounded,motion.jumped,motion.blocked,motion.unresolved_overlap"
        command = "command.velocity.y=s.frame<75?epok::Fixed(6.0):epok::Fixed(0.0);command.heading=0.0;command.turn=true;"
        delta = "if(at.x-s.previous.x!=view.displacement.x||at.y-s.previous.y!=view.displacement.y||at.z-s.previous.z!=view.displacement.z)s.failures|=8;"
        camera = "if(probe_slot==0){auto* rig=epok::active_object_registry->resolve<epok::FirstPersonCamera>(actor->camera);if(!rig||rig->sample().status!=epok::CameraRigStatus::Applied||rig->sample().position.x!=at.x||rig->sample().position.z!=at.z)s.failures|=16;}"
        travel = "uint32_t((at.z-s.start.z).raw())"
    else:
        position = "return {actor.root.transform.position[0],actor.root.transform.position[1],0.0};"
        words = "uint32_t(motion.status),uint32_t(motion.velocity.x.raw()),uint32_t(motion.velocity.y.raw()),uint32_t(motion.displacement.x.raw()),uint32_t(motion.displacement.y.raw()),motion.grounded,motion.jumped,motion.on_wall,motion.on_ceiling,motion.blocked"
        command = "command.velocity.x=s.frame<75?epok::Fixed(96.0):epok::Fixed(0.0);"
        delta = "if(at.x-s.previous.x!=view.displacement.x||at.y-s.previous.y!=view.displacement.y)s.failures|=8;"
        camera = ""
        travel = "uint32_t((at.x-s.start.x).raw())"
    for key, value in {"POSITION": position, "WORDS": words, "COMMAND": command, "DELTA": delta,
                       "CAMERA": camera, "TRAVEL": travel, "DOMAIN": "World3D" if dimension == "3D" else "World2D",
                       "DIM": dimension, "MAGIC": str(magic) + "u"}.items():
        source = source.replace(key, value)
    for key in ("base", "cpp", "reporter"):
        source = source.replace('"' + key.upper() + '"', '"' + identity(key) + '"')
    write(project / "assets/scripts/CharacterProviderProbe.hpp", source)
    classes = {c["cpp_name"]: c for c in json.loads(run("--project", project, "--reflect"))["classes"]}
    functions = {f["name"]: f for f in classes[native]["functions"]}
    event = next(f for f in classes["CharacterProviderProbe"]["functions"] if f["name"] == "apply_command")
    bp_path = project / "assets/Blueprints/BP_CharacterProviderProbe.epokbp"
    if not bp_path.exists():
        run("--project", project, "--new-blueprint", "BP_CharacterProviderProbe", "--parent", identity("base"))
    asset = documents.loads(bp_path.read_text(encoding="utf-8"))
    nodes = []

    def node(kind, inputs=None, **fields):
        value = {"id": str(uuid.uuid4()), "kind": {"kind": kind, **fields}, "inputs": inputs or {}, "outputs": {}}
        nodes.append(value)
        return value

    entry = node("entry")
    submitted = node("call", {"command": {"kind": "parameter", "name": "command"}}, function=functions["submit_command"]["id"])
    sampled = node("call", function=functions["motion"]["id"])
    returned = node("return", {"value": {"kind": "link", "node": sampled["id"], "pin": "value"}})
    entry["outputs"]["next"] = [submitted["id"]]
    submitted["outputs"]["next"] = [returned["id"]]
    asset["functions"] = [{"id": str(uuid.uuid4()), "name": "apply_command", "override_id": event["id"],
                           "parameters": event["parameters"], "returns": event["returns"], "entry": entry["id"], "nodes": nodes}]
    write(bp_path, asset)
    write(project / "assets/scripts/LuaCharacterProviderProbe.lua", '''---@class LuaCharacterProviderProbe : CharacterProviderProbe
local LuaCharacterProviderProbe = CharacterProviderProbe:extend()
---@override
function LuaCharacterProviderProbe:apply_command(command)
    self:submit_command(command)
    return self:motion()
end
return LuaCharacterProviderProbe
''')
    write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/LuaCharacterProviderProbe.lua": identity("lua")}})
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
        return {"id": str(uuid.uuid4()), "name": name, "class": {"name": cls, "class_id": classes[cls]["id"]}, "properties": props, "overrides": list(props)}

    scene = place("Floor", classes["epok::Actor" + dimension]["id"])
    floor = scene["actors"][-1]
    floor["components"][0]["properties"]["position"] = [0, -.25, 0] if dimension == "3D" else [200, 16]
    floor["components"].append(component("Collider", "epok::Collider" + dimension + "Component", {
        "collider" if dimension == "3D" else "collider2d": {"half_extents": [100, .25, 100]} if dimension == "3D" else {"size": [2000, 16]}}))
    write(scene_path, scene)
    camera_id = None
    if dimension == "3D":
        scene = place("Camera before players", classes["epok::FirstPersonCamera"]["id"])
        camera_id = scene["actors"][-1]["id"]
    for slot, (name, cls) in enumerate((("CppCharacterProviderProbe", identity("cpp")),
                                      ("BP_CharacterProviderProbe", asset["id"]), ("LuaCharacterProviderProbe", identity("lua")))):
        scene = place("Same display name", cls)
        actor = scene["actors"][-1]
        assert len(actor["components"]) == (3 if dimension == "3D" else 4)
        actor["properties"]["probe_slot"] = slot
        actor["overrides"] = list(actor["properties"])
        actor["components"][0]["properties"]["position"] = [slot * 10, 1, 0] if dimension == "3D" else [slot * 200, 0]
        if dimension == "3D":
            shape = next(c for c in actor["components"] if c["class"]["class_id"] == classes["epok::Collider3DComponent"]["id"])
            shape["properties"]["collider"] = {"half_extents": [.4, 1, .4]}
            if slot == 0:
                actor["properties"]["camera"] = camera_id
                camera = next(a for a in scene["actors"] if a["id"] == camera_id)
                camera["properties"]["target"] = actor["id"]
                camera["overrides"] = list(camera["properties"])
        actor["overrides"] = list(actor["properties"])
        actor["components"].append(component("Reporter", "CharacterStepReporter", {"probe_slot": slot}))
        write(scene_path, scene)
    run("--project", project, "--compile-blueprints")
    if prepare_only:
        print("PASS", domain, "three derived character classes prepared", flush=True)
        return
    results = []
    for mode in modes or ("native_cpp", "vm_bytecode", "vm_source"):
        manifest["lua_execution"] = mode
        write(descriptor, manifest)
        run("--project", project, "--analyze-memory", label=mode + "-build.log")
        build = project / ".epok/build"
        execution = int(re.search(r"-DEPOK_LUA_MODE=(\d+)", (build / "sources.mk").read_text())[1])
        expected = magic + execution
        address = int(re.findall(r"0x([0-9a-fA-F]+)\s+character_class_probe\s*$", (build / "epok.map").read_text(), re.M)[-1], 16) & 0x1fffff
        with socket.socket() as probe:
            if os.name == "nt":
                probe.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
            probe.bind(("127.0.0.1", 8077))
        with (out / (mode + "-play.log")).open("w", encoding="utf-8") as log:
            process = subprocess.Popen([str(EDITOR), "--project", str(project), "--play-psx", "--stop-after", "8"], cwd=ROOT, stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
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
                assert values[0:3] == [expected, 120, 0] and values[3] > 0 and values[4] == 1 and values[6] > 0 and values[7] == 3, (domain, mode, values)
                assert process.wait(timeout=30) == 0
            finally:
                if process.poll() is None:
                    if os.name == "nt":
                        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True, timeout=15)
                    else:
                        process.terminate()
                    process.wait(timeout=15)
        results.append({"domain": domain, "mode": mode, "cpp": values[:8], "blueprint": values[8:16], "lua": values[16:],
                        "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "exe_sha256": hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest(),
                        "ram_static_bytes": json.loads((build / "memory-report.json").read_text())["ram"]["used"]})
        (out / "result.json").write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
        print("PASS", domain, mode, "three derived characters, same-step automatic physics and complete motion records", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/character-classes")
    parser.add_argument("--domain", choices=("world3d", "world2d"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    for domain in args.domain or ("world3d", "world2d"):
        verify(args.output.resolve(), domain, args.mode, args.prepare_only, args.standalone_only)
