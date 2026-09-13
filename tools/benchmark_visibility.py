"""Optimized host CPU visibility experiment; does not estimate PSX FPS.

Builds a 168-chunk synthetic spatial mesh and compares the renderer AABB test
with and without caches for stationary, translating, rotating, and shared-instance
cameras. Includes exact narrow-product rounding, the wide fallback, and basis
product reuse under translation. Defers masks until the exact matrix repeats.
Reports three-run medians and requires identical accepted-chunk counts.
"""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
env = os.environ.copy()
compiler = shutil.which("g++") or shutil.which("clang++")
if not compiler and os.name == "nt":
    vswhere = Path(os.environ.get("ProgramFiles(x86)", "C:/Program Files (x86)")) / "Microsoft Visual Studio/Installer/vswhere.exe"
    installation = subprocess.check_output([str(vswhere), "-latest", "-products", "*", "-requires", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"], text=True).strip()
    vcvars = Path(installation) / "VC/Auxiliary/Build/vcvars64.bat"
    for line in subprocess.check_output(f'"{vcvars}" >nul && set', shell=True, text=True).splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            if key:
                env[key] = value
    compiler = shutil.which("cl.exe", path=env.get("Path", env.get("PATH")))
if not compiler:
    raise SystemExit("Install a C++20 host compiler to run this experiment.")
with tempfile.TemporaryDirectory(prefix="epok-visibility-benchmark-") as temp:
    folder = Path(temp)
    source = ROOT / "tests/runtime/visibility_benchmark.cpp"
    target = folder / ("benchmark.exe" if os.name == "nt" else "benchmark")
    if Path(compiler).name.lower() == "cl.exe":
        command = [compiler, "/nologo", "/std:c++20", "/O2", "/EHsc", str(source), f"/I{ROOT / 'runtime'}", f"/Fe:{target}", f"/Fo:{folder / 'benchmark.obj'}"]
    else:
        command = [compiler, "-std=c++20", "-O2", str(source), f"-I{ROOT / 'runtime'}", "-o", str(target)]
    result = subprocess.run(command, cwd=temp, env=env, capture_output=True, text=True)
    if result.returncode:
        raise SystemExit(result.stdout + result.stderr)
    subprocess.run([str(target)], check=True, timeout=60)
