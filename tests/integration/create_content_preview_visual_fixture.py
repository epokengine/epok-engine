"""Capture actual native media previews, both with and without imported originals."""
import argparse
import hashlib
import json
import math
import os
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
    parser.add_argument("--output", type=Path, default=REPO / "artifacts/content-previews")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    project = Path(tempfile.mkdtemp(prefix="epok-preview-visual-")) / "Media Showcase"
    env = dict(os.environ, EPOK_EDITOR_HOME=str(REPO), LOCALAPPDATA=str(output / "profile"))

    def run(*flags):
        subprocess.run([str(args.editor.resolve()), *map(str, flags)], cwd=REPO,
                       env=env, check=True, timeout=120)

    run("--create-project", project, "--name", "Media Showcase")
    assets = project / "assets"
    shutil.copyfile(REPO / "resources/branding/epok.png", assets / "Epok.png")
    run("--project", project, "--import-texture", "assets/Epok.png", "--asset", "assets/Epok.epokasset")
    with wave.open(str(assets / "Melody.wav"), "wb") as audio:
        audio.setparams((1, 2, 22050, 0, "NONE", "not compressed"))
        samples = []
        for i in range(22050 * 8):
            t = i / 22050
            envelope = (0.25 + 0.65 * math.sin(t * 2.1) ** 2) * min(1, t * 3, (8 - t) * 3)
            samples.append(int(22000 * envelope * math.sin(t * 440 * math.tau)))
        audio.writeframes(struct.pack("<" + "h" * len(samples), *samples))
    run("--project", project, "--import-audio", "assets/Melody.wav", "--asset", "assets/Melody.epokasset")
    run("--project", project, "--new-blueprint", "BP_Cube")
    next(assets.rglob("BP_Cube.epokbp")).rename(assets / "BP_Cube.epokbp")
    # File is intentionally malformed: its fallback icon must survive decoding failure.
    (assets / "Damaged.png").write_bytes(b"invalid PNG fixture")
    key = "assets/Damaged.png:" + hashlib.sha256(b"invalid PNG fixture").hexdigest() + ":"
    documents.write_text(project / "UserSettings/ImportState.epokprefs", json.dumps({"entries": {key: "Omitted"}}))
    prefs = project / "UserSettings/ContentBrowser.epokprefs"
    for name, settings in [
        ("default", {"tile_size": 180}),
        ("originals", {"tile_size": 180, "hide_unprocessed": False}),
        ("list", {"list": True, "hide_unprocessed": False}),
        ("compact", {"tile_size": 96, "hide_unprocessed": False}),
    ]:
        documents.write_text(prefs, json.dumps(settings))
        run("--project", project, "--screenshot-content-browser", "--window-size", "1440x700",
            "--screenshot", output / (name + ".png"))
    (output / "project.txt").write_text(str(project), encoding="utf-8")
    print(f"Media preview fixture: {project}")


if __name__ == "__main__":
    main()
