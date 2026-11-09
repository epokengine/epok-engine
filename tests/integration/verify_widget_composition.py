"""Run nested C++/Blueprint/Lua widget behavior on PSX in each Lua mode.

Generate the ordinary project with the Native Play regression test, setting
EPOK_WIDGET_EVIDENCE_DIR to OUTPUT/project. The fixture contains two placed
containers, each with three reusable child classes and independently bound text.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import struct
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

EDITOR = ROOT / "target/debug/epok-editor.exe"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(out, name, command, **kwargs):
    with (out / name).open("w", encoding="utf-8") as log:
        result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=900, **kwargs)
    assert result.returncode == 0, (name, (out / name).read_text(encoding="utf-8")[-5000:])


def emulate(out, project, mode, components=False, unbound=False, focus_links=False, focus_lifetime=False):
    with socket.socket() as probe:
        if os.name == "nt":
            probe.setsockopt(socket.SOL_SOCKET, socket.SO_EXCLUSIVEADDRUSE, 1)
        probe.bind(("127.0.0.1", 8077))
    values = None
    expected = [0, 6, 0, 84, 0] if unbound else [7, 6, 1, 18 if components else 46, 0]
    symbol = "widget_component_probe" if components else "nested_widget_probe"
    if focus_links:
        expected = [7, 9, 1, 18, 0]
        symbol = "widget_focus_probe"
    if focus_lifetime:
        expected = [7, 3, 1, 15, 0]
        symbol = "widget_focus_probe"
    with (out / f"{mode}-play.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(EDITOR), "--project", str(project), "--play-psx", "--stop-after", "6"],
                                   cwd=ROOT, stdout=log, stderr=log,
                                   creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        try:
            deadline = time.monotonic() + 120
            while process.poll() is None and time.monotonic() < deadline:
                try:
                    with urllib.request.urlopen("http://127.0.0.1:8077/api/v1/cpu/ram/raw", timeout=2) as response:
                        ram = response.read()
                    mapping = (project / ".epok/build/epok.map").read_text()
                    address = int(re.findall(r"0x([0-9a-fA-F]+)\s+" + symbol + r"\s*$", mapping, re.M)[-1], 16) & 0x1fffff
                    values = list(struct.unpack_from("<5I", ram, address))
                    if values == expected:
                        break
                except (OSError, ValueError):
                    pass
                time.sleep(.1)
            assert values == expected, {"mode": mode, "actual": values, "expected": expected}
            assert process.wait(timeout=30) == 0
        finally:
            if process.poll() is None:
                if os.name == "nt":
                    subprocess.run(["taskkill.exe", "/PID", str(process.pid), "/T", "/F"], capture_output=True, check=False)
                else:
                    process.terminate()
                process.wait(timeout=15)
    return values


def standalone(out, project):
    result = subprocess.run([str(EDITOR), "--project", str(project), "--export-psx"], cwd=ROOT,
                            capture_output=True, text=True, timeout=900)
    (out / "export.log").write_text(result.stdout + result.stderr, encoding="utf-8")
    assert result.returncode == 0, result.stdout + result.stderr
    export = Path(result.stdout.strip())
    env = os.environ.copy()
    env["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + env["PATH"]
    run(out, "standalone-build.log", ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
        str(export / "build.ps1"), "-Make", str(ROOT / ".tools/mips/bin/make.exe"),
        "-Nugget", str(ROOT / "third_party/nugget")], cwd=export, env=env)
    original = project / ".epok/build/epok.ps-exe"
    assert (export / "epok.ps-exe").read_bytes() == original.read_bytes()
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    (out / "standalone-result.json").write_text(json.dumps({"mode": manifest["lua_execution"],
        "export": str(export), "identical": True, "editor_sha256": sha(EDITOR), "exe_sha256": sha(original)}, indent=2) + "\n")
    print("PASS standalone nested widget composition, byte-identical", manifest["lua_execution"], flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p4/widget-nesting-runtime")
    parser.add_argument("--mode", action="append", choices=("native_cpp", "vm_bytecode", "vm_source"))
    parser.add_argument("--standalone-only", action="store_true")
    parser.add_argument("--components", action="store_true", help="Verify six authored component instances in C++/Blueprint/Lua widget classes")
    parser.add_argument("--unbound-components", action="store_true", help="With --components, verify explicit None placement references leave all six labels unchanged")
    parser.add_argument("--focus-links", action="store_true", help="Verify persistent focus in six placed/duplicated and three dynamically spawned C++/Blueprint/Lua widgets")
    parser.add_argument("--focus-lifetime", action="store_true", help="Verify focus target destruction and slot reuse in C++/Blueprint/Lua widgets")
    args = parser.parse_args()
    if args.unbound_components and not args.components:
        parser.error("--unbound-components requires --components")
    if args.focus_links and (args.components or args.unbound_components):
        parser.error("--focus-links uses its own authored widget fixture")
    if args.focus_lifetime and (args.components or args.focus_links):
        parser.error("--focus-lifetime uses its own authored widget fixture")
    out = args.output.resolve()
    project = out / "project"
    out.mkdir(parents=True, exist_ok=True)
    assert project.is_dir(), "Generate the source fixture with EPOK_WIDGET_EVIDENCE_DIR first"
    if args.standalone_only:
        standalone(out, project)
        return
    descriptor = next(project.glob("*.epokproject"))
    results = []
    for mode in args.mode or ("native_cpp", "vm_bytecode", "vm_source"):
        manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
        manifest["lua_execution"] = mode
        documents.write_text(descriptor, documents.dumps(manifest))
        run(out, f"{mode}-build.log", [str(EDITOR), "--project", str(project), "--analyze-memory"], cwd=ROOT)
        values = emulate(out, project, mode, args.components, args.unbound_components, args.focus_links, args.focus_lifetime)
        build = project / ".epok/build"
        sources = {str(p.relative_to(project)).replace("\\", "/"): sha(p)
                   for folder in ("assets", "ProjectSettings") for p in (project / folder).rglob("*") if p.is_file()}
        results.append({"mode": mode, "probe": values, "editor_sha256": sha(EDITOR),
            "exe_sha256": sha(build / "epok.ps-exe"), "project_sources": sources,
            "memory": json.loads((build / "memory-report.json").read_text())["ram"]["used"],
            "runtime_sources": {name: sha(build / name) for name in
                ("epok.hpp", "object_model.hpp", "actor_tables.hpp", "hud_core.hpp", "hud.hpp", "lua_runtime.hpp")}})
        scope = ("Three widget classes in C++/Blueprint/Lua, duplicated with six independently bound C++/Blueprint/Lua components; authored properties and own behavior"
                 if args.components else "Two containers with six independently bound C++/Blueprint/Lua child widgets; behavior, local bounds and HUD output")
        if args.unbound_components:
            scope = "Six C++/Blueprint/Lua components with explicit None placement references survive class refresh and cooking; no text writes, six original labels and zero dropped primitives"
        if args.focus_links:
            scope = "Nine C++/Blueprint/Lua widget instances: placed, duplicated and dynamically spawned; own initial focus and navigation survive identity/slot remapping, with 18 glyphs and zero drops"
        if args.focus_lifetime:
            scope = "Destroy the focused child of each dynamically spawned C++/Blueprint/Lua widget; all three replacement controls reuse those slots with new generations and cannot inherit Canvas focus or incoming navigation; 15 glyphs and zero drops"
        (out / "result.json").write_text(json.dumps({"scope": scope,
            "cases": results}, indent=2) + "\n", encoding="utf-8")
        print("PASS", mode, "nested widget behavior and independent controls", values, flush=True)


if __name__ == "__main__":
    main()
