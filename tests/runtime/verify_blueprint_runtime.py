"""Compile real Blueprint/PsyQo headers and run deterministic host contract tests.

Only display constants and EASTL's host-incompatible callback wrapper are adapted
for host tests, matching verify_spatial.py. MIPS syntax validation additionally
uses the real EASTL headers and pinned target compiler; no emulator is launched.
"""
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


def host_compiler():
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
        raise SystemExit("No C++20 host compiler found (g++, clang++, or Visual Studio C++ tools required).")
    return compiler, env


def main():
    compiler, env = host_compiler()
    with tempfile.TemporaryDirectory(prefix="epok-blueprint-runtime-") as temp:
        path = Path(temp)
        (path / "EASTL").mkdir()
        documents.write_text(path / "EASTL/functional.h", "#pragma once\n#include <functional>\nnamespace eastl {using std::function;}\n")
        documents.write_text(path / "display.hh", "#pragma once\nnamespace epok {inline constexpr int display_width=320,display_height=240;}\n")
        includes = [path, ROOT / "third_party/nugget", ROOT / "runtime", ROOT / "templates"]
        cases = [("test_blueprint_runtime", 0), ("test_blueprint_runtime", 1), ("test_blueprint_spawn", 1), ("test_actor_tables", 0), ("test_timeline", 0), ("test_timeline_director", 0), ("test_timeline_adapters", 0), ("test_particle_effect", 0), ("test_blueprint_playback", 0)]
        for test, tracing in [case for case in cases if len(_sys.argv)==1 or case[0] in _sys.argv[1:]]:
            source = ROOT / f"tests/runtime/{test}.cpp"
            target = path / (f"{test}-{tracing}" + (".exe" if os.name == "nt" else ""))
            if Path(compiler).name.lower() == "cl.exe":
                command = [compiler, "/nologo", "/std:c++20", "/EHsc", "/W3", f"/DEPOK_BLUEPRINT_TRACE={tracing}", str(source), f"/Fe:{target}", f"/Fo:{path / (test + '-' + str(tracing) + '.obj')}", *[f"/I{p}" for p in includes]]
            else:
                command = [compiler, "-std=c++20", "-Wall", "-Wextra", "-fno-rtti", "-fno-exceptions", f"-DEPOK_BLUEPRINT_TRACE={tracing}", str(source), "-o", str(target), *[f"-I{p}" for p in includes]]
            subprocess.run(command, cwd=temp, env=env, check=True)
            subprocess.run([str(target)], cwd=temp, check=True, timeout=30)
        target_compiler = ROOT / ".tools/mips/bin/mipsel-none-elf-g++.exe"
        if target_compiler.is_file():
            # This separate TU uses no test doubles, assertions, STL containers,
            # or host-only services. Template instantiation catches target errors.
            target_source = path / "target.cpp"
            documents.write_text(target_source, '#define EPOK_BLUEPRINTS 1\n#define EPOK_BLUEPRINT_TRACE 1\n#include "blueprint_template.hpp"\n#include "blueprint_api.hpp"\n#include "blueprint_debug.hpp"\n'
                                     'template class epok::bp::Continuations<8>;\n'
                                     '#include "timeline_runtime.hpp"\ntemplate class epok::timeline::Director<8>;\n'
                                     '#include "particle_effect_runtime.hpp"\n'
                                     '#include "TimelineAdapters.hpp"\n'
                                     'template class epok::bp::Timeline<16>;\n'
                                     'template class epok::bp::TraceRing<128>;\n'
                                     'template class epok::bp::Debugger<32>;\n'
                                     'struct Test:epok::ActorComponent {void tick(epok::Fixed)override{}};\n'
                                     'template struct epok::ObjectPool<Test,4>;\n'
                                     'epok::Fixed probe(epok::Fixed a, epok::Fixed b) { return epok::bp::mul(a,b); }\n')
            nugget = ROOT / "third_party/nugget"
            target_includes = [nugget / "third_party/EASTL/include", nugget / "third_party/EABase/include/Common", path, nugget, ROOT / "runtime", ROOT / "templates"]
            subprocess.run([str(target_compiler), "-std=c++20", "-mips1", "-mabi=32", "-EL", "-msoft-float", "-ffreestanding", "-fno-rtti", "-fno-exceptions", "-fsyntax-only", str(target_source), *[f"-I{p}" for p in target_includes]], check=True, cwd=temp)
            print("Blueprint runtime compiled against real pinned MIPS/PsyQo/EASTL headers.")
        else:
            print("MIPS header validation skipped: run SDK setup to install the pinned target compiler.")


if __name__ == "__main__":
    main()
