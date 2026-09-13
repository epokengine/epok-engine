"""Phase 0: reflected compiled track/marker versus host and handwritten MIPS.

Uses isolated projects and one emulator at a time. Measures guest interpreter
cycles and linked sections; does not claim FPS, peak stack, or hardware timing.
"""
import argparse
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import tempfile

from verify_blueprints import ROOT, identity, run, write
from verify_blueprint_performance import tool

KEYS = [(0, -40961), (2048, 137), (4096, 40001)]


def expected(tick):
    tick = min(tick, 4096)
    for (a, x), (b, y) in zip(KEYS, KEYS[1:]):
        if tick < b:
            return x + (y - x) * (tick - a) // (b - a)
    return KEYS[-1][1]


def create(directory, variant):
    root = directory / variant
    run("--create-project", root, "--name", "Timeline Phase Zero")
    class_id, property_id, marker_id = identity(), identity(), identity()
    header = f'''#pragma once
#include "epok.hpp"
class EPOK_CLASS(Blueprintable,Id="{class_id}") TimelineProbe : public epok::Behaviour {{
public:
    EPOK_PROPERTY(EditAnywhere,TimelineAnimatable,Id="{property_id}") epok::Fixed progress=0.0;
    EPOK_PROPERTY(EditAnywhere) epok::Fixed editable_only=0.0;
    int32_t elapsed=0; uint16_t marker_cursor=0;
    void update(epok::Transform&,epok::Fixed) override;
}};
'''
    write(root / "assets/scripts/Probe.hpp", header)
    manifest = json.loads(run("--project", root, "--reflect"))
    cls = next(c for c in manifest["classes"] if c["id"] == class_id)
    target = next(p for p in cls["properties"] if p["id"] == property_id)
    assert target["timeline"]["native_field"]["interpolation"] == ["linear", "step", "smoothstep", "ease_in", "ease_out"], target
    assert next(p for p in cls["properties"] if p["name"] == "editable_only")["timeline"] is None
    write(root / "assets/scripts/Probe.hpp", header.replace("EditAnywhere,TimelineAnimatable", "ReadOnly,TimelineAnimatable"))
    assert "TimelineAnimatable requires" in run("--project", root, "--reflect", ok=False)
    write(root / "assets/scripts/Probe.hpp", header)
    recovered = json.loads(run("--project", root, "--reflect"))
    assert any(p["id"] == property_id for c in recovered["classes"] for p in c["properties"])
    # The prototype emits a typed field assignment after resolving the stable
    # property ID. No untyped offsets, name-based runtime dispatch or field pointer.
    table = ",".join(f"{{{t},{v}}}" for t, v in KEYS)
    work = f'''static constexpr epok::timeline::Key keys[]={{{table}}};
    static constexpr epok::timeline::Marker markers[]={{{{2048,1}}}};
    object.{target['name']}=epok::Fixed(epok::timeline::sample({{keys,3}},object.elapsed),epok::Fixed::RAW);
    uint16_t marker;
    while(epok::timeline::poll_marker(markers,1,object.marker_cursor,object.elapsed,marker))
        timeline_probe[2]=timeline_probe[2]+marker;
'''
    if variant == "handwritten":
        work = '''int64_t value;
    if(object.elapsed<2048)value=-40961+int64_t(41098)*object.elapsed/2048;
    else value=137+int64_t(39864)*(object.elapsed-2048)/2048;
    object.progress=epok::Fixed(int32_t(value),epok::Fixed::RAW);
    if(!object.marker_cursor&&object.elapsed>=2048){object.marker_cursor=1;timeline_probe[2]=timeline_probe[2]+1;}
'''
    source = '''#include "Probe.hpp"
#include "timeline.hpp"
extern "C" {
volatile int32_t timeline_probe[132]={};
__attribute__((noinline,used)) void timeline_begin(){asm volatile("nop":::"memory");}
__attribute__((noinline,used)) void timeline_end(){asm volatile("nop\\nnop":::"memory");}
__attribute__((noinline,used)) void timeline_work(TimelineProbe& object){
    WORK
}
}
void TimelineProbe::update(epok::Transform&,epok::Fixed){
    if(elapsed<4096){elapsed+=68;if(elapsed>4096)elapsed=4096;}
    timeline_begin();timeline_work(*this);timeline_end();
    const int32_t frame=timeline_probe[1];
    if(frame<64)timeline_probe[4+frame]=progress.raw();
    if(frame<64){
        // Profile parity runs outside the measured work interval in both variants.
        static constexpr epok::timeline::Key profile_keys[]={{0,-40961},{2048,137},{4096,40001}};
        timeline_probe[68+frame]=epok::timeline::sample({profile_keys,3,epok::timeline::Interpolation(frame%5),bool(frame%2)},(frame*503)%4097);
    }
    timeline_probe[0]=0x544c5030;
    timeline_probe[1]=frame+1;
    timeline_probe[3]=sizeof(TimelineProbe);
}
'''.replace("WORK", work)
    write(root / "assets/scripts/Probe.cpp", source)
    write(root / "prototype-identity.json", {"class": class_id, "property": property_id, "marker": marker_id})
    scene_path = root / "assets/scenes/Main.epokmap"
    import epok_documents as documents
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["entities"][0]["script"] = {"name": "TimelineProbe", "class_id": class_id,
        "provider": {"id": "cpp", "version": 1}, "backend": {"id": "native", "version": 1}, "properties": {}}
    write(scene_path, scene)
    run("--project", root, "--build-psx")
    elf = root / ".epok/build/epok.elf"
    symbols = {}
    for line in tool("nm", "-S", "--defined-only", elf).splitlines():
        match = re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line)
        if match:
            symbols[match[3]] = (int(match[1], 16), int(match[2], 16))
    size = tool("size", "--format=berkeley", elf)
    match = re.search(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+\d+\s+\S+\s+", size, re.MULTILINE)
    assert match, size
    metrics = dict(zip(("text", "data", "bss"), map(int, match.groups())))
    metrics.update(work_code_bytes=symbols["timeline_work"][1],
                   ps_exe_bytes=(root / ".epok/build/epok.ps-exe").stat().st_size)
    export = Path(run("--project", root, "--export-psx").strip())
    env = os.environ.copy()
    env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
        "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")],
        cwd=export, env=env, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert (export / "epok.ps-exe").read_bytes() == (root / ".epok/build/epok.ps-exe").read_bytes()
    metrics["standalone_identical"] = True
    return root, symbols, metrics


