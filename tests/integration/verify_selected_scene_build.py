"""Build a disposable project with real MIPS tools; no emulator or PSX needed."""
import copy
from pathlib import Path
import re
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents
from project_paths import project_manifest


def main():
    exe = ROOT / "target/debug/epok-editor.exe"
    folder = Path(tempfile.mkdtemp(prefix="epok-selected-build-")) / "project"
    flags = getattr(subprocess, "CREATE_NO_WINDOW", 0)

    def run(*args, success=True):
        result = subprocess.run([str(exe), *map(str, args)], capture_output=True,
                                text=True, timeout=240, creationflags=flags)
        output = result.stdout + result.stderr
        assert (result.returncode == 0) == success, output
        return output

    run("--create-project", folder, "--template", "basic")
    manifest_path = project_manifest(folder)
    manifest = documents.loads(manifest_path.read_text(encoding="utf-8"))
    scene = documents.loads((folder / manifest["startup_scene"]).read_text(encoding="utf-8"))
    startup_name = scene["name"]
    for name in ["A", "B", "Excluded"]:
        other = copy.deepcopy(scene)
        other["name"] = name
        (folder / f"assets/scenes/{name}.epokmap").write_text(documents.dumps(other), encoding="utf-8")
    manifest["play"] = dict(target="embedded", data="executable", content="selected_scenes",
                            selected_scenes=["assets/scenes/A.epokmap", "assets/scenes/B.epokmap"],
                            initial_scene="assets/scenes/B.epokmap")

    def save():
        manifest_path.write_text(documents.dumps(manifest), encoding="utf-8")

    def build(expected):
        print(f"Building scene banks: {expected}", flush=True)
        save()
        output = run("--project", folder, "--build-psx", "--use-play-profile")
        header = (folder / ".epok/build/scene.hh").read_text()
        names = re.findall(r'\{"([^"\n]+)",load_bank_\d+\}', header)
        assert names == expected, names
        assert (folder / ".epok/build/epok.ps-exe").read_bytes().startswith(b"PS-X EXE")
        return output

    build(["B", "A"])
    assert "Reusing verified PSX build" in build(["B", "A"])
    manifest["play"]["initial_scene"] = "assets/scenes/A.epokmap"
    assert "Reusing verified PSX build" not in build(["A", "B"])
    manifest["play"]["content"] = "whole_game"
    build([startup_name, "A", "B", "Excluded"])
    # A newly saved unregistered scene must invalidate Whole Game's receipt.
    other = copy.deepcopy(scene)
    other["name"] = "New"
    (folder / "assets/scenes/New.epokmap").write_text(documents.dumps(other), encoding="utf-8")
    assert "Reusing verified PSX build" not in build([startup_name, "A", "B", "Excluded", "New"])
    manifest["play"]["content"] = "current_scene"
    build([startup_name])
    manifest["play"].update(content="selected_scenes", selected_scenes=[], initial_scene=None)
    save()
    previous = (folder / ".epok/build/epok.ps-exe").read_bytes()
    for action in ["--build-psx", "--play-psx"]:
        output = run("--project", folder, action, "--use-play-profile", success=False)
        assert "no scenes are selected" in output, output
    assert previous == (folder / ".epok/build/epok.ps-exe").read_bytes()
    print(f"Selected/current/whole builds, startup changes, cache reuse/invalidation and empty warnings passed: {folder}")


if __name__ == "__main__":
    main()
