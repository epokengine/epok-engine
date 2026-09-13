"""Check real GPU clear packets for every Epok video mode and both parities."""
from pathlib import Path
import os
import subprocess
import tempfile
from verify_blueprint_runtime import host_compiler

ROOT = Path(__file__).resolve().parents[2]

def main():
    compiler, env = host_compiler()
    nugget = ROOT / "third_party/nugget"
    includes = [ROOT / "runtime", nugget, nugget / "third_party/EASTL/include",
                nugget / "third_party/EABase/include/Common"]
    with tempfile.TemporaryDirectory(prefix="epok-frame-clear-") as folder:
        folder = Path(folder)
        # The pinned EASTL targets GCC/MIPS; adapt only its standard containers
        # for MSVC. GPU primitives and fragment headers remain the real SDK.
        (folder / "EASTL").mkdir()
        (folder / "EASTL/array.h").write_text("#pragma once\n#include <array>\nnamespace eastl {using std::array;}\n")
        (folder / "EASTL/utility.h").write_text("#pragma once\n#include <utility>\nnamespace eastl {using std::forward;}\n")
        includes.insert(0, folder)
        target = folder / ("frame-clear.exe" if os.name == "nt" else "frame-clear")
        source = Path(__file__).with_name("test_frame_clear.cpp")
        if Path(compiler).name.lower() == "cl.exe":
            command = [compiler, "/nologo", "/std:c++20", "/EHsc", "/W3", str(source),
                       f"/Fe:{target}", f"/Fo:{folder / 'frame-clear.obj'}",
                       *[f"/I{p}" for p in includes]]
        else:
            command = [compiler, "-std=c++20", "-Wall", "-Wextra", str(source),
                       "-o", str(target), *[f"-I{p}" for p in includes]]
        subprocess.run(command, env=env, cwd=folder, check=True)
        subprocess.run([str(target)], cwd=folder, check=True, timeout=10)
    print("PASS all 10 video modes: clear packets, VRAM bounds and display-field masking")

if __name__ == "__main__":
    main()
