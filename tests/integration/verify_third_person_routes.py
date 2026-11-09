"""Walk the shipped ramp side, wall and camera obstruction in every flavor."""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes
from starter_bindings import player_roles

DRIVER = "2171609e-fe78-55b3-b83c-f42dff486183"
SOURCE = r'''#pragma once
#include "epok.hpp"
inline epok::Fixed route_abs(epok::Fixed value){return value<0.0?-value:value;}
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t third_person_routes_probe[8]={};}
class EPOK_CLASS(Blueprintable,Id="DRIVER_ID") ArenaSurfaceRoute:public epok::Actor3D {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::Input;
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Character3D") epok::ObjectId player{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") epok::ObjectId camera{};
 uint32_t frame=0,failures=0,up=0,down=0,wall=0,blocked=0,clear=0,hash=2166136261u;
 void place(epok::Character3D& actor,epok::CameraRig3D& rig,epok::Fixed x,epok::Fixed z){
  auto* data=actor.data();data->transform.position[0]=x;data->transform.position[1]=0.0;data->transform.position[2]=z;
  data->transform.rotation[1]=0.0;actor.motor.reset_motion();rig.set_angles(0.0,18.0);rig.snap();
 }
 void tick(epok::Fixed)override{
  if(frame>300)return;
  auto* registry=epok::active_object_registry;auto* actor=registry?registry->resolve<epok::Character3D>(player):nullptr;
  auto* rig=registry?registry->resolve<epok::CameraRig3D>(camera):nullptr;
  if(!actor||!rig||!actor->data()){failures|=1;return;}
  const auto& position=actor->data()->transform.position;
  if(frame){
   const auto motion=actor->motion();const auto view=rig->sample();
   if(motion.status!=epok::CharacterMotionStatus::Applied||view.status!=epok::CameraRigStatus::Applied)failures|=2;
   if(frame<=70){
    up+=motion.grounded&&position[1]>epok::Fixed(0.2);
    if(up&&position[0]>epok::Fixed(-8.5)&&motion.grounded&&route_abs(position[1])<epok::Fixed(0.1))down=1;
   }else if(frame<=140){
    wall+=motion.blocked;
    if(position[0]<epok::Fixed(-15.3))failures|=4;
   }else if(frame<=210){
    blocked+=view.blocked;
    if(view.blocked&&(view.position.z<=epok::Fixed(-15.5)||view.position.z>=position[2]))failures|=8;
   }else if(frame>220){
    clear+=!view.blocked;
    const auto reach=rig->distance*epok::MathLibrary::cosine_degrees(rig->pitch);
    if(route_abs(view.position.z-(position[2]-reach))>epok::Fixed(0.02))failures|=16;
   }
   const uint32_t words[]={uint32_t(position[0].raw()),uint32_t(position[1].raw()),uint32_t(position[2].raw()),
     uint32_t(motion.facing.raw()),uint32_t(view.position.x.raw()),uint32_t(view.position.y.raw()),uint32_t(view.position.z.raw())};
   for(auto word:words)hash=(hash^word)*16777619u;
  }
  if(frame==0)place(*actor,*rig,-12.0,2.5);
  if(frame==70)place(*actor,*rig,-14.5,-10.0);
  if(frame==140)place(*actor,*rig,-7.0,-14.5);
  if(frame==210)place(*actor,*rig,-7.0,-6.0);
  if(frame==300){auto* p=third_person_routes_probe;p[0]=MAGIC+EPOK_LUA_MODE;p[1]=300;p[2]=failures;p[3]=up;
   p[4]=(hash&0xfffffffeu)|down;p[5]=wall;p[6]=blocked;p[7]=clear;++frame;return;}
  uint16_t buttons=frame<70?uint16_t(1u<<unsigned(epok::Button::Right)):
   frame<140?uint16_t(1u<<unsigned(epok::Button::Left)):0;
  epok::input.sample(0,true,buttons);epok::gameplay_controls().begin_frame();epok::input.begin_tick();epok::gameplay_controls().begin_tick();++frame;
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
        run("--create-project", project, "--template", "third-person", "--gameplay", flavor, "--name", "Arena Routes " + flavor)
    magic = 0x57000000 | (uuid.uuid4().int & 0xffffff)
    write(project / "assets/scripts/ArenaSurfaceRoute.hpp", SOURCE.replace("DRIVER_ID", DRIVER).replace("MAGIC", str(magic)))
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["actors"] = [a for a in scene["actors"] if a["class"].get("class_id") != DRIVER]
    roles = player_roles(project, scene, "third-person", flavor)
    player, camera = roles or (
        next(a for a in scene["actors"] if a["class"]["name"] == "epok::Character3D"),
        next(a for a in scene["actors"] if a["class"]["name"] == "epok::ThirdPersonCamera"))
    props = {"player": player["id"], "camera": camera["id"]}
    root_class = next(c["class"] for c in player["components"] if c.get("root"))
    scene["actors"].insert(0, {"id": str(uuid.uuid4()), "name": "Arena route", "class": {"name": "ArenaSurfaceRoute", "class_id": DRIVER},
        "components": [{"id": str(uuid.uuid4()), "name": "Root", "root": True, "class": root_class, "properties": {"position": [0, 0, 0]}, "overrides": ["position"]}],
        "properties": props, "overrides": list(props)})
    write(scene_path, scene)

    def check(values, expected):
        assert values[:3] == [expected, 300, 0], values
        assert values[3] > 0 and values[4] & 1 and values[5] >= 30 and values[6] >= 60 and values[7] >= 70, values
    run_modes(out, project, flavor, modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",)),
              "third_person_routes_probe", magic, check, provider_count=1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/third-person-surfaces")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    args = parser.parse_args()
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.out, flavor, args.mode)
    if not (args.flavor or args.mode):
        rows = [row for flavor in ("cpp", "blueprint", "lua") for row in json.loads((args.out / flavor / "result.json").read_text())]
        assert len(rows) == 5 and len({tuple(row["sample"][1:]) for row in rows}) == 1, rows
        print("PASS five template modes preserve step-up/down, wall response and camera retraction", flush=True)
