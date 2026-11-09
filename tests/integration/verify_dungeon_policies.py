"""Actual generated project rules: explicit ownership, wrong actors, once, reset."""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes

PROBE = "795a13bd-6614-5b87-a537-a1a658aa2edf"


def verify(out, domain, flavor, modes, prepare_only=False):
    out = out/domain/flavor
    out.mkdir(parents=True, exist_ok=True)
    project = out/"project"

    def run(*args, label):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT,
                                capture_output=True, text=True, encoding="utf-8", timeout=900)
        (out/label).write_text(result.stdout+result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout+result.stderr
        return result.stdout.strip()

    if not project.exists():
        run("--create-project", project, "--template", "dungeon" if domain=="3d" else "basic",
            "--gameplay", flavor, label="create.log")
        if domain=="2d":
            manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
            path = project/manifest["startup_scene"]
            scene = documents.loads(path.read_text(encoding="utf-8"))
            scene["actors"] = []
            write(path, scene)
            run("--project", project, "--add-actor", "epok::Actor2D:Policy Player", label="player.log")
            run("--project", project, "--generate-dungeon", "2d", "--dungeon-seed", "1",
                "--dungeon-settings", json.dumps({"source":"Recipe", "spawn_actor":"Policy Player"}), label="generate.log")
    magic = 0x6b000000 | (uuid.uuid4().int & 0xffffff)
    source = (ROOT/"tests/fixtures/native-scenes/DungeonPolicyProbe.hpp").read_text().replace("DUNGEON_POLICY_MAGIC", str(magic))
    write(project/"assets/scripts/DungeonPolicyProbe.hpp", source)
    metadata = json.loads(run("--project", project, "--reflect", label="reflect.json"))
    classes = {c["cpp_name"]:c for c in metadata["classes"]}
    installed = documents.loads((project/"ProjectSettings/Template.epoksettings").read_text(encoding="utf-8"))
    roles = installed["roles"]
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    path = project/manifest["startup_scene"]
    scene = documents.loads(path.read_text(encoding="utf-8"))
    generator = scene["dungeons"][0]
    player = next(a for a in scene["actors"] if a["id"] == generator["spawn_actor"])
    key = next(a for a in scene["actors"] if any(c["class"].get("class_id")==roles["key"]["class_id"] for c in a["components"]))
    pickup = next(c for c in key["components"] if c["class"].get("class_id")==roles["key"]["class_id"])
    flag = next(c for c in key["components"] if c["id"]==pickup["properties"]["collected_flag"])
    assert flag["class"]["class_id"]==roles["key_state"]["class_id"]
    door = next(c for a in scene["actors"] for c in a["components"] if c["class"].get("class_id")==roles["door"]["class_id"] and c["properties"]["requirement_flag"]==flag["id"])
    collider_property = "collider_3d" if domain=="3d" else "collider_2d"
    assert any(c["id"]==door["properties"][collider_property] for a in scene["actors"] for c in a["components"])
    if domain=="2d" and not any(c["class"].get("class_id")==classes["epok::Camera2DComponent"]["id"] for c in player["components"]):
        camera = classes["epok::Camera2DComponent"]
        player["components"].append({"id":str(uuid.uuid4()),"name":"Camera","class":{"name":camera["cpp_name"],"class_id":camera["id"]},"properties":{},"overrides":[]})
    observer = next(a for a in scene["actors"] if a["id"]==generator["root"])
    props = {"player":player["id"],"pickup":pickup["id"],"flag":flag["id"],"door":door["id"]}
    observer["components"] = [c for c in observer["components"] if c["class"].get("class_id")!=PROBE]
    observer["components"].append({"id":str(uuid.uuid4()),"name":"Policy probe","class":{"name":"DungeonPolicyProbe","class_id":PROBE},"properties":props,"overrides":list(props)})
    for i, actor in enumerate(scene["actors"]):
        actor["name"] = f"Renamed {domain} actor {i}"
    assert all(len(a["components"])<=8 for a in scene["actors"])
    write(path, scene)
    if prepare_only:
        print("PASS prepared",domain,flavor,project,flush=True)
        return

    def check(values, expected):
        assert values==[expected,8,0,1,1,1,1,0], values
    run_modes(out, project, domain+"_"+flavor,
              modes or (("native_cpp","vm_bytecode","vm_source") if flavor=="lua" else ("native_cpp",)),
              "dungeon_policy_probe",magic,check,provider_count=1)


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--out",type=Path,default=ROOT/"artifacts/gameplay-helpers/p7/dungeon-policies")
    parser.add_argument("--domain",choices=("3d","2d"),action="append")
    parser.add_argument("--flavor",choices=("cpp","blueprint","lua"),action="append")
    parser.add_argument("--mode",choices=("native_cpp","vm_bytecode","vm_source"),action="append")
    parser.add_argument("--prepare-only",action="store_true")
    args=parser.parse_args()
    for domain in args.domain or ("3d","2d"):
        for flavor in args.flavor or ("cpp","blueprint","lua"):
            verify(args.out,domain,flavor,args.mode,args.prepare_only)
