"""Check default lifecycle events and dead-node/reroute compilation through the CLI."""
import argparse
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import uuid

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools"))
import epok_documents as documents


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--editor", type=Path, default=REPO / "target/release/epok-editor.exe")
    parser.add_argument("--build-psx", action="store_true")
    args = parser.parse_args()
    editor = args.editor.resolve()
    root = Path(tempfile.mkdtemp(prefix="epok-bp-connectivity-")) / "Connectivity"

    def run(*arguments, ok=True):
        result = subprocess.run([str(editor), *map(str, arguments)], cwd=REPO,
                                capture_output=True, text=True, encoding="utf-8", errors="replace",
                                timeout=240, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        assert (result.returncode == 0) == ok, result.stdout + result.stderr
        return result.stdout

    def node(kind, **fields):
        return {"id": str(uuid.uuid4()), "kind": {"kind": kind, **fields}, "inputs": {}, "outputs": {}}

    run("--create-project", root, "--name", "Blueprint Connectivity")
    path = Path(run("--project", root, "--new-blueprint", "BP_Connectivity").strip())
    bp = documents.loads(path.read_text(encoding="utf-8"))
    assert [graph["name"] for graph in bp["functions"]] == ["start", "update", "on_trigger"]
    update = bp["functions"][1]
    variable = str(uuid.uuid4())
    bp["variables"].append({"id": variable, "name": "score", "value_type": {"kind": "fixed"},
                            "default": 0, "editable": True})
    knot = node("reroute")
    data = node("reroute")
    data["inputs"]["value"] = {"kind": "literal", "value_type": {"kind": "fixed"}, "value": 7}
    setter = node("set_variable", member=variable)
    setter["inputs"]["value"] = {"kind": "link", "node": data["id"], "pin": "value"}
    update["nodes"][0]["outputs"]["next"] = [knot["id"]]
    knot["outputs"]["next"] = [setter["id"]]
    update["nodes"].extend([knot, data, setter])
    missing = str(uuid.uuid4())
    dead = [node("branch"), node("call_on", **{"class": missing, "function": missing}),
            node("builtin", operation={"kind": "play_timeline_asset", "asset": missing})]
    update["nodes"].extend(dead)
    bp["layout"]["positions"].update({knot["id"]: [290, 255], data["id"]: [290, 315],
                                       setter["id"]: [430, 220]})
    for index, item in enumerate(dead):
        bp["layout"]["positions"][item["id"]] = [700, 170 * index]
    documents.write_text(path, json.dumps(bp, indent=2), encoding="utf-8")
    before = path.read_bytes()
    run("--project", root, "--compile-blueprints")
    assert path.read_bytes() == before, "Compilation modified the authored document"
    broken = copy.deepcopy(bp)
    broken["functions"][1]["nodes"][0]["outputs"]["next"] = [dead[0]["id"]]
    documents.write_text(path, json.dumps(broken), encoding="utf-8")
    run("--project", root, "--compile-blueprints", ok=False)
    path.write_bytes(before)
    if args.build_psx:
        scene_path = root / "assets/scenes/Main.epokmap"
        scene = documents.loads(scene_path.read_text(encoding="utf-8"))
        entity = copy.deepcopy(scene["entities"][0])
        entity.update(id=str(uuid.uuid4()), name="Blueprint Cube", kind="Mesh",
                      position=[0, 0, 0], scale=[1, 1, 1])
        scene["entities"].append(entity)
        entity["script"] = {"name": bp["name"], "class_id": bp["id"],
                            "provider": {"id": "blueprint", "version": 1},
                            "backend": {"id": "native", "version": 1}, "properties": {}}
        entity["collider"] = {"trigger": True}
        documents.write_text(scene_path, json.dumps(scene), encoding="utf-8")
        run("--project", root, "--build-psx")
        assert (root / ".epok/build/epok.ps-exe").is_file()
    print(json.dumps({"passed": True, "project": str(root), "blueprint": str(path),
                      "psx_build": args.build_psx}, indent=2), flush=True)


if __name__ == "__main__":
    main()
