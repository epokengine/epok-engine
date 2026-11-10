"""Prove an unused query context adds no linked cost to the same Basic starter."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from verify_camera_rigs import ROOT, EDITOR


def verify(out):
    out.mkdir(parents=True, exist_ok=True)
    project = out / "project"
    commands = []
    if not project.exists():
        commands.append(("create", ["--create-project", str(project), "--template", "basic"]))
    commands.append(("build", ["--project", str(project), "--analyze-memory"]))
    for label, args in commands:
        result = subprocess.run([str(EDITOR), *args], cwd=ROOT, capture_output=True, text=True, encoding="utf-8", timeout=900)
        (out / (label + ".log")).write_text(result.stdout + result.stderr, encoding="utf-8")
        assert result.returncode == 0, result.stdout + result.stderr
    build = project / ".epok/build"
    memory = json.loads((build / "memory-report.json").read_text(encoding="utf-8"))
    actual = {"ram_static_bytes": memory["ram"]["used"], "exe_bytes": (build / "epok.ps-exe").stat().st_size}
    reference = "artifacts/gameplay-helpers/p7/shared-storage-footprint/result.json"
    previous = json.loads((ROOT / reference).read_text(encoding="utf-8"))["budget"]
    baseline = {"ram_static_bytes": previous["static_ram_bytes"], "exe_bytes": previous["exe_bytes"], "evidence": reference}
    assert actual["ram_static_bytes"] == baseline["ram_static_bytes"] and actual["exe_bytes"] == baseline["exe_bytes"], (actual, baseline)
    generated = "\n".join(path.read_text(encoding="utf-8") for path in build.glob("*.hh"))
    assert "ObjectPool<epok::SpatialQueryContextComponent" not in generated
    assert "ObjectPool<epok::CharacterMotor3DComponent" not in generated
    report = {"editor_sha256": hashlib.sha256(EDITOR.read_bytes()).hexdigest(), "actual": actual, "baseline": baseline,
              "delta_ram_static_bytes": 0, "delta_exe_bytes": 0, "unused_context_pool_absent": True,
              "unused_motor_pool_absent": True}
    (out / "result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("PASS unused query context: zero linked RAM/EXE delta and no pool", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "artifacts/gameplay-helpers/p8/query-context-footprint")
    verify(parser.parse_args().out)
