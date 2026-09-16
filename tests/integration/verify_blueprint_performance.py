"""Measure equivalent native/BP/Lua workloads; no wall-clock or estimated FPS claims.

Builds through the production CLI. --emulator runs isolated PCS-X instances,
collecting guest CPU cycles at identical native entry/end markers. Execution is
serialized because the emulator debugger and the production builder are shared.
Pass --include-lua to include Gameplay-v2 NativeCpp, VmBytecode and VmSource.
"""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import argparse
import contextlib
import json
import os
from pathlib import Path
import re
import shutil
import statistics
import subprocess
import tempfile

from verify_blueprints import ROOT, identity, literal, node, run, write


ITERATIONS = 16
WARMUP_FRAMES = 10
SAMPLE_FRAMES = 120
LUA_VARIANTS = {
    "lua_native_cpp": "native_cpp",
    "lua_vm_bytecode": "vm_bytecode",
    "lua_vm_source": "vm_source",
}


def link(source):
    return {"kind": "link", "node": source["id"], "pin": "value"}


def tool(name, *args):
    candidates = [shutil.which(f"mipsel-none-elf-{name}"),
                  ROOT / f".tools/mips/bin/mipsel-none-elf-{name}",
                  ROOT / f".tools/mips/bin/mipsel-none-elf-{name}.exe"]
    binary = next((Path(path) for path in candidates if path and Path(path).is_file()), None)
    assert binary, f"missing mipsel-none-elf-{name} on PATH or under .tools/mips/bin"
    result = subprocess.run([str(binary), *map(str, args)], capture_output=True,
                            text=True, encoding="utf-8", errors="replace", timeout=60)
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout


def create_project(directory, variant):
    root = directory / variant
    run("--create-project", root, "--name", "Equivalent Blueprint Performance")
    base_id, child_id = identity(), identity()
    value_id, steps_id, slot_id = identity(), identity(), identity()
    header = '''#pragma once
#include "epok.hpp"
extern "C" {
extern volatile int32_t blueprint_perf_probe[132];
void blueprint_perf_begin();
void blueprint_perf_end();
}
class EPOK_CLASS(Blueprintable, Id="BASE") PerfBase : public epok::Actor3D {
public:
    EPOK_PROPERTY(EditAnywhere, Id="VALUE") epok::Fixed value=1.0;
    EPOK_PROPERTY(EditAnywhere, Id="STEPS") uint32_t steps=0;
    EPOK_PROPERTY(EditAnywhere, Id="SLOT") uint32_t slot=0;
    EPOK_FUNCTION(BlueprintEvent) virtual void work() {}
    void tick(epok::Fixed) override;
};
'''.replace('"BASE"', f'"{base_id}"').replace('"VALUE"', f'"{value_id}"').replace('"STEPS"', f'"{steps_id}"').replace('"SLOT"', f'"{slot_id}"')
    source = '''#include "Perf.hpp"
#include "blueprint_runtime.hpp"
extern "C" {
volatile int32_t blueprint_perf_probe[132]={};
__attribute__((noinline,used)) void blueprint_perf_begin(){asm volatile("nop":::"memory");}
__attribute__((noinline,used)) void blueprint_perf_end(){asm volatile("nop\\nnop":::"memory");}
}
__attribute__((noinline)) void PerfBase::tick(epok::Fixed){
    blueprint_perf_begin();
    work();
    blueprint_perf_probe[0]=0x42505031;
    blueprint_perf_probe[4+slot*2]=value.raw();
    blueprint_perf_probe[5+slot*2]=int32_t(steps);
    blueprint_perf_probe[1]=blueprint_perf_probe[1]+1;
    blueprint_perf_end();
}
'''
    if variant == "cpp":
        header += f'''class EPOK_CLASS(Blueprintable, Id="{child_id}") CppPerf : public PerfBase {{
public:
    EPOK_FUNCTION(BlueprintEvent) void work() override;
}};
'''
        source += f'''__attribute__((noinline)) void CppPerf::work(){{
    for(uint32_t i=0;i<{ITERATIONS};++i){{
        value=epok::bp::add(epok::bp::mul(value,epok::Fixed(4090,epok::Fixed::RAW)),epok::Fixed(16,epok::Fixed::RAW));
        steps=epok::bp::uadd(steps,uint32_t(1));
    }}
}}
'''
    write(root / "assets/scripts/Perf.hpp", header)
    write(root / "assets/scripts/Perf.cpp", source)
    reflected = json.loads(run("--project", root, "--reflect"))
    base = next(c for c in reflected["classes"] if c["id"] == base_id)
    if variant == "blueprint":
        path = Path(run("--project", root, "--new-blueprint", "BP_Perf", "--parent", base_id).strip())
        asset = documents.loads(path.read_text(encoding="utf-8"))
        child_id = asset["id"]
        event = next(f for f in base["functions"] if f["name"] == "work")
        entry, loop, end = node("entry"), node("loop", count=ITERATIONS), node("return")
        value = node("get_variable", member=value_id)
        multiply = node("binary", {"a": link(value), "b": literal("fixed", 4090 / 4096)}, op="multiply")
        add = node("binary", {"a": link(multiply), "b": literal("fixed", 16 / 4096)}, op="add")
        assign = node("set_variable", {"value": link(add)}, member=value_id)
        steps = node("get_variable", member=steps_id)
        increment = node("binary", {"a": link(steps), "b": literal("uint32", 1)}, op="add")
        # Follow the actual reflected parameter contract.
        step_type = next(p["value_type"] for p in base["properties"] if p["id"] == steps_id)
        increment["inputs"]["b"]["value_type"] = step_type
        save_steps = node("set_variable", {"value": link(increment)}, member=steps_id)
        entry["outputs"]["next"] = [loop["id"]]
        loop["outputs"] = {"body": [assign["id"]], "next": [end["id"]]}
        assign["outputs"]["next"] = [save_steps["id"]]
        asset["functions"] = [{"id": identity(), "name": "work", "override_id": event["id"],
                               "parameters": event["parameters"], "returns": event["returns"],
                               "entry": entry["id"], "nodes": [entry, loop, end, value, multiply, add, assign, steps, increment, save_steps]}]
        write(path, asset)
        run("--project", root, "--compile-blueprints")
    elif variant in LUA_VARIANTS:
        path = Path(run("--project", root, "--new-lua-class", "LuaPerf",
                        "--parent", "PerfBase").strip())
        source = f'''---@class LuaPerf : PerfBase
local LuaPerf = PerfBase:extend()

---@override
function LuaPerf:work()
    for i = 1, {ITERATIONS} do
        self.value = self.value * 0.99853515625 + 0.00390625
        self.steps = self.steps + 1
    end
end

return LuaPerf
'''
        write(path, source)
        identities = documents.loads(
            (root / "ProjectSettings/LuaClasses.epoksettings").read_text(encoding="utf-8")
        )
        child_id = identities["classes"]["assets/scripts/LuaPerf.lua"]
        manifest_path = next(root.glob("*.epokproject"))
        manifest = documents.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["lua_profile"] = "gameplay_v2"
        manifest["lua_execution"] = LUA_VARIANTS[variant]
        write(manifest_path, manifest)
    scene_path = root / "assets/scenes/Main.epokmap"
    initial = documents.loads(scene_path.read_text(encoding="utf-8"))
    return root, child_id, initial


