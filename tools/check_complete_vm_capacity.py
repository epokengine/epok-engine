"""Early capacity experiment: complete scene with the already-ported Lua policies.

This intentionally hybrid project is not acceptance of a Complete Lua template.
Run --prepare without the console queue; --build requires its exclusive ownership.
Existing acceptance projects and receipts are read-only inputs.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

SOURCE = ROOT / "artifacts/gameplay-helpers/p8/player-vitals"
DEFAULT_OUT = ROOT / "artifacts/gameplay-helpers/p8/execution-reorg/full-scene-vm"
EDITOR = ROOT / "target/debug/epok-editor.exe"
POLICIES = {
    "DungeonRunFlow": "df3f7e3e-b99c-5530-9d86-a44f87e67c5f",
    "DungeonPlayerVitals": "070a2ac6-e00f-5e9c-8bd2-560acb25ad2f",
}
CURRENT_POLICIES = {
    **POLICIES,
    "DungeonPlayerCombat": "e8e7c9ff-ff0e-5c38-9eec-5434e6a45354",
    "DungeonEncounter": "825024e2-d9f8-5578-8198-bc52b5c217f2",
    "DungeonEconomyPolicy": "d3ed89cb-587b-57cb-a136-8172feb4c774",
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def audit(out):
    receipt = json.loads((SOURCE / "validation.json").read_text())
    snapshot = json.loads((SOURCE / "source-snapshot.json").read_text())
    changed = [name for name, expected in snapshot.items()
               if not (ROOT / name).exists() or digest(ROOT / name) != expected]
    report = {
        "recorded_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "receipt": str((SOURCE / "validation.json").relative_to(ROOT)),
        "status": receipt["status"],
        "receipt_sha256": digest(SOURCE / "validation.json"),
        "snapshot_files": len(snapshot), "changed_snapshot_files": changed,
        "receipt_editor_sha256": receipt["editor_sha256"],
        "current_editor_sha256": digest(EDITOR),
        "classification": "current-source at audit" if not changed and digest(EDITOR) == receipt["editor_sha256"]
                          else "historical pinned acceptance; changed dependencies require scoped revalidation",
        "tests_repeated": False,
        "remaining": receipt["remaining"],
    }
    write(out.parent / "state-audit.json", report)
    return report


def prepare(out, reuse_project=None, source_project=None):
    source = source_project.resolve(strict=True) if source_project else SOURCE / "complete/project"
    policies = CURRENT_POLICIES if source_project else POLICIES
    project = reuse_project.resolve() if reuse_project else out / "project"
    if reuse_project:
        assert project.is_relative_to(DEFAULT_OUT.parent.resolve()), "Only reuse mutable execution-reorg fixtures"
    if not project.exists():
        shutil.copytree(source, project,
                        ignore=shutil.ignore_patterns(".epok", "exports", "UserSettings"))
    if source_project:
        assert source != project, "Source fixture must remain a read-only input"
        # Update the retained hybrid fixture without duplicating SDK/build caches.
        # Preserve every changed old script before replacing the private copy.
        before = {}
        for original in (source / "assets/scripts").rglob("*"):
            if not original.is_file():
                continue
            relative = original.relative_to(source)
            target = project / relative
            if target.exists() and digest(target) != digest(original):
                backup = out / "previous-project-sources" / relative
                backup.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(target, backup)
                before[relative.as_posix()] = digest(target)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(original, target)
        manifest = documents.loads(next(source.glob("*.epokproject")).read_text(encoding="utf-8"))
        scene_relative = manifest["startup_scene"]
        shutil.copyfile(source / scene_relative, project / scene_relative)
        write(out / "source-update.json", {"source": str(source), "previous_script_hashes": before,
                                          "build_cache_reused": str(project / ".epok/build")})
    scripts = project / "assets/scripts/DungeonCombat"
    classes = {}
    for name, identity in policies.items():
        authored = scripts / (name + ".hpp")
        if authored.exists():
            authored.unlink()  # Only the new experiment's copied policy.
        target = scripts / (name + ".lua")
        shutil.copyfile(ROOT / "resources/templates/dungeon-complete-flavors/lua" / target.name, target)
        classes[target.relative_to(project).as_posix()] = identity
    write(project / "ProjectSettings/LuaClasses.epoksettings", {"classes": classes})
    manifest_path = next(project.glob("*.epokproject"))
    manifest = documents.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["lua_execution"] = "vm_bytecode"
    write(manifest_path, manifest)
    original_scene = source / manifest["startup_scene"]
    assert (project / manifest["startup_scene"]).read_bytes() == original_scene.read_bytes()
    scene = documents.loads(original_scene.read_text(encoding="utf-8"))
    assets = {}
    for path in (source / "assets").rglob("*"):
        if path.is_file() and "scripts" not in path.relative_to(source / "assets").parts:
            relative = path.relative_to(source)
            assert digest(project / relative) == digest(path), relative
            assets[relative.as_posix()] = digest(path)
    record = {
        "scope": f"Full-scene capacity experiment with {len(policies)} real Lua policies; remaining rules are C++, not provider acceptance",
        "project": str(project.relative_to(ROOT)), "actors": len(scene["actors"]),
        "components": sum(len(actor["components"]) for actor in scene["actors"]),
        "unchanged_assets": assets,
        "ported_rules": list(policies),
        "source_project": str(source),
        "source_policy_hashes": {name: digest(ROOT / "resources/templates/dungeon-complete-flavors/lua" / (name + ".lua")) for name in policies},
        "editor_sha256": digest(EDITOR),
        "source_receipt_sha256": digest(SOURCE / "validation.json"),
        "minimum_runtime_reserve": 65536,
        "hypothesis": "The complete scene plus the real Lua VM exceeds the existing RAM gate before the full rules port",
        "success_criterion": "Both modes link and retain at least 65536 bytes of main RAM after all static reservations; then boot and measure VM failures/peak",
    }
    write(out / "experiment.json", record)
    return project


def build(out, mode):
    experiment = json.loads((out / "experiment.json").read_text())
    project = ROOT / experiment["project"]
    assert project.exists(), "Run --prepare first"
    assert digest(EDITOR) == experiment["editor_sha256"], "Pinned editor changed; prepare a new experiment"
    manifest_path = next(project.glob("*.epokproject"))
    manifest = documents.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["lua_execution"] = mode
    write(manifest_path, manifest)
    started = time.monotonic()
    started_ns = time.time_ns()
    result = subprocess.run([str(EDITOR), "--project", str(project), "--analyze-memory"],
                            cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace")
    (out / (mode + "-build.log")).write_text(result.stdout + result.stderr, encoding="utf-8")
    report = {"mode": mode, "exit_code": result.returncode,
              "seconds": round(time.monotonic() - started, 2),
              "scope": experiment["scope"], "editor_sha256": digest(EDITOR)}
    mips_started = "Compiling C++ / PsyQo for MIPS" in result.stdout + result.stderr
    report["mips_compilation_started"] = mips_started
    report["stale_build_artifacts_ignored"] = {}
    memory_path = project / ".epok/build/memory-report.json"
    if memory_path.exists() and result.returncode == 0:
        memory = json.loads(memory_path.read_text())
        shutil.copyfile(memory_path, out / (mode + "-memory-report.json"))
        report.update(static_ram_bytes=memory["ram"]["used"],
                      unreserved_bytes=memory["ram"]["capacity"] - memory["ram"]["used"],
                      remaining_after_runtime_reserve=memory["ram"]["capacity"] - memory["ram"]["used"] - 65536,
                      executable_sha256=memory.get("executable_hash"))
    elif memory_path.exists():
        report["previous_memory_report_ignored"] = digest(memory_path)
    for name in ("epok.map", "sources.mk", "scripts/generated/lua/lua_value_config.hpp"):
        path = project / ".epok/build" / name
        if path.exists():
            if result.returncode == 0 or (mips_started and path.stat().st_mtime_ns >= started_ns):
                shutil.copyfile(path, out / (mode + "-" + path.name))
            else:
                report["stale_build_artifacts_ignored"][name] = digest(path)
    mapping = project / ".epok/build/epok.map"
    if mapping.exists() and result.returncode and mips_started and mapping.stat().st_mtime_ns >= started_ns:
        match = re.search(r"0x([0-9a-fA-F]+)\s+__heap_base = __bss_end", mapping.read_text())
        if match:
            linked_bytes = int(match[1], 16) - 0x80000000
            report.update(linker_static_extent_bytes=linked_bytes,
                          linker_ram_overflow_bytes=max(0, linked_bytes - 2097152),
                          minimum_reduction_for_64k_reserve_bytes=max(0, linked_bytes + 65536 - 2097152),
                          extent_scope="Failed-link map lower bound, before successful report heap alignment; not a booted image")
    config_path = project / ".epok/build/scripts/generated/lua/lua_value_config.hpp"
    if config_path.exists() and (result.returncode == 0 or (mips_started and config_path.stat().st_mtime_ns >= started_ns)):
        config = config_path.read_text()
        words = int(re.search(r"#define EPOK_LUA_REQUIRED_VALUE_WORDS (\d+)", config)[1])
        arena = int(re.search(r"#define EPOK_LUA_PLANNED_ARENA_BYTES (\d+)", config)[1])
        slots_match = re.search(r"#define EPOK_LUA_PLANNED_VALUE_SLOTS (\d+)", config)
        slots = int(slots_match[1]) if slots_match else 64
        report.update(required_value_words=words, planned_arena_bytes=arena,
                      planned_value_slots=slots,
                      static_wire_value_payload_bytes=16 * slots * (4 + 4 * words),
                      static_wire_value_scratch_bytes=16 * slots * (4 + 4 * words) + (128 if slots_match else 0),
                      full_profile_arena_ceiling_bytes=98304 if mode == "vm_bytecode" else 131072)
    report["decision"] = ("preparation failed before MIPS compilation; no current capacity measurement"
                          if result.returncode and not mips_started else
                          "capacity gate failed; prioritize measured memory costs before full VM game port"
                          if result.returncode else "static analysis completed; boot/runtime peaks still required")
    write(out / (mode + "-result.json"), report)
    print(json.dumps(report, indent=2), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--prepare", action="store_true")
    parser.add_argument("--reuse-project", type=Path, help="Reuse an existing mutable execution-reorg fixture and its build cache")
    parser.add_argument("--source-project", type=Path, help="Read current Complete scene/scripts and use all five migrated Lua policies; no full-provider acceptance claim")
    parser.add_argument("--build", choices=("vm_bytecode", "vm_source"))
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    report = audit(args.out)
    print(json.dumps({key: report[key] for key in ("classification", "snapshot_files", "changed_snapshot_files")}, indent=2), flush=True)
    if args.prepare:
        print("Prepared", prepare(args.out, args.reuse_project, args.source_project), flush=True)
    if args.build:
        build(args.out, args.build)


if __name__ == "__main__":
    main()
