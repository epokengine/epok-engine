"""Capture startup and native asset icons using real project creation/import paths."""
import argparse
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import wave

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "tools"))
import epok_documents as documents


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--editor", type=Path, default=REPO / "target/debug/epok-editor.exe")
    parser.add_argument("--output", type=Path, default=REPO / "artifacts/project-startup")
    args = parser.parse_args()
    editor, output = args.editor.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    project = Path(tempfile.mkdtemp(prefix="epok-startup-visual-")) / "Epok Showcase"

    def run(*flags):
        subprocess.run([str(editor), *map(str, flags)], cwd=REPO, check=True, timeout=120)

    run("--create-project", project, "--name", "Epok Showcase")
    assets = project / "assets"
    for flag, name, suffix in [
        ("--new-blueprint", "Hero", ".epokbp"),
        ("--new-timeline", "Intro", ".timeline.json"),
        ("--new-particle-effect", "Fire", ".particle-effect.json"),
    ]:
        run("--project", project, flag, name)
        next(assets.rglob(name + suffix)).rename(assets / (name + suffix))
    shutil.copyfile(next(assets.rglob("*.epokmap")), assets / "Level.epokmap")
    sources = assets / "Sources"
    sources.mkdir()
    shutil.copyfile(REPO / "resources/branding/epok.png", sources / "Logo.png")
    run("--project", project, "--import-texture", "assets/Sources/Logo.png", "--asset", "assets/Logo.epokasset")
    with wave.open(str(sources / "Tone.wav"), "wb") as audio:
        audio.setparams((1, 2, 22050, 0, "NONE", "not compressed"))
        audio.writeframes(b"\0\0" * 2205)
    run("--project", project, "--import-audio", "assets/Sources/Tone.wav", "--asset", "assets/Tone.epokasset")
    shutil.copyfile(REPO / "resources/models/EpokMannequin.fbx", sources / "Mannequin.fbx")
    run("--project", project, "--import-fbx", "assets/Sources/Mannequin.fbx")
    wanted = {"Skeleton": "Rig", "SkeletalMesh": "Mannequin", "AnimationClip": "Walk", "Material": "Surface"}
    for path in list(assets.rglob("*.epokasset")):
        raw = path.read_bytes()
        meta_size = struct.unpack_from("<I", raw, 8)[0]
        meta = json.loads(raw[16:16 + meta_size])
        if meta["kind"] in wanted:
            path.rename(assets / (wanted.pop(meta["kind"]) + ".epokasset"))
    assert not wanted, wanted
    for path in sorted(assets.rglob("*"), key=lambda p: len(p.parts), reverse=True):
        if path.is_dir() and not any(path.iterdir()):
            path.rmdir()
    prefs = project / "UserSettings/ContentBrowser.epokprefs"
    documents.write_text(prefs, json.dumps({"hide_unprocessed": True}))
    run("--project", project, "--screenshot-loading", "--screenshot", output / "splash.png")
    run("--project", project, "--screenshot-content-browser", "--window-size", "1440x700", "--screenshot", output / "icons.png")
    documents.write_text(prefs, json.dumps({"hide_unprocessed": True, "list": True}))
    run("--project", project, "--screenshot-content-browser", "--window-size", "1440x700", "--screenshot", output / "list.png")
    documents.write_text(prefs, json.dumps({"hide_unprocessed": True}))
    run("--project", project, "--screenshot", output / "editor.png")
    (output / "project.txt").write_text(str(project), encoding="utf-8")
    print(f"Visual fixture: {project}")


if __name__ == "__main__":
    main()
