"""Run Basic and Sample's actual selected-flavor classes and inherited defaults."""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes, export_probe

PROBE = "499e691a-f1ec-589c-92da-7f3f7793ebd1"
DERIVED = "122cb8ca-86c5-5e0c-8829-24003dc45121"
SOURCE = r'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t small_starter_probe[8]={};}
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="PROBE_ID") SmallStarterProbe:public epok::ActorComponent {
 uint32_t frame=0,fail=0,hash=2166136261u;
public:
 static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId primary{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId secondary{};
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::ActorComponent") epok::ObjectId inherited{};
 void tick(epok::Fixed)override{
  if(frame>=120)return;
  auto* registry=epok::active_object_registry;
  auto* first=registry?registry->resolve<epok::Actor3D>(primary):nullptr;
  auto* second=registry?registry->resolve<epok::Actor3D>(secondary):nullptr;
  auto* component=registry?registry->resolve<epok::ActorComponent>(inherited):nullptr;
  if(!first||!first->data()){fail|=1;return;}
  const auto& transform=first->data()->transform;
  const int32_t a=transform.rotation[1].raw();
  const int32_t b=second&&second->data()?second->data()->transform.rotation[1].raw():0;
  uint32_t primary_ok=0,inherited_ok=0;
  if constexpr(IS_SAMPLE){
   primary_ok=first->class_id()==epok::Actor3D::static_class_id;
   inherited_ok=component&&component->class_id()==epok::detail::compact_class_id("DERIVED_ID");
   if(!second||!inherited_ok||component->owner_id()!=secondary)fail|=2;
   if(b!=a*2||transform.rotation[0].raw()||transform.rotation[2].raw())fail|=4;
  }else{
   primary_ok=first->class_id()==epok::detail::compact_class_id("ROOT_ID");
   inherited_ok=first->root_id().valid();
   for(unsigned i=0;i<3;++i)if(transform.position[i].raw()||transform.rotation[i].raw()||transform.scale[i]!=epok::Fixed(1.0))fail|=8;
  }
  if(!primary_ok||!inherited_ok)fail|=16;
  for(unsigned i=0;i<3;++i){hash=(hash^uint32_t(transform.position[i].raw()))*16777619u;hash=(hash^uint32_t(transform.rotation[i].raw()))*16777619u;}
  hash=(hash^uint32_t(b))*16777619u;
  if(++frame==120){
   if constexpr(IS_SAMPLE)if(a<60*4096||a>130*4096)fail|=32;
   auto* out=small_starter_probe;out[0]=MAGIC+EPOK_LUA_MODE;out[1]=frame;out[2]=fail;
   out[3]=uint32_t(a);out[4]=uint32_t(b);out[5]=hash;out[6]=primary_ok;out[7]=inherited_ok;
  }
 }
};
'''


def verify(out, template, flavor, modes, prepare_only, export):
    out = out / template / flavor
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    count = 0

    def run(*args, label=None):
        nonlocal count
        count += 1
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / (label or f"command-{count}.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if not project.exists():
        run("--create-project", project, "--template", template, "--gameplay", flavor, "--name", f"Small {template} {flavor}")
    installed = documents.loads((project / "ProjectSettings/Template.epoksettings").read_text(encoding="utf-8"))
    assert installed["template"] == template and installed["gameplay"] == flavor
    role = installed["roles"]["spinner" if template == "sample" else "game_root"]
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    # Repeat preparation preserves the original generated actors and removes only
    # this fixture's instrumentation and authored derived component.
    for actor in scene["actors"]:
        actor["components"] = [c for c in actor["components"] if c["class"].get("class_id") not in (PROBE, DERIVED)]
    if template == "sample":
        primary, original = next((a, c) for a in scene["actors"] for c in a["components"] if c["class"].get("class_id") == role["class_id"])
        secondary = next(a for a in scene["actors"] if a["id"] != primary["id"] and any(c["class"]["name"] == "epok::Mesh3DComponent" for c in a["components"]))
        if flavor == "cpp":
            write(project / "assets/scripts/FastSpinner.hpp", '#pragma once\n#include "Spinner.hpp"\n'
                  f'class EPOK_CLASS(Blueprintable,Owners=World3D,Id="{DERIVED}") FastSpinner:public Spinner {{}};\n')
        elif flavor == "blueprint":
            path = project / "assets/Blueprints/FastSpinner.epokbp"
            if not path.exists():
                run("--project", project, "--new-blueprint", "FastSpinner", "--parent", role["class_id"])
            asset = documents.loads(path.read_text(encoding="utf-8"))
            asset["id"] = DERIVED
            write(path, asset)
        else:
            write(project / "assets/scripts/FastSpinner.lua", '---@class FastSpinner : Spinner\nlocal FastSpinner = Spinner:extend()\nreturn FastSpinner\n')
            identities_path = project / "ProjectSettings/LuaClasses.epoksettings"
            identities = documents.loads(identities_path.read_text(encoding="utf-8"))
            identities["classes"]["assets/scripts/FastSpinner.lua"] = DERIVED
            write(identities_path, identities)
        inherited = {"id": str(uuid.uuid4()), "name": "Faster inherited behavior", "class": {"name": "FastSpinner", "class_id": DERIVED}, "properties": {"speed": 90}, "overrides": ["speed"]}
        secondary["components"].append(inherited)
        assert original.get("properties", {}).get("speed", 45) == 45
    else:
        assert len(scene["actors"]) == 2
        primary = next(a for a in scene["actors"] if a["class"].get("class_id") == role["class_id"])
        assert len(primary["components"]) == 1 and primary["components"][0].get("root")
        assert not any(c["class"]["name"] == "epok::Mesh3DComponent" for a in scene["actors"] for c in a["components"])
        secondary, inherited = None, None
    # Renaming does not alter any class, component or scene identity binding.
    for index, actor in enumerate(scene["actors"]):
        actor["name"] = f"Renamed actor {index}"
    camera = next(a for a in scene["actors"] if any(c["class"]["name"] == "epok::Camera3DComponent" for c in a["components"]))
    props = {"primary": primary["id"], "secondary": secondary["id"] if secondary else None, "inherited": inherited["id"] if inherited else None}
    camera["components"].append({"id": str(uuid.uuid4()), "name": "Observation", "class": {"name": "SmallStarterProbe", "class_id": PROBE}, "properties": props, "overrides": list(props)})
    magic = 0x5C000000 | (uuid.uuid4().int & 0xffffff)
    source = SOURCE.replace("PROBE_ID", PROBE).replace("DERIVED_ID", DERIVED).replace("ROOT_ID", role["class_id"]).replace("IS_SAMPLE", "true" if template == "sample" else "false").replace("MAGIC", str(magic))
    write(project / "assets/scripts/SmallStarterProbe.hpp", source)
    write(scene_path, scene)
    if prepare_only:
        run("--project", project, "--compile-blueprints")
        return

    def check(values, expected):
        assert values[:3] == [expected, 120, 0] and values[6:] == [1, 1], values
        if template == "basic":
            assert values[3:5] == [0, 0], values
        else:
            assert 60*4096 < values[3] < 130*4096 and values[4] == values[3]*2, values

    run_modes(out, project, f"{template}-{flavor}", modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",)), "small_starter_probe", magic, check, provider_count=1)
    flags = (project / ".epok/build/sources.mk").read_text(encoding="utf-8")
    for flag in ("-DEPOK_INTERACTION", "-DEPOK_GAMEPLAY_CONTROL", "-DEPOK_WORLD_GAMEPLAY", "-DEPOK_DIALOGUE", "-DEPOK_CHARACTER"):
        assert flag not in flags, flag
    if export:
        export_probe(out, project, run, ("object_model.hpp", "actor_tables.hpp"))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p7/small-starters")
    parser.add_argument("--template", choices=("basic", "sample"), action="append")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--export", action="store_true")
    args = parser.parse_args()
    for template in args.template or ("basic", "sample"):
        for flavor in args.flavor or ("cpp", "blueprint", "lua"):
            verify(args.out, template, flavor, args.mode, args.prepare_only, args.export)
    if not (args.template or args.flavor or args.mode or args.prepare_only):
        for template in ("basic", "sample"):
            rows = [row for flavor in ("cpp", "blueprint", "lua") for row in json.loads((args.out/template/flavor/"result.json").read_text())]
            assert len(rows) == 5 and len({tuple(row["sample"][1:]) for row in rows}) == 1, rows
        print("PASS both templates have identical five-mode semantic traces", flush=True)
