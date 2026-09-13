"""Build mixed XA/SFX banks for Serial and CD without opening hardware/emulators."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents
from project_paths import project_manifest

EXE = ROOT / "target/debug" / ("epok-editor.exe" if os.name == "nt" else "epok-editor")
FLAGS = getattr(subprocess, "CREATE_NO_WINDOW", 0)


def main():
    out = ROOT / "artifacts/serial-xa" / str(time.time_ns())
    out.mkdir(parents=True)
    project = out / "project"

    def run(*args, success=True):
        result = subprocess.run([str(EXE), *map(str, args)], capture_output=True,
                                text=True, creationflags=FLAGS, timeout=300)
        text = result.stdout + result.stderr
        with (out / "build.log").open("a", encoding="utf-8") as log:
            log.write(text)
        assert (result.returncode == 0) == success, text
        return text

    def write(path, value):
        documents.write_text(path, json.dumps(value, indent=2), encoding="utf-8")

    run("--create-project", project, "--template", "basic")
    config = documents.loads((ROOT / "Editor.epokconfig").read_text(encoding="utf-8"))
    for key in ("make", "toolchain_bin", "nugget", "emulator", "psxavenc", "mkpsxiso"):
        config[key] = str((ROOT / config[key]).resolve())
    write(project / "Local.epokconfig", config)
    shutil.copyfile(ROOT / "tests/fixtures/stereo-tone.mp3", project / "assets/music.mp3")
    run("--project", project, "--import-audio", "assets/music.mp3", "--audio-usage", "bgm", "--loop")
    run("--project", project, "--import-audio", "assets/music.mp3", "--asset", "assets/sfx.epokasset", "--trim-end", "0.1")
    ids = {a["path"]: a["id"] for a in json.loads(run("--project", project, "--scan-assets"))["assets"]}
    manifest_path = project_manifest(project)
    manifest = documents.loads(manifest_path.read_text(encoding="utf-8"))
    scene_path = project / manifest["startup_scene"]
    scene = documents.loads(scene_path.read_text(encoding="utf-8"))
    scene["entities"][0]["audio"] = dict(clip=ids["assets/music.epokasset"], pitch=1.)
    scene["entities"].append(dict(id=str(uuid.uuid4()), name="SFX", kind="Mesh",
                                  position=[0, 0, 0], rotation=[0, 0, 0], scale=[1, 1, 1],
                                  audio=dict(clip=ids["assets/sfx.epokasset"])))
    write(scene_path, scene)
    build = project / ".epok/build"

    def compile(target, data):
        manifest["play"] = dict(target=target, data=data, content="current_scene")
        write(manifest_path, manifest)
        log = run("--project", project, "--build-psx", "--use-play-profile")
        report = json.loads((build / "memory-report.json").read_text())
        bank = (build / "audio-bank.hh").read_text()
        assert "audio_clip_count = 2" in bank and "audio_data_" in bank
        assert any(n["asset"] == "assets/sfx.epokasset" for n in report["spu"]["root"]["children"])
        if target == "serial":
            assert "Warning: XA music omitted for Serial:" in log, log
            assert "Warning: XA music omitted for Serial:" in (project / ".epok/Build.log").read_text()
            assert any("XA music omitted for Serial:" in w for w in report["warnings"])
            assert ".XA;1" not in bank and "{nullptr,0,0,false}" in bank
            assert not (build / "disc.xml").exists()
            assert all(".XA" not in n["name"] for n in report["files"]["root"]["children"])
            assert json.loads((build / "build-summary.json").read_text())["external_bytes"] == 0
        else:
            assert "XA music omitted" not in log and ".XA;1" in bank
            assert (build / "epok.cue").is_file()
            assert any(".XA" in n["name"] for n in report["files"]["root"]["children"])
        return log

    compile("serial", "executable")
    assert "Reusing verified PSX build" in compile("serial", "executable")
    compile("serial", "host")
    compile("embedded", "disc")
    # Stale XA files from a CD build must not reenter a Serial payload/report.
    compile("serial", "executable")
    # Rules specific to omitted XA playback must not block Serial either.
    scene["entities"][0]["audio"]["pitch"] = 2.
    write(scene_path, scene)
    compile("serial", "host")
    manifest["play"] = dict(target="embedded", data="disc", content="current_scene")
    write(manifest_path, manifest)
    assert "XA music requires pitch 1.0" in run("--project", project, "--build-psx", "--use-play-profile", success=False)
    print(f"Serial XA omission, cached warnings, SFX accounting and CD round trip passed: {out}")


if __name__ == "__main__":
    main()
