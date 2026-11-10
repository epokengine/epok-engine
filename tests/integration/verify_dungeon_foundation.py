"""Validate generated key/door rules against the common character in five modes."""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes
from starter_bindings import player_roles

DRIVER = "876dd07a-c5a7-531d-bc6e-3fa53c5f99aa"
LISTENER = "b5f013c9-aefb-594d-bc56-d00e7930c63d"
SOURCE = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t dungeon_foundation_probe[8]={};inline uint32_t dungeon_foundation_trace[180*6]={};}
inline uint32_t dungeon_foreign_overlap=0;
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="LISTENER_ID") ForeignKeyObserver:public epok::ActorComponent {
public:
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId key{};
 void trigger_event(epok::ObjectId other,epok::TriggerPhase phase)override{if(other==key&&phase==epok::TriggerPhase::Enter)++dungeon_foreign_overlap;}
};
class EPOK_CLASS(Blueprintable,Id="DRIVER_ID") DungeonFoundationRoute:public epok::Actor3D {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::Input;
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Character3D") epok::ObjectId player{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") epok::ObjectId camera{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId key{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId door{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId foreign{};
 uint32_t frame=0,failures=0,key_closed=0,door_closed=0,opened=0,hash=2166136261u;
 void move(epok::Character3D& actor,epok::Fixed x,epok::Fixed z){
  actor.data()->transform.position[0]=x;actor.data()->transform.position[1]=0.0;actor.data()->transform.position[2]=z;actor.motor.reset_motion();
 }
 void tick(epok::Fixed)override{
  if(frame>180)return;
  auto* r=epok::active_object_registry;auto* actor=r?r->resolve<epok::Character3D>(player):nullptr;
  auto* rig=r?r->resolve<epok::CameraRig3D>(camera):nullptr;auto* held=r?r->resolve<epok::Actor3D>(key):nullptr;
  auto* gate=r?r->resolve<epok::Actor3D>(door):nullptr;auto* other=r?r->resolve<epok::Actor3D>(foreign):nullptr;
  if(!actor||!rig||!held||!gate||!other){failures|=1;return;}
  const auto at=epok::gameplay_world_affine(held->data());
  if(!at.success){failures|=2;return;}
  if(frame==0){
   dungeon_foreign_overlap=0;move(*actor,at.position[0]-epok::Fixed(4.0),at.position[2]);
   other->data()->transform.position[0]=at.position[0];other->data()->transform.position[1]=at.position[1]-epok::Fixed(0.92);other->data()->transform.position[2]=at.position[2];rig->snap();
  }else{
   const auto usage=epok::ResourceLibrary::snapshot();
   if(usage.sprite_triangles||usage.particles||usage.particle_dropped)failures|=32;
   if(frame==180&&!usage.frame_scanlines)failures|=64;
   const auto motion=actor->motion();const auto view=rig->sample();
   if(motion.status!=epok::CharacterMotionStatus::Applied||view.status!=epok::CameraRigStatus::Applied||rig->target!=player)failures|=4;
   if(frame<=30){key_closed+=held->active();door_closed+=gate->active();if(!held->active()||!gate->active())failures|=8;}
   if(frame>40){opened+=!held->active()&&!gate->active();if(held->active()||gate->active())failures|=16;}
   const uint32_t words[]={uint32_t(actor->data()->transform.position[0].raw()),uint32_t(actor->data()->transform.position[1].raw()),
    uint32_t(actor->data()->transform.position[2].raw()),held->active(),gate->active(),view.blocked};
   for(unsigned i=0;i<6;++i){dungeon_foundation_trace[(frame-1)*6+i]=words[i];hash=(hash^words[i])*16777619u;}
  }
  if(frame==30){other->set_active(false);move(*actor,at.position[0],at.position[2]);rig->snap();}
  if(frame==180){auto* p=dungeon_foundation_probe;p[0]=MAGIC+EPOK_LUA_MODE;p[1]=180;p[2]=failures;p[3]=dungeon_foreign_overlap;
   p[4]=key_closed;p[5]=door_closed;p[6]=opened;p[7]=hash;++frame;return;}
  epok::input.sample(0,true,0);epok::gameplay_controls().begin_frame();epok::input.begin_tick();epok::gameplay_controls().begin_tick();++frame;
 }
};
'''


def verify(out, flavor, modes):
    out = out / flavor
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"

    def run(*args):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        assert result.returncode == 0, result.stdout + result.stderr
    if not project.exists():
        run("--create-project", project, "--template", "dungeon", "--gameplay", flavor, "--name", "Dungeon Foundation " + flavor)
    magic = 0x59000000 | (uuid.uuid4().int & 0xffffff)
    write(project / "assets/scripts/DungeonFoundationRoute.hpp", SOURCE.replace("DRIVER_ID", DRIVER).replace("LISTENER_ID", LISTENER).replace("MAGIC", str(magic)))
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    assert manifest["rendering"]["sprite_triangle_budget"] == 64, "Generate a fresh project with the Dungeon sprite reserve"
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["actors"] = [a for a in scene["actors"] if a["class"].get("class_id") != DRIVER and a["name"] != "Foreign key contact"]
    roles = player_roles(project, scene, "dungeon", flavor)
    player, camera = roles or (
        next(a for a in scene["actors"] if a["class"]["name"] == "epok::Character3D"),
        next(a for a in scene["actors"] if a["class"]["name"] == "epok::ThirdPersonCamera"))
    gate = next(a for a in scene["actors"] if any(c["class"]["name"] == "DungeonDoor" for c in a["components"]))
    gate_component = next(c for c in gate["components"] if c["class"]["name"] == "DungeonDoor")
    requirement = gate_component["properties"].get("requirement_flag")
    if requirement:
        key = next(a for a in scene["actors"] if any(c["id"] == requirement for c in a["components"]))
        installed = documents.loads((project/"ProjectSettings/Template.epoksettings").read_text(encoding="utf-8"))
        flag = next(c for c in key["components"] if c["id"] == requirement)
        assert flag["class"]["class_id"] == installed["roles"]["key_state"]["class_id"]
        assert gate_component["properties"]["initial_state"] == 2
        assert any(c["id"] == gate_component["properties"]["collider_3d"] for c in gate["components"])
    else:
        key = next(a for a in scene["actors"] if a["id"] == gate_component["properties"]["key"])
    key_component = next(c for c in key["components"] if c["class"]["name"] == "DungeonKey")
    assert key_component["properties"].get("accepted_initiator",key_component["properties"].get("player")) == player["id"]
    if requirement:
        assert key_component["properties"]["collected_flag"] == requirement
        assert key_component["properties"]["auto_collect"] is True
    import copy
    shape = copy.deepcopy(next(c for c in player["components"] if c["class"]["name"] == "epok::Collider3DComponent"))
    shape.update({"id": str(uuid.uuid4()), "name": "Foreign Collider", "inherited": False})
    shape.pop("default_id", None)
    root_class = next(c["class"] for c in player["components"] if c.get("root"))
    actor_class = next(a["class"] for a in scene["actors"] if a["class"]["name"] == "epok::Actor3D")

    def root():
        return {"id": str(uuid.uuid4()), "name": "Root", "root": True, "class": root_class, "properties": {"position": [0, 0, 0]}, "overrides": ["position"]}
    other = {"id": str(uuid.uuid4()), "name": "Foreign key contact", "class": actor_class,
        "components": [root(), shape, {"id": str(uuid.uuid4()), "name": "Foreign observer", "class": {"name": "ForeignKeyObserver", "class_id": LISTENER}, "properties": {"key": key["id"]}, "overrides": ["key"]}], "properties": {}, "overrides": []}
    scene["actors"].append(other)
    props = {"player": player["id"], "camera": camera["id"], "key": key["id"], "door": gate["id"], "foreign": other["id"]}
    scene["actors"].insert(0, {"id": str(uuid.uuid4()), "name": "Dungeon route", "class": {"name": "DungeonFoundationRoute", "class_id": DRIVER},
        "components": [root()], "properties": props, "overrides": list(props)})
    player["name"] = "Renamed hero"
    camera["name"] = "Renamed view"
    key["name"] = "Renamed key"
    gate["name"] = "Renamed gate"
    write(scene_path, scene)

    def check(values, expected):
        assert values[:3] == [expected, 180, 0], values
        assert values[3] > 0 and values[4:6] == [30, 30] and values[6] >= 120, values
    run_modes(out, project, flavor, modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",)),
              "dungeon_foundation_probe", magic, check, provider_count=1,trace=("dungeon_foundation_trace",180*6))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/dungeon-foundation")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    args = parser.parse_args()
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.out, flavor, args.mode)
    if not (args.flavor or args.mode):
        rows = [row for flavor in ("cpp", "blueprint", "lua") for row in json.loads((args.out / flavor / "result.json").read_text())]
        assert len(rows) == 5 and len({tuple(row["sample"][1:]) for row in rows}) == 1, rows
        print("PASS five Dungeon modes preserve character bindings, player-only keys and linked doors", flush=True)
