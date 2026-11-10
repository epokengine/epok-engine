"""Run generated starter gameplay over the shared character/camera components.

The native fixture supplies deterministic pad samples and observes the real
template input subclass, motor, animation bank and camera. It supplies no game
movement or animation policy. Compare the five authoring/execution cases.
"""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes, export_probe
from starter_bindings import player_roles

DRIVER = "c2821d24-c27b-5175-b190-90cec12316de"
REPORTER = "03735d01-6088-57f6-b397-5a28a26e4bb5"

SOURCE = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" { inline uint32_t third_person_foundation_probe[8]={}; }
class EPOK_CLASS(Blueprintable,Id="DRIVER") FoundationRouteInput:public epok::Actor3D {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::Input;
 uint32_t frame=0,failures=0,hash=2166136261u,states=0,jumps=0,captured_fall=0,travel=0;
 uint32_t capture_start=0;
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Character3D") epok::ObjectId player{};
 epok::Fixed previous_y=0.0;
 epok::InputCapture capture{};
 void tick(epok::Fixed)override{
  if(frame>=240)return;
  using G=epok::GameplayInputLibrary;using A=epok::GameplayAction;
  auto* character=epok::active_object_registry->resolve<epok::Character3D>(player);
  if(!capture_start&&character&&character->motion().jumped){
   capture=G::capture(id(),G::mask(A::MoveX)|G::mask(A::MoveY)|G::mask(A::Jump),100,0);capture_start=frame;
  }
  if(capture_start&&frame==capture_start+20){if(!G::release(capture))failures|=1;capture={};}
  uint16_t buttons=0;
  if((frame>=10&&frame<80)||(frame>=110&&frame<160))buttons|=1u<<unsigned(epok::Button::Up);
  if((frame>=80&&frame<110)||(frame>=160&&frame<190))buttons|=1u<<unsigned(epok::Button::Right);
  if(frame==25||frame==125)buttons|=1u<<unsigned(epok::Button::Cross);
  epok::input.sample(0,true,buttons,true,128,128,frame>=40&&frame<60?200:128,128);
  epok::gameplay_controls().begin_frame();epok::input.begin_tick();epok::gameplay_controls().begin_tick();
  ++frame;
 }
};
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="REPORTER") FoundationRouteReporter:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere,ActorClass="FoundationRouteInput") epok::ObjectId driver{};
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::SkeletalLocomotionComponent") epok::ObjectId animation{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::CameraRig3D") epok::ObjectId camera{};
 void tick(epok::Fixed)override{
  auto* registry=epok::active_object_registry;
  auto* route=registry?registry->resolve<FoundationRouteInput>(driver):nullptr;
  if(!route||!route->frame||route->frame>240||third_person_foundation_probe[0])return;
  auto* player=registry->resolve<epok::Character3D>(owner_id());
  auto* visual=registry->resolve<epok::SkeletalLocomotionComponent>(animation);
  auto* rig=registry->resolve<epok::CameraRig3D>(camera);
  if(!player||!visual||!rig){route->failures|=2;return;}
  const auto motion=player->motion();const auto playback=visual->sample();const auto view=rig->sample();
  if(motion.status!=epok::CharacterMotionStatus::Applied)route->failures|=4;
  if(playback.status!=epok::LocomotionStatus::Applied)route->failures|=8;
  if(view.status!=epok::CameraRigStatus::Applied||epok::active_camera().get()!=rig->data())route->failures|=16;
  route->states|=1u<<unsigned(playback.state);route->jumps+=motion.jumped;
  if(route->capture_start&&route->frame>route->capture_start&&route->frame<=route->capture_start+20){
   using G=epok::GameplayInputLibrary;using A=epok::GameplayAction;
   if(!G::captured(A::MoveX,0)||!G::captured(A::MoveY,0)||!G::captured(A::Jump,0))route->failures|=32;
   if(!motion.grounded&&motion.velocity.y<route->previous_y)++route->captured_fall;
  }
  route->previous_y=motion.velocity.y;
  route->travel+=uint32_t(epok::MathLibrary::length2(motion.displacement.x,motion.displacement.z).raw());
  const uint32_t words[]={uint32_t(motion.velocity.x.raw()),uint32_t(motion.velocity.y.raw()),uint32_t(motion.velocity.z.raw()),
    uint32_t(player->data()->transform.position[0].raw()),uint32_t(player->data()->transform.position[1].raw()),uint32_t(player->data()->transform.position[2].raw()),
    uint32_t(playback.state),uint32_t(playback.clip),uint32_t(playback.rate.raw()),uint32_t(view.yaw.raw()),uint32_t(view.pitch.raw())};
  for(auto word:words)route->hash=(route->hash^word)*16777619u;
  if(route->frame==240){auto* p=third_person_foundation_probe;p[0]=MAGIC+EPOK_LUA_MODE;p[1]=240;p[2]=route->failures;
   p[3]=route->states;p[4]=route->jumps;p[5]=route->hash;p[6]=route->captured_fall;p[7]=route->travel;}
 }
};
'''


def verify(out, flavor, modes, prepare_only=False, standalone_only=False):
    out = out / flavor
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"

    def run(*args, label="command.log"):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT,
                                capture_output=True, text=True, encoding="utf-8", timeout=900)
        (out / label).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if standalone_only:
        export_probe(out, project, run, ("character3d.hpp", "character_input.hpp", "character_animation.hpp", "camera_rig.hpp"))
        return
    if not project.exists():
        run("--create-project", project, "--template", "third-person", "--gameplay", flavor, "--name", "Foundation " + flavor)
    magic = 0x4F000000 | (uuid.uuid4().int & 0xffffff)
    write(project / "assets/scripts/FoundationRoute.hpp", SOURCE.replace("DRIVER", DRIVER).replace("REPORTER", REPORTER).replace("MAGIC", str(magic)))
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["actors"] = [a for a in scene["actors"] if a["class"].get("class_id") != DRIVER]
    roles = player_roles(project, scene, "third-person", flavor)
    player, camera = roles or (
        next(a for a in scene["actors"] if a["class"]["name"] == "epok::Character3D"),
        next(a for a in scene["actors"] if a["class"]["name"] == "epok::ThirdPersonCamera"))
    assert camera.get("attach") is None
    animation = next(c for c in player["components"] if c["class"]["name"] == "epok::SkeletalLocomotionComponent")
    for actor in (player, camera):
        actor["name"] = "Renamed " + actor["name"]
    root_class = next(c["class"] for c in player["components"] if c.get("root"))
    driver = {"id": str(uuid.uuid4()), "name": "Route Input", "class": {"name": "FoundationRouteInput", "class_id": DRIVER},
              "components": [{"id": str(uuid.uuid4()), "name": "Root", "class": root_class, "root": True,
                              "properties": {"position": [0, 0, 0]}, "overrides": ["position"]}],
              "properties": {"player": player["id"]}, "overrides": ["player"]}
    scene["actors"].insert(0, driver)
    player["components"] = [c for c in player["components"] if c["class"].get("class_id") != REPORTER]
    props = {"driver": driver["id"], "animation": animation["id"], "camera": camera["id"]}
    player["components"].append({"id": str(uuid.uuid4()), "name": "Route Reporter", "class": {"name": "FoundationRouteReporter", "class_id": REPORTER},
                                 "properties": props, "overrides": list(props)})
    write(scene_path, scene)
    if prepare_only:
        print("PASS", flavor, "renamed template route prepared", flush=True)
        return

    def check(values, expected):
        assert values[:3] == [expected, 240, 0], values
        assert values[3] == 63 and values[4] == 2 and values[6] > 0 and values[7] > 0, values
    run_modes(out, project, flavor, modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",)),
              "third_person_foundation_probe", magic, check, provider_count=1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/third-person-foundation")
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
        print("PASS five template execution cases have identical semantic traces", flush=True)
