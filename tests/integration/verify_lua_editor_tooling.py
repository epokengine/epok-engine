#!/usr/bin/env python3
"""Proves the generated Lua definitions are enforced in the editor.

A Lua author is meant to see a wrong assignment, a wrong argument and an
undeclared member flagged in VS Code before the Epok compiler is ever run. That
claim only holds if two things are true at once, and this script measures both
on ONE real project created through the production CLI:

  (a) the definitions and settings the editor generates (`.epok/lua/epok.d.lua`
      and `.luarc.json`) report NOTHING on a correct class, so an author is not
      trained to ignore them; and
  (b) a deliberately broken copy of that same class reports EXACTLY the three
      errors, on the right lines, from the Lua Language Server -- and the Epok
      compiler rejects the same three constructs with its own diagnostics.

The project is built once with `--build-psx`, which is what refreshes the script
catalog and therefore rewrites the definitions from the current C++, Blueprint
and Lua declarations.

Requires `lua-language-server` on PATH; without it the script skips.

    cargo build --locked --release
    python3 tests/integration/verify_lua_editor_tooling.py
    python3 tests/integration/verify_lua_editor_tooling.py --keep
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EDITOR = ROOT / "target/release/epok-editor"
REPORT = ROOT / "artifacts/lua_editor_tooling/report.json"

# The class under test. The `---@class` line is what binds the local to the
# generated `---@class Cube` declaration; the declaration form always writes it,
# so nothing extra is needed to give the language server a type for `self`.
CLEAN = """\
---@class Cube : epok.Actor3D
local Cube = epok.Actor3D:extend()

Cube.speed = 90.0

---@param amount Fixed
---@return Fixed
function Cube:take_damage(amount)
    self.speed = self.speed - amount
    return self.speed
end

function Cube:tick(delta_seconds)
    self.speed = self.speed + delta_seconds
    self:take_damage(1.0)
end

return Cube
"""
CORRECT_BODY = """\
    self.speed = self.speed + delta_seconds
    self:take_damage(1.0)"""
# A Bool written to a Fixed field, a string passed to a Fixed parameter, and a
# member no declaration has. One construct per line, so the reported line
# numbers are themselves part of the proof.
BROKEN_BODY = """\
    self.speed = true
    self:take_damage("x")
    self.speed = self.speedd"""

RESULTS = []


def record(passed, line):
    RESULTS.append({"passed": bool(passed), "line": line})
    print(f"{'PASS' if passed else 'FAIL'}: {line}", flush=True)
    return bool(passed)


def skip(reason):
    print(f"SKIP: {reason}", flush=True)
    return 0


def editor(project, *arguments):
    """One production CLI invocation; returns (exit code, stdout+stderr)."""
    done = subprocess.run(
        [str(EDITOR), "--project", str(project), *arguments],
        capture_output=True,
        text=True,
        cwd=str(project),
    )
    return done.returncode, done.stdout + done.stderr


def check(project, out):
    """Run the language server over a project; returns its diagnostics.

    `--checklevel=Error` is deliberate: the generated `.luarc.json` raises the
    three checks to `Error!`, so an error level run is exactly what an author
    sees flagged red in the editor.
    """
    if out.exists():
        out.unlink()
    subprocess.run(
        [
            "lua-language-server",
            "--check",
            str(project),
            "--checklevel=Error",
            "--check_format=json",
            f"--check_out_path={out}",
        ],
        capture_output=True,
        text=True,
    )
    if not out.exists():
        return None
    reported = json.loads(out.read_text() or "[]")
    if not reported:
        return []
    found = []
    for uri, items in reported.items():
        for item in items:
            found.append(
                {
                    "file": uri.rsplit("/", 1)[-1],
                    "code": item.get("code"),
                    # The language server counts lines from 0; every human-facing
                    # number in this report counts from 1, as the editor does.
                    "line": item["range"]["start"]["line"] + 1,
                    "message": item["message"].splitlines()[0],
                }
            )
    return sorted(found, key=lambda d: d["line"])


def epok_diagnostics(output, script):
    """The compiler's own diagnostics for one script, as (line, message)."""
    found = []
    for piece in output.replace("\\n", "\n").splitlines():
        marker = f"{script}:"
        if marker not in piece:
            continue
        tail = piece.split(marker, 1)[1]
        parts = tail.split(":", 2)
        if len(parts) == 3 and parts[0].isdigit():
            found.append((int(parts[0]), parts[2].strip().rstrip('"')))
    return sorted(found)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--keep", action="store_true", help="retain the projects")
    args = parser.parse_args()

    if not EDITOR.exists():
        print(f"Missing {EDITOR}; run: cargo build --locked --release", file=sys.stderr)
        return 1
    if shutil.which("lua-language-server") is None:
        return skip(
            "lua-language-server is not on PATH; install it (brew install "
            "lua-language-server) to check the generated definitions"
        )

    workspace = Path(tempfile.mkdtemp(prefix="epok-lua-tooling-"))
    try:
        return run(workspace)
    finally:
        if args.keep:
            print(f"Kept {workspace}")
        else:
            shutil.rmtree(workspace, ignore_errors=True)


