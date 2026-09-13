"""Cook, boot, capture and profile the authored seven-layer fireball example."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

from verify_blueprints import ROOT, documents, run, write
from verify_blueprint_performance import tool


def main():
    root = Path(tempfile.mkdtemp(prefix="epok-fireball-")) / "Game"
    shutil.copytree(ROOT / "examples/timeline-spell", root,
                    ignore=shutil.ignore_patterns(".epok", "exports", "Local.epokconfig"))
    # This Phase 3 fixture profiles the original presentation scene. The example
    # now opens Combat by default; its Blueprint consumer has a separate test.
    project_path = root / "Timeline Spell.epokproject"
    project = documents.loads(project_path.read_text())
    project["startup_scene"] = "assets/scenes/Main.epokmap"
    write(project_path, project)
    source = json.loads((root / "assets/Effects/Fireball.particle-effect.json").read_text())
    assert len(source["layers"]) == 7
    assert [m["name"] for m in source["timeline"]["markers"]] == ["Cast", "Launch", "Impact", "Aftermath"]
    marker_checks = ""
    for i, marker in enumerate(source["timeline"]["markers"]):
        value = int.from_bytes(hashlib.sha256(marker["id"].encode()).digest()[:8], "little")
        marker_checks += f"fireball_probe[{10+i}]=timeline::sequences.marker_revision(sequence,UINT64_C({value}));"
    cpp = root / "assets/scripts/FireballShowcase.cpp"
    text = cpp.read_text().replace('void FireballShowcase::update', '''extern "C" {
volatile uint32_t fireball_probe[16]={};
__attribute__((noinline,used)) void fireball_capture(){asm volatile("nop":::"memory");}
}
void FireballShowcase::update''')
    text = text.replace("if(!component)return;", """if(!component)return;
    static unsigned ticks=0;++ticks;
    if(performance_stats.simulation_scanlines>fireball_probe[14])fireball_probe[14]=performance_stats.simulation_scanlines;
    if(sprite_stats.dropped>fireball_probe[15])fireball_probe[15]=sprite_stats.dropped;
    unsigned phase=ticks==16?1:ticks==60?2:ticks==100?3:ticks==156?4:ticks==350?5:0;
    if(phase){
        auto sequence=effects::pool.sequence(component->playback);
        fireball_probe[0]=0x46495245;fireball_probe[1]=ticks;fireball_probe[2]=phase;
        fireball_probe[3]=particle_stats.alive;fireball_probe[4]=particle_stats.peak;
        fireball_probe[5]=particle_stats.spawned;fireball_probe[6]=particle_stats.dropped;
        fireball_probe[7]=effects::pool.stats.active;fireball_probe[8]=effects::pool.stats.completed;
        fireball_probe[9]=uint32_t(effects::pool.state(component->playback));
        MARKERS
        fireball_capture();
    }
""".replace("MARKERS", marker_checks))
    write(cpp, text)
    run("--project", root, "--build-psx")
    elf = root / ".epok/build/epok.elf"
    symbols = {m[3]: int(m[1], 16) for line in tool("nm", "-S", "--defined-only", elf).splitlines()
               if (m := re.match(r"^([0-9a-fA-F]+)\s+([0-9a-fA-F]+)\s+\S\s+(.+)$", line))}
    portable = root / ".epok/fireball-emulator"
    portable.mkdir(parents=True)
    script = portable / "capture.lua"
    write(script, '''local ffi=require('ffi')
local probe=ffi.cast('volatile uint32_t*',PCSX.getMemPtr()+PROBE)
fireball_break=PCSX.addBreakpoint(CAPTURE,'Exec',4,'Fireball stage capture',function()
 local phase=tonumber(probe[2]);local name=FOLDER..'/'..phase
 local file=assert(io.open(name..'.json','w'));file:write('[')
 for i=0,15 do if i>0 then file:write(',') end file:write(tostring(tonumber(probe[i]))) end
 file:write(']');file:close()
 local ss=PCSX.GPU.takeScreenShot()
 if ss.width>0 and ss.height>0 then
  local data=assert(io.open(name..'.raw','wb'));data:write(tostring(ss.data));data:close()
  local size=assert(io.open(name..'.size','w'));size:write(ss.width..' '..ss.height..' '..(tonumber(ss.bpp)==0 and 2 or 3));size:close()
 end
 if phase==5 then PCSX.quit(0) end
 return true
end)
'''.replace("PROBE", str(symbols["fireball_probe"] & 0x1fffff)).replace("CAPTURE", str(symbols["fireball_capture"]))
       .replace("FOLDER", json.dumps(portable.as_posix())))
    exe = ROOT / ".tools/redux/pcsx-redux.exe"
    with (portable / "emulator.log").open("w") as log:
        process = subprocess.Popen([str(exe), "-portable", str(portable), "-run", "-stdout", "-interpreter", "-softgpu", "-2mb",
            "-no-gdb", "-loadexe", str(root / ".epok/build/epok.ps-exe"), "-bios", str(exe.parent / "openbios.bin"),
            "-dofile", str(script), "-debugger"], cwd=portable, stdout=log, stderr=log, stdin=subprocess.DEVNULL,
            creationflags=0x08000000 if os.name == "nt" else 0)
        try:
            assert process.wait(timeout=150) == 0
        finally:
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    rows = [json.loads((portable / f"{i}.json").read_text()) for i in range(1, 6)]
    assert all(r[0] == 0x46495245 and r[6] == 0 and r[15] == 0 for r in rows), rows
    assert rows[3][10:14] == [1, 1, 1, 1], rows
    assert rows[-1][3] == 0 and rows[-1][7:10] == [0, 1, 3], rows
    from PIL import Image
    for i in range(1, 6):
        w, h, depth = map(int, (portable / f"{i}.size").read_text().split())
        raw = (portable / f"{i}.raw").read_bytes()
        if depth == 2:
            rgb = bytearray()
            for offset in range(0, w * h * 2, 2):
                pixel = int.from_bytes(raw[offset:offset+2], "little")
                rgb.extend(((pixel >> shift) & 31) * 255 // 31 for shift in (0, 5, 10))
        else:
            rgb = raw[:w*h*3]
        image = Image.frombytes("RGB", (w, h), bytes(rgb))
        image.save(ROOT / f"artifacts/timelines/fireball-stage-{i}.png")
    export = Path(run("--project", root, "--export-psx").strip())
    env = os.environ.copy()
    env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(export / "build.ps1"),
        "-Make", str(ROOT / ".tools/mips/bin/make.exe"), "-Nugget", str(ROOT / "third_party/nugget")], cwd=export,
        env=env, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert (export / "epok.ps-exe").read_bytes() == (root / ".epok/build/epok.ps-exe").read_bytes()
    report = {"project": str(root), "stages": rows, "standalone_identical": True,
              "maximum_completed_frame_simulation_scanlines": rows[-1][14]}
    write(ROOT / "artifacts/timelines/phase3-fireball.json", report)
    print("PASS authored fireball stages, markers, cleanup, budgets, MIPS, PCSX and identical standalone export:", report)


if __name__ == "__main__":
    main()
