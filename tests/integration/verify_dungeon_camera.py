"""Compare real Complete camera/movement traces under identical logical inputs.

Copies two authored projects into disposable fixtures. An Input-group component
supplies pad samples and one reused guard actor is the stationary target; a PostPhysics observer records the
actual controllers' output. No controller or camera rule is replaced by the test.
The source projects and their save cards are never modified.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import uuid

from gameplay_probe import run_modes
from verify_camera_rigs import ROOT, EDITOR, documents, write

NAMESPACE = uuid.UUID("ed1868d1-1ec6-5e06-8a0c-424c332cd12b")
DRIVER = str(uuid.uuid5(NAMESPACE, "input-component-class"))
REPORTER = str(uuid.uuid5(NAMESPACE, "reporter-class"))
MAGIC = 0x43414D50
FRAMES = 180
WORDS = 19
TRACE_CAPACITY = 32

SOURCE = r'''#pragma once
#include "epok.hpp"
#include "DungeonCombat/DungeonGame.hpp"
extern "C" {
inline uint32_t dungeon_camera_probe[8]={};
inline uint32_t dungeon_camera_trace[TRACE_CAPACITY*19]={};
inline volatile uint32_t dungeon_camera_frames=0;
}
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="DRIVER_ID") CompleteCameraInput:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::Input;
 uint32_t frame=0,failures=0,locked=0,aimed=0,normal=0,hash=2166136261u;
 void tick(epok::Fixed)override{
  if(frame>=180)return;
  auto* game=DungeonGame::current;
  uint16_t buttons=0;uint8_t rx=128,ry=128;
  if(game&&game->controls_enabled()){
   const unsigned n=frame++;
   if(n>=10&&n<25)buttons|=1u<<unsigned(epok::Button::Up);
   if(n>=50&&n<60)buttons|=1u<<unsigned(epok::Button::Right);
   if(n==30||n==70||n==120||n==135)buttons|=1u<<unsigned(epok::Button::R1);
   if(n>=70&&n<105)buttons|=1u<<unsigned(epok::Button::L1);
   if(n==150)buttons|=1u<<unsigned(epok::Button::Cross);
   if((n>=8&&n<20)||(n>=72&&n<92)){rx=180;ry=140;}
  }
  epok::input.sample(0,true,buttons,true,128,128,rx,ry);
  epok::gameplay_controls().begin_frame();epok::input.begin_tick();epok::gameplay_controls().begin_tick();
 }
};
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="REPORTER_ID") CompleteCameraReporter:public epok::ActorComponent {
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere,ComponentClass="CompleteCameraInput") epok::ObjectId driver{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId camera{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId marker{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::UIActor") epok::ObjectId reticle{};
 void tick(epok::Fixed)override{
  auto* registry=epok::active_object_registry;
  auto* route=registry?registry->resolve<CompleteCameraInput>(driver):nullptr;
  if(!route||!route->frame||dungeon_camera_probe[0])return;
  auto* view=registry->resolve<epok::Actor3D>(camera);
  auto* lock=registry->resolve<epok::Actor3D>(marker);
  auto* aim=registry->resolve<epok::UIActor>(reticle);
  auto* player=get_owner();auto* game=DungeonGame::current;
  if(!player||!view||!lock||!aim||!game){route->failures|=1;return;}
  const auto* body=player->data();const auto* camera_data=view->data();const auto* marker_data=lock->data();
  if(epok::active_camera().get()!=camera_data)route->failures|=2;
  const bool selected=epok::is_active(marker_data);
  route->locked+=selected;route->aimed+=game->aiming;route->normal+=!selected&&!game->aiming;
  if(game->aiming&&selected)route->failures|=4;
  uint32_t* const stored=dungeon_camera_trace+((route->frame-1)%TRACE_CAPACITY)*19;
  uint32_t* row=stored;
  *row++=route->frame-1;
  for(const auto* data:{body,camera_data}){
   for(auto value:data->transform.position)*row++=uint32_t(value.raw());
   for(auto value:data->transform.rotation)*row++=uint32_t(value.raw());
  }
  *row++=game->aiming;*row++=selected;
  for(auto value:marker_data->transform.position)*row++=uint32_t(value.raw());
  *row++=aim->data()->text.enabled;
  for(unsigned k=0;k<19;++k)route->hash=(route->hash^stored[k])*16777619u;
  asm volatile("":::"memory");
  dungeon_camera_frames=route->frame;
  if(route->frame==180){
   auto* p=dungeon_camera_probe;p[1]=180;p[2]=route->failures;p[3]=route->locked;p[4]=route->aimed;
   p[5]=route->normal;p[6]=route->hash;p[7]=game->controls_enabled();p[0]=MAGIC_VALUE;
  }
 }
};
'''


def identity(name):
    return str(uuid.uuid5(NAMESPACE, name))


def prepare(source, out):
    project = out / "project"
    assert not project.exists(), f"Use a fresh output; preserve earlier evidence: {project}"
    shutil.copytree(source, project, ignore=shutil.ignore_patterns(".epok", "exports"))
    write(project / "assets/scripts/CompleteCameraProbe.hpp", SOURCE.replace("DRIVER_ID", DRIVER).replace("REPORTER_ID", REPORTER).replace("MAGIC_VALUE", str(MAGIC)).replace("TRACE_CAPACITY", str(TRACE_CAPACITY)))
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    by_name = {a["name"]: a for a in scene["actors"]}
    player = by_name["Player Aqua"]
    # Keep the jump trial outside the lobby terminals' ordinary Cross interaction
    # radius. The source controller and its interface policy remain unchanged.
    next(c for c in player["components"] if c.get("root"))["properties"]["position"] = [0, 0, 0]
    driver = {"id": identity("input-component"), "name": "Camera Route Input",
              "class": {"name": "CompleteCameraInput", "class_id": DRIVER}, "properties": {}, "overrides": []}
    by_name["Station Collision"]["components"].append(driver)
    # The route stays outside combat. Reuse an inactive resident render actor
    # for the same stationary target instead of allocating another ActorData.
    # Normal guard/boss acceptance runs the untouched full-game scene separately.
    target = by_name["Target Guard 1"]
    target["name"] = "Target Camera Probe"
    target["active"] = True
    for component in target["components"]:
        if component.get("root"):
            component["properties"]["position"] = [0, 0, 6]
    refs = {"driver": driver["id"], "camera": by_name["Follow Camera"]["id"],
            "marker": by_name["Lock-on Marker"]["id"], "reticle": by_name["Aim Reticle"]["id"]}
    player["components"].append({"id": identity("reporter"), "name": "Camera Route Reporter",
                                 "class": {"name": "CompleteCameraReporter", "class_id": REPORTER},
                                 "properties": refs, "overrides": list(refs)})
    assert len(scene["actors"]) <= 64 and all(len(a.get("components", [])) <= 8 for a in scene["actors"])
    write(scene_path, scene)
    authored = {str(p.relative_to(source)): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in source.rglob("*") if p.is_file() and not {".epok", "exports"}.intersection(p.relative_to(source).parts)}
    (out / "source-inventory.json").write_text(json.dumps(authored, indent=2) + "\n", encoding="utf-8")
    return project


def validate_trace(words):
    assert len(words) == FRAMES * WORDS
    rows = [words[i:i + WORDS] for i in range(0, len(words), WORDS)]
    assert [row[0] for row in rows] == list(range(FRAMES))
    for n, row in enumerate(rows):
        assert row[13] == (70 <= n < 105), (n, "free aim", row[13])
        assert row[14] == (30 <= n < 70 or 120 <= n < 135), (n, "lock-on", row[14])
    assert len({row[11] for row in rows[8:20]}) > 1
    assert len({row[11] for row in rows[72:92]}) > 1
    assert rows[0][1:4] != rows[25][1:4]
    signed = lambda n: n - 2**32 if n >= 2**31 else n
    assert max(signed(row[2]) for row in rows[150:]) > signed(rows[149][2]) + 4096, "Jump trial must actually leave the ground"
    return rows


def main():
    parser = argparse.ArgumentParser()
    baseline = parser.add_mutually_exclusive_group(required=True)
    baseline.add_argument("--baseline", type=Path)
    baseline.add_argument("--baseline-trace", type=Path,
                          help="Reuse a preserved accepted trace without replaying its source project")
    parser.add_argument("--migrated", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--prepare-only", action="store_true")
    args = parser.parse_args()
    outputs = []
    cases = [("migrated", args.migrated)]
    if args.baseline_trace:
        path = args.baseline_trace.resolve(strict=True)
        words = json.loads(path.read_text(encoding="utf-8"))
        validate_trace(words)
        outputs.append(words)
        args.out.mkdir(parents=True, exist_ok=True)
        (args.out / "baseline-reuse.json").write_text(json.dumps({
            "trace": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "words": len(words), "classification": "Frozen expected behavior; source project not replayed",
        }, indent=2) + "\n", encoding="utf-8")
    else:
        cases.insert(0, ("baseline", args.baseline))
    for name, source in cases:
        out = args.out.resolve() / name
        out.mkdir(parents=True, exist_ok=True)
        project = prepare(source.resolve(strict=True), out)
        if args.prepare_only:
            continue

        def check(values, expected):
            assert values[:3] == [expected, FRAMES, 0] and values[7] == 1, values
            assert values[3] >= 30 and values[4] == 35 and values[5] >= 30, values

        run_modes(out, project, name, ("native_cpp",), "dungeon_camera_probe", MAGIC, check,
                  provider_count=1, trace=("dungeon_camera_trace", FRAMES * WORDS),
                  trace_ring=("dungeon_camera_frames", TRACE_CAPACITY, WORDS))
        words = json.loads((out / "native_cpp-dungeon_camera_trace.json").read_text(encoding="utf-8"))
        validate_trace(words)
        outputs.append(words)
    if args.prepare_only:
        print("Prepared requested disposable camera routes", flush=True)
        return
    differences = [{"frame": i // WORDS, "word": i % WORDS, "baseline": a, "migrated": b}
                   for i, (a, b) in enumerate(zip(*outputs)) if a != b]
    result = {"matched": not differences, "frames": FRAMES, "words_per_frame": WORDS,
              "editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "differences": differences,
              "scope": "Actual Complete controllers; normal, lock-on, simultaneous L1/R1, free aim, release, reacquisition and jump",
              "limit": "Behavior trace; no frame-time or all-view visual acceptance claim"}
    (args.out / "comparison.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    assert not differences, differences[:20]
    print("PASS", FRAMES * WORDS, "exact Q12/state trace words", flush=True)


if __name__ == "__main__":
    main()
