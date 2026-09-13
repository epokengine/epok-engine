"""Capture the real compact splash and window restoration on success/failure."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents
from project_paths import project_manifest


def main():
    out = ROOT / "artifacts/splash" / str(time.time_ns())
    out.mkdir(parents=True)
    project = out / "project"
    exe = ROOT / "target/debug" / ("epok-editor.exe" if os.name == "nt" else "epok-editor")

    def run(*args):
        result = subprocess.run([str(exe), *map(str, args)], capture_output=True,
                                text=True, timeout=60,
                                creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        with (out / "verification.log").open("a", encoding="utf-8") as log:
            log.write(result.stdout + result.stderr)
        assert result.returncode == 0, result.stdout + result.stderr

    run("--create-project", project, "--template", "basic", "--name", "Epok Showcase")
    path = project_manifest(project)
    manifest = documents.loads(path.read_text(encoding="utf-8"))
    manifest["auto_build"] = False
    documents.write_text(path, json.dumps(manifest))
    run("--project", project, "--screenshot-loading", "--screenshot", out / "splash.png")
    run("--project", project, "--window-size", "1024x720", "--screenshot", out / "editor.png")
    run("--project", out / "missing-project", "--window-size", "1024x720",
        "--screenshot", out / "failed-open.png")
    splash = Image.open(out / "splash.png").size
    editor = Image.open(out / "editor.png").size
    failure = Image.open(out / "failed-open.png").size
    # Compare logical proportions as captures may use the monitor's DPI scale.
    assert abs(splash[0] / splash[1] - 640 / 400) < 0.01, splash
    assert abs(editor[0] / editor[1] - 1024 / 720) < 0.01, editor
    assert failure == editor, (failure, editor)
    assert abs(splash[0] / editor[0] - 640 / 1024) < 0.02, (splash, editor)
    assert not (project / ".epok/build/epok.ps-exe").exists()
    print(f"Splash and window restoration passed: {out}")


if __name__ == "__main__":
    main()
