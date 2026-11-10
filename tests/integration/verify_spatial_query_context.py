"""Call the shared query context from real C++/Blueprint/Lua project policies."""
import argparse
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes, export_probe

NS = uuid.UUID("1de5347b-8618-5dd0-884a-4588e86a8ad6")
identity = lambda name: str(uuid.uuid5(NS, name))
CONTEXT = "382ad26d-3bc1-53e4-94a2-c1e18e6b25e9"


def verify(out, modes, prepare_only, export, camera=False):
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"

    def run(*args, label):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / label).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "Query Context", label="create.log")
    magic = 0x6D000000 | (uuid.uuid4().int & 0xffffff)
    header = f'''#pragma once
#include "epok.hpp"
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {{inline uint32_t spatial_query_context_probe[24]={{}};}}
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="{identity("base")}") QueryProbeBase:public epok::ActorComponent {{
 bool done=false;
public:
 static constexpr uint64_t static_class_id=epok::detail::compact_class_id("{identity("base")}");
 uint64_t class_id()const override{{return m_runtime_class_id?m_runtime_class_id:static_class_id;}}
 EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::SpatialQueryContextComponent") epok::ObjectId context{{}};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor3D") epok::ObjectId subject{{}},wall{{}};
 EPOK_PROPERTY(EditAnywhere) epok::Fixed origin_x=0.0;
 EPOK_PROPERTY(EditAnywhere) uint32_t slot=0;
 EPOK_FUNCTION(BlueprintEvent,Id="{identity("exercise")}") virtual bool exercise(){{return false;}}
 void tick(epok::Fixed)override{{
  if(done||slot>=3)return;done=true;const bool valid=exercise();auto* p=spatial_query_context_probe+slot*8;
  p[0]={magic}+EPOK_LUA_MODE;p[1]=1;p[2]=valid?0:1;p[3]=5;p[4]=20;p[5]=1;p[6]=1;
  p[7]=epok::object_storage::hooks.statistics?epok::object_storage::hooks.statistics->rejected:~0u;
 }}
}};
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="{identity("cpp")}") CppQueryProbe:public QueryProbeBase {{
public:
 bool exercise()override{{
  auto* ctx=epok::active_object_registry->resolve<epok::SpatialQueryContextComponent>(context);if(!ctx)return false;
  ctx->reset_queries();
  const auto one=ctx->raycast_segment({{origin_x,0.0,0.0}},{{10.0,0.0,0.0}},1,subject,false);
  const auto pair=ctx->raycast_pair({{origin_x,0.0,0.0}},{{10.0,0.0,0.0}},{{origin_x,0.0,5.0}},{{10.0,0.0,0.0}},1,subject,false);
  const bool published=ctx->publish_actor(wall);
  const auto moved=ctx->move(subject,{{1.0,0.0,0.0}},1);
  return one.hit&&one.actor==wall&&pair.first.actor==wall&&!pair.second.hit&&published&&moved.valid&&moved.displacement.x==epok::Fixed(1.0);
 }}
}};
'''
    if camera:
        header = header.replace(' epok::ObjectId context{};\n', ' epok::ObjectId context{};\n EPOK_PROPERTY(EditAnywhere,ComponentClass="epok::SpringArmComponent") epok::ObjectId arm{};\n')
        header = header.replace('  return one.hit', '''  auto* camera=epok::active_object_registry->resolve<epok::SpringArmComponent>(arm);if(!camera)return false;
  const auto normal=camera->solve_shared({origin_x,0.0,0.0},{10.0,0.0,0.0},subject,epok::Fixed(1.0/60.0),context,1);
  const auto repeated=camera->solve_shared({origin_x,0.0,0.0},{10.0,0.0,0.0},subject,epok::Fixed(1.0/60.0),context,1);
  const auto near=camera->solve_shared({origin_x,0.0,0.0},{100.0,0.0,0.0},subject,epok::Fixed(1.0/60.0),context,1);
  const auto clear=camera->solve_shared({origin_x,0.0,5.0},{100.0,0.0,0.0},subject,epok::Fixed(1.0/60.0),context,1);
  const bool camera_ok=normal.blocked&&normal.fraction==repeated.fraction&&near.blocked&&near.fraction>epok::Fixed(0.0)&&near.fraction<epok::Fixed(.04)&&!clear.blocked&&clear.fraction==epok::Fixed(1.0);
  return camera_ok&&one.hit''')
    write(project / "assets/scripts/QueryProbe.hpp", header)
    metadata = json.loads(run("--project", project, "--reflect", label="reflect.json"))
    classes = {c["cpp_name"]: c for c in metadata["classes"]}
    base, context = classes["QueryProbeBase"], classes["epok::SpatialQueryContextComponent"]
    fields = {p["name"]: p for p in base["properties"]}
    methods = {f["name"]: f for f in context["functions"]}
    signature = next(f for f in base["functions"] if f["name"] == "exercise")
    path = project / "assets/Blueprints/BPQueryProbe.epokbp"
    if not path.exists():
        run("--project", project, "--new-blueprint", "BPQueryProbe", "--parent", base["id"], label="blueprint-create.log")
    asset = documents.loads(path.read_text(encoding="utf-8"))
    nodes = []

    def node(kind, inputs=None, **fields):
        value = {"id": str(uuid.uuid4()), "kind": {"kind": kind, **fields}, "inputs": inputs or {}, "outputs": {}}
        nodes.append(value)
        return value

    def link(value, field=""):
        return {"kind": "link", "node": value["id"], "pin": "value" + ("." + field if field else "")}

    def literal(kind, value):
        return {"kind": "literal", "value_type": {"kind": kind}, "value": value}

    entry = node("entry")
    variables = {name: node("get_variable", member=fields[name]["id"]) for name in (("context", "subject", "wall", "origin_x", "arm") if camera else ("context", "subject", "wall", "origin_x"))}

    def vector(x, y, z):
        operation = "4fe0f030-091c-4d55-984d-769077622c33"
        return link(node("operation", {operation + ":parameter:0": x,
                                       operation + ":parameter:1": literal("fixed", y),
                                       operation + ":parameter:2": literal("fixed", z)}, operation=operation))

    origin = vector(link(variables["origin_x"]), 0, 0)
    other = vector(link(variables["origin_x"]), 0, 5)
    delta = vector(literal("fixed", 10), 0, 0)
    previous = entry

    def call(name, inputs=None, target="context"):
        nonlocal previous
        receiver = classes["epok::SpringArmComponent"] if target == "arm" else context
        method = next(f for f in receiver["functions"] if f["name"] == name)
        value = node("call_on", {"__target": link(variables[target]), **(inputs or {})},
                     **{"class": receiver["id"], "function": method["id"]})
        previous["outputs"]["next"] = [value["id"]]
        previous = value
        return value

    call("reset_queries")
    shared = {"mask": literal("uint32", 1), "ignore": link(variables["subject"]), "triggers": literal("bool", False)}
    single = call("raycast_segment", {"origin": origin, "displacement": delta, **shared})
    pair = call("raycast_pair", {"first_origin": origin, "first_displacement": delta,
                                 "second_origin": other, "second_displacement": delta, **shared})
    published = call("publish_actor", {"actor": link(variables["wall"])})
    moved = call("move", {"actor": link(variables["subject"]), "displacement": vector(literal("fixed", 1), 0, 0), "mask": literal("uint32", 1)})
    conditions = [link(single, "hit"), link(node("binary", {"a": link(single, "actor"), "b": link(variables["wall"])}, op="equal")),
                  link(node("binary", {"a": link(pair, "first.actor"), "b": link(variables["wall"])}, op="equal")),
                  link(node("not", {"value": link(pair, "second.hit")})), link(published), link(moved, "valid"),
                  link(node("binary", {"a": link(moved, "displacement.x"), "b": literal("fixed", 1)}, op="equal"))]
    if camera:
        inputs = {"origin": origin, "offset": delta, "ignore": link(variables["subject"]), "delta_seconds": literal("fixed", 1 / 60),
                  "query_context": link(variables["context"]), "collision_revision": literal("uint32", 1)}
        normal = call("solve_shared", inputs, "arm")
        repeated = call("solve_shared", inputs, "arm")
        near = call("solve_shared", {**inputs, "offset": vector(literal("fixed", 100), 0, 0)}, "arm")
        clear = call("solve_shared", {**inputs, "origin": other, "offset": vector(literal("fixed", 100), 0, 0)}, "arm")
        conditions += [link(normal, "blocked"), link(near, "blocked"),
                       link(node("binary", {"a": link(normal, "fraction"), "b": link(repeated, "fraction")}, op="equal")),
                       link(node("binary", {"a": link(near, "fraction"), "b": literal("fixed", 0)}, op="greater")),
                       link(node("binary", {"a": link(near, "fraction"), "b": literal("fixed", .04)}, op="less")),
                       link(node("not", {"value": link(clear, "blocked")})),
                       link(node("binary", {"a": link(clear, "fraction"), "b": literal("fixed", 1)}, op="equal"))]
    result = conditions[0]
    for condition in conditions[1:]:
        result = link(node("binary", {"a": result, "b": condition}, op="and"))
    finish = node("return", {"value": result})
    previous["outputs"]["next"] = [finish["id"]]
    asset["functions"] = [{"id": identity("graph"), "name": "exercise", "override_id": signature["id"],
                           "parameters": [], "returns": signature["returns"], "entry": entry["id"], "nodes": nodes}]
    write(path, asset)
    lua_source = '''---@class LuaQueryProbe : QueryProbeBase
local LuaQueryProbe = QueryProbeBase:extend()
---@override
function LuaQueryProbe:exercise()
 local ctx = epok.cast(self.context, "epok::SpatialQueryContextComponent")
 ctx:reset_queries()
 local origin = epok.math.vector3(self.origin_x, 0.0, 0.0)
 local other = epok.math.vector3(self.origin_x, 0.0, 5.0)
 local delta = epok.math.vector3(10.0, 0.0, 0.0)
 local one = ctx:raycast_segment(origin, delta, 1, self.subject, false)
 local pair = ctx:raycast_pair(origin, delta, other, delta, 1, self.subject, false)
 local published = ctx:publish_actor(self.wall)
 local moved = ctx:move(self.subject, epok.math.vector3(1.0, 0.0, 0.0), 1)
 return one.hit and one.actor == self.wall and pair.first.actor == self.wall and not pair.second.hit and published and moved.valid and moved.displacement.x == 1.0
end
return LuaQueryProbe
'''
    if camera:
        lua_source = lua_source.replace(' return one.hit', ''' local camera = epok.cast(self.arm, "epok::SpringArmComponent")
 local normal = camera:solve_shared(origin, delta, self.subject, 1.0 / 60.0, self.context, 1)
 local repeated = camera:solve_shared(origin, delta, self.subject, 1.0 / 60.0, self.context, 1)
 local near = camera:solve_shared(origin, epok.math.vector3(100.0, 0.0, 0.0), self.subject, 1.0 / 60.0, self.context, 1)
 local clear = camera:solve_shared(other, epok.math.vector3(100.0, 0.0, 0.0), self.subject, 1.0 / 60.0, self.context, 1)
 local camera_ok = normal.blocked and normal.fraction == repeated.fraction and near.blocked and near.fraction > 0.0 and near.fraction < 0.04 and not clear.blocked and clear.fraction == 1.0
 return camera_ok and one.hit''')
    write(project / "assets/scripts/LuaQueryProbe.lua", lua_source)
    registry_path = project / "ProjectSettings/LuaClasses.epoksettings"
    registry = documents.loads(registry_path.read_text(encoding="utf-8")) if registry_path.exists() else {"classes": {}}
    registry["classes"]["assets/scripts/LuaQueryProbe.lua"] = identity("lua")
    write(registry_path, registry)
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["actors"] = [scene["actors"][0]]

    def ref(name):
        return {"name": name, "class_id": classes[name]["id"]}

    def component(cls, props, root=False):
        result = {"id": str(uuid.uuid4()), "name": cls["name"], "class": cls, "properties": props, "overrides": list(props)}
        if root:
            result["root"] = True
        return result

    def actor(name, x):
        return {"id": str(uuid.uuid4()), "name": name, "class": ref("epok::Actor3D"), "properties": {}, "overrides": [],
                "components": [component(ref("epok::SceneComponent3D"), {"position": [x, 0, 0]}, True),
                               component(ref("epok::Collider3DComponent"), {"collider": {"enabled": True}})]}

    providers = [ref("CppQueryProbe"), {"name": "BPQueryProbe", "class_id": asset["id"]}, {"name": "LuaQueryProbe", "class_id": identity("lua")}]
    for slot, provider in enumerate(providers):
        pawn, wall = actor("Query subject " + str(slot), slot * 20), actor("Query wall " + str(slot), slot * 20 + 4)
        ctx = component(ref("epok::SpatialQueryContextComponent"), {})
        rule = component(provider, {"context": ctx["id"], "subject": pawn["id"], "wall": wall["id"], "origin_x": slot * 20, "slot": slot})
        pawn["components"] += [ctx, rule]
        if camera:
            arm = component(ref("epok::SpringArmComponent"), {"collision_mask": 1, "minimum_fraction": 0, "adaptive_pull_in": True})
            pawn["components"].append(arm)
            rule["properties"]["arm"] = arm["id"]
            rule["overrides"].append("arm")
        scene["actors"] += [pawn, wall]
    write(scene_path, scene)
    run("--project", project, "--compile-blueprints", label="compile-blueprints.log")
    if prepare_only:
        print("PASS prepared real query context providers", flush=True)
        return

    def check(values, expected):
        assert values == [expected, 1, 0, 5, 20, 1, 1, 0], values

    run_modes(out, project, "query-camera" if camera else "query-context", modes, "spatial_query_context_probe", magic, check)
    if export:
        export_probe(out, project, run, ("gameplay_api.hpp", "camera_rig.hpp") if camera else ("gameplay_api.hpp",))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p8/query-context")
    parser.add_argument("--mode", action="append", choices=("native_cpp", "vm_bytecode", "vm_source"))
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--export", action="store_true")
    parser.add_argument("--camera", action="store_true", help="Also exercise SpringArm shared queries and close-obstacle clearance")
    options = parser.parse_args()
    verify(options.out, options.mode, options.prepare_only, options.export, options.camera)
