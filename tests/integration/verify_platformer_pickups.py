"""Real selected-language coin policy: initiator checks, once-only state and reset."""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes

PROBE = "061e2973-0830-5447-a720-172e7d7bfe8f"
REPORTER = "bde92c25-ee83-590d-8087-542c5e5107a1"
SOURCE = r'''
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t platformer_pickup_probe[8]={};}
class EPOK_CLASS(Blueprintable,Owners=World2D,Id="REPORTER") PlatformerPickupReporter:public epok::ActorComponent {
    uint32_t frame=0;
public:
    static constexpr epok::TickGroup tick_group=epok::TickGroup::PostPhysics;
    void tick(epok::Fixed)override{
        if(++frame!=3)return;
        auto* marker=static_cast<epok::Actor2D*>(get_owner());
        platformer_pickup_probe[0]=MAGIC+EPOK_LUA_MODE;
        platformer_pickup_probe[1]=frame;
        platformer_pickup_probe[2]=marker->root.position_x().raw()/4096;
        platformer_pickup_probe[3]=marker->root.position_y().raw()/4096;
    }
};
'''


def verify(out, flavor, modes):
    out = out/flavor
    out.mkdir(parents=True, exist_ok=True)
    project = out/"project"

    def run(*args):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=900)
        assert result.returncode == 0, result.stdout+result.stderr
    if not project.exists():
        run("--create-project", project, "--template", "platformer-2d", "--gameplay", flavor)
    magic = 0x65000000 | (uuid.uuid4().int & 0xffffff)
    source = (ROOT/"tests/fixtures/native-scenes/PlatformerPickupPolicyProbe.hpp").read_text()
    write(project/"assets/scripts/PlatformerPickupPolicyProbe.hpp", source+SOURCE.replace("REPORTER", REPORTER).replace("MAGIC", str(magic)))
    installed = documents.loads((project/"ProjectSettings/Template.epoksettings").read_text())
    assert installed["template"] == "platformer-2d" and installed["gameplay"] == flavor
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text())
    path = project/manifest["startup_scene"]
    scene = documents.loads(path.read_text())
    scene["actors"] = [a for a in scene["actors"] if not any(c["class"].get("class_id") == PROBE for c in a["components"])]
    player = next(a for a in scene["actors"] if a["class"]["class_id"] == installed["roles"]["player"]["class_id"])
    pickup = next(c for a in scene["actors"] for c in a["components"] if c["class"]["class_id"] == installed["roles"]["pickup"]["class_id"])
    for i, actor in enumerate(scene["actors"]):
        actor["name"] = f"Renamed platformer actor {i}"
    root_component = next(c["class"] for c in player["components"] if c.get("root"))
    actor_class = next(a["class"] for a in scene["actors"] if len(a["components"]) == 1 and a["class"]["name"] == "epok::Actor2D")

    def component(name, identity, props):
        return {"id": str(uuid.uuid4()), "name": name, "class": {"name": name, "class_id": identity}, "properties": props, "overrides": list(props)}
    scene["actors"].append({"id": str(uuid.uuid4()), "name": "Policy observation", "class": actor_class, "components": [
        {"id": str(uuid.uuid4()), "name": "Root", "root": True, "class": root_component, "properties": {"position": [0, 0]}, "overrides": ["position"]},
        component("PlatformerPickupPolicyProbe", PROBE, {"player": player["id"], "pickup": pickup["id"]}),
        component("PlatformerPickupReporter", REPORTER, {})]})
    write(path, scene)

    def check(values, expected):
        assert values == [expected, 3, 11, 11, 0, 0, 0, 0], values
    run_modes(out, project, flavor, modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",)),
              "platformer_pickup_probe", magic, check, provider_count=1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT/"artifacts/gameplay-helpers/p7/platformer-pickups")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    args = parser.parse_args()
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.out, flavor, args.mode)
    if not (args.flavor or args.mode):
        rows = [row for flavor in ("cpp", "blueprint", "lua") for row in json.loads((args.out/flavor/"result.json").read_text())]
        assert len(rows) == 5 and len({tuple(row["sample"][1:]) for row in rows}) == 1
        print("PASS five coin-policy cases, eleven checks each", flush=True)
