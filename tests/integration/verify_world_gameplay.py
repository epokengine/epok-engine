"""Exercise shared world helpers and real provider policies in both domains."""
import argparse
import copy
import json
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes, export_probe

NAMESPACE = uuid.UUID("f4f96602-cf97-5b93-933c-373143624e53")
KINDS = {
    "Flag": ("GameplayFlagComponent", None, "value_changed", "bool previous,bool current", "previous,current", "current"),
    "Pickup": ("PickupComponent", "can_collect", "collected", "epok::ObjectId initiator,uint32_t port", "initiator,port", "true"),
    "Door": ("DoorComponent", "can_open", "door_changed", "epok::DoorState previous,epok::DoorState current", "previous,current", "current==epok::DoorState::Open"),
    "Switch": ("SwitchComponent", "can_switch", "switched", "epok::ObjectId initiator,bool value", "initiator,value", "value"),
}
DRIVER_SOURCE = r'''
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
extern "C" {inline uint32_t world_gameplay_probe[24]={};}
class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="DRIVER_ID") WorldGameplayDriver:public epok::ActorComponent {
 uint32_t frame=0,fail=0,hash=2166136261u;
public:
 EPOK_PROPERTY(EditAnywhere) uint32_t slot=0;
 EPOK_PROPERTY(EditAnywhere,ComponentClass="WorldFlagProbe") epok::ObjectId flag{};
 EPOK_PROPERTY(EditAnywhere,ComponentClass="WorldPickupProbe") epok::ObjectId pickup{};
 EPOK_PROPERTY(EditAnywhere,ComponentClass="WorldDoorProbe") epok::ObjectId door{};
 EPOK_PROPERTY(EditAnywhere,ComponentClass="WorldSwitchProbe") epok::ObjectId button{};
 EPOK_PROPERTY(EditAnywhere,ActorClass="epok::Actor") epok::ObjectId foreign{};
 void tick(epok::Fixed)override{
  if(frame>12||slot>=3)return;
  auto* r=epok::active_object_registry;auto* f=r?r->resolve<WorldFlagProbe>(flag):nullptr;
  auto* p=r?r->resolve<WorldPickupProbe>(pickup):nullptr;auto* d=r?r->resolve<WorldDoorProbe>(door):nullptr;
  auto* b=r?r->resolve<WorldSwitchProbe>(button):nullptr;
  if(!f||!p||!d||!b){fail|=1;return;}
  auto source=owner_id();auto* gate=d->get_owner();auto* level=r->resolve<epok::Level>(gate->level_id());
  auto* c3=level->get_component<epok::Collider3DComponent>(*gate);auto* c2=level->get_component<epok::Collider2DComponent>(*gate);
  auto blocked=[&](){return c3?c3->enabled():c2?c2->is_enabled():false;};
  auto expect=[&](bool condition,uint32_t bit){if(!condition)fail|=bit;};
  using H=epok::HelperResult;using S=epok::DoorState;
  switch(frame){
  case 0:
   expect(!f->value()&&d->state()==S::Closed&&blocked(),2);
   p->allowed=false;expect(p->collect(source,1)==H::Unavailable,4);p->allowed=true;
   expect(p->collect(foreign,1)==H::InvalidInitiator,8);
   expect(p->collect(source,1)==H::Applied&&p->is_collected()&&f->value(),16);
   expect(p->collect(source,1)==H::AlreadyCollected&&!p->get_owner()->active(),32);
   d->allowed=false;expect(d->open(source)==H::Unavailable,64);d->allowed=true;
   expect(d->open(source)==H::Queued&&blocked(),128);break;
  case 1:
   expect(d->state()==S::Open&&!blocked(),256);expect(d->close()==H::Queued&&!blocked(),512);break;
  case 2:
   expect(d->state()==S::Closed&&blocked(),1024);
   b->allowed=false;expect(b->activate(source,1)==H::Unavailable,2048);b->allowed=true;
   expect(b->activate(source,1)==H::Applied&&!f->value(),4096);
   expect(d->open(source)==H::RequirementUnmet,8192);
   expect(b->activate(source,1)==H::Applied&&f->value(),16384);expect(d->open(source)==H::Queued,32768);break;
  case 3:expect(d->state()==S::Open&&!blocked(),65536);f->set_value(false);d->close();break;
  case 4:expect(d->state()==S::Closed&&blocked(),131072);f->set_value(true);d->open(source);f->set_value(false);break;
  case 5:expect(d->state()==S::Closed&&blocked()&&d->last_result()==H::RequirementUnmet,262144);f->set_value(true);d->lock();break;
  case 6:expect(d->state()==S::Locked&&blocked(),524288);expect(d->open(source)==H::Queued,1048576);break;
  case 7:expect(d->state()==S::Open&&!blocked(),2097152);d->reset_door();break;
  case 8:
   expect(d->state()==S::Closed&&blocked(),4194304);
   expect(b->activate(source,1)==H::Applied&&!f->value(),8388608);expect(b->reset_switch()==H::Unchanged,16777216);
   p->get_owner()->set_active(true);expect(p->reset_collection(),33554432);
   expect(p->collect(source,1)==H::Applied&&f->value(),67108864);d->open(source);break;
  case 9:expect(d->state()==S::Open&&!blocked(),134217728);d->close();break;
  case 10:expect(d->state()==S::Closed&&blocked(),268435456);break;
  }
  expect(f->count==f->parent_calls&&p->count==p->parent_calls&&d->count==d->parent_calls&&b->count==b->parent_calls,536870912);
  expect(f->truth==f->value()&&f->truth==f->parent_truth&&p->truth==p->parent_truth&&d->truth==(d->state()==S::Open)&&d->truth==d->parent_truth&&b->truth==b->parent_truth,1073741824);
  const uint32_t words[]={frame,p->is_collected(),f->value(),uint32_t(d->state()),d->has_pending_transition(),f->count,p->count,d->count,b->count,f->truth,p->truth,d->truth,b->truth};
  for(auto word:words)hash=(hash^word)*16777619u;
  if(frame==12){auto* out=world_gameplay_probe+slot*8;out[0]=MAGIC+EPOK_LUA_MODE;out[1]=12;out[2]=fail;out[3]=p->count;out[4]=f->count;out[5]=d->count;out[6]=b->count;out[7]=hash;}
  ++frame;
 }
};
'''