def build_instances(project, variant, class_id, initial, count):
    scene = json.loads(json.dumps(initial))
    prototype = scene["actors"][0]
    scene["actors"] = []
    for i in range(count):
        actor = json.loads(json.dumps(prototype))
        name = ("BP_Perf" if variant == "blueprint" else
                "LuaPerf" if variant in LUA_VARIANTS else "CppPerf")
        actor.update(id=identity(), name=f"Perf{i}",
                     **{"class": {"class_id": class_id, "name": name},
                        "properties": {"slot": i}, "overrides": ["slot"]})
        for component in actor["components"]:
            component["id"] = identity()
        scene["actors"].append(actor)
    write(project / "assets/scenes/Main.epokmap", scene)
    run("--project", project, "--build-psx")
    build = project / ".epok/build"
    elf = build / "epok.elf"
    symbols = tool("nm", "-S", "--defined-only", "-C", elf)
    table = {}
    for line in symbols.splitlines():
        match = re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line)
        if match:
            table[match[3]] = {"address": int(match[1], 16), "bytes": int(match[2], 16)}
    work_name = ("BP_Perf::work()" if variant == "blueprint" else
                 "LuaPerf::work()" if variant in LUA_VARIANTS else "CppPerf::work()")
    vm = variant in ("lua_vm_bytecode", "lua_vm_source")
    if not vm:
        assert work_name in table, f"Missing measurable virtual function {work_name}"
    for marker in ("blueprint_perf_begin", "blueprint_perf_end", "blueprint_perf_probe"):
        assert marker in table, f"Missing measurement symbol {marker}"
    disassembly = tool("objdump", "-d", "-C", elf)
    if vm:
        prologue, frames = "interpreted Lua VM method", []
    else:
        function = re.search(r"^[0-9a-fA-F]+ <" + re.escape(work_name) + r">:\n(.*?)(?=\n[0-9a-fA-F]+ <|\Z)",
                             disassembly, re.MULTILINE | re.DOTALL)
        assert function, work_name
        prologue = "\n".join(function[1].splitlines()[:12])
        frames = [int(n) for n in re.findall(r"addiu\s+\$?sp,\s*\$?sp,\s*-(\d+)", prologue)]
        # A function with no negative SP adjustment is a zero-frame leaf here;
        # unusual register-based adjustments must not silently become zero.
        assert not re.search(r"(?:subu|addu)\s+\$?sp,", prologue), prologue
    size_output = tool("size", "--format=berkeley", elf)
    match = re.search(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+\d+\s+\S+\s+", size_output, re.MULTILINE)
    assert match, size_output
    text_bytes, data_bytes, bss_bytes = map(int, match.groups())
    metrics = {"instances": count, "ps_exe_bytes": (build / "epok.ps-exe").stat().st_size,
               "text_bytes": text_bytes, "data_bytes": data_bytes, "bss_bytes": bss_bytes,
               "linked_resident_bytes": text_bytes + data_bytes + bss_bytes,
               "work_symbol_bytes": table.get(work_name, {}).get("bytes", 0),
               "work_prologue_frame_bytes": max(frames, default=0),
               "work_prologue": prologue, "guest_cycles": None}
    return metrics, table


def emulator(project, count, symbols):
    portable = project / f".epok/perf-emulator-{count}"
    portable.mkdir(parents=True, exist_ok=True)
    output = portable / "cycles.json"
    script = portable / "measure.lua"
    begin = symbols["blueprint_perf_begin"]["address"]
    end = symbols["blueprint_perf_end"]["address"]
    probe = symbols["blueprint_perf_probe"]["address"] & 0x1fffff
    warmup, samples = WARMUP_FRAMES * count, SAMPLE_FRAMES * count
    lua = '''local ffi=require('ffi')
local started=nil
local callbacks=0
local samples={}
local finished=false
local memory=PCSX.getMemPtr()
local probe=ffi.cast('volatile int32_t*',memory+PROBE)
local function finish()
  local file=assert(io.open(OUTPUT,'w'))
  file:write('{"callbacks":'..callbacks..',"cycles":[')
  for i,v in ipairs(samples) do if i>1 then file:write(',') end file:write(tostring(v)) end
  file:write('],"probe":[')
  for i=0,131 do if i>0 then file:write(',') end file:write(tostring(tonumber(probe[i]))) end
  file:write(']}');file:close()
  PCSX.quit(0)
end
perf_begin_breakpoint=PCSX.addBreakpoint(BEGIN,'Exec',4,'BP perf begin',function()
  if finished then return true end
  assert(started==nil,'Overlapping performance invocation')
  started=tonumber(PCSX.getCPUCycles())
  return true
end)
perf_end_breakpoint=PCSX.addBreakpoint(END,'Exec',4,'BP perf end',function()
  if finished then return true end
  assert(started~=nil,'Missing performance entry marker')
  local elapsed=tonumber(PCSX.getCPUCycles())-started
  started=nil;callbacks=callbacks+1
  if callbacks>WARMUP then samples[#samples+1]=elapsed end
  if #samples==SAMPLES then finished=true;finish() end
  return true
end)
'''.replace("PROBE", str(probe)).replace("OUTPUT", json.dumps(str(output).replace("\\", "/"))).replace("BEGIN", str(begin)).replace("END", str(end)).replace("WARMUP", str(warmup)).replace("SAMPLES", str(samples))
    write(script, lua)
    executable = next((path for path in (
        ROOT / ".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/PCSX-Redux",
        ROOT / ".tools/macos/redux/PCSX-Redux.app/Contents/MacOS/pcsx-redux",
        ROOT / ".tools/redux/pcsx-redux.exe",
        ROOT / ".tools/redux/pcsx-redux",
    ) if path.is_file()), None)
    assert executable, "PCSX-Redux is not installed under .tools"
    bios = next(iter(sorted((ROOT / ".tools").rglob("openbios.bin"))), None)
    assert bios, "openbios.bin is not installed under .tools"
    command = [str(executable), "-portable", str(portable), "-run", "-stdout", "-interpreter", "-softgpu",
               "-2mb", "-no-gdb", "-loadexe", str(project / ".epok/build/epok.ps-exe"),
               "-bios", str(bios), "-dofile", str(script), "-debugger"]
    with (portable / "emulator.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen(command, cwd=portable, stdout=log, stderr=log, stdin=subprocess.DEVNULL)
        try:
            assert process.wait(timeout=150) == 0, f"Emulator failed: {portable}"
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    assert output.is_file(), f"No CPU samples: {portable}"
    data = documents.loads(output.read_text(encoding="utf-8"))
    assert len(data["cycles"]) == samples and data["callbacks"] == warmup + samples, data
    assert all(cycle > 0 for cycle in data["cycles"]), data
    expected = 4096
    for _ in range((WARMUP_FRAMES + SAMPLE_FRAMES) * ITERATIONS):
        expected = expected * 4090 // 4096 + 16
    values = data["probe"]
    assert values[:2] == [0x42505031, warmup + samples], values
    for index in range(count):
        assert values[4 + index * 2:6 + index * 2] == [expected, (WARMUP_FRAMES + SAMPLE_FRAMES) * ITERATIONS], (index, values)
    ordered = sorted(data["cycles"])
    return {"samples": samples, "warmup_calls": warmup, "sum": sum(ordered),
            "min": ordered[0], "median": statistics.median(ordered),
            "p95": ordered[(len(ordered) * 95 - 1) // 100], "max": ordered[-1],
            "semantic_probe": values, "raw_samples": data["cycles"]}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emulator", action="store_true")
    parser.add_argument("--include-lua", action="store_true",
                        help="Also build/measure Gameplay-v2 NativeCpp and both VM modes")
    parser.add_argument("--keep", action="store_true")
    args = parser.parse_args()
    # Native script translation units use -Os. Match the generated main unit's
    # final optimization flag instead of comparing the default mixed -Os/-O2.
    os.environ["EPOK_RUNTIME_OPT"] = "-Os"
    with (contextlib.nullcontext(tempfile.mkdtemp(prefix="epok-bp-performance-")) if args.keep else tempfile.TemporaryDirectory(prefix="epok-bp-performance-")) as directory:
        directory = Path(directory)
        if args.keep:
            print(f"Retained performance projects: {directory}", flush=True)
        variants = ["cpp", "blueprint", *LUA_VARIANTS] if args.include_lua else ["cpp", "blueprint"]
        projects = {variant: create_project(directory, variant) for variant in variants}
        results = []
        for count in (1, 16, 64):
            pair = {"instances": count}
            for variant, (project, class_id, initial) in projects.items():
                metrics, symbols = build_instances(project, variant, class_id, initial, count)
                if args.emulator:
                    metrics["guest_cycles"] = emulator(project, count, symbols)
                pair[variant] = metrics
                print(f"Measured {variant}: {count} instances, text={metrics['text_bytes']}, BSS={metrics['bss_bytes']}", flush=True)
            measured_fields = ("ps_exe_bytes", "text_bytes", "data_bytes", "bss_bytes",
                               "linked_resident_bytes", "work_symbol_bytes",
                               "work_prologue_frame_bytes")
            for variant in variants[1:]:
                pair[f"{variant}_minus_cpp"] = {
                    key: pair[variant][key] - pair["cpp"][key]
                    for key in measured_fields
                }
            if args.emulator:
                cpp = pair["cpp"]["guest_cycles"]
                for variant in variants[1:]:
                    measured = pair[variant]["guest_cycles"]
                    assert cpp["semantic_probe"] == measured["semantic_probe"], \
                        f"Native/{variant} workload results differ"
                    pair[f"median_cycle_ratio_{variant}_over_cpp"] = measured["median"] / cpp["median"]
            results.append(pair)
        report = {"passed": True, "cpu_measured": args.emulator,
                  "optimization": {"native_scripts": "-Os", "generated_main": "-Os", "EPOK_RUNTIME_OPT": "-Os"},
                  "workload": {"loop_iterations": ITERATIONS,
                  "fixed_expression_raw": "value=(value*4090)/4096+16; steps=saturating_add(steps,1)",
                  "warmup_frames": WARMUP_FRAMES, "sample_frames": SAMPLE_FRAMES},
                  "limitations": ["Guest interpreter cycles between identical markers include dispatch, probe writes and any interrupts; not host time or FPS.",
                                  "Stack measurement is this function's declared MIPS prologue frame, not peak stack including callees/interrupts.",
                                  "Linked resident sections are text+data+BSS, not peak allocator or VRAM/SPU usage.",
                                  "Whole-binary differences include Blueprint class registration/runtime hooks; both workloads use identical deterministic arithmetic helpers.",
                                  "Cold-start warmup calls are excluded, raw measured cycle samples are retained."],
                  "variants": variants, "results": results}
        write(ROOT / "artifacts/blueprints/performance-validation.json", report)
        print("PASS: equivalent native/Blueprint workload and measured build" + ("/CPU" if args.emulator else " (CPU not requested)") + " costs", flush=True)


if __name__ == "__main__":
    main()
