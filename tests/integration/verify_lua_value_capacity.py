"""Prove compact scalar scratch and full 32-word copy semantics in both VMs."""
import argparse
from pathlib import Path
import subprocess
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write
from gameplay_probe import run_modes


def verify(out, wide, modes, lifetimes=False, reuse_project=None):
    label = "lifetimes32" if lifetimes else "wide32" if wide else "scalar"
    out = out / label
    out.mkdir(parents=True, exist_ok=True)
    project = reuse_project.resolve() if reuse_project else out / "project"
    namespace = uuid.UUID("963a6a1c-285f-5768-8c52-abf138be10fc")
    identity = lambda name: str(uuid.uuid5(namespace, label + ":" + name))
    if reuse_project:
        assert project.is_relative_to((ROOT / "artifacts").resolve())
        previous = documents.loads((project / "ProjectSettings/LuaClasses.epoksettings").read_text(encoding="utf-8"))
        assert previous["classes"]["assets/scripts/LuaCapacityProbe.lua"] == identity("lua"), "Reuse only the same fixture case"

    def run(*args):
        result = subprocess.run([str(EDITOR), *map(str, args)], cwd=ROOT,
                                capture_output=True, text=True, encoding="utf-8", timeout=900)
        assert result.returncode == 0, result.stdout + result.stderr
    if not project.exists():
        run("--create-project", project, "--template", "basic", "--gameplay", "lua", "--name", "Lua Capacity " + label)
    magic = 0x53000000 | (uuid.uuid4().int & 0xffffff)
    source = r'''#pragma once
#include "epok.hpp"
#if defined(EPOK_LUA_MODE) && EPOK_LUA_MODE != 0
#include "lua_runtime.hpp"
#endif
#ifndef EPOK_LUA_MODE
#define EPOK_LUA_MODE 0
#endif
struct EPOK_VALUE(Id="VALUE_ID") CapacitySnapshot {
FIELDS
};
extern "C" {inline uint32_t lua_capacity_probe[8]={};}
class EPOK_CLASS(Blueprintable,Owners=World3D,Id="BASE") CapacityProbeBase:public epok::ActorComponent {
 void publish(bool valid){
  auto* p=lua_capacity_probe;p[0]=MAGIC+EPOK_LUA_MODE;p[1]=1;p[2]=valid?0:1;
#ifdef EPOK_LUA_MAX_VALUE_WORDS
  p[3]=EPOK_LUA_MAX_VALUE_WORDS;p[5]=sizeof(epok::lua::WireValue);
#endif
  p[4]=0x1234;p[6]=41;p[7]=10;
 }
public:
 EPOK_FUNCTION(BlueprintCallable) CapacitySnapshot snapshot(){CapacitySnapshot value;auto* words=reinterpret_cast<int32_t*>(&value);for(unsigned i=0;i<32;++i)words[i]=int32_t(i+10);return value;}
 EPOK_FUNCTION(BlueprintCallable) void scalar(){publish(true);}
 EPOK_FUNCTION(BlueprintCallable) epok::GameplayVector3 vector_value(){return {1.25,-2.5,3.75};}
 EPOK_FUNCTION(BlueprintCallable) void verify_vector(epok::GameplayVector3 original,epok::GameplayVector3 edited){
  publish(original.x==epok::Fixed(1.25)&&original.y==epok::Fixed(-2.5)&&original.z==epok::Fixed(3.75)&&
   edited.x==original.x&&edited.y==original.z&&edited.z==original.z);
 }
 EPOK_FUNCTION(BlueprintCallable) void verify(CapacitySnapshot original,CapacitySnapshot edited){
  bool valid=true;const auto* a=reinterpret_cast<const int32_t*>(&original);const auto* b=reinterpret_cast<const int32_t*>(&edited);
  for(unsigned i=0;i<32;++i){valid=valid&&a[i]==int32_t(i+10)&&b[i]==int32_t(i==31?10:i+10);}publish(valid);
 }
 LIFETIME_API
};
'''
    extra = r'''
 EPOK_FUNCTION(BlueprintCallable,BlueprintEvent) virtual CapacitySnapshot bounce(CapacitySnapshot value){return value;}
 DEPTH_API
 EPOK_FUNCTION(BlueprintCallable) uint32_t zero(){return 0;}
 EPOK_FUNCTION(BlueprintCallable) CapacitySnapshot reenter(CapacitySnapshot value){return bounce(value);}
 EPOK_FUNCTION(BlueprintCallable) void verify_lifetimes(CapacitySnapshot original,CapacitySnapshot edited,CapacitySnapshot recursive,uint32_t iterations){
  bool valid=iterations==1024;const auto* a=reinterpret_cast<const int32_t*>(&original);const auto* b=reinterpret_cast<const int32_t*>(&edited);const auto* c=reinterpret_cast<const int32_t*>(&recursive);
  for(unsigned i=0;i<32;++i)valid=valid&&a[i]==int32_t(i+10)&&b[i]==int32_t(i==31?10:i+10)&&c[i]==int32_t(i==0?24:i+10);
  publish(valid);lua_capacity_probe[6]=iterations;
#ifdef EPOK_LUA_PLANNED_VALUE_SLOTS
  lua_capacity_probe[7]=EPOK_LUA_PLANNED_VALUE_SLOTS;
#endif
 }
'''
    source = source.replace("LIFETIME_API", extra if lifetimes else "")
    source = source.replace("DEPTH_API", "\n".join(
        f" EPOK_FUNCTION(BlueprintCallable,BlueprintEvent) virtual CapacitySnapshot depth{index}(CapacitySnapshot input){{return input;}}"
        for index in range(15)))
    source = source.replace("VALUE_ID", identity("value")).replace("BASE", identity("base")).replace("MAGIC", str(magic))
    source = source.replace("FIELDS", "\n".join(f" int32_t word_{i}=0;" for i in range(32)))
    write(project / "assets/scripts/CapacityProbe.hpp", source)
    body = """    local original = self:snapshot()
    local edited = original
    edited.word_31 = original.word_0
    self:verify(original, edited)""" if wide else """    local original = self:vector_value()
    local edited = original
    edited.y = original.z
    self:verify_vector(original, edited)"""
    helpers = ""
    if lifetimes:
        body = """    local original = self:snapshot()
    local edited = original
    local visits = self:zero()
    for index = 1, 1024 do
        local transient = self:snapshot()
        local alias = original
        edited = self:reenter(transient)
        edited.word_31 = alias.word_0
        visits = visits + 1
    end
    local recursive = self:depth0(original)
    self:verify_lifetimes(original, edited, recursive, visits)"""
        helpers = """
---@override
function LuaCapacityProbe:bounce(value)
    local copied = value
    copied.word_30 = value.word_30
    return copied
end
"""
        # The profile rejects direct recursion. Distinct typed overrides force
        # the same maximum16 native/Lua re-entrant frames without broadening it.
        for index in range(15):
            helpers += f"---@override\nfunction LuaCapacityProbe:depth{index}(input)\n"
            helpers += (f"    local copied = input\n    copied.word_0 = input.word_0 + 1\n    return self:depth{index + 1}(copied)\n"
                        if index < 14 else "    return input\n") + "end\n"
    write(project / "assets/scripts/LuaCapacityProbe.lua", "---@class LuaCapacityProbe : CapacityProbeBase\nlocal LuaCapacityProbe = CapacityProbeBase:extend()\n---@override\nfunction LuaCapacityProbe:begin_play()\n" + body + "\nend\n" + helpers + "return LuaCapacityProbe\n")
    registry_path = project / "ProjectSettings/LuaClasses.epoksettings"
    registry = documents.loads(registry_path.read_text(encoding="utf-8")) if registry_path.exists() else {"classes": {}}
    registry["classes"]["assets/scripts/LuaCapacityProbe.lua"] = identity("lua")
    write(registry_path, registry)
    descriptor = next(project.glob("*.epokproject"))
    manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    actor = scene["actors"][0]
    actor["components"] = [c for c in actor["components"] if c["class"].get("class_id") != identity("lua")]
    actor["components"].append({"id": str(uuid.uuid4()), "name": "Capacity Probe", "class": {"name": "LuaCapacityProbe", "class_id": identity("lua")}, "properties": {}, "overrides": []})
    write(scene_path, scene)

    def check(values, expected):
        width = 32 if wide else 3
        config = (project / ".epok/build/scripts/generated/lua/lua_value_config.hpp").read_text()
        assert f"EPOK_LUA_REQUIRED_VALUE_WORDS {width}" in config
        if lifetimes:
            import re
            slots = int(re.search(r"EPOK_LUA_PLANNED_VALUE_SLOTS (\d+)", config)[1])
            assert 1 <= slots < 64, slots
            assert values == [expected, 1, 0, width, 0x1234, 4 + width * 4, 1024, slots], values
        else:
            assert values == [expected, 1, 0, width, 0x1234, 4 + width * 4, 41, 10], values
    run_modes(out, project, label, modes or ("vm_bytecode", "vm_source"), "lua_capacity_probe", magic, check, provider_count=1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p5/lua-value-capacity")
    parser.add_argument("--case", choices=("scalar", "wide32", "lifetimes32"), action="append")
    parser.add_argument("--mode", choices=("vm_bytecode", "vm_source"), action="append")
    parser.add_argument("--reuse-project", type=Path, help="Reuse the same case's existing artifact project/build cache, retaining results in --out")
    args = parser.parse_args()
    if args.reuse_project and len(args.case or ()) != 1:
        parser.error("--reuse-project requires exactly one --case")
    for case in args.case or ("scalar", "wide32", "lifetimes32"):
        verify(args.out, case != "scalar", args.mode, case == "lifetimes32", args.reuse_project)
