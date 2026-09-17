"""Compare native/software/silent using a disposable copy of an installed demo.

Usage: python tests/integration/verify_native_music.py SOURCE_PROJECT
Never modifies the source project or controls an existing emulator process.
"""
from pathlib import Path
import json, os, re, shutil, socket, struct, subprocess, sys, time, urllib.request

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
PORT = 8079

def main():
    source = Path(sys.argv[1]).resolve()
    project = ROOT / ".epok" / f"native-driver-denise-{time.time_ns()}"
    out = ROOT / "artifacts" / project.name
    out.mkdir(parents=True)
    project.mkdir(parents=True)
    for name in ("assets", "ProjectSettings"):
        if (source/name).exists(): shutil.copytree(source/name, project/name)
    for p in source.glob("*.epokproject"): shutil.copy2(p, project/p.name)
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text())
    # Keep the comparison under the runtime heap/stack guard in both drivers.
    # Profiling HUD assets/code are disabled identically; counters stay enabled.
    for key in manifest.get("debug",{}):manifest["debug"][key]=False
    manifest["rendering"]["motion_interpolation"]=False
    documents.write_text(next(project.glob("*.epokproject")),documents.dumps(manifest))
    scene_path = project/manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text())
    music = next(c["properties"]["audio"] for actor in scene["actors"] if actor["name"]=="Music" for c in actor["components"] if "audio" in c.get("properties",{}))
    asset = project/"assets/Audio/bgm/hm-bonneforces-exe.epokasset"
    original = asset.read_bytes(); meta_len, source_len = struct.unpack_from("<II", original, 8)
    metadata = json.loads(original[16:16+meta_len]); payload = original[16+meta_len:]
    assert len(payload)==source_len and metadata["id"]==music["clip"]
    recipe = metadata["settings"]["options"]["target_overrides"]["psx"]["music_conversion"]
    print(f"Isolated project: {project}\nEvidence: {out}", flush=True)
    results = {}
    for label, driver, enabled in (("native", "NativeSpu", True), ("silent", "NativeSpu", False), ("software", "SoftwareReference", True)):
        recipe["driver"]=driver; encoded=json.dumps(metadata).encode()
        asset.write_bytes(b"EPOKAS01"+struct.pack("<II",len(encoded),len(payload))+encoded+payload)
        music["play_on_start"]=enabled;documents.write_text(scene_path,documents.dumps(scene))
        with (out/f"{label}-build.log").open("w") as log:
            p=subprocess.run([str(ROOT/"target/debug/epok-editor.exe"),"--project",str(project),"--build-psx"],cwd=ROOT,stdout=log,stderr=log,creationflags=FLAGS,timeout=300)
        assert p.returncode==0, (label,"build failed",out/f"{label}-build.log")
        build=project/".epok/build"; symbols=(build/"epok.map").read_text()
        report=json.loads((build/"audio/sequence-report.json").read_text())[music["clip"]]
        shutil.copy2(build/"epok.ps-exe",out/f"{label}.ps-exe")
        shutil.copy2(build/"epok.map",out/f"{label}.map")
        portable=project/".epok/emulator";portable.mkdir(exist_ok=True)
        with socket.socket() as s:s.bind(("127.0.0.1",PORT))
        def request(path,post=False):
            req=urllib.request.Request(f"http://127.0.0.1:{PORT}/api/v1/"+path,data=b"" if post else None)
            with urllib.request.urlopen(req,timeout=4) as r:return r.read()
        def address(name):
            m=re.search(r"0x([0-9a-f]+)\s+"+re.escape(name)+r"\b",symbols);assert m,name
            return int(m[1],16)&0x1fffff
        def snapshot():
            request("execution-flow?function=pause",True); ram=request("cpu/ram/raw")
            stats=struct.unpack_from("<16I",ram,address("epok::music_sequence_stats"))
            timing=struct.unpack_from("<Q",ram,address("epok::sequence_timing_stats")+16)[0]
            perf=struct.unpack_from("<2I",ram,address("epok::performance_stats"))
            request("execution-flow?function=resume",True)
            return dict(stats=stats,ticks=timing,performance=perf)
        with (out/f"{label}-emulator.log").open("w") as log:
            process=subprocess.Popen([str(ROOT/".tools/redux/pcsx-redux.main"),"-portable",str(portable),"-logfile",str(out/f"{label}-runtime.log"),"-run","-stdout","-fastboot","-noupdate","-interpreter","-softgpu","-2mb","-no-gdb","-no-ui","-webserver","-webserver-port",str(PORT),"-loadexe",str(build/"epok.ps-exe")],cwd=portable,stdout=log,stderr=log,creationflags=FLAGS)
            try:
                deadline=time.monotonic()+30
                while True:
                    assert process.poll() is None, ("emulator failed",label)
                    try:
                        if json.loads(request("execution-flow"))["running"]:break
                    except (OSError,ValueError):pass
                    assert time.monotonic()<deadline;time.sleep(.1)
                time.sleep(3); first=snapshot();time.sleep(7);last=snapshot()
                dt=last["stats"][15]-first["stats"][15]
                assert dt>1000000 and last["stats"][0]==1 and last["stats"][11]==0 and last["stats"][12]==0,last
                result=dict(first=first,last=last,clock_seconds=dt/1e6,report=report,
                    fps=(last["performance"][0]-first["performance"][0])*1e6/dt,
                    service_cpu_percent=(last["ticks"]-first["ticks"])*625/2646/dt*100,
                    irq_hz=(last["stats"][1]-first["stats"][1])*1e6/dt,
                    max_service_us=last["stats"][14]*625/2646)
                assert (last["stats"][2]>0)==enabled,last
                results[label]=result
                (out/"results.json").write_text(json.dumps(results,indent=2))
                print(label,json.dumps({k:v for k,v in result.items() if k not in ("report","first","last")}),flush=True)
            finally:
                process.terminate();process.wait(timeout=10)
    assert results["native"]["service_cpu_percent"]<results["software"]["service_cpu_percent"]
    print("PASS native music uses less measured sequence-service CPU",flush=True)

if __name__=="__main__":main()