def emulator(root, symbols, host_profiles):
    portable = root / ".epok/timeline-emulator"
    portable.mkdir(parents=True)
    output = portable / "measurements.json"
    script = portable / "measure.lua"
    lua = '''local ffi=require('ffi')
local probe=ffi.cast('volatile int32_t*',PCSX.getMemPtr()+PROBE)
local first=nil
local cycles={}
local finished=false
timeline_first=PCSX.addBreakpoint(BEGIN,'Exec',4,'Timeline begin',function()
  if tonumber(probe[1])==130 then
    if finished then return true end
    finished=true
    local file=assert(io.open(OUTPUT,'w'))
    file:write('{"probe":[')
    for i=0,131 do if i>0 then file:write(',') end file:write(tostring(tonumber(probe[i]))) end
    file:write('],"cycles":[')
    for i,v in ipairs(cycles) do if i>1 then file:write(',') end file:write(tostring(v)) end
    file:write(']}');file:close();PCSX.quit(0);return true
  end
  first=tonumber(PCSX.getCPUCycles());return true
end)
timeline_last=PCSX.addBreakpoint(END,'Exec',4,'Timeline end',function()
  if first then cycles[#cycles+1]=tonumber(PCSX.getCPUCycles())-first;first=nil end
  return true
end)
'''.replace("PROBE", str(symbols["timeline_probe"][0] & 0x1fffff)).replace("BEGIN", str(symbols["timeline_begin"][0])).replace("END", str(symbols["timeline_end"][0])).replace("OUTPUT", json.dumps(output.as_posix()))
    write(script, lua)
    executable = ROOT / ".tools/redux/pcsx-redux.exe"
    with (portable / "emulator.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(executable), "-portable", str(portable), "-run", "-stdout", "-interpreter", "-softgpu",
            "-2mb", "-no-gdb", "-loadexe", str(root / ".epok/build/epok.ps-exe"), "-bios", str(executable.parent / "openbios.bin"),
            "-dofile", str(script), "-debugger"], cwd=portable, stdout=log, stderr=log, stdin=subprocess.DEVNULL,
            creationflags=0x08000000 if os.name == "nt" else 0)
        try:
            assert process.wait(timeout=150) == 0
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    data = json.loads(output.read_text())
    assert data["probe"][:3] == [0x544c5030, 130, 1], data
    assert data["probe"][4:68] == [expected((i + 1) * 68) for i in range(64)], data
    assert data["probe"][68:132] == host_profiles, "Typed/eased host versus MIPS mismatch"
    assert len(data["cycles"]) == 130 and min(data["cycles"]) > 0
    data["active_median_cycles"] = statistics.median(data["cycles"][10:60])
    data["instance_bytes"] = data["probe"][3]
    return data


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emulator", action="store_true")
    args = parser.parse_args()
    os.environ["EPOK_RUNTIME_OPT"] = "-Os"
    directory = Path(tempfile.mkdtemp(prefix="epok-timeline-prototype-"))
    print(f"Retained prototype: {directory}", flush=True)
    host_source = directory / "host.rs"
    write(host_source, '#[path=' + json.dumps((ROOT / "src/timeline_curve.rs").as_posix()) + '] mod timeline_curve;\n'
        'fn main(){let keys=[(0,-40961),(2048,137),(4096,40001)];'
        'for i in 1..=64{println!("{}",timeline_curve::sample(&keys,i*68));}'
        'for i in 0..64{println!("{}",timeline_curve::sample_mode(&keys,(i*503)%4097,(i%5) as u8,i%2!=0));}}\n')
    host_binary = directory / ("host.exe" if os.name == "nt" else "host")
    subprocess.run(["rustc", "--edition=2024", str(host_source), "-o", str(host_binary)], cwd=ROOT, check=True, timeout=60)
    host_output = list(map(int, subprocess.check_output([str(host_binary)], text=True).splitlines()))
    host_values, host_profiles = host_output[:64], host_output[64:]
    assert host_values == [expected((i + 1) * 68) for i in range(64)]
    report = {}
    for variant in ("compiled", "handwritten"):
        root, symbols, metrics = create(directory, variant)
        if args.emulator:
            metrics["emulator"] = emulator(root, symbols, host_profiles)
            assert metrics["emulator"]["probe"][4:68] == host_values
        report[variant] = metrics
        print(f"PASS {variant}: {metrics['work_code_bytes']} bytes work code; standalone rebuild identical", flush=True)
    if args.emulator:
        assert report["compiled"]["emulator"]["probe"] == report["handwritten"]["emulator"]["probe"]
    report["fixture"] = str(directory)
    write(ROOT / "artifacts/timelines/phase0.json", report)
    print("PASS: explicit reflection opt-in, host Q12 samples, MIPS, standalone exports" + (", emulator parity and CPU measurements" if args.emulator else "; emulator not requested"))


if __name__ == "__main__":
    main()
