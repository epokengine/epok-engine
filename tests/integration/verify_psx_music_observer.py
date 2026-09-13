"""Observe guest clock/stack/heap without adding instructions to the PSX binary.

Uses the documented PCSX-Redux Lua FFI/breakpoint APIs:
https://pcsx-redux.consoledev.net/Lua/memory-and-registers/
https://pcsx-redux.consoledev.net/Lua/breakpoints/
Pass an existing isolated music test build; this does not modify its ELF/assets.
"""
from pathlib import Path
import argparse
import hashlib
import json
import re
import socket
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("build", type=Path)
    parser.add_argument("--seconds", type=int, default=228)
    args = parser.parse_args()
    build = args.build.resolve()
    assert build.is_relative_to(ROOT / ".epok") and 5 <= args.seconds <= 240
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 8077))
    symbols = (build / "epok.map").read_text(encoding="utf-8")

    def address(name):
        return int(re.search(r"0x([0-9a-f]+)\s+" + re.escape(name), symbols)[1], 16)

    heap_base = address("__heap_start")
    heap_metadata = int(re.search(r"\.bss\.s_heap_metadata\s+0x([0-9a-f]+)", symbols)[1], 16)
    disassembly = subprocess.check_output(
        [str(ROOT / ".tools/mips/bin/mipsel-none-elf-objdump.exe"), "-d", "--no-show-raw-insn", str(build / "epok.elf")], text=True)
    adjustments = [(int(pc, 16), int(size)) for pc, size in re.findall(
        r"^\s*([0-9a-f]+):\s+addiu\s+sp,sp,-(\d+)\s*$", disassembly, re.M)]
    assert adjustments, "No stack allocation instructions found"
    # Variable-sized stack allocation needs a separate observer, not a zero cost.
    dynamic = re.findall(r"^\s*[0-9a-f]+:\s+(?:subu|addu)\s+sp,sp,.*$", disassembly, re.M)
    assert not dynamic, f"Unaccounted dynamic stack allocations: {dynamic}"
    out = ROOT / "artifacts/midi-completion" / f"p4-observer-{time.time_ns()}"
    out.mkdir(parents=True)
    output = out / "observations.json"
    lua = r'''local ffi=require('ffi')
local memory=PCSX.getMemPtr()
local regs=PCSX.getRegisters()
local stats=ffi.cast('volatile uint32_t*',memory+STATS)
local timing=ffi.cast('volatile uint32_t*',memory+TIMING)
local service_ticks=ffi.cast('volatile uint64_t*',memory+TIMING+16)
local heap=ffi.cast('volatile uint32_t*',memory+HEAP_METADATA)
local stack_min=0x80200000
local stack_calls=0
local previous_cycle=nil
local elapsed_cycles=0
local first_clock=nil
local max_drift=0
local frames=0
local done=false
local watch={}
local adjustments=ADJUSTMENTS
for _,entry in ipairs(adjustments) do
  local size=entry[2]
  watch[#watch+1]=PCSX.addBreakpoint(entry[1],'Exec',4,'Music stack observer',function()
    local sp=tonumber(regs.GPR.n.sp)-size
    if sp>=0x80010000 and sp<0x80200000 and sp<stack_min then stack_min=sp end
    stack_calls=stack_calls+1
    return true
  end)
end
music_clock_observer=PCSX.addBreakpoint(FRAME,'Exec',4,'Music clock observer',function()
  if done or tonumber(stats[0])~=1 or tonumber(stats[10])~=1 then return true end
  frames=frames+1
  local cycles=tonumber(PCSX.getCPUCycles())
  local clock=tonumber(stats[15])
  if previous_cycle then
    local delta=cycles-previous_cycle
    if delta<0 then delta=delta+4294967296 end
    elapsed_cycles=elapsed_cycles+delta
    local drift=math.abs((clock-first_clock)-elapsed_cycles/33.8688)
    if drift>max_drift then max_drift=drift end
  else first_clock=clock end
  previous_cycle=cycles
  if elapsed_cycles/33.8688>=DURATION then
    done=true
    local high=tonumber(heap[3])
    local file=assert(io.open(OUTPUT,'w'))
    file:write(string.format('{'..
      '"frames":%d,"guest_cycles":%.0f,"musical_elapsed_us":%d,"maximum_clock_drift_us":%.9f,'..
      '"loops":%d,"service_gap_us":%d,"stack_min_address":%.0f,"stack_peak_bytes":%d,'..
      '"stack_callbacks":%d,"heap_high_address":%.0f,"heap_peak_extent_bytes":%d,'..
      '"service_max_us":%.9f,"service_average_us":%.9f,"service_cpu_percent":%.9f,'..
      '"key_on_max_delay_us":%d,"key_ons":%d,"physical_peak":%d,'..
      '"steals":%d,"denied":%d,"capacity_errors":%d,"pitch_clamps":%d,"error":%d,"clock_faults":%d}',
      frames,elapsed_cycles,clock-first_clock,max_drift,tonumber(stats[4]),tonumber(stats[13]),
      stack_min,0x80200000-stack_min,stack_calls,high,math.max(0,high-HEAP_BASE),
      tonumber(stats[14])*625/2646,tonumber(service_ticks[0])*625/2646/math.max(1,tonumber(stats[1])),
      tonumber(service_ticks[0])*625/2646/math.max(1,clock)*100,
      tonumber(timing[1]),tonumber(timing[2]),tonumber(stats[9]),tonumber(stats[5]),tonumber(stats[6]),
      tonumber(stats[7]),tonumber(stats[8]),tonumber(stats[11]),tonumber(stats[12])))
    file:close();PCSX.quit(0)
  end
  return true
end)
'''
    values = dict(STATS=address("epok::music_sequence_stats") & 0x1fffff,
                  TIMING=address("epok::sequence_timing_stats") & 0x1fffff,
                  HEAP_METADATA=heap_metadata & 0x1fffff, HEAP_BASE=heap_base,
                  FRAME=address("Spinner::update("), DURATION=args.seconds * 1_000_000,
                  OUTPUT=json.dumps(output.as_posix()),
                  ADJUSTMENTS="{" + ",".join("{" + f"{pc},{size}" + "}" for pc, size in adjustments) + "}")
    for token, value in values.items():
        lua = lua.replace(token, str(value))
    script = out / "observe.lua"
    script.write_text(lua, encoding="utf-8")
    (out / "scope.json").write_text(json.dumps(dict(
        build=str(build), executable_sha256=hashlib.sha256((build / "epok.ps-exe").read_bytes()).hexdigest(),
        stack_prologues=len(adjustments), heap_base=heap_base,
        scope="Guest executable downward stack adjustments, including startup and IRQ functions; BIOS-private stacks are outside this measurement. Heap extent includes allocator headers/fragmentation; it is not live payload bytes."), indent=2), encoding="utf-8")
    print(f"Evidence: {out}", flush=True)
    with (out / "emulator.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(ROOT / ".tools/redux/pcsx-redux.main"), "-portable", str(out),
            "-run", "-stdout", "-fastboot", "-noupdate", "-interpreter", "-softgpu", "-2mb",
            "-no-ui", "-no-gdb", "-webserver", "-webserver-port", "8077", "-debugger",
            "-bios", str(ROOT / ".tools/redux/openbios-fastboot.bin"),
            "-loadexe", str(build / "epok.ps-exe"), "-dofile", str(script)],
            cwd=out, stdout=log, stderr=log, creationflags=FLAGS)
        try:
            assert process.wait(timeout=args.seconds * 6 + 90) == 0
        finally:
            if process.poll() is None:
                process.terminate();process.wait(timeout=10)
    report = json.loads(output.read_text(encoding="utf-8"))
    print(json.dumps(report), flush=True)
    assert report["maximum_clock_drift_us"] <= report["service_gap_us"], "Independent guest-clock drift"
    assert report["service_max_us"] <= 2000 and report["service_gap_us"] <= 3000, "Normal service CPU/gap gate"
    assert report["key_on_max_delay_us"] <= 3100, "Actual key-on deadline"
    assert not any(report[k] for k in ("steals", "denied", "capacity_errors", "pitch_clamps", "error", "clock_faults")), "Musical loss or fault"
    assert heap_base <= report["heap_high_address"] < report["stack_min_address"], "Heap/stack overlap"
    if args.seconds >= 225:
        assert report["loops"] >= 2, "Two full loops required"
    print("PASS guest clock and executable stack/heap observation", flush=True)


if __name__ == "__main__":
    main()
