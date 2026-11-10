"""Execute interaction overrides and parent calls in both domains on the PSX target."""
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
OUT = ROOT / "artifacts/gameplay-helpers/p2/providers"
PROJECT = OUT / "project"
BASE = "c476395a-125b-569b-943d-c75fcb4c1885"
CPP = "d2bafcc1-300d-5ac9-bee1-028e2217d01c"
DRIVER = "a386c3b4-238f-52b5-a40d-c96a3295ef01"
LUA = "bd80bda1-0f76-5cd1-9396-caf1d8e31ae6"
MAGIC = 0x494e5432


def run(*args):
    result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT,
                            capture_output=True, text=True, encoding="utf-8", timeout=900)
    if result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    return result.stdout


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    documents.write_text(path, documents.dumps(value) if isinstance(value, dict) else value)


def identity():
    return str(uuid.uuid4())


def prepare():
    OUT.mkdir(parents=True, exist_ok=True)
    if not PROJECT.exists():
        run("--create-project", PROJECT, "--template", "basic", "--name", "Interaction Providers")
    write(PROJECT / "assets/scripts/InteractionProbe.hpp", '''#pragma once
#include "epok.hpp"
extern "C" { inline uint32_t interaction_probe[48]={}; }
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="BASE") InteractionProbe : public epok::InteractableComponent {
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 EPOK_PROPERTY(EditAnywhere) bool allowed=true;
 EPOK_FUNCTION(BlueprintEvent,BlueprintPure) bool can_interact(epok::ObjectId initiator) const override {
   return allowed&&initiator.valid()&&epok::InteractableComponent::can_interact(initiator);
 }
 EPOK_FUNCTION(BlueprintEvent) void interaction_started(epok::ObjectId initiator,uint32_t port) override {
   epok::InteractableComponent::interaction_started(initiator,port);
   if(probe_slot<6)++interaction_probe[probe_slot*8+4];
 }
 void interaction_finished(epok::ObjectId,uint32_t,epok::InteractionResult) override {
   if(probe_slot<6)++interaction_probe[probe_slot*8+6];
 }
 EPOK_FUNCTION(BlueprintCallable) void stamp(){if(probe_slot<6)++interaction_probe[probe_slot*8+5];}
};
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="CPP") CppInteractionProbe : public InteractionProbe {
public:
 bool can_interact(epok::ObjectId initiator) const override{return InteractionProbe::can_interact(initiator);}
 void interaction_started(epok::ObjectId initiator,uint32_t port) override{InteractionProbe::interaction_started(initiator,port);stamp();}
};
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="DRIVER") InteractionDriver : public epok::ActorComponent {
 bool done=false;
public:
 EPOK_PROPERTY(EditAnywhere,ComponentClass="InteractionProbe") epok::ObjectId target{};
 void on_frame(uint32_t) override {
   if(done)return;done=true;
   auto* value=epok::active_object_registry->resolve<InteractionProbe>(target);
   if(!value||value->probe_slot>=6)return;
   auto* output=interaction_probe+value->probe_slot*8;
   output[1]=uint32_t(value->availability(owner_id()));
   value->allowed=false;output[2]=uint32_t(value->availability(owner_id()));value->allowed=true;
   output[3]=uint32_t(value->request(owner_id(),0));
   output[7]=!value->is_busy();output[0]=0x494e5432;
 }
};
'''.replace('"BASE"', f'"{BASE}"').replace('"CPP"', f'"{CPP}"').replace('"DRIVER"', f'"{DRIVER}"'))
    metadata = json.loads(run("--project", PROJECT, "--reflect"))
    base = next(c for c in metadata["classes"] if c["id"] == BASE)
    functions = {f["name"]: f for f in base["functions"]}
    path = PROJECT / "assets/Blueprints/BP_InteractionProbe.epokbp"
    if not path.exists():
        run("--project", PROJECT, "--new-blueprint", "BP_InteractionProbe", "--parent", BASE)
    asset = documents.loads(path.read_text(encoding="utf-8"))

    def node(kind, inputs=None, **fields):
        return {"id": identity(), "kind": {"kind": kind, **fields}, "inputs": inputs or {}, "outputs": {}}

    def graph(name):
        function = functions[name]
        params = {p["name"]: {"kind": "parameter", "name": p["name"]} for p in function["parameters"]}
        entry, parent = node("entry"), node("call_parent", params)
        if function["returns"]["kind"] == "bool":
            finish = node("return", {"value": {"kind": "link", "node": parent["id"], "pin": "value"}})
            entry["outputs"]["next"] = [parent["id"]]
            parent["outputs"]["next"] = [finish["id"]]
        else:
            finish = node("call", function=functions["stamp"]["id"])
            entry["outputs"]["next"] = [parent["id"]]
            parent["outputs"]["next"] = [finish["id"]]
        return {"id": identity(), "name": name, "override_id": function["id"],
                "parameters": function["parameters"], "returns": function["returns"],
                "entry": entry["id"], "nodes": [entry, parent, finish]}

    asset["functions"] = [graph("can_interact"), graph("interaction_started")]
    write(path, asset)
    write(PROJECT / "assets/scripts/LuaInteractionProbe.lua", '''---@class LuaInteractionProbe : InteractionProbe
local LuaInteractionProbe = InteractionProbe:extend()
---@override
function LuaInteractionProbe:can_interact(initiator)
    return LuaInteractionProbe.super.can_interact(self, initiator)
end
---@override
function LuaInteractionProbe:interaction_started(initiator, port)
    LuaInteractionProbe.super.interaction_started(self, initiator, port)
    self:stamp()
end
return LuaInteractionProbe
''')
    write(PROJECT / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/LuaInteractionProbe.lua": LUA}})
    # Current-format actors/components, not legacy renderer slots or name lookups.
    descriptor = next(PROJECT.glob("*.epokproject"))
    settings = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = PROJECT / settings["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    camera = copy.deepcopy(scene["actors"][0])
    scene["actors"] = [camera]
    reflected = {c["cpp_name"]: c for c in metadata["classes"]}

    def actor(name, domain):
        value = copy.deepcopy(camera)
        cls = reflected[f"epok::Actor{domain}D"]
        value.update(id=identity(), name=name, **{"class": {"name": cls["cpp_name"], "class_id": cls["id"]}},
                     properties={}, overrides=[])
        root = value["components"][0]
        component = reflected[f"epok::SceneComponent{domain}D"]
        root.update(id=identity(), **{"class": {"name": component["cpp_name"], "class_id": component["id"]}})
        root["properties"] = {}
        root["overrides"] = []
        value["components"] = [root]
        return value

    def component(name, class_id, props):
        return {"id": identity(), "name": name, "class": {"name": name, "class_id": class_id},
                "properties": props, "overrides": list(props)}

    for domain in (3, 2):
        for index, (name, class_id) in enumerate((("CppInteractionProbe", CPP), ("BP_InteractionProbe", asset["id"]), ("LuaInteractionProbe", LUA))):
            slot = (3 - domain) * 3 + index
            source, target = actor(f"Source {slot}", domain), actor(f"Target {slot}", domain)
            endpoint = component(name, class_id, {"probe_slot": slot})
            target["components"].append(endpoint)
            source["components"].append(component("InteractionDriver", DRIVER, {"target": endpoint["id"]}))
            scene["actors"].extend([source, target])
    write(scene_path, scene)
    run("--project", PROJECT, "--compile-blueprints")
    return descriptor


def emulate(mode):
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 8077))
    mapping = (PROJECT / ".epok/build/epok.map").read_text()
    address = int(re.findall(r"0x([0-9a-fA-F]+)\s+interaction_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
    expected = [MAGIC, 0, 1, 0, 1, 1, 1, 1] * 6
    with (OUT / f"{mode}-play.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(EDITOR), "--project", str(PROJECT), "--play-psx", "--stop-after", "6"],
                                   cwd=ROOT, stdout=log, stderr=log,
                                   creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        values = None
        try:
            deadline = time.monotonic() + 120
            while process.poll() is None and time.monotonic() < deadline:
                try:
                    with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                        values = list(struct.unpack_from("<48I", response.read(), address))
                    if values == expected:
                        break
                except (OSError, ValueError):
                    pass
                time.sleep(.1)
            assert values == expected, {"mode": mode, "actual": values, "expected": expected}
            assert process.wait(timeout=30) == 0
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    return values


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--mode", action="append", choices=("native_cpp", "vm_bytecode", "vm_source"))
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    if args.standalone_only:
        export = Path(run("--project", PROJECT, "--export-psx").strip())
        env = os.environ.copy()
        env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
        result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
                                 "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
                                cwd=export, env=env, capture_output=True, text=True, timeout=900)
        (OUT / "standalone-build.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        original = (PROJECT / ".epok/build/epok.ps-exe").read_bytes()
        assert (export / "epok.ps-exe").read_bytes() == original
        mode = documents.loads(next(PROJECT.glob("*.epokproject")).read_text())["lua_execution"]
        (OUT / "standalone-result.json").write_text(json.dumps({"mode": mode, "export": str(export), "identical": True,
                                                              "sha256": hashlib.sha256(original).hexdigest()}, indent=2) + "\n", encoding="utf-8")
        print("PASS standalone interaction rebuild, byte-identical", mode, flush=True)
        return
    descriptor = prepare()
    if args.prepare_only:
        print("PASS interaction fixture prepared and Blueprint compiled", flush=True)
        return
    path = OUT / "result.json"
    results = json.loads(path.read_text()) if args.mode and path.exists() else []
    for mode in args.mode or ("native_cpp", "vm_bytecode", "vm_source"):
        settings = documents.loads(descriptor.read_text(encoding="utf-8"))
        settings["lua_execution"] = mode
        write(descriptor, settings)
        output = run("--project", PROJECT, "--analyze-memory")
        (OUT / f"{mode}-build.log").write_text(output, encoding="utf-8")
        values = emulate(mode)
        results = [r for r in results if r["mode"] != mode]
        results.append({"mode": mode, "values": values,
                        "memory": json.loads((PROJECT / ".epok/build/memory-report.json").read_text())["ram"]["used"],
                        "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(),
                        "exe_sha256": hashlib.sha256((PROJECT / ".epok/build/epok.ps-exe").read_bytes()).hexdigest(),
                        "runtime_sources": {name: hashlib.sha256((PROJECT / ".epok/build" / name).read_bytes()).hexdigest()
                                            for name in ("epok.hpp", "object_model.hpp", "interaction.hpp", "gameplay_control.hpp")}})
        path.write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
        print("PASS", mode, "C++/Blueprint/Lua bool overrides, parent calls and completion in 2D/3D", flush=True)


if __name__ == "__main__":
    main()
