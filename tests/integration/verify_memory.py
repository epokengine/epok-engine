"""Real MIPS memory accounting for Play profiles; never opens serial/emulator.

Keeps an isolated fixture and reports under artifacts/memory/<timestamp>.
"""
import copy
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import time
import wave

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents
from project_paths import project_manifest
from PIL import Image
from verify_streaming import package, uid

EXE = ROOT / "target/debug" / ("epok-editor.exe" if os.name == "nt" else "epok-editor")
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)


def main():
    out = ROOT / "artifacts/memory" / str(time.time_ns())
    out.mkdir(parents=True)
    project = out / "project"

    def run(*args, success=True):
        result = subprocess.run([str(EXE), *map(str, args)], capture_output=True,
                                text=True, creationflags=FLAGS, timeout=300)
        with (out / "build.log").open("a", encoding="utf-8") as log:
            log.write(result.stdout + result.stderr)
        assert (result.returncode == 0) == success, result.stdout + result.stderr
        return result.stdout

    def write(path, value):
        documents.write_text(path, json.dumps(value, indent=2), encoding="utf-8")

    def children(node):
        for child in node["children"]:
            yield child
            yield from children(child)

    def accounting(space):
        def check(node):
            if node["children"]:
                assert sum(n["bytes"] for n in node["children"]) == node["bytes"], node["name"]
                for n in node["children"]:
                    check(n)
        check(space["root"])
        assert space["root"]["bytes"] == max(space["used"], space["capacity"] or 0)

    run("--create-project", project, "--template", "basic")
    config = documents.loads((ROOT / "Editor.epokconfig").read_text(encoding="utf-8"))
    for key in ("make", "toolchain_bin", "nugget", "emulator", "psxavenc", "mkpsxiso"):
        config[key] = str((ROOT / config[key]).resolve())
    write(project / "Local.epokconfig", config)
    Image.new("RGBA", (32, 16), (48, 128, 208, 255)).save(project / "assets/shared.png")
    run("--project", project, "--import-texture", "assets/shared.png")
    with wave.open(str(project / "assets/tone.wav"), "wb") as wav:
        wav.setparams((1, 2, 22050, 0, "NONE", "not compressed"))
        wav.writeframes(b"".join(struct.pack("<h", (i % 40 - 20) * 400) for i in range(2205)))
    run("--project", project, "--import-audio", "assets/tone.wav")
    assets = json.loads(run("--project", project, "--scan-assets"))["assets"]
    ids = {a["path"]: a["id"] for a in assets}
    manifest_path = project_manifest(project)
    manifest = documents.loads(manifest_path.read_text(encoding="utf-8"))
    assert manifest["build"]["generate_asset_report"] is True
    manifest["auto_build"] = False
    manifest["play"] = dict(target="serial", content="current_scene", data="executable")
    write(manifest_path, manifest)
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    geometry, group, material = uid(), uid(), uid()
    package(project / "assets/plane.epokasset", geometry, dict(
        version=1, vertices=[[-1,-1,6],[-1,1,6],[1,1,6],[1,-1,6]],
        faces=[dict(id=uid(), vertices=[0,1,2,3], group=group, material=material)],
        groups=[dict(id=group, name="Plane", parent=None)],
        materials=[dict(id=material, name="Shared", material=dict(
            texture=ids["assets/shared.epokasset"], color=[1.,1.,1.], unlit=True))]))
    mesh = dict(id=uid(), name="Plane", kind="Mesh", position=[0,0,0], rotation=[0,0,0],
                scale=[1,1,1], editable_mesh=dict(asset=geometry))
    scene["entities"].append(mesh)
    mesh["audio"] = dict(clip=ids["assets/tone.epokasset"], play_on_start=True, volume=1., pitch=1.)
    write(scene_path, scene)
    other = copy.deepcopy(scene)
    other["name"] = "Second"
    write(project / "assets/scenes/Second.epokmap", other)
    write(project / "ProjectSettings/Maps.epoksettings", dict(scenes=["assets/scenes/Second.epokmap"]))

    def analyze(label):
        run("--project", project, "--build-psx", "--use-play-profile")
        path = project / ".epok/build/memory-report.json"
        report = json.loads(path.read_text(encoding="utf-8"))
        summary = json.loads((project / ".epok/build/build-summary.json").read_text())
        assert summary["report_hash"] and summary["automatic_report"]
        assert summary["exe_bytes"] == (project / ".epok/build/epok.ps-exe").stat().st_size
        assert summary["external_bytes"] == sum(n["bytes"] for n in report["files"]["root"]["children"] if n["name"] != "epok.ps-exe")
        for key in ("ram", "spu", "files", "scratchpad"):
            accounting(report[key])
        for bank in report["scenes"]:
            accounting(bank["vram"])
        (out / f"{label}.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        assert report["profile"]["target"] == "serial"
        assert report["ram"]["capacity"] == 2097152
        assert not (project / ".epok/build/epok.bin").exists()
        return report

    current = analyze("current")
    assert len(current["scenes"]) == 1
    manifest["play"]["content"] = "whole_game"
    write(manifest_path, manifest)
    whole = analyze("whole")
    assert len(whole["scenes"]) == 2
    assert whole["ram"]["used"] > current["ram"]["used"]
    textures = next(n for n in whole["ram"]["root"]["children"] if n["name"] == "Textures")
    shared = [n for n in textures["children"] if n["asset"] == "assets/shared.epokasset"]
    assert len(shared) == 1 and len(shared[0]["scenes"]) == 2, shared
    assert shared[0]["bytes"] == 1024, shared  # 32x16 8-bit pixels + 256x16-bit palette
    assert any(".sbss" in n["detail"] for n in children(whole["ram"]["root"]))
    assert whole["ram"]["used"] > whole["files"]["used"]
    assert whole["spu"]["used"] == current["spu"]["used"]
    assert whole["scenes"][0]["vram"]["used"] == whole["scenes"][1]["vram"]["used"]
    # Stale staging files cannot change a subsequent report.
    (project / ".epok/build/audio/unselected.adpcm").write_bytes(b"x" * 100000)
    again = analyze("repeat")
    assert again["spu"]["used"] == whole["spu"]["used"]
    assert again["files"]["used"] == whole["files"]["used"]
    assert again["executable_hash"] == whole["executable_hash"]
    manifest["play"]["data"] = "host"
    manifest["rendering"].update(streaming_geometry=True, streaming_pool_pages=2)
    write(manifest_path, manifest)
    host = analyze("host")
    assert any(n["name"] == "GEOMETRY.BIN" for n in host["files"]["root"]["children"])
    assert any("Unirom/PCDrv" in warning for warning in host["warnings"])
    manifest["build"]["generate_asset_report"] = False
    write(manifest_path, manifest)
    run("--project", project, "--build-psx", "--use-play-profile")
    summary_path = project / ".epok/build/build-summary.json"
    summary = json.loads(summary_path.read_text())
    assert summary["report_hash"] is None and not summary["automatic_report"]
    binary = (project / ".epok/build/epok.ps-exe").read_bytes()
    # Generation alone must preserve compiled files, even when source edits are pending.
    scene["entities"][0]["name"] = "Pending source edit"
    write(scene_path, scene)
    output = run("--project", project, "--generate-asset-report")
    assert "Compiling C++" not in output
    assert (project / ".epok/build/epok.ps-exe").read_bytes() == binary
    assert json.loads(summary_path.read_text())["report_hash"]
    generated = json.loads((project / ".epok/build/memory-report.json").read_text())
    assert generated["executable_hash"] == host["executable_hash"]
    geometry_path = project / ".epok/build/GEOMETRY.BIN"
    geometry_bytes = geometry_path.read_bytes()
    try:
        geometry_path.write_bytes(geometry_bytes + b"changed")
        run("--project", project, "--generate-asset-report", success=False)
    finally:
        geometry_path.write_bytes(geometry_bytes)
    # UI uses the same fresh report and the saved profile.
    run("--project", project, "--screenshot-memory", "--screenshot", out / "memory.png")
    run("--project", project, "--screenshot-play-menu", "--screenshot", out / "play-menu.png")
    (project / ".epok/build/main.o").unlink()
    run("--project", project, "--screenshot-build-progress", "--screenshot", out / "progress.png")
    print(f"Memory acceptance passed: {out}")


if __name__ == "__main__":
    main()
