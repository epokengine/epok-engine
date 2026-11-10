"""Run the same routed-input contract from C++, Blueprint and all Lua backends.

One fixture contains three independent callers. Native fixture methods only seed
pad/clock state and record results; each caller owns its capture and API calls.
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
OUT = ROOT / "artifacts/gameplay-helpers/p1-provider"
PROJECT = OUT / "project"
BASE = "e15ccb5d-d19d-5cda-bdef-2cbfc8247ec3"
CPP = "4fd0c6a2-91d7-5ead-8e9a-1934e7e0bca6"
LUA = "10f916b2-9c5b-5840-8fb5-a8033165a2d7"
MAGIC = 0x47504331


def run(*args):
    result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT,
                            capture_output=True, text=True, encoding="utf-8", timeout=900)
    if result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    return result.stdout


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    documents.write_text(path, documents.dumps(value) if isinstance(value, dict) else value)


def ref(node):
    return {"kind": "link", "node": node["id"], "pin": "value"}


def literal(kind, value):
    return {"kind": "literal", "value_type": {"kind": kind}, "value": value}


def prepare():
    OUT.mkdir(parents=True, exist_ok=True)
    if not PROJECT.exists():
        run("--create-project", PROJECT, "--template", "basic", "--name", "Control Providers")
    header = '''#pragma once
#include "epok.hpp"
EPOK_NATIVE_REQUIREMENTS("gameplay-control")
extern "C" { inline uint32_t gameplay_control_probe[24]={}; }
class EPOK_CLASS(Blueprintable, Id="BASE") ControlProbe : public epok::Actor3D {
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 EPOK_FUNCTION(BlueprintCallable) void seed(){epok::input.reset();epok::gameplay_controls().reset();}
 EPOK_FUNCTION(BlueprintCallable) void press(){
   epok::input.sample(0,true,0);epok::gameplay_controls().begin_frame();
   epok::input.sample(0,true,1u<<unsigned(epok::Button::Cross));epok::gameplay_controls().begin_frame();
   epok::input.begin_tick();epok::gameplay_controls().begin_tick();
 }
 EPOK_FUNCTION(BlueprintCallable) void observe(epok::GameplayActionSample blocked,epok::GameplayActionSample own){
   if(probe_slot>=3)return;auto* p=gameplay_control_probe+probe_slot*8;
   p[0]=0x47504331;p[1]=uint32_t(blocked.value.raw());p[2]=blocked.pressed;
   p[3]=uint32_t(own.value.raw());p[4]=own.pressed;
 }
 EPOK_FUNCTION(BlueprintCallable) void finish(bool taken,bool repeated,bool released,bool still_valid){
   if(probe_slot>=3)return;auto* p=gameplay_control_probe+probe_slot*8;
   using G=epok::GameplayInputLibrary;
   auto transient=G::capture(id(),G::mask(epok::GameplayAction::Confirm),100,0);
   auto* scope=epok::active_object_registry->resolve<epok::Level>(level_id());
   scope->set_active(id(),false);scope->set_active(id(),true);
   p[5]=taken;p[6]=repeated;p[7]=released&&!still_valid&&!G::valid(transient);
 }
};
class EPOK_CLASS(Blueprintable, Id="CPP") CppControlProbe : public ControlProbe {
public:
 void begin_play() override{
   using G=epok::GameplayInputLibrary;using A=epok::GameplayAction;
   seed();auto token=G::capture(id(),G::mask(A::Confirm),100,0);
   token=G::owner_capture(id(),G::mask(A::Confirm),0);press();
   observe(G::sample(A::Jump,0),G::context_sample(token,A::Confirm,false));
   const bool taken=G::context_take_pressed(token,A::Confirm,false);
   const bool repeated=G::context_take_pressed(token,A::Confirm,true);
   const bool released=G::release(token);finish(taken,repeated,released,G::valid(token));
 }
};
'''.replace('"BASE"', f'"{BASE}"').replace('"CPP"', f'"{CPP}"')
    write(PROJECT / "assets/scripts/ControlProbe.hpp", header)
    metadata = json.loads(run("--project", PROJECT, "--reflect"))
    base = next(c for c in metadata["classes"] if c["id"] == BASE)
    functions = {f["name"]: f for f in base["functions"]}
    library = next(c for c in metadata["function_libraries"] if c["cpp_name"] == "epok::GameplayInputLibrary")
    operations = {o["name"]: o for o in library["operations"]}
    path = PROJECT / "assets/Blueprints/BP_ControlProbe.epokbp"
    if not path.exists():
        run("--project", PROJECT, "--new-blueprint", "BP_ControlProbe", "--parent", BASE)
    asset = documents.loads(path.read_text(encoding="utf-8"))
    begin = next(g for g in asset["functions"] if g["name"] == "begin_play")
    nodes = []
    def node(kind, inputs=None, **fields):
        value = {"id": str(uuid.uuid4()), "kind": {"kind": kind, **fields}, "inputs": inputs or {}, "outputs": {}}
        nodes.append(value)
        return value
    def operation(name, values):
        op = operations[name]
        return node("operation", {p["id"]: v for p, v in zip(op["parameters"], values)}, operation=op["id"])
    action_type = operations["mask"]["parameters"][0]["value_type"]
    action = lambda value: {"kind": "literal", "value_type": action_type, "value": value}
    zero, false = literal("uint32", 0), literal("bool", False)
    entry = node("entry")
    seed = node("call", function=functions["seed"]["id"])
    self_node = node("builtin", operation={"kind": "self_object"})
    mask = operation("mask", [action(6)])
    capture = operation("capture", [ref(self_node), ref(mask), literal("int32", 100), zero])
    recovered = operation("owner_capture", [ref(self_node), ref(mask), zero])
    press = node("call", function=functions["press"]["id"])
    blocked = operation("sample", [action(4), zero])
    own = operation("context_sample", [ref(recovered), action(6), false])
    observe = node("call", {"blocked": ref(blocked), "own": ref(own)}, function=functions["observe"]["id"])
    taken = operation("context_take_pressed", [ref(recovered), action(6), false])
    repeated = operation("context_take_pressed", [ref(capture), action(6), literal("bool", True)])
    released = operation("release", [ref(capture)])
    valid = operation("valid", [ref(capture)])
    finished = node("call", {"taken": ref(taken), "repeated": ref(repeated),
                             "released": ref(released), "still_valid": ref(valid)}, function=functions["finish"]["id"])
    flow = [entry, seed, capture, press, observe, taken, repeated, released, finished]
    for first, second in zip(flow, flow[1:]):
        first["outputs"]["next"] = [second["id"]]
    begin.update(entry=entry["id"], nodes=nodes)
    asset["functions"] = [begin]
    write(path, asset)
    write(PROJECT / "assets/scripts/LuaControlProbe.lua", '''---@class LuaControlProbe : ControlProbe
local LuaControlProbe = ControlProbe:extend()
---@override
function LuaControlProbe:begin_play()
    self:seed()
    local token = epok.gameplay_input.capture(self.ref, epok.gameplay_input.mask(6), 100, 0)
    token = epok.gameplay_input.owner_capture(self.ref, epok.gameplay_input.mask(6), 0)
    self:press()
    local blocked = epok.gameplay_input.sample(4, 0)
    local own = epok.gameplay_input.context_sample(token, 6, false)
    self:observe(blocked, own)
    local taken = epok.gameplay_input.context_take_pressed(token, 6, false)
    local repeated = epok.gameplay_input.context_take_pressed(token, 6, true)
    local released = epok.gameplay_input.release(token)
    local still_valid = epok.gameplay_input.valid(token)
    self:finish(taken, repeated, released, still_valid)
end
return LuaControlProbe
''')
    write(PROJECT / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/LuaControlProbe.lua": LUA}})
    descriptor = next(PROJECT.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = PROJECT / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    camera = scene["actors"][0]
    scene["actors"] = [camera]
    for index, (name, class_id) in enumerate((("CppControlProbe", CPP), ("BP_ControlProbe", asset["id"]), ("LuaControlProbe", LUA))):
        actor = copy.deepcopy(camera)
        actor.update(id=str(uuid.uuid4()), name=name, **{"class": {"name": name, "class_id": class_id}},
                     properties={"probe_slot": index}, overrides=["probe_slot"])
        actor["components"] = [c for c in actor["components"] if c.get("root")]
        for component in actor["components"]:
            component["id"] = str(uuid.uuid4())
        scene["actors"].append(actor)
    write(scene_path, scene)
    run("--project", PROJECT, "--compile-blueprints")
    return descriptor


def emulate(mode):
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 8077))
    mapping = (PROJECT / ".epok/build/epok.map").read_text()
    address = int(re.findall(r"0x([0-9a-fA-F]+)\s+gameplay_control_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
    expected = [MAGIC, 0, 0, 4096, 1, 1, 0, 1] * 3
    with (OUT / f"{mode}-play.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(EDITOR), "--project", str(PROJECT), "--play-psx", "--stop-after", "6"],
                                   cwd=ROOT, stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        values = None
        try:
            deadline = time.monotonic() + 120
            while process.poll() is None and time.monotonic() < deadline:
                try:
                    with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                        values = list(struct.unpack_from("<24I", response.read(), address))
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
    parser.add_argument("--mode", action="append", choices=("native_cpp", "vm_bytecode", "vm_source"),
                        help="rerun selected modes, retaining the other recorded results")
    parser.add_argument("--standalone-only", action="store_true",
                        help="rebuild the last prepared mode's standalone export")
    args = parser.parse_args()
    if args.standalone_only:
        export = Path(run("--project", PROJECT, "--export-psx").strip())
        env = os.environ.copy()
        env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
        result = subprocess.run([
            "powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
            "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
            cwd=export, env=env, capture_output=True, text=True, timeout=900)
        (OUT / "standalone-build.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        original = (PROJECT / ".epok/build/epok.ps-exe").read_bytes()
        assert (export / "epok.ps-exe").read_bytes() == original
        mode = documents.loads(next(PROJECT.glob("*.epokproject")).read_text())["lua_execution"]
        (OUT / "standalone-result.json").write_text(json.dumps({
            "mode": mode, "export": str(export), "identical": True,
            "sha256": hashlib.sha256(original).hexdigest()}, indent=2) + "\n", encoding="utf-8")
        print("PASS standalone rebuild, byte-identical", mode, flush=True)
        return
    descriptor = prepare()
    if args.prepare_only:
        print("Provider fixture prepared and Blueprint compiled.")
        return
    result_path = OUT / "result.json"
    results = json.loads(result_path.read_text()) if args.mode and result_path.exists() else []
    for mode in args.mode or ("native_cpp", "vm_bytecode", "vm_source"):
        manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
        manifest["lua_execution"] = mode
        write(descriptor, manifest)
        output = run("--project", PROJECT, "--analyze-memory")
        (OUT / f"{mode}-build.log").write_text(output, encoding="utf-8")
        flags = (PROJECT / ".epok/build/sources.mk").read_text()
        assert "-DEPOK_GAMEPLAY_CONTROL" in flags
        values = emulate(mode)
        results = [r for r in results if r["mode"] != mode]
        results.append({"mode": mode, "cpp": values[:8], "blueprint": values[8:16], "lua": values[16:],
                        "memory": json.loads((PROJECT / ".epok/build/memory-report.json").read_text())["ram"]["used"],
                        "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(),
                        "exe_sha256": hashlib.sha256((PROJECT / ".epok/build/epok.ps-exe").read_bytes()).hexdigest(),
                        "runtime_sources": {name: hashlib.sha256((PROJECT / ".epok/build" / name).read_bytes()).hexdigest()
                                            for name in ("epok.hpp", "object_model.hpp", "gameplay_control.hpp")}})
        result_path.write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
        print("PASS", mode, "C++, Blueprint and Lua capture/record round trips", flush=True)


if __name__ == "__main__":
    main()
