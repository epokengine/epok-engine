"""Capture current template composition and linked PSX budgets for helpers P0.

The report distinguishes unavailable flavors from failed checks and leaves
runtime stack/heap peaks unknown unless an external capture supplies them.
Existing generated projects are reused only with their matching provenance.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time

import epok_documents as documents

ROOT = Path(__file__).resolve().parents[1]
TEMPLATES = ("basic", "sample", "third-person", "platformer-2d", "dungeon", "dungeon-complete")
FLAVORS = ("cpp", "blueprint", "lua")
MODES = ("native_cpp", "vm_bytecode", "vm_source")
SCENARIOS = {
    "basic": ["empty-scene", "helper-installation"],
    "sample": ["spinner-default", "spinner-override"],
    "third-person": ["move-jump-step", "orbit-obstruction", "input-capture"],
    "platformer-2d": ["run-jump-platform", "coin", "input-capture"],
    "dungeon": ["generation", "key-door", "regeneration"],
    "dungeon-complete": ["intro", "guards", "boss", "terminals", "economy", "restart", "receipts"],
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_report(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(".tmp")
    temporary.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    temporary.replace(path)


def run(command: list[str], log: Path) -> dict:
    started = time.monotonic()
    with log.open("w", encoding="utf-8") as stream:
        result = subprocess.run(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, timeout=1200)
    return {"command": command, "exit_code": result.returncode,
            "seconds": round(time.monotonic() - started, 3), "log": str(log.relative_to(ROOT))}


def composition(project: Path) -> dict:
    manifest = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8"))
    scene = documents.loads((project / manifest["startup_scene"]).read_text(encoding="utf-8"))
    actors = scene["actors"]
    return {
        "map_version": scene["version"], "actors": len(actors),
        "components": sum(len(actor.get("components", [])) for actor in actors),
        "max_components": max((len(a.get("components", [])) for a in actors), default=0),
        "roles": [{"name": a["name"], "class": a.get("class"),
                   "components": [{"name": c["name"], "class": c.get("class"),
                                   "properties": c.get("properties", {})} for c in a.get("components", [])]}
                  for a in actors if a["name"] in ("Player", "Visual", "Camera", "Player Aqua", "Game")],
        "sources": {str(p.relative_to(project)): digest(p)
                    for folder in ("assets/scripts", "assets/Blueprints", "assets/blueprints")
                    for p in sorted((project / folder).rglob("*"))
                    if p.is_file() and p.suffix.lower() in (".cpp", ".hpp", ".lua", ".epokbp")},
        "manifest_sha256": digest(next(project.glob("*.epokproject"))),
        "scene_sha256": digest(project / manifest["startup_scene"]),
    }


def linked_budget(project: Path, mode: str | None = None) -> dict:
    build = project / ".epok/build"
    memory = json.loads((build / "memory-report.json").read_text(encoding="utf-8"))
    used = memory["ram"]["used"]
    available = 2 * 1024 * 1024 - used
    if mode is None:
        mode = documents.loads(next(project.glob("*.epokproject")).read_text(encoding="utf-8")).get("lua_execution", "native_cpp")
    arena = {"vm_bytecode": 98304, "vm_source": 131072}.get(mode, 0)
    if arena:
        import re
        config = build / "scripts/generated/lua/lua_value_config.hpp"
        if config.is_file():
            planned = re.search(r"^#define EPOK_LUA_PLANNED_ARENA_BYTES (\d+)$", config.read_text(), re.M)
            if planned:
                arena = int(planned[1])
        flags = (build / "sources.mk").read_text()
        explicit = re.findall(r"-DEPOK_LUA_ARENA_BYTES=(\d+)\b", flags)
        if explicit:
            assert len(set(explicit)) == 1, "Conflicting Lua arena overrides"
            arena = int(explicit[0])
    return {
        "static_ram_bytes": used, "unassigned_ram_bytes": available,
        "minimum_runtime_headroom_bytes": 65536,
        "linked_lua_vm_arena_bytes": arena,
        "remaining_after_runtime_headroom_bytes": available - 65536,
        # VM rows already include their arena in the ELF; do not subtract it twice.
        # Native projections are arena-only estimates, excluding interpreter/code cost.
        "remaining_after_proposed_bytecode_arena_and_headroom": None if arena else available - 98304 - 65536,
        "remaining_after_proposed_source_arena_and_headroom": None if arena else available - 131072 - 65536,
        "runtime_heap_peak": None, "runtime_stack_peak": None,
        "exe_bytes": (build / "epok.ps-exe").stat().st_size,
        "executable_sha256": digest(build / "epok.ps-exe"),
        "elf_sha256": digest(build / "epok.elf"),
        "memory_report": str((build / "memory-report.json").relative_to(ROOT)),
        "warnings": memory.get("warnings", []),
    }


def revalidate(editor: Path, output: Path) -> None:
    """Refresh runtime builds without replacing the original generation evidence."""
    original = json.loads((output / "baseline.json").read_text(encoding="utf-8"))
    for case in original["cases"]:
        if "composition" not in case:
            continue
        project = output / f"{case['template']}-{case['flavor']}"
        current = composition(project)
        for key in ("actors", "components", "max_components", "sources"):
            if current[key] != case["composition"][key]:
                raise SystemExit(f"Authored {key} changed in {project}; capture a new baseline instead.")
    rows = original["measurements"]
    additional = output / "adapter-builds.json"
    if additional.is_file():
        rows += json.loads(additional.read_text(encoding="utf-8"))
    report = {
        "version": 1, "generation_evidence": "baseline.json", "editor_sha256": digest(editor),
        "budget_note": "VM arenas are included in linked static allocations; native arena projections exclude interpreter/code costs.",
        "runtime_sources": {str(p.relative_to(ROOT)): digest(p) for p in sorted((ROOT / "runtime").iterdir())
                            if p.is_file() and p.suffix in (".hpp", ".cpp", ".mk")},
        "measurements": [], "failures": [],
    }
    destination = output / "final-validation.json"
    for previous in rows:
        row = {key: previous[key] for key in ("template", "flavor", "mode")}
        project = output / f"{row['template']}-{row['flavor']}"
        descriptor = next(project.glob("*.epokproject"))
        manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
        manifest["lua_execution"] = row["mode"]
        documents.write_text(descriptor, documents.dumps(manifest))
        label = f"final-{row['template']}-{row['flavor']}-{row['mode']}"
        row["build"] = run([str(editor), "--project", str(project), "--analyze-memory"], output / f"{label}.log")
        if row["build"]["exit_code"] == 0:
            row["budget"] = linked_budget(project, row["mode"])
        else:
            report["failures"].append(label)
        report["measurements"].append(row)
        write_report(destination, report)
        print(label, "PASS" if "budget" in row else "FAILED", flush=True)
    if report["failures"]:
        raise SystemExit(1)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--editor", type=Path, default=ROOT / "target/debug/epok-editor.exe")
    parser.add_argument("--output", type=Path, default=ROOT / "artifacts/gameplay-helpers/p0")
    parser.add_argument("--build", action="store_true", help="Link all C++ starters and Third Person in each authoring/execution mode")
    parser.add_argument("--revalidate", action="store_true",
                        help="Rebuild an existing baseline with current runtime sources, retaining its original evidence")
    args = parser.parse_args()
    editor = args.editor.resolve(strict=True)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    if args.revalidate:
        revalidate(editor, output)
        return
    report_path = output / "baseline.json"
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    report = {"version": 1, "commit": commit, "editor_sha256": digest(editor),
              "runtime_sources": {str(p.relative_to(ROOT)): digest(p) for p in sorted((ROOT / "runtime").glob("*.hpp"))},
              "cases": [], "measurements": []}
    provenance = output / "provenance.json"
    identity = {"commit": commit, "editor_sha256": report["editor_sha256"]}
    if provenance.exists() and json.loads(provenance.read_text()) != identity:
        raise SystemExit("Baseline executable/source changed; use a different output directory.")
    write_report(provenance, identity)
    failures = []
    for template in TEMPLATES:
        for flavor in FLAVORS:
            case_name = f"{template}-{flavor}"
            project = output / case_name
            supported = template != "dungeon-complete" or flavor == "cpp"
            result = {"template": template, "flavor": flavor, "baseline_supported": supported,
                      "scenarios": SCENARIOS[template], "execution_modes": list(MODES) if flavor == "lua" else ["native_cpp"],
                      "final_acceptance": "pending"}
            if not project.exists():
                result["generation"] = run([str(editor), "--create-project", str(project),
                                             "--name", "Helpers Baseline", "--template", template,
                                             "--gameplay", flavor], output / f"{case_name}-create.log")
            else:
                result["generation"] = {"exit_code": 0, "reused": True}
            actual_supported = result["generation"]["exit_code"] == 0
            if actual_supported != supported:
                failures.append(case_name)
            if supported and actual_supported:
                result["composition"] = composition(project)
                if result["composition"]["actors"] > 64 or result["composition"]["max_components"] > 8:
                    failures.append(case_name + ": composition exceeds bounds")
            report["cases"].append(result)
            write_report(report_path, report)
            print(case_name, "generated" if actual_supported else "unavailable", flush=True)
            if not args.build or not supported or not actual_supported:
                continue
            if flavor != "cpp" and template != "third-person":
                continue
            for mode in MODES if flavor == "lua" else ("native_cpp",):
                descriptor = next(project.glob("*.epokproject"))
                manifest = documents.loads(descriptor.read_text(encoding="utf-8"))
                manifest["lua_execution"] = mode
                documents.write_text(descriptor, documents.dumps(manifest))
                row = {"template": template, "flavor": flavor, "mode": mode}
                row["build"] = run([str(editor), "--project", str(project), "--analyze-memory"],
                                   output / f"{case_name}-{mode}-build.log")
                if row["build"]["exit_code"] == 0:
                    row["budget"] = linked_budget(project, mode)
                    # Preserve each mode's measured report before the shared staging directory is reused.
                    write_report(output / f"{case_name}-{mode}-memory.json", row["budget"])
                else:
                    failures.append(f"{case_name}/{mode}: build")
                report["measurements"].append(row)
                write_report(report_path, report)
                print(case_name, mode, "linked" if row["build"]["exit_code"] == 0 else "FAILED", flush=True)
    report["generation_cases"] = len(report["cases"])
    report["execution_cases"] = sum(len(row["execution_modes"]) for row in report["cases"])
    report["failures"] = failures
    write_report(report_path, report)
    print(json.dumps({"report": str(report_path), "failures": failures}))
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
