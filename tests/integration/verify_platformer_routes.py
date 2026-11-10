"""Real template tiles/lift with the selected project input and shared motor.

Only initial poses/velocity and pad samples are supplied by the fixture. The
shipped collider, one-way tiles, motor, path follower and carry service run normally.
"""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes
from starter_bindings import platformer_roles

DRIVER = "89f3bff0-af37-5698-88de-06342f9f9d7c"
SOURCE = r'''#pragma once
#include "epok.hpp"
inline epok::Fixed route_abs(epok::Fixed value){return value<0.0?-value:value;}
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t platformer_routes_probe[8]={};}
class EPOK_CLASS(Blueprintable,Id="DRIVER_ID") PlatformerSurfaceRoute:public epok::Actor2D {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::Input;
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Character2D") epok::ObjectId player{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor2D") epok::ObjectId lift{};
 uint32_t frame=0,failures=0,passed_up=0,landed=0,jumped=0,carried=0,moving=0;
 epok::Fixed previous_player[2]{},previous_lift[2]{};
 void place(epok::Character2D& actor,epok::Fixed x,epok::Fixed y,epok::Fixed vy){
  actor.root.set_position(x,y);actor.motor.reset_motion();actor.body.set_velocity(0.0,vy);
 }
 void tick(epok::Fixed)override{
  if(frame>360)return;
  auto* registry=epok::active_object_registry;
  auto* actor=registry?registry->resolve<epok::Character2D>(player):nullptr;
  auto* platform=registry?registry->resolve<epok::Actor2D>(lift):nullptr;
  epok::Affine2D matrix;epok::Fixed zero[2]={0.0,0.0},at[2]{};
  if(!actor||!platform||!epok::world2d::world_of(platform->root,matrix)){failures|=1;return;}
  matrix.point(zero,at);
  const auto x=actor->root.position_x(),y=actor->root.position_y();
  if(frame){
   const auto motion=actor->motion();
   if(motion.status!=epok::CharacterMotionStatus::Applied)failures|=2;
   if(frame<=120){
    passed_up+=motion.velocity.y<0.0&&y+epok::Fixed(12.0)<epok::Fixed(32.0);
    if(motion.on_ceiling)failures|=4;
    const bool on_plank=motion.grounded&&actor->body.last_hit_tile&&actor->body.last_tile[1]==2&&
      actor->body.last_tile[0]>=-13&&actor->body.last_tile[0]<=-11;
    landed+=on_plank;
    if(on_plank&&route_abs(y-epok::Fixed(20.0))>epok::Fixed(0.1))failures|=8;
   }
   if(frame>120&&frame<=200)jumped+=motion.jumped;
   if(frame>200&&motion.grounded&&actor->body.last_collider==lift){
    ++carried;
    if(route_abs(x-at[0])>epok::Fixed(0.1)||route_abs(y-(at[1]-epok::Fixed(15.0)))>epok::Fixed(0.1))failures|=16;
    if((at[0]!=previous_lift[0]||at[1]!=previous_lift[1])&&frame>202){
     ++moving;
     if(route_abs((x-previous_player[0])-(at[0]-previous_lift[0]))>epok::Fixed(0.1)||
        route_abs((y-previous_player[1])-(at[1]-previous_lift[1]))>epok::Fixed(0.1))failures|=32;
    }
   }
  }
  if(frame==0)place(*actor,-184.0,50.0,-264.0);
  if(frame==200)place(*actor,at[0],at[1]-epok::Fixed(15.0),0.0);
  previous_player[0]=actor->root.position_x();previous_player[1]=actor->root.position_y();
  previous_lift[0]=at[0];previous_lift[1]=at[1];
  if(frame==360){auto* p=platformer_routes_probe;p[0]=MAGIC+EPOK_LUA_MODE;p[1]=360;p[2]=failures;
   p[3]=passed_up;p[4]=landed;p[5]=jumped;p[6]=carried;p[7]=moving;++frame;return;}
  const uint16_t buttons=frame==120?uint16_t(1u<<unsigned(epok::Button::Cross)):0;
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
        run("--create-project", project, "--template", "platformer-2d", "--gameplay", flavor, "--name", "Surface Routes " + flavor)
    magic = 0x55000000 | (uuid.uuid4().int & 0xffffff)
    write(project / "assets/scripts/PlatformerSurfaceRoute.hpp", SOURCE.replace("DRIVER_ID", DRIVER).replace("MAGIC", str(magic)))
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["actors"] = [a for a in scene["actors"] if a["class"].get("class_id") != DRIVER]
    roles = platformer_roles(project, scene, flavor)
    player = roles[0] if roles else next(a for a in scene["actors"] if a["class"]["name"] == "epok::Character2D")
    lift = next(a for a in scene["actors"] if a["name"] == "Lift")
    props = {"player": player["id"], "lift": lift["id"]}
    root_class = next(c["class"] for c in player["components"] if c.get("root"))
    scene["actors"].insert(0, {"id": str(uuid.uuid4()), "name": "Surface route", "class": {"name": "PlatformerSurfaceRoute", "class_id": DRIVER},
        "components": [{"id": str(uuid.uuid4()), "name": "Root", "root": True, "class": root_class, "properties": {"position": [0, 0]}, "overrides": ["position"]}],
        "properties": props, "overrides": list(props)})
    write(scene_path, scene)

    def check(values, expected):
        assert values[:3] == [expected, 360, 0], values
        assert values[3] > 0 and values[4] >= 30 and values[5] == 1 and values[6] >= 120 and values[7] >= 100, values
    run_modes(out, project, flavor, modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",)),
              "platformer_routes_probe", magic, check, provider_count=1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/platformer-surfaces")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    args = parser.parse_args()
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.out, flavor, args.mode)
    if not (args.flavor or args.mode):
        rows = [row for flavor in ("cpp", "blueprint", "lua") for row in json.loads((args.out / flavor / "result.json").read_text())]
        assert len(rows) == 5 and len({tuple(row["sample"][1:]) for row in rows}) == 1, rows
        print("PASS five template modes preserve one-way tile traversal, jumping and lift carry", flush=True)
