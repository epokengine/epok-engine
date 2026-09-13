"""Create an isolated, compiled native Blueprint canvas fixture for screenshot QA.

Does not launch the GUI or modify an existing project. The printed command opens
the production editor; --screenshot OUTPUT may be appended by the caller.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import argparse
import copy
import json
from pathlib import Path
import subprocess
import tempfile
import uuid

REPO = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--editor", type=Path, default=REPO / "target/debug/epok-editor.exe")
    parser.add_argument("--destination", type=Path)
    args = parser.parse_args()
    editor = args.editor.resolve()
    project = (args.destination or Path(tempfile.mkdtemp(prefix="epok-bp-visual-")) / "Canvas Fixture").resolve()
    if project.exists():
        raise SystemExit("Destination must not exist; no existing project will be overwritten.")

    def run(*arguments):
        result = subprocess.run([str(editor), *map(str, arguments)], cwd=REPO, capture_output=True,
                                text=True, encoding="utf-8", errors="replace", timeout=180,
                                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        return result.stdout

    def write(path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        documents.write_text(path, value if isinstance(value, str) else json.dumps(value, indent=2), encoding="utf-8")

    def identity(name):
        return str(uuid.uuid5(uuid.NAMESPACE_URL, "epok:blueprint-visual-fixture:" + name))

    run("--create-project", project, "--name", "Blueprint Canvas")
    class_id = identity("InteractionBase")
    source = '''#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable, Id="CLASS_ID") InteractionBase : public epok::Behaviour {
public:
    EPOK_PROPERTY(EditAnywhere, Id="ENABLED_ID") bool enabled = true;
    EPOK_PROPERTY(EditAnywhere, Id="PROGRESS_ID") epok::Fixed progress = 0.0;
    EPOK_FUNCTION(BlueprintCallable) void enable_input(bool active) { enabled = active; }
    EPOK_FUNCTION(BlueprintCallable) void interact(epok::Fixed strength) { progress += strength; }
    EPOK_FUNCTION(BlueprintEvent) virtual void on_interaction() {}
    void start(epok::Transform&) override { on_interaction(); }
    void update(epok::Transform&, epok::Fixed) override {}
};
'''.replace("CLASS_ID", class_id).replace("ENABLED_ID", identity("enabled")).replace("PROGRESS_ID", identity("progress"))
    write(project / "assets/scripts/InteractionBase.hpp", source)
    write(project / "assets/scripts/InteractionBase.cpp", '#include "InteractionBase.hpp"\n')
    reflected = json.loads(run("--project", project, "--reflect"))
    native = next(c for c in reflected["classes"] if c["id"] == class_id)
    functions = {f["name"]: f for f in native["functions"]}
    bp_path = Path(run("--project", project, "--new-blueprint", "BP_Interaction", "--parent", class_id).strip().removeprefix("\\\\?\\"))
    bp = documents.loads(bp_path.read_text(encoding="utf-8"))
    nodes, positions = [], {}

    def node(name, kind, position, inputs=None, **settings):
        item = {"id": identity("node:" + name), "kind": {"kind": kind, **settings}, "inputs": inputs or {}, "outputs": {}}
        nodes.append(item)
        positions[item["id"]] = position
        return item

    def link(source):
        return {"kind": "link", "node": source["id"], "pin": "value"}

    def literal(kind, value):
        return {"kind": "literal", "value_type": {"kind": kind}, "value": value}

    def wire(source, target, port="next"):
        source["outputs"].setdefault(port, []).append(target["id"])

    helper_id = identity("function:compute-strength")
    entry = node("entry", "entry", [10, 20])
    enabled = node("enabled", "get_variable", [10, 165], member=identity("enabled"))
    branch = node("branch", "branch", [220, 20], {"condition": link(enabled)})
    enable = node("enable", "call", [420, 20], {"active": literal("bool", True)}, function=functions["enable_input"]["id"])
    delay = node("delay", "delay", [630, 20], {"seconds": literal("fixed", 0.25)})
    compute = node("compute", "call", [420, 220], {"amount": literal("fixed", 1.0)}, function=helper_id)
    interact = node("interact", "call", [630, 220], {"strength": link(compute)}, function=functions["interact"]["id"])
    disable = node("disable", "call", [420, 400], {"active": literal("bool", False)}, function=functions["enable_input"]["id"])
    done = node("done", "return", [800, 220])
    wire(entry, branch)
    wire(branch, enable, "true")
    wire(branch, disable, "false")
    wire(enable, delay)
    wire(delay, compute)
    wire(compute, interact)
    wire(interact, done)
    wire(disable, done)
    event = functions["on_interaction"]
    event_graph = {"id": identity("graph"), "name": event["name"], "override_id": event["id"],
                   "parameters": event["parameters"], "returns": event["returns"], "entry": entry["id"], "nodes": nodes}
    # A real, synchronously called visual function: progress + amount. The event
    # executes it once before consuming its return; no unsupported purity flag.
    nodes = []
    helper_entry = node("helper-entry", "entry", [30, 20])
    progress = node("helper-progress", "get_variable", [30, 155], member=identity("progress"))
    add = node("helper-add", "binary", [260, 135],
               {"a": link(progress), "b": {"kind": "link", "node": helper_entry["id"], "pin": "amount"}}, op="add")
    helper_return = node("helper-return", "return", [480, 20], {"value": link(add)})
    wire(helper_entry, helper_return)
    helper_graph = {"id": helper_id, "name": "compute_strength", "override_id": None,
                    "parameters": [{"name": "amount", "value_type": {"kind": "fixed"}, "direction": "value"}],
                    "returns": {"kind": "fixed"}, "entry": helper_entry["id"], "nodes": nodes}
    bp["functions"] = [event_graph, helper_graph]
    bp["layout"] = {"positions": positions, "comments": {}}
    initial = documents.loads((project / "assets/scenes/Main.epokmap").read_text(encoding="utf-8"))
    prototype = next((entity for entity in initial["entities"] if entity["kind"] == "Mesh"), initial["entities"][0])
    root_entity = copy.deepcopy(prototype)
    root_entity.update(id=identity("template:root"), name="Interaction Root", kind="Empty", parent=None, script=None, blueprint_instance=None, position=[0,0,0], rotation=[0,0,0], scale=[1,1,1])
    root_entity["collider"] = {"trigger": True}
    cube_entity = copy.deepcopy(prototype)
    cube_entity.update(id=identity("template:cube"), name="Visual Cube", kind="Mesh", parent=None, script=None, blueprint_instance=None, position=[0,1,0], rotation=[0,0,0], scale=[1,1,1])
    bp["template"] = {"entities": [{"entity": root_entity, "parent": None}, {"entity": cube_entity, "parent": root_entity["id"]}], "overrides": {}, "references": [], "construction": []}
    write(bp_path, bp)
    run("--project", project, "--compile-blueprints")
    relative = bp_path.relative_to(project).as_posix()
    command = [str(editor), "--project", str(project), "--open-blueprint", relative, "--screenshot-blueprint-canvas"]
    print(json.dumps({"project": str(project), "blueprint": relative, "command": command}, indent=2))


if __name__ == "__main__":
    main()
