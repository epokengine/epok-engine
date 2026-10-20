"""Check installed production language servers without a desktop or browser.

Run the native editor acceptance fixture first (or retain its generated API
workspaces), then run this script. No GitHub sign-in or Copilot request is made.
"""
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

PROJECT = ROOT / "artifacts/code-editor/native/QA Project"
LOCAL = Path(os.environ.get("LOCALAPPDATA", os.environ.get("XDG_DATA_HOME", ""))) / "Epok"
CONFIG = documents.loads((LOCAL / "CodeEditor.epokprefs").read_text(encoding="utf-8"))


from code_language_client import Client, file_uri


report = {}
lua = Client(CONFIG["lua_ls"], PROJECT, {"Lua": {"runtime": {"version": "Lua 5.2"},
             "workspace": {"library": [str(PROJECT / ".epok/lua")], "checkThirdParty": False}}})
try:
    report["lua"] = lua.complete(PROJECT / "assets/scripts/CodeProbe.lua", "epok.", "Actor3D")
    print("PASS actual LuaLS completion includes the generated engine API")
finally:
    lua.close()

pocket_root = PROJECT / ".epok/code/pocket"
pocket_builtins = {name: "disable" for name in ["basic", "bit", "bit32", "coroutine",
    "debug", "ffi", "io", "jit", "math", "os", "package", "string", "table", "table.clear", "table.new", "utf8"]}
pocket = Client(CONFIG["lua_ls"], pocket_root, {"Lua": {"runtime": {"version": "Lua 5.4", "builtin": pocket_builtins},
                "workspace": {"library": [str(pocket_root / "api")], "checkThirdParty": False}},
                "files": {"associations": {"*.plua": "lua"}}})
try:
    source = next(pocket_root.glob("*/main.plua"))
    report["pocket"] = pocket.complete(source, source.read_text(encoding="utf-8") + "\nscreen.", "pixel")
    assert not any("Actor3D" in item for item in report["pocket"])
    print("PASS actual LuaLS completion includes compiler-generated PocketLua APIs")
finally:
    pocket.close()

cpp = Client(CONFIG["clangd"], PROJECT, {},
             [f"--compile-commands-dir={PROJECT / '.epok/code/cpp'}", "--header-insertion=never"])
try:
    report["cpp"] = cpp.complete(PROJECT / "assets/scripts/CppProbe.hpp",
                                '#include "epok.hpp"\nvoid probe() { epok::Act', "Actor3D")
    print("PASS actual clangd completion includes the C++ engine API")
finally:
    cpp.close()

copilot = Client(CONFIG["copilot"], PROJECT, {"telemetry": {"telemetryLevel": "off"}}, ["--stdio"])
try:
    assert isinstance(copilot.initialize.get("capabilities"), dict)
    report["copilot"] = {"initialized": True, "serverInfo": copilot.initialize.get("serverInfo")}
    print("PASS official Copilot server initializes without requesting sign-in or a completion")
finally:
    copilot.close()

(ROOT / "artifacts/code-editor/language-services.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