def run(workspace):
    clean = workspace / "Clean"
    done = subprocess.run(
        [str(EDITOR), "--create-project", str(clean), "--name", "LuaTooling"],
        capture_output=True,
        text=True,
    )
    if done.returncode != 0:
        record(False, f"project creation failed: {done.stdout}{done.stderr}")
        return finish()
    scripts = clean / "assets/scripts"
    scripts.mkdir(parents=True, exist_ok=True)
    (scripts / "Cube.lua").write_text(CLEAN)

    # The catalog refresh that rewrites the definitions from the registry.
    code, output = editor(clean, "--build-psx")
    if not record(code == 0, "the correct class builds"):
        print(output, file=sys.stderr)
        return finish()

    stub = clean / ".epok/lua/epok.d.lua"
    config = clean / ".luarc.json"
    record(stub.is_file(), "a catalog refresh writes .epok/lua/epok.d.lua")
    record(config.is_file(), "a catalog refresh writes .luarc.json")
    if not (stub.is_file() and config.is_file()):
        return finish()

    # The class the author wrote is in the definitions, annotated function and
    # shorthand property alike: they come through the registry, so the editor,
    # the language server and the compiler read one declaration.
    text = stub.read_text()
    record("---@class Cube : epok.Actor3D" in text, "the Lua class is declared")
    record(
        "---@field speed Fixed Editable in the Inspector." in text,
        "a property declared by a literal renders with that literal's type",
    )
    record(
        "---@field super epok.Actor3D" in text,
        "the qualified parent receiver is declared",
    )
    record(
        "function _epok__Object:extend() end" in text,
        "the declaration entry point is declared on the hierarchy root",
    )
    record(
        "epok.Actor3D = _epok__Actor3D" in text,
        "a parent class is also bound as a value, so `epok.Actor3D:extend()` resolves",
    )
    record(
        "---@param amount Fixed\n---@return Fixed\nfunction _Cube:take_damage(amount) end"
        in text,
        "an annotated function renders with its signature",
    )

    severity = json.loads(config.read_text()).get("diagnostics.severity", {})
    record(
        all(
            severity.get(rule) == "Error!"
            for rule in ("assign-type-mismatch", "param-type-mismatch", "undefined-field")
        ),
        "the generated settings raise the three checks to errors",
    )

    out = workspace / "check.json"
    reported = check(clean, out)
    if reported is None:
        record(False, "the language server produced no report")
        return finish()
    record(reported == [], f"(a) the correct class reports nothing: {reported}")

    # (b) The same project, three constructs broken, one per line.
    broken = workspace / "Broken"
    shutil.copytree(clean, broken)
    source = broken / "assets/scripts/Cube.lua"
    body = source.read_text()
    assert CORRECT_BODY in body
    source.write_text(body.replace(CORRECT_BODY, BROKEN_BODY))
    first = CLEAN.splitlines().index(CORRECT_BODY.splitlines()[0]) + 1

    reported = check(broken, out) or []
    expected = [
        (first + 0, "assign-type-mismatch"),
        (first + 1, "param-type-mismatch"),
        (first + 2, "undefined-field"),
    ]
    actual = [(d["line"], d["code"]) for d in reported]
    if not record(actual == expected, f"(b) the language server reports {expected}"):
        print(json.dumps(reported, indent=2), file=sys.stderr)

    # The compiler must reject the very same three constructs. Same project,
    # same lines, its own diagnostics: the two tools agree by construction.
    code, output = editor(broken, "--build-psx")
    record(code != 0, "(b) the broken class fails to build")
    compiler = epok_diagnostics(output, "Cube.lua")
    record(
        [line for line, _ in compiler] == [line for line, _ in expected],
        f"(b) the compiler rejects the same three lines: {compiler}",
    )
    return finish(compiler, reported)


def finish(compiler=None, language_server=None):
    passed = sum(1 for r in RESULTS if r["passed"])
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "checks": RESULTS,
                "passed": passed,
                "total": len(RESULTS),
                "language_server": language_server,
                "compiler": compiler,
            },
            indent=2,
        )
        + "\n"
    )
    print(f"\n{passed}/{len(RESULTS)} checks passed. Report: {REPORT}")
    return 0 if passed == len(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main())
