"""Extend both native locomotion policies through C++, Blueprint and Lua on PSX."""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes, export_probe


def verify(out, domain, modes, prepare_only, standalone_only):
    out = out / domain
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    dim = "3D" if domain == "world3d" else "2D"
    driver = "SkeletalLocomotionComponent" if dim == "3D" else "SpriteLocomotion2DComponent"
    namespace = uuid.UUID("c479e55d-7251-55f2-923e-ff2ccfe77b76")
    identity = lambda name: str(uuid.uuid5(namespace, domain + ":" + name))
    magic = 0x52000000 | (uuid.uuid4().int & 0xffffff)
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
        export_probe(out, project, run, ["character_animation.hpp", "character2d.hpp", "character3d.hpp"])
        return
    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Animation " + dim)
    source = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t character_animation_probe[24]={};}
extern "C" {inline uint32_t character_animation_trace[3][120][5]={};}
struct AnimationProbeState {uint32_t steps=0,callbacks=0,failures=0,states=0,changes=0,hash=2166136261u;};
inline AnimationProbeState animation_states[3];
BANK
inline uint32_t animation_hash(epok::LocomotionSample value){uint32_t result=2166136261u;const uint32_t words[]={uint32_t(value.status),uint32_t(value.state),uint32_t(value.clip),uint32_t(value.rate.raw()),value.changed};for(auto word:words)result=(result^word)*16777619u;return result;}
class EPOK_CLASS(Blueprintable,Id="ACTOR") AnimationCharacterProbe:public epok::CharacterDIM {
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 void tick(epok::Fixed)override{if(probe_slot>=3)return;const auto step=animation_states[probe_slot].steps;epok::CharacterCommandDIM command;COMMAND command.jump=step==20;submit_command(command);}
};
class EPOK_CLASS(Abstract,Blueprintable,Domain=DOMAIN,Owners=DOMAIN,Id="BASE") AnimationProviderProbe:public epok::DRIVER {
 epok::Fixed original_rate_=1.0;
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t probe_slot=0;
 EPOK_FUNCTION(BlueprintCallable) epok::LocomotionSelection default_selection(epok::CharacterMotionSampleDIM motion,epok::LocomotionState previous,bool finished){auto value=epok::DRIVER::select_animation(motion,previous,finished);original_rate_=value.rate;return value;}
 EPOK_FUNCTION(BlueprintCallable) void check_original(epok::LocomotionSelection value){if(probe_slot<3&&value.rate!=original_rate_)animation_states[probe_slot].failures|=32;}
 EPOK_FUNCTION(BlueprintCallable) void check_motion(epok::CharacterMotionSampleDIM original,epok::Fixed copied_y){auto* actor=epok::active_object_registry->resolve<epok::CharacterDIM>(owner_id());if(probe_slot<3&&(!actor||original.velocity.y!=actor->motion().velocity.y||copied_y!=epok::Fixed(123.0)))animation_states[probe_slot].failures|=64;}
 EPOK_FUNCTION(BlueprintCallable) void observe_animation(epok::LocomotionSample value){
  if(probe_slot>=3)return;auto& state=animation_states[probe_slot];if(state.steps>=120)return;
  if(value.status!=epok::LocomotionStatus::Applied||value.rate!=epok::Fixed(1.25))state.failures|=1;
  if(animation_hash(value)!=animation_hash(sample()))state.failures|=2;
  if(update(epok::Fixed(1.0/60.0)).status!=epok::LocomotionStatus::Busy)state.failures|=4;
  auto* actor=epok::active_object_registry->resolve<epok::CharacterDIM>(owner_id());
  if(!actor||actor->motion().status!=epok::CharacterMotionStatus::Applied)state.failures|=8;
  PLAYBACK
  auto* trace=character_animation_trace[probe_slot][state.steps];trace[0]=unsigned(value.state);trace[1]=uint32_t(value.clip);trace[2]=uint32_t(value.rate.raw());trace[3]=value.changed;trace[4]=CURSOR;
  state.states|=1u<<unsigned(value.state);state.changes+=value.changed;++state.callbacks;
  state.hash=(state.hash^animation_hash(value))*16777619u;state.hash=(state.hash^CURSOR)*16777619u;++state.steps;
  if(state.steps==120){auto* p=character_animation_probe+probe_slot*8;p[1]=state.steps;p[2]=state.failures;p[3]=state.states;p[4]=state.changes;p[5]=state.hash;p[6]=state.callbacks;p[7]=MOVEMENT;p[0]=MAGIC+EPOK_LUA_MODE;}
 }
 void begin_play()override{epok::DRIVER::begin_play();if(probe_slot>=3)return;animation_states[probe_slot]={};clips={0,1,2,3,4,5};INITIALIZE}
};
class EPOK_CLASS(Blueprintable,Id="CPP") CppAnimationProviderProbe:public AnimationProviderProbe {
public:
 epok::LocomotionSelection select_animation(epok::CharacterMotionSampleDIM motion,epok::LocomotionState previous,bool finished)override{auto original=default_selection(motion,previous,finished);auto value=original;value.rate=1.25;check_original(original);auto copy=motion;copy.velocity.y=123.0;check_motion(motion,copy.velocity.y);return value;}
 void animation_applied(epok::LocomotionSample value)override{observe_animation(value);}
};
'''
    if dim == "3D":
        bank = "inline const epok::AnimationClip animation_clips[]={{nullptr,nullptr,nullptr,6,nullptr},{nullptr,nullptr,nullptr,8,nullptr},{nullptr,nullptr,nullptr,10,nullptr},{nullptr,nullptr,nullptr,5,nullptr},{nullptr,nullptr,nullptr,4,nullptr},{nullptr,nullptr,nullptr,3,nullptr}};\ninline const epok::SkeletalMesh animation_model={nullptr,nullptr,nullptr,nullptr,nullptr,0,animation_clips,6,{}};"
        command = "command.velocity.y=step<10||step>=110?epok::Fixed(0.0):step<60?epok::Fixed(2.0):epok::Fixed(6.0);"
        initialize = "auto* data=get_owner()->data();data->animator.enabled=true;data->animator.model=&animation_model;"
        playback = "auto& animator=actor->data()->animator;if(animator.clip!=value.clip||animator.looping!=(unsigned(value.state)<=2)||animator.playing)state.failures|=16;"
        cursor = "actor->data()->animator.ticks"
        movement = "uint32_t(actor->data()->transform.position[2].raw())"
    else:
        bank = "inline const epok::SpriteFrame animation_frames[]={{{0,0,8,8},epok::Fixed(0.05),11},{{8,0,8,8},epok::Fixed(0.05),12}};\ninline const epok::SpriteClip animation_clips[]={{animation_frames,2,true,nullptr},{animation_frames,2,true,nullptr},{animation_frames,2,true,nullptr},{animation_frames,2,false,nullptr},{animation_frames,2,false,nullptr},{animation_frames,2,false,nullptr}};"
        command = "command.velocity.x=step<10||step>=110?epok::Fixed(0.0):step<60?epok::Fixed(96.0):epok::Fixed(192.0);"
        initialize = "animate_air=true;auto* sprite=epok::world2d::find<epok::Sprite2DComponent>(get_owner());sprite->clips=animation_clips;sprite->clip_count=6;"
        playback = "auto* sprite=epok::world2d::find<epok::Sprite2DComponent>(actor);if(!sprite||sprite->animation!=value.clip||sprite->speed_scale!=value.rate)state.failures|=16;"
        cursor = "uint32_t(sprite->frame)*4096+uint32_t(sprite->elapsed.raw())"
        movement = "uint32_t(actor->root.transform.position[0].raw())-probe_slot*200u*4096u"
    for key, value in {"BANK": bank, "COMMAND": command, "INITIALIZE": initialize, "PLAYBACK": playback, "CURSOR": cursor,
                       "MOVEMENT": movement, "DOMAIN": "World3D" if dim == "3D" else "World2D", "DRIVER": driver,
                       "DIM": dim, "MAGIC": str(magic) + "u"}.items():
        source = source.replace(key, value)
    for key in ("actor", "base", "cpp"):
        source = source.replace('"' + key.upper() + '"', '"' + identity(key) + '"')
    write(project / "assets/scripts/AnimationProviderProbe.hpp", source)
    classes = {c["cpp_name"]: c for c in json.loads(run("--project", project, "--reflect"))["classes"]}
    base_functions = {f["name"]: f for f in classes["AnimationProviderProbe"]["functions"]}
    events = {f["name"]: f for f in classes["epok::" + driver]["functions"]}
    bp_path = project / "assets/Blueprints/BP_AnimationProviderProbe.epokbp"
    if not bp_path.exists():
        run("--project", project, "--new-blueprint", "BP_AnimationProviderProbe", "--parent", identity("base"))
    asset = documents.loads(bp_path.read_text(encoding="utf-8"))
    asset["functions"] = []
    asset.setdefault("layout", {})["split_pins"] = []
    for method, called in (("select_animation", "default_selection"), ("animation_applied", "observe_animation")):
        event = events[method]
        entry = {"id": str(uuid.uuid4()), "kind": {"kind": "entry"}, "inputs": {}, "outputs": {}}
        call = {"id": str(uuid.uuid4()), "kind": {"kind": "call", "function": base_functions[called]["id"]},
                "inputs": {p["name"]: {"kind": "parameter", "name": p["name"]} for p in event["parameters"]}, "outputs": {}}
        returned = {"id": str(uuid.uuid4()), "kind": {"kind": "return"}, "inputs": {}, "outputs": {}}
        if method == "select_animation":
            returned["inputs"]["value"] = {"kind": "literal", "value_type": event["returns"],
                                            "value": {"state": 0, "clip": 0, "looping": True, "rate": 1.25}}
            asset["layout"]["split_pins"].extend([call["id"] + ":out:value", returned["id"] + ":in:value"])
            for field in ("state", "clip", "looping"):
                returned["inputs"]["value." + field] = {"kind": "link", "node": call["id"], "pin": "value." + field}
            returned["inputs"]["value.rate"] = {"kind": "literal", "value_type": {"kind": "fixed"}, "value": 1.25}
            check_original = {"id": str(uuid.uuid4()), "kind": {"kind": "call", "function": base_functions["check_original"]["id"]},
                              "inputs": {"value": {"kind": "link", "node": call["id"], "pin": "value"}}, "outputs": {}}
            check_motion = {"id": str(uuid.uuid4()), "kind": {"kind": "call", "function": base_functions["check_motion"]["id"]},
                            "inputs": {"original": {"kind": "parameter", "name": "motion"},
                                       "copied_y": {"kind": "literal", "value_type": {"kind": "fixed"}, "value": 123.0}}, "outputs": {}}
        entry["outputs"]["next"] = [call["id"]]
        call["outputs"]["next"] = [check_original["id"]] if method == "select_animation" else [returned["id"]]
        if method == "select_animation":
            check_original["outputs"]["next"] = [check_motion["id"]]
            check_motion["outputs"]["next"] = [returned["id"]]
        asset["functions"].append({"id": str(uuid.uuid4()), "name": method, "override_id": event["id"],
                                   "parameters": event["parameters"], "returns": event["returns"], "entry": entry["id"],
                                   "nodes": [entry, call, check_original, check_motion, returned] if method == "select_animation" else [entry, call, returned]})
    write(bp_path, asset)
    write(project / "assets/scripts/LuaAnimationProviderProbe.lua", '''---@class LuaAnimationProviderProbe : AnimationProviderProbe
