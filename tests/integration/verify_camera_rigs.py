"""Exercise each camera preset through derived C++, Blueprint and Lua classes on PSX."""
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
from profile_forest_route import save_png
EDITOR = ROOT / "target/debug/epok-editor.exe"
NAMESPACE = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")
PRESETS = {"first_person": "FirstPersonCamera", "third_person": "ThirdPersonCamera",
           "top_down": "TopDownCamera", "isometric": "IsometricCamera"}


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    documents.write_text(path, documents.dumps(value) if isinstance(value, dict) else value)


def verify(out, rig, modes, prepare_only, standalone_only):
    out = out / rig
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    identity = lambda name: str(uuid.uuid5(NAMESPACE, f"camera-acceptance:{rig}:{name}"))
    counter = 0

    def run(*command, label=None):
        nonlocal counter
        counter += 1
        result = subprocess.run([str(EDITOR), *map(str, command)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / (label or f"command-{counter}.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if standalone_only:
        export = Path(run("--project", project, "--export-psx", label="standalone-export.log"))
        assert (export / "camera_rig.hpp").is_file()
        env = os.environ.copy()
        env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
        result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
                                 "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
                                cwd=export, env=env, capture_output=True, text=True, timeout=900)
        (out / "standalone-build.log").write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        original = (project / ".epok/build/epok.ps-exe").read_bytes()
        assert (export / "epok.ps-exe").read_bytes() == original
        (out / "standalone-result.json").write_text(json.dumps({"rig": rig, "identical": True, "export": str(export),
            "sha256": hashlib.sha256(original).hexdigest(),
            "camera_sdk_sha256": hashlib.sha256((export / "camera_rig.hpp").read_bytes()).hexdigest()}, indent=2) + "\n", encoding="utf-8")
        print("PASS", rig, "standalone rebuild, byte-identical", flush=True)
        return
    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Camera " + rig)
    run_magic = 0x43000000 | (uuid.uuid4().int & 0xffffff)
    source = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t camera_rig_probe[24]={};}
struct EPOK_FUNCTION_LIBRARY(Category="Camera Probe",Id="LIBRARY") CameraProbeLibrary {
 EPOK_FUNCTION(BlueprintCallable) static void observe(epok::CameraRigSample view,uint32_t operation,uint32_t slot){
  if(slot>=3)return;auto* p=camera_rig_probe+slot*8;if(p[2]+p[3]==0)p[4]=2166136261u;
  ++p[operation==1?2:3];
  const uint32_t words[]={operation,uint32_t(view.status),uint32_t(view.position.x.raw()),uint32_t(view.position.y.raw()),uint32_t(view.position.z.raw()),uint32_t(view.yaw.raw()),uint32_t(view.pitch.raw()),uint32_t(view.roll.raw()),uint32_t(view.distance.raw()),view.blocked};
  for(auto word:words)p[4]=(p[4]^word)*16777619u;
 }
};
class EPOK_CLASS(Blueprintable,Id="BASE") CameraProbe:public epok::PRESET {
public:EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
};
class EPOK_CLASS(Blueprintable,Id="CPP") CppCameraProbe:public CameraProbe {
public:
 epok::CameraRigSample calculate_view(epok::Fixed delta_seconds)override{auto view=CameraProbe::calculate_view(delta_seconds);CameraProbeLibrary::observe(view,1,probe_slot);return view;}
 void view_updated(epok::CameraRigSample view)override{CameraProbe::view_updated(view);CameraProbeLibrary::observe(view,2,probe_slot);}
};
class EPOK_CLASS(Blueprintable,Domain=World3D,Owners=World3D,Id="DRIVER") CameraProbeDriver:public epok::ActorComponent {
 bool done_=false;
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 void on_frame(uint32_t)override{
  if(done_||probe_slot>=3)return;done_=true;auto* camera=epok::active_object_registry->resolve<epok::CameraRig3D>(owner_id());if(!camera)return;
  auto* p=camera_rig_probe+probe_slot*8;auto check=[&](bool value,unsigned bit){p[7]|=1u<<bit;if(!value)p[1]|=1u<<bit;};
  const epok::Fixed dt(1.0/60.0);const auto target_id=camera->target;auto* target=epok::active_object_registry->resolve<epok::Actor3D>(target_id);
  auto* level=epok::active_object_registry->resolve<epok::Level>(camera->level_id());auto* input=level?level->get_component<epok::CameraOrbitInputComponent>(*camera):nullptr;
  if(!target||!target->data()||!input)return;input->enabled=true;
  const auto first=camera->snap();check(first.status==epok::CameraRigStatus::Applied,0);
  check(camera->data()->transform.position[0]==first.position.x&&camera->data()->transform.position[1]==first.position.y&&camera->data()->transform.position[2]==first.position.z,1);
  check(camera->make_active()&&epok::active_camera().get()==camera->data(),2);p[5]=epok::active_camera().get()==camera->data();
  check(camera->set_angles(370.0,42.0)&&camera->yaw==10.0&&camera->pitch==epok::MathLibrary::clamp(epok::Fixed(42.0),camera->pitch_min,camera->pitch_max),3);
  check(camera->add_look({-25.0,400.0})&&camera->yaw==345.0&&camera->pitch==camera->pitch_max,4);
  check(camera->refresh(dt).status==epok::CameraRigStatus::Applied&&camera->data()->transform.rotation[1]==345.0,5);
  camera->set_angles(0.0,first.pitch);
  epok::input.reset();epok::gameplay_controls().reset();epok::input.sample(0,true,0,true,128,128,255,128);epok::gameplay_controls().begin_frame();epok::input.begin_tick();epok::gameplay_controls().begin_tick();
  check(input->update(dt)==epok::CameraRigStatus::Applied&&camera->yaw>0.0,6);
  const auto yaw=camera->yaw;auto capture=epok::GameplayInputLibrary::capture(target_id,epok::GameplayInputLibrary::look_actions(),100,0);
  check(input->update(dt)==epok::CameraRigStatus::Applied&&camera->yaw==yaw,7);epok::GameplayInputLibrary::release(capture);
  capture=epok::GameplayInputLibrary::capture(target_id,epok::GameplayInputLibrary::movement_actions(),100,0);input->update(dt);
  check(camera->yaw>yaw,8);epok::GameplayInputLibrary::release(capture);input->enabled=false;epok::input.reset();epok::gameplay_controls().reset();
  const auto before=camera->refresh(dt);target->data()->transform.position[0]=4.0;camera->follow_response=6.0;
  const auto smooth=camera->refresh(dt);check(smooth.status==epok::CameraRigStatus::Applied&&smooth.position.x>before.position.x&&smooth.position.x<before.position.x+epok::Fixed(4.0),9);
  const auto snapped=camera->snap();check(snapped.status==epok::CameraRigStatus::Applied&&snapped.position.x==before.position.x+epok::Fixed(4.0),10);
  check(camera->refresh(0.0).status==epok::CameraRigStatus::NoStep&&camera->data()->transform.position[0]==snapped.position.x,11);
  const auto distance=camera->distance;camera->distance=-1.0;check(camera->refresh(dt).status==epok::CameraRigStatus::InvalidConfiguration&&camera->data()->transform.position[0]==snapped.position.x,12);camera->distance=distance;
  level->destroy_actor(target_id);const auto missing=camera->refresh(dt);p[6]=uint32_t(missing.status);
  check(missing.status==epok::CameraRigStatus::InvalidTarget&&camera->data()->transform.position[0]==snapped.position.x,13);
  p[0]=RUN_MAGIC+EPOK_LUA_MODE;
 }
};
'''
    for key in ("library", "base", "cpp", "driver"):
        source = source.replace('"' + key.upper() + '"', '"' + identity(key) + '"')
    source = source.replace("PRESET", PRESETS[rig]).replace("RUN_MAGIC", str(run_magic) + "u")
    write(project / "assets/scripts/CameraProbe.hpp", source)
    metadata = json.loads(run("--project", project, "--reflect"))
    classes = {c["cpp_name"]: c for c in metadata["classes"]}
    functions = {f["name"]: f for f in classes["epok::CameraRig3D"]["functions"]}
    slot = classes["CameraProbe"]["properties"][0]["id"]
    operation = next(o for library in metadata["function_libraries"] if library["cpp_name"] == "CameraProbeLibrary" for o in library["operations"] if o["name"] == "observe")
    bp_path = project / "assets/Blueprints/BP_CameraProbe.epokbp"
    if not bp_path.exists():
        run("--project", project, "--new-blueprint", "BP_CameraProbe", "--parent", identity("base"))
    asset = documents.loads(bp_path.read_text(encoding="utf-8"))
    asset["functions"] = []
    for number, name in ((1, "calculate_view"), (2, "view_updated")):
        function = functions[name]
        nodes = []

        def node(kind, inputs=None, **fields):
            value = {"id": str(uuid.uuid4()), "kind": {"kind": kind, **fields}, "inputs": inputs or {}, "outputs": {}}
            nodes.append(value)
            return value

        link = lambda value: {"kind": "link", "node": value["id"], "pin": "value"}
        params = {p["name"]: {"kind": "parameter", "name": p["name"]} for p in function["parameters"]}
        entry, parent = node("entry"), node("call_parent", params)
        get_slot = node("get_variable", member=slot)
        arguments = [link(parent) if number == 1 else params["view"],
                     {"kind": "literal", "value_type": {"kind": "uint32"}, "value": number}, link(get_slot)]
        observed = node("operation", {p["id"]: v for p, v in zip(operation["parameters"], arguments)}, operation=operation["id"])
        flow = [entry, parent, observed]
        if number == 1:
            flow.append(node("return", {"value": link(parent)}))
        for first, second in zip(flow, flow[1:]):
            first["outputs"]["next"] = [second["id"]]
        asset["functions"].append({"id": str(uuid.uuid4()), "name": name, "override_id": function["id"],
            "parameters": function["parameters"], "returns": function["returns"], "entry": entry["id"], "nodes": nodes})
    write(bp_path, asset)
    # The category spelling is resolved by the same library catalog used by Lua.
    write(project / "assets/scripts/LuaCameraProbe.lua", '''---@class LuaCameraProbe : CameraProbe
local LuaCameraProbe = CameraProbe:extend()
---@override
function LuaCameraProbe:calculate_view(delta_seconds)
    local view = LuaCameraProbe.super.calculate_view(self, delta_seconds)
    epok.camera_probe.observe(view, 1, self.probe_slot)
    return view
end
---@override
function LuaCameraProbe:view_updated(view)
    LuaCameraProbe.super.view_updated(self, view)
    epok.camera_probe.observe(view, 2, self.probe_slot)
end
return LuaCameraProbe
''')
    write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/LuaCameraProbe.lua": identity("lua")}})
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    original = copy.deepcopy(scene["actors"][0])
    scene["actors"] = [original]

    def component(name, class_id, props):
        return {"id": str(uuid.uuid4()), "name": name, "class": {"name": name, "class_id": class_id},
                "properties": props, "overrides": list(props)}

    def actor(name, position, class_name="epok::Actor3D", class_id=None, props=None):
        value = copy.deepcopy(original)
        value.update(id=str(uuid.uuid4()), name=name, **{"class": {"name": class_name, "class_id": class_id or classes[class_name]["id"]}},
                     properties=props or {}, overrides=list(props or {}))
        value["components"] = [c for c in value["components"] if c.get("root")]
        value["components"][0]["id"] = str(uuid.uuid4())
        value["components"][0]["properties"].update(position=position, rotation=[0, 0, 0])
        return value

    wall = actor("Obstruction", [0, 2, -3])
    wall["components"].append(component("Collider", classes["epok::Collider3DComponent"]["id"], {"collider": {
        "enabled": True, "trigger": False, "center": [0, 0, 0], "half_extents": [100, 3, .1], "layer": 1, "mask": 4294967295}}))
    scene["actors"].append(wall)
    for name, position, scale, color in [
        ("Ground", [4, -.1, 0], [20, .1, 20], [.2, .3, .25]),
        ("Red marker", [2, 1, 4], [1, 2, 1], [.9, .2, .15]),
        ("Yellow marker", [4, 1, 4], [1, 2, 1], [.95, .8, .15]),
        ("Blue marker", [6, 1, 4], [1, 2, 1], [.15, .4, .9]),
    ]:
        marker = actor(name, position)
        marker["components"][0]["properties"]["scale"] = scale
        marker["components"].append(component("Mesh", classes["epok::Mesh3DComponent"]["id"],
                                               {"material": {"color": color, "unlit": True}}))
        scene["actors"].append(marker)
    providers = (("CppCameraProbe", identity("cpp")), ("BP_CameraProbe", asset["id"]), ("LuaCameraProbe", identity("lua")))
    targets = []
    for index in range(3):
        target = actor("Same target name", [0, 1, 0])
        targets.append(target["id"])
        scene["actors"].append(target)
    write(scene_path, scene)
    run("--project", project, "--compile-blueprints")
    for index, (name, class_id) in enumerate(providers):
        run("--project", project, "--add-actor", class_id + ":Same camera name")
        scene = documents.loads(scene_path.read_text(encoding="utf-8"))
        camera = scene["actors"][-1]
        assert camera["class"]["class_id"] == class_id
        assert len(camera["components"]) == (3 if rig == "third_person" else 2)
        camera["properties"].update(target=targets[index], probe_slot=index, automatic_tick=False)
        camera["overrides"] = list(camera["properties"])
        camera["components"].append(component("Input", classes["epok::CameraOrbitInputComponent"]["id"], {"enabled": False}))
        camera["components"].append(component("Driver", identity("driver"), {"probe_slot": index}))
        write(scene_path, scene)
    if prepare_only:
        print("PASS", rig, "three derived providers compiled", flush=True)
        return

    def emulate(mode):
        flags = (project / ".epok/build/sources.mk").read_text()
        execution = int(re.search(r"-DEPOK_LUA_MODE=(\d+)", flags)[1])
        assert "-DEPOK_GAMEPLAY_CONTROL" in flags
        magic = run_magic + execution
        with socket.socket() as probe:
            if os.name == "nt":
                probe.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
            probe.bind(("127.0.0.1", 8077))
        address = int(re.findall(r"0x([0-9a-fA-F]+)\s+camera_rig_probe\s*$", (project / ".epok/build/epok.map").read_text(), re.M)[-1], 16) & 0x1fffff
        with (out / f"{mode}-play.log").open("w", encoding="utf-8") as log:
            process = subprocess.Popen([str(EDITOR), "--project", str(project), "--play-psx", "--stop-after", "6"], cwd=ROOT,
                stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
            values = None
            try:
                deadline = time.monotonic() + 180
                while process.poll() is None and time.monotonic() < deadline:
                    try:
                        with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                            values = list(struct.unpack_from("<24I", response.read(), address))
                        if all(values[i * 8] == magic for i in range(3)):
                            break
                    except (OSError, ValueError):
                        pass
                    time.sleep(.1)
                assert values is not None and values[:8] == values[8:16] == values[16:], (rig, mode, values)
                assert values[0] == magic and values[1] == 0 and values[2] > values[3] > 0, (rig, mode, values)
                assert values[5:8] == [1, 3, 16383], (rig, mode, values)
                time.sleep(.25)
                with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/gpu/vram/raw", timeout=5) as response:
                    pixels = response.read()
                (out / f"{mode}-vram.bin").write_bytes(pixels)
                rendering = manifest.get("rendering", {})
                save_png(pixels, out / f"{mode}-view.png", rendering.get("width", 640), rendering.get("height", 480))
                assert process.wait(timeout=30) == 0
            finally:
                if process.poll() is None:
                    if os.name == "nt":
                        subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"], capture_output=True, timeout=15)
                    else:
                        process.terminate()
                    process.wait(timeout=15)
        return values

    result_path = out / "result.json"
    results = json.loads(result_path.read_text()) if modes and result_path.exists() else []
    for mode in modes or ("native_cpp", "vm_bytecode", "vm_source"):
        manifest["lua_execution"] = mode
        write(descriptor, manifest)
        run("--project", project, "--analyze-memory", label=f"{mode}-build.log")
        values = emulate(mode)
        row = {"rig": rig, "mode": mode, "run_magic": run_magic, "checks_per_provider": 14,
            "cpp": values[:8], "blueprint": values[8:16], "lua": values[16:],
            "ram_static_bytes": json.loads((project / ".epok/build/memory-report.json").read_text())["ram"]["used"],
            "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(),
            "exe_sha256": hashlib.sha256((project / ".epok/build/epok.ps-exe").read_bytes()).hexdigest(),
            "framebuffer": f"{mode}-view.png",
            "camera_sdk_sha256": hashlib.sha256((project / ".epok/build/camera_rig.hpp").read_bytes()).hexdigest()}
        results = [r for r in results if r["mode"] != mode] + [row]
        result_path.write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")
        print("PASS", rig, mode, "three derived providers, full view callbacks and 14 camera checks", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/camera-providers")
    parser.add_argument("--rig", choices=tuple(PRESETS), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    for rig in args.rig or PRESETS:
        verify(args.output.resolve(), rig, args.mode, args.prepare_only, args.standalone_only)


if __name__ == "__main__":
    main()
