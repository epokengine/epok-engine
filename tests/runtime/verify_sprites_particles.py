"""Run runtime animation/pool tests using the actual PsyQo fixed-point type."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
env = os.environ.copy()
compiler = shutil.which("g++") or shutil.which("clang++")
if not compiler and os.name == "nt":
    vswhere = Path(os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)")) / "Microsoft Visual Studio/Installer/vswhere.exe"
    installation = subprocess.check_output([str(vswhere), "-latest", "-products", "*", "-requires", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"], text=True).strip()
    vcvars = Path(installation) / "VC/Auxiliary/Build/vcvars64.bat"
    output = subprocess.check_output(f'"{vcvars}" >nul && set', shell=True, text=True)
    for line in output.splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            if key:
                env[key] = value
    compiler = shutil.which("cl.exe", path=env.get("Path", env.get("PATH")))
if not compiler:
    raise SystemExit("A C++20 compiler is required.")
with tempfile.TemporaryDirectory(prefix="epok-sprite-tests-") as temp:
    path = Path(temp)
    # PsyQo only requires EASTL's function declaration for unused text formatting.
    (path / "EASTL").mkdir()
    documents.write_text(path / "EASTL/functional.h", "#pragma once\n#include <functional>\nnamespace eastl {using std::function;}\n")
    documents.write_text(path / "display.hh", "#pragma once\nnamespace epok {inline constexpr int display_width=320,display_height=240;}\n")
    target = path / ("sprites.exe" if os.name == "nt" else "sprites")
    source = ROOT / "tests/runtime/sprites_particles.cpp"
    includes = [path, ROOT / "third_party/nugget", ROOT / "runtime"]
    if Path(compiler).name.lower() == "cl.exe":
        command = [compiler, "/nologo", "/std:c++20", "/EHsc", "/W3", str(source), f"/Fe:{target}", f"/Fo:{path / 'sprites.obj'}", *[f"/I{p}" for p in includes]]
    else:
        command = [compiler, "-std=c++20", "-Wall", "-Wextra", str(source), "-o", str(target), *[f"-I{p}" for p in includes]]
    subprocess.run(command, cwd=temp, env=env, check=True)
    subprocess.run([str(target)], cwd=temp, check=True, timeout=30)
