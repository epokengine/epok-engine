"""Reflect native structs, lower nested split pins and build them for PSX."""
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
    root = Path(tempfile.mkdtemp(prefix="epok-split-pins-")) / "SplitPins"
    def run(*arguments, ok=True):
        result = subprocess.run([str(args.editor.resolve()), *map(str, arguments)], cwd=REPO,
                                capture_output=True, text=True, encoding="utf-8", errors="replace",
                                timeout=300, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        assert (result.returncode == 0) == ok, result.stdout + result.stderr
        return result.stdout
    def write(path, value):
        documents.write_text(path, json.dumps(value, indent=2), encoding="utf-8")
    def node(kind, **extra):
        return dict(id=str(uuid.uuid4()), kind=dict(kind=kind, **extra), inputs={}, outputs={})
    def literal(ty, value):
        return dict(kind="literal", value_type=ty, value=value)
    def link(source, pin):
        return dict(kind="link", node=source["id"], pin=pin)
    run("--create-project", root, "--name", "Split Pins")
    class_id = str(uuid.uuid4())
    header = root / "assets/scripts/SplitFixture.hpp"
    header.write_text('''#pragma once
#include "epok.hpp"
enum class SplitMode { First, Second };
struct SplitDetails { bool enabled; int32_t count; SplitMode mode; };
struct SplitData { epok::Fixed point[3]; epok::Fixed uv[2]; SplitDetails details; epok::Transform pose; };
class EPOK_CLASS(Blueprintable, Id="CLASS") SplitFixture : public epok::Behaviour {
public:
    EPOK_FUNCTION(BlueprintCallable, BlueprintPure) SplitData echo(const SplitData& data) const { return data; }
    EPOK_FUNCTION(BlueprintCallable) SplitData produce(const SplitData& data) { ++calls; return data; }
    EPOK_FUNCTION(BlueprintCallable) void consume(epok::Fixed x, bool enabled, int32_t count, SplitMode mode, const SplitData& data) {
        result=x; accepted=enabled; total=count; last=data; current=mode;
    }
    int calls=0, total=0; bool accepted=false; epok::Fixed result=0.0;
    SplitData last{}; SplitMode current=SplitMode::First;
};
'''.replace('"CLASS"', f'"{class_id}"'), encoding="utf-8")
    reflected = json.loads(run("--project", root, "--reflect"))
    cls = next(c for c in reflected["classes"] if c["id"] == class_id)
    funcs = {f["name"]: f for f in cls["functions"]}
    ty = funcs["echo"]["returns"]
    members = {f["name"]: f["value_type"] for f in ty["fields"]}
    assert set(members) == {"point", "uv", "details", "pose"}
    assert [f["name"] for f in members["details"]["fields"]] == ["enabled", "count", "mode"]
    path = Path(run("--project", root, "--new-blueprint", "BP_SplitPins", "--parent", class_id).strip())
    bp = documents.loads(path.read_text(encoding="utf-8"))
    graph = bp["functions"][0]
    entry = graph["nodes"][0]
    transform = graph["parameters"][0]["name"]
    defaults = dict(point=[2,3,4], uv=[8,9], details=dict(enabled=True, count=7, mode=1),
                    pose=dict(position=[5,6,7], rotation=[0,0,0], scale=[1,1,1]))
    pure = node("call", function=funcs["echo"]["id"])
    pure["inputs"]["data"] = literal(ty, defaults)
    for name, field_ty in members.items():
        pure["inputs"][f"data.{name}"] = literal(field_ty, defaults[name])
    for i, axis in enumerate("xyz"):
        pure["inputs"][f"data.point.{axis}"] = literal(dict(kind="fixed"), i+2)
    pure["inputs"]["data.point.y"] = link(entry, transform + ".position.y")
    pure["inputs"]["data.uv.x"] = literal(dict(kind="fixed"), 8)
    pure["inputs"]["data.uv.y"] = literal(dict(kind="fixed"), 9)
    pure["inputs"]["data.pose.position"] = literal(dict(kind="vector", length=3), [5,6,7])
    pure["inputs"]["data.pose.position.x"] = literal(dict(kind="fixed"), 10)
    # Missing child values deliberately exercise disconnect/paste fallback.
    producer = node("call", function=funcs["produce"]["id"])
    producer["inputs"]["data"] = link(pure, "value")
    consumer = node("call", function=funcs["consume"]["id"])
    consumer["inputs"].update(x=link(producer, "value.pose.position.x"),
                              enabled=link(producer, "value.details.enabled"),
                              count=link(producer, "value.details.count"),
                              mode=link(producer, "value.details.mode"),
                              data=literal(ty, defaults))
    # A projected native array must convert into the compiler's vector value.
    consumer["inputs"]["data.point"] = link(producer, "value.pose.scale")
    consumer["inputs"]["data.uv"] = link(producer, "value.uv")
    consumer["inputs"]["data.details"] = link(producer, "value.details")
    consumer["inputs"]["data.pose"] = link(producer, "value.pose")
    entry["outputs"]["next"] = [producer["id"]]
    producer["outputs"]["next"] = [consumer["id"]]
    graph["nodes"].extend([pure, producer, consumer])
    bp["layout"]["split_pins"] = [f'{entry["id"]}:out:{transform}',
        f'{entry["id"]}:out:{transform}.position', f'{producer["id"]}:out:value',
        f'{producer["id"]}:out:value.details', f'{producer["id"]}:out:value.pose',
        f'{producer["id"]}:out:value.pose.position']
    bp["layout"]["positions"].update({pure["id"]: [350,50], producer["id"]: [650,50], consumer["id"]: [1000,50]})
    write(path, bp)
    before = path.read_bytes()
    run("--project", root, "--compile-blueprints")
    assert path.read_bytes() == before
    for bad in ["value.details.missing", "value.point.w"]:
        broken = copy.deepcopy(bp)
        broken["functions"][0]["nodes"][-1]["inputs"]["x"]["pin"] = bad
        write(path, broken)
        run("--project", root, "--compile-blueprints", ok=False)
    broken = copy.deepcopy(bp)
    broken["functions"][0]["nodes"][-3]["inputs"]["data.point.x"] = literal(dict(kind="bool"), True)
    write(path, broken)
    run("--project", root, "--compile-blueprints", ok=False)
    write(path, bp)
    if args.build_psx:
        # Promotion produces these same ordinary variable/Get/Set nodes. Verify
        # both native struct storage and vector storage in the target build.
        data_variable, vector_variable = str(uuid.uuid4()), str(uuid.uuid4())
        bp["variables"].extend([
            dict(id=data_variable, name="promoted_data", value_type=ty, default=defaults, editable=True),
            dict(id=vector_variable, name="promoted_vector", value_type=dict(kind="vector", length=3), default=[1,2,3], editable=True),
        ])
        set_data = node("set_variable", member=data_variable)
        set_data["inputs"]["value"] = link(producer, "value")
        set_vector = node("set_variable", member=vector_variable)
        set_vector["inputs"]["value"] = link(producer, "value.point")
        get_vector = node("get_variable", member=vector_variable)
        producer["outputs"]["next"] = [set_data["id"]]
        set_data["outputs"]["next"] = [set_vector["id"]]
        set_vector["outputs"]["next"] = [consumer["id"]]
        consumer["inputs"]["x"] = link(get_vector, "value.x")
        graph["nodes"].extend([set_data, set_vector, get_vector])
        write(path, bp)
        scene_path = root / "assets/scenes/Main.epokmap"
        scene = documents.loads(scene_path.read_text(encoding="utf-8"))
        entity = copy.deepcopy(scene["entities"][0])
        entity.update(id=str(uuid.uuid4()), name="Split Pin Cube", kind="Mesh", position=[0,0,0], scale=[1,1,1])
        entity["script"] = dict(name=bp["name"], class_id=bp["id"], provider=dict(id="blueprint", version=1), backend=dict(id="native", version=1), properties={})
        scene["entities"].append(entity)
        write(scene_path, scene)
        run("--project", root, "--build-psx")
        assert (root / ".epok/build/epok.ps-exe").is_file()
        generated = (root / f'.epok/build/scripts/generated/{bp["id"]}.hpp').read_text(encoding="utf-8")
        assert generated.count("this->produce(") == 1, "Split output fanout must reuse the impure call result"
    print(json.dumps(dict(passed=True, project=str(root), blueprint=str(path), psx_build=args.build_psx), indent=2))


if __name__ == "__main__":
    main()
