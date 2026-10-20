"""Exercise generated Epok definitions with the installed, unmodified LuaLS.

Provide a project whose current editor build has generated .epok/lua/epok.d.lua.
All probes are in-memory LSP documents; no gameplay source is rewritten.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys

from code_language_client import Client, file_uri

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import epok_documents as documents

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--project", type=Path, required=True)
parser.add_argument("--source", action="append", help="Project-relative Lua source (repeatable); defaults to all gameplay scripts")
parser.add_argument("--report", type=Path, default=ROOT / "artifacts/code-editor/lua-api-diagnostics.json")
args = parser.parse_args()
project = args.project.resolve()
user_data = Path(os.environ.get("LOCALAPPDATA", os.environ.get("XDG_DATA_HOME", str(Path.home() / ".local/share"))))
config = documents.loads((user_data / "Epok/CodeEditor.epokprefs").read_text(encoding="utf-8"))
client = Client(config["lua_ls"], project, {"Lua": {
    "runtime": {"version": "Lua 5.2"},
    "workspace": {"library": [str(project / ".epok/lua")], "checkThirdParty": False, "ignoreDir": [".epok/code"]},
    "diagnostics": {"globals": ["epok"]}, "telemetry": {"enable": False}}})
report = {"checks": []}
try:
    sources = [project / source for source in args.source] if args.source else sorted((project / "assets/scripts").rglob("*.lua"))
    assert sources, "No gameplay Lua scripts found"
    source_hashes = {source: hashlib.sha256(source.read_bytes()).hexdigest() for source in sources}
    report["sourceDiagnostics"] = {}
    for index, source in enumerate(sources):
        text = source.read_text(encoding="utf-8")
        # Force an observed diagnostic transition even for a completely clean
        # file, for which LuaLS need not publish an initial empty notification.
        seeded = client.diagnostics(source, text + f"\n---@class __EpokSourceControl{index} : __EpokMissingBase\n")
        assert any(d.get("code") == "undefined-doc-class" for d in seeded), seeded
        diagnostics = client.diagnostics(source, text)
        report["sourceDiagnostics"][str(source.relative_to(project))] = diagnostics
        assert all(d.get("code") == "unused-local" for d in diagnostics), {"source": str(source), "diagnostics": diagnostics}
        print(f"PASS {source.name}: no API, type or duplicate-method diagnostics ({len(diagnostics)} unused-local hints)", flush=True)
    report["checks"].append("all real gameplay scripts have no API, type or duplicate-method diagnostics")

    probe = "\n".join([
        "---@class LuaApiDiagnosticsProbe : epok.ActorComponent",
        "local Probe = epok.ActorComponent:extend()",
        "Probe.camera = epok.ComponentRef(epok.SceneComponent3D)",
        "Probe.visual = epok.ComponentRef(epok.Mesh3DComponent)",
        "function Probe:check()",
        "    self.camera:set_local_position(1, 2, 3)",
        "    self.visual:play_clip(0, true)",
        "    epok.collision.move(epok.owner(), epok.math.vector3(0, 0, 0), 4294967295)",
        '    self.camera:set_local_position("wrong", 2, 3)',
        '    self.visual:play_clip(0, "wrong")',
        "    self.camera:clip_frames(0)",
        "    self.visual:does_not_exist()",
        "    epok.collision.move(12, epok.math.vector3(0, 0, 0), 4294967295)",
        "end",
        "return Probe",
    ])
    probe_path = project / "assets/scripts/__EpokLuaDiagnosticsProbe.lua"
    diagnostics = client.diagnostics(probe_path, probe)
    report["negativeDiagnostics"] = diagnostics
    report["probeHovers"] = [client.request("textDocument/hover", {
        "textDocument": {"uri": file_uri(probe_path)},
        "position": {"line": line, "character": character}}) for line, character in [(2, 8), (2, 24), (5, 10), (5, 22)]]
    actual = {(d["range"]["start"]["line"], d.get("code")) for d in diagnostics}
    expected = {(8, "param-type-mismatch"), (9, "param-type-mismatch"),
                (10, "undefined-field"), (11, "undefined-field"), (12, "param-type-mismatch")}
    assert actual == expected, {"expected": sorted(expected), "actual": sorted(actual), "diagnostics": diagnostics}
    report["checks"].append("wrong argument types, wrong reference class and nonexistent APIs remain errors")
    print("PASS invalid parameters, reference class and methods are still rejected")

    cast_probe = "\n".join([
        "---@class LuaCastDiagnosticsProbe : epok.ActorComponent",
        "local Probe = epok.ActorComponent:extend()",
        "function Probe:check()",
        '    local actor3d = epok.cast(epok.owner(), "epok::Actor3D")',
        '    local actor2d = epok.cast(epok.owner(), "epok::Actor2D")',
        '    local mesh = epok.cast(epok.owner(), "epok::Mesh3DComponent")',
        "    actor3d:set_active(false)",
        "    actor2d:set_active(false)",
        "    mesh:play_clip(0, true)",
        '    actor3d:set_active("wrong")',
        "    actor3d:play_clip(0, true)",
        "end",
        "return Probe",
    ])
    cast_path = project / "assets/scripts/__EpokLuaCastDiagnosticsProbe.lua"
    diagnostics = client.diagnostics(cast_path, cast_probe)
    report["castDiagnostics"] = diagnostics
    actual = {(d["range"]["start"]["line"], d.get("code")) for d in diagnostics}
    expected = {(9, "param-type-mismatch"), (10, "undefined-field")}
    assert actual == expected, {"expected": sorted(expected), "actual": sorted(actual), "diagnostics": diagnostics}
    report["castHovers"] = []
    for line, reference_type in [(3, "epok.Actor3D.Ref"), (4, "epok.Actor2D.Ref"), (5, "epok.Mesh3DComponent.Ref")]:
        hover = client.request("textDocument/hover", {"textDocument": {"uri": file_uri(cast_path)},
                              "position": {"line": line, "character": 12}})
        assert reference_type in json.dumps(hover), hover
        report["castHovers"].append(hover)
    report["checks"].append("cast preserves the exact 2D/3D actor or component reference and rejects invalid calls")
    print("PASS cast narrows references and preserves parameter and method checks")
    cast_labels = client.complete(project / "assets/scripts/__EpokLuaCastCompletionProbe.lua",
        'local actor = epok.cast(epok.owner(), "epok::Actor3D")\nactor:', "set_active")
    assert not any("play_clip" in label for label in cast_labels), cast_labels
    report["castCompletions"] = cast_labels
    report["checks"].append("cast completion includes inherited actor methods and excludes component-only APIs")
    print("PASS cast completion includes inherited actor methods")

    path = project / "assets/scripts/__EpokLuaCompletionProbe.lua"
    labels = client.complete(path, "---@type epok.SceneComponent3D.Ref\nlocal camera\ncamera:", "set_local_position")
    assert not any("clip_frames" in label for label in labels), labels
    report["cameraCompletions"] = labels
    report["checks"].append("typed camera completion includes inherited methods and excludes mesh-only APIs")
    print("PASS camera reference completion has the correct reflected methods")
    hover = client.request("textDocument/hover", {"textDocument": {"uri": file_uri(probe_path)},
                           "position": {"line": 5, "character": 22}})
    report["cameraHover"] = hover
    hover_text = json.dumps(hover)
    assert "set_local_position" in hover_text and all(f"{name}: number" in hover_text for name in ["x", "y", "z"]), hover
    report["checks"].append("method hover contains the reflected numeric parameter signature")
    print("PASS hover exposes the reflected parameter signature")
    definitions = project / ".epok/lua/epok.d.lua"
    definition_text = definitions.read_text(encoding="utf-8")
    # A new, clean meta document need not publish an empty push notification.
    # First provoke an in-memory annotation error, then verify it clears when
    # the unchanged generated text is restored (nothing is written to disk).
    seeded = client.diagnostics(definitions, definition_text + "\n---@class __EpokDiagnosticControl : __EpokMissingBase\n")
    assert any(d.get("code") == "undefined-doc-class" for d in seeded), seeded
    diagnostics = client.diagnostics(definitions, definition_text)
    report["definitionDiagnostics"] = diagnostics
    assert not diagnostics, diagnostics
    report["checks"].append("generated definitions have no LuaLS annotation or type diagnostics")
    print("PASS generated definitions have no annotation or type diagnostics")
    assert all(hashlib.sha256(source.read_bytes()).hexdigest() == digest for source, digest in source_hashes.items())
    report["sourceHashes"] = {str(source.relative_to(project)): digest for source, digest in source_hashes.items()}
    report["checks"].append("all gameplay sources remain byte-identical")
finally:
    client.close()
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2), encoding="utf-8")