def verify(out, domain, modes, prepare_only, export):
    out = out / domain
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    identity = lambda name: str(uuid.uuid5(NAMESPACE, domain + ":" + name))
    magic = 0x5B000000 | (uuid.uuid4().int & 0xffffff)
    counter = 0

    def run(*args, label=None):
        nonlocal counter
        counter += 1
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT, capture_output=True,
                                text=True, encoding="utf-8", timeout=900)
        (out / (label or f"command-{counter}.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
        return result.stdout.strip()

    if not project.exists():
        run("--create-project", project, "--template", "basic", "--name", "World Helpers " + domain)
    native = '#pragma once\n#include "epok.hpp"\n'
    for stem, (core, query, event, parameters, arguments, truth) in KINDS.items():
        name = f"World{stem}Probe"
        native += f'''class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="{identity(name)}") {name}:public epok::{core} {{
public:
 static constexpr uint64_t static_class_id=epok::detail::compact_class_id("{identity(name)}");
 uint64_t class_id()const override{{return m_runtime_class_id?m_runtime_class_id:static_class_id;}}
 EPOK_PROPERTY(EditAnywhere) bool allowed=true;
 EPOK_PROPERTY(EditAnywhere) uint32_t count=0;
 EPOK_PROPERTY(EditAnywhere) bool truth=false;
 EPOK_PROPERTY(EditAnywhere) uint32_t parent_calls=0;
 EPOK_PROPERTY(EditAnywhere) bool parent_truth=false;
'''
        if query:
            native += f'EPOK_FUNCTION(BlueprintEvent,BlueprintPure) bool {query}(epok::ObjectId initiator)const override{{return epok::{core}::{query}(initiator);}}\n'
        parent_truth = 'initiator.valid()&&port==1' if stem == "Pickup" else truth
        native += f'EPOK_FUNCTION(BlueprintEvent) void {event}({parameters})override{{epok::{core}::{event}({arguments});++parent_calls;parent_truth={parent_truth};}}\n}};\n'
        native += f'''class EPOK_CLASS(Blueprintable,Owners=World3D|World2D,Id="{identity("Cpp"+stem)}") CppWorld{stem}:public {name} {{
public:
 static constexpr uint64_t static_class_id=epok::detail::compact_class_id("{identity("Cpp"+stem)}");
 uint64_t class_id()const override{{return m_runtime_class_id?m_runtime_class_id:static_class_id;}}
'''
        if query:
            native += f'bool {query}(epok::ObjectId initiator)const override{{return {name}::{query}(initiator)&&allowed;}}\n'
        native += f'void {event}({parameters})override{{{name}::{event}({arguments});++count;truth={truth};}}\n}};\n'
    native += DRIVER_SOURCE.replace("DRIVER_ID", identity("driver")).replace("MAGIC", str(magic))
    write(project / "assets/scripts/WorldGameplayProbe.hpp", native)
    metadata = json.loads(run("--project", project, "--reflect"))
    classes = {c["cpp_name"]: c for c in metadata["classes"]}
    provider_classes = {"cpp": {}, "blueprint": {}, "lua": {}}

    def node(kind, inputs=None, **fields):
        return {"id": str(uuid.uuid4()), "kind": {"kind": kind, **fields}, "inputs": inputs or {}, "outputs": {}}

    def link(value):
        return {"kind": "link", "node": value["id"], "pin": "value"}

    for stem, (core, query, event, _, arguments, truth) in KINDS.items():
        base = classes[f"World{stem}Probe"]
        fields = {p["name"]: p for p in base["properties"]}
        functions = {f["name"]: f for f in base["functions"]}
        path = project / f"assets/Blueprints/BPWorld{stem}.epokbp"
        if not path.exists():
            run("--project", project, "--new-blueprint", f"BPWorld{stem}", "--parent", base["id"])
        asset = documents.loads(path.read_text(encoding="utf-8"))
        graphs = []
        for method in filter(None, (query, event)):
            signature = functions[method]
            params = {p["name"]: {"kind": "parameter", "name": p["name"]} for p in signature["parameters"]}
            entry, parent = node("entry"), node("call_parent", params)
            nodes = [entry, parent]
            entry["outputs"]["next"] = [parent["id"]]
            if method == query:
                allowed = node("get_variable", member=fields["allowed"]["id"])
                both = node("binary", {"a": link(parent), "b": link(allowed)}, op="and")
                finish = node("return", {"value": link(both)})
                parent["outputs"]["next"] = [finish["id"]]
                nodes += [allowed, both, finish]
            else:
                count = node("get_variable", member=fields["count"]["id"])
                one = node("literal", value_type={"kind": "uint32"}, value=1)
                add = node("binary", {"a": link(count), "b": link(one)}, op="add")
                assign = node("set_variable", {"value": link(add)}, member=fields["count"]["id"])
                nodes += [count, one, add, assign]
                if stem == "Pickup":
                    value = node("literal", value_type={"kind": "bool"}, value=True)
                    nodes.append(value)
                    expression = link(value)
                elif stem == "Door":
                    value = node("literal", value_type=signature["parameters"][1]["value_type"], value=1)
                    equal = node("binary", {"a": params["current"], "b": link(value)}, op="equal")
                    nodes += [value, equal]
                    expression = link(equal)
                else:
                    expression = params["current" if stem == "Flag" else "value"]
                finish = node("set_variable", {"value": expression}, member=fields["truth"]["id"])
                parent["outputs"]["next"] = [assign["id"]]
                assign["outputs"]["next"] = [finish["id"]]
                nodes.append(finish)
            graphs.append({"id": str(uuid.uuid4()), "name": method, "override_id": signature["id"],
                           "parameters": signature["parameters"], "returns": signature["returns"],
                           "entry": entry["id"], "nodes": nodes})
        asset["functions"] = graphs
        write(path, asset)
        lua_name = f"LuaWorld{stem}"
        text = f'---@class {lua_name} : World{stem}Probe\nlocal {lua_name} = World{stem}Probe:extend()\n'
        if query:
            text += f'---@override\nfunction {lua_name}:{query}(initiator)\n return {lua_name}.super.{query}(self, initiator) and self.allowed\nend\n'
        lua_truth = truth.replace("epok::DoorState::Open", "epok.DoorState.Open")
        text += f'---@override\nfunction {lua_name}:{event}({arguments})\n {lua_name}.super.{event}(self, {arguments})\n self.count = self.count + 1\n self.truth = {lua_truth}\nend\nreturn {lua_name}\n'
        write(project / f"assets/scripts/{lua_name}.lua", text)
        provider_classes["cpp"][stem] = {"name": f"CppWorld{stem}", "class_id": identity("Cpp" + stem)}
        provider_classes["blueprint"][stem] = {"name": f"BPWorld{stem}", "class_id": asset["id"]}
        provider_classes["lua"][stem] = {"name": lua_name, "class_id": identity(lua_name)}
    write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": {f"assets/scripts/LuaWorld{stem}.lua": identity(f"LuaWorld{stem}") for stem in KINDS}})
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["actors"] = [scene["actors"][0]]
    dimension = 2 if domain == "world2d" else 3

    def component(cls, name, props):
        return {"id": str(uuid.uuid4()), "name": name, "class": cls, "properties": props, "overrides": list(props)}

    def actor(name):
        cls = classes[f"epok::Actor{dimension}D"]
        root = classes[f"epok::SceneComponent{dimension}D"]
        return {"id": str(uuid.uuid4()), "name": name, "class": {"name": cls["cpp_name"], "class_id": cls["id"]},
                "components": [{**component({"name": root["cpp_name"], "class_id": root["id"]}, "Root", {}), "root": True}], "properties": {}, "overrides": []}

    for slot, provider in enumerate(("cpp", "blueprint", "lua")):
        source, foreign, state, pickup, door, button = [actor(f"{provider} {name}") for name in ("source", "foreign", "state", "pickup", "door", "switch")]
        flag = component(provider_classes[provider]["Flag"], "Flag", {})
        collect = component(provider_classes[provider]["Pickup"], "Pickup", {"accepted_initiator": source["id"], "collected_flag": flag["id"], "deactivate_on_collect": True})
        gate = component(provider_classes[provider]["Door"], "Door", {"requirement_flag": flag["id"]})
        switch = component(provider_classes[provider]["Switch"], "Switch", {"target_flag": flag["id"]})
        collider_class = classes[f"epok::Collider{dimension}DComponent"]
        collider = component({"name": collider_class["cpp_name"], "class_id": collider_class["id"]}, "Blocker", {"collider": {"enabled": True}} if dimension == 3 else {})
        driver = component({"name": "WorldGameplayDriver", "class_id": identity("driver")}, "Driver", {
            "slot": slot, "flag": flag["id"], "pickup": collect["id"], "door": gate["id"], "button": switch["id"], "foreign": foreign["id"]})
        for owner, additions in ((source, [driver]), (state, [flag]), (pickup, [collect]), (door, [collider, gate]), (button, [switch])):
            owner["components"] += additions
        scene["actors"] += [source, foreign, state, pickup, door, button]
    write(scene_path, scene)
    run("--project", project, "--compile-blueprints")
    if prepare_only:
        return

    def check(values, expected):
        assert values[:7] == [expected, 12, 0, 2, 9, 9, 3], values

    run_modes(out, project, domain, modes, "world_gameplay_probe", magic, check)
    if export:
        export_probe(out, project, run, ("world_gameplay.hpp", "interaction.hpp", "object_model.hpp"))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p6/world-gameplay")
    parser.add_argument("--domain", choices=("world2d", "world3d"), action="append")
    parser.add_argument("--mode", choices=("native_cpp", "vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--export", action="store_true")
    args = parser.parse_args()
    for domain in args.domain or ("world3d", "world2d"):
        verify(args.out, domain, args.mode or ("native_cpp", "vm_bytecode", "vm_source"), args.prepare_only, args.export)
