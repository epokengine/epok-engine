"""P3 cooked native dialogue acceptance on PSX; reflected provider helpers follow."""
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
OUT = ROOT / "artifacts/gameplay-helpers/p3/cooked-native"
PROJECT = OUT / "project"
MAGIC = 0x444c4733
NS = uuid.UUID("18263f9d-d972-530a-91ac-739c4ed39dc4")


def identity(name):
    return str(uuid.uuid5(NS, "fixture.dialogue." + name))


def compact(name):
    return int.from_bytes(hashlib.sha256(uuid.UUID(identity(name)).bytes).digest()[:4], "little")


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    documents.write_text(path, documents.dumps(value) if isinstance(value, dict) else value)


def run(*args, log=None):
    result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT,
                            capture_output=True, text=True, encoding="utf-8", timeout=900)
    output = result.stdout + result.stderr
    if log:
        (OUT / log).write_text(output, encoding="utf-8")
    assert result.returncode == 0, output
    return result.stdout


def package(path, kind, name, value, json_source=False):
    source = (json.dumps(value, ensure_ascii=False) if json_source else documents.dumps(value)).encode()
    meta = {"version": 2, "id": identity(name), "kind": kind, "importer_version": 1,
            "source": path, "source_hash": hashlib.sha256(source).hexdigest(), "settings": {"type": "Authored"}}
    encoded = json.dumps(meta).encode()
    destination = PROJECT / path
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(b"EPOKAS01" + struct.pack("<II", len(encoded), len(source)) + encoded + source)


