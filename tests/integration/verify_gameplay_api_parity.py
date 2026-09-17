#!/usr/bin/env python3
"""Focused host acceptance for the shared gameplay operation catalog.

This runner intentionally stays GPU/emulator independent. Target correctness and
performance commands are recorded separately in the initiative validation report.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[2]
PYTHON = sys.executable
LUA_MODES = ("native_cpp", "vm_bytecode", "vm_source")


def run(*command: str) -> None:
    print("+", " ".join(command), flush=True)
    subprocess.run(command, cwd=ROOT, check=True)


def editor_path(explicit: str | None) -> Path:
    if explicit:
        path = Path(explicit).resolve()
        if path.is_file():
            return path
        raise AssertionError(f"Editor executable does not exist: {path}")
    for name in ("epok-editor", "epok-editor.exe"):
        path = ROOT / "target/debug" / name
        if path.is_file():
            return path
    raise AssertionError("Build the debug editor first (cargo build --locked --bins).")


def run_editor(editor: Path, project: Path, *arguments: str) -> None:
    run(str(editor), "--project", str(project), *arguments)


def replace_lua_mode(manifest: Path, mode: str) -> None:
    text = manifest.read_text(encoding="utf-8")
    lines = text.splitlines()
    replacement = f"lua_execution: {mode}"
    for index, line in enumerate(lines):
        if line.startswith("lua_execution:"):
            lines[index] = replacement
            break
    else:
        lines.append(replacement)
    manifest.write_text("\n".join(lines) + "\n", encoding="utf-8")


def build_examples(editor: Path) -> None:
    source = ROOT / "examples/gameplay-api-parity"
    with tempfile.TemporaryDirectory(prefix="epok-gameplay-parity-") as directory:
        examples = Path(directory) / "gameplay-api-parity"
        shutil.copytree(source, examples)

        blueprint = examples / "blueprint-only"
        run_editor(editor, blueprint, "--compile-blueprints")
        run_editor(editor, blueprint, "--build-psx")
        assert (blueprint / ".epok/build/epok.ps-exe").is_file()

        lua = examples / "lua-only"
        manifest = next(lua.glob("*.epokproject"))
        for mode in LUA_MODES:
            replace_lua_mode(manifest, mode)
            run_editor(editor, lua, "--build-psx")
            executable = lua / ".epok/build/epok.ps-exe"
            assert executable.is_file() and executable.read_bytes().startswith(b"PS-X EXE")
        print("Gameplay parity examples cooked in Blueprint and all Lua modes.")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--editor", help="Path to a previously built debug editor")
    parser.add_argument(
        "--skip-example-builds",
        action="store_true",
        help="Run host checks only; leaves the target example-build gate unverified",
    )
    args = parser.parse_args()
    run(PYTHON, "tools/gameplay_api_parity.py")
    run(PYTHON, "tests/runtime/verify_spatial.py", "gameplay_api")
    run(
        "cargo",
        "test",
        "--locked",
        "gameplay_api_parity",
        "--",
        "--nocapture",
    )

    coverage_path = ROOT / "knowledge/initiatives/gameplay-api-parity/coverage.json"
    coverage = json.loads(coverage_path.read_text(encoding="utf-8"))
    public = [
        row
        for row in coverage["capabilities"]
        if row["disposition"] == "public_gameplay"
    ]
    assert public, "coverage contains no public gameplay operations"
    for row in public:
        assert row["status"] == "implemented", row["candidate_key"]
        assert all(
            value["state"] == "implemented" for value in row["support"].values()
        ), row["candidate_key"]

    examples = ROOT / "examples/gameplay-api-parity"
    required = [
        examples / "lua-only/assets/scripts/GameplayDemo.lua",
        examples / "blueprint-only/assets/blueprints/BP_GameplayParity.epokbp",
    ]
    assert all(path.is_file() for path in required), "authoring examples are missing"
    if not args.skip_example_builds:
        build_examples(editor_path(args.editor))
    print(f"Gameplay parity host acceptance passed for {len(public)} public rows.")


if __name__ == "__main__":
    main()
