"""Check unused-helper cost and flag-only capability retention with real builds."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import uuid
from verify_camera_rigs import ROOT, EDITOR, documents, write

sys.path.insert(0, str(ROOT / "tools"))
from gameplay_helpers_baseline import linked_budget


def main(out):
    out.mkdir(parents=True, exist_ok=True)
    rows = []
    for case in ("unused", "flag-only"):
        project = out / case

        def run(*arguments, label):
            result = subprocess.run([str(EDITOR), *map(str, arguments)], cwd=ROOT,
                                    capture_output=True, text=True, encoding="utf-8", timeout=900)
            (out / f"{case}-{label}.log").write_text(result.stdout + result.stderr, encoding="utf-8")
            assert result.returncode == 0, result.stdout + result.stderr
            return result.stdout

        if not project.exists():
            run("--create-project", project, "--template", "basic", label="create")
        if case == "flag-only":
            registry = json.loads(run("--project", project, "--reflect", label="reflect"))
            classes = {c["cpp_name"]: c for c in registry["classes"]}
            def reference(name):
                return {"name": name, "class_id": classes[name]["id"]}
            manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
            scene_path = project / manifest["startup_scene"]
            scene = documents.loads(scene_path.read_text(encoding="utf-8"))
            scene["actors"] = [scene["actors"][0]]
            scene["actors"].append({
                "id": str(uuid.uuid4()), "name": "Shared Flag", "class": reference("epok::Actor3D"),
                "properties": {}, "overrides": [], "components": [
                    {"id": str(uuid.uuid4()), "name": "Root", "class": reference("epok::SceneComponent3D"), "root": True, "properties": {}, "overrides": []},
                    {"id": str(uuid.uuid4()), "name": "Flag", "class": reference("epok::GameplayFlagComponent"), "properties": {}, "overrides": []},
                ],
            })
            write(scene_path, scene)
        run("--project", project, "--analyze-memory", label="build")
        build = project / ".epok/build"
        flags = (build / "sources.mk").read_text(encoding="utf-8")
        generated = (build / "scene.hh").read_text(encoding="utf-8")
        assert ("-DEPOK_WORLD_GAMEPLAY" in flags) == (case == "flag-only"), flags
        assert "-DEPOK_INTERACTION" not in flags and "-DEPOK_GAMEPLAY_CONTROL" not in flags, flags
        assert ("ObjectPool<epok::GameplayFlagComponent" in generated) == (case == "flag-only")
        for name in ("PickupComponent", "DoorComponent", "SwitchComponent", "InteractableComponent", "InteractorComponent"):
            assert f"ObjectPool<epok::{name}" not in generated, name
        budget = linked_budget(project)
        row = {"case": case, **budget}
        if case == "unused":
            baseline_path = ROOT / "artifacts/gameplay-helpers/p5/third-person-basic-cost.json"
            baseline = json.loads(baseline_path.read_text())
            row["baseline"] = str(baseline_path.relative_to(ROOT))
            row["delta_static_ram_bytes"] = budget["static_ram_bytes"] - baseline["ram_static_bytes"]
            row["delta_exe_bytes"] = budget["exe_bytes"] - baseline["exe_bytes"]
            assert row["delta_static_ram_bytes"] == row["delta_exe_bytes"] == 0, row
        rows.append(row)
        print("PASS", case, "real linked budget and optional pools", flush=True)
    report = {"editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "cases": rows}
    (out / "result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p6/footprint")
    main(parser.parse_args().out)
