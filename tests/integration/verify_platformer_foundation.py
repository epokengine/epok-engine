"""Exercise generated 2D input/motor/sprite, attached camera and coin rules."""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes, export_probe
from starter_bindings import platformer_roles

DRIVER = "aa3902d0-6029-5df2-bd83-5a8e8fbb0231"
REPORTER = "29b083e2-6280-5932-8d4b-de0c0f976304"
SOURCE = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" { inline uint32_t platformer_foundation_probe[8]={}; }
class EPOK_CLASS(Blueprintable,Id="DRIVER") PlatformerRouteInput:public epok::Actor2D {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::Input;
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Character2D") epok::ObjectId player{};
 uint32_t frame=0,failures=0,hash=2166136261u,states=0,jumps=0,gravity_steps=0,travel=0,capture_start=0;
 epok::Fixed previous_y=0.0;int32_t initial_camera=0;
 epok::InputCapture capture{};
 void tick(epok::Fixed)override{
  if(frame>=240)return;
  using G=epok::GameplayInputLibrary;using A=epok::GameplayAction;
  auto* character=epok::active_object_registry->resolve<epok::Character2D>(player);
  if(!capture_start&&character&&character->motion().jumped){capture=G::capture(id(),G::mask(A::MoveX)|G::mask(A::MoveY)|G::mask(A::Jump),100,0);capture_start=frame;}
  if(capture_start&&frame==capture_start+20){if(!G::release(capture))failures|=1;capture={};}
  uint16_t buttons=0;
  if(frame>=10&&frame<45)buttons|=1u<<unsigned(epok::Button::Left);
  if(frame>=45&&frame<210)buttons|=1u<<unsigned(epok::Button::Right);
  if(frame==60||frame==130)buttons|=1u<<unsigned(epok::Button::Cross);
  epok::input.sample(0,true,buttons);epok::gameplay_controls().begin_frame();epok::input.begin_tick();epok::gameplay_controls().begin_tick();++frame;
 }
};
class EPOK_CLASS(Blueprintable,Owners=World2D,Id="REPORTER") PlatformerRouteReporter:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere,ActorClass="PlatformerRouteInput") epok::ObjectId driver{};
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::SpriteLocomotion2DComponent") epok::ObjectId animation{};
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::Camera2DComponent") epok::ObjectId camera{};
COIN_PROPERTIES
 void tick(epok::Fixed)override{
  auto* registry=epok::active_object_registry;auto* route=registry?registry->resolve<PlatformerRouteInput>(driver):nullptr;
  if(!route||!route->frame||route->frame>240||platformer_foundation_probe[0])return;
  auto* player=registry->resolve<epok::Character2D>(owner_id());auto* visual=registry->resolve<epok::SpriteLocomotion2DComponent>(animation);
  auto* view=registry->resolve<epok::Camera2DComponent>(camera);auto* camera_actor=view?registry->resolve<epok::Actor2D>(view->owner_id()):nullptr;
  if(!player||!visual||!view||!camera_actor){route->failures|=2;return;}
  const auto motion=player->motion();const auto playback=visual->sample();
  if(motion.status!=epok::CharacterMotionStatus::Applied)route->failures|=4;
  if(playback.status!=epok::LocomotionStatus::Applied)route->failures|=8;
  if(camera_actor->root.attach_parent!=player->root.id()||!view->is_current())route->failures|=16;
  route->states|=1u<<unsigned(playback.state);route->jumps+=motion.jumped;
  if(route->capture_start&&route->frame>route->capture_start&&route->frame<=route->capture_start+20){
   using G=epok::GameplayInputLibrary;using A=epok::GameplayAction;
   if(!G::captured(A::MoveX,0)||!G::captured(A::MoveY,0)||!G::captured(A::Jump,0))route->failures|=32;
   if(!motion.grounded&&motion.velocity.y>route->previous_y)++route->gravity_steps;
  }
  route->previous_y=motion.velocity.y;
  route->travel+=uint32_t(epok::MathLibrary::length2(motion.displacement.x,motion.displacement.y).raw());
  const uint32_t words[]={uint32_t(player->root.transform.position[0].raw()),uint32_t(player->root.transform.position[1].raw()),
   uint32_t(motion.velocity.x.raw()),uint32_t(motion.velocity.y.raw()),uint32_t(motion.displacement.x.raw()),uint32_t(motion.displacement.y.raw()),
   motion.grounded,motion.jumped,uint32_t(playback.state),uint32_t(playback.clip)};
  for(auto word:words)route->hash=(route->hash^word)*16777619u;
  if(route->frame==30)route->initial_camera=view->position[0].raw();
  if(route->frame==240){
   if(!view->initialized||view->position[0].raw()==route->initial_camera||!route->travel)route->failures|=64;
   const epok::ObjectId coins[]={COIN_NAMES};uint32_t collected=0;
   for(auto id:coins){auto* coin=registry->resolve<epok::Actor>(id);if(coin&&!epok::is_active(coin))++collected;}
   auto* p=platformer_foundation_probe;p[0]=MAGIC+EPOK_LUA_MODE;p[1]=240;p[2]=route->failures;p[3]=route->states;
   p[4]=route->jumps;p[5]=route->hash;p[6]=route->gravity_steps;p[7]=collected;
  }
 }
};
'''


def verify(out, flavor, modes, prepare_only=False, standalone_only=False):
    out = out / flavor
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"

    def run(*args, label="command.log"):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / label).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if standalone_only:
        export_probe(out, project, run, ("character2d.hpp", "character_input.hpp", "character_animation.hpp", "world2d_components.hpp"))
        return
    if not project.exists():
        run("--create-project", project, "--template", "platformer-2d", "--gameplay", flavor, "--name", "Platformer Foundation " + flavor)
    magic = 0x51000000 | (uuid.uuid4().int & 0xffffff)
    source = SOURCE.replace("DRIVER", DRIVER).replace("REPORTER", REPORTER).replace("MAGIC", str(magic))
    source = source.replace("COIN_PROPERTIES", "\n".join(f" EPOK_PROPERTY(EditAnywhere) epok::ObjectId coin{i}{{}};" for i in range(8)))
    source = source.replace("COIN_NAMES", ",".join(f"coin{i}" for i in range(8)))
    write(project / "assets/scripts/PlatformerRoute.hpp", source)
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["actors"] = [a for a in scene["actors"] if a["class"].get("class_id") != DRIVER]
    roles = platformer_roles(project, scene, flavor)
    player = roles[0] if roles else next(a for a in scene["actors"] if a["class"]["name"] == "epok::Character2D")
    animation = next(c for c in player["components"] if c["class"]["name"] == "epok::SpriteLocomotion2DComponent")
    camera = roles[1] if roles else next(a for a in scene["actors"] if any(c["class"]["name"] == "epok::Camera2DComponent" for c in a["components"]))
    camera_component = next(c for c in camera["components"] if c["class"]["name"] == "epok::Camera2DComponent")
    coins = roles[2] if roles else [a for a in scene["actors"] if any(c["class"]["name"] == "CoinPickup" for c in a["components"])]
    assert len(coins) == 8 and camera["attach"]["actor"] == player["id"]
    root_class = next(c["class"] for c in player["components"] if c.get("root"))
    driver = {"id": str(uuid.uuid4()), "name": "Route Input", "class": {"name": "PlatformerRouteInput", "class_id": DRIVER},
              "components": [{"id": str(uuid.uuid4()), "name": "Root", "root": True, "class": root_class,
                              "properties": {"position": [0, 0]}, "overrides": ["position"]}],
              "properties": {"player": player["id"]}, "overrides": ["player"]}
    scene["actors"].insert(0, driver)
    player["name"] = "Renamed player"
    camera["name"] = "Renamed camera"
    player["components"] = [c for c in player["components"] if c["class"].get("class_id") != REPORTER]
    props = {"driver": driver["id"], "animation": animation["id"], "camera": camera_component["id"],
             **{f"coin{i}": coin["id"] for i, coin in enumerate(coins)}}
    player["components"].append({"id": str(uuid.uuid4()), "name": "Route Reporter", "class": {"name": "PlatformerRouteReporter", "class_id": REPORTER},
                                 "properties": props, "overrides": list(props)})
    write(scene_path, scene)
    if prepare_only:
        print("PASS", flavor, "renamed platformer route prepared", flush=True)
        return

    def check(values, expected):
        assert values[:3] == [expected, 240, 0], values
        assert values[3] == 3 and values[4] > 0 and values[6] > 0 and values[7] > 0, values
    run_modes(out, project, flavor, modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",)),
              "platformer_foundation_probe", magic, check, provider_count=1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/platformer-foundation")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--standalone-only", action="store_true")
    args = parser.parse_args()
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.out, flavor, args.mode, args.prepare_only, args.standalone_only)
    if not (args.prepare_only or args.standalone_only or args.flavor or args.mode):
        rows = [row for flavor in ("cpp", "blueprint", "lua") for row in json.loads((args.out / flavor / "result.json").read_text())]
        assert len(rows) == 5 and len({tuple(row["sample"][1:]) for row in rows}) == 1, rows
        print("PASS five platformer cases have identical semantic traces and coin results", flush=True)
