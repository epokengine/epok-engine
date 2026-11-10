"""Full Platformer plus an ordinary project widget, interaction and conversation."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes
from starter_bindings import platformer_roles

TEXT = "Hello from the meadow."
PROBE = "8b1c0661-2abc-510b-9ba5-c12af10cd6bc"


def package(project, path, kind, value, identity):
    payload = json.dumps(value, ensure_ascii=False).encode("utf-8")
    meta = json.dumps({"version": 2, "id": identity, "kind": kind, "importer_version": 1,
                       "source": path, "source_hash": hashlib.sha256(payload).hexdigest(),
                       "settings": {"type": "Authored"}}).encode("utf-8")
    target = project/path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(b"EPOKAS01"+struct.pack("<II", len(meta), len(payload))+meta+payload)


def verify(out, flavor, modes, prepare_only=False):
    out = out/flavor
    out.mkdir(parents=True, exist_ok=True)
    project = out/"project"

    def run(*args, label):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=900)
        (out/label).write_text(result.stdout+result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout+result.stderr
        return result.stdout.strip()
    if not project.exists():
        run("--create-project", project, "--template", "platformer-2d", "--gameplay", flavor, label="create.log")
    widget_path = {"cpp": "assets/scripts/MeadowDialogue.hpp", "blueprint": "assets/Blueprints/MeadowDialogue.epokbp", "lua": "assets/scripts/MeadowDialogue.lua"}[flavor]
    if not (project/widget_path).exists():
        run("--project", project, "--new-dialogue-widget", "MeadowDialogue", "--presentation", "classic_box", "--gameplay", flavor, label="widget-create.log")
    magic = 0x66000000 | (uuid.uuid4().int & 0xffffff)
    source = (ROOT/"tests/fixtures/native-scenes/PlatformerDialogueProbe.hpp").read_text().replace("PLATFORMER_DIALOGUE_MAGIC", str(magic))
    write(project/"assets/scripts/PlatformerDialogueProbe.hpp", source)
    metadata = json.loads(run("--project", project, "--reflect", label="reflect.json"))
    classes = {c["cpp_name"]: c for c in metadata["classes"]}
    ids = {name: str(uuid.uuid4()) for name in ("table", "conversation", "line", "end", "widget")}
    package(project, "assets/dialogue/MeadowStrings.epokasset", "StringTable",
            {"version": 1, "pageable": True, "languages": ["English"], "entries": [{"key": "hello", "values": [TEXT]}]}, ids["table"])
    package(project, "assets/dialogue/MeadowTalk.epokasset", "Dialogue",
            {"version": 1, "entry": ids["line"], "participants": [], "bindings": [], "nodes": [
                {"id": ids["line"], "kind": {"kind": "line", "speaker": None, "text": {"table": ids["table"], "key": "hello"}, "next": ids["end"], "policy": {"milliseconds_per_glyph": 0, "auto_advance_ms": None}}},
                {"id": ids["end"], "kind": {"kind": "end"}}]}, ids["conversation"])
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text())
    path = project/manifest["startup_scene"]
    scene = documents.loads(path.read_text())
    removed = {a["id"] for a in scene["actors"] if a.get("name") in ("Meadow dialogue widget", "Meadow speaker", "Meadow observer")}
    while True:
        descendants = {a["id"] for a in scene["actors"] if a.get("logical_parent") in removed or (a.get("attach") or {}).get("actor") in removed}
        if descendants <= removed:
            break
        removed |= descendants
    scene["actors"] = [a for a in scene["actors"] if a["id"] not in removed]
    for actor in scene["actors"]:
        temporary = {classes["epok::InteractorComponent"]["id"], classes["epok::DialogueInteractionComponent"]["id"], PROBE}
        actor["components"] = [c for c in actor["components"] if c["class"].get("class_id") not in temporary]
    player, _, _ = platformer_roles(project, scene, flavor)
    player_id = player["id"]
    assert len(scene["actors"]) == 52, "the complete level uses one organizational world root"
    write(path, scene)
    # Use the same factory as the UI designer/catalog. A root-only hand-built
    # actor would omit the widget's authored control actors and their true cost.
    run("--project", project, "--add-actor", "MeadowDialogue:Meadow dialogue widget", label="widget-place.log")
    scene = documents.loads(path.read_text())
    widget = next(a for a in scene["actors"] if a["name"] == "Meadow dialogue widget")
    ids["widget"] = widget["id"]
    player = next(a for a in scene["actors"] if a["id"] == player_id)
    widget_actor_count = len(scene["actors"])-52

    def component(name, class_name, props=None, root=False):
        props = props or {}
        cls = classes[class_name]
        return {"id": str(uuid.uuid4()), "name": name, "class": {"name": class_name, "class_id": cls["id"]}, "properties": props, "overrides": list(props), "root": root}

    interactor = component("Interaction", "epok::InteractorComponent", {"auto_input": False})
    player["components"].append(interactor)
    assert len(player["components"]) == 8
    speaker = next(a for a in scene["actors"] if any(c["class"].get("class_id") == classes["epok::Skeleton2DComponent"]["id"] for c in a["components"]))
    dialogue = component("Dialogue", "epok::DialogueInteractionComponent", {"conversation": ids["conversation"], "presenter": ids["widget"], "automatic_input": False, "priority": 100})
    speaker["components"].append(dialogue)
    observer = next(a for a in scene["actors"] if a["id"] == player["logical_parent"])
    assert len(observer["components"]) == 1
    observer["components"].append(component("Probe", "PlatformerDialogueProbe", {"player": player["id"], "interactor": interactor["id"], "dialogue": dialogue["id"]}))
    assert len(scene["actors"]) == 64 and widget_actor_count == 12
    assert all(len(a["components"]) <= 8 for a in scene["actors"])
    (out/"composition-cost.json").write_text(json.dumps({"starter_actors": 52, "widget_actors": widget_actor_count, "additional_speaker_and_observer_actors": 0, "total": len(scene["actors"])}, indent=2)+"\n", encoding="utf-8")
    write(path, scene)
    if prepare_only:
        print("PASS prepared full Platformer dialogue project", project, flush=True)
        return

    def check(values, expected):
        assert values == [expected, 14, 0, 1, 1, 1, len(TEXT), 0], values
    run_modes(out, project, flavor, modes or (("native_cpp", "vm_bytecode", "vm_source") if flavor == "lua" else ("native_cpp",)),
              "platformer_dialogue_probe", magic, check, provider_count=1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT/"artifacts/gameplay-helpers/p7/platformer-dialogue")
    parser.add_argument("--flavor", choices=("cpp", "blueprint", "lua"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    args = parser.parse_args()
    for flavor in args.flavor or ("cpp", "blueprint", "lua"):
        verify(args.out, flavor, args.mode, args.prepare_only)
    if not (args.flavor or args.mode or args.prepare_only):
        rows = [row for flavor in ("cpp", "blueprint", "lua") for row in json.loads((args.out/flavor/"result.json").read_text())]
        assert len(rows) == 5 and len({tuple(row["sample"][1:]) for row in rows}) == 1
        print("PASS full Platformer interaction and ordinary dialogue widget in five execution cases", flush=True)