local LuaAnimationProviderProbe = AnimationProviderProbe:extend()
---@override
function LuaAnimationProviderProbe:select_animation(motion, previous, finished)
    local original = self:default_selection(motion, previous, finished)
    local value = original
    value.rate = 1.25
    value.state = original.state
    value.clip = original.clip
    value.looping = original.looping
    self:check_original(original)
    local copy = motion
    copy.velocity.y = 123.0
    self:check_motion(motion, copy.velocity.y)
    return value
end
---@override
function LuaAnimationProviderProbe:animation_applied(value)
    self:observe_animation(value)
end
return LuaAnimationProviderProbe
''')
    write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {"assets/scripts/LuaAnimationProviderProbe.lua": identity("lua")}})
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

    scene = place("Floor", classes["epok::Actor" + dim]["id"])
    floor = scene["actors"][-1]
    floor["components"][0]["properties"]["position"] = [0, -.25, 0] if dim == "3D" else [200, 16]
    floor["components"].append(component("Collider", "epok::Collider" + dim + "Component", {
        "collider" if dim == "3D" else "collider2d": {"half_extents": [100, .25, 100]} if dim == "3D" else {"size": [2000, 16]}}))
    write(scene_path, scene)
    for slot, (cls, cls_id) in enumerate((("CppAnimationProviderProbe", identity("cpp")),
                                         ("BP_AnimationProviderProbe", asset["id"]), ("LuaAnimationProviderProbe", identity("lua")))):
        scene = place("Same display name", identity("actor"))
        actor = scene["actors"][-1]
        actor["properties"]["probe_slot"] = slot
        actor["overrides"] = list(actor["properties"])
        actor["components"][0]["properties"]["position"] = [slot * 10, 1, 0] if dim == "3D" else [slot * 200, 0]
        if dim == "3D":
            next(c for c in actor["components"] if c["class"]["class_id"] == classes["epok::Collider3DComponent"]["id"])["properties"]["collider"] = {"half_extents": [.4, 1, .4]}
        actor["components"].append(component("Visual", "epok::Mesh3DComponent" if dim == "3D" else "epok::Sprite2DComponent", {}))
        policy = component("Animation", "AnimationProviderProbe", {"probe_slot": slot})
        policy["class"] = {"name": cls, "class_id": cls_id}
        actor["components"].append(policy)
        write(scene_path, scene)
    run("--project", project, "--compile-blueprints")
    if prepare_only:
        print("PASS", domain, "three editable animation policy overrides prepared", flush=True)
        return

    def check(values, expected):
        assert values[:3] == [expected, 120, 0] and values[4] >= 4 and values[6] == 120 and values[7] > 0, values
        assert values[3] & 1 and values[3] & (1 << 1) and values[3] & (1 << 2), values
        assert values[3] & (1 << 3) and values[3] & (1 << 4) and values[3] & (1 << 5), values

    run_modes(out, project, domain, modes, "character_animation_probe", magic, check)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/character-animation")
    parser.add_argument("--domain", choices=("world3d", "world2d"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    for domain in args.domain or ("world3d", "world2d"):
        verify(args.output.resolve(), domain, args.mode, args.prepare_only, args.standalone_only)
