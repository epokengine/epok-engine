"""Check serial readiness against real PCDrv startup reads in a compiled PSX game.

Uses a private Redux instance/cards and a BIOS puts breakpoint for ordering only.
This is not an FPS benchmark and does not change the game or send serial commands.
"""
import argparse
import json
import re
import socket
import subprocess
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-dir", type=Path, required=True)
    parser.add_argument("--emulator", type=Path, default=ROOT / ".tools/redux/pcsx-redux.main")
    parser.add_argument("--bios", type=Path, default=ROOT / ".tools/redux/openbios-fastboot.bin")
    parser.add_argument("--marker", choices=("runtime", "startup"), default="startup")
    parser.add_argument("--expected-reads", type=int, default=4)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    build = args.build_dir.resolve(strict=True)
    mapping_path = build / "epok.map"
    if not mapping_path.exists():
        mapping_path = build / "debug/epok.map"
    mapping = mapping_path.read_text()

    def address(name):
        matches = re.findall(r"0x([0-9a-fA-F]+)\s+" + re.escape(name) + r"\s*$", mapping, re.M)
        assert matches, name
        return int(matches[-1], 16) & 0x1fffff

    out = args.out or ROOT / "artifacts/serial-startup" / str(time.time_ns())
    out = out.resolve()
    portable = out / "emulator"
    portable.mkdir(parents=True)
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    fixture = out / "observe.lua"
    fixture.write_text("""
local ffi=require('ffi')
local mem=PCSX.getMemPtr()
local regs=PCSX.getRegisters()
local stream=ffi.cast('uint32_t*',mem+@STREAM@)
local failed=ffi.cast('uint8_t*',mem+@FAILED@)
local snapshots={}
EPOK_STARTUP_PUTS=PCSX.addBreakpoint(0xa0,'Exec',4,'Serial startup ordering',function()
 if tonumber(regs.GPR.n.t1)~=0x3e then return true end
 local p=bit.band(tonumber(regs.GPR.n.a0),0x1fffff)
 local text=ffi.string(mem+p)
 local marker=text:match('^EPOK: (%a+) ready')
 if marker=='runtime' or marker=='startup' then
  snapshots[#snapshots+1]=string.format('{"marker":"%s","reads":%d,"bytes":%d,"errors":%d,"timeouts":%d,"failed":%d}',marker,tonumber(stream[0]),tonumber(stream[1]),tonumber(stream[4]),tonumber(stream[5]),tonumber(failed[0]))
  if marker=='@MARKER@' then PCSX.pauseEmulator() end
 end
 return true
end)
PCSX.WebServer=PCSX.WebServer or {};PCSX.WebServer.Handlers=PCSX.WebServer.Handlers or {}
PCSX.WebServer.Handlers['serial-startup']=function()return '['..table.concat(snapshots,',')..']' end
""".replace("@STREAM@", str(address("epok::streaming_stats")))
        .replace("@FAILED@", str(address("epok::stream_failed")))
        .replace("@MARKER@", args.marker), encoding="utf-8")
    command = [str(args.emulator.resolve()), "-portable", str(portable),
               "-logfile", str(out / "runtime.log"), "-run", "-stdout", "-fastboot",
               "-noupdate", "-interpreter", "-softgpu", "-2mb", "-no-gdb", "-no-ui",
               "-debugger", "-webserver", "-webserver-port", str(port),
               "-bios", str(args.bios.resolve()), "-loadexe", str(build / "epok.ps-exe"),
               "-pcdrv", "-pcdrvbase", str(build), "-dofile", str(fixture)]
    with (out / "process.log").open("w") as log:
        process = subprocess.Popen(command, cwd=portable, stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 30
            while True:
                assert process.poll() is None, (out / "process.log").read_text()[-3000:]
                assert time.monotonic() < deadline, "No readiness marker; inspect " + str(out)
                try:
                    snapshots = json.load(urllib.request.urlopen(
                        f"http://127.0.0.1:{port}/api/v1/lua/serial-startup", timeout=2))
                except OSError:
                    time.sleep(.1)
                    continue
                final = next((s for s in snapshots if s["marker"] == args.marker), None)
                if final:
                    break
                time.sleep(.1)
            accepted = (final["reads"] == args.expected_reads and
                        final["bytes"] == args.expected_reads * 65536 and
                        final["errors"] == final["timeouts"] == final["failed"] == 0)
            result = dict(accepted=accepted, expected_reads=args.expected_reads, snapshots=snapshots)
            (out / "result.json").write_text(json.dumps(result, indent=2))
            print(json.dumps(result, indent=2), flush=True)
            print("Evidence:", out, flush=True)
            assert accepted, "Readiness announced before PCDrv startup completed"
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)


if __name__ == "__main__":
    main()
