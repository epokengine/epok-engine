"""Real SDK reflection, inherited event placeholders and preserved Blueprint wires."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import uuid

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents


def verify(editor, project):
    def run(*args):
        result = subprocess.run([str(editor), *map(str, args)], cwd=ROOT,
                                capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=180)
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout

    run("--create-project", project, "--name", "EventContract", "--template", "sample")
    manifest = json.loads(run("--project", project, "--reflect"))
    native = {c["cpp_name"]: c for c in manifest["classes"]}
    for name in ["epok::Actor", "epok::ActorComponent"]:
        events = {f["name"]: f for f in native[name]["functions"]}
        assert events["tick"]["parameters"][0]["name"] == "delta_seconds"
        assert events["end_play"]["parameters"][0]["name"] == "end_play_reason"
    for cls in manifest["classes"]:
        for function in cls["functions"]:
            for parameter in function["parameters"]:
                assert parameter["name"] and not re.fullmatch(r"(?:arg|param|parameter)\d+", parameter["name"], re.I), (cls["cpp_name"], function)

    run("--project", project, "--new-blueprint", "BP_AC_Parent", "--parent", "epok::ActorComponent")
    parent_path = project / "assets/Blueprints/BP_AC_Parent.epokbp"
    parent = documents.loads(parent_path.read_text(encoding="utf-8"))
    assert [g["name"] for g in parent["functions"]] == ["begin_play", "tick", "end_play"]
    assert all(len(g["nodes"]) == 1 and not g["nodes"][0]["outputs"] for g in parent["functions"])
    run("--project", project, "--new-blueprint", "BP_AC_Child", "--parent", "BP_AC_Parent")
    child_path = project / "assets/Blueprints/BP_AC_Child.epokbp"
    child = documents.loads(child_path.read_text(encoding="utf-8"))

    def generated(asset):
        return (project / f".epok/blueprints/scripts/generated/{asset['id']}.hpp").read_text(encoding="utf-8")

    run("--project", project, "--compile-blueprints")
    for asset in [parent, child]:
        assert all(f"void {name}(" not in generated(asset) for name in ["begin_play", "tick", "end_play"])

    for asset, path, target in [(parent, parent_path, "epok::ActorComponent"), (child, child_path, "BP_AC_Parent")]:
        graph = next(g for g in asset["functions"] if g["name"] == "tick")
        graph["parameters"][0]["name"] = "arg0"  # Old serialized graph binding.
        call = str(uuid.uuid4())
        graph["nodes"][0]["outputs"]["next"] = [call]
        graph["nodes"].append({"id": call, "kind": {"kind": "call_parent"},
                               "inputs": {"arg0": {"kind": "parameter", "name": "arg0"}}, "outputs": {}})
        documents.write_text(path, json.dumps(asset), encoding="utf-8")
        run("--project", project, "--compile-blueprints")
        assert f"{target}::tick(" in generated(asset)

    # A child's saved override UUID must remain valid when its parent is unwired.
    next(g for g in parent["functions"] if g["name"] == "tick")["nodes"][0]["outputs"].clear()
    documents.write_text(parent_path, json.dumps(parent), encoding="utf-8")
    run("--project", project, "--compile-blueprints")
    assert "void tick(" not in generated(parent)
    assert "BP_AC_Parent::tick(" in generated(child)
    run("--project", project, "--build-psx")
    print("PASS: SDK names, disconnected inheritance, explicit parent dispatch, saved arg0 wires, child override IDs and PSX build")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--editor", type=Path, default=ROOT / "target/debug/epok-editor.exe")
    parser.add_argument("--destination", type=Path, help="Retain a new disposable project here")
    args = parser.parse_args()
    if args.destination:
        verify(args.editor.resolve(), args.destination.resolve())
    else:
        with tempfile.TemporaryDirectory(prefix="epok-event-contract-") as directory:
            verify(args.editor.resolve(), Path(directory) / "Game")
