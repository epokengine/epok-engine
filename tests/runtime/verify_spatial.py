"""Run input/collision using PsyQo Fixed and lifecycle/scene service host tests."""
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
SOURCES = [ROOT / "tests/runtime/spatial.cpp", ROOT / "tests/runtime/transform_cache.cpp", ROOT / "tests/runtime/lifecycle.cpp", ROOT / "tests/runtime/memory_card.cpp", ROOT / "tests/runtime/utility.cpp", ROOT / "tests/runtime/palette.cpp"]
SOURCES.append(ROOT / "tests/runtime/gameplay_api.cpp")
SOURCES.append(ROOT / "tests/runtime/navigation.cpp")
SOURCES.append(ROOT / "tests/runtime/terrain_collision.cpp")
SOURCES.append(ROOT / "tests/runtime/navigation_motion.cpp")
SOURCES.append(ROOT / "tests/runtime/frustum.cpp")
SOURCES.append(ROOT / "tests/runtime/visibility.cpp")
SOURCES.append(ROOT / "tests/runtime/polygon.cpp")
SOURCES.append(ROOT / "tests/runtime/streaming_pool.cpp")
SOURCES.append(ROOT / "tests/runtime/streaming_backend.cpp")
SOURCES.append(ROOT / "tests/runtime/streaming_disabled.cpp")
SOURCES.append(ROOT / "tests/runtime/streaming_archive_fits.cpp")
SOURCES.append(ROOT / "tests/runtime/transition.cpp")
SOURCES.append(ROOT / "tests/runtime/scene_transitions.cpp")
SOURCES.append(ROOT / "tests/runtime/streaming_host.cpp")
SOURCES.append(ROOT / "tests/runtime/motion_interpolation.cpp")
SOURCES.append(ROOT / "tests/runtime/serial_kernel.cpp")
SOURCES.append(ROOT / "tests/runtime/audio_disabled.cpp")
SOURCES.append(ROOT / "tests/runtime/sequence_kernel.cpp")
SOURCES.append(ROOT / "tests/runtime/sequence_service.cpp")
SOURCES.append(ROOT / "tests/runtime/object_model.cpp")
SOURCES.append(ROOT / "tests/runtime/hud_layout.cpp")
SOURCES.append(ROOT / "tests/runtime/world2d.cpp")
SOURCES.append(ROOT / "tests/runtime/actor_blueprint.cpp")
SOURCES.append(ROOT / "tests/runtime/actor_services.cpp")
SOURCES.append(ROOT / "tests/runtime/instrument_bank.cpp")
SOURCES.append(ROOT / "tests/runtime/instrument_allocator.cpp")
SOURCES.append(ROOT / "tests/runtime/instrument_synth.cpp")
SOURCES.append(ROOT / "tests/runtime/instrument_preview.cpp")
SOURCES.append(ROOT / "tests/runtime/instrument_service.cpp")
SOURCES.append(ROOT / "tests/runtime/native_music_service.cpp")
SOURCES.append(ROOT / "tests/runtime/native_music_mixed.cpp")
SOURCES.append(ROOT / "tests/runtime/native_envelope.cpp")
SOURCES.append(ROOT / "tests/runtime/instrument_reverb.cpp")
SOURCES.append(ROOT / "tests/runtime/instrument_source_preview.cpp")
SOURCES.append(ROOT / "tests/runtime/spu_transfer.cpp")
if len(_sys.argv) > 1:
    selected = set(_sys.argv[1:])
    missing = selected - {source.stem for source in SOURCES}
    if missing:
        raise SystemExit(f"Unknown native test names: {sorted(missing)}")
    SOURCES = [source for source in SOURCES if source.stem in selected]
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
    raise SystemExit("No C++17 host compiler found (g++, clang++, or Visual Studio C++ tools required).")
with tempfile.TemporaryDirectory(prefix="epok-spatial-") as temp:
    path = Path(temp)
    (path / "EASTL").mkdir()
    documents.write_text(path / "EASTL/functional.h", "#pragma once\n#include <functional>\nnamespace eastl {using std::function;}\n")
    documents.write_text(path / "display.hh", "#pragma once\nnamespace epok {inline constexpr int display_width=320,display_height=240;}\n")
    # Compile the real streaming backend against a deterministic CD transport.
    backend = path / "streaming-backend"
    (backend / "psyqo").mkdir(parents=True)
    shutil.copyfile(ROOT / "runtime/streaming.hpp", backend / "streaming.hpp")
    shutil.copyfile(ROOT / "tests/runtime/streaming_transport_stub.hpp", backend / "music.hpp")
    documents.write_text(backend / "psyqo/gpu.hh", "#pragma once\n")
    host = path / "streaming-host"
    shutil.copytree(backend, host)
    (host / "common/kernel").mkdir(parents=True)
    shutil.copyfile(ROOT / "tests/runtime/pcdrv_stub.hpp", host / "common/kernel/pcdrv.h")
    documents.write_text(host / "data-config.hh", "#pragma once\n#define EPOK_HOST_DATA 1\n")
    includes = [path, ROOT / "third_party/nugget", ROOT / "runtime", ROOT / "native"]
    audio = path / "audio-runtime"
    (audio / "common/hardware").mkdir(parents=True)
    shutil.copyfile(ROOT / "runtime/audio.hpp", audio / "audio.hpp")
    for name in ("spu_transfer.hpp", "sequence_service.hpp", "native_music_runtime.hpp", "sequence_instrument_service.hpp", "sequence_data.hpp", "native_music_data.hpp", "native_music_service.hpp", "sequence_kernel.hpp", "sequence_lock.hpp", "sequence_tables.hpp", "instrument_bank.hpp", "instrument_allocator.hpp", "instrument_synth.hpp", "instrument_preparation.hpp", "instrument_reverb.hpp"):
        shutil.copyfile(ROOT / "runtime" / name, audio / name)
    shutil.copyfile(ROOT / "tests/runtime/audio_transport_stub.hpp", audio / "epok.hpp")
    for name in ("dma.h", "spu.h", "hwregs.h"):
        documents.write_text(audio / "common/hardware" / name, "#pragma once\n")
    for source in SOURCES:
        target = path / (source.stem + ("-test.exe" if os.name == "nt" else "-test"))
        if Path(compiler).name.lower() == "cl.exe":
            command = [compiler, "/nologo", "/std:c++20", "/EHsc", "/W3", "/D_CRT_SECURE_NO_WARNINGS", str(source), f"/Fe:{target}", f"/Fo:{path / (source.stem + '.obj')}", *[f"/I{p}" for p in includes]]
        else:
            command = [compiler, "-std=c++20", "-Wall", "-Wextra", str(source), "-o", str(target), *[f"-I{p}" for p in includes]]
        if source.stem in ("instrument_preview", "instrument_source_preview"):
            native_source = ROOT / "native" / source.name
            if Path(compiler).name.lower() == "cl.exe":
                native_object = path / f"{source.stem}_native.obj"
                subprocess.run([compiler, "/nologo", "/std:c++20", "/EHsc", "/W4", "/WX", "/c", str(native_source), f"/Fo:{native_object}", *[f"/I{p}" for p in includes]], cwd=temp, env=env, check=True)
                command.append(str(native_object))
            else:
                command.append(str(native_source))
        subprocess.run(command, cwd=temp, env=env, check=True)
        subprocess.run([str(target)], cwd=temp, check=True, timeout=30)
