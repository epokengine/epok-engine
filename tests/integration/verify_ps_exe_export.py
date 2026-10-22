"""Real MIPS PS-EXE export: package contents, reproducibility, XA rejection and
a relocated PCDrv run with no disc and no access to the project build folder.

Uses a fresh project and port 8099. Never connects to physical serial hardware.
The relocated run starts PCSX-Redux directly with the packaged executable and
PCDrv rooted at the extracted folder, as a PCDrv-compatible loader would.
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import epok_documents as documents
from project_paths import project_manifest
from verify_streaming import uid
import argparse
import hashlib
import json
import re
import shutil
import socket
import struct
import subprocess
import time
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[2]
EXE = ROOT / "target/debug/epok-editor.exe"
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)
PORT = 8099
DOCUMENTS = {"epok-package.json", "README.txt", "LICENSE", "THIRD_PARTY_NOTICES.md",
             "licenses/nugget-MIT.txt", "licenses/EASTL.txt", "licenses/EASTL-third-party.txt",
             "licenses/EABase.txt", "licenses/psx-font.txt", "licenses/GCC-GPL-3.txt",
             "licenses/GCC-Runtime-Exception-3.1.txt"}


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    parser.add_argument("--no-emulator", action="store_true",
                        help="Check packaging only; skip the relocated PCSX-Redux run")
    args = parser.parse_args()
    out = args.output or ROOT / "artifacts/ps-exe-export" / str(time.time_ns())
    out.mkdir(parents=True, exist_ok=False)
    folder = out / "project"
    with socket.socket() as available:
        available.bind(("127.0.0.1", PORT))

    def save(path, value):
        path.parent.mkdir(parents=True, exist_ok=True)
        documents.write_text(path, json.dumps(value, indent=2), encoding="utf-8")

    def run(*arguments, success=True):
        result = subprocess.run([str(EXE), *map(str, arguments)], capture_output=True, text=True,
                                creationflags=FLAGS, timeout=300)
        text = result.stdout + result.stderr
        with (out / "build.log").open("a", encoding="utf-8") as log:
            log.write(text)
        assert (result.returncode == 0) == success, text
        return text

    # The template's level is editable meshes; streaming moves them to GEOMETRY.BIN.
    # New projects keep scenes outside the EXE, so the startup scene is SCENE00.BIN.
    run("--create-project", folder, "--template", "third-person")
    config = documents.loads((ROOT / "Editor.epokconfig").read_text())
    for key in ("make", "toolchain_bin", "nugget", "emulator", "psxavenc", "mkpsxiso"):
        config[key] = str((ROOT / config[key]).resolve())
    config["web_port"] = PORT
    save(folder / "Local.epokconfig", config)
    manifest_path = project_manifest(folder)
    manifest = documents.loads(manifest_path.read_text())
    manifest["rendering"]["streaming_geometry"] = True
    # A Play profile the export must neither use nor change.
    manifest["play"] = dict(target="window", content="current_scene", data="executable")
    documents.write_text(manifest_path, documents.dumps(manifest))
    scene_path = folder / manifest["startup_scene"]
    scene_text = scene_path.read_text()
    saved_manifest = manifest_path.read_bytes()

    # Leftovers from earlier configurations must never enter the package.
    build = folder / ".epok/build"
    build.mkdir(parents=True, exist_ok=True)
    for stale in ("epok.cue", "epok.bin", "epok.iso", "music/M0000000.XA", "video/V0000000.STR",
                  "stale-notes.txt", "GEOMETRY.OLD"):
        (build / stale).parent.mkdir(parents=True, exist_ok=True)
        (build / stale).write_bytes(b"stale " + stale.encode())

    archive = out / "exports/Relocated-ps-exe.zip"
    log = run("--project", folder, "--export-ps-exe", "--debug-symbols", "--output", archive)
    assert "Exported PS-EXE package:" in log, log
    assert manifest_path.read_bytes() == saved_manifest, "Export changed the saved project settings"
    with zipfile.ZipFile(archive) as package_zip:
        names = set(package_zip.namelist())
        expected = DOCUMENTS | {"epok.ps-exe", "GEOMETRY.BIN", "SCENE00.BIN", "debug/epok.elf",
                                "debug/epok.map"}
        assert names == expected, sorted(names ^ expected)
        description = json.loads(package_zip.read("epok-package.json"))
        geometry_bytes = package_zip.read("GEOMETRY.BIN")
        scene_bytes = package_zip.read("SCENE00.BIN")
        exe_bytes = package_zip.read("epok.ps-exe")
        readme = package_zip.read("README.txt").decode()
    assert exe_bytes.startswith(b"PS-X EXE")
    assert geometry_bytes == (build / "GEOMETRY.BIN").read_bytes()
    assert len(geometry_bytes) > 0 and len(geometry_bytes) % 65536 == 0
    files = {f["path"]: f for f in description["files"]}
    assert files["epok.ps-exe"]["sha256"] == sha256(exe_bytes)
    assert files["GEOMETRY.BIN"]["sha256"] == sha256(geometry_bytes)
    assert files["GEOMETRY.BIN"]["role"] == "pcdrv"
    assert scene_bytes == (build / "SCENE00.BIN").read_bytes() and len(scene_bytes) % 2048 == 0
    assert files["SCENE00.BIN"]["sha256"] == sha256(scene_bytes) and files["SCENE00.BIN"]["role"] == "pcdrv"
    assert description["loader"]["pcdrv_required"]
    assert description["loader"]["pcdrv_files"] == ["GEOMETRY.BIN", "SCENE00.BIN"], description["loader"]
    assert description["pcdrv_root"] == "." and description["content"] == "whole_game"
    rendering = manifest["rendering"]
    assert description["display"] == dict(width=rendering["width"], height=rendering["height"],
                                          interlaced=rendering["height"] == 480, video_mode="ntsc")
    assert "PCDrv server's root folder" in readme and "GEOMETRY.BIN" in readme and "SCENE00.BIN" in readme
    print("PASS package: explicit file list, certified hashes, manifest and README", flush=True)

    first = archive.read_bytes()
    log = run("--project", folder, "--export-ps-exe", "--debug-symbols", "--output", archive)
    assert "Reusing verified PSX build" in log, log
    assert archive.read_bytes() == first, "Unchanged exports must be byte-identical"
    print("PASS repeat export reuses the certified build and reproduces the ZIP", flush=True)

    # XA music is rejected by name, and the previous package stays as it was.
    shutil.copyfile(ROOT / "tests/fixtures/stereo-tone.mp3", folder / "assets/music.mp3")
    run("--project", folder, "--import-audio", "assets/music.mp3", "--audio-usage", "bgm", "--loop")
    ids = {a["path"]: a["id"] for a in json.loads(run("--project", folder, "--scan-assets"))["assets"]}
    scene = documents.loads(scene_text)
    scene["actors"][0]["components"].append({
        "class": dict(class_id="7f0eb028-5301-4ac7-b93b-5665fab12b20", name="epok::AudioComponent"),
        "id": uid(), "name": "Music", "overrides": ["audio"], "root": False,
        "properties": dict(audio=dict(clip=ids["assets/music.epokasset"]))})
    documents.write_text(scene_path, documents.dumps(scene))
    # Serial Play with the export's exact profile omits XA and succeeds. Its
    # receipt must not let the export skip the stricter staging check.
    serial = dict(manifest, play=dict(target="serial", data="host", content="whole_game"))
    documents.write_text(manifest_path, documents.dumps(serial))
    log = run("--project", folder, "--build-psx", "--use-play-profile")
    assert "Warning: XA music omitted for Serial" in log, log
    log = run("--project", folder, "--export-ps-exe", "--debug-symbols", "--output", archive, success=False)
    assert "PS-EXE export cannot include XA music: assets/music.mp3" in log, log
    assert "Reusing verified PSX build" not in log, log
    assert archive.read_bytes() == first, "A failed export replaced the previous package"
    assert not list(archive.parent.glob(".*.partial")), "A failed export left a partial archive"
    manifest_path.write_bytes(saved_manifest)
    documents.write_text(scene_path, scene_text)
    print("PASS XA music is rejected with its asset name, even after Serial Play; the previous ZIP is intact",
          flush=True)

    if args.no_emulator:
        print(f"PS-EXE packaging checks passed: {out}")
        return

    def relocated(name):
        target = out / name
        with zipfile.ZipFile(archive) as package_zip:
            package_zip.extractall(target)
        return target

    good = relocated("relocated")
    corrupt = relocated("relocated-corrupt")
    data = bytearray((corrupt / "GEOMETRY.BIN").read_bytes())
    data[len(data) // 2] ^= 0xff
    (corrupt / "GEOMETRY.BIN").write_bytes(bytes(data))
    missing = relocated("relocated-missing")
    (missing / "GEOMETRY.BIN").unlink()
    # The original build folder must not be reachable during the run.
    build.rename(build.with_name("build.disabled"))

    emulator = Path(config["emulator"])
    if emulator.name == "pcsx-redux.exe" and emulator.with_name("pcsx-redux.main").is_file():
        emulator = emulator.with_name("pcsx-redux.main")
    bios = next((b for b in (emulator.parent / "openbios-fastboot.bin", emulator.parent / "openbios.bin")
                 if b.is_file()), None)
    symbols = (good / "debug/epok.map").read_text()
    match = re.search(r"0x([0-9a-fA-F]+)\s+epok::streaming_stats\b", symbols)
    assert match, "epok::streaming_stats"
    counters = int(match[1], 16) & 0x1fffff

    def request(path):
        with urllib.request.urlopen(f"http://127.0.0.1:{PORT}/api/v1/{path}", timeout=5) as response:
            return response.read()

    def play(folder_name, directory, accept, settle=0):
        portable = out / f"redux-{folder_name}"
        portable.mkdir()
        save(portable / "pcsx.json", {})
        command = [str(emulator), "-portable", str(portable), "-logfile", str(portable / "runtime.log"),
                   "-run", "-stdout", "-fastboot", "-noupdate", "-interpreter", "-softgpu", "-2mb",
                   "-no-gdb", "-webserver", "-webserver-port", str(PORT), "-no-ui",
                   "-loadexe", str(directory / "epok.ps-exe"), "-pcdrv", "-pcdrvbase", str(directory)]
        if bios:
            command += ["-bios", str(bios)]
        with (portable / "console.log").open("w") as console:
            process = subprocess.Popen(command, cwd=portable, stdout=console, stderr=console,
                                       creationflags=FLAGS)
            try:
                deadline = time.monotonic() + 90
                while True:
                    assert process.poll() is None, (portable / "runtime.log").read_text(errors="replace")
                    try:
                        stats = struct.unpack_from("<7I", request("cpu/ram/raw"), counters)
                        if accept(stats):
                            time.sleep(settle)
                            return struct.unpack_from("<7I", request("cpu/ram/raw"), counters)
                    except (OSError, ValueError):
                        pass
                    assert time.monotonic() < deadline, (folder_name, "no expected streaming state")
                    time.sleep(0.2)
            finally:
                if process.poll() is None:
                    subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                                   capture_output=True, creationflags=FLAGS)
                process.wait(timeout=15)

    results = {}
    # Keep running after the first page: no late errors or timeouts either.
    stats = play("relocated", good, lambda s: s[0] > 0 and s[1] > 0, settle=5)
    assert stats[0] > 0 and stats[4:6] == (0, 0), stats
    results["relocated"] = list(stats)
    print(f"PASS relocated package streamed {stats[1]} bytes through PCDrv with no errors", flush=True)
    results["corrupt"] = list(play("corrupt", corrupt, lambda s: s[4] > 0))
    print("PASS a modified GEOMETRY.BIN fails its page integrity check", flush=True)
    results["missing"] = list(play("missing", missing, lambda s: s[4] > 0))
    print("PASS a missing GEOMETRY.BIN is reported as a streaming error", flush=True)
    build.with_name("build.disabled").rename(build)
    save(out / "report.json", dict(archive=str(archive), sha256=sha256(first), streaming=results))
    print(f"PS-EXE export and relocated PCDrv run passed: {out}")


if __name__ == "__main__":
    main()
