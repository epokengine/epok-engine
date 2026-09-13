"""Rebuild a copied Timeline/VFX project and isolated export without the old root."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath, PureWindowsPath
import shutil
import subprocess
import tempfile

from verify_blueprints import ROOT, documents, run, write


def authored(root):
    return {path.relative_to(root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in (root / "assets").rglob("*") if path.is_file()}


def portable_manifest(directory):
    manifest = documents.loads((directory / "Scripts.epokmanifest").read_text(encoding="utf-8"))
    assert manifest["version"] == 2 and manifest["dependency_base"] == "project", manifest
    assert manifest["file_base"] == "manifest", manifest
    for key in ("dependencies", "files", "native_sources"):
        for value in manifest[key]:
            path = PurePosixPath(value)
            assert value and not path.is_absolute() and not PureWindowsPath(value).drive, value
            assert ".." not in path.parts and "\\" not in value, value
    for path in manifest["files"]:
        assert (directory / path).is_file(), path
    return manifest


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--emulator", action="store_true", help="Smoke-test Play from the relocated project")
    args = parser.parse_args()
    fixture = Path(tempfile.mkdtemp(prefix="epok-timeline-relocation-")).resolve()
    original = fixture / "Original Game"
    moved = fixture / "Relocated Game"
    archived = fixture / "Original Unavailable"
    standalone = fixture / "Independent Export"
    print("Retained relocation fixture:", fixture, flush=True)
    shutil.copytree(ROOT / "examples/timeline-spell", original,
                    ignore=shutil.ignore_patterns(".epok", "exports", "Local.epokconfig"))
    run("--project", original, "--build-psx")
    expected = (original / ".epok/build/epok.ps-exe").read_bytes()
    sources = authored(original)
    original_manifest = portable_manifest(original / ".epok/build")
    export = Path(run("--project", original, "--export-psx").strip())
    assert portable_manifest(export) == original_manifest
    assert (export / "docs/timelines.md").is_file()
    assert (export / "docs/blueprints.md").is_file()
    # Copy caches too: stale absolute reflection/debug/make paths must not silently
    # keep the old project alive. The standalone copy has no compiled intermediates.
    shutil.copytree(original, moved)
    shutil.copytree(export, standalone,
                    ignore=shutil.ignore_patterns("*.o", "*.d", "*.elf", "*.ps-exe"))
    # Only rename this verifier's newly created fixture. Check both resolved
    # targets before making the original path unavailable; retain every file.
    assert original.resolve().parent == fixture and archived.resolve().parent == fixture
    assert not archived.exists() and original.is_dir()
    original.rename(archived)
    assert not original.exists()
    run("--project", moved, "--build-psx")
    assert (moved / ".epok/build/epok.ps-exe").read_bytes() == expected
    assert authored(moved) == sources, "Relocation must preserve authored bytes and identities"
    assert portable_manifest(moved / ".epok/build") == original_manifest
    graph = json.loads((moved / ".epok/ArtifactDependencies.json").read_text())["nodes"]
    for key in ("stage:.epok/build", "native-sdk:.epok/build", "executable:.epok/build/epok.ps-exe"):
        assert not graph[key]["stale"], (key, graph[key])
    assert graph["executable:.epok/build/epok.ps-exe"]["signature"] == hashlib.sha256(expected).hexdigest()
    forbidden = (str(original).replace("\\", "/"), str(original), str(original).replace("\\", "\\\\"))
    for directory in (standalone, moved / ".epok/build"):
        for path in directory.rglob("*"):
            if path.is_file() and path.suffix in (".cpp", ".hpp", ".hh", ".mk", ".epokmanifest", ".epokdebug"):
                text = path.read_text(encoding="utf-8")
                assert not any(value in text for value in forbidden), path
    environment = os.environ.copy()
    environment["PATH"] = str(ROOT / ".tools/mips/bin") + os.pathsep + environment["PATH"]
    # Exercise the default Make -> platform launcher route, including its private
    # SDK build. The combat check also invokes the Windows launcher directly.
    result = subprocess.run([str(ROOT / ".tools/mips/bin/make.exe"), "BUILD=Release",
                             f"NUGGET_DIR={(ROOT / 'third_party/nugget').as_posix()}"],
                            cwd=standalone, env=environment, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    assert (standalone / "epok.ps-exe").read_bytes() == expected
    if args.emulator:
        output = run("--project", moved, "--play-psx", "--stop-after", "4")
        write(fixture / "relocated-play.log", output)
        assert "Emulator running" in output, output
    report = {"passed": True, "fixture": str(fixture), "original_path_unavailable": True,
              "authored_bytes_unchanged": True, "project_rebuild_identical": True,
              "isolated_export_rebuild_identical": True, "script_manifest_version": 2,
              "ps_exe_bytes": len(expected), "emulator_smoke": args.emulator}
    write(ROOT / "artifacts/timelines/phase5-relocation.json", report)
    print("PASS: relocated Timeline/VFX project and independent export; identical MIPS output, portable manifests, preserved authoring IDs.", flush=True)


if __name__ == "__main__":
    main()
