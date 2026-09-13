"""Production-CLI acceptance for timelines, template factories and typed resources.

This never builds the editor or starts a concurrent emulator. Invoke after a
locked host build; --emulator owns exactly one bounded PCSX-Redux run. The size
report measures linked binaries, not FPS or an equivalent-C++ performance claim.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import argparse
import contextlib
import copy
import hashlib
import json
from pathlib import Path
import re
import socket
import struct
import subprocess
import tempfile
import time
import urllib.request
import wave
import zlib

from verify_blueprints import EDITOR, ROOT, identity, literal, node, run, write


def compact(value):
    return int.from_bytes(hashlib.sha256(value.encode()).digest()[:8], "little")


def wire(*nodes):
    for previous, following in zip(nodes, nodes[1:]):
        previous["outputs"]["next"] = [following["id"]]


def reference(node):
    return {"kind": "link", "node": node["id"], "pin": "value"}


def binding(name, class_id, provider="cpp"):
    return {"name": name, "class_id": class_id, "provider": {"id": provider, "version": 1},
            "backend": {"id": "native", "version": 1}, "properties": {}}


def imported_resources(project):
    source = project / "assets/source"
    source.mkdir(parents=True, exist_ok=True)
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">2I5B", 16, 16, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress((b"\0" + b"\xff\x10\x10\xff" * 16) * 16))
    png += chunk(b"IEND", b"")
    (source / "Color.png").write_bytes(png)
    with wave.open(str(source / "Silence.wav"), "wb") as audio:
        audio.setnchannels(1)
        audio.setsampwidth(2)
        audio.setframerate(22050)
        audio.writeframes(b"\0\0" * 256)
    texture = run("--project", project, "--import-texture", "assets/source/Color.png",
                  "--asset", "assets/Textures/Color.epokasset")
    sound = run("--project", project, "--import-audio", "assets/source/Silence.wav",
                "--asset", "assets/Audio/Silence.epokasset")
    pattern = r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}"
    return re.findall(pattern, texture)[-1], re.findall(pattern, sound)[-1]


def native_sources(project, ids, visual=0):
    header = r'''#pragma once
#include "epok.hpp"
namespace epok::bp {
epok::EntityHandle spawn(uint64_t,const char*,epok::Entity*);
bool is_a(epok::EntityHandle,uint64_t);
epok::Behaviour* behaviour(epok::EntityHandle);
}
extern "C" { inline int32_t blueprint_feature_probe[32]={}; }
class EPOK_CLASS(Blueprintable,Id="ROOT_ID") TrackRoot : public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere,Id="PROGRESS_ID") epok::Fixed progress=0.0;
    EPOK_PROPERTY(EditAnywhere,Id="CHILD_ID") epok::EntityHandle child{};
    bool stopped=false;
    EPOK_FUNCTION(BlueprintEvent) virtual void on_ready() {}
    EPOK_FUNCTION(BlueprintPure) bool should_stop() const {return stopped;}
    EPOK_FUNCTION(BlueprintCallable) void record_link(epok::EntityHandle child) {
        auto* item=child.get();
        if(item&&epok::bp::behaviour(child)&&epok::bp::is_a(child,LEAF_HASH)) {
            ++blueprint_feature_probe[3];
            if(blueprint_feature_probe[2]==blueprint_feature_probe[1]-1)++blueprint_feature_probe[5];
            if(item->transform.position[0].raw()==12288)++blueprint_feature_probe[16];
        }else ++blueprint_feature_probe[19];
    }
    EPOK_FUNCTION(BlueprintCallable) void record_assets() {
        if(entity().material.texture>=0&&entity().audio.enabled&&entity().audio.clip>=0)++blueprint_feature_probe[20];
        else ++blueprint_feature_probe[19];
    }
    EPOK_FUNCTION(BlueprintCallable) void record_update() {++blueprint_feature_probe[6];}
    EPOK_FUNCTION(BlueprintCallable) void record_finish() {
        ++blueprint_feature_probe[7];
        if(stopped)++blueprint_feature_probe[9];
        if(progress.raw()!=40960)++blueprint_feature_probe[19];
    }
    EPOK_FUNCTION(BlueprintCallable) void record_branch() {++blueprint_feature_probe[10];}
    EPOK_FUNCTION(BlueprintCallable) void record_remote(int32_t value,int32_t duplicate,int32_t pure,int32_t invalid) {
        if(value==7&&duplicate==7&&pure==7&&invalid==0)++blueprint_feature_probe[25];
        else ++blueprint_feature_probe[19];
    }
    EPOK_FUNCTION(BlueprintCallable) void record_vector(epok::EntityHandle spawned,epok::EntityHandle invalid,epok::Fixed x,epok::Fixed second) {
        auto* item=spawned.get();
        if(item&&!invalid.get()&&item->transform.position[0].raw()==8192&&item->transform.position[1].raw()==12288&&
           item->transform.position[2].raw()==16384&&x.raw()==8192&&second.raw()==28672)++blueprint_feature_probe[28];
        else ++blueprint_feature_probe[19];
    }
    void start(epok::Transform&) override {
        ++blueprint_feature_probe[1];
        stopped=entity().name[0]=='R'&&entity().name[4]=='0';
        if(progress.raw()!=0)++blueprint_feature_probe[19];
        on_ready();
    }
    void update(epok::Transform&,epok::Fixed) override {}
    void on_destroy() override {++blueprint_feature_probe[11];}
};
class EPOK_CLASS(Blueprintable,Id="LEAF_ID") TemplateLeaf : public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere,Id="LEAF_VALUE_ID") epok::Fixed preview_gain=0.0;
    int32_t hits=0;
    EPOK_FUNCTION(BlueprintCallable) int32_t hit(int32_t amount,int32_t answer) {
        ++blueprint_feature_probe[24];hits+=amount;
        if(answer!=77)++blueprint_feature_probe[19];
        return hits;
    }
    EPOK_FUNCTION(BlueprintPure) int32_t read_count() const {return hits;}
    void start(epok::Transform&) override {++blueprint_feature_probe[2];}
    void update(epok::Transform&,epok::Fixed) override {}
    void on_destroy() override {++blueprint_feature_probe[22];}
};
class EPOK_CLASS(Blueprintable,Id="DRIVER_ID") FeatureDriver : public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere,Id="DRIVER_VALUE_ID") epok::Fixed preview_gain=0.0;
    epok::EntityHandle roots[5];int ticks=0;
    void start(epok::Transform&) override {
        blueprint_feature_probe[0]=0x42504632;
        blueprint_feature_probe[15]=epok::bp::is_a(epok::handle(&entity()),DRIVER_HASH)?1:0;
        const char* names[]={"Root0","Root1","Root2","Root3"};
        for(int i=0;i<4;++i){roots[i]=epok::bp::spawn(VISUAL_HASH,names[i],nullptr);if(!roots[i].get())++blueprint_feature_probe[19];}
        blueprint_feature_probe[13]=!epok::bp::spawn(VISUAL_HASH,"Overflow",nullptr).get();
        blueprint_feature_probe[14]=epok::bp::is_a(roots[0],ROOT_HASH)&&epok::bp::is_a(roots[0],VISUAL_HASH);
        blueprint_feature_probe[17]=!epok::bp::is_a(roots[0],LEAF_HASH);
    }
    void update(epok::Transform&,epok::Fixed) override {
        ++ticks;
        if(ticks==1&&roots[1].get())epok::destroy_entity(roots[1].get());
        if(ticks==20){
            if(auto* value=epok::bp::behaviour(roots[0]))blueprint_feature_probe[8]=static_cast<TrackRoot*>(value)->progress.raw();
            for(int i=0;i<4;++i)if(roots[i].get())epok::destroy_entity(roots[i].get());
            int invalid=0;for(int i=0;i<4;++i)if(!roots[i].get())++invalid;
            blueprint_feature_probe[12]=invalid;
        }
        if(ticks==21){
            roots[4]=epok::bp::spawn(VISUAL_HASH,"Root4",nullptr);
            if(!roots[4].get())++blueprint_feature_probe[19];
            blueprint_feature_probe[18]=!roots[0].get()&&!roots[1].get();
        }
        if(ticks==40){
            if(auto* value=epok::bp::behaviour(roots[4]))blueprint_feature_probe[23]=static_cast<TrackRoot*>(value)->progress.raw();
            blueprint_feature_probe[31]=1;
        }
    }
};
class EPOK_CLASS(Blueprintable,Id="SPAWN_ID") SpawnProbe : public epok::Behaviour {
public:
    void start(epok::Transform&) override {++blueprint_feature_probe[26];}
    void update(epok::Transform&,epok::Fixed) override {}
    void on_destroy() override {++blueprint_feature_probe[27];}
};
'''
    for name in ["root", "leaf", "driver", "progress", "child", "spawn", "leaf_value", "driver_value"]:
        header = header.replace(name.upper() + "_ID", ids[name])
    for name in ["root", "leaf", "driver"]:
        header = header.replace(name.upper() + "_HASH", str(compact(ids[name])) + "ULL")
    header = header.replace("VISUAL_HASH", str(visual) + "ULL")
    write(project / "assets/scripts/Features.hpp", header)
    # Include native runtime helper definitions only in the translation unit:
    # reflection still consumes real declarations and target-layout headers.
    write(project / "assets/scripts/Features.cpp", '#include "Features.hpp"\n#ifdef EPOK_BLUEPRINTS\n#include "blueprint_spawn.hpp"\n#endif\n')


def configure(project):
    ids = {name: identity() for name in ["root", "leaf", "driver", "progress", "child", "texture", "clip", "spawn", "spawn_type", "leaf_value", "driver_value"]}
    native_sources(project, ids)
    reflection = json.loads(run("--project", project, "--reflect"))
    parent = next(c for c in reflection["classes"] if c["id"] == ids["root"])
    behaviour_id = next(c["id"] for c in reflection["classes"] if c["cpp_name"] == "epok::Behaviour")
    functions = {f["name"]: f for f in parent["functions"]}
    path = Path(run("--project", project, "--new-blueprint", "BP_Timeline", "--parent", ids["root"]).strip())
    blueprint = documents.loads(path.read_text(encoding="utf-8"))
    leaf_path = Path(run("--project", project, "--new-blueprint", "BP_Leaf", "--parent", ids["leaf"]).strip())
    leaf = documents.loads(leaf_path.read_text(encoding="utf-8"))
    leaf_functions = {f["name"]: f for c in reflection["classes"] if c["id"] == ids["leaf"] for f in c["functions"]}
    # Two generated classes call each other through DIFFERENT functions. This
    # exercises real foreign-BP C++ completeness without synchronous recursion.
    query_entry, query_return = node("entry"), node("return", {"value": literal("int32", 77)})
    wire(query_entry, query_return)
    query = {"id": identity(), "name": "answer", "override_id": None, "parameters": [],
             "returns": {"kind": "int32"}, "entry": query_entry["id"], "nodes": [query_entry, query_return]}
    remote_entry = node("entry")
    ask_owner = node("call_on", {"__target": {"kind": "parameter", "name": "owner"}},
                     **{"class": blueprint["id"], "function": query["id"]})
    hit = node("call", {"amount": {"kind": "parameter", "name": "amount"}, "answer": reference(ask_owner)},
               function=leaf_functions["hit"]["id"])
    remote_return = node("return", {"value": reference(hit)})
    wire(remote_entry, ask_owner, hit, remote_return)
    remote = {"id": identity(), "name": "remote_hit", "override_id": None,
              "parameters": [{"name": "owner", "value_type": {"kind": "entity_ref", "class": blueprint["id"]}, "direction": "value"},
                             {"name": "amount", "value_type": {"kind": "int32"}, "direction": "value"}],
              "returns": {"kind": "int32"}, "entry": remote_entry["id"], "nodes": [remote_entry, ask_owner, hit, remote_return]}
    leaf["functions"] = [remote]
    texture, clip = imported_resources(project)
    blueprint["variables"] = [
        {"id": ids["texture"], "name": "color_asset", "value_type": {"kind": "asset_ref", "asset_kind": "Texture"}, "default": texture, "editable": True},
        {"id": ids["clip"], "name": "audio_asset", "value_type": {"kind": "asset_ref", "asset_kind": "AudioClip"}, "default": clip, "editable": True},
        {"id": ids["spawn_type"], "name": "spawn_type", "value_type": {"kind": "class_ref", "base": behaviour_id}, "default": ids["spawn"], "editable": True},
    ]
    scene_path = project / "assets/scenes/Main.epokmap"
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    root_entity = copy.deepcopy(scene["entities"][0])
    root_entity.update(id=identity(), name="TemplateRoot", kind="Empty", parent=None, position=[0, 0, 0],
                       rotation=[0, 0, 0], scale=[1, 1, 1], script=None, audio={"play_on_start": False})
    child_entity = copy.deepcopy(root_entity)
    child_entity.update(id=identity(), name="TemplateChild", audio=None,
                        script=binding("BP_Leaf", leaf["id"], "blueprint"))
    blueprint["template"] = {
        "entities": [{"entity": root_entity, "parent": None}, {"entity": child_entity, "parent": root_entity["id"]}],
        "references": [{"owner": root_entity["id"], "member": ids["child"], "target": child_entity["id"]}],
        "construction": [{"operation": "set_position", "entity": child_entity["id"], "value": [2, 0, 0]},
                         {"operation": "translate", "entity": child_entity["id"], "offset": [1, 0, 0]}],
    }
    entry = node("entry")
    child = node("get_variable", member=ids["child"])
    owner = node("builtin", operation={"kind": "self_entity"})
    color = node("get_variable", member=ids["texture"])
    sound = node("get_variable", member=ids["clip"])
    record_link = node("call", {"child": reference(child)}, function=functions["record_link"]["id"])
    set_texture = node("builtin", {"target": reference(owner), "texture": reference(color)}, operation={"kind": "set_texture"})
    set_audio = node("builtin", {"target": reference(owner), "clip": reference(sound)}, operation={"kind": "set_audio_clip"})
    record_assets = node("call", function=functions["record_assets"]["id"])
    child_cast = node("builtin", {"target": reference(child)}, operation={"kind": "cast", "class": leaf["id"]})
    owner_cast = node("builtin", {"target": reference(owner)}, operation={"kind": "cast", "class": blueprint["id"]})
    remote_hit = node("call_on", {"__target": reference(child_cast), "owner": reference(owner_cast), "amount": literal("int32", 7)},
                      **{"class": leaf["id"], "function": remote["id"]})
    pure_count = node("call_on", {"__target": reference(child_cast)}, **{"class": leaf["id"], "function": leaf_functions["read_count"]["id"]})
    invalid_count = node("call_on", {"__target": {"kind": "literal", "value_type": {"kind": "entity_ref", "class": leaf["id"]}, "value": None}},
                         **{"class": leaf["id"], "function": leaf_functions["read_count"]["id"]})
    record_remote = node("call", {"value": reference(remote_hit), "duplicate": reference(remote_hit), "pure": reference(pure_count),
                                  "invalid": reference(invalid_count)}, function=functions["record_remote"]["id"])
    spawn_type = node("get_variable", member=ids["spawn_type"])
    dynamic_spawn = node("builtin", {"class": reference(spawn_type), "parent": reference(owner)},
                         operation={"kind": "spawn_class", "base": behaviour_id})
    invalid_spawn = node("builtin", {"class": {"kind": "literal", "value_type": {"kind": "class_ref", "base": behaviour_id}, "value": None},
                                      "parent": reference(owner)}, operation={"kind": "spawn_class", "base": behaviour_id})
    vector3 = node("make_vector", {name: literal("fixed", value) for name, value in zip(("x", "y", "z"), (2, 3, 4))}, length=3)
    vector2 = node("make_vector", {"x": literal("fixed", 6), "y": literal("fixed", 7)}, length=2)
    first = node("vector_component", {"value": reference(vector3)}, length=3, index=0)
    second = node("vector_component", {"value": reference(vector2)}, length=2, index=1)
    set_position = node("builtin", {"target": reference(dynamic_spawn), "value": reference(vector3)}, operation={"kind": "set_position"})
    record_vector = node("call", {"spawned": reference(dynamic_spawn), "invalid": reference(invalid_spawn), "x": reference(first), "second": reference(second)},
                         function=functions["record_vector"]["id"])
    destroy_spawn = node("builtin", {"target": reference(dynamic_spawn)}, operation={"kind": "destroy_entity"})
    timeline = node("timeline", keys=[[0, 0], [0.1, 10]], looping=False, member=ids["progress"])
    delay = node("delay", {"seconds": literal("fixed", 0.033)})
    record_branch = node("call", function=functions["record_branch"]["id"])
    should_stop = node("call", function=functions["should_stop"]["id"])
    branch = node("branch", {"condition": reference(should_stop)})
    stop = node("stop_timeline", node=timeline["id"])
    updated = node("call", function=functions["record_update"]["id"])
    finished = node("call", function=functions["record_finish"]["id"])
    wire(entry, record_link, set_texture, set_audio, record_assets, remote_hit, record_remote, dynamic_spawn, invalid_spawn,
         set_position, record_vector, destroy_spawn, timeline, delay, record_branch, branch)
    timeline["outputs"]["updated"] = [updated["id"]]
    timeline["outputs"]["finished"] = [finished["id"]]
    branch["outputs"]["true"] = [stop["id"]]
    function = functions["on_ready"]
    blueprint["functions"] = [{"id": identity(), "name": function["name"], "override_id": function["id"],
                               "parameters": function["parameters"], "returns": function["returns"], "entry": entry["id"],
                               "nodes": [entry, child, owner, color, sound, record_link, set_texture, set_audio,
                                         record_assets, child_cast, owner_cast, remote_hit, pure_count, invalid_count, record_remote,
                                         spawn_type, dynamic_spawn, invalid_spawn, vector3, vector2, first, second, set_position, record_vector, destroy_spawn,
                                         timeline, delay, record_branch, should_stop, branch, stop, updated, finished]}, query]
    write(path, blueprint)
    write(leaf_path, leaf)
    native_sources(project, ids, compact(blueprint["id"]))
    driver = copy.deepcopy(root_entity)
    driver.update(id=identity(), name="FeatureDriver", audio=None, script=binding("FeatureDriver", ids["driver"]))
    scene["entities"].append(driver)
    write(scene_path, scene)
    run("--project", project, "--compile-blueprints")
    # Invalid resource kinds and broken template references must not rewrite originals.
    wrong = copy.deepcopy(blueprint)
    wrong["variables"][0]["default"] = clip
    write(path, wrong)
    before = path.read_bytes()
    assert "expected Texture" in run("--project", project, "--compile-blueprints", ok=False)
    assert before == path.read_bytes()
    wrong = copy.deepcopy(blueprint)
    wrong["template"]["references"][0]["target"] = identity()
    write(path, wrong)
    before = path.read_bytes()
    assert "missing" in run("--project", project, "--compile-blueprints", ok=False).lower()
    assert before == path.read_bytes()
    write(path, blueprint)
    derived_path = Path(run("--project", project, "--new-blueprint", "BP_AudioOverrides", "--parent", blueprint["id"]).strip())
    derived = documents.loads(derived_path.read_text(encoding="utf-8"))
    derived["defaults"] = {ids["progress"]: 0, ids["clip"]: None}
    write(derived_path, derived)
    return blueprint


def emulator(project):
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 8077))
    log_path = project / "feature-emulator.log"
    with log_path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(EDITOR), "--project", str(project), "--play-psx", "--stop-after", "8"],
                                   cwd=ROOT, stdout=log, stderr=log)
        values = None
        try:
            deadline = time.monotonic() + 150
            while time.monotonic() < deadline and process.poll() is None:
                try:
                    symbols = (project / ".epok/build/epok.map").read_text()
                    found = re.search(r"0x([0-9a-fA-F]+)\s+blueprint_feature_probe\b", symbols)
                    if found:
                        with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                            values = struct.unpack_from("<32i", response.read(), int(found.group(1), 16) & 0x1fffff)
                        if values[31] == 1:
                            break
                except (OSError, ValueError):
                    pass
                time.sleep(0.1)
            assert values and values[0] == 0x42504632 and values[31] == 1, (values, log_path.read_text(errors="replace"))
            assert values[1:4] == (5, 5, 5), values
            assert values[5] == 5 and values[6] > 0 and values[7] == 3, values
            assert 0 < values[8] < 40960 and values[9] == 0 and values[10] == 4, values
            assert values[11:16] == (4, 4, 1, 1, 1), values
            assert values[16:21] == (5, 1, 1, 0, 5), values
            assert values[22] == 4 and values[23] == 40960, values
            assert values[24:26] == (5, 5), values
            assert values[26:29] == (5, 5, 5), values
            assert process.wait(timeout=45) == 0, log_path.read_text(errors="replace")
        finally:
            if process.poll() is None:
                process.wait(timeout=45)
    return values


def verify_audio_selection(project, blueprint):
    """Exercise observation and actual bank regeneration through production CLI."""
    path = project / "assets/Blueprints/BP_Timeline.epokbp"
    original = path.read_bytes()
    derived_path = project / "assets/Blueprints/BP_AudioOverrides.epokbp"
    derived_original = derived_path.read_bytes()
    scene_path = project / "assets/scenes/Main.epokmap"
    scene_original = scene_path.read_bytes()
    native_path = project / "assets/scripts/Features.hpp"
    native_original = native_path.read_bytes()
    derived = documents.loads(derived_original.decode("utf-8"))
    inherited_clip = next(v["id"] for v in blueprint["variables"] if v["value_type"].get("asset_kind") == "AudioClip")
    inherited_numeric = next(member for member in derived["defaults"] if member != inherited_clip)
    derived_selection = f"blueprint-audio:{derived['id']}"
    targets = (".epok/build", ".epok/build-blueprint-debug")
    def graph():
        return json.loads((project / ".epok/ArtifactDependencies.json").read_text())["nodes"]
    def bank(target):
        return f"generated-resource:{target}/audio-bank.hh"
    before = graph()
    original_executables = {target: (project / target / "epok.ps-exe").read_bytes() for target in targets}
    selection = f"blueprint-audio:{blueprint['id']}"
    for target in targets:
        assert selection in before[bank(target)]["dependencies"]
        assert "audio-catalog" in before[bank(target)]["dependencies"]
        assert "scene-catalog" not in before[bank(target)]["dependencies"]
        assert not any(key.startswith("blueprint:") for key in before[bank(target)]["dependencies"])
    try:
        changed = copy.deepcopy(blueprint)
        vector = next(node for fn in changed["functions"] for node in fn["nodes"]
                      if node["kind"]["kind"] == "make_vector")
        vector["inputs"]["x"]["value"] = 9
        write(path, changed)
        run("--project", project, "--compile-blueprints")
        logic = graph()
        for target in targets:
            assert logic[bank(target)] == before[bank(target)], "Scalar graph logic invalidated the audio bank"
            assert logic[f"stage:{target}"]["stale"], "Changed generated Blueprint must invalidate its stage"
        changed["variables"].append({"id": identity(), "name": "spell_power",
                                     "value_type": {"kind": "fixed"}, "default": 12,
                                     "editable": True, "timeline_animatable": True})
        write(path, changed)
        run("--project", project, "--compile-blueprints")
        for target in targets:
            assert graph()[bank(target)] == before[bank(target)], "Numeric declaration invalidated the audio bank"
        changed["variables"][-1]["default"] = 24
        write(path, changed)
        run("--project", project, "--compile-blueprints")
        for target in targets:
            assert graph()[bank(target)] == before[bank(target)], "Numeric default invalidated the audio bank"
        derived["defaults"][inherited_numeric] = 3
        write(derived_path, derived)
        run("--project", project, "--compile-blueprints")
        for target in targets:
            assert graph()[bank(target)] == before[bank(target)], "Inherited numeric default invalidated the audio bank"
        template_entity = changed["template"]["entities"][0]["entity"]
        template_entity["position"] = [3, 0, 0]
        template_entity["audio"]["volume"] = 0.5
        changed["template"].setdefault("construction", []).append(
            {"operation": "set_color", "entity": template_entity["id"], "color": [0.25, 0.5, 0.75]})
        changed["template"].setdefault("overrides", {}).setdefault(
            template_entity["id"], {"members": {}})["members"]["uq.entity.scale.v1"] = [2, 2, 2]
        template_child = changed["template"]["entities"][1]["entity"]
        template_child["script"]["properties"]["preview_gain"] = 2
        write(path, changed)
        run("--project", project, "--compile-blueprints")
        for target in targets:
            assert graph()[bank(target)] == before[bank(target)], "Template controls invalidated the audio bank"
        # Blueprint-only compilation observes source selection. Full staging
        # also observes the resolved scene/audio catalogs; check the untouched
        # debug destination after that publication, not just after graph compile.
        scene = documents.loads(scene_original.decode("utf-8"))
        driver = next(entity for entity in scene["entities"] if entity["name"] == "FeatureDriver")
        driver["script"]["properties"]["preview_gain"] = 3
        write(scene_path, scene)
        native_path.write_text(native_original.decode("utf-8").replace("progress=0.0", "progress=1.0"), encoding="utf-8")
        run("--project", project, "--build-psx")
        for target in targets:
            assert graph()[bank(target)] == before[bank(target)], "Resolved numeric/template data invalidated the audio bank"
        print("PASS: Numeric native, inherited, scene and template changes preserve both audio banks", flush=True)
        # A graph literal can contribute an additional clip independently of
        # the Blueprint variable default and the AudioSource in its template.
        imported = run("--project", project, "--import-audio", "assets/source/Silence.wav",
                       "--asset", "assets/Audio/GraphOnly.epokasset")
        clip = re.findall(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}", imported)[-1]
        derived["defaults"][inherited_clip] = clip
        write(derived_path, derived)
        run("--project", project, "--compile-blueprints")
        for target in targets:
            assert derived_selection in graph()[bank(target)]["stale"], "Inherited clip override did not invalidate its bank"
        run("--project", project, "--build-psx")
        assert f"asset:{clip}" in graph()[bank(targets[0])]["dependencies"]
        assert (project / targets[0] / f"audio/{clip}.adpcm").is_file()
        assert graph()[bank(targets[1])]["stale"], "Normal build certified an old inherited-clip debug bank"
        # The normal bank now contains the inherited clip; the debug bank still
        # retains its original output. Source-only compilation must preserve
        # each destination's most recently built bytes, not a shared baseline.
        retained = graph()
        print("PASS: Inherited clip selection stages its payload and preserves destination independence", flush=True)
        derived_path.write_bytes(derived_original)
        changed = copy.deepcopy(blueprint)
        audio_variable = next(variable for variable in changed["variables"]
                              if variable["value_type"].get("asset_kind") == "AudioClip")
        audio_variable["default"] = clip
        write(path, changed)
        run("--project", project, "--compile-blueprints")
        for target in targets:
            assert selection in graph()[bank(target)]["stale"], "Clip default did not invalidate its bank"
        changed = copy.deepcopy(blueprint)
        set_audio = next(node for fn in changed["functions"] for node in fn["nodes"]
                         if node["kind"].get("operation", {}).get("kind") == "set_audio_clip")
        set_audio["inputs"]["clip"] = {"kind": "literal", "value_type": {"kind": "asset_ref", "asset_kind": "AudioClip"}, "value": clip}
        write(path, changed)
        run("--project", project, "--compile-blueprints")
        selected = graph()
        for target in targets:
            assert selection in selected[bank(target)]["stale"]
            assert selected[bank(target)]["signature"] == retained[bank(target)]["signature"]
        run("--project", project, "--build-psx")
        rebuilt = graph()
        assert not rebuilt[bank(targets[0])]["stale"]
        assert f"asset:{clip}" in rebuilt[bank(targets[0])]["dependencies"]
        assert (project / targets[0] / f"audio/{clip}.adpcm").is_file()
        assert rebuilt[bank(targets[1])]["stale"], "A normal build certified an old debug bank"
    finally:
        path.write_bytes(original)
        derived_path.write_bytes(derived_original)
        scene_path.write_bytes(scene_original)
        native_path.write_bytes(native_original)
    run("--project", project, "--compile-blueprints")
    assert graph()[bank(targets[0])]["stale"], "Source restoration certified an old bank"
    run("--project", project, "--build-psx")
    run("--project", project, "--build-psx", "--blueprint-debug")
    restored = graph()
    for target in targets:
        assert not restored[bank(target)]["stale"]
        assert restored[bank(target)]["signature"] == before[bank(target)]["signature"]
        assert f"asset:{clip}" not in restored[bank(target)]["dependencies"]
        assert (project / target / "epok.ps-exe").read_bytes() == original_executables[target]
    print("PASS: Restored sources rebuild byte-identical normal/debug executables", flush=True)


def verify_duplicate_audio_identity(project, blueprint):
    """A copied Blueprint identity must not certify the last-discovered copy."""
    path = project / "assets/Blueprints/BP_Timeline.epokbp"
    duplicate = path.with_name("DuplicateAudioIdentity.epokbp")
    assert not duplicate.exists()
    targets = (".epok/build", ".epok/build-blueprint-debug")
    original_executables = {target: (project / target / "epok.ps-exe").read_bytes() for target in targets}
    selection = f"blueprint-audio:{blueprint['id']}"
    def graph():
        return json.loads((project / ".epok/ArtifactDependencies.json").read_text())["nodes"]
    def bank(target):
        return f"generated-resource:{target}/audio-bank.hh"
    try:
        duplicate.write_bytes(path.read_bytes())
        error = run("--project", project, "--compile-blueprints", ok=False)
        assert "Duplicate class UUID" in error, error
        ambiguous = graph()
        assert any("duplicate identity" in reason for reason in ambiguous[selection]["stale"].values())
        for target in targets:
            assert selection in ambiguous[bank(target)]["stale"]
    finally:
        duplicate.unlink()
    run("--project", project, "--compile-blueprints")
    for target in targets:
        assert graph()[bank(target)]["stale"], "Repair certified a retained bank"
    run("--project", project, "--build-psx")
    run("--project", project, "--build-psx", "--blueprint-debug")
    repaired = graph()
    for target in targets:
        assert not repaired[bank(target)]["stale"]
        assert (project / target / "epok.ps-exe").read_bytes() == original_executables[target]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emulator", action="store_true")
    parser.add_argument("--keep", action="store_true")
    args = parser.parse_args()
    manager = contextlib.nullcontext(tempfile.mkdtemp(prefix="epok-bp-features-")) if args.keep else tempfile.TemporaryDirectory(prefix="epok-bp-features-")
    with manager as directory:
        project = Path(directory) / "Blueprint Features"
        if args.keep:
            print(f"Retained project: {project}", flush=True)
        run("--create-project", project, "--name", "Blueprint Features")
        blueprint = configure(project)
        run("--project", project, "--build-psx")
        normal = project / ".epok/build/epok.ps-exe"
        normal_size = normal.stat().st_size
        release_map = (normal.parent / "epok.map").read_text()
        assert not re.search(r"0x[0-9a-fA-F]+\s+epok_blueprint_debug_hook\b", release_map)
        run("--project", project, "--build-psx", "--blueprint-debug")
        debug = project / ".epok/build-blueprint-debug/epok.ps-exe"
        debug_map = (debug.parent / "epok.map").read_text()
        assert re.search(r"0x[0-9a-fA-F]+\s+epok_blueprint_debug_hook\b", debug_map)
        assert re.search(r"0x[0-9a-fA-F]+\s+epok_blueprint_debug_snapshot\b", debug_map)
        assert normal_size == normal.stat().st_size, "Debug cooking changed the release artifact"
        provenance = json.loads((project / ".epok/ArtifactDependencies.json").read_text())["nodes"]
        blueprint_ids = sorted(documents.loads(path.read_text())["id"] for path in (project / "assets").rglob("*.epokbp"))
        source_set = provenance["blueprint-sources"]
        assert not source_set["stale"]
        assert source_set["signature"] == hashlib.sha256(json.dumps(blueprint_ids, separators=(",", ":")).encode()).hexdigest()
        for target, executable in [(".epok/build", normal), (".epok/build-blueprint-debug", debug)]:
            compiled = provenance[f"executable:{target}/epok.ps-exe"]
            assert not compiled["stale"]
            assert compiled["signature"] == hashlib.sha256(executable.read_bytes()).hexdigest()
            assert set(compiled["dependencies"]) == {f"stage:{target}", f"build-options:{target}", f"native-build:{target}"}
            assert not provenance[f"native-build:{target}"]["stale"]
            assert not provenance[f"native-sdk:{target}"]["stale"]
            assert f"native-metadata:file:project:{target}/sdk/libpsyqo.a" in provenance[f"native-build:{target}"]["dependencies"]
            assert not provenance[f"stage:{target}"]["stale"]
            assert "blueprint-sources" in provenance[f"staged-file:{target}/Scripts.epokmanifest"]["dependencies"]
            sources = provenance[f"staged-file:{target}/sources.mk"]
            assert not sources["stale"]
            assert sources["signature"] == hashlib.sha256((executable.parent / "sources.mk").read_bytes()).hexdigest()
            if executable == debug:
                assert f"build-options:{target}" in sources["dependencies"]
            display = provenance[f"generated-resource:{target}/display.hh"]
            assert not display["stale"] and display["dependencies"] == ["display-settings"]
            assert display["signature"] == hashlib.sha256((executable.parent / "display.hh").read_bytes()).hexdigest()
            audio_files = list((executable.parent / "audio").glob("*.adpcm"))
            assert audio_files, "The typed AudioClip fixture must stage a real audio payload"
            bank = provenance[f"generated-resource:{target}/audio-bank.hh"]
            assert not bank["stale"]
            assert bank["signature"] == hashlib.sha256((executable.parent / "audio-bank.hh").read_bytes()).hexdigest()
            assert any(key.startswith("audio-selection:scene-") for key in bank["dependencies"])
            assert not any(key.startswith(("scene-file:", "scene-editor:")) for key in bank["dependencies"])
            for payload in audio_files:
                resource = provenance[f"generated-resource:{target}/audio/{payload.name}"]
                assert not resource["stale"] and resource["dependencies"] == [f"asset:{payload.stem}"]
                assert resource["signature"] == hashlib.sha256(payload.read_bytes()).hexdigest()
                assert f"asset:{payload.stem}" in bank["dependencies"]
        verify_audio_selection(project, blueprint)
        verify_duplicate_audio_identity(project, blueprint)
        values = emulator(project) if args.emulator else None
        report = {"passed": True, "project": str(project), "blueprint": blueprint["id"], "probe": values,
                  "release_ps_exe_bytes": normal_size, "debug_ps_exe_bytes": debug.stat().st_size,
                  "debug_delta_bytes": debug.stat().st_size - normal_size,
                  "timelines": ["interpolation", "updated", "finished-once", "stop", "destroy-cancel", "fresh-instance-reset"],
                  "templates": ["root-child", "construction", "all-bound-before-start", "internal-handle-remap", "typed-pool-exhaustion"],
                  "typed_resources": ["Texture", "AudioClip", "wrong-kind-rejected"],
                  "audio_selection": ["scalar-logic-isolation", "numeric-declaration-isolation", "numeric-default-isolation",
                                      "inherited-numeric-default-isolation", "inherited-clip-invalidated-staged",
                                      "template-controls-isolation",
                                      "template-script-values-isolation", "scene-script-values-isolation", "native-numeric-default-isolation",
                                      "clip-default-invalidated", "graph-literal-added", "independent-debug-bank",
                                      "source-restoration-stale", "clip-removed", "identical-restored-executables",
                                      "duplicate-identity-rejected", "duplicate-repair-rebuilt"],
                  "foreign_calls": ["BP-to-BP-to-BP", "inherited-native", "exactly-once-result", "pure", "invalid-default", "checked-cast"],
                  "class_ref_spawn": ["variable-class", "base-check", "null-default", "spawn-destroy"],
                  "vectors": ["MakeVector2", "MakeVector3", "VectorComponent", "SetPosition"],
                  "equivalent_cpp_baseline_measured": False, "fps_claimed": False}
        write(ROOT / "artifacts/blueprints/features-validation.json", report)
        print("PASS: Timeline/template/resource native and debug builds" + (" and emulator RAM assertions" if args.emulator else ""), flush=True)


if __name__ == "__main__":
    main()