def prepare():
    OUT.mkdir(parents=True, exist_ok=True)
    if not PROJECT.exists():
        run("--create-project", PROJECT, "--template", "basic", "--name", "Cooked Dialogue")
    driver = r'''#pragma once
#include "epok.hpp"
extern "C" {inline volatile uint32_t dialogue_probe[16]={};}
class EPOK_CLASS(Blueprintable,Id="DRIVER") DialogueCookProbe:public epok::DialoguePresenter {
 bool done=false,started=false;
 static void entered(epok::ObjectId,uint32_t){dialogue_probe[3]=dialogue_probe[3]+1;}
public:
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId initiator{};
 EPOK_PROPERTY(EditAnywhere,AssetKind=Dialogue) uint64_t conversation=0;
 EPOK_PROPERTY(EditAnywhere,AssetKind=DialogueStyle) uint64_t style=0;
 EPOK_FUNCTION(BlueprintEvent,BlueprintPure,Id="CONDITION") virtual bool allowed(epok::ObjectId source) const{return source.valid();}
 EPOK_FUNCTION(BlueprintEvent,Id="ACTION") virtual void award(epok::ObjectId source,uint32_t port){(void)source;(void)port;dialogue_probe[4]=dialogue_probe[4]+1;}
 void on_frame(uint32_t) override {
   if(done||epok::scene_loading())return;
   namespace d=epok::dialogue;auto& session=d::session();
   if(!started){
     started=true;dialogue_probe[0]=0x444c4733;
     const auto* resource=d::presenter_resource(id(),conversation,style);
     if(!resource){dialogue_probe[1]=999;done=true;return;}
     const d::Assignment roles[]={{ROLE,id()}};
     d::Start options;options.owner=id();options.initiator=initiator;options.presenter=id();options.roles=roles;options.role_count=1;options.entered=&entered;options.timing=d::presenter_timing(id(),*resource);
     dialogue_probe[1]=uint32_t(session.start(*resource->table,options));
     dialogue_probe[5]=session.snapshot().pages;
     dialogue_probe[12]=epok::localization.languages;
     dialogue_probe[13]=session.snapshot().generation;
     dialogue_probe[14]=sizeof(epok::Text);dialogue_probe[15]=sizeof(d::Session);
     return;
   }
   const auto state=session.snapshot();
   if(state.phase==d::Phase::Selecting){
     dialogue_probe[6]=state.choices;session.move_selection(4);
     dialogue_probe[7]=session.snapshot().choice_page;
     dialogue_probe[9]=session.choice_id(0)==OPTION;
     session.advance();
   } else if(state.phase==d::Phase::Waiting||state.phase==d::Phase::Revealing)session.advance();
   if(!session.active()){
     dialogue_probe[2]=1;dialogue_probe[10]=uint32_t(session.snapshot().phase);dialogue_probe[11]=uint32_t(session.snapshot().result);
     dialogue_probe[8]=!epok::GameplayInputLibrary::valid(epok::GameplayInputLibrary::owner_capture(id(),epok::GameplayInputLibrary::all_actions(),0));
     done=true;
   }
   auto* slot=data();slot->text.enabled=session.active();slot->text.set_text(session.text());slot->text.visible_glyphs=uint16_t(session.snapshot().revealed);
 }
};
'''
    for key, value in {"DRIVER": identity("driver"), "CONDITION": identity("condition-member"),
                       "ACTION": identity("action-member"), "ROLE": str(compact("role")),
                       "OPTION": str(compact("option4"))}.items():
        driver = driver.replace(key, value)
    write(PROJECT / "assets/scripts/DialogueCookProbe.hpp", driver)
    entries = [{"key": "line", "values": ["[color=#ff8800]" + "ÁBC long text " * 100 + "[/color]", "Texto traducido " * 100]}]
    entries += [{"key": f"option{i}", "values": [f"Option {i}", f"Opción {i}"]} for i in range(6)]
    package("assets/dialogue/strings.epokasset", "StringTable", "strings",
            {"version": 1, "pageable": True, "languages": ["English", "Español"], "entries": entries}, True)
    text = lambda key: {"table": identity("strings"), "key": key}
    nodes = [
        {"id": identity("line"), "kind": {"kind": "line", "speaker": None, "text": text("line"), "next": identity("choice"), "policy": None}},
        {"id": identity("choice"), "kind": {"kind": "choice", "speaker": None, "prompt": None,
         "options": [{"id": identity(f"option{i}"), "text": text(f"option{i}"), "next": identity("branch"), "condition": None} for i in range(6)]}},
        {"id": identity("branch"), "kind": {"kind": "branch", "condition": identity("condition-binding"), "yes": identity("event"), "no": identity("end")}},
        {"id": identity("event"), "kind": {"kind": "event", "action": identity("action-binding"), "next": identity("end")}},
        {"id": identity("end"), "kind": {"kind": "end"}},
    ]
    package("assets/dialogue/conversation.epokasset", "Dialogue", "conversation",
            {"version": 1, "entry": identity("line"), "nodes": nodes,
             "participants": [{"id": identity("role"), "label": "Rules", "display_name": None, "portrait": None}],
             "bindings": [{"id": identity(f"{kind}-binding"), "label": kind, "participant": identity("role"), "kind": kind,
                           "class": identity("driver"), "member": identity(f"{kind}-member")} for kind in ("condition", "action")]})
    package("assets/dialogue/style.epokasset", "DialogueStyle", "style",
            {"version": 2, "line_policy": {"milliseconds_per_glyph": 0, "auto_advance_ms": None},
             "open_sound": None, "advance_sound": None, "close_sound": None})
    descriptor = next(PROJECT.glob("*.epokproject"))
    settings = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = PROJECT / settings["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    camera = copy.deepcopy(scene["actors"][0])
    def component(name, class_id, props=None, root=False):
        props = props or {}
        return {"id": identity(name), "name": name, "class": {"name": name, "class_id": class_id},
                "properties": props, "overrides": list(props), "root": root}
    player = copy.deepcopy(camera)
    player.update(id=identity("player"), name="Initiator", components=[component("epok::SceneComponent3D", "ed73d249-b6cb-4a3c-a0e8-696de55e286f", root=True)])
    props = {"initiator": identity("player"), "conversation": identity("conversation"), "style": identity("style"),
             "body": identity("epok::TextComponent"), **{f"choice{i}": identity(f"Choice{i}Text") for i in range(4)}}
    def rect(size):
        return {"rect": {"anchor_min": [0,0], "anchor_max": [0,0], "pivot": [0,0], "position": [0,0], "size": size, "rotation": 0}}
    label = {"id": identity("presenter"), "name": "Presenter", "class": {"name": "DialogueCookProbe", "class_id": identity("driver")},
             "properties": props, "overrides": list(props), "active": True, "components": [
                 component("epok::RectTransformComponent", "dc805165-6c65-48dc-8ff8-4a638a5d21df", rect([272,72]), True),
                 component("epok::CanvasComponent", "f2cfb26b-af53-4a46-9d91-1debea72e01b"),
                 component("epok::AudioComponent", "7f0eb028-5301-4ac7-b93b-5665fab12b20"),
                 component("epok::TextComponent", "fd7f11d1-7ccf-40e8-a7ea-56d89deb3f34")]}
    for inherited, default_id in zip(label["components"][1:3], ("0762122d-cb6b-52f6-8fb1-3287494801fc", "b0e900ca-43e6-5bdf-bdbd-a52f0416b36b")):
        inherited.update(inherited=True, default_id=default_id)
    choices = [{"id": identity(f"Choice{i}"), "name": f"Choice{i}",
                "class": {"name": "epok::UIActor", "class_id": "b09bd2fa-8b09-4c0f-a33a-c3ca08b21d8f"},
                "properties": {}, "overrides": [], "active": True, "logical_parent": identity("presenter"),
                "attach": {"actor": identity("presenter"), "component": identity("epok::RectTransformComponent")}, "components": [
                    component(f"Choice{i}Root", "dc805165-6c65-48dc-8ff8-4a638a5d21df", rect([248,16]), True),
                    component(f"Choice{i}Text", "fd7f11d1-7ccf-40e8-a7ea-56d89deb3f34")]} for i in range(4)]
    scene["actors"] = [camera, player, label, *choices]
    scene.pop("bake", None)
    write(scene_path, scene)


def emulate():
    with socket.socket() as probe:
        if os.name == "nt":
            probe.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
        probe.bind(("127.0.0.1", 8077))
    mapping = (PROJECT / ".epok/build/epok.map").read_text()
    address = int(re.findall(r"0x([0-9a-fA-F]+)\s+dialogue_probe\s*$", mapping, re.M)[-1], 16) & 0x1fffff
    with (OUT / "play.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(EDITOR), "--project", str(PROJECT), "--play-psx", "--stop-after", "6"], cwd=ROOT,
                                   stdout=log, stderr=log, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        values = None
        try:
            deadline = time.monotonic() + 120
            while process.poll() is None and time.monotonic() < deadline:
                try:
                    with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                        values = list(struct.unpack_from("<16I", response.read(), address))
                    if values[0] == MAGIC and values[2] == 1:
                        break
                except (OSError, ValueError):
                    pass
                time.sleep(.1)
            assert values and values[:5] == [MAGIC,0,1,5,1] and values[5] > 1, values
            assert values[6:15] == [6,1,1,1,5,8,2,1,528], values
            assert process.wait(timeout=30) == 0
        finally:
            if process.poll() is None:
                if os.name == "nt":
                    subprocess.run(["taskkill.exe", "/PID", str(process.pid), "/T", "/F"], capture_output=True, check=False)
                else:
                    process.terminate()
                process.wait(timeout=15)
    return values


def main():
    global OUT, PROJECT
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="Evidence directory; keeps earlier phase results intact")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone", action="store_true")
    args = parser.parse_args()
    if args.output:
        OUT = args.output.resolve()
        PROJECT = OUT / "project"
    prepare()
    if args.prepare_only:
        print("PASS cooked dialogue fixture prepared")
        return
    run("--project", PROJECT, "--analyze-memory", log="build.log")
    values = emulate()
    result = {"phase": "P3", "scope": "native cooked asset execution; provider helpers remain pending", "probe": values,
              "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(),
              "exe_sha256": hashlib.sha256((PROJECT / ".epok/build/epok.ps-exe").read_bytes()).hexdigest(),
              "memory": json.loads((PROJECT / ".epok/build/memory-report.json").read_text())["ram"]["used"]}
    if args.standalone:
        export = Path(run("--project", PROJECT, "--export-psx", log="export.log").strip())
        env = os.environ.copy()
        env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
        rebuilt = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
                                  "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
                                 cwd=export, env=env, capture_output=True, text=True, timeout=900)
        (OUT / "standalone.log").write_text(rebuilt.stdout + rebuilt.stderr, encoding="utf-8")
        assert rebuilt.returncode == 0, rebuilt.stdout + rebuilt.stderr
        assert (export / "epok.ps-exe").read_bytes() == (PROJECT / ".epok/build/epok.ps-exe").read_bytes()
        result["standalone_identical"] = True
    result["runtime_sources"] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in (PROJECT / ".epok/build").glob("*.hpp")}
    (OUT / "result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print("PASS PSX cooked dialogue, long pages, six choices, typed callbacks and cleanup", flush=True)


if __name__ == "__main__":
    main()
