"""Integration checks using the real compiler and emulator, no Python packages needed."""
import sys as _sys
from pathlib import Path as _Path
_sys.path.insert(0, str(_Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
import json
import pathlib
import socket
import struct
import subprocess
import time
import urllib.request
import zlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
SAMPLE = ROOT / "examples/sample-game"
EXE = ROOT / "target/debug/epok-editor.exe"
ARTIFACTS = ROOT / "artifacts"
ARTIFACTS.mkdir(exist_ok=True)
FLAGS = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0


def build(root=SAMPLE, success=True):
    result = subprocess.run([str(EXE), "--project", str(root), "--build-psx"],
                            capture_output=True, text=True, timeout=120, creationflags=FLAGS)
    documents.write_text(ARTIFACTS / ("build-ok.log" if success else "build-error.log"),
        result.stdout + result.stderr, encoding="utf-8")
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    if success:
        assert (root / ".epok/build/epok.ps-exe").read_bytes().startswith(b"PS-X EXE")
    return result


def request(path, post=False):
    req = urllib.request.Request("http://127.0.0.1:8077/api/v1/" + path,
                                 data=b"" if post else None)
    with urllib.request.urlopen(req, timeout=2) as response:
        return response.read()


def png(data, destination, project):
    # First framebuffer at VRAM (0,0), using the project's native output size.
    settings = documents.loads((project_manifest(project)).read_text())
    display = settings.get('rendering', dict(width=640, height=480))
    width, height = display['width'], display['height']
    rows = []
    for y in range(height):
        row = bytearray(b"\0")
        for x in range(width):
            pixel = struct.unpack_from("<H", data, (y * 1024 + x) * 2)[0]
            row.extend(((pixel & 31) * 255 // 31, ((pixel >> 5) & 31) * 255 // 31,
                        ((pixel >> 10) & 31) * 255 // 31))
        rows.append(row)

    def chunk(kind, contents):
        return struct.pack(">I", len(contents)) + kind + contents + struct.pack(">I", zlib.crc32(kind + contents))
    destination.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
                            + chunk(b"IDAT", zlib.compress(b"".join(rows))) + chunk(b"IEND", b""))


def main():
    build()
    obj = SAMPLE / ".epok/build/scripts/Spinner.o"
    before = obj.stat().st_mtime_ns
    build()
    assert obj.stat().st_mtime_ns == before, "Unchanged script was recompiled"
    print("PASS native PS-X EXE and incremental script compilation", flush=True)

    # Deliberately broken C++ lives in an isolated project, never in user assets.
    project = ROOT / ".epok" / ("verify-" + str(time.time_ns()))
    subprocess.run([str(EXE), "--create-project", str(project), "--template", "sample"],
                   check=True, creationflags=FLAGS)
    script = project / "assets/scripts/Spinner.cpp"
    original = script.read_text()
    documents.write_text(script, original + "\nthis is not valid C++;\n")
    result = build(project, success=False)
    assert "no executable launched" in result.stderr
    documents.write_text(script, original)
    build(project)
    print("PASS C++ error reporting and recovery", flush=True)

    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 8077))  # Do not touch another running emulator.
    with (ARTIFACTS / "emulator.log").open("w", encoding="utf-8") as log:
        process = subprocess.Popen([str(EXE), "--project", str(SAMPLE), "--play-psx", "--stop-after", "15"],
                                   stdout=log, stderr=log, creationflags=FLAGS)
        try:
            deadline = time.monotonic() + 12
            while True:
                try:
                    state = json.loads(request("execution-flow"))
                    assert state["running"] and not state["8mb"]
                    break
                except (OSError, ValueError):
                    if time.monotonic() >= deadline:
                        raise RuntimeError("Emulator did not start; see artifacts/emulator.log")
                    time.sleep(0.2)
            time.sleep(2)
            request("execution-flow?function=pause", True)
            assert not json.loads(request("execution-flow"))["running"]
            first = request("gpu/vram/raw")
            assert len(first) == 1024 * 512 * 2
            time.sleep(0.2)
            assert request("gpu/vram/raw") == first, "VRAM changed while paused"
            request("execution-flow?function=resume", True)
            assert json.loads(request("execution-flow"))["running"]
            time.sleep(0.7)
            request("execution-flow?function=pause", True)
            second = request("gpu/vram/raw")
            assert first != second, "C++ Spinner did not change the emulated frame"
            png(second, ARTIFACTS / "psx-output.png", project)
            assert process.wait(timeout=20) == 0
            print("PASS emulator boot, pause/resume, C++ animation, Stop and framebuffer capture", flush=True)
        finally:
            if process.poll() is None:
                # Let the editor's bounded Stop complete and reap its owned emulator.
                process.wait(timeout=25)
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 8077))
    print("PASS emulator process cleanup", flush=True)


if __name__ == "__main__":
    main()
